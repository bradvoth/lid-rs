//! Atomic claims for `cargo-lid-rs`, one file per slice: a slice's claims
//! are the specs registered from `src/spec/<slice>.rs`, which is how
//! `phase-check 5` finds them (`docs/intent/phase/lld.md`).

mod cargo_lid_rs;
mod coach;
mod headless_canopy_agent;
mod init;
mod layout;
mod lld_review;
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

pub use coach::{
    TheCoachsSliceIsTheFlagsValueOrTheBranchName,
    TheDoorDefaultsToCanopysProductionDoor,
    TheMaxCostDefaultsToFive,
    AnyOtherArgumentToCoachIsRejectedByName,
    PackageAndWorkspaceRefuseEachOther,
    NeitherFlagSettlesOnNoTarget,
    APackageNamingAMemberIsThatMembersManifestDirectory,
    APackageNamingNoMemberStopsTheRunListingTheMembers,
    TheWorkspaceFlagNamesTheWorkspaceRoot,
    NoTargetInAOneMemberWorkspaceIsThatMembersDirectory,
    NoTargetInAWorkspaceOfSeveralMembersStopsNamingTheFlag,
    TheDocumentIsTheSlicesLldUnderThatDirectory,
    TheDocumentsPathIsPrintedWhenTheSessionOpens,
    TheArtifactChecksRunOnceBeforeTheFirstQuestion,
    AnUnreadableGuidelineStopsTheRunNamingItsPath,
    ADriftedChecklistIsReportedAndTheInterviewProceeds,
    AReaderDeclaringTooManyToolsIsReportedAndTheInterviewProceeds,
    TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline,
    TheOpeningNamesTheSlice,
    AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment,
    TheOpeningLeadsWithWhatTheRepositoryHolds,
    ThePreambleIsTheIndexTheHldThenTheGuidance,
    TheIntentIndexNamesEveryIntentDocumentInTheWorkspace,
    TheIntentIndexIsSortedSoTwoRunsAgree,
    EveryIndexRowNamesItsDocumentRelativeToTheWorkspaceRoot,
    ThisRunsOwnDocumentIsMarkedInTheIndex,
    TheDocumentsTheIndexNamesAreNamedAndNotCarried,
    TheOpeningCarriesTheSoleHldWhole,
    AnIndexWithoutExactlyOneHldCarriesNoHld,
    TheProjectsGuidanceIsTheWorkspacesAgentsFileWhole,
    ClaudeMdIsTheGuidanceWhenThereIsNoAgentsFile,
    NeitherGuidanceFileCarriesNoGuidance,
    TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools,
    ReadAndGrepAreDeclaredWithTheCanopyClientsSchemas,
    EverySessionTheCoachOpensIsDialledWithTheMaxCost,
    TheFirstTurnSettlesOnTheOpeningBeforeTheHumanIsRead,
    TheModelsSettledAnswerIsPrinted,
    TheConversationEndsAtDoneOrEndOfFile,
    TheCoachingSessionIsStoppedBeforeTheClientExits,
    ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    TheJudgesAnswerATurnThatDrafted,
    AFailedDraftIsNotATurnThatDrafted,
    ATurnThatDraftedNothingIsAnsweredByTheHuman,
    TheJudgesAnswerAtMostOneDraftingTurnInARow,
    AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    TheDraftedDocumentSurvivesAHalt,
    TheCoachingSessionsNarratorPrintsTheModelsText,
    EveryToolCallIsAnnouncedBeforeItIsRouted,
    AReadIsAnnouncedByThePathItNames,
    AGrepOrGlobIsAnnouncedByThePatternItNames,
    ADraftIsAnnouncedByTheDocumentItReplaces,
    AskAnnouncesNothing,
    ACallWhoseArgumentsLackItsSubjectAnnouncesNothing,
    AReaderSessionsToolCallsAreAnnouncedToo,
    AReaderSessionNarratesNothing,
    AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    AReadIsRoutedToTheCanopyClientsReadOverItsConfinement,
    AGrepIsRoutedToTheCanopyClientsGrepOverItsConfinement,
    DraftReplacesTheDocumentWholeCreatingItsDirectory,
    DraftWritesTheSlicesLldAndNoOtherPath,
    DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict,
    AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped,
    TheStallWindowIsPrintedOnceBesideTheQuestion,
    EndingTheConversationInsideAskIsAToolError,
    AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
    TheJudgesTurnIsTheDocumentChecksThenTheReader,
    TheDocumentChecksRunOverThePathDraftWrote,
    AJudgingWhoseChecksAllHeldSaysSo,
    TheArtifactChecksAreNotInTheJudgesTurn,
    AJudgingsVerdictIsTheFourDocumentChecksAndNothingElse,
    TheHumanIsToldHowTheDocumentChecksFoundTheDocument,
    AnUnreadableDocumentsSentenceIsLandedInPlaceOfTheChecksFailures,
    AnUnreadableDocumentIsToldToTheHuman,
    AnUnreadableDocumentsVerdictIsThatTheChecksDoNotHold,
    ADocumentThatCannotBeReadBackDoesNotEndTheConversation,
    TheReadersSystemIsTheSyncedReaderBody,
    AReaderSessionDeclaresTheCanopyClientsObservationTools,
    AReaderSessionCarriesNoPhase,
    AReaderTurnIsRunByTheCanopyClientsOwnDispatch,
    TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens,
    TheReaderIsAFreshSessionForEveryJudging,
    TheReaderIsGivenTheDocumentAndAskedForFindings,
    AReaderSessionIsStoppedWhenItAnswers,
    AFailedReadersSentenceIsLandedInPlaceOfItsFindings,
    AFailedReaderIsToldToTheHuman,
    AReaderThatCannotBeConsultedDoesNotEndTheConversation,
    TheDocumentChecksAreLandedWhetherOrNotTheReaderAnswers,
    TheJudgesAreLandedAsOneUserMessageUnderAHeading,
    TheEndingPrintsThePathAndWhetherTheChecksHold,
    TheEndingNamesThePhaseOneCommitTheCoachDoesNotMake,
    ARunThatDraftedNothingEndsSayingSo,
};

