//! Claims for the trace slice — the registry, rendered as a committed page and
//! compared against the file that stands there (`lid-rs/src/trace/lld.md`).
//!
//! The slice is the generator and check 26's comparison, for one library member
//! at a time: `document_path` says where the page goes, `render` composes it
//! from one crate's registrations, `sections`, `slice_of`, `row`, `citations`
//! and `ledger` are the leaves it composes from, `committed` reads the file that
//! is there, and `stale` is the verdict. The claims below are those nine items,
//! in that order.
//!
//! **The document's location is claimed on the function, never on the constant.**
//! The path is data — `DOCUMENT`, joined onto a manifest directory — and a claim
//! whose only implementer is a `const` is true from the skeleton onward, so no
//! test of it can be red before the leaf exists.
//! [`TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory`] is therefore
//! cited on [`document_path`](crate::trace::document_path), which the constant
//! feeds, the way the registry and outcome canaries put presence on a function
//! rather than on the static it reads.
//!
//! **Determinism is a claim, because `linkme` promises no order.** Freshness is
//! only meaningful if one registry renders one byte sequence, and a document
//! whose rows arrive in link order is fresh on one machine and stale on the
//! next. [`OneRegistryRendersOneDocumentWhateverItsOrder`] is what the sorting
//! in [`sections`](crate::trace::sections) and
//! [`citations`](crate::trace::citations) is for, and it is separately
//! falsifiable from every other claim here: a generator that grouped, counted
//! and formatted every row correctly and emitted them unsorted falsifies this
//! one alone. It is also why no claim below fixes a sort key — the order that
//! matters is that there is one, and a key named in a claim would be a second
//! place to change it.
//!
//! **The canary refusal and the crate scope are this slice's own claims.** The
//! graph slice states both for its checks, and the reasoning transfers while the
//! claims do not: the mutants of [`render`](crate::trace::render) and
//! [`sections`](crate::trace::sections) are killed by tests written against
//! those two functions, so their claims live where those tests do.
//! [`RenderRefusesAStrippedRegistry`] reuses the graph slice's
//! [`CanaryStripped`](crate::graph::CanaryStripped) as the answer rather than
//! adding a second one, because a stripped registry renders the empty document
//! and an unguarded regeneration would erase a committed file.
//!
//! **The empty document is claimed twice, from two sides.**
//! [`SectionsScopeToTheCrateTheyWereGiven`] says a foreign registration gets no
//! section; [`ACrateWithNoClaimsRendersTheEmptyDocument`] says the whole
//! document is then the empty string, which is what makes an absent file a
//! passing state rather than a special case. Scoping alone would leave a header
//! and a count line standing over no rows, so a freshly scaffolded package with
//! no claims of its own would fail its own gate on the first run.
//!
//! **The header carries the one guarantee the row cannot give.** A row is built
//! from what the registry holds, and the registry holds a claim's parts, never
//! its sentence — for a claim marked [`Language::Free`](crate::claim::Language::Free)
//! it holds nothing of the text at all.
//! [`TheHeaderNamesThatAFreeClaimsRowHoldsNoWording`] keeps that qualification in
//! the generated page instead of in prose about it, so a reader of a diff is not
//! left to infer a wording-change guarantee that holds for controlled claims
//! only.
//!
//! **A location is plain `file:line` text.**
//! [`ACitationNamesTheCitingItemWithItsSite`] fixes the form as
//! `item (file:line)`, which is the form every report in this crate already
//! uses. No claim here asks for a link: the distance from a registered file to
//! the document is a property of the build layout, and a generator that probed
//! for it would make the rendered text depend on where it was built, which is
//! the one thing a freshness comparison cannot tolerate.
//!
//! **What has no claim here, and where it lives.** There is no shape section and
//! no conformance section — the shape pass has no caller in this workspace and
//! no conformance level is readable, so a count over either would be a column
//! nothing produces; both are deferred to the change that gives them a producer,
//! into the section shape [`Ledger`](crate::trace::Ledger) and
//! [`TheDocumentNamesEachSectionsLedgerBelowThatSectionsName`] define here. No
//! claim reads the `trace` setting, which no crate that can be depended on from
//! here can read. And **no claim is about the emitted tests**: the freshness test
//! and the ignored regeneration test are `intent_graph!()`'s, in the graph
//! slice's module, plain and uncited as every emitted test in this workspace is,
//! and they land after this slice's Phase 7 — a claim about them would be an
//! orphan in a module no phase of this branch may write.
//!
//! **The controlled language was held by hand.** `lid-rs/src/lib.rs` declares no
//! `pub mod trace;` yet — the document lands that line between this phase and
//! Phase 3 — so nothing compiles this file and check 13 does not run on it.
//! Every verb below is the base lexicon's (`be`, `carry`, `name`, `equal`,
//! `refuse`), which is what a published crate answers to whatever a project's
//! file says; every claim is one sentence with one `shall` and one terminator;
//! and every trigger names a Rust item by intra-doc link, placed first in its
//! clause so the link the derive records is the one intended. The items Phase 3
//! has yet to create are linked by the paths the document's Shape table gives.
//! None of these claims is marked free.

