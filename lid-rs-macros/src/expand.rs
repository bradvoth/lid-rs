//! Expansion logic for the citation macros. The emitted forms are specified
//! by `docs/intent/registry/lld.md` (the expansion contract) and pinned by
//! `lid-rs`'s registry-content tests.

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Block, Data, DataEnum, DeriveInput, Fields, Ident, ItemEnum, ItemFn, ItemStruct,
    LitStr, Path, Token, parse_quote,
};

use crate::claim::Unwanted;

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
    let expanded = crate::claim::expansion(&item)?;
    let claim = expanded.claim;
    let free = expanded.free;
    let outcome = outcome_emission(ident, expanded.unwanted.as_ref());
    Ok(quote! {
        // The derive's own emissions reference the struct they sit on; when a
        // spec is retired with #[deprecated], only *citation* sites should
        // warn, never the definition it decorates.
        #[automatically_derived]
        #[allow(deprecated)]
        impl ::lid_rs::Spec for #ident {
            const NAME: &'static str = concat!(module_path!(), "::", stringify!(#ident));
            const FREE: bool = #free;
        }
        const _: () = {
            #[allow(deprecated, missing_docs, clippy::missing_docs_in_private_items)]
            #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::SPECS)]
            #[linkme(crate = ::lid_rs::__private::linkme)]
            static META: ::lid_rs::SpecMeta = ::lid_rs::SpecMeta {
                name: <#ident as ::lid_rs::Spec>::NAME,
                file: file!(),
                line: line!(),
                claim: #claim,
            };
        };
        #outcome
    })
}

/// E1's emission for one claim: the const block an unwanted claim naming a
/// variant carries, and no tokens at all for every other claim.
///
/// The one decision is whether the claim named a variant of something that
/// resolves as a path. An owner that is not a path names no type, so there is
/// nothing to bound and nothing to register — and the empty owner an object
/// without a variant carries is never a path, which is what makes the two
/// negative cases one case here.
fn outcome_emission(ident: &Ident, unwanted: Option<&Unwanted>) -> TokenStream {
    match unwanted.and_then(named_variant) {
        Some((owner, variant)) => outcome_bound(ident, &owner, &variant),
        None => TokenStream::new(),
    }
}

/// The owner as a path, and the identifier its object's last segment names.
///
/// `None` where either is not a path: an owner or an object the author wrote as
/// something other than one — a rustdoc disambiguator, a generic argument, the
/// empty string — names no type this emission could reach, and a claim carrying
/// one is left to the checks that read the text rather than the type.
fn named_variant(unwanted: &Unwanted) -> Option<(Path, Ident)> {
    let owner: Path = syn::parse_str(&unwanted.owner).ok()?;
    let object: Path = syn::parse_str(&unwanted.object).ok()?;
    Some((owner, object.segments.last()?.ident.clone()))
}

