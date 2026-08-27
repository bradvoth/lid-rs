use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::Project;
use crate::spec;

/// Where the skill lives inside the `lid-rs` package.
const SKILL_IN_CRATE: &str = "skill";

/// Where the project's copy lives, relative to the workspace root.
pub const SKILL_IN_PROJECT: &str = ".claude/skills/lid-rs";

/// What `sync` was asked to do.
#[derive(Debug, PartialEq, Eq)]
enum Mode {
    /// Write the project's copy.
    Write,
    /// Compare only; fail on any difference.
    Check,
}

/// Runs `sync` for the project the current directory belongs to.
pub fn run(args: &[String]) -> Result<(), String> {
    let mode = mode_of(args)?;
    let project = Project::load_graph()?;
    match mode {
        Mode::Write => write(&project),
        Mode::Check => check(&project),
    }
}

/// Parses `--check`; anything else is rejected by name.
#[implements(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
fn mode_of(args: &[String]) -> Result<Mode, String> {
    match args {
        [] => Ok(Mode::Write),
        [flag] if flag == "--check" => Ok(Mode::Check),
        [flag, ..] => Err(format!("unknown flag `{flag}` for sync; the only flag is --check")),
    }
}

/// Writes every file of every artifact the dependency ships to its place in
/// the project, then registers the hooks; a second run changes nothing.
#[implements(spec::TheSkillCopyLivesAtTheWorkspaceRoot, spec::SyncMirrorsEveryArtifactTheDependencyShips)]
pub fn write(project: &Project) -> Result<(), String> {
    artifacts().iter().try_for_each(|artifact| write_artifact(project, artifact))
}

/// Writes one artifact's files under its project root.
fn write_artifact(project: &Project, artifact: &Artifact) -> Result<(), String> {
    write_files(&artifact_root(project, artifact)?, &artifact_files(project, artifact)?)
}

/// Fails naming every file the project's copies are missing, have extra, or
/// differ in, against what the dependency ships; writes nothing.
#[implements(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing, spec::SyncMirrorsEveryArtifactTheDependencyShips)]
pub fn check(project: &Project) -> Result<(), String> {
    let differences: Vec<String> = artifacts()
        .iter()
        .map(|artifact| artifact_differences(project, artifact))
        .collect::<Result<Vec<_>, _>>()?
        .concat();
    differences.is_empty().then_some(()).ok_or_else(|| {
        format!(
            "the project's copies are missing or differ from what the resolved lid-rs ships; run `cargo lid-rs sync`:\n  {}",
            differences.join("\n  ")
        )
    })
}

/// One artifact's named differences, each prefixed with its project path.
fn artifact_differences(project: &Project, artifact: &Artifact) -> Result<Vec<String>, String> {
    let root = artifact_root(project, artifact)?;
    let source = artifact_files(project, artifact)?;
    let current = existing_files(&root);
    Ok(describe_differences(&current, &source)
        .into_iter()
        .map(|difference| format!("{}/{difference}", artifact.in_project))
        .collect())
}

/// The files currently under `root`, keyed by path relative to `root`; empty
/// if `root` doesn't exist.
fn existing_files(root: &Path) -> BTreeMap<PathBuf, String> {
    read_relative_files(root).unwrap_or_default()
}

/// Named differences between `current` and `source`: a path missing from
/// `current`, a path in `current` not in `source`, or a path whose content
/// differs.
fn describe_differences(current: &BTreeMap<PathBuf, String>, source: &BTreeMap<PathBuf, String>) -> Vec<String> {
    let paths: BTreeSet<&PathBuf> = current.keys().chain(source.keys()).collect();
    paths.into_iter().filter_map(|path| describe_one_difference(path, current.get(path), source.get(path))).collect()
}

/// One relative path's difference, if any, between the project's copy and
/// the dependency's skill.
fn describe_one_difference(relative: &Path, current: Option<&String>, source: Option<&String>) -> Option<String> {
    match (current, source) {
        (None, Some(_)) => Some(format!("{} is missing", relative.display())),
        (Some(_), None) => Some(format!("{} is not part of the skill", relative.display())),
        (Some(c), Some(s)) if c != s => Some(format!("{} differs", relative.display())),
        _ => None,
    }
}

/// Recursively reads every file under `root`, keyed by its path relative to
/// `root`.
pub(crate) fn read_relative_files(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let mut files = BTreeMap::new();
    collect_relative_files(root, root, &mut files)?;
    Ok(files)
}

/// Visits every entry directly under `dir` (a subtree of `root`), dispatching
/// each to a directory recursion or a file read.
fn collect_relative_files(root: &Path, dir: &Path, files: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|e| format!("reading {}: {e}", dir.display()))?.path();
        visit_entry(root, &path, files)?;
    }
    Ok(())
}

/// Recurses into `path` if it's a directory, otherwise records its content.
fn visit_entry(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    if path.is_dir() { collect_relative_files(root, path, files) } else { insert_relative_file(root, path, files) }
}

