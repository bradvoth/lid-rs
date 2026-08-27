use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::mapping::EdgeRecord;
use crate::mutants::{self, Registry, SpecRecord, dump_registry};
use crate::project::{Project, capture};
use crate::spec;
use crate::sync;

/// The phases that have a check, as the messages name them.
const CHECKED_PHASES: &str = "1, 2, 3, 4, 5, 7";

/// Usage for `phase-check`.
const PHASE_CHECK_USAGE: &str = "usage: cargo lid-rs phase-check <n> [--slice <name>]";

/// Usage for `hook`.
const HOOK_USAGE: &str = "usage: cargo lid-rs hook <commit-msg <file> | subagent-start | subagent-stop>";

/// What a worker is told when it tries to end without a phase commit.
const REFUSAL: &str = "No `phase N:` commit has been made since this phase started (HEAD is unchanged). \
Either commit the phase now — the commit-msg hook runs its check — or end with the numbered decisions \
that block it; the next stop is allowed.";

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

/// Which stop attempt this is: Claude Code's `stop_hook_active` is true
/// when it is already continuing because an earlier stop was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attempt {
    /// The worker's first attempt to stop.
    First,
    /// A stop after a refusal.
    Retry,
}

/// What a hook reads from the JSON Claude Code passes on stdin — the
/// boundary type; everything past it takes plain values.
#[derive(Debug, PartialEq, Eq)]
pub struct HookInput {
    /// The subagent's id, the key its `HEAD` record is filed under.
    pub agent_id: String,
    /// Whether this stop follows a refusal.
    pub attempt: Attempt,
}

/// The stop hook's answer, rendered to stdout at the boundary.
#[derive(Debug, PartialEq, Eq)]
pub enum StopDecision {
    /// The worker may stop.
    Allow,
    /// The worker is kept running, with this reason as its next instruction.
    Refuse(String),
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

/// `hook <commit-msg <file> | subagent-start | subagent-stop>`: one dispatch
/// over the hook kind; the subagent hooks read their input from stdin.
pub fn hook(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("commit-msg") => commit_msg_from_args(&args[1..]),
        Some("subagent-start") => subagent_start_from_stdin(),
        Some("subagent-stop") => subagent_stop_from_stdin(),
        Some(_) | None => Err(HOOK_USAGE.to_string()),
    }
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

/// `hook commit-msg <file>`: the message's subject, judged against this
/// project's phase checks.
#[implements(spec::TaggedCommitsRunTheirPhaseCheck)]
fn hook_commit_msg(project: &Project, message_file: &Path) -> Result<(), String> {
    let message = std::fs::read_to_string(message_file)
        .map_err(|e| format!("reading the commit message at {}: {e}", message_file.display()))?;
    let slice = resolve_slice(project, None)?;
    commit_msg_verdict(subject_of(&message), |phase| check(project, phase, slice.as_deref()))
}

/// The commit-msg hook's decision for a subject: a tagged subject runs its
/// phase's check and its failure refuses the commit; an untagged one passes
/// without a check; a tag naming a phase without a check is refused by name.
#[implements(
    spec::TaggedCommitsRunTheirPhaseCheck,
    spec::UntaggedCommitsPassTheHook,
    spec::MistypedTagsAreRefusedNotIgnored,
)]
pub fn commit_msg_verdict(subject: &str, check: impl FnOnce(Phase) -> Result<(), String>) -> Result<(), String> {
    match tag_of(subject) {
        Tag::Untagged => Ok(()),
        Tag::Checked(phase) => check(phase),
        Tag::Unchecked(n) => Err(format!(
            "`phase {n}:` names a phase with no check of its own; the phases with one are {CHECKED_PHASES}, \
             and a commit that is not a phase commit carries no `phase N:` tag"
        )),
    }
}

