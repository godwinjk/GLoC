//! # gloc-macro
//!
//! Procedural macros for [GLOC](https://github.com/godwinjk/gloc) — the universal business
//! logic architecture for Rust.
//!
//! ## `#[reactor_state]`
//!
//! Automatically injects the three required derives (`Clone`, `PartialEq`, `Debug`)
//! onto any state struct or enum. Extra derives can be passed via `derive(...)`.
//!
//! ```rust,ignore
//! use gloc_macro::reactor_state;
//!
//! // Required derives injected automatically
//! #[reactor_state]
//! pub struct CounterState { pub count: i32 }
//!
//! // Extra derives appended after the required three
//! #[reactor_state(derive(Hash))]
//! pub struct TaggedState { pub tag: String }
//! ```
//!
//! ## `#[reactor]`
//!
//! The `#[reactor]` attribute macro eliminates all `Reactor` trait boilerplate.
//! It supports two modes, selectable per struct:
//!
//! ### Mode A — bring your own State type
//!
//! ```rust,ignore
//! use gloc_macro::{reactor, reactor_state};
//!
//! #[reactor_state]
//! pub struct CounterState { pub count: i32 }
//!
//! #[reactor(state = CounterState)]
//! pub struct CounterReactor {}
//!
//! impl CounterReactor {
//!     pub fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//! }
//! ```
//!
//! ### Mode B — let gloc generate the State struct
//!
//! ```rust,ignore
//! use gloc_macro::reactor;
//!
//! #[reactor]
//! pub struct CounterReactor {
//!     #[state] pub count: i32,
//! }
//! ```
//!
//! ### What is always generated
//!
//! | Generated item | Description |
//! |---|---|
//! | `impl Reactor` | `state()`, `emit()` with change-detection |
//! | `new(initial)` | Constructor (suppress with `no_new`) |
//! | tracing in `emit()` | State-transition logs (opt-in via `tracing` feature) |
//!
//! ### Attribute arguments
//!
//! | Argument | Effect |
//! |---|---|
//! | `state = SomeType` | Mode A — use this type as the managed state |
//! | `no_new` | Do not generate `new()` |

use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

mod args;
mod codegen;
mod errors;
mod mode_a;
mod mode_b;
mod reactor_state;

use args::CubitArgs;
use errors::from_syn;

// ---------------------------------------------------------------------------
// #[reactor_state]
// ---------------------------------------------------------------------------

/// Automatically injects `Clone`, `PartialEq`, and `Debug` derives onto a
/// state struct or enum. Extra derives can be added via `derive(...)`.
///
/// # Examples
///
/// ```rust,ignore
/// // Required derives only
/// #[reactor_state]
/// pub struct CounterState { pub count: i32 }
///
/// // With extra derives
/// #[reactor_state(derive(Hash, serde::Serialize))]
/// pub struct CounterState { pub count: i32 }
///
/// // Works on enums too
/// #[reactor_state]
/// pub enum LoadingState { Idle, Loading, Done }
/// ```
#[proc_macro_attribute]
pub fn reactor_state(args: TokenStream, input: TokenStream) -> TokenStream {
    let state_args = match reactor_state::ReactorStateArgs::parse(args.into()) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(e.into_compile_error()),
    };

    let item = parse_macro_input!(input as DeriveInput);
    TokenStream::from(reactor_state::expand(&state_args, &item))
}

// ---------------------------------------------------------------------------
// #[reactor]
// ---------------------------------------------------------------------------

/// Zero-boilerplate Reactor implementation generator.
///
/// Place `#[reactor]` (Mode B) or `#[reactor(state = SomeType)]` (Mode A) on any
/// named-field struct to have GLoC generate the full [`gloc::Reactor`] trait
/// implementation, a constructor, and an observer registration method.
///
/// See the [crate-level docs](crate) for full usage examples and the list of
/// supported arguments.
#[proc_macro_attribute]
pub fn reactor(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(syn::Error::into_compile_error(e)),
    };
    let reactor_args = match CubitArgs::from_list(attr_args.as_slice()) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let item = parse_macro_input!(input as ItemStruct);

    let output = match &reactor_args.state {
        Some(state_path) => mode_a::expand(&item, state_path, &reactor_args),
        None => {
            let has_state_fields = item
                .fields
                .iter()
                .any(|f| f.attrs.iter().any(|a| a.path().is_ident("state")));

            if !has_state_fields {
                from_syn(syn::Error::new_spanned(
                    &item.ident,
                    "No state type found for this reactor.\n\
                     \n\
                     Provide one of:\n\
                     • `#[reactor(state = MyStateType)]`  — use an existing State type (Mode A)\n\
                     • `#[state] field: Type` inside the struct  — let gloc generate the State (Mode B)",
                ))
            } else {
                mode_b::expand(&item, &reactor_args)
            }
        }
    };

    TokenStream::from(output)
}
