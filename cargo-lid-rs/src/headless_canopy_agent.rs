
use std::path::PathBuf;

use lid_rs::implements;

pub mod door;
pub mod ending;
pub mod review;
pub mod tools;
pub mod turn;

use door::Door;

use crate::phase::Phase;
use crate::project::Project;
use crate::spec;

/// `canopy [--slice <name>] [--door <url>] [--max-cost <amount>]`: the
/// flags — the door defaulting to canopy's production door, `max_cost` to
/// 5, any other flag rejected by name — the key from `CANOPY_KEY`, the
/// precondition, then the phases; prints, per phase, the sessions it opened
/// and its ending, and last the terminal state. `Ok` is *PR-ready*, printed
/// with the branch and every decision the phases recorded; `Err` is
/// *stopped* — the state and its decisions — which the binary's dispatcher
/// prints and exits 1 on.
#[implements(
    spec::CanopyTakesSliceDoorAndMaxCostAsItsFlags,
    spec::TheKeyComesFromCanopyKeyOrTheRunStopsFirst,
    spec::MaxCostIsTheFlagsAmountOrFive,
    spec::TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt,
    spec::PrReadyEndsWithTheBranchAndEveryRecordedDecision,
)]
pub fn run(args: &[String]) -> Result<(), String> {
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
/// before any session opens: the checked-out branch is `lld/<slice>` — the
/// slice given, or the branch's as `phase-check` reads it; the history
/// holds a `phase 1:` commit; the tree is clean; the committed phases are
/// those whose subjects `tag_of` recognises; and a compile-time slice
/// (`execution_class`) has the human's acceptance file
/// (`compile_time_accepted`). The first failure is the run's first
/// decision.
#[implements(
    spec::ThePreconditionNeedsTheSliceBranchCheckedOut,
    spec::ThePreconditionNeedsAPhaseOneCommit,
    spec::ThePreconditionNeedsACleanTree,
    spec::CommittedPhasesAreReadFromTheSubjectTags,
    spec::ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance,
)]
pub fn precondition(project: &Project, slice: Option<&str>) -> Result<Precondition, Stop> {
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

/// Phases 2, 3, 4, 5, and 7 in order, one worker session each — a
/// committed phase is skipped and said so — each commit reviewed by a fresh
/// session before the next phase opens: one flow decision per phase result.
/// A first rejection opens one rework session with the findings, whose
/// commit is a second `phase <n>:` commit reviewed the same way; a second
/// rejection stops the run with the findings. Prints each phase's ending —
/// the commit it made or the decisions that stopped it; the phase's sessions
/// are printed as [`Session::open`](turn::Session::open) opens them.
#[implements(
    spec::ThePhasesAreOneSessionEachInOrder,
    spec::CommittedPhasesAreSkippedAndSaidSo,
    spec::EveryPhasePrintsItsSessionsAndItsEnding,
    spec::EveryCommittedPhaseIsReviewedBeforeTheNextOpens,
    spec::AFirstRejectionOpensOneReworkSession,
    spec::AReworksCommitIsASecondPhaseCommitOnTheBranch,
    spec::AReworksCommitIsReviewedByAFreshSession,
    spec::ASecondRejectionEndsTheRunWithTheFindings,
)]
pub fn build(project: &Project, door: &Door, state: &Precondition, max_cost: f64) -> Outcome {
    todo!()
}
