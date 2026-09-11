//! Claims for the `phase` slice (`docs/intent/phase/lld.md`): a phase is
//! run by an agent that can only edit, and its commit is the check passing.

use lid_rs::Spec;

// ---- phase-check: which phases have a check -----------------------------------

/// When `phase-check` is given a phase with no commit of its own — 0, 6, or
/// any number above 7 — it shall fail naming the phases that have a check.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesWithoutACommitHaveNoCheck;

/// When phase 1 is checked, the tool shall run the LLD's mechanical checks,
/// then rustdoc with broken intra-doc links denied, then the doctests, in
/// that order.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseOneChecksTheDocs;

/// When phase 2 is checked, the tool shall run `cargo check --all-targets`
/// and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoChecksTheClaimsBuild;

/// When the workspace builds with warnings and no error — a skeleton's
/// `todo!()`, a citation of a claim name this phase retired — phase 2's
/// check shall pass, so a claim the design turns out to need is committable
/// while the rest of the slice stands.
#[derive(Spec)]
#[lid(free)]
pub struct WarningsDoNotFailPhaseTwosCheck;

/// When phase 3 or 4 is checked, the tool shall run `cargo check
/// --all-targets` and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourCheckTheSkeletonTypeChecks;

/// When phase 7 is checked, the tool shall run the README §4.5 gate in its
/// order — check, clippy, doc, doctests, lib tests, one `cargo package`
/// naming every package that publishes, `sync --check`, `mutants`.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce;

/// The name [`PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce`]
/// carried while the gate's package step was one invocation per package. The
/// alias registers no claim, so the graph sees only the claim it points at;
/// every citation of this name warns with its replacement, and those citations
/// are the later phases' work list.
#[deprecated = "replaced by PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce"]
pub type PhaseSevenRunsTheGateInOrder =
    PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce;

/// When a step of a phase's sequence fails, the check shall stop there and
/// fail naming that step, running no later step.
#[derive(Spec)]
#[lid(free)]
pub struct ACheckStopsAtTheFirstFailingStep;

// ---- phase-check 5: the red run ----------------------------------------------

/// When the slice's claims are identified, they shall be the registered
/// specs whose source file is the slice's claims file, wherever the layout
/// puts that file, read from the registry dump, never from Rust source.
#[derive(Spec)]
#[lid(free)]
pub struct ASlicesClaimsAreTheSpecsInItsSpecFile;

/// When `--slice` is absent, the slice shall be the current branch's name
/// with the `lld/` prefix removed; a detached `HEAD` or a branch not of that
/// form names no slice, which fails the phase 5 check naming the
/// convention; any other flag shall be rejected by name.
#[derive(Spec)]
#[lid(free)]
pub struct TheSliceComesFromTheBranchName;

/// When the branch's name after `lld/` contains `--`, the slice shall be the
/// part before the first `--` — `lld/phase--companion` is a change to the
/// slice `phase`, made on its own branch because the branch that built the
/// slice is kept and git admits no `lld/phase/companion` beside it.
#[derive(Spec)]
#[lid(free)]
pub struct AChangeBranchNamesItsSliceBeforeTheDoubleDash;

/// When the slice's spec file registers no claims, the phase 5 check shall
/// fail naming the slice, never pass vacuously.
#[derive(Spec)]
#[lid(free)]
pub struct ASliceWithNoClaimsFailsTheRedCheck;

/// When the red run locates its base, the base shall be the newest commit
/// reachable from `HEAD` whose subject starts `phase 7:`, whichever slice
/// that gate was for.
#[derive(Spec)]
#[lid(free)]
pub struct TheBaseIsTheNewestGateCommitReachableFromHead;

/// When a base exists, the red set shall be those of the slice's registered
/// claims whose `struct <Name>` line is an added line of `git diff <base>`
/// taken over the slice's claims file, wherever the layout puts that file.
#[derive(Spec)]
#[lid(free)]
pub struct TheRedSetIsTheClaimsAddedSinceTheBase;

