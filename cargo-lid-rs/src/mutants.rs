//! Check 12 orchestration: enumerate mutants, join them to their validating
//! tests through the dumped registries, and run `cargo-mutants` per test-set
//! group (`docs/intent/cargo-lid-rs/lld.md`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::mapping::{EdgeRecord, TestPlan, plan_for_mutant};
use crate::project::{Project, Scope, capture};
use crate::spec;

/// One mutant from `cargo mutants --list --json`.
#[derive(Debug)]
pub struct Mutant {
    /// The unique mutant name, as matched by `-F`.
    pub name: String,
    /// Workspace-relative source file.
    pub file: String,
    /// Unqualified function name.
    pub function: String,
}

/// One registered claim: its name and the file it is defined in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecRecord {
    /// The spec's `NAME`.
    pub name: String,
    /// Source file of the `derive(Spec)` site.
    pub file: String,
}

/// The dumped registries the planner joins over.
#[derive(Debug, Default)]
pub struct Registry {
    /// Every claim, with its definition file (`phase-check 5` reads these).
    pub specs: Vec<SpecRecord>,
    /// Implementation edges across the workspace's crates.
    pub impls: Vec<EdgeRecord>,
    /// Validation edges across the workspace's crates.
    pub validations: Vec<EdgeRecord>,
}

/// Runs the mutants subcommand: locate, scope, list, collect registries,
/// plan, execute.
pub fn run(args: &[String]) -> Result<(), String> {
    let project = Project::load()?;
    let scope = parse_scope(args, project.configured_scope())?;
    let diff = write_diff_file(&scope, &project)?;
    let mutants = list_mutants(&project, diff.as_deref())?;
    let registry = collect_registries(&project)?;
    let groups = group_by_plan(&mutants, &registry);
    run_groups(&project, &groups, diff.as_deref())
}

/// Parses `--full` / `--diff-base <ref>` over the configured scope.
fn parse_scope(args: &[String], configured: Scope) -> Result<Scope, String> {
    let mut scope = configured;
    let mut rest = args;
    while let Some((flag, tail)) = rest.split_first() {
        (scope, rest) = apply_flag(flag, tail)?;
    }
    Ok(scope)
}

/// Applies one flag, returning the new scope and the unconsumed arguments.
#[implements(spec::ScopeFlagsOverrideTheConfiguredScope)]
fn apply_flag<'a>(flag: &str, tail: &'a [String]) -> Result<(Scope, &'a [String]), String> {
    match flag {
        "--full" => Ok((Scope::Full, tail)),
        "--diff-base" => {
            let base = tail
                .first()
                .ok_or_else(|| "--diff-base requires a git ref".to_string())?;
            Ok((Scope::Diff { base: base.clone() }, &tail[1..]))
        }
        other => Err(format!("unknown flag `{other}` for mutants")),
    }
}

/// For diff scope, writes `git diff <base>` to a file under the build
/// directory and returns its path; `None` for full scope.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn write_diff_file(scope: &Scope, project: &Project) -> Result<Option<PathBuf>, String> {
    let Scope::Diff { base } = scope else {
        return Ok(None);
    };
    let diff = capture(project.git()?.args(["diff", base]))?;
    let path = project.target_directory()?.join("lid-mutants.diff");
    std::fs::write(&path, &diff).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok(Some(path))
}

/// Enumerates mutants, restricted by the diff when one is given.
fn list_mutants(project: &Project, diff: Option<&Path>) -> Result<Vec<Mutant>, String> {
    let out = capture(project.cargo()?.args(list_args(diff)))?;
    parse_mutants(&out)
}

/// The `cargo mutants --list` argument list — pure, so the diff pass-through
/// is unit-testable.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn list_args(diff: Option<&Path>) -> Vec<String> {
    let mut args: Vec<String> = ["mutants", "--list", "--json"].map(String::from).into();
    if let Some(d) = diff {
        args.push("--in-diff".to_string());
        args.push(d.display().to_string());
    }
    args
}

