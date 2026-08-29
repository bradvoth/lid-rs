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

#[cfg(test)]
mod tests {
    use lid_rs::validates;

    use super::super::replay::{self, Replay, SessionScript, mentions, strings};
    use super::*;
    use crate::phase::fixture;

    /// The fixture's precondition state: slice `hello` on `lld/hello`, Phase 1 committed.
    fn state(project: &Project) -> Precondition {
        Precondition { slice: "hello".to_string(), branch: "lld/hello".to_string(), crate_root: project.root().expect("root"), committed: vec![Phase::One] }
    }

    /// A reviewer driven on the replay for Phase 3's commit `abc1234`.
    fn run(project: &Project, replay: &Replay) -> Result<Review, String> {
        review(project, &replay.door("k"), Phase::Three, &state(project), "abc1234", 5.0)
    }

    #[test]
    #[validates(spec::TheReviewBlockParsesToApprovedOrFindings)]
    fn the_review_block_parses_to_approved_or_findings() {
        assert_eq!(review_of("Looks right.\n\n```review\napproved: yes\n```\n").expect("approved"), Review::Approved);
        let rejected = review_of("```review\napproved: no\n1. the leaf branches\n2. a helper sits in phase.rs\n```\n").expect("rejected");
        assert_eq!(rejected, Review::Rejected(strings(&["the leaf branches", "a helper sits in phase.rs"])));
        let malformed = ["No block at all.", "```review\napproved: maybe\n```", "```review\napproved: yes\n```\n```review\napproved: no\n1. x\n```", "```review\napproved: no\n```"];
        assert!(malformed.iter().all(|text| review_of(text).is_err()), "none, two, an unknown verdict, and a rejection without findings are malformed");
        mentions(&review_of("No block at all.").expect_err("malformed"), &["```review", "approved:"]);
    }

    #[test]
    #[validates(spec::TheReviewBlockParsesToApprovedOrFindings)]
    fn the_review_block_is_found_and_its_verdict_read() {
        assert_eq!(review_block("Before.\n```review\napproved: yes\n```\nAfter.\n").expect("one block"), "approved: yes\n");
        assert!(review_block("```review\na\n```\n```review\nb\n```").is_err(), "two blocks");
        assert_eq!(verdict_of("approved: no\n1. one\n2. two\n").expect("rejected"), Review::Rejected(strings(&["one", "two"])));
        mentions(&verdict_of("approved: perhaps\n").expect_err("malformed"), &["```review", "approved:"]);
    }

    #[test]
    #[validates(spec::TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles, spec::TheReviewerPolicyAdmitsOnlyTheObservationTools, spec::MaxCostIsTheFlagsAmountOrFive)]
    fn the_review_prompt_names_the_commit_the_lld_and_the_skill_files() {
        let (_dir, project) = fixture::copy("canopy-review-prompt");
        let text = review_prompt(Phase::Three, &state(&project), "abc1234");
        mentions(&text, &["3", "hello", "abc1234", "docs/intent/hello/lld.md", "SKILL.md", "references/phase-3.md", "references/discipline.md", "```review"]);
        mentions(&text.to_lowercase(), &["refut"]);
        let (settings, prompt) = review_session(&project, Phase::Three, &state(&project), "abc1234", 2.5).expect("the dial and the prompt");
        assert_eq!((prompt, settings.system), (text, agent_body(&project, "lid-rs-review").expect("body")));
        assert_eq!((settings.policy, settings.params, settings.max_cost), (policy_for(&REVIEW_TOOLS), json!({}), 2.5), "the amount given, not the default");
    }

    #[test]
    #[validates(spec::AMissingReviewBlockIsAskedForOnceMore)]
    fn a_missing_review_block_is_asked_for_once_more() {
        let (_dir, project) = fixture::copy("canopy-review-reask");
        let script = SessionScript::new("r-reask").page(replay::settling_page("I think it is fine.")).page(replay::settling_page("```review\napproved: yes\n```\n"));
        let replay = Replay::serve(vec![script]);
        assert_eq!(run(&project, &replay).expect("verdict"), Review::Approved);
        let messages = replay::user_messages(&replay.landed("r-reask"));
        assert_eq!(messages.len(), 2, "the prompt, then the format once more");
        mentions(&messages[1], &["```review", "approved:"]);
    }

    #[test]
    #[validates(spec::ASecondMissingReviewBlockIsARejection, spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    fn a_second_missing_review_block_is_a_rejection() {
        let (_dir, project) = fixture::copy("canopy-review-noverdict");
        let script = SessionScript::new("r-none").page(replay::settling_page("Fine.")).page(replay::settling_page("Still fine.")).page(replay::settling_page("```review\napproved: yes\n```\n"));
        let replay = Replay::serve(vec![script]);
        assert_eq!(run(&project, &replay).expect("verdict"), Review::Rejected(strings(&[NO_VERDICT])));
        assert_eq!(replay::user_messages(&replay.landed("r-none")).len(), 2, "no third ask");
        assert!(replay.stopped("r-none"), "the reviewer's session is stopped at its end");
    }

    #[test]
    #[validates(spec::TheReviewBlockParsesToApprovedOrFindings)]
    fn a_rejection_with_findings_is_the_reviewers_verdict() {
        let (_dir, project) = fixture::copy("canopy-review-rejected");
        let replay = Replay::serve(vec![SessionScript::new("r-no").page(replay::settling_page("```review\napproved: no\n1. the leaf branches\n```\n"))]);
        assert_eq!(run(&project, &replay).expect("verdict"), Review::Rejected(strings(&["the leaf branches"])));
    }
}
