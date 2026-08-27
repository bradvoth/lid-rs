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
    match tool_name {
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => ToolKind::Edit,
        "Read" | "Grep" | "Glob" | "LSP" => ToolKind::Observation,
        _ => ToolKind::Command,
    }
}

/// The slice's crate: the workspace package whose manifest directory holds
/// `docs/intent/<slice>/lld.md`.
#[implements(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
pub fn slice_crate(project: &Project, slice: &str) -> Result<PathBuf, String> {
    let lld = Path::new("docs/intent").join(slice).join("lld.md");
    project
        .member_manifest_dirs()
        .into_iter()
        .find(|dir| dir.join(&lld).is_file())
        .ok_or_else(|| format!("no workspace package holds {}: `{slice}` has no crate, so no phase agent can run it", lld.display()))
}

/// The phase's allowed set, as paths relative to the slice's crate; a
/// directory entry allows everything under it.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheSlicesSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteOnlyTheSliceModule,
)]
pub fn allowed_paths(phase: Phase, slice: &str) -> Vec<PathBuf> {
    let module = slice.replace('-', "_");
    let entries: Vec<String> = match phase {
        Phase::One => vec![],
        Phase::Two => vec![format!("src/spec/{module}.rs"), "src/spec/mod.rs".to_string()],
        Phase::Three | Phase::Four => vec![format!("src/{module}.rs"), format!("src/{module}"), "src/lib.rs".to_string()],
        Phase::Five | Phase::Seven => vec![format!("src/{module}.rs"), format!("src/{module}")],
    };
    entries.into_iter().map(PathBuf::from).collect()
}

/// The verdict for a target: normalised against the crate first, then
/// matched against the phase's set.
#[implements(spec::PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy)]
pub fn allowed(phase: Phase, crate_root: &Path, slice: &str, target: &Path) -> Verdict {
    match within_crate(crate_root, target) {
        None => Verdict::Refused(format!("`{}` is outside the slice's crate at {}", target.display(), crate_root.display())),
        Some(relative) => verdict_of(&relative, &allowed_paths(phase, slice)),
    }
}

/// Whether a crate-relative path is in the phase's set.
fn verdict_of(relative: &Path, allowed: &[PathBuf]) -> Verdict {
    if matches_any(relative, allowed) {
        Verdict::Allowed
    } else {
        Verdict::Refused(format!("`{}` is not in this phase's allowed set", relative.display()))
    }
}

/// The target relative to the crate root, or none when it has a parent
/// component or lies outside the crate.
fn within_crate(crate_root: &Path, target: &Path) -> Option<PathBuf> {
    use std::path::Component;
    let clean = target.components().all(|c| !matches!(c, Component::ParentDir));
    (clean && target.is_absolute()).then(|| target.strip_prefix(crate_root).ok().map(Path::to_path_buf)).flatten()
}

/// Whether a relative path is one of, or under, the allowed entries.
fn matches_any(relative: &Path, allowed: &[PathBuf]) -> bool {
    allowed.iter().any(|entry| relative == entry || relative.starts_with(entry))
}

/// The reason an edit is refused: the discipline rows tagged for the phase,
/// from the synced skill, and what the phase may do instead.
#[implements(spec::ARefusedEditQuotesTheDisciplineRow)]
pub fn refusal_reason(project: &Project, phase: Phase, relative: &Path, allowed: &[PathBuf]) -> String {
    let rows = discipline_rows(project, phase).unwrap_or_default();
    let permitted: Vec<String> = allowed.iter().map(|p| format!("`{}`", p.display())).collect();
    format!(
        "`{}` is not Phase {}'s to write.\n\nThe skill's rule for this moment:\n{}\n\nIn this phase you may write only {}. \
         Proceed within those paths, or end with a ```stop block naming the decision this needs.",
        relative.display(),
        number_of(phase),
        rows.join("\n"),
        permitted.join(", ")
    )
}

/// The `discipline.md` rows whose phase column names this phase.
fn discipline_rows(project: &Project, phase: Phase) -> Result<Vec<String>, String> {
    Ok(table_rows_tagged(&read_synced(project, "references/discipline.md")?, &number_of(phase).to_string()))
}

