//! # gloc-dioxus — GLoC state management for Dioxus
//!
//! This crate connects your GLoC reactors to Dioxus components. It lets any
//! component in your app read and update shared state **without passing data
//! through props** — no matter how deep in the component tree it lives.
//!
//! ---
//!
//! ## The problem it solves — prop drilling
//!
//! Without gloc-dioxus you have to pass your reactor as a prop through every
//! component between the root and the one that actually needs it:
//!
//! ```text
//! App (owns counter)
//!  └── Layout (receives counter, doesn't use it)
//!       └── Sidebar (receives counter, doesn't use it)
//!            └── CounterWidget (finally uses counter)
//! ```
//!
//! With gloc-dioxus every component calls `use_gloc` directly — no plumbing:
//!
//! ```text
//! App   ──── use_gloc_provide(|| CounterReactor::new(...))
//!
//! CounterWidget ──── use_gloc::<CounterReactor>()   ← works anywhere in the tree
//! ```
//!
//! ---
//!
//! ## Two things to learn
//!
//! | Hook | Where to call it | What it does |
//! |------|-----------------|--------------|
//! | [`use_gloc_provide`] | App root (or any parent) | Creates the reactor and makes it available |
//! | [`use_gloc`] | Any component | Gets a handle to the reactor |
//!
//! That's the whole API. The handle you get from `use_gloc` is called
//! [`GlocHandle<R>`] and has three methods: `state()`, `update()`, `listen()`.
//!
//! ---
//!
//! ## Step 1 — Define your reactor (normal GLoC, nothing special)
//!
//! ```rust,ignore
//! use gloc::reactor;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! pub struct CounterState {
//!     pub count: i32,
//! }
//!
//! #[reactor(state = CounterState)]
//! pub struct CounterReactor {}
//!
//! impl CounterReactor {
//!     pub fn increment(&mut self) {
//!         self.emit(CounterState { count: self.count + 1 });
//!     }
//!     pub fn decrement(&mut self) {
//!         self.emit(CounterState { count: self.count - 1 });
//!     }
//!     pub fn reset(&mut self) {
//!         self.emit(CounterState { count: 0 });
//!     }
//! }
//! ```
//!
//! Nothing in your reactor needs to know about Dioxus. It stays pure business
//! logic.
//!
//! ---
//!
//! ## Step 2 — Provide it at the app root
//!
//! Call [`use_gloc_provide`] once per reactor type near the top of your
//! component tree. This is a Dioxus hook, so it must be called **at the top
//! of the component function**, before any `rsx!`.
//!
//! ```rust,ignore
//! use gloc_dioxus::use_gloc_provide;
//!
//! #[component]
//! fn App() -> Element {
//!     // Provide as many reactors as you need — one call per type.
//!     use_gloc_provide(|| CounterReactor::new(CounterState { count: 0 }));
//!     use_gloc_provide(|| ThemeReactor::new(Theme::Light));
//!     use_gloc_provide(|| CartReactor::new(CartState::empty()));
//!
//!     rsx! {
//!         Router::<Route> {}
//!     }
//! }
//! ```
//!
//! The closure (`|| CounterReactor::new(...)`) runs **exactly once** — when
//! the `App` component first mounts. On every subsequent re-render Dioxus
//! reuses the reactor that was already created.
//!
//! ---
//!
//! ## Step 3 — Consume it anywhere in the tree
//!
//! Call [`use_gloc`] inside any component that needs the reactor. You do not
//! need to pass anything as a prop.
//!
//! ```rust,ignore
//! use gloc_dioxus::use_gloc;
//!
//! #[component]
//! fn CounterWidget() -> Element {
//!     // Get a handle to CounterReactor from the context tree.
//!     let counter = use_gloc::<CounterReactor>();
//!
//!     // Read state — the component re-renders automatically when this changes.
//!     let count = counter.state().count;
//!
//!     rsx! {
//!         div {
//!             p { "Count: {count}" }
//!             button { onclick: move |_| counter.update(|r| r.increment()), "+" }
//!             button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
//!             button { onclick: move |_| counter.update(|r| r.reset()),     "Reset" }
//!         }
//!     }
//! }
//! ```
//!
//! ---
//!
//! ## Reading state — `handle.state()`
//!
//! ```rust,ignore
//! let counter = use_gloc::<CounterReactor>();
//!
//! let count = counter.state().count;   // i32
//! let label = counter.state().label;   // String
//! ```
//!
//! `state()` returns a **clone** of the current state. Calling it inside the
//! render body makes Dioxus watch the underlying signal — when the state
//! changes, the component re-renders and `state()` returns the new value.
//!
//! You can call `state()` as many times as you like:
//!
//! ```rust,ignore
//! let count     = counter.state().count;
//! let label     = counter.state().label.clone();
//! let is_active = counter.state().count > 0;
//! ```
//!
//! ---
//!
//! ## Mutating state — `handle.update()`
//!
//! ```rust,ignore
//! // Call any method on your reactor inside the closure.
//! counter.update(|r| r.increment());
//! counter.update(|r| r.reset());
//!
//! // Multiple calls in one update are fine too.
//! counter.update(|r| {
//!     r.increment();
//!     r.increment();
//! });
//! ```
//!
//! `update` is typically called inside event handlers:
//!
//! ```rust,ignore
//! button {
//!     onclick: move |_| counter.update(|r| r.increment()),
//!     "+"
//! }
//! ```
//!
//! **What happens under the hood:**
//!
//! ```text
//! counter.update(|r| r.increment())
//!     │
//!     ├─ 1. locks the reactor (Arc<Mutex<R>>)
//!     ├─ 2. calls r.increment()  →  emit(new_state) inside
//!     ├─ 3. GlocStream fires (observers + listen() callbacks)
//!     └─ 4. Dioxus signal updated  →  component re-renders
//! ```
//!
//! Because [`GlocHandle`] is `Copy`, the same handle can be moved into
//! multiple closures without needing `.clone()`:
//!
//! ```rust,ignore
//! let counter = use_gloc::<CounterReactor>();
//!
//! rsx! {
//!     button { onclick: move |_| counter.update(|r| r.increment()), "+" }
//!     button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
//!     button { onclick: move |_| counter.update(|r| r.reset()),     "Reset" }
//!     //       ^^^^^^^ counter is Copy — moved into all three closures, no clone needed
//! }
//! ```
//!
//! ---
//!
//! ## Event-driven dispatch — `fire()`
//!
//! If your reactor uses `neutrons`, pass `fire()` inside `update`:
//!
//! ```rust,ignore
//! #[derive(Debug)]
//! pub enum CounterEvent {
//!     Increment,
//!     Decrement,
//!     AddBy(i32),
//!     Reset,
//! }
//!
//! #[reactor(state = CounterState, neutrons = CounterEvent)]
//! pub struct CounterReactor {}
//!
//! impl CounterReactor {
//!     fn on_event(&mut self, event: CounterEvent) {
//!         match event {
//!             CounterEvent::Increment  => self.emit(CounterState { count: self.count + 1 }),
//!             CounterEvent::Decrement  => self.emit(CounterState { count: self.count - 1 }),
//!             CounterEvent::AddBy(n)   => self.emit(CounterState { count: self.count + n }),
//!             CounterEvent::Reset      => self.emit(CounterState { count: 0 }),
//!         }
//!     }
//! }
//! ```
//!
//! ```rust,ignore
//! let counter = use_gloc::<CounterReactor>();
//!
//! rsx! {
//!     button { onclick: move |_| counter.update(|r| r.fire(CounterEvent::Increment)), "+" }
//!     button { onclick: move |_| counter.update(|r| r.fire(CounterEvent::AddBy(5))), "+5" }
//!     button { onclick: move |_| counter.update(|r| r.fire(CounterEvent::Reset)),    "Reset" }
//! }
//! ```
//!
//! ---
//!
//! ## Multiple reactors in one component
//!
//! You can call `use_gloc` as many times as you need — one call per reactor
//! type. Each is completely independent.
//!
//! ```rust,ignore
//! #[component]
//! fn AppHeader() -> Element {
//!     let theme   = use_gloc::<ThemeReactor>();
//!     let session = use_gloc::<SessionReactor>();
//!
//!     let is_dark  = theme.state() == Theme::Dark;
//!     let username = session.state().username.clone();
//!
//!     rsx! {
//!         header {
//!             span { "Hello, {username}" }
//!             button {
//!                 onclick: move |_| theme.update(|r| r.toggle()),
//!                 if is_dark { "Light mode" } else { "Dark mode" }
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ---
//!
//! ## Shared state across sibling components
//!
//! Every component that calls `use_gloc::<R>()` gets a handle to the
//! **exact same reactor instance**. Mutating it in one component re-renders
//! all other components that read its state.
//!
//! ```text
//! App
//! ├── use_gloc_provide(|| CounterReactor::new(...))   ← one shared reactor
//! │
//! ├── CounterDisplay   ── use_gloc::<CounterReactor>()  ← reads count
//! ├── CounterControls  ── use_gloc::<CounterReactor>()  ← writes count
//! └── CounterHistory   ── use_gloc::<CounterReactor>()  ← reads count
//!
//! When CounterControls calls counter.update(|r| r.increment()):
//!   → CounterDisplay  re-renders (it read state())
//!   → CounterHistory  re-renders (it read state())
//!   → CounterControls re-renders (it read state())
//! ```
//!
//! ---
//!
//! ## Side-effect listeners — `handle.listen()`
//!
//! Use `listen` when you need to react to a transition without re-rendering
//! a component — for example, logging, analytics, or navigation.
//!
//! ```rust,ignore
//! #[component]
//! fn CartPage() -> Element {
//!     let cart = use_gloc::<CartReactor>();
//!
//!     // Register once when the component mounts.
//!     // The closure fires on every real state change.
//!     use_effect(move || {
//!         cart.listen(|old, new| {
//!             if old.item_count < new.item_count {
//!                 println!("Item added — cart now has {} items", new.item_count);
//!             }
//!         });
//!     });
//!
//!     rsx! { /* ... */ }
//! }
//! ```
//!
//! The closure receives **both** the old and new state, so you can act only
//! on specific transitions. It runs synchronously on the UI thread — keep
//! it fast and non-blocking.
//!
//! ---
//!
//! ## Global observer (optional)
//!
//! Set a [`GlocObserver`] once at app startup to log every transition from
//! every reactor — useful for debugging:
//!
//! ```rust,ignore
//! use gloc::{set_observer, GlocObserver};
//!
//! struct Logger;
//!
//! impl GlocObserver for Logger {
//!     fn on_create(&self, name: &str) {
//!         println!("[gloc] created:    {name}");
//!     }
//!     fn on_transition(&self, name: &str, old: &str, new: &str) {
//!         println!("[gloc] transition: {name}  {old} → {new}");
//!     }
//!     fn on_close(&self, name: &str) {
//!         println!("[gloc] closed:     {name}");
//!     }
//! }
//!
//! fn main() {
//!     set_observer(Logger);   // call before dioxus::launch(App)
//!     dioxus::launch(App);
//! }
//! ```
//!
//! ---
//!
//! ## How reactivity works — the simple version
//!
//! Dioxus re-renders a component when a **signal** it read has changed.
//! Internally, gloc-dioxus keeps a `Signal<R::State>` for each reactor.
//!
//! ```text
//! component renders
//!     counter.state()          ← reads Signal<CounterState>
//!                                 Dioxus notes: "this component depends on this signal"
//!
//! user clicks button
//!     counter.update(|r| r.increment())
//!         └─ signal.set(new_state)   ← Dioxus marks component as dirty
//!
//! Dioxus re-renders the component
//!     counter.state()          ← reads the signal again, gets the new value
//! ```
//!
//! You never touch signals directly — `state()` and `update()` handle all of
//! that for you.
//!
//! ---
//!
//! ## Full working example
//!
//! ```rust,ignore
//! use dioxus::prelude::*;
//! use gloc::reactor;
//! use gloc_dioxus::{use_gloc, use_gloc_provide};
//!
//! // --- State ---
//! #[derive(Clone, PartialEq, Debug)]
//! pub struct CounterState { pub count: i32 }
//!
//! // --- Reactor ---
//! #[reactor(state = CounterState)]
//! pub struct CounterReactor {}
//!
//! impl CounterReactor {
//!     pub fn increment(&mut self) {
//!         self.emit(CounterState { count: self.count + 1 });
//!     }
//!     pub fn decrement(&mut self) {
//!         self.emit(CounterState { count: self.count - 1 });
//!     }
//! }
//!
//! // --- App root: provide the reactor ---
//! #[component]
//! fn App() -> Element {
//!     use_gloc_provide(|| CounterReactor::new(CounterState { count: 0 }));
//!     rsx! { Counter {} }
//! }
//!
//! // --- Child: consume the reactor ---
//! #[component]
//! fn Counter() -> Element {
//!     let counter = use_gloc::<CounterReactor>();
//!     let count   = counter.state().count;
//!
//!     rsx! {
//!         div {
//!             h1 { "{count}" }
//!             button { onclick: move |_| counter.update(|r| r.decrement()), "−" }
//!             button { onclick: move |_| counter.update(|r| r.increment()), "+" }
//!         }
//!     }
//! }
//!
//! fn main() {
//!     dioxus::launch(App);
//! }
//! ```
//!
//! [`GlocObserver`]: gloc::GlocObserver

mod context;
mod handle;
mod hook;

pub use handle::GlocHandle;
pub use hook::{use_gloc, use_gloc_provide};
