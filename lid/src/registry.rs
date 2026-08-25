use linkme::distributed_slice;

/// One registered claim.
#[derive(Debug)]
pub struct SpecMeta {
    /// `core::any::type_name` of the spec struct. The join key: every [`Edge`]
    /// produces its `spec` field from the same expression, so the two sides of
    /// a join can never disagree about naming.
    pub name: &'static str,
    /// Source file of the registration site.
    pub file: &'static str,
    /// Source line of the registration site.
    pub line: u32,
}

/// One citation: an implementation or validation site naming a claim.
#[derive(Debug)]
pub struct Edge {
    /// `core::any::type_name` of the cited spec struct (joins [`SpecMeta::name`]).
    pub spec: &'static str,
    /// Path of the citing item, for human-readable reports.
    pub item: &'static str,
    /// Source file of the citation site.
    pub file: &'static str,
    /// Source line of the citation site.
    pub line: u32,
}

/// Every claim registered by `derive(Spec)`, gathered at link time.
#[distributed_slice]
pub static SPECS: [SpecMeta];

/// Every implementation citation registered by `#[implements]`.
#[distributed_slice]
pub static IMPLEMENTATIONS: [Edge];

/// Every validation citation registered by `#[validates]`.
#[distributed_slice]
pub static VALIDATIONS: [Edge];

// Hand-authored implementation edge for the enumeration claim: the slice
// declarations above are what deliver it, and statics cannot carry
// `#[implements]`. Replaced by module-level tracing once it exists.
#[doc = "Implements [`crate::spec::LinkedRegistrationsAreEnumerable`]."]
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::IMPLEMENTATIONS)]
    static EDGE: ::lid::Edge = ::lid::Edge {
        spec: <crate::spec::LinkedRegistrationsAreEnumerable as ::lid::Spec>::NAME,
        item: module_path!(),
        file: file!(),
        line: line!(),
    };
};
