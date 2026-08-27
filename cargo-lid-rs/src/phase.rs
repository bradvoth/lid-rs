
use std::path::PathBuf;

use lid_rs::implements;

pub mod ending;
pub mod integrity;
pub mod policy;
pub mod tally;

use ending::{Ending, ending_of, refusal_for, stage_and_commit, subject_matches};
use integrity::{outside_policy_clean, synced_artifacts_match};
use policy::{ToolKind, Verdict, allowed_paths, execution_class, kind_of, slice_crate};
use tally::{Event, Tally};

use crate::mapping::EdgeRecord;
use crate::mutants::{self, Registry, SpecRecord, dump_registry};
use crate::project::Project;
use crate::spec;
use crate::sync;

/// The phases that have a check, as the messages name them.
const CHECKED_PHASES: &str = "1, 2, 3, 4, 5, 7";

/// Usage for `phase-check`.
const PHASE_CHECK_USAGE: &str = "usage: cargo lid-rs phase-check <n> [--slice <name>]";

/// Usage for `hook`.
const HOOK_USAGE: &str = "usage: cargo lid-rs hook <pre-tool <n> | post-edit <n> | stop <n>>";

/// A phase with a commit of its own and a check attached — the closed set.
/// Phases 0, 6, and 8 have no commit (skill, working state) and no check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::PhasesWithoutACommitHaveNoCheck)]
pub enum Phase {
    /// The LLD: docs and doctests.
    One,
    /// The claims: build and lint.
    Two,
    /// The layer-0 skeleton: type-checks.
    Three,
    /// Every further layer: type-checks.
    Four,
    /// The validations: red against `todo!()`.
    Five,
    /// The gate.
    Seven,
}

impl TryFrom<u8> for Phase {
    type Error = String;

    /// The phase for a number, or an error naming the phases that have a check.
    #[implements(spec::PhasesWithoutACommitHaveNoCheck)]
    fn try_from(n: u8) -> Result<Self, String> {
        match n {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            7 => Ok(Self::Seven),
            other => Err(format!("phase {other} has no check of its own; the phases with one are {CHECKED_PHASES}")),
        }
    }
}

/// One step of a phase's check — the closed set of things a check runs,
/// which is README §4.5's list plus the red run.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::PhaseSevenRunsTheGateInOrder)]
pub enum Step {
    /// `cargo check --all-targets`.
    Check,
    /// `cargo clippy --all-targets -- -D warnings`.
    Clippy,
    /// `cargo doc --no-deps` with broken intra-doc links denied.
    Doc,
    /// `cargo test --doc`.
    DocTests,
    /// `cargo test --lib`.
    LibTests,
    /// `cargo package -p <name> --allow-dirty` for one publishing package.
    Package(String),
    /// `sync --check`, through the library.
    SyncCheck,
    /// `mutants`, through the library.
    Mutants,
    /// The phase 5 red run over the slice's validations.
    Red,
}

/// What a commit subject's tag says.
#[derive(Debug, PartialEq, Eq)]
pub enum Tag {
    /// No `phase N:` prefix: not a phase commit.
    Untagged,
    /// A phase with a check.
    Checked(Phase),
    /// A `phase N:` prefix naming a phase without a check.
    Unchecked(u8),
}

/// One validation's outcome at phase 5.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// The claim the test cites.
    pub claim: String,
    /// The test's libtest-relative path.
    pub test: String,
    /// Whether the test passed — which, at phase 5, is the failure.
    pub passed: bool,
}

/// One package's registry, dumped from its own test binary.
struct PackageRegistry {
    /// The package name `cargo test -p` takes.
    package: String,
    /// Its specs and edges.
    registry: Registry,
}

/// `phase-check <n> [--slice <name>]`: parse, locate the project, check.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn run(args: &[String]) -> Result<(), String> {
    let (phase, given) = parse_args(args)?;
    let project = Project::load_graph()?;
    let slice = resolve_slice(&project, given)?;
    check(&project, phase, slice.as_deref())
}

