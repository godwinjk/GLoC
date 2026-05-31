//! `#[reactor_state]` — automatic derive injection for GLoC state structs.
//!
//! Every GLoC state type requires `Clone + PartialEq + Debug`. Writing
//! `#[derive(Clone, PartialEq, Debug)]` every time is repetitive and
//! easy to forget. `#[reactor_state]` injects those three derives automatically,
//! and lets the developer pass additional derives via `derive(...)` without
//! repeating the mandatory ones.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use gloc_macro::reactor_state;
//!
//! // Required derives injected automatically
//! #[reactor_state]
//! struct CounterState { pub count: i32 }
//!
//! // Extra derives appended after the required three
//! #[reactor_state(derive(Hash, serde::Serialize))]
//! struct TaggedState { pub tag: u32 }
//!
//! // Works on enums too
//! #[reactor_state]
//! enum LoadingState { Idle, Loading, Done }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, DeriveInput, Meta, Path, Token};

use crate::errors::error;

// ---------------------------------------------------------------------------
// Arg parsing
// ---------------------------------------------------------------------------

/// Extra derive paths extracted from `#[reactor_state(derive(Path, Path, ...))]`.
///
/// Uses manual `syn` parsing because the `derive(...)` list syntax is not
/// supported by darling's `multiple` — darling expects `key = value` pairs,
/// not `key(value, value)` lists.
pub struct ReactorStateArgs {
    pub extras: Vec<Path>,
}

impl ReactorStateArgs {
    /// Parses the raw attribute token stream into `ReactorStateArgs`.
    ///
    /// Accepts:
    /// - Empty args: `#[reactor_state]`
    /// - Derive list: `#[reactor_state(derive(Hash, serde::Serialize))]`
    pub fn parse(tokens: proc_macro2::TokenStream) -> syn::Result<Self> {
        // Empty args — nothing to parse
        if tokens.is_empty() {
            return Ok(Self { extras: vec![] });
        }

        // Parse as a single Meta::List — `derive(Hash, ...)`
        let meta: Meta = syn::parse2(quote! { dummy(#tokens) })?;
        let mut extras = vec![];

        if let Meta::List(list) = meta {
            // Iterate the top-level items — should be `derive(...)`
            let nested = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

            for item in nested {
                match item {
                    Meta::List(ref inner) if inner.path.is_ident("derive") => {
                        // Parse the paths inside derive(...)
                        let paths = inner
                            .parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
                        extras.extend(paths);
                    }
                    other => {
                        return Err(syn::Error::new_spanned(
                            other,
                            "#[reactor_state] only accepts `derive(...)` as an argument.\n\
                             Example: `#[reactor_state(derive(Hash, serde::Serialize))]`",
                        ));
                    }
                }
            }
        }

        Ok(Self { extras })
    }
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Entry point — expands `#[reactor_state(...)]` on a struct or enum.
///
/// # What it generates
///
/// ```rust,ignore
/// // Input
/// #[reactor_state(derive(Hash))]
/// struct MyState { pub value: i32 }
///
/// // Output
/// #[derive(Clone, PartialEq, Debug, Hash)]
/// struct MyState { pub value: i32 }
/// ```
pub fn expand(args: &ReactorStateArgs, input: &DeriveInput) -> TokenStream {
    // Guard: unions are not valid state types.
    if matches!(input.data, syn::Data::Union(_)) {
        return error(
            &input.ident,
            "#[reactor_state] does not support unions. Use a struct or enum.",
        );
    }

    let extra_derives = &args.extras;

    // Build the derive attribute — required three + user extras.
    let derive_attr = if extra_derives.is_empty() {
        quote! {
            #[derive(Clone, PartialEq, Debug)]
        }
    } else {
        quote! {
            #[derive(Clone, PartialEq, Debug, #(#extra_derives),*)]
        }
    };

    // Preserve all other attributes on the item (strip only #[reactor_state]).
    let attrs: Vec<_> = input
        .attrs
        .iter()
        .filter(|a| !a.path().is_ident("reactor_state"))
        .collect();

    let vis = &input.vis;
    let ident = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match &input.data {
        syn::Data::Struct(data) => {
            let fields = &data.fields;
            quote! {
                #derive_attr
                #(#attrs)*
                #vis struct #ident #impl_generics #where_clause #fields
            }
        }
        syn::Data::Enum(data) => {
            let variants = data.variants.iter();
            quote! {
                #derive_attr
                #(#attrs)*
                #vis enum #ident #ty_generics #where_clause {
                    #(#variants),*
                }
            }
        }
        syn::Data::Union(_) => unreachable!(),
    }
}
