//! Claims for the validate slice — a validator's trace says which claims it
//! reached, and how they closed (`lid-rs/src/validate/lld.md`).
//!
//! The slice is one type and a set of functions over it:
//! [`Capture`](crate::validate::Capture) builds a
//! [`Trace`](crate::validate::Trace),
//! a [`Returned`](crate::validate::Returned) guard closes each span's outcome,
//! [`captured`](crate::validate::captured) is the seam that hands one back,
//! [`unreached`](crate::validate::unreached) is check 23,
//! [`wrong_outcome`](crate::validate::wrong_outcome) is check 24's dispatch over
//! [`ClaimMeta::pattern`](crate::claim::ClaimMeta::pattern) with a rule under
//! each arm, and [`render`](crate::validate::render) prints the tree. The claims
//! below are those items, in that order.
//!
//! **Every check and every rule is given a
//! [`SpecMeta`](crate::registry::SpecMeta), not a
//! [`ClaimMeta`](crate::claim::ClaimMeta).** A rule's first act is to select the
//! spans whose `lid.claims` holds the claim's name, and a name is what
//! `SpecMeta` has (`lid-rs/src/registry/mod.rs:6-22`) and `ClaimMeta` does not
//! (`lid-rs/src/claim/mod.rs:57-78`); a free claim's `ClaimMeta` is
//! `Ubiquitous` with every part empty, so 294 of the workspace's 471 claims
//! share one value and no rule could tell them apart. The pattern and the
//! expected object are still `ClaimMeta`'s and are still linked as such, since
//! `SpecMeta::claim` is where they live.
//!
//! **Two words are used throughout and mean one thing each.** A rule is
//! *satisfied* by a trace when the pattern's required trace is exhibited —
//! [`event_driven`](crate::validate::event_driven) and its six siblings answer
//! that question and nothing else. A *finding* is a check firing:
//! [`unreached`](crate::validate::unreached) carries check 23's and
//! [`wrong_outcome`](crate::validate::wrong_outcome) carries check 24's, so a
//! finding is the absence of satisfaction and the two polarities are opposite on
//! purpose. Fixing that here rather than at Phase 3 is what stops the dispatch
//! and the rules under it from being written to disagree, since the LLD names
//! both `wrong_outcome` — check 24's own name, which is a failure — and seven
//! rules named for the property they hold.
//!
//! **No claim below is about the emission, which is the LLD's Open question 2
//! answered no.** `#[implements]`'s span and `#[validates]`'s capture land in
//! `lid-rs-macros/src/expand.rs` as a hand cascade after this slice's leaves, so
//! a claim about a span being opened or `lid.claims` being written would have no
//! implementer in `lid-rs/src/validate/` at all, and the only test that could
//! redden it is a test that runs a traced function — which Phase 5 note 1
//! forbids, because at Phase 5 the emission does not exist. The wire contract is
//! held instead by the claims of the items that *read* it: what
//! [`Capture`](crate::validate::Capture) records of a span, and what each rule
//! makes of the fields the contract names. A claim derived here and reddened
//! four commits later is a claim no phase's red set can express, which is the
//! honest option the document rejected.
//!
//! **No claim is cited on a constant, and two constants are claimed through the
//! functions that read them.** `const TARGET: &str = todo!();` does not compile,
//! so a test of a constant is green from the Phase 3 skeleton and `red_verdict`
//! refuses it (`cargo-lid-rs/src/phase/mod.rs:912-919`). `TARGET`'s value is
//! therefore claimed by [`ASpanAtTheLidTargetEntersTheTrace`] and
//! [`ASpanAtAnotherTargetIsNotCaptured`], which are about the target filter, and
//! `MIN_INPUTS`'s by
//! [`TheEnoughSpansRuleIsSatisfiedByMinInputsCitingSpans`], which is about the
//! count. This is the rule vocab stated for `Traceable::NOUN` and slice 21 for
//! `DOCUMENT`.
//!
//! **`SPAN_LEVEL` and `LEVEL` carry no claim, because nothing in this slice
//! reads them.** The level a span opens at is read by the emission, and the ramp
//! level is read by `#[validates]`, and both of those are in `lid-rs-macros`. A
//! claim about either would be cited on the constant itself — the one thing the
//! paragraph above rules out — so the ramp is documented in the LLD's decisions
//! table and Deferred 8 rather than asserted here. It is the one part of the
//! slice's behaviour no claim of this branch holds.
//!
//! **Each arm of check 24's dispatch is a claim, and so is the conjunction under
//! it.** [`wrong_outcome`](crate::validate::wrong_outcome) is the slice's one
//! flow node and every branch of it is a decision, so five claims name which
//! rule reads the trace for which [`Pattern`](crate::claim::Pattern), and
//! [`ASatisfiedRuleIsNoFinding`] holds the other direction for all five at once
//! — without it, a dispatch that always found something would keep every one of
//! them. The seven rules get two claims apiece, one per direction, because a
//! rule answering a constant keeps whichever direction its constant agrees with;
//! [`ubiquitous`](crate::validate::ubiquitous) gets the same two over the
//! conjunction of its three, which is what makes dropping one conjunct
//! falsifiable.
//!
//! **The `✓ promised` mark is claimed as the absence of a finding, which the LLD
//! left unsaid.** README's worked output puts the mark on one span's line and
//! says nothing about when it is earned; the renderer takes a
//! [`Trace`](crate::validate::Trace) and some claims and can therefore know
//! exactly one thing about a claim — whether
//! [`wrong_outcome`](crate::validate::wrong_outcome) found anything for it.
//! [`TheRenderedTreeMarksAKeptClaimAsPromised`] and
//! [`TheRenderedTreeLeavesAClaimWithAFindingUnmarked`] say so from both sides,
//! rather than leaving Phase 5 to assert a mark that no claim gives a meaning.
//! The `[flow]` marker and the `[redacted]` field are Deferred 6 and are claimed
//! nowhere: an absence that is true of the empty string is not an assertion.
//!
//! **What else has no claim here, and where it lives.** Nothing claims
//! `lid_rs::spawn`, the root-keyed global layer, check 25, trace persistence or
//! the `cargo lid-rs validate` command — Deferred 1 through 4 and 9, each in a
//! crate or a runtime no phase of this branch may reach. Nothing claims
//! `[workspace.metadata.lid_rs.runtime]`, which no crate depended on from here
//! can read (Deferred 5). Nothing claims the policy attributes or a recorded
//! parameter *value*: a span records a parameter's noun, and the recording
//! method that would give it a value is a breaking change vocab's Open 1 handed
//! forward (Deferred 7). And nothing claims `lid_rs::__private::tracing`, which
//! is a re-export observed only by a path compiling.
//!
//! **The controlled language was held by hand.** `lid-rs/src/lib.rs` declares no
//! `pub mod validate;` yet — the document lands that line between this phase and
//! Phase 3 — so nothing compiles this file and check 13 does not run on it.
//! Every verb below is the base lexicon's (`be`, `carry`, `name`), which is what
//! a published crate answers to whatever a project's file says; every claim is
//! one sentence with one `shall` and one terminator; every field name holding a
//! period is inside backticks, where no terminator and no clause-closing comma
//! is read; and every trigger names a Rust item by intra-doc link, placed first
//! in its clause so the link the derive records is the one intended. The items
//! Phase 3 has yet to create are linked by the paths the document's Shape table
//! gives. None of these claims is marked free.
//!
//! **No claim's name displaces the eighth `Spec` implementor.**
//! `lid-rs/tests/ui/fail/not_a_spec.stderr` pins rustc's alphabetically
//! truncated list, whose eighth entry is
//! [`ACrateRootClaimsSliceIsEmpty`](crate::trace::spec::ACrateRootClaimsSliceIsEmpty),
//! and every name below sorts after it. Slice 21 re-blessed that fixture with
//! `TRYBUILD=overwrite` when its claims did not; here the fixture is in no phase
//! row of this slice, so the re-bless would have been a hand commit, and naming
//! around it costs nothing.

