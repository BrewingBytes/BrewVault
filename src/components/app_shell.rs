use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::LAST_INTERACTION_SECS;
use crate::components::bottom_nav::BottomNav;
use crate::components::context_menu::ContextMenu;
use crate::components::delete_confirm_modal::DeleteConfirmModal;
use crate::components::toast::Toast;
use crate::models::app_state::{APP_STATE, LockState};
use crate::routes::Route;
use crate::views::lock::Lock;
use crate::views::setup::Setup;
use dioxus::prelude::*;

/// Returns the current Unix epoch in whole seconds, or 0 on error.
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Root layout component that wraps every route.
///
/// Renders the correct top-level screen based on [`LockState`]:
/// - `FirstRun`  → [`Setup`] view (full-screen, no nav)
/// - `Locked`    → [`Lock`] view (full-screen, no nav)
/// - `Unlocked`  → normal route content via [`Outlet`] + conditional [`BottomNav`]
///
/// Records user interactions (mouse moves, key presses) to
/// `LAST_INTERACTION_SECS` and runs a background loop that locks the vault
/// after the configured auto-lock timeout elapses.
///
/// Suppresses the WebView's built-in context menu globally.
#[component]
pub fn AppShell() -> Element {
    let route = use_route::<Route>();

    // -----------------------------------------------------------------------
    // Auto-lock background loop
    // -----------------------------------------------------------------------
    use_future(|| async {
        use tokio::time::{Duration, sleep};

        loop {
            sleep(Duration::from_secs(1)).await;

            let timeout = APP_STATE.read().auto_lock_timeout;
            let lock_state = APP_STATE.read().lock_state.clone();

            if lock_state != LockState::Unlocked {
                continue;
            }

            let Some(timeout) = timeout else {
                continue;
            };

            let last = LAST_INTERACTION_SECS.load(Ordering::Relaxed);
            if last == 0 {
                // Interaction tracking hasn't started yet (app just launched).
                continue;
            }

            let elapsed = now_secs().saturating_sub(last);
            if elapsed >= timeout.as_secs() {
                APP_STATE.write().lock();
            }
        }
    });

    let lock_state = APP_STATE.read().lock_state.clone();

    match lock_state {
        LockState::FirstRun => rsx! {
            div {
                class: "h-screen bg-base",
                ondragstart: move |e| e.prevent_default(),
                Setup {}
            }
        },
        LockState::Locked => rsx! {
            div {
                class: "h-screen bg-base",
                ondragstart: move |e| e.prevent_default(),
                Lock {}
            }
        },
        LockState::Unlocked => {
            let show_nav = route != Route::Add {};
            rsx! {
                div {
                    class: "h-screen bg-base flex flex-col overflow-hidden relative",
                    // Suppress the WebView's built-in "Inspect Element" context menu globally
                    oncontextmenu: move |e| e.prevent_default(),
                    // Prevent drag-to-reveal white WebView background
                    ondragstart: move |e| e.prevent_default(),
                    // Record interactions for auto-lock (mouse)
                    onmousemove: move |_| {
                        LAST_INTERACTION_SECS.store(now_secs(), Ordering::Relaxed);
                    },
                    // Record interactions for auto-lock (keyboard)
                    onkeydown: move |_: Event<KeyboardData>| {
                        LAST_INTERACTION_SECS.store(now_secs(), Ordering::Relaxed);
                    },

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
    }
}