use lid_rs::Spec;

// ---- Where the document goes -------------------------------------------------

/// When [`document_path`](crate::trace::document_path) is given a package's
/// manifest directory, the path it carries shall be that directory joined with
/// `docs/intent/trace.md`.
#[derive(Spec)]
pub struct TheDocumentPathIsTheIntentDocumentOfTheManifestDirectory;

// ---- `render`: the refusal, the empty case, and what the document holds ------

/// When the canary triple is absent from the registries
/// [`render`](crate::trace::render) is given, it shall refuse with
/// [`CanaryStripped`](crate::graph::CanaryStripped).
#[derive(Spec)]
pub struct RenderRefusesAStrippedRegistry;

/// When [`render`](crate::trace::render) is given no claim registered by the
/// crate it was given, the document it carries shall be empty.
#[derive(Spec)]
pub struct ACrateWithNoClaimsRendersTheEmptyDocument;

/// When [`render`](crate::trace::render) is given a claim registered by the
/// crate it was given, the document it carries shall carry a header naming that
/// the row of a claim marked [`Language::Free`](crate::claim::Language::Free)
/// holds none of that claim's wording.
#[derive(Spec)]
pub struct TheHeaderNamesThatAFreeClaimsRowHoldsNoWording;

/// When [`render`](crate::trace::render) is given claims registered by the crate
/// it was given, the document it carries shall name each
/// [`Section`](crate::trace::Section) those claims fall into.
#[derive(Spec)]
pub struct TheDocumentNamesEverySectionItsClaimsFallInto;

/// When [`render`](crate::trace::render) is given a claim registered by the
/// crate it was given, the document it carries shall carry the
/// [`row`](crate::trace::row) of that claim.
#[derive(Spec)]
pub struct TheDocumentCarriesTheRowOfEveryClaimOfTheCrate;

/// When [`render`](crate::trace::render) carries a document holding a
/// [`Section`](crate::trace::Section), that document shall name the
/// [`Ledger`](crate::trace::Ledger) of that section below the section's own
/// name.
#[derive(Spec)]
pub struct TheDocumentNamesEachSectionsLedgerBelowThatSectionsName;

/// When [`render`](crate::trace::render) carries a document holding a
/// [`Section`](crate::trace::Section), that document shall name the
/// [`Ledger`](crate::trace::Ledger) of all the crate's claims above its first
/// section.
#[derive(Spec)]
pub struct TheDocumentNamesTheLedgerOfAllItsClaimsAboveItsSections;

/// When [`render`](crate::trace::render) is given one crate's registrations in
/// two orders, the two documents it carries shall equal one another.
#[derive(Spec)]
pub struct OneRegistryRendersOneDocumentWhateverItsOrder;

// ---- `sections`: the grouping, the crate scope, and each section's ledger ----

/// When [`sections`](crate::trace::sections) is given claims registered by the
/// crate it was given, it shall carry one [`Section`](crate::trace::Section) for
/// each slice their names spell.
#[derive(Spec)]
pub struct SectionsGroupTheCratesClaimsByTheirSlice;

/// When [`sections`](crate::trace::sections) is given a claim registered by
/// another crate, it shall not carry a [`Section`](crate::trace::Section) for
/// that claim.
#[derive(Spec)]
pub struct SectionsScopeToTheCrateTheyWereGiven;

/// When [`sections`](crate::trace::sections) is given a claim whose
/// [`slice_of`](crate::trace::slice_of) is empty, the
/// [`Section`](crate::trace::Section) it carries for that claim shall be named
/// for the crate it was given.
#[derive(Spec)]
pub struct ACrateRootSlicesSectionIsNamedForTheCrate;

/// When [`sections`](crate::trace::sections) carries a
/// [`Section`](crate::trace::Section), the [`Ledger`](crate::trace::Ledger) that
/// section carries shall be the [`ledger`](crate::trace::ledger) of the claims
/// in that section.
#[derive(Spec)]
pub struct EachSectionCarriesTheLedgerOfItsOwnClaims;

// ---- `slice_of`: the section a claim's name spells ---------------------------

/// When [`slice_of`](crate::trace::slice_of) is given a claim's name, it shall
/// carry the segments of that name between its crate segment and its trailing
/// `spec` and claim segments.
#[derive(Spec)]
pub struct TheSliceIsTheSegmentsBetweenTheCrateAndTheSpecModule;