/// What a hook reads from the JSON Claude Code passes on stdin — the
/// boundary type; everything past it takes plain values.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HookInput {
    /// The subagent's id, the key its tally is filed under.
    pub agent_id: String,
    /// The tool being called (`PreToolUse`, `PostToolUse`).
    pub tool_name: Option<String>,
    /// The file the tool targets, when it targets one.
    pub tool_path: Option<PathBuf>,
    /// The agent's final message (`Stop`).
    pub last_message: String,
    /// True when Claude Code is already continuing because a stop was refused.
    pub stop_hook_active: bool,
}

/// A hook's answer, rendered to stdout at the boundary in the form Claude
/// Code reads for that event.
#[derive(Debug, PartialEq, Eq)]
pub enum HookVerdict {
    /// The call, or the stop, proceeds; nothing is printed.
    Allow,
    /// The call is denied, or the agent is kept running, with this reason.
    Refuse(String),
    /// The call proceeded; this text is handed back as additional context.
    Context(String),
}

/// `hook <pre-tool <n> | post-edit <n> | stop <n>>`: one dispatch over the
/// hook kind; each reads Claude Code's JSON from stdin and prints its
/// verdict.
pub fn hook(args: &[String]) -> Result<(), String> {
    let (kind, phase) = parse_hook_args(args)?;
    let project = Project::load_graph()?;
    let input = HookInput::from_json(&read_stdin()?)?;
    let verdict = match kind {
        HookKind::PreTool => hook_pre_tool(&project, phase, &input)?,
        HookKind::PostEdit => hook_post_edit(&project, &input)?,
        HookKind::Stop => hook_stop(&project, phase, &input)?,
    };
    print!("{}", render(kind, &verdict));
    Ok(())
}

/// The three hooks a phase agent declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookKind {
    /// Before every tool call: the path policy and the tally.
    PreTool,
    /// After every edit: clippy as context.
    PostEdit,
    /// When the agent ends: the check, then the commit.
    Stop,
}

/// `<kind> <n>`: which hook, for which phase.
fn parse_hook_args(args: &[String]) -> Result<(HookKind, Phase), String> {
    todo!()
}

/// The hook's stdin, whole.
fn read_stdin() -> Result<String, String> {
    todo!()
}

impl HookInput {
    /// The fields the hooks use, from Claude Code's hook JSON; fields absent
    /// for an event are empty.
    pub fn from_json(json: &str) -> Result<Self, String> {
        todo!()
    }
}

/// Renders a verdict the way Claude Code reads it for the event: a deny
/// decision for `PreToolUse`, additional context for `PostToolUse`, a
/// block decision for `Stop`; nothing when allowed.
fn render(kind: HookKind, verdict: &HookVerdict) -> String {
    todo!()
}

/// `hook pre-tool <n>`: an editing tool's target must be in the phase's
/// allowed set; every call is tallied.
#[implements(spec::ReadsAreNeverRefused, spec::EveryToolCallIsTallied)]
fn hook_pre_tool(project: &Project, phase: Phase, input: &HookInput) -> Result<HookVerdict, String> {
    todo!()
}

/// The policy verdict for one edit, as the hook renders it.
#[implements(spec::ARefusedEditQuotesTheDisciplineRow)]
fn edit_verdict(project: &Project, phase: Phase, input: &HookInput) -> Result<HookVerdict, String> {
    todo!()
}

/// `hook post-edit <n>`: clippy, its output as context.
#[implements(spec::EveryEditIsFollowedByClippy)]
fn hook_post_edit(project: &Project, input: &HookInput) -> Result<HookVerdict, String> {
    todo!()
}

/// `hook stop <n>`: the final message's ending decides — a `stop` block
/// ends the phase uncommitted; a `commit` block runs the check and, with
/// integrity intact, commits the phase's paths.
#[implements(spec::AStopBlockEndsThePhaseWithoutACommit, spec::ACommitBlockRunsThePhasesCheck)]
fn hook_stop(project: &Project, phase: Phase, input: &HookInput) -> Result<HookVerdict, String> {
    todo!()
}

