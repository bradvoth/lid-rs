
use std::path::{Path, PathBuf};

use lid_rs::implements;

pub mod door;
pub mod ending;
#[cfg(test)]
pub mod replay;
pub mod review;
pub mod tools;
pub mod turn;

use door::Door;
use ending::{WorkerEnd, number, worker};
use review::{Review, review};

use crate::phase::policy::{ExecutionClass, compile_time_accepted, execution_class, slice_crate};
use crate::phase::{Phase, Tag};
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
        Self { slice: None, door: PRODUCTION_DOOR.to_string(), max_cost: DEFAULT_MAX_COST }
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
    match pair {
        [flag, value] => Ok((flag.as_str(), value.as_str())),
        [flag] => Err(format!("the flag `{flag}` for canopy needs a value")),
        [] | [_, _, _, ..] => Err("canopy takes its flags as `--flag value` pairs".to_string()),
    }
}

/// `--max-cost`'s value as an amount in the provider's currency; one that
/// is not a number is rejected, quoting it.
#[implements(spec::MaxCostIsTheFlagsAmountOrFive)]
fn amount(value: &str) -> Result<f64, String> {
    value.parse::<f64>().map_err(|_| format!("`{value}` is not an amount for --max-cost, in the provider's currency"))
}

/// The API key as `CANOPY_KEY` holds it; unset, the precondition stop
/// naming the variable — before any session opens.
#[implements(spec::TheKeyComesFromCanopyKeyOrTheRunStopsFirst)]
pub fn api_key(found: Option<String>) -> Result<String, Stop> {
    found.ok_or_else(|| Stop {
        at: At::Precondition,
        decisions: vec![format!("{KEY_VARIABLE} is unset: set it to an API key bound to the canopy config this run is to use")],
    })
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
    /// The phases with a `phase N:` commit among the branch's own commits,
    /// by `tag_of`.
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
/// slice given or the branch's own ([`slice_of`]); the branch's own commits
/// — those since it diverged from the default branch ([`log_subjects`]) —
/// hold a `phase 1:` commit ([`phase_one_present`]); the tree is clean
/// ([`clean_tree`]); the slice has a crate; a compile-time slice has the
/// human's acceptance file ([`acceptance`]). The committed phases are read
/// from those same subjects ([`committed_phases`]).
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
    Stop { at: At::Precondition, decisions: vec![reason] }
}

/// The checked-out branch, from `git symbolic-ref --short HEAD`; a detached
/// `HEAD` is on no branch and is the error.
fn current_branch(project: &Project) -> Result<String, String> {
    let output = project.git()?.args(["symbolic-ref", "--quiet", "--short", "HEAD"]).output().map_err(|e| format!("running git symbolic-ref: {e}"))?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .ok_or_else(|| "`HEAD` is detached: the run needs the slice's branch checked out".to_string())
}

/// The slice the branch is for: the branch must be `lld/<slice>`, for the
/// slice given or, when none is, the one the branch names — as
/// `phase-check` reads it through [`crate::phase::slice_of_branch`]. Any
/// other branch stops the run naming it.
#[implements(spec::ThePreconditionNeedsTheSliceBranchCheckedOut)]
pub fn slice_of(branch: &str, given: Option<&str>) -> Result<String, Stop> {
    let wanted = given.unwrap_or("<slice>");
    match crate::phase::slice_of_branch(branch) {
        Some(slice) if given.is_none_or(|name| name == slice) => Ok(slice),
        Some(_) | None => Err(Stop {
            at: At::Precondition,
            decisions: vec![format!("the checked-out branch is `{branch}`, not `lld/{wanted}`: the run needs the slice's branch checked out")],
        }),
    }
}

/// The branch a slice's branch is taken from when the repository names no
/// default: git's own default name.
const DEFAULT_BRANCH: &str = "main";

/// The default branch this branch left: the target of
/// `refs/remotes/origin/HEAD` when the remote names one — `origin/main` and
/// the like — and [`DEFAULT_BRANCH`] when it does not, a repository with no
/// remote naming none.
fn default_branch(project: &Project) -> Result<String, String> {
    let output = project.git()?.args(["symbolic-ref", "--quiet", "--short", "refs/remotes/origin/HEAD"]).output().map_err(|e| format!("running git symbolic-ref: {e}"))?;
    let named = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(output.status.success().then_some(named).filter(|name| !name.is_empty()).unwrap_or_else(|| DEFAULT_BRANCH.to_string()))
}

/// Where the branch left the default branch ([`default_branch`]): the merge
/// base of the two, from `git merge-base <default> HEAD`, or the empty
/// string when there is none — a branch sharing no commit with the default
/// branch, a fresh repository's first among them, has its whole history to
/// itself.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags, spec::ThePreconditionNeedsAPhaseOneCommit)]
pub fn fork_point(project: &Project) -> Result<String, String> {
    let default = default_branch(project)?;
    let output = project.git()?.args(["merge-base", &default, "HEAD"]).output().map_err(|e| format!("running git merge-base: {e}"))?;
    let base = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() { Ok(base) } else { Ok(String::new()) }
}