/// Records `path`'s content keyed by its path relative to `root`.
fn insert_relative_file(root: &Path, path: &Path, files: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    let relative = path.strip_prefix(root).expect("path is under root").to_path_buf();
    let content = std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    files.insert(relative, content);
    Ok(())
}

/// Writes `files`, each at its relative path under `root`, creating
/// directories.
fn write_files(root: &Path, files: &BTreeMap<PathBuf, String>) -> Result<(), String> {
    for (relative, content) in files {
        write_file(&root.join(relative), content)?;
    }
    Ok(())
}

/// Writes a file, creating its directories.
fn write_file(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(path, content).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// One artifact the `lid-rs` crate ships and `sync` mirrors: a directory,
/// at its path in the crate and its path in the project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Artifact {
    /// Path relative to the crate's manifest directory.
    pub in_crate: &'static str,
    /// Path relative to the workspace root.
    pub in_project: &'static str,
}

/// Everything the resolved `lid-rs` ships for the project, in mirror order:
/// the skill directory, the workflows, the phase agents.
#[implements(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
pub fn artifacts() -> [Artifact; 3] {
    [
        Artifact { in_crate: SKILL_IN_CRATE, in_project: SKILL_IN_PROJECT },
        Artifact { in_crate: "workflow", in_project: ".claude/workflows" },
        Artifact { in_crate: "agent", in_project: ".claude/agents" },
    ]
}

/// The files of one artifact as shipped by the `lid-rs` the project
/// resolves — registry or path, never the tool's own build — as relative
/// path → content; a missing dependency or a `lid-rs` that ships no such
/// artifact fails naming which.
#[implements(
    spec::TheSkillComesFromTheResolvedLidRsDependency,
    spec::AMissingSkillSourceFailsByName,
    spec::SyncMirrorsEveryArtifactTheDependencyShips,
)]
fn artifact_files(project: &Project, artifact: &Artifact) -> Result<BTreeMap<PathBuf, String>, String> {
    let dir = project
        .lid_rs_package_dir()
        .ok_or("the project resolves no `lid-rs` dependency; add it first")?;
    let root = dir.join(artifact.in_crate);
    read_relative_files(&root).map_err(|e| {
        format!(
            "the resolved `lid-rs` at {} ships no {} ({}): {e}; a lid-rs of {} or later is needed",
            dir.display(),
            artifact.in_crate,
            root.display(),
            env!("CARGO_PKG_VERSION")
        )
    })
}

