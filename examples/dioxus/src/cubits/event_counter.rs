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
