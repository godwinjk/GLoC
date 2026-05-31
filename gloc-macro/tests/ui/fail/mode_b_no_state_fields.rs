use gloc_macro::reactor;

// Error: #[reactor] with no `#[state]` fields and no `state = T` arg.
#[reactor]
struct NoStateReactor {
    regular_field: i32,
}

fn main() {}
