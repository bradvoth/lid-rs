//! Claims for `coach` (`docs/intent/coach/lld.md`): an LLD is written by
//! interview, judged by the judges that will judge it, and left for the human
//! to commit.
//!
//! Two boundaries these claims keep. The canopy client's machinery is not
//! restated: `Door`, `Session`, `drive`, the forward/completion pairing, the
//! payload digest, `confine`, `read_tool`, `grep_tool` and the five observation
//! and editing tools are `headless-canopy-agent`'s claims — so what a read or a
//! grep answers, how it is confined, and which of a turn's text `drive` hands a
//! narrator are asserted there, and what is asserted below is what the coach
//! does with them: which sessions it opens, what it dials them with, where its
//! own dispatch routes a call, which narrator each session is driven with, and
//! what it prints. Nor is
//! `lld-review`'s machinery: the four document checks, the two artifact checks
//! and the reader's own behaviour are that slice's claims, and what is
//! asserted below is that the coach runs them, when, over which document, and
//! what it does with their answers.

use lid_rs::Spec;

// ---- the subcommand: `parse_args` and the flags it settles ---------------------

/// When `coach` is given `--slice <name>`, that name shall be the slice;
/// absent the flag, the slice shall be the current branch's name with `lld/`
/// removed.
#[derive(Spec)]
pub struct TheCoachsSliceIsTheFlagsValueOrTheBranchName;

/// When `--door` is absent, `coach` shall open its sessions on canopy's
/// production door, `https://api.canopyhq.dev`.
#[derive(Spec)]
pub struct TheDoorDefaultsToCanopysProductionDoor;

/// When `--max-cost` is absent, the run's budget shall be five.
#[derive(Spec)]
pub struct TheMaxCostDefaultsToFive;

/// When `coach` is given an argument that is none of `--package`,
/// `--workspace`, `--slice`, `--door` and `--max-cost`, it shall be rejected
/// by name.
#[derive(Spec)]
pub struct AnyOtherArgumentToCoachIsRejectedByName;

/// When both `--package` and `--workspace` are given, the two flags shall
/// refuse each other, stopping the run, a document having one place.
#[derive(Spec)]
pub struct PackageAndWorkspaceRefuseEachOther;

/// When neither `--package` nor `--workspace` is given, the flags shall settle
/// on no target, a flag parser being handed no project and unable to count a
/// workspace's members.
#[derive(Spec)]
pub struct NeitherFlagSettlesOnNoTarget;

// ---- where the document goes: `package_dir`'s five outcomes, then the path -----

/// When the target names a package that is a member of the workspace, the
/// document's directory shall be that member's manifest directory, and not the
/// first member found to hold a document of that slice's name.
#[derive(Spec)]
pub struct APackageNamingAMemberIsThatMembersManifestDirectory;

/// When the target names a package matching no member of the workspace, it
/// shall stop the run listing the members.
#[derive(Spec)]
pub struct APackageNamingNoMemberStopsTheRunListingTheMembers;

/// When the target is `--workspace`, the document's directory shall be the
/// workspace root, where a slice whose product is the workspace rather than a
/// crate keeps its LLD and no package holds it.
#[derive(Spec)]
pub struct TheWorkspaceFlagNamesTheWorkspaceRoot;

/// When there is no target and the workspace has exactly one member, the
/// document's directory shall be that member's manifest directory.
#[derive(Spec)]
pub struct NoTargetInAOneMemberWorkspaceIsThatMembersDirectory;

/// When there is no target and the workspace has several members, that shall
/// be a stop naming the flag, rather than a package inferred from the
/// directory the command was run in.
#[derive(Spec)]
pub struct NoTargetInAWorkspaceOfSeveralMembersStopsNamingTheFlag;

/// When the document's directory is settled, the document shall be
/// `docs/intent/<slice>/lld.md` under it.
#[derive(Spec)]
pub struct TheDocumentIsTheSlicesLldUnderThatDirectory;