/// Parses the engine's JSON mutant list. Blank output is a valid empty
/// listing: with an empty `--in-diff` the engine prints nothing, not `[]`.
fn parse_mutants(json: &str) -> Result<Vec<Mutant>, String> {
    if json.trim().is_empty() {
        return Ok(Vec::new());
    }
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("parsing mutant list: {e}"))?;
    let array = value.as_array().ok_or("mutant list is not a JSON array")?;
    array.iter().map(mutant_of).collect()
}

/// Extracts one mutant's identity fields.
fn mutant_of(value: &serde_json::Value) -> Result<Mutant, String> {
    let get = |pointer: &str| {
        value
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("mutant entry missing {pointer}: {value}"))
    };
    Ok(Mutant {
        name: get("/name")?,
        file: get("/file")?,
        function: get("/function/function_name")?,
    })
}

/// Collects the registries of every library member by running each member's
/// own `--lib` test binary in dump mode — the only binary whose `#[validates]`
/// edges exist (README §5.2). Members without the graph checks dump nothing
/// and contribute nothing.
#[implements(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
pub fn collect_registries(project: &Project) -> Result<Registry, String> {
    let mut registry = Registry::default();
    for package in project.library_members() {
        let dumped = dump_registry(project, &package)?;
        registry.specs.extend(dumped.specs);
        registry.impls.extend(dumped.impls);
        registry.validations.extend(dumped.validations);
    }
    Ok(registry)
}

/// One crate's registry, dumped by its `intent_graph!()` dump test.
#[implements(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
pub fn dump_registry(project: &Project, package: &str) -> Result<Registry, String> {
    let out = capture(
        project
            .cargo()?
            .args([
                "test", "-p", package, "--lib", "--", "--exact",
                "intent_graph::registry_dump_for_tooling", "--nocapture",
            ])
            .env("LID_DUMP", "1"),
    )?;
    Ok(parse_dump(&out))
}

/// Parses `LID-DUMP` lines into edge records.
fn parse_dump(output: &str) -> Registry {
    let mut registry = Registry::default();
    for line in output.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let ["LID-DUMP", kind, spec_name, item, file] = fields.as_slice() else {
            continue;
        };
        let record = EdgeRecord {
            spec: (*spec_name).to_string(),
            item: (*item).to_string(),
            file: (*file).to_string(),
        };
        match *kind {
            "SPEC" => registry.specs.push(SpecRecord { name: record.spec, file: record.item }),
            "IMPL" => registry.impls.push(record),
            "VALID" => registry.validations.push(record),
            _ => {}
        }
    }
    registry
}

/// Groups mutant names by their test plan, so each distinct test set gets one
/// engine run. `BTreeMap` keeps run order deterministic.
fn group_by_plan(mutants: &[Mutant], registry: &Registry) -> BTreeMap<TestPlan, Vec<String>> {
    let mut groups: BTreeMap<TestPlan, Vec<String>> = BTreeMap::new();
    for mutant in mutants {
        let plan = plan_for_mutant(
            &mutant.file,
            &mutant.function,
            &registry.impls,
            &registry.validations,
        );
        groups.entry(plan).or_default().push(mutant.name.clone());
    }
    groups
}

/// A mutant that outlived the tests chosen for it.
#[derive(Debug, PartialEq, Eq)]
pub struct Survivor {
    /// The engine's mutant name.
    pub name: String,
    /// The engine's summary for it (`MissedMutant` or `Timeout`).
    pub verdict: String,
    /// The tests it survived.
    pub plan: TestPlan,
}

/// Runs every group, then fails if any mutant survived.
#[implements(spec::SurvivingMutantsFailTheGate)]
fn run_groups(
    project: &Project,
    groups: &BTreeMap<TestPlan, Vec<String>>,
    diff: Option<&Path>,
) -> Result<(), String> {
    let output_root = project.target_directory()?.join("lid-mutants");
    let survivors = run_groups_with(groups, |index, plan, names| {
        run_group(project, plan, names, diff, &output_root.join(index.to_string()))
    })?;
    if survivors.is_empty() {
        Ok(())
    } else {
        Err(survivor_report(&survivors))
    }
}

