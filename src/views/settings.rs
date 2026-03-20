use dioxus::prelude::*;

use crate::components::{
    icons::{IChevronRight, IMoon, IShield, ITrash},
    setting_row::SettingRow,
    settings_divider::SettingsDivider,
    toggle::Toggle,
};

/// Application settings view.
#[component]
pub fn Settings() -> Element {
    let mut dark_mode = use_signal(|| true);
    let mut show_seconds = use_signal(|| false);
    let mut biometric = use_signal(|| false);

    rsx! {
        div {
            class: "h-full overflow-y-auto",
            // Title
            div {
                class: "px-6 pt-4 pb-2 text-2xl font-bold text-primary",
                "Settings"
            }

            // GENERAL
            SettingsDivider { label: "General" }
            SettingRow {
                icon: rsx! { IMoon { class: "w-4 h-4" } },
                label: "Dark Mode",
                on_click: move |_| dark_mode.set(!dark_mode()),
                Toggle {
                    checked: dark_mode(),
                    on_change: move |v| dark_mode.set(v),
                }
            }
            SettingRow {
                icon: rsx! { IMoon { class: "w-4 h-4" } },
                label: "Show seconds",
                on_click: move |_| show_seconds.set(!show_seconds()),
                Toggle {
                    checked: show_seconds(),
                    on_change: move |v| show_seconds.set(v),
                }
            }

            // SECURITY
            SettingsDivider { label: "Security" }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Auto-lock timeout",
                sub_label: "5 minutes",
                on_click: move |_| {},
                IChevronRight { class: "w-4 h-4 text-muted" }
            }
            SettingRow {
                icon: rsx! { IShield { class: "w-4 h-4" } },
                label: "Biometric unlock",
                on_click: move |_| biometric.set(!biometric()),
                Toggle {
                    checked: biometric(),
                    on_change: move |v| biometric.set(v),
                }
            }

            // DANGER ZONE
            SettingsDivider { label: "Danger Zone" }
            SettingRow {
                icon: rsx! { ITrash { class: "w-4 h-4" } },
                label: "Clear all entries",
                danger: true,
                on_click: move |_| {},
                IChevronRight { class: "w-4 h-4 text-danger" }
            }
        }
    }
}
