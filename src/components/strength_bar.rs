use dioxus::prelude::*;

/// Password strength level computed from character count.
#[derive(Clone, Copy, PartialEq)]
pub enum Strength {
    Weak,
    Fair,
    Strong,
}

impl Strength {
    pub fn from_password(pw: &str) -> Option<Self> {
        if pw.is_empty() {
            None
        } else if pw.len() < 8 {
            Some(Strength::Weak)
        } else if pw.len() < 12 {
            Some(Strength::Fair)
        } else {
            Some(Strength::Strong)
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Strength::Weak => "WEAK",
            Strength::Fair => "FAIR",
            Strength::Strong => "STRONG",
        }
    }

    pub fn label_color(self) -> &'static str {
        match self {
            Strength::Weak => "text-danger",
            Strength::Fair => "text-warn",
            Strength::Strong => "text-accent",
        }
    }
}

/// Three-segment password strength bar with a text label.
#[component]
pub fn StrengthBar(password: String) -> Element {
    let strength = Strength::from_password(&password);

    let Some(s) = strength else {
        return rsx! { div { class: "h-[3px]" } };
    };

    let seg1 = match s {
        Strength::Weak => "bg-danger",
        _ => "bg-accent",
    };
    let seg2 = match s {
        Strength::Weak => "bg-edge",
        Strength::Fair => "bg-warn",
        Strength::Strong => "bg-accent",
    };
    let seg3 = match s {
        Strength::Strong => "bg-accent",
        _ => "bg-edge",
    };
    let lc = s.label_color();

    rsx! {
        div { class: "mt-1 flex flex-col gap-1",
            div { class: "flex gap-1",
                div { class: "h-[3px] flex-1 rounded-full {seg1}" }
                div { class: "h-[3px] flex-1 rounded-full {seg2}" }
                div { class: "h-[3px] flex-1 rounded-full {seg3}" }
            }
            span { class: "text-[10px] font-semibold tracking-wider {lc}",
                "{s.label()}"
            }
        }
    }
}
