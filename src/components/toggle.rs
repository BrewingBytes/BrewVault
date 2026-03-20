use dioxus::prelude::*;

/// Fully-controlled ON/OFF toggle switch.
///
/// Does not manage its own state — the caller owns `checked` and handles `on_change`.
#[component]
pub fn Toggle(checked: bool, on_change: EventHandler<bool>) -> Element {
    let track_class = if checked {
        "w-11 h-6 rounded-xl relative cursor-pointer bg-accent border border-accent transition-colors duration-200"
    } else {
        "w-11 h-6 rounded-xl relative cursor-pointer bg-[#1e1e1e] border border-[#2a2a2a] transition-colors duration-200"
    };

    let thumb_left = if checked { "left-[21px]" } else { "left-0.5" };

    rsx! {
        div {
            class: "{track_class}",
            onclick: move |_| on_change.call(!checked),
            div {
                class: "w-4 h-4 rounded-full bg-white absolute top-0.5 shadow-sm transition-[left] duration-[180ms] {thumb_left}",
            }
        }
    }
}
