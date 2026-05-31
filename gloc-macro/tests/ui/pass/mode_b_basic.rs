use gloc::Reactor;
use gloc_macro::reactor;

#[reactor]
struct ToggleReactor {
    #[state]
    pub active: bool,
}

impl ToggleReactor {
    fn toggle(&mut self) {
        let next = !self.state().active;
        self.emit(ToggleReactorState { active: next });
    }
}

fn main() {
    let mut r = ToggleReactor::new(ToggleReactorState { active: false });
    r.toggle();
    assert!(r.state().active);
}