/// The commit path of the stop hook: integrity, the check, integrity again,
/// then staging and committing; each failure is the refusal it names.
#[implements(
    spec::SyncedArtifactsMustMatchAtTheStop,
    spec::ChangesOutsideThePolicyRefuseTheStop,
    spec::ARefusalCarriesTheOutputTheRuleAndThePermittedMoves,
)]
fn commit_phase(project: &Project, phase: Phase, input: &HookInput, message: &str) -> Result<HookVerdict, String> {
    todo!()
}

/// Clippy over the workspace, captured: the output, or "clean".
fn clippy_output(project: &Project) -> Result<String, String> {
    todo!()
}

/// Runs a phase's check: its plan, executed in order.
pub fn check(project: &Project, phase: Phase, slice: Option<&str>) -> Result<(), String> {
    execute(project, slice, &plan(phase, &project.publishing_members()))
}

/// A phase's steps, in order, as data. `publishing` names the packages
/// `cargo package` runs for at phase 7.
#[implements(
    spec::PhaseOneChecksTheDocs,
    spec::PhaseTwoChecksTheClaimsBuildAndLint,
    spec::PhasesThreeAndFourCheckTheSkeletonTypeChecks,
    spec::PhaseSevenRunsTheGateInOrder,
)]
pub fn plan(phase: Phase, publishing: &[String]) -> Vec<Step> {
    match phase {
        Phase::One => vec![Step::Doc, Step::DocTests],
        Phase::Two => vec![Step::Check, Step::Clippy],
        Phase::Three | Phase::Four => vec![Step::Check],
        Phase::Five => vec![Step::Red],
        Phase::Seven => gate(publishing),
    }
}

/// README §4.5 in order: the five cargo steps, a package per publishing
/// member, then the two library steps.
#[implements(spec::PhaseSevenRunsTheGateInOrder)]
fn gate(publishing: &[String]) -> Vec<Step> {
    [Step::Check, Step::Clippy, Step::Doc, Step::DocTests, Step::LibTests]
        .into_iter()
        .chain(publishing.iter().cloned().map(Step::Package))
        .chain([Step::SyncCheck, Step::Mutants])
        .collect()
}

/// Runs steps in order against the project; the first failure is the
/// result, naming its step, and no later step runs.
pub fn execute(project: &Project, slice: Option<&str>, steps: &[Step]) -> Result<(), String> {
    execute_with(steps, |step| run_step(project, slice, step))
}

/// `execute` over any runner: the first failure is the result and no later
/// step runs.
#[implements(spec::ACheckStopsAtTheFirstFailingStep)]
fn execute_with(steps: &[Step], mut run: impl FnMut(&Step) -> Result<(), String>) -> Result<(), String> {
    steps.iter().try_for_each(|step| run(step).map_err(|e| format!("{step:?} failed: {e}")))
}

/// Runs one step: one dispatch over the closed set. The red run needs a
/// slice; without one it fails naming the branch convention.
#[implements(spec::PhaseSevenRunsTheGateInOrder, spec::TheSliceComesFromTheBranchName)]
fn run_step(project: &Project, slice: Option<&str>, step: &Step) -> Result<(), String> {
    match step {
        Step::Check => cargo_step(project, &["check", "--all-targets"], &[]),
        Step::Clippy => cargo_step(project, &["clippy", "--all-targets", "--", "-D", "warnings"], &[]),
        Step::Doc => cargo_step(project, &["doc", "--no-deps"], &[("RUSTDOCFLAGS", "-D rustdoc::broken_intra_doc_links")]),
        Step::DocTests => cargo_step(project, &["test", "--doc"], &[]),
        Step::LibTests => cargo_step(project, &["test", "--lib"], &[]),
        Step::Package(name) => cargo_step(project, &["package", "-p", name, "--allow-dirty"], &[]),
        Step::SyncCheck => sync::check(project),
        Step::Mutants => mutants::run(&[]),
        Step::Red => check_red(project, slice.ok_or(NO_SLICE)?),
    }
}

