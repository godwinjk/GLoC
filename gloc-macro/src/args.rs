//! Attribute argument parsing for `#[cubit(...)]`.
//!
//! Uses [`darling`] to declaratively parse the key-value arguments that the
//! developer passes inside the attribute parentheses. All fields are optional,
//! which allows the macro to distinguish "not provided" from "provided but
//! empty" and produce helpful errors for each case.
//!
//! # Supported syntax
//!
//! ```text
//! #[cubit]                            // Mode B — #[state] fields inside struct
//! #[cubit(state = MyStateType)]       // Mode A — bring-your-own state type
//! #[cubit(state = MyStateType, no_new)]         // skip new() generation
//! #[cubit(state = MyStateType, no_observers)]   // skip on_change() generation
//! #[cubit(no_new, no_observers)]      // Mode B with both suppressions
//! ```

use darling::FromMeta;
use syn::Path;

/// Parsed form of the arguments inside `#[cubit(...)]`.
///
/// Constructed by [`CubitArgs::from_list`] (derived via `darling`) from the
/// attribute's token stream. Unknown keys produce a compile-time error
/// automatically.
#[derive(Debug, Default, FromMeta)]
pub struct CubitArgs {
    /// Mode A: the existing State type to wire up.
    ///
    /// When `Some`, the macro skips State struct generation and uses this path
    /// directly as `Cubit::State`. When `None`, the macro looks for `#[state]`
    /// fields on the cubit struct (Mode B).
    pub state: Option<Path>,

    /// When `true`, suppresses generation of the `new(initial: State) -> Self`
    /// constructor. Use this when your cubit needs a custom constructor that
    /// takes extra arguments or performs extra setup.
    #[darling(default)]
    pub no_new: bool,

    /// When `true`, suppresses generation of the `on_change(callback)` observer
    /// method and the internal `__gloc_listeners` field. Use this when you do
    /// not need reactive subscriptions (e.g. a backend-only cubit).
    #[darling(default)]
    pub no_observers: bool,
}
