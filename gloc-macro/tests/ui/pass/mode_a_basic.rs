use gloc::Cubit;
use gloc_macro::cubit;

#[derive(Clone, PartialEq, Debug)]
struct CounterState { pub count: i32 }

#[cubit(state = CounterState)]
struct CounterCubit {}

impl CounterCubit {
    fn increment(&mut self) {
        let next = self.state().count + 1;
        self.emit(CounterState { count: next });
    }
}

fn main() {
    let mut c = CounterCubit::new(CounterState { count: 0 });
    c.increment();
    assert_eq!(c.state().count, 1);
}