/// Runs every group through `run` with its distinct index, accumulating
/// survivors, so one run reports them all; only an engine failure (an `Err`
/// from `run`) stops early.
#[implements(spec::EveryGroupRunsBeforeSurvivorsAreReported, spec::AMutantsVerdictComesFromItsOwnGroupsRun)]
fn run_groups_with(
    groups: &BTreeMap<TestPlan, Vec<String>>,
    mut run: impl FnMut(usize, &TestPlan, &[String]) -> Result<Vec<Survivor>, String>,
) -> Result<Vec<Survivor>, String> {
    let mut survivors = Vec::new();
    for (index, (plan, names)) in groups.iter().enumerate() {
        survivors.extend(run(index, plan, names)?);
    }
    Ok(survivors)
}

/// One engine invocation for a group into a fresh `output` directory, judged
/// from the outcomes the engine writes there. The exit status is not
/// consulted: it conflates the group's mutants with any the engine added.
fn run_group(
    project: &Project,
    plan: &TestPlan,
    names: &[String],
    diff: Option<&Path>,
    output: &Path,
) -> Result<Vec<Survivor>, String> {
    let _ = std::fs::remove_dir_all(output);
    std::fs::create_dir_all(output).map_err(|e| format!("creating {}: {e}", output.display()))?;
    project
        .cargo()?
        .args(group_args(names, plan, diff, output))
        .status()
        .map_err(|e| format!("running cargo mutants: {e}"))?;
    let path = output.join("mutants.out/outcomes.json");
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("the engine left no outcomes at {} for {names:?}: {e}", path.display()))?;
    group_verdicts(names, &verdicts(&json)?, plan)
}

/// The engine's `outcomes.json` as mutant name → summary.
fn verdicts(json: &str) -> Result<BTreeMap<String, String>, String> {
    let doc: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("parsing the engine's outcomes: {e}"))?;
    let outcomes = doc
        .pointer("/outcomes")
        .and_then(serde_json::Value::as_array)
        .ok_or("the engine's outcomes carry no outcomes array")?;
    Ok(outcomes.iter().filter_map(verdict_of).collect())
}

/// One outcome entry as `(mutant name, summary)`; baseline entries have no
/// mutant and yield nothing.
fn verdict_of(outcome: &serde_json::Value) -> Option<(String, String)> {
    let name = outcome.pointer("/scenario/Mutant/name")?.as_str()?;
    let summary = outcome.pointer("/summary")?.as_str()?;
    Some((name.to_string(), summary.to_string()))
}

/// The group's own survivors, judged from the engine's verdicts; mutants the
/// engine included but the group did not select are not judged here.
#[implements(spec::AMutantsVerdictComesFromItsOwnGroupsRun)]
fn group_verdicts(
    names: &[String],
    verdicts: &BTreeMap<String, String>,
    plan: &TestPlan,
) -> Result<Vec<Survivor>, String> {
    names
        .iter()
        .filter_map(|name| survivor_of(name, verdicts.get(name).map(String::as_str), plan).transpose())
        .collect()
}

/// One mutant's fate from its verdict: caught or unviable is fine, missed or
/// timed out survives, anything else — including no verdict at all — fails.
#[implements(spec::AnEngineRunWithoutAVerdictIsAFailure, spec::SurvivingMutantsFailTheGate)]
fn survivor_of(name: &str, verdict: Option<&str>, plan: &TestPlan) -> Result<Option<Survivor>, String> {
    match verdict {
        None => Err(format!(
            "the engine reported no verdict for `{name}`; a mutant without a verdict is not a caught mutant"
        )),
        Some("CaughtMutant" | "Unviable") => Ok(None),
        Some(fate @ ("MissedMutant" | "Timeout")) => Ok(Some(Survivor {
            name: name.to_string(),
            verdict: fate.to_string(),
            plan: plan.clone(),
        })),
        Some(other) => Err(format!("unrecognised engine verdict `{other}` for `{name}`")),
    }
}

