use dioxus::prelude::*;

use crate::components::button::{Button, ButtonVariant};
use crate::components::strength_bar::StrengthBar;
use crate::models::app_state::APP_STATE;
use crate::models::error::TotpError;

/// Modal for changing (or removing) the vault master password.
///
/// `on_close` is called on cancel or successful completion.
#[component]
pub fn ChangePasswordModal(on_close: EventHandler<()>) -> Element {
    let has_password = APP_STATE.read().has_password;

    let mut current_pw = use_signal(String::new);
    let mut new_pw = use_signal(String::new);
    let mut confirm_pw = use_signal(String::new);
    let mut error_msg = use_signal(String::new);
    let is_rekeying = use_signal(|| false);

    let new_pw_val = new_pw.read().clone();
    let confirm_val = confirm_pw.read().clone();

    // Validation
    let new_too_short = !new_pw_val.is_empty() && new_pw_val.len() < 8;
    let mismatch = !confirm_val.is_empty() && new_pw_val != confirm_val;

    let can_save = !is_rekeying()
        && new_pw_val.len() >= 8
        && new_pw_val == confirm_val
        && (!has_password || !current_pw.read().is_empty());

    // Save handler called from both the Button and the Enter keydown
    let mut do_save = move || {
        error_msg.set(String::new());
        let cur = current_pw.read().clone();
        let new = new_pw.read().clone();

        match APP_STATE.write().change_password(&cur, Some(&new)) {
            Ok(()) => {
                on_close(());
            }
            Err(TotpError::WrongPassword) => {
                error_msg.set("Current password is incorrect.".to_string());
            }
            Err(e) => {
                error_msg.set(e.to_string());
            }
        }
    };

    let on_remove_password = move |_: Event<MouseData>| {
        error_msg.set(String::new());
        let cur = current_pw.read().clone();

        match APP_STATE.write().change_password(&cur, None) {
            Ok(()) => {
                on_close(());
            }
            Err(TotpError::WrongPassword) => {
                error_msg.set("Current password is incorrect.".to_string());
            }
            Err(e) => {
                error_msg.set(e.to_string());
            }
        }
    };

    let save_label = if is_rekeying() {
        "Saving…"
    } else {
        "Save Changes"
    };

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 bg-[rgba(0,0,0,0.6)] flex items-center justify-center z-50 px-6",
            onclick: move |_| on_close(()),

            // Modal panel (stop propagation so click inside doesn't close)
            div {
                class: "w-full max-w-sm bg-surface border border-edge rounded-2xl p-6 flex flex-col gap-5",
                style: "box-shadow: 0 8px 32px rgba(0,0,0,0.6);",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "flex items-center justify-between",
                    h2 { class: "text-base font-semibold text-primary", "Change Password" }
                    button {
                        class: "text-muted hover:text-primary transition-colors text-lg leading-none cursor-pointer bg-transparent border-none",
                        onclick: move |_| on_close(()),
                        "×"
                    }
                }

                div { class: "flex flex-col gap-4",
                    // Current password (only when vault is password-protected)
                    if has_password {
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs text-muted font-medium", "Current password" }
                            input {
                                class: "w-full bg-surface2 border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                                r#type: "password",
                                placeholder: "Enter current password",
                                value: "{current_pw}",
                                oninput: move |e| current_pw.set(e.value()),
                            }
                        }
                    }

                    // New password
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-muted font-medium", "New password" }
                        input {
                            class: "w-full bg-surface2 border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                            r#type: "password",
                            placeholder: "At least 8 characters",
                            value: "{new_pw}",
                            oninput: move |e| new_pw.set(e.value()),
                        }
                        StrengthBar { password: new_pw_val.clone() }
                        if new_too_short {
                            p { class: "text-xs text-danger", "At least 8 characters required" }
                        }
                    }

                    // Confirm password
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs text-muted font-medium", "Confirm new password" }
                        input {
                            class: "w-full bg-surface2 border border-edge rounded-xl px-3 py-2.5 text-sm text-primary outline-none focus:border-accent transition-colors",
                            r#type: "password",
                            placeholder: "Repeat new password",
                            value: "{confirm_pw}",
                            oninput: move |e| confirm_pw.set(e.value()),
                            onkeydown: move |e| {
                                if e.key() == Key::Enter && can_save {
                                    do_save();
                                }
                            },
                        }
                        if mismatch {
                            p { class: "text-xs text-danger", "Passwords don't match" }
                        }
                    }
                }

                // Error
                if !error_msg.read().is_empty() {
                    p { class: "text-xs text-danger", "{error_msg}" }
                }

                // Actions
                div { class: "flex flex-col gap-2",
                    Button {
                        label: save_label,
                        variant: ButtonVariant::Primary,
                        disabled: !can_save,
                        on_click: move |_: Event<MouseData>| do_save(),
                    }

                    if has_password {
                        button {
                            class: "text-xs text-danger text-center py-1 cursor-pointer bg-transparent border-none",
                            disabled: current_pw.read().is_empty() || is_rekeying(),
                            onclick: on_remove_password,
                            "Remove password"
                        }
                    }
                }
            }
        }
    }
}
