use dioxus::prelude::*;

/// A settings row with an icon, label, optional sub-label, and a right-slot for children.
#[component]
pub fn SettingRow(
    icon: Element,
    label: String,
    #[props(default = None)] sub_label: Option<String>,
    #[props(default = false)] danger: bool,
    #[props(default = None)] on_click: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let mut hovered = use_signal(|| false);
    let interactive = on_click.is_some();

    let row_bg = if interactive && hovered() {
        "bg-[#141414]"
    } else {
        "bg-transparent"
    };

    let icon_color = if danger { "text-danger" } else { "text-muted" };
    let label_color = if danger {
        "text-danger"
    } else {
        "text-primary"
    };

    rsx! {
        div {
            class: "flex items-center gap-3 px-6 py-3 transition-colors duration-150 {row_bg}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |e| {
                if let Some(handler) = &on_click {
                    handler.call(e);
                }
            },
            // Icon container
            div {
                class: "w-9 h-9 rounded-xl flex-shrink-0 bg-surface2 border border-edge flex items-center justify-center {icon_color}",
                {icon}
            }
            // Text
            div {
                class: "flex-1 min-w-0",
                div { class: "text-sm font-normal {label_color}", "{label}" }
                if let Some(sub) = sub_label {
                    div { class: "text-xs text-[#3a3a3a] mt-px", "{sub}" }
                }
            }
            // Right slot
            div {
                class: "flex-shrink-0",
                {children}
            }
        }
    }
}
