//! Rename modal — standalone, reusable public component.
//!
//! Renders inline (replacing the context menu content). The parent passes
//! `on_confirm` and `on_cancel` callbacks so this component is independent
//! of the context menu and can be reused elsewhere.

use dioxus::prelude::*;

use crate::models::totp::TotpEntry;

/// Inline rename form that replaces the context-menu surface content.
///
/// # Props
///
/// | Prop         | Type                                   | Description                         |
/// |--------------|----------------------------------------|-------------------------------------|
/// | `entry`      | [`TotpEntry`]                          | Entry being renamed (pre-fills form). |
/// | `on_confirm` | `EventHandler<(String, String)>`       | Called with `(new_issuer, new_account)` when Confirm is clicked. |
/// | `on_cancel`  | `EventHandler<()>`                     | Called when Cancel is clicked.       |
#[component]
pub fn RenameModal(
    entry: TotpEntry,
    on_confirm: EventHandler<(String, String)>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut issuer = use_signal(|| entry.issuer.clone());
    let mut account = use_signal(|| entry.account.clone());

    let issuer_empty = issuer().trim().is_empty();

    let issuer_border = if issuer_empty {
        "border-danger"
    } else {
        "border-edge focus-within:border-accent"
    };

    rsx! {
        div {
            id: "rename-modal",
            class: "p-4",
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    on_cancel.call(());
                }
                if e.key() == Key::Tab {
                    e.prevent_default();
                    let dir: i32 = if e.modifiers().contains(Modifiers::SHIFT) { -1 } else { 1 };
                    let script = format!(r#"
                        (function() {{
                            var modal = document.getElementById('rename-modal');
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

            // Title
            h3 {
                class: "text-base font-semibold text-primary mb-1",
                "Rename"
            }
            p {
                class: "text-[13px] text-muted mb-4",
                "Update the name shown for this account."
            }

            // Issuer field
            div { class: "mb-3",
                label {
                    class: "text-[11px] font-medium text-muted uppercase tracking-wide mb-[6px] block",
                    "Issuer"
                }
                input {
                    class: "w-full bg-surface2 border {issuer_border} rounded-[10px] px-3 py-[9px] text-[14px] text-primary outline-none transition-colors duration-[80ms] placeholder:text-[#333]",
                    r#type: "text",
                    placeholder: "e.g. GitHub",
                    value: "{issuer}",
                    autofocus: true,
                    oninput: move |e| issuer.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter && !issuer().trim().is_empty() {
                            on_confirm.call((issuer().trim().to_string(), account().trim().to_string()));
                        }
                    },
                }
            }

            // Account field
            div { class: "mb-4",
                label {
                    class: "text-[11px] font-medium text-muted uppercase tracking-wide mb-[6px] block",
                    "Account"
                }
                input {
                    class: "w-full bg-surface2 border border-edge rounded-[10px] px-3 py-[9px] text-[14px] text-primary outline-none focus:border-accent transition-colors duration-[80ms] placeholder:text-[#333]",
                    r#type: "text",
                    placeholder: "e.g. user@example.com",
                    value: "{account}",
                    oninput: move |e| account.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter && !issuer().trim().is_empty() {
                            on_confirm.call((issuer().trim().to_string(), account().trim().to_string()));
                        }
                    },
                }
            }

            // Actions
            div {
                class: "flex gap-[10px]",
                // Cancel
                button {
                    class: "flex-1 bg-surface2 border border-edge rounded-[10px] px-3 py-2 text-sm text-muted cursor-pointer transition-colors duration-[80ms] hover:bg-surface",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                // Confirm — disabled when issuer is empty/whitespace
                button {
                    class: if issuer_empty {
                        "flex-1 bg-[#181818] text-[#2a2a2a] rounded-[10px] px-3 py-2 text-sm cursor-default"
                    } else {
                        "flex-1 bg-accent text-white rounded-[10px] px-3 py-2 text-sm font-semibold cursor-pointer transition-colors duration-[80ms]"
                    },
                    disabled: issuer_empty,
                    onclick: move |_| {
                        if !issuer_empty {
                            on_confirm.call((issuer().trim().to_string(), account().trim().to_string()));
                        }
                    },
                    "Confirm"
                }
            }
        }
    }
}
