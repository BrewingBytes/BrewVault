//! Floating pill toast notification.
//!
//! Any part of the app can trigger a toast by writing to the [`TOAST`] global
//! signal with a [`ToastData`] value carrying the message and colours.
//!
//! # Example
//! ```rust,no_run
//! use brew_vault::components::toast::{next_toast_id, TOAST, ToastData};
//! *TOAST.write() = Some(ToastData {
//!     id: next_toast_id(),
//!     text: "Copied".to_string(),
//!     bg_color: "#0f1825".to_string(),
//!     text_color: "#4f8ef7".to_string(),
//! });
//! ```

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use dioxus::prelude::*;
use tokio::time::sleep;

static TOAST_GEN: AtomicU64 = AtomicU64::new(1);

/// Returns a unique ID for the next toast. Call once per write to [`TOAST`].
pub fn next_toast_id() -> u64 {
    TOAST_GEN.fetch_add(1, Ordering::Relaxed)
}

/// Payload written to [`TOAST`] to show a pill notification.
#[derive(Clone)]
pub struct ToastData {
    /// Unique monotonic ID used to detect toast replacement.
    pub id: u64,
    pub text: String,
    /// CSS colour value for the pill background (e.g. `"#0f1825"`).
    pub bg_color: String,
    /// CSS colour value for the pill text (e.g. `"#4f8ef7"`).
    pub text_color: String,
}

/// Global signal that drives the toast. Write `Some(ToastData { … })` to show a toast.
pub static TOAST: GlobalSignal<Option<ToastData>> = GlobalSignal::new(|| None);

/// Floating pill notification rendered by [`AppShell`].
///
/// Reads [`TOAST`]. When the signal is `Some`, shows the pill with a rise-in
/// animation, waits 1 400 ms, fades it out over 200 ms, then clears the signal.
///
/// **Note:** `bottom-[86px]` is tied to the BottomNav bar height.
#[component]
pub fn Toast() -> Element {
    let mut exiting = use_signal(|| false);

    // IMPORTANT: use_future must come before the early-return guard so that
    // Dioxus sees the same hook count on every render.
    use_future(move || async move {
        loop {
            // Read and immediately drop the guard so we don't hold it across
            // an `.await` point.
            let snapshot_id = TOAST.read().as_ref().map(|t| t.id);
            if snapshot_id.is_some() {
                sleep(Duration::from_millis(1400)).await;
                // If the toast was replaced while we slept, skip the exit
                // animation and let the loop restart for the new toast.
                let current_id = TOAST.read().as_ref().map(|t| t.id);
                if current_id == snapshot_id {
                    exiting.set(true);
                    sleep(Duration::from_millis(200)).await;
                    // Re-check: a new toast may have arrived during the fade window.
                    let final_id = TOAST.read().as_ref().map(|t| t.id);
                    if final_id == snapshot_id {
                        *TOAST.write() = None;
                    }
                    exiting.set(false);
                }
            } else {
                sleep(Duration::from_millis(50)).await;
            }
        }
    });

    let data = TOAST.read().clone();
    let Some(toast) = data else {
        return rsx! {};
    };

    let anim_style = if exiting() {
        "opacity: 0; transform: translateY(6px); transition: opacity 200ms ease-in, transform 200ms ease-in;".to_string()
    } else {
        "animation: toast-in 140ms ease-out forwards;".to_string()
    };

    rsx! {
        div {
            class: "absolute bottom-[86px] inset-x-0 mx-auto w-fit rounded-full px-4 py-[7px] text-xs font-medium whitespace-nowrap pointer-events-none",
            style: "{anim_style} background-color: {toast.bg_color}; color: {toast.text_color};",
            { toast.text }
        }
    }
}
