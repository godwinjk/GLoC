//! Mode A code generation — bring-your-own State type.
//!
//! Activated when the developer supplies `#[cubit(state = SomeType)]`.
//! The State struct is **not** generated; the developer is responsible for
//! defining it (with `Clone + PartialEq + Debug`) wherever they like.
//!
//! ## What this module generates
//!
//! Given:
//! ```rust,ignore
//! #[derive(Clone, PartialEq, Debug)]
//! struct CounterState { count: i32 }
//!
//! #[cubit(state = CounterState)]
//! struct CounterCubit {}
//! ```
//!
//! It produces:
//! ```rust,ignore
//! struct CounterCubit {
//!     __gloc_state: CounterState,
//!     __gloc_listeners: Vec<Box<dyn Fn(&CounterState)>>,
//! }
//!
//! impl ::gloc::Cubit for CounterCubit { ... }
//! impl CounterCubit { pub fn new(...) -> Self { ... } }
//! impl CounterCubit { pub fn on_change(...) { ... } }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, Path, Type};

use crate::args::CubitArgs;
use crate::codegen::{impl_cubit, impl_new, impl_on_change, path_to_type};
use crate::errors::error;

/// Entry point for Mode A expansion.
///
/// # Parameters
///
/// - `item`       — the parsed cubit struct from the user's source.
/// - `state_path` — the path to the user-supplied State type (from `state = X`).
/// - `args`       — the full parsed attribute args (for `no_new`, `no_observers`).
///
/// # Errors
///
/// Returns a `compile_error!` token stream if:
/// - The struct uses unnamed (tuple) fields — named fields are required.
/// - Both `state = X` and `#[state]` inner fields are present (mode conflict).
pub fn expand(item: &ItemStruct, state_path: &Path, args: &CubitArgs) -> TokenStream {
    // Guard: tuple structs are not supported.
    if matches!(item.fields, Fields::Unnamed(_)) {
        return error(
            &item.fields,
            "#[cubit] does not support tuple structs. Use named fields: `struct Foo { ... }`.",
        );
    }

    // Guard: mode conflict — #[state] inner fields alongside state = X.
    let has_state_fields = item.fields.iter().any(|f| {
        f.attrs.iter().any(|a| a.path().is_ident("state"))
    });
    if has_state_fields {
        return error(
            item,
            "#[cubit] conflict: `state = SomeType` and `#[state]` fields cannot be used together. \
             Pick one: either supply `state = SomeType` (Mode A) or annotate fields with \
             `#[state]` (Mode B).",
        );
    }

    let state_type: Type = path_to_type(state_path);
    let struct_name = &item.ident;
    let vis = &item.vis;
    let attrs = item.attrs.iter().filter(|a| !a.path().is_ident("cubit"));
    let has_observers = !args.no_observers;

    // Collect user-defined fields (none for Mode A typically, but allowed).
    let named_user_fields: Vec<syn::Field> = match &item.fields {
        Fields::Named(f) => f.named.iter().cloned().collect(),
        Fields::Unit => vec![],
        Fields::Unnamed(_) => unreachable!(),
    };
    let user_fields = {
        let fields = named_user_fields.iter();
        quote! { #(#fields,)* }
    };

    // Injected infrastructure fields.
    let state_field = quote! {
        /// Internal state storage — managed by `#[cubit]`. Do not access directly.
        __gloc_state: #state_type,
    };

    let listeners_field = if has_observers {
        quote! {
            /// Registered `on_change` callbacks — managed by `#[cubit]`. Do not access directly.
            __gloc_listeners: ::std::vec::Vec<::std::boxed::Box<dyn Fn(&#state_type)>>,
        }
    } else {
        quote! {}
    };

    // Rewrite the struct with injected fields appended.
    let rewritten_struct = quote! {
        #(#attrs)*
        #vis struct #struct_name {
            #user_fields
            #state_field
            #listeners_field
        }
    };

    let cubit_impl    = impl_cubit(struct_name, &state_type, has_observers);
    let new_impl      = if args.no_new { quote! {} } else { impl_new(struct_name, &state_type, has_observers, &named_user_fields) };
    let observer_impl = if has_observers { impl_on_change(struct_name, &state_type) } else { quote! {} };

    quote! {
        #rewritten_struct
        #cubit_impl
        #new_impl
        #observer_impl
    }
}
