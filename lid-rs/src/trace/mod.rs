#![doc = include_str!("lld.md")]
use crate::canary;
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
    /// [`Language::Controlled`].
    pub controlled: usize,
    /// How many of the claims carry
    /// [`Language::Free`].
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
    Path::new(manifest_dir).join(DOCUMENT)
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
    canary::triple_is_present(specs, impls, validations).then_some(()).ok_or(CanaryStripped)
}

/// The document's opening: what a reader of it is looking at, and the one
/// guarantee a row cannot give.
///
/// A row is built from what the registry holds, and the registry holds a claim's
/// parts and never its sentence — for a claim marked
/// [`Language::Free`] it holds nothing of the text
/// at all. So a change to a free claim's wording changes no row here, and the
/// page says so itself rather than leaving a reader of a diff to infer a
/// guarantee that holds for the claims held to the language only.
///
/// A function and not a `const`, for the reason [`DOCUMENT`] is fed to
/// [`document_path`]: a claim whose only implementer is data is true from the
/// skeleton onward, so no test of it could be red before the leaf exists.
#[implements(spec::TheHeaderNamesThatAFreeClaimsRowHoldsNoWording)]
fn header() -> &'static str {
    "# Intent trace\n\
     \n\
     Generated from this crate's registry by `cargo test --lib -- --ignored regen`, and\n\
     compared against the registry by check 26. Nothing on this page is hand-written.\n\
     \n\
     A row carries what the registry holds, which is a claim's parts and never its\n\
     sentence. For a claim marked free the registry holds none of its parts, so a change\n\
     to a free claim's wording changes no row here; for a claim held to the controlled\n\
     language it changes the row that claim stands in.\n"
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
    let total: usize = [counts.controlled, counts.free].iter().sum();
    format!("Claims: {} controlled, {} free, {total} total.", counts.controlled, counts.free)
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
    let below_crate = name.split_once("::").map_or("", |(_, rest)| rest);
    below_crate.rsplit_once("::spec::").map_or("", |(slice, _)| slice)
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
    match language {
        Language::Controlled => "held",
        Language::Free => "free",
    }
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
    let mut cited: Vec<String> = edges
        .iter()
        .filter(|edge| edge.spec == spec_name)
        .map(|edge| format!("{} ({}:{})", edge.item, edge.file, edge.line))
        .collect();
    cited.sort();
    cited
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
    Ledger {
        controlled: specs.iter().filter(|meta| meta.claim.language == Language::Controlled).count(),
        free: specs.iter().filter(|meta| meta.claim.language == Language::Free).count(),
    }
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
    std::fs::read_to_string(path).unwrap_or_default()
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
    let generated_lines: Vec<&str> = generated.lines().collect();
    let committed_lines: Vec<&str> = committed.lines().collect();
    (0..generated_lines.len().max(committed_lines.len()))
        .find(|index| generated_lines.get(*index) != committed_lines.get(*index))
        .map(|index| index + 1)
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
    format!(
        "the committed {DOCUMENT} differs from this crate's registry at line {line}; \
         write it again with `cargo test --lib -- --ignored regen`"
    )
}

#[cfg(test)]
mod tests {
    //! What each leaf answers over synthetic registrations, and the two branches
    //! of the one leaf that reads the filesystem.
    //!
    //! Every case builds its own registry — a [`SpecMeta`] per claim, an
    //! [`Edge`] per citation — and every case that renders carries the canary
    //! triple, since a registry without it is refused before anything is
    //! grouped. Nothing here reads the real registries: that is the emitted
    //! tests' job, and they land after this slice's Phase 7.
    //!
    //! Three habits are deliberate.
    //!
    //! A case that means "this crate has no claims of its own" is written with
    //! *another* crate's registration rather than with none. Over an empty slice
    //! the scope filter rejects nothing, the grouping folds over nothing, and
    //! the answer is the empty document without any leaf being reached — which
    //! is a green test standing for a rule nobody kept.
    //!
    //! Every assertion names content: the claim's name, the pattern, the citing
    //! item, the count line the section stands over. A test that asserted only
    //! that something had been rendered would be satisfied by every formatting
    //! leaf replaced with the empty string, which is the substitution the
    //! mutation gate makes first.
    //!
    //! And where a position is what a claim says — a section's ledger below its
    //! name, the crate's ledger above its first section — the case compares two
    //! offsets in the one rendered document rather than pinning a whole layout,
    //! so a document whose wording changes still answers the same question.

