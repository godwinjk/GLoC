//! [`ReactorTester`] — the test harness for GLoC reactors.

use std::sync::{Arc, Mutex};

use gloc_core::{provider::GlocProvider, stream::ListenerHandle, Reactor};

/// Shared, captured transition log. Factored out to keep struct fields readable.
type TransitionLog<S> = Arc<Mutex<Vec<(S, S)>>>;

// ---------------------------------------------------------------------------
// ReactorTester
// ---------------------------------------------------------------------------

/// A test harness that wraps a reactor, runs mutations, and captures every
/// emitted state transition for assertion.
///
/// Each call to [`act`](Self::act) is a single unit of work: it runs the
/// provided closure through the reactor and records one transition if the
/// state actually changed. Calling `act` multiple times accumulates all
/// transitions in order — this is the idiomatic way to verify a sequence of
/// state changes.
///
/// # Design
///
/// `ReactorTester` wraps the reactor in a [`GlocProvider`], registers a
/// stream listener on construction, and records the *new* state of every
/// real transition. Change-detection is preserved: emitting an equal state
/// produces no entry.
///
/// # Example
///
/// ```rust
/// use gloc_core::{Reactor, ReactorBase};
/// use gloc_test::ReactorTester;
///
/// #[derive(Clone, PartialEq, Debug)]
/// struct S(i32);
///
/// struct Counter { state: S, stream: gloc_core::stream::GlocStream<S> }
/// impl Counter {
///     fn new() -> Self { let s = S(0); Self { stream: gloc_core::stream::GlocStream::new(s.clone()), state: s } }
///     fn inc(&mut self) { let n = self.state.0 + 1; self.emit(S(n)); }
/// }
/// impl Reactor for Counter {
///     type State = S;
///     fn state(&self) -> &S { &self.state }
///     fn emit(&mut self, next: S) {
///         if next != self.state {
///             let old = self.state.clone();
///             self.state = next.clone();
///             self.stream.emit_transition(&old, &next);
///         }
///     }
///     fn stream(&self) -> gloc_core::stream::GlocStream<S> { self.stream.clone() }
/// }
///
/// let tester = ReactorTester::new(Counter::new());
/// tester.act(|r| r.inc());
/// tester.act(|r| r.inc());
/// tester.assert_states(&[S(1), S(2)]);
/// ```
pub struct ReactorTester<R: Reactor>
where
    R::State: Send,
{
    provider: GlocProvider<R>,
    /// Every (old, new) pair captured since construction.
    transitions: TransitionLog<R::State>,
    /// Keeps the listener alive for the tester's lifetime.
    _handle: ListenerHandle,
}