/// The phase 5 failure when no slice is known.
const NO_SLICE: &str = "phase 5 needs a slice: the branch is not `lld/<slice>`, and no --slice <name> was given";

/// The phase 5 red run: the slice's claims, their validations, each run
/// alone; fails naming every unvalidated claim and every green test.
#[implements(spec::AGreenValidationFailsTheRedCheck)]
pub fn check_red(project: &Project, slice: &str) -> Result<(), String> {
    let registries = package_registries(project)?;
    let claims = require_claims(all_slice_claims(&registries, slice), slice)?;
    let outcomes = run_validations(project, &registries, &claims)?;
    red_verdict(&unvalidated(&claims, &outcomes), &outcomes)
}

/// `phase-check`'s arguments: the phase number, then optionally
/// `--slice <name>`; any other flag is rejected by name.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn parse_args(args: &[String]) -> Result<(Phase, Option<String>), String> {
    let (first, rest) = args.split_first().ok_or(PHASE_CHECK_USAGE)?;
    let phase = first
        .parse::<u8>()
        .map_err(|_| format!("`{first}` is not a phase number\n{PHASE_CHECK_USAGE}"))
        .and_then(Phase::try_from)?;
    Ok((phase, slice_flag(rest)?))
}

/// The `--slice <name>` flag, if given; anything else is rejected by name.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn slice_flag(rest: &[String]) -> Result<Option<String>, String> {
    match rest {
        [] => Ok(None),
        [flag, name] if flag == "--slice" => Ok(Some(name.clone())),
        [flag] if flag == "--slice" => Err("--slice requires a name".to_string()),
        [flag, ..] => Err(format!("unknown flag `{flag}` for phase-check; the only flag is --slice <name>")),
    }
}

/// The slice given, or the current branch's, or none — a detached `HEAD`
/// is on no branch and so names no slice.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn resolve_slice(project: &Project, given: Option<String>) -> Result<Option<String>, String> {
    Ok(given.or(current_branch(project)?.and_then(|branch| slice_of_branch(&branch))))
}

/// The slice an `lld/<slice>` branch is for, or none for any other name.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn slice_of_branch(branch: &str) -> Option<String> {
    branch.strip_prefix("lld/").map(str::to_string)
}

/// The repository's current branch name, or none on a detached `HEAD`
/// (`symbolic-ref` exits non-zero there, quietly).
fn current_branch(project: &Project) -> Result<Option<String>, String> {
    let output = project
        .git()?
        .args(["symbolic-ref", "--quiet", "--short", "HEAD"])
        .output()
        .map_err(|e| format!("running git symbolic-ref: {e}"))?;
    Ok(output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string()))
}

/// What the subject's `phase N:` prefix, if any, names.
pub fn tag_of(subject: &str) -> Tag {
    match phase_number(subject) {
        None => Tag::Untagged,
        Some(n) => Phase::try_from(n).map_or(Tag::Unchecked(n), Tag::Checked),
    }
}

/// The `N` of a `phase N:` prefix.
fn phase_number(subject: &str) -> Option<u8> {
    subject.strip_prefix("phase ")?.split_once(':')?.0.trim().parse().ok()
}

/// Runs one cargo command with its output inherited, so the check's output
/// is what the caller prints.
fn cargo_step(project: &Project, args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    let status = project
        .cargo()?
        .args(args)
        .envs(env.iter().copied())
        .status()
        .map_err(|e| format!("running cargo {}: {e}", args.join(" ")))?;
    status.success().then_some(()).ok_or_else(|| format!("cargo {} exited with {status}", args.join(" ")))
}

