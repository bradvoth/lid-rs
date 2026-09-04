//! Claims for `headless-canopy-agent`
//! (`docs/intent/headless-canopy-agent/lld.md`): a slice is built unattended
//! by one canopy session per phase, with this program's code between them.

use lid_rs::Spec;

// ---- canopy: the entry ---------------------------------------------------------

/// When `canopy` is invoked, it shall accept `--slice <name>`, `--door <url>`
/// — the door defaulting to canopy's production door — and `--max-cost
/// <amount>`, and reject any other flag by name; the key is never an
/// argument.
#[derive(Spec)]
pub struct CanopyTakesSliceDoorAndMaxCostAsItsFlags;

/// When `canopy` runs, the API key shall be read from `CANOPY_KEY`, and an
/// unset variable shall be a precondition stop naming it, before any session
/// opens.
#[derive(Spec)]
pub struct TheKeyComesFromCanopyKeyOrTheRunStopsFirst;

/// When the client dials or refreshes a session, the key shall be presented
/// there and nowhere else: in no record it lands, no tool argument, no file
/// it writes, and no line it prints.
#[derive(Spec)]
pub struct TheKeyIsPresentedOnlyToTheDoor;

/// When a phase ends, the run shall print the session ids that phase opened
/// and either the commit it made or the decisions that stopped it.
#[derive(Spec)]
pub struct EveryPhasePrintsItsSessionsAndItsEnding;

/// When the run ends, its last line shall be the terminal state — PR-ready
/// or stopped — and its exit status shall be 0 for PR-ready and 1 for
/// stopped.
#[derive(Spec)]
pub struct TheLastLineIsTheTerminalStateAndTheExitStatusFollowsIt;

/// When every phase is committed and the gate has passed, the run shall end
/// PR-ready, printing the branch and every decision the phases recorded.
#[derive(Spec)]
pub struct PrReadyEndsWithTheBranchAndEveryRecordedDecision;

// ---- the precondition ----------------------------------------------------------

/// When the run starts, the checked-out branch shall be `lld/<slice>` — the
/// slice from `--slice` or read from the branch as `phase-check` reads it —
/// and any other branch shall stop the run naming it, before any session
/// opens.
#[derive(Spec)]
pub struct ThePreconditionNeedsTheSliceBranchCheckedOut;

/// When the branch's own commits — those since it diverged from the default
/// branch, never the whole ancestry — hold no `phase 1:` commit, the run
/// shall stop at the precondition, in the phase LLD's words, before any
/// session opens.
#[derive(Spec)]
pub struct ThePreconditionNeedsAPhaseOneCommit;

/// When the working tree is not clean, the run shall stop at the
/// precondition naming the dirty paths, before any session opens.
#[derive(Spec)]
pub struct ThePreconditionNeedsACleanTree;

/// When the precondition reads which phases are committed, they shall be
/// those of the branch's own commits — those since it diverged from the
/// default branch, never the whole ancestry — whose subject `tag_of`
/// recognises, read from git, never from a model.
#[derive(Spec)]
pub struct CommittedPhasesAreReadFromTheSubjectTags;

/// When the slice's crate is compile-time and the human's acceptance file is
/// absent, the run shall stop at the precondition naming the file, before
/// any session opens.
#[derive(Spec)]
pub struct ACompileTimeSliceStopsAtThePreconditionWithoutAcceptance;

/// When the run builds, it shall start at the first phase without a commit
/// and say of each committed phase that it is skipped.
#[derive(Spec)]
pub struct CommittedPhasesAreSkippedAndSaidSo;

// ---- a phase is a session ------------------------------------------------------

/// When the run builds, it shall open one worker session for each of Phases
/// 2, 3, 4, 5, and 7 in that order, Phase 6 having no session of its own.
#[derive(Spec)]
pub struct ThePhasesAreOneSessionEachInOrder;

/// When a session is dialled, its `settings` shall carry `system`, `policy`,
/// `params` (empty, for the model's defaults), and `max_cost`, and no other
/// key — every other budget being the config's.
#[derive(Spec)]
pub struct TheDialCarriesExactlyFourSettings;

/// When a session is dialled, its `max_cost` shall be the amount given to
/// `--max-cost`, in the provider's currency, or 5 when the flag is absent.
#[derive(Spec)]
pub struct MaxCostIsTheFlagsAmountOrFive;

