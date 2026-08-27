use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::mapping::EdgeRecord;
use crate::mutants::{Registry, SpecRecord};
use crate::project::Project;
use crate::spec;

/// A phase with a commit of its own and a check attached — the closed set.
/// Phases 0, 6, and 8 have no commit (skill, working state) and no check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::PhasesWithoutACommitHaveNoCheck)]
pub enum Phase {
    /// The LLD: docs and doctests.
    One,
    /// The claims: build and lint.
    Two,
    /// The layer-0 skeleton: type-checks.
    Three,
    /// Every further layer: type-checks.
    Four,
    /// The validations: red against `todo!()`.
    Five,
    /// The gate.
    Seven,
}

impl TryFrom<u8> for Phase {
    type Error = String;

    /// The phase for a number, or an error naming the phases that have a check.
    #[implements(spec::PhasesWithoutACommitHaveNoCheck)]
    fn try_from(n: u8) -> Result<Self, String> {
        todo!()
    }
}

/// One step of a phase's check — the closed set of things a check runs,
/// which is README §4.5's list plus the red run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::PhaseSevenRunsTheGateInOrder)]
pub enum Step {
    /// `cargo check --all-targets`.
    Check,
    /// `cargo clippy --all-targets -- -D warnings`.
    Clippy,
    /// `cargo doc --no-deps` with broken intra-doc links denied.
    Doc,
    /// `cargo test --doc`.
    DocTests,
    /// `cargo test --lib`.
    LibTests,
    /// `cargo package -p <name> --allow-dirty` for one publishing package.
    Package(String),
    /// `sync --check`, through the library.
    SyncCheck,
    /// `mutants`, through the library.
    Mutants,
    /// The phase 5 red run over the slice's validations.
    Red,
}

/// What a hook reads from the JSON Claude Code passes on stdin — the
/// boundary type; everything past it takes plain values.
#[derive(Debug, PartialEq, Eq)]
pub struct HookInput {
    /// The subagent's id, the key its `HEAD` record is filed under.
    pub agent_id: String,
    /// True when Claude Code is already continuing because a stop was refused.
    pub stop_hook_active: bool,
}

/// The stop hook's answer, rendered to stdout at the boundary.
#[derive(Debug, PartialEq, Eq)]
pub enum StopDecision {
    /// The worker may stop.
    Allow,
    /// The worker is kept running, with this reason as its next instruction.
    Refuse(String),
}

/// `phase-check <n> [--slice <name>]`: parse, locate the project, check.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn run(args: &[String]) -> Result<(), String> {
    todo!()
}

/// `hook <commit-msg <file> | subagent-start | subagent-stop>`: one dispatch
/// over the hook kind; the subagent hooks read their input from stdin.
pub fn hook(args: &[String]) -> Result<(), String> {
    todo!()
}

/// Runs a phase's check: its plan, executed in order.
pub fn check(project: &Project, phase: Phase, slice: Option<&str>) -> Result<(), String> {
    todo!()
}

/// A phase's steps, in order, as data. `publishing` names the packages
/// `cargo package` runs for at phase 7.
#[implements(
    spec::PhaseOneChecksTheDocs,
    spec::PhaseTwoChecksTheClaimsBuildAndLint,
    spec::PhasesThreeAndFourCheckTheSkeletonTypeChecks,
    spec::PhaseSevenRunsTheGateInOrder,
)]
pub fn plan(phase: Phase, publishing: &[String]) -> Vec<Step> {
    todo!()
}

/// Runs steps in order; the first failure is the result, naming its step,
/// and no later step runs.
#[implements(spec::ACheckStopsAtTheFirstFailingStep)]
pub fn execute(project: &Project, slice: Option<&str>, steps: &[Step]) -> Result<(), String> {
    todo!()
}

/// Runs one step: one dispatch over the closed set.
#[implements(spec::PhaseSevenRunsTheGateInOrder)]
fn run_step(project: &Project, slice: Option<&str>, step: &Step) -> Result<(), String> {
    todo!()
}

/// The phase 5 red run: the slice's claims, their validations, each run
/// alone; fails naming every unvalidated claim and every green test.
#[implements(spec::AGreenValidationFailsTheRedCheck)]
pub fn check_red(project: &Project, slice: &str) -> Result<(), String> {
    todo!()
}

/// `hook commit-msg <file>`: a tagged subject runs its phase's check and
/// refuses the commit on failure; an untagged one passes; a mistyped tag is
/// refused by name.
#[implements(
    spec::TaggedCommitsRunTheirPhaseCheck,
    spec::UntaggedCommitsPassTheHook,
    spec::MistypedTagsAreRefusedNotIgnored,
)]
fn hook_commit_msg(project: &Project, message_file: &Path) -> Result<(), String> {
    todo!()
}

/// `hook subagent-start`: records the repository's `HEAD` under the agent's
/// id.
#[implements(spec::AStartingWorkerRecordsHead)]
fn hook_subagent_start(project: &Project, input: &HookInput) -> Result<(), String> {
    todo!()
}