/// When the coaching session opens, the coach shall print the document's path,
/// it being the one thing a human can get wrong here that no later check would
/// question.
#[derive(Spec)]
pub struct TheDocumentsPathIsPrintedWhenTheSessionOpens;

// ---- before the interview ------------------------------------------------------

/// When a coach run starts, `lld-check`'s two artifact checks — the
/// guideline's checklist and the reader's declared tools — shall run once,
/// before the first question is asked.
#[derive(Spec)]
pub struct TheArtifactChecksRunOnceBeforeTheFirstQuestion;

/// When the synced guideline cannot be read, the run shall stop naming the
/// path it looked for, the guideline being half the system prompt.
#[derive(Spec)]
pub struct AnUnreadableGuidelineStopsTheRunNamingItsPath;

/// When the guideline reads but its checklist omits a check the tool applies,
/// the human shall be told which check it omits and the interview shall
/// proceed.
#[derive(Spec)]
pub struct ADriftedChecklistIsReportedAndTheInterviewProceeds;

/// When the reader's frontmatter declares more than the observation tools, the
/// human shall be told what it declares and the interview shall proceed, that
/// being their project's problem rather than this conversation's.
#[derive(Spec)]
pub struct AReaderDeclaringTooManyToolsIsReportedAndTheInterviewProceeds;

// ---- what the session is opened with -------------------------------------------

/// When the coaching session is dialled, its system prompt shall be the synced
/// interview method followed by the synced guideline, in that order.
#[derive(Spec)]
pub struct TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline;

/// When the conversation opens, its first user message shall name the slice
/// the document is being written for.
#[derive(Spec)]
pub struct TheOpeningNamesTheSlice;

/// When a document already exists at that path, the opening message shall
/// carry it whole and say the model is amending it rather than writing one.
#[derive(Spec)]
pub struct AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment;

// ---- what the opening carries: the repository, before what this run is ---------

/// When the opening is composed, what the repository holds shall come before
/// what this run is, so that the walk of directories a model would otherwise
/// spend its first minutes on is done once, by the coach.
#[derive(Spec)]
pub struct TheOpeningLeadsWithWhatTheRepositoryHolds;

/// When the repository's part of the opening is composed, it shall carry the
/// intent index, then the HLD, then the project's guidance, in that order.
#[derive(Spec)]
pub struct ThePreambleIsTheIndexTheHldThenTheGuidance;

/// When the intent index is built, it shall name every `docs/intent` document
/// in the workspace: the HLD at the workspace root and under each member, and
/// every slice's `<slice>/lld.md` under either.
#[derive(Spec)]
pub struct TheIntentIndexNamesEveryIntentDocumentInTheWorkspace;

/// When the intent index is built, its paths shall be sorted, so that one run's
/// index and the next's agree.
#[derive(Spec)]
pub struct TheIntentIndexIsSortedSoTwoRunsAgree;

/// When the index renders a row, it shall name that document relative to the
/// workspace root, which is the form `confine` takes: an absolute path is one
/// `read` and `grep` refuse as written, so an index in any other form is a list
/// of paths the one tool it feeds will not accept.
#[derive(Spec)]
pub struct EveryIndexRowNamesItsDocumentRelativeToTheWorkspaceRoot;

/// When the index lists the document this run is writing, that row shall be
/// marked as the one the model is about to write rather than consult.
#[derive(Spec)]
pub struct ThisRunsOwnDocumentIsMarkedInTheIndex;

/// When the index names a document that is neither the HLD it carries, the
/// project's guidance, nor this run's own, the opening shall carry that
/// document's path and not its text, a first message carrying twelve documents
/// burying the one that mattered.
#[derive(Spec)]
pub struct TheDocumentsTheIndexNamesAreNamedAndNotCarried;

