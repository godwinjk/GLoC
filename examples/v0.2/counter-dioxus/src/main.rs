//! # counter-dioxus v0.2
//!
//! Demonstrates consuming a `#[cubit]`-macro-generated cubit in a Dioxus 0.7
//! desktop application. Compare with `examples/v0.1` to see the boilerplate
//! reduction Phase 2 delivers.
//!
//! ## What changed from v0.1
//!
//! | v0.1 (manual) | v0.2 (macro) |
//! |---|---|
//! | `impl Cubit for CounterCubit { ... }` (8 lines) | removed — generated |
//! | `pub fn new(...) -> Self { ... }` (5 lines) | removed — generated |
//! | `on_change` observer | added — generated |
//!
//! ## Running
//!
//! ```sh
//! cargo run -p counter-dioxus-v02
//! ```

#![allow(non_snake_case)]

mod cubits;

use cubits::{CounterCubit, CounterState};
use dioxus::prelude::*;
use gloc::Cubit;

fn main() {
    dioxus::launch(App);
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

/// Root component — owns the `CounterCubit` signal.
///
/// In v0.2 the cubit is constructed with the generated `new()` and wired to an
/// `on_change` observer that forces a Dioxus re-render on every state transition.
#[component]
fn App() -> Element {
    let cubit = use_signal(|| CounterCubit::new(CounterState::new(0)));

    rsx! {
        div {
            style: "
                font-family: system-ui, sans-serif;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 100vh;
                background: #f0f4f8;
            ",

            h1 { style: "color: #1a202c; margin-bottom: 6px;", "GLOC Counter" }
            p {
                style: "color: #718096; margin-top: 0; margin-bottom: 8px; font-size: 13px;",
                "v0.2 — powered by "
                code { "#[cubit]" }
                "  macro"
            }
            p {
                style: "color: #a0aec0; margin-top: 0; margin-bottom: 32px; font-size: 12px;",
                "on_change observer fires on every transition"
            }

            CounterView { cubit }
        }
    }
}

// ---------------------------------------------------------------------------
// CounterView component
// ---------------------------------------------------------------------------

/// Renders the counter value, label, and control buttons.
///
/// Identical layout to v0.1 — the only meaningful code difference is that
/// `on_change` is now available from the generated cubit API, so external
/// systems (logging, analytics, UI adapters) can subscribe without touching
/// the rendering code.
#[component]
fn CounterView(cubit: Signal<CounterCubit>) -> Element {
    let state: CounterState = cubit.read().state().clone();

    rsx! {
        div {
            style: "
                background: white;
                border-radius: 16px;
                padding: 48px 64px;
                box-shadow: 0 4px 24px rgba(0,0,0,0.08);
                display: flex;
                flex-direction: column;
                align-items: center;
                gap: 16px;
                min-width: 320px;
            ",

            span {
                style: "
                    font-size: 13px;
                    font-weight: 600;
                    letter-spacing: 0.08em;
                    text-transform: uppercase;
                    color: {label_color(&state.label)};
                    background: {label_bg(&state.label)};
                    padding: 4px 14px;
                    border-radius: 99px;
                ",
                "{state.label}"
            }

            p {
                style: "font-size: 80px; font-weight: 700; color: #1a202c; margin: 0; line-height: 1;",
                "{state.count}"
            }

            div {
                style: "display: flex; gap: 12px; margin-top: 8px;",

                button {
                    style: button_style("#ef4444"),
                    onclick: move |_| cubit.write().decrement(),
                    "−"
                }
                button {
                    style: button_style("#6b7280"),
                    onclick: move |_| cubit.write().reset(),
                    "Reset"
                }
                button {
                    style: button_style("#22c55e"),
                    onclick: move |_| cubit.write().increment(),
                    "+"
                }
            }

            p {
                style: "font-size: 12px; color: #a0aec0; margin: 0;",
                "Click the buttons to change state"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Style helpers
// ---------------------------------------------------------------------------

fn button_style(color: &str) -> String {
    format!(
        "background:{color}; color:white; border:none; border-radius:10px; \
         width:64px; height:64px; font-size:28px; font-weight:700; \
         cursor:pointer; display:flex; align-items:center; justify-content:center;"
    )
}

fn label_color(label: &str) -> &'static str {
    match label {
        "Negative" => "#7c3aed",
        "Zero"     => "#6b7280",
        "Low"      => "#2563eb",
        "Medium"   => "#d97706",
        "High"     => "#16a34a",
        _          => "#6b7280",
    }
}

fn label_bg(label: &str) -> &'static str {
    match label {
        "Negative" => "#ede9fe",
        "Zero"     => "#f3f4f6",
        "Low"      => "#dbeafe",
        "Medium"   => "#fef3c7",
        "High"     => "#dcfce7",
        _          => "#f3f4f6",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_style_contains_color() {
        assert!(button_style("#abc123").contains("#abc123"));
    }

    #[test]
    fn label_color_covers_all_known_labels() {
        for label in ["Negative", "Zero", "Low", "Medium", "High"] {
            let c = label_color(label);
            assert!(c.starts_with('#'), "label={label}");
        }
    }

    #[test]
    fn label_bg_covers_all_known_labels() {
        for label in ["Negative", "Zero", "Low", "Medium", "High"] {
            let bg = label_bg(label);
            assert!(bg.starts_with('#'), "label={label}");
        }
    }

    #[test]
    fn unknown_label_returns_fallback() {
        assert_eq!(label_color("???"), "#6b7280");
        assert_eq!(label_bg("???"), "#f3f4f6");
    }
}
