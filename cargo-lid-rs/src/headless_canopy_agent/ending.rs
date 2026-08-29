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

#[cfg(test)]
mod tests {
    use lid_rs::validates;

    use super::super::replay::{self, Replay, SessionScript, mentions, sha256, strings};
    use super::*;
    use crate::phase::fixture;
    use crate::phase::tally;

    /// The fixture's precondition state: slice `hello` on `lld/hello`, Phase 1 committed.
    fn state(project: &Project) -> Precondition {
        Precondition { slice: "hello".to_string(), branch: "lld/hello".to_string(), crate_root: project.root().expect("root"), committed: vec![Phase::One] }
    }

    /// A worker driven on the replay for Phase 3.
    fn run(project: &Project, replay: &Replay, key: &str) -> Result<WorkerEnd, String> {
        worker(project, &replay.door(key), Phase::Three, &state(project), None, 5.0)
    }

    /// The reason a worker ended refused with; any other ending fails.
    fn refused_reason(end: WorkerEnd) -> String {
        match end {
            WorkerEnd::Refused(reason) => reason,
            WorkerEnd::Committed(_, _) | WorkerEnd::Decisions(_) => panic!("the ninth refusal ends the run: {end:?}"),
        }
    }

    #[test]
    #[validates(spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter)]
    fn the_system_prompt_is_the_synced_agent_body_without_its_frontmatter() {
        assert_eq!(without_frontmatter("---\nname: x\ntools: Read\n---\nYou run Phase 3.\n"), "You run Phase 3.\n");
        assert_eq!(without_frontmatter("No frontmatter here.\n"), "No frontmatter here.\n");
        let (dir, project) = fixture::copy("canopy-agent-body");
        let synced = std::fs::read_to_string(dir.join(".claude/agents/lid-rs-phase-3.md")).expect("synced");
        let body = agent_body(&project, "lid-rs-phase-3").expect("read");
        assert!(synced.trim_end().ends_with(body.trim_end()) && !body.contains("tools: Read") && body.contains("Phase 3"), "{body}");
        mentions(&agent_body(&project, "lid-rs-phase-9").expect_err("missing"), &[".claude/agents/lid-rs-phase-9.md"]);
    }

    #[test]
    #[validates(spec::TheWorkerPromptCarriesTheSliceTheLogAndTheSubject)]
    fn the_commit_subject_a_phase_must_use() {
        let subjects: Vec<String> = [Phase::Two, Phase::Three, Phase::Four, Phase::Five].iter().map(|p| subject_for(*p, "hello")).collect();
        assert_eq!(subjects, strings(&["phase 2: claims for hello", "phase 3: skeleton for hello", "phase 4: descend for hello", "phase 5: failing tests (red) for hello"]));
        assert!(subject_for(Phase::Seven, "hello").starts_with("phase 7:"));
    }

    #[test]
    #[validates(spec::TheWorkerPromptCarriesTheSliceTheLogAndTheSubject, spec::TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter, spec::TheWorkerPolicyAdmitsExactlyTheFiveTools, spec::MaxCostIsTheFlagsAmountOrFive)]
    fn the_worker_prompt_carries_the_slice_the_branch_the_lld_the_log_and_the_subject() {
        let (_dir, project) = fixture::copy("canopy-worker-prompt");
        let text = prompt_text(Phase::Three, &state(&project), "abc1234 phase 1: LLD for hello", None);
        mentions(&text, &["hello", "lld/hello", "docs/intent/hello/lld.md", "abc1234 phase 1: LLD for hello", "phase 3: skeleton for hello"]);
        let (settings, prompt) = worker_session(&project, Phase::Three, &state(&project), None, 2.5).expect("the dial and the prompt");
        mentions(&prompt, &["phase 1: LLD for hello", "phase 3: skeleton for hello"]);
        assert_eq!(settings.system, agent_body(&project, "lid-rs-phase-3").expect("body"));
        assert_eq!((settings.params, settings.max_cost, settings.policy.tools.len(), settings.policy.allows.len()), (json!({}), 2.5, 5, 5), "the amount given, not the default");
    }

    #[test]
    #[validates(spec::AReworkPromptCarriesTheReviewersFindings)]
    fn a_rework_prompt_carries_the_reviewers_findings() {
        assert_eq!(findings_section(None), "");
        let section = findings_section(Some(&strings(&["the leaf branches", "a helper sits in phase.rs"])));
        mentions(&section.to_lowercase(), &["reject", "1. the leaf branches", "2. a helper sits in phase.rs"]);
        let (_dir, project) = fixture::copy("canopy-rework-prompt");
        mentions(&prompt_text(Phase::Three, &state(&project), "log", Some(&strings(&["the leaf branches"]))), &["1. the leaf branches"]);
        assert!(!prompt_text(Phase::Three, &state(&project), "log", None).contains("the leaf branches"), "a first attempt carries no findings");
    }

    #[test]
    #[validates(spec::EverySessionIsStoppedWhenItsPhaseEnds, spec::AWorkersStopBlockEndsTheRunWithItsDecisions, spec::AHaltEndsTheRunWithItsReason, spec::TheSettledTextGoesToTheStopVerdictAsTheSession)]
    fn the_worker_session_is_stopped_however_the_phase_ends() {
        let (_dir, project) = fixture::copy("canopy-worker-stopped");
        let decisions = SessionScript::new("w-decisions").page(replay::settling_page("Blocked.\n\n```stop\n1. needs an LLD change\n2. and a claim\n```\n"));
        let halted = SessionScript::new("w-halted").page(vec![replay::halted("max_requests reached")]);
        let replay = Replay::serve(vec![decisions, halted]);
        assert_eq!(run(&project, &replay, "k").expect("ended"), WorkerEnd::Decisions(strings(&["needs an LLD change", "and a claim"])));
        assert_eq!(run(&project, &replay, "k").expect_err("halted"), "max_requests reached");
        assert!(replay.stopped("w-decisions") && replay.stopped("w-halted"), "both sessions stopped");
    }

