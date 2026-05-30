//! # gloc-full
//!
//! Umbrella crate that re-exports everything from `gloc-core` and `gloc-macro`
//! so users can add a single dependency and get the full GLOC experience.
//!
//! ```toml
//! [dependencies]
//! gloc-full = "0.1"
//! ```
//!
//! ```rust,ignore
//! use gloc_full::{cubit, Cubit, State};
//! ```

pub use gloc::*;
pub use gloc_macro::*;
