//! TOTP credential card component.
//!
//! Renders a single [`TotpEntry`] as a horizontal card showing the issuer
//! avatar, account details, live TOTP code, and a seconds-remaining countdown.
//! The code and countdown update automatically every second via a background
//! async loop.

use std::{sync::Arc, time::Duration};

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::{
    components::ring::Ring,
    models::totp::TotpEntry,
    totp::{format_code, generate_code, initials, seconds_remaining},
};

/// A live TOTP credential card.
///
/// Displays a single [`TotpEntry`] as a horizontal row:
///
/// ```text
/// ┌─────────────────────────────────────────┐
/// │ [GH]  GitHub                   123 456  │
/// │       user@example.com             23s  │
/// └─────────────────────────────────────────┘
/// ```
///
/// A background loop ticks every second, updating the countdown and
/// regenerating the code at the start of each new TOTP window.
///
/// # Props
///
/// | Prop    | Type        | Description                        |
/// |---------|-------------|------------------------------------|
/// | `entry` | [`TotpEntry`] | The credential to display.       |
#[component]
pub fn TotpCard(entry: TotpEntry) -> Element {
    let mut code = use_signal(|| generate_code(&entry).unwrap_or_else(|_| "------".into()));
    let mut secs = use_signal(|| seconds_remaining(&entry));

    let future_entry = Arc::new(entry.clone());
    use_future(move || {
        let entry = future_entry.clone();
        async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                let s = seconds_remaining(&entry);
                secs.set(s);
                if s == entry.period as u8
                    && let Ok(c) = generate_code(&entry)
                {
                    code.set(c);
                }
            }
        }
    });

    let secs_val = secs();
    let progress = secs_val as f64 / entry.period as f64;
    let avatar = initials(&entry.issuer);
    let color = if secs_val <= 7 {
        "text-danger"
    } else {
        "text-accent"
    };

    rsx! {
        div {
            class: "bg-surface border border-edge rounded-2xl px-4 py-3 flex items-center gap-4 w-full",

            // Avatar
            div {
                class: "bg-surface2 rounded-xl w-11 h-11 flex items-center justify-center shrink-0",
                span {
                    class: "text-primary text-sm font-bold tracking-wide",
                    { avatar }
                }
            }

            // Issuer + account
            div {
                class: "flex flex-col gap-0.5 min-w-0 flex-1",
                span {
                    class: "text-primary text-sm font-semibold leading-tight truncate",
                    { entry.issuer.clone() }
                }
                span {
                    class: "text-muted text-xs truncate",
                    { entry.account.clone() }
                }
            }

            // TOTP code + countdown
            div {
                class: "flex flex-col shrink-0",
                span {
                    class: format!("font-mono text-xl font-bold tracking-widest {color}"),
                    { format_code(&code()) }
                }
                span {
                    class: "text-muted text-xs tabular-nums self-end",
                    { format!("{}s", secs_val) }
                }
            }

            // Time-remaining ring
            Ring { progress, warn: secs_val <= 7 }
        }
    }
}
