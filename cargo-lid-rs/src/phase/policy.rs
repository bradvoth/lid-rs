//! The per-phase path policy (`docs/intent/phase/lld.md` § `hook pre-tool`):
//! which files a phase agent may write, where the slice's crate and its
//! companion are, and what kind of execution editing it entails.

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
    /// `Read`, `Grep`, `Glob`, `LSP`, and the workflow's `StructuredOutput`
    /// — the answer a `schema` forces, which reads and writes nothing:
    /// never refused.
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

/// A policy refusal that is not about one path — the slice's crates
/// themselves cannot be written in any phase — as distinct from a hook that
/// cannot decide, which is an error. The pre-tool hook maps it to a denied
/// edit like any other: tallied, quoting the discipline row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// Why, for the agent: what the policy found and the key it names.
    pub reason: String,
}

/// Whether editing this slice executes the agent's code at compile time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionClass {
    /// Nothing the agent writes runs before Phase 5.
    Ordinary,
    /// The slice's crate has a `proc-macro` or `custom-build` target — named.
    CompileTime(String),
}

/// Which of a slice's crates a path is judged against — one table each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// The slice's own crate, whose manifest directory holds its LLD.
    Own,
    /// The companion a proc-macro crate's manifest names, where the slice's
    /// claims, its citing module, and its fixtures live.
    Companion,
}

/// The crates a phase may write for one slice, resolved once per hook call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceCrates {
    /// The slice, as the branch names it.
    pub slice: String,
    /// The slice's own crate.
    pub own: PathBuf,
    /// Its companion, when the slice's crate is a proc-macro crate.
    pub companion: Option<PathBuf>,
}

impl SliceCrates {
    /// The slice's crates: its own, and the companion its manifest names.
    /// The outer error is a hook that cannot decide — no package holds the
    /// slice's LLD; the inner is the policy's refusal of a proc-macro crate
    /// with no usable companion, which the pre-tool hook denies the edit
    /// with and the stop hook and `phase-check` fail with.
    pub fn resolve(project: &Project, slice: &str) -> Result<Result<Self, Refusal>, String> {
        let own = slice_crate(project, slice)?;
        Ok(companion(project, &own).map(|companion| Self { slice: slice.to_string(), own, companion }))
    }

    /// The crate that holds the slice's claims — where the red run diffs the
    /// spec file: the companion when there is one, else the slice's own.
    #[implements(spec::AProcMacroSlicesClaimsAreHeldByItsCompanion)]
    pub fn claims_crate(&self) -> &Path {
        todo!()
    }

    /// The seats a phase writes in, each with its crate: the slice's own,
    /// then the companion when there is one — the order the stop stages and
    /// the integrity check filters in.
    #[implements(spec::TheStopStagesBothCratesAllowedPaths, spec::IntegrityFiltersAgainstBothCratesAllowedPaths)]
    pub fn seats(&self) -> Vec<(Seat, &Path)> {
        todo!()
    }

    /// The seat a target lies under, with the target relative to that seat's
    /// crate (`within_crate`, tried for the own crate and then the
    /// companion); none when the target has a parent component or lies under
    /// neither, which is refused before any table is consulted.
    #[implements(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy, spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
    pub fn seat_of(&self, target: &Path) -> Option<(Seat, PathBuf)> {
        todo!()
    }
}

