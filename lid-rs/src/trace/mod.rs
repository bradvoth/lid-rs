#![doc = include_str!("lld.md")]
use crate::claim::Language;
use crate::graph::CanaryStripped;
use crate::registry::{Edge, SpecMeta};
use lid_rs::implements;
use std::collections::BTreeMap;
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
/// [`Ledger`] and carrying the [`row`] of every claim in it. That composition is
/// `document_text`'s, so what stands here is the two decisions and nothing else.
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
    trusted(specs, impls, validations)?;
    let parts = sections(crate_name, specs);
    if parts.is_empty() {
        return Ok(String::new());
    }
    Ok(document_text(&parts, impls, validations))
}

/// Whether the registrations [`render`] was given may be rendered from at all,
/// carried as the refusal [`render`] answers with.
///
/// A registry whose canary triple is absent was stripped by the linker or never
/// populated, and it is indistinguishable from a crate that registered nothing:
/// it renders as the empty document, regeneration then erases every row of a
/// committed file, and the freshness check demands the erasure. So the question
/// is settled before anything is grouped or rendered.
///
/// It carries [`CanaryStripped`] rather than a `bool` because the answer this
/// leaf gives is the answer [`render`] returns — a `bool` would leave the
/// refusal itself written in the composition, where no test could tell a wrong
/// answer from a missing one.
#[implements(spec::RenderRefusesAStrippedRegistry)]
fn trusted(
    specs: &[SpecMeta],
    impls: &[Edge],
    validations: &[Edge],
) -> Result<(), CanaryStripped> {
    todo!("whether {specs:?}, {impls:?} and {validations:?} carry the canary triple")
}

/// The document's opening: what a reader of it is looking at, and the one
/// guarantee a row cannot give.
///
/// A row is built from what the registry holds, and the registry holds a claim's
/// parts and never its sentence — for a claim marked
/// [`Language::Free`](crate::claim::Language::Free) it holds nothing of the text
/// at all. So a change to a free claim's wording changes no row here, and the
/// page says so itself rather than leaving a reader of a diff to infer a
/// guarantee that holds for the claims held to the language only.
///
/// A function and not a `const`, for the reason [`DOCUMENT`] is fed to
/// [`document_path`]: a claim whose only implementer is data is true from the
/// skeleton onward, so no test of it could be red before the leaf exists.
#[implements(spec::TheHeaderNamesThatAFreeClaimsRowHoldsNoWording)]
fn header() -> &'static str {
    todo!("the document's header")
}

/// The whole document below the guard: the header, the ledger of every claim the
/// crate registered, and one part per [`Section`] in the order [`sections`]
/// carries them.
///
/// The crate's own ledger is the [`ledger`] of the claims its sections hold, and
/// not a second count over the registrations it was handed: which claims are the
/// crate's is [`sections`]'s answer, and counting them again here would be a
/// second place for the crate scope to be decided.
#[implements(
    spec::TheHeaderNamesThatAFreeClaimsRowHoldsNoWording,
    spec::TheDocumentNamesTheLedgerOfAllItsClaimsAboveItsSections,
    spec::TheDocumentNamesEverySectionItsClaimsFallInto,
    spec::TheDocumentCarriesTheRowOfEveryClaimOfTheCrate,
)]
fn document_text(parts: &[Section<'_>], impls: &[Edge], validations: &[Edge]) -> String {
    let claims: Vec<&SpecMeta> = parts.iter().flat_map(|part| part.claims.iter().copied()).collect();
    let body: Vec<String> = parts.iter().map(|part| section_text(part, impls, validations)).collect();
    format!("{}\n{}\n\n{}", header(), ledger_line(ledger(&claims)), body.join("\n"))
}

/// One [`Section`]'s part of the document: its name, its own [`Ledger`] below
/// that name, and the [`row`] of every claim it holds, under the one heading
/// those rows stand beneath.
///
/// The section's ledger stands below its name and above its rows, so that a
/// count and the rows it counts are read together and a diff that changes one
/// shows the other.
#[implements(
    spec::TheDocumentNamesEverySectionItsClaimsFallInto,
    spec::TheDocumentNamesEachSectionsLedgerBelowThatSectionsName,
    spec::TheDocumentCarriesTheRowOfEveryClaimOfTheCrate,
)]
fn section_text(part: &Section<'_>, impls: &[Edge], validations: &[Edge]) -> String {
    let rows: Vec<String> = part.claims.iter().map(|meta| row(meta, impls, validations)).collect();
    format!(
        "## {}\n\n{}\n\n| Claim | Pattern | Language | Implemented by | Validated by |\n\
         | --- | --- | --- | --- | --- |\n{}\n",
        part.name,
        ledger_line(part.ledger),
        rows.join("\n"),
    )
}

/// One [`Ledger`] as the count line a section or a whole document stands over:
/// each counted category with its number, and the total.
///
/// The shape a count section takes, written in one place so that a section's
/// line and the document's line are one answer rather than two, and so that the
/// shape pass fills the same shape when it has a caller.
#[implements(
    spec::TheDocumentNamesEachSectionsLedgerBelowThatSectionsName,
    spec::TheDocumentNamesTheLedgerOfAllItsClaimsAboveItsSections,
)]
fn ledger_line(counts: Ledger) -> String {
    todo!("the count line of {counts:?}")
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
    let prefix = format!("{crate_name}::");
    let mut grouped: BTreeMap<&str, Vec<&'a SpecMeta>> = BTreeMap::new();
    for meta in specs.iter().filter(|meta| meta.name.starts_with(&prefix)) {
        grouped.entry(section_name(crate_name, meta.name)).or_default().push(meta);
    }
    grouped.into_iter().map(|(name, claims)| section(name, claims)).collect()
}

