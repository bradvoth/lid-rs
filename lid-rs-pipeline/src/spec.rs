//! Claims for the `lid-rs-pipeline` slice — a slice's phase is what its branch
//! says (`lid-rs-pipeline/src/lld.md`).
//!
//! The slice builds one command, `status`: the branch's state as a value, the
//! report that value is written into, and the rendering a human reads. The
//! claims below are that state's three-part reading, what else the state
//! carries, the three findings a reading raises, the one occasion it refuses,
//! and the report and rendering built from it.
//!
//! **The state's reading is three claims, not one.** "The newest `phase N:`
//! subject made on this branch" has three independently falsifiable parts, and
//! the document leaves the cut to this phase. As one claim it would need one
//! table-driven validator — check 14 binds a claim to a single validator whose
//! name is the claim's, and check 7's threshold is 4 — so a wrong answer about
//! any one part would be reported as the same validator failing. As three, each
//! part gets a validator that can fail alone, and the middle one —
//! [`ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase`] — is the part
//! pipeline `§4.2` records as having actually been got wrong: reading the whole
//! ancestry skipped every phase of a branch cut after a merge and reported it
//! PR-ready with no session opened. A cut that left that part inside a claim
//! about "the newest" would bury the one failure the specification has already
//! paid for.
//!
//! **Every phase commit's trailers are reported, not only the newest's**, which
//! is what the state's row in the Shape table asks for and what
//! [`EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers`] states. Three things
//! carry it. The branch's own commits are already enumerated in full by one
//! `git log` over the fork point
//! ([`log_subjects`](cargo_lid_rs::headless_canopy_agent::log_subjects)), so
//! every phase commit's trailers are the same range and the same call with a
//! wider format, and not a second read whose length grows with the branch. The
//! `Lid-Rs-Tool` failure path is written of *a* phase commit rather than of the
//! newest, and a resume at Phase 6 across a tool change puts the mismatch on an
//! older commit, which only the whole-branch reading raises. And the trailers
//! are asked for "so a reader sees which agent and which tally produced each
//! phase", which the newest commit's trailers cannot answer. That is why the
//! state holds a [`PhaseCommit`](crate::PhaseCommit) per phase commit rather
//! than one record.
//!
//! **The phase that runs next is the first one with no commit, not the one
//! after the newest.** This is the rule the workspace already runs by: a run
//! walks [`PHASES`](cargo_lid_rs::headless_canopy_agent::PHASES) — 2, 3, 4, 5,
//! 7, Phase 6 having no commit of its own — and skips each phase the branch
//! already holds a commit for, so a branch holding 2, 3 and 5 is resumed at 4
//! and not at 7. A `status` that answered "the newest plus one" would name a
//! phase the pipeline would not run. Stating it as "the first with no commit"
//! also keeps Phase 4's layers out of the claim: a `phase 4:` subject is
//! committed once for every layer, so a branch carrying one has finished
//! Phase 4 as far as any reading of its commits can tell.
//!
//! **No claim reads a layer.** The state carries no layer field, no subject
//! carries a layer — a Phase 4 subject is exactly `phase 4: descend for
//! <slice>` — and the layer's home in the specification is the `Lid-Rs-Layer`
//! trailer (`pipeline.md` `§4.1`, `§5.1`), which nothing writes today. If
//! `commit --phase N --layer L` ever writes it, it arrives as one of the
//! trailers [`EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers`] already
//! carries, and needs no claim of its own then either.
//!
//! **A slice the layout cannot resolve is two claims, because it has two wrong
//! answers.** An implementation that refuses the reading is wrong, and so is
//! one that raises the finding and then answers a restart verdict it had no
//! document to reach. [`AnUnresolvableSliceIsAFindingBesideTheStateAndNotARefusal`]
//! is the first; [`AnUnresolvableSliceLeavesTheRestartVerdictUnanswered`] is the
//! second, and it is the silent one — a reading that answered "resume" for a
//! slice whose `lld.md` was never found would tell the pipeline to carry on
//! from a check nothing made. Both are conditioned on the condition — no
//! workspace member holds a document for the slice the branch names — and not
//! on the door that reports it, because one door cannot:
//! [`own_crate`](cargo_lid_rs::layout::own_crate) refuses by name for such a
//! slice, while [`lld_path`](cargo_lid_rs::layout::lld_path) answers `Ok` in
//! both arms, falling back to a workspace-root path it never probes, and
//! supplies only the path the restart verdict compares against.
//!
//! **A finding is what the reading raises about one commit or one name, and the
//! state is everything else.** The three findings — a subject that opens
//! `phase ` with no readable number, a `Lid-Rs-Tool` trailer this binary does
//! not match, and a slice the layout cannot resolve — are what the finding
//! schema's fields fit, because each is about one thing in one place.
//! Everything else the reading answers is the state, which is not a defect, and
//! [`TheReportCarriesTheStateAndTheFindingsTheReadingRaised`] is what makes the
//! pair one report. That a report carries more than a finding list is already
//! this workspace's answer and not a new one: the catalog's report carries the
//! command it ran and the status its findings amount to beside them
//! (`cargo-lid-rs/src/catalog/mod.rs`). What remains open is whether the
//! specification's blanket "every command writes a report in the finding
//! schema" grows a variant for a reading or names this command an exception —
//! an amendment to `pipeline.md` that changes no claim here.
//!
//! **The 0-and-1 exit mapping is not restated here.** The catalog slice already
//! states it, over a finding list, as
//! `cargo_lid_rs::catalog::spec::TheStatusIsZeroWithNoFindingAndOneWithAFinding`,
//! and this slice's report carries a finding list of the same schema. A second
//! copy of that rule in this crate would be one rule in two places, which is
//! what reusing the catalog's types exists to prevent. The occasion that *is*
//! this slice's — a branch naming no slice at all, where there is no state to
//! answer with — is [`ABranchThatNamesNoSliceIsRefusedNamingTheConvention`],
//! stated of this slice's own item.
//!
//! **Four things these claims turn on that the document's Shape table leaves to
//! the code.** Each is implied by a Behaviour sentence or a failure-path row
//! rather than invented here, and each has a place in the skeleton: the report
//! is written by [`write_report`](crate::write_report) and not by the reading
//! that built it; uncommitted work has a field of its own in the state rather
//! than being a reason to refuse; a restart verdict the reading could not reach
//! is an absent [`Status::restart`](crate::Status::restart) rather than a third
//! [`Restart`](crate::Restart) variant, because the variants are the reasons a
//! run resumes or begins again and "unanswered" is not a reason; and whether
//! the slice exists at all is [`own_crate`](cargo_lid_rs::layout::own_crate)'s
//! answer.
//!
//! **The verbs are the base lexicon's, and no verb was added for this slice.**
//! Three findings and the dirty tree are stated with *report* — "state in the
//! output, without stopping" — which is what each of those four sentences
//! means, the "without stopping" being exactly the half that distinguishes a
//! reading's finding from a refusal. The one sentence that is a refusal uses
//! *refuse*. Nothing here needs a verb that constrains an implementer's return
//! type, so no claim carries a signature obligation beyond the shape its
//! response object already names.
//!
//! **What has no claim here.** `commit`, `pr-body`, the composites, the session
//! loop, publication, `CODEOWNERS` and the rulesets: the document defers each of
//! them, naming the slice or the file that owns it, so a claim for any of them
//! would be a claim about code this slice does not build, with nothing for a
//! validation to attach itself to.
//!
//! None of these claims is marked free: each is written in the controlled
//! language, and the derive reads them from this module.

