use gloc_macro::cubit;

#[derive(Clone, PartialEq, Debug)]
struct MyState(i32);

// Error: tuple structs are not supported.
#[cubit(state = MyState)]
struct TupleCubit(i32);

fn main() {}
