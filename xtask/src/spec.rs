//! Atomic claims for the xtask slice. Derived from `docs/intent/xtask/lld.md`.

use lid_rs::Spec;

/// When the gate self-test runs, each fixture shall fail its designated gate
/// with its designated diagnostic, and a fixture whose gate passes shall fail
/// the self-test.
#[derive(Spec)]
pub struct EveryGateFixtureFailsItsGate;
