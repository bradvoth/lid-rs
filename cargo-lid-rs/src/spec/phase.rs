//! Claims for the `phase` slice (`docs/intent/phase/lld.md`): a phase commit
//! gates itself, and the hooks that let an agent build a slice unattended.

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
/// with the `lld/` prefix removed; a branch not of that form shall fail the
/// phase 5 check naming the convention; any other flag shall be rejected by
/// name.
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

// ---- hook commit-msg ---------------------------------------------------------

/// When the commit message's subject line begins `phase N:` for a phase with
/// a check, the hook shall run that check and, when it fails, refuse the
/// commit with the check's output.
#[derive(Spec)]
pub struct TaggedCommitsRunTheirPhaseCheck;

/// When the subject line carries no `phase N:` tag, the hook shall allow the
/// commit without running any check.
#[derive(Spec)]
pub struct UntaggedCommitsPassTheHook;

/// When the subject line's tag names a phase with no check, the hook shall
/// refuse the commit naming the phases that have one, never treat it as
/// untagged.
#[derive(Spec)]
pub struct MistypedTagsAreRefusedNotIgnored;

// ---- hook subagent-start / subagent-stop --------------------------------------

/// When `subagent-start` runs, it shall read the agent's id from the hook
/// input and record the repository's current `HEAD` under
/// `<target>/lid-rs/agents/<agent_id>`.
#[derive(Spec)]
pub struct AStartingWorkerRecordsHead;

/// When `subagent-stop` finds `HEAD` differs from the agent's recorded one,
/// it shall allow the stop.
#[derive(Spec)]
pub struct AWorkerThatCommittedMayStop;

/// When `subagent-stop` finds `HEAD` unchanged from the agent's record and
/// `stop_hook_active` is false, it shall refuse the stop — a block decision
/// whose reason instructs the worker to commit the phase or state the
/// decisions that block it.
#[derive(Spec)]
pub struct AWorkerThatDidNotCommitIsRefusedOnce;

/// When `subagent-stop` runs with `stop_hook_active` true, it shall allow the
/// stop regardless of `HEAD`.
#[derive(Spec)]
pub struct ASecondStopAttemptIsAllowed;

/// When `subagent-stop` finds no record for the agent's id, it shall allow
/// the stop.
#[derive(Spec)]
pub struct AStopWithoutARecordIsAllowed;

// ---- init and sync: installing the hooks and the workflow ---------------------

/// When `sync` runs, it shall mirror each artifact the resolved `lid-rs`
/// ships — `skill/`, `workflow/lid-rs.js`, `agent/lid-rs-phase.md`,
/// `hooks/commit-msg` — to its place in the project, and `--check` shall
/// hold every one to the skill's any-difference rule.
#[derive(Spec)]
pub struct SyncMirrorsEveryArtifactTheDependencyShips;

/// When `sync` runs, it shall set the repository's `core.hooksPath` to
/// `.lid-rs/hooks`; `sync --check` shall fail when it is not set to that.
#[derive(Spec)]
pub struct SyncAssertsTheHooksPath;

/// When `init` runs in a repository whose `core.hooksPath` is already set to
/// another value, it shall report that as a conflict.
#[derive(Spec)]
pub struct AForeignHooksPathIsAnInitConflict;
