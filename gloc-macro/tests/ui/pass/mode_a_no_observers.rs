use gloc::Cubit;
use gloc_macro::cubit;

#[derive(Clone, PartialEq, Debug)]
struct LeanState { pub x: f64 }

#[cubit(state = LeanState, no_observers)]
struct LeanCubit {}

fn main() {
    let mut c = LeanCubit::new(LeanState { x: 1.0 });
    c.emit(LeanState { x: 2.0 });
    assert_eq!(c.state().x, 2.0);
}