use lid_rs::Spec;

// ---- The state: the newest `phase N:` subject made on this branch ------------

/// When [`status`](crate::status) reads a branch whose own commits carry a
/// `phase N:` subject, the phase the [`Status`](crate::Status) it answers with
/// names shall be the phase of the newest of those commits.
#[derive(Spec)]
pub struct TheStateIsTheNewestPhaseCommitTheBranchMade;

/// When [`status`](crate::status) reads a branch, a commit the branch's base
/// also reaches shall be no part of the [`Status`](crate::Status) it answers
/// with, whatever subject that commit carries.
#[derive(Spec)]
pub struct ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase;

/// When [`status`](crate::status) reads a commit made on the branch, the phase
/// it takes that commit for shall be the number of the subject's `phase <N>:`
/// prefix, and no phase at all where the subject carries no such prefix.
#[derive(Spec)]
pub struct APhaseIsTheNumberOfItsSubjectsPhasePrefix;

// ---- What else the state carries ---------------------------------------------

/// When [`status`](crate::status) answers for a branch, the phase it names as
/// next shall be the first of the phases a run builds
/// ([`PHASES`](cargo_lid_rs::headless_canopy_agent::PHASES)) that the branch's
/// own commits hold no commit for.
#[derive(Spec)]
pub struct ThePhaseNextIsTheFirstOneTheBranchHasNoCommitFor;

