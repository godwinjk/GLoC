//! # GLoC Feature Showcase — Dioxus Desktop
//!
//! Demonstrates every major GLoC feature across navigable pages.
//! All reactors are provided **once** in `App` — `use_gloc` works from any
//! page without prop drilling, and state persists across navigation.
//!
//! | Page      | Reactor              | Feature                                           |
//! |-----------|----------------------|---------------------------------------------------|
//! | /counter  | `CounterReactor`     | `gloc_builder!` — rebuilds on every emit          |
//! | /neutrons | `EventCounterReactor`| `gloc_builder!(when:)` — rebuilds only when guard passes |
//! | /theme    | `ThemeReactor`       | `gloc_consumer!(build_when:, listen_when:)` — both guards |
//! | /cart     | `CartReactor`        | `gloc_listener!(when:)` — side effect gated on status |
//! | sidebar   | `ClickTrackerReactor`| Mode B — shared across all pages                  |

#![allow(non_snake_case)]

mod cubits;

use cubits::{
    CartReactor, CartState, CartStatus, ClickTrackerReactor, ClickTrackerReactorState,
    CounterEvent, CounterReactor, CounterState, EventCounterReactor, Theme, ThemeReactor,
};
use dioxus::prelude::*;
use gloc_dioxus::{gloc_builder, gloc_consumer, gloc_listener, use_gloc, use_gloc_provide};

fn main() {
    launch(App);
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/counter")]
    CounterPage {},
    #[route("/neutrons")]
    NeutronsPage {},
    #[route("/theme")]
    ThemePage {},
    #[route("/cart")]
    CartPage {},
}

// ---------------------------------------------------------------------------
// App — provides all reactors, applies global theme
// ---------------------------------------------------------------------------

#[component]
fn App() -> Element {
    // Provided once here — accessible from every page via use_gloc::<R>().
    // State is NOT reset on navigation; it persists for the app's lifetime.
    use_gloc_provide(|| CounterReactor::new(CounterState::new(0)));
    use_gloc_provide(|| EventCounterReactor::new(CounterState::new(0)));
    use_gloc_provide(|| {
        ClickTrackerReactor::new(ClickTrackerReactorState {
            total: 0,
            last_action: String::new(),
        })
    });
    use_gloc_provide(|| ThemeReactor::new(Theme::Light));
    use_gloc_provide(|| CartReactor::new(CartState::default()));

    let theme = use_gloc::<ThemeReactor>();
    let bg = theme.state().background().to_string();
    let text = theme.state().text_color().to_string();

    rsx! {
        div {
            style: "
                font-family: system-ui, -apple-system, sans-serif;
                min-height: 100vh;
                background: {bg};
                color: {text};
                transition: background 0.3s, color 0.3s;
            ",
            Router::<Route> {}
        }
    }
}

// ---------------------------------------------------------------------------
// Layout — sidebar nav + page outlet
// ---------------------------------------------------------------------------