/// When the config refuses the dial because it pins one of the four
/// settings, the run shall stop there with the door's sentence, which names
/// the setting.
#[derive(Spec)]
pub struct AConfigThatPinsADialledSettingStopsTheRunNamingIt;

/// When a session is open, the client shall land no `app.policy.configured`
/// in it, so the tool surface the door fixed at the dial is the session's
/// for its life.
#[derive(Spec)]
pub struct NoPolicyRecordIsLandedAfterTheDial;

/// When the client drives a run, every request it makes of the door shall
/// belong to the `converse`, `execute`, or `stop` face, never to
/// `configure`.
#[derive(Spec)]
pub struct TheClientCallsOnlyTheConverseExecuteAndStopFaces;

/// When a phase's worker is dialled, `system` shall be the synced
/// `.claude/agents/lid-rs-phase-<n>.md` with its frontmatter removed, and a
/// missing agent file shall stop the run naming its path.
#[derive(Spec)]
pub struct TheSystemPromptIsTheSyncedAgentBodyWithoutItsFrontmatter;

/// When the worker's policy is built, it shall declare `read`, `grep`,
/// `glob`, `edit`, and `write` for the requestee `lid-rs`, each with the
/// JSON schema of its arguments carrying its description, and `allows`
/// shall list exactly those five pairs.
#[derive(Spec)]
pub struct TheWorkerPolicyAdmitsExactlyTheFiveTools;

/// When the worker's prompt is landed, it shall carry the slice, the branch,
/// the LLD's path, the branch's `git log --oneline`, and the commit subject
/// this phase must use.
#[derive(Spec)]
pub struct TheWorkerPromptCarriesTheSliceTheLogAndTheSubject;

/// When a phase is reworked after a rejection, the worker's prompt shall
/// carry the reviewer's findings.
#[derive(Spec)]
pub struct AReworkPromptCarriesTheReviewersFindings;

/// When a phase ends — committed, stopped, refused, halted, or quiet — every
/// session it opened shall be stopped through the door.
#[derive(Spec)]
pub struct EverySessionIsStoppedWhenItsPhaseEnds;

// ---- a session's tools are its host's ------------------------------------------

/// When a session's policy is built, it shall be built from the declarations
/// its host hands it: `tools` those declarations, `allows` exactly their
/// `(requestee, op)` pairs, whatever tools a host declares.
#[derive(Spec)]
pub struct APolicyIsBuiltFromTheDeclarationsItsHostHands;

/// When a session is opened, it shall carry the declarations it was dialled
/// with, whatever tools they name, so the set a forwarded call is judged
/// against is its host's rather than one fixed for this host's five.
#[derive(Spec)]
pub struct ASessionCarriesTheDeclarationsItsHostDialled;

/// When a turn runs a paired forward's tool, it shall run it through the
/// executor its caller handed to `drive`, whose answer is the completion's
/// outcome.
#[derive(Spec)]
pub struct APairedForwardRunsThroughTheExecutorTheTurnWasHanded;

/// When an `inference.responded` carries tool uses and text of its own,
/// `drive` shall hand that text to the narrator its caller passed.
#[derive(Spec)]
pub struct ThePreCallTextOfAToolAskingResponseGoesToTheNarrator;

/// When a response settles the turn, its text shall not be handed to the
/// narrator, the turn answering with it.
#[derive(Spec)]
pub struct ASettlingResponsesTextIsNotNarrated;

/// When a response asks for tools but carries no text of its own, nothing
/// shall be handed to the narrator.
#[derive(Spec)]
pub struct AResponseThatCarriedNoTextNarratesNothing;

/// When this host drives a turn — a phase worker's or a reviewer's — it
/// shall pass the narrator that shows nothing, so that what a phase prints
/// is its sessions and its ending and nothing more.
#[derive(Spec)]
pub struct ThisHostDrivesEveryTurnWithTheSilentNarrator;

/// When a session carries no phase, a tool call in it shall do its work
/// without asking the pre-tool verdict, there being no phase whose policy
/// could bound it.
#[derive(Spec)]
pub struct ASessionWithoutAPhaseAsksNoPreToolVerdict;