/// The kind of a tool by its name.
pub fn kind_of(tool_name: &str) -> ToolKind {
    match tool_name {
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => ToolKind::Edit,
        "Read" | "Grep" | "Glob" | "LSP" | "StructuredOutput" => ToolKind::Observation,
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

/// The `[package.metadata.lid_rs]` key that names a proc-macro crate's
/// companion.
const COMPANION_KEY: &str = "companion";

/// The companion the slice's crate names, when it is a proc-macro crate —
/// all from the metadata document, never by parsing the manifest. None for
/// an ordinary crate, whatever its metadata carries; for a proc-macro crate,
/// the member its key names, or the refusal for a key that names none, a
/// package that is not a member, or one that is itself a proc-macro crate.
#[implements(spec::AnOrdinaryCrateHasNoCompanion)]
pub fn companion(project: &Project, crate_root: &Path) -> Result<Option<PathBuf>, Refusal> {
    if is_proc_macro(project, crate_root) { named_companion(project, crate_root).map(Some) } else { Ok(None) }
}

/// A proc-macro crate's companion: the package its
/// `[package.metadata.lid_rs] companion` setting names
/// (`Project::package_setting_at`), or the refusal for a crate that names
/// none — no phase of such a slice can produce a claim.
#[implements(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames, spec::AProcMacroCrateNamingNoCompanionRefusesEveryEdit)]
fn named_companion(project: &Project, crate_root: &Path) -> Result<PathBuf, Refusal> {
    match project.package_setting_at(crate_root, COMPANION_KEY) {
        None => Err(companion_refusal("names no package")),
        Some(name) => member_companion(project, &name),
    }
}

/// The named companion as a workspace member: its manifest directory
/// (`Project::member_dir_named`), or the refusal for a name that is not a
/// member's.
#[implements(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames, spec::ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit)]
fn member_companion(project: &Project, name: &str) -> Result<PathBuf, Refusal> {
    match project.member_dir_named(name) {
        None => Err(companion_refusal(&format!("names `{name}`, which is not a workspace member"))),
        Some(dir) => usable_companion(project, name, dir),
    }
}

/// The member as a companion that can hold claims: its directory, or the
/// refusal for a member that is itself a proc-macro crate.
#[implements(spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit)]
fn usable_companion(project: &Project, name: &str, dir: PathBuf) -> Result<PathBuf, Refusal> {
    if is_proc_macro(project, &dir) { Err(companion_refusal(&format!("names `{name}`, which is itself a proc-macro crate"))) } else { Ok(dir) }
}

/// Whether the crate at `crate_root` declares a `proc-macro` target
/// (`Project::target_kinds_at`) — the crate that registers no claim and can
/// cite none.
#[implements(spec::AnOrdinaryCrateHasNoCompanion, spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit)]
fn is_proc_macro(project: &Project, crate_root: &Path) -> bool {
    todo!()
}

/// The policy's refusal of a proc-macro crate without a usable companion:
/// names the key, `[package.metadata.lid_rs] companion`, what it does
/// (`what`), and why that refuses every edit — no phase of such a slice can
/// produce a claim.
#[implements(
    spec::AProcMacroCrateNamingNoCompanionRefusesEveryEdit,
    spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit,
    spec::ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit,
)]
fn companion_refusal(what: &str) -> Refusal {
    todo!()
}

/// One phase's allowed set for one seat, as paths relative to that seat's
/// crate; a directory entry allows everything under it. `Seat::Own` is the
/// LLD's first path table, `Seat::Companion` its second.
#[implements(spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
pub fn allowed_paths(phase: Phase, slice: &str, seat: Seat) -> Vec<PathBuf> {
    match seat {
        Seat::Own => own_table(phase, slice),
        Seat::Companion => companion_table(phase, slice),
    }
}

/// The LLD's first path table — what a phase may write in the slice's own
/// crate — one row per phase. Phase 1 is the human's: no agent writes in it.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule,
)]
fn own_table(phase: Phase, slice: &str) -> Vec<PathBuf> {
    match phase {
        Phase::One => Vec::new(),
        Phase::Two => spec_files(slice),
        Phase::Three | Phase::Four => module_and(slice, &["src/lib.rs"]),
        Phase::Five | Phase::Seven => module_and(slice, &[]),
    }
}

/// The LLD's second path table — what a phase may write in the companion
/// — one row per phase: the first table's rows, and at Phases 5 and 7 the
/// `tests/ui` fixtures too. Phase 1 is the human's here as well.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceModuleAndLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceModuleAndUiFixtures,
)]
fn companion_table(phase: Phase, slice: &str) -> Vec<PathBuf> {
    match phase {
        Phase::One => Vec::new(),
        Phase::Two => spec_files(slice),
        Phase::Three | Phase::Four => module_and(slice, &["src/lib.rs"]),
        Phase::Five | Phase::Seven => module_and(slice, &["tests/ui"]),
    }
}

/// Phase 2's row of either table: the slice's spec file (`spec_file_of` —
/// the slice name in snake_case) and `src/spec/mod.rs`.
#[implements(spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles, spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles)]
fn spec_files(slice: &str) -> Vec<PathBuf> {
    todo!()
}

/// The slice's module — `src/<slice>.rs` and the directory `src/<slice>`,
/// the slice name in snake_case as the spec file's is — followed by
/// `extras`, crate-relative: the rows of Phases 3 to 7 in either table.
#[implements(
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceModuleAndLibraryRoot,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceModuleAndUiFixtures,
)]
fn module_and(slice: &str, extras: &[&str]) -> Vec<PathBuf> {
    todo!()
}

