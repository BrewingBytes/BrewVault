//! Global application state and the [`AppState`] struct that owns it.
//!
//! [`APP_STATE`] is the single source of truth for the running application.
//! All UI components read from and write to it via Dioxus's [`GlobalSignal`].
//! Storage operations are driven through the [`rusqlite::Connection`] kept
//! inside [`AppState`], so every mutation goes through one place.
//!
//! # Lock state machine
//!
//! ```text
//! ┌──────────┐  setup_with_password / setup_no_password  ┌──────────┐
//! │ FirstRun │ ─────────────────────────────────────────▶│ Unlocked │
//! └──────────┘                                           └──────────┘
//!                                                             │  ▲
//!                                          lock()             │  │  unlock(pw)
//!                                                             ▼  │
//!                                                        ┌────────┐
//!                                                        │ Locked │
//!                                                        └────────┘
//! ```

use std::sync::{Arc, Mutex};
use std::time::Duration;

use dioxus::prelude::*;
use rusqlite::Connection;

use crate::models::{error::TotpError, totp::TotpEntry};
use crate::storage;

pub static APP_STATE: GlobalSignal<AppState> = GlobalSignal::new(AppState::new);

/// Which screen the app should render.
#[derive(Clone, Debug, PartialEq)]
pub enum LockState {
    /// No database exists — show the first-run setup screen.
    FirstRun,
    /// Database exists with a password — show the lock/unlock screen.
    Locked,
    /// Vault is open and entries are accessible.
    Unlocked,
}

/// Central application state, including the open vault database connection.
///
/// The [`Connection`] is wrapped in `Arc<Mutex<_>>` so that [`AppState`] can
/// satisfy the `Clone` bound required by Dioxus signals while still owning the
/// live database handle. Cloning `AppState` clones the `Arc`, so all clones
/// share the same underlying connection.
#[derive(Clone)]
pub struct AppState {
    entries: Vec<TotpEntry>,
    db: Option<Arc<Mutex<Connection>>>,
    /// Which screen to render.
    pub lock_state: LockState,
    /// Whether the vault is currently password-protected.
    /// Stays true/false across lock/unlock cycles.
    pub has_password: bool,
    /// Configured auto-lock timeout. `None` = disabled.
    pub auto_lock_timeout: Option<Duration>,
}

impl AppState {
    /// Detects vault state from disk and sets up the initial `AppState`.
    ///
    /// - `FirstRun`: no DB yet, everything blank.
    /// - `NoPassword`: opens DB immediately → `Unlocked`.
    /// - `PasswordProtected`: leaves DB closed → `Locked`.
    pub fn new() -> Self {
        match storage::detect_vault_state() {
            Ok(storage::VaultState::FirstRun) => Self {
                entries: vec![],
                db: None,
                lock_state: LockState::FirstRun,
                has_password: false,
                auto_lock_timeout: None,
            },
            Ok(storage::VaultState::NoPassword) => {
                let conn =
                    storage::open_db(storage::NO_PASSWORD_KEY).expect("failed to open vault");
                storage::init_schema(&conn).expect("failed to init schema");
                storage::migrate_sort_order(&conn).expect("failed to migrate");
                let entries = storage::load_entries(&conn).expect("failed to load entries");
                let secs = storage::get_auto_lock_secs(&conn).unwrap_or(0);
                Self {
                    entries,
                    db: Some(Arc::new(Mutex::new(conn))),
                    lock_state: LockState::Unlocked,
                    has_password: false,
                    auto_lock_timeout: secs_to_duration(secs),
                }
            }
            Ok(storage::VaultState::PasswordProtected) => Self {
                entries: vec![],
                db: None,
                lock_state: LockState::Locked,
                has_password: true,
                auto_lock_timeout: None, // loaded after unlock
            },
            Err(e) => panic!("failed to detect vault state: {e}"),
        }
    }

    // -----------------------------------------------------------------------
    // Auth lifecycle
    // -----------------------------------------------------------------------

    /// First-run: set a master password and transition to `Unlocked`.
    ///
    /// Validates password length (≥ 8) and rejects the sentinel key.
    pub fn setup_with_password(&mut self, password: &str) -> Result<(), TotpError> {
        if password.len() < 8 {
            return Err(TotpError::PasswordTooShort);
        }
        if password == storage::NO_PASSWORD_KEY {
            return Err(TotpError::ReservedPassword);
        }

        let conn = storage::open_db(password)?;
        storage::init_schema(&conn)?;
        let hash = storage::argon2_hash(password)?;
        storage::set_meta(&conn, storage::META_PASSWORD_SET, "true")?;
        storage::set_meta(&conn, storage::META_PASSWORD_HASH, &hash)?;

        self.entries = vec![];
        self.db = Some(Arc::new(Mutex::new(conn)));
        self.lock_state = LockState::Unlocked;
        self.has_password = true;
        Ok(())
    }

