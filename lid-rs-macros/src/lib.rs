#![doc = include_str!("../docs/intent/macros/lld.md")]

use proc_macro::TokenStream;

mod expand;

/// Derives `lid_rs::Spec` for a unit struct: `NAME` from the definition-site
/// module path plus the identifier, and a `SPECS` registration.
#[proc_macro_derive(Spec)]
pub fn derive_spec(input: TokenStream) -> TokenStream {
    expand::derive_spec(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Cites the specs an item implements: appends `Implements [...]` doc lines
/// and registers one `IMPLEMENTATIONS` edge per cited spec.
#[proc_macro_attribute]
pub fn implements(args: TokenStream, item: TokenStream) -> TokenStream {
    expand::citation(args.into(), item.into(), expand::Verb::Implements)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Cites the specs a test validates: appends `Validates [...]` doc lines and
/// registers one `VALIDATIONS` edge per cited spec. Use on `#[cfg(test)]`
/// unit tests inside the library — never under `tests/`, where separate
/// binaries never link into the registry.
#[proc_macro_attribute]
pub fn validates(args: TokenStream, item: TokenStream) -> TokenStream {
    expand::citation(args.into(), item.into(), expand::Verb::Validates)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Module-level tracing by containment: invoked inside a module, registers
/// one `IMPLEMENTATIONS` edge per cited spec with the enclosing module path
/// as the item.
#[proc_macro]
pub fn implements_module(input: TokenStream) -> TokenStream {
    expand::implements_module(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Attaches a foreign spec ID (compliance matrix, customer requirement) to a
/// spec struct as `#[doc(alias = "...")]`, keeping it greppable and
/// rustdoc-searchable.
#[proc_macro_attribute]
pub fn spec(args: TokenStream, item: TokenStream) -> TokenStream {
    expand::foreign_key(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
