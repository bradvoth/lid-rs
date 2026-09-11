//! Atomic claims for `cargo-lid-rs`, one file per slice: a slice's claims
//! are the specs registered from `src/spec/<slice>.rs`, which is how
//! `phase-check 5` finds them (`docs/intent/phase/lld.md`).

mod cargo_lid_rs;
mod layout;

pub use cargo_lid_rs::{
    CargoInsertedSubcommandNameIsDiscarded,
    UnknownSubcommandsFailWithUsage,
    TheProjectRootComesFromCargoMetadata,
    MutationScopeFallsBackFromWorkspaceToPackageToDiff,
    ScopeFlagsOverrideTheConfiguredScope,
    DiffScopePassesThroughToTheEngine,
    ValidationEdgesComeFromTheOwningCrateTestBinary,
    MembersWithoutALibraryTargetAreSkipped,
    TracedMutantsRunOnlyTheirValidatingTests,
    UntracedMutantsFallBackToModuleTests,
    SurvivingMutantsFailTheGate,
    AMutantsVerdictComesFromItsOwnGroupsRun,
    AnEngineRunWithoutAVerdictIsAFailure,
    EveryGroupRunsBeforeSurvivorsAreReported,
};


/// The name [`crate::coach::spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools`] carried
/// while the coach's tool set was three. The alias registers no claim, so the
/// graph sees only the claim it points at; every citation of this name warns
/// with its replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools"]
pub type TheCoachDeclaresExactlyTheReadDraftAndAskTools = crate::coach::spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools;



pub use layout::{
    AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    ACrateRootSlicesDirectoryIsItsCratesSrc,
    ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
    ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    ASliceNoMemberHoldsIsRefusedByName,
    ADocumentBesideTheCodeIsTheSlicesLld,
    ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
    ASlicesDocumentIsNeverUnderItsCompanion,
    ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
    ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec,
    ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode,
    ANamedFileBesideTheDocumentIsTheSlicesIntentFile,
    ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
    ACompanionDirectoryIsTheOneTheManifestNames,
    ACompanionIsNeverReadFromADirectorysShape,
};



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
// phase may write. Two older aliases of the first two names — the ones they
// carried before a companion — are gone rather than re-pointed: no citation
// named them, and an alias no citation names is deleted by the next Phase 2 on
// the slice.

/// The name [`crate::phase::spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent`]
/// carried while a phase's set named the slice's directory rather than the
/// code in it.
#[deprecated = "replaced by PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent"]
pub type PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot =
    crate::phase::spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent;

/// The name [`crate::phase::spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent`]
/// carried while a phase's set named the slice's directory rather than the
/// code in it.
#[deprecated = "replaced by PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent"]
pub type PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule = crate::phase::spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent;

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

