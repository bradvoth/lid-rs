//! The worker and its ending (`docs/intent/headless-canopy-agent/lld.md`
//! § A phase is a session, § The ending): the phase agent's session, its
//! prompt, and the stop protocol driven through the phase library.

use lid_rs::implements;
use serde_json::json;

use super::Precondition;
use super::door::{Door, Settings, policy_for};
use super::tools::Tool;
use super::turn::{Halt, Session, drive};
use crate::phase::ending::{Ending, ending_of};
use crate::phase::{HookInput, HookVerdict, Phase, hook_stop};
use crate::project::Project;
use crate::spec;

/// The tools a worker's policy admits: all five.
pub const WORKER_TOOLS: [Tool; 5] = [Tool::Read, Tool::Grep, Tool::Glob, Tool::Edit, Tool::Write];

/// How many turns a worker gets: eight refusals are landed as the next
/// message; the ninth consecutive refusal ends the run.
pub const STOP_ROUNDS: usize = 9;

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

/// How one turn ended under the stop verdict.
enum Round {
    /// The verdict let the phase end: committed, or stopped by decisions.
    Ended(WorkerEnd),
    /// The verdict refused, with the reason the next message carries.
    Refused(String),
}

/// A synced agent file — `.claude/agents/<name>.md` under the project's
/// root — with its frontmatter removed ([`without_frontmatter`]); a missing
/// file is an error naming its path.
#[implements(spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter)]
pub fn agent_body(project: &Project, name: &str) -> Result<String, String> {
    todo!()
}

/// An agent file's text without its frontmatter: the block between the
/// leading `---` line and the next; text with no frontmatter is itself.
#[implements(spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter)]
pub fn without_frontmatter(text: &str) -> String {
    todo!()
}

/// The number a phase is written as in an agent file's name and a commit
/// subject.
pub(super) fn number(phase: Phase) -> u8 {
    todo!()
}

/// The branch's `git log --oneline`, whole.
fn oneline_log(project: &Project) -> Result<String, String> {
    todo!()
}

/// The commit subject a phase must use, as the skill's phase files and the
/// workflow name it: `phase 2: claims for <slice>`, `phase 3: skeleton for
/// <slice>`, `phase 4: descend for <slice>`, `phase 5: failing tests (red)
/// for <slice>`, and for Phase 7 `phase 7: <version>: <what and why>`.
#[implements(spec::TheWorkerPromptCarriesTheSliceTheLogAndTheSubject)]
pub fn subject_for(phase: Phase, slice: &str) -> String {
    todo!()
}

/// The worker's user message: the slice, the branch, the LLD's path
/// (`docs/intent/<slice>/lld.md` in the slice's crate), the branch's
/// `git log --oneline`, the commit subject this phase must use
/// ([`subject_for`]), and what a rework adds ([`findings_section`]).
#[implements(spec::TheWorkerPromptCarriesTheSliceTheLogAndTheSubject)]
pub fn prompt_text(phase: Phase, state: &Precondition, log: &str, findings: Option<&[String]>) -> String {
    todo!()
}

/// What a rework's prompt adds: the reviewer's findings, numbered, under a
/// line saying the phase's commit was rejected and this session reworks
/// it; nothing on a first attempt.
#[implements(spec::AReworkPromptCarriesTheReviewersFindings)]
pub fn findings_section(findings: Option<&[String]>) -> String {
    todo!()
}

/// The worker's dial and prompt: `system` the synced
/// `lid-rs-phase-<n>.md` body, the policy admitting the five tools, empty
/// `params`, `max_cost`; and the user message ([`prompt_text`]).
#[implements(spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter, spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn worker_session(project: &Project, phase: Phase, state: &Precondition, findings: Option<&[String]>, max_cost: f64) -> Result<(Settings, String), String> {
    let system = agent_body(project, &format!("lid-rs-phase-{}", number(phase)))?;
    let settings = Settings { system, policy: policy_for(&WORKER_TOOLS), params: json!({}), max_cost };
    Ok((settings, prompt_text(phase, state, &oneline_log(project)?, findings)))
}

