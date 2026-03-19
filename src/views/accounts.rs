use std::{collections::BTreeMap, time::Duration};

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::components::account_row::AccountRow;
use crate::components::button::{Button, ButtonVariant};
use crate::components::section_label::SectionLabel;
use crate::models::app_state::APP_STATE;
use crate::models::totp::TotpEntry;
use crate::routes::Route;

/// Returns the number of seconds remaining in the current 30-second TOTP window.
fn global_seconds_remaining() -> u8 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (30 - (now.as_secs() % 30)) as u8
}

/// Header bar for the Accounts view showing the title, account count,
/// a live TOTP refresh countdown, and an add button.
#[component]
fn AccountsHeader() -> Element {
    let mut secs = use_signal(global_seconds_remaining);
    let nav = use_navigator();

    use_future(move || async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            secs.set(global_seconds_remaining());
        }
    });

    let secs_val = secs();
    let count = APP_STATE.read().get_entries().len();
    let secs_color = if secs_val <= 8 {
        "text-warn"
    } else {
        "text-muted"
    };

    rsx! {
        div { class: "px-6 pt-3.5 flex-shrink-0",
            div { class: "flex items-center justify-between mb-3.5",
                div {
                    span { class: "text-2xl font-bold text-primary leading-tight block",
                        "Authenticator"
                    }
                    span { class: "text-xs mt-1 text-muted block",
                        "{count} accounts · refreshes in "
                        span { class: "{secs_color}", "{secs_val}s" }
                    }
                }
                Button {
                    label: "+",
                    variant: ButtonVariant::Round,
                    on_click: move |_| { nav.push(Route::Add {}); },
                }
            }
        }
    }
}

/// Controlled search input that writes the user's query into `query`.
///
/// Displays a clear button when the query is non-empty.
#[component]
fn SearchBar(query: Signal<String>) -> Element {
    let mut focused = use_signal(|| false);

    let border = if focused() {
        "border-[#2a2a2a]"
    } else {
        "border-edge"
    };

    rsx! {
        div { class: "flex items-center gap-2 bg-surface {border} border rounded-xl px-3 py-2 transition-colors duration-200 mx-6 mb-3",
            // Magnifier icon
            svg {
                class: "w-3.5 h-3.5 stroke-[#333] stroke-2 flex-shrink-0",
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                view_box: "0 0 24 24",
                circle { cx: "11", cy: "11", r: "8" }
                line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
            }
            input {
                class: "flex-1 bg-transparent border-none text-primary text-sm outline-none placeholder:text-[#252525]",
                r#type: "text",
                placeholder: "Search",
                value: "{query}",
                onfocus: move |_| focused.set(true),
                onblur: move |_| focused.set(false),
                oninput: move |e| query.set(e.value()),
            }
            if !query().is_empty() {
                button {
                    class: "text-white text-xs cursor-pointer border-none bg-transparent",
                    onclick: move |_| query.set(String::new()),
                    "✕"
                }
            }
        }
    }
}

/// Main accounts list view.
///
/// Displays the [`AccountsHeader`], a [`SearchBar`], and the list of TOTP
/// entries grouped alphabetically by issuer. Shows an empty-state message when
/// no accounts have been added yet (or no entries match the search query).
#[component]
pub fn Accounts() -> Element {
    let query = use_signal(String::new);

    let entries = APP_STATE.read().get_entries().to_vec();

    let filtered: Vec<TotpEntry> = entries
        .iter()
        .filter(|e| {
            let q = query().to_lowercase();
            q.is_empty()
                || e.issuer.to_lowercase().contains(&q)
                || e.account.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let groups: BTreeMap<char, Vec<TotpEntry>> =
        filtered.into_iter().fold(BTreeMap::new(), |mut m, e| {
            let key = e
                .issuer
                .chars()
                .next()
                .unwrap_or('#')
                .to_uppercase()
                .next()
                .unwrap_or('#');
            m.entry(key).or_default().push(e);
            m
        });

    rsx! {
        div { class: "h-full flex flex-col",
            AccountsHeader {}
            SearchBar { query }
            if groups.is_empty() {
                div { class: "flex-1 flex flex-col items-center justify-center gap-2 text-muted",
                    span { class: "text-sm font-medium text-primary", "No accounts yet" }
                    span { class: "text-xs text-center", "Press the + button to add your first account" }
                }
            } else {
                div { class: "flex-1 overflow-y-auto px-6 pb-4",
                    for (letter, group) in groups {
                        SectionLabel { label: letter.to_string() }
                        for entry in group {
                            AccountRow { entry }
                        }
                    }
                }
            }
        }
    }
}
