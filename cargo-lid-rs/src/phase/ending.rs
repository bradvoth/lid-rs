//! The stop protocol (`docs/intent/phase/lld.md` § `hook stop`): how a phase
//! agent's final message ends the phase, and what a refusal tells it.

use std::path::Path;

use lid_rs::implements;

use super::Phase;
use crate::project::Project;
use crate::spec;

/// How the agent's final message ends the phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    /// A ```` ```commit ```` block: the proposed commit message.
    Commit(String),
    /// A ```` ```stop ```` block: the numbered decisions that block the phase.
    Stop(String),
}

/// The check a failing output belongs to, for the `gates.md` row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// `missing_docs`.
    Three,
    /// `wildcard_enum_match_arm`.
    Six,
    /// `cognitive_complexity`.
    Seven,
    /// `fn_params_excessive_bools`.
    Eight,
    /// `too_many_lines`.
    Nine,
    /// The Phase 5 red run.
    RedRun,
    /// A surviving mutant.
    Twelve,
}

/// The message's ending, or the format when it has none or both.
#[implements(spec::AFinalMessageCarriesExactlyOneEnding)]
pub fn ending_of(message: &str) -> Result<Ending, String> {
    todo!()
}

/// The bodies of the fenced blocks tagged `lang`.
fn fenced_blocks<'a>(message: &'a str, lang: &str) -> Vec<&'a str> {
    todo!()
}

/// The commit message's subject must carry this phase's tag.
#[implements(spec::ACommitSubjectMustCarryThisPhasesTag)]
pub fn subject_matches(phase: Phase, message: &str) -> Result<(), String> {
    todo!()
}

/// The refusal for a failing check: the output, the `gates.md` row for the
/// check it names, and what the phase permits.
#[implements(spec::ARefusalCarriesTheOutputTheRuleAndThePermittedMoves)]
pub fn refusal_for(project: &Project, phase: Phase, output: &str) -> String {
    todo!()
}

/// The check a failing output names, if it names one.
#[implements(spec::AFailingOutputNamesItsCheck)]
pub fn check_of_output(output: &str) -> Option<Check> {
    todo!()
}

/// The check a clippy lint name maps to.
#[implements(spec::AFailingOutputNamesItsCheck)]
fn check_of_lint(lint: &str) -> Option<Check> {
    todo!()
}

/// The `gates.md` row for a check, from the synced skill.
fn gates_row(project: &Project, check: Check) -> Result<String, String> {
    todo!()
}

/// What the phase's policy lets the agent do about a failure.
fn permitted_moves(phase: Phase) -> String {
    todo!()
}

/// Stages exactly `paths` and commits `message` with `trailers`; the new
/// commit's hash.
#[implements(spec::OnlyThePoliciesPathsAreStaged)]
pub fn stage_and_commit(project: &Project, paths: &[&Path], message: &str, trailers: &str) -> Result<String, String> {
    todo!()
}
