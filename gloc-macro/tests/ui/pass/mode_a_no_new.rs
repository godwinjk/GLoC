use gloc::Reactor;
use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct MyState {
    pub value: i32,
}

#[reactor(state = MyState, no_new)]
struct MyReactor {}

impl MyReactor {
    // Custom constructor since no_new suppresses the generated one.
    pub fn with_value(v: i32) -> Self {
        Self {
            __gloc_stream: ::gloc::GlocStream::new(MyState { value: v }),
            __gloc_state: MyState { value: v },
        }
    }
}

fn main() {
    let r = MyReactor::with_value(42);
    assert_eq!(r.state().value, 42);
}