use lid_rs::Spec;

// ---- `Capture`: which spans enter the tree -----------------------------------

/// When a span whose target is `lid` is opened under
/// [`Capture`](crate::validate::Capture), the [`Trace`](crate::validate::Trace)
/// it captured shall carry that span.
#[derive(Spec)]
pub struct ASpanAtTheLidTargetEntersTheTrace;

/// When a span whose target is not `lid` is opened under
/// [`Capture`](crate::validate::Capture), the [`Trace`](crate::validate::Trace)
/// it captured shall not carry that span.
#[derive(Spec)]
pub struct ASpanAtAnotherTargetIsNotCaptured;

/// When [`Capture`](crate::validate::Capture) is given a span opened inside
/// [`validate`](crate::validate), it shall not carry that span.
#[derive(Spec)]
pub struct ASpanOpenedInsideTheCaptureIsNotCaptured;

/// When [`Capture`](crate::validate::Capture) is asked its interest in a
/// callsite, it shall carry the `Interest::sometimes` interest rather than one
/// derived from its target filter.
#[derive(Spec)]
pub struct TheCapturesInterestInACallsiteIsSometimes;

// ---- `Capture`: what a recorded span holds -----------------------------------

/// When [`Capture`](crate::validate::Capture) records a span, the
/// [`Span`](crate::validate::Span) it records shall carry the name that span
/// was opened with.
#[derive(Spec)]
pub struct ARecordedSpanCarriesTheNameItWasOpenedWith;

