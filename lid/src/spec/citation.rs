//! Claims for the citation-macro slice. Derived from
//! `docs/intent/macros/lld.md`.
//!
//! The implementing code lives in `lid-macros`, a proc-macro crate that links
//! into no target binary and therefore cannot carry citations itself; its
//! implementation edges are hand-authored at the re-export site in `lid`'s
//! crate root (the standing exception for proc-macro crates).

use lid::Spec;

/// When a unit struct derives `Spec`, its `NAME` shall be the definition-site
/// module path joined with the struct identifier.
#[derive(Spec)]
pub struct DerivedSpecsCarryTheirDefinitionPath;

/// When a unit struct derives `Spec`, a `SpecMeta` registration for it shall
/// appear in [`crate::SPECS`].
#[derive(Spec)]
pub struct DerivedSpecsRegisterIntoSpecs;

/// When an item carries `#[implements(...)]`, one [`crate::IMPLEMENTATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing item.
#[derive(Spec)]
pub struct ImplementsCitationsRegisterEdges;

/// When a test fn carries `#[validates(...)]`, one [`crate::VALIDATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing test.
#[derive(Spec)]
pub struct ValidatesCitationsRegisterEdges;

/// When `implements_module!` is invoked inside a module, the registered
/// edge's item shall be the enclosing module path.
#[derive(Spec)]
pub struct ModuleCitationsTraceByContainment;

/// When a citation is malformed — an unresolvable path, a type that does not
/// implement `Spec`, an empty citation list, a generic path, or a derive
/// target that is not a unit struct — compilation shall fail.
#[derive(Spec)]
pub struct MalformedCitationsFailToCompile;