/// When the intent index found exactly one HLD, the opening shall carry that
/// HLD whole, it being the design every slice sits inside.
#[derive(Spec)]
pub struct TheOpeningCarriesTheSoleHldWhole;

/// When the intent index found other than exactly one HLD, the opening shall
/// carry no HLD, which of several governs this slice being a design question
/// with a human's answer rather than one a coach may make silently.
#[derive(Spec)]
pub struct AnIndexWithoutExactlyOneHldCarriesNoHld;

/// When the workspace root holds an `AGENTS.md`, the opening shall carry it
/// whole as the project's guidance.
#[derive(Spec)]
pub struct TheProjectsGuidanceIsTheWorkspacesAgentsFileWhole;

/// When the workspace root holds no `AGENTS.md`, the opening shall carry its
/// `CLAUDE.md` whole in place of it, for a project that arrived at the
/// methodology by another road.
#[derive(Spec)]
pub struct ClaudeMdIsTheGuidanceWhenThereIsNoAgentsFile;

/// When the workspace root holds neither `AGENTS.md` nor `CLAUDE.md`, the
/// opening shall carry no guidance.
#[derive(Spec)]
pub struct NeitherGuidanceFileCarriesNoGuidance;

// ---- the tools the session declares ---------------------------------------------

/// When the coaching session is dialled, it shall declare exactly four tools —
/// `read`, `grep`, `draft` and `ask` — a set of the coach's own, holding
/// neither of the canopy client's `edit` and `write`, which a coach that writes
/// one document must not have.
#[derive(Spec)]
pub struct TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools;

/// When `read` and `grep` are declared, each shall carry the canopy client's
/// own `schema_of` for that tool, two descriptions of one tool being two things
/// nothing keeps in step.
#[derive(Spec)]
pub struct ReadAndGrepAreDeclaredWithTheCanopyClientsSchemas;

/// When the coach opens any session — the coaching session, or a reader's — it
/// shall dial it with the run's budget, so the run is bounded one session at a
/// time rather than in total.
#[derive(Spec)]
pub struct EverySessionTheCoachOpensIsDialledWithTheMaxCost;

// ---- the loop ------------------------------------------------------------------

/// When the conversation opens, its first turn shall be driven on the opening
/// as the conversation's first user message, before the human is read for a
/// line, so that the coach's first question is that turn's settled answer.
#[derive(Spec)]
pub struct TheFirstTurnSettlesOnTheOpeningBeforeTheHumanIsRead;

/// When the model's turn settles, its settled text shall be printed for the
/// human.
#[derive(Spec)]
pub struct TheModelsSettledAnswerIsPrinted;

/// When the human's input is `done` alone on a line, or reaches end of file,
/// the conversation shall end.
#[derive(Spec)]
pub struct TheConversationEndsAtDoneOrEndOfFile;

/// When the conversation ends, however it ends, the coaching session shall be
/// stopped through the door before the client exits, a session left open being
/// an unsealed log.
#[derive(Spec)]
pub struct TheCoachingSessionIsStoppedBeforeTheClientExits;

/// When the loop asks whether a turn drafted, it shall read what the coach's
/// own executor recorded while running the turn's tools, and infer nothing
/// from the model's words.
#[derive(Spec)]
pub struct ThatATurnDraftedIsRecordedByTheExecutorNotInferred;

/// When a turn called `draft` and the call succeeded, the judges' findings
/// shall be landed as the next user message rather than the run waiting for
/// the human to notice them.
#[derive(Spec)]
pub struct TheJudgesAnswerATurnThatDrafted;

/// When a turn's `draft` call failed, that turn shall not count as one that
/// wrote the document, and the judges shall not run.
#[derive(Spec)]
pub struct AFailedDraftIsNotATurnThatDrafted;

/// When a turn drafted nothing, the next user message shall be the human's.
#[derive(Spec)]
pub struct ATurnThatDraftedNothingIsAnsweredByTheHuman;

