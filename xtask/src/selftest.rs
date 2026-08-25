//! The gate's self-test: every check that cannot be demonstrated by a lib
//! test gets a fixture crate violating exactly one gate, and this module
//! asserts the gate fails on it with the expected diagnostic
//! (`docs/intent/xtask/lld.md § Gate self-test`, HLD Goal 3).

use std::path::{Path, PathBuf};
use std::process::Command;

use lid::implements;

use crate::mutants::cargo_command;
use crate::workspace_root;
use crate::spec;

/// Which gate command a fixture is expected to fail.
#[derive(Debug, Clone, Copy)]
pub enum Gate {
    /// `cargo check` (check 4).
    Check,
    /// `cargo clippy --all-targets -- -D warnings` (checks 3, 6–9, retirement).
    Clippy,
    /// `cargo doc` with broken-intra-doc-links denied (check 2).
    Doc,
    /// `cargo test --doc` (check 5).
    Doctest,
    /// `cargo mutants` (check 12).
    Mutants,
}

/// One fixture: a crate under `xtask/fixtures/<name>` that must fail `gate`
/// with a diagnostic containing `expect`.
#[derive(Debug)]
pub struct Fixture {
    /// Directory name under `xtask/fixtures/`.
    pub name: &'static str,
    /// The gate expected to catch it.
    pub gate: Gate,
    /// Substring the gate's output must contain.
    pub expect: &'static str,
    /// Whether the synthesized crate depends on `lid`.
    pub needs_lid: bool,
}

/// The fixture table: one row per gate-failure demonstration.
pub const FIXTURES: &[Fixture] = &[
    Fixture { name: "broken_doc_link", gate: Gate::Doc, expect: "unresolved link", needs_lid: false },
    Fixture { name: "missing_docs", gate: Gate::Clippy, expect: "missing documentation", needs_lid: false },
    Fixture { name: "skeleton_incoherence", gate: Gate::Check, expect: "mismatched types", needs_lid: false },
    Fixture { name: "broken_example", gate: Gate::Doctest, expect: "FAILED", needs_lid: false },
    Fixture { name: "swallowed_case", gate: Gate::Clippy, expect: "wildcard matches known variants", needs_lid: false },
    Fixture { name: "undeclared_decision", gate: Gate::Clippy, expect: "cognitive complexity", needs_lid: false },
    Fixture { name: "flag_argument", gate: Gate::Clippy, expect: "bool", needs_lid: false },
    Fixture { name: "inlined_concept", gate: Gate::Clippy, expect: "too many lines", needs_lid: false },
    Fixture { name: "retired_spec", gate: Gate::Clippy, expect: "deprecated", needs_lid: true },
    Fixture { name: "vacuous_test", gate: Gate::Mutants, expect: "MISSED", needs_lid: false },
];

/// Runs every fixture, reporting all failures at once.
#[implements(spec::EveryGateFixtureFailsItsGate)]
pub fn run_all() -> Result<(), String> {
    let failures: Vec<String> = FIXTURES
        .iter()
        .filter_map(|f| check_fixture(f).err().map(|e| format!("{}: {e}", f.name)))
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!("gate self-test failures:\n{}", failures.join("\n")))
    }
}

/// Runs a single fixture by name (used by the check-12 validation, which
/// pins the vacuous-test demonstration on its own).
#[implements(spec::SurvivingMutantsFailTheGate)]
pub fn run_fixture(name: &str) -> Result<(), String> {
    let fixture = FIXTURES
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("no fixture named `{name}`"))?;
    check_fixture(fixture)
}

/// Synthesizes and gates one fixture.
fn check_fixture(fixture: &Fixture) -> Result<(), String> {
    let dir = synthesize(fixture)?;
    let mut command = gate_command(fixture.gate);
    command.current_dir(&dir);
    expect_gate_failure(&mut command, fixture.expect)
}

/// Synthesizes the detached crate for a fixture under
/// `target/gate-selftest/<name>` and returns its directory.
fn synthesize(fixture: &Fixture) -> Result<PathBuf, String> {
    let root = workspace_root()?;
    let dir = root.join("target/gate-selftest").join(fixture.name);
    let write = |path: PathBuf, content: &str| {
        std::fs::write(&path, content).map_err(|e| format!("writing {}: {e}", path.display()))
    };
    std::fs::create_dir_all(dir.join("src"))
        .map_err(|e| format!("creating {}: {e}", dir.display()))?;
    write(dir.join("Cargo.toml"), &manifest(fixture, &root))?;
    let clippy_config = std::fs::read_to_string(root.join("clippy.toml"))
        .map_err(|e| format!("reading clippy.toml: {e}"))?;
    write(dir.join("clippy.toml"), &clippy_config)?;
    let source_path = root
        .join("xtask/fixtures")
        .join(fixture.name)
        .join("src/lib.rs");
    let source = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("reading {}: {e}", source_path.display()))?;
    write(dir.join("src/lib.rs"), &source)?;
    Ok(dir)
}