/// Every library member's registry, each from its own test binary.
fn package_registries(project: &Project) -> Result<Vec<PackageRegistry>, String> {
    project
        .library_members()
        .into_iter()
        .map(|package| dump_registry(project, &package).map(|registry| PackageRegistry { package, registry }))
        .collect()
}

/// The slice's claims across every package.
fn all_slice_claims(registries: &[PackageRegistry], slice: &str) -> Vec<String> {
    registries.iter().flat_map(|r| slice_claims(&r.registry.specs, slice)).collect()
}

/// The file a slice's claims register from: `src/spec/<slice>.rs`, the
/// slice name in snake_case.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn spec_file_of(slice: &str) -> String {
    format!("src/spec/{}.rs", slice.replace('-', "_"))
}

/// The names of the specs registered from the slice's spec file.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn slice_claims(specs: &[SpecRecord], slice: &str) -> Vec<String> {
    let file = spec_file_of(slice);
    let nested = format!("/{file}");
    specs
        .iter()
        .filter(|s| s.file == file || s.file.ends_with(&nested))
        .map(|s| s.name.clone())
        .collect()
}

/// The claims, or the failure for a slice that registers none.
#[implements(spec::ASliceWithNoClaimsFailsTheRedCheck)]
fn require_claims(claims: Vec<String>, slice: &str) -> Result<Vec<String>, String> {
    if claims.is_empty() {
        Err(format!("no claims for slice `{slice}`: nothing registers from {}", spec_file_of(slice)))
    } else {
        Ok(claims)
    }
}

/// `(claim, test)` for every validation edge on the claims, the test's item
/// path made libtest-relative.
#[implements(spec::EachValidationRunsAloneByExactName)]
pub fn claim_validations(validations: &[EdgeRecord], claims: &[String]) -> Vec<(String, String)> {
    validations
        .iter()
        .filter(|e| claims.contains(&e.spec))
        .filter_map(|e| e.item.split_once("::").map(|(_, rest)| (e.spec.clone(), rest.to_string())))
        .collect()
}

/// Every validation of the claims, run alone in its own package.
fn run_validations(project: &Project, registries: &[PackageRegistry], claims: &[String]) -> Result<Vec<Outcome>, String> {
    registries
        .iter()
        .flat_map(|r| claim_validations(&r.registry.validations, claims).into_iter().map(move |v| (r.package.as_str(), v)))
        .map(|(package, (claim, test))| run_test(project, package, &test).map(|passed| Outcome { claim, test, passed }))
        .collect()
}

/// Runs one test alone, quietly; true when it passes.
#[implements(spec::EachValidationRunsAloneByExactName)]
fn run_test(project: &Project, package: &str, test: &str) -> Result<bool, String> {
    let status = project
        .cargo()?
        .args(["test", "-p", package, "--lib", "--", "--exact", test])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("running cargo test for {test}: {e}"))?;
    Ok(status.success())
}

/// The claims no outcome validates.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses)]
pub fn unvalidated(claims: &[String], outcomes: &[Outcome]) -> Vec<String> {
    claims.iter().filter(|c| !outcomes.iter().any(|o| o.claim == **c)).cloned().collect()
}

