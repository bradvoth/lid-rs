
use std::path::{Path, PathBuf};

use lid_rs::implements;

pub mod door;
pub mod ending;
pub mod review;
pub mod tools;
pub mod turn;

use door::Door;
use ending::{WorkerEnd, worker};
use review::{Review, review};

use crate::phase::Phase;
use crate::phase::policy::{ExecutionClass, execution_class, slice_crate};
use crate::project::Project;
use crate::spec;

/// Canopy's production door: `--door`'s default.
pub const PRODUCTION_DOOR: &str = "https://api.canopyhq.dev";

/// `--max-cost`'s default, in the provider's currency.
pub const DEFAULT_MAX_COST: f64 = 5.0;

/// The environment variable the API key is read from.
pub const KEY_VARIABLE: &str = "CANOPY_KEY";

/// The phases with a worker session, in order; Phase 7's session also does
/// Phase 6, which has no commit of its own.
pub const PHASES: [Phase; 5] = [Phase::Two, Phase::Three, Phase::Four, Phase::Five, Phase::Seven];

/// The flags `canopy` takes, with their defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    /// `--slice <name>`; the branch's slice when absent.
    pub slice: Option<String>,
    /// `--door <url>`; the production door when absent.
    pub door: String,
    /// `--max-cost <amount>`, in the provider's currency; 5 when absent.
    pub max_cost: f64,
}

impl Default for Flags {
    /// No slice, [`PRODUCTION_DOOR`], and [`DEFAULT_MAX_COST`]: what the
    /// flags say when none is given.
    #[implements(spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags, spec::MaxCostIsTheFlagsAmountOrFive)]
    fn default() -> Self {
        todo!()
    }
}

/// `canopy [--slice <name>] [--door <url>] [--max-cost <amount>]`: the
/// flags, the key from `CANOPY_KEY`, the precondition, then the phases, and
/// last the terminal state. A stop at the key or the precondition is
/// rendered by [`stopped`]; [`terminal`] renders the build's outcome, `Ok`
/// for *PR-ready* — printed here — and `Err` for *stopped*, which the
/// binary's dispatcher prints and exits 1 on.
pub fn run(args: &[String]) -> Result<(), String> {
    let flags = parse_flags(args)?;
    let key = api_key(std::env::var(KEY_VARIABLE).ok()).map_err(|stop| stopped(&stop))?;
    let project = Project::load_graph()?;
    let state = precondition(&project, flags.slice.as_deref()).map_err(|stop| stopped(&stop))?;
    let outcome = build(&project, &Door::new(&flags.door, &key), &state, flags.max_cost);
    println!("{}", terminal(&state.branch, outcome)?);
    Ok(())
}

/// The flags: every `--flag value` pair applied in turn to the defaults.
fn parse_flags(args: &[String]) -> Result<Flags, String> {
    args.chunks(2).try_fold(Flags::default(), flag_applied)
}

/// One `--flag value` pair ([`flag_pair`]) applied — the one decision over
/// the flag's name: `--slice` names the slice, `--door` the door,
/// `--max-cost` the amount ([`amount`]); any other flag is rejected by
/// name, and the key is never a flag.
#[implements(spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags)]
fn flag_applied(flags: Flags, pair: &[String]) -> Result<Flags, String> {
    let (flag, value) = flag_pair(pair)?;
    match flag {
        "--slice" => Ok(Flags { slice: Some(value.to_string()), ..flags }),
        "--door" => Ok(Flags { door: value.to_string(), ..flags }),
        "--max-cost" => Ok(Flags { max_cost: amount(value)?, ..flags }),
        other => Err(format!("unknown flag `{other}` for canopy; the flags are --slice <name>, --door <url>, --max-cost <amount>")),
    }
}

/// One chunk of the arguments as its flag and its value; a flag without its
/// value is rejected by name.
#[implements(spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags)]
fn flag_pair(pair: &[String]) -> Result<(&str, &str), String> {
    todo!()
}

/// `--max-cost`'s value as an amount in the provider's currency; one that
/// is not a number is rejected, quoting it.
#[implements(spec::MaxCostIsTheFlagsAmountOrFive)]
fn amount(value: &str) -> Result<f64, String> {
    todo!()
}

