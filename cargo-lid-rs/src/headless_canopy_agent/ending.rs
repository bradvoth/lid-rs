//! The worker and its ending (`docs/intent/headless-canopy-agent/lld.md`
//! § A phase is a session, § The ending): the phase agent's session, its
//! prompt, and the stop protocol driven through the phase library.

use lid_rs::implements;

use super::Precondition;
use super::door::{Door, Settings};
use crate::phase::Phase;
use crate::project::Project;
use crate::spec;

/// How a worker session ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerEnd {
    /// The check passed and the phase committed: the hash, and the
    /// decisions the commit's body recorded.
    Committed(String, Vec<String>),
    /// A `stop` block: its numbered decisions, nothing committed.
    Decisions(Vec<String>),
    /// The ninth consecutive refusal: its reason, the tree left dirty.
    Refused(String),
}

/// A synced agent file — `.claude/agents/<name>.md` under the project's
/// root — with its frontmatter removed; a missing file is an error naming
/// its path.
#[implements(spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter)]
pub fn agent_body(project: &Project, name: &str) -> Result<String, String> {
    todo!()
}

/// The worker's dial and prompt: `system` the phase agent's body, the
/// policy admitting the five tools, `max_cost`; and the user message —
/// the slice, the branch, the LLD's path, the branch's `git log
/// --oneline`, the commit subject this phase must use, and on a rework
/// the reviewer's findings.
#[implements(
    spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter,
    spec::TheWorkerPolicyAdmitsExactlyTheFiveTools,
    spec::TheWorkerPromptCarriesTheSliceTheLogAndTheSubject,
    spec::AReworkPromptCarriesTheReviewersFindings,
)]
pub fn worker_session(project: &Project, phase: Phase, state: &Precondition, findings: Option<&[String]>, max_cost: f64) -> Result<(Settings, String), String> {
    todo!()
}

/// Drives one worker session to its end: the turn, then the settled text
/// to the phase library's stop verdict under `canopy:<session>` — a commit
/// block runs the check and commits with that agent in the trailer; a
/// refusal is landed as the next user message and the turn driven again,
/// the ninth consecutive one ending the run; a stop block ends it with its
/// decisions. The session is stopped whichever way it ends. `Err` is a
/// reason outside the model's doing — a halt, a quiet tail, the provider's
/// sentence, the door's — that stops the run.
#[implements(
    spec::TheSettledTextGoesToTheStopVerdictAsTheSession,
    spec::TheCommitNamesItsSessionAsTheAgent,
    spec::ARefusedStopIsLandedAsTheNextUserMessage,
    spec::TheNinthConsecutiveRefusalEndsTheRun,
    spec::AWorkersStopBlockEndsTheRunWithItsDecisions,
    spec::EverySessionIsStoppedWhenItsPhaseEnds,
)]
pub fn worker(project: &Project, door: &Door, phase: Phase, state: &Precondition, findings: Option<&[String]>, max_cost: f64) -> Result<WorkerEnd, String> {
    todo!()
}
