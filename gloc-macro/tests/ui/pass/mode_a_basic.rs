use gloc::Reactor;
use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct CounterState {
    pub count: i32,
}

#[reactor(state = CounterState)]
struct CounterReactor {}

impl CounterReactor {
    fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState { count: next });
    }
}

fn main() {
    let mut r = CounterReactor::new(CounterState { count: 0 });
    r.increment();
    assert_eq!(r.state().count, 1);
}
