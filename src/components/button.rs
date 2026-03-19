//! Generic reusable button component.

use dioxus::prelude::*;

/// Visual style variant for a [`Button`].
#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    /// Full-width accent-colored primary action button.
    Primary,
    /// Compact secondary button (e.g. back navigation).
    Secondary,
    /// Circular accent icon button (e.g. the + add button).
    Round,
}

/// A styled, accessible button with primary, secondary, and round variants.
///
/// # Props
///
/// | Prop       | Type                        | Description                                     |
/// |------------|-----------------------------|-------------------------------------------------|
/// | `label`    | `&'static str`              | Button text.                                    |
/// | `variant`  | [`ButtonVariant`]           | Visual style (`Primary` or `Secondary`).        |
/// | `disabled` | `bool`                      | Disables interaction and applies muted styling. |
/// | `on_click` | `EventHandler<MouseEvent>`  | Click handler (not called when disabled).       |
#[component]
pub fn Button(
    label: &'static str,
    variant: ButtonVariant,
    #[props(default = false)] disabled: bool,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let style = match &variant {
        ButtonVariant::Round => "line-height: 0",
        _ => "",
    };

    let class = match (&variant, disabled) {
        (ButtonVariant::Primary, false) => {
            "w-full py-3 rounded-xl bg-accent text-white text-sm font-semibold cursor-pointer transition-all duration-200"
        }
        (ButtonVariant::Primary, true) => {
            "w-full py-3 rounded-xl bg-[#181818] text-[#2a2a2a] cursor-default"
        }
        (ButtonVariant::Secondary, _) => {
            "bg-surface2 border border-edge rounded-xl px-3 py-1.5 text-[#666] text-sm cursor-pointer"
        }
        (ButtonVariant::Round, _) => {
            "w-9 h-9 rounded-full bg-accent flex items-center justify-center \
             flex-shrink-0 border-none p-0 cursor-pointer text-white text-2xl font-light leading-none"
        }
    };

    rsx! {
        button {
            class: class,
            style: style,
            disabled: disabled,
            onclick: move |e| {
                if !disabled {
                    on_click.call(e);
                }
            },
            { label }
        }
    }
}
