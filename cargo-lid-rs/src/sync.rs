use std::collections::BTreeMap;
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

/// Writes every file of the dependency's skill to the project's copy; a
/// second run changes nothing.
#[implements(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
pub fn write(project: &Project) -> Result<(), String> {
    write_files(&copy_root(project)?, &skill_files(project)?)
}

/// Fails naming every file the project's copy is missing, has extra, or
/// differs in, against the dependency's skill; writes nothing.
#[implements(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
pub fn check(project: &Project) -> Result<(), String> {
    let root = copy_root(project)?;
    let source = skill_files(project)?;
    let current = existing_files(&root);
    let differences = describe_differences(&current, &source);
    if differences.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} is missing or differs from the skill shipped by the resolved lid-rs; run `cargo lid-rs sync`:\n  {}",
            root.display(),
            differences.join("\n  ")
        ))
    }
}

/// The skill shipped by the `lid-rs` the project resolves, as relative path
/// → file content.
#[implements(spec::TheSkillComesFromTheResolvedLidRsDependency, spec::AMissingSkillSourceFailsByName)]
fn skill_files(project: &Project) -> Result<BTreeMap<PathBuf, String>, String> {
    let dir = project
        .lid_rs_package_dir()
        .ok_or("the project resolves no `lid-rs` dependency; add it first")?;
    let root = dir.join(SKILL_IN_CRATE);
    read_relative_files(&root).map_err(|e| {
        format!(
            "the resolved `lid-rs` at {} ships no skill ({}): {e}; a lid-rs of 0.2.1 or later is needed",
            dir.display(),
            root.display()
        )
    })
}

/// The project's copy root: `<workspace_root>/.claude/skills/lid-rs/`.
#[implements(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
fn copy_root(project: &Project) -> Result<PathBuf, String> {
    Ok(project.root()?.join(SKILL_IN_PROJECT))
}

/// The files currently under `root`, keyed by path relative to `root`; empty
/// if `root` doesn't exist.
fn existing_files(root: &Path) -> BTreeMap<PathBuf, String> {
    todo!()
}

/// Named differences between `current` and `source`: a path missing from
/// `current`, a path in `current` not in `source`, or a path whose content
/// differs.
fn describe_differences(current: &BTreeMap<PathBuf, String>, source: &BTreeMap<PathBuf, String>) -> Vec<String> {
    todo!()
}

/// One relative path's difference, if any, between the project's copy and
/// the dependency's skill.
fn describe_one_difference(relative: &Path, current: Option<&String>, source: Option<&String>) -> Option<String> {
    todo!()
}

/// Recursively reads every file under `root`, keyed by its path relative to
/// `root`.
fn read_relative_files(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    todo!()
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

    #[test]
    #[validates(spec::TheSkillComesFromTheResolvedLidRsDependency)]
    fn the_skill_comes_from_the_resolved_lid_rs_dependency() {
        let root = scratch("source");
        let source = skill_source(&project_at(&root)).expect("this checkout's lid-rs ships a skill");
        let canonical = std::fs::read_to_string(lid_rs_checkout().join(SKILL_IN_CRATE)).expect("canonical skill");
        assert!(source == canonical, "the source is the dependency's skill, byte for byte");
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
        let without = skill_source(&no_dependency).expect_err("no lid-rs must fail");
        let old_lid_rs = scratch("old-lid-rs");
        let too_old = Project::from_json(&doc(&root, &[("app", &root), ("lid-rs", &old_lid_rs)])).expect("parses");
        let too_old = skill_source(&too_old).expect_err("a lid-rs without a skill must fail");
        assert!(without.contains("lid-rs") && too_old.contains(&old_lid_rs.display().to_string()), "{without}\n{too_old}");
    }

    #[test]
    #[validates(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
    fn the_skill_copy_lives_at_the_workspace_root() {
        let root = scratch("write");
        let project = project_at(&root);
        write(&project).expect("first write");
        let path = root.join(SKILL_IN_PROJECT);
        let first = std::fs::read_to_string(&path).expect("the copy exists at the workspace root");
        write(&project).expect("second write");
        let second = std::fs::read_to_string(&path).expect("still there");
        let canonical = std::fs::read_to_string(lid_rs_checkout().join(SKILL_IN_CRATE)).expect("canonical");
        assert!(first == canonical && second == first, "the copy is the dependency's skill and a rerun changes nothing");
    }

    #[test]
    #[validates(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
    fn sync_check_fails_on_any_difference_and_writes_nothing() {
        let root = scratch("check");
        let project = project_at(&root);
        let path = root.join(SKILL_IN_PROJECT);
        let absent = check(&project).expect_err("an absent copy fails");
        let created_by_check = path.exists();
        write(&project).expect("write");
        let identical = check(&project);
        std::fs::write(&path, "edited\n").expect("edit");
        let edited = check(&project).expect_err("an edited copy fails");
        let overwritten = std::fs::read_to_string(&path).expect("read") != "edited\n";
        assert_eq!(
            (absent.contains("SKILL.md"), created_by_check, identical, edited.contains("SKILL.md"), overwritten),
            (true, false, Ok(()), true, false)
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
}
