//! Atomic claims for the xtask slice. Derived from `docs/intent/xtask/lld.md`.

use lid::Spec;

/// When a mutant's function carries implementation edges, its mutation run
/// shall use exactly the tests validating the specs those edges cite.
#[derive(Spec)]
pub struct TracedMutantsRunOnlyTheirValidatingTests;

/// When a mutant's function has no implementation edge, its test set shall be
/// the tests validating specs implemented in the same file, or the full suite
/// when none exist.
#[derive(Spec)]
pub struct UntracedMutantsFallBackToModuleTests;

/// When any mutant survives its test set, the mutants command shall report
/// failure.
#[derive(Spec)]
pub struct SurvivingMutantsFailTheGate;

/// When mutation scope is `diff`, the generated diff shall be passed through
/// to the mutation engine's `--in-diff`.
#[derive(Spec)]
pub struct DiffScopePassesThroughToTheEngine;

/// When the gate self-test runs, each fixture shall fail its designated gate
/// with its designated diagnostic, and a fixture whose gate passes shall fail
/// the self-test.
#[derive(Spec)]
pub struct EveryGateFixtureFailsItsGate;
