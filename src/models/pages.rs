//! UI page identifiers used by [`crate::models::app_state::AppState`] to track
//! which screen is currently visible.

use serde::{Deserialize, Serialize};

/// The top-level pages of the BrewVault UI.
///
/// Stored on [`crate::models::app_state::AppState`] and read by the router /
/// top-level component to decide which screen to render. [`AppPage::Home`] is
/// the default — the vault entry list shown on launch.
#[derive(Serialize, Deserialize, Clone, Default)]
pub enum AppPage {
    /// The main vault screen listing all saved TOTP entries.
    #[default]
    Home,
    /// The form for adding a new TOTP entry to the vault.
    AddEntry,
    /// Application settings.
    Settings,
}
