//! Claims for the outcome slice — a signature keeps the promise its claim
//! makes, for the two of README `§4.7`'s tier-1 checks that need no syntax
//! (`lid-rs/src/outcome/lld.md`).
//!
//! The slice is checks 21 and 22 and nothing else: `derive(Outcome)` and the
//! `OUTCOMES` registration it fills, E1 as a compile-time bound emitted beside
//! an unwanted claim, E2 as an intersection of the two registries, and the
//! canary enum that tells a stripped `OUTCOMES` from an empty one. The claims
//! below are those four groups.
//!
//! **E1's claims say what the emission is, never what a check reports.** A
//! claim that fails E1 does not compile — a nonexistent variant is `E0599`, an
//! enum without the derive is `E0277` — so there is no run in which anything
//! reports it, and a claim worded as a report could be validated by no test
//! and no fixture. Each of the four states what `derive(Spec)`'s expansion
//! carries, which a trybuild fixture observes by compiling or failing to.
//! E2's claims are ordinary and say what the work leaves report.
//!
//! **E1's emission is four claims, because it packs four independently
//! falsifiable parts.** The document states it as one rule — "for an unwanted
//! claim whose object is `Owner::Variant`, `derive(Spec)` can emit a const
//! block that bounds `Owner` on `Outcome` and matches the variant" — and each
//! part fails on its own with a diagnostic of its own. An expansion that
//! emitted the block for every pattern falsifies
//! [`AClaimOutsideTheUnwantedPatternCarriesNoOutcomeBound`] and no other; one
//! that emitted it for an empty `owner` falsifies
//! [`AnUnwantedClaimWithAnEmptyOwnerCarriesNoOutcomeBound`] and would name a
//! variant of nothing; one that matched the variant without the bound loses
//! `E0277` for an underived enum
//! ([`AnUnwantedClaimNamingAVariantBoundsItsOwnerOnOutcome`]); one that bounds
//! without matching loses `E0599` for a variant that does not exist
//! ([`TheOutcomeBoundNamesTheObjectsLastSegmentAsAVariant`]). Cut as one claim,
//! check 14 would bind all four diagnostics to a single fixture harness and any
//! of the four regressions would read as the same failure.
//!
//! **What an empty `owner` means is cited, not restated.** The interpretation —
//! an object that does not end in two capitalised segments names no variant, so
//! its owner is empty — is the `claim` slice's delivered
//! [`TheOwnerIsEmptyForAnObjectWithoutAVariant`](crate::claim::spec::TheOwnerIsEmptyForAnObjectWithoutAVariant),
//! and the two claims that turn on it link it rather than saying it again. What
//! is this slice's, and new, is what the emission and the intersection *do* with
//! that empty owner: emit no bound, and count no owner.
//!
//! **E2 is five claims over two leaves, plus the crate scope.**
//! [`claimed_owners`](crate::outcome::claimed_owners) answers which enums are in
//! scope — README's rule is "enums that own at least one claimed variant" — and
//! its three claims are the owner it carries and the two claims it must pass
//! over, a pattern that is not unwanted and an owner that is empty. An
//! implementation filtering on the pattern alone would carry the empty string as
//! an owner and put every enum in scope, which is why those are two claims and
//! not one. [`unclaimed_variants`](crate::outcome::unclaimed_variants) answers
//! which variants count as claimed and what the report holds: a variant an
//! unwanted claim names is passed over, a variant of a claimed owner that no
//! unwanted claim names is reported, and the text of a reported variant carries
//! the site a reader needs to find it. The scope claim,
//! [`UnclaimedVariantsScopeToTheInvokingCrate`], is this slice's own and not the
//! graph slice's `GraphChecksScopeToTheCurrentCrate`: the reasoning transfers,
//! the claim does not, because the mutants of this function are killed by tests
//! written against this function.
//!
//! **No claim here fixes how a claim's `owner` is spelled against an enum's
//! [`NAME`](crate::outcome::Outcome::NAME).** The document's Decisions row makes
//! the claim side emit `<Owner as Outcome>::NAME`, so that "both sides then come
//! from one const and cannot disagree"; its Shape row makes
//! [`claimed_owners`](crate::outcome::claimed_owners) "the one place
//! `ClaimMeta.owner`'s full-path-versus-last-segment distinction is resolved",
//! which reads as a comparison of two written paths reduced to their last
//! segments. The two readings need different code and a different registration —
//! the first needs a key no delivered field of `ClaimMeta` carries — and
//! choosing between them is an edit to the document, not this phase's. Every
//! claim below is therefore worded over "the enum a claimed owner names" and
//! holds under either.
//!
//! **The canary is one claim of its own, which was the document's Deferred 6.**
//! [`TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked`] is separately
//! falsifiable from every E2 claim: registering the canary under `cfg(test)`
//! leaves each E2 claim true and leaves a downstream binary unable to tell an
//! `OUTCOMES` that was stripped from one that is legitimately empty. That is the
//! registry slice's precedent, which made presence a claim with a validation
//! edge of its own.
//!
//! **What has no claim here, and where it lives.** Checks 19 and 20 are
//! Deferred 1 and belong to the change that adds them; `conformance.level` and
//! `aliases` cannot be read from where these checks run, so no claim states a
//! severity; span recording on the trait is slice 20's; the emitted
//! `every_variant_has_a_claim` test and the `OUTCOME` dump line are the graph
//! slice's, so nothing here claims an entry point or a dump format. E1's and
//! `derive(Outcome)`'s implementers are in `lid-rs-macros`, which links into no
//! binary: their edges are hand-authored at the re-export in this crate's root,
//! as the `claim` and `lid_rs_macros` slices' are, and they land with the
//! derive's hand commits rather than with a phase of this slice.
//!
//! **The controlled language was held by hand.** `lid-rs/src/lib.rs` declares no
//! `pub mod outcome;` yet — the document lands that line between this phase and
//! Phase 3 — so nothing compiles this file and check 13 does not run on it.
//! Every verb below is the base lexicon's (`be`, `carry`, `name`), which is what
//! a published crate answers to whatever a project's file says; every claim is
//! one sentence with one `shall` and one terminator; and every trigger names a
//! Rust item by intra-doc link, placed first in its clause so the link the
//! derive records is the one intended. The items Phase 3 has yet to create are
//! linked by the paths the document's Shape table gives. None of these claims is
//! marked free.

