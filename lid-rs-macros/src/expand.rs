//! Expansion logic for the citation macros. The emitted forms are specified
//! by `docs/intent/registry/lld.md` (the expansion contract) and pinned by
//! `lid-rs`'s registry-content tests.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DeriveInput, Fields, ItemEnum, ItemFn, ItemStruct, LitStr, Path, Token,
    parse_quote,
};

/// Which citation attribute is expanding, selecting the doc verb and the
/// registry slice.
pub enum Verb {
    /// `#[implements(...)]` → `IMPLEMENTATIONS`.
    Implements,
    /// `#[validates(...)]` → `VALIDATIONS`.
    Validates,
}

impl Verb {
    /// The doc-line verb.
    fn doc_word(&self) -> &'static str {
        match self {
            Verb::Implements => "Implements",
            Verb::Validates => "Validates",
        }
    }

    /// The registry slice the edge lands in.
    fn slice(&self) -> TokenStream {
        match self {
            Verb::Implements => quote!(::lid_rs::IMPLEMENTATIONS),
            Verb::Validates => quote!(::lid_rs::VALIDATIONS),
        }
    }
}

/// Expands `derive(Spec)`.
pub fn derive_spec(input: TokenStream) -> syn::Result<TokenStream> {
    let item: DeriveInput = syn::parse2(input)?;
    ensure_unit_struct(&item)?;
    let ident = &item.ident;
    Ok(quote! {
        // The derive's own emissions reference the struct they sit on; when a
        // spec is retired with #[deprecated], only *citation* sites should
        // warn, never the definition it decorates.
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::lid_rs::Spec for #ident {
            const NAME: &'static str = concat!(module_path!(), "::", stringify!(#ident));
        }
        const _: () = {
            #[allow(deprecated, missing_docs, clippy::missing_docs_in_private_items)]
            #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::SPECS)]
            #[linkme(crate = ::lid_rs::__private::linkme)]
            static META: ::lid_rs::SpecMeta = ::lid_rs::SpecMeta {
                name: <#ident as ::lid_rs::Spec>::NAME,
                file: file!(),
                line: line!(),
            };
        };
    })
}

/// Rejects derive targets that are not plain unit structs.
fn ensure_unit_struct(item: &DeriveInput) -> syn::Result<()> {
    let unit = matches!(&item.data, Data::Struct(s) if matches!(s.fields, Fields::Unit));
    if !unit || !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.ident,
            "lid-rs: derive(Spec) applies to non-generic unit structs only — a claim has no runtime shape",
        ));
    }
    Ok(())
}

/// Expands `#[implements]` / `#[validates]`: dispatches on the item kind.
///
/// Dispatch is by parse-attempt rather than by matching `syn::Item`: that
/// enum is foreign and `#[non_exhaustive]`, so it cannot be matched without
/// the wildcard arm check 6 denies — and rejecting unknown item kinds is the
/// wanted behaviour anyway.
pub fn citation(args: TokenStream, item: TokenStream, verb: Verb) -> syn::Result<TokenStream> {
    let paths = parse_spec_paths(args)?;
    if let Ok(f) = syn::parse2::<ItemFn>(item.clone()) {
        return Ok(cite_fn(f, &paths, &verb));
    }
    if let Ok(s) = syn::parse2::<ItemStruct>(item.clone()) {
        return Ok(cite_struct(s, &paths, &verb));
    }
    if let Ok(e) = syn::parse2::<ItemEnum>(item.clone()) {
        return Ok(cite_enum(e, &paths, &verb));
    }
    Err(syn::Error::new_spanned(
        item,
        "lid-rs: citations apply to fns, structs, and enums",
    ))
}

