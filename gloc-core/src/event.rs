/// A neutron fired into a reactor to trigger a state transition.
///
/// Any type that is `Debug + Send + 'static` automatically implements `Neutron`
/// via the blanket impl — no manual implementation needed.
///
/// In nuclear fission, neutrons trigger chain reactions. In GLoC, a `Neutron`
/// triggers a state transition via [`Reactor::fire()`](crate::Reactor).
///
/// # Requirements
///
/// - [`Debug`]  — required for diagnostics, observer logging, and test output.
/// - [`Send`]   — neutrons may be fired across threads via [`GlocProvider`](crate::GlocProvider).
/// - `'static`  — neutrons may be captured in closures or stored temporarily.
///
/// # Blanket implementation
///
/// Any type that satisfies the three bounds above automatically implements
/// `Neutron`, so you never need to write `impl Neutron for MyEvent {}` yourself.
///
/// # Comparison with State
///
/// | | [`State`](crate::State) | `Neutron` |
/// |---|---|---|
/// | `Clone` | required — shared via stream | not required — consumed on fire |
/// | `PartialEq` | required — change-detection | not required — neutrons are not compared |
/// | `Debug` | required | required |
/// | `Send` | implicit | required |
///
/// # Example
///
/// ```rust
/// use gloc_core::Neutron;
///
/// #[derive(Debug)]
/// enum CounterEvent {
///     Increment,
///     Decrement,
///     Reset,
///     AddBy(i32),
/// }
///
/// fn assert_is_neutron<N: Neutron>() {}
/// assert_is_neutron::<CounterEvent>();
/// ```
pub trait Neutron: std::fmt::Debug + Send + 'static {}

/// Blanket implementation: every `Debug + Send + 'static` type is a `Neutron`.
impl<T: std::fmt::Debug + Send + 'static> Neutron for T {}

/// Familiar alias for [`Neutron`] — use whichever name fits your mental model.
///
/// `Event` is kept as a type alias so existing code that references `gloc::Event`
/// continues to compile without modification. New code should prefer [`Neutron`].
pub type Event = dyn Neutron;