/// When a session carries no phase, nothing shall be tallied under it —
/// neither an observation, nor an edit, nor the check after an edit, nor a
/// denial — there being no phase for a tally to belong to.
#[derive(Spec)]
pub struct ASessionWithoutAPhaseKeepsNoTally;

// ---- the five tools ------------------------------------------------------------

/// When a forward's `op` is classified, `read`, `grep`, `glob`, `edit`, and
/// `write` shall map to the tool of that name and any other `op` to none.
#[derive(Spec)]
pub struct AForwardsOpClassifiesToItsToolOrToNone;

/// When any tool is given a path that resolves outside the workspace root —
/// by `..`, by being absolute, or through a symlink — it shall refuse before
/// any verdict is asked, `read` included.
#[derive(Spec)]
pub struct EveryToolConfinesItsPathToTheWorkspace;

/// When `read` is given a file, it shall return its text with line numbers
/// from `offset` for at most `limit` lines; given a directory, its entries.
#[derive(Spec)]
pub struct ReadReturnsNumberedLinesOrADirectorysEntries;

/// When `grep` runs, it shall match its pattern as a literal, case-sensitive
/// substring in files under `path` (the root by default) narrowed by `glob`,
/// never entering `target/`, `.git/`, or a symlinked directory and never
/// reading a symlinked file, returning `path:line: text` per match and no
/// more than 200 lines.
#[derive(Spec)]
pub struct GrepIsALiteralSubstringSearchCappedAtTwoHundredLines;

/// When `glob` runs, it shall return the paths under the root matching its
/// pattern, sorted, never any under `target/`, `.git/`, or a symlinked
/// directory.
#[derive(Spec)]
pub struct GlobReturnsMatchingPathsSorted;

/// When `glob` is given a pattern that is absolute or has a `..` component,
/// it shall refuse before any verdict is asked; when a match resolves outside
/// the root through a symlink, it shall be omitted from the answer.
#[derive(Spec)]
pub struct AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot;

/// When `edit` finds `old_string` exactly once, or `replace_all` is set and
/// it finds it at all, it shall replace every such occurrence in the
/// existing file.
#[derive(Spec)]
pub struct EditReplacesTheOneOccurrenceOrAllOnRequest;

/// When `old_string` matches no place, or more than one without
/// `replace_all`, `edit` shall change nothing and return an error naming
/// the count.
#[derive(Spec)]
pub struct AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount;

/// When `write` runs, it shall create the file or replace its content whole.
#[derive(Spec)]
pub struct WriteCreatesOrReplacesTheFileWhole;

/// When `read`, `grep`, or `glob` runs in a session carrying a phase, it
/// shall ask the phase library's pre-tool verdict as `Read`, `Grep`, or
/// `Glob` before its work, so the observation is tallied under the session.
#[derive(Spec)]
pub struct ObservationsAreTalliedThroughThePreToolVerdict;

/// When the pre-tool verdict refuses an `edit` or `write`, the tool shall
/// return the refusal's wording as its error and leave the file untouched.
#[derive(Spec)]
pub struct ARefusedEditIsTheToolsErrorAndTheFileIsUntouched;

/// When an `edit` or `write` is allowed and made in a session carrying a
/// phase, the tool's result shall be the text of the post-edit verdict.
#[derive(Spec)]
pub struct AnAllowedEditReturnsThePostEditVerdictsText;

/// When the reviewer's session is dialled, its policy shall declare and
/// allow `read`, `grep`, and `glob` and nothing else.
#[derive(Spec)]
pub struct TheReviewerPolicyAdmitsOnlyTheObservationTools;

/// When a forward for `edit` or `write` arrives in the reviewer's session,
/// it shall be refused here, and the file left untouched.
#[derive(Spec)]
pub struct AnEditForwardedToTheReviewerIsRefusedHere;

// ---- driving a turn ------------------------------------------------------------

/// When the client follows a session's tail, each read shall be parked with
/// a `wait` of 25 s clamped to the credential's remaining life, from the
/// cursor the last page reached.
#[derive(Spec)]
pub struct TheTailIsFollowedByParkedReadsWithinTheCredentialsLife;

