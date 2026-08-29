//! The reviewer (`docs/intent/headless-canopy-agent/lld.md` § The
//! reviewer): a fresh session with the observation tools only, prompted to
//! refute a committed phase, and its verdict block.

use lid_rs::implements;
use serde_json::json;

use super::Precondition;
use super::door::{Door, Settings, policy_for};
use super::ending::{agent_body, turn};
use super::tools::Tool;
use super::turn::Session;
use crate::phase::Phase;
use crate::project::Project;
use crate::spec;

/// The tools the reviewer's policy admits: the three observation tools.
pub const REVIEW_TOOLS: [Tool; 3] = [Tool::Read, Tool::Grep, Tool::Glob];

/// The one finding of a review that missed its block twice.
pub const NO_VERDICT: &str = "the reviewer gave no verdict";

/// The reviewer's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub enum Review {
    /// `approved: yes`: the next phase opens.
    Approved,
    /// `approved: no` with its numbered findings.
    Rejected(Vec<String>),
}

/// The one ```` ```review ```` block in the reviewer's text
/// ([`review_block`]), parsed ([`verdict_of`]): `approved: yes`;
/// `approved: no` followed by numbered lines; anything else — no block,
/// two, or a malformed one — is the error, worded as the format to ask
/// for.
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub fn review_of(text: &str) -> Result<Review, String> {
    verdict_of(review_block(text)?)
}

/// The body of the one ```` ```review ```` block in a text; none or more
/// than one is the error, worded as the format to ask for.
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub fn review_block(text: &str) -> Result<&str, String> {
    todo!()
}

/// A review block's verdict: `approved: yes` is approved; `approved: no`
/// followed by numbered lines is rejected with those findings; anything
/// else is the error, worded as the format to ask for.
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub fn verdict_of(block: &str) -> Result<Review, String> {
    todo!()
}

/// The reviewer's user message: the phase, the slice, the commit, the LLD's
/// path, and the skill's files for that phase — `SKILL.md`,
/// `references/phase-<n>.md`, `references/discipline.md` — prompting it to
/// refute, and to answer with one ```` ```review ```` block.
#[implements(spec::TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles)]
pub fn review_prompt(phase: Phase, state: &Precondition, commit: &str) -> String {
    todo!()
}

/// The reviewer's dial and prompt: `system` the synced `lid-rs-review.md`
/// body, the policy admitting `read`, `grep`, and `glob` and nothing else,
/// empty `params`, `max_cost`; and the user message ([`review_prompt`]).
#[implements(spec::TheReviewerPolicyAdmitsOnlyTheObservationTools, spec::TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles)]
pub fn review_session(project: &Project, phase: Phase, state: &Precondition, commit: &str, max_cost: f64) -> Result<(Settings, String), String> {
    let system = agent_body(project, "lid-rs-review")?;
    let settings = Settings { system, policy: policy_for(&REVIEW_TOOLS), params: json!({}), max_cost };
    Ok((settings, review_prompt(phase, state, commit)))
}

/// Drives one reviewer session to a verdict: dialled with the reviewer's
/// settings, its turns run ([`verdicts`]), and the session stopped
/// whichever way they ended — a stop the door refuses is the error even
/// after a verdict. `Err` is a reason outside the model's doing that stops
/// the run.
#[implements(spec::EverySessionIsStoppedWhenItsPhaseEnds)]
pub fn review(project: &Project, door: &Door, phase: Phase, state: &Precondition, commit: &str, max_cost: f64) -> Result<Review, String> {
    let (settings, prompt) = review_session(project, phase, state, commit, max_cost)?;
    let mut session = Session::open(door, &settings, phase, REVIEW_TOOLS.to_vec())?;
    let verdict = verdicts(project, &mut session, &prompt);
    let sealed = session.stop();
    verdict.and_then(|review| sealed.map(|()| review))
}

/// The reviewer's turns: the prompt's turn parsed; a missing or malformed
/// block is asked for once more with the format ([`second_ask`]).
#[implements(spec::AMissingReviewBlockIsAskedForOnceMore)]
fn verdicts(project: &Project, session: &mut Session, prompt: &str) -> Result<Review, String> {
    match review_of(&turn(project, session, prompt)?) {
        Ok(review) => Ok(review),
        Err(format) => second_ask(project, session, &format),
    }
}

/// The one more ask, the format as the next user message: its turn parsed;
/// a second miss is a rejection whose one finding is that the reviewer
/// gave no verdict.
#[implements(spec::AMissingReviewBlockIsAskedForOnceMore, spec::ASecondMissingReviewBlockIsARejection)]
fn second_ask(project: &Project, session: &mut Session, format: &str) -> Result<Review, String> {
    match review_of(&turn(project, session, format)?) {
        Ok(review) => Ok(review),
        Err(_) => Ok(Review::Rejected(vec![NO_VERDICT.to_string()])),
    }
}
