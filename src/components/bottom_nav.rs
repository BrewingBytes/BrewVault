use dioxus::prelude::*;

#[component]
pub fn BottomNav() -> Element {
    rsx! {
        div {
            class: "flex-shrink-0 flex justify-around items-center py-3 bg-surface border-t border-edge",
        }
    }
}