/// When an `app.invoke.forward` arrives, it shall be paired with the first
/// held `app.invoke.payload` whose digest equals the forward's and whose
/// `to`, like the forward's, is `lid-rs`; the paired tool then runs.
#[derive(Spec)]
pub struct AForwardIsPairedWithTheHeldPayloadOfItsDigest;

/// When a forward or its payload is addressed to a principal other than
/// `lid-rs`, the client shall land no completion for it.
#[derive(Spec)]
pub struct AForwardToAnotherPrincipalIsNotAnswered;

/// When a payload's digest is computed, it shall be `sha256` of the
/// canonical JSON of `{"to", "args"}`, and shall reproduce canopy's
/// published vector.
#[derive(Spec)]
pub struct ThePayloadDigestReproducesCanopysVector;

/// When JSON is canonicalised, object keys shall be in byte order at every
/// depth and numbers rendered as `f64`.
#[derive(Spec)]
pub struct CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64;

/// When a paired tool has run, the client shall land `app.invoke.completed`
/// carrying the forward's digest, `to` the payload record's producer, and
/// `outcome` `success` with the result or `error` with the message.
#[derive(Spec)]
pub struct ACompletionAnswersThePayloadsProducer;

/// When a completion is landed, its idempotency key shall be `:<forward
/// cursor>:65534:0`, so a retried append lands once.
#[derive(Spec)]
pub struct ACompletionsIdempotencyKeyIsTheForwardsCursor;

/// When an `app.invoke.denied` arrives, the client shall execute nothing and
/// count it as a refusal in the tally of the phase its session carries.
#[derive(Spec)]
pub struct ADenialIsCountedAsARefusal;

/// When an `inference.responded` arrives without `tool_uses` while one user
/// message is outstanding, the turn shall settle with `response.text` as
/// the model's final message; with `tool_uses`, the tail is followed on.
#[derive(Spec)]
pub struct ATurnSettlesOnAResponseWithoutToolUses;

/// When an `inference.responded` carries `terminal`, the user message shall
/// be landed once more; a second terminal in the same turn shall stop the
/// run naming the provider's sentence.
#[derive(Spec)]
pub struct AProviderTerminalIsRetriedOnceThenStopsTheRun;

/// When an `app.session.halted` arrives, the run shall stop with the halt's
/// reason as the decision.
#[derive(Spec)]
pub struct AHaltEndsTheRunWithItsReason;

/// When a session's tail delivers nothing for fifteen minutes, the client
/// shall stop the session and the run shall stop saying so.
#[derive(Spec)]
pub struct AQuietTailForFifteenMinutesStopsTheRun;

/// When the credential a request is about to bear — a read, a landed
/// message, a completion, or a stop alike — is within five seconds of its
/// expiry, the client shall refresh it with the API key before making that
/// request, keeping the session and its cursor.
#[derive(Spec)]
pub struct TheCredentialIsRefreshedBeforeItExpires;

// ---- the door, on the wire -----------------------------------------------------

/// When the door refuses any request — the dial, a send, a tail, a refresh,
/// or a stop — the run shall stop with the door's own sentence.
#[derive(Spec)]
pub struct ADoorRefusalStopsTheRunWithItsSentence;

/// When the door answers `429` and this request has been shed fewer than
/// three times, the client shall pause for the shed's seconds and send the
/// same request again.
#[derive(Spec)]
pub struct AShedIsWaitedOutAndTheSameRequestSentAgain;

/// When a `429` answer carries a `Retry-After` header holding a whole number
/// of seconds, the pause before the next attempt shall be that many seconds.
#[derive(Spec)]
pub struct AShedsPauseIsItsRetryAfterSeconds;

/// When a `429` answer carries no `Retry-After` header, or one that is not a
/// whole number of seconds, the pause before the next attempt shall be five
/// seconds.
#[derive(Spec)]
pub struct AMissingOrUnreadableRetryAfterPausesFiveSeconds;

/// When a request's fourth answer is `429`, the client shall wait no longer
/// and shall treat that answer as a refusal, like any other status.
#[derive(Spec)]
pub struct TheFourthShedIsARefusalLikeAnyOtherStatus;

/// When the door answers a status other than `429`, the client shall wait
/// out nothing and refuse on that first answer.
#[derive(Spec)]
pub struct EveryStatusButAShedIsRefusedOnItsFirstAnswer;

