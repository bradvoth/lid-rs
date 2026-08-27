//! Claims for the subcommand shell and check 12 (`docs/intent/cargo-lid-rs/lld.md`).

use lid_rs::Spec;

// ---- Subcommand shell -------------------------------------------------------

/// When the first argument is the subcommand name cargo inserts when running
/// an external subcommand (`lid-rs`), the shell shall discard it and dispatch
/// on the remaining arguments, so the cargo, alias, and direct invocation
/// forms behave identically.
#[derive(Spec)]
pub struct CargoInsertedSubcommandNameIsDiscarded;

/// When the subcommand is missing or unknown, the shell shall fail with a
/// usage message naming the subcommands it accepts.
#[derive(Spec)]
pub struct UnknownSubcommandsFailWithUsage;

/// When the tool locates the project it operates on, it shall use the
/// `workspace_root` and `target_directory` reported by `cargo metadata`, so
/// invocation from any directory inside the project behaves as invocation
/// from its root.
#[derive(Spec)]
pub struct TheProjectRootComesFromCargoMetadata;

// ---- Scope ------------------------------------------------------------------

/// When `mutation_scope` is absent from `[workspace.metadata.lid_rs]`, the
/// tool shall read it from the root package's `[package.metadata.lid_rs]`;
/// when it is absent there too, the scope shall be `diff` against `main`.
#[derive(Spec)]
pub struct MutationScopeFallsBackFromWorkspaceToPackageToDiff;

/// When `--full` is given, the scope shall be the whole tree regardless of
/// configuration; when `--diff-base <ref>` is given, the scope shall be the
/// diff against that ref; any other flag shall be rejected by name.
#[derive(Spec)]
pub struct ScopeFlagsOverrideTheConfiguredScope;

/// When mutation scope is `diff`, the generated diff shall be passed through
/// to the mutation engine's `--in-diff`.
#[derive(Spec)]
pub struct DiffScopePassesThroughToTheEngine;

// ---- Registry collection ----------------------------------------------------

/// When registries are collected for mutation planning, each crate's edges
/// shall come from that crate's own `--lib` test binary, the only binary its
/// validation edges link into.
#[derive(Spec)]
pub struct ValidationEdgesComeFromTheOwningCrateTestBinary;

/// When a workspace member declares no library target, registry collection
/// shall skip it rather than run `cargo test --lib` against it.
#[derive(Spec)]
pub struct MembersWithoutALibraryTargetAreSkipped;

// ---- Mutant → test-set mapping and execution --------------------------------

/// When a mutant's function carries implementation edges, its mutation run
/// shall use exactly the tests validating the specs those edges cite; when
/// that set is empty, the full suite shall run instead, so zero reachable
/// tests can never mean zero tests run.
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

/// When the engine's run for a group reports outcomes, only the mutants the
/// group selected shall be judged from it; any other mutant the engine
/// included shall be ignored there and judged in its own group.
#[derive(Spec)]
pub struct AMutantsVerdictComesFromItsOwnGroupsRun;

/// When the engine's outcomes carry no verdict, or an unrecognised one, for a
/// mutant the group selected, the mutants command shall fail naming the
/// mutant, never treat it as caught.
#[derive(Spec)]
pub struct AnEngineRunWithoutAVerdictIsAFailure;

/// When a group has survivors, the remaining groups shall still run, and the
/// failure shall name every survivor with the tests it survived.
#[derive(Spec)]
pub struct EveryGroupRunsBeforeSurvivorsAreReported;
