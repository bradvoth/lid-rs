use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::mapping::EdgeRecord;
use crate::mutants::{Registry, SpecRecord};
use crate::project::Project;
use crate::spec;

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
        todo!()
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

/// What a hook reads from the JSON Claude Code passes on stdin — the
/// boundary type; everything past it takes plain values.
#[derive(Debug, PartialEq, Eq)]
pub struct HookInput {
    /// The subagent's id, the key its `HEAD` record is filed under.
    pub agent_id: String,
    /// True when Claude Code is already continuing because a stop was refused.
    pub stop_hook_active: bool,
}

/// The stop hook's answer, rendered to stdout at the boundary.
#[derive(Debug, PartialEq, Eq)]
pub enum StopDecision {
    /// The worker may stop.
    Allow,
    /// The worker is kept running, with this reason as its next instruction.
    Refuse(String),
}

/// `phase-check <n> [--slice <name>]`: parse, locate the project, check.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn run(args: &[String]) -> Result<(), String> {
    todo!()
}

/// `hook <commit-msg <file> | subagent-start | subagent-stop>`: one dispatch
/// over the hook kind; the subagent hooks read their input from stdin.
pub fn hook(args: &[String]) -> Result<(), String> {
    todo!()
}

/// Runs a phase's check: its plan, executed in order.
pub fn check(project: &Project, phase: Phase, slice: Option<&str>) -> Result<(), String> {
    todo!()
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
    todo!()
}

/// Runs steps in order against the project; the first failure is the
/// result, naming its step, and no later step runs.
pub fn execute(project: &Project, slice: Option<&str>, steps: &[Step]) -> Result<(), String> {
    todo!()
}

/// `execute` over any runner: the first failure is the result and no later
/// step runs.
#[implements(spec::ACheckStopsAtTheFirstFailingStep)]
fn execute_with(steps: &[Step], run: impl FnMut(&Step) -> Result<(), String>) -> Result<(), String> {
    todo!()
}

/// Runs one step: one dispatch over the closed set.
#[implements(spec::PhaseSevenRunsTheGateInOrder)]
fn run_step(project: &Project, slice: Option<&str>, step: &Step) -> Result<(), String> {
    todo!()
}

/// The phase 5 red run: the slice's claims, their validations, each run
/// alone; fails naming every unvalidated claim and every green test.
#[implements(spec::AGreenValidationFailsTheRedCheck)]
pub fn check_red(project: &Project, slice: &str) -> Result<(), String> {
    todo!()
}

/// `hook commit-msg <file>`: the message's subject, judged against this
/// project's phase checks.
#[implements(spec::TaggedCommitsRunTheirPhaseCheck)]
fn hook_commit_msg(project: &Project, message_file: &Path) -> Result<(), String> {
    todo!()
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
    todo!()
}

/// `hook subagent-start`: records the repository's `HEAD` under the agent's
/// id.
#[implements(spec::AStartingWorkerRecordsHead)]
fn hook_subagent_start(project: &Project, input: &HookInput) -> Result<(), String> {
    todo!()
}

/// `hook subagent-stop`: the recorded `HEAD`, the current one, and whether
/// this is a retry decide whether the worker may stop.
fn hook_subagent_stop(project: &Project, input: &HookInput) -> Result<StopDecision, String> {
    todo!()
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

/// `phase-check`'s arguments: the phase number, then optionally
/// `--slice <name>`; any other flag is rejected by name.
#[implements(spec::TheSliceComesFromTheBranchName)]
fn parse_args(args: &[String]) -> Result<(Phase, Option<String>), String> {
    todo!()
}

/// The slice an `lld/<slice>` branch is for, or none for any other name.
#[implements(spec::TheSliceComesFromTheBranchName)]
pub fn slice_of_branch(branch: &str) -> Option<String> {
    todo!()
}

/// The repository's current branch name.
fn current_branch(project: &Project) -> Result<String, String> {
    todo!()
}

/// The message's subject: its first line that is not a `#` comment.
fn subject_of(message: &str) -> &str {
    todo!()
}

/// What the subject's `phase N:` prefix, if any, names.
#[implements(spec::UntaggedCommitsPassTheHook, spec::MistypedTagsAreRefusedNotIgnored)]
pub fn tag_of(subject: &str) -> Tag {
    todo!()
}

/// Runs one cargo command with its output inherited, so the check's output
/// is what the caller prints; fails naming the step.
fn cargo_step(project: &Project, step: &Step, args: &[&str], env: &[(&str, &str)]) -> Result<(), String> {
    todo!()
}

/// Every library member's registry, each from its own test binary.
fn package_registries(project: &Project) -> Result<Vec<PackageRegistry>, String> {
    todo!()
}

/// The file a slice's claims register from: `src/spec/<slice>.rs`, the
/// slice name in snake_case.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn spec_file_of(slice: &str) -> String {
    todo!()
}

