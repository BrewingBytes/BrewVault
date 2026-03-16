//! Circular SVG progress indicator for TOTP time-remaining display.
//!
//! The [`Ring`] component renders a 32×32 SVG with two concentric circles:
//! a static track and an animated arc driven by a `progress` value (0.0–1.0).
//!
//! # Color behavior
//! - **Normal** (`warn = false`): arc drawn in `var(--color-accent)`
//! - **Warning** (`warn = true`): arc drawn in `var(--color-danger)`
//!
//! A CSS `transition: stroke 0.3s ease` smooths the color change at the threshold.
//!
//! # SVG technique
//! The arc is achieved with `stroke-dasharray` set to the full circumference and
//! `stroke-dashoffset` set to `circumference × (1 − progress)`, so only the
//! leading `progress` fraction of the stroke is painted.  The SVG is rotated
//! −90° so the arc starts from the 12 o'clock position.

use dioxus::prelude::*;

const CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * 13.0; // ≈ 81.681

/// A small circular progress ring.
///
/// Renders a 32×32 SVG ring that fills clockwise from the top according to
/// `progress`.  Intended to be embedded inside a [`TotpCard`] to show how
/// much time remains in the current TOTP window.
///
/// # Props
///
/// | Prop       | Type   | Description                                      |
/// |------------|--------|--------------------------------------------------|
/// | `progress` | `f64`  | Fill fraction in `[0.0, 1.0]`. `1.0` = full ring; `0.0` = empty. |
/// | `warn`     | `bool` | When `true`, draws the arc in `var(--color-danger)` instead of `var(--color-accent)`. |
#[component]
pub fn Ring(
    /// Fill fraction in `[0.0, 1.0]`.  Values outside the range are clamped.
    progress: f64,
    /// When `true`, the arc is drawn in the warning color.
    warn: bool,
) -> Element {
    let progress = progress.clamp(0.0, 1.0);
    let offset = CIRCUMFERENCE * (1.0 - progress);
    let stroke_color = if warn {
        "var(--color-danger)"
    } else {
        "var(--color-accent)"
    };

    rsx! {
        svg {
            width: "32",
            height: "32",
            view_box: "0 0 32 32",
            style: "transform: rotate(-90deg)",
            fill: "none",
            // Track circle — always-visible background ring
            circle {
                cx: "16",
                cy: "16",
                r: "13",
                stroke: "var(--color-edge)",
                stroke_width: "2",
            }
            // Progress arc — length driven by stroke-dashoffset
            circle {
                cx: "16",
                cy: "16",
                r: "13",
                stroke: stroke_color,
                stroke_width: "2",
                stroke_dasharray: format!("{:.3}", CIRCUMFERENCE),
                stroke_dashoffset: format!("{:.3}", offset),
                style: "transition: stroke 0.3s ease",
            }
        }
    }
}