/// When the model drafts again in reply to the judges, that turn shall be
/// answered by the human, so that the judges answer at most one drafting turn
/// in a row and every second one is a decision the human makes.
#[derive(Spec)]
pub struct TheJudgesAnswerAtMostOneDraftingTurnInARow;

/// When a halt reaches the loop, the conversation shall end with that halt's
/// sentence.
#[derive(Spec)]
pub struct AHaltReachingTheLoopEndsTheConversationWithItsSentence;

/// When a halt ends the conversation, the document `draft` wrote shall stay
/// where it was written, `draft` having written it to disk rather than into the
/// session.
#[derive(Spec)]
pub struct TheDraftedDocumentSurvivesAHalt;

// ---- what the human sees while a turn runs --------------------------------------

/// When the coaching session's turn is driven, its narrator shall print what
/// the model said before calling a tool, that being where a model says what it
/// is about to look for and worth more to the human than any summary the coach
/// could invent.
#[derive(Spec)]
pub struct TheCoachingSessionsNarratorPrintsTheModelsText;

/// When a tool call reaches the coach's executor, it shall be announced before
/// it is routed.
#[derive(Spec)]
pub struct EveryToolCallIsAnnouncedBeforeItIsRouted;

/// When a `read` is announced, its line shall name the path that call names.
#[derive(Spec)]
pub struct AReadIsAnnouncedByThePathItNames;

/// When a call naming a pattern — a `grep`'s or a `glob`'s — is announced, its
/// line shall name that pattern.
#[derive(Spec)]
pub struct AGrepOrGlobIsAnnouncedByThePatternItNames;

/// When a `draft` is announced, its line shall name the document it is about to
/// replace, which is the coach's own path rather than an argument of the call.
#[derive(Spec)]
pub struct ADraftIsAnnouncedByTheDocumentItReplaces;

/// When `ask` is called, nothing shall be announced for it, the question it
/// prints being its own announcement and a line above it only pushing that
/// question up the screen.
#[derive(Spec)]
pub struct AskAnnouncesNothing;

/// When a call's arguments do not carry the subject its line would name,
/// nothing shall be announced for it, a line guessing at what it meant being a
/// second, worse account of one fault.
#[derive(Spec)]
pub struct ACallWhoseArgumentsLackItsSubjectAnnouncesNothing;

/// When a reader session's turn calls a tool, that call shall be announced as
/// the coaching session's calls are, the judging being the other place a run
/// goes quiet for minutes.
#[derive(Spec)]
pub struct AReaderSessionsToolCallsAreAnnouncedToo;

/// When a reader session's turn is driven, it shall be driven with the canopy
/// client's silent narrator, what the reader has to say being landed whole a
/// moment later and the human owing it a reading rather than a draft of it.
#[derive(Spec)]
pub struct AReaderSessionNarratesNothing;

// ---- the four tools: `execute` and what it routes to ----------------------------

/// When a forward names an `op` the coach did not declare, the coach's
/// executor shall refuse it naming that `op`.
#[derive(Spec)]
pub struct AnOpTheCoachDidNotDeclareIsRefusedWithItsName;

/// When a forward names `read`, the coach's executor shall answer it with the
/// canopy client's `read_tool` over that client's `confine`, so that a file
/// reaches the coach exactly as it reaches a phase worker and what a read
/// answers is asserted once, there.
#[derive(Spec)]
pub struct AReadIsRoutedToTheCanopyClientsReadOverItsConfinement;

/// When a forward names `grep`, the coach's executor shall answer it with the
/// canopy client's `grep_tool` over that client's `confine`, under the same
/// confinement its `read` is bounded by, so what a grep answers is asserted
/// once, there.
#[derive(Spec)]
pub struct AGrepIsRoutedToTheCanopyClientsGrepOverItsConfinement;

