#![doc = include_str!("lld.md")]
use crate::graph::CanaryStripped;
use crate::registry::{Edge, SpecMeta};
use lid_rs::implements;
use std::path::{Path, PathBuf};

/// Where a package's generated trace document stands, relative to that
/// package's own root.
///
/// Data, and uncited: the claim about where the document goes is
/// [`TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory`](spec::TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory),
/// cited on [`document_path`], which this feeds. A claim whose only implementer
/// is a `const` is true from the skeleton onward, so no test of it could be red
/// before the leaf exists.
pub const DOCUMENT: &str = "docs/intent/trace.md";

/// One count line's material: how many of a set of claims are held to the
/// controlled language, and how many carry the free mark.
///
/// The shape a count section takes, fixed here so that the shape pass fills the
/// same shape when it has a caller. The free ramp is the one count the registry
/// can answer today, from
/// [`ClaimMeta::language`](crate::claim::ClaimMeta::language).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ledger {
    /// How many of the claims carry
    /// [`Language::Controlled`](crate::claim::Language::Controlled).
    pub controlled: usize,
    /// How many of the claims carry
    /// [`Language::Free`](crate::claim::Language::Free).
    pub free: usize,
}

/// One slice's part of the document: the section's name, the claims it holds in
/// the order their rows are written, and the section's own [`Ledger`].
///
/// The claims are borrowed from the registrations [`sections`] was given, which
/// is why the type carries their lifetime: a [`SpecMeta`] is neither `Clone` nor
/// `Copy`, and copying one here would be a second answer to what the registry
/// already holds.
///
/// The claims and not their rendered rows: [`sections`] is given no citation
/// edges, so it cannot render a row at all. [`render`] passes each of these to
/// [`row`] with the edges it holds.
#[derive(Debug)]
pub struct Section<'a> {
    /// The section's name, as the claims' own names spell the slice — the
    /// crate's name where that spelling is empty.
    pub name: String,
    /// The claims of this section, in the order their rows are written.
    pub claims: Vec<&'a SpecMeta>,
    /// The [`ledger`] of exactly the claims above.
    pub ledger: Ledger,
}

/// Where the invoking package's document stands: [`DOCUMENT`] joined onto that
/// package's manifest directory.
///
/// The emitted tests pass `env!("CARGO_MANIFEST_DIR")`, which is what makes the
/// document per-package without any setting being read: the macro expands while
/// the invoking crate is compiled, so each member addresses its own root.
#[implements(spec::TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory)]
pub fn document_path(manifest_dir: &str) -> PathBuf {
    todo!("the trace document of the package at {manifest_dir}")
}

/// The whole document for one crate, rendered from that crate's registrations.
///
/// Canary-first: a registry whose triple was stripped renders as a crate with no
/// claims, and regenerating from that would erase every row of a committed file
/// while check 26 demanded the erasure. So a stripped registry is refused with
/// [`CanaryStripped`] rather than answered, reusing the graph slice's type
/// because a second stripped-registry error would be a second answer to one
/// question.
///
/// Below the guard it composes the header — which carries the qualification that
/// a free claim's row holds none of that claim's wording — the ledger line over
/// all the crate's claims, and one part per [`Section`], each naming its own
/// [`Ledger`] and carrying the [`row`] of every claim in it.
///
/// Its one other decision is the empty case: a crate whose own claims are none
/// renders the empty string, not a header standing over no rows. That is what
/// makes an absent document a passing state for a freshly scaffolded package
/// rather than a special case written into the check.
///
/// Parameterized over the slices as the graph slice's checks are, so every
/// branch is reachable from synthetic registrations while the emitted tests
/// apply this same function to the real registries.
#[implements(
    spec::RenderRefusesAStrippedRegistry,
    spec::ACrateWithNoClaimsRendersTheEmptyDocument,
    spec::TheHeaderNamesThatAFreeClaimsRowHoldsNoWording,
    spec::TheDocumentNamesEverySectionItsClaimsFallInto,
    spec::TheDocumentCarriesTheRowOfEveryClaimOfTheCrate,
    spec::TheDocumentNamesEachSectionsLedgerBelowThatSectionsName,
    spec::TheDocumentNamesTheLedgerOfAllItsClaimsAboveItsSections,
    spec::OneRegistryRendersOneDocumentWhateverItsOrder,
)]
pub fn render(
    crate_name: &str,
    specs: &[SpecMeta],
    impls: &[Edge],
    validations: &[Edge],
) -> Result<String, CanaryStripped> {
    todo!("the document of {crate_name} over {specs:?}, {impls:?} and {validations:?}")
}

