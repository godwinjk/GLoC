use crate::state::State;
use crate::stream::GlocStream;

/// The core abstraction for state management in GLoC.
///
/// A `Reactor` owns a single piece of state and exposes methods that call
/// [`emit`](Reactor::emit) to produce the next state. Every reactor carries a
/// built-in [`GlocStream`] — subscribable by adapters, UI components, and
/// other reactors — that broadcasts every real state transition automatically.
///
/// # Implementing
///
/// Use the `#[reactor]` macro — it generates the full trait impl, a
/// constructor, and wires the stream:
///
/// ```rust,ignore
/// #[reactor(state = CounterState)]
/// pub struct CounterReactor {}
///
/// impl CounterReactor {
///     pub fn increment(&mut self) {
///         self.emit(CounterState { count: self.count + 1 });
///     }
/// }
/// ```
///
/// # Manual implementation
///
/// Manual impls must carry a `GlocStream` field and implement `stream()`:
///
/// ```rust
/// use gloc_core::{Reactor, State};
/// use gloc_core::stream::GlocStream;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct CounterState { count: i32 }
///
/// struct CounterReactor { state: CounterState, stream: GlocStream<CounterState> }
///
/// impl CounterReactor {
///     pub fn new(count: i32) -> Self {
///         let state = CounterState { count };
///         Self { stream: GlocStream::new(state.clone()), state }
///     }
/// }
///
/// impl Reactor for CounterReactor {
///     type State = CounterState;
///     fn state(&self) -> &CounterState { &self.state }
///     fn emit(&mut self, next: CounterState) {
///         if next != self.state {
///             let old = self.state.clone();
///             self.state = next.clone();
///             self.stream.emit_transition(&old, &next);
///         }
///     }
///     fn stream(&self) -> GlocStream<CounterState> { self.stream.clone() }
/// }
/// ```
pub trait Reactor {
    /// The type that represents this reactor's domain state.
    type State: State;

    /// Returns a shared reference to the reactor's current state.
    fn state(&self) -> &Self::State;

    /// Transitions the reactor to `next`.
    ///
    /// Implementations should guard against redundant emissions — if
    /// `next == self.state()` the state should remain unchanged and the
    /// stream should not fire. Use the `#[reactor]` macro for automatic
    /// change-detection.
    fn emit(&mut self, next: Self::State);

    /// Returns a clone of the reactor's built-in stream.
    ///
    /// The stream broadcasts every real state transition to all subscribers.
    /// Any number of listeners can register — UI components, other reactors,
    /// loggers — and they all receive `(old, new)` synchronously on every emit.
    ///
    /// ```text
    /// reactor.stream().listen(|old, new| { /* react */ });
    /// ```
    fn stream(&self) -> GlocStream<Self::State>;

    /// Called when the reactor is explicitly closed (e.g. via
    /// [`GlocProvider::release`](crate::provider::GlocProvider::release)).
    /// Override to clean up resources — close connections, cancel timers, etc.
    fn on_close(&mut self) {}
}

/// A ready-to-use reactor implementation for simple cases.
///
/// `ReactorBase<S>` wraps any [`State`] with built-in change-detection,
/// a reactive [`GlocStream`], and automatic global observer wiring.
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
pub struct ReactorBase<S: State + Send> {
    state: S,
    stream: GlocStream<S>,
}

impl<S: State + Send> std::fmt::Debug for ReactorBase<S>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactorBase")
            .field("state", &self.state)
            .finish()
    }
}

impl<S: State + Send + 'static> ReactorBase<S> {
    /// Creates a new `ReactorBase` with the given `initial` state.
    ///
    /// Fires `GlocObserver::on_create` if a global observer is registered.
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
        let stream = GlocStream::new(initial.clone());
        if let Some(obs) = crate::observer::observer() {
            obs.on_create(std::any::type_name::<Self>());
        }
        Self {
            state: initial,
            stream,
        }
    }
}

impl<S: State + Send + 'static> Reactor for ReactorBase<S> {
    type State = S;

    fn state(&self) -> &S {
        &self.state
    }

    /// Transitions to `next` if it differs from the current state.
    ///
    /// Fires the stream and notifies the global observer on every real
    /// transition. Identical states are silently ignored.
    fn emit(&mut self, next: S) {
        if next != self.state {
            let old = self.state.clone();
            self.state = next.clone();
            self.stream.emit_transition(&old, &next);
            if let Some(obs) = crate::observer::observer() {
                obs.on_transition(
                    std::any::type_name::<Self>(),
                    &format!("{old:?}"),
                    &format!("{next:?}"),
                );
            }
        }
    }

    /// Returns a clone of this reactor's built-in stream.
    fn stream(&self) -> GlocStream<S> {
        self.stream.clone()
    }
}
