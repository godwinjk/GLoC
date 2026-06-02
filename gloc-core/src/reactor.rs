use crate::state::State;

/// The core abstraction for state management in GLoC.
///
/// A `Reactor` owns a single piece of state and exposes methods that call
/// [`emit`](Reactor::emit) to produce the next state. Callers can interact
/// with a `Reactor` in two ways:
///
/// - **Direct method calls** — invoke domain methods (e.g. `reactor.increment()`)
///   that call `emit` internally. Simple and ergonomic for most use-cases.
/// - **Event dispatch** — call `reactor.dispatch(Event)` and the reactor handles
///   the event and emits the next state. Opt-in via the `#[reactor]` macro.
///
/// Both styles are just methods — there is no separate `Bloc` type in GLoC.
///
/// # Design (SOLID)
///
/// - **Single Responsibility** — each `Reactor` manages one cohesive slice of
///   domain state (e.g. `AuthReactor`, `CartReactor`).
/// - **Open / Closed** — extend behaviour by composing reactors rather than
///   modifying existing ones.
/// - **Dependency Inversion** — callers depend on the `Reactor` *trait*, not
///   concrete implementations, enabling easy mocking and injection in tests.
///
/// # Implementing
///
/// ```rust
/// use gloc_core::{Reactor, State};
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct CounterState {
///     count: i32,
/// }
///
/// struct CounterReactor {
///     state: CounterState,
/// }
///
/// impl CounterReactor {
///     pub fn new(initial: i32) -> Self {
///         Self { state: CounterState { count: initial } }
///     }
///
///     pub fn increment(&mut self) {
///         let next = self.state().count + 1;
///         self.emit(CounterState { count: next });
///     }
///
///     pub fn decrement(&mut self) {
///         let next = self.state().count - 1;
///         self.emit(CounterState { count: next });
///     }
/// }
///
/// impl Reactor for CounterReactor {
///     type State = CounterState;
///
///     fn state(&self) -> &CounterState {
///         &self.state
///     }
///
///     fn emit(&mut self, next: CounterState) {
///         if &next != self.state() {
///             self.state = next;
///         }
///     }
/// }
/// ```
pub trait Reactor {
    /// The type that represents this reactor's domain state.
    ///
    /// Must implement [`State`], which requires `Clone + PartialEq + Debug`.
    type State: State;

    /// Returns a shared reference to the reactor's current state.
    ///
    /// Implementations must return the most recently emitted state, or the
    /// initial state if [`emit`](Reactor::emit) has never been called.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { self.s = s; } }
    /// let r = R { s: S(0) };
    /// assert_eq!(r.state(), &S(0));
    /// ```
    fn state(&self) -> &Self::State;

    /// Transitions the reactor to `next`, replacing the current state.
    ///
    /// Implementations **should** guard against redundant emissions:
    /// if `next == self.state()` the state should remain unchanged and no
    /// downstream notification should fire. Change-detection ensures correct
    /// behaviour and avoids unnecessary re-renders or side effects.
    ///
    /// # Parameters
    ///
    /// - `next` — the new state value. It is consumed (moved) into the reactor.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc_core::{Reactor, State};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct R { s: S }
    /// # impl Reactor for R { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if &s != &self.s { self.s = s; } } }
    /// let mut r = R { s: S(0) };
    /// r.emit(S(1));
    /// assert_eq!(r.state(), &S(1));
    ///
    /// // Emitting the same state is a no-op.
    /// r.emit(S(1));
    /// assert_eq!(r.state(), &S(1));
    /// ```
    fn emit(&mut self, next: Self::State);

    /// Called by [`GlocProvider::release()`] when the provider releases this reactor.
    /// Override to clean up resources — close connections, cancel timers, etc.
    /// Never called when the reactor is used directly without a provider.
    fn on_close(&mut self) {}
}

/// A ready-to-use, heap-allocated `Reactor` implementation.
///
/// `ReactorBase` wraps any [`State`] and provides the standard `emit`/`state`
/// behaviour with **change-detection** built in: emitting a value equal to the
/// current state is a no-op.
///
/// Use this directly when you want a reactor without writing a custom struct, or
/// as the inner backing store of your own reactor implementation.
///
/// # Type Parameters
///
/// - `S` — a type that implements [`State`] (`Clone + PartialEq + Debug`).
///
/// # Example
///
/// ```rust
/// use gloc_core::{Reactor, ReactorBase};
///
/// let mut reactor = ReactorBase::new(0_i32);
/// assert_eq!(*reactor.state(), 0);
///
/// reactor.emit(42);
/// assert_eq!(*reactor.state(), 42);
///
/// // Duplicate emission is ignored.
/// reactor.emit(42);
/// assert_eq!(*reactor.state(), 42);
/// ```
#[derive(Debug)]
pub struct ReactorBase<S: State> {
    state: S,
}

impl<S: State> ReactorBase<S> {
    /// Creates a new `ReactorBase` with the given `initial` state.
    ///
    /// # Parameters
    ///
    /// - `initial` — the starting state. Subsequent calls to [`emit`](Reactor::emit)
    ///   will replace this value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc_core::{Reactor, ReactorBase};
    ///
    /// let reactor = ReactorBase::new("idle");
    /// assert_eq!(*reactor.state(), "idle");
    /// ```
    pub fn new(initial: S) -> Self {
        Self { state: initial }
    }
}

impl<S: State> Reactor for ReactorBase<S> {
    type State = S;

    /// Returns a reference to the current state held by this `ReactorBase`.
    fn state(&self) -> &S {
        &self.state
    }

    /// Replaces the current state with `next` if and only if `next != self.state()`.
    ///
    /// This change-detection guard ensures that downstream listeners are not
    /// notified when the logical state has not actually changed.
    fn emit(&mut self, next: S) {
        if next != self.state {
            self.state = next;
        }
    }
}
