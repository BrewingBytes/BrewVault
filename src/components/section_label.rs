//! Alphabetical section header for the grouped accounts list.

use dioxus::prelude::*;

/// All-caps group header rendered above each letter group in the accounts list.
///
/// # Props
///
/// | Prop    | Type     | Description              |
/// |---------|----------|--------------------------|
/// | `label` | `String` | The letter/group heading. |
#[component]
pub fn SectionLabel(label: String) -> Element {
    rsx! {
        div {
            class: "pt-3.5 pb-1.5 text-[10px] font-bold uppercase tracking-widest text-[#2e2e2e]",
            { label }
        }
    }
}