/// The project-side root of one artifact.
#[implements(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
fn artifact_root(project: &Project, artifact: &Artifact) -> Result<PathBuf, String> {
    Ok(project.root()?.join(artifact.in_project))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    /// This checkout's `lid-rs` crate directory.
    fn lid_rs_checkout() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../lid-rs").canonicalize().expect("lid-rs checkout")
    }

    /// A fresh scratch directory.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-sync-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// A metadata document for a project rooted at `root` whose resolved
    /// packages are `packages` as `(name, manifest dir)`.
    fn doc(root: &Path, packages: &[(&str, &Path)]) -> String {
        let entries: Vec<String> = packages
            .iter()
            .map(|(name, dir)| {
                format!(r#"{{"name":"{name}","manifest_path":"{}","targets":[{{"kind":["lib"],"name":"{name}"}}]}}"#, dir.join("Cargo.toml").display())
            })
            .collect();
        format!(
            r#"{{"workspace_root":"{}","target_directory":"{}","packages":[{}]}}"#,
            root.display(),
            root.join("target").display(),
            entries.join(",")
        )
    }

    /// A synthetic project at a scratch root that resolves this checkout's lid-rs.
    fn project_at(root: &Path) -> Project {
        Project::from_json(&doc(root, &[("app", root), ("lid-rs", &lid_rs_checkout())])).expect("parses")
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_string()).collect()
    }

    /// The canonical skill's files, relative to its `skill/` root.
    fn canonical_skill_files() -> BTreeMap<PathBuf, String> {
        read_relative_files(&lid_rs_checkout().join(SKILL_IN_CRATE)).expect("this checkout's lid-rs ships a skill")
    }

    #[test]
    #[validates(spec::TheSkillComesFromTheResolvedLidRsDependency)]
    fn the_skill_comes_from_the_resolved_lid_rs_dependency() {
        let root = scratch("source");
        let source = artifact_files(&project_at(&root), &artifacts()[0]).expect("this checkout's lid-rs ships a skill");
        assert_eq!(source, canonical_skill_files(), "every file in the source is the dependency's skill, byte for byte");
    }

    #[test]
    #[validates(spec::TheSkillComesFromTheResolvedLidRsDependency)]
    fn the_live_workspace_resolves_its_own_lid_rs() {
        // From this crate's directory, the path dependency resolves to the checkout.
        let project = Project::load_graph().expect("cargo metadata with dependencies");
        let dir = project.lid_rs_package_dir().expect("lid-rs is a dependency of cargo-lid-rs");
        assert_eq!(dir.canonicalize().expect("exists"), lid_rs_checkout());
    }

    #[test]
    #[validates(spec::AMissingSkillSourceFailsByName)]
    fn a_missing_skill_source_fails_by_name() {
        let root = scratch("missing");
        let no_dependency = Project::from_json(&doc(&root, &[("app", &root)])).expect("parses");
        let without = artifact_files(&no_dependency, &artifacts()[0]).expect_err("no lid-rs must fail");
        let old_lid_rs = scratch("old-lid-rs");
        let too_old = Project::from_json(&doc(&root, &[("app", &root), ("lid-rs", &old_lid_rs)])).expect("parses");
        let too_old = artifact_files(&too_old, &artifacts()[0]).expect_err("a lid-rs without a skill directory must fail");
        assert!(without.contains("lid-rs") && too_old.contains(&old_lid_rs.display().to_string()), "{without}\n{too_old}");
    }

    #[test]
    #[validates(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
    fn the_skill_copy_lives_at_the_workspace_root() {
        let root = scratch("write");
        let project = project_at(&root);
        write(&project).expect("first write");
        let copy_dir = root.join(SKILL_IN_PROJECT);
        let first = read_relative_files(&copy_dir).expect("the copy exists at the workspace root");
        write(&project).expect("second write");
        let second = read_relative_files(&copy_dir).expect("still there");
        assert!(
            first == canonical_skill_files() && second == first,
            "the copy is every file of the dependency's skill, and a rerun changes nothing"
        );
    }

    #[test]
    #[validates(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
    fn sync_check_fails_on_any_difference_and_writes_nothing() {
        let root = scratch("check");
        let project = project_at(&root);
        let copy_dir = root.join(SKILL_IN_PROJECT);
        let absent = check(&project).expect_err("an absent copy fails");
        let created_by_check = copy_dir.exists();
        write(&project).expect("write");
        let identical = check(&project);

        let skill_md = copy_dir.join("SKILL.md");
        std::fs::write(&skill_md, "edited\n").expect("edit");
        let edited = check(&project).expect_err("an edited file fails");

        let extra = copy_dir.join("stray.md");
        std::fs::write(&extra, "not part of the skill\n").expect("write extra");
        let with_extra = check(&project).expect_err("an extra file fails");

        let overwritten = std::fs::read_to_string(&skill_md).expect("read") != "edited\n";
        let extra_removed = !extra.exists();
        assert_eq!(
            (
                absent.contains(SKILL_IN_PROJECT),
                created_by_check,
                identical,
                edited.contains("SKILL.md"),
                with_extra.contains("stray.md"),
                overwritten,
                extra_removed,
            ),
            (true, false, Ok(()), true, true, false, false)
        );
    }

    #[test]
    #[validates(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
    fn sync_flags_are_check_or_nothing() {
        // `run` rejects the flag before touching the project.
        let via_run = run(&strings(&["--bogus"])).is_err();
        assert_eq!(
            (mode_of(&[]), mode_of(&strings(&["--check"])), mode_of(&strings(&["--bogus"])).is_err_and(|e| e.contains("--bogus")), via_run),
            (Ok(Mode::Write), Ok(Mode::Check), true, true)
        );
    }
    #[test]
    #[validates(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
    fn the_mirror_table_names_every_artifact() {
        let rows: Vec<(&str, &str)> = artifacts().iter().map(|a| (a.in_crate, a.in_project)).collect();
        assert_eq!(rows, [("skill", ".claude/skills/lid-rs"), ("workflow", ".claude/workflows"), ("agent", ".claude/agents")]);
    }

    #[test]
    #[validates(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
    fn every_artifact_the_checkout_ships_has_files_and_a_project_root() {
        let root = scratch("artifacts");
        let project = project_at(&root);
        for artifact in &artifacts() {
            let files = artifact_files(&project, artifact).expect("the checkout ships it");
            assert!(!files.is_empty(), "{} ships files", artifact.in_crate);
            assert_eq!(artifact_root(&project, artifact).expect("root"), root.join(artifact.in_project));
        }
    }

    #[test]
    #[validates(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
    fn a_write_mirrors_the_workflow_and_the_agents() {
        let root = scratch("artifacts-write");
        let project = project_at(&root);
        write(&project).expect("writes every artifact");
        assert!(root.join(".claude/workflows/lid-rs.js").is_file());
        assert!(root.join(".claude/agents/lid-rs-phase-2.md").is_file());
        assert!(root.join(".claude/agents/lid-rs-review.md").is_file());
    }

    #[test]
    #[validates(spec::SyncMirrorsEveryArtifactTheDependencyShips)]
    fn an_extra_file_in_any_mirrored_directory_fails_the_check() {
        let root = scratch("artifacts-extra");
        let project = project_at(&root);
        write(&project).expect("writes every artifact");
        check(&project).expect("a fresh mirror passes");
        std::fs::write(root.join(".claude/agents/extra.md"), "x").expect("write");
        let err = check(&project).expect_err("an extra file is a difference");
        assert!(err.contains("extra.md"), "{err}");
    }
}
