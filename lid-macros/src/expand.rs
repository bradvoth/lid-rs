//! Expansion logic for the citation macros. The emitted forms are specified
//! by `docs/intent/registry/lld.md` (the expansion contract) and pinned by
//! `lid`'s registry-content tests.

use proc_macro2::TokenStream;

/// Which citation attribute is expanding, selecting the doc verb and the
/// registry slice.
pub enum Verb {
    /// `#[implements(...)]` → `IMPLEMENTATIONS`.
    Implements,
    /// `#[validates(...)]` → `VALIDATIONS`.
    Validates,
}

/// Expands `derive(Spec)`.
pub fn derive_spec(input: TokenStream) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}

/// Expands `#[implements]` / `#[validates]`.
pub fn citation(args: TokenStream, item: TokenStream, verb: Verb) -> syn::Result<TokenStream> {
    let _ = (args, item, verb);
    todo!()
}

/// Expands `implements_module!`.
pub fn implements_module(input: TokenStream) -> syn::Result<TokenStream> {
    let _ = input;
    todo!()
}

/// Expands `#[spec("FOREIGN-ID")]`.
pub fn foreign_key(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let _ = (args, item);
    todo!()
}
