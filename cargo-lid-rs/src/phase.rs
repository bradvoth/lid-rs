use std::path::Path;

use lid_rs::implements;

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
