use gloc::Cubit;
use gloc_macro::cubit;

#[cubit]
struct ToggleCubit {
    #[state]
    pub active: bool,
}

impl ToggleCubit {
    fn toggle(&mut self) {
        let next = !self.state().active;
        self.emit(ToggleCubitState { active: next });
    }
}

fn main() {
    let mut c = ToggleCubit::new(ToggleCubitState { active: false });
    c.toggle();
    assert!(c.state().active);
}
