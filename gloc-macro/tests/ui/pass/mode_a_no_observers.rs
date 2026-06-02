use gloc::Reactor;
use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct LeanState {
    pub x: f64,
}

// no_observers was removed; stream is always present now.
// This test validates a minimal Mode A reactor still works correctly.
#[reactor(state = LeanState)]
struct LeanReactor {}

fn main() {
    let mut r = LeanReactor::new(LeanState { x: 1.0 });
    r.emit(LeanState { x: 2.0 });
    assert_eq!(r.state().x, 2.0);
}
