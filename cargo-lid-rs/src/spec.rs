//! Claims for the subcommand shell and check 12 (`src/lld.md`), and the retired
//! names of other slices' claims, kept while their deprecation windows stand.

use lid_rs::Spec;

// ---- Subcommand shell -------------------------------------------------------

/// When the first argument is the subcommand name cargo inserts when running
/// an external subcommand (`lid-rs`), the shell shall discard it and dispatch
/// on the remaining arguments, so the cargo, alias, and direct invocation
/// forms behave identically.
#[derive(Spec)]
#[lid(free)]
pub struct CargoInsertedSubcommandNameIsDiscarded;

/// When the subcommand is missing or unknown, the shell shall fail with a
/// usage message naming the subcommands it accepts.
#[derive(Spec)]
#[lid(free)]
pub struct UnknownSubcommandsFailWithUsage;

/// When the tool locates the project it operates on, it shall use the
/// `workspace_root` and `target_directory` reported by `cargo metadata`, so
/// invocation from any directory inside the project behaves as invocation
/// from its root.
#[derive(Spec)]
#[lid(free)]
pub struct TheProjectRootComesFromCargoMetadata;

// ---- Scope ------------------------------------------------------------------

/// When `mutation_scope` is absent from `[workspace.metadata.lid_rs]`, the
/// tool shall read it from the root package's `[package.metadata.lid_rs]`;
/// when it is absent there too, the scope shall be `diff` against `main`.
#[derive(Spec)]
#[lid(free)]
pub struct MutationScopeFallsBackFromWorkspaceToPackageToDiff;

/// When `--full` is given, the scope shall be the whole tree regardless of
/// configuration; when `--diff-base <ref>` is given, the scope shall be the
/// diff against that ref; any other flag shall be rejected by name.
#[derive(Spec)]
#[lid(free)]
pub struct ScopeFlagsOverrideTheConfiguredScope;

/// When mutation scope is `diff`, the generated diff shall be passed through
/// to the mutation engine's `--in-diff`.
#[derive(Spec)]
#[lid(free)]
pub struct DiffScopePassesThroughToTheEngine;

// ---- Registry collection ----------------------------------------------------

/// When registries are collected for mutation planning, each crate's edges
/// shall come from that crate's own `--lib` test binary, the only binary its
/// validation edges link into.
#[derive(Spec)]
#[lid(free)]
pub struct ValidationEdgesComeFromTheOwningCrateTestBinary;

/// When a workspace member declares no library target, registry collection
/// shall skip it rather than run `cargo test --lib` against it.
#[derive(Spec)]
#[lid(free)]
pub struct MembersWithoutALibraryTargetAreSkipped;

// ---- Mutant → test-set mapping and execution --------------------------------

/// When a mutant's function carries implementation edges, its mutation run
/// shall use exactly the tests validating the specs those edges cite; when
/// that set is empty, the full suite shall run instead, so zero reachable
/// tests can never mean zero tests run.
#[derive(Spec)]
#[lid(free)]
pub struct TracedMutantsRunOnlyTheirValidatingTests;

/// When a mutant has no function, or its function has no implementation
/// edge, its test set shall be the tests validating specs implemented in the
/// same file, or the full suite when none exist.
#[derive(Spec)]
#[lid(free)]
pub struct UntracedMutantsFallBackToModuleTests;

/// When any mutant survives its test set, the mutants command shall report
/// failure.
#[derive(Spec)]
#[lid(free)]
pub struct SurvivingMutantsFailTheGate;

/// When the engine's run for a group reports outcomes, only the mutants the
/// group selected shall be judged from it; any other mutant the engine
/// included shall be ignored there and judged in its own group.
#[derive(Spec)]
#[lid(free)]
pub struct AMutantsVerdictComesFromItsOwnGroupsRun;

/// When the engine's outcomes carry no verdict, or an unrecognised one, for a
/// mutant the group selected, the mutants command shall fail naming the
/// mutant, never treat it as caught.
#[derive(Spec)]
#[lid(free)]
pub struct AnEngineRunWithoutAVerdictIsAFailure;

/// When a group has survivors, the remaining groups shall still run, and the
/// failure shall name every survivor with the tests it survived.
#[derive(Spec)]
#[lid(free)]
pub struct EveryGroupRunsBeforeSurvivorsAreReported;

// ---- Retired names, kept while their deprecation windows stand ----------------
//
// These alias claims of other slices, which now live beside their own code.
// They register nothing; a citation of one warns with its replacement.





/// The name [`crate::coach::spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools`] carried
/// while the coach's tool set was three. The alias registers no claim, so the
/// graph sees only the claim it points at; every citation of this name warns
/// with its replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools"]
pub type TheCoachDeclaresExactlyTheReadDraftAndAskTools = crate::coach::spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools;






// The names below are the ones the `phase` slice's claims carried before a
// proc-macro crate's slice had a companion. Each alias registers no claim,
// so the graph sees only the claim it points at; every citation of the old
// name warns with its replacement, and those citations are the later phases'
// work list.

/// The name [`crate::phase::spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles`] carried while the
/// slice's crate was the only crate a phase could write.
#[deprecated = "replaced by PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles"]
pub type PhaseTwoMayWriteOnlyTheSlicesSpecFiles = crate::phase::spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles;

/// The name [`crate::phase::spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy`] carried
/// while there was one crate to be outside of.
#[deprecated = "replaced by PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy"]
pub type PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy = crate::phase::spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy;

// The four names below are the ones the `phase` slice's path-table claims
// carried while a phase's allowed set named the slice's *directory*. Colocation
// puts the slice's document, its claims file and its acceptance file in that
// directory, so each of those names asserted a permission over artifacts no
// phase may write. Older aliases of the first two names are gone rather than
// re-pointed: no citation named them, and an alias no citation names is
// deleted by the next Phase 2 on the slice. That has now happened twice — the
// pair from before a companion, and the pair these names carried while a
// phase's set named the slice's directory rather than the code in it, retired
// when `lld/phase--crate-root-paths` narrowed both to exclude another slice's
// code.

/// The name [`crate::phase::spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims`]
/// carried while a phase's set named the slice's directory in the companion
/// rather than the code in it.
#[deprecated = "replaced by PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims"]
pub type PhasesThreeAndFourMayWriteTheCompanionsSliceModuleAndLibraryRoot =
    crate::phase::spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims;

/// The name [`crate::phase::spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims`]
/// carried while a phase's set named the slice's directory in the companion
/// rather than the code in it.
#[deprecated = "replaced by PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims"]
pub type PhasesFiveAndSevenMayWriteTheCompanionsSliceModuleAndUiFixtures =
    crate::phase::spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims;

