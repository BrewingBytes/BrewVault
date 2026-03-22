use dioxus::prelude::*;

use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::LAST_INTERACTION_SECS;
use crate::components::button::{Button, ButtonVariant};
use crate::components::icons::ILock;
use crate::models::app_state::APP_STATE;
use crate::models::error::TotpError;
use crate::storage;

/// Vault lock screen — shown when the vault is locked and needs a password.
///
/// Handles two cases:
/// - Password-protected vault: shows a password input and verifies on unlock.
/// - No-password vault (auto-locked): shows a single "Unlock" button that
///   reopens the vault with the sentinel key — no user input required.
#[component]
pub fn Lock() -> Element {
    let has_password = APP_STATE.read().has_password;
    let mut password = use_signal(String::new);
    let mut error_msg = use_signal(String::new);
    let mut wrong_attempts = use_signal(|| 0u32);

    let mut do_unlock = move || {
        let pw = if has_password {
            password.read().clone()
        } else {
            storage::NO_PASSWORD_KEY.to_string()
        };
        error_msg.set(String::new());

        match APP_STATE.write().unlock(&pw) {
            Ok(()) => {
                password.set(String::new());
                // Reset the auto-lock timer so the user isn't immediately re-locked.
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                LAST_INTERACTION_SECS.store(now, Ordering::Relaxed);
            }
            Err(TotpError::WrongPassword) => {
                *wrong_attempts.write() += 1;
                password.set(String::new());
                if wrong_attempts() >= 3 {
                    error_msg.set(
                        "Wrong password. If this continues, your vault may be corrupted."
                            .to_string(),
                    );
                } else {
                    error_msg.set("Wrong password.".to_string());
                }
            }
            Err(e) => {
                error_msg.set(e.to_string());
            }
        }
    };

    rsx! {
        div { class: "h-screen bg-base flex items-center justify-center px-6",
            div { class: "w-full max-w-sm flex flex-col items-center gap-6",

                // Icon + heading
                div { class: "flex flex-col items-center gap-3",
                    ILock { class: "w-10 h-10 text-accent" }
                    div { class: "text-center",
                        h1 { class: "text-xl font-bold text-primary", "BrewVault" }
                        p { class: "text-sm text-muted mt-1",
                            if has_password {
                                "Enter your password to unlock"
                            } else {
                                "Your vault is locked"
                            }
                        }
                    }
                }

                // Password field — only shown for password-protected vaults
                if has_password {
                    div { class: "w-full flex flex-col gap-1.5",
                        label { class: "text-xs text-muted font-medium", "Password" }
                        input {
                            class: "w-full bg-surface border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                            r#type: "password",
                            placeholder: "Master password",
                            autofocus: true,
                            value: "{password}",
                            oninput: move |e| {
                                password.set(e.value());
                                error_msg.set(String::new());
                            },
                            onkeydown: move |e| {
                                if e.key() == Key::Enter && !password.read().is_empty() {
                                    do_unlock();
                                }
                            },
                        }
                    }
                }

                // Unlock button
                div { class: "w-full",
                    Button {
                        label: "Unlock",
                        variant: ButtonVariant::Primary,
                        disabled: has_password && password.read().is_empty(),
                        on_click: move |_: Event<MouseData>| do_unlock(),
                    }
                }

                // Error message
                if !error_msg.read().is_empty() {
                    p { class: "text-xs text-danger text-center", "{error_msg}" }
                }
            }
        }
    }
}
