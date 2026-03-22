use dioxus::prelude::*;

/// A reusable radio-button circle.
///
/// Renders a 16 px circle: accent-bordered with a filled inner dot when
/// `selected`, muted-bordered when unselected.
#[component]
pub fn Radio(selected: bool, on_click: EventHandler<()>) -> Element {
    let border = if selected {
        "border-accent"
    } else {
        "border-muted"
    };

    rsx! {
        div {
            class: "w-4 h-4 rounded-full border-2 {border} flex items-center justify-center cursor-pointer flex-shrink-0",
            onclick: move |_| on_click(()),
            if selected {
                div { class: "w-2 h-2 rounded-full bg-accent" }
            }
        }
    }
}
