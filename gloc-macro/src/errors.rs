//! Compile-time diagnostic helpers for the `#[cubit]` macro.
//!
//! All error messages are authored here in one place so they stay consistent,
//! easy to search for in the source, and simple to update if wording improves.

use proc_macro2::TokenStream;
use quote::quote_spanned;
use syn::spanned::Spanned;

/// Emits a `compile_error!` token stream pointing at `node`.
///
/// # Parameters
///
/// - `node`    — the AST node whose span is used for the error highlight.
/// - `message` — the human-readable error message shown to the developer.
pub fn error<T: Spanned>(node: &T, message: &str) -> TokenStream {
    let span = node.span();
    quote_spanned! { span => compile_error!(#message); }
}

/// Converts a [`syn::Error`] into a compile-error token stream.
///
/// Use this at the top-level macro entry point to turn any `Err` from
/// internal parsing into a proper compiler diagnostic.
pub fn from_syn(err: syn::Error) -> TokenStream {
    err.to_compile_error()
}
