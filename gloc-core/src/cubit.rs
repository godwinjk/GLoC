use crate::state::State;

/// The core abstraction for simple, function-driven state management.
///
/// A `Cubit` owns a single piece of state and exposes methods that call
/// [`emit`](Cubit::emit) to produce the next state. Unlike a full [`Bloc`],
/// a `Cubit` has **no event type** — callers invoke methods directly, making
/// it ideal for straightforward domain logic that does not need an explicit
/// event bus.
///
/// # Design (SOLID)
///
/// - **Single Responsibility** — each `Cubit` manages one cohesive slice of
///   domain state (e.g. `AuthCubit`, `CartCubit`).
/// - **Open / Closed** — you extend behaviour by composing cubits rather than
///   modifying existing ones.
/// - **Dependency Inversion** — callers depend on the `Cubit` *trait*, not
///   concrete implementations, enabling easy mocking and injection in tests.
///
/// # Implementing
///
/// ```rust
/// use gloc::{Cubit, State};
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct CounterState {
///     count: i32,
/// }
///
/// struct CounterCubit {
///     state: CounterState,
/// }
///
/// impl CounterCubit {
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
/// impl Cubit for CounterCubit {
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
pub trait Cubit {
    /// The type that represents this cubit's domain state.
    ///
    /// Must implement [`State`], which requires `Clone + PartialEq + Debug`.
    type State: State;

    /// Returns a shared reference to the cubit's current state.
    ///
    /// Implementations must return the most recently emitted state, or the
    /// initial state if [`emit`](Cubit::emit) has never been called.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc::{Cubit, State};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct C { s: S }
    /// # impl Cubit for C { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { self.s = s; } }
    /// let c = C { s: S(0) };
    /// assert_eq!(c.state(), &S(0));
    /// ```
    fn state(&self) -> &Self::State;

    /// Transitions the cubit to `next`, replacing the current state.
    ///
    /// Implementations **should** guard against redundant emissions:
    /// if `next == self.state()` the state should remain unchanged and no
    /// downstream notification should fire. This mirrors Flutter Bloc's
    /// behaviour and avoids unnecessary re-renders or side effects.
    ///
    /// # Parameters
    ///
    /// - `next` — the new state value. It is consumed (moved) into the cubit.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use gloc::{Cubit, State};
    /// # #[derive(Clone, PartialEq, Debug)] struct S(i32);
    /// # struct C { s: S }
    /// # impl Cubit for C { type State = S; fn state(&self) -> &S { &self.s } fn emit(&mut self, s: S) { if &s != &self.s { self.s = s; } } }
    /// let mut c = C { s: S(0) };
    /// c.emit(S(1));
    /// assert_eq!(c.state(), &S(1));
    ///
    /// // Emitting the same state is a no-op.
    /// c.emit(S(1));
    /// assert_eq!(c.state(), &S(1));
    /// ```
    fn emit(&mut self, next: Self::State);
}

/// A ready-to-use, heap-allocated `Cubit` implementation.
///
/// `CubitBase` wraps any [`State`] and provides the standard `emit`/`state`
/// behaviour with **change-detection** built in: emitting a value equal to the
/// current state is a no-op.
///
/// Use this directly when you want a cubit without writing a custom struct, or
/// use it as the inner backing store of your own cubit implementation.
///
/// # Type Parameters
///
/// - `S` — a type that implements [`State`] (`Clone + PartialEq + Debug`).
///
/// # Example
///
/// ```rust
/// use gloc::{Cubit, CubitBase};
///
/// let mut cubit = CubitBase::new(0_i32);
/// assert_eq!(*cubit.state(), 0);
///
/// cubit.emit(42);
/// assert_eq!(*cubit.state(), 42);
///
/// // Duplicate emission is ignored.
/// cubit.emit(42);
/// assert_eq!(*cubit.state(), 42);
/// ```
#[derive(Debug)]
pub struct CubitBase<S: State> {
    state: S,
}

impl<S: State> CubitBase<S> {
    /// Creates a new `CubitBase` with the given `initial` state.
    ///
    /// # Parameters
    ///
    /// - `initial` — the starting state. Subsequent calls to [`emit`](Cubit::emit)
    ///   will replace this value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use gloc::{Cubit, CubitBase};
    ///
    /// let cubit = CubitBase::new("idle");
    /// assert_eq!(*cubit.state(), "idle");
    /// ```
    pub fn new(initial: S) -> Self {
        Self { state: initial }
    }
}

impl<S: State> Cubit for CubitBase<S> {
    type State = S;

    /// Returns a reference to the current state held by this `CubitBase`.
    fn state(&self) -> &S {
        &self.state
    }

    /// Replaces the current state with `next` if and only if `next != self.state()`.
    ///
    /// This change-detection guard ensures that downstream listeners are not
    /// notified when the logical state has not actually changed, matching the
    /// semantics of `flutter_bloc`'s `Cubit`.
    fn emit(&mut self, next: S) {
        if next != self.state {
            self.state = next;
        }
    }
}
