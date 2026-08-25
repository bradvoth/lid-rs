//! The gate's self-test: every check that cannot be demonstrated by a lib
//! test gets a fixture crate violating exactly one gate, and this module
//! asserts the gate fails on it with the expected diagnostic
//! (`docs/intent/xtask/lld.md § Gate self-test`, HLD Goal 3).

use std::path::PathBuf;
use std::process::Command;

use lid::implements;

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
    Fixture { name: "swallowed_case", gate: Gate::Clippy, expect: "wildcard-enum-match-arm", needs_lid: false },
    Fixture { name: "undeclared_decision", gate: Gate::Clippy, expect: "cognitive-complexity", needs_lid: false },
    Fixture { name: "flag_argument", gate: Gate::Clippy, expect: "fn-params-excessive-bools", needs_lid: false },
    Fixture { name: "inlined_concept", gate: Gate::Clippy, expect: "too-many-lines", needs_lid: false },
    Fixture { name: "retired_spec", gate: Gate::Clippy, expect: "deprecated", needs_lid: true },
    Fixture { name: "vacuous_test", gate: Gate::Mutants, expect: "MISSED", needs_lid: false },
];

/// Runs every fixture, reporting all failures at once.
#[implements(spec::EveryGateFixtureFailsItsGate)]
pub fn run_all() -> Result<(), String> {
    todo!()
}

/// Runs a single fixture by name (used by the check-12 validation, which
/// pins the vacuous-test demonstration on its own).
#[implements(spec::SurvivingMutantsFailTheGate)]
pub fn run_fixture(name: &str) -> Result<(), String> {
    let _ = name;
    todo!()
}

/// Synthesizes the detached crate for a fixture under
/// `target/gate-selftest/<name>` and returns its directory.
fn synthesize(fixture: &Fixture) -> Result<PathBuf, String> {
    let _ = fixture;
    todo!()
}

/// The gate command for a fixture, run in `dir`.
fn gate_command(gate: Gate, dir: &PathBuf) -> Command {
    let _ = (gate, dir);
    todo!()
}

/// Runs the gate and demands it fail with the expected diagnostic.
fn expect_gate_failure(command: &mut Command, expect: &str) -> Result<(), String> {
    let _ = (command, expect);
    todo!()
}
