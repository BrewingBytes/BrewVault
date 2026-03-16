use dioxus::prelude::*;

use crate::routes::Route;

/// Circular accent-coloured button that navigates to the Add route.
#[component]
pub fn AddButton() -> Element {
    let nav = use_navigator();
    rsx! {
        button {
            class: "w-9 h-9 rounded-full bg-accent flex items-center justify-center \
                    flex-shrink-0 border-none cursor-pointer text-white text-2xl font-light leading-none",
            onclick: move |_| {
                nav.push(Route::Add {});
            },
            "+"
        }
    }
}