    /// First-run: skip password and transition to `Unlocked`.
    pub fn setup_no_password(&mut self) -> Result<(), TotpError> {
        let conn = storage::open_db(storage::NO_PASSWORD_KEY)?;
        storage::init_schema(&conn)?;
        storage::set_meta(&conn, storage::META_PASSWORD_SET, "false")?;

        self.entries = vec![];
        self.db = Some(Arc::new(Mutex::new(conn)));
        self.lock_state = LockState::Unlocked;
        self.has_password = false;
        Ok(())
    }

    /// Unlock the vault with `password`. Opens the database, loads entries.
    ///
    /// Returns [`TotpError::WrongPassword`] if the key does not match.
    pub fn unlock(&mut self, password: &str) -> Result<(), TotpError> {
        let conn = storage::open_db(password)?;

        // Test the key — SQLCipher only fails on the first real schema read.
        let key_ok = conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| {
                r.get::<_, i64>(0)
            })
            .is_ok();
        if !key_ok {
            return Err(TotpError::WrongPassword);
        }

        storage::migrate_sort_order(&conn)?;
        let entries = storage::load_entries(&conn)?;
        let secs = storage::get_auto_lock_secs(&conn).unwrap_or(0);

        self.entries = entries;
        self.db = Some(Arc::new(Mutex::new(conn)));
        self.lock_state = LockState::Unlocked;
        self.auto_lock_timeout = secs_to_duration(secs);
        Ok(())
    }

    /// Lock the vault: clear entries and close the database connection.
    pub fn lock(&mut self) {
        self.entries.clear();
        self.db = None;
        self.lock_state = LockState::Locked;
    }

    /// Change the master password (or remove it if `new_pw` is `None`).
    ///
    /// Steps:
    /// 1. If `has_password`: verify `current_pw` against stored Argon2 hash.
    /// 2. Validate `new_pw` (length, not sentinel).
    /// 3. PRAGMA rekey the live connection.
    /// 4. Update meta table.
    pub fn change_password(
        &mut self,
        current_pw: &str,
        new_pw: Option<&str>,
    ) -> Result<(), TotpError> {
        // Validate new password first (before touching the DB).
        let new_key = if let Some(pw) = new_pw {
            if pw.len() < 8 {
                return Err(TotpError::PasswordTooShort);
            }
            if pw == storage::NO_PASSWORD_KEY {
                return Err(TotpError::ReservedPassword);
            }
            pw.to_string()
        } else {
            storage::NO_PASSWORD_KEY.to_string()
        };

        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");

        // Verify current password if one is set.
        if self.has_password {
            let hash = storage::get_password_hash(&conn)?.ok_or(TotpError::WrongPassword)?;
            if !storage::argon2_verify(current_pw, &hash) {
                return Err(TotpError::WrongPassword);
            }
        }

        // Rekey (modifies the file in-place).
        storage::rekey(&conn, &new_key)?;

        // Update meta.
        let is_pw = new_pw.is_some();
        storage::set_meta(
            &conn,
            storage::META_PASSWORD_SET,
            if is_pw { "true" } else { "false" },
        )?;
        if let Some(pw) = new_pw {
            let hash = storage::argon2_hash(pw)?;
            storage::set_meta(&conn, storage::META_PASSWORD_HASH, &hash)?;
        } else {
            storage::delete_meta(&conn, storage::META_PASSWORD_HASH)?;
        }

        self.has_password = is_pw;
        Ok(())
    }

    /// Update the auto-lock timeout and persist it to the meta table.
    ///
    /// Pass `0` to disable auto-lock.
    pub fn set_auto_lock(&mut self, secs: u64) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::set_meta(&conn, storage::META_AUTO_LOCK_SECS, &secs.to_string())?;
        self.auto_lock_timeout = secs_to_duration(secs);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Entry accessors
    // -----------------------------------------------------------------------

    /// Returns a slice of all vault entries currently held in memory,
    /// ordered by `sort_order DESC`.
    pub fn get_entries(&self) -> &[TotpEntry] {
        &self.entries
    }

    /// Assigns `sort_order = max + 1` and persists `entry` to the vault
    /// database, then appends it to the in-memory list.
    pub fn add_entry(&mut self, mut entry: TotpEntry) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        let max_so = storage::max_sort_order(&conn)?;
        entry.sort_order = max_so + 1;
        storage::insert_entry(&conn, &entry)?;
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));
        Ok(())
    }

    /// Removes the entry with `id` from the vault database then removes it
    /// from the in-memory list.
    pub fn remove_entry(&mut self, id: &str) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::delete_entry(&conn, id)?;
        self.entries.retain(|e| e.id != id);
        Ok(())
    }

    /// Deletes all entries from the vault database and clears the in-memory list.
    pub fn remove_all_entries(&mut self) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::delete_all_entries(&conn)?;
        self.entries.clear();
        Ok(())
    }

    /// Renames the issuer and account fields of the entry with `id`.
    pub fn rename_entry(&mut self, id: &str, issuer: &str, account: &str) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        let n = storage::rename_entry_db(&conn, id, issuer, account)?;
        if n == 0 {
            return Err(TotpError::StorageError(
                rusqlite::Error::QueryReturnedNoRows,
            ));
        }
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.issuer = issuer.to_string();
            entry.account = account.to_string();
        }
        Ok(())
    }

    /// Moves the entry with `id` to a different group.
    pub fn update_entry_group(&mut self, id: &str, group: Option<&str>) -> Result<(), TotpError> {
        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        let n = storage::update_group_db(&conn, id, group)?;
        if n == 0 {
            return Err(TotpError::StorageError(
                rusqlite::Error::QueryReturnedNoRows,
            ));
        }
        let (new_group, new_sort_order) = storage::get_entry_group_and_sort_order(&conn, id)?;
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.group = new_group;
            entry.sort_order = new_sort_order;
        }
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));
        Ok(())
    }

    /// Moves the entry with `id` one position up within its group.
    pub fn move_entry_up(&mut self, id: &str) -> Result<(), TotpError> {
        let pos = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| TotpError::StorageError(rusqlite::Error::QueryReturnedNoRows))?;

        let group = self.entries[pos].group.clone();

        let group_positions: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.group == group)
            .map(|(i, _)| i)
            .collect();

        let group_idx = group_positions
            .iter()
            .position(|&p| p == pos)
            .ok_or_else(|| TotpError::StorageError(rusqlite::Error::QueryReturnedNoRows))?;

        if group_idx == 0 {
            return Err(TotpError::StorageError(
                rusqlite::Error::QueryReturnedNoRows,
            ));
        }

        let above_pos = group_positions[group_idx - 1];
        let id_above = self.entries[above_pos].id.clone();

        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::swap_sort_order_db(&conn, id, &id_above)?;

        let so_a = self.entries[pos].sort_order;
        let so_b = self.entries[above_pos].sort_order;
        self.entries[pos].sort_order = so_b;
        self.entries[above_pos].sort_order = so_a;
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));

        Ok(())
    }

    /// Moves the entry with `id` one position down within its group.
    pub fn move_entry_down(&mut self, id: &str) -> Result<(), TotpError> {
        let pos = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| TotpError::StorageError(rusqlite::Error::QueryReturnedNoRows))?;

        let group = self.entries[pos].group.clone();

        let group_positions: Vec<usize> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.group == group)
            .map(|(i, _)| i)
            .collect();

        let group_idx = group_positions
            .iter()
            .position(|&p| p == pos)
            .ok_or_else(|| TotpError::StorageError(rusqlite::Error::QueryReturnedNoRows))?;

        if group_idx == group_positions.len() - 1 {
            return Err(TotpError::StorageError(
                rusqlite::Error::QueryReturnedNoRows,
            ));
        }

        let below_pos = group_positions[group_idx + 1];
        let id_below = self.entries[below_pos].id.clone();

        let arc = self.db_or_err()?;
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::swap_sort_order_db(&conn, id, &id_below)?;

        let so_a = self.entries[pos].sort_order;
        let so_b = self.entries[below_pos].sort_order;
        self.entries[pos].sort_order = so_b;
        self.entries[below_pos].sort_order = so_a;
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Returns a clone of the DB Arc, or `Err(VaultLocked)` if the vault is locked.
    fn db_or_err(&self) -> Result<Arc<Mutex<Connection>>, TotpError> {
        self.db.clone().ok_or(TotpError::VaultLocked)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn secs_to_duration(secs: u64) -> Option<Duration> {
    if secs > 0 {
        Some(Duration::from_secs(secs))
    } else {
        None
    }
}
