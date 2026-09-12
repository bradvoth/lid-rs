//! Integrity at the stop (`docs/intent/phase/lld.md` § Security posture):
//! the synced artifacts unchanged, nothing changed outside the staged set,
//! and the files the hook itself wrote still as it wrote them.

use std::path::{Path, PathBuf};

use lid_rs::implements;

use super::Phase;
use super::policy::SliceCrates;
use crate::project::Project;
use super::spec;
use crate::sync;

/// The synced artifacts match the dependency's, as a refusal reason.
#[implements(spec::SyncedArtifactsMustMatchAtTheStop)]
pub fn synced_artifacts_match(project: &Project) -> Result<(), String> {
    sync::check(project).map_err(|e| format!("the synced artifacts changed since the phase started — code the check ran may have written them: {e}"))
}

/// Nothing outside the phase's staged set — `policy::staged_paths`: both
/// crates' allowed paths, and at Phase 7 the two root files the bump wrote —
/// is modified, untracked, or deleted; otherwise the offenders, named. The
/// two root files are not this check's to judge: `bumped_files_untouched`
/// holds them to what the bump wrote.
#[implements(spec::ChangesOutsideTheStagedSetRefuseTheStop, spec::IntegrityFiltersAgainstBothCratesStagedPaths)]
pub fn outside_policy_clean(project: &Project, phase: Phase, crates: &SliceCrates) -> Result<(), String> {
    let staged = super::policy::staged_paths(project, phase, crates)?;
    let outside: Vec<String> = changed_paths(project)?.iter().filter(|path| !under_any(path, &staged)).map(|path| path.display().to_string()).collect();
    outside
        .is_empty()
        .then_some(())
        .ok_or_else(|| format!("changed outside the phase's staged set — code the check ran may have written them: {}", outside.join(", ")))
}

/// Each of the files the hook itself wrote — `version_files`, the root
/// `Cargo.toml` and `Cargo.lock` as the Phase 7 bump left them — still equals
/// that content byte for byte; otherwise the refusal naming the file that
/// differs. Nothing to hold at any other phase, which passes.
#[implements(spec::TheBumpedRootFilesMustStillEqualWhatTheBumpWrote)]
pub fn bumped_files_untouched(project: &Project, version_files: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
    let root = project.root()?;
    let moved: Vec<String> = version_files
        .iter()
        .filter(|(path, written)| std::fs::read(root.join(path)).ok().as_ref() != Some(written))
        .map(|(path, _)| path.display().to_string())
        .collect();
    moved
        .is_empty()
        .then_some(())
        .ok_or_else(|| format!("changed since the bump wrote it — code the check ran may have rewritten it: {}", moved.join(", ")))
}

/// The workspace-relative `paths` with each one's bytes as it is now — what
/// the bump wrote, read back right after it for `bumped_files_untouched` to
/// hold the tree to after the check.
#[implements(spec::TheBumpedRootFilesMustStillEqualWhatTheBumpWrote)]
pub fn contents_of(project: &Project, paths: &[PathBuf]) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let root = project.root()?;
    paths
        .iter()
        .map(|path| std::fs::read(root.join(path)).map(|bytes| (path.clone(), bytes)).map_err(|e| format!("reading {}: {e}", path.display())))
        .collect()
}