/// When [`Capture`](crate::validate::Capture) records a span opened with
/// fields, the [`Span`](crate::validate::Span) it records shall carry those
/// fields.
#[derive(Spec)]
pub struct ARecordedSpanCarriesTheFieldsItWasOpenedWith;

/// When [`Capture`](crate::validate::Capture) records a span whose
/// `lid.claims` field names several claims separated by `\x1f`, the
/// [`Span`](crate::validate::Span) it records shall carry each of those claims.
#[derive(Spec)]
pub struct TheClaimsFieldIsSplitAtTheUnitSeparator;

/// When [`Capture`](crate::validate::Capture) is given a field recorded after a
/// span was opened, the [`Span`](crate::validate::Span) it records for that
/// span shall carry that field.
#[derive(Spec)]
pub struct AFieldRecordedAfterCreationReachesItsSpan;

/// When [`Capture`](crate::validate::Capture) records a span carrying a
/// `lid.outcome` field, the [`Span`](crate::validate::Span) it records shall
/// carry that field as its
/// [`outcome`](crate::validate::Span::outcome).
#[derive(Spec)]
pub struct TheOutcomeFieldIsCarriedAsTheSpansOutcome;

// ---- `Capture`: the enter-stack that gives every span its parent -------------

/// When [`Capture`](crate::validate::Capture) records a span opened while
/// another span is entered, the parent of the
/// [`Span`](crate::validate::Span) it records shall be that entered span.
#[derive(Spec)]
pub struct ARecordedSpansParentIsTheSpanThatWasEntered;

/// When [`Capture`](crate::validate::Capture) records a span opened while it
/// has entered no span, the [`Span`](crate::validate::Span) it records shall
/// not name a parent.
#[derive(Spec)]
pub struct ASpanOpenedWithNoSpanEnteredHasNoParent;

/// When [`Capture`](crate::validate::Capture) records a span opened after an
/// entered span was exited, the parent of the
/// [`Span`](crate::validate::Span) it records shall be the span entered before
/// that one.
#[derive(Spec)]
pub struct ASpanOpenedAfterAnExitTakesTheEnclosingParent;

// ---- `Returned`: the guard that says the body returned -----------------------

/// When a [`Returned`](crate::validate::Returned) guard is dropped while its
/// thread is not panicking, the [`Span`](crate::validate::Span) recorded for
/// the span it guards shall carry an
/// [`outcome`](crate::validate::Span::outcome).
#[derive(Spec)]
pub struct TheReturnedGuardRecordsAnOutcomeWhenItsBodyReturns;

/// When a [`Returned`](crate::validate::Returned) guard is dropped while its
/// thread is panicking, the [`Span`](crate::validate::Span) recorded for the
/// span it guards shall not carry an
/// [`outcome`](crate::validate::Span::outcome).
#[derive(Spec)]
pub struct TheReturnedGuardRecordsNoOutcomeWhileItsBodyUnwinds;

// ---- `captured`: the seam between the emission and the checks ----------------

/// When [`captured`](crate::validate::captured) is given a closure that opens a
/// span at the `lid` target, the [`Trace`](crate::validate::Trace) it answers
/// with shall carry that span.
#[derive(Spec)]
pub struct TheCapturedTraceCarriesTheSpansOpenedInTheClosure;

/// When [`captured`](crate::validate::captured) is given a closure after an
/// earlier closure opened a span, the [`Trace`](crate::validate::Trace) it
/// answers with shall not carry that earlier span.
#[derive(Spec)]
pub struct TheCapturedTraceCarriesNoEarlierRunsSpans;

