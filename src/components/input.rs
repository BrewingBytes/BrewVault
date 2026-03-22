//! Reusable labeled text input component.

use dioxus::prelude::*;

use crate::components::icons::{IEye, IEyeOff};

/// A labeled text input field.
///
/// Renders an uppercase label above a styled input box.
///
/// # Props
///
/// | Prop          | Type                          | Description                                      |
/// |---------------|-------------------------------|--------------------------------------------------|
/// | `label`       | `&'static str`                | Uppercase label shown above input.               |
/// | `placeholder` | `&'static str`                | Placeholder text inside input.                   |
/// | `value`       | `Signal<String>`              | Controlled value signal.                         |
/// | `mono`        | `bool`                        | Use monospace font + wide tracking.              |
/// | `password`    | `bool`                        | Render as password field with show/hide toggle.  |
/// | `autofocus`   | `bool`                        | Focus this field on mount.                       |
/// | `onkeydown`   | `EventHandler<KeyboardEvent>` | Optional keydown handler (pass `|_| {}` to ignore). |
#[component]
pub fn Input(
    label: &'static str,
    placeholder: &'static str,
    value: Signal<String>,
    #[props(default = false)] mono: bool,
    #[props(default = false)] password: bool,
    #[props(default = false)] autofocus: bool,
    #[props(default)] onkeydown: EventHandler<KeyboardEvent>,
) -> Element {
    let mut show = use_signal(|| false);

    let input_type = if password && !show() {
        "password"
    } else {
        "text"
    };

    let pr = if password { "pr-9" } else { "pr-3" };
    let base = format!(
        "w-full pl-3 {pr} py-[9px] bg-surface2 border border-edge rounded-[10px] text-primary text-sm outline-none placeholder:text-[#252525] transition-colors"
    );
    let input_class = if mono {
        format!("{base} font-mono tracking-wide")
    } else {
        base
    };

    rsx! {
        div {
            label {
                class: "text-[10px] font-bold uppercase tracking-wider text-[#2e2e2e] mb-1 block",
                { label }
            }
            div { class: "relative",
                input {
                    class: "{input_class}",
                    r#type: input_type,
                    placeholder: placeholder,
                    autofocus: autofocus,
                    value: value(),
                    oninput: move |e| value.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| onkeydown.call(e),
                }
                if password {
                    button {
                        class: "absolute right-2.5 top-1/2 -translate-y-1/2 text-muted hover:text-primary transition-colors cursor-pointer bg-transparent border-none p-0.5",
                        r#type: "button",
                        aria_label: if show() { "Hide password" } else { "Show password" },
                        tabindex: 0,
                        onclick: move |e| {
                            e.stop_propagation();
                            show.set(!show());
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            // Enter/Space toggle visibility but must NOT submit the form
                            if e.key() == Key::Enter || e.key() == Key::Character(" ".to_string()) {
                                e.prevent_default();
                                show.set(!show());
                            }
                        },
                        if show() {
                            IEyeOff { class: "w-4 h-4" }
                        } else {
                            IEye { class: "w-4 h-4" }
                        }
                    }
                }
            }
        }
    }
}