#[component]
fn Layout() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    let is_dark = theme.state() == Theme::Dark;
    let card_bg = theme.state().card_background().to_string();
    let border_color = if is_dark {
        "rgba(255,255,255,0.07)"
    } else {
        "rgba(0,0,0,0.08)"
    };
    let btn_bg = if is_dark { "#f0f4f8" } else { "#1a202c" };
    let btn_color = if is_dark { "#1a202c" } else { "#f0f4f8" };
    let btn_label = theme.state().label().to_string();
    let total = tracker.state().total;
    let last_action = tracker.state().last_action.clone();

    rsx! {
        div { style: "display: flex; min-height: 100vh;",

            // ── Sidebar ───────────────────────────────────────────────────
            nav {
                style: "
                    width: 220px; flex-shrink: 0; min-height: 100vh;
                    background: {card_bg};
                    border-right: 1px solid {border_color};
                    display: flex; flex-direction: column;
                    padding: 24px 14px;
                    box-sizing: border-box;
                    transition: background 0.3s;
                ",

                // Brand
                div { style: "margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid {border_color};",
                    p { style: "margin: 0; font-size: 18px; font-weight: 800; letter-spacing: -0.02em;", "GLoC" }
                    p { style: "margin: 2px 0 0; font-size: 10px; opacity: 0.35; font-family: monospace;", "feature showcase" }
                }

                // Nav links
                p { style: "margin: 0 0 6px 4px; font-size: 9px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; opacity: 0.35;", "Pages" }
                NavLink { to: Route::Home {},        label: "🏠  Home" }
                NavLink { to: Route::CounterPage {},  label: "🔢  Counter" }
                NavLink { to: Route::NeutronsPage {}, label: "⚛️   Neutrons" }
                NavLink { to: Route::ThemePage {},    label: "🎨  Theme" }
                NavLink { to: Route::CartPage {},     label: "🛒  Cart" }

                div { style: "flex: 1;" }

                // ── Feature 3: ClickTrackerReactor ─────────────────────────
                // This widget is always visible — it proves that
                // ClickTrackerReactor is shared across every page.
                // Every button press on any page increments this counter.
                div {
                    style: "
                        border-radius: 10px; padding: 12px; margin-bottom: 12px;
                        background: rgba(59,130,246,0.08);
                        border: 1px solid rgba(59,130,246,0.18);
                    ",
                    p { style: "margin: 0 0 6px; font-size: 9px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.12em; opacity: 0.45;", "Click Tracker" }
                    p { style: "margin: 0; font-size: 30px; font-weight: 800; line-height: 1;", "{total}" }
                    p {
                        style: "margin: 4px 0 0; font-size: 10px; opacity: 0.38; font-family: monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        { if last_action.is_empty() { "—".to_string() } else { last_action } }
                    }
                    p { style: "margin: 6px 0 0; font-size: 9px; opacity: 0.25;", "#[reactor] Mode B · all pages" }
                }

                // Theme toggle
                button {
                    style: "
                        width: 100%; padding: 9px; border-radius: 8px; border: none;
                        background: {btn_bg}; color: {btn_color};
                        font-size: 12px; font-weight: 700; cursor: pointer;
                        transition: background 0.3s, color 0.3s;
                    ",
                    onclick: move |_| {
                        theme.update(|r| r.toggle());
                        tracker.update(|r| r.record("theme toggle"));
                    },
                    "{btn_label}"
                }
            }

            // ── Page content ──────────────────────────────────────────────
            div {
                style: "flex: 1; padding: 32px 28px; min-width: 0; box-sizing: border-box;",
                Outlet::<Route> {}
            }
        }
    }
}

/// Sidebar navigation link.
#[component]
fn NavLink(to: Route, label: String) -> Element {
    rsx! {
        Link {
            to,
            style: "
                display: block; padding: 8px 10px; border-radius: 8px;
                text-decoration: none; color: inherit;
                font-size: 13px; font-weight: 600; margin-bottom: 2px;
                opacity: 0.7;
            ",
            "{label}"
        }
    }
}

// ---------------------------------------------------------------------------
// Home page
// ---------------------------------------------------------------------------

#[component]
fn Home() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();

    rsx! {
        div { style: "max-width: 600px;",

            h1 { style: "margin: 0 0 8px; font-size: 26px; font-weight: 800;", "GLoC Feature Showcase" }
            p {
                style: "margin: 0 0 28px; font-size: 14px; opacity: 0.5; line-height: 1.65;",
                "GLoC Reactor- Universal Rust state management "
                "Navigate between pages using the sidebar. All reactors are provided once at the app root."
            }

            // Key concept callout
            div {
                style: "
                    background: {card_bg}; border-radius: 14px;
                    padding: 20px 24px; margin-bottom: 24px;
                    box-shadow: 0 4px 20px rgba(0,0,0,0.08);
                    border-left: 3px solid #3b82f6;
                ",
                p { style: "margin: 0 0 8px; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.1em; color: #3b82f6;", "Key concept" }
                p {
                    style: "margin: 0; font-size: 13px; line-height: 1.65; opacity: 0.7;",
                    "All five reactors are provided via "
                    code { style: "font-family: monospace; background: rgba(59,130,246,0.12); padding: 1px 5px; border-radius: 4px;", "use_gloc_provide" }
                    " in "
                    code { style: "font-family: monospace; background: rgba(59,130,246,0.12); padding: 1px 5px; border-radius: 4px;", "App()" }
                    " — once, at startup. Any page calls "
                    code { style: "font-family: monospace; background: rgba(59,130,246,0.12); padding: 1px 5px; border-radius: 4px;", "use_gloc::<R>()" }
                    " to consume them. "
                    "Navigate away and back — the counter keeps its value."
                }
            }

            // Feature index
            div { style: "display: flex; flex-direction: column; gap: 10px;",
                FeatureRow { badge: "Counter",  path: "/counter",  desc: "gloc_builder! — rebuilds on every emit (no guard)" }
                FeatureRow { badge: "Neutrons", path: "/neutrons", desc: "gloc_builder!(when:) — rebuild guard + neutron dispatch" }
                FeatureRow { badge: "Theme",    path: "/theme",    desc: "gloc_consumer!(build_when:, listen_when:) — both guards" }
                FeatureRow { badge: "Cart",     path: "/cart",     desc: "gloc_listener!(when:) — side effect gated on status transition" }
                FeatureRow { badge: "Sidebar",  path: "/",         desc: "#[reactor] Mode B — ClickTracker shared across all pages" }
            }
        }
    }
}

