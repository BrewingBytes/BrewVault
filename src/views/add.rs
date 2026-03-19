//! View for manually adding a new TOTP account.
//!
//! Layout:
//!
//! ```text
//! ┌─────────────────────────────────┐
//! │ [← Back]   Add Account          │
//! ├─────────────────────────────────┤
//! │   ┌─────────────────────────┐   │
//! │   │  ┌──────────────────┐   │   │  QR placeholder (static)
//! │   │  │  [ ]  QR BOX [ ] │   │   │
//! │   │  └──────────────────┘   │   │
//! │   │  Scan QR Code           │   │
//! │   │  QR scanning coming soon│   │
//! │   └─────────────────────────┘   │
//! │   ──── or enter manually ────   │
//! │   ACCOUNT NAME  [__________]    │
//! │   ISSUER        [__________]    │
//! │   SECRET KEY    [__________]    │
//! │   Your secrets never leave...   │
//! │   [     Add Account (btn)  ]    │
//! └─────────────────────────────────┘
//! ```

use dioxus::prelude::*;

use crate::{
    components::{
        button::{Button, ButtonVariant},
        input::Input,
        text_divider::TextDivider,
    },
    models::{
        app_state::APP_STATE,
        totp::{Algorithm, Digits, TotpEntry},
    },
    routes::Route,
};

/// View for adding a new TOTP account manually.
#[component]
pub fn Add() -> Element {
    let account = use_signal(String::new);
    let issuer = use_signal(String::new);
    let secret = use_signal(String::new);
    let nav = use_navigator();

    let can_submit =
        !account().trim().is_empty() && !issuer().trim().is_empty() && !secret().trim().is_empty();

    let handle_submit = move |_| {
        let entry = TotpEntry {
            id: uuid::Uuid::new_v4().to_string(),
            issuer: issuer().trim().to_string(),
            account: account().trim().to_string(),
            secret: secret().trim().to_uppercase(),
            algorithm: Algorithm::Sha1,
            digits: Digits::Six,
            period: 30,
        };
        let _ = APP_STATE.write().add_entry(entry);
        nav.push(Route::Accounts {});
    };

    rsx! {
        div {
            class: "h-full flex flex-col overflow-y-auto",

            // Back row
            div {
                class: "flex items-center gap-3 px-6 pt-3 flex-shrink-0",
                Button {
                    label: "← Back",
                    variant: ButtonVariant::Secondary,
                    on_click: move |_| { nav.push(Route::Accounts {}); },
                }
                span {
                    class: "text-lg font-bold text-primary",
                    "Add Account"
                }
            }

            // QR placeholder
            div {
                class: "bg-surface rounded-3xl border border-edge p-6 flex flex-col items-center gap-3 mb-4 mx-6 mt-4",
                div {
                    class: "relative w-36 h-36 bg-[#0e0e0e] rounded-xl border border-dashed border-[#252525] flex items-center justify-center",
                    // Corner brackets
                    div {
                        class: "absolute w-4 h-4 border-t border-l border-accent top-0 left-0",
                        style: "border-width: 2.5px; border-radius: 3px 0 0 0;",
                    }
                    div {
                        class: "absolute w-4 h-4 border-t border-r border-accent top-0 right-0",
                        style: "border-width: 2.5px; border-radius: 0 3px 0 0;",
                    }
                    div {
                        class: "absolute w-4 h-4 border-b border-l border-accent bottom-0 left-0",
                        style: "border-width: 2.5px; border-radius: 0 0 0 3px;",
                    }
                    div {
                        class: "absolute w-4 h-4 border-b border-r border-accent bottom-0 right-0",
                        style: "border-width: 2.5px; border-radius: 0 0 3px 0;",
                    }
                }
                span {
                    class: "text-sm font-medium text-[#666]",
                    "Scan QR Code"
                }
                span {
                    class: "text-xs text-[#2e2e2e] mt-0.5",
                    "QR scanning coming soon"
                }
            }

            // Divider
            div {
                class: "mb-4 mx-6",
                TextDivider { label: "or enter manually" }
            }

            // Manual form
            div {
                class: "px-6 flex flex-col gap-4",
                Input {
                    label: "Account Name",
                    placeholder: "e.g. work@company.com",
                    value: account,
                }
                Input {
                    label: "Issuer",
                    placeholder: "e.g. GitHub, Google…",
                    value: issuer,
                }
                Input {
                    label: "Secret Key",
                    placeholder: "JBSWY3DPEHPK3PXP",
                    value: secret,
                    mono: true,
                }
            }

            // Submit button + security note
            div {
                class: "px-6 mt-4 mb-2",
                p {
                    class: "text-xs text-center text-[#222] mb-2.5",
                    "Your secrets never leave this device"
                }
                Button {
                    label: "Add Account",
                    variant: ButtonVariant::Primary,
                    disabled: !can_submit,
                    on_click: handle_submit,
                }
            }
        }
    }
}
