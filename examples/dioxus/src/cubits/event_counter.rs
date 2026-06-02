//! `EventCounterReactor` — demonstrates `reactor.fire(neutron)` style.
//!
//! Uses `#[reactor(state = CounterState, neutrons = CounterEvent)]` to generate
//! a `fire()` method. The user writes `on_event()` to handle each variant.
//!
//! Both direct methods (`increment()`) and neutron firing (`fire(Ev::Inc)`)
//! coexist on the same reactor — they are not mutually exclusive.

use gloc::reactor;
use gloc::Reactor;

use super::counter::CounterState;

// ---------------------------------------------------------------------------
// Event enum — any Debug + Send + 'static type is automatically a Neutron
// ---------------------------------------------------------------------------

/// All the things a user can ask the counter to do.
#[derive(Debug)]
pub enum CounterEvent {
    Increment,
    Decrement,
    Reset,
    AddBy(i32),
}

// ---------------------------------------------------------------------------
// Reactor — macro generates fire(), new(), subscribe()
// ---------------------------------------------------------------------------

/// Counter reactor driven by neutron firing.
///
/// `#[reactor(state = CounterState, neutrons = CounterEvent)]` generates:
/// - `impl Reactor for EventCounterReactor` — `state()`, `emit()` with change-detection
/// - `pub fn new(initial: CounterState) -> Self`
/// - `pub fn fire(&mut self, neutron: CounterEvent)` — calls `self.on_event(neutron)`
/// - `subscribe()`, `attach_listener()`
#[reactor(state = CounterState, neutrons = CounterEvent)]
pub struct EventCounterReactor {}

impl EventCounterReactor {
    /// Direct method — coexists with fire(), both are valid.
    #[allow(dead_code)]
    pub fn increment(&mut self) {
        self.emit(CounterState::new(self.state().count + 1));
    }

    /// Neutron handler — called by `fire()`.
    ///
    /// The match here is the only place neutron → state logic lives.
    /// `emit()` handles change-detection, stream notification, and observer.
    fn on_event(&mut self, neutron: CounterEvent) {
        match neutron {
            CounterEvent::Increment => self.emit(CounterState::new(self.state().count + 1)),
            CounterEvent::Decrement => self.emit(CounterState::new(self.state().count - 1)),
            CounterEvent::Reset => self.emit(CounterState::new(0)),
            CounterEvent::AddBy(n) => self.emit(CounterState::new(self.state().count + n)),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gloc_test::reactor_test;

    use super::*;

    fn counter(n: i32) -> EventCounterReactor {
        EventCounterReactor::new(CounterState::new(n))
    }

    // ---- happy path ----

    #[test]
    fn increment_neutron_increases_count() {
        reactor_test! {
            build: counter(0),
            acts: [|r| r.fire(CounterEvent::Increment)],
            expect_states: [CounterState::new(1)],
        }
    }

    #[test]
    fn decrement_neutron_decreases_count() {
        reactor_test! {
            build: counter(3),
            acts: [|r| r.fire(CounterEvent::Decrement)],
            expect_states: [CounterState::new(2)],
        }
    }

    #[test]
    fn reset_neutron_returns_to_zero() {
        reactor_test! {
            build: counter(42),
            acts: [|r| r.fire(CounterEvent::Reset)],
            expect_states: [CounterState::new(0)],
        }
    }

    #[test]
    fn add_by_neutron_adds_correct_amount() {
        reactor_test! {
            build: counter(10),
            acts: [|r| r.fire(CounterEvent::AddBy(5))],
            expect_states: [CounterState::new(15)],
        }
    }

    #[test]
    fn neutron_sequence_captures_each_step() {
        reactor_test! {
            build: counter(0),
            acts: [
                |r| r.fire(CounterEvent::Increment),
                |r| r.fire(CounterEvent::AddBy(4)),
                |r| r.fire(CounterEvent::Decrement),
                |r| r.fire(CounterEvent::Reset),
            ],
            expect_states: [
                CounterState::new(1),
                CounterState::new(5),
                CounterState::new(4),
                CounterState::new(0),
            ],
        }
    }

    // ---- edge cases ----

    #[test]
    fn reset_from_zero_emits_nothing() {
        reactor_test! {
            build: counter(0),
            acts: [|r| r.fire(CounterEvent::Reset)],
            expect_no_emissions: true,
        }
    }

    #[test]
    fn add_by_zero_emits_nothing() {
        reactor_test! {
            build: counter(5),
            acts: [|r| r.fire(CounterEvent::AddBy(0))],
            expect_no_emissions: true,
        }
    }

    // ---- boundary ----

    #[test]
    fn add_by_negative_value_decreases_count() {
        reactor_test! {
            build: counter(10),
            acts: [|r| r.fire(CounterEvent::AddBy(-10))],
            expect_states: [CounterState::new(0)],
        }
    }

    #[test]
    fn direct_method_and_neutron_coexist() {
        // Verifies that direct increment() and fire(Increment) both work
        // and each produces its own captured transition.
        reactor_test! {
            build: counter(0),
            acts: [
                |r| r.increment(),
                |r| r.fire(CounterEvent::Increment),
            ],
            expect_states: [CounterState::new(1), CounterState::new(2)],
        }
    }
}
