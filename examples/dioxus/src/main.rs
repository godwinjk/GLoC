//! # GLoC Feature Showcase — Dioxus Desktop
//!
//! Demonstrates every major GLoC feature in one app, organised into
//! clearly-labelled sections so each pattern is easy to find.
//!
//! | Section | Reactor | Feature |
//! |---------|---------|---------|
//! | 1 | `CounterReactor` | `#[reactor]` Mode A — direct method calls |
//! | 2 | `EventCounterReactor` | `events = E` — event dispatch |
//! | 3 | `ClickTrackerReactor` | `#[reactor]` Mode B — generated state struct |
//! | 4 | `ThemeReactor` | Enum state + global theme |
//! | 5 | `CartReactor` | Complex state — multiple fields in one `State` |
//!
//! Every reactor registers an `on_change` observer at startup — state
//! transitions print to the terminal as you interact with the UI.

#![allow(non_snake_case)]

mod cubits;

use cubits::{
    CartReactor, CartState, CartStatus, ClickTrackerReactor, ClickTrackerReactorState,
    CounterEvent, CounterReactor, CounterState, EventCounterReactor, Theme, ThemeReactor,
};
use dioxus::prelude::*;
use gloc::Reactor;

fn main() {
    dioxus::launch(App);
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

#[component]
fn App() -> Element {
    // 1. CounterReactor — Mode A, direct methods
    let counter = use_signal(|| {
        let mut r = CounterReactor::new(CounterState::new(0));
        r.on_change(|_old, s| {
            println!(
                "[CounterReactor]      count={:>4}  label={}",
                s.count, s.label
            )
        });
        r
    });

    // 2. EventCounterReactor — event dispatch
    let event_counter = use_signal(|| {
        let mut r = EventCounterReactor::new(CounterState::new(0));
        r.on_change(|_old, s| {
            println!(
                "[EventCounterReactor] count={:>4}  label={}",
                s.count, s.label
            )
        });
        r
    });

    // 3. ClickTrackerReactor — Mode B, generated state
    let tracker = use_signal(|| {
        let mut r = ClickTrackerReactor::new(ClickTrackerReactorState {
            total: 0,
            last_action: String::new(),
        });
        r.on_change(|_old, s| {
            println!(
                "[ClickTrackerReactor] total={:>4}  last={}",
                s.total, s.last_action
            )
        });
        r
    });

    // 4. ThemeReactor — enum state
    let theme = use_signal(|| {
        let mut r = ThemeReactor::new(Theme::Light);
        r.on_change(|_old, s| println!("[ThemeReactor]        theme={:?}", s));
        r
    });

    // 5. CartReactor — complex state
    let cart = use_signal(|| {
        let mut r = CartReactor::new(CartState::empty());
        r.on_change(|_old, s| println!(
            "[CartReactor]         items={:>2}  subtotal={:.2}  discount={:.0}%  total={:.2}  status={:?}",
            s.items.len(), s.subtotal, s.discount * 100.0, s.total, s.status
        ));
        r
    });

    let bg = theme.read().state().background().to_string();
    let text_color = theme.read().state().text_color().to_string();
    let card_bg = theme.read().state().card_background().to_string();

    rsx! {
        div {
            style: "
                font-family: system-ui, -apple-system, sans-serif;
                min-height: 100vh;
                padding: 32px 24px 48px;
                background: {bg};
                color: {text_color};
                transition: background 0.3s, color 0.3s;
                box-sizing: border-box;
            ",

            // ── App header ────────────────────────────────────────────────
            AppHeader { theme, tracker }

            // ── Feature grid — 2 columns ──────────────────────────────────
            div {
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                    gap: 20px;
                    max-width: 860px;
                    margin: 0 auto;
                ",

                // Section 1 — Mode A: direct method calls
                FeatureSection {
                    badge: "Feature 1",
                    title: "#[reactor]  Mode A",
                    subtitle: "counter.increment()  — direct method",
                    card_bg: card_bg.clone(),
                    ModeAView { counter, tracker }
                }

                // Section 2 — Event dispatch
                FeatureSection {
                    badge: "Feature 2",
                    title: "Event Dispatch",
                    subtitle: "reactor.dispatch(event)  — events = E",
                    card_bg: card_bg.clone(),
                    DispatchView { event_counter, tracker }
                }

                // Section 3 — Mode B: generated state struct
                FeatureSection {
                    badge: "Feature 3",
                    title: "#[reactor]  Mode B",
                    subtitle: "#[state] fields  — generated state struct",
                    card_bg: card_bg.clone(),
                    ModeBView { tracker }
                }

                // Section 4 — Enum state
                FeatureSection {
                    badge: "Feature 4",
                    title: "Enum State",
                    subtitle: "enum Theme {{ Light, Dark }}  — any type is a State",
                    card_bg: card_bg.clone(),
                    EnumStateView { theme, tracker }
                }
            }

            // Section 5 — Complex state (full width below the grid)
            div {
                style: "max-width: 860px; margin: 20px auto 0;",
                FeatureSection {
                    badge: "Feature 5",
                    title: "Complex State",
                    subtitle: "CartState {{ items, subtotal, discount, total, status }}",
                    card_bg: card_bg.clone(),
                    CartView { cart }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// App header
// ---------------------------------------------------------------------------

#[component]
fn AppHeader(theme: Signal<ThemeReactor>, tracker: Signal<ClickTrackerReactor>) -> Element {
    let is_dark = *theme.read().state() == Theme::Dark;
    let btn_bg = if is_dark { "#f0f4f8" } else { "#1a202c" };
    let btn_color = if is_dark { "#1a202c" } else { "#f0f4f8" };
    let btn_label = theme.read().state().label().to_string();

    rsx! {
        div {
            style: "
                max-width: 860px;
                margin: 0 auto 28px;
                display: flex;
                align-items: center;
                justify-content: space-between;
                flex-wrap: wrap;
                gap: 12px;
            ",

            div {
                h1 {
                    style: "margin: 0 0 2px; font-size: 22px; font-weight: 800;",
                    "GLoC Feature Showcase"
                }
                p {
                    style: "margin: 0; font-size: 13px; opacity: 0.45;",
                    "State management for any Rust application  \u{00B7}  on_change prints to terminal"
                }
            }

            button {
                style: "
                    padding: 10px 22px; border-radius: 10px; border: none;
                    background: {btn_bg}; color: {btn_color};
                    font-size: 14px; font-weight: 700; cursor: pointer;
                    transition: background 0.3s, color 0.3s; white-space: nowrap;
                ",
                onclick: move |_| {
                    theme.write().toggle();
                    tracker.write().record("theme toggle");
                },
                "🎨  {btn_label}"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Section wrapper — badge chip + title + subtitle + content
// ---------------------------------------------------------------------------

#[component]
fn FeatureSection(
    badge: &'static str,
    title: &'static str,
    subtitle: &'static str,
    card_bg: String,
    children: Element,
) -> Element {
    rsx! {
        div {
            style: "
                background: {card_bg};
                border-radius: 16px;
                padding: 20px 24px 24px;
                box-shadow: 0 4px 20px rgba(0,0,0,0.10);
                display: flex;
                flex-direction: column;
                gap: 16px;
                transition: background 0.3s;
            ",

            // Section header
            div {
                span {
                    style: "
                        display: inline-block;
                        font-size: 10px; font-weight: 800;
                        letter-spacing: 0.10em; text-transform: uppercase;
                        background: rgba(59,130,246,0.15); color: #3b82f6;
                        padding: 2px 8px; border-radius: 99px; margin-bottom: 6px;
                    ",
                    "{badge}"
                }
                p { style: "margin: 0; font-size: 15px; font-weight: 700;", "{title}" }
                p {
                    style: "margin: 2px 0 0; font-size: 11px; opacity: 0.40; font-family: monospace;",
                    "{subtitle}"
                }
            }

            { children }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 1 — Mode A: direct method calls
// ---------------------------------------------------------------------------

#[component]
fn ModeAView(counter: Signal<CounterReactor>, tracker: Signal<ClickTrackerReactor>) -> Element {
    let count = counter.read().state().count;
    let label = counter.read().state().label.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;", "{count}" }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;", "{label}" }

            CodeChip { text: "counter.increment()" }

            div { style: "display: flex; gap: 10px;",
                ActionBtn {
                    color: "#ef4444",
                    onclick: move |_| { counter.write().decrement(); tracker.write().record("mode-a decrement"); },
                    "−"
                }
                ActionBtn {
                    color: "#6b7280",
                    onclick: move |_| { counter.write().reset(); tracker.write().record("mode-a reset"); },
                    "Reset"
                }
                ActionBtn {
                    color: "#22c55e",
                    onclick: move |_| { counter.write().increment(); tracker.write().record("mode-a increment"); },
                    "+"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 2 — Event dispatch
// ---------------------------------------------------------------------------

#[component]
fn DispatchView(
    event_counter: Signal<EventCounterReactor>,
    tracker: Signal<ClickTrackerReactor>,
) -> Element {
    let count = event_counter.read().state().count;
    let label = event_counter.read().state().label.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;", "{count}" }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;", "{label}" }

            CodeChip { text: "reactor.dispatch(CounterEvent::Increment)" }

            div { style: "display: flex; gap: 10px; flex-wrap: wrap; justify-content: center;",
                ActionBtn {
                    color: "#ef4444",
                    onclick: move |_| {
                        event_counter.write().dispatch(CounterEvent::Decrement);
                        tracker.write().record("dispatch Decrement");
                    },
                    "−"
                }
                ActionBtn {
                    color: "#6b7280",
                    onclick: move |_| {
                        event_counter.write().dispatch(CounterEvent::Reset);
                        tracker.write().record("dispatch Reset");
                    },
                    "Reset"
                }
                ActionBtn {
                    color: "#22c55e",
                    onclick: move |_| {
                        event_counter.write().dispatch(CounterEvent::Increment);
                        tracker.write().record("dispatch Increment");
                    },
                    "+"
                }
                ActionBtn {
                    color: "#8b5cf6",
                    onclick: move |_| {
                        event_counter.write().dispatch(CounterEvent::AddBy(5));
                        tracker.write().record("dispatch AddBy(5)");
                    },
                    "+5"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 3 — Mode B: generated state struct
// ---------------------------------------------------------------------------

#[component]
fn ModeBView(tracker: Signal<ClickTrackerReactor>) -> Element {
    let total = tracker.read().state().total;
    let last_action = tracker.read().state().last_action.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 12px;",

            CodeChip { text: "#[state] pub total: u32" }

            div { style: "display: flex; flex-direction: column; gap: 8px; font-size: 14px;",
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "opacity: 0.5;", "Total clicks across app" }
                    span { style: "font-weight: 800; font-size: 28px;", "{total}" }
                }
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { style: "opacity: 0.5;", "Last action" }
                    span {
                        style: "font-weight: 600; font-size: 13px; opacity: 0.7; font-family: monospace;",
                        if last_action.is_empty() { "—" } else { "{last_action}" }
                    }
                }
            }

            p {
                style: "margin: 0; font-size: 11px; opacity: 0.30; text-align: center;",
                "Counts every button press in the app"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 4 — Enum state
// ---------------------------------------------------------------------------

#[component]
fn EnumStateView(theme: Signal<ThemeReactor>, tracker: Signal<ClickTrackerReactor>) -> Element {
    let is_dark = *theme.read().state() == Theme::Dark;
    let btn_label = theme.read().state().label().to_string();
    let icon = if is_dark { "🌙" } else { "☀️" };
    let mode_text = if is_dark { "Dark Mode" } else { "Light Mode" };

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            CodeChip { text: "enum Theme {{ Light, Dark }}" }

            div { style: "font-size: 52px; line-height: 1;", "{icon}" }
            span { style: "font-size: 13px; font-weight: 700; opacity: 0.55;", "{mode_text}" }

            button {
                style: "
                    padding: 10px 24px; border-radius: 10px; border: none;
                    background: rgba(59,130,246,0.15); color: #3b82f6;
                    font-size: 14px; font-weight: 700; cursor: pointer;
                    width: 100%;
                ",
                onclick: move |_| {
                    theme.write().toggle();
                    tracker.write().record("theme toggle");
                },
                "{btn_label}"
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 5 — Complex state (CartReactor, full width)
// ---------------------------------------------------------------------------

#[component]
fn CartView(cart: Signal<CartReactor>) -> Element {
    let items = cart.read().state().items.clone();
    let subtotal = cart.read().state().subtotal;
    let discount = cart.read().state().discount;
    let total = cart.read().state().total;
    let status = cart.read().state().status.clone();
    let locked = matches!(status, CartStatus::CheckedOut);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 14px;",

            // Feature hint + status badge
            div { style: "display: flex; align-items: center; gap: 10px; flex-wrap: wrap;",
                CodeChip { text: "CartState: items, subtotal, discount, total, status" }
                span {
                    style: "font-size: 12px; font-weight: 700; color: {status.color()};
                            background: {status.color()}22; padding: 3px 10px;
                            border-radius: 99px; white-space: nowrap;",
                    "{status.label()}"
                }
            }

            // Add item buttons
            if !locked {
                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    for (name, price) in [("Book", 12.99_f64), ("Pen", 1.49), ("Bag", 24.99)] {
                        button {
                            style: "
                                padding: 7px 14px; border-radius: 8px; border: none;
                                background: #3b82f6; color: white;
                                font-size: 13px; font-weight: 600; cursor: pointer;
                            ",
                            onclick: move |_| cart.write().add_item(name, price),
                            "+ {name}  ${price:.2}"
                        }
                    }
                }
            }

            // Item list
            if items.is_empty() {
                p {
                    style: "margin: 0; opacity: 0.35; font-size: 14px; text-align: center; padding: 8px 0;",
                    "Cart is empty — add some items above"
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 6px;",
                    for (i, item) in items.iter().enumerate() {
                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; font-size: 14px;",
                            span { "{item.name}" }
                            div { style: "display: flex; gap: 10px; align-items: center;",
                                span { style: "opacity: 0.55;", "${item.price:.2}" }
                                if !locked {
                                    button {
                                        style: "border: none; background: #ef444422; color: #ef4444;
                                                border-radius: 6px; padding: 2px 8px; cursor: pointer; font-size: 12px;",
                                        onclick: move |_| cart.write().remove_item(i),
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Discount selector
            if !locked && !items.is_empty() {
                div { style: "display: flex; gap: 8px; align-items: center; flex-wrap: wrap;",
                    span { style: "font-size: 12px; opacity: 0.45;", "Discount:" }
                    for (lbl, pct) in [("None", 0.0_f64), ("10%", 0.1), ("20%", 0.2), ("25%", 0.25)] {
                        {
                            let active     = (discount - pct).abs() < 0.001;
                            let border_col = if active { "#3b82f6" } else { "transparent" };
                            let bg_col     = if active { "#3b82f622" } else { "rgba(128,128,128,0.1)" };
                            rsx! {
                                button {
                                    style: "padding: 4px 10px; border-radius: 6px; font-size: 12px;
                                            font-weight: 600; cursor: pointer; color: inherit;
                                            border: 2px solid {border_col}; background: {bg_col};",
                                    onclick: move |_| cart.write().apply_discount(pct),
                                    "{lbl}"
                                }
                            }
                        }
                    }
                }
            }

            // Totals
            if !items.is_empty() {
                div {
                    style: "border-top: 1px solid rgba(128,128,128,0.18); padding-top: 10px;
                            display: flex; flex-direction: column; gap: 4px;",
                    div { style: "display: flex; justify-content: space-between; font-size: 13px; opacity: 0.55;",
                        span { "Subtotal" }
                        span { "${subtotal:.2}" }
                    }
                    if discount > 0.0 {
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; color: #16a34a;",
                            span { "Discount  ({(discount * 100.0) as u32}%)" }
                            span { "−${(subtotal * discount):.2}" }
                        }
                    }
                    div {
                        style: "display: flex; justify-content: space-between; font-size: 17px; font-weight: 800; margin-top: 4px;",
                        span { "Total" }
                        span { "${total:.2}" }
                    }
                }
            }

            // Actions
            if !items.is_empty() {
                div { style: "display: flex; gap: 8px;",
                    if !locked {
                        button {
                            style: "flex: 1; padding: 10px; border-radius: 8px; border: none;
                                    background: #16a34a; color: white; font-weight: 700;
                                    font-size: 14px; cursor: pointer;",
                            onclick: move |_| cart.write().checkout(),
                            "✓  Checkout"
                        }
                    }
                    button {
                        style: "padding: 10px 16px; border-radius: 8px; border: none;
                                background: rgba(128,128,128,0.12); color: inherit;
                                font-size: 14px; cursor: pointer;",
                        onclick: move |_| cart.write().clear(),
                        "Clear"
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared micro-components
// ---------------------------------------------------------------------------

/// Small monospace pill showing a code snippet — used as a feature hint.
#[component]
fn CodeChip(text: &'static str) -> Element {
    rsx! {
        span {
            style: "
                display: inline-block;
                font-family: monospace;
                font-size: 11px;
                background: rgba(128,128,128,0.12);
                padding: 3px 9px;
                border-radius: 6px;
                opacity: 0.65;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
                max-width: 100%;
            ",
            "{text}"
        }
    }
}

/// Consistent coloured action button used for counter controls.
#[component]
fn ActionBtn(color: &'static str, onclick: EventHandler<MouseEvent>, children: Element) -> Element {
    rsx! {
        button {
            style: "
                background: {color}; color: white; border: none; border-radius: 10px;
                min-width: 52px; height: 52px; padding: 0 14px;
                font-size: 22px; font-weight: 800; cursor: pointer;
            ",
            onclick: move |e| onclick.call(e),
            { children }
        }
    }
}
