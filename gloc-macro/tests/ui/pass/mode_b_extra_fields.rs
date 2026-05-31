use gloc::Reactor;
use gloc_macro::reactor;

// Non-#[state] fields (`step`) stay on the reactor, not in the generated state.
#[reactor]
struct StepReactor {
    #[state]
    pub count: i32,
    pub step: i32,
}

impl StepReactor {
    fn advance(&mut self) {
        let next = self.state().count + self.step;
        self.emit(StepReactorState { count: next });
    }
}

fn main() {
    let mut r = StepReactor::new(5, StepReactorState { count: 0 });
    r.advance();
    assert_eq!(r.state().count, 5);
}
