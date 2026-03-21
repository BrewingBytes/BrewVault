//! Right-click / long-press context menu — global singleton, mirrors the Toast pattern.
//!
//! Write to [`CONTEXT_MENU`] to open the menu; set it to `None` to close.
//! [`AppShell`] renders `ContextMenu {}` unconditionally so only one DOM node exists.

use dioxus::prelude::*;

use crate::components::delete_confirm_modal::DELETE_MODAL;
use crate::components::icons::{
    IArrowDown, IArrowLeft, IArrowUp, ICheck, ICircleSlash, IEdit, IFolder, IPlus, ITrash,
};
use crate::components::rename_modal::RenameModal;
use crate::components::toast::{TOAST, ToastData, next_toast_id};
use crate::models::app_state::APP_STATE;
use crate::models::totp::TotpEntry;

/// Data needed to position and populate the context menu.
pub struct ContextMenuData {
    pub entry: TotpEntry,
    /// Viewport X coordinate for the menu's top-left corner.
    pub x: f64,
    /// Viewport Y coordinate for the menu's top-left corner.
    pub y: f64,
    /// Whether the entry can be moved up in its group.
    pub can_move_up: bool,
    /// Whether the entry can be moved down in its group.
    pub can_move_down: bool,
}

/// Global signal that drives the context menu singleton.
///
/// Write `Some(ContextMenuData { … })` from an `AccountRow` right-click handler
/// to open the menu. Write `None` to close it.
pub static CONTEXT_MENU: GlobalSignal<Option<ContextMenuData>> = GlobalSignal::new(|| None);

/// Which sub-view is shown inside the menu surface.
#[derive(Clone, PartialEq)]
enum MenuView {
    Main,
    Rename,
    Category,
}

/// Shared styles for a normal menu item row.
const ITEM_CLASS: &str = "px-[14px] py-[9px] text-[13.5px] text-primary flex items-center gap-[10px] hover:bg-surface2 transition-colors duration-[80ms] cursor-pointer select-none";
/// Shared styles for a disabled menu item row (Move Up / Down at boundary).
const ITEM_DISABLED_CLASS: &str = "px-[14px] py-[9px] text-[13.5px] text-disabled flex items-center gap-[10px] cursor-not-allowed select-none";
/// Thin horizontal divider between menu groups.
const DIVIDER_CLASS: &str = "h-[1px] bg-edge my-[3px]";

