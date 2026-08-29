//! The reviewer (`docs/intent/headless-canopy-agent/lld.md` § The
//! reviewer): a fresh session with the observation tools only, prompted to
//! refute a committed phase, and its verdict block.

use lid_rs::implements;

use super::Precondition;
use super::door::{Door, Settings};
use crate::phase::Phase;
use crate::project::Project;
use crate::spec;

/// The reviewer's verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub enum Review {
    /// `approved: yes`: the next phase opens.
    Approved,
    /// `approved: no` with its numbered findings.
    Rejected(Vec<String>),
}

/// The one ```` ```review ```` block in the reviewer's text, parsed:
/// `approved: yes`; `approved: no` followed by numbered lines; anything
/// else — no block, two, or a malformed one — is the error.
#[implements(spec::TheReviewBlockParsesToApprovedOrFindings)]
pub fn review_of(text: &str) -> Result<Review, String> {
    todo!()
}

/// The reviewer's dial and prompt: `system` the synced `lid-rs-review.md`
/// body, the policy admitting `read`, `grep`, and `glob` and nothing else,
/// `max_cost`; and the user message naming the phase, the slice, the
/// commit, the LLD, and the skill's files for that phase, prompting it to
/// refute.
#[implements(spec::TheReviewerPolicyAdmitsOnlyTheObservationTools, spec::TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles)]
pub fn review_session(project: &Project, phase: Phase, state: &Precondition, commit: &str, max_cost: f64) -> Result<(Settings, String), String> {
    todo!()
}

/// Drives one reviewer session to a verdict: the turn, its text parsed; a
/// missing or malformed block is asked for once more with the format, and
/// a second miss is a rejection whose one finding is that the reviewer gave
/// no verdict. The session is stopped whichever way it ends. `Err` is a
/// reason outside the model's doing that stops the run.
#[implements(spec::AMissingReviewBlockIsAskedForOnceMore, spec::ASecondMissingReviewBlockIsARejection, spec::EverySessionIsStoppedWhenItsPhaseEnds)]
pub fn review(project: &Project, door: &Door, phase: Phase, state: &Precondition, commit: &str, max_cost: f64) -> Result<Review, String> {
    todo!()
}