/// A file of the synced skill, read from the project's copy.
pub(super) fn read_synced(project: &Project, relative: &str) -> Result<String, String> {
    let path = project.root()?.join(crate::sync::SKILL_IN_PROJECT).join(relative);
    std::fs::read_to_string(&path).map_err(|e| format!("reading the synced skill file {}: {e}", path.display()))
}

/// The rows of a Markdown table whose first cell contains `tag` as a
/// whole word, without the header and separator rows.
pub(super) fn table_rows_tagged(markdown: &str, tag: &str) -> Vec<String> {
    markdown
        .lines()
        .filter(|line| line.starts_with('|'))
        .skip(2)
        .filter(|line| first_cell(line).split(',').any(|entry| entry.trim() == tag))
        .map(str::to_string)
        .collect()
}

/// The first cell of a Markdown table row.
fn first_cell(row: &str) -> &str {
    row.trim_start_matches('|').split('|').next().unwrap_or("").trim()
}

/// The number a phase is written as in the skill's tables.
pub(super) fn number_of(phase: Phase) -> u8 {
    match phase {
        Phase::One => 1,
        Phase::Two => 2,
        Phase::Three => 3,
        Phase::Four => 4,
        Phase::Five => 5,
        Phase::Seven => 7,
    }
}

/// The slice's execution class, from the crate's target kinds.
#[implements(spec::ACompileTimeSliceIsDisclosed)]
pub fn execution_class(project: &Project, crate_root: &Path) -> Result<ExecutionClass, String> {
    let kinds = project.target_kinds_at(crate_root);
    Ok(kinds
        .iter()
        .find(|kind| kind.as_str() == "proc-macro" || kind.as_str() == "custom-build")
        .map_or(ExecutionClass::Ordinary, |kind| ExecutionClass::CompileTime(kind.clone())))
}

