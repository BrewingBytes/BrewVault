use dioxus::prelude::*;

/// View for adding a new TOTP account.
#[component]
pub fn Add() -> Element {
    rsx! { div { class: "h-full flex items-center justify-center text-muted", "Add" } }
}