/// Drives one worker session to its end: dialled with the worker's
/// settings, its turns run ([`rounds`]), and the session stopped whichever
/// way they ended — a stop the door refuses is the error even after a
/// commit. `Err` is a reason outside the model's doing — a halt, a quiet
/// tail, the provider's sentence, the door's — that stops the run.
#[implements(spec::EverySessionIsStoppedWhenItsPhaseEnds)]
pub fn worker(project: &Project, door: &Door, phase: Phase, state: &Precondition, findings: Option<&[String]>, max_cost: f64) -> Result<WorkerEnd, String> {
    let (settings, prompt) = worker_session(project, phase, state, findings, max_cost)?;
    let mut session = Session::open(door, &settings, phase, WORKER_TOOLS.to_vec())?;
    let end = rounds(project, &mut session, prompt);
    let sealed = session.stop();
    end.and_then(|ended| sealed.map(|()| ended))
}

/// The worker's turns: the prompt, then each refusal's reason as the next
/// user message in the same session, until the stop verdict lets the phase
/// end; the ninth consecutive refusal ends the run with its reason, the
/// tree left dirty and uncommitted.
#[implements(spec::ARefusedStopIsLandedAsTheNextUserMessage, spec::TheNinthConsecutiveRefusalEndsTheRun)]
fn rounds(project: &Project, session: &mut Session, prompt: String) -> Result<WorkerEnd, String> {
    let mut message = prompt;
    for _ in 0..STOP_ROUNDS {
        match round(project, session, &message)? {
            Round::Ended(end) => return Ok(end),
            Round::Refused(reason) => message = reason,
        }
    }
    Ok(WorkerEnd::Refused(message))
}

/// One turn and its verdict: the settled text handed to the phase library's
/// stop verdict under `canopy:<session>` ([`stop_input`]) — allowed, the
/// ending it carried ([`ended`]); refused, the reason.
#[implements(spec::TheSettledTextGoesToTheStopVerdictAsTheSession)]
fn round(project: &Project, session: &mut Session, message: &str) -> Result<Round, String> {
    let settled = turn(project, session, message)?;
    match hook_stop(project, session.phase, &stop_input(session, &settled))? {
        HookVerdict::Allow => Ok(Round::Ended(ended(project, &settled)?)),
        HookVerdict::Refuse(reason) | HookVerdict::Context(reason) => Ok(Round::Refused(reason)),
    }
}

/// One turn's settled text; a halt is the reason that stops the run
/// ([`halt_reason`]).
pub(super) fn turn(project: &Project, session: &mut Session, message: &str) -> Result<String, String> {
    drive(project, session, message).map(|settled| settled.text).map_err(halt_reason)
}

/// A halt as the reason the run stops with: the platform's reason, the
/// provider's sentence, the quiet tail said so, or the door's sentence.
#[implements(
    spec::AHaltEndsTheRunWithItsReason,
    spec::AProviderTerminalIsRetriedOnceThenStopsTheRun,
    spec::AQuietTailForFifteenMinutesStopsTheRun,
    spec::ADoorRefusalStopsTheRunWithItsSentence,
)]
pub fn halt_reason(halt: Halt) -> String {
    todo!()
}

/// The phase library's input for the stop verdict: the session's agent id
/// (`canopy:<session>`, which the commit's `Lid-Rs-Agent` trailer carries)
/// and the settled text as the agent's final message.
#[implements(spec::TheSettledTextGoesToTheStopVerdictAsTheSession, spec::TheCommitNamesItsSessionAsTheAgent)]
pub fn stop_input(session: &Session, text: &str) -> HookInput {
    todo!()
}

/// What an allowed stop ended with: a commit block, and the phase's check
/// has committed — the new `HEAD` and the decisions the commit's body
/// numbered; a stop block — its numbered decisions, nothing committed.
#[implements(spec::AWorkersStopBlockEndsTheRunWithItsDecisions)]
fn ended(project: &Project, text: &str) -> Result<WorkerEnd, String> {
    match ending_of(text)? {
        Ending::Commit(message) => Ok(WorkerEnd::Committed(head(project)?, numbered(&message))),
        Ending::Stop(decisions) => Ok(WorkerEnd::Decisions(numbered(&decisions))),
    }
}

/// The commit `HEAD` names, from `git rev-parse HEAD`.
fn head(project: &Project) -> Result<String, String> {
    todo!()
}

/// The numbered lines of a text — `1. …`, `2. …` — without their numbers:
/// the decisions a stop block, a commit body, or a review block carries.
pub fn numbered(text: &str) -> Vec<String> {
    todo!()
}