/// The gate's failure message, one line per survivor.
fn survivor_report(survivors: &[Survivor]) -> String {
    let lines: Vec<String> = survivors
        .iter()
        .map(|s| format!("  {} — {} against {:?}", s.name, s.verdict, s.plan))
        .collect();
    format!(
        "mutation gate failed: {} mutant(s) survived their validating tests — the citations are \
         decorative (README §4.3):\n{}",
        survivors.len(),
        lines.join("\n")
    )
}

/// The `cargo mutants` argument list for one group — pure, so the pass-through
/// of diff and test filters is unit-testable.
fn group_args(names: &[String], plan: &TestPlan, diff: Option<&Path>, output: &Path) -> Vec<String> {
    let alternation: Vec<String> = names.iter().map(|n| regex_escape(n)).collect();
    // --test-workspace: a mutant in one crate (e.g. the proc-macro crate) is
    // often killed only by a dependent crate's tests; package-scoped testing
    // would let it survive unexercised.
    let mut args: Vec<String> =
        ["mutants", "--baseline", "skip", "--test-workspace", "true", "--output"].map(String::from).into();
    args.push(output.display().to_string());
    args.push("-F".to_string());
    args.push(format!("^({})$", alternation.join("|")));
    if let Some(d) = diff {
        args.push("--in-diff".to_string());
        args.push(d.display().to_string());
    }
    args.extend(filter_args(plan));
    args
}

/// The test-narrowing arguments for a plan; empty for the full suite.
fn filter_args(plan: &TestPlan) -> Vec<String> {
    let (TestPlan::Traced(filters) | TestPlan::ModuleFallback(filters)) = plan else {
        return Vec::new();
    };
    let mut args = vec![
        "--cargo-test-arg=--lib".to_string(),
        "--cargo-test-arg=--".to_string(),
        "--cargo-test-arg=--exact".to_string(),
    ];
    args.extend(filters.iter().map(|f| format!("--cargo-test-arg={f}")));
    args
}

