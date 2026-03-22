use dioxus::prelude::*;

use crate::components::icons::ICheck;
use crate::components::toast::{TOAST, ToastData, next_toast_id};
use crate::models::app_state::APP_STATE;

const OPTIONS: &[(u64, &str)] = &[
    (0, "Off"),
    (60, "1 minute"),
    (300, "5 minutes"),
    (600, "10 minutes"),
    (900, "15 minutes"),
    (1800, "30 minutes"),
];

/// Modal picker for the auto-lock timeout.
///
/// `on_close` is called when the user dismisses the modal.
#[component]
pub fn AutoLockPicker(on_close: EventHandler<()>) -> Element {
    let current_secs = APP_STATE
        .read()
        .auto_lock_timeout
        .map(|d| d.as_secs())
        .unwrap_or(0);

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-[rgba(0,0,0,0.6)] flex items-center justify-center z-50 px-6",
            onclick: move |_| on_close(()),

            // Modal panel
            div {
                id: "auto-lock-modal",
                class: "w-full max-w-sm bg-surface border border-edge rounded-2xl overflow-hidden",
                style: "box-shadow: 0 8px 32px rgba(0,0,0,0.6);",
                onclick: move |e| e.stop_propagation(),
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Tab {
                        e.prevent_default();
                        let dir: i32 = if e.modifiers().contains(Modifiers::SHIFT) { -1 } else { 1 };
                        let script = format!(r#"
                            (function() {{
                                var modal = document.getElementById('auto-lock-modal');
                                if (!modal) return;
                                var focusable = Array.from(modal.querySelectorAll('button:not([disabled]), [tabindex="0"]:not([disabled])'));
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
                div { class: "flex items-center justify-between px-5 py-4 border-b border-edge",
                    h2 { class: "text-sm font-semibold text-primary", "Auto-lock" }
                    button {
                        class: "text-muted hover:text-primary transition-colors text-lg leading-none cursor-pointer bg-transparent border-none",
                        onclick: move |_| on_close(()),
                        "×"
                    }
                }

                // Options list
                div { class: "py-1",
                    for (secs, label) in OPTIONS.iter().copied() {
                        {
                            let is_selected = secs == current_secs;
                            rsx! {
                                div {
                                    key: "{secs}",
                                    class: "flex items-center justify-between px-5 py-3 cursor-pointer hover:bg-surface2 transition-colors duration-[80ms]",
                                    role: "option",
                                    aria_selected: if is_selected { "true" } else { "false" },
                                    tabindex: 0,
                                    onclick: move |_| {
                                        if let Err(e) = APP_STATE.write().set_auto_lock(secs) {
                                            *TOAST.write() = Some(ToastData {
                                                id: next_toast_id(),
                                                text: format!("Failed to save: {e}"),
                                                bg_color: "#1e0808".to_string(),
                                                text_color: "#f75f4f".to_string(),
                                            });
                                        }
                                        on_close(());
                                    },
                                    onkeydown: move |e| {
                                        if e.key() == Key::Enter || e.key() == Key::Character(" ".to_string()) {
                                            if let Err(e) = APP_STATE.write().set_auto_lock(secs) {
                                                *TOAST.write() = Some(ToastData {
                                                    id: next_toast_id(),
                                                    text: format!("Failed to save: {e}"),
                                                    bg_color: "#1e0808".to_string(),
                                                    text_color: "#f75f4f".to_string(),
                                                });
                                            }
                                            on_close(());
                                        }
                                    },
                                    span { class: "text-sm text-primary", "{label}" }
                                    if is_selected {
                                        ICheck { class: "w-4 h-4 text-accent" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
