//! All cubits for the v0.2 counter-dioxus example.
//! Each cubit is UI-agnostic; the macro generates all trait boilerplate.

pub mod counter;
pub use counter::{CounterCubit, CounterState};
