//! `#[spec("...")]` attaches a foreign ID as a doc alias and compiles.

use lid_rs::Spec;

/// The system shall accept a foreign spec key.
#[derive(Spec)]
#[lid_rs::spec("SOC2-CC6.1-003")]
struct AuditedClaim;

fn main() {}
