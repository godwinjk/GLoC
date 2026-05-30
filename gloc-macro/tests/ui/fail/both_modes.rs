use gloc_macro::cubit;

#[derive(Clone, PartialEq, Debug)]
struct MyState { pub value: i32 }

// Error: cannot use both `state = T` (Mode A) and `#[state]` fields (Mode B).
#[cubit(state = MyState)]
struct Conflict {
    #[state]
    count: i32,
}

fn main() {}
