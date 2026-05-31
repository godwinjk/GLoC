//! `EventCounterReactor` — demonstrates `reactor.dispatch(event)` style.
//!
//! Uses `#[reactor(state = CounterState, events = CounterEvent)]` to generate
//! a `dispatch()` method. The user writes `on_event()` to handle each variant.
//!
//! Both direct methods (`increment()`) and event dispatch (`dispatch(Ev::Inc)`)
//! coexist on the same reactor — they are not mutually exclusive.

use gloc::reactor;
use gloc::Reactor;

use super::counter::CounterState;

// ---------------------------------------------------------------------------
// Event enum — any Debug + Send + 'static type is automatically an Event
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
// Reactor — macro generates dispatch(), new(), on_change(), subscribe()
// ---------------------------------------------------------------------------

/// Counter reactor driven by event dispatch.
///
/// `#[reactor(state = CounterState, events = CounterEvent)]` generates:
/// - `impl Reactor for EventCounterReactor` — `state()`, `emit()` with change-detection
/// - `pub fn new(initial: CounterState) -> Self`
/// - `pub fn dispatch(&mut self, event: CounterEvent)` — calls `self.on_event(event)`
/// - `pub fn on_change(...)`, `subscribe()`, `attach_listener()`
#[reactor(state = CounterState, events = CounterEvent)]
pub struct EventCounterReactor {}

impl EventCounterReactor {
    /// Direct method — coexists with dispatch, both are valid.
    #[allow(dead_code)]
    pub fn increment(&mut self) {
        self.emit(CounterState::new(self.state().count + 1));
    }

    /// Event handler — called by `dispatch()`.
    ///
    /// The match here is the only place event → state logic lives.
    /// `emit()` handles change-detection, stream notification, and observer.
    fn on_event(&mut self, event: CounterEvent) {
        match event {
            CounterEvent::Increment => self.emit(CounterState::new(self.state().count + 1)),
            CounterEvent::Decrement => self.emit(CounterState::new(self.state().count - 1)),
            CounterEvent::Reset => self.emit(CounterState::new(0)),
            CounterEvent::AddBy(n) => self.emit(CounterState::new(self.state().count + n)),
        }
    }
}
