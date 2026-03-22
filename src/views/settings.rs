use std::path::PathBuf;

use dioxus::prelude::*;

use crate::components::{
    auto_lock_picker::AutoLockPicker,
    change_password_modal::ChangePasswordModal,
    export_modal::ExportModal,
    icons::{
        ICheck, IChevronRight, ICloud, IDownload, IGlobe, IInfo, ILock, IShield, IStar, ITrash,
        IUpload,
    },
    import_modal::ImportModal,
    setting_row::SettingRow,
    settings_divider::SettingsDivider,
    toast::{TOAST, ToastData, next_toast_id},
    toggle::Toggle,
};
use crate::file_picker;
use crate::models::app_state::APP_STATE;
use crate::totp::initials;

fn nyi() {
    *TOAST.write() = Some(ToastData {
        id: next_toast_id(),
        text: "Not yet implemented".to_string(),
        bg_color: "#1e0808".to_string(),
        text_color: "#f75f4f".to_string(),
    });
}

/// Profile card shown at the top of Settings.
#[component]
fn ProfileCard(initials: String, name: String, email: String) -> Element {
    rsx! {
        div {
            class: "mx-6 mt-3.5 mb-0.5 bg-surface rounded-2xl border border-edge py-3.5 px-4",
            div {
                class: "flex items-center gap-3",
                // Avatar
                div {
                    class: "w-11 h-11 rounded-xl flex-shrink-0 bg-[#111d30] border border-[#1e3050] flex items-center justify-center text-lg font-bold text-accent",
                    "{initials}"
                }
                // Name + email
                div {
                    class: "flex-1 min-w-0",
                    div { class: "text-sm font-semibold text-primary", "{name}" }
                    div { class: "text-xs text-[#383838] mt-0.5 truncate", "{email}" }
                }
                // PRO badge
                div {
                    class: "bg-[#0e1e0e] text-[#3d9e5f] text-[9.5px] font-bold tracking-wide rounded-lg px-2 py-0.5 flex-shrink-0",
                    "PRO"
                }
            }
        }
    }
}