/// `hook subagent-stop`: the recorded `HEAD`, the current one, and whether
/// this is a retry decide whether the worker may stop.
fn hook_subagent_stop(project: &Project, input: &HookInput) -> Result<StopDecision, String> {
    todo!()
}

/// What a commit subject's tag says.
#[derive(Debug, PartialEq, Eq)]
pub enum Tag {
    /// No `phase N:` prefix: not a phase commit.
    Untagged,
    /// A phase with a check.
    Checked(Phase),
    /// A `phase N:` prefix naming a phase without a check.
    Unchecked(u8),
}

/// One validation's outcome at phase 5.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The claim the test cites.
    pub claim: String,
    /// The test's libtest-relative path.
    pub test: String,
    /// Whether the test passed — which, at phase 5, is the failure.
    pub passed: bool,
}

/// One package's registry, dumped from its own test binary.
struct PackageRegistry {
    /// The package name `cargo test -p` takes.
    package: String,
    /// Its specs and edges.
    registry: Registry,
}

/// `phase-check`'s arguments: the phase number, then optionally
/// `--slice <name>`; any other flag is rejected by name.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn parse_args(args: &[String]) -> Result<(Phase, Option<String>), String> {
    todo!()
}

/// The slice an `lld/<slice>` branch is for, or none for any other name.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn slice_of_branch(branch: &str) -> Option<String> {
    todo!()
}

/// The repository's current branch name.
fn current_branch(project: &Project) -> Result<String, String> {
    todo!()
}

/// The message's subject: its first line that is not a `#` comment.
fn subject_of(message: &str) -> &str {
    todo!()
}

/// What the subject's `phase N:` prefix, if any, names.
#[implements(spec::UntaggedCommitsPassTheHook, spec::MistypedTagsAreRefusedNotIgnored)]
pub fn tag_of(subject: &str) -> Tag {
    todo!()
}

/// Runs one cargo command with its output inherited, so the check's output
/// is what the caller prints; fails naming the step.
fn cargo_step(project: &Project, step: &Step, args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    todo!()
}

/// Every library member's registry, each from its own test binary.
fn package_registries(project: &Project) -> Result<Vec<PackageRegistry>, String> {
    todo!()
}

/// The file a slice's claims register from: `src/spec/<slice>.rs`, the
/// slice name in snake_case.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn spec_file_of(slice: &str) -> String {
    todo!()
}

/// The names of the specs registered from the slice's spec file.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn slice_claims(specs: &[SpecRecord], slice: &str) -> Vec<String> {
    todo!()
}

/// The claims, or the failure for a slice that registers none.
#[implements(spec::ASliceWithNoClaimsFailsTheRedCheck)]
fn require_claims(claims: Vec<String>, slice: &str) -> Result<Vec<String>, String> {
    todo!()
}

/// `(claim, test)` for every validation edge on the claims, the test's item
/// path made libtest-relative.
#[implements(spec::EachValidationRunsAloneByExactName)]
pub fn claim_validations(validations: &[EdgeRecord], claims: &[String]) -> Vec<(String, String)> {
    todo!()
}

/// Runs one test alone; true when it passes.
#[implements(spec::EachValidationRunsAloneByExactName)]
fn run_test(project: &Project, package: &str, test: &str) -> Result<bool, String> {
    todo!()
}

/// The claims no outcome validates.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses)]
pub fn unvalidated(claims: &[String], outcomes: &[Outcome]) -> Vec<String> {
    todo!()
}

/// The verdict: pass when nothing is unvalidated and nothing is green;
/// otherwise the failure naming every unvalidated claim and green test.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses, spec::AGreenValidationFailsTheRedCheck)]
pub fn red_verdict(unvalidated: &[String], outcomes: &[Outcome]) -> Result<(), String> {
    todo!()
}

/// The hook's stdin, whole.
fn read_stdin() -> Result<String, String> {
    todo!()
}

impl HookInput {
    /// The fields the hooks use, from Claude Code's hook JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        todo!()
    }
}

/// Where an agent's `HEAD` record lives: `<target>/lid-rs/agents/<agent_id>`.
#[implements(spec::AStartingWorkerRecordsHead)]
fn record_path(project: &Project, agent_id: &str) -> Result<PathBuf, String> {
    todo!()
}

/// The repository's current `HEAD`.
fn head(project: &Project) -> Result<String, String> {
    todo!()
}

/// Whether the worker may stop, from its recorded `HEAD`, the current one,
/// and whether Claude Code is already retrying after a refusal.
#[implements(
    spec::AWorkerThatCommittedMayStop,
    spec::AWorkerThatDidNotCommitIsRefusedOnce,
    spec::ASecondStopAttemptIsAllowed,
    spec::AStopWithoutARecordIsAllowed,
)]
pub fn stop_decision(recorded: Option<&str>, head: &str, retry: bool) -> StopDecision {
    todo!()
}

/// What the stop hook prints: nothing to allow, Claude Code's block
/// decision JSON to refuse.
#[implements(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
pub fn render(decision: &StopDecision) -> String {
    todo!()
}