/// The commits `git log` is asked for — the one decision over the fork
/// point: those after it, which are the branch's own, or the whole history
/// when there is none, which is then the branch's own.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags)]
pub fn own_commits(fork_point: &str) -> String {
    if fork_point.is_empty() { "HEAD".to_string() } else { format!("{fork_point}..HEAD") }
}

/// The subjects of the branch's own commits — those since it diverged from
/// the default branch ([`fork_point`], [`own_commits`]), never the whole
/// ancestry, since a merged slice leaves its `phase 1:`…`phase 7:` subjects
/// in every later branch's ancestry — newest first, from
/// `git log --format=%s`.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags, spec::ThePreconditionNeedsAPhaseOneCommit)]
pub fn log_subjects(project: &Project) -> Result<Vec<String>, String> {
    let range = own_commits(&fork_point(project)?);
    let log = crate::project::capture(project.git()?.args(["log", "--format=%s", &range]))?;
    Ok(log.lines().map(str::to_string).collect())
}

/// The paths one commit touched, from `git show --name-only --format=`:
/// what the reviewer is told to read, since its tools reach the worktree
/// and not git's objects.
#[implements(spec::TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles)]
pub fn commit_paths(project: &Project, commit: &str) -> Result<Vec<String>, String> {
    let shown = crate::project::capture(project.git()?.args(["show", "--name-only", "--format=", commit]))?;
    Ok(shown.lines().filter(|line| !line.is_empty()).map(str::to_string).collect())
}

/// The branch's own commits must hold a `phase 1:` commit; without one the
/// run stops in the phase LLD's words: no `phase 1:` commit means the run
/// stops before doing anything — the LLD is the human's. The ancestry a
/// merged slice left behind is not this branch's work and says nothing
/// about it.
#[implements(spec::ThePreconditionNeedsAPhaseOneCommit)]
pub fn phase_one_present(subjects: &[String]) -> Result<(), Stop> {
    subjects.iter().any(|subject| subject.starts_with("phase 1:")).then_some(()).ok_or_else(|| Stop {
        at: At::Precondition,
        decisions: vec!["the branch has no `phase 1:` commit: the LLD is the human's, and the run stops before doing anything".to_string()],
    })
}

/// The paths `git status --porcelain` reports: changed, staged, or
/// untracked.
fn dirty_paths(project: &Project) -> Result<Vec<String>, String> {
    let status = crate::project::capture(project.git()?.args(["status", "--porcelain"]))?;
    Ok(status.lines().filter_map(|line| line.get(3..)).map(str::to_string).collect())
}

/// The tree must be clean; a dirty one stops the run naming the paths.
#[implements(spec::ThePreconditionNeedsACleanTree)]
pub fn clean_tree(dirty: &[String]) -> Result<(), Stop> {
    dirty.is_empty().then_some(()).ok_or_else(|| Stop {
        at: At::Precondition,
        decisions: vec![format!("the working tree is not clean; commit or discard {} first", dirty.join(", "))],
    })
}

