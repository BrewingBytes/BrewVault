use crate::components::icons::{ICog, IGrid};
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn BottomNav() -> Element {
    let route = use_route::<Route>();
    let accounts_class = if route == (Route::Accounts {}) {
        "text-accent font-semibold"
    } else {
        "text-[#333] font-normal"
    };
    let settings_class = if route == (Route::Settings {}) {
        "text-accent font-semibold"
    } else {
        "text-[#333] font-normal"
    };

    rsx! {
        div { class: "flex items-center border-t border-edge bg-base py-3 flex-shrink-0",
            Link {
                to: Route::Accounts {},
                class: "flex-1 flex flex-col items-center gap-1 bg-transparent border-none cursor-pointer py-1.5 transition-colors duration-150 {accounts_class}",
                IGrid {}
                span { class: "text-[10px] tracking-wide", "Accounts" }
            }
            Link {
                to: Route::Settings {},
                class: "flex-1 flex flex-col items-center gap-1 bg-transparent border-none cursor-pointer py-1.5 transition-colors duration-150 {settings_class}",
                ICog {}
                span { class: "text-[10px] tracking-wide", "Settings" }
            }
        }
    }
}
