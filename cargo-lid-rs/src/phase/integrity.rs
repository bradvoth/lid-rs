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
    sync::check(project).map_err(|e| format!("the synced artifacts changed since the phase started — code the check ran may have written them: {e}"))
}

/// Nothing outside the allowed set (paths relative to the workspace root)
/// is modified, untracked, or deleted; otherwise the offenders, named.
#[implements(spec::ChangesOutsideThePolicyRefuseTheStop)]
pub fn outside_policy_clean(project: &Project, allowed: &[PathBuf]) -> Result<(), String> {
    let outside: Vec<String> = changed_paths(project)?
        .into_iter()
        .filter(|path| !under_any(path, allowed))
        .map(|path| path.display().to_string())
        .collect();
    outside.is_empty().then_some(()).ok_or_else(|| {
        format!(
            "changed outside this phase's allowed paths — the agent could not have written these, so the code the check ran did: {}",
            outside.join(", ")
        )
    })
}

/// The changed paths within the allowed set — what a commit would stage.
#[implements(spec::NothingToCommitIsARefusal)]
pub fn changed_within(project: &Project, allowed: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    Ok(changed_paths(project)?.into_iter().filter(|path| under_any(path, allowed)).collect())
}

/// `git status --porcelain` as workspace-relative paths, without the
/// target directory: the hooks themselves write there (the tally).
fn changed_paths(project: &Project) -> Result<Vec<PathBuf>, String> {
    let output = crate::project::capture(project.git()?.args(["status", "--porcelain", "--untracked-files=all"]))?;
    let target = project.target_directory()?.strip_prefix(project.root()?).map(Path::to_path_buf).ok();
    Ok(output
        .lines()
        .filter_map(status_path)
        .filter(|path| target.as_deref().is_none_or(|t| !path.starts_with(t)))
        .collect())
}

/// The path of one porcelain status line; a rename's new name.
fn status_path(line: &str) -> Option<PathBuf> {
    let entry = line.get(3..)?;
    let path = entry.rsplit(" -> ").next().unwrap_or(entry);
    Some(PathBuf::from(path.trim_matches('"')))
}

/// Whether a path is one of, or under, the allowed entries.
fn under_any(path: &Path, allowed: &[PathBuf]) -> bool {
    allowed.iter().any(|entry| path == entry || path.starts_with(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    #[test]
    #[validates(spec::SyncedArtifactsMustMatchAtTheStop)]
    fn synced_artifacts_are_checked_against_the_dependency() {
        let (dir, project) = fixture::copy("integrity-synced");
        synced_artifacts_match(&project).expect("a fresh fixture is synced");
        std::fs::write(dir.join(".claude/agents/extra.md"), "x").expect("write");
        let err = synced_artifacts_match(&project).expect_err("an extra agent file is a difference");
        assert!(err.contains("extra.md"), "{err}");
    }

    #[test]
    #[validates(spec::ChangesOutsideThePolicyRefuseTheStop)]
    fn changes_outside_the_policy_are_named() {
        let (dir, project) = fixture::copy("integrity-outside");
        let allowed = paths(&["src/hello.rs", "src/hello"]);
        outside_policy_clean(&project, &allowed).expect("clean");
        std::fs::write(dir.join("src/hello.rs"), "//! changed\n").expect("write");
        outside_policy_clean(&project, &allowed).expect("a change inside the policy is fine");
        std::fs::write(dir.join("src/stray.rs"), "").expect("write");
        std::fs::write(dir.join("Cargo.toml"), "broken").expect("write");
        let err = outside_policy_clean(&project, &allowed).expect_err("outside changes are refused");
        assert!(err.contains("src/stray.rs") && err.contains("Cargo.toml") && !err.contains("hello.rs"), "{err}");
    }

    #[test]
    #[validates(spec::NothingToCommitIsARefusal)]
    fn what_a_commit_would_stage_is_the_changes_within_the_policy() {
        let (dir, project) = fixture::copy("integrity-within");
        let allowed = paths(&["src/hello.rs", "src/hello"]);
        assert!(changed_within(&project, &allowed).expect("status").is_empty());
        std::fs::create_dir_all(dir.join("src/hello")).expect("dir");
        std::fs::write(dir.join("src/hello/part.rs"), "").expect("write");
        std::fs::write(dir.join("README.md"), "x").expect("write");
        assert_eq!(changed_within(&project, &allowed).expect("status"), paths(&["src/hello/part.rs"]));
    }
}