    #[test]
    #[validates(spec::TheSettledTextGoesToTheStopVerdictAsTheSession, spec::TheCommitNamesItsSessionAsTheAgent)]
    fn the_settled_text_goes_to_the_stop_verdict_as_the_session() {
        let session = replay::session(Door::new("http://127.0.0.1:1", "k"), "3f0c1c9a", Phase::Three, WORKER_TOOLS.to_vec());
        let input = stop_input(&session, "Done.\n\n```stop\n1. x\n```\n");
        assert_eq!(input, HookInput { agent_id: "canopy:3f0c1c9a".to_string(), last_message: "Done.\n\n```stop\n1. x\n```\n".to_string(), ..HookInput::default() });
    }

    #[test]
    #[validates(spec::AWorkersStopBlockEndsTheRunWithItsDecisions)]
    fn numbered_lines_are_the_decisions_without_their_numbers() {
        assert_eq!(numbered("1. needs an LLD change\n2. and a claim\n"), strings(&["needs an LLD change", "and a claim"]));
        assert_eq!(numbered("phase 3: skeleton for hello\n\nBody.\n\n1. kept the enum\n"), strings(&["kept the enum"]));
        assert_eq!(numbered("No decisions.\n"), Vec::<String>::new());
    }

    #[test]
    #[validates(spec::ARefusedStopIsLandedAsTheNextUserMessage)]
    fn a_refused_stop_is_landed_as_the_next_user_message() {
        let (_dir, project) = fixture::copy("canopy-worker-refused-once");
        let script = SessionScript::new("w-refused").page(replay::settling_page("I finished the phase.")).page(replay::settling_page("```stop\n1. a decision\n```\n"));
        let replay = Replay::serve(vec![script]);
        assert_eq!(run(&project, &replay, "k").expect("ended"), WorkerEnd::Decisions(strings(&["a decision"])));
        let messages = replay::user_messages(&replay.landed("w-refused"));
        assert_eq!(messages.len(), 2, "the prompt, then the refusal's reason");
        mentions(&messages[1], &["```commit", "```stop"]);
        assert_eq!(tally::load(&project, "canopy:w-refused").expect("tally").stop_refusals, 1);
    }

    #[test]
    #[validates(spec::TheNinthConsecutiveRefusalEndsTheRun, spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    fn the_ninth_consecutive_refusal_ends_the_run() {
        let (_dir, project) = fixture::copy("canopy-worker-refused-nine");
        let script = (0..9).fold(SessionScript::new("w-nine"), |s, i| s.page(replay::settling_page(&format!("Attempt {i}, no block."))));
        let replay = Replay::serve(vec![script.page(replay::settling_page("```stop\n1. never reached\n```\n"))]);
        let reason = refused_reason(run(&project, &replay, "k").expect("ended"));
        mentions(&reason, &["```commit", "```stop"]);
        assert_eq!((STOP_ROUNDS, replay::user_messages(&replay.landed("w-nine")).len()), (9, 9), "the prompt and eight reasons; the ninth refusal is not landed");
        assert!(replay.stopped("w-nine"), "the refused session is stopped like any other");
    }

    #[test]
    #[validates(spec::AHaltEndsTheRunWithItsReason, spec::AProviderTerminalIsRetriedOnceThenStopsTheRun, spec::AQuietTailForFifteenMinutesStopsTheRun, spec::ADoorRefusalStopsTheRunWithItsSentence)]
    fn a_halt_is_the_reason_the_run_stops_with() {
        assert_eq!(halt_reason(Halt::Halted("max_requests reached".to_string())), "max_requests reached");
        assert_eq!(halt_reason(Halt::Refused("the session has stopped".to_string())), "the session has stopped");
        mentions(&halt_reason(Halt::Terminal(replay::TERMINAL_SENTENCE.to_string())), &[replay::TERMINAL_SENTENCE]);
        mentions(&halt_reason(Halt::Quiet).to_lowercase(), &["fifteen minutes"]);
    }

    #[test]
    #[validates(spec::NoPolicyRecordIsLandedAfterTheDial, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces, spec::TheKeyIsPresentedOnlyToTheDoor)]
    fn a_worker_session_lands_only_messages_and_completions_and_calls_only_the_five_routes() {
        let (_dir, project) = fixture::copy("canopy-worker-faces");
        let digest = sha256(r#"{"to":"lid-rs","args":{"path":"src/hello.rs"}}"#);
        let script = SessionScript::new("w-faces").page(replay::tool_call_page("read", json!({ "path": "src/hello.rs" }), &digest)).page(replay::settling_page("```stop\n1. done looking\n```\n"));
        let replay = Replay::serve(vec![script]);
        run(&project, &replay, "key-secret").expect("ended");
        let kinds: Vec<String> = replay.landed("w-faces").iter().map(|l| l.kind.clone()).collect();
        assert_eq!(kinds, strings(&["app.client.user_message", "app.invoke.completed"]), "never app.policy.configured");
        let seen = replay.seen();
        assert!(seen.iter().all(|s| s.route().is_some()), "every request is one of the five routes: {seen:?}");
        let leaked = seen.iter().any(|s| s.path.contains("key-secret") || s.body.as_ref().is_some_and(|b| b.to_string().contains("key-secret")));
        assert!(!leaked, "the key is in no record and on no path");
    }
}
