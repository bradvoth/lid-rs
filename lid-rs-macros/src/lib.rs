#![doc = include_str!("lld.md")]

use proc_macro::TokenStream;

mod claim;
mod expand;

/// Derives `lid_rs::Spec` for a unit struct: `NAME` from the definition-site
/// module path plus the identifier, and a `SPECS` registration.
///
/// `#[lid(free)]` is the derive's helper attribute, inert until the derive
/// reads it: it marks a claim free of the controlled language, and the registry
/// counts the marks.
#[proc_macro_derive(Spec, attributes(lid))]
pub fn derive_spec(input: TokenStream) -> TokenStream {
    expand::derive_spec(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Derives `lid_rs::outcome::Outcome` for an enum: `NAME` from the
/// definition-site module path plus the enum's identifier, and one `OUTCOMES`
/// registration per declared variant, each keyed by the name read back through
/// that implementation.
///
/// It applies to non-generic enums and to nothing else — an outcome is a fixed
/// set of variants, and any other target is a compile error at the item.
#[proc_macro_derive(Outcome)]
pub fn derive_outcome(input: TokenStream) -> TokenStream {
    expand::derive_outcome(input.into())
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

/// Pins a fn as routing: declares that its body must stay a flow decision, so
/// that the shape pass fails when it acquires work. Takes no arguments and
/// emits the fn unchanged — the mark is read from source by `lid-rs-shape`,
/// never from this expansion. Accepted on free fns, inherent methods, trait
/// method declarations and trait-impl methods; not on closures, which are
/// classified by the fn containing them.
#[proc_macro_attribute]
pub fn flow(args: TokenStream, item: TokenStream) -> TokenStream {
    expand::passthrough_pin(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Pins a fn as a leaf: declares that a public fn's work is intended, so that
/// rule B allows and counts it. Takes no arguments and emits the fn unchanged
/// — the mark is read from source by `lid-rs-shape`, never from this
/// expansion. Accepted in the same four fn positions as [`flow`].
#[proc_macro_attribute]
pub fn leaf(args: TokenStream, item: TokenStream) -> TokenStream {
    expand::passthrough_pin(args.into(), item.into())
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
