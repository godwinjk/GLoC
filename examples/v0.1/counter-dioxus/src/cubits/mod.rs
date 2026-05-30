//! All cubit modules for the counter-dioxus example.
//!
//! Each cubit lives in its own file and is entirely UI-agnostic.
//! The `main.rs` wires them into Dioxus signals; the cubits themselves
//! know nothing about the rendering layer.

pub mod counter;

pub use counter::{CounterCubit, CounterState};
