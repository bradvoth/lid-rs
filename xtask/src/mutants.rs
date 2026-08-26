//! Check 12 orchestration: enumerate mutants, join them to their validating
//! tests through the dumped registries, and run `cargo-mutants` per test-set
//! group (`docs/intent/xtask/lld.md`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use lid_rs::implements;

use crate::mapping::{EdgeRecord, TestPlan, plan_for_mutant};
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

/// The dumped registries the planner joins over.
#[derive(Debug, Default)]
pub struct Registry {
    /// Implementation edges across the workspace's crates.
    pub impls: Vec<EdgeRecord>,
    /// Validation edges across the workspace's crates.
    pub validations: Vec<EdgeRecord>,
}

/// How much of the tree to mutate, from `[workspace.metadata.lid_rs]` or flags.
#[derive(Debug, PartialEq, Eq)]
pub enum Scope {
    /// Only code touched by the diff against a base ref.
    Diff {
        /// The git ref the diff is taken against.
        base: String,
    },
    /// The whole tree.
    Full,
}

/// Runs the mutants task: scope, list, collect registries, plan, execute.
pub fn run(args: &[String]) -> Result<(), String> {
    let scope = parse_scope(args)?;
    let diff = write_diff_file(&scope)?;
    let mutants = list_mutants(diff.as_deref())?;
    let registry = collect_registries()?;
    let groups = group_by_plan(&mutants, &registry);
    run_groups(&groups, diff.as_deref())
}

/// Parses `--full` / `--diff-base <ref>` over the configured default scope.
fn parse_scope(args: &[String]) -> Result<Scope, String> {
    let mut scope = configured_scope()?;
    let mut rest = args;
    while let Some((flag, tail)) = rest.split_first() {
        (scope, rest) = apply_flag(flag, tail)?;
    }
    Ok(scope)
}

/// Applies one flag, returning the new scope and the unconsumed arguments.
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

/// The workspace's configured scope from `[workspace.metadata.lid_rs]`, read via
/// `cargo metadata` so no TOML parser is needed. Diff scope defaults its base
/// to `main`; CI overrides with `--diff-base`.
fn configured_scope() -> Result<Scope, String> {
    let configured = metadata()?
        .pointer("/metadata/lid_rs/mutation_scope")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("diff")
        .to_string();
    if configured == "full" {
        Ok(Scope::Full)
    } else {
        Ok(Scope::Diff { base: "main".to_string() })
    }
}

/// The parsed `cargo metadata` document.
fn metadata() -> Result<serde_json::Value, String> {
    let out = capture(cargo_command().args(["metadata", "--format-version", "1", "--no-deps"]))?;
    serde_json::from_str(&out).map_err(|e| format!("parsing cargo metadata: {e}"))
}

/// For diff scope, writes `git diff <base>` to a file under `target/` and
/// returns its path; `None` for full scope.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn write_diff_file(scope: &Scope) -> Result<Option<PathBuf>, String> {
    let Scope::Diff { base } = scope else {
        return Ok(None);
    };
    let root = crate::workspace_root()?;
    let diff = capture(Command::new("git").args(["diff", base]).current_dir(&root))?;
    let path = root.join("target/lid-mutants.diff");
    std::fs::write(&path, &diff).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok(Some(path))
}

