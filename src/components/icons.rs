use dioxus::prelude::*;

/// 2×2 grid icon used for the Accounts nav item.
#[component]
pub fn IGrid() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "w-5 h-5",
            // top-left square
            rect { x: "3", y: "3", width: "7", height: "7", rx: "1" }
            // top-right square
            rect { x: "14", y: "3", width: "7", height: "7", rx: "1" }
            // bottom-left square
            rect { x: "3", y: "14", width: "7", height: "7", rx: "1" }
            // bottom-right square
            rect { x: "14", y: "14", width: "7", height: "7", rx: "1" }
        }
    }
}

/// Magnifier/search icon.
///
/// `class` controls size and color via Tailwind. `stroke="currentColor"` means
/// the stroke colour follows the CSS `color` property (use `text-*` classes).
/// Default: `w-7 h-7 text-[#252525]` — sized for empty states.
#[component]
pub fn IMagnifier(
    #[props(default = "w-7 h-7 text-[#252525]".to_string())] class: String,
) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

/// Chevron pointing right — used as a navigation indicator in settings rows.
#[component]
pub fn IChevronRight(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 18l6-6-6-6" }
        }
    }
}

/// Moon icon — used for dark mode setting.
#[component]
pub fn IMoon(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" }
        }
    }
}

/// Shield icon — used for security settings.
#[component]
pub fn IShield(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" }
        }
    }
}

/// Trash icon — used for destructive actions.
#[component]
pub fn ITrash(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 6h18" }
            path { d: "M8 6V4h8v2M19 6l-1 14H6L5 6" }
        }
    }
}

/// Cloud icon — used for cloud sync setting.
#[component]
pub fn ICloud(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z" }
        }
    }
}

/// Download icon — used for export backup.
#[component]
pub fn IDownload(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "7 10 12 15 17 10" }
            line { x1: "12", y1: "15", x2: "12", y2: "3" }
        }
    }
}

/// Upload icon — used for import accounts.
#[component]
pub fn IUpload(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" }
            polyline { points: "17 8 12 3 7 8" }
            line { x1: "12", y1: "3", x2: "12", y2: "15" }
        }
    }
}

/// Globe icon — used for language setting.
#[component]
pub fn IGlobe(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "2", y1: "12", x2: "22", y2: "12" }
            path { d: "M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" }
        }
    }
}

/// Bell icon.
#[component]
pub fn IBell(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" }
            path { d: "M13.73 21a2 2 0 0 1-3.46 0" }
        }
    }
}

/// Info icon — used for version row.
#[component]
pub fn IInfo(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}

/// Star icon — used for rate the app row.
#[component]
pub fn IStar(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" }
        }
    }
}

/// Check icon — used for selected language indicator.
#[component]
pub fn ICheck(#[props(default = "w-4 h-4".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

/// Pencil/edit icon — used for rename actions in the context menu.
#[component]
pub fn IEdit(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" }
            path { d: "M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" }
        }
    }
}

/// Arrow pointing up — used for Move Up in the context menu.
#[component]
pub fn IArrowUp(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "19", x2: "12", y2: "5" }
            polyline { points: "5 12 12 5 19 12" }
        }
    }
}

/// Arrow pointing down — used for Move Down in the context menu.
#[component]
pub fn IArrowDown(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            polyline { points: "19 12 12 19 5 12" }
        }
    }
}

/// Folder icon — used for Change Category in the context menu.
#[component]
pub fn IFolder(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" }
        }
    }
}

/// Arrow pointing left — used for the back row in the category picker.
#[component]
pub fn IArrowLeft(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "19", y1: "12", x2: "5", y2: "12" }
            polyline { points: "12 19 5 12 12 5" }
        }
    }
}

/// Plus icon — used for "New category…" row in the category picker.
#[component]
pub fn IPlus(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "12", y1: "5", x2: "12", y2: "19" }
            line { x1: "5", y1: "12", x2: "19", y2: "12" }
        }
    }
}

/// Circle-slash icon — used for "No group" row in the category picker.
#[component]
pub fn ICircleSlash(#[props(default = "w-[15px] h-[15px]".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.75",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "4.93", y1: "4.93", x2: "19.07", y2: "19.07" }
        }
    }
}

/// Gear/cog icon used for the Settings nav item.
#[component]
pub fn ICog() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            class: "w-5 h-5",
            path {
                d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z"
            }
            path {
                d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
            }
        }
    }
}