/// The verdict for a target: refused before any table when it has a parent
/// component or lies under neither of the slice's crates; otherwise judged
/// by the table of the seat it lies under, relative to that crate.
#[implements(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy, spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
pub fn allowed(phase: Phase, crates: &SliceCrates, target: &Path) -> Verdict {
    match crates.seat_of(target) {
        None => Verdict::Refused(format!("`{}` has a parent component or lies outside the slice's crates", target.display())),
        Some((seat, relative)) => verdict_of(&relative, &allowed_paths(phase, &crates.slice, seat)),
    }
}

/// The phase's allowed paths of both crates — each seat's table relative to
/// its crate, made relative to the workspace root git runs at — which the
/// stop hook filters integrity against and stages.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
pub fn workspace_paths(project: &Project, phase: Phase, crates: &SliceCrates) -> Result<Vec<PathBuf>, String> {
    let root = project.root()?;
    let per_seat: Vec<Vec<PathBuf>> = crates
        .seats()
        .into_iter()
        .map(|(seat, crate_root)| seat_workspace_paths(&root, phase, &crates.slice, seat, crate_root))
        .collect::<Result<_, _>>()?;
    Ok(per_seat.concat())
}

/// One seat's table, workspace-relative: each entry under the crate's own
/// prefix.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
fn seat_workspace_paths(root: &Path, phase: Phase, slice: &str, seat: Seat, crate_root: &Path) -> Result<Vec<PathBuf>, String> {
    let prefix = crate_prefix(root, crate_root)?;
    Ok(allowed_paths(phase, slice, seat).into_iter().map(|entry| prefix.join(entry)).collect())
}