/// The API key as `CANOPY_KEY` holds it; unset, the precondition stop
/// naming the variable — before any session opens.
#[implements(spec::TheKeyComesFromCanopyKeyOrTheRunStopsFirst)]
pub fn api_key(found: Option<String>) -> Result<String, Stop> {
    todo!()
}

/// What the precondition established, from git and the phase library: the
/// slice and its branch, the slice's crate, and the phases already
/// committed on the branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Precondition {
    /// The slice, from `--slice` or from the branch.
    pub slice: String,
    /// The checked-out branch, `lld/<slice>`.
    pub branch: String,
    /// The workspace package holding the slice's LLD.
    pub crate_root: PathBuf,
    /// The phases with a `phase N:` commit in the branch's history, by
    /// `tag_of`.
    pub committed: Vec<Phase>,
}

/// Where a run stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At {
    /// Before any session opened.
    Precondition,
    /// In a phase's worker session.
    Phase(Phase),
    /// In the review of a phase's commit.
    Review(Phase),
}

/// A run that stopped: where, and the decisions that stopped it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stop {
    /// Where.
    pub at: At,
    /// The numbered decisions, in the phase LLD's words where it has them.
    pub decisions: Vec<String>,
}

/// The precondition, read with git and the phase library and no model,
/// before any session opens, each check in turn and the first failure the
/// run's first decision: the checked-out branch is `lld/<slice>` for the
/// slice given or the branch's own ([`slice_of`]); the history holds a
/// `phase 1:` commit ([`phase_one_present`]); the tree is clean
/// ([`clean_tree`]); the slice has a crate; a compile-time slice has the
/// human's acceptance file ([`acceptance`]). The committed phases are read
/// from the same log's subjects ([`committed_phases`]).
pub fn precondition(project: &Project, slice: Option<&str>) -> Result<Precondition, Stop> {
    let branch = current_branch(project).map_err(at_precondition)?;
    let slice = slice_of(&branch, slice)?;
    let subjects = log_subjects(project).map_err(at_precondition)?;
    phase_one_present(&subjects)?;
    clean_tree(&dirty_paths(project).map_err(at_precondition)?)?;
    let crate_root = slice_crate(project, &slice).map_err(at_precondition)?;
    acceptance(project, &crate_root, &slice)?;
    Ok(Precondition { slice, branch, crate_root, committed: committed_phases(&subjects) })
}

/// A failure to read the repository at the precondition, as the stop it is.
fn at_precondition(reason: String) -> Stop {
    todo!()
}

/// The checked-out branch, from `git symbolic-ref --short HEAD`; a detached
/// `HEAD` is on no branch and is the error.
fn current_branch(project: &Project) -> Result<String, String> {
    todo!()
}

/// The slice the branch is for: the branch must be `lld/<slice>`, for the
/// slice given or, when none is, the one the branch names — as
/// `phase-check` reads it through [`crate::phase::slice_of_branch`]. Any
/// other branch stops the run naming it.
#[implements(spec::ThePreconditionNeedsTheSliceBranchCheckedOut)]
pub fn slice_of(branch: &str, given: Option<&str>) -> Result<String, Stop> {
    todo!()
}

/// The subjects of the branch's history, newest first, from
/// `git log --format=%s`.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags)]
fn log_subjects(project: &Project) -> Result<Vec<String>, String> {
    todo!()
}

/// The history must hold a `phase 1:` commit; without one the run stops in
/// the phase LLD's words: no `phase 1:` commit means the run stops before
/// doing anything — the LLD is the human's.
#[implements(spec::ThePreconditionNeedsAPhaseOneCommit)]
pub fn phase_one_present(subjects: &[String]) -> Result<(), Stop> {
    todo!()
}

/// The paths `git status --porcelain` reports: changed, staged, or
/// untracked.
fn dirty_paths(project: &Project) -> Result<Vec<String>, String> {
    todo!()
}

/// The tree must be clean; a dirty one stops the run naming the paths.
#[implements(spec::ThePreconditionNeedsACleanTree)]
pub fn clean_tree(dirty: &[String]) -> Result<(), Stop> {
    todo!()
}

