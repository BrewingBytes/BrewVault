//! Horizontal text divider component.

use dioxus::prelude::*;

/// A horizontal divider with a centered text label.
///
/// ```text
/// ──────────── or enter manually ────────────
/// ```
///
/// # Props
///
/// | Prop    | Type           | Description               |
/// |---------|----------------|---------------------------|
/// | `label` | `&'static str` | Text shown at the center. |
#[component]
pub fn TextDivider(label: &'static str) -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2.5",
            div { class: "flex-1 h-px bg-edge" }
            span { class: "text-xs text-[#2e2e2e]", { label } }
            div { class: "flex-1 h-px bg-edge" }
        }
    }
}