#[component]
fn FeatureRow(badge: &'static str, path: &'static str, desc: &'static str) -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();

    rsx! {
        div {
            style: "
                display: flex; align-items: center; gap: 12px;
                background: {card_bg}; border-radius: 10px; padding: 12px 16px;
                box-shadow: 0 2px 8px rgba(0,0,0,0.06);
            ",
            span {
                style: "
                    font-size: 10px; font-weight: 800; letter-spacing: 0.1em;
                    text-transform: uppercase; white-space: nowrap;
                    background: rgba(59,130,246,0.12); color: #3b82f6;
                    padding: 3px 9px; border-radius: 99px; flex-shrink: 0;
                ",
                "{badge}"
            }
            span { style: "font-size: 13px; opacity: 0.6;", "{desc}" }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature pages — each reuses existing view components
// ---------------------------------------------------------------------------

/// Feature 1 — CounterReactor with gloc_builder!
#[component]
fn CounterPage() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();
    rsx! {
        div { style: "max-width: 420px;",
            FeatureSection {
                badge: "Feature 1",
                title: "gloc_builder!",
                subtitle: "builder re-runs on every emit — BlocBuilder",
                card_bg,
                ModeAView {}
            }
        }
    }
}

/// Feature 2 — EventCounterReactor: neutron firing + gloc_builder!(when:)
#[component]
fn NeutronsPage() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();
    rsx! {
        div { style: "max-width: 420px;",
            FeatureSection {
                badge: "Feature 2",
                title: "Neutron Firing + build_when",
                subtitle: "gloc_builder!(when: ...) — rebuilds only when guard passes. \
                For testing no. 3 will not be visible",
                card_bg,
                DispatchView {}
            }
        }
    }
}

/// Feature 4 — ThemeReactor with gloc_consumer! + both guards
#[component]
fn ThemePage() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();
    rsx! {
        div { style: "max-width: 420px;",
            FeatureSection {
                badge: "Feature 4",
                title: "gloc_consumer! with guards",
                subtitle: "build_when + listen_when — BlocConsumer",
                card_bg,
                EnumStateView {}
            }
        }
    }
}

