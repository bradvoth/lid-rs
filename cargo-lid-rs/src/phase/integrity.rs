//! Integrity at the stop (`docs/intent/phase/lld.md` § Security posture):
//! the synced artifacts unchanged, and nothing changed outside the policy.

use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::Project;
use crate::spec;
use crate::sync;

/// The synced artifacts match the dependency's, as a refusal reason.
#[implements(spec::SyncedArtifactsMustMatchAtTheStop)]
pub fn synced_artifacts_match(project: &Project) -> Result<(), String> {
    todo!()
}

/// Nothing outside the allowed set (paths relative to the workspace root)
/// is modified, untracked, or deleted; otherwise the offenders, named.
#[implements(spec::ChangesOutsideThePolicyRefuseTheStop)]
pub fn outside_policy_clean(project: &Project, allowed: &[PathBuf]) -> Result<(), String> {
    todo!()
}

/// The changed paths within the allowed set — what a commit would stage.
#[implements(spec::NothingToCommitIsARefusal)]
pub fn changed_within(project: &Project, allowed: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    todo!()
}

/// `git status --porcelain` as workspace-relative paths.
fn changed_paths(project: &Project) -> Result<Vec<PathBuf>, String> {
    todo!()
}

/// Whether a path is one of, or under, the allowed entries.
fn under_any(path: &Path, allowed: &[PathBuf]) -> bool {
    todo!()
}
