//! The per-phase path policy (`docs/intent/phase/lld.md` § `hook pre-tool`):
//! which files a phase agent may write, where the slice's crate is, and
//! what kind of execution editing it entails.

use std::path::{Path, PathBuf};

use lid_rs::implements;

use super::Phase;
use crate::project::Project;
use crate::spec;

/// What a tool call does, for the policy and the tally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    /// `Edit`, `Write`, `MultiEdit`, `NotebookEdit`: subject to the policy.
    Edit,
    /// `Read`, `Grep`, `Glob`, `LSP`: never refused.
    Observation,
    /// `Bash` and anything else that runs: absent from the agents' tools.
    Command,
}

/// The policy's answer for one target path.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Within the phase's allowed set.
    Allowed,
    /// Outside it, with the reason the agent is given.
    Refused(String),
}

/// Whether editing this slice executes the agent's code at compile time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionClass {
    /// Nothing the agent writes runs before Phase 5.
    Ordinary,
    /// The slice's crate has a `proc-macro` or `custom-build` target — named.
    CompileTime(String),
}

/// The kind of a tool by its name.
pub fn kind_of(tool_name: &str) -> ToolKind {
    todo!()
}

/// The slice's crate: the workspace package whose manifest directory holds
/// `docs/intent/<slice>/lld.md`.
#[implements(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
pub fn slice_crate(project: &Project, slice: &str) -> Result<PathBuf, String> {
    todo!()
}

/// The phase's allowed set, as paths relative to the slice's crate; a
/// directory entry allows everything under it.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheSlicesSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteOnlyTheSliceModule,
)]
pub fn allowed_paths(phase: Phase, slice: &str) -> Vec<PathBuf> {
    todo!()
}

/// The verdict for a target: normalised against the crate first, then
/// matched against the phase's set.
#[implements(spec::PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy)]
pub fn allowed(phase: Phase, crate_root: &Path, slice: &str, target: &Path) -> Verdict {
    todo!()
}

/// The target relative to the crate root, or none when it has a parent
/// component or lies outside the crate.
fn within_crate(crate_root: &Path, target: &Path) -> Option<PathBuf> {
    todo!()
}

/// Whether a relative path is one of, or under, the allowed entries.
fn matches_any(relative: &Path, allowed: &[PathBuf]) -> bool {
    todo!()
}

/// The reason an edit is refused: the discipline rows tagged for the phase,
/// from the synced skill, and what the phase may do instead.
#[implements(spec::ARefusedEditQuotesTheDisciplineRow)]
pub fn refusal_reason(project: &Project, phase: Phase, relative: &Path, allowed: &[PathBuf]) -> String {
    todo!()
}

/// The `discipline.md` rows whose phase column names this phase.
fn discipline_rows(project: &Project, phase: Phase) -> Result<Vec<String>, String> {
    todo!()
}

/// The slice's execution class, from the crate's target kinds.
#[implements(spec::ACompileTimeSliceIsDisclosed)]
pub fn execution_class(project: &Project, crate_root: &Path) -> Result<ExecutionClass, String> {
    todo!()
}

/// Whether the human has accepted a compile-time slice: the file
/// `docs/intent/<slice>/compile-time-accepted` exists in the slice's crate.
#[implements(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
pub fn compile_time_accepted(crate_root: &Path, slice: &str) -> bool {
    todo!()
}