/// Feature 5 — CartReactor with gloc_listener!(when:)
#[component]
fn CartPage() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let card_bg = theme.state().card_background().to_string();
    rsx! {
        div { style: "max-width: 560px;",
            FeatureSection {
                badge: "Feature 5",
                title: "Complex State + gloc_listener!(when:)",
                subtitle: "listener gated on status transition — BlocListener(listenWhen:)",
                card_bg,
                CartView {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Feature 1 — gloc_builder! (CounterReactor)
// ---------------------------------------------------------------------------

#[component]
fn ModeAView() -> Element {
    let counter = use_gloc::<CounterReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    gloc_builder!(CounterReactor, |state| rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;",
                "{state.count}"
            }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;",
                "{state.label}"
            }

            CodeChip { text: "gloc_builder!(CounterReactor, |state| rsx! {{ ... }})" }

            div { style: "display: flex; gap: 10px;",
                ActionBtn {
                    color: "#ef4444",
                    onclick: move |_| {
                        counter.update(|r| r.decrement());
                        tracker.update(|r| r.record("counter decrement"));
                    },
                    "−"
                }
                ActionBtn {
                    color: "#6b7280",
                    onclick: move |_| {
                        counter.update(|r| r.reset());
                        tracker.update(|r| r.record("counter reset"));
                    },
                    "Reset"
                }
                ActionBtn {
                    color: "#22c55e",
                    onclick: move |_| {
                        counter.update(|r| r.increment());
                        tracker.update(|r| r.record("counter increment"));
                    },
                    "+"
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Feature 2 — neutron firing (EventCounterReactor)
// ---------------------------------------------------------------------------

#[component]
fn DispatchView() -> Element {
    let event_counter = use_gloc::<EventCounterReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    gloc_builder!(EventCounterReactor,
        when: |_, new|  new.count != 3,
        |state| rsx! {
        div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",

            p { style: "font-size: 64px; font-weight: 800; margin: 0; line-height: 1;",
                "{state.count}"
            }
            span { style: "font-size: 12px; opacity: 0.45; font-weight: 600;",
                "{state.label}"
            }

            CodeChip { text: "gloc_builder!(when: |old, new| old.count != new.count, ...)" }

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
    })
}

// ---------------------------------------------------------------------------
// Feature 4 — gloc_consumer! (ThemeReactor)
// ---------------------------------------------------------------------------

#[component]
fn EnumStateView() -> Element {
    let theme = use_gloc::<ThemeReactor>();
    let tracker = use_gloc::<ClickTrackerReactor>();

    gloc_consumer!(ThemeReactor,
        build_when:  |old, new| old != new,
        build: |state| rsx! {
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 14px;",
                CodeChip { text: "gloc_consumer!(build_when:, listen_when:, ...)" }
                div { style: "font-size: 52px; line-height: 1;",
                    { if *state == Theme::Dark { "🌙" } else { "☀️" } }
                }
                span { style: "font-size: 13px; font-weight: 700; opacity: 0.55;",
                    { if *state == Theme::Dark { "Dark Mode" } else { "Light Mode" } }
                }
                button {
                    style: "
                        padding: 10px 24px; border-radius: 10px; border: none;
                        background: rgba(59,130,246,0.15); color: #3b82f6;
                        font-size: 14px; font-weight: 700; cursor: pointer; width: 100%;
                    ",
                    onclick: move |_| {
                        theme.update(|r| r.toggle());
                        tracker.update(|r| r.record("theme toggle"));
                    },
                    { state.label().to_string() }
                }
            }
        },
        listen_when: |old, new| old != new,
        listen: |old, new| { println!("[theme] {old:?} → {new:?}"); }
    )
}

// ---------------------------------------------------------------------------
// Feature 5 — CartReactor with gloc_listener!
// ---------------------------------------------------------------------------

#[component]
fn CartView() -> Element {
    let cart = use_gloc::<CartReactor>();

    gloc_listener!(CartReactor,
        when: |old, new| !matches!(old.status, CartStatus::CheckedOut) && matches!(new.status, CartStatus::CheckedOut),
        |_old, new| { println!("[cart] checked out — total: ${:.2}", new.total); }
    );

    let items = cart.state().items.clone();
    let subtotal = cart.state().subtotal;
    let discount = cart.state().discount;
    let total = cart.state().total;
    let status = cart.state().status.clone();
    let locked = matches!(status, CartStatus::CheckedOut);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 14px;",

            div { style: "display: flex; align-items: center; gap: 10px; flex-wrap: wrap;",
                CodeChip { text: "gloc_listener!(CartReactor, |old, new| {{ ... }})" }
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
// Shared components
// ---------------------------------------------------------------------------

/// Section wrapper — badge chip + title + subtitle + content slot.
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

/// Small monospace pill showing a code snippet.
#[component]
fn CodeChip(text: &'static str) -> Element {
    rsx! {
        span {
            style: "
                display: inline-block; font-family: monospace; font-size: 11px;
                background: rgba(128,128,128,0.12); padding: 3px 9px;
                border-radius: 6px; opacity: 0.65; white-space: nowrap;
                overflow: hidden; text-overflow: ellipsis; max-width: 100%;
            ",
            "{text}"
        }
    }
}

/// Coloured action button used for counter controls.
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