/// When [`captured`](crate::validate::captured) is given a closure opening
/// several spans, the [`Trace`](crate::validate::Trace) it answers with shall
/// carry them in the order they were opened.
#[derive(Spec)]
pub struct TheCapturedTraceIsInTheOrderItsSpansWereOpened;

// ---- Check 23: the claim no span cites ---------------------------------------

/// When [`unreached`](crate::validate::unreached) is given a
/// [`Trace`](crate::validate::Trace) in which no span cites the claim it was
/// given, it shall carry a finding.
#[derive(Spec)]
pub struct NoSpanCitingTheClaimIsAFinding;

/// When [`unreached`](crate::validate::unreached) is given a
/// [`Trace`](crate::validate::Trace) in which a span cites the claim it was
/// given, it shall not carry a finding.
#[derive(Spec)]
pub struct ASpanCitingTheClaimIsNoFinding;

// ---- Check 24: the dispatch over the claim's pattern -------------------------

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`SpecMeta`](crate::registry::SpecMeta) carrying
/// [`Pattern::EventDriven`](crate::claim::Pattern::EventDriven) and a
/// [`Trace`](crate::validate::Trace) that
/// [`event_driven`](crate::validate::event_driven) is not satisfied by, it
/// shall carry a finding.
#[derive(Spec)]
pub struct TheEventDrivenRuleDecidesAnEventDrivenClaim;

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`SpecMeta`](crate::registry::SpecMeta) carrying
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and a
/// [`Trace`](crate::validate::Trace) that
/// [`unwanted`](crate::validate::unwanted) is not satisfied by, it shall carry
/// a finding.
#[derive(Spec)]
pub struct TheUnwantedRuleDecidesAnUnwantedClaim;

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`SpecMeta`](crate::registry::SpecMeta) carrying
/// [`Pattern::Optional`](crate::claim::Pattern::Optional) and a
/// [`Trace`](crate::validate::Trace) that
/// [`optional`](crate::validate::optional) is not satisfied by, it shall carry
/// a finding.
#[derive(Spec)]
pub struct TheOptionalRuleDecidesAnOptionalClaim;

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`SpecMeta`](crate::registry::SpecMeta) carrying
/// [`Pattern::StateDriven`](crate::claim::Pattern::StateDriven) and a
/// [`Trace`](crate::validate::Trace) that
/// [`state_driven`](crate::validate::state_driven) is not satisfied by, it
/// shall carry a finding.
#[derive(Spec)]
pub struct TheStateDrivenRuleDecidesAStateDrivenClaim;

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`SpecMeta`](crate::registry::SpecMeta) carrying
/// [`Pattern::Ubiquitous`](crate::claim::Pattern::Ubiquitous) and a
/// [`Trace`](crate::validate::Trace) that
/// [`ubiquitous`](crate::validate::ubiquitous) is not satisfied by, it shall
/// carry a finding.
#[derive(Spec)]
pub struct TheUbiquitousRuleDecidesAUbiquitousClaim;

/// When [`wrong_outcome`](crate::validate::wrong_outcome) is given a
/// [`Trace`](crate::validate::Trace) that the rule of its
/// [`SpecMeta`](crate::registry::SpecMeta)'s
/// [`pattern`](crate::claim::ClaimMeta::pattern) is satisfied by, it shall not
/// carry a finding.
#[derive(Spec)]
pub struct ASatisfiedRuleIsNoFinding;

// ---- Check 24's event-driven rule --------------------------------------------

/// When [`event_driven`](crate::validate::event_driven) is given a
/// [`Trace`](crate::validate::Trace) in which a span citing the claim closed
/// `Ok` of the claim's [`object`](crate::claim::ClaimMeta::object), it shall be
/// satisfied.
#[derive(Spec)]
pub struct TheEventDrivenRuleIsSatisfiedByAnOkOutcome;

/// When [`event_driven`](crate::validate::event_driven) is given a
/// [`Trace`](crate::validate::Trace) in which no span citing the claim closed
/// `Ok` of the claim's [`object`](crate::claim::ClaimMeta::object), it shall
/// not be satisfied.
#[derive(Spec)]
pub struct TheEventDrivenRuleIsNotSatisfiedWithoutAnOkOutcome;

