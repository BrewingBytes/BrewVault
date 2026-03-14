use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::models::{pages::AppPage, totp::TotpEntry};

pub static APP_STATE: GlobalSignal<AppState> = GlobalSignal::new(AppState::new);

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    entries: Vec<TotpEntry>,
    show_page: AppPage,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            show_page: AppPage::default(),
        }
    }

    pub fn add_entry(&mut self, entry: TotpEntry) {
        self.entries.push(entry);
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
