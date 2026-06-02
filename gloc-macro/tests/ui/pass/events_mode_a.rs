use gloc::Reactor;
use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}

#[derive(Debug)]
enum CounterEvent {
    Increment,
    Decrement,
    Reset,
    AddBy(i32),
}

#[reactor(state = CounterState, neutrons = CounterEvent)]
struct CounterReactor {}

impl CounterReactor {
    // Direct method — coexists with fire()
    pub fn increment(&mut self) {
        self.emit(CounterState {
            count: self.state().count + 1,
        });
    }

    fn on_event(&mut self, neutron: CounterEvent) {
        match neutron {
            CounterEvent::Increment => self.emit(CounterState {
                count: self.state().count + 1,
            }),
            CounterEvent::Decrement => self.emit(CounterState {
                count: self.state().count - 1,
            }),
            CounterEvent::Reset => self.emit(CounterState { count: 0 }),
            CounterEvent::AddBy(n) => self.emit(CounterState {
                count: self.state().count + n,
            }),
        }
    }
}

fn main() {
    let mut r = CounterReactor::new(CounterState { count: 0 });

    r.increment(); // direct
    assert_eq!(r.state().count, 1);

    r.fire(CounterEvent::Increment); // neutron fire
    assert_eq!(r.state().count, 2);

    r.fire(CounterEvent::AddBy(8)); // with payload
    assert_eq!(r.state().count, 10);

    r.fire(CounterEvent::Reset);
    assert_eq!(r.state().count, 0);
}
