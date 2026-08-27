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
    PhaseTwoChecksTheClaimsBuildAndLint,
    PhasesThreeAndFourCheckTheSkeletonTypeChecks,
    PhaseSevenRunsTheGateInOrder,
    ACheckStopsAtTheFirstFailingStep,
    ASlicesClaimsAreTheSpecsInItsSpecFile,
    TheSliceComesFromTheBranchName,
    ASliceWithNoClaimsFailsTheRedCheck,
    EveryClaimNeedsAValidationBeforePhaseFivePasses,
    EachValidationRunsAloneByExactName,
    AGreenValidationFailsTheRedCheck,
    TaggedCommitsRunTheirPhaseCheck,
    UntaggedCommitsPassTheHook,
    MistypedTagsAreRefusedNotIgnored,
    AStartingWorkerRecordsHead,
    AWorkerThatCommittedMayStop,
    AWorkerThatDidNotCommitIsRefusedOnce,
    ASecondStopAttemptIsAllowed,
    AStopWithoutARecordIsAllowed,
    SyncMirrorsEveryArtifactTheDependencyShips,
    SyncAssertsTheHooksPath,
    AForeignHooksPathIsAnInitConflict,
};

pub use sync::{
    TheSkillComesFromTheResolvedLidRsDependency,
    TheSkillCopyLivesAtTheWorkspaceRoot,
    SyncCheckFailsOnAnyDifferenceAndWritesNothing,
    AMissingSkillSourceFailsByName,
};
