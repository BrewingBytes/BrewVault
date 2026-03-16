use dioxus::prelude::*;

/// Application settings view.
#[component]
pub fn Settings() -> Element {
    rsx! { div { class: "h-full flex items-center justify-center text-muted", "Settings" } }
}
