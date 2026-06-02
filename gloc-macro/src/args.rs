//! Attribute argument parsing for `#[reactor(...)]`.
//!
//! Uses [`darling`] to declaratively parse the key-value arguments that the
//! developer passes inside the attribute parentheses. All fields are optional,
//! which allows the macro to distinguish "not provided" from "provided but
//! empty" and produce helpful errors for each case.
//!
//! # Supported syntax
//!
//! ```text
//! #[reactor]                                          // Mode B — #[state] fields inside struct
//! #[reactor(state = MyStateType)]                     // Mode A — bring-your-own state type
//! #[reactor(state = MyStateType, neutrons = MyEvent)]   // Mode A + event dispatch
//! #[reactor(state = MyStateType, no_new)]             // skip new() generation
//! ```

use darling::FromMeta;
use syn::Path;

/// Parsed form of the arguments inside `#[reactor(...)]`.
///
/// Constructed by [`CubitArgs::from_list`] (derived via `darling`) from the
/// attribute's token stream. Unknown keys produce a compile-time error
/// automatically.
#[derive(Debug, Default, FromMeta)]
pub struct CubitArgs {
    /// Mode A: the existing State type to wire up.
    ///
    /// When `Some`, the macro skips State struct generation and uses this path
    /// directly as `Reactor::State`. When `None`, the macro looks for `#[state]`
    /// fields on the reactor struct (Mode B).
    pub state: Option<Path>,

    /// Optional event enum type for event-driven dispatch.
    ///
    /// When `Some`, the macro generates `pub fn dispatch(&mut self, event: E)`
    /// which calls `self.on_event(event)`. The user must write `fn on_event`
    /// in any impl block — the macro does not generate the handler body.
    pub neutrons: Option<Path>,

    /// When `true`, suppresses generation of the `new(initial: State) -> Self`
    /// constructor. Use this when your reactor needs a custom constructor that
    /// takes extra arguments or performs extra setup.
    #[darling(default)]
    pub no_new: bool,
}