/// Cites a fn by injecting one registration per spec at the top of its body
/// and appending the doc lines — uniform for free fns and methods, since
/// `impl` blocks admit no free consts but every fn body admits items.
fn cite_fn(mut f: ItemFn, paths: &[Path], verb: &Verb) -> TokenStream {
    let item_expr = item_path_expr(&f.sig.ident);
    for path in paths {
        let registration = edge_registration(verb, path, &item_expr);
        f.block.stmts.insert(0, parse_quote!(#registration));
    }
    f.attrs.extend(doc_attrs(verb, paths));
    f.into_token_stream()
}

/// Cites a struct: doc lines on the item, sibling registrations after it,
/// which is legal at the module scope where struct items live.
fn cite_struct(mut s: ItemStruct, paths: &[Path], verb: &Verb) -> TokenStream {
    s.attrs.extend(doc_attrs(verb, paths));
    let registrations = sibling_registrations(&s.ident, paths, verb);
    let item = s.into_token_stream();
    quote!(#item #(#registrations)*)
}

/// Cites an enum: doc lines on the item, sibling registrations after it.
fn cite_enum(mut e: ItemEnum, paths: &[Path], verb: &Verb) -> TokenStream {
    e.attrs.extend(doc_attrs(verb, paths));
    let registrations = sibling_registrations(&e.ident, paths, verb);
    let item = e.into_token_stream();
    quote!(#item #(#registrations)*)
}

/// The per-spec registrations emitted as siblings of a non-fn item.
fn sibling_registrations(ident: &syn::Ident, paths: &[Path], verb: &Verb) -> Vec<TokenStream> {
    let item_expr = item_path_expr(ident);
    paths
        .iter()
        .map(|p| edge_registration(verb, p, &item_expr))
        .collect()
}

/// Expands `implements_module!`: edges whose item is the enclosing module.
pub fn implements_module(input: TokenStream) -> syn::Result<TokenStream> {
    let paths = parse_spec_paths(input)?;
    let item_expr = quote!(module_path!());
    let registrations = paths
        .iter()
        .map(|p| edge_registration(&Verb::Implements, p, &item_expr));
    Ok(quote!(#(#registrations)*))
}

/// Expands `#[spec("FOREIGN-ID")]`: re-emits the struct with a doc alias so
/// the foreign ID stays greppable and rustdoc-searchable.
pub fn foreign_key(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let id: LitStr = syn::parse2(args)
        .map_err(|e| syn::Error::new(e.span(), "lid-rs: expected a single foreign-ID string literal"))?;
    let mut item: DeriveInput = syn::parse2(item)?;
    item.attrs.push(parse_quote!(#[doc(alias = #id)]));
    Ok(item.into_token_stream())
}

/// Parses a citation argument list: one or more plain, non-generic paths.
fn parse_spec_paths(args: TokenStream) -> syn::Result<Vec<Path>> {
    let span = proc_macro2::Span::call_site();
    let paths: Punctuated<Path, Token![,]> =
        Punctuated::parse_terminated.parse2(args).map_err(|e| {
            syn::Error::new(e.span(), "lid-rs: expected a comma-separated list of spec paths")
        })?;
    if paths.is_empty() {
        return Err(syn::Error::new(
            span,
            "lid-rs: cite at least one spec, or remove the attribute",
        ));
    }
    paths.iter().try_for_each(ensure_plain_path)?;
    Ok(paths.into_iter().collect())
}

/// Rejects path segments carrying generic arguments.
fn ensure_plain_path(path: &Path) -> syn::Result<()> {
    if path.segments.iter().any(|s| !s.arguments.is_none()) {
        return Err(syn::Error::new_spanned(
            path,
            "lid-rs: spec paths are plain paths — a claim type takes no generic arguments",
        ));
    }
    Ok(())
}

/// Renders a path for the doc line, joining segments the way it was written.
fn render_path(path: &Path) -> String {
    let joined = path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    if path.leading_colon.is_some() {
        format!("::{joined}")
    } else {
        joined
    }
}

/// Builds the appended doc lines: one paragraph break, then one line per spec.
fn doc_attrs(verb: &Verb, paths: &[Path]) -> Vec<Attribute> {
    let mut attrs: Vec<Attribute> = vec![parse_quote!(#[doc = ""])];
    for path in paths {
        let line = format!("{} [`{}`].", verb.doc_word(), render_path(path));
        attrs.push(parse_quote!(#[doc = #line]));
    }
    attrs
}

/// The `concat!(module_path!(), "::", ident)` expression naming a cited item.
fn item_path_expr(ident: &syn::Ident) -> TokenStream {
    let name = ident.to_string();
    quote!(concat!(module_path!(), "::", #name))
}

/// One registration in the contract form from `docs/intent/registry/lld.md`.
fn edge_registration(verb: &Verb, path: &Path, item_expr: &TokenStream) -> TokenStream {
    let slice = verb.slice();
    quote! {
        const _: () = {
            #[allow(missing_docs, clippy::missing_docs_in_private_items)]
            #[::lid_rs::__private::linkme::distributed_slice(#slice)]
            #[linkme(crate = ::lid_rs::__private::linkme)]
            static EDGE: ::lid_rs::Edge = ::lid_rs::Edge {
                spec: <#path as ::lid_rs::Spec>::NAME,
                item: #item_expr,
                file: file!(),
                line: line!(),
            };
        };
    }
}
