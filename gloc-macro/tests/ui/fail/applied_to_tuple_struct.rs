use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct MyState(i32);

// Error: tuple structs are not supported.
#[reactor(state = MyState)]
struct TupleReactor(i32);

fn main() {}
