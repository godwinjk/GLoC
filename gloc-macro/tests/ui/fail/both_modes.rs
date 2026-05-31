use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct MyState {
    pub value: i32,
}

// Error: cannot use both `state = T` (Mode A) and `#[state]` fields (Mode B).
#[reactor(state = MyState)]
struct Conflict {
    #[state]
    count: i32,
}

fn main() {}
