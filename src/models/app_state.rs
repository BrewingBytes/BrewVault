//! Global application state and the [`AppState`] struct that owns it.
//!
//! [`APP_STATE`] is the single source of truth for the running application.
//! All UI components read from and write to it via Dioxus's [`GlobalSignal`].
//! Storage operations are driven through the [`rusqlite::Connection`] kept
//! inside [`AppState`], so every mutation goes through one place.

use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::models::{error::TotpError, totp::TotpEntry};
use crate::storage;

pub static APP_STATE: GlobalSignal<AppState> = GlobalSignal::new(AppState::new);

/// Central application state, including the open vault database connection.
///
/// The [`Connection`] is wrapped in `Arc<Mutex<_>>` so that [`AppState`] can
/// satisfy the `Clone` bound required by Dioxus signals while still owning the
/// live database handle. Cloning `AppState` clones the `Arc`, so all clones
/// share the same underlying connection.
///
/// The `db` field is skipped during serialisation/deserialisation; on
/// deserialise it defaults to `None` (no live connection).
#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    entries: Vec<TotpEntry>,
    /// Open, encrypted vault connection. `None` only when the value was
    /// produced by deserialisation — in normal operation `new()` always
    /// provides a live connection or panics.
    #[serde(skip)]
    db: Option<Arc<Mutex<Connection>>>,
}

impl AppState {
    /// Fallible constructor: opens the vault database, initialises the schema,
    /// and loads all persisted entries. Propagates any storage error to the caller.
    pub fn try_new() -> Result<Self, TotpError> {
        let conn = storage::open_and_init()?;
        let entries = storage::load_entries(&conn)?;
        Ok(Self {
            entries,
            db: Some(Arc::new(Mutex::new(conn))),
        })
    }

    /// Infallible wrapper around [`try_new`] required by `GlobalSignal::new`.
    /// Panics with a clear message if the vault database cannot be opened.
    pub fn new() -> Self {
        Self::try_new().expect("failed to open vault database")
    }

    /// Assigns `sort_order = max + 1` and persists `entry` to the vault
    /// database, then appends it to the in-memory list (sorted by sort_order DESC).
    ///
    /// Returns `Err` if the database write fails; the in-memory list is **not**
    /// updated in that case, keeping it in sync with the on-disk state.
    pub fn add_entry(&mut self, mut entry: TotpEntry) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        let max_so = storage::max_sort_order(&conn)?;
        entry.sort_order = max_so + 1;
        storage::insert_entry(&conn, &entry)?;
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));
        Ok(())
    }

    /// Removes the entry with `id` from the vault database then removes it from
    /// the in-memory list.
    ///
    /// Returns `Err` if the database delete fails (including if the entry does
    /// not exist); the in-memory list is **not** updated in that case.
    pub fn remove_entry(&mut self, id: &str) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::delete_entry(&conn, id)?;
        self.entries.retain(|e| e.id != id);
        Ok(())
    }

    /// Renames the issuer and account fields of the entry with `id`.
    ///
    /// Returns `Err` if the entry does not exist or the database write fails.
    pub fn rename_entry(&mut self, id: &str, issuer: &str, account: &str) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
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

    /// Moves the entry with `id` to a different group, placing it at the top
    /// of that group (sort_order = max in new group + 1).
    ///
    /// Pass `None` to move the entry to the ungrouped section.
    /// Returns `Err` if the entry does not exist or the database write fails.
    pub fn update_entry_group(&mut self, id: &str, group: Option<&str>) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        let n = storage::update_group_db(&conn, id, group)?;
        if n == 0 {
            return Err(TotpError::StorageError(
                rusqlite::Error::QueryReturnedNoRows,
            ));
        }
        // Reload the updated sort_order from the DB (the SQL computed it for us)
        let updated = storage::load_entries(&conn)?;
        if let Some(fresh) = updated.iter().find(|e| e.id == id)
            && let Some(entry) = self.entries.iter_mut().find(|e| e.id == id)
        {
            entry.group = fresh.group.clone();
            entry.sort_order = fresh.sort_order;
        }
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));
        Ok(())
    }

    /// Moves the entry with `id` one position up within its group (higher sort_order
    /// = closer to the top of the displayed list).
    ///
    /// Returns `Err` if the entry is already first in its group, does not exist,
    /// or the database write fails.
    pub fn move_entry_up(&mut self, id: &str) -> Result<(), TotpError> {
        // Find this entry's index in the flat sorted vec
        let pos = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| TotpError::StorageError(rusqlite::Error::QueryReturnedNoRows))?;

        let group = self.entries[pos].group.clone();

        // Collect indices of entries in the same group, in sort_order DESC order
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

        // Swap with the entry above (lower index = higher sort_order = visually above)
        let above_pos = group_positions[group_idx - 1];
        let id_above = self.entries[above_pos].id.clone();

        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::swap_sort_order_db(&conn, id, &id_above)?;

        // Swap sort_orders in memory
        let so_a = self.entries[pos].sort_order;
        let so_b = self.entries[above_pos].sort_order;
        self.entries[pos].sort_order = so_b;
        self.entries[above_pos].sort_order = so_a;
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));

        Ok(())
    }

    /// Moves the entry with `id` one position down within its group (lower sort_order
    /// = closer to the bottom of the displayed list).
    ///
    /// Returns `Err` if the entry is already last in its group, does not exist,
    /// or the database write fails.
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

        // Swap with the entry below (higher index = lower sort_order = visually below)
        let below_pos = group_positions[group_idx + 1];
        let id_below = self.entries[below_pos].id.clone();

        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::swap_sort_order_db(&conn, id, &id_below)?;

        let so_a = self.entries[pos].sort_order;
        let so_b = self.entries[below_pos].sort_order;
        self.entries[pos].sort_order = so_b;
        self.entries[below_pos].sort_order = so_a;
        self.entries.sort_by(|a, b| b.sort_order.cmp(&a.sort_order));

        Ok(())
    }

    /// Returns a slice of all vault entries currently held in memory,
    /// ordered by `sort_order DESC`.
    pub fn get_entries(&self) -> &[TotpEntry] {
        &self.entries
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