    use super::{
        Ledger, citations, committed, document_path, ledger, ledger_line, render, row, sections,
        slice_of, spec, stale,
    };
    use crate::claim::{ClaimMeta, Language, Pattern};
    use crate::graph::CanaryStripped;
    use crate::registry::{Edge, SpecMeta};
    use lid_rs::validates;
    use std::path::{Path, PathBuf};

    /// The crate whose claims every case below is scoped to.
    const CRATE: &str = "lid_rs";

    /// A crate that registered no claim of its own — what a freshly scaffolded
    /// package is, whose binary links [`CRATE`]'s registrations all the same.
    const CONSUMER: &str = "app";

    /// The canary triple's join key, so a synthetic registry counts as one that
    /// survived linking. It is a claim of [`CRATE`]'s `registry` slice, so a
    /// document rendered for [`CRATE`] carries a `registry` section for it.
    const CANARY: &str =
        <crate::registry::spec::CanaryConfirmsRegistryPresence as crate::Spec>::NAME;

    /// A claim of [`CRATE`]'s `greeter` slice, and the one most cases render.
    const GREETING: &str = "lid_rs::greeter::spec::AGreetingNamesItsAddressee";

    /// A second claim of the same slice, whose name sorts after [`GREETING`], so
    /// a case can supply the two in either order.
    const FAREWELL: &str = "lid_rs::greeter::spec::TheFarewellIsBrief";

    /// A claim of a second slice of [`CRATE`], so that a document has more than
    /// one section and a section's own ledger differs from the crate's.
    const COUNT: &str = "lid_rs::counter::spec::ACountIsANumber";

    /// A claim of [`CRATE`]'s own root, whose name spells no slice between the
    /// crate and `spec`.
    const ROOT_CLAIM: &str = "lid_rs::spec::TheCrateRootCarriesAClaim";

    /// A claim another crate registered, which a consumer's binary links beside
    /// [`CRATE`]'s and which no document of [`CRATE`] may report.
    const FOREIGN: &str = "other_crate::greeter::spec::AForeignClaim";

    /// The file every synthetic registration and citation stands in.
    const SITE: &str = "synthetic.rs";

    /// The item implementing [`GREETING`], named by the citations of the row it
    /// stands in.
    const GREET_FN: &str = "lid_rs::greeter::greet";

    /// The item validating [`GREETING`], on the other side of the same row.
    const GREET_TEST: &str = "lid_rs::greeter::tests::a_greeting_names_its_addressee";

    /// The headings the [`three_sections`] registry's document names: one per
    /// slice its claims fall into, in the order the sections are written.
    const HEADINGS: [&str; 3] = ["## counter", "## greeter", "## registry"];

    /// The names a slice is read out of, and the segments each one spells.
    ///
    /// The second is a claim of a module nested inside a slice, which is what
    /// makes the answer the segments rather than one segment.
    const SLICES: [(&str, &str); 2] = [
        ("lid_rs::graph::spec::CoveredGraphsPassTheGraphCheck", "graph"),
        ("lid_rs::vocab::lexicon::spec::ALexiconIsReadOnce", "vocab::lexicon"),
    ];

    /// A generated document, against which [`COMMITTED`] is the file that stands
    /// in the tree.
    const GENERATED: &str = "one\ntwo\nthree\nfour\n";

    /// The committed document, differing from [`GENERATED`] at its third line
    /// and again at its fourth — two differences, so that *first* is a question
    /// with an answer.
    const COMMITTED: &str = "one\ntwo\nTHREE\nFOUR\n";

    /// The invocation a stale document's verdict names, as pipeline §5.1 spells
    /// it.
    const REGEN: &str = "cargo test --lib -- --ignored regen";

