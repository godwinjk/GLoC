/// A marker trait for any type that can serve as state within a [`Cubit`](crate::Cubit).
///
/// Implementing this trait on a type signals that it is intended to be used as
/// an immutable snapshot of some domain's state. GLoC does not enforce
/// immutability at the language level, but callers are expected to treat emitted
/// state values as read-only once published.
///
/// # Requirements
///
/// - [`Clone`]  — every state transition produces a fresh owned value.
/// - [`PartialEq`] — change detection: a new emission is propagated only if
///   the new state differs from the current one, preventing unnecessary work.
/// - [`Debug`]  — required for diagnostics, DevTools logging, and test output.
///
/// # Blanket implementation
///
/// Any type that satisfies the three bounds above automatically implements
/// `State`, so you never need to write `impl State for MyState {}` yourself.
///
/// # Example
///
/// ```rust
/// use gloc_core::State;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct CounterState {
///     count: i32,
/// }
///
/// fn assert_is_state<S: State>() {}
/// assert_is_state::<CounterState>();
/// ```
pub trait State: Clone + PartialEq + std::fmt::Debug {}

/// Blanket implementation: every `Clone + PartialEq + Debug` type is a `State`.
impl<T: Clone + PartialEq + std::fmt::Debug> State for T {}
