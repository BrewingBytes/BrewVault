use dioxus::prelude::*;

#[component]
pub fn Add() -> Element {
    rsx! { div { class: "h-full flex items-center justify-center text-muted", "Add" } }
}