/// When a request is sent again after a shed, it shall carry the same
/// idempotency key as its first attempt, so a shed that in fact landed does
/// not land twice.
#[derive(Spec)]
pub struct ARetriedSendCarriesTheFirstAttemptsIdempotencyKey;

// ---- the ending ----------------------------------------------------------------

/// When the worker's turn settles, its text shall be handed to the phase
/// library's stop verdict with `canopy:<session>` as the agent id.
#[derive(Spec)]
pub struct TheSettledTextGoesToTheStopVerdictAsTheSession;

/// When the stop verdict refuses, its reason shall be landed as the next
/// user message in the same session and the turn driven again.
#[derive(Spec)]
pub struct ARefusedStopIsLandedAsTheNextUserMessage;

/// When the stop verdict refuses for the ninth consecutive time, the run
/// shall stop with the reason, the tree left dirty and uncommitted.
#[derive(Spec)]
pub struct TheNinthConsecutiveRefusalEndsTheRun;

/// When the worker's text carries a `stop` block, the run shall stop with
/// its numbered decisions, committing nothing.
#[derive(Spec)]
pub struct AWorkersStopBlockEndsTheRunWithItsDecisions;

/// When the stop verdict commits, the commit's `Lid-Rs-Agent` trailer —
/// written between `Lid-Rs-Phase` and `Lid-Rs-Tools` — shall be
/// `canopy:<session>`, the session that produced it.
#[derive(Spec)]
pub struct TheCommitNamesItsSessionAsTheAgent;

// ---- the reviewer --------------------------------------------------------------

/// When a phase is committed, a reviewer session shall run to a verdict
/// before the next phase's session opens, and only its approval shall open
/// it.
#[derive(Spec)]
pub struct EveryCommittedPhaseIsReviewedBeforeTheNextOpens;

/// When the reviewer is dialled, `system` shall be the synced
/// `lid-rs-review.md` body, and its one user message shall name the phase,
/// the slice, the commit, the paths that commit touched, the LLD, and the
/// skill's files for that phase, prompting it to refute.
#[derive(Spec)]
pub struct TheReviewPromptNamesTheCommitTheLldAndTheSkillFiles;

/// When the reviewer's text carries one `review` block, `approved: yes`
/// shall parse as approved, `approved: no` followed by numbered lines as
/// rejected with those findings, and anything else as malformed.
#[derive(Spec)]
pub struct TheReviewBlockParsesToApprovedOrFindings;

/// When the reviewer rejects a phase and the run's budget has a rework
/// left, that rework shall be spent to open another session of that phase's
/// worker with the findings.
#[derive(Spec)]
pub struct ARejectionWithAReworkLeftOpensAnotherWorkerSession;

/// When a rework is spent, it shall come from the run's one pool of six,
/// shared by every phase, so the reworks a phase may spend are those the
/// phases before it did not.
#[derive(Spec)]
pub struct TheReworkBudgetIsOnePoolOfSixAcrossEveryPhase;

/// When a rework session's check passes, its commit shall be another
/// `phase <n>:` commit on the branch, the rejected commit left in history
/// unamended.
#[derive(Spec)]
pub struct AReworksCommitIsAnotherPhaseCommitOnTheBranch;

/// When a rework session commits, a fresh reviewer session shall review
/// that commit before the next phase opens.
#[derive(Spec)]
pub struct AReworksCommitIsReviewedByAFreshSession;

/// When the reviewer rejects a phase and no rework remains, the run shall
/// stop with the findings as its decisions.
#[derive(Spec)]
pub struct ARejectionWithTheBudgetSpentEndsTheRunWithTheFindings;

/// When a rejection stops the run because no rework remains, the stop shall
/// say that the rework budget is what ended it.
#[derive(Spec)]
pub struct AnExhaustedBudgetIsSaidToBeWhatStoppedTheRun;

/// When the reviewer's text carries no well-formed `review` block, the
/// format shall be landed once more as the next user message.
#[derive(Spec)]
pub struct AMissingReviewBlockIsAskedForOnceMore;

/// When the reviewer's text misses the block a second time, the phase shall
/// count as rejected with the one finding that the reviewer gave no
/// verdict.
#[derive(Spec)]
pub struct ASecondMissingReviewBlockIsARejection;
