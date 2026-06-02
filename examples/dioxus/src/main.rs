//! # GLoC Feature Showcase — Dioxus Desktop
//!
//! Demonstrates every major GLoC feature in one app, organised into
//! clearly-labelled sections so each pattern is easy to find.
//!
//! | Section | Reactor | Feature |
//! |---------|---------|---------|
//! | 1 | `CounterReactor` | `#[reactor]` Mode A — direct method calls |
//! | 2 | `EventCounterReactor` | `neutrons = N` — neutron firing via `fire()` |
//! | 3 | `ClickTrackerReactor` | `#[reactor]` Mode B — generated state struct |
//! | 4 | `ThemeReactor` | Enum state + global theme |
//! | 5 | `CartReactor` | Complex state — multiple fields in one `State` |
//!
//! All reactors are provided via `use_gloc_provide` in `App` and consumed
//! with `use_gloc::<R>()` in each child component — no prop drilling.

#![allow(non_snake_case)]

mod cubits;

use cubits::{
    CartReactor, CartState, CartStatus, ClickTrackerReactor, ClickTrackerReactorState,
    CounterEvent, CounterReactor, CounterState, EventCounterReactor, Theme, ThemeReactor,
};
use dioxus::prelude::*;
use gloc_dioxus::{use_gloc, use_gloc_provide};

fn main() {
    dioxus::launch(App);
}

// ---------------------------------------------------------------------------
// Root component — provides all reactors into the Dioxus context tree
// ---------------------------------------------------------------------------

