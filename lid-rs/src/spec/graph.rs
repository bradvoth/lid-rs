//! Claims for the intent-graph slice. Derived from
//! `docs/intent/intent-graph/lld.md`.

use lid_rs::Spec;

/// When a spec registered by the current crate has no implementation edge,
/// the implementer check shall fail, naming the spec and its source location.
#[derive(Spec)]
pub struct UncitedSpecsFailTheGraphCheck;

/// When a spec registered by the current crate has no validation edge, the
/// validation check shall fail, naming the spec and its source location.
#[derive(Spec)]
pub struct UnvalidatedSpecsFailTheGraphCheck;

/// When the canary triple is absent from the registries, each graph check
/// shall fail before asserting anything over them.
#[derive(Spec)]
pub struct GraphChecksRequireThePresentCanary;

/// When a spec was registered by a different crate in the same binary, the
/// current crate's graph checks shall not fail on it.
#[derive(Spec)]
pub struct GraphChecksScopeToTheCurrentCrate;

/// When every current-crate spec has at least one edge in the checked set,
/// the graph check shall pass.
#[derive(Spec)]
pub struct CoveredGraphsPassTheGraphCheck;