/// The synthesized crate's manifest: the workspace's lint set inlined, a
/// `[workspace]` table so the parent workspace is not adopted, and `lid` as a
/// path dependency where the fixture cites specs.
fn manifest(fixture: &Fixture, root: &Path) -> String {
    let lid_dep = if fixture.needs_lid {
        format!("lid = {{ path = {:?} }}\n", root.join("lid"))
    } else {
        String::new()
    };
    format!(
        r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2024"

[dependencies]
{lid_dep}
[workspace]

[lints.rust]
missing_docs = "deny"

[lints.rustdoc]
broken_intra_doc_links = "deny"

[lints.clippy]
cognitive_complexity = "warn"
fn_params_excessive_bools = "warn"
too_many_lines = "warn"
wildcard_enum_match_arm = "deny"
missing_docs_in_private_items = "warn"
"#,
        name = fixture.name,
        lid_dep = lid_dep,
    )
}

/// The gate command for a fixture kind.
fn gate_command(gate: Gate) -> Command {
    let mut command = cargo_command();
    match gate {
        Gate::Check => {
            command.args(["check", "--all-targets"]);
        }
        Gate::Clippy => {
            command.args(["clippy", "--all-targets", "--", "-D", "warnings"]);
        }
        Gate::Doc => {
            command.args(["doc", "--no-deps"]);
            command.env("RUSTDOCFLAGS", "-D rustdoc::broken_intra_doc_links");
        }
        Gate::Doctest => {
            command.args(["test", "--doc"]);
        }
        Gate::Mutants => {
            command.arg("mutants");
        }
    }
    command
}

/// Runs the gate and demands it fail with the expected diagnostic.
#[implements(spec::EveryGateFixtureFailsItsGate)]
fn expect_gate_failure(command: &mut Command, expect: &str) -> Result<(), String> {
    let output = command
        .output()
        .map_err(|e| format!("spawning gate: {e}"))?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() {
        return Err("gate PASSED on a fixture that must fail it".to_string());
    }
    if !text.contains(expect) {
        return Err(format!(
            "gate failed, but without the expected diagnostic `{expect}`; output:\n{text}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid::validates;
    use std::sync::Mutex;

    /// Serializes the tests that create and inspect fixture directories, so
    /// one test's cleanup cannot race another's synthesis.
    static FIXTURE_DIRS: Mutex<()> = Mutex::new(());

    /// Removes a fixture's synthesized source so a later existence assertion
    /// proves *this* run synthesized it, not a stale earlier one.
    fn forget_fixture(name: &str) {
        let dir = workspace_root()
            .expect("workspace root resolves")
            .join("target/gate-selftest")
            .join(name)
            .join("src");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    #[validates(spec::SurvivingMutantsFailTheGate)]
    fn surviving_mutants_fail_the_gate() {
        let _serialized = FIXTURE_DIRS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        forget_fixture("vacuous_test");
        run_fixture("vacuous_test").expect(
            "the vacuous-test fixture must leave a surviving mutant, and the gate must report it as failure",
        );
        let synthesized = workspace_root()
            .expect("workspace root resolves")
            .join("target/gate-selftest/vacuous_test/src/lib.rs");
        assert!(synthesized.exists(), "run_fixture must actually synthesize and run the fixture");
    }

    #[test]
    #[validates(spec::EveryGateFixtureFailsItsGate)]
    fn every_gate_fixture_fails_its_gate() {
        let _serialized = FIXTURE_DIRS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for fixture in FIXTURES {
            forget_fixture(fixture.name);
        }
        run_all().expect("every fixture must fail its designated gate with its designated diagnostic");
        let root = workspace_root().expect("workspace root resolves");
        for fixture in FIXTURES {
            let dir = root.join("target/gate-selftest").join(fixture.name).join("src");
            assert!(dir.exists(), "run_all must have exercised {}", fixture.name);
        }
    }

    // Validates both claims: the lookup is part of run_fixture's role in the
    // check-12 demonstration, so this test must sit in the narrowed test set
    // of mutants in run_fixture — a lesson taught by a surviving mutant whose
    // killing test cited only the other claim.
    #[test]
    #[validates(spec::EveryGateFixtureFailsItsGate)]
    fn a_passing_gate_fails_the_selftest() {
        let mut command = cargo_command();
        command.arg("--version");
        let result = expect_gate_failure(&mut command, "anything");
        assert!(
            result.is_err_and(|e| e.contains("PASSED")),
            "a gate that passes on a violation fixture must fail the self-test"
        );
    }

    #[test]
    #[validates(spec::SurvivingMutantsFailTheGate, spec::EveryGateFixtureFailsItsGate)]
    fn unknown_fixture_names_are_rejected() {
        let result = run_fixture("no_such_fixture");
        assert!(
            result.is_err_and(|e| e.contains("no fixture")),
            "asking for an unknown fixture must be an error, not a silent pass"
        );
    }
}