// ---- Check 24's unwanted rule ------------------------------------------------

/// When [`unwanted`](crate::validate::unwanted) is given a
/// [`Trace`](crate::validate::Trace) in which a span citing the claim closed
/// `Err` of the claim's [`object`](crate::claim::ClaimMeta::object), it shall
/// be satisfied.
#[derive(Spec)]
pub struct TheUnwantedRuleIsSatisfiedByAnErrOutcome;

/// When [`unwanted`](crate::validate::unwanted) is given a
/// [`Trace`](crate::validate::Trace) in which no span citing the claim closed
/// `Err` of the claim's [`object`](crate::claim::ClaimMeta::object), it shall
/// not be satisfied.
#[derive(Spec)]
pub struct TheUnwantedRuleIsNotSatisfiedWithoutAnErrOutcome;

// ---- Check 24's optional rule ------------------------------------------------

/// When [`optional`](crate::validate::optional) is given a
/// [`Trace`](crate::validate::Trace) in which a span citing the claim carries a
/// `lid.feature` field equal to the claim's
/// [`object`](crate::claim::ClaimMeta::object), it shall be satisfied.
#[derive(Spec)]
pub struct TheOptionalRuleIsSatisfiedByASpanCarryingTheFeature;

/// When [`optional`](crate::validate::optional) is given a
/// [`Trace`](crate::validate::Trace) in which no span citing the claim carries
/// a `lid.feature` field equal to the claim's
/// [`object`](crate::claim::ClaimMeta::object), it shall not be satisfied.
#[derive(Spec)]
pub struct TheOptionalRuleIsNotSatisfiedWithoutThatFeature;

// ---- Check 24's state-driven rule --------------------------------------------

/// When [`state_driven`](crate::validate::state_driven) is given a
/// [`Trace`](crate::validate::Trace) in which the spans citing the claim carry
/// two distinct `lid.state` values, it shall be satisfied.
#[derive(Spec)]
pub struct TheStateDrivenRuleIsSatisfiedByTwoDistinctStates;

/// When [`state_driven`](crate::validate::state_driven) is given a
/// [`Trace`](crate::validate::Trace) in which the spans citing the claim carry
/// fewer than two distinct `lid.state` values, it shall not be satisfied.
#[derive(Spec)]
pub struct TheStateDrivenRuleIsNotSatisfiedByOneState;

// ---- Check 24's ubiquitous rule, and the three it conjoins -------------------

/// When [`ubiquitous`](crate::validate::ubiquitous) is given a
/// [`Trace`](crate::validate::Trace) that
/// [`enough_spans`](crate::validate::enough_spans) and
/// [`distinct_inputs`](crate::validate::distinct_inputs) and
/// [`no_panics`](crate::validate::no_panics) are satisfied by, it shall be
/// satisfied.
#[derive(Spec)]
pub struct TheUbiquitousRuleIsSatisfiedByItsThreeRulesTogether;

/// When [`ubiquitous`](crate::validate::ubiquitous) is given a
/// [`Trace`](crate::validate::Trace) that one of
/// [`enough_spans`](crate::validate::enough_spans) and
/// [`distinct_inputs`](crate::validate::distinct_inputs) and
/// [`no_panics`](crate::validate::no_panics) is not satisfied by, it shall not
/// be satisfied.
#[derive(Spec)]
pub struct TheUbiquitousRuleIsNotSatisfiedWhenOneOfItsRulesIsNot;

/// When [`enough_spans`](crate::validate::enough_spans) is given a
/// [`Trace`](crate::validate::Trace) holding at least
/// [`MIN_INPUTS`](crate::validate::MIN_INPUTS) spans citing the claim, it shall
/// be satisfied.
#[derive(Spec)]
pub struct TheEnoughSpansRuleIsSatisfiedByMinInputsCitingSpans;

/// When [`enough_spans`](crate::validate::enough_spans) is given a
/// [`Trace`](crate::validate::Trace) holding fewer spans citing the claim than
/// [`MIN_INPUTS`](crate::validate::MIN_INPUTS), it shall not be satisfied.
#[derive(Spec)]
pub struct TheEnoughSpansRuleIsNotSatisfiedByFewerCitingSpans;

