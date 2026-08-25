//! Check 12 orchestration: enumerate mutants, join them to their validating
//! tests through the registry, and run `cargo-mutants` per test-set group
//! (`docs/intent/xtask/lld.md`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use lid::implements;

use crate::mapping::{TestPlan, plan_for_mutant};
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

/// How much of the tree to mutate, from `[workspace.metadata.lid]` or flags.
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

/// Runs the mutants task: scope, list, plan, execute.
pub fn run(args: &[String]) -> Result<(), String> {
    let scope = parse_scope(args)?;
    let diff = write_diff_file(&scope)?;
    let mutants = list_mutants(diff.as_deref())?;
    let groups = group_by_plan(&mutants);
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

/// The workspace's configured scope from `[workspace.metadata.lid]`, read via
/// `cargo metadata` so no TOML parser is needed. Diff scope defaults its base
/// to `main`; CI overrides with `--diff-base`.
fn configured_scope() -> Result<Scope, String> {
    let out = capture(cargo_command().args(["metadata", "--format-version", "1", "--no-deps"]))?;
    let value: serde_json::Value =
        serde_json::from_str(&out).map_err(|e| format!("parsing cargo metadata: {e}"))?;
    let configured = value
        .pointer("/metadata/lid/mutation_scope")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("diff");
    if configured == "full" {
        Ok(Scope::Full)
    } else {
        Ok(Scope::Diff { base: "main".to_string() })
    }
}

/// For diff scope, writes `git diff <base>` to a file under `target/` and
/// returns its path; `None` for full scope.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn write_diff_file(scope: &Scope) -> Result<Option<PathBuf>, String> {
    let Scope::Diff { base } = scope else {
        return Ok(None);
    };
    let diff = capture(Command::new("git").args(["diff", base]))?;
    let path = PathBuf::from("target/lid-mutants.diff");
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

/// Parses the engine's JSON mutant list.
fn parse_mutants(json: &str) -> Result<Vec<Mutant>, String> {
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

/// Groups mutant names by their test plan, so each distinct test set gets one
/// engine run. `BTreeMap` keeps run order deterministic.
fn group_by_plan(mutants: &[Mutant]) -> BTreeMap<TestPlan, Vec<String>> {
    let mut groups: BTreeMap<TestPlan, Vec<String>> = BTreeMap::new();
    for mutant in mutants {
        let plan = plan_for_mutant(
            &mutant.file,
            &mutant.function,
            &lid::IMPLEMENTATIONS,
            &lid::VALIDATIONS,
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
    let mut args: Vec<String> = ["mutants", "--baseline", "skip", "-F"].map(String::from).into();
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
    use lid::validates;

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