/// When [`status`](crate::status) reads a branch whose own commits carry no
/// `phase N:` subject, the [`Status`](crate::Status) it answers with shall name
/// the commit the branch left its base at and no phase, rather than a phase the
/// base carries.
#[derive(Spec)]
pub struct ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint;

/// When [`status`](crate::status) answers for a branch, the
/// [`Status`](crate::Status) it answers with shall carry the trailers of every
/// `phase N:` commit the branch made, and not the newest commit's alone.
#[derive(Spec)]
pub struct EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers;

/// When a commit newer than the newest [`PhaseCommit`](crate::PhaseCommit) the
/// branch made changed the slice's `lld.md` or the workspace's `hld.md`, the
/// [`Restart`](crate::Restart) the [`Status`](crate::Status) carries shall be
/// the one that restarts the pipeline at Phase 2, naming the document that
/// changed.
#[derive(Spec)]
pub struct ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo;

/// When [`status`](crate::status) reads a branch whose working tree holds
/// uncommitted work, it shall report that work in the
/// [`Status`](crate::Status) it answers with, rather than refusing to read.
#[derive(Spec)]
pub struct ADirtyWorkingTreeIsReportedAndNotRefused;

// ---- The findings a reading raises -------------------------------------------

/// When a [`PhaseCommit`](crate::PhaseCommit) the branch made carries a subject
/// that opens `phase ` and names no number a phase can be read from,
/// [`status`](crate::status) shall report a
/// [`Finding`](cargo_lid_rs::catalog::Finding) against that commit, rather than
/// passing over it or ending the reading.
#[derive(Spec)]
pub struct AMalformedPhaseSubjectIsAFindingAgainstItsCommit;

/// When a [`PhaseCommit`](crate::PhaseCommit) the branch made carries a
/// [`Trailer`](crate::Trailer) named `Lid-Rs-Tool` whose value is not the
/// running binary's version, [`status`](crate::status) shall report a
/// [`Finding`](cargo_lid_rs::catalog::Finding) naming that commit, so that a
/// resume across a tool change is visible.
#[derive(Spec)]
pub struct AToolTrailerTheBinaryDoesNotMatchIsAFinding;

/// When the branch [`status`](crate::status) reads names a slice no workspace
/// member holds a document for, it shall report a
/// [`Finding`](cargo_lid_rs::catalog::Finding) naming that slice beside the
/// [`Status`](crate::Status) in the [`Report`](crate::Report) it answers with,
/// rather than refusing the reading.
#[derive(Spec)]
pub struct AnUnresolvableSliceIsAFindingBesideTheStateAndNotARefusal;

/// When the branch [`status`](crate::status) reads names a slice no workspace
/// member holds a document for, the [`Status`](crate::Status) it answers with
/// shall carry no [`Restart`](crate::Restart) verdict, that verdict needing the
/// `lld.md` the layout could not place.
#[derive(Spec)]
pub struct AnUnresolvableSliceLeavesTheRestartVerdictUnanswered;

// ---- The one occasion a reading refuses --------------------------------------

/// When [`status`](crate::status) is given a branch that is not `lld/<slice>`
/// and no slice beside it, it shall refuse naming that convention, reading no
/// state.
#[derive(Spec)]
pub struct ABranchThatNamesNoSliceIsRefusedNamingTheConvention;

// ---- The report, and the rendering built from it ------------------------------

/// When [`status`](crate::status) has read a branch, the
/// [`Report`](crate::Report) it answers with shall carry that branch's
/// [`Status`](crate::Status) beside the findings the reading raised, carrying
/// the empty finding list where it raised none.
#[derive(Spec)]
pub struct TheReportCarriesTheStateAndTheFindingsTheReadingRaised;

/// When [`write_report`](crate::write_report) is given a reading's report, the
/// [`Report`](crate::Report) it is handed shall be written to
/// `target/lid/status.json`, whether or not the reading raised a finding.
#[derive(Spec)]
pub struct TheReportIsWrittenToStatusJsonWhateverItFound;

/// When [`rendering`](crate::rendering) is given a [`Report`](crate::Report),
/// the text it answers with shall be built from that report and never from a
/// second reading of the repository.
#[derive(Spec)]
pub struct TheRenderingIsBuiltFromTheReportAndNeverFromASecondReading;

/// When [`rendering`](crate::rendering) is given a [`Report`](crate::Report)
/// that holds findings, the text it answers with shall name each of them.
#[derive(Spec)]
pub struct TheRenderingNamesEveryFindingTheReportHolds;