/// The phases the subjects commit: those whose tag
/// [`crate::phase::tag_of`] recognises as a phase with a check, in the
/// subjects' order.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags)]
pub fn committed_phases(subjects: &[String]) -> Vec<Phase> {
    todo!()
}

/// The one decision over the slice's execution class
/// ([`crate::phase::policy::execution_class`]): a compile-time slice needs
/// the human's acceptance ([`accepted`]); an ordinary slice needs nothing.
#[implements(spec::ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance)]
pub fn acceptance(project: &Project, crate_root: &Path, slice: &str) -> Result<(), Stop> {
    match execution_class(project, crate_root).map_err(at_precondition)? {
        ExecutionClass::Ordinary => Ok(()),
        ExecutionClass::CompileTime(_) => accepted(crate_root, slice),
    }
}

/// A compile-time slice's acceptance: the human's file
/// ([`crate::phase::policy::compile_time_accepted`]) is present, or the run
/// stops naming it — `docs/intent/<slice>/compile-time-accepted` in the
/// slice's crate.
#[implements(spec::ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance)]
pub fn accepted(crate_root: &Path, slice: &str) -> Result<(), Stop> {
    todo!()
}

/// The two terminal states; there is no third and no waiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Every phase committed and the gate passed.
    PrReady {
        /// Every decision the phases recorded.
        decisions: Vec<String>,
    },
    /// Stopped at the precondition, a phase, or its review: where, and the
    /// decisions that stopped it, passed through as the phase produced them.
    Stopped(Stop),
}

/// What every phase's sessions share.
struct Run<'a> {
    /// The workspace.
    project: &'a Project,
    /// The door the sessions are dialled on.
    door: &'a Door,
    /// What the precondition established.
    state: &'a Precondition,
    /// Each session's cost ceiling.
    max_cost: f64,
}

/// Which attempt of a phase is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Attempt {
    /// The phase's first worker session.
    First,
    /// The one rework session a rejection opens.
    Rework,
}

/// Phases 2, 3, 4, 5, and 7 in order, each through [`phase_outcome`]: a
/// phase that ends well adds its decisions to the run's; the first that
/// stops ends the run there. The phase's sessions are printed as
/// [`Session::open`](turn::Session::open) opens them, and its ending as
/// [`attempt`] and [`reviewed`] learn it.
#[implements(spec::ThePhasesAreOneSessionEachInOrder)]
pub fn build(project: &Project, door: &Door, state: &Precondition, max_cost: f64) -> Outcome {
    let run = Run { project, door, state, max_cost };
    let mut decisions = Vec::new();
    for phase in PHASES {
        match phase_outcome(&run, phase) {
            Ok(recorded) => decisions.extend(recorded),
            Err(stop) => return Outcome::Stopped(stop),
        }
    }
    Outcome::PrReady { decisions }
}

/// One phase: skipped, and said so, when the branch already has its commit;
/// otherwise its first attempt.
#[implements(spec::CommittedPhasesAreSkippedAndSaidSo)]
fn phase_outcome(run: &Run, phase: Phase) -> Result<Vec<String>, Stop> {
    if run.state.committed.contains(&phase) { Ok(skipped(phase)) } else { attempt(run, phase, Attempt::First, None) }
}

/// Says the phase is already committed and skipped; a skipped phase records
/// no decisions.
#[implements(spec::CommittedPhasesAreSkippedAndSaidSo)]
pub fn skipped(phase: Phase) -> Vec<String> {
    todo!()
}

/// One worker session of a phase: the worker driven to its end, that
/// ending printed ([`ending_line`]), then what it means for the run
/// ([`phase_ended`]).
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
fn attempt(run: &Run, phase: Phase, which: Attempt, findings: Option<&[String]>) -> Result<Vec<String>, Stop> {
    let end = worker(run.project, run.door, phase, run.state, findings, run.max_cost);
    println!("{}", ending_line(phase, &end));
    phase_ended(run, phase, which, end)
}

