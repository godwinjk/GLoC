//! # gloc-test
//!
//! Testing utilities for GLoC reactors, inspired by [`bloc_test`] from Flutter.
//!
//! The centrepiece is [`ReactorTester`] — a harness that wraps any reactor,
//! captures every emitted state transition, and provides fluent assertions.
//! The companion [`reactor_test!`] macro gives a compact, declarative syntax
//! for common test patterns.
//!
//! [`bloc_test`]: https://pub.dev/packages/bloc_test
//!
//! ## Quick start
//!
//! ```rust
//! use gloc_core::{Reactor, ReactorBase};
//! use gloc_test::{ReactorTester, reactor_test};
//!
//! // --- subject under test ---
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { count: i32 }
//!
//! struct CounterReactor { state: CounterState, stream: gloc_core::stream::GlocStream<CounterState> }
//! impl CounterReactor {
//!     fn new() -> Self {
//!         let state = CounterState { count: 0 };
//!         Self { stream: gloc_core::stream::GlocStream::new(state.clone()), state }
//!     }
//!     fn increment(&mut self) { self.emit(CounterState { count: self.state.count + 1 }); }
//!     fn reset(&mut self) { self.emit(CounterState { count: 0 }); }
//! }
//! impl Reactor for CounterReactor {
//!     type State = CounterState;
//!     fn state(&self) -> &CounterState { &self.state }
//!     fn emit(&mut self, next: CounterState) {
//!         if next != self.state {
//!             let old = self.state.clone();
//!             self.state = next.clone();
//!             self.stream.emit_transition(&old, &next);
//!         }
//!     }
//!     fn stream(&self) -> gloc_core::stream::GlocStream<CounterState> { self.stream.clone() }
//! }
//!
//! // --- tests ---
//!
//! fn increment_emits_new_states() {
//!     let tester = ReactorTester::new(CounterReactor::new());
//!     tester.act(|r| r.increment());
//!     tester.act(|r| r.increment());
//!     tester.assert_states(&[
//!         CounterState { count: 1 },
//!         CounterState { count: 2 },
//!     ]);
//! }
//!
//! fn reset_after_increment() {
//!     reactor_test! {
//!         build: CounterReactor::new(),
//!         acts: [
//!             |r| r.increment(),
//!             |r| r.reset(),
//!         ],
//!         expect_states: [
//!             CounterState { count: 1 },
//!             CounterState { count: 0 },
//!         ],
//!     }
//! }
//!
//! fn duplicate_emit_is_silent() {
//!     reactor_test! {
//!         build: CounterReactor::new(),
//!         acts: [|r| r.reset()],  // reset from 0 → no change
//!         expect_no_emissions: true,
//!     }
//! }
//! ```

pub mod tester;

pub use tester::ReactorTester;

// ---------------------------------------------------------------------------
// reactor_test! — declarative test macro
// ---------------------------------------------------------------------------

/// Compact, declarative test harness for GLoC reactors.
///
/// Each `acts` entry is a closure called as a separate [`ReactorTester::act`]
/// invocation, so every mutation is captured as its own transition.
///
/// # Variants
///
/// **Assert a state sequence**
/// ```rust,ignore
/// reactor_test! {
///     build: MyReactor::new(),
///     acts: [
///         |r| r.some_method(),
///         |r| r.another_method(),
///     ],
///     expect_states: [
///         MyState { value: 1 },
///         MyState { value: 2 },
///     ],
/// }
/// ```
///
/// **Assert that no state was emitted**
/// ```rust,ignore
/// reactor_test! {
///     build: MyReactor::new(),
///     acts: [|r| r.noop()],
///     expect_no_emissions: true,
/// }
/// ```
///
/// **Assert state transitions (old → new pairs)**
/// ```rust,ignore
/// reactor_test! {
///     build: MyReactor::new(),
///     acts: [|r| r.some_method()],
///     expect_transitions: [
///         (MyState { value: 0 }, MyState { value: 1 }),
///     ],
/// }
/// ```
#[macro_export]
macro_rules! reactor_test {
    // --- variant: acts + expect_states ---
    (
        build: $build:expr,
        acts: [$($act:expr),* $(,)?],
        expect_states: [$($state:expr),* $(,)?]
        $(,)?
    ) => {{
        let tester = $crate::ReactorTester::new($build);
        $(tester.act($act);)*
        tester.assert_states(&[$($state),*]);
    }};

    // --- variant: acts + expect_no_emissions ---
    (
        build: $build:expr,
        acts: [$($act:expr),* $(,)?],
        expect_no_emissions: true
        $(,)?
    ) => {{
        let tester = $crate::ReactorTester::new($build);
        $(tester.act($act);)*
        tester.assert_no_emissions();
    }};

    // --- variant: no acts + expect_no_emissions (idle reactor check) ---
    (
        build: $build:expr,
        expect_no_emissions: true
        $(,)?
    ) => {{
        let tester = $crate::ReactorTester::new($build);
        tester.assert_no_emissions();
    }};

    // --- variant: acts + expect_transitions (old → new pairs) ---
    (
        build: $build:expr,
        acts: [$($act:expr),* $(,)?],
        expect_transitions: [$( ($from:expr, $to:expr) ),* $(,)?]
        $(,)?
    ) => {{
        let tester = $crate::ReactorTester::new($build);
        $(tester.act($act);)*
        tester.assert_transitions(&[$(($from, $to)),*]);
    }};
}
