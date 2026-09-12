//! Claims for the `lid-rs-macros` slice. Derived from
//! `lid-rs-macros/src/lld.md`.
//!
//! The implementing code lives in `lid-rs-macros`, a proc-macro crate that links
//! into no target binary and therefore cannot carry citations itself; its
//! implementation edges are hand-authored at the re-export site in `lid-rs`'s
//! crate root (the standing exception for proc-macro crates).
//!
//! The last three claims are the async refusal and the shape pins
//! (`lid-rs-macros/src/lld.md`, "Citing an `async fn` is refused" and "The
//! shape pins"). They are three and not four because the document makes the
//! refusal one unconditional act shared by both verbs — `refuse_async` runs
//! before `cite_fn` wraps anything, whichever attribute called it — so one
//! claim names both attributes and a validator that showed the two verbs
//! differing would falsify it. The pins are two claims because their two
//! behaviours fail separately: an expansion that dropped an attribute of the
//! pinned item would falsify [`PinsExpandToTheirItemUnchanged`] while every
//! argument was still refused, and one that accepted and ignored an argument
//! would falsify [`PinsRefuseArguments`] while every item still came through
//! unchanged.
//!
//! None of the six claims above is falsified by the change, so none is renamed.
//! [`MalformedCitationsFailToCompile`] enumerates a closed list of five
//! malformations, each of which still fails; the async fn and the pin argument
//! are two new refusals with two new claims, not a sixth and seventh case of
//! that one. The edge-registration claims describe what a compiled program's
//! registry holds, and a refused `async fn` compiles into no program.
//!
//! Each trigger links the attribute as `lid-rs` re-exports it —
//! [`implements`](crate::implements), [`flow`](crate::flow),
//! [`leaf`](crate::leaf) — because the items that keep the claims,
//! `refuse_async` and `passthrough_pin` in `lid_rs_macros::expand`, are private
//! to a crate this one cannot link into by intra-doc link. The hand edges in
//! `lid-rs/src/lib.rs` name that module as the implementer, as the document's
//! "What Phase 3 writes in the companion" spells them. The three new claims are
//! written in the controlled language and carry no `#[lid(free)]` mark; the
//! six earlier claims keep theirs, since rewording them is the slice's own
//! burn-down and not this change's.

use lid_rs::Spec;

/// When a unit struct derives `Spec`, its `NAME` shall be the definition-site
/// module path joined with the struct identifier.
#[derive(Spec)]
#[lid(free)]
pub struct DerivedSpecsCarryTheirDefinitionPath;

/// When a unit struct derives `Spec`, a `SpecMeta` registration for it shall
/// appear in [`crate::SPECS`].
#[derive(Spec)]
#[lid(free)]
pub struct DerivedSpecsRegisterIntoSpecs;

/// When an item carries `#[implements(...)]`, one [`crate::IMPLEMENTATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing item.
#[derive(Spec)]
#[lid(free)]
pub struct ImplementsCitationsRegisterEdges;

/// When a test fn carries `#[validates(...)]`, one [`crate::VALIDATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing test.
#[derive(Spec)]
#[lid(free)]
pub struct ValidatesCitationsRegisterEdges;

/// When `implements_module!` is invoked inside a module, the registered
/// edge's item shall be the enclosing module path.
#[derive(Spec)]
#[lid(free)]
pub struct ModuleCitationsTraceByContainment;

/// When a citation is malformed — an unresolvable path, a type that does not
/// implement `Spec`, an empty citation list, a generic path, or a derive
/// target that is not a unit struct — compilation shall fail.
#[derive(Spec)]
#[lid(free)]
pub struct MalformedCitationsFailToCompile;

// ---- Citing an `async fn` is refused -----------------------------------------

/// When [`implements`](crate::implements) or [`validates`](crate::validates)
/// is written on an `async fn`, compilation shall fail with one diagnostic
/// spanned on the `async` keyword, the same for either attribute.
#[derive(Spec)]
pub struct AsyncCitationsFailToCompile;

// ---- The shape pins ----------------------------------------------------------

/// When [`flow`](crate::flow) or [`leaf`](crate::leaf) with no arguments is
/// written on a fn, the expansion shall be that fn's tokens unchanged.
#[derive(Spec)]
pub struct PinsExpandToTheirItemUnchanged;

/// When [`flow`](crate::flow) or [`leaf`](crate::leaf) is given a non-empty
/// argument list, compilation shall fail with one diagnostic spanned on those
/// arguments.
#[derive(Spec)]
pub struct PinsRefuseArguments;