/// The const block an unwanted claim naming a variant carries: E1's bound in
/// its two halves, and the `CLAIMED_OWNERS` entry that stands where the bound
/// has already held.
///
/// Reading the owner's `NAME` through the trait is the first half — an enum
/// that derives nothing is `E0277`, reported at the claim — and it is the same
/// read the registration needs, so the bound is not written twice. The pattern
/// is the second half: a segment the owner does not declare as a variant is
/// `E0599`, also at the claim. That pattern is a struct pattern, which holds
/// alike for a fieldless, a tuple, and a struct variant, and it is matched
/// through an `Option` so that the arm beside it stays reachable however few
/// variants the enum declares.
fn outcome_bound(ident: &Ident, owner: &Path, variant: &Ident) -> TokenStream {
    let name = variant.to_string();
    quote! {
        const _: () = {
            const _: fn(::core::option::Option<&#owner>) -> bool = |value| {
                ::core::matches!(value, ::core::option::Option::Some(#owner::#variant { .. }))
            };
            #[allow(deprecated, missing_docs, clippy::missing_docs_in_private_items)]
            #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::CLAIMED_OWNERS)]
            #[linkme(crate = ::lid_rs::__private::linkme)]
            static OWNER: ::lid_rs::outcome::ClaimedOwner = ::lid_rs::outcome::ClaimedOwner {
                spec: <#ident as ::lid_rs::Spec>::NAME,
                owner: <#owner as ::lid_rs::outcome::Outcome>::NAME,
                variant: #name,
            };
        };
    }
}

/// Expands `derive(Outcome)`.
pub fn derive_outcome(input: TokenStream) -> syn::Result<TokenStream> {
    let item: DeriveInput = syn::parse2(input)?;
    let data = plain_enum(&item)?;
    let ident = &item.ident;
    let registrations = data.variants.iter().map(|v| outcome_registration(ident, &v.ident));
    Ok(quote! {
        #[automatically_derived]
        impl ::lid_rs::outcome::Outcome for #ident {
            const NAME: &'static str = concat!(module_path!(), "::", stringify!(#ident));
        }
        #(#registrations)*
    })
}

/// The body of a derive target that is a plain enum, or the derive's refusal.
///
/// An outcome is a fixed set of variants: a struct or a union declares none,
/// and a generic enum declares one set per instantiation while the name and the
/// registrations are one set per definition.
fn plain_enum(item: &DeriveInput) -> syn::Result<&DataEnum> {
    match &item.data {
        Data::Enum(data) if item.generics.params.is_empty() => Ok(data),
        Data::Enum(_) | Data::Struct(_) | Data::Union(_) => Err(syn::Error::new_spanned(
            &item.ident,
            "lid-rs: derive(Outcome) applies to non-generic enums only — an outcome is a fixed set of variants",
        )),
    }
}

/// One `OUTCOMES` registration, in the form the hand-written canary pins.
///
/// The key is read through the enum's own implementation rather than spelled a
/// second time, so the registration and every claim that meets it produce it
/// from one const.
fn outcome_registration(ident: &Ident, variant: &Ident) -> TokenStream {
    let name = variant.to_string();
    quote! {
        const _: () = {
            #[allow(missing_docs, clippy::missing_docs_in_private_items)]
            #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::OUTCOMES)]
            #[linkme(crate = ::lid_rs::__private::linkme)]
            static ENTRY: ::lid_rs::outcome::OutcomeMeta = ::lid_rs::outcome::OutcomeMeta {
                owner: <#ident as ::lid_rs::outcome::Outcome>::NAME,
                variant: #name,
                file: file!(),
                line: line!(),
            };
        };
    }
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
        let guard = name_guard(&f.sig.ident, &paths, &verb);
        let cited = cite_fn(f, &paths, &verb)?;
        return Ok(quote!(#cited #guard));
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

/// Check 14's guard for the cited fn, which only a validator is held to.
///
/// `#[implements]` names the code that keeps a claim and is under no rule about
/// its name; `#[validates]` names the test that observes one, and a test's name
/// is what makes `cargo test` print the requirements. See
/// [`crate::claim::validator_name`], which decides whether a guard is emitted
/// at all.
fn name_guard(ident: &syn::Ident, paths: &[Path], verb: &Verb) -> TokenStream {
    match verb {
        Verb::Validates => crate::claim::validator_name(ident, paths),
        Verb::Implements => TokenStream::new(),
    }
}

/// Refuses to cite an `async fn`, before anything is wrapped around its body.
///
/// Both wrappings assume a body that runs to completion where it is written;
/// on an `async fn` the body is a future, and the `#[implements]` guard would
/// be held across every `.await` in it, attributing every unrelated future
/// polled in the interval to the cited claim. The refusal is one unconditional
/// act for both verbs: an error spanned on the `async` keyword, or `Ok(())`
/// (`lid-rs-macros/src/lld.md`, "Citing an `async fn` is refused").
///
/// This is the layer-0 body, and it refuses nothing: the signature's
/// `asyncness` is read and `Ok(())` answered whatever it holds. It is a value
/// rather than a `todo!()` because this function runs inside every citation in
/// the workspace (the LLD's "How `refuse_async` is skeletoned"); the refusal
/// itself is the leaf `fail/async_implements.rs` is red against until it lands.
fn refuse_async(sig: &syn::Signature) -> syn::Result<()> {
    let _ = sig.asyncness;
    Ok(())
}

/// Cites a fn: the runtime observation around its body, one registration per
/// spec at the top of it, and the doc lines — uniform for free fns and
/// methods, since `impl` blocks admit no free consts but every fn body admits
/// items.
///
/// Its first act is [`refuse_async`], so a signature that cannot be wrapped is
/// an error at `citation`'s `?` rather than a wrapping that has already
/// happened.
fn cite_fn(mut f: ItemFn, paths: &[Path], verb: &Verb) -> syn::Result<TokenStream> {
    refuse_async(&f.sig)?;
    let item_expr = item_path_expr(&f.sig.ident);
    f.block = Box::new(observed(&f, paths, verb));
    for path in paths {
        let registration = edge_registration(verb, path, &item_expr);
        f.block.stmts.insert(0, parse_quote!(#registration));
    }
    f.attrs.extend(doc_attrs(verb, paths));
    Ok(f.into_token_stream())
}

/// The body a cited fn runs under: a span for the code that keeps a claim, a
/// capture for the test that observes one (README §6.4).
fn observed(f: &ItemFn, paths: &[Path], verb: &Verb) -> Block {
    match verb {
        Verb::Implements => span_around(&f.sig.ident, paths, &f.block),
        Verb::Validates => capture_around(paths, &f.block),
    }
}

/// `#[implements]`'s span: `target = "lid"`, the cited claims, and the outcome
/// recorded on the way out.
///
/// The claims are joined with `lid_rs::validate::SEPARATOR` rather than a
/// character written here, because the split at the other end is
/// `lid_rs::validate::cited_claims` and a second spelling is a silent empty
/// join rather than a compile error.
///
/// The join is written *inside* the `span!` invocation and not bound to a local
/// above it, so that it is evaluated only when the callsite is enabled.
/// `tracing` skips a disabled callsite's field expressions; a `let` above the
/// macro is a `String` allocated on every call of every cited fn in the
/// workspace, subscriber or no subscriber, which is a cost README §6.7's
/// production story cannot carry.
///
/// `lid.outcome` is reserved with `field::Empty` and recorded by a `Returned`
/// guard on drop, so a body that unwinds leaves it absent — which is what
/// `lid_rs::validate::no_panics` reads. It records `()` and not the value: a
/// rendering of the returned value needs a bound on the return type that this
/// workspace's 549 citation sites do not carry, and that is deferred with the
/// `Traceable` recording method.
fn span_around(ident: &Ident, paths: &[Path], body: &Block) -> Block {
    let name = ident.to_string();
    parse_quote!({
        let __lid_span = ::lid_rs::__private::tracing::span!(
            target: ::lid_rs::validate::TARGET,
            ::lid_rs::validate::SPAN_LEVEL,
            #name,
            lid.claims = [#(<#paths as ::lid_rs::Spec>::NAME),*]
                .join(&::lid_rs::validate::SEPARATOR.to_string())
                .as_str(),
            lid.outcome = ::lid_rs::__private::tracing::field::Empty,
        );
        let __lid_entered = __lid_span.enter();
        let __lid_returned = ::lid_rs::validate::Returned::of(&__lid_span);
        #body
    })
}

/// `#[validates]`'s capture: the test's body run under a fresh subscriber, what
/// checks 23 and 24 make of it printed, and any unwind resumed so the test
/// still fails on its own assertion.
///
/// The printing is here and not in `report`, which answers a `String` so that a
/// validator can read it. The findings do not fail the test: the ramp arrives
/// at `Ramp::Warn` (`lid-rs/src/validate/lld.md`, Deferred 8).
fn capture_around(paths: &[Path], body: &Block) -> Block {
    parse_quote!({
        let (__lid_trace, __lid_done) =
            ::lid_rs::validate::captured(|| #body);
        let __lid_report = ::lid_rs::validate::report(
            &__lid_trace,
            &[#(<#paths as ::lid_rs::Spec>::NAME),*],
        );
        if !__lid_report.is_empty() {
            ::std::eprintln!("{}", __lid_report);
        }
        match __lid_done {
            ::core::result::Result::Ok(__lid_answer) => __lid_answer,
            ::core::result::Result::Err(__lid_panic) => {
                ::std::panic::resume_unwind(__lid_panic)
            }
        }
    })
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

/// Expands `#[flow]` and `#[leaf]`: the whole expansion of both pins.
///
/// A pin is a mark `lid-rs-shape` reads from source, never from an expansion,
/// so the item's tokens are emitted unchanged and never parsed — what the
/// compiler sees is what the author wrote. Neither pin takes arguments: a
/// non-empty `args` is an error spanned on the arguments, because nothing reads
/// them and an argument accepted and ignored is a mark whose meaning the next
/// reader has to guess (`lid-rs-macros/src/lld.md`, "The shape pins").
pub fn passthrough_pin(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    todo!("passthrough_pin: args=`{args}`, item=`{item}`")
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
