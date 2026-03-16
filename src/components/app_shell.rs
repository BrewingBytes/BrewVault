use crate::components::bottom_nav::BottomNav;
use crate::routes::Route;
use dioxus::prelude::*;

/// Root layout component that wraps every route.
///
/// Renders the page content via [`Outlet`] and conditionally shows
/// [`BottomNav`] — the nav bar is hidden on the Add route.
#[component]
pub fn AppShell() -> Element {
    let route = use_route::<Route>();
    let show_nav = route != Route::Add {};

    rsx! {
        div {
            class: "h-screen bg-base flex flex-col overflow-hidden",
            div { class: "flex-1 overflow-hidden", Outlet::<Route> {} }
            if show_nav {
                BottomNav {}
            }
        }
    }
}
