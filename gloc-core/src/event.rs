/// A marker trait for any type that can be dispatched to a [`Reactor`](crate::Reactor).
///
/// Implementing this trait on a type signals that it is intended to be used as
/// a discrete instruction dispatched to a reactor via `dispatch()`. Unlike
/// [`State`](crate::State), events are **consumed** — they are moved into the
/// reactor and processed exactly once, not stored or compared.
///
/// # Requirements
///
/// - [`Debug`]  — required for diagnostics, observer logging, and test output.
/// - [`Send`]   — events may be dispatched across threads via [`GlocConsumer`](crate::GlocConsumer).
/// - `'static`  — events may be captured in closures or stored temporarily.
///
/// # Blanket implementation
///
/// Any type that satisfies the three bounds above automatically implements
/// `Event`, so you never need to write `impl Event for MyEvent {}` yourself.
///
/// # Comparison with State
///
/// | | [`State`](crate::State) | `Event` |
/// |---|---|---|
/// | `Clone` | required — shared via stream | not required — consumed on dispatch |
/// | `PartialEq` | required — change-detection | not required — events are not compared |
/// | `Debug` | required | required |
/// | `Send` | implicit | required |
///
/// # Example
///
/// ```rust
/// use gloc_core::Event;
///
/// #[derive(Debug)]
/// enum CounterEvent {
///     Increment,
///     Decrement,
///     Reset,
///     AddBy(i32),
/// }
///
/// fn assert_is_event<E: Event>() {}
/// assert_is_event::<CounterEvent>();
/// ```
pub trait Event: std::fmt::Debug + Send + 'static {}

/// Blanket implementation: every `Debug + Send + 'static` type is an `Event`.
impl<T: std::fmt::Debug + Send + 'static> Event for T {}
