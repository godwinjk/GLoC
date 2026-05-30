//! # gloc-macro
//!
//! Procedural macros for [GLOC](https://github.com/godwinjk/gloc) — the universal business
//! logic architecture for Rust.
//!
//! ## `#[cubit]`
//!
//! The `#[cubit]` attribute macro eliminates all `Cubit` trait boilerplate.
//! It supports two modes, selectable per struct:
//!
//! ### Mode A — bring your own State type
//!
//! Supply an existing `Clone + PartialEq + Debug` type via `state = SomeType`.
//! The macro injects the infrastructure fields and generates all impls.
//!
//! ```rust,ignore
//! use gloc_macro::cubit;
//!
//! #[derive(Clone, PartialEq, Debug)]
//! pub struct CounterState { pub count: i32 }
//!
//! #[cubit(state = CounterState)]
//! pub struct CounterCubit {}
//!
//! impl CounterCubit {
//!     pub fn increment(&mut self) {
//!         let next = self.state().count + 1;
//!         self.emit(CounterState { count: next });
//!     }
//! }
//! ```
//!
//! ### Mode B — let gloc generate the State struct
//!
//! Annotate fields with `#[state]`. The macro generates a
//! `{CubitName}State` struct from those fields and wires everything up.
//! Non-`#[state]` fields remain on the cubit as private implementation details.
//!
//! ```rust,ignore
//! use gloc_macro::cubit;
//!
//! #[cubit]
//! pub struct CounterCubit {
//!     #[state] pub count: i32,
//!     step: i32,   // not managed state — stays on the cubit
//! }
//!
//! impl CounterCubit {
//!     pub fn increment(&mut self) {
//!         let next = self.state().count + self.step;
//!         self.emit(CounterCubitState { count: next });
//!     }
//! }
//! ```
//!
//! ### What is always generated
//!
//! | Generated item | Description |
//! |---|---|
//! | `impl Cubit` | `state()`, `emit()` with change-detection |
//! | `new(initial)` | Constructor (suppress with `no_new`) |
//! | `on_change(callback)` | Observer registration (suppress with `no_observers`) |
//! | tracing in `emit()` | State-transition logs (opt-in via `tracing` feature) |
//!
//! ### Attribute arguments
//!
//! | Argument | Effect |
//! |---|---|
//! | `state = SomeType` | Mode A — use this type as the managed state |
//! | `no_new` | Do not generate `new()` |
//! | `no_observers` | Do not generate `on_change()` or the listener field |

use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemStruct};

mod args;
mod codegen;
mod errors;
mod mode_a;
mod mode_b;

use args::CubitArgs;
use errors::from_syn;

/// Zero-boilerplate Cubit implementation generator.
///
/// Place `#[cubit]` (Mode B) or `#[cubit(state = SomeType)]` (Mode A) on any
/// named-field struct to have GLOC generate the full [`gloc::Cubit`] trait
/// implementation, a constructor, and an observer registration method.
///
/// See the [crate-level docs](crate) for full usage examples and the list of
/// supported arguments.
#[proc_macro_attribute]
pub fn cubit(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the raw token stream into a list of nested meta items, then into
    // our typed CubitArgs struct via darling. This is the syn 2.x approach
    // (syn::AttributeArgs was removed in syn 2).
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(syn::Error::into_compile_error(e)),
    };
    let cubit_args = match CubitArgs::from_list(&attr_args) {
        Ok(a) => a,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    // Parse the annotated struct.
    let item = parse_macro_input!(input as ItemStruct);

    // Guard: macro only supports structs (enforced by parse, but emit a
    // human-readable error for enums or other items that slip through via
    // manual TokenStream manipulation).
    let output = match &cubit_args.state {
        // Mode A — developer supplied an explicit state type.
        Some(state_path) => mode_a::expand(&item, state_path, &cubit_args),

        // Mode B — generate State from #[state] fields, or emit an error if
        // neither mode has enough information.
        None => {
            let has_state_fields = item
                .fields
                .iter()
                .any(|f| f.attrs.iter().any(|a| a.path().is_ident("state")));

            if !has_state_fields {
                from_syn(syn::Error::new_spanned(
                    &item.ident,
                    "No state type found for this cubit.\n\
                     \n\
                     Provide one of:\n\
                     • `#[cubit(state = MyStateType)]`  — use an existing State type (Mode A)\n\
                     • `#[state] field: Type` inside the struct  — let gloc generate the State (Mode B)",
                ))
            } else {
                mode_b::expand(&item, &cubit_args)
            }
        }
    };

    TokenStream::from(output)
}
