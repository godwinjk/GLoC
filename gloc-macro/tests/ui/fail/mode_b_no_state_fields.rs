use gloc_macro::cubit;

// Error: #[cubit] with no `#[state]` fields and no `state = T` arg.
#[cubit]
struct NoStateCubit {
    regular_field: i32,
}

fn main() {}
