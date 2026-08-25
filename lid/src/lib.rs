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
