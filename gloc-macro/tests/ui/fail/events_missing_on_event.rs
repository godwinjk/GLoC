use gloc_macro::reactor;

#[derive(Clone, PartialEq, Debug)]
struct S(i32);

#[derive(Debug)]
enum Ev {
    Inc,
}

// Error: fire() is generated, but the user forgot to write on_event().
// Rust will report: no method named `on_event` found for struct `R`.
#[reactor(state = S, neutrons = Ev)]
struct R {}

fn main() {
    let mut r = R::new(S(0));
    r.fire(Ev::Inc); // compile error — on_event not defined
}