/// The one decision over how a worker session ended: a commit goes to
/// review ([`reviewed`]), on the first attempt and on the rework alike; a
/// `stop` block's decisions, the ninth refusal's reason, or a reason
/// outside the model's doing stops the run at the phase.
#[implements(
    spec::EveryCommittedPhaseIsReviewedBeforeTheNextOpens,
    spec::AReworksCommitIsReviewedByAFreshSession,
    spec::AWorkersStopBlockEndsTheRunWithItsDecisions,
    spec::TheNinthConsecutiveRefusalEndsTheRun,
)]
fn phase_ended(run: &Run, phase: Phase, which: Attempt, end: Result<WorkerEnd, String>) -> Result<Vec<String>, Stop> {
    match end {
        Err(reason) | Ok(WorkerEnd::Refused(reason)) => Err(Stop { at: At::Phase(phase), decisions: vec![reason] }),
        Ok(WorkerEnd::Decisions(decisions)) => Err(Stop { at: At::Phase(phase), decisions }),
        Ok(WorkerEnd::Committed(hash, decisions)) => reviewed(run, phase, which, &hash, decisions),
    }
}

/// A committed phase's review: the reviewer driven to its verdict, that
/// verdict printed ([`verdict_line`]), then what it means for the run
/// ([`phase_judged`]) — `decisions` being the commit's, yielded on
/// approval.
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
fn reviewed(run: &Run, phase: Phase, which: Attempt, commit: &str, decisions: Vec<String>) -> Result<Vec<String>, Stop> {
    let verdict = review(run.project, run.door, phase, run.state, commit, run.max_cost);
    println!("{}", verdict_line(phase, &verdict));
    phase_judged(run, phase, which, verdict, decisions)
}

/// The one decision over a review's verdict and which attempt it judged:
/// approval yields the phase's decisions; a first rejection opens the one
/// rework session with the findings ([`attempt`]), whose commit is a second
/// `phase <n>:` commit on the branch reviewed the same way; a second
/// rejection stops the run with the findings, and a reason outside the
/// model's doing stops it too.
#[implements(
    spec::EveryCommittedPhaseIsReviewedBeforeTheNextOpens,
    spec::AFirstRejectionOpensOneReworkSession,
    spec::AReworksCommitIsASecondPhaseCommitOnTheBranch,
    spec::ASecondRejectionEndsTheRunWithTheFindings,
)]
fn phase_judged(run: &Run, phase: Phase, which: Attempt, verdict: Result<Review, String>, decisions: Vec<String>) -> Result<Vec<String>, Stop> {
    match (verdict, which) {
        (Err(reason), Attempt::First | Attempt::Rework) => Err(Stop { at: At::Review(phase), decisions: vec![reason] }),
        (Ok(Review::Approved), Attempt::First | Attempt::Rework) => Ok(decisions),
        (Ok(Review::Rejected(findings)), Attempt::First) => attempt(run, phase, Attempt::Rework, Some(findings.as_slice())),
        (Ok(Review::Rejected(findings)), Attempt::Rework) => Err(Stop { at: At::Review(phase), decisions: findings }),
    }
}

/// A phase's ending as the run prints it: the commit it made, the decisions
/// that stopped it, the refusal that ended it, or the reason outside the
/// model's doing.
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
pub fn ending_line(phase: Phase, end: &Result<WorkerEnd, String>) -> String {
    todo!()
}

/// A review's verdict as the run prints it: approved, rejected with its
/// findings, or the reason outside the model's doing.
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
pub fn verdict_line(phase: Phase, verdict: &Result<Review, String>) -> String {
    todo!()
}

/// The terminal state as the run's last line: *PR-ready* is `Ok` with the
/// branch and every decision, *stopped* is `Err` with where and the
/// decisions — so the exit status follows it.
#[implements(spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt)]
pub fn terminal(branch: &str, outcome: Outcome) -> Result<String, String> {
    match outcome {
        Outcome::PrReady { decisions } => Ok(pr_ready(branch, &decisions)),
        Outcome::Stopped(stop) => Err(stopped(&stop)),
    }
}

/// *PR-ready*, rendered: every decision the phases recorded, then the
/// branch, on the last line with the state.
#[implements(spec::PrReadyEndsWithTheBranchAndEveryRecordedDecision, spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt)]
pub fn pr_ready(branch: &str, decisions: &[String]) -> String {
    todo!()
}

/// *Stopped*, rendered: the numbered decisions, then where it stopped, on
/// the last line with the state.
#[implements(spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt)]
pub fn stopped(stop: &Stop) -> String {
    todo!()
}
