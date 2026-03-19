use std::{collections::HashMap, time::Duration};

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::components::account_row::AccountRow;
use crate::components::button::{Button, ButtonVariant};
use crate::components::icons::IMagnifier;
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

/// Splits entries into labeled groups (priority-ordered) and ungrouped entries.
///
/// Priority order: Dev → Work → Personal, then any other named groups alphabetically.
/// Entries with `group: None` are returned separately and rendered flat without a label.
fn group_entries(entries: Vec<TotpEntry>) -> (Vec<(String, Vec<TotpEntry>)>, Vec<TotpEntry>) {
    const PRIORITY: &[&str] = &["Dev", "Work", "Personal"];

    let mut ungrouped: Vec<TotpEntry> = Vec::new();
    let mut map: HashMap<String, Vec<TotpEntry>> = HashMap::new();

    for entry in entries {
        match entry.group.clone() {
            Some(g) => map.entry(g).or_default().push(entry),
            None => ungrouped.push(entry),
        }
    }

    let mut ordered: Vec<(String, Vec<TotpEntry>)> = Vec::new();
    for &name in PRIORITY {
        if let Some(group) = map.remove(name) {
            ordered.push((name.to_string(), group));
        }
    }

    let mut remaining: Vec<_> = map.into_iter().collect();
    remaining.sort_by(|a, b| a.0.cmp(&b.0));
    ordered.extend(remaining);

    (ordered, ungrouped)
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
            IMagnifier { class: "w-3.5 h-3.5 text-[#333] flex-shrink-0" }
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
/// When no search is active, entries are grouped by their `group` field with
/// priority ordering (Dev → Work → Personal → others). When a search query is
/// active, a flat filtered list is shown without section labels. Two distinct
/// empty states handle the no-entries and no-results cases.
#[component]
pub fn Accounts() -> Element {
    let query = use_signal(String::new);

    let entries = APP_STATE.read().get_entries().to_vec();
    let has_any_entries = !entries.is_empty();
    let search_active = !query().is_empty();

    let filtered: Vec<TotpEntry> = entries
        .into_iter()
        .filter(|e| {
            let q = query().to_lowercase();
            q.is_empty()
                || e.issuer.to_lowercase().contains(&q)
                || e.account.to_lowercase().contains(&q)
        })
        .collect();

    rsx! {
        div { class: "h-full flex flex-col",
            AccountsHeader {}
            SearchBar { query }

            if !has_any_entries {
                // No entries at all
                div { class: "flex flex-col items-center justify-center mt-16 gap-2",
                    IMagnifier {}
                    span { class: "text-sm text-[#252525]", "No accounts yet" }
                    span { class: "text-xs text-[#252525] text-center px-8",
                        "Press the + button to add your first account"
                    }
                }
            } else if search_active && filtered.is_empty() {
                // Search returned no results
                div { class: "flex flex-col items-center justify-center mt-16 gap-2",
                    IMagnifier {}
                    span { class: "text-sm text-[#252525]",
                        "No results for \"{query}\""
                    }
                }
            } else if search_active {
                // Flat filtered list — no section labels
                div { class: "flex-1 overflow-y-auto px-6 pb-4",
                    for entry in filtered {
                        AccountRow { entry }
                    }
                }
            } else {
                // Grouped list: labeled groups first, then ungrouped entries flat
                div { class: "flex-1 overflow-y-auto px-6 pb-4",
                    {
                        let (labeled, ungrouped) = group_entries(filtered);
                        rsx! {
                            for (group_name, group_entries) in labeled {
                                SectionLabel { label: group_name }
                                for entry in group_entries {
                                    AccountRow { entry }
                                }
                            }
                            for entry in ungrouped {
                                AccountRow { entry }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::totp::{Algorithm, Digits};

    fn entry(issuer: &str, group: Option<&str>) -> TotpEntry {
        TotpEntry {
            id: issuer.to_string(),
            issuer: issuer.to_string(),
            account: "a@b.com".to_string(),
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            algorithm: Algorithm::Sha1,
            digits: Digits::Six,
            period: 30,
            group: group.map(str::to_string),
        }
    }

    #[test]
    fn ungrouped_entries_returned_separately() {
        let entries = vec![entry("GitHub", None), entry("AWS", None)];
        let (labeled, ungrouped) = group_entries(entries);
        assert!(labeled.is_empty());
        assert_eq!(ungrouped.len(), 2);
    }

    #[test]
    fn priority_order_dev_work_personal() {
        let entries = vec![
            entry("App1", Some("Personal")),
            entry("App2", Some("Work")),
            entry("App3", Some("Dev")),
        ];
        let (labeled, ungrouped) = group_entries(entries);
        assert!(ungrouped.is_empty());
        let names: Vec<&str> = labeled.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["Dev", "Work", "Personal"]);
    }

    #[test]
    fn unknown_groups_sorted_alphabetically_after_priority() {
        let entries = vec![
            entry("App1", Some("Personal")),
            entry("App2", Some("Zebra")),
            entry("App3", Some("Alpha")),
        ];
        let (labeled, _) = group_entries(entries);
        let names: Vec<&str> = labeled.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(names, ["Personal", "Alpha", "Zebra"]);
    }

    #[test]
    fn mixed_grouped_and_ungrouped() {
        let entries = vec![entry("App1", Some("Work")), entry("App2", None)];
        let (labeled, ungrouped) = group_entries(entries);
        assert_eq!(labeled.len(), 1);
        assert_eq!(labeled[0].0, "Work");
        assert_eq!(ungrouped.len(), 1);
        assert_eq!(ungrouped[0].issuer, "App2");
    }
}