/// The names of the specs registered from the slice's spec file.
#[implements(spec::ASlicesClaimsAreTheSpecsInItsSpecFile)]
pub fn slice_claims(specs: &[SpecRecord], slice: &str) -> Vec<String> {
    todo!()
}

/// The claims, or the failure for a slice that registers none.
#[implements(spec::ASliceWithNoClaimsFailsTheRedCheck)]
fn require_claims(claims: Vec<String>, slice: &str) -> Result<Vec<String>, String> {
    todo!()
}

/// `(claim, test)` for every validation edge on the claims, the test's item
/// path made libtest-relative.
#[implements(spec::EachValidationRunsAloneByExactName)]
pub fn claim_validations(validations: &[EdgeRecord], claims: &[String]) -> Vec<(String, String)> {
    todo!()
}

/// Runs one test alone; true when it passes.
#[implements(spec::EachValidationRunsAloneByExactName)]
fn run_test(project: &Project, package: &str, test: &str) -> Result<bool, String> {
    todo!()
}

/// The claims no outcome validates.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses)]
pub fn unvalidated(claims: &[String], outcomes: &[Outcome]) -> Vec<String> {
    todo!()
}

/// The verdict: pass when nothing is unvalidated and nothing is green;
/// otherwise the failure naming every unvalidated claim and green test.
#[implements(spec::EveryClaimNeedsAValidationBeforePhaseFivePasses, spec::AGreenValidationFailsTheRedCheck)]
pub fn red_verdict(unvalidated: &[String], outcomes: &[Outcome]) -> Result<(), String> {
    todo!()
}

/// The hook's stdin, whole.
fn read_stdin() -> Result<String, String> {
    todo!()
}

impl HookInput {
    /// The fields the hooks use, from Claude Code's hook JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        todo!()
    }
}

/// Where an agent's `HEAD` record lives: `<target>/lid-rs/agents/<agent_id>`.
#[implements(spec::AStartingWorkerRecordsHead)]
fn record_path(project: &Project, agent_id: &str) -> Result<PathBuf, String> {
    todo!()
}

/// The repository's current `HEAD`.
fn head(project: &Project) -> Result<String, String> {
    todo!()
}

/// Whether the worker may stop, from its recorded `HEAD`, the current one,
/// and whether Claude Code is already retrying after a refusal.
#[implements(
    spec::AWorkerThatCommittedMayStop,
    spec::AWorkerThatDidNotCommitIsRefusedOnce,
    spec::ASecondStopAttemptIsAllowed,
    spec::AStopWithoutARecordIsAllowed,
)]
pub fn stop_decision(recorded: Option<&str>, head: &str, retry: bool) -> StopDecision {
    todo!()
}

