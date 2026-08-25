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
    let _ = args;
    todo!()
}

/// The workspace's configured scope from `[workspace.metadata.lid]`, read via
/// `cargo metadata` so no TOML parser is needed.
fn configured_scope() -> Result<Scope, String> {
    todo!()
}

/// For diff scope, writes `git diff <base>` to a file under `target/` and
/// returns its path; `None` for full scope.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn write_diff_file(scope: &Scope) -> Result<Option<PathBuf>, String> {
    let _ = scope;
    todo!()
}

/// Enumerates mutants, restricted by the diff when one is given.
#[implements(spec::DiffScopePassesThroughToTheEngine)]
fn list_mutants(diff: Option<&Path>) -> Result<Vec<Mutant>, String> {
    let _ = diff;
    todo!()
}

/// Groups mutant names by their test plan, so each distinct test set gets one
/// engine run. `BTreeMap` keeps run order deterministic.
fn group_by_plan(mutants: &[Mutant]) -> BTreeMap<TestPlan, Vec<String>> {
    let _ = mutants;
    todo!()
}

/// Runs one engine invocation per group, failing if any mutant survives.
#[implements(spec::SurvivingMutantsFailTheGate)]
fn run_groups(
    groups: &BTreeMap<TestPlan, Vec<String>>,
    diff: Option<&Path>,
) -> Result<(), String> {
    let _ = (groups, diff);
    todo!()
}

/// The `cargo mutants` argument list for one group — pure, so the pass-through
/// of diff and test filters is unit-testable.
fn group_args(names: &[String], plan: &TestPlan, diff: Option<&Path>) -> Vec<String> {
    let _ = (names, plan, diff);
    todo!()
}

/// Escapes a mutant name for use inside the `-F` anchored alternation.
fn regex_escape(name: &str) -> String {
    let _ = name;
    todo!()
}

/// Runs a command, returning stdout or a message including stderr.
fn capture(command: &mut Command) -> Result<String, String> {
    let _ = command;
    todo!()
}