use lid_rs::Spec;

// ---- `derive(Outcome)`: the trait's key, and the registration ----------------

/// When an enum derives [`Outcome`](crate::outcome::Outcome), the
/// [`NAME`](crate::outcome::Outcome::NAME) its implementation carries shall be
/// that enum's definition-site module path joined with the enum identifier.
#[derive(Spec)]
pub struct TheOutcomeNameIsTheEnumsDefinitionSitePath;

/// When an enum derives [`Outcome`](crate::outcome::Outcome),
/// [`OUTCOMES`](crate::OUTCOMES) shall carry one
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) for each variant that enum
/// declares.
#[derive(Spec)]
pub struct EachVariantOfAnOutcomeEnumRegistersIntoOutcomes;

/// When an enum derives [`Outcome`](crate::outcome::Outcome), the owning-enum
/// key of each [`OutcomeMeta`](crate::outcome::OutcomeMeta) registered for it
/// shall be the [`NAME`](crate::outcome::Outcome::NAME) read through that
/// enum's own implementation, rather than a path written a second time.
#[derive(Spec)]
pub struct AnOutcomeRegistrationIsKeyedByTheEnumsOutcomeName;

/// When an enum derives [`Outcome`](crate::outcome::Outcome), each
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) registered for it shall name
/// the identifier of the variant it stands for.
#[derive(Spec)]
pub struct AnOutcomeRegistrationNamesTheVariantItStandsFor;

/// When an enum derives [`Outcome`](crate::outcome::Outcome), each
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) registered for it shall carry
/// the file and the line its registration stands at.
#[derive(Spec)]
pub struct AnOutcomeRegistrationCarriesTheSiteItStandsAt;