/// When `draft` is called, it shall replace the document whole, creating its
/// directory, there being no partial edit of it.
#[derive(Spec)]
pub struct DraftReplacesTheDocumentWholeCreatingItsDirectory;

/// When `draft` is called, the one path it can write shall be the slice's LLD
/// — `content` being its only argument — so no call of it reaches the module,
/// the claims or a manifest.
#[derive(Spec)]
pub struct DraftWritesTheSlicesLldAndNoOtherPath;

/// When `draft` succeeds, it shall answer with the path and the number of
/// bytes written, and not with the document checks' verdict, which the judges'
/// turn runs a moment later so that the checks are run once and read once.
#[derive(Spec)]
pub struct DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict;

/// When `ask` is called, it shall put its one question to the human and answer
/// the model with what they typed.
#[derive(Spec)]
pub struct AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped;

/// When a question is put to the human, canopy's fifteen-minute invoke stall
/// shall be printed beside it once, as it is asked, and at no later moment.
#[derive(Spec)]
pub struct TheStallWindowIsPrintedOnceBesideTheQuestion;

/// When the human answers an outstanding question with `done` or end of file,
/// `ask` shall answer the model with a tool error saying the conversation is
/// over, a tool having no other channel to say it through.
#[derive(Spec)]
pub struct EndingTheConversationInsideAskIsAToolError;

/// When the conversation was ended inside `ask`, the loop shall end as that
/// turn settles, rather than reading the human again.
#[derive(Spec)]
pub struct AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles;

// ---- what the judges say -------------------------------------------------------

/// When the judges answer a drafting turn, the four document checks shall come
/// first in their message and the reader's findings second.
#[derive(Spec)]
pub struct TheJudgesTurnIsTheDocumentChecksThenTheReader;

/// When the document checks are run for the judges' turn, they shall be run
/// over the path `draft` wrote rather than over a path resolved again, their
/// failures rendered as `lld-check` renders them for a human.
#[derive(Spec)]
pub struct TheDocumentChecksRunOverThePathDraftWrote;

/// When none of a judging's four document checks failed, its message shall say
/// that every check held, a message that said nothing of them reading as a
/// message that ran none of them.
#[derive(Spec)]
pub struct AJudgingWhoseChecksAllHeldSaysSo;

/// When the judges' message is composed, the two artifact checks shall be left
/// out of it, being about the project's synced guideline and reader rather
/// than the document, which is all a model whose only writing tool is `draft`
/// can act on.
#[derive(Spec)]
pub struct TheArtifactChecksAreNotInTheJudgesTurn;

/// When a judging's four document checks have run, its verdict shall be whether
/// none of them failed and nothing besides, the reader — answering or not —
/// having no part in a statement about checks.
#[derive(Spec)]
pub struct AJudgingsVerdictIsTheFourDocumentChecksAndNothingElse;

/// When a judging's four document checks have run, the human shall be told how
/// they found the document, the line they are shown for a judging being where a
/// run says whether what was just written holds.
#[derive(Spec)]
pub struct TheHumanIsToldHowTheDocumentChecksFoundTheDocument;

/// When the document cannot be read back at the moment a judging runs, the
/// read's own sentence shall be landed in place of the document checks'
/// failures, `draft` having reported the bytes it wrote and the checks having
/// read the path again.
#[derive(Spec)]
pub struct AnUnreadableDocumentsSentenceIsLandedInPlaceOfTheChecksFailures;

/// When the document cannot be read back for a judging, the human shall be told
/// that no check ran over it, they being the only one who can put back a
/// document that has gone from under the run.
#[derive(Spec)]
pub struct AnUnreadableDocumentIsToldToTheHuman;

/// When the document cannot be read back for a judging, that judging's verdict
/// shall be that the document checks do not hold, a verdict being a statement
/// about checks that ran and none did.
#[derive(Spec)]
pub struct AnUnreadableDocumentsVerdictIsThatTheChecksDoNotHold;