/// The changed paths within `set` — the editing set for the nothing-to-commit
/// test, the staged set for what a commit stages.
#[implements(spec::NothingChangedInTheEditingSetIsARefusal)]
pub fn changed_within(project: &Project, set: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    Ok(changed_paths(project)?.into_iter().filter(|path| under_any(path, set)).collect())
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
    use crate::phase::policy::{staged_paths, workspace_paths};
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
    #[validates(spec::ChangesOutsideTheStagedSetRefuseTheStop)]
    fn changes_outside_the_staged_set_refuse_the_stop_naming_them() {
        let (dir, project) = fixture::copy("integrity-outside");
        // Phase 7 of `hello` in the fixture's one crate: `src/hello.rs`,
        // `src/hello` — and the two root files the bump writes.
        let crates = SliceCrates { slice: "hello".to_string(), own: dir.clone(), companion: None };
        outside_policy_clean(&project, Phase::Seven, &crates).expect("clean");
        std::fs::write(dir.join("src/hello.rs"), "//! changed\n").expect("write");
        outside_policy_clean(&project, Phase::Seven, &crates).expect("a change inside the staged set is fine");
        std::fs::write(dir.join("src/stray.rs"), "").expect("write");
        std::fs::write(dir.join("Cargo.toml"), "broken").expect("write");
        let err = outside_policy_clean(&project, Phase::Seven, &crates).expect_err("outside changes are refused");
        // The root manifest is in Phase 7's staged set: `bumped_files_untouched`
        // holds it to what the bump wrote, and this check does not name it.
        assert!(err.contains("src/stray.rs") && !err.contains("hello.rs") && !err.contains("Cargo.toml"), "{err}");
        // At any other phase the root files are outside the staged set, and named like anything else.
        let at_three = outside_policy_clean(&project, Phase::Three, &crates).expect_err("outside changes are refused");
        assert!(at_three.contains("Cargo.toml") && at_three.contains("src/stray.rs"), "{at_three}");
    }

    #[test]
    #[validates(spec::IntegrityFiltersAgainstBothCratesStagedPaths)]
    fn integrity_filters_against_both_crates_staged_paths() {
        let (dir, project) = fixture::two_member_workspace("integrity-companion", "", "");
        // Phase 5 of `m`: `src/m.rs` and `src/m` in `owner`; those and `tests/ui` in `app`.
        let crates = SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: Some(dir.join("app")) };
        outside_policy_clean(&project, Phase::Five, &crates).expect("clean");
        std::fs::write(dir.join("owner/src/m.rs"), "//! m\n").expect("write");
        std::fs::create_dir_all(dir.join("app/tests/ui")).expect("dir");
        std::fs::write(dir.join("app/tests/ui/fail.rs"), "").expect("write");
        outside_policy_clean(&project, Phase::Five, &crates).expect("a change under either crate's table is the phase's own");
        std::fs::create_dir_all(dir.join("owner/tests/ui")).expect("dir");
        std::fs::write(dir.join("owner/tests/ui/fail.rs"), "").expect("write");
        std::fs::write(dir.join("app/src/lib.rs"), "// changed\n").expect("write");
        let err = outside_policy_clean(&project, Phase::Five, &crates).expect_err("outside either table is named");
        let named = err.contains("owner/tests/ui/fail.rs") && err.contains("app/src/lib.rs");
        assert!(named && !err.contains("owner/src/m.rs") && !err.contains("app/tests/ui/fail.rs"), "{err}");
        // At Phase 7 the staged set of both crates carries the two files the
        // bump writes at the workspace root, and no other manifest: the root's
        // is not named there, while a member's is, beside the two above.
        std::fs::write(dir.join("Cargo.toml"), "[workspace]\nresolver = \"2\"\nmembers = [\"owner\", \"app\"]\n# bumped\n").expect("write");
        std::fs::write(dir.join("owner/Cargo.toml"), "[package]\nname = \"owner\"\nversion = \"0.1.0\"\nedition = \"2021\"\n# bumped\n").expect("write");
        let at_seven = outside_policy_clean(&project, Phase::Seven, &crates).expect_err("outside either table is named");
        let only_the_members_manifest = at_seven.contains("owner/Cargo.toml") && !at_seven.replace("owner/Cargo.toml", "").contains("Cargo.toml");
        assert!(only_the_members_manifest && at_seven.contains("owner/tests/ui/fail.rs") && at_seven.contains("app/src/lib.rs"), "{at_seven}");
    }

    #[test]
    #[validates(spec::NothingChangedInTheEditingSetIsARefusal)]
    fn nothing_changed_in_the_editing_set_is_a_refusal_read_from_the_changes_within_it() {
        let (dir, project) = fixture::versioned("integrity-within");
        let crates = SliceCrates { slice: "hello".to_string(), own: dir.clone(), companion: None };
        let editing = workspace_paths(&project, Phase::Seven, &crates).expect("the editing set");
        assert_eq!(editing, paths(&["src/hello.rs", "src/hello"]));
        assert!(changed_within(&project, &editing).expect("status").is_empty());
        // A Phase 7 stop whose agent changed nothing: the bump lands — the
        // wrong subject stops it there, before the check — and dirties the two
        // root files, which are the staged set's and not the editing set's.
        crate::phase::hook_stop(&project, Phase::Seven, &fixture::stop_input("n", "```commit\nphase 7: 9.9.9: nothing\n```\n")).expect("hook");
        let staged = staged_paths(&project, Phase::Seven, &crates).expect("the staged set");
        assert_eq!(changed_within(&project, &staged).expect("status"), paths(&["Cargo.lock", "Cargo.toml"]), "what the bump wrote");
        assert!(changed_within(&project, &editing).expect("status").is_empty(), "nothing the agent could have written changed: nothing to commit");
        // What the agent writes is read from the editing set; a change elsewhere is not.
        std::fs::create_dir_all(dir.join("src/hello")).expect("dir");
        std::fs::write(dir.join("src/hello/part.rs"), "").expect("write");
        std::fs::write(dir.join("README.md"), "x").expect("write");
        assert_eq!(changed_within(&project, &editing).expect("status"), paths(&["src/hello/part.rs"]));
    }

    #[test]
    #[validates(spec::TheBumpedRootFilesMustStillEqualWhatTheBumpWrote)]
    fn the_bumped_root_files_must_still_equal_what_the_bump_wrote() {
        let (dir, project) = fixture::copy("integrity-bumped");
        let files = paths(&["Cargo.toml", "Cargo.lock"]);
        let written = contents_of(&project, &files).expect("read as the bump left them");
        let (names, manifest): (Vec<PathBuf>, &[u8]) = (written.iter().map(|(path, _)| path.clone()).collect(), &written[0].1);
        assert_eq!((names, manifest), (files, std::fs::read(dir.join("Cargo.toml")).expect("manifest").as_slice()));
        bumped_files_untouched(&project, &written).expect("still as the bump wrote them");
        // Something the check ran rewrote the manifest: refused naming that file, and not the lock beside it.
        std::fs::write(dir.join("Cargo.toml"), "broken").expect("write");
        let err = bumped_files_untouched(&project, &written).expect_err("Cargo.toml moved since the bump");
        assert!(err.contains("Cargo.toml") && !err.contains("Cargo.lock"), "{err}");
        // At any other phase the hook wrote nothing, so there is nothing to hold.
        bumped_files_untouched(&project, &[]).expect("nothing to hold");
    }
}
