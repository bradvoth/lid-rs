//! `#[spec("...")]` attaches a foreign ID as a doc alias and compiles.

use lid::Spec;

/// The system shall accept a foreign spec key.
#[derive(Spec)]
#[lid::spec("SOC2-CC6.1-003")]
struct AuditedClaim;

fn main() {}
