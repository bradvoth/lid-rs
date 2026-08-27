//! Claims for the `phase` slice (`docs/intent/phase/lld.md`): a phase is
//! run by an agent that can only edit, and its commit is the check passing.

use lid_rs::Spec;

// ---- phase-check: which phases have a check -----------------------------------

/// When `phase-check` is given a phase with no commit of its own — 0, 6, or
/// any number above 7 — it shall fail naming the phases that have a check.
#[derive(Spec)]
pub struct PhasesWithoutACommitHaveNoCheck;

/// When phase 1 is checked, the tool shall run rustdoc with broken intra-doc
/// links denied, then the doctests, in that order.
#[derive(Spec)]
pub struct PhaseOneChecksTheDocs;

/// When phase 2 is checked, the tool shall run `cargo check --all-targets`,
/// then clippy with warnings denied, in that order.
#[derive(Spec)]
pub struct PhaseTwoChecksTheClaimsBuildAndLint;

/// When phase 3 or 4 is checked, the tool shall run `cargo check
/// --all-targets` and nothing else.
#[derive(Spec)]
pub struct PhasesThreeAndFourCheckTheSkeletonTypeChecks;

/// When phase 7 is checked, the tool shall run the README §4.5 gate in its
/// order — check, clippy, doc, doctests, lib tests, `cargo package` for each
/// package that publishes, `sync --check`, `mutants`.
#[derive(Spec)]
pub struct PhaseSevenRunsTheGateInOrder;

/// When a step of a phase's sequence fails, the check shall stop there and
/// fail naming that step, running no later step.
#[derive(Spec)]
pub struct ACheckStopsAtTheFirstFailingStep;

// ---- phase-check 5: the red run ----------------------------------------------

/// When the slice's claims are identified, they shall be the registered
/// specs whose source file is `src/spec/<slice>.rs` — the slice name in
/// snake_case — read from the registry dump, never from Rust source.
#[derive(Spec)]
pub struct ASlicesClaimsAreTheSpecsInItsSpecFile;

/// When `--slice` is absent, the slice shall be the current branch's name
/// with the `lld/` prefix removed; a detached `HEAD` or a branch not of that
/// form names no slice, which fails the phase 5 check naming the
/// convention; any other flag shall be rejected by name.
#[derive(Spec)]
pub struct TheSliceComesFromTheBranchName;

/// When the slice's spec file registers no claims, the phase 5 check shall
/// fail naming the slice, never pass vacuously.
#[derive(Spec)]
pub struct ASliceWithNoClaimsFailsTheRedCheck;

/// When a slice claim has no validation edge, the phase 5 check shall fail
/// naming the claim.
#[derive(Spec)]
pub struct EveryClaimNeedsAValidationBeforePhaseFivePasses;

/// When the slice's validations are run, each shall run alone as `cargo test
/// --lib -p <package> -- --exact <path>`, with the citing item's path made
/// libtest-relative, and its exit status shall be its outcome.
#[derive(Spec)]
pub struct EachValidationRunsAloneByExactName;

/// When a slice validation passes at phase 5, the check shall fail naming
/// the test; when every validation fails, the check shall pass.
#[derive(Spec)]
pub struct AGreenValidationFailsTheRedCheck;

// ---- hook pre-tool: the path policy ------------------------------------------

/// When the policy locates the slice's crate, it shall be the workspace
/// package whose manifest directory holds `docs/intent/<slice>/lld.md`,
/// found on the filesystem, never by parsing Rust.
#[derive(Spec)]
pub struct TheSlicesCrateIsTheOneHoldingItsLld;

/// When a Phase 2 agent edits or writes, the target shall be the slice's
/// spec file or `src/spec/mod.rs` in the slice's crate, and nothing else.
#[derive(Spec)]
pub struct PhaseTwoMayWriteOnlyTheSlicesSpecFiles;

/// When a Phase 3 or 4 agent edits or writes, the target shall be the
/// slice's module, a file under its directory, or `src/lib.rs` in the
/// slice's crate, and nothing else.
#[derive(Spec)]
pub struct PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot;

/// When a Phase 5 or 7 agent edits or writes, the target shall be the
/// slice's module or a file under its directory, and nothing else.
#[derive(Spec)]
pub struct PhasesFiveAndSevenMayWriteOnlyTheSliceModule;

/// When a target path contains a parent component or resolves outside the
/// slice's crate, the policy shall refuse it before any allowed set is
/// consulted.
#[derive(Spec)]
pub struct PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy;