/// The verdict: pass when nothing is unvalidated and nothing is green;
/// otherwise the failure naming every unvalidated claim and green test.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses, spec::AGreenValidationFailsTheRedCheck)]
pub fn red_verdict(unvalidated: &[String], outcomes: &[Outcome]) -> Result<(), String> {
    let problems: Vec<String> = unvalidated
        .iter()
        .map(|c| format!("claim {c} has no #[validates] test"))
        .chain(outcomes.iter().filter(|o| o.passed).map(|o| format!("test {} passes before implementation exists", o.test)))
        .collect();
    problems.is_empty().then_some(()).ok_or_else(|| format!("phase 5 is not red:\n  {}", problems.join("\n  ")))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use lid_rs::validates;

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_string()).collect()
    }

    /// This crate as a project (`--no-deps` metadata suffices for root,
    /// target, and git).
    fn this_project() -> Project {
        Project::load_at(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("cargo metadata")
    }

    /// A fresh scratch git repository.
    fn scratch_repo(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-phase-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        git(&dir, &["init", "-q"]);
        dir
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git").args(args).current_dir(dir).status().expect("git");
        assert!(status.success(), "git {args:?} in {}", dir.display());
    }

    fn spec_record(name: &str, file: &str) -> SpecRecord {
        SpecRecord { name: name.to_string(), file: file.to_string() }
    }

    fn edge(spec: &str, item: &str) -> EdgeRecord {
        EdgeRecord { spec: spec.to_string(), item: item.to_string(), file: "x.rs".to_string() }
    }

    fn red(claim: &str, test: &str) -> Outcome {
        Outcome { claim: claim.to_string(), test: test.to_string(), passed: false }
    }

    fn green(claim: &str, test: &str) -> Outcome {
        Outcome { claim: claim.to_string(), test: test.to_string(), passed: true }
    }

    #[test]
    #[validates(spec::PhasesWithoutACommitHaveNoCheck)]
    fn phases_without_a_commit_have_no_check() {
        for n in [0, 6, 8, 9, 42] {
            let err = Phase::try_from(n).expect_err("no check for that phase");
            assert!(err.contains("1, 2, 3, 4, 5, 7"), "{n}: {err}");
        }
        let checked: Vec<Phase> = [1u8, 2, 3, 4, 5, 7].iter().map(|&n| Phase::try_from(n).expect("has a check")).collect();
        assert_eq!(checked, [Phase::One, Phase::Two, Phase::Three, Phase::Four, Phase::Five, Phase::Seven]);
    }

    #[test]
    #[validates(spec::PhaseOneChecksTheDocs)]
    fn phase_one_checks_the_docs() {
        assert_eq!(plan(Phase::One, &[]), [Step::Doc, Step::DocTests]);
    }

    #[test]
    #[validates(spec::PhaseTwoChecksTheClaimsBuildAndLint)]
    fn phase_two_checks_the_claims_build_and_lint() {
        assert_eq!(plan(Phase::Two, &[]), [Step::Check, Step::Clippy]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourCheckTheSkeletonTypeChecks)]
    fn phases_three_and_four_check_the_skeleton_type_checks() {
        assert_eq!(plan(Phase::Three, &[]), [Step::Check]);
        assert_eq!(plan(Phase::Four, &[]), [Step::Check]);
    }

    #[test]
    #[validates(spec::PhaseSevenRunsTheGateInOrder)]
    fn phase_seven_runs_the_gate_in_order() {
        let expected = [
            Step::Check,
            Step::Clippy,
            Step::Doc,
            Step::DocTests,
            Step::LibTests,
            Step::Package("a".to_string()),
            Step::Package("b".to_string()),
            Step::SyncCheck,
            Step::Mutants,
        ];
        assert_eq!(plan(Phase::Seven, &strings(&["a", "b"])), expected);
        assert_eq!(plan(Phase::Five, &[]), [Step::Red]);
    }

    #[test]
    #[validates(spec::PhaseSevenRunsTheGateInOrder)]
    fn the_gate_packages_the_workspace_members_that_publish() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml");
        let mut members = Project::load_at(&workspace).expect("cargo metadata").publishing_members();
        members.sort();
        // xtask is `publish = false`; the three published crates remain.
        assert_eq!(members, strings(&["cargo-lid-rs", "lid-rs", "lid-rs-macros"]));
    }

    #[test]
    #[validates(spec::ACheckStopsAtTheFirstFailingStep)]
    fn a_check_stops_at_the_first_failing_step() {
        let steps = [Step::Check, Step::Clippy, Step::Doc];
        let mut ran = Vec::new();
        let err = execute_with(&steps, |step| {
            ran.push(step.clone());
            if *step == Step::Clippy { Err("lint fired".to_string()) } else { Ok(()) }
        })
        .expect_err("the failing step fails the check");
        assert!(err.contains("Clippy") && err.contains("lint fired"), "{err}");
        assert_eq!(ran, [Step::Check, Step::Clippy], "no step after the failure runs");
        let mut all = Vec::new();
        execute_with(&steps, |step| {
            all.push(step.clone());
            Ok(())
        })
        .expect("every step passing passes the check");
        assert_eq!(all, steps);
    }

    #[test]
    #[validates(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
    fn a_slices_claims_are_the_specs_in_its_spec_file() {
        assert_eq!(spec_file_of("phase-gate"), "src/spec/phase_gate.rs");
        let specs = [
            spec_record("A", "cargo-lid-rs/src/spec/phase_gate.rs"),
            spec_record("B", "cargo-lid-rs/src/spec/sync.rs"),
            spec_record("C", "cargo-lid-rs/src/spec/phase_gate.rs"),
        ];
        assert_eq!(slice_claims(&specs, "phase-gate"), strings(&["A", "C"]));
        assert!(slice_claims(&specs, "init").is_empty());
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn the_slice_comes_from_the_branch_name() {
        assert_eq!(slice_of_branch("lld/phase-gate"), Some("phase-gate".to_string()));
        assert_eq!(slice_of_branch("main"), None);
        assert_eq!(slice_of_branch("feature/lld/x"), None);
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn phase_check_takes_the_phase_and_an_optional_slice() {
        assert_eq!(parse_args(&strings(&["5"])).expect("parses"), (Phase::Five, None));
        let with_slice = parse_args(&strings(&["5", "--slice", "login"])).expect("parses");
        assert_eq!(with_slice, (Phase::Five, Some("login".to_string())));
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn phase_check_rejects_other_flags_by_name() {
        let err = parse_args(&strings(&["3", "--bogus"])).expect_err("unknown flag");
        assert!(err.contains("--bogus"), "{err}");
        let with_value = parse_args(&strings(&["3", "--bogus", "x"])).expect_err("unknown flag with a value");
        assert!(with_value.contains("--bogus"), "{with_value}");
        let bare = parse_args(&strings(&["5", "--slice"])).expect_err("--slice needs a name");
        assert!(bare.contains("requires a name"), "{bare}");
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn phase_check_requires_a_phase_number() {
        assert!(parse_args(&strings(&["x"])).is_err(), "a non-number is not a phase");
        assert!(parse_args(&[]).is_err(), "the phase is required");
        assert!(run(&strings(&["3", "--bogus"])).is_err(), "the subcommand fails the same way");
    }

    /// A project whose repository is the scratch repo at `root`.
    fn project_in_repo(root: &Path) -> Project {
        Project::from_json(&format!(
            r#"{{"workspace_root":"{}","target_directory":"{}","packages":[]}}"#,
            root.display(),
            root.join("target").display()
        ))
        .expect("parses")
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn the_slice_is_the_given_one_or_the_branchs() {
        let root = scratch_repo("resolve-slice");
        let project = project_in_repo(&root);
        assert_eq!(resolve_slice(&project, None).expect("git"), None, "the default branch is no slice");
        git(&root, &["checkout", "-q", "-b", "lld/demo"]);
        assert_eq!(resolve_slice(&project, None).expect("git"), Some("demo".to_string()));
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn a_detached_head_names_no_slice() {
        let root = scratch_repo("resolve-slice-detached");
        let project = project_in_repo(&root);
        git(&root, &["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-q", "--allow-empty", "-m", "x"]);
        git(&root, &["checkout", "-q", "--detach"]);
        assert_eq!(resolve_slice(&project, None).expect("git"), None, "detached HEAD: no branch, no slice");
        assert_eq!(resolve_slice(&project, Some("given".to_string())).expect("git"), Some("given".to_string()));
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn a_given_slice_wins_over_the_branchs() {
        let root = scratch_repo("resolve-slice-given");
        let project = project_in_repo(&root);
        git(&root, &["checkout", "-q", "-b", "lld/demo"]);
        assert_eq!(resolve_slice(&project, Some("given".to_string())).expect("git"), Some("given".to_string()));
    }

    #[test]
    #[validates(spec::TheSliceComesFromTheBranchName)]
    fn the_red_step_needs_a_slice() {
        let err = run_step(&this_project(), None, &Step::Red).expect_err("no slice, no red run");
        assert!(err.contains("lld/<slice>") && err.contains("--slice"), "{err}");
    }

    #[test]
    #[validates(spec::ASliceWithNoClaimsFailsTheRedCheck)]
    fn a_slice_with_no_claims_fails_the_red_check() {
        let err = require_claims(vec![], "login").expect_err("no claims is a failure");
        assert!(err.contains("login") && err.contains("no claims"), "{err}");
        assert_eq!(require_claims(strings(&["A"]), "login").expect("claims pass through"), strings(&["A"]));
    }

    #[test]
    #[validates(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses)]
    fn every_claim_needs_a_validation_before_phase_five_passes() {
        let outcomes = [red("A", "t::a")];
        assert_eq!(unvalidated(&strings(&["A", "B"]), &outcomes), strings(&["B"]));
        let err = red_verdict(&strings(&["B"]), &outcomes).expect_err("an unvalidated claim fails");
        assert!(err.contains('B'), "{err}");
    }

    #[test]
    #[validates(spec::EachValidationRunsAloneByExactName)]
    fn each_validation_runs_alone_by_exact_name() {
        let edges = [
            edge("A", "cargo_lid_rs::phase::tests::a_test"),
            edge("Z", "cargo_lid_rs::other::tests::z_test"),
            edge("B", "cargo_lid_rs::phase::tests::b_test"),
        ];
        assert_eq!(
            claim_validations(&edges, &strings(&["A", "B"])),
            [
                ("A".to_string(), "phase::tests::a_test".to_string()),
                ("B".to_string(), "phase::tests::b_test".to_string()),
            ]
        );
        // A real run, alone: this test binary's own passing probe passes; a
        // name libtest matches nothing for runs zero tests and so "passes",
        // which at phase 5 is the failing direction.
        let project = this_project();
        assert!(run_test(&project, "cargo-lid-rs", "phase::tests::probe_that_passes").expect("cargo test runs"));
        assert!(
            !run_test(&project, "cargo-lid-rs", "phase::tests::probe_that_fails").expect("cargo test runs"),
            "a failing test is reported as such"
        );
    }

    /// A known-green test for `run_test` to run alone.
    #[test]
    fn probe_that_passes() {}

    /// A known-red test for `run_test` to run alone: it fails only when run
    /// by itself (`--exact`), which the suite never does.
    #[test]
    fn probe_that_fails() {
        let alone = std::env::args().any(|a| a == "--exact");
        assert!(!alone, "the probe fails when run alone, as run_test does");
    }

    #[test]
    #[validates(spec::AGreenValidationFailsTheRedCheck)]
    fn the_red_run_fails_on_an_implemented_slice() {
        // The sync slice is implemented: every one of its validations passes,
        // so its red run must fail and name one of them. (Not this slice: its
        // validations include this test, and the run would recurse.)
        let err = check_red(&this_project(), "sync").expect_err("green validations fail the red run");
        assert!(err.contains("sync::tests::the_skill_copy_lives_at_the_workspace_root"), "{err}");
    }

    #[test]
    #[validates(spec::AGreenValidationFailsTheRedCheck)]
    fn a_green_validation_fails_the_red_check() {
        let err = red_verdict(&[], &[red("A", "t::a"), green("B", "t::b")]).expect_err("green fails");
        assert!(err.contains("t::b") && !err.contains("t::a"), "{err}");
        red_verdict(&[], &[red("A", "t::a"), red("B", "t::b")]).expect("all red passes");
    }
}