/// The name [`TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools`] carried
/// while the coach's tool set was three. The alias registers no claim, so the
/// graph sees only the claim it points at; every citation of this name warns
/// with its replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools"]
pub type TheCoachDeclaresExactlyTheReadDraftAndAskTools = coach::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools;

pub use headless_canopy_agent::{
    CanopyTakesSliceDoorAndMaxCostAsItsFlags,
    TheKeyComesFromCanopyKeyOrTheRunStopsFirst,
    TheKeyIsPresentedOnlyToTheDoor,
    EveryPhasePrintsItsSessionsAndItsEnding,
    TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt,
    PrReadyEndsWithTheBranchAndEveryRecordedDecision,
    ThePreconditionNeedsTheSliceBranchCheckedOut,
    ThePreconditionNeedsAPhaseOneCommit,
    ThePreconditionNeedsACleanTree,
    CommittedPhasesAreReadFromTheSubjectTags,
    ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance,
    CommittedPhasesAreSkippedAndSaidSo,
    ThePhasesAreOneSessionEachInOrder,
    TheDialCarriesExactlyFourSettings,
    MaxCostIsTheFlagsAmountOrFive,
    AConfigThatPinsADialledSettingStopsTheRunNamingIt,
    NoPolicyRecordIsLandedAfterTheDial,
    TheClientCallsOnlyTheConverseExecuteAndStopFaces,
    TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter,
    TheWorkerPolicyAdmitsExactlyTheFiveTools,
    TheWorkerPromptCarriesTheSliceTheLogAndTheSubject,
    AReworkPromptCarriesTheReviewersFindings,
    EverySessionIsStoppedWhenItsPhaseEnds,
    APolicyIsBuiltFromTheDeclarationsItsHostHands,
    ASessionCarriesTheDeclarationsItsHostDialled,
    APairedForwardRunsThroughTheExecutorTheTurnWasHanded,
    ThePreCallTextOfAToolAskingResponseGoesToTheNarrator,
    ASettlingResponsesTextIsNotNarrated,
    AResponseThatCarriedNoTextNarratesNothing,
    ThisHostDrivesEveryTurnWithTheSilentNarrator,
    ASessionWithoutAPhaseAsksNoPreToolVerdict,
    ASessionWithoutAPhaseKeepsNoTally,
    AForwardsOpClassifiesToItsToolOrToNone,
    EveryToolConfinesItsPathToTheWorkspace,
    ReadReturnsNumberedLinesOrADirectorysEntries,
    GrepIsALiteralSubstringSearchCappedAtTwoHundredLines,
    GlobReturnsMatchingPathsSorted, AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot,
    EditReplacesTheOneOccurrenceOrAllOnRequest,
    AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount,
    WriteCreatesOrReplacesTheFileWhole,
    ObservationsAreTalliedThroughThePreToolVerdict,
    ARefusedEditIsTheToolsErrorAndTheFileIsUntouched,
    AnAllowedEditReturnsThePostEditVerdictsText,
    TheReviewerPolicyAdmitsOnlyTheObservationTools,
    AnEditForwardedToTheReviewerIsRefusedHere,
    TheTailIsFollowedByParkedReadsWithinTheCredentialsLife,
    AForwardIsPairedWithTheHeldPayloadOfItsDigest,
    AForwardToAnotherPrincipalIsNotAnswered,
    ThePayloadDigestReproducesCanopysVector,
    CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64,
    ACompletionAnswersThePayloadsProducer,
    ACompletionsIdempotencyKeyIsTheForwardsCursor,
    ADenialIsCountedAsARefusal,
    ATurnSettlesOnAResponseWithoutToolUses,
    AProviderTerminalIsRetriedOnceThenStopsTheRun,
    AHaltEndsTheRunWithItsReason,
    AQuietTailForFifteenMinutesStopsTheRun,
    TheCredentialIsRefreshedBeforeItExpires,
    ADoorRefusalStopsTheRunWithItsSentence,
    AShedIsWaitedOutAndTheSameRequestSentAgain,
    AShedsPauseIsItsRetryAfterSeconds,
    AMissingOrUnreadableRetryAfterPausesFiveSeconds,
    TheFourthShedIsARefusalLikeAnyOtherStatus,
    EveryStatusButAShedIsRefusedOnItsFirstAnswer,
    ARetriedSendCarriesTheFirstAttemptsIdempotencyKey,
    TheSettledTextGoesToTheStopVerdictAsTheSession,
    ARefusedStopIsLandedAsTheNextUserMessage,
    TheNinthConsecutiveRefusalEndsTheRun,
    AWorkersStopBlockEndsTheRunWithItsDecisions,
    TheCommitNamesItsSessionAsTheAgent,
    EveryCommittedPhaseIsReviewedBeforeTheNextOpens,
    TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles,
    TheReviewBlockParsesToApprovedOrFindings,
    ARejectionWithAReworkLeftOpensAnotherWorkerSession,
    TheReworkBudgetIsOnePoolOfSixAcrossEveryPhase,
    AReworksCommitIsAnotherPhaseCommitOnTheBranch,
    AReworksCommitIsReviewedByAFreshSession,
    ARejectionWithTheBudgetSpentEndsTheRunWithTheFindings,
    AnExhaustedBudgetIsSaidToBeWhatStoppedTheRun,
    AMissingReviewBlockIsAskedForOnceMore,
    ASecondMissingReviewBlockIsARejection,
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

pub use lld_review::{
    TheSliceIsTheFlagsValueOrTheBranchName,
    AnyOtherArgumentIsRejectedByName,
    TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt,
    AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot,
    AnUnreadableLldFailsNamingItsPath,
    LldCheckExitsZeroOnlyWhenEveryCheckHolds,
    EveryFailureIsReportedNotOnlyTheFirst,
    AFailureNamesItsCheckItsFileItsLineAndItsRule,
    ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator,
    ADocumentWithoutADecisionsTableFails,
    EveryDecisionsRowFillsItsFourCells,
    EveryShapeRowNamesAnIdentifierAndARole,
    ADocumentWithNoShapeTableHoldsThatCheck,
    EveryDeferredItemIsANumberedListItem,
    ADocumentWithNoDeferredHeadingHoldsThatCheck,
    EveryCheckIsNamedInTheGuidelinesChecklist,
    TheReaderDeclaresOnlyTheObservationTools,
    AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine,
    TheArtifactChecksRunWhateverSliceIsNamed,
    AnUnreadableSyncedArtifactFailsNamingItsPath,
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
    AChangeBranchNamesItsSliceBeforeTheDoubleDash,
    ASliceWithNoClaimsFailsTheRedCheck,
    TheBaseIsTheNewestGateCommitReachableFromHead,
    TheRedSetIsTheClaimsAddedSinceTheBase,
    AProcMacroSlicesClaimsAreHeldByItsCompanion,
    AFreshSliceHasEveryClaimInTheRedSet,
    AnEmptyRedSetAfterAGateFailsTheRedCheck,
    EveryClaimNeedsAValidationBeforePhaseFivePasses,
    EachValidationRunsAloneByExactName,
    AGreenValidationFailsTheRedCheck,
    TheSlicesCrateIsTheOneHoldingItsLld,
    PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles,
    PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy,
    AnOrdinaryCrateHasNoCompanion,
    TheCompanionIsTheMemberTheProcMacroCratesMetadataNames,
    AProcMacroCrateNamingNoCompanionRefusesEveryEdit,
    ACompanionThatIsAProcMacroCrateRefusesEveryEdit,
    ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit,
    APathUnderTheCompanionIsJudgedByTheCompanionsTable,
    PhaseTwoMayWriteOnlyTheCompanionsSpecFiles,
    PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
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
    IntegrityFiltersAgainstBothCratesAllowedPaths,
    OnlyThePoliciesPathsAreStaged,
    TheStopStagesBothCratesAllowedPaths,
    NothingToCommitIsARefusal,
    TheTallyIsWrittenAsTrailers,
    ACompileTimeSliceIsDisclosed, ACompileTimeSliceNeedsTheHumansAcceptance,
    SyncMirrorsEveryArtifactTheDependencyShips,
};

// The names below are the ones the `phase` slice's claims carried before a
// proc-macro crate's slice had a companion. Each alias registers no claim,
// so the graph sees only the claim it points at; every citation of the old
// name warns with its replacement, and those citations are the later phases'
// work list.

/// The name [`PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles`] carried while the
/// slice's crate was the only crate a phase could write.
#[deprecated = "replaced by PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles"]
pub type PhaseTwoMayWriteOnlyTheSlicesSpecFiles = phase::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles;

/// The name [`PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy`] carried
/// while there was one crate to be outside of.
#[deprecated = "replaced by PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy"]
pub type PathsOutsideTheSlicesCrateAreRefusedBeforeThePolicy = phase::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy;

// The four names below are the ones the `phase` slice's path-table claims
// carried while a phase's allowed set named the slice's *directory*. Colocation
// puts the slice's document, its claims file and its acceptance file in that
// directory, so each of those names asserted a permission over artifacts no
// phase may write. Two older aliases of the first two names — the ones they
// carried before a companion — are gone rather than re-pointed: no citation
// named them, and an alias no citation names is deleted by the next Phase 2 on
// the slice.

/// The name [`PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent`]
/// carried while a phase's set named the slice's directory rather than the
/// code in it.
#[deprecated = "replaced by PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent"]
pub type PhasesThreeAndFourMayWriteTheOwnCratesSliceModuleAndLibraryRoot =
    phase::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent;

/// The name [`PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent`]
/// carried while a phase's set named the slice's directory rather than the
/// code in it.
#[deprecated = "replaced by PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent"]
pub type PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceModule = phase::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent;

/// The name [`PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims`]
/// carried while a phase's set named the slice's directory in the companion
/// rather than the code in it.
#[deprecated = "replaced by PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims"]
pub type PhasesThreeAndFourMayWriteTheCompanionsSliceModuleAndLibraryRoot =
    phase::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims;

/// The name [`PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims`]
/// carried while a phase's set named the slice's directory in the companion
/// rather than the code in it.
#[deprecated = "replaced by PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims"]
pub type PhasesFiveAndSevenMayWriteTheCompanionsSliceModuleAndUiFixtures =
    phase::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims;

pub use sync::{
    TheSkillComesFromTheResolvedLidRsDependency,
    TheSkillCopyLivesAtTheWorkspaceRoot,
    SyncCheckFailsOnAnyDifferenceAndWritesNothing,
    AMissingSkillSourceFailsByName,
};