/// When an edit or write is refused, the reason shall quote the
/// `discipline.md` row for that moment and name what the phase may do
/// instead: proceed within its allowed paths, or end with the decision.
#[derive(Spec)]
pub struct ARefusedEditQuotesTheDisciplineRow;

/// When a phase agent calls a tool that does not edit — Read, Grep, Glob,
/// LSP — the hook shall allow it whatever the path.
#[derive(Spec)]
pub struct ReadsAreNeverRefused;

/// When any tool call reaches the hook, the agent's tally shall count it by
/// kind — edit, observation, command — and count every refusal, under the
/// agent's id.
#[derive(Spec)]
pub struct EveryToolCallIsTallied;

// ---- hook post-edit: clippy after every edit ---------------------------------

/// When an Edit or Write completes, the hook shall run clippy on the
/// workspace with warnings denied and hand its output, or "clean", back as
/// additional context, refusing nothing.
#[derive(Spec)]
pub struct EveryEditIsFollowedByClippy;

// ---- hook stop: the check, then the commit -----------------------------------

/// When the agent's final message carries neither a `commit` block nor a
/// `stop` block, or carries both, the stop shall be refused with the
/// format.
#[derive(Spec)]
pub struct AFinalMessageCarriesExactlyOneEnding;

/// When the final message carries a `stop` block, the hook shall commit
/// nothing and allow the stop.
#[derive(Spec)]
pub struct AStopBlockEndsThePhaseWithoutACommit;

/// When a `commit` block's subject does not begin with the agent's own
/// `phase <n>:` tag, the stop shall be refused naming the expected tag.
#[derive(Spec)]
pub struct ACommitSubjectMustCarryThisPhasesTag;

/// When the final message carries a `commit` block with this phase's tag,
/// the hook shall run the phase's check, and a failing check shall refuse
/// the stop.
#[derive(Spec)]
pub struct ACommitBlockRunsThePhasesCheck;

/// When a check fails, the refusal's reason shall be, in order, the failing
/// step's output, the `gates.md` row for the check that fired, and what the
/// phase's policy permits.
#[derive(Spec)]
pub struct ARefusalCarriesTheOutputTheRuleAndThePermittedMoves;

/// When a clippy lint in the failing output is one the gate relies on, it
/// shall name its check — `cognitive_complexity` 7,
/// `fn_params_excessive_bools` 8, `too_many_lines` 9,
/// `wildcard_enum_match_arm` 6, `missing_docs` 3 — and a red-run failure
/// shall name the Phase 5 rule, a survivor check 12.
#[derive(Spec)]
pub struct AFailingOutputNamesItsCheck;

/// When the synced artifacts differ from the dependency's, before or after
/// the check, the stop shall be refused naming them, and nothing shall be
/// committed.
#[derive(Spec)]
pub struct SyncedArtifactsMustMatchAtTheStop;

/// When, after the check, any path outside the phase's allowed set has
/// changed, the stop shall be refused naming it, and nothing shall be
/// committed.
#[derive(Spec)]
pub struct ChangesOutsideThePolicyRefuseTheStop;

/// When the check and both integrity checks pass, the hook shall stage
/// exactly the phase's allowed paths and commit the block's message.
#[derive(Spec)]
pub struct OnlyThePoliciesPathsAreStaged;

/// When nothing under the phase's allowed paths has changed, the stop shall
/// be refused as having nothing to commit.
#[derive(Spec)]
pub struct NothingToCommitIsARefusal;

/// When a phase commit is made, its message shall end with the
/// `Lid-Rs-Phase`, `Lid-Rs-Tools`, `Lid-Rs-Checks`, and `Lid-Rs-Refusals`
/// trailers rendered from the agent's tally.
#[derive(Spec)]
pub struct TheTallyIsWrittenAsTrailers;

// ---- execution class -----------------------------------------------------------

/// When the slice's crate declares a `proc-macro` or `custom-build` target,
/// its execution class shall be compile-time, naming which; otherwise it
/// shall be ordinary.
#[derive(Spec)]
pub struct ACompileTimeSliceIsDisclosed;

// ---- sync: the mirrored artifacts ------------------------------------------------

/// When `sync` runs, it shall mirror each artifact the resolved `lid-rs`
/// ships — the `skill/`, `workflow/`, and `agent/` directories — to its
/// place in the project, and `--check` shall hold every one to the skill's
/// any-difference rule.
#[derive(Spec)]
pub struct SyncMirrorsEveryArtifactTheDependencyShips;