/// A crate's manifest directory relative to the workspace root, or the
/// failure naming both when it is not under the root.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
fn crate_prefix(root: &Path, crate_root: &Path) -> Result<PathBuf, String> {
    todo!()
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

    /// The slice `hello` in an ordinary crate at `/w/app`, with no companion.
    fn own_only() -> SliceCrates {
        SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/app"), companion: None }
    }

    /// The slice `hello` in a proc-macro crate at `/w/mac` whose companion
    /// is `/w/app`.
    fn with_companion() -> SliceCrates {
        SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/mac"), companion: Some(PathBuf::from("/w/app")) }
    }

    /// Whether a phase refuses a target for these crates.
    fn refused_for(crates: &SliceCrates, phase: Phase, target: &str) -> bool {
        matches!(allowed(phase, crates, Path::new(target)), Verdict::Refused(_))
    }

    /// Whether a phase refuses a target under `/w/app`, the one crate.
    fn refused(phase: Phase, target: &str) -> bool {
        refused_for(&own_only(), phase, target)
    }

    /// Asserts whether each target is refused for these crates at this phase.
    fn check_refusals(crates: &SliceCrates, phase: Phase, cases: &[(&str, bool)]) {
        for (target, expected) in cases {
            assert_eq!(refused_for(crates, phase, target), *expected, "{phase:?} {target}");
        }
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
    #[validates(spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles)]
    fn phase_two_may_write_only_the_own_crates_spec_files() {
        assert_eq!(allowed_paths(Phase::Two, "hello", Seat::Own), paths(&["src/spec/hello.rs", "src/spec/mod.rs"]));
        check_refusals(&own_only(), Phase::Two, &[("/w/app/src/spec/hello.rs", false), ("/w/app/src/hello.rs", true)]);
        // With a companion, the own crate is still judged by its own row.
        check_refusals(&with_companion(), Phase::Two, &[("/w/mac/src/spec/mod.rs", false), ("/w/mac/src/lib.rs", true)]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot)]
    fn phases_three_and_four_may_write_the_own_crates_slice_module_and_library_root() {
        let expected = paths(&["src/hello.rs", "src/hello", "src/lib.rs"]);
        assert_eq!(allowed_paths(Phase::Three, "hello", Seat::Own), expected);
        assert_eq!(allowed_paths(Phase::Four, "hello", Seat::Own), expected);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot)]
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
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule)]
    fn phases_five_and_seven_may_write_only_the_own_crates_slice_module() {
        let expected = paths(&["src/hello.rs", "src/hello"]);
        assert_eq!(allowed_paths(Phase::Five, "hello", Seat::Own), expected);
        assert_eq!(allowed_paths(Phase::Seven, "hello", Seat::Own), expected);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule)]
    fn phases_five_and_seven_refuse_the_rest() {
        let cases = [
            (Phase::Seven, "/w/app/src/hello.rs", false),
            (Phase::Five, "/w/app/src/lib.rs", true),
            (Phase::Seven, "/w/app/docs/intent/hello/lld.md", true),
        ];
        for (phase, target, expected) in cases {
            assert_eq!(refused(phase, target), expected, "{phase:?} {target}");
        }
        // The own crate's row admits no fixtures, even when the companion's does.
        check_refusals(&with_companion(), Phase::Five, &[("/w/mac/tests/ui/fail.rs", true), ("/w/mac/src/hello/part.rs", false)]);
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn paths_outside_the_slices_crates_are_refused_before_the_policy() {
        let targets = ["/w/app/src/hello/../../Cargo.toml", "/w/app/src/hello/../spec/hello.rs", "/etc/passwd", "/w/other/src/hello.rs", "src/hello.rs"];
        for target in targets {
            assert!(refused(Phase::Three, target), "{target}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn a_path_under_neither_crate_is_refused_before_either_table() {
        // Under neither crate, or reached through a parent component from
        // either — even one that resolves inside — is refused as outside.
        let targets = ["/w/other/src/hello.rs", "/w/app/src/hello/../../Cargo.toml", "/w/mac/../app/src/hello.rs", "/w/app/../mac/src/hello.rs"];
        for target in targets {
            let Verdict::Refused(why) = allowed(Phase::Three, &with_companion(), Path::new(target)) else { panic!("{target} was allowed") };
            assert!(why.contains("outside the slice's crates"), "{target}: {why}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn a_target_is_seated_and_made_crate_relative_without_parent_components() {
        let crates = with_companion();
        let cases = [
            ("/w/mac/src/hello.rs", Some((Seat::Own, PathBuf::from("src/hello.rs")))),
            ("/w/app/tests/ui/fail.rs", Some((Seat::Companion, PathBuf::from("tests/ui/fail.rs")))),
            ("/w/mac/src/../Cargo.toml", None),
            ("/w/app/../mac/src/hello.rs", None),
        ];
        for (target, expected) in cases {
            assert_eq!(crates.seat_of(Path::new(target)), expected, "{target}");
        }
        assert_eq!(own_only().seat_of(Path::new("/w/other/src/hello.rs")), None, "no companion: under the own crate or nowhere");
    }

    #[test]
    #[validates(spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
    fn a_path_under_the_companion_is_judged_by_the_companions_table() {
        let crates = with_companion();
        // At Phase 5 the two rows differ: fixtures are the companion's to write.
        check_refusals(&crates, Phase::Five, &[("/w/app/tests/ui/fail.rs", false), ("/w/mac/tests/ui/fail.rs", true)]);
        // Each row is relative to its own crate: the union, not one set.
        check_refusals(&crates, Phase::Two, &[("/w/app/src/spec/hello.rs", false), ("/w/app/src/hello.rs", true)]);
        assert_ne!(allowed_paths(Phase::Five, "hello", Seat::Own), allowed_paths(Phase::Five, "hello", Seat::Companion));
    }

    #[test]
    #[validates(spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles)]
    fn phase_two_may_write_only_the_companions_spec_files() {
        assert_eq!(allowed_paths(Phase::Two, "hello", Seat::Companion), paths(&["src/spec/hello.rs", "src/spec/mod.rs"]));
        assert_eq!(allowed_paths(Phase::Two, "phase-gate", Seat::Companion), paths(&["src/spec/phase_gate.rs", "src/spec/mod.rs"]));
        let cases = [("/w/app/src/spec/mod.rs", false), ("/w/app/src/lib.rs", true), ("/w/app/tests/ui/fail.rs", true)];
        check_refusals(&with_companion(), Phase::Two, &cases);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheCompanionsSliceModuleAndLibraryRoot)]
    fn phases_three_and_four_may_write_the_companions_slice_module_and_library_root() {
        let expected = paths(&["src/hello.rs", "src/hello", "src/lib.rs"]);
        assert_eq!(allowed_paths(Phase::Three, "hello", Seat::Companion), expected);
        assert_eq!(allowed_paths(Phase::Four, "hello", Seat::Companion), expected);
        // `src/lib.rs` is where the hand-authored edges go; fixtures and claims are not this row's.
        check_refusals(&with_companion(), Phase::Four, &[("/w/app/src/lib.rs", false)]);
        check_refusals(&with_companion(), Phase::Three, &[("/w/app/tests/ui/fail.rs", true), ("/w/app/src/spec/hello.rs", true)]);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceModuleAndUiFixtures)]
    fn phases_five_and_seven_may_write_the_companions_slice_module_and_ui_fixtures() {
        let expected = paths(&["src/hello.rs", "src/hello", "tests/ui"]);
        assert_eq!(allowed_paths(Phase::Five, "hello", Seat::Companion), expected);
        assert_eq!(allowed_paths(Phase::Seven, "hello", Seat::Companion), expected);
        // The fixtures are `tests/ui` alone; the library root is not this row's.
        check_refusals(&with_companion(), Phase::Seven, &[("/w/app/tests/ui/fail.rs", false), ("/w/app/tests/other.rs", true)]);
        check_refusals(&with_companion(), Phase::Five, &[("/w/app/src/hello/part.rs", false), ("/w/app/src/lib.rs", true)]);
    }

    #[test]
    #[validates(spec::AProcMacroSlicesClaimsAreHeldByItsCompanion)]
    fn a_proc_macro_slices_claims_are_held_by_its_companion() {
        assert_eq!(with_companion().claims_crate(), Path::new("/w/app"), "the companion, not the macro crate");
        let ordinary = SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/lib"), companion: None };
        assert_eq!(ordinary.claims_crate(), Path::new("/w/lib"), "an ordinary crate holds its own");
    }

    #[test]
    #[validates(spec::AnOrdinaryCrateHasNoCompanion)]
    fn an_ordinary_crate_has_no_companion() {
        let (dir, project) = fixture::two_member_workspace("companion-ordinary", &fixture::companion_setting("app"), "");
        assert_eq!(companion(&project, &dir.join("owner")), Ok(None), "the setting counts only for a proc-macro crate");
        let crates = SliceCrates::resolve(&project, "m").expect("the LLD is held").expect("no refusal");
        assert_eq!(crates, SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: None });
    }

    #[test]
    #[validates(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames)]
    fn the_companion_is_the_member_the_proc_macro_crates_metadata_names() {
        let owner = format!("{}{}", fixture::PROC_MACRO_LIB, fixture::companion_setting("app"));
        let (dir, project) = fixture::two_member_workspace("companion-named", &owner, "");
        assert_eq!(companion(&project, &dir.join("owner")), Ok(Some(dir.join("app"))));
        let crates = SliceCrates::resolve(&project, "m").expect("the LLD is held").expect("no refusal");
        assert_eq!(crates, SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: Some(dir.join("app")) });
    }

    #[test]
    #[validates(spec::TheStopStagesBothCratesAllowedPaths)]
    fn the_stop_stages_both_crates_allowed_paths() {
        let (dir, project) = fixture::two_member_workspace("stage-both", "", "");
        let crates = SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: Some(dir.join("app")) };
        let both = workspace_paths(&project, Phase::Five, &crates).expect("both under the root");
        assert_eq!(both, paths(&["owner/src/m.rs", "owner/src/m", "app/src/m.rs", "app/src/m", "app/tests/ui"]));
        // What the commit stages is the changes within that set, and nothing else.
        std::fs::write(dir.join("owner/src/m.rs"), "//! m\n").expect("write");
        std::fs::create_dir_all(dir.join("app/tests/ui")).expect("dir");
        std::fs::write(dir.join("app/tests/ui/fail.rs"), "").expect("write");
        std::fs::write(dir.join("app/src/lib.rs"), "// changed\n").expect("write");
        assert_eq!(crate::phase::integrity::changed_within(&project, &both).expect("status"), paths(&["app/tests/ui/fail.rs", "owner/src/m.rs"]));
        let own = SliceCrates { companion: None, ..crates };
        assert_eq!(workspace_paths(&project, Phase::Five, &own).expect("under the root"), paths(&["owner/src/m.rs", "owner/src/m"]));
    }

    #[test]
    #[validates(spec::TheStopStagesBothCratesAllowedPaths)]
    fn a_crate_outside_the_root_cannot_be_staged() {
        assert_eq!(crate_prefix(Path::new("/w"), Path::new("/w/app")).expect("under the root"), PathBuf::from("app"));
        let err = crate_prefix(Path::new("/w"), Path::new("/elsewhere/app")).expect_err("not under the root");
        assert!(err.contains("/elsewhere/app") && err.contains("/w"), "names both: {err}");
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
