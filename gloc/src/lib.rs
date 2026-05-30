//! # gloc
//!
//! The one-stop crate for GLOC — re-exports everything from `gloc-core`
//! and `gloc-macro` so users only need a single dependency.
//!
//! ```toml
//! [dependencies]
//! gloc = "0.1"
//! ```
//!
//! ```rust,ignore
//! use gloc::{cubit, Cubit, State};
//! ```

pub use gloc_core::*;
pub use gloc_macro::*;
