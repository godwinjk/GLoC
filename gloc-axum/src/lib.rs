//! Axum adapter for GLoC reactors.
//!
//! This crate eliminates the boilerplate of wiring a [`Reactor`] into an Axum
//! application. Axum requires `State<T>` where `T: Clone + Send + Sync + 'static`.
//! [`GlocProvider<R>`] already satisfies those bounds, but manually constructing
//! the `Arc<Mutex<R>>` + [`GlocStream`] pair on every project creates repetitive
//! setup. [`new_axum_state`] and [`AxumReactor`] collapse that into a single call.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use gloc::Reactor;
//! use gloc_axum::{new_axum_state, AxumReactor};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { pub count: i32 }
//!
//! struct CounterReactor { state: CounterState }
//!
//! impl CounterReactor {
//!     fn new() -> Self { Self { state: CounterState { count: 0 } } }
//!     fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//! }
//!
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if &next != self.state() { self.state = next; }
//!     }
//! }
//!
//! // One call — ready to pass to Router::with_state()
//! let state = new_axum_state(CounterReactor::new());
//! // or: let state = AxumReactor::new(CounterReactor::new());
//! ```

use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use gloc::provider::GlocProvider;
use gloc::stream::GlocStream;
use gloc::Reactor;

/// Wraps a reactor into a [`GlocProvider`] that satisfies Axum's
/// `Clone + Send + Sync + 'static` requirements for shared state.
///
/// This is the free-function form of [`AxumReactor::new`]. Prefer it when
/// you want to store the consumer as-is in `Router::with_state()` without
/// going through a newtype.
///
/// # Type parameters
///
/// - `R` — the concrete reactor type. Must be `Send + 'static` so Axum can
///   move the state into request-handler tasks safely.
///
/// # Example
///
/// ```rust,no_run
/// # use gloc::Reactor;
/// # use gloc_axum::new_axum_state;
/// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
/// # struct R { s: S }
/// # impl R { fn new() -> Self { Self { s: S(0) } } }
/// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if &s != &self.s { self.s = s; } } }
/// let state = new_axum_state(R::new());
/// // pass `state` to axum::Router::with_state(state)
/// ```
pub fn new_axum_state<R>(reactor: R) -> GlocProvider<R>
where
    R: Reactor + Send + 'static,
    R::State: Clone + PartialEq + Debug + Send + 'static,
{
    let initial = reactor.state().clone();
    let stream = GlocStream::new(initial);
    let shared = Arc::new(Mutex::new(reactor));
    GlocProvider::new(shared, stream)
}

/// A newtype around [`GlocProvider<R>`] that makes its Axum-compatibility
/// explicit at the type level.
///
/// Use `AxumReactor<R>` when you want the type signature of your handler
/// parameters to clearly communicate that this state was created for Axum,
/// rather than exposing the generic `GlocProvider` directly. Both forms are
/// functionally identical; choose whichever reads better for your project.
///
/// # Clone behaviour
///
/// Cloning is free — it only increments `Arc` reference counts on the shared
/// reactor and stream. Axum clones the state for each request handler.
///
/// # Deref
///
/// `AxumReactor<R>` derefs to `GlocProvider<R>`, so all consumer methods
/// (`state()`, `update()`, `listen()`) are available without unwrapping.
///
/// # Example
///
/// ```rust,no_run
/// # use gloc::Reactor;
/// # use gloc_axum::AxumReactor;
/// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
/// # struct R { s: S }
/// # impl R { fn new() -> Self { Self { s: S(0) } } }
/// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if &s != &self.s { self.s = s; } } }
/// let state = AxumReactor::new(R::new());
/// let count = state.state().0;   // Deref into GlocProvider
/// ```
pub struct AxumReactor<R: Reactor>(GlocProvider<R>)
where
    R::State: Send;

impl<R> AxumReactor<R>
where
    R: Reactor + Send + 'static,
    R::State: Clone + PartialEq + Debug + Send + 'static,
{
    /// Creates a new `AxumReactor` by wrapping `reactor` in the shared
    /// primitives required for multi-handler, multi-thread access.
    ///
    /// The initial state is read from `reactor.state()` and used to seed
    /// the [`GlocStream`], so listeners registered before the first mutation
    /// still have a valid baseline state.
    pub fn new(reactor: R) -> Self {
        Self(new_axum_state(reactor))
    }
}

/// Manual `Clone` — delegates to `GlocProvider::clone` which only increments
/// `Arc` reference counts, so cloning `AxumReactor` is always cheap.
impl<R> Clone for AxumReactor<R>
where
    R: Reactor,
    R::State: Send,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// `Deref` exposes all [`GlocProvider`] methods directly on `AxumReactor`
/// without requiring callers to reach into the tuple field.
impl<R> Deref for AxumReactor<R>
where
    R: Reactor,
    R::State: Send,
{
    type Target = GlocProvider<R>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