/// Enumerates mutants, restricted by the diff when one is given.
fn list_mutants(diff: Option<&Path>) -> Result<Vec<Mutant>, String> {
    let out = capture(cargo_command().args(list_args(diff)))?;
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

/// Collects the registries of every workspace crate by running each crate's
/// own `--lib` test binary in dump mode — the only binary whose `#[validates]`
/// edges exist (README §5.2). Crates without the graph checks dump nothing
/// and contribute nothing.
#[implements(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
pub fn collect_registries() -> Result<Registry, String> {
    let mut registry = Registry::default();
    for package in workspace_packages()? {
        let dumped = dump_registry(&package)?;
        registry.impls.extend(dumped.impls);
        registry.validations.extend(dumped.validations);
    }
    Ok(registry)
}

/// Names of the workspace's member packages.
fn workspace_packages() -> Result<Vec<String>, String> {
    let doc = metadata()?;
    let packages = doc
        .pointer("/packages")
        .and_then(serde_json::Value::as_array)
        .ok_or("cargo metadata has no packages array")?;
    Ok(packages
        .iter()
        .filter_map(|p| p.pointer("/name").and_then(serde_json::Value::as_str))
        .map(str::to_string)
        .collect())
}

/// One crate's registry, dumped by its `intent_graph!()` dump test.
#[implements(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
pub fn dump_registry(package: &str) -> Result<Registry, String> {
    let out = capture(
        cargo_command()
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

/// Runs one engine invocation per group, failing if any mutant survives.
#[implements(spec::SurvivingMutantsFailTheGate)]
fn run_groups(
    groups: &BTreeMap<TestPlan, Vec<String>>,
    diff: Option<&Path>,
) -> Result<(), String> {
    for (plan, names) in groups {
        let status = cargo_command()
            .args(group_args(names, plan, diff))
            .status()
            .map_err(|e| format!("running cargo mutants: {e}"))?;
        if !status.success() {
            return Err(format!(
                "mutation gate failed: a mutant in {names:?} survived its validating tests \
                 (plan {plan:?}) — the citation is decorative (README §4.3)"
            ));
        }
    }
    Ok(())
}

/// The `cargo mutants` argument list for one group — pure, so the pass-through
/// of diff and test filters is unit-testable.
fn group_args(names: &[String], plan: &TestPlan, diff: Option<&Path>) -> Vec<String> {
    let alternation: Vec<String> = names.iter().map(|n| regex_escape(n)).collect();
    // --test-workspace: a mutant in one crate (e.g. the proc-macro crate) is
    // often killed only by a dependent crate's tests; package-scoped testing
    // would let it survive unexercised.
    let mut args: Vec<String> =
        ["mutants", "--baseline", "skip", "--test-workspace", "true", "-F"].map(String::from).into();
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

/// The cargo binary this xtask was itself invoked through.
pub(crate) fn cargo_command() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
}

/// Runs a command, returning stdout or a message including stderr.
fn capture(command: &mut Command) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("running {command:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("non-utf8 command output: {e}"))
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
        let scope = Scope::Diff { base: "HEAD".to_string() };
        let path = write_diff_file(&scope)
            .expect("git diff HEAD must succeed in this repository")
            .expect("diff scope must yield a diff file");
        assert!(path.ends_with("lid-mutants.diff"), "{path:?}");
        assert!(path.exists(), "diff file must exist at {path:?}");
        assert_eq!(write_diff_file(&Scope::Full).expect("full scope never fails"), None);
    }

    #[test]
    #[validates(spec::SurvivingMutantsFailTheGate)]
    fn engine_failures_fail_the_gate() {
        let mut groups: BTreeMap<TestPlan, Vec<String>> = BTreeMap::new();
        groups.insert(TestPlan::FullSuite, vec!["any_mutant".to_string()]);
        let missing_diff = Path::new("target/definitely-missing.diff");
        let result = run_groups(&groups, Some(missing_diff));
        assert!(result.is_err(), "an engine failure must fail the mutants gate");
    }

    #[test]
    #[validates(spec::ValidationEdgesComeFromTheOwningCrateTestBinary)]
    fn registries_come_from_the_owning_crate_test_binary() {
        let registry = collect_registries().expect("collecting workspace registries must succeed");
        let lid_test_edge = registry.validations.iter().any(|e| {
            e.item == "lid_rs::canary::tests::canary_confirms_registry_presence"
        });
        assert!(
            lid_test_edge,
            "a cfg(test)-only lid validation edge must be present, proving the dump \
             came from lid's own test binary: {:#?}",
            registry.validations
        );
        let xtask_test_edge = registry.validations.iter().any(|e| e.item.starts_with("xtask::"));
        assert!(
            xtask_test_edge,
            "collect_registries must have walked every workspace crate, xtask included"
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
        let args = group_args(&["m(1)".to_string()], &plan, None);
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
        let full = group_args(&["m".to_string()], &TestPlan::FullSuite, None);
        let narrowed = full.iter().any(|a| a.starts_with("--cargo-test-arg"));
        assert!(!narrowed, "{full:?}");
    }
}