/// When the slice's crate has a companion, the package that holds the
/// slice's claims — where the red run diffs the spec file and runs each
/// validation — shall be the companion, since a proc-macro crate registers
/// no claim and can cite none.
#[derive(Spec)]
#[lid(free)]
pub struct AProcMacroSlicesClaimsAreHeldByItsCompanion;

/// When no gate commit is reachable from `HEAD`, the red set shall be every
/// claim of the slice.
#[derive(Spec)]
#[lid(free)]
pub struct AFreshSliceHasEveryClaimInTheRedSet;

/// When a base exists and the red set is empty, the phase 5 check shall fail
/// naming the base, never pass vacuously.
#[derive(Spec)]
#[lid(free)]
pub struct AnEmptyRedSetAfterAGateFailsTheRedCheck;

/// When a claim in the red set has no validation edge, the phase 5 check
/// shall fail naming the claim.
#[derive(Spec)]
#[lid(free)]
pub struct EveryClaimNeedsAValidationBeforePhaseFivePasses;

/// When the red set's validations are run, each shall run alone as `cargo
/// test --lib -p <package> -- --exact <path>`, with the citing item's path
/// made libtest-relative, and its exit status shall be its outcome.
#[derive(Spec)]
#[lid(free)]
pub struct EachValidationRunsAloneByExactName;

/// When a validation of a red-set claim passes at phase 5, the check shall
/// fail naming the test; when every one fails, the check shall pass, whatever
/// the slice's other validations do.
#[derive(Spec)]
#[lid(free)]
pub struct AGreenValidationFailsTheRedCheck;

// ---- hook pre-tool: the path policy ------------------------------------------

/// When the policy locates the slice's crate, it shall be the workspace
/// package whose manifest directory holds the slice's document, wherever the
/// layout puts that document, found on the filesystem and never by parsing
/// Rust.
#[derive(Spec)]
#[lid(free)]
pub struct TheSlicesCrateIsTheOneHoldingItsLld;

/// When a Phase 2 agent edits or writes under the slice's own crate, the
/// target shall be the slice's claims file — the layout's answer for the
/// slice, placed in that crate — or `src/spec/mod.rs`, and nothing else in
/// that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles;

// The four claims below — these two and their companion-seat twins — say what
// a phase may write under the slice's directory rather than naming the
// directory. Colocation puts the slice's document, its claims file, and every
// other file of its intent (a compile-time slice's acceptance) in that same
// directory, and a directory admits everything under it: naming it would hand
// Phases 3, 4, 5 and 7 the design document, the claims, and the human's
// acceptance. A phase's own artifact is Rust source, and of the Rust source
// beside the code only the claims file is another phase's, so that is the rule
// the directory entry was standing in for — and it refuses an intent file
// invented later without the policy learning its name.

/// When a Phase 3 or 4 agent edits or writes under the slice's own crate, the
/// target shall be the slice's module file, `src/lib.rs`, or a Rust source
/// file under the slice's directory other than its claims file — wherever the
/// layout puts that file — and nothing else in that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent;

/// When a Phase 5 or 7 agent edits or writes under the slice's own crate, the
/// target shall be the slice's module file or a Rust source file under the
/// slice's directory other than its claims file — wherever the layout puts
/// that file — and nothing else in that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent;

/// When a target path contains a parent component or resolves outside both
/// the slice's crate and its companion, the policy shall refuse it before
/// any allowed set is consulted.
#[derive(Spec)]
#[lid(free)]
pub struct PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy;

// ---- hook pre-tool: a proc-macro crate's companion ---------------------------

/// When the slice's crate declares no `proc-macro` target, its companion
/// shall be none, whatever its package metadata carries.
#[derive(Spec)]
#[lid(free)]
pub struct AnOrdinaryCrateHasNoCompanion;

/// When the slice's crate is a proc-macro crate whose package metadata, as
/// `cargo metadata` reports it, names a workspace member under
/// `lid_rs.companion`, the companion shall be that member's manifest
/// directory — read from the metadata, never by parsing the manifest.
#[derive(Spec)]
#[lid(free)]
pub struct TheCompanionIsTheMemberTheProcMacroCratesMetadataNames;

