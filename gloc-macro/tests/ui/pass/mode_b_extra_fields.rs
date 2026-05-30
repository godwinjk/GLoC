use gloc::Cubit;
use gloc_macro::cubit;

// Non-#[state] fields (`step`) stay on the cubit, not in the generated state.
#[cubit]
struct StepCubit {
    #[state]
    pub count: i32,
    pub step: i32,
}

impl StepCubit {
    fn advance(&mut self) {
        let next = self.state().count + self.step;
        self.emit(StepCubitState { count: next });
    }
}

fn main() {
    let mut c = StepCubit::new(5, StepCubitState { count: 0 });
    c.advance();
    assert_eq!(c.state().count, 5);
}