/// Escapes regex metacharacters in a mutant name for the `-F` alternation,
/// mirroring `regex::escape`'s character set.
fn regex_escape(name: &str) -> String {
    let mut escaped = String::with_capacity(name.len());
    for c in name.chars() {
        if r"\.+*?()|[]{}^$#".contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    #[test]
    #[validates(spec::DiffScopePassesThroughToTheEngine)]
    fn diff_scope_passes_through_to_the_engine() {
        let with = list_args(Some(Path::new("target/lid-diff.patch")));
        let position = with.iter().position(|a| a == "--in-diff");
        assert!(position.is_some(), "--in-diff missing: {with:?}");
        assert_eq!(with[position.expect("checked above") + 1], "target/lid-diff.patch");
        let without = list_args(None);
        assert!(!without.contains(&"--in-diff".to_string()));
    }

    #[test]
    #[validates(spec::DiffScopePassesThroughToTheEngine)]
    fn diff_scope_writes_a_real_diff_file() {
        let project = Project::load().expect("cargo metadata must succeed in this repository");
        let scope = Scope::Diff { base: "HEAD".to_string() };
        let path = write_diff_file(&scope, &project)
            .expect("git diff HEAD must succeed in this repository")
            .expect("diff scope must yield a diff file");
        assert!(path.ends_with("lid-mutants.diff"), "{path:?}");
        assert!(path.exists(), "diff file must exist at {path:?}");
        assert_eq!(write_diff_file(&Scope::Full, &project).expect("full scope never fails"), None);
    }

    #[test]
    #[validates(spec::ScopeFlagsOverrideTheConfiguredScope)]
    fn scope_flags_override_the_configured_scope() {
        let against = |base: &str| Scope::Diff { base: base.to_string() };
        let cases: [(&[&str], Scope, Option<Scope>); 5] = [
            (&[], against("main"), Some(against("main"))),
            (&["--full"], against("main"), Some(Scope::Full)),
            (&["--diff-base", "origin/main"], Scope::Full, Some(against("origin/main"))),
            (&["--diff-base"], against("main"), None),
            (&["--bogus"], against("main"), None),
        ];
        for (flags, configured, expected) in cases {
            let flags: Vec<String> = flags.iter().map(|f| (*f).to_string()).collect();
            assert_eq!(parse_scope(&flags, configured).ok(), expected, "flags {flags:?}");
        }
    }

    #[test]
    #[validates(spec::ScopeFlagsOverrideTheConfiguredScope)]
    fn unknown_flags_are_rejected_by_name() {
        let result = parse_scope(&["--bogus".to_string()], Scope::Full);
        assert!(result.is_err_and(|e| e.contains("--bogus")));
    }

    #[test]
    #[validates(spec::SurvivingMutantsFailTheGate)]
    fn engine_failures_fail_the_gate() {
        let project = Project::load().expect("cargo metadata must succeed in this repository");
        let mut groups: BTreeMap<TestPlan, Vec<String>> = BTreeMap::new();
        groups.insert(TestPlan::FullSuite, vec!["any_mutant".to_string()]);
        let missing_diff = Path::new("target/definitely-missing.diff");
        let result = run_groups(&project, &groups, Some(missing_diff));
        assert!(result.is_err(), "an engine failure must fail the mutants gate");
    }

    #[test]
    #[validates(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
    fn registries_come_from_the_owning_crate_test_binary() {
        let project = Project::load().expect("cargo metadata must succeed in this repository");
        let registry =
            collect_registries(&project).expect("collecting workspace registries must succeed");
        let lid_test_edge = registry.validations.iter().any(|e| {
            e.item == "lid_rs::canary::tests::canary_confirms_registry_presence"
        });
        assert!(
            lid_test_edge,
            "a cfg(test)-only lid-rs validation edge must be present, proving the dump \
             came from lid-rs's own test binary: {:#?}",
            registry.validations
        );
        let xtask_test_edge = registry.validations.iter().any(|e| e.item.starts_with("xtask::"));
        assert!(
            xtask_test_edge,
            "collect_registries must have walked every library member, xtask included"
        );
    }

    #[test]
    fn empty_engine_output_means_no_mutants() {
        // An empty --in-diff makes `cargo mutants --list --json` print
        // nothing at all (not `[]`); that is a clean zero-mutant run, and on
        // every push to main it is the *common* case.
        let mutants = parse_mutants("").expect("blank output is a valid empty listing");
        assert!(mutants.is_empty());
        let whitespace = parse_mutants("\n").expect("whitespace-only output likewise");
        assert!(whitespace.is_empty());
    }

    #[test]
    fn dump_lines_parse_into_records() {
        let out = "noise\nLID-DUMP\tIMPL\ts\ti\tf\nLID-DUMP\tVALID\ts2\ti2\tf2\nLID-DUMP\tSPEC\tn\tf\t1\n";
        let registry = parse_dump(out);
        let shape = (
            registry.impls.len(),
            registry.validations.len(),
            registry.impls[0].spec.as_str(),
            registry.validations[0].item.as_str(),
        );
        assert_eq!(shape, (1, 1, "s", "i2"));
    }

    #[test]
    fn group_args_select_mutants_and_narrow_tests() {
        let plan = TestPlan::Traced(vec!["a::tests::t1".to_string()]);
        let args = group_args(&["m(1)".to_string()], &plan, None, Path::new("out"));
        let expected_tail = [
            r"-F".to_string(),
            r"^(m\(1\))$".to_string(),
            "--cargo-test-arg=--lib".to_string(),
            "--cargo-test-arg=--".to_string(),
            "--cargo-test-arg=--exact".to_string(),
            "--cargo-test-arg=a::tests::t1".to_string(),
        ];
        for expected in &expected_tail {
            assert!(args.contains(expected), "missing {expected}: {args:?}");
        }
    }

    #[test]
    fn full_suite_groups_run_unfiltered() {
        let full = group_args(&["m".to_string()], &TestPlan::FullSuite, None, Path::new("out"));
        let narrowed = full.iter().any(|a| a.starts_with("--cargo-test-arg"));
        assert!(!narrowed, "{full:?}");
    }

    /// The engine's outcomes.json shape, as observed from cargo-mutants 27.1.0.
    fn outcomes_json(entries: &[(&str, &str)]) -> String {
        let items: Vec<String> = entries
            .iter()
            .map(|(name, summary)| format!(r#"{{"scenario":{{"Mutant":{{"name":"{name}","file":"f"}}}},"summary":"{summary}"}}"#))
            .collect();
        format!(r#"{{"outcomes":[{}],"total_mutants":{},"cargo_mutants_version":"27.1.0"}}"#, items.join(","), entries.len())
    }

    fn plan() -> TestPlan {
        TestPlan::Traced(vec!["a::tests::t".to_string()])
    }

    #[test]
    #[validates(spec::AMutantsVerdictComesFromItsOwnGroupsRun)]
    fn a_mutants_verdict_comes_from_its_own_groups_run() {
        // The engine included a stowaway the group never selected and missed it.
        let json = outcomes_json(&[("src/a.rs:1:1: replace f with ()", "CaughtMutant"), ("src/b.rs:9:9: delete field x", "MissedMutant")]);
        let verdicts = verdicts(&json).expect("outcomes parse");
        let group = vec!["src/a.rs:1:1: replace f with ()".to_string()];
        assert_eq!(group_verdicts(&group, &verdicts, &plan()), Ok(vec![]), "the stowaway is not this group's verdict");
    }

    #[test]
    #[validates(spec::SurvivingMutantsFailTheGate, spec::AMutantsVerdictComesFromItsOwnGroupsRun)]
    fn a_missed_group_mutant_is_a_survivor() {
        let json = outcomes_json(&[("m1", "CaughtMutant"), ("m2", "MissedMutant"), ("m3", "Unviable"), ("m4", "Timeout")]);
        let verdicts = verdicts(&json).expect("outcomes parse");
        let group: Vec<String> = ["m1", "m2", "m3", "m4"].iter().map(|m| (*m).to_string()).collect();
        let survivors = group_verdicts(&group, &verdicts, &plan()).expect("all verdicts present");
        let names: Vec<(&str, &str)> = survivors.iter().map(|s| (s.name.as_str(), s.verdict.as_str())).collect();
        assert_eq!(names, vec![("m2", "MissedMutant"), ("m4", "Timeout")]);
    }

    #[test]
    #[validates(spec::AnEngineRunWithoutAVerdictIsAFailure)]
    fn an_engine_run_without_a_verdict_is_a_failure() {
        let missing = survivor_of("m9", None, &plan()).expect_err("no verdict must not pass");
        let unknown = survivor_of("m9", Some("Failure"), &plan()).expect_err("an unrecognised verdict must not pass");
        assert!(missing.contains("m9") && unknown.contains("Failure"), "{missing}\n{unknown}");
    }

    #[test]
    #[validates(spec::EveryGroupRunsBeforeSurvivorsAreReported)]
    fn every_group_runs_before_survivors_are_reported() {
        let mut groups: BTreeMap<TestPlan, Vec<String>> = BTreeMap::new();
        groups.insert(TestPlan::Traced(vec!["a".to_string()]), vec!["m1".to_string()]);
        groups.insert(TestPlan::FullSuite, vec!["m2".to_string()]);
        let mut ran = Vec::new();
        let survivors = run_groups_with(&groups, |index, plan, names| {
            ran.push((index, names.to_vec()));
            Ok(vec![Survivor { name: names[0].clone(), verdict: "MissedMutant".to_string(), plan: plan.clone() }])
        })
        .expect("survivors are not engine failures");
        let indices: Vec<usize> = ran.iter().map(|(i, _)| *i).collect();
        assert_eq!((indices, survivors.len()), (vec![0, 1], 2), "both groups ran, each with its own index, and both survivors were kept");
    }
}