/// Whether the human has accepted a compile-time slice: the file
/// `docs/intent/<slice>/compile-time-accepted` exists in the slice's crate.
#[implements(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
pub fn compile_time_accepted(crate_root: &Path, slice: &str) -> bool {
    crate_root.join("docs/intent").join(slice).join("compile-time-accepted").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    /// Whether a phase refuses a crate-relative target under `/w/app`.
    fn refused(phase: Phase, target: &str) -> bool {
        matches!(allowed(phase, Path::new("/w/app"), "hello", Path::new(target)), Verdict::Refused(_))
    }

    #[test]
    #[validates(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
    fn the_slices_crate_is_the_one_holding_its_lld() {
        let workspace = fixture::workspace();
        let expected = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().expect("dir");
        assert_eq!(slice_crate(&workspace, "phase").expect("found").canonicalize().expect("dir"), expected);
        let macros = slice_crate(&workspace, "macros").expect("found");
        assert!(macros.ends_with("lid-rs-macros"), "{}", macros.display());
        let err = slice_crate(&workspace, "skill").expect_err("a workspace-only slice has no crate");
        assert!(err.contains("skill"), "{err}");
    }

    #[test]
    #[validates(spec::PhaseTwoMayWriteOnlyTheSlicesSpecFiles)]
    fn phase_two_may_write_only_the_slices_spec_files() {
        assert_eq!(allowed_paths(Phase::Two, "hello"), paths(&["src/spec/hello.rs", "src/spec/mod.rs"]));
        assert!(!refused(Phase::Two, "/w/app/src/spec/hello.rs"));
        assert!(refused(Phase::Two, "/w/app/src/hello.rs"));
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot)]
    fn phases_three_and_four_may_write_the_slice_module_and_the_library_root() {
        let expected = paths(&["src/hello.rs", "src/hello", "src/lib.rs"]);
        assert_eq!(allowed_paths(Phase::Three, "hello"), expected);
        assert_eq!(allowed_paths(Phase::Four, "hello"), expected);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot)]
    fn phases_three_and_four_refuse_the_rest() {
        let cases = [
            (Phase::Three, "/w/app/src/hello/policy.rs", false),
            (Phase::Four, "/w/app/src/lib.rs", false),
            (Phase::Three, "/w/app/src/spec/hello.rs", true),
            (Phase::Three, "/w/app/src/hello_extra.rs", true),
        ];
        for (phase, target, expected) in cases {
            assert_eq!(refused(phase, target), expected, "{phase:?} {target}");
        }
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheSliceModule)]
    fn phases_five_and_seven_may_write_only_the_slice_module() {
        let expected = paths(&["src/hello.rs", "src/hello"]);
        assert_eq!(allowed_paths(Phase::Five, "hello"), expected);
        assert_eq!(allowed_paths(Phase::Seven, "hello"), expected);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheSliceModule)]
    fn phases_five_and_seven_refuse_the_rest() {
        let cases = [
            (Phase::Seven, "/w/app/src/hello.rs", false),
            (Phase::Five, "/w/app/src/lib.rs", true),
            (Phase::Seven, "/w/app/docs/intent/hello/lld.md", true),
        ];
        for (phase, target, expected) in cases {
            assert_eq!(refused(phase, target), expected, "{phase:?} {target}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy)]
    fn paths_outside_the_slices_crate_are_refused_before_the_policy() {
        let targets = ["/w/app/src/hello/../../Cargo.toml", "/w/app/src/hello/../spec/hello.rs", "/etc/passwd", "/w/other/src/hello.rs", "src/hello.rs"];
        for target in targets {
            assert!(refused(Phase::Three, target), "{target}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy)]
    fn a_target_is_made_crate_relative_without_parent_components() {
        let root = Path::new("/w/app");
        assert_eq!(within_crate(root, Path::new("/w/app/src/hello.rs")), Some(PathBuf::from("src/hello.rs")));
        assert_eq!(within_crate(root, Path::new("/w/app/src/../Cargo.toml")), None);
    }

    #[test]
    #[validates(spec::ARefusedEditQuotesTheDisciplineRow)]
    fn a_refused_edit_quotes_the_discipline_row() {
        let workspace = fixture::workspace();
        let reason = refusal_reason(&workspace, Phase::Seven, Path::new("docs/intent/phase/lld.md"), &paths(&["src/phase.rs", "src/phase"]));
        let expected = ["Phase 8 event", "src/phase.rs", "```stop"];
        assert!(expected.iter().all(|e| reason.contains(e)), "the row tagged 7 and what the phase may do: {reason}");
    }

    #[test]
    #[validates(spec::ARefusedEditQuotesTheDisciplineRow)]
    fn table_rows_are_selected_by_phase_tag() {
        let rows = table_rows_tagged("| Phase(s) | When | Do this |\n|---|---|---|\n| 6, 7 | a | b |\n| 2, 3 | c | d |\n| 7 | e | f |\n", "7");
        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("| a |") && rows[1].contains("| e |"));
    }

    #[test]
    #[validates(spec::ACompileTimeSliceIsDisclosed)]
    fn a_proc_macro_crate_is_a_compile_time_slice() {
        let workspace = fixture::workspace();
        let macros = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lid-rs-macros");
        assert_eq!(execution_class(&workspace, &macros).expect("class"), ExecutionClass::CompileTime("proc-macro".to_string()));
        let this = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        assert_eq!(execution_class(&workspace, &this).expect("class"), ExecutionClass::Ordinary);
    }

    #[test]
    #[validates(spec::ACompileTimeSliceIsDisclosed)]
    fn a_build_script_makes_a_compile_time_slice() {
        let (dir, _) = fixture::copy("build-script");
        std::fs::write(dir.join("build.rs"), "fn main() {}\n").expect("build.rs");
        let with_build = Project::load_graph_at(&dir.join("Cargo.toml")).expect("metadata");
        assert_eq!(execution_class(&with_build, &dir).expect("class"), ExecutionClass::CompileTime("custom-build".to_string()));
    }

    #[test]
    #[validates(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
    fn acceptance_is_a_file_in_the_slices_intent_directory() {
        let (dir, _) = fixture::copy("acceptance");
        assert!(!compile_time_accepted(&dir, "hello"));
        std::fs::write(dir.join("docs/intent/hello/compile-time-accepted"), "").expect("accept");
        assert!(compile_time_accepted(&dir, "hello"));
    }
}
