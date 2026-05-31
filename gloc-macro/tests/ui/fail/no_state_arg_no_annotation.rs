use gloc_macro::reactor;

// Error: no `state = T` arg and no `#[state]` fields.
#[reactor]
struct BadReactor {}

fn main() {}