/// What the stop hook prints: nothing to allow, Claude Code's block
/// decision JSON to refuse.
#[implements(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
pub fn render(decision: &StopDecision) -> String {
    todo!()
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

    fn spec_record(name: &str, file: &str) -> SpecRecord {
        SpecRecord { name: name.to_string(), file: file.to_string() }
    }

    fn edge(spec: &str, item: &str) -> EdgeRecord {
        EdgeRecord { spec: spec.to_string(), item: item.to_string(), file: "x.rs".to_string() }
    }

    fn outcome(claim: &str, test: &str, passed: bool) -> Outcome {
        Outcome { claim: claim.to_string(), test: test.to_string(), passed }
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
        assert_eq!(parse_args(&strings(&["5"])).expect("parses"), (Phase::Five, None));
        assert_eq!(
            parse_args(&strings(&["5", "--slice", "login"])).expect("parses"),
            (Phase::Five, Some("login".to_string()))
        );
        let err = parse_args(&strings(&["3", "--bogus"])).expect_err("unknown flag");
        assert!(err.contains("--bogus"), "{err}");
        assert!(parse_args(&strings(&["x"])).is_err(), "a non-number is not a phase");
        assert!(parse_args(&[]).is_err(), "the phase is required");
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
        let outcomes = [outcome("A", "t::a", false)];
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
    }

    /// A known-green test for `run_test` to run alone.
    #[test]
    fn probe_that_passes() {}

    #[test]
    #[validates(spec::AGreenValidationFailsTheRedCheck)]
    fn a_green_validation_fails_the_red_check() {
        let err = red_verdict(&[], &[outcome("A", "t::a", false), outcome("B", "t::b", true)]).expect_err("green fails");
        assert!(err.contains("t::b") && !err.contains("t::a"), "{err}");
        red_verdict(&[], &[outcome("A", "t::a", false), outcome("B", "t::b", false)]).expect("all red passes");
    }

    #[test]
    #[validates(spec::TaggedCommitsRunTheirPhaseCheck)]
    fn tagged_commits_run_their_phase_check() {
        assert_eq!(subject_of("# comment\n\nphase 3: skeleton\n\nbody"), "phase 3: skeleton");
        assert_eq!(tag_of("phase 3: skeleton for x"), Tag::Checked(Phase::Three));
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
    fn a_starting_worker_records_head() {
        let input = HookInput::from_json(r#"{"agent_id":"a1b2","stop_hook_active":false,"cwd":"/x"}"#).expect("parses");
        assert_eq!(input, HookInput { agent_id: "a1b2".to_string(), stop_hook_active: false });
        let project = this_project();
        let agent = format!("phase-tests-{}", std::process::id());
        let path = record_path(&project, &agent).expect("path");
        assert!(path.ends_with(Path::new("lid-rs/agents").join(&agent)), "{}", path.display());
        let _ = std::fs::remove_file(&path);
        hook_subagent_start(&project, &HookInput { agent_id: agent, stop_hook_active: false }).expect("records");
        let recorded = std::fs::read_to_string(&path).expect("the record exists");
        assert_eq!(recorded.trim(), head(&project).expect("git").trim());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    #[validates(spec::AWorkerThatCommittedMayStop)]
    fn a_worker_that_committed_may_stop() {
        assert_eq!(stop_decision(Some("aaa"), "bbb", false), StopDecision::Allow);
    }

    #[test]
    #[validates(spec::AWorkerThatDidNotCommitIsRefusedOnce)]
    fn a_worker_that_did_not_commit_is_refused_once() {
        let StopDecision::Refuse(reason) = stop_decision(Some("aaa"), "aaa", false) else {
            panic!("an unmoved HEAD on the first attempt is refused");
        };
        assert!(reason.contains("commit"), "{reason}");
        let rendered = render(&StopDecision::Refuse(reason.clone()));
        let json: serde_json::Value = serde_json::from_str(&rendered).expect("the refusal is Claude Code's JSON");
        assert_eq!(json["decision"], "block");
        assert_eq!(json["reason"], reason);
        assert_eq!(render(&StopDecision::Allow), "", "allowing prints nothing");
    }

    #[test]
    #[validates(spec::ASecondStopAttemptIsAllowed)]
    fn a_second_stop_attempt_is_allowed() {
        assert_eq!(stop_decision(Some("aaa"), "aaa", true), StopDecision::Allow);
    }

    #[test]
    #[validates(spec::AStopWithoutARecordIsAllowed)]
    fn a_stop_without_a_record_is_allowed() {
        assert_eq!(stop_decision(None, "aaa", false), StopDecision::Allow);
        let project = this_project();
        let input = HookInput { agent_id: format!("phase-tests-none-{}", std::process::id()), stop_hook_active: false };
        assert_eq!(hook_subagent_stop(&project, &input).expect("decides"), StopDecision::Allow);
    }
}
