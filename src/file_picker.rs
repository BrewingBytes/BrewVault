//! Async file picker wrappers using [`rfd::AsyncFileDialog`].
//!
//! These free async functions are safe to call from a Dioxus `spawn` context
//! because `rfd::AsyncFileDialog` handles the macOS main-thread requirement
//! internally.

use std::path::PathBuf;

/// Open a native "Save File" dialog.
///
/// Returns the chosen path, or `None` if the user cancelled.
pub async fn save_file(default_name: &str, extensions: &[&str]) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_file_name(default_name)
        .add_filter("BrewVault backup", extensions)
        .save_file()
        .await
        .map(|h| h.path().to_path_buf())
}

/// Open a native "Open File" dialog.
///
/// Returns the chosen path, or `None` if the user cancelled.
pub async fn open_file(extensions: &[&str]) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("BrewVault backup", extensions)
        .pick_file()
        .await
        .map(|h| h.path().to_path_buf())
}
