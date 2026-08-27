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

pub use sync::{
    TheSkillComesFromTheResolvedLidRsDependency,
    TheSkillCopyLivesAtTheWorkspaceRoot,
    SyncCheckFailsOnAnyDifferenceAndWritesNothing,
    AMissingSkillSourceFailsByName,
};