/// When the slice's crate is a proc-macro crate whose package metadata names
/// no `lid_rs.companion`, the policy shall refuse every edit naming
/// `[package.metadata.lid_rs] companion`, since no phase of such a slice can
/// produce a claim.
#[derive(Spec)]
#[lid(free)]
pub struct AProcMacroCrateNamingNoCompanionRefusesEveryEdit;

/// When the named companion is itself a proc-macro crate, the policy shall
/// refuse every edit naming the key, as it does for a missing companion.
#[derive(Spec)]
#[lid(free)]
pub struct ACompanionThatIsAProcMacroCrateRefusesEveryEdit;

/// When the named companion is not a workspace member, the policy shall
/// refuse every edit naming the key, as it does for a missing companion.
#[derive(Spec)]
#[lid(free)]
pub struct ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit;

/// When the slice's crate has a companion and the target path resolves under
/// the companion, the policy shall judge it by the phase's companion table,
/// relative to the companion — the allowed set is the union of the two
/// tables, each relative to its crate.
#[derive(Spec)]
#[lid(free)]
pub struct APathUnderTheCompanionIsJudgedByTheCompanionsTable;

/// When a Phase 2 agent edits or writes under the companion, the target
/// shall be the slice's claims file — the same answer the layout gives for
/// the slice, placed in the companion rather than in the slice's own crate
/// — or `src/spec/mod.rs` there, and nothing else in the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoMayWriteOnlyTheCompanionsSpecFiles;

/// When a Phase 3 or 4 agent edits or writes under the companion, the target
/// shall be the slice's module file there, `src/lib.rs` — where the
/// hand-authored edges citing the slice's claims go — or a Rust source file
/// under the slice's directory other than its claims file, which the
/// companion is the crate that holds, and nothing else in the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims;

/// When a Phase 5 or 7 agent edits or writes under the companion, the target
/// shall be the slice's module file there, a Rust source file under the
/// slice's directory other than its claims file, or a file under `tests/ui/`
/// — the one place a compile-failure fixture can live — and nothing else in
/// the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims;

/// When an edit or write is refused, the reason shall quote the
/// `discipline.md` row for that moment and name what the phase may do
/// instead: proceed within its allowed paths, or end with the decision.
#[derive(Spec)]
#[lid(free)]
pub struct ARefusedEditQuotesTheDisciplineRow;

/// When a phase agent calls a tool that does not edit — Read, Grep, Glob,
/// LSP, or the workflow's `StructuredOutput` — the hook shall allow it
/// whatever the path.
#[derive(Spec)]
#[lid(free)]
pub struct ReadsAreNeverRefused;

/// When any tool call reaches the hook, the agent's tally shall count it by
/// kind — edit, observation, command — and count every refusal, under the
/// agent's id.
#[derive(Spec)]
#[lid(free)]
pub struct EveryToolCallIsTallied;

// ---- hook post-edit: clippy after every edit ---------------------------------

/// When an Edit or Write completes, the hook shall run clippy on the
/// workspace with warnings denied and hand its output, or "clean", back as
/// additional context, refusing nothing.
#[derive(Spec)]
#[lid(free)]
pub struct EveryEditIsFollowedByClippy;

// ---- hook stop: the check, then the commit -----------------------------------

/// When the agent's final message carries neither a `commit` block nor a
/// `stop` block, or carries both, the stop shall be refused with the
/// format.
#[derive(Spec)]
#[lid(free)]
pub struct AFinalMessageCarriesExactlyOneEnding;

/// When the final message carries a `stop` block, the hook shall commit
/// nothing and allow the stop.
#[derive(Spec)]
#[lid(free)]
pub struct AStopBlockEndsThePhaseWithoutACommit;

/// When a `commit` block's subject does not begin with the agent's own
/// `phase <n>:` tag, the stop shall be refused naming the expected tag.
#[derive(Spec)]
#[lid(free)]
pub struct ACommitSubjectMustCarryThisPhasesTag;