/// When the document cannot be read back for a judging, the conversation shall
/// continue, the human being mid-interview and the model owed the answer that a
/// judging which could not read what was written must still give.
#[derive(Spec)]
pub struct ADocumentThatCannotBeReadBackDoesNotEndTheConversation;

/// When a reader session is dialled, its `system` shall be the reader's synced
/// body.
#[derive(Spec)]
pub struct TheReadersSystemIsTheSyncedReaderBody;

/// When a reader session is dialled, its declarations shall be the canopy
/// client's three observation tools.
#[derive(Spec)]
pub struct AReaderSessionDeclaresTheCanopyClientsObservationTools;

/// When a reader session is dialled, it shall carry no phase, a reading
/// belonging to no phase whose policy would judge it or whose tally would
/// count it.
#[derive(Spec)]
pub struct AReaderSessionCarriesNoPhase;

/// When a reader's turn forwards a tool call, it shall be run by the canopy
/// client's own dispatch rather than by the coach's, `drive` taking the
/// executor for a turn as a parameter.
#[derive(Spec)]
pub struct AReaderTurnIsRunByTheCanopyClientsOwnDispatch;

/// When a judging is about to open a reader session, the coach shall print the
/// judging's heading to the terminal first, so that the opening line the
/// session prints belongs to the judging rather than reading as the coaching
/// session reopening.
#[derive(Spec)]
pub struct TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens;

/// When a judging consults the reader, it shall open a session of its own for
/// that reading, a reader that remembers its last reading reading the diff
/// rather than the document a phase will receive.
#[derive(Spec)]
pub struct TheReaderIsAFreshSessionForEveryJudging;

/// When the reader is asked, its message shall name the document and ask for
/// findings in the form the reader's own definition asks for them.
#[derive(Spec)]
pub struct TheReaderIsGivenTheDocumentAndAskedForFindings;

/// When the reader's turn settles, its session shall be stopped.
#[derive(Spec)]
pub struct AReaderSessionIsStoppedWhenItAnswers;

/// When consulting the reader fails, that failure's sentence shall be landed
/// in place of the findings, saying the reader could not be consulted.
#[derive(Spec)]
pub struct AFailedReadersSentenceIsLandedInPlaceOfItsFindings;

/// When consulting the reader fails, the human shall be told so, the second
/// opinion they were about to be given not having arrived.
#[derive(Spec)]
pub struct AFailedReaderIsToldToTheHuman;

/// When consulting the reader fails, the conversation shall continue, a lost
/// second opinion not being a reason to end a session the human is in the
/// middle of.
#[derive(Spec)]
pub struct AReaderThatCannotBeConsultedDoesNotEndTheConversation;

/// When the reader could not be consulted, the document checks' findings shall
/// be landed regardless.
#[derive(Spec)]
pub struct TheDocumentChecksAreLandedWhetherOrNotTheReaderAnswers;

/// When the judges' findings are landed, they shall be one user message under
/// a heading saying what it is, so that a reader of the sealed log can tell
/// the judges' turn from the human's.
#[derive(Spec)]
pub struct TheJudgesAreLandedAsOneUserMessageUnderAHeading;

// ---- the ending ----------------------------------------------------------------

/// When the run ends having drafted, it shall print the document's path and
/// whether the document checks hold, as the last judging found them.
#[derive(Spec)]
pub struct TheEndingPrintsThePathAndWhetherTheChecksHold;

/// When the run ends having drafted, it shall say what the human owes —
/// reading the document once more and committing it as `phase 1: LLD for
/// <slice>` — which the coach does not do.
#[derive(Spec)]
pub struct TheEndingNamesThePhaseOneCommitTheCoachDoesNotMake;

/// When nothing was drafted in the whole conversation, the run shall end
/// saying so: no document at that path, nothing to check, and nothing to
/// commit.
#[derive(Spec)]
pub struct ARunThatDraftedNothingEndsSayingSo;