/// `hook subagent-start`: records the repository's `HEAD` under the agent's
/// id.
#[implements(spec::AStartingWorkerRecordsHead)]
fn hook_subagent_start(project: &Project, input: &HookInput) -> Result<(), String> {
    let path = record_path(project, &input.agent_id)?;
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(&path, head(project)?).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// `hook subagent-stop`: the recorded `HEAD`, the current one, and whether
/// this is a retry decide whether the worker may stop.
fn hook_subagent_stop(project: &Project, input: &HookInput) -> Result<StopDecision, String> {
    let recorded = std::fs::read_to_string(record_path(project, &input.agent_id)?).ok();
    Ok(stop_decision(recorded.as_deref().map(str::trim), head(project)?.trim(), input.attempt))
}

/// `hook commit-msg`'s arguments: the message file.
fn commit_msg_from_args(args: &[String]) -> Result<(), String> {
    let file = args.first().ok_or(HOOK_USAGE)?;
    hook_commit_msg(&Project::load_graph()?, Path::new(file))
}

/// `hook subagent-start`, its input read from stdin.
fn subagent_start_from_stdin() -> Result<(), String> {
    hook_subagent_start(&Project::load()?, &HookInput::from_json(&read_stdin()?)?)
}

/// `hook subagent-stop`, its input read from stdin and its decision printed.
fn subagent_stop_from_stdin() -> Result<(), String> {
    let decision = hook_subagent_stop(&Project::load()?, &HookInput::from_json(&read_stdin()?)?)?;
    print!("{}", render(&decision));
    Ok(())
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

/// The slice given, or the current branch's, or none.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn resolve_slice(project: &Project, given: Option<String>) -> Result<Option<String>, String> {
    Ok(given.or(slice_of_branch(&current_branch(project)?)))
}

/// The slice an `lld/<slice>` branch is for, or none for any other name.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn slice_of_branch(branch: &str) -> Option<String> {
    branch.strip_prefix("lld/").map(str::to_string)
}

/// The repository's current branch name; a detached `HEAD` is an error.
fn current_branch(project: &Project) -> Result<String, String> {
    Ok(capture(project.git()?.args(["symbolic-ref", "--short", "HEAD"]))?.trim().to_string())
}

/// The message's subject: its first line that is neither empty nor a `#`
/// comment.
fn subject_of(message: &str) -> &str {
    message.lines().map(str::trim).find(|line| !line.is_empty() && !line.starts_with('#')).unwrap_or("")
}

/// What the subject's `phase N:` prefix, if any, names.
#[implements(spec::UntaggedCommitsPassTheHook, spec::MistypedTagsAreRefusedNotIgnored)]
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

/// The hook's stdin, whole.
fn read_stdin() -> Result<String, String> {
    std::io::read_to_string(std::io::stdin()).map_err(|e| format!("reading the hook input: {e}"))
}

impl HookInput {
    /// The fields the hooks use, from Claude Code's hook JSON; the stop flag
    /// is absent at start, so it defaults to false.
    #[implements(spec::AStartingWorkerRecordsHead)]
    pub fn from_json(json: &str) -> Result<Self, String> {
        let doc: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("parsing the hook input: {e}"))?;
        let agent_id = doc
            .pointer("/agent_id")
            .and_then(serde_json::Value::as_str)
            .ok_or("the hook input carries no agent_id")?
            .to_string();
        let retrying = doc.pointer("/stop_hook_active").and_then(serde_json::Value::as_bool).unwrap_or(false);
        Ok(Self { agent_id, attempt: if retrying { Attempt::Retry } else { Attempt::First } })
    }
}

/// Where an agent's `HEAD` record lives: `<target>/lid-rs/agents/<agent_id>`.
#[implements(spec::AStartingWorkerRecordsHead)]
fn record_path(project: &Project, agent_id: &str) -> Result<PathBuf, String> {
    Ok(project.target_directory()?.join("lid-rs/agents").join(agent_id))
}

