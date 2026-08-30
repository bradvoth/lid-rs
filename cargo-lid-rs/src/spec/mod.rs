//! Atomic claims for `cargo-lid-rs`, one file per slice: a slice's claims
//! are the specs registered from `src/spec/<slice>.rs`, which is how
//! `phase-check 5` finds them (`docs/intent/phase/lld.md`).

mod cargo_lid_rs;
mod init;
mod phase;
mod sync;

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

pub use init::{
    InitTargetsThePackageInTheCurrentDirectory,
    InitWritesNothingWhenAnyTargetConflicts,
    InitAddsLidRsAtTheToolsOwnVersion,
    InitAppendsTheManifestTables,
    InitWiresTheLibraryIntoTheGraph,
    BinOnlyPackagesGainALibrary,
    EmittedFilesCarryThePackageFacts,
    MutationOutputIsIgnoredWithoutConflict,
    AnInitialisedPackagePassesItsOwnGate,
    NewCreatesALibraryPackageThenInitialisesIt,
};

pub use phase::{
    PhasesWithoutACommitHaveNoCheck,
    PhaseOneChecksTheDocs,
    PhaseTwoChecksTheClaimsBuild,
    WarningsDoNotFailPhaseTwosCheck,
    PhasesThreeAndFourCheckTheSkeletonTypeChecks,
    PhaseSevenRunsTheGateInOrder,
    ACheckStopsAtTheFirstFailingStep,
    ASlicesClaimsAreTheSpecsInItsSpecFile,
    TheSliceComesFromTheBranchName,
    ASliceWithNoClaimsFailsTheRedCheck,
    TheBaseIsTheNewestGateCommitReachableFromHead,
    TheRedSetIsTheClaimsAddedSinceTheBase,
    AFreshSliceHasEveryClaimInTheRedSet,
    AnEmptyRedSetAfterAGateFailsTheRedCheck,
    EveryClaimNeedsAValidationBeforePhaseFivePasses,
    EachValidationRunsAloneByExactName,
    AGreenValidationFailsTheRedCheck,
    TheSlicesCrateIsTheOneHoldingItsLld,
    PhaseTwoMayWriteOnlyTheSlicesSpecFiles,
    PhasesThreeAndFourMayWriteTheSliceModuleAndTheLibraryRoot,
    PhasesFiveAndSevenMayWriteOnlyTheSliceModule,
    PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy,
    ARefusedEditQuotesTheDisciplineRow,
    ReadsAreNeverRefused,
    EveryToolCallIsTallied,
    EveryEditIsFollowedByClippy,
    AFinalMessageCarriesExactlyOneEnding,
    AStopBlockEndsThePhaseWithoutACommit,
    ACommitSubjectMustCarryThisPhasesTag,
    ACommitBlockRunsThePhasesCheck,
    ARefusalCarriesTheOutputTheRuleAndThePermittedMoves,
    AFailingOutputNamesItsCheck,
    SyncedArtifactsMustMatchAtTheStop,
    ChangesOutsideThePolicyRefuseTheStop,
    OnlyThePoliciesPathsAreStaged,
    NothingToCommitIsARefusal,
    TheTallyIsWrittenAsTrailers,
    ACompileTimeSliceIsDisclosed, ACompileTimeSliceNeedsTheHumansAcceptance,
    SyncMirrorsEveryArtifactTheDependencyShips,
};

/// The name [`PhaseTwoChecksTheClaimsBuild`] carried while phase 2's check
/// also linted. The alias registers no claim, so the graph sees only the
/// claim it points at; every citation of this name warns with its
/// replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by PhaseTwoChecksTheClaimsBuild"]
pub type PhaseTwoChecksTheClaimsBuildAndLint = phase::PhaseTwoChecksTheClaimsBuild;

pub use sync::{
    TheSkillComesFromTheResolvedLidRsDependency,
    TheSkillCopyLivesAtTheWorkspaceRoot,
    SyncCheckFailsOnAnyDifferenceAndWritesNothing,
    AMissingSkillSourceFailsByName,
};
