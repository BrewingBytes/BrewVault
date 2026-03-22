//! Export modal — passphrase entry + StrengthBar before the OS save dialog.

use chrono::Local;
use dioxus::prelude::*;

use crate::backup;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::strength_bar::StrengthBar;
use crate::components::toast::{TOAST, ToastData, next_toast_id};
use crate::file_picker;
use crate::models::app_state::APP_STATE;

/// Modal that prompts for a passphrase then triggers the OS save dialog.
///
/// Flow: user fills passphrase + confirm → clicks Export → modal closes →
/// OS save dialog opens → on success toast "Vault exported to …"
///
/// `on_close` is called when the user cancels or after a successful export.
#[component]
pub fn ExportModal(on_close: EventHandler<()>) -> Element {
    let passphrase = use_signal(String::new);
    let confirm = use_signal(String::new);
    let mut exporting = use_signal(|| false);

    let pw_val = passphrase.read().clone();
    let cn_val = confirm.read().clone();

    let entry_count = APP_STATE.read().get_entries().len();
    let no_entries = entry_count == 0;

    let mismatch = !cn_val.is_empty() && pw_val != cn_val;
    let can_export = !no_entries && !pw_val.is_empty() && pw_val == cn_val && !exporting();
    let button_label: &'static str = if no_entries {
        "No accounts to export"
    } else {
        "Export"
    };

    let do_export = move |_: Event<MouseData>| {
        // Re-check exporting() at call time — can_export is captured at render but a
        // double-click can fire before the component re-renders with the updated signal.
        if !can_export || exporting() {
            return;
        }
        exporting.set(true);
        let pw = passphrase.read().clone();
        let entries: Vec<_> = APP_STATE.read().get_entries().to_vec();
        // on_close is called inside the future so the component stays mounted
        // while the OS save dialog is open (Dioxus drops spawned futures when
        // the component that called spawn unmounts).

        spawn(async move {
            let default_name = format!("brewvault-{}.brewvault", Local::now().format("%Y-%m-%d"));
            let path = file_picker::save_file(&default_name, &["brewvault"]).await;

            match path {
                None => {
                    // User cancelled the OS dialog — just close the modal.
                    exporting.set(false);
                    on_close(());
                }
                Some(p) => {
                    let result = tokio::task::spawn_blocking(move || {
                        backup::export_vault(&entries, &pw, &p).map(|_| p)
                    })
                    .await;

                    exporting.set(false);
                    on_close(());

                    match result {
                        Ok(Ok(p)) => {
                            let fname = p
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("file")
                                .to_string();
                            *TOAST.write() = Some(ToastData {
                                id: next_toast_id(),
                                text: format!("Vault exported to {fname}"),
                                bg_color: "#0f1825".to_string(),
                                text_color: "#4f8ef7".to_string(),
                            });
                        }
                        Ok(Err(e)) => {
                            *TOAST.write() = Some(ToastData {
                                id: next_toast_id(),
                                text: format!("Export failed: {e}"),
                                bg_color: "#1e0808".to_string(),
                                text_color: "#f75f4f".to_string(),
                            });
                        }
                        Err(_) => {
                            *TOAST.write() = Some(ToastData {
                                id: next_toast_id(),
                                text: "Export failed: internal error".to_string(),
                                bg_color: "#1e0808".to_string(),
                                text_color: "#f75f4f".to_string(),
                            });
                        }
                    }
                }
            }
        });
    };

    rsx! {
        // Backdrop
        div {
            id: "export-modal-backdrop",
            class: "fixed inset-0 bg-[rgba(0,0,0,0.6)] flex items-center justify-center z-50 px-6",
            onclick: move |_| {
                if !exporting() {
                    on_close(());
                }
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape && !exporting() {
                    on_close(());
                }
            },

            // Modal panel
            div {
                id: "export-modal",
                class: "w-full max-w-sm bg-surface border border-edge rounded-2xl p-6 flex flex-col gap-5",
                style: "box-shadow: 0 8px 32px rgba(0,0,0,0.6);",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e: KeyboardEvent| {
                    // Focus trap: Tab/Shift+Tab cycles through focusable elements inside modal
                    if e.key() == Key::Tab {
                        e.prevent_default();
                        let dir: i32 = if e.modifiers().contains(Modifiers::SHIFT) { -1 } else { 1 };
                        let script = format!(r#"
                            (function() {{
                                var modal = document.getElementById('export-modal');
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
                    h2 { class: "text-base font-semibold text-primary", "Export Backup" }
                    button {
                        class: "text-muted hover:text-primary transition-colors text-lg leading-none cursor-pointer bg-transparent border-none",
                        onclick: move |_| on_close(()),
                        "×"
                    }
                }

                div { class: "flex flex-col gap-4",
                    // Passphrase
                    div {
                        Input {
                            label: "Passphrase",
                            placeholder: "Choose a strong passphrase",
                            value: passphrase,
                            password: true,
                            autofocus: true,
                        }
                        StrengthBar { password: pw_val.clone() }
                    }

                    // Confirm
                    div {
                        Input {
                            label: "Confirm Passphrase",
                            placeholder: "Repeat passphrase",
                            value: confirm,
                            password: true,
                        }
                        if mismatch {
                            p { class: "text-xs text-danger mt-1", "Passphrases don't match" }
                        }
                    }

                    // Help text
                    p {
                        class: "text-xs text-muted",
                        "Save this passphrase — it's needed to import the backup."
                    }
                }

                // Actions
                div { class: "flex gap-2",
                    // Cancel
                    button {
                        class: "flex-1 bg-surface2 border border-edge rounded-[10px] px-3 py-2 text-sm text-muted cursor-pointer transition-colors duration-[80ms] hover:bg-surface",
                        onclick: move |_| on_close(()),
                        "Cancel"
                    }
                    // Export
                    div { class: "flex-1",
                        Button {
                            label: button_label,
                            variant: ButtonVariant::Primary,
                            disabled: !can_export,
                            on_click: do_export,
                        }
                    }
                }
            }
        }
    }
}
