use linkme::distributed_slice;

/// One registered claim.
#[derive(Debug)]
pub struct SpecMeta {
    /// The spec struct's [`Spec::NAME`](crate::Spec::NAME). The join key:
    /// every [`Edge`] produces its `spec` field from the same associated
    /// const, so the two sides of a join can never disagree about naming.
    pub name: &'static str,
    /// Source file of the registration site.
    pub file: &'static str,
    /// Source line of the registration site.
    pub line: u32,
}

/// One citation: an implementation or validation site naming a claim.
#[derive(Debug)]
pub struct Edge {
    /// The cited spec struct's [`Spec::NAME`](crate::Spec::NAME) (joins
    /// [`SpecMeta::name`]).
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

// The slice declarations above are what deliver the enumeration claim, and
// statics cannot carry `#[implements]` — the whole module is traced by
// containment instead.
lid::implements_module!(crate::spec::LinkedRegistrationsAreEnumerable);