/// The name of the [`Section`] a claim falls into: the slice its name spells,
/// and the crate's own name where that is empty.
///
/// A crate-root slice keeps its claims in `src/spec.rs`, so their names spell no
/// segment at all between the crate and `spec` — the crate is the slice there,
/// and it names the section rather than leaving the document with a part nothing
/// is called.
#[implements(
    spec::ACrateRootSlicesSectionIsNamedForTheCrate,
    spec::SectionsGroupTheCratesClaimsByTheirSlice,
)]
fn section_name<'a>(crate_name: &'a str, spec_name: &'a str) -> &'a str {
    let slice = slice_of(spec_name);
    if slice.is_empty() { crate_name } else { slice }
}

/// One group of claims as a [`Section`]: the claims in the order their rows are
/// written, and the [`ledger`] of exactly those claims.
///
/// The order is by name, because `linkme` promises none across link units and a
/// freshness comparison requires that one registry render one byte sequence.
#[implements(
    spec::EachSectionCarriesTheLedgerOfItsOwnClaims,
    spec::OneRegistryRendersOneDocumentWhateverItsOrder,
)]
fn section<'a>(name: &str, mut claims: Vec<&'a SpecMeta>) -> Section<'a> {
    claims.sort_by_key(|meta| meta.name);
    let counts = ledger(&claims);
    Section { name: name.to_string(), claims, ledger: counts }
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
///
/// The [`Pattern`](crate::claim::Pattern) is written as the enum spells its own
/// variant, so a pattern added to the language needs no second table here saying
/// what to call it.
#[implements(
    spec::ARowNamesItsClaim,
    spec::ARowNamesItsClaimsPattern,
    spec::ARowNamesAFreeClaimAsFree,
    spec::ARowDoesNotNameAHeldClaimAsFree,
    spec::ARowCarriesItsClaimsImplementingItems,
    spec::ARowCarriesItsClaimsValidatingItems,
)]
pub fn row(meta: &SpecMeta, impls: &[Edge], validations: &[Edge]) -> String {
    format!(
        "| `{}` | {:?} | {} | {} | {} |",
        meta.name,
        meta.claim.pattern,
        language_mark(meta.claim.language),
        citations(meta.name, impls).join(", "),
        citations(meta.name, validations).join(", "),
    )
}

/// What a row says of a claim held to the controlled language, and what it says
/// of one marked free of it.
///
/// The distinction the free ramp is counted by, said once: a row and a
/// [`Ledger`] answer the same question about the same claim, and a reader who
/// counts the marked rows of a section must arrive at that section's free count.
#[implements(spec::ARowNamesAFreeClaimAsFree, spec::ARowDoesNotNameAHeldClaimAsFree)]
fn language_mark(language: Language) -> &'static str {
    todo!("what a row calls {language:?}")
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
///
/// The claims arrive as references because that is what a [`Section`] holds and
/// what a document's own count is taken over: a [`SpecMeta`] is neither `Clone`
/// nor `Copy`, so a signature over the registrations themselves could count a
/// registry but never a section of one.
#[implements(
    spec::TheControlledCountIsTheClaimsHeldToTheLanguage,
    spec::TheFreeCountIsTheClaimsMarkedFree,
)]
pub fn ledger(specs: &[&SpecMeta]) -> Ledger {
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
///
/// A search and a message: whether the two differ, and where, is one question,
/// and what to tell whoever must repair it is another. The verdict is the first
/// answer carried into the second, so a document that agrees produces no message
/// at all rather than a message nobody reads.
#[implements(
    spec::AnAgreeingDocumentIsNotStale,
    spec::AStaleDocumentIsNamedByItsFirstDifferingLine,
    spec::AStaleDocumentsMessageNamesTheRegenCommand,
)]
pub fn stale(generated: &str, committed: &str) -> Option<String> {
    first_difference(generated, committed).map(repair_message)
}

/// Where the generated and the committed documents first part company, counted
/// from the first line as one, or [`Option::None`] where they agree.
///
/// The whole of check 26's question. A document absent from the tree arrives
/// here as the empty string, so it differs at the first line of anything that
/// was generated and agrees with a crate that generated nothing.
///
/// A line and not a byte offset: the reader of the verdict opens the file at a
/// line, and a document whose rows are one claim each puts the disagreement in
/// the row that caused it.
#[implements(
    spec::AnAgreeingDocumentIsNotStale,
    spec::AStaleDocumentIsNamedByItsFirstDifferingLine,
)]
fn first_difference(generated: &str, committed: &str) -> Option<usize> {
    todo!("the first line at which {generated} differs from {committed}")
}

/// What a stale document's verdict says: the line the two first differ at, and
/// the invocation that writes the document again.
///
/// The message is read by whoever has to make the tree agree with the registry,
/// and the only thing they may do about it is regenerate: a hand-edited document
/// is one the next generation overwrites. So the repair is named in the verdict
/// rather than left to be remembered.
#[implements(
    spec::AStaleDocumentIsNamedByItsFirstDifferingLine,
    spec::AStaleDocumentsMessageNamesTheRegenCommand,
)]
fn repair_message(line: usize) -> String {
    todo!("the verdict on a document differing at line {line}")
}

pub mod spec;