/// When the final message carries a `commit` block with this phase's tag,
/// the hook shall run the phase's check, and a failing check shall refuse
/// the stop.
#[derive(Spec)]
#[lid(free)]
pub struct ACommitBlockRunsThePhasesCheck;

/// When a check fails, the refusal's reason shall be, in order, the failing
/// step's output, the `gates.md` row for the check that fired, and what the
/// phase's policy permits.
#[derive(Spec)]
#[lid(free)]
pub struct ARefusalCarriesTheOutputTheRuleAndThePermittedMoves;

/// When a clippy lint in the failing output is one the gate relies on, it
/// shall name its check — `cognitive_complexity` 7,
/// `fn_params_excessive_bools` 8, `too_many_lines` 9,
/// `wildcard_enum_match_arm` 6, `missing_docs` 3 — and a red-run failure
/// shall name the Phase 5 rule, a survivor check 12.
#[derive(Spec)]
#[lid(free)]
pub struct AFailingOutputNamesItsCheck;

/// When the synced artifacts differ from the dependency's, before or after
/// the check, the stop shall be refused naming them, and nothing shall be
/// committed.
#[derive(Spec)]
#[lid(free)]
pub struct SyncedArtifactsMustMatchAtTheStop;

/// When, after the check, any path outside the phase's allowed set has
/// changed, the stop shall be refused naming it, and nothing shall be
/// committed.
#[derive(Spec)]
#[lid(free)]
pub struct ChangesOutsideThePolicyRefuseTheStop;

/// When the slice's crate has a companion, the integrity check shall filter
/// `git status` against both crates' allowed sets, so a change under the
/// companion's table is the phase's own and any other is named.
#[derive(Spec)]
#[lid(free)]
pub struct IntegrityFiltersAgainstBothCratesAllowedPaths;

/// When the check and both integrity checks pass, the hook shall stage
/// exactly the phase's allowed paths and commit the block's message.
#[derive(Spec)]
#[lid(free)]
pub struct OnlyThePoliciesPathsAreStaged;

/// When the slice's crate has a companion and the check passes, the hook
/// shall stage the phase's allowed paths of both crates — each table
/// relative to its crate — and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct TheStopStagesBothCratesAllowedPaths;

/// When nothing under the phase's allowed paths has changed, the stop shall
/// be refused as having nothing to commit.
#[derive(Spec)]
#[lid(free)]
pub struct NothingToCommitIsARefusal;

/// When a phase commit is made, its message shall end with the
/// `Lid-Rs-Phase`, `Lid-Rs-Agent`, `Lid-Rs-Tools`, `Lid-Rs-Checks`, and
/// `Lid-Rs-Refusals` trailers rendered from the agent's tally, the agent
/// being the id the tally was kept under.
#[derive(Spec)]
#[lid(free)]
pub struct TheTallyIsWrittenAsTrailers;

// ---- execution class -----------------------------------------------------------

/// When the slice's crate declares a `proc-macro` or `custom-build` target,
/// its execution class shall be compile-time, naming which; otherwise it
/// shall be ordinary.
#[derive(Spec)]
#[lid(free)]
pub struct ACompileTimeSliceIsDisclosed;

/// When the slice is compile-time and the `compile-time-accepted` file of
/// its intent — wherever the layout puts that file in the slice's own crate
/// — does not exist, the policy shall refuse every edit, naming the file the
/// human commits to accept it.
#[derive(Spec)]
#[lid(free)]
pub struct ACompileTimeSliceNeedsTheHumansAcceptance;

// ---- sync: the mirrored artifacts ------------------------------------------------

/// When `sync` runs, it shall mirror each artifact the resolved `lid-rs`
/// ships — the `skill/`, `workflow/`, and `agent/` directories — to its
/// place in the project, and `--check` shall hold every one to the skill's
/// any-difference rule.
#[derive(Spec)]
#[lid(free)]
pub struct SyncMirrorsEveryArtifactTheDependencyShips;
