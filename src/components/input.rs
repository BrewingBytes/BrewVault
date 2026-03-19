//! Reusable labeled text input component.

use dioxus::prelude::*;

/// A labeled text input field.
///
/// Renders an uppercase label above a styled input box. Set `mono: true` for
/// fields like secret keys that benefit from a monospace font.
///
/// # Props
///
/// | Prop          | Type              | Description                          |
/// |---------------|-------------------|--------------------------------------|
/// | `label`       | `&'static str`    | Uppercase label shown above input.   |
/// | `placeholder` | `&'static str`    | Placeholder text inside input.       |
/// | `value`       | `Signal<String>`  | Controlled value signal.             |
/// | `mono`        | `bool`            | Use monospace font + wide tracking.  |
#[component]
pub fn Input(
    label: &'static str,
    placeholder: &'static str,
    value: Signal<String>,
    #[props(default = false)] mono: bool,
) -> Element {
    let input_class = if mono {
        "w-full px-3 py-2.5 bg-surface border border-edge rounded-xl text-primary text-sm outline-none placeholder:text-[#252525] font-mono tracking-wide"
    } else {
        "w-full px-3 py-2.5 bg-surface border border-edge rounded-xl text-primary text-sm outline-none placeholder:text-[#252525]"
    };

    rsx! {
        div {
            label {
                class: "text-[10px] font-bold uppercase tracking-wider text-[#2e2e2e] mb-1 block",
                { label }
            }
            input {
                class: input_class,
                r#type: "text",
                placeholder: placeholder,
                value: value(),
                oninput: move |e| value.set(e.value()),
            }
        }
    }
}