/// The repository's current `HEAD`, the value a worker's record holds.
#[implements(spec::AStartingWorkerRecordsHead)]
fn head(project: &Project) -> Result<String, String> {
    Ok(capture(project.git()?.args(["rev-parse", "HEAD"]))?.trim().to_string())
}

/// Whether the worker may stop, from its recorded `HEAD`, the current one,
/// and whether Claude Code is already retrying after a refusal.
#[implements(
    spec::AWorkerThatCommittedMayStop,
    spec::AWorkerThatDidNotCommitIsRefusedOnce,
    spec::ASecondStopAttemptIsAllowed,
    spec::AStopWithoutARecordIsAllowed,
)]
pub fn stop_decision(recorded: Option<&str>, head: &str, attempt: Attempt) -> StopDecision {
    match (recorded, attempt) {
        (None, _) | (Some(_), Attempt::Retry) => StopDecision::Allow,
        (Some(r), Attempt::First) if r == head => StopDecision::Refuse(REFUSAL.to_string()),
        (Some(_), Attempt::First) => StopDecision::Allow,
    }
}

/// What the stop hook prints: nothing to allow, Claude Code's block
/// decision JSON to refuse.
#[implements(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
pub fn render(decision: &StopDecision) -> String {
    match decision {
        StopDecision::Allow => String::new(),
        StopDecision::Refuse(reason) => serde_json::json!({ "decision": "block", "reason": reason }).to_string(),
    }
}

#[cfg(test)]
mod tests {
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

    #[test]
    #[validates(spec::TaggedCommitsRunTheirPhaseCheck)]
    fn the_subject_is_the_first_line_that_is_not_a_comment() {
        assert_eq!(subject_of("# comment\n\nphase 3: skeleton\n\nbody"), "phase 3: skeleton");
        assert_eq!(tag_of("phase 3: skeleton for x"), Tag::Checked(Phase::Three));
    }

    #[test]
    #[validates(spec::TaggedCommitsRunTheirPhaseCheck)]
    fn tagged_commits_run_their_phase_check() {
        let mut checked = None;
        let err = commit_msg_verdict("phase 3: skeleton", |phase| {
            checked = Some(phase);
            Err("check 4 fired".to_string())
        })
        .expect_err("a failing check refuses the commit");
        assert_eq!(checked, Some(Phase::Three));
        assert!(err.contains("check 4 fired"), "{err}");
        commit_msg_verdict("phase 5: red", |_| Ok(())).expect("a passing check allows the commit");
    }

    #[test]
    #[validates(spec::TaggedCommitsRunTheirPhaseCheck)]
    fn the_hook_reads_the_message_file_and_judges_its_subject() {
        let dir = scratch_repo("commit-msg");
        let project = this_project();
        let file = dir.join("COMMIT_EDITMSG");
        std::fs::write(&file, "phase 6: leaves\n").expect("write");
        let err = hook_commit_msg(&project, &file).expect_err("a mistyped tag is refused");
        assert!(err.contains("1, 2, 3, 4, 5, 7"), "{err}");
        std::fs::write(&file, "# comment\n\ntidy up\n").expect("write");
        hook_commit_msg(&project, &file).expect("an untagged commit passes");
        assert!(hook_commit_msg(&project, &dir.join("missing")).is_err(), "an unreadable message file is an error");
    }

    #[test]
    #[validates(spec::UntaggedCommitsPassTheHook)]
    fn untagged_commits_pass_the_hook() {
        assert_eq!(tag_of("0.2.2: the slice commit"), Tag::Untagged);
        assert_eq!(tag_of("phases are great"), Tag::Untagged);
        commit_msg_verdict("tidy up", |phase| panic!("no check runs for an untagged commit, ran {phase:?}"))
            .expect("allowed");
    }

