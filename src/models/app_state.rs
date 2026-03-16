use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{pages::AppPage, totp::TotpEntry};
use crate::storage;

pub static APP_STATE: GlobalSignal<AppState> = GlobalSignal::new(AppState::new);

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    entries: Vec<TotpEntry>,
    show_page: AppPage,
}

impl AppState {
    pub fn new() -> Self {
        let entries = storage::with_db(storage::load_entries)
            .unwrap_or_default();
        Self {
            entries,
            show_page: AppPage::default(),
        }
    }

    pub fn add_entry(&mut self, entry: TotpEntry) {
        let _ = storage::with_db(|conn| storage::save_entry(conn, &entry));
        self.entries.push(entry);
    }

    pub fn remove_entry(&mut self, id: &str) {
        let _ = storage::with_db(|conn| storage::delete_entry(conn, id));
        self.entries.retain(|e| e.id != id);
    }

    pub fn set_page(&mut self, page: AppPage) {
        self.show_page = page;
    }

    pub fn get_entries(&self) -> &[TotpEntry] {
        &self.entries
    }

    pub fn get_current_page(&self) -> &AppPage {
        &self.show_page
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
