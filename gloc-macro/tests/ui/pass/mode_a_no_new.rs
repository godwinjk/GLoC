use gloc::Cubit;
use gloc_macro::cubit;

#[derive(Clone, PartialEq, Debug)]
struct MyState { pub value: i32 }

#[cubit(state = MyState, no_new)]
struct MyCubit {}

impl MyCubit {
    // Custom constructor since no_new suppresses the generated one.
    pub fn with_value(v: i32) -> Self {
        Self {
            __gloc_state: MyState { value: v },
            __gloc_listeners: Vec::new(),
        }
    }
}

fn main() {
    let c = MyCubit::with_value(42);
    assert_eq!(c.state().value, 42);
}
