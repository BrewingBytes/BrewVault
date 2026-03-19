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

    /// Persists `entry` to the vault database then appends it to the in-memory list.
    ///
    /// Returns `Err` if the database write fails; the in-memory list is **not**
    /// updated in that case, keeping it in sync with the on-disk state.
    pub fn add_entry(&mut self, entry: TotpEntry) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::save_entry(&conn, &entry)?;
        self.entries.push(entry);
        Ok(())
    }

    /// Removes the entry with `id` from the vault database then removes it from
    /// the in-memory list.
    ///
    /// Returns `Err` if the database delete fails; the in-memory list is **not**
    /// updated in that case, keeping it in sync with the on-disk state.
    pub fn remove_entry(&mut self, id: &str) -> Result<(), TotpError> {
        let arc = self.db.as_ref().expect("DB not initialized");
        let conn = arc.lock().expect("DB mutex poisoned");
        storage::delete_entry(&conn, id)?;
        self.entries.retain(|e| e.id != id);
        Ok(())
    }

    /// Returns a slice of all vault entries currently held in memory.
    pub fn get_entries(&self) -> &[TotpEntry] {
        &self.entries
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
