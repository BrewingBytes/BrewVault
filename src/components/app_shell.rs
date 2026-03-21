use crate::components::bottom_nav::BottomNav;
use crate::components::context_menu::ContextMenu;
use crate::components::delete_confirm_modal::DeleteConfirmModal;
use crate::components::toast::Toast;
use crate::routes::Route;
use dioxus::prelude::*;

/// Root layout component that wraps every route.
///
/// Renders the page content via [`Outlet`] and conditionally shows
/// [`BottomNav`]. Also renders the global singletons [`Toast`],
/// [`ContextMenu`], and [`DeleteConfirmModal`] unconditionally (they are
/// no-ops when their respective signals are `None`).
///
/// Suppresses the default WebView context menu globally via
/// `oncontextmenu: prevent_default` on the root element — the app provides
/// its own context menu via the `ContextMenu` singleton.
#[component]
pub fn AppShell() -> Element {
    let route = use_route::<Route>();
    let show_nav = route != Route::Add {};

    rsx! {
        div {
            class: "h-screen bg-base flex flex-col overflow-hidden relative",
            // Suppress the WebView's built-in "Inspect Element" context menu globally
            oncontextmenu: move |e| e.prevent_default(),

            div { class: "flex-1 overflow-hidden", Outlet::<Route> {} }
            if show_nav {
                BottomNav {}
            }
            Toast {}
            ContextMenu {}
            DeleteConfirmModal {}
        }
    }
}
