//! Delete confirmation modal — standalone, reusable public component.
//!
//! Writes to [`DELETE_MODAL`] to open the modal; setting it to `None` closes it.
//! [`AppShell`] renders this component unconditionally (same pattern as `Toast`).

use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::toast::{TOAST, ToastData, next_toast_id};
use crate::models::app_state::APP_STATE;
use crate::models::totp::TotpEntry;

/// Global signal that controls the delete confirmation modal.
///
/// Set to `Some(entry)` to open the modal for that entry.
/// Set to `None` (or press Cancel) to close without deleting.
pub static DELETE_MODAL: GlobalSignal<Option<TotpEntry>> = GlobalSignal::new(|| None);

/// Delete confirmation modal component.
///
/// Renders a backdrop + centered modal when [`DELETE_MODAL`] is `Some`.
/// Provides Cancel (safe default focus) and Delete buttons.
#[component]
pub fn DeleteConfirmModal() -> Element {
    let modal_read = DELETE_MODAL.read();

    if modal_read.is_none() {
        return rsx! {};
    }

    let entry = modal_read.as_ref().unwrap().clone();
    drop(modal_read);

    let issuer = entry.issuer.clone();
    let account = entry.account.clone();
    let entry_id = entry.id.clone();

    rsx! {
        // Backdrop — click to cancel
        div {
            class: "fixed inset-0 z-[60] flex items-center justify-center",
            style: "background: var(--color-overlay);",
            onclick: move |_| { *DELETE_MODAL.write() = None; },

            // Modal surface — stop propagation so clicks on it don't close
            div {
                id: "delete-confirm-modal",
                class: "bg-surface border border-edge rounded-2xl p-6 max-w-[320px] w-full mx-4",
                style: "box-shadow: var(--shadow-menu);",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Escape {
                        *DELETE_MODAL.write() = None;
                    }
                    if e.key() == Key::Tab {
                        e.prevent_default();
                        let dir: i32 = if e.modifiers().contains(Modifiers::SHIFT) { -1 } else { 1 };
                        let script = format!(r#"
                            (function() {{
                                var modal = document.getElementById('delete-confirm-modal');
                                if (!modal) return;
                                var focusable = Array.from(modal.querySelectorAll('button:not([disabled])'));
                                if (focusable.length === 0) return;
                                var idx = focusable.indexOf(document.activeElement);
                                var next = ((idx + {dir}) % focusable.length + focusable.length) % focusable.length;
                                focusable[next].focus();
                            }})();
                        "#);
                        let _ = dioxus::document::eval(&script);
                    }
                },

                // Title
                h2 {
                    class: "text-base font-semibold text-primary mb-2",
                    "Delete Entry?"
                }

                // Body
                p {
                    class: "text-sm text-muted mb-1",
                    "This will permanently remove "
                    span { class: "text-primary font-medium", "{issuer}" }
                    " ("
                    span { class: "text-primary", "{account}" }
                    ")."
                }

                // Warning
                p {
                    class: "text-xs text-warn mb-5",
                    "This action cannot be undone."
                }

                // Actions
                div {
                    class: "flex gap-[10px]",
                    // Cancel — safe default (gets focus via autofocus)
                    button {
                        class: "flex-1 bg-surface2 border border-edge rounded-[10px] px-4 py-2 text-sm text-muted cursor-pointer transition-colors duration-[80ms] hover:bg-surface",
                        autofocus: true,
                        onclick: move |_| { *DELETE_MODAL.write() = None; },
                        "Cancel"
                    }
                    // Delete
                    Button {
                        label: "Delete",
                        variant: ButtonVariant::Danger,
                        on_click: move |_| {
                            let id = entry_id.clone();
                            let iss = issuer.clone();
                            match APP_STATE.write().remove_entry(&id) {
                                Ok(()) => {
                                    *TOAST.write() = Some(ToastData {
                                        id: next_toast_id(),
                                        text: format!("Deleted {iss}"),
                                        bg_color: "#1a0a0a".to_string(),
                                        text_color: "#f75f4f".to_string(),
                                    });
                                    *DELETE_MODAL.write() = None;
                                }
                                Err(_) => {
                                    *TOAST.write() = Some(ToastData {
                                        id: next_toast_id(),
                                        text: format!("Failed to delete {iss}"),
                                        bg_color: "#1e0808".to_string(),
                                        text_color: "#f75f4f".to_string(),
                                    });
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}
