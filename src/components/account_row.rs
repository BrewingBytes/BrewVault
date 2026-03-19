//! Live TOTP account row for the grouped accounts list.
//!
//! Renders a single [`TotpEntry`] as a horizontal row with a colored avatar,
//! issuer + account labels, the live TOTP code, and a countdown ring. Clicking
//! the row copies the raw code to the clipboard and briefly highlights the row.

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::{
    components::ring::Ring,
    models::totp::TotpEntry,
    totp::{format_code, generate_code, initials, seconds_remaining},
};

/// Deterministic background/foreground color pair for an issuer's avatar.
///
/// Hashes the issuer name and picks one of eight dark-background / bright-text
/// pairs so every issuer gets a consistent color across renders.
fn icon_colors(issuer: &str) -> (&'static str, &'static str) {
    const PALETTE: [(&str, &str); 8] = [
        ("#1a2a1a", "#4caf50"),
        ("#1a1a2e", "#4f8ef7"),
        ("#2a1a1a", "#f75f4f"),
        ("#2a2a1a", "#f97316"),
        ("#1a2a2a", "#26c6da"),
        ("#251a2e", "#ab47bc"),
        ("#1a251a", "#66bb6a"),
        ("#2e251a", "#ffa726"),
    ];
    let mut hasher = DefaultHasher::new();
    issuer.hash(&mut hasher);
    let idx = (hasher.finish() as usize) % PALETTE.len();
    PALETTE[idx]
}

/// A live TOTP account row.
///
/// Displays a single [`TotpEntry`] as a tappable row. Clicking copies the code
/// to the clipboard and shows a brief "copied" state. The code and countdown
/// update automatically every second via a background async loop.
///
/// # Props
///
/// | Prop    | Type        | Description                        |
/// |---------|-------------|------------------------------------|
/// | `entry` | [`TotpEntry`] | The credential to display.       |
#[component]
pub fn AccountRow(entry: TotpEntry) -> Element {
    let mut code = use_signal(|| generate_code(&entry).unwrap_or_else(|_| "------".into()));
    let mut secs = use_signal(|| seconds_remaining(&entry));
    let mut hovered = use_signal(|| false);
    let mut copied = use_signal(|| false);

    // 1-second tick: update countdown and regenerate code at window boundary.
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

    // Reset copied highlight 1.6s after it is set.
    use_future(move || async move {
        loop {
            if copied() {
                sleep(Duration::from_millis(1600)).await;
                copied.set(false);
            } else {
                sleep(Duration::from_millis(50)).await;
            }
        }
    });

    let secs_val = secs();
    let progress = secs_val as f64 / entry.period as f64;
    let warn = progress < 0.2;
    let avatar = initials(&entry.issuer);
    let (icon_bg, icon_fg) = icon_colors(&entry.issuer);
    let icon_size = if avatar.len() == 1 {
        "text-sm"
    } else {
        "text-xs"
    };

    let (bg, border) = if copied() {
        ("bg-[#0f1825]", "border-[#1e3258]")
    } else if hovered() {
        ("bg-[#141414]", "border-[#222]")
    } else {
        ("bg-surface", "border-edge")
    };

    let code_color = if copied() {
        "text-accent"
    } else if warn {
        "text-warn"
    } else {
        "text-primary"
    };

    rsx! {
        div {
            class: "{bg} border {border} rounded-2xl px-4 py-3 flex items-center gap-4 w-full mb-2 cursor-pointer transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/50",
            tabindex: "0",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |_| {
                let raw_code = code().replace(' ', "");
                spawn(async move {
                    let js = format!("navigator.clipboard.writeText('{raw_code}')");
                    if dioxus::document::eval(&js).await.is_ok() {
                        copied.set(true);
                    }
                });
            },
            onkeydown: move |e| {
                if e.key() == Key::Enter || e.key() == Key::Character(" ".to_string()) {
                    let raw_code = code().replace(' ', "");
                    spawn(async move {
                        let js = format!("navigator.clipboard.writeText('{raw_code}')");
                        if dioxus::document::eval(&js).await.is_ok() {
                            copied.set(true);
                        }
                    });
                }
            },

            // Colored avatar with initials
            div {
                class: "rounded-xl w-11 h-11 flex items-center justify-center shrink-0",
                style: "background-color: {icon_bg};",
                span {
                    class: "{icon_size} font-bold tracking-wide",
                    style: "color: {icon_fg};",
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

            // TOTP code + sub-label
            div {
                class: "flex flex-col items-end shrink-0",
                span {
                    class: "font-mono text-xl font-bold tracking-widest {code_color}",
                    { format_code(&code()) }
                }
                if copied() {
                    span {
                        class: "text-[10px] tabular-nums text-accent/50",
                        "copied"
                    }
                } else {
                    span {
                        class: "text-[10px] tabular-nums text-[#2a2a2a]",
                        "{secs_val}s"
                    }
                }
            }

            // Time-remaining ring
            Ring { progress, warn }
        }
    }
}
