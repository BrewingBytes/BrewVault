pub mod components;
pub mod models;
pub mod routes;
pub mod storage;
pub mod totp;
pub mod views;

use std::sync::atomic::AtomicU64;

/// Stores the Unix epoch seconds of the last user interaction.
/// Written on every mousemove/keydown (via AppShell) without triggering
/// a re-render — uses atomic store instead of GlobalSignal.
pub static LAST_INTERACTION_SECS: AtomicU64 = AtomicU64::new(0);