    #[test]
    #[validates(spec::MistypedTagsAreRefusedNotIgnored)]
    fn mistyped_tags_are_refused_not_ignored() {
        assert_eq!(tag_of("phase 6: leaves"), Tag::Unchecked(6));
        assert_eq!(tag_of("phase 0: name"), Tag::Unchecked(0));
        let err = commit_msg_verdict("phase 6: leaves", |phase| panic!("no check for {phase:?}")).expect_err("refused");
        assert!(err.contains("1, 2, 3, 4, 5, 7"), "{err}");
    }

    #[test]
    #[validates(spec::AStartingWorkerRecordsHead)]
    fn the_hook_input_carries_the_agent_id_and_the_attempt() {
        let input = HookInput::from_json(r#"{"agent_id":"a1b2","stop_hook_active":false,"cwd":"/x"}"#).expect("parses");
        assert_eq!(input, HookInput { agent_id: "a1b2".to_string(), attempt: Attempt::First });
        let retry = HookInput::from_json(r#"{"agent_id":"a1b2","stop_hook_active":true}"#).expect("parses");
        assert_eq!(retry.attempt, Attempt::Retry);
        assert!(HookInput::from_json(r#"{"cwd":"/x"}"#).is_err(), "no agent_id is an error");
    }

    #[test]
    #[validates(spec::AStartingWorkerRecordsHead)]
    fn a_starting_worker_records_head() {
        let project = this_project();
        let agent = format!("phase-tests-{}", std::process::id());
        let path = record_path(&project, &agent).expect("path");
        assert!(path.ends_with(Path::new("lid-rs/agents").join(&agent)), "{}", path.display());
        let _ = std::fs::remove_file(&path);
        hook_subagent_start(&project, &HookInput { agent_id: agent, attempt: Attempt::First }).expect("records");
        let recorded = std::fs::read_to_string(&path).expect("the record exists");
        let expected = capture(std::process::Command::new("git").args(["rev-parse", "HEAD"]).current_dir(project.root().expect("root")))
            .expect("git");
        assert_eq!(recorded.trim(), expected.trim(), "the record is the repository's HEAD");
        assert_eq!(head(&project).expect("git").trim(), expected.trim());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    #[validates(spec::AWorkerThatCommittedMayStop)]
    fn a_worker_that_committed_may_stop() {
        assert_eq!(stop_decision(Some("aaa"), "bbb", Attempt::First), StopDecision::Allow);
    }

    #[test]
    #[validates(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
    fn a_worker_that_did_not_commit_is_refused_once() {
        let StopDecision::Refuse(reason) = stop_decision(Some("aaa"), "aaa", Attempt::First) else {
            panic!("an unmoved HEAD on the first attempt is refused");
        };
        assert!(reason.contains("commit"), "{reason}");
    }

    #[test]
    #[validates(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
    fn a_refusal_renders_as_a_block_decision() {
        let rendered = render(&StopDecision::Refuse("why".to_string()));
        let json: serde_json::Value = serde_json::from_str(&rendered).expect("the refusal is Claude Code's JSON");
        assert_eq!(json["decision"], "block");
        assert_eq!(json["reason"], "why");
        assert_eq!(render(&StopDecision::Allow), "", "allowing prints nothing");
    }

    #[test]
    #[validates(spec::ASecondStopAttemptIsAllowed)]
    fn a_second_stop_attempt_is_allowed() {
        assert_eq!(stop_decision(Some("aaa"), "aaa", Attempt::Retry), StopDecision::Allow);
    }

    #[test]
    #[validates(spec::AStopWithoutARecordIsAllowed)]
    fn a_stop_without_a_record_is_allowed() {
        assert_eq!(stop_decision(None, "aaa", Attempt::First), StopDecision::Allow);
        let project = this_project();
        let input = HookInput { agent_id: format!("phase-tests-none-{}", std::process::id()), attempt: Attempt::First };
        assert_eq!(hook_subagent_stop(&project, &input).expect("decides"), StopDecision::Allow);
    }
}
