//! # counter-dioxus
//!
//! A minimal desktop counter application that demonstrates how to consume a
//! [`gloc::Cubit`] inside a [Dioxus](https://dioxuslabs.com) 0.7 component
//! tree.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │               Dioxus UI layer               │
//! │  App()  ──signal──►  CounterView()          │
//! │    │                     │                  │
//! │    │   read state        │  call methods    │
//! │    └──────────────► CounterCubit ◄──────────┘
//! └─────────────────────────────────────────────┘
//!         (business logic — no UI knowledge)
//! ```
//!
//! The `CounterCubit` lives inside a Dioxus `Signal<CounterCubit>`.
//! Components read state by calling `cubit.read().state()` and mutate it by
//! calling methods on `cubit.write()`. The signal's change-notification system
//! triggers re-renders automatically — GLOC's own `emit()` change-detection
//! prevents redundant writes.
//!
//! ## Running
//!
//! ```sh
//! cargo run -p counter-dioxus
//! ```

#![allow(non_snake_case)]

mod cubits;

use cubits::{CounterCubit, CounterState};
use dioxus::prelude::*;
use gloc::Cubit;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    // `launch` starts the Dioxus desktop event loop and renders `App` as the
    // root component. All platform bootstrapping is handled by Dioxus.
    dioxus::launch(App);
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

/// Root component — owns the `CounterCubit` signal and passes it to children.
///
/// The cubit is stored in a [`Signal`] so that Dioxus tracks reads and writes
/// and schedules re-renders only when state actually changes.
///
/// # Ownership model
///
/// `Signal<CounterCubit>` is created once here and handed to child components
/// as a prop. This mirrors `BlocProvider` from Flutter Bloc: a single cubit
/// instance scoped to a subtree of the component hierarchy.
#[component]
fn App() -> Element {
    // Initialise the cubit starting at zero. The signal holds exclusive
    // ownership; child components receive a copy of the signal handle
    // (cheap — it is reference-counted internally).
    let cubit = use_signal(|| CounterCubit::new(0));

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
                gap: 0;
            ",

            // Title
            h1 {
                style: "color: #1a202c; margin-bottom: 8px;",
                "GLOC Counter"
            }
            p {
                style: "color: #718096; margin-top: 0; margin-bottom: 40px; font-size: 14px;",
                "Powered by gloc::Cubit  ·  Dioxus 0.7 desktop"
            }

            // The actual counter widget — receives the cubit signal as a prop.
            CounterView { cubit }
        }
    }
}

// ---------------------------------------------------------------------------
// CounterView component
// ---------------------------------------------------------------------------

/// Displays the counter value and provides controls to manipulate it.
///
/// # Props
///
/// - `cubit` — a [`Signal`] wrapping the [`CounterCubit`]. Reading
///   `.state()` through the signal registers this component as a subscriber
///   so it re-renders whenever the state changes.
///
/// # Responsibilities
///
/// - **Read** current state via `cubit.read().state()` — never accesses
///   fields directly, always goes through the `Cubit` trait.
/// - **Write** state by calling domain methods on `cubit.write()`, which
///   internally call `emit()` and trigger signal updates.
#[component]
fn CounterView(cubit: Signal<CounterCubit>) -> Element {
    // Snapshot the current state for rendering. Because we read through the
    // signal, Dioxus will re-run this closure whenever the signal changes.
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

            // Label (derived from count magnitude inside the cubit)
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

            // Count display
            p {
                style: "
                    font-size: 80px;
                    font-weight: 700;
                    color: #1a202c;
                    margin: 0;
                    line-height: 1;
                ",
                "{state.count}"
            }

            // Controls row
            div {
                style: "display: flex; gap: 12px; margin-top: 8px;",

                // Decrement button
                button {
                    style: button_style("#ef4444"),
                    onclick: move |_| cubit.write().decrement(),
                    "−"
                }

                // Reset button
                button {
                    style: button_style("#6b7280"),
                    onclick: move |_| cubit.write().reset(),
                    "Reset"
                }

                // Increment button
                button {
                    style: button_style("#22c55e"),
                    onclick: move |_| cubit.write().increment(),
                    "+"
                }
            }

            // Keyboard hint
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

/// Returns an inline CSS string for an action button with the given accent
/// colour. Kept as a pure function so it is trivially testable and reusable
/// without pulling in any Dioxus-specific types.
fn button_style(color: &str) -> String {
    format!(
        "background:{color}; color:white; border:none; border-radius:10px; \
         width:64px; height:64px; font-size:28px; font-weight:700; \
         cursor:pointer; display:flex; align-items:center; \
         justify-content:center; transition:opacity 0.15s;",
    )
}

/// Maps a label string to its pill text colour.
fn label_color(label: &str) -> &'static str {
    match label {
        "Negative" => "#7c3aed",
        "Zero" => "#6b7280",
        "Low" => "#2563eb",
        "Medium" => "#d97706",
        "High" => "#16a34a",
        _ => "#6b7280",
    }
}

/// Maps a label string to its pill background colour.
fn label_bg(label: &str) -> &'static str {
    match label {
        "Negative" => "#ede9fe",
        "Zero" => "#f3f4f6",
        "Low" => "#dbeafe",
        "Medium" => "#fef3c7",
        "High" => "#dcfce7",
        _ => "#f3f4f6",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// button_style always contains the provided colour.
    #[test]
    fn button_style_contains_color() {
        let style = button_style("#ff0000");
        assert!(style.contains("#ff0000"));
    }

    /// label_color returns a non-empty string for every known label.
    #[test]
    fn label_color_returns_value_for_all_known_labels() {
        let labels = ["Negative", "Zero", "Low", "Medium", "High"];
        for label in labels {
            let color = label_color(label);
            assert!(!color.is_empty(), "no color for label: {label}");
            assert!(color.starts_with('#'), "color should be hex: {color}");
        }
    }

    /// label_bg returns a non-empty string for every known label.
    #[test]
    fn label_bg_returns_value_for_all_known_labels() {
        let labels = ["Negative", "Zero", "Low", "Medium", "High"];
        for label in labels {
            let bg = label_bg(label);
            assert!(!bg.is_empty(), "no bg for label: {label}");
            assert!(bg.starts_with('#'), "bg should be hex: {bg}");
        }
    }

    /// Unknown labels fall back to a safe default (no panic).
    #[test]
    fn label_color_unknown_returns_fallback() {
        assert_eq!(label_color("Unknown"), "#6b7280");
    }

    #[test]
    fn label_bg_unknown_returns_fallback() {
        assert_eq!(label_bg("Unknown"), "#f3f4f6");
    }
}
