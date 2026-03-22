use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::icons::ILock;
use crate::components::radio::Radio;
use crate::components::strength_bar::StrengthBar;
use crate::models::app_state::APP_STATE;
use crate::storage::NO_PASSWORD_KEY;

/// First-run setup view.
///
/// Shown instead of the normal app shell when no vault database exists yet.
/// The user picks a master password or skips password protection.
#[component]
pub fn Setup() -> Element {
    let mut use_password = use_signal(|| true);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut error_msg = use_signal(String::new);

    let pw = password.read().clone();
    let cf = confirm.read().clone();
    let using_pw = use_password();

    let reserved = using_pw && pw == NO_PASSWORD_KEY;

    let can_submit = if using_pw {
        pw.len() >= 8 && pw == cf && !reserved
    } else {
        true
    };

    let inline_err = if using_pw && !pw.is_empty() && pw.len() < 8 {
        Some("At least 8 characters required")
    } else if using_pw && reserved {
        Some("That password is reserved — please choose a different one")
    } else if using_pw && !cf.is_empty() && pw != cf {
        Some("Passwords don't match")
    } else {
        None
    };

    let mut do_submit = move || {
        error_msg.set(String::new());
        if use_password() {
            let pw = password.read().clone();
            match APP_STATE.write().setup_with_password(&pw) {
                Ok(()) => {}
                Err(e) => error_msg.set(e.to_string()),
            }
        } else {
            match APP_STATE.write().setup_no_password() {
                Ok(()) => {}
                Err(e) => error_msg.set(e.to_string()),
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
                        h1 { class: "text-xl font-bold text-primary", "Secure your vault" }
                        p { class: "text-sm text-muted mt-1",
                            "Choose how to protect your entries."
                        }
                    }
                }

                // Radio options
                div { class: "w-full bg-surface border border-edge rounded-2xl divide-y divide-edge overflow-hidden",
                    div {
                        class: "flex items-center gap-3 px-4 py-3.5 cursor-pointer",
                        onclick: move |_| use_password.set(true),
                        Radio {
                            selected: using_pw,
                            on_click: move |_| use_password.set(true),
                        }
                        div {
                            p { class: "text-sm font-medium text-primary", "Set a password" }
                            p { class: "text-xs text-muted mt-0.5",
                                "Encrypts your vault with your master password"
                            }
                        }
                    }
                    div {
                        class: "flex items-center gap-3 px-4 py-3.5 cursor-pointer",
                        onclick: move |_| use_password.set(false),
                        Radio {
                            selected: !using_pw,
                            on_click: move |_| use_password.set(false),
                        }
                        div {
                            p { class: "text-sm font-medium text-primary", "No password" }
                            p { class: "text-xs text-muted mt-0.5",
                                "Vault is still encrypted, but opens automatically"
                            }
                        }
                    }
                }

                // Password fields (only when "Set a password" is selected)
                if using_pw {
                    div { class: "w-full flex flex-col gap-3",
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs text-muted font-medium", "Password" }
                            input {
                                class: "w-full bg-surface border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                                r#type: "password",
                                placeholder: "Enter password",
                                value: "{password}",
                                oninput: move |e| password.set(e.value()),
                            }
                            StrengthBar { password: pw.clone() }
                        }

                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs text-muted font-medium", "Confirm password" }
                            input {
                                class: "w-full bg-surface border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                                r#type: "password",
                                placeholder: "Confirm password",
                                value: "{confirm}",
                                oninput: move |e| confirm.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Enter && can_submit {
                                        do_submit();
                                    }
                                },
                            }
                        }

                        if let Some(msg) = inline_err {
                            p { class: "text-xs text-danger", "{msg}" }
                        }
                    }
                }

                // Submit button
                div { class: "w-full",
                    Button {
                        label: "Get Started",
                        variant: ButtonVariant::Primary,
                        disabled: !can_submit,
                        on_click: move |_: Event<MouseData>| do_submit(),
                    }
                }

                if !error_msg.read().is_empty() {
                    p { class: "text-xs text-danger text-center", "{error_msg}" }
                }

                // Recovery notice
                if using_pw {
                    p { class: "text-xs text-warn text-center px-4",
                        "Warning: if you forget your password, your vault data cannot be recovered."
                    }
                }
            }
        }
    }
}