/// The invoking crate's claims, grouped into the sections their names spell.
///
/// Scoped to `crate_name` by the [`SpecMeta::name`] prefix, as
/// [`graph_orphans`](crate::graph::graph_orphans) is: a consumer's binary links
/// this crate's registrations beside its own, and an unscoped document would
/// report one crate's claims in another crate's page.
///
/// Each group is named by [`slice_of`], taking the crate's own name where that
/// is empty, and carries the [`ledger`] of exactly its own claims. The claims
/// within a group are sorted by name and the groups by section, because
/// `linkme` promises no order across link units and a freshness comparison
/// requires that one registry render one byte sequence.
///
/// A `Vec` and not an `impl Iterator`: nothing downstream depends on laziness,
/// and `!` coerces to a `Vec` where it implements no trait — which is what lets
/// this signature stand before its body does.
#[implements(
    spec::SectionsGroupTheCratesClaimsByTheirSlice,
    spec::SectionsScopeToTheCrateTheyWereGiven,
    spec::ACrateRootSlicesSectionIsNamedForTheCrate,
    spec::EachSectionCarriesTheLedgerOfItsOwnClaims,
    spec::OneRegistryRendersOneDocumentWhateverItsOrder,
)]
pub fn sections<'a>(crate_name: &str, specs: &'a [SpecMeta]) -> Vec<Section<'a>> {
    todo!("the sections of {crate_name} over {specs:?}")
}

/// The slice a claim's name spells: the segments between the crate segment and
/// the trailing `spec` and claim segments.
///
/// `graph` from `lid_rs::graph::spec::X`, and empty from `cargo_lid_rs::spec::X`
/// — a crate-root slice, whose section [`sections`] names for the crate instead.
///
/// Read from the name and never from [`SpecMeta::file`], so the document assumes
/// nothing about where a slice's files stand.
#[implements(
    spec::TheSliceIsTheSegmentsBetweenTheCrateAndTheSpecModule,
    spec::ACrateRootClaimsSliceIsEmpty,
)]
pub fn slice_of(name: &str) -> &str {
    todo!("the slice segments of {name}")
}

/// One claim's row: its name, the [`Pattern`](crate::claim::Pattern) it carries,
/// whether it is held to the language or marked free, and the items that
/// implement and validate it.
///
/// The two edge sets are passed separately because a row names them separately;
/// each is turned into text by [`citations`].
#[implements(
    spec::ARowNamesItsClaim,
    spec::ARowNamesItsClaimsPattern,
    spec::ARowNamesAFreeClaimAsFree,
    spec::ARowDoesNotNameAHeldClaimAsFree,
    spec::ARowCarriesItsClaimsImplementingItems,
    spec::ARowCarriesItsClaimsValidatingItems,
)]
pub fn row(meta: &SpecMeta, impls: &[Edge], validations: &[Edge]) -> String {
    todo!("the row of {meta:?} among {impls:?} and {validations:?}")
}

/// The items in `edges` citing `spec_name`, each written `item (file:line)`,
/// sorted.
///
/// One function for both edge sets, which is why it takes the set rather than
/// naming which set it was given: what a citation says is the same on either
/// side of a row, and an `EdgeKind` here would be a decision with no consequence.
///
/// Plain text and not a link: the distance from a registered file to the
/// document is a property of the build layout, so a generator that discovered it
/// would make the rendered bytes depend on where they were rendered.
#[implements(
    spec::ACitationNamesTheCitingItemWithItsSite,
    spec::ACitationIsNotMadeForAnotherClaimsEdge,
    spec::OneRegistryRendersOneDocumentWhateverItsOrder,
)]
pub fn citations(spec_name: &str, edges: &[Edge]) -> Vec<String> {
    todo!("the citations of {spec_name} among {edges:?}")
}

/// The controlled and free counts over a set of claims, read from
/// [`ClaimMeta::language`](crate::claim::ClaimMeta::language).
///
/// The ramp the registry can answer today: the free mark is on the registered
/// struct, not only on [`Spec::FREE`](crate::Spec::FREE), so a consumer holding a
/// [`SpecMeta`] can count it without any new machinery.
#[implements(
    spec::TheControlledCountIsTheClaimsHeldToTheLanguage,
    spec::TheFreeCountIsTheClaimsMarkedFree,
)]
pub fn ledger(specs: &[SpecMeta]) -> Ledger {
    todo!("the controlled and free counts over {specs:?}")
}

/// The document as it stands at `path`, or the empty string where no file
/// stands there.
///
/// The only item of this slice that reads the filesystem, and the only branch of
/// it a synthetic input cannot produce. Absence answering the empty document is
/// one rule with no exception: a crate with claims and no file is stale, and a
/// crate with no claims of its own renders the empty document and matches.
#[implements(
    spec::TheCommittedDocumentIsTheFilesText,
    spec::AnAbsentDocumentIsTheEmptyDocument,
)]
pub fn committed(path: &Path) -> String {
    todo!("the committed document at {path:?}")
}

/// Check 26's verdict: `None` where the generated and committed documents agree,
/// and otherwise a message naming the first line at which they differ and the
/// invocation that writes the document again.
///
/// An absent file reaches this as the empty string, so absence is staleness only
/// where something was generated.
#[implements(
    spec::AnAgreeingDocumentIsNotStale,
    spec::AStaleDocumentIsNamedByItsFirstDifferingLine,
    spec::AStaleDocumentsMessageNamesTheRegenCommand,
)]
pub fn stale(generated: &str, committed: &str) -> Option<String> {
    todo!("whether {generated} differs from {committed}")
}

pub mod spec;