/// Inline language picker that replaces the language row when open.
#[component]
fn LanguagePicker(selected: String, on_select: EventHandler<String>) -> Element {
    let options = ["English", "French", "Romanian"];

    rsx! {
        div {
            class: "mx-6 my-1 bg-surface rounded-xl border border-edge overflow-hidden",
            for lang in options {
                {
                    let is_selected = selected == lang;
                    let bg = if is_selected { "bg-[#1a1a1a]" } else { "bg-transparent" };
                    let text_color = if is_selected { "text-primary" } else { "text-muted" };
                    let lang_str = lang.to_string();
                    rsx! {
                        div {
                            key: "{lang}",
                            class: "flex items-center justify-between px-3.5 py-2.5 border-b border-edge last:border-b-0 cursor-pointer transition-colors duration-100 {bg}",
                            onclick: move |_| on_select.call(lang_str.clone()),
                            span { class: "text-sm {text_color}", "{lang}" }
                            if is_selected {
                                ICheck { class: "w-3.5 h-3.5 text-accent" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Inline delete confirmation panel that replaces the delete row when open.
#[component]
fn DeleteConfirm(on_cancel: EventHandler<()>, on_delete: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "mx-6 my-1 bg-[#120808] border border-[#2e1010] rounded-xl p-3.5",
            div { class: "text-[#e05050] text-sm font-semibold mb-1", "Delete all accounts?" }
            div { class: "text-[#3a3a3a] text-xs mb-3.5", "This action is permanent and cannot be undone." }
            div {
                class: "flex gap-2",
                button {
                    class: "flex-1 py-2.5 bg-surface2 border border-edge rounded-xl text-[#666] text-sm cursor-pointer",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: "flex-1 py-2.5 bg-[#1e0808] border border-[#3a1010] rounded-xl text-danger text-sm font-semibold cursor-pointer",
                    onclick: move |_| on_delete.call(()),
                    "Delete all"
                }
            }
        }
    }
}

/// Returns a human-readable label for the current auto-lock timeout.
fn auto_lock_label() -> String {
    match APP_STATE.read().auto_lock_timeout {
        None => "Off".to_string(),
        Some(d) => {
            let secs = d.as_secs();
            if secs < 60 {
                format!("{secs}s")
            } else {
                let mins = secs / 60;
                if mins == 1 {
                    "1 minute".to_string()
                } else {
                    format!("{mins} minutes")
                }
            }
        }
    }
}

/// Application settings view.
#[component]
pub fn Settings() -> Element {
    let biometric_unlock = use_signal(|| false);
    let block_screenshots = use_signal(|| false);
    let cloud_sync = use_signal(|| false);
    let mut language_picker_open = use_signal(|| false);
    let selected_language = use_signal(|| "English".to_string());
    let mut delete_confirm_open = use_signal(|| false);
    let mut change_password_open = use_signal(|| false);
    let mut auto_lock_picker_open = use_signal(|| false);
    let mut export_modal_open = use_signal(|| false);
    let mut import_path: Signal<Option<PathBuf>> = use_signal(|| None);

    let profile_name = "BrewVault";
    let profile_email = "brewvault@brewingbytes.com";
    let profile_initials = initials(profile_name);

    rsx! {
        div {
            class: "h-full overflow-y-auto pb-6",

            ProfileCard {
                initials: profile_initials,
                name: profile_name.to_string(),
                email: profile_email.to_string(),
            }

            // SECURITY
            SettingsDivider { label: "Security" }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Biometric unlock",
                on_click: move |_| {},
                Toggle {
                    checked: biometric_unlock(),
                    on_change: move |_| nyi(),
                }
            }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Auto-lock",
                sub_label: auto_lock_label(),
                on_click: move |_| auto_lock_picker_open.set(true),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Block screenshots",
                on_click: move |_| {},
                Toggle {
                    checked: block_screenshots(),
                    on_change: move |_| nyi(),
                }
            }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Change Password",
                on_click: move |_| change_password_open.set(true),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            SettingRow {
                icon: rsx! { ILock { class: "w-4 h-4" } },
                label: "Lock vault",
                on_click: move |_| APP_STATE.write().lock(),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }

            // BACKUP & SYNC
            SettingsDivider { label: "Backup & Sync" }
            SettingRow {
                icon: rsx! { ICloud { class: "w-4 h-4" } },
                label: "Cloud sync",
                on_click: move |_| {},
                Toggle {
                    checked: cloud_sync(),
                    on_change: move |_| nyi(),
                }
            }
            SettingRow {
                icon: rsx! { IDownload { class: "w-4 h-4" } },
                label: "Export backup",
                on_click: move |_| export_modal_open.set(true),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            SettingRow {
                icon: rsx! { IUpload { class: "w-4 h-4" } },
                label: "Import accounts",
                on_click: move |_| {
                    spawn(async move {
                        if let Some(p) = file_picker::open_file(&["brewvault"]).await {
                            import_path.set(Some(p));
                        }
                    });
                },
                IChevronRight { class: "w-4 h-4 text-muted" }
            }

            // PREFERENCES
            SettingsDivider { label: "Preferences" }
            SettingRow {
                icon: rsx! { IGlobe { class: "w-4 h-4" } },
                label: "Language",
                sub_label: selected_language(),
                on_click: move |_| language_picker_open.set(!language_picker_open()),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            if language_picker_open() {
                LanguagePicker {
                    selected: selected_language(),
                    on_select: move |_lang| {
                        nyi();
                        language_picker_open.set(false);
                    },
                }
            }

            // ABOUT
            SettingsDivider { label: "About" }
            SettingRow {
                icon: rsx! { IInfo { class: "w-4 h-4" } },
                label: "Version",
                on_click: None,
                span { class: "text-muted text-xs", { env!("CARGO_PKG_VERSION") } }
            }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Privacy policy",
                on_click: move |_| nyi(),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            SettingRow {
                icon: rsx! { IStar { class: "w-4 h-4" } },
                label: "Rate the app",
                on_click: move |_| nyi(),
                IChevronRight { class: "w-4 h-4 text-muted" }
            }

            // DANGER ZONE
            SettingsDivider { label: "Danger Zone" }
            if !delete_confirm_open() {
                SettingRow {
                    icon: rsx! { ITrash { class: "w-4 h-4" } },
                    label: "Delete all accounts",
                    danger: true,
                    on_click: move |_| delete_confirm_open.set(true),
                    IChevronRight { class: "w-4 h-4 text-danger" }
                }
            }
            if delete_confirm_open() {
                DeleteConfirm {
                    on_cancel: move |_| delete_confirm_open.set(false),
                    on_delete: move |_| {
                        let ok = APP_STATE.write().remove_all_entries().is_ok();
                        delete_confirm_open.set(false);
                        *TOAST.write() = Some(ToastData {
                            id: next_toast_id(),
                            text: if ok {
                                "All accounts deleted".to_string()
                            } else {
                                "Failed to delete accounts".to_string()
                            },
                            bg_color: if ok { "#0f1825".to_string() } else { "#1e0808".to_string() },
                            text_color: if ok { "#4f8ef7".to_string() } else { "#f75f4f".to_string() },
                        });
                    },
                }
            }
        }

        // Modals rendered outside the scrollable area so they sit above everything
        if change_password_open() {
            ChangePasswordModal {
                on_close: move |_| change_password_open.set(false),
            }
        }
        if auto_lock_picker_open() {
            AutoLockPicker {
                on_close: move |_| auto_lock_picker_open.set(false),
            }
        }
        if export_modal_open() {
            ExportModal {
                on_close: move |_| export_modal_open.set(false),
            }
        }
        if let Some(p) = import_path.read().clone() {
            ImportModal {
                path: p,
                on_close: move |_| import_path.set(None),
            }
        }
    }
}
