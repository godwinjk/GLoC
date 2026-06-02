//! Mode A code generation — bring-your-own State type.
//!
//! Activated when the developer supplies `#[reactor(state = SomeType)]`.
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
//! #[reactor(state = CounterState)]
//! struct CounterReactor {}
//! ```
//!
//! It produces:
//! ```rust,ignore
//! struct CounterReactor {
//!     __gloc_state: CounterState,
//!     __gloc_stream: ::gloc::GlocStream<CounterState>,
//! }
//!
//! impl ::gloc::Reactor for CounterReactor { ... }
//! impl CounterReactor { pub fn new(...) -> Self { ... } }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, Path, Type};

use crate::args::CubitArgs;
use crate::codegen::{impl_deref, impl_fire, impl_new, impl_reactor, path_to_type};
use crate::errors::error;

/// Entry point for Mode A expansion.
///
/// # Parameters
///
/// - `item`       — the parsed reactor struct from the user's source.
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
            "#[reactor] does not support tuple structs. Use named fields: `struct Foo { ... }`.",
        );
    }

    // Guard: mode conflict — #[state] inner fields alongside state = X.
    let has_state_fields = item
        .fields
        .iter()
        .any(|f| f.attrs.iter().any(|a| a.path().is_ident("state")));
    if has_state_fields {
        return error(
            &item.ident,
            "#[reactor] conflict: `state = SomeType` and `#[state]` fields cannot be used together. \
             Pick one: either supply `state = SomeType` (Mode A) or annotate fields with \
             `#[state]` (Mode B).",
        );
    }

    let state_type: Type = path_to_type(state_path);
    let struct_name = &item.ident;
    let vis = &item.vis;
    let attrs = item.attrs.iter().filter(|a| !a.path().is_ident("reactor"));

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
    // `__gloc_state` holds the current state value.
    // `__gloc_stream` is always present — `GlocObserver` uses it via `emit()`.
    let state_field = quote! {
        /// Internal state storage — managed by `#[reactor]`. Do not access directly.
        __gloc_state: #state_type,
    };

    let stream_field = quote! {
        /// Reactive stream — managed by `#[reactor]`. Do not access directly.
        __gloc_stream: ::gloc::GlocStream<#state_type>,
    };

    // Rewrite the struct with injected fields appended.
    let rewritten_struct = quote! {
        #(#attrs)*
        #vis struct #struct_name {
            #user_fields
            #state_field
            #stream_field
        }
    };

    let reactor_impl = impl_reactor(struct_name, &state_type, true);
    let new_impl = if args.no_new {
        quote! {}
    } else {
        impl_new(struct_name, &state_type, true, &named_user_fields)
    };

    let deref_impl = impl_deref(struct_name, &state_type);
    let fire_impl = if let Some(event_path) = &args.neutrons {
        let neutron_type = path_to_type(event_path);
        impl_fire(struct_name, &neutron_type)
    } else {
        quote! {}
    };

    quote! {
        #rewritten_struct
        #reactor_impl
        #deref_impl
        #new_impl
        #fire_impl
    }
}