    /// A synthetic registration of one claim, at a fixed site.
    ///
    /// Only the parts this slice reads are given: the registered name a section
    /// and the crate scope are taken from, the pattern a row names, and the
    /// language a row marks and a ledger counts.
    fn spec_meta(name: &'static str, language: Language, pattern: Pattern) -> SpecMeta {
        SpecMeta {
            name,
            file: SITE,
            line: 1,
            claim: ClaimMeta {
                language,
                pattern,
                trigger: "",
                verb: "",
                negated: false,
                object: "",
                owner: "",
                templates: &[],
            },
        }
    }

    /// A claim held to the controlled language.
    fn held(name: &'static str) -> SpecMeta {
        spec_meta(name, Language::Controlled, Pattern::EventDriven)
    }

    /// A claim marked `#[lid(free)]`, whose parts the registry holds none of.
    fn free_claim(name: &'static str) -> SpecMeta {
        spec_meta(name, Language::Free, Pattern::Ubiquitous)
    }

    /// A synthetic citation of `spec` by `item`, at `line` of [`SITE`].
    fn edge(spec: &'static str, item: &'static str, line: u32) -> Edge {
        Edge { spec, item, file: SITE, line }
    }

    /// The canary's own citation, in whichever edge set it is given as: the
    /// least a registry must carry to be rendered from at all.
    fn canary_edges() -> [Edge; 1] {
        [edge(CANARY, "lid_rs::canary::present", 1)]
    }

    /// A registry of [`CRATE`]'s claims falling into three sections, whose
    /// ledgers are `counter` one free, `greeter` two controlled, `registry` one
    /// free — and the crate's own two and two.
    ///
    /// The `greeter` ledger and the crate's differ from every other count line
    /// in the document, which is what lets a case find one of them by its text.
    fn three_sections() -> [SpecMeta; 4] {
        [free_claim(CANARY), held(GREETING), held(FAREWELL), free_claim(COUNT)]
    }