/// The context menu singleton rendered by [`AppShell`].
#[component]
pub fn ContextMenu() -> Element {
    // --- Hooks must come before any early return ---
    let mut view = use_signal(|| MenuView::Main);
    let mut last_entry_id = use_signal(String::new);
    let mut new_cat_input = use_signal(String::new);
    let mut new_cat_expanded = use_signal(|| false);

    // Reset to Main view whenever a new menu opens (different entry ID).
    use_effect(move || {
        let new_id = CONTEXT_MENU
            .read()
            .as_ref()
            .map(|d| d.entry.id.clone())
            .unwrap_or_default();
        if new_id != last_entry_id() {
            last_entry_id.set(new_id);
            view.set(MenuView::Main);
            new_cat_input.set(String::new());
            new_cat_expanded.set(false);
        }
    });

    // --- Early return if menu is closed ---
    let menu_read = CONTEXT_MENU.read();
    if menu_read.is_none() {
        return rsx! {};
    }

    let data = menu_read.as_ref().unwrap();
    let entry = data.entry.clone();
    let x = data.x;
    let y = data.y;
    let can_move_up = data.can_move_up;
    let can_move_down = data.can_move_down;
    drop(menu_read);

    // Collect existing group names for the category picker.
    // Priority groups (Dev → Work → Personal) always appear first in that order,
    // followed by any other groups from existing entries sorted alphabetically.
    let all_groups: Vec<String> = {
        const PRIORITY: &[&str] = &["Dev", "Work", "Personal"];
        let mut seen: std::collections::HashSet<String> =
            PRIORITY.iter().map(|s| s.to_string()).collect();
        // Start with priority groups in order
        let mut groups: Vec<String> = PRIORITY.iter().map(|s| s.to_string()).collect();
        // Collect any additional groups from entries, sorted alphabetically
        let mut extra: Vec<String> = APP_STATE
            .read()
            .get_entries()
            .iter()
            .filter_map(|e| e.group.clone())
            .filter(|g| seen.insert(g.clone()))
            .collect();
        extra.sort();
        groups.extend(extra);
        groups
    };
    let current_group = entry.group.clone();

    rsx! {
        // Full-screen transparent overlay — click-outside closes, Escape closes
        div {
            id: "ctx-menu-overlay",
            class: "fixed inset-0 z-40",
            tabindex: "0",
            onclick: move |_| { *CONTEXT_MENU.write() = None; },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Escape {
                    *CONTEXT_MENU.write() = None;
                }
            },
        }

        // Menu surface — positioned at cursor, CSS clamp keeps it inside the viewport
        div {
            class: "fixed z-50 bg-surface border border-edge rounded-xl py-1 min-w-[180px]",
            style: "left: 0; top: 0; transform: translate(clamp(8px, {x}px, calc(100vw - 100% - 8px)), clamp(8px, {y}px, calc(100vh - 100% - 8px))); box-shadow: var(--shadow-menu);",

            match view() {
                // ─── Main menu ───────────────────────────────────────────
                MenuView::Main => rsx! {
                    // Rename
                    div {
                        class: ITEM_CLASS,
                        onclick: move |e| { e.stop_propagation(); view.set(MenuView::Rename); },
                        IEdit {}
                        span { "Rename" }
                    }

                    div { class: DIVIDER_CLASS }

                    // Move Up
                    if can_move_up {
                        div {
                            class: ITEM_CLASS,
                            onclick: {
                                let id = entry.id.clone();
                                move |e| {
                                    e.stop_propagation();
                                    let _ = APP_STATE.write().move_entry_up(&id);
                                    *CONTEXT_MENU.write() = None;
                                }
                            },
                            IArrowUp {}
                            span { "Move Up" }
                        }
                    } else {
                        div {
                            class: ITEM_DISABLED_CLASS,
                            IArrowUp { class: "w-[15px] h-[15px] opacity-35".to_string() }
                            span { "Move Up" }
                        }
                    }

                    // Move Down
                    if can_move_down {
                        div {
                            class: ITEM_CLASS,
                            onclick: {
                                let id = entry.id.clone();
                                move |e| {
                                    e.stop_propagation();
                                    let _ = APP_STATE.write().move_entry_down(&id);
                                    *CONTEXT_MENU.write() = None;
                                }
                            },
                            IArrowDown {}
                            span { "Move Down" }
                        }
                    } else {
                        div {
                            class: ITEM_DISABLED_CLASS,
                            IArrowDown { class: "w-[15px] h-[15px] opacity-35".to_string() }
                            span { "Move Down" }
                        }
                    }

                    div { class: DIVIDER_CLASS }

                    // Change Category
                    div {
                        class: ITEM_CLASS,
                        onclick: move |e| { e.stop_propagation(); view.set(MenuView::Category); },
                        IFolder {}
                        span { "Change Category" }
                    }

                    div { class: DIVIDER_CLASS }

                    // Delete — danger color
                    div {
                        class: "px-[14px] py-[9px] text-[13.5px] text-danger flex items-center gap-[10px] hover:bg-surface2 transition-colors duration-[80ms] cursor-pointer select-none",
                        onclick: {
                            let e_clone = entry.clone();
                            move |ev| {
                                ev.stop_propagation();
                                *DELETE_MODAL.write() = Some(e_clone.clone());
                                *CONTEXT_MENU.write() = None;
                            }
                        },
                        ITrash {}
                        span { "Delete" }
                    }
                },

                // ─── Rename view (inline form) ────────────────────────────
                MenuView::Rename => rsx! {
                    RenameModal {
                        entry: entry.clone(),
                        on_confirm: {
                            let id = entry.id.clone();
                            let iss_for_toast = entry.issuer.clone();
                            move |(new_issuer, new_account): (String, String)| {
                                if APP_STATE
                                    .write()
                                    .rename_entry(&id, &new_issuer, &new_account)
                                    .is_ok()
                                {
                                    *TOAST.write() = Some(ToastData {
                                        id: next_toast_id(),
                                        text: format!("Renamed {iss_for_toast}"),
                                        bg_color: "#0f1825".to_string(),
                                        text_color: "#4f8ef7".to_string(),
                                    });
                                }
                                *CONTEXT_MENU.write() = None;
                            }
                        },
                        on_cancel: move |_| { view.set(MenuView::Main); },
                    }
                },

                // ─── Category picker ──────────────────────────────────────
                MenuView::Category => rsx! {
                    // Back arrow row
                    div {
                        class: "px-[14px] py-[9px] text-[13.5px] text-muted flex items-center gap-[10px] hover:bg-surface2 transition-colors duration-[80ms] cursor-pointer select-none",
                        onclick: move |e| { e.stop_propagation(); view.set(MenuView::Main); },
                        IArrowLeft {}
                        span { "Change Category" }
                    }

                    div { class: DIVIDER_CLASS }

                    // Scrollable group list (max 6 rows)
                    div {
                        class: "max-h-[216px] overflow-y-auto",

                        // Current group (always first, active style)
                        if let Some(ref g) = current_group {
                            {
                                let g = g.clone();
                                rsx! {
                                    div {
                                        class: "px-[14px] py-[9px] text-[13.5px] text-accent flex items-center gap-[10px] select-none cursor-default",
                                        IFolder { class: "w-[15px] h-[15px]".to_string() }
                                        span { class: "flex-1", "{g}" }
                                        ICheck { class: "w-[13px] h-[13px]".to_string() }
                                    }
                                }
                            }
                        }

                        // Other groups (alphabetical, excluding current)
                        for g in all_groups.iter().filter(|g| Some(*g) != current_group.as_ref()) {
                            {
                                let g = g.clone();
                                let entry_id = entry.id.clone();
                                rsx! {
                                    div {
                                        class: ITEM_CLASS,
                                        onclick: move |ev| {
                                            ev.stop_propagation();
                                            let gname = g.clone();
                                            if APP_STATE
                                                .write()
                                                .update_entry_group(&entry_id, Some(&gname))
                                                .is_ok()
                                            {
                                                *TOAST.write() = Some(ToastData {
                                                    id: next_toast_id(),
                                                    text: format!("Moved to {gname}"),
                                                    bg_color: "#0f1825".to_string(),
                                                    text_color: "#4f8ef7".to_string(),
                                                });
                                            }
                                            *CONTEXT_MENU.write() = None;
                                        },
                                        IFolder {}
                                        span { "{g}" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: DIVIDER_CLASS }

                    // "New category…" row
                    if !new_cat_expanded() {
                        div {
                            class: "px-[14px] py-[9px] text-[13.5px] text-muted italic flex items-center gap-[10px] hover:bg-surface2 transition-colors duration-[80ms] cursor-pointer select-none",
                            onclick: move |e| { e.stop_propagation(); new_cat_expanded.set(true); },
                            IPlus {}
                            span { "New category…" }
                        }
                    } else {
                        // Inline new-category input
                        div {
                            class: "px-[10px] py-[8px] flex items-center gap-[8px] bg-surface2 border-t border-edge",
                            onclick: move |e| e.stop_propagation(),
                            IFolder { class: "w-[14px] h-[14px] text-muted shrink-0".to_string() }
                            input {
                                class: "flex-1 bg-transparent text-[13px] text-primary outline-none placeholder:text-[#333]",
                                r#type: "text",
                                placeholder: "Category name",
                                autofocus: true,
                                value: "{new_cat_input}",
                                oninput: move |e| new_cat_input.set(e.value()),
                                onkeydown: {
                                    let entry_id = entry.id.clone();
                                    move |e: KeyboardEvent| {
                                        if e.key() == Key::Escape {
                                            new_cat_expanded.set(false);
                                            new_cat_input.set(String::new());
                                        } else if e.key() == Key::Enter {
                                            let cat = new_cat_input().trim().to_string();
                                            if !cat.is_empty() {
                                                if APP_STATE
                                                    .write()
                                                    .update_entry_group(&entry_id, Some(&cat))
                                                    .is_ok()
                                                {
                                                    *TOAST.write() = Some(ToastData {
                                                        id: next_toast_id(),
                                                        text: format!("Moved to {cat}"),
                                                        bg_color: "#0f1825".to_string(),
                                                        text_color: "#4f8ef7".to_string(),
                                                    });
                                                }
                                                *CONTEXT_MENU.write() = None;
                                            }
                                        }
                                    }
                                },
                            }
                            // Add button
                            {
                                let entry_id = entry.id.clone();
                                let cat_val = new_cat_input();
                                let cat_empty = cat_val.trim().is_empty();
                                rsx! {
                                    button {
                                        class: if cat_empty {
                                            "text-[12px] bg-[#181818] text-[#2a2a2a] rounded-[6px] px-2 py-1 cursor-default"
                                        } else {
                                            "text-[12px] bg-accent text-white rounded-[6px] px-2 py-1 cursor-pointer"
                                        },
                                        disabled: cat_empty,
                                        onclick: move |ev| {
                                            ev.stop_propagation();
                                            let cat = new_cat_input().trim().to_string();
                                            if !cat.is_empty() {
                                                if APP_STATE
                                                    .write()
                                                    .update_entry_group(&entry_id, Some(&cat))
                                                    .is_ok()
                                                {
                                                    *TOAST.write() = Some(ToastData {
                                                        id: next_toast_id(),
                                                        text: format!("Moved to {cat}"),
                                                        bg_color: "#0f1825".to_string(),
                                                        text_color: "#4f8ef7".to_string(),
                                                    });
                                                }
                                                *CONTEXT_MENU.write() = None;
                                            }
                                        },
                                        "Add"
                                    }
                                }
                            }
                        }
                    }

                    // "No group" row
                    {
                        let entry_id = entry.id.clone();
                        rsx! {
                            div {
                                class: "px-[14px] py-[9px] text-[13.5px] text-muted italic flex items-center gap-[10px] hover:bg-surface2 transition-colors duration-[80ms] cursor-pointer select-none",
                                onclick: move |ev| {
                                    ev.stop_propagation();
                                    if APP_STATE
                                        .write()
                                        .update_entry_group(&entry_id, None)
                                        .is_ok()
                                    {
                                        *TOAST.write() = Some(ToastData {
                                            id: next_toast_id(),
                                            text: "Removed from group".to_string(),
                                            bg_color: "#0f1825".to_string(),
                                            text_color: "#4f8ef7".to_string(),
                                        });
                                    }
                                    *CONTEXT_MENU.write() = None;
                                },
                                ICircleSlash {}
                                span { "No group" }
                            }
                        }
                    }
                },
            }
        }
    }
}