/// The phases the branch's own subjects commit: those whose tag
/// [`crate::phase::tag_of`] recognises as a phase with a check, in the
/// subjects' order.
#[implements(spec::CommittedPhasesAreReadFromTheSubjectTags)]
pub fn committed_phases(subjects: &[String]) -> Vec<Phase> {
    subjects
        .iter()
        .filter_map(|subject| match crate::phase::tag_of(subject) {
            Tag::Checked(phase) => Some(phase),
            Tag::Untagged | Tag::Unchecked(_) => None,
        })
        .collect()
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
    compile_time_accepted(crate_root, slice).then_some(()).ok_or_else(|| Stop {
        at: At::Precondition,
        decisions: vec![format!(
            "`{slice}` is a compile-time slice: editing it executes the model's code after every edit. Commit \
             docs/intent/{slice}/compile-time-accepted in the slice's crate to accept that, or run another slice."
        )],
    })
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

/// Says the phase is already committed and skipped ([`skipped_line`]); a
/// skipped phase records no decisions.
#[implements(spec::CommittedPhasesAreSkippedAndSaidSo)]
pub fn skipped(phase: Phase) -> Vec<String> {
    println!("{}", skipped_line(phase));
    Vec::new()
}

/// What the run prints for a phase already committed: that it is skipped,
/// naming the phase.
#[implements(spec::CommittedPhasesAreSkippedAndSaidSo)]
pub fn skipped_line(phase: Phase) -> String {
    format!("phase {}: already committed on this branch; skipped", number(phase))
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
    let n = number(phase);
    match end {
        Ok(WorkerEnd::Committed(hash, _)) => format!("phase {n}: committed {hash}"),
        Ok(WorkerEnd::Decisions(decisions)) => format!("phase {n}: stopped, uncommitted, with the decisions: {}", decisions.join("; ")),
        Ok(WorkerEnd::Refused(reason)) => format!("phase {n}: refused a ninth time, the tree left uncommitted: {reason}"),
        Err(reason) => format!("phase {n}: ended without a commit: {reason}"),
    }
}

/// A review's verdict as the run prints it: approved, rejected with its
/// findings, or the reason outside the model's doing.
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
pub fn verdict_line(phase: Phase, verdict: &Result<Review, String>) -> String {
    let n = number(phase);
    match verdict {
        Ok(Review::Approved) => format!("phase {n}: the review approved the commit"),
        Ok(Review::Rejected(findings)) => format!("phase {n}: the review rejected the commit: {}", findings.join("; ")),
        Err(reason) => format!("phase {n}: the review ended without a verdict: {reason}"),
    }
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
    let listed: String = decisions.iter().enumerate().map(|(at, decision)| format!("{}. {decision}\n", at + 1)).collect();
    format!("{listed}PR-ready: every phase is committed on {branch} and the gate passed")
}

/// *Stopped*, rendered: the numbered decisions, then where it stopped, on
/// the last line with the state.
#[implements(spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt)]
pub fn stopped(stop: &Stop) -> String {
    let listed: String = stop.decisions.iter().enumerate().map(|(at, decision)| format!("{}. {decision}\n", at + 1)).collect();
    let where_at = match stop.at {
        At::Precondition => "the precondition".to_string(),
        At::Phase(phase) => format!("phase {}", number(phase)),
        At::Review(phase) => format!("the review of phase {}", number(phase)),
    };
    format!("{listed}stopped at {where_at}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use lid_rs::validates;

    use super::replay::{self, Replay, Route, Seen, SessionScript};
    use super::*;
    use crate::phase::fixture;

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    /// The last line of a rendering.
    fn last_line(text: &str) -> &str {
        text.trim_end().lines().last().unwrap_or("")
    }

    /// Asserts a text mentions every needle.
    fn mentions(text: &str, needles: &[&str]) {
        assert!(needles.iter().all(|needle| text.contains(needle)), "expected {needles:?} in: {text}");
    }

    /// Asserts a stop is at `at` and its decisions mention every needle.
    fn stop_says(stop: &Stop, at: At, needles: &[&str]) {
        assert_eq!(stop.at, at, "{stop:?}");
        mentions(&stop.decisions.join("\n"), needles);
    }

    /// Asserts the replay opened exactly one session, `id`, dialled with the
    /// synced agent `agent` as its system prompt.
    fn only_dial_is(replay: &Replay, id: &str, dir: &Path, agent: &str) {
        assert_eq!(replay.opened(), [id], "exactly one session opens");
        assert_eq!(system_of(&first_dial(replay)).trim(), agent_text(dir, agent).trim());
    }

    /// Asserts a commit body carries `Lid-Rs-Agent: <agent>` between the phase
    /// and the tools trailers.
    fn agent_trailer_between(body: &str, agent: &str) {
        let phase_at = body.find("Lid-Rs-Phase: ").expect("the phase trailer");
        let agent_at = body.find(&format!("Lid-Rs-Agent: {agent}")).expect("the agent trailer");
        let tools_at = body.find("Lid-Rs-Tools:").expect("the tools trailer");
        assert!(phase_at < agent_at && agent_at < tools_at, "{body}");
    }

    /// Asserts a log holds two `phase 3:` commits over the LLD's, the older
    /// one being `rejected`, unamended.
    fn two_phase_three_commits(log: &[String], rejected: &str) {
        let subjects: Vec<&str> = log.iter().filter_map(|line| line.split_once(' ').map(|(_, s)| s)).collect();
        assert_eq!(subjects, ["phase 3: skeleton for hello", "phase 3: skeleton for hello", "phase 1: LLD for hello"]);
        assert!(log[1].starts_with(rejected), "the rejected commit stays in history unamended: {log:?}");
    }

    /// The fixture's precondition state as `precondition` establishes it,
    /// with these phases committed.
    fn state(project: &Project, committed: Vec<Phase>) -> Precondition {
        Precondition { slice: "hello".to_string(), branch: "lld/hello".to_string(), crate_root: project.root().expect("root"), committed }
    }

    /// A worker session whose one turn ends with a `stop` block naming one decision.
    fn stopping_session(id: &str, decision: &str) -> SessionScript {
        SessionScript::new(id).page(replay::settling_page(&format!("Blocked.\n\n```stop\n1. {decision}\n```\n")))
    }

    /// A worker session whose one turn ends with a `commit` block.
    fn committing_session(id: &str, message: &str) -> SessionScript {
        SessionScript::new(id).page(replay::settling_page(&format!("Done.\n\n```commit\n{message}```\n")))
    }

    /// A reviewer session whose one turn ends with a `review` block.
    fn reviewing_session(id: &str, block: &str) -> SessionScript {
        SessionScript::new(id).page(replay::settling_page(&format!("Reviewed.\n\n```review\n{block}```\n")))
    }

    /// The first dial the replay saw.
    fn first_dial(replay: &Replay) -> Seen {
        replay.seen().into_iter().find(|seen| seen.route() == Some(Route::Start)).expect("a dial")
    }

    /// The `system` a dial carried.
    fn system_of(dial: &Seen) -> String {
        dial.body.as_ref().and_then(|body| body["settings"]["system"].as_str()).expect("the dial's system").to_string()
    }

    /// The `max_cost` a dial carried.
    fn max_cost_of(dial: &Seen) -> f64 {
        dial.body.as_ref().and_then(|body| body["settings"]["max_cost"].as_f64()).expect("the dial's max_cost")
    }

    /// A synced agent file's body, its frontmatter cut by hand.
    fn agent_text(dir: &Path, name: &str) -> String {
        let text = std::fs::read_to_string(dir.join(".claude/agents").join(format!("{name}.md"))).expect("the synced agent");
        let after_open = text.strip_prefix("---\n").expect("frontmatter opens");
        let close = after_open.find("\n---\n").expect("frontmatter closes");
        after_open[close + "\n---\n".len()..].to_string()
    }

    /// Stages everything and commits it under `subject`; the new `HEAD`.
    fn commit_all(dir: &Path, subject: &str) -> String {
        fixture::git(dir, &["add", "-A"]);
        fixture::git(dir, &["commit", "-q", "--allow-empty", "-m", subject]);
        fixture::head(dir)
    }

    /// `git log --format=%H %s`, newest first.
    fn log_lines(dir: &Path) -> Vec<String> {
        let out = std::process::Command::new("git").args(["log", "--format=%H %s"]).current_dir(dir).output().expect("git log");
        String::from_utf8_lossy(&out.stdout).lines().map(str::to_string).collect()
    }

    #[test]
    #[validates(spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags)]
    fn canopy_takes_slice_door_and_max_cost_as_its_flags() {
        assert_eq!(PRODUCTION_DOOR, "https://api.canopyhq.dev");
        let defaults = Flags { slice: None, door: PRODUCTION_DOOR.to_string(), max_cost: DEFAULT_MAX_COST };
        assert_eq!(parse_flags(&[]).expect("no flags"), defaults);
        let all = parse_flags(&strings(&["--slice", "login", "--door", "http://127.0.0.1:1", "--max-cost", "2.5"])).expect("all three");
        assert_eq!(all, Flags { slice: Some("login".to_string()), door: "http://127.0.0.1:1".to_string(), max_cost: 2.5 });
    }

    #[test]
    #[validates(spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags)]
    fn any_other_flag_is_rejected_by_name_and_the_key_is_never_a_flag() {
        let key = parse_flags(&strings(&["--key", "secret"])).expect_err("the key is never a flag");
        assert!(key.contains("--key") && !key.contains("secret"), "{key}");
        let bare = parse_flags(&strings(&["--slice"])).expect_err("a flag needs its value");
        assert!(bare.contains("--slice"), "{bare}");
        let door = parse_flags(&strings(&["--slice", "login", "--dor", "x"])).expect_err("a misspelt flag");
        assert!(door.contains("--dor"), "{door}");
    }

    #[test]
    #[validates(spec::MaxCostIsTheFlagsAmountOrFive)]
    fn max_cost_is_the_flags_amount_or_five() {
        let given = parse_flags(&strings(&["--max-cost", "0.25"])).expect("an amount").max_cost;
        assert_eq!((DEFAULT_MAX_COST, Flags::default().max_cost, given, amount("12").expect("an integer amount")), (5.0, 5.0, 0.25, 12.0));
        mentions(&amount("lots").expect_err("not a number"), &["lots"]);
    }

    #[test]
    #[validates(spec::TheKeyComesFromCanopyKeyOrTheRunStopsFirst)]
    fn the_key_comes_from_canopy_key_or_the_run_stops_first() {
        assert_eq!(KEY_VARIABLE, "CANOPY_KEY");
        assert_eq!(api_key(Some("k-1".to_string())).expect("set"), "k-1");
        let stop = api_key(None).expect_err("unset");
        assert_eq!(stop.decisions.len(), 1, "{stop:?}");
        stop_says(&stop, At::Precondition, &["CANOPY_KEY"]);
    }

    #[test]
    #[validates(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
    fn every_phase_prints_its_ending() {
        let committed = ending_line(Phase::Three, &Ok(WorkerEnd::Committed("abc1234".to_string(), strings(&["keep it"]))));
        mentions(&committed, &["3", "abc1234"]);
        let decisions = ending_line(Phase::Five, &Ok(WorkerEnd::Decisions(strings(&["needs a claim", "and a row"]))));
        mentions(&decisions, &["needs a claim", "and a row"]);
        mentions(&ending_line(Phase::Two, &Ok(WorkerEnd::Refused("phase 2 is not clean".to_string()))), &["phase 2 is not clean"]);
        mentions(&ending_line(Phase::Two, &Err("max_requests reached".to_string())), &["max_requests reached"]);
        let (_dir, project) = fixture::copy("canopy-ending-reported");
        let blocked = "Blocked.\n\n```stop\n1. the gate needs a claim\n2. and an LLD row\n```\n";
        let replay = Replay::serve(vec![SessionScript::new("w7").page(replay::settling_page(blocked))]);
        let before_seven = state(&project, vec![Phase::Two, Phase::Three, Phase::Four, Phase::Five]);
        let stopped_at_seven = Stop { at: At::Phase(Phase::Seven), decisions: strings(&["the gate needs a claim", "and an LLD row"]) };
        let outcome = build(&project, &replay.door("k"), &before_seven, 5.0);
        assert_eq!(outcome, Outcome::Stopped(stopped_at_seven), "the ending the run reports is the worker's own, decision for decision");
        assert_eq!(replay.opened(), ["w7"], "and the session it names is the one that phase opened");
    }

    #[test]
    #[validates(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
    fn every_phase_prints_its_review_verdict() {
        let findings = strings(&["the leaf branches", "a helper sits in phase.rs"]);
        mentions(&verdict_line(Phase::Three, &Ok(Review::Approved)).to_lowercase(), &["3", "approved"]);
        mentions(&verdict_line(Phase::Three, &Ok(Review::Rejected(findings.clone()))), &["the leaf branches", "a helper sits in phase.rs"]);
        mentions(&verdict_line(Phase::Three, &Err("the session halted: budget".to_string())), &["the session halted: budget"]);
        let (dir, project) = fixture::copy("canopy-verdict-reported");
        let judgements = vec![reviewing_session("r3-yes", "approved: yes\n"), reviewing_session("r3-no", "approved: no\n1. the leaf branches\n2. a helper sits in phase.rs\n")];
        let replay = Replay::serve(judgements);
        let (door, committed) = (replay.door("k"), state(&project, vec![Phase::One]));
        let run = Run { project: &project, door: &door, state: &committed, max_cost: 5.0 };
        let (commit, recorded) = (fixture::head(&dir), strings(&["kept the enum"]));
        let carried = reviewed(&run, Phase::Three, Attempt::First, &commit, recorded.clone()).expect("the approval");
        let stop = reviewed(&run, Phase::Three, Attempt::Rework, &commit, recorded.clone()).expect_err("the rework's rejection");
        let rejected = Stop { at: At::Review(Phase::Three), decisions: findings };
        assert_eq!((carried, stop), (recorded, rejected), "an approval carries the commit's own decisions out; a rejection reports the reviewer's own findings");
        assert_eq!(replay.opened(), ["r3-yes", "r3-no"], "each verdict from the session that reached it");
    }

    #[test]
    #[validates(spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt)]
    fn the_last_line_is_the_terminal_state_and_the_exit_status_follows_it() {
        let ready = terminal("lld/hello", Outcome::PrReady { decisions: strings(&["a"]) }).expect("PR-ready is Ok: exit 0");
        mentions(last_line(&ready), &["PR-ready"]);
        let stop = Stop { at: At::Phase(Phase::Five), decisions: strings(&["x", "y"]) };
        let text = terminal("lld/hello", Outcome::Stopped(stop.clone())).expect_err("stopped is Err: exit 1");
        assert_eq!(text, stopped(&stop));
        mentions(&last_line(&text).to_lowercase(), &["stopped", "5"]);
        mentions(&text, &["1. x", "2. y"]);
        let at_review = stopped(&Stop { at: At::Review(Phase::Three), decisions: strings(&["z"]) });
        mentions(&last_line(&at_review).to_lowercase(), &["review", "3"]);
    }

    #[test]
    #[validates(spec::PrReadyEndsWithTheBranchAndEveryRecordedDecision)]
    fn pr_ready_ends_with_the_branch_and_every_recorded_decision() {
        let text = pr_ready("lld/hello", &strings(&["a decision", "another"]));
        mentions(&text, &["1. a decision", "2. another"]);
        mentions(last_line(&text), &["lld/hello", "PR-ready"]);
        mentions(last_line(&pr_ready("lld/hello", &[])), &["lld/hello", "PR-ready"]);
        let (dir, project) = fixture::copy("canopy-pr-ready");
        std::fs::write(dir.join("src/hello.rs"), "//! The hello slice.\n\n/// Greets, warmly.\npub fn greet() -> &'static str {\n    \"hello there\"\n}\n").expect("the phase's edit");
        let message = "phase 3: skeleton for hello\n\nThe skeleton.\n\n1. kept `greet` returning a static string\n";
        let replay = Replay::serve(vec![committing_session("w3", message), reviewing_session("r3", "approved: yes\n")]);
        let only_three = state(&project, vec![Phase::Two, Phase::Four, Phase::Five, Phase::Seven]);
        let outcome = build(&project, &replay.door("k"), &only_three, 5.0);
        let ready = Outcome::PrReady { decisions: strings(&["kept `greet` returning a static string"]) };
        assert_eq!((outcome.clone(), replay.opened()), (ready, strings(&["w3", "r3"])), "the phase's worker and its reviewer, and the decisions that commit recorded");
        mentions(&terminal(&only_three.branch, outcome).expect("PR-ready is Ok"), &["1. kept `greet` returning a static string", "lld/hello"]);
    }

    #[test]
    #[validates(spec::ThePreconditionNeedsTheSliceBranchCheckedOut)]
    fn the_precondition_needs_the_slice_branch_checked_out() {
        let from_branch = slice_of("lld/hello", None).expect("the branch names the slice");
        let given = slice_of("lld/hello", Some("hello")).expect("the given slice matches");
        assert_eq!((from_branch.as_str(), given.as_str()), ("hello", "hello"));
        stop_says(&slice_of("lld/hello", Some("login")).expect_err("another slice's branch"), At::Precondition, &["lld/hello", "login"]);
        stop_says(&slice_of("main", None).expect_err("no slice branch"), At::Precondition, &["main"]);
    }

    #[test]
    #[validates(spec::ThePreconditionNeedsTheSliceBranchCheckedOut)]
    fn the_precondition_reads_the_branch_with_git() {
        let (dir, project) = fixture::copy("canopy-precondition-branch");
        commit_all(&dir, "phase 1: LLD for hello");
        let state = precondition(&project, None).expect("lld/hello is checked out");
        assert_eq!((state.slice.as_str(), state.branch.as_str()), ("hello", "lld/hello"));
        assert_eq!(state.crate_root.canonicalize().expect("crate"), dir.canonicalize().expect("dir"));
        fixture::git(&dir, &["checkout", "-q", "main"]);
        stop_says(&precondition(&project, None).expect_err("main is no slice branch"), At::Precondition, &["main"]);
    }

    #[test]
    #[validates(spec::ThePreconditionNeedsAPhaseOneCommit)]
    fn the_precondition_needs_a_phase_one_commit() {
        phase_one_present(&strings(&["phase 2: claims for hello", "phase 1: LLD for hello"])).expect("present");
        stop_says(&phase_one_present(&strings(&["docs: notes", "initial"])).expect_err("absent"), At::Precondition, &["phase 1:"]);
        let (dir, project) = fixture::copy("canopy-precondition-phase-one");
        fixture::git(&dir, &["checkout", "-q", "--orphan", "fresh"]);
        fixture::git(&dir, &["commit", "-q", "-m", "start"]);
        fixture::git(&dir, &["branch", "-M", "lld/hello"]);
        stop_says(&precondition(&project, None).expect_err("a history without phase 1"), At::Precondition, &["phase 1:"]);
    }

    #[test]
    #[validates(spec::ThePreconditionNeedsAPhaseOneCommit, spec::CommittedPhasesAreReadFromTheSubjectTags)]
    fn a_phase_one_commit_in_the_ancestry_is_not_this_branchs() {
        let (dir, project) = fixture::copy("canopy-precondition-ancestry");
        assert_eq!(log_subjects(&project).expect("subjects"), Vec::<String>::new(), "the fixture's `phase 1:` sits on `main`, behind the branch point");
        stop_says(&precondition(&project, None).expect_err("a merged slice's tags are not this branch's work"), At::Precondition, &["phase 1:"]);
        commit_all(&dir, "phase 1: LLD for hello");
        assert_eq!(log_subjects(&project).expect("subjects"), strings(&["phase 1: LLD for hello"]), "the branch's own commit, and only it");
        assert_eq!(precondition(&project, None).expect("its own phase 1").committed, [Phase::One]);
    }

    #[test]
    #[validates(spec::CommittedPhasesAreReadFromTheSubjectTags)]
    fn the_branch_left_the_default_branch_the_remote_names() {
        let (dir, project) = fixture::copy("canopy-precondition-default-branch");
        commit_all(&dir, "phase 1: LLD for hello");
        let own = fixture::head(&dir);
        fixture::git(&dir, &["update-ref", "refs/remotes/origin/trunk", &own]);
        fixture::git(&dir, &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/trunk"]);
        assert_eq!(log_subjects(&project).expect("subjects"), Vec::<String>::new(), "the fork point is the remote's default branch, not `main`");
        commit_all(&dir, "phase 2: claims for hello");
        assert_eq!(log_subjects(&project).expect("subjects"), strings(&["phase 2: claims for hello"]), "what the branch added after it");
    }

    #[test]
    #[validates(spec::ThePreconditionNeedsACleanTree)]
    fn the_precondition_needs_a_clean_tree() {
        clean_tree(&[]).expect("clean");
        let stop = clean_tree(&strings(&["src/hello.rs", "notes.md"])).expect_err("dirty");
        assert_eq!(stop.at, At::Precondition);
        assert!(stop.decisions[0].contains("src/hello.rs") && stop.decisions[0].contains("notes.md"), "{stop:?}");
        let (dir, project) = fixture::copy("canopy-precondition-dirty");
        commit_all(&dir, "phase 1: LLD for hello");
        std::fs::write(dir.join("src/hello.rs"), "//! changed\n").expect("write");
        let stop = precondition(&project, None).expect_err("a modified file");
        assert!(stop.decisions[0].contains("src/hello.rs"), "{stop:?}");
    }

    #[test]
    #[validates(spec::CommittedPhasesAreReadFromTheSubjectTags)]
    fn committed_phases_are_read_from_the_subject_tags() {
        let subjects = strings(&["phase 6: leaves", "docs: the phase 3: tag in a body", "phase 3: skeleton for hello", "phase 2: claims for hello", "phase 1: LLD for hello"]);
        assert_eq!(committed_phases(&subjects), [Phase::Three, Phase::Two, Phase::One]);
        let (dir, project) = fixture::copy("canopy-precondition-committed");
        commit_all(&dir, "phase 1: LLD for hello");
        assert_eq!(precondition(&project, None).expect("state").committed, [Phase::One]);
        commit_all(&dir, "phase 2: claims for hello");
        commit_all(&dir, "phase 3: skeleton for hello");
        commit_all(&dir, "phase 6: leaves");
        assert_eq!(precondition(&project, None).expect("state").committed, [Phase::Three, Phase::Two, Phase::One]);
    }

    #[test]
    #[validates(spec::ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance)]
    fn a_compile_time_slice_stops_at_the_precondition_without_acceptance() {
        let (dir, project) = fixture::copy("canopy-acceptance");
        let root = project.root().expect("root");
        acceptance(&project, &root, "hello").expect("an ordinary slice needs no acceptance");
        let stop = accepted(&root, "hello").expect_err("no acceptance file");
        assert_eq!(stop.at, At::Precondition);
        assert!(stop.decisions[0].contains("docs/intent/hello/compile-time-accepted"), "{stop:?}");
        std::fs::write(dir.join("build.rs"), "fn main() {}\n").expect("build.rs");
        let compile_time = Project::load_graph_at(&dir.join("Cargo.toml")).expect("metadata");
        let stop = acceptance(&compile_time, &root, "hello").expect_err("a compile-time slice without acceptance");
        assert!(stop.decisions[0].contains("compile-time-accepted"), "{stop:?}");
        std::fs::write(dir.join("docs/intent/hello/compile-time-accepted"), "").expect("accept");
        acceptance(&compile_time, &root, "hello").expect("accepted");
        accepted(&root, "hello").expect("accepted");
    }

    #[test]
    #[validates(spec::CommittedPhasesAreSkippedAndSaidSo)]
    fn committed_phases_are_skipped_and_said_so() {
        mentions(&skipped_line(Phase::Two).to_lowercase(), &["2", "skip"]);
        assert!(skipped(Phase::Two).is_empty(), "a skipped phase records no decisions");
        let (_dir, project) = fixture::copy("canopy-skip-all");
        let replay = Replay::serve(vec![]);
        let all = state(&project, PHASES.to_vec());
        assert_eq!(build(&project, &replay.door("k"), &all, 5.0), Outcome::PrReady { decisions: vec![] });
        assert!(replay.seen().is_empty(), "every phase committed: no session opens");
    }

    #[test]
    #[validates(spec::CommittedPhasesAreSkippedAndSaidSo, spec::ThePhasesAreOneSessionEachInOrder)]
    fn the_run_starts_at_the_first_phase_without_a_commit() {
        let (dir, project) = fixture::copy("canopy-skip-some");
        let replay = Replay::serve(vec![stopping_session("s5", "phase 5 needs a decision")]);
        let committed = state(&project, vec![Phase::One, Phase::Two, Phase::Three, Phase::Four]);
        let outcome = build(&project, &replay.door("k"), &committed, 0.75);
        assert_eq!(outcome, Outcome::Stopped(Stop { at: At::Phase(Phase::Five), decisions: strings(&["phase 5 needs a decision"]) }));
        assert_eq!(replay.opened(), ["s5"]);
        let dial = first_dial(&replay);
        let (system, agent) = (system_of(&dial), agent_text(&dir, "lid-rs-phase-5"));
        assert_eq!((system.trim(), max_cost_of(&dial)), (agent.trim(), 0.75), "Phase 5's agent, and the amount given to build rather than the default");
    }

    #[test]
    #[validates(spec::ThePhasesAreOneSessionEachInOrder, spec::AWorkersStopBlockEndsTheRunWithItsDecisions, spec::MaxCostIsTheFlagsAmountOrFive)]
    fn the_phases_are_one_session_each_in_order() {
        assert_eq!(PHASES, [Phase::Two, Phase::Three, Phase::Four, Phase::Five, Phase::Seven]);
        let (dir, project) = fixture::copy("canopy-order");
        let replay = Replay::serve(vec![stopping_session("s2", "stop here")]);
        let outcome = build(&project, &replay.door("k"), &state(&project, vec![Phase::One]), 2.5);
        assert_eq!(outcome, Outcome::Stopped(Stop { at: At::Phase(Phase::Two), decisions: strings(&["stop here"]) }));
        only_dial_is(&replay, "s2", &dir, "lid-rs-phase-2");
        assert_eq!(max_cost_of(&first_dial(&replay)), 2.5, "the amount given to build reaches the dial, not the default");
    }

    #[test]
    #[validates(spec::AConfigThatPinsADialledSettingStopsTheRunNamingIt)]
    fn a_config_that_pins_a_dialled_setting_stops_the_run_naming_it() {
        let (_dir, project) = fixture::copy("canopy-pinned");
        let sentence = "the config pins `params`; the dial may not set it";
        let replay = Replay::serve(vec![SessionScript::new("s2").refusing(Route::Start, 403, sentence)]);
        let outcome = build(&project, &replay.door("k"), &state(&project, vec![Phase::One]), 5.0);
        assert_eq!(outcome, Outcome::Stopped(Stop { at: At::Phase(Phase::Two), decisions: strings(&[sentence]) }));
        assert_eq!(replay.seen().len(), 1, "the refused dial is the only request");
    }

    #[test]
    #[validates(spec::AHaltEndsTheRunWithItsReason)]
    fn a_halt_ends_the_run_with_its_reason() {
        let (_dir, project) = fixture::copy("canopy-halted");
        let replay = Replay::serve(vec![SessionScript::new("s2").page(vec![replay::requested(3), replay::halted("max_requests reached")])]);
        let outcome = build(&project, &replay.door("k"), &state(&project, vec![Phase::One]), 5.0);
        assert_eq!(outcome, Outcome::Stopped(Stop { at: At::Phase(Phase::Two), decisions: strings(&["max_requests reached"]) }));
        assert!(replay.stopped("s2"), "the halted session is stopped by the client too");
    }

    #[test]
    #[validates(spec::EveryCommittedPhaseIsReviewedBeforeTheNextOpens)]
    fn every_committed_phase_is_reviewed_before_the_next_opens() {
        let (dir, project) = fixture::copy("canopy-reviewed");
        let replay = Replay::serve(vec![reviewing_session("r3", "approved: yes\n")]);
        let (door, committed) = (replay.door("k"), state(&project, vec![Phase::One]));
        let run = Run { project: &project, door: &door, state: &committed, max_cost: 5.0 };
        let commit = fixture::head(&dir);
        let end = Ok(WorkerEnd::Committed(commit.clone(), strings(&["kept"])));
        assert_eq!(phase_ended(&run, Phase::Three, Attempt::First, end).expect("approved"), strings(&["kept"]));
        only_dial_is(&replay, "r3", &dir, "lid-rs-review");
        mentions(&replay::user_messages(&replay.landed("r3"))[0], &[commit.as_str(), "src/hello.rs"]);
    }

    #[test]
    #[validates(spec::AFirstRejectionOpensOneReworkSession, spec::AReworkPromptCarriesTheReviewersFindings)]
    fn a_first_rejection_opens_one_rework_session_with_the_findings() {
        let (dir, project) = fixture::copy("canopy-rework");
        let replay = Replay::serve(vec![stopping_session("w3-rework", "cannot fix without an LLD change")]);
        let (door, committed) = (replay.door("k"), state(&project, vec![Phase::One]));
        let run = Run { project: &project, door: &door, state: &committed, max_cost: 5.0 };
        let findings = strings(&["the leaf branches", "a helper sits in phase.rs"]);
        let stop = phase_judged(&run, Phase::Three, Attempt::First, Ok(Review::Rejected(findings)), vec![]).expect_err("the rework stopped");
        assert_eq!(stop, Stop { at: At::Phase(Phase::Three), decisions: strings(&["cannot fix without an LLD change"]) });
        only_dial_is(&replay, "w3-rework", &dir, "lid-rs-phase-3");
        mentions(&replay::user_messages(&replay.landed("w3-rework"))[0], &["1. the leaf branches", "2. a helper sits in phase.rs"]);
    }

    #[test]
    #[validates(spec::ASecondRejectionEndsTheRunWithTheFindings)]
    fn a_second_rejection_ends_the_run_with_the_findings() {
        let (dir, project) = fixture::copy("canopy-rejected-twice");
        let replay = Replay::serve(vec![reviewing_session("r3-again", "approved: no\n1. still branches\n2. still untraced\n")]);
        let (door, committed) = (replay.door("k"), state(&project, vec![Phase::One]));
        let run = Run { project: &project, door: &door, state: &committed, max_cost: 5.0 };
        let end = Ok(WorkerEnd::Committed(fixture::head(&dir), vec![]));
        let stop = phase_ended(&run, Phase::Three, Attempt::Rework, end).expect_err("the second rejection stops the run");
        assert_eq!(stop, Stop { at: At::Review(Phase::Three), decisions: strings(&["still branches", "still untraced"]) });
        assert_eq!(replay.opened(), ["r3-again"], "no third session opens");
    }

    #[test]
    #[validates(spec::AReworksCommitIsASecondPhaseCommitOnTheBranch, spec::AReworksCommitIsReviewedByAFreshSession, spec::TheCommitNamesItsSessionAsTheAgent, spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    fn a_reworks_commit_is_a_second_phase_commit_reviewed_by_a_fresh_session() {
        let (dir, project) = fixture::copy("canopy-rework-commit");
        std::fs::write(dir.join("src/hello.rs"), "//! The hello slice.\n\n/// Greets.\npub fn greet() -> &'static str {\n    \"hi\"\n}\n").expect("the rejected skeleton");
        let rejected = commit_all(&dir, "phase 3: skeleton for hello");
        std::fs::write(dir.join("src/hello.rs"), "//! The hello slice.\n\n/// Greets, warmly.\npub fn greet() -> &'static str {\n    \"hello there\"\n}\n").expect("the rework's edit");
        let replay = Replay::serve(vec![committing_session("w3", "phase 3: skeleton for hello\n\nReworked.\n"), reviewing_session("r3", "approved: yes\n")]);
        let (door, committed) = (replay.door("k"), state(&project, vec![Phase::One]));
        let run = Run { project: &project, door: &door, state: &committed, max_cost: 5.0 };
        let judged = phase_judged(&run, Phase::Three, Attempt::First, Ok(Review::Rejected(strings(&["greet is not warm"]))), vec![]);
        assert_eq!(judged.expect("the rework's commit is approved"), Vec::<String>::new());
        two_phase_three_commits(&log_lines(&dir), &rejected);
        let body = std::process::Command::new("git").args(["log", "-1", "--format=%B"]).current_dir(&dir).output().expect("git");
        agent_trailer_between(&String::from_utf8_lossy(&body.stdout), "canopy:w3");
        assert_eq!(replay.opened(), ["w3", "r3"], "the rework, then a fresh reviewer");
        mentions(&replay::user_messages(&replay.landed("r3"))[0], &[fixture::head(&dir).as_str()]);
        assert!(replay.stopped("w3") && replay.stopped("r3"), "the committed rework's session and its reviewer's are both stopped");
    }
}