    /// A fresh scratch directory outside the repository, for the one leaf that
    /// reads the filesystem.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-trace-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        dir
    }

    /// The document of a package is the intent document of that package's own
    /// root, which is what makes it per-package without any setting being read.
    #[test]
    #[validates(spec::TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory)]
    fn the_document_path_is_the_intent_document_of_the_manifest_directory() {
        let manifest = "/workspace/app";
        assert_eq!(
            document_path(manifest),
            Path::new(manifest).join("docs/intent/trace.md"),
            "the document stands at `docs/intent/trace.md` of the package it was asked for"
        );
    }

    /// A registry whose canary triple is absent is not rendered from at all.
    ///
    /// It is indistinguishable from a crate that registered nothing, so
    /// regenerating from it would erase every row of a committed document and
    /// the freshness check would then demand the erasure.
    #[test]
    #[validates(spec::RenderRefusesAStrippedRegistry)]
    fn render_refuses_a_stripped_registry() {
        let specs = [held(GREETING)];
        let refused = render(CRATE, &specs, &[], &[]);
        assert!(
            matches!(refused, Err(CanaryStripped)),
            "a stripped registry must be refused rather than rendered"
        );
    }

    /// A crate that registered no claim of its own renders the empty document,
    /// which is what makes an absent file a passing state for it rather than a
    /// special case written into the check.
    ///
    /// Written with two of another crate's registrations rather than with none:
    /// over an empty registry the scope filter rejects nothing and the answer is
    /// the empty document without the grouping being reached at all.
    #[test]
    #[validates(spec::ACrateWithNoClaimsRendersTheEmptyDocument)]
    fn a_crate_with_no_claims_renders_the_empty_document() {
        let specs = [free_claim(CANARY), held(GREETING)];
        let edges = canary_edges();
        let document = render(CONSUMER, &specs, &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        assert!(document.is_empty(), "a crate with no claims of its own has no page: {document:?}");
    }

    /// The header carries the one guarantee a row cannot give: a free claim's
    /// row holds none of that claim's wording, because the registry holds a free
    /// claim's parts as empty.
    ///
    /// The qualification is looked for above the first section, since that is
    /// where a reader of a diff meets it. Both words are asserted because the
    /// count line stands in the same region and names the free mark itself —
    /// `wording` is the word no count line can carry.
    #[test]
    #[validates(spec::TheHeaderNamesThatAFreeClaimsRowHoldsNoWording)]
    fn the_header_names_that_a_free_claims_row_holds_no_wording() {
        let specs = [free_claim(CANARY), held(GREETING)];
        let edges = canary_edges();
        let document = render(CRATE, &specs, &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        let heading = document.find("## greeter").expect("the document names its sections");
        let header = &document[..heading];
        assert!(header.contains("wording"), "the header names what a free row holds none of: {document}");
        assert!(header.contains("free"), "the header names whose row that is: {document}");
    }

    /// Every slice the crate's claims fall into is a named part of the document.
    #[test]
    #[validates(spec::TheDocumentNamesEverySectionItsClaimsFallInto)]
    fn the_document_names_every_section_its_claims_fall_into() {
        let edges = canary_edges();
        let document = render(CRATE, &three_sections(), &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        for heading in HEADINGS {
            assert!(document.contains(heading), "the document names `{heading}`: {document}");
        }
    }

    /// Every claim of the crate stands in the document as its own row, which is
    /// what makes a wording change appear in the diff beside its validators.
    #[test]
    #[validates(spec::TheDocumentCarriesTheRowOfEveryClaimOfTheCrate)]
    fn the_document_carries_the_row_of_every_claim_of_the_crate() {
        let specs = three_sections();
        let edges = canary_edges();
        let document = render(CRATE, &specs, &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        for meta in &specs {
            let line = row(meta, &edges, &edges);
            assert!(document.contains(&line), "the document carries `{line}`: {document}");
        }
    }

    /// A section's own count line stands below that section's name, so that a
    /// count and the rows it counts are read together.
    ///
    /// The `greeter` ledger is two controlled and no free, which no other count
    /// line in this document carries.
    #[test]
    #[validates(spec::TheDocumentNamesEachSectionsLedgerBelowThatSectionsName)]
    fn the_document_names_each_sections_ledger_below_that_sections_name() {
        let edges = canary_edges();
        let document = render(CRATE, &three_sections(), &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        let name = document.find("## greeter").expect("the document names the section");
        let counts = document
            .find(&ledger_line(Ledger { controlled: 2, free: 0 }))
            .expect("the section's own ledger");
        assert!(name < counts, "a section's ledger stands below its name: {document}");
    }

    /// The crate's own count line stands above its first section, summing the
    /// claims every section holds: the ramp's number, in one line of a diff.
    #[test]
    #[validates(spec::TheDocumentNamesTheLedgerOfAllItsClaimsAboveItsSections)]
    fn the_document_names_the_ledger_of_all_its_claims_above_its_sections() {
        let edges = canary_edges();
        let document = render(CRATE, &three_sections(), &edges, &edges)
            .expect("the canary stands in the synthetic registry");
        let counts = document
            .find(&ledger_line(Ledger { controlled: 2, free: 2 }))
            .expect("the crate's own ledger, over every section's claims");
        let first = document.find(HEADINGS[0]).expect("the document names its first section");
        assert!(counts < first, "the crate's ledger stands above its sections: {document}");
    }

    /// One registry renders one byte sequence, whatever order it arrives in.
    ///
    /// `linkme` promises no order across link units, so a document whose rows or
    /// citations arrived in link order would be fresh on one machine and stale
    /// on the next. Both the registrations and the citations are reversed, since
    /// the rows and the citing items within a row are sorted separately — and
    /// the comparison is between two renders, never against a literal, because a
    /// literal would fix a layout instead of asserting an order.
    #[test]
    #[validates(spec::OneRegistryRendersOneDocumentWhateverItsOrder)]
    fn one_registry_renders_one_document_whatever_its_order() {
        let forward = three_sections();
        let backward = [free_claim(COUNT), held(FAREWELL), held(GREETING), free_claim(CANARY)];
        let cited = [edge(CANARY, "lid_rs::canary::present", 1), edge(GREETING, GREET_FN, 7)];
        let uncited = [edge(GREETING, GREET_FN, 7), edge(CANARY, "lid_rs::canary::present", 1)];
        let one = render(CRATE, &forward, &cited, &cited).expect("the canary stands in the registry");
        let other =
            render(CRATE, &backward, &uncited, &uncited).expect("the canary stands in the registry");
        assert_eq!(one, other, "one registry renders one document, whatever order it arrived in");
    }

    /// The crate's claims are grouped into the slices their names spell, one
    /// section per slice and every claim of that slice in it.
    #[test]
    #[validates(spec::SectionsGroupTheCratesClaimsByTheirSlice)]
    fn sections_group_the_crates_claims_by_their_slice() {
        let specs = [held(GREETING), held(FAREWELL), free_claim(COUNT)];
        let parts = sections(CRATE, &specs);
        let grouped: Vec<(&str, usize)> =
            parts.iter().map(|part| (part.name.as_str(), part.claims.len())).collect();
        assert_eq!(grouped, [("counter", 1), ("greeter", 2)], "one section per slice, holding its own claims");
    }

    /// A consumer's binary links this crate's registrations beside its own, so a
    /// document reports the claims of the crate it was given and no other's.
    ///
    /// The foreign registration is carried beside one of the crate's own: with
    /// the foreign claim alone the filter rejects everything, the answer is the
    /// empty `Vec`, and nothing of the grouping is reached.
    #[test]
    #[validates(spec::SectionsScopeToTheCrateTheyWereGiven)]
    fn sections_scope_to_the_crate_they_were_given() {
        let specs = [held(FOREIGN), held(GREETING)];
        let named: Vec<&str> = sections(CRATE, &specs)
            .iter()
            .flat_map(|part| part.claims.iter().map(|meta| meta.name))
            .collect();
        assert_eq!(named, [GREETING], "another crate's claim is not this document's");
    }

    /// A crate-root slice keeps its claims in `src/spec.rs`, so their names
    /// spell no slice at all: the crate is the slice there, and it names the
    /// section rather than leaving a part of the document nothing is called.
    #[test]
    #[validates(spec::ACrateRootSlicesSectionIsNamedForTheCrate)]
    fn a_crate_root_slices_section_is_named_for_the_crate() {
        let specs = [held(ROOT_CLAIM)];
        let named: Vec<String> = sections(CRATE, &specs).into_iter().map(|part| part.name).collect();
        assert_eq!(named, [CRATE], "a crate-root claim's section is named for its crate");
    }

    /// A section's ledger counts exactly the claims that section holds, so a
    /// reader who counts a section's marked rows arrives at its free count.
    ///
    /// The two sections' ledgers differ, so a ledger taken over every claim
    /// rather than over each section's own answers neither.
    #[test]
    #[validates(spec::EachSectionCarriesTheLedgerOfItsOwnClaims)]
    fn each_section_carries_the_ledger_of_its_own_claims() {
        let specs = [held(GREETING), free_claim(FAREWELL), held(COUNT)];
        let ledgers: Vec<Ledger> = sections(CRATE, &specs).iter().map(|part| part.ledger).collect();
        assert_eq!(
            ledgers,
            [Ledger { controlled: 1, free: 0 }, Ledger { controlled: 1, free: 1 }],
            "each section counts its own claims and no others"
        );
    }

    /// The slice is what stands between the crate segment and the trailing
    /// `spec` and claim segments — read from the name, so the document assumes
    /// nothing about where a slice's files stand.
    #[test]
    #[validates(spec::TheSliceIsTheSegmentsBetweenTheCrateAndTheSpecModule)]
    fn the_slice_is_the_segments_between_the_crate_and_the_spec_module() {
        for (name, slice) in SLICES {
            assert_eq!(slice_of(name), slice, "the slice `{name}` spells");
        }
    }

    /// A crate-root claim's name holds no segment between its crate and its
    /// `spec`, and the answer is the empty string rather than a guess.
    #[test]
    #[validates(spec::ACrateRootClaimsSliceIsEmpty)]
    fn a_crate_root_claims_slice_is_empty() {
        assert_eq!(
            slice_of("cargo_lid_rs::spec::TheCatalogNamesEveryCheck"),
            "",
            "a crate-root claim spells no slice"
        );
    }

    /// A row names the claim it stands for, which is the key everything else in
    /// the row is joined to it by.
    #[test]
    #[validates(spec::ARowNamesItsClaim)]
    fn a_row_names_its_claim() {
        let claim = held(GREETING);
        let line = row(&claim, &[], &[]);
        assert!(line.contains(GREETING), "the row names its claim: {line}");
    }

    /// A row names the pattern the claim's opener was read as, which is the part
    /// of the controlled language a reader of the page can check at a glance.
    #[test]
    #[validates(spec::ARowNamesItsClaimsPattern)]
    fn a_row_names_its_claims_pattern() {
        let claim = spec_meta(GREETING, Language::Controlled, Pattern::Unwanted);
        let line = row(&claim, &[], &[]);
        assert!(line.contains("Unwanted"), "the row names the claim's pattern: {line}");
    }

    /// A row says of a claim marked free that it is free, which is the same
    /// answer its section's ledger counts it into.
    #[test]
    #[validates(spec::ARowNamesAFreeClaimAsFree)]
    fn a_row_names_a_free_claim_as_free() {
        let claim = free_claim(GREETING);
        let line = row(&claim, &[], &[]);
        assert!(line.contains("free"), "a marked claim's row names it free: {line}");
    }

    /// A row says nothing of the kind about a claim held to the language.
    ///
    /// The row is also asserted to name its claim, because the empty string
    /// names nothing free either — and the empty string is what every formatting
    /// leaf is replaced with when the mutation gate runs.
    #[test]
    #[validates(spec::ARowDoesNotNameAHeldClaimAsFree)]
    fn a_row_does_not_name_a_held_claim_as_free() {
        let claim = held(GREETING);
        let line = row(&claim, &[], &[]);
        assert!(line.contains(GREETING), "the row is the claim's, and not the empty string: {line}");
        assert!(!line.contains("free"), "a claim held to the language is not named free: {line}");
    }

    /// A row carries the items implementing its claim, which is the half of the
    /// trace matrix that says the code was written.
    #[test]
    #[validates(spec::ARowCarriesItsClaimsImplementingItems)]
    fn a_row_carries_its_claims_implementing_items() {
        let claim = held(GREETING);
        let impls = [edge(GREETING, GREET_FN, 7)];
        let line = row(&claim, &impls, &[]);
        assert!(line.contains(GREET_FN), "the row carries the item implementing it: {line}");
    }

    /// A row carries the items validating its claim, which is the other half.
    #[test]
    #[validates(spec::ARowCarriesItsClaimsValidatingItems)]
    fn a_row_carries_its_claims_validating_items() {
        let claim = held(GREETING);
        let validations = [edge(GREETING, GREET_TEST, 31)];
        let line = row(&claim, &[], &validations);
        assert!(line.contains(GREET_TEST), "the row carries the item validating it: {line}");
    }

    /// A citation names the citing item and the site it stands at, as plain
    /// `item (file:line)` text — the form every report in this crate uses, and
    /// not a link, whose target would depend on where the document was built.
    #[test]
    #[validates(spec::ACitationNamesTheCitingItemWithItsSite)]
    fn a_citation_names_the_citing_item_with_its_site() {
        let edges = [edge(GREETING, GREET_FN, 7)];
        assert_eq!(
            citations(GREETING, &edges),
            [format!("{GREET_FN} ({SITE}:7)")],
            "a citation is the citing item with its site"
        );
    }

    /// An edge citing another claim belongs in that claim's row, so it is not
    /// named here.
    ///
    /// One edge of each is given, and the answer is asserted whole: a body that
    /// named every edge fails on the first, and one that named none fails on the
    /// second.
    #[test]
    #[validates(spec::ACitationIsNotMadeForAnotherClaimsEdge)]
    fn a_citation_is_not_made_for_another_claims_edge() {
        let edges = [edge(FAREWELL, "lid_rs::greeter::farewell", 3), edge(GREETING, GREET_FN, 7)];
        assert_eq!(
            citations(GREETING, &edges),
            [format!("{GREET_FN} ({SITE}:7)")],
            "the citations of one claim are not another's"
        );
    }

    /// The controlled count is how many of the claims were held to the language.
    #[test]
    #[validates(spec::TheControlledCountIsTheClaimsHeldToTheLanguage)]
    fn the_controlled_count_is_the_claims_held_to_the_language() {
        let greeting = held(GREETING);
        let farewell = free_claim(FAREWELL);
        let count = held(COUNT);
        let claims = [&greeting, &farewell, &count];
        assert_eq!(ledger(&claims).controlled, 2, "two of the three were held to the language");
    }

    /// The free count is how many carry the mark, which is the ramp's own
    /// number: the one count the registry can answer today.
    ///
    /// It differs from the controlled count over the same claims, so a ledger
    /// that answered one number twice fails here.
    #[test]
    #[validates(spec::TheFreeCountIsTheClaimsMarkedFree)]
    fn the_free_count_is_the_claims_marked_free() {
        let greeting = held(GREETING);
        let farewell = free_claim(FAREWELL);
        let count = held(COUNT);
        let claims = [&greeting, &farewell, &count];
        assert_eq!(ledger(&claims).free, 1, "one of the three carries the mark");
    }

    /// The committed document is the text of the file that stands in the tree.
    ///
    /// Against a temporary directory and never a path in the repository: this is
    /// the slice's only filesystem reader, and a test that read a committed file
    /// would pass or fail on the state of the checkout.
    #[test]
    #[validates(spec::TheCommittedDocumentIsTheFilesText)]
    fn the_committed_document_is_the_files_text() {
        let path = scratch("committed").join("trace.md");
        std::fs::write(&path, GENERATED).expect("a document standing at the path");
        assert_eq!(committed(&path), GENERATED, "the committed document is the file's own text");
    }

    /// A path no file stands at answers the empty string, which is the one rule
    /// with no exception: a crate with claims and no document is stale, and a
    /// crate with no claims renders the empty document and matches.
    ///
    /// The scratch directory is created empty, so nothing stands at the path.
    #[test]
    #[validates(spec::AnAbsentDocumentIsTheEmptyDocument)]
    fn an_absent_document_is_the_empty_document() {
        let path = scratch("absent").join("trace.md");
        let document = committed(&path);
        assert!(document.is_empty(), "no file stands at the path: {document:?}");
    }

    /// A document that agrees with what the registry renders is not stale, and
    /// produces no message at all rather than a message nobody reads.
    #[test]
    #[validates(spec::AnAgreeingDocumentIsNotStale)]
    fn an_agreeing_document_is_not_stale() {
        assert_eq!(stale(GENERATED, GENERATED), None, "a document that agrees is not stale");
    }

    /// The verdict names the line the two documents first part company at, so
    /// that whoever reads it opens the file where the disagreement is.
    ///
    /// The two differ at their third line and again at their fourth: a verdict
    /// naming the last difference, or the count of them, does not name the
    /// third.
    #[test]
    #[validates(spec::AStaleDocumentIsNamedByItsFirstDifferingLine)]
    fn a_stale_document_is_named_by_its_first_differing_line() {
        let message = stale(GENERATED, COMMITTED).expect("a differing document is stale");
        assert!(message.contains('3'), "the verdict names the first differing line: {message}");
    }

    /// The verdict names the invocation that writes the document again, because
    /// regenerating is the only repair: a hand-edited document is one the next
    /// generation overwrites.
    #[test]
    #[validates(spec::AStaleDocumentsMessageNamesTheRegenCommand)]
    fn a_stale_documents_message_names_the_regen_command() {
        let message = stale(GENERATED, COMMITTED).expect("a differing document is stale");
        assert!(message.contains(REGEN), "the verdict names `{REGEN}`: {message}");
    }
}

pub mod spec;