/// When [`distinct_inputs`](crate::validate::distinct_inputs) is given a
/// [`Trace`](crate::validate::Trace) in which the spans citing the claim carry
/// differing `lid.noun` fields, it shall be satisfied.
#[derive(Spec)]
pub struct TheDistinctInputsRuleIsSatisfiedByDifferingNouns;

/// When [`distinct_inputs`](crate::validate::distinct_inputs) is given a
/// [`Trace`](crate::validate::Trace) in which two spans citing the claim carry
/// the same `lid.noun` fields, it shall not be satisfied.
#[derive(Spec)]
pub struct TheDistinctInputsRuleIsNotSatisfiedByRepeatedNouns;

/// When [`no_panics`](crate::validate::no_panics) is given a
/// [`Trace`](crate::validate::Trace) in which every span citing the claim
/// recorded a `lid.outcome`, it shall be satisfied.
#[derive(Spec)]
pub struct TheNoPanicsRuleIsSatisfiedByAnOutcomeOnEveryCitingSpan;

/// When [`no_panics`](crate::validate::no_panics) is given a
/// [`Trace`](crate::validate::Trace) in which a span citing the claim recorded
/// no `lid.outcome`, it shall not be satisfied.
#[derive(Spec)]
pub struct TheNoPanicsRuleIsNotSatisfiedByASpanThatRecordedNoOutcome;

// ---- `render`: the tree as this slice can print it ----------------------------

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace), the tree it answers with shall name every
/// span that trace carries.
#[derive(Spec)]
pub struct TheRenderedTreeNamesEverySpanOfTheTrace;

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace) holding a span with a parent, the line it
/// carries for that span shall be indented below the parent's line by a
/// box-drawing branch.
#[derive(Spec)]
pub struct TheRenderedTreeSetsAChildSpanBelowItsParent;

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace) holding a span citing a claim, the line it
/// carries for that span shall name that claim.
#[derive(Spec)]
pub struct TheRenderedTreeNamesTheClaimBesideTheSpanThatCitesIt;

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace) holding a span that recorded an outcome,
/// the line it carries for that span shall name that outcome.
#[derive(Spec)]
pub struct TheRenderedTreeNamesEachSpansRecordedOutcome;

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace) and a claim
/// [`wrong_outcome`](crate::validate::wrong_outcome) carries no finding for,
/// the line it carries for the span citing that claim shall name that claim as
/// promised.
#[derive(Spec)]
pub struct TheRenderedTreeMarksAKeptClaimAsPromised;

/// When [`render`](crate::validate::render) is given a
/// [`Trace`](crate::validate::Trace) and a claim
/// [`wrong_outcome`](crate::validate::wrong_outcome) carries a finding for, the
/// line it carries for the span citing that claim shall not name that claim as
/// promised.
#[derive(Spec)]
pub struct TheRenderedTreeLeavesAClaimWithAFindingUnmarked;

// ---- `report`: what a validator prints ---------------------------------------

/// When [`report`](crate::validate::report) is given a
/// [`Trace`](crate::validate::Trace) and a claim
/// [`unreached`](crate::validate::unreached) carries a finding for, it shall
/// name that claim as unreached.
#[derive(Spec)]
pub struct TheReportNamesAnUnreachedClaim;

/// When [`report`](crate::validate::report) is given a
/// [`Trace`](crate::validate::Trace) and a claim
/// [`wrong_outcome`](crate::validate::wrong_outcome) carries a finding for, it
/// shall name that claim as a wrong outcome.
#[derive(Spec)]
pub struct TheReportNamesAClaimWithAWrongOutcome;

/// When [`report`](crate::validate::report) is given a
/// [`Trace`](crate::validate::Trace) and a claim that neither
/// [`unreached`](crate::validate::unreached) nor
/// [`wrong_outcome`](crate::validate::wrong_outcome) carries a finding for, it
/// shall be empty.
#[derive(Spec)]
pub struct TheReportOfAClaimWithNoFindingIsEmpty;

/// When [`report`](crate::validate::report) carries a finding, it shall carry
/// the [`Trace`](crate::validate::Trace) as
/// [`render`](crate::validate::render) draws it.
#[derive(Spec)]
pub struct TheReportCarriesTheRenderedTree;

/// When [`report`](crate::validate::report) is given a claim that
/// [`SPECS`](crate::SPECS) does not register, it shall be empty.
#[derive(Spec)]
pub struct TheReportOfAnUnregisteredClaimIsEmpty;