#[component]
fn App() -> Element {
    // Each call injects one reactor type into the component tree.
    // Descendants call use_gloc::<R>() to consume — no prop drilling.
    use_gloc_provide(|| CounterReactor::new(CounterState::new(0)));
    use_gloc_provide(|| EventCounterReactor::new(CounterState::new(0)));
    use_gloc_provide(|| {
        ClickTrackerReactor::new(ClickTrackerReactorState {
            total: 0,
            last_action: String::new(),
        })
    });
    use_gloc_provide(|| ThemeReactor::new(Theme::Light));
    use_gloc_provide(|| CartReactor::new(CartState::empty()));

    let theme = use_gloc::<ThemeReactor>();
    let bg = theme.state().background().to_string();
    let text_color = theme.state().text_color().to_string();
    let card_bg = theme.state().card_background().to_string();

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
            AppHeader {}

            // ── Feature grid — 2 columns ──────────────────────────────────
            div {
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
                    gap: 20px;
                    max-width: 860px;
                    margin: 0 auto;
                ",

                FeatureSection {
                    badge: "Feature 1",
                    title: "#[reactor]  Mode A",
                    subtitle: "counter.increment()  — direct method",
                    card_bg: card_bg.clone(),
                    ModeAView {}
                }

                FeatureSection {
                    badge: "Feature 2",
                    title: "Neutron Firing",
                    subtitle: "reactor.fire(neutron)  — neutrons = N",
                    card_bg: card_bg.clone(),
                    DispatchView {}
                }

                FeatureSection {
                    badge: "Feature 3",
                    title: "#[reactor]  Mode B",
                    subtitle: "#[state] fields  — generated state struct",
                    card_bg: card_bg.clone(),
                    ModeBView {}
                }

                FeatureSection {
                    badge: "Feature 4",
                    title: "Enum State",
                    subtitle: "enum Theme {{ Light, Dark }}  — any type is a State",
                    card_bg: card_bg.clone(),
                    EnumStateView {}
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
                    CartView {}
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// App header — consumes ThemeReactor and ClickTrackerReactor directly
// ---------------------------------------------------------------------------

#[component]
fn AppHeader() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    let is_dark = theme.state() == Theme::Dark;
    let btn_bg = if is_dark { "#f0f4f8" } else { "#1a202c" };
    let btn_color = if is_dark { "#1a202c" } else { "#f0f4f8" };
    let btn_label = theme.state().label().to_string();

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
                    "State management for any Rust application  \u{00B7}  use_gloc — no prop drilling"
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
                    theme.update(|r| r.toggle());
                    tracker.update(|r| r.record("theme toggle"));
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
fn ModeAView() -> Element {
    let counter = use_gloc::<CounterReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    let count = counter.state().count;
    let label = counter.state().label.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;", "{count}" }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;", "{label}" }

            CodeChip { text: "counter.update(|r| r.increment())" }

            div { style: "display: flex; gap: 10px;",
                ActionBtn {
                    color: "#ef4444",
                    onclick: move |_| {
                        counter.update(|r| r.decrement());
                        tracker.update(|r| r.record("mode-a decrement"));
                    },
                    "−"
                }
                ActionBtn {
                    color: "#6b7280",
                    onclick: move |_| {
                        counter.update(|r| r.reset());
                        tracker.update(|r| r.record("mode-a reset"));
                    },
                    "Reset"
                }
                ActionBtn {
                    color: "#22c55e",
                    onclick: move |_| {
                        counter.update(|r| r.increment());
                        tracker.update(|r| r.record("mode-a increment"));
                    },
                    "+"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 2 — Neutron firing
// ---------------------------------------------------------------------------

#[component]
fn DispatchView() -> Element {
    let event_counter = use_gloc::<EventCounterReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    let count = event_counter.state().count;
    let label = event_counter.state().label.clone();

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;", "{count}" }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;", "{label}" }

            CodeChip { text: "counter.update(|r| r.fire(CounterEvent::Increment))" }

            div { style: "display: flex; gap: 10px; flex-wrap: wrap; justify-content: center;",
                ActionBtn {
                    color: "#ef4444",
                    onclick: move |_| {
                        event_counter.update(|r| r.fire(CounterEvent::Decrement));
                        tracker.update(|r| r.record("fire Decrement"));
                    },
                    "−"
                }
                ActionBtn {
                    color: "#6b7280",
                    onclick: move |_| {
                        event_counter.update(|r| r.fire(CounterEvent::Reset));
                        tracker.update(|r| r.record("fire Reset"));
                    },
                    "Reset"
                }
                ActionBtn {
                    color: "#22c55e",
                    onclick: move |_| {
                        event_counter.update(|r| r.fire(CounterEvent::Increment));
                        tracker.update(|r| r.record("fire Increment"));
                    },
                    "+"
                }
                ActionBtn {
                    color: "#8b5cf6",
                    onclick: move |_| {
                        event_counter.update(|r| r.fire(CounterEvent::AddBy(5)));
                        tracker.update(|r| r.record("fire AddBy(5)"));
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
fn ModeBView() -> Element {
    let tracker = use_gloc::<ClickTrackerReactor>();

    let total = tracker.state().total;
    let last_action = tracker.state().last_action.clone();

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
fn EnumStateView() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    let is_dark = theme.state() == Theme::Dark;
    let btn_label = theme.state().label().to_string();
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
                    theme.update(|r| r.toggle());
                    tracker.update(|r| r.record("theme toggle"));
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
fn CartView() -> Element {
    let cart = use_gloc::<CartReactor>();

    let items = cart.state().items.clone();
    let subtotal = cart.state().subtotal;
    let discount = cart.state().discount;
    let total = cart.state().total;
    let status = cart.state().status.clone();
    let locked = matches!(status, CartStatus::CheckedOut);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 14px;",

            div { style: "display: flex; align-items: center; gap: 10px; flex-wrap: wrap;",
                CodeChip { text: "CartState: items, subtotal, discount, total, status" }
                span {
                    style: "font-size: 12px; font-weight: 700; color: {status.color()};
                            background: {status.color()}22; padding: 3px 10px;
                            border-radius: 99px; white-space: nowrap;",
                    "{status.label()}"
                }
            }

            if !locked {
                div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                    for (name, price) in [("Book", 12.99_f64), ("Pen", 1.49), ("Bag", 24.99)] {
                        button {
                            style: "
                                padding: 7px 14px; border-radius: 8px; border: none;
                                background: #3b82f6; color: white;
                                font-size: 13px; font-weight: 600; cursor: pointer;
                            ",
                            onclick: move |_| cart.update(|r| r.add_item(name, price)),
                            "+ {name}  ${price:.2}"
                        }
                    }
                }
            }

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
                                        onclick: move |_| cart.update(|r| r.remove_item(i)),
                                        "✕"
                                    }
                                }
                            }
                        }
                    }
                }
            }

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
                                    onclick: move |_| cart.update(|r| r.apply_discount(pct)),
                                    "{lbl}"
                                }
                            }
                        }
                    }
                }
            }

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

            if !items.is_empty() {
                div { style: "display: flex; gap: 8px;",
                    if !locked {
                        button {
                            style: "flex: 1; padding: 10px; border-radius: 8px; border: none;
                                    background: #16a34a; color: white; font-weight: 700;
                                    font-size: 14px; cursor: pointer;",
                            onclick: move |_| cart.update(|r| r.checkout()),
                            "✓  Checkout"
                        }
                    }
                    button {
                        style: "padding: 10px 16px; border-radius: 8px; border: none;
                                background: rgba(128,128,128,0.12); color: inherit;
                                font-size: 14px; cursor: pointer;",
                        onclick: move |_| cart.update(|r| r.clear()),
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
