//! Import modal — passphrase entry for decrypting a chosen `.brewvault` file.
//!
//! State machine:
//! ```text
//! IDLE ──[Import]──▶ IMPORTING ──[success]──▶ RESULT(n_new, n_skipped)
//!          ▲                         └──[error]──▶ IDLE (error shown, passphrase retained)
//! RESULT ──[Close]──▶ (modal closes, toast shown)
//! ```

use std::path::PathBuf;

use dioxus::prelude::*;

use crate::backup;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::toast::{TOAST, ToastData, next_toast_id};
use crate::models::app_state::APP_STATE;
use crate::models::error::BackupError;
use crate::models::error::TotpError;

#[derive(Clone, PartialEq)]
enum ImportState {
    Idle,
    Importing,
    Result { imported: usize, skipped: usize },
}

/// Import modal.
///
/// `path` is the file chosen before the modal opened.
/// `on_close` is called when the modal should be dismissed.
#[component]
pub fn ImportModal(path: PathBuf, on_close: EventHandler<()>) -> Element {
    let passphrase = use_signal(String::new);
    let mut error_msg = use_signal(String::new);
    let mut state = use_signal(|| ImportState::Idle);
    // Wrap path in a Signal so all do_import captures are Copy.
    let path_signal: Signal<PathBuf> = use_signal(|| path);

    let fname = path_signal
        .read()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("backup")
        .to_string();

    let is_idle = matches!(*state.read(), ImportState::Idle);
    let is_importing = matches!(*state.read(), ImportState::Importing);
    let can_import = is_idle && !passphrase.read().is_empty();
    let import_button_label: &'static str = if is_importing {
        "Importing\u{2026}"
    } else {
        "Import"
    };

    // All captures are Copy (Signal<T>), so do_import is itself Copy and can
    // be used in both the Button's on_click and the Input wrapper's onkeydown.
    let mut do_import = move || {
        if !can_import {
            return;
        }
        let pw = passphrase.read().clone();
        let path = path_signal.read().clone();
        state.set(ImportState::Importing);
        error_msg.set(String::new());

        spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || backup::import_vault(&path, &pw)).await;

            match result {
                Ok(Ok(entries)) => {
                    // Add entries; count new vs duplicate
                    let mut n_new = 0usize;
                    let mut n_skipped = 0usize;
                    let mut db_err: Option<String> = None;
                    let mut app = APP_STATE.write();
                    for entry in entries {
                        match app.add_entry(entry) {
                            Ok(true) => n_new += 1,
                            Ok(false) => n_skipped += 1,
                            Err(e) => {
                                db_err = Some(format!("Import failed: {e}"));
                                break;
                            }
                        }
                    }
                    drop(app);
                    if let Some(msg) = db_err {
                        error_msg.set(msg);
                        state.set(ImportState::Idle);
                    } else {
                        state.set(ImportState::Result {
                            imported: n_new,
                            skipped: n_skipped,
                        });
                    }
                }
                Ok(Err(TotpError::Backup(BackupError::WrongPassphrase))) => {
                    // Passphrase intentionally retained so user can correct it
                    error_msg.set("Incorrect passphrase.".to_string());
                    state.set(ImportState::Idle);
                }
                Ok(Err(TotpError::Backup(BackupError::BiometricNotSupported))) => {
                    error_msg.set("Biometric-only backups are not supported.".to_string());
                    state.set(ImportState::Idle);
                }
                Ok(Err(TotpError::Backup(BackupError::Io(e)))) => {
                    error_msg.set(format!("Could not read backup file: {e}"));
                    state.set(ImportState::Idle);
                }
                Ok(Err(TotpError::Backup(BackupError::InvalidFormat(msg)))) => {
                    error_msg.set(format!("Not a valid .brewvault file: {msg}"));
                    state.set(ImportState::Idle);
                }
                Ok(Err(e)) => {
                    error_msg.set(format!("Import failed: {e}"));
                    state.set(ImportState::Idle);
                }
                Err(_) => {
                    error_msg.set("Import failed: internal error.".to_string());
                    state.set(ImportState::Idle);
                }
            }
        });
    };

    let on_close_result = move |_: Event<MouseData>| {
        if let ImportState::Result { imported, .. } = *state.read() {
            *TOAST.write() = Some(ToastData {
                id: next_toast_id(),
                text: if imported == 1 {
                    "Imported 1 account".to_string()
                } else {
                    format!("Imported {imported} accounts")
                },
                bg_color: "#0f1825".to_string(),
                text_color: "#4f8ef7".to_string(),
            });
        }
        on_close(());
    };

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-[rgba(0,0,0,0.6)] flex items-center justify-center z-50 px-6",
            onclick: move |e| {
                // Only close on backdrop click in IDLE state; don't close mid-import
                if is_idle {
                    e.stop_propagation();
                    on_close(());
                }
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && is_idle {
                    on_close(());
                }
            },

            // Modal panel
            div {
                id: "import-modal",
                class: "w-full max-w-sm bg-surface border border-edge rounded-2xl p-6 flex flex-col gap-5",
                style: "box-shadow: 0 8px 32px rgba(0,0,0,0.6);",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e: KeyboardEvent| {
                    // Focus trap (IDLE and RESULT states)
                    if e.key() == Key::Tab {
                        e.prevent_default();
                        let dir: i32 = if e.modifiers().contains(Modifiers::SHIFT) { -1 } else { 1 };
                        let script = format!(r#"
                            (function() {{
                                var modal = document.getElementById('import-modal');
                                if (!modal) return;
                                var focusable = Array.from(modal.querySelectorAll('input:not([disabled]), button:not([disabled])'));
                                if (focusable.length === 0) return;
                                var idx = focusable.indexOf(document.activeElement);
                                var next = ((idx + {dir}) % focusable.length + focusable.length) % focusable.length;
                                focusable[next].focus();
                            }})();
                        "#);
                        let _ = dioxus::document::eval(&script);
                    }
                },

                // Header
                div { class: "flex items-center justify-between",
                    h2 { class: "text-base font-semibold text-primary", "Import Accounts" }
                    if is_idle {
                        button {
                            class: "text-muted hover:text-primary transition-colors text-lg leading-none cursor-pointer bg-transparent border-none",
                            onclick: move |_| on_close(()),
                            "×"
                        }
                    }
                }

                match state.read().clone() {
                    ImportState::Idle | ImportState::Importing => {
                        rsx! {
                            div { class: "flex flex-col gap-4",
                                // Filename
                                p { class: "text-sm text-muted",
                                    "File: "
                                    span { class: "text-primary font-medium", "{fname}" }
                                }

                                // Passphrase — wrapped in div for Enter-to-import
                                div {
                                    onkeydown: move |e: KeyboardEvent| {
                                        if e.key() == Key::Enter && can_import {
                                            do_import();
                                        }
                                    },
                                    Input {
                                        label: "Passphrase",
                                        placeholder: "Enter backup passphrase",
                                        value: passphrase,
                                        password: true,
                                        autofocus: true,
                                    }
                                }

                                if !error_msg.read().is_empty() {
                                    p { class: "text-xs text-danger", "{error_msg}" }
                                }
                            }

                            // Actions
                            div { class: "flex gap-2",
                                button {
                                    class: "flex-1 bg-surface2 border border-edge rounded-[10px] px-3 py-2 text-sm text-muted cursor-pointer transition-colors duration-[80ms] hover:bg-surface",
                                    disabled: is_importing,
                                    onclick: move |_| on_close(()),
                                    "Cancel"
                                }
                                div { class: "flex-1",
                                    Button {
                                        label: import_button_label,
                                        variant: ButtonVariant::Primary,
                                        disabled: !can_import || is_importing,
                                        on_click: move |_: Event<MouseData>| do_import(),
                                    }
                                }
                            }
                        }
                    }
                    ImportState::Result { imported, skipped } => {
                        rsx! {
                            div { class: "flex flex-col gap-3",
                                p { class: "text-sm text-primary font-medium", "Import complete" }
                                div { class: "bg-surface2 rounded-xl px-4 py-3 flex flex-col gap-1.5",
                                    div { class: "flex justify-between text-sm",
                                        span { class: "text-muted", "New accounts" }
                                        span { class: "text-primary font-semibold", "{imported}" }
                                    }
                                    div { class: "flex justify-between text-sm",
                                        span { class: "text-muted", "Already existed" }
                                        span { class: "text-[#555]", "{skipped}" }
                                    }
                                }
                            }

                            Button {
                                label: "Close",
                                variant: ButtonVariant::Primary,
                                on_click: on_close_result,
                            }
                        }
                    }
                }
            }
        }
    }
}
