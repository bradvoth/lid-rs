//! Claims for the registry slice. Derived from `docs/intent/registry/lld.md`.

use lid_rs::Spec;

/// When a binary links a crate containing registration statics, iterating
/// [`crate::SPECS`], [`crate::IMPLEMENTATIONS`], and [`crate::VALIDATIONS`]
/// shall yield those registrations.
#[derive(Spec)]
pub struct LinkedRegistrationsAreEnumerable;

/// When the canary spec, implementation edge, and validation edge are all
/// enumerable in the registries, the canary's presence check shall report
/// `true`.
#[derive(Spec)]
pub struct CanaryConfirmsRegistryPresence;

/// When any entry of the canary triple is missing from its registry, the
/// canary's presence check shall report `false`.
#[derive(Spec)]
pub struct CanaryDetectsAStrippedRegistry;
