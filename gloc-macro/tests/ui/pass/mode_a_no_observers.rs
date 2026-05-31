use gloc::Reactor;
use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct LeanState {
    pub x: f64,
}

#[reactor(state = LeanState, no_observers)]
struct LeanReactor {}

fn main() {
    let mut r = LeanReactor::new(LeanState { x: 1.0 });
    r.emit(LeanState { x: 2.0 });
    assert_eq!(r.state().x, 2.0);
}
