use dioxus::prelude::*;

/// Section heading divider for the settings view.
#[component]
pub fn SettingsDivider(label: String) -> Element {
    rsx! {
        div {
            class: "px-6 pt-4 pb-1 text-[10px] font-bold uppercase tracking-widest text-[#2e2e2e]",
            "{label}"
        }
    }
}
