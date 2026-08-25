#![doc = include_str!("../../docs/intent/hld.md")]

// Macro expansions address this crate as `::lid::…`; this makes that path
// resolve inside `lid` itself. Consequence: downstream renames of the
// dependency are unsupported (see the registry LLD).
extern crate self as lid;

pub mod canary;
#[doc = include_str!("../../docs/intent/registry/lld.md")]
pub mod registry;
pub mod spec;

pub use registry::{Edge, IMPLEMENTATIONS, SPECS, SpecMeta, VALIDATIONS};

pub use lid_macros::{Spec, implements, implements_module, spec, validates};

// Hand-authored implementation edges for the citation claims: lid-macros is a
// proc-macro crate, which links into no target binary and so can neither
// carry citations nor register anything — its edges live here, at the
// re-export boundary that is its public surface. This is the standing
// exception for proc-macro crates, not bootstrap residue.
#[doc = "Implements [`crate::spec::DerivedSpecsCarryTheirDefinitionPath`], \
[`crate::spec::DerivedSpecsRegisterIntoSpecs`], \
[`crate::spec::ImplementsCitationsRegisterEdges`], \
[`crate::spec::ValidatesCitationsRegisterEdges`], \
[`crate::spec::ModuleCitationsTraceByContainment`], \
[`crate::spec::MalformedCitationsFailToCompile`]."]
const _: () = {
    /// One hand edge per (claim, macro item) pair.
    macro_rules! macro_edge {
        ($spec:path, $item:literal) => {
            const _: () = {
                #[allow(missing_docs, clippy::missing_docs_in_private_items)]
                #[::lid::__private::linkme::distributed_slice(::lid::IMPLEMENTATIONS)]
                static EDGE: ::lid::Edge = ::lid::Edge {
                    spec: <$spec as ::lid::Spec>::NAME,
                    item: $item,
                    file: file!(),
                    line: line!(),
                };
            };
        };
    }
    macro_edge!(crate::spec::DerivedSpecsCarryTheirDefinitionPath, "lid_macros::Spec");
    macro_edge!(crate::spec::DerivedSpecsRegisterIntoSpecs, "lid_macros::Spec");
    macro_edge!(crate::spec::ImplementsCitationsRegisterEdges, "lid_macros::implements");
    macro_edge!(crate::spec::ValidatesCitationsRegisterEdges, "lid_macros::validates");
    macro_edge!(crate::spec::ModuleCitationsTraceByContainment, "lid_macros::implements_module");
    macro_edge!(crate::spec::MalformedCitationsFailToCompile, "lid_macros::expand");
};

/// Trait implemented by every claim item, via `derive(Spec)`.
///
/// The registration statics emitted by `#[implements]` and `#[validates]`
/// produce their join key as `<cited::Path as lid::Spec>::NAME`, which is what
/// turns a citation of a renamed or deleted claim into a compile error (and
/// surfaces `#[deprecated]` on the spec at every citation site).
pub trait Spec {
    /// Canonical name of the claim: definition-site `module_path!()` plus the
    /// item's identifier. The registry join key — single-sourced here, so
    /// every citation agrees on it no matter which path or re-export the
    /// citing site wrote.
    const NAME: &'static str;
}

#[doc(hidden)]
pub mod __private {
    pub use linkme;
}