// ---- E1 (check 21): the bound `derive(Spec)` emits beside an unwanted claim --

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) carries
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and a non-empty
/// `owner`, the derive's expansion shall carry a const block bounding that
/// owner on [`Outcome`](crate::outcome::Outcome).
#[derive(Spec)]
pub struct AnUnwantedClaimNamingAVariantBoundsItsOwnerOnOutcome;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) carries
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and a non-empty
/// `owner`, the const block the derive's expansion carries shall name the last
/// segment of that claim's `object` as a variant of that owner.
#[derive(Spec)]
pub struct TheOutcomeBoundNamesTheObjectsLastSegmentAsAVariant;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) carries a
/// [`Pattern`](crate::claim::Pattern) other than
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted), the derive's
/// expansion shall not carry a const block bounding its owner on
/// [`Outcome`](crate::outcome::Outcome).
#[derive(Spec)]
pub struct AClaimOutsideTheUnwantedPatternCarriesNoOutcomeBound;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) carries
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and the empty `owner`
/// that
/// [`TheOwnerIsEmptyForAnObjectWithoutAVariant`](crate::claim::spec::TheOwnerIsEmptyForAnObjectWithoutAVariant)
/// records, the derive's expansion shall not carry a const block bounding an
/// owner on [`Outcome`](crate::outcome::Outcome).
#[derive(Spec)]
pub struct AnUnwantedClaimWithAnEmptyOwnerCarriesNoOutcomeBound;

// ---- E2 (check 22), first side: which enums the check is about ---------------

/// When [`claimed_owners`](crate::outcome::claimed_owners) is given a
/// [`SpecMeta`](crate::SpecMeta) whose claim carries
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and a non-empty
/// `owner`, it shall carry that owner.
#[derive(Spec)]
pub struct AnUnwantedClaimsOwnerIsAClaimedOwner;

/// When [`claimed_owners`](crate::outcome::claimed_owners) is given a
/// [`SpecMeta`](crate::SpecMeta) whose claim carries a
/// [`Pattern`](crate::claim::Pattern) other than
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted), it shall not carry
/// that claim's `owner`.
#[derive(Spec)]
pub struct AClaimOutsideTheUnwantedPatternIsNoClaimedOwner;

/// When [`claimed_owners`](crate::outcome::claimed_owners) is given a
/// [`SpecMeta`](crate::SpecMeta) whose claim carries
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and the empty `owner`
/// that
/// [`TheOwnerIsEmptyForAnObjectWithoutAVariant`](crate::claim::spec::TheOwnerIsEmptyForAnObjectWithoutAVariant)
/// records, it shall not carry an owner for that claim.
#[derive(Spec)]
pub struct AnUnwantedClaimWithAnEmptyOwnerIsNoClaimedOwner;

// ---- E2, second side: which variants are claimed, and what is reported -------

/// When [`unclaimed_variants`](crate::outcome::unclaimed_variants) is given an
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) whose enum is named by no owner
/// [`claimed_owners`](crate::outcome::claimed_owners) carries, it shall not
/// name that variant.
#[derive(Spec)]
pub struct AVariantOfAnEnumNoClaimOwnsIsNotReported;

/// When [`unclaimed_variants`](crate::outcome::unclaimed_variants) is given an
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) that the `object` of an
/// unwanted claim it was given names, it shall not name that variant.
#[derive(Spec)]
pub struct AVariantAnUnwantedClaimNamesIsNotReported;

/// When [`unclaimed_variants`](crate::outcome::unclaimed_variants) is given an
/// [`OutcomeMeta`](crate::outcome::OutcomeMeta) whose enum a claimed owner
/// names and whose variant the `object` of no unwanted claim names, it shall
/// name that variant.
#[derive(Spec)]
pub struct AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported;

/// When [`unclaimed_variants`](crate::outcome::unclaimed_variants) names a
/// variant, the text it carries for that variant shall be the registered name
/// of the variant with the file and the line it stands at, formatted
/// `name (file:line)`.
#[derive(Spec)]
pub struct AReportedVariantIsNamedWithItsFileAndLine;

/// When [`unclaimed_variants`](crate::outcome::unclaimed_variants) is given a
/// [`SpecMeta`](crate::SpecMeta) that a crate other than the one it was given
/// registered, it shall not name a variant of the enum that claim owns.
#[derive(Spec)]
pub struct UnclaimedVariantsScopeToTheInvokingCrate;

// ---- The fourth registry's canary --------------------------------------------

/// When a binary links the crate that ships
/// [`CanaryOutcome`](crate::outcome::canary::CanaryOutcome),
/// [`OUTCOMES`](crate::OUTCOMES) shall carry one entry for each variant that
/// enum declares, its registration standing outside `cfg(test)`.
#[derive(Spec)]
pub struct TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked;