/// When [`slice_of`](crate::trace::slice_of) is given a name holding no segment
/// between its crate segment and its trailing `spec` segment, it shall carry the
/// empty string.
#[derive(Spec)]
pub struct ACrateRootClaimsSliceIsEmpty;

// ---- `row`: one claim's line ---------------------------------------------------

/// When [`row`](crate::trace::row) is given a [`SpecMeta`](crate::SpecMeta), the
/// row it carries shall name that registration's [`name`](crate::SpecMeta::name).
#[derive(Spec)]
pub struct ARowNamesItsClaim;

/// When [`row`](crate::trace::row) is given a [`SpecMeta`](crate::SpecMeta), the
/// row it carries shall name the [`Pattern`](crate::claim::Pattern) that claim
/// carries.
#[derive(Spec)]
pub struct ARowNamesItsClaimsPattern;

/// When [`row`](crate::trace::row) is given a claim marked
/// [`Language::Free`](crate::claim::Language::Free), the row it carries shall
/// name that claim as free.
#[derive(Spec)]
pub struct ARowNamesAFreeClaimAsFree;

/// When [`row`](crate::trace::row) is given a claim carrying
/// [`Language::Controlled`](crate::claim::Language::Controlled), the row it
/// carries shall not name that claim as free.
#[derive(Spec)]
pub struct ARowDoesNotNameAHeldClaimAsFree;

/// When [`row`](crate::trace::row) is given a [`SpecMeta`](crate::SpecMeta) and
/// the implementation edges, the row it carries shall carry the
/// [`citations`](crate::trace::citations) of that claim among those edges.
#[derive(Spec)]
pub struct ARowCarriesItsClaimsImplementingItems;

/// When [`row`](crate::trace::row) is given a [`SpecMeta`](crate::SpecMeta) and
/// the validation edges, the row it carries shall carry the
/// [`citations`](crate::trace::citations) of that claim among those edges.
#[derive(Spec)]
pub struct ARowCarriesItsClaimsValidatingItems;

// ---- `citations`: the citing items, and the site each stands at --------------

/// When [`citations`](crate::trace::citations) is given an
/// [`Edge`](crate::Edge) citing the claim it was given, it shall name that
/// edge's item with its file and line as `item (file:line)`.
#[derive(Spec)]
pub struct ACitationNamesTheCitingItemWithItsSite;

/// When [`citations`](crate::trace::citations) is given an
/// [`Edge`](crate::Edge) citing another claim, it shall not name that edge's
/// item.
#[derive(Spec)]
pub struct ACitationIsNotMadeForAnotherClaimsEdge;

// ---- `ledger`: the ramp the registry can count today -------------------------

/// When [`ledger`](crate::trace::ledger) is given claims, the controlled count
/// of the [`Ledger`](crate::trace::Ledger) it carries shall be how many of them
/// carry [`Language::Controlled`](crate::claim::Language::Controlled).
#[derive(Spec)]
pub struct TheControlledCountIsTheClaimsHeldToTheLanguage;

/// When [`ledger`](crate::trace::ledger) is given claims, the free count of the
/// [`Ledger`](crate::trace::Ledger) it carries shall be how many of them carry
/// [`Language::Free`](crate::claim::Language::Free).
#[derive(Spec)]
pub struct TheFreeCountIsTheClaimsMarkedFree;

// ---- `committed`: the document as it stands, or its absence ------------------

/// When [`committed`](crate::trace::committed) is given a path a file stands at,
/// it shall carry that file's text.
#[derive(Spec)]
pub struct TheCommittedDocumentIsTheFilesText;

/// When [`committed`](crate::trace::committed) is given a path no file stands
/// at, it shall carry the empty string.
#[derive(Spec)]
pub struct AnAbsentDocumentIsTheEmptyDocument;

// ---- `stale`: check 26's verdict ---------------------------------------------

/// When [`stale`](crate::trace::stale) is given a generated document and a
/// committed document that equal one another, it shall carry
/// [`Option::None`].
#[derive(Spec)]
pub struct AnAgreeingDocumentIsNotStale;

/// When [`stale`](crate::trace::stale) is given a generated document differing
/// from the committed one, the message it carries shall name the first line at
/// which the two differ.
#[derive(Spec)]
pub struct AStaleDocumentIsNamedByItsFirstDifferingLine;

/// When [`stale`](crate::trace::stale) is given a generated document differing
/// from the committed one, the message it carries shall name the
/// `cargo test --lib -- --ignored regen` invocation that writes the document
/// again.
#[derive(Spec)]
pub struct AStaleDocumentsMessageNamesTheRegenCommand;