impl<R: Reactor + Send + 'static> ReactorTester<R>
where
    R::State: Send,
{
    /// Constructs a `ReactorTester` from a fully initialised reactor.
    ///
    /// A stream listener is registered immediately so that transitions
    /// produced by any subsequent [`act`](Self::act) call are captured.
    ///
    /// # Parameters
    ///
    /// - `reactor` — the reactor instance under test; its current state
    ///   becomes the initial state of the harness.
    pub fn new(reactor: R) -> Self {
        let arc = Arc::new(Mutex::new(reactor));
        let provider = GlocProvider::new(arc);

        let transitions: TransitionLog<R::State> = Arc::new(Mutex::new(Vec::new()));
        let tx_clone = transitions.clone();

        let _handle = provider.listen(move |old, new| {
            tx_clone.lock().unwrap().push((old.clone(), new.clone()));
        });

        Self {
            provider,
            transitions,
            _handle,
        }
    }

    /// Applies one mutation to the reactor.
    ///
    /// The closure receives `&mut R` and may call any domain method or
    /// `fire()` call. If the reactor's state changes as a result, the
    /// transition is recorded and available via [`emitted_states`](Self::emitted_states)
    /// and [`transitions`](Self::captured_transitions).
    ///
    /// Returns `&self` so calls can be chained:
    /// ```rust,ignore
    /// tester.act(|r| r.increment()).act(|r| r.reset());
    /// ```
    ///
    /// # Parameters
    ///
    /// - `f` — a closure that performs exactly one logical action on the reactor.
    ///   Use separate `act` calls to assert intermediate states.
    pub fn act(&self, f: impl FnOnce(&mut R)) -> &Self {
        self.provider.update(f);
        self
    }

    /// Returns a clone of the reactor's current state.
    pub fn state(&self) -> R::State {
        self.provider.state()
    }

    /// Returns every *new* state emitted since the tester was constructed,
    /// in the order they were produced.
    ///
    /// Transitions where the state did not change are excluded (change-detection
    /// in `emit()` guarantees this).
    pub fn emitted_states(&self) -> Vec<R::State> {
        self.transitions
            .lock()
            .unwrap()
            .iter()
            .map(|(_, new)| new.clone())
            .collect()
    }

    /// Returns every `(old, new)` state pair captured since construction,
    /// in the order they were produced.
    pub fn captured_transitions(&self) -> Vec<(R::State, R::State)> {
        self.transitions.lock().unwrap().clone()
    }

    // -----------------------------------------------------------------------
    // Assertions
    // -----------------------------------------------------------------------

    /// Asserts that the sequence of emitted states equals `expected`.
    ///
    /// Panics with a diff-friendly message if the actual sequence does not
    /// match. The comparison uses `PartialEq` and the message uses `Debug`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// tester.act(|r| r.increment());
    /// tester.act(|r| r.increment());
    /// tester.assert_states(&[CounterState { count: 1 }, CounterState { count: 2 }]);
    /// ```
    pub fn assert_states(&self, expected: &[R::State]) {
        let actual = self.emitted_states();
        assert_eq!(
            actual, expected,
            "\nreactor state sequence mismatch\n  actual:   {actual:?}\n  expected: {expected:?}"
        );
    }

    /// Asserts that no state transition was emitted.
    ///
    /// Use this to verify that a no-op mutation or a duplicate emit is
    /// correctly swallowed by the reactor's change-detection guard.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Reset from already-zero — no change expected.
    /// tester.act(|r| r.reset());
    /// tester.assert_no_emissions();
    /// ```
    pub fn assert_no_emissions(&self) {
        let actual = self.emitted_states();
        assert!(
            actual.is_empty(),
            "expected no state emissions, but got: {actual:?}"
        );
    }

    /// Asserts the full sequence of `(old, new)` transition pairs.
    ///
    /// Use this when you need to verify the *path* through state space, not
    /// just the final values.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// tester.act(|r| r.increment());
    /// tester.assert_transitions(&[
    ///     (CounterState { count: 0 }, CounterState { count: 1 }),
    /// ]);
    /// ```
    pub fn assert_transitions(&self, expected: &[(R::State, R::State)]) {
        let actual = self.captured_transitions();
        assert_eq!(
            actual,
            expected,
            "\nreactor transition sequence mismatch\n  actual:   {actual:?}\n  expected: {expected:?}"
        );
    }

    /// Asserts the reactor's current state equals `expected`.
    ///
    /// This checks the live state, not the emission history. Useful after
    /// a sequence of mutations to confirm the final resting state.
    pub fn assert_state(&self, expected: &R::State) {
        let actual = self.state();
        assert_eq!(
            &actual, expected,
            "\nreactor current state mismatch\n  actual:   {actual:?}\n  expected: {expected:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gloc_core::Reactor;

    use super::*;

    // ---- minimal inline reactor ----

    #[derive(Clone, PartialEq, Debug)]
    struct Count(i32);

    struct Counter {
        state: Count,
        stream: gloc_core::stream::GlocStream<Count>,
    }

    impl Counter {
        fn new(n: i32) -> Self {
            let state = Count(n);
            Self {
                stream: gloc_core::stream::GlocStream::new(state.clone()),
                state,
            }
        }

        fn increment(&mut self) {
            let n = self.state.0 + 1;
            self.emit(Count(n));
        }
        fn add(&mut self, n: i32) {
            let v = self.state.0 + n;
            self.emit(Count(v));
        }
        fn reset(&mut self) {
            self.emit(Count(0));
        }
    }

    impl Reactor for Counter {
        type State = Count;

        fn state(&self) -> &Count {
            &self.state
        }

        fn emit(&mut self, next: Count) {
            if next != self.state {
                let old = self.state.clone();
                self.state = next.clone();
                self.stream.emit_transition(&old, &next);
            }
        }

        fn stream(&self) -> gloc_core::stream::GlocStream<Count> {
            self.stream.clone()
        }
    }

    // ---- happy path ----

    #[test]
    fn captures_single_emission() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.increment());
        tester.assert_states(&[Count(1)]);
    }

    #[test]
    fn captures_sequence_of_emissions() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.increment());
        tester.act(|r| r.increment());
        tester.act(|r| r.increment());
        tester.assert_states(&[Count(1), Count(2), Count(3)]);
    }

    #[test]
    fn act_chaining_works() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.increment()).act(|r| r.increment());
        tester.assert_states(&[Count(1), Count(2)]);
    }

    // ---- edge cases ----

    #[test]
    fn no_emission_on_noop() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.reset()); // already 0 → no change
        tester.assert_no_emissions();
    }

    #[test]
    fn assert_state_returns_current_state() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.add(5));
        tester.assert_state(&Count(5));
    }

    // ---- boundary ----

    #[test]
    fn assert_transitions_captures_old_and_new() {
        let tester = ReactorTester::new(Counter::new(0));
        tester.act(|r| r.increment());
        tester.act(|r| r.add(9));
        tester.assert_transitions(&[(Count(0), Count(1)), (Count(1), Count(10))]);
    }

    #[test]
    fn empty_tester_has_no_emissions() {
        let tester = ReactorTester::new(Counter::new(42));
        tester.assert_no_emissions();
        tester.assert_state(&Count(42));
    }

    // ---- trait-object test (required by code standards) ----

    #[test]
    fn works_via_dyn_reactor_state_assertion() {
        fn check(reactor: impl Reactor<State = Count> + Send + 'static) {
            let tester = ReactorTester::new(reactor);
            tester.act(|r| r.emit(Count(7)));
            tester.assert_states(&[Count(7)]);
        }
        check(Counter::new(0));
    }
}
