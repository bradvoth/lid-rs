//! `#[spec("...")]` attaches a foreign ID as a doc alias and compiles.
//!
//! The claim is marked `#[lid(free)]` because this fixture is about the foreign
//! key, not about the controlled language: holding it to a grammar it was not
//! written for would make it a claim about the system, which it is not.

use lid_rs::Spec;

/// The system shall accept a foreign spec key.
#[derive(Spec)]
#[lid(free)]
#[lid_rs::spec("SOC2-CC6.1-003")]
struct AuditedClaim;

fn main() {}
