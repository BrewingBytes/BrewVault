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
    models::totp::TotpEntry,
    totp::{generate_code, seconds_remaining},
};

/// Formats a raw TOTP digit string for display.
///
/// Inserts a space at the midpoint so the code is easier to read at a glance.
///
/// # Examples
///
/// ```text
/// "123456"   → "123 456"
/// "12345678" → "1234 5678"
/// ```
fn format_code(code: &str) -> String {
    let mid = code.len() / 2;
    format!("{} {}", &code[..mid], &code[mid..])
}

/// Derives up to two uppercase initials from an issuer name.
///
/// - Multi-word names use the first letter of each of the first two words:
///   `"Brewing Bytes"` → `"BB"`.
/// - Single-word names use the first two characters: `"GitHub"` → `"GI"`,
///   `"X"` → `"X"`.
fn initials(issuer: &str) -> String {
    let mut words = issuer.split_whitespace();
    match (words.next(), words.next()) {
        (Some(a), Some(b)) => format!(
            "{}{}",
            a.chars()
                .next()
                .unwrap_or_default()
                .to_uppercase()
                .next()
                .unwrap_or_default(),
            b.chars()
                .next()
                .unwrap_or_default()
                .to_uppercase()
                .next()
                .unwrap_or_default(),
        ),
        (Some(a), None) => a.chars().take(2).flat_map(|c| c.to_uppercase()).collect(),
        _ => String::new(),
    }
}

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
    let avatar = initials(&entry.issuer);

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
                    class: "font-mono text-accent text-xl font-bold tracking-widest",
                    { format_code(&code()) }
                }
                span {
                    class: "text-muted text-xs tabular-nums self-end",
                    { format!("{}s", secs_val) }
                }
            }
        }
    }
}
