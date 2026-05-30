use gloc_macro::cubit;

// Error: no `state = T` arg and no `#[state]` fields.
#[cubit]
struct BadCubit {}

fn main() {}
