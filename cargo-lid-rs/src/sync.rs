use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::Project;
use crate::spec;

/// Where the skill lives inside the `lid-rs` package.
const SKILL_IN_CRATE: &str = "skill/SKILL.md";

/// Where the project's copy lives, relative to the workspace root.
pub const SKILL_IN_PROJECT: &str = ".claude/skills/lid-rs/SKILL.md";

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

/// Writes the dependency's skill to the project's copy; a second run changes
/// nothing.
#[implements(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
pub fn write(project: &Project) -> Result<(), String> {
    write_file(&copy_path(project)?, &skill_source(project)?)
}

/// Fails naming the project's copy when it is absent or differs from the
/// dependency's skill; writes nothing.
#[implements(spec::SyncCheckFailsOnAnyDifferenceAndWritesNothing)]
pub fn check(project: &Project) -> Result<(), String> {
    let path = copy_path(project)?;
    let source = skill_source(project)?;
    let current = std::fs::read_to_string(&path).unwrap_or_default();
    if current == source {
        Ok(())
    } else {
        Err(format!(
            "{} is missing or differs from the skill shipped by the resolved lid-rs; run `cargo lid-rs sync`",
            path.display()
        ))
    }
}

/// The skill shipped by the `lid-rs` the project resolves.
#[implements(spec::TheSkillComesFromTheResolvedLidRsDependency, spec::AMissingSkillSourceFailsByName)]
fn skill_source(project: &Project) -> Result<String, String> {
    let dir = project
        .lid_rs_package_dir()
        .ok_or("the project resolves no `lid-rs` dependency; add it first")?;
    let path = dir.join(SKILL_IN_CRATE);
    std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "the resolved `lid-rs` at {} ships no skill ({}): {e}; a lid-rs of 0.2 or later is needed",
            dir.display(),
            path.display()
        )
    })
}

/// The project's copy: `<workspace_root>/.claude/skills/lid-rs/SKILL.md`.
#[implements(spec::TheSkillCopyLivesAtTheWorkspaceRoot)]
fn copy_path(project: &Project) -> Result<PathBuf, String> {
    Ok(project.root()?.join(SKILL_IN_PROJECT))
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
