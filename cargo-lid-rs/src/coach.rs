//! Coaching an LLD into existence (`docs/intent/coach/lld.md`): `cargo lid-rs
//! coach` opens one canopy session whose system prompt is the synced interview
//! method and the synced guideline, puts the model's questions to the human
//! through a tool, writes the document the answers settle, and lands the
//! document checks and the reader over what was written as the next turn. The
//! human commits it as `phase 1:`; this does not.
//!
//! **What this slice owns.** The subcommand's flags; the package the document
//! goes under and the path within it; the two prompts a coaching session is
//! opened with; the loop of human turn, model turn and judges' turn; a tool set
//! of its own — `read`, `draft`, `ask` — and the dispatch that routes a
//! forwarded call to one of them; the record ([`Noted`]) that dispatch keeps of
//! what one turn did, which is how the loop knows a turn drafted and how it
//! learns the human ended the conversation inside a tool; what a judging
//! produced ([`Judging`]), whose verdict the loop carries out for the ending to
//! print; and the ending, which says what the human owes.
//!
//! **What it borrows, and does not restate.** The door, the session, the turn
//! loop, the forward/completion pairing and the workspace confinement are
//! [`crate::headless_canopy_agent`]'s: [`Session::open`] dials, [`drive`] runs
//! one turn through the executor this module hands it, and `read` is that
//! client's [`read_tool`] over its own [`confine`], so a file reaches the coach
//! exactly as it reaches a phase worker. The four document checks, the two
//! artifact checks and the reader are [`crate::lld_review`]'s. What is asserted
//! here is only what this slice does with them: which sessions it opens, what
//! it dials them with, where its dispatch routes a call, when the judges run,
//! and what it prints.
//!
//! **What it gives up.** The canopy client bounds what leaves by confining
//! every read to the workspace. A coaching conversation is not in the
//! workspace: what the human types exists nowhere else, and every word of it is
//! landed in the tenant's log and sealed there, with the drafted document and
//! the reader's findings about it. `docs/intent/coach/lld.md` § Security
//! posture states that trade, so that running `coach` is choosing it.
//!
//! [`Session::open`]: crate::headless_canopy_agent::turn::Session::open
//! [`drive`]: crate::headless_canopy_agent::turn::drive
//! [`read_tool`]: crate::headless_canopy_agent::tools::read_tool
//! [`confine`]: crate::headless_canopy_agent::tools::confine

use std::path::{Path, PathBuf};

use lid_rs::implements;
use serde_json::Value;

use crate::headless_canopy_agent::door::{Door, ToolDecl};
use crate::headless_canopy_agent::tools::ToolResult;
use crate::headless_canopy_agent::turn::{Halt, Session};
use crate::project::Project;
use crate::spec;

/// `coach [--package <name> | --workspace] [--slice <name>] [--door <url>]
/// [--max-cost <amount>]`: the flags, the artifact checks, the document's
/// path, one coaching session held for as long as the conversation lasts, and
/// what the human owes when it ends. The API key is read from `CANOPY_KEY` and
/// never appears on a command line; the coaching session is stopped before this
/// returns, however the conversation ended. The verdict the ending prints is
/// the one [`converse`] answered with — the last judging's, or none when
/// nothing was ever drafted — handed straight to [`owed`] rather than
/// recomputed here.
#[implements(
    spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName,
    spec::TheDocumentsPathIsPrintedWhenTheSessionOpens,
    spec::TheArtifactChecksRunOnceBeforeTheFirstQuestion,
    spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost,
    spec::TheCoachingSessionIsStoppedBeforeTheClientExits,
    spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
)]
pub fn run(args: &[String]) -> Result<(), String> {
    let _ = args;
    todo!()
}

/// The flags `coach` takes, with their defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    /// `--package <name>` or `--workspace`; none when neither was given,
    /// which only a project can settle.
    pub target: Option<Where>,
    /// `--slice <name>`; the branch's slice when absent.
    pub slice: Option<String>,
    /// `--door <url>`; canopy's production door when absent.
    pub door: String,
    /// `--max-cost <amount>`, in the provider's currency; five when absent.
    pub max_cost: f64,
}

impl Default for Flags {
    /// No target, no slice, canopy's production door, and a budget of five:
    /// what the flags say when none is given. A flag parser is handed no
    /// project and cannot count a workspace's members, so no target is what
    /// neither `--package` nor `--workspace` settles on.
    #[implements(
        spec::TheDoorDefaultsToCanopysProductionDoor,
        spec::TheMaxCostDefaultsToFive,
        spec::NeitherFlagSettlesOnNoTarget,
    )]
    fn default() -> Self {
        todo!()
    }
}

/// The flags settled from the arguments: every `--flag value` pair applied to
/// the defaults, `--workspace` standing alone. A flag absent from the arguments
/// keeps what [`Flags::default`] says — canopy's production door, a budget of
/// five, and no target — and absence is what only this sees, the arguments
/// being here and nowhere else. Any other argument is rejected by name, and
/// `--package` and `--workspace` refuse each other, a document having one
/// place.
#[implements(
    spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName,
    spec::TheDoorDefaultsToCanopysProductionDoor,
    spec::TheMaxCostDefaultsToFive,
    spec::AnyOtherArgumentToCoachIsRejectedByName,
    spec::PackageAndWorkspaceRefuseEachOther,
    spec::NeitherFlagSettlesOnNoTarget,
)]
pub fn parse_args(args: &[String]) -> Result<Flags, String> {
    let _ = args;
    todo!()
}

/// Where the document goes, as the flags name it — never as the directory the
/// command was run in implies it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Where {
    /// `--package <name>`: the workspace member of that name.
    Package(String),
    /// `--workspace`: the workspace root, where a slice whose product is the
    /// workspace rather than a crate keeps its LLD and no package holds it.
    Workspace,
}

/// The directory the document goes under, which needs the project the flags do
/// not have: the named member's manifest directory; the workspace root for
/// `--workspace`; the only member when there is no target and the workspace has
/// one; a stop naming the flag when there is no target and it has several; and
/// a stop listing the members when a name matches none.
#[implements(
    spec::APackageNamingAMemberIsThatMembersManifestDirectory,
    spec::APackageNamingNoMemberStopsTheRunListingTheMembers,
    spec::TheWorkspaceFlagNamesTheWorkspaceRoot,
    spec::NoTargetInAOneMemberWorkspaceIsThatMembersDirectory,
    spec::NoTargetInAWorkspaceOfSeveralMembersStopsNamingTheFlag,
)]
pub fn package_dir(project: &Project, target: Option<&Where>) -> Result<PathBuf, String> {
    let _ = (project, target);
    todo!()
}

/// The document itself: `docs/intent/<slice>/lld.md` under the directory
/// [`package_dir`] settled — the path the walk will look for it at.
#[implements(spec::TheDocumentIsTheSlicesLldUnderThatDirectory)]
pub fn document_path(package_dir: &Path, slice: &str) -> PathBuf {
    let _ = (package_dir, slice);
    todo!()
}

/// One coaching run: the door its sessions are dialled on, the coaching session
/// it holds, the one document it writes, and the budget every session it opens
/// — this one and each judging's reader — is dialled with, so the run is
/// bounded one session at a time rather than in total. The dialling is done by
/// the items that open sessions — [`run`] and [`reader_findings`] — which is
/// where that budget is asserted; this only carries it to them.
pub struct Coach {
    /// The door every session of this run is dialled on.
    pub door: Door,
    /// The coaching session, held for as long as the conversation lasts.
    pub session: Session,
    /// The document `draft` writes and the judges read.
    pub path: PathBuf,
    /// Each session's cost ceiling, in the provider's currency.
    pub max_cost: f64,
}

/// What one turn's executor records while it runs that turn's tools: whether
/// `draft` was called, whether the call wrote the document, and whether a tool
/// learned the conversation is over. The facts [`converse`] reads once the turn
/// has settled — held by the turn rather than by the session, and never
/// inferred from the model's words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred)]
pub struct Noted {
    /// Whether the turn called `draft`.
    pub drafted: bool,
    /// Whether that call wrote the document.
    pub wrote: bool,
    /// Whether the human ended the conversation inside a tool call — `done` or
    /// end of file, typed as the answer to a question `ask` put to them. The
    /// model is answered with a tool error and its turn settles; this is the
    /// only thing that carries the ending back out to the loop, `ask` returning
    /// nothing to it.
    pub ended: bool,
}

/// The coaching session's system prompt: the synced interview method, then the
/// synced guideline, in that order — how to interview, and the questions the
/// document will be judged by. The guideline is read from
/// [`lld_review::GUIDELINE`](crate::lld_review::GUIDELINE), the path that slice
/// already names, a second copy of a synced path here being two strings nothing
/// keeps in step. A guideline that cannot be read is the error naming the path,
/// it being half of this.
#[implements(
    spec::TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline,
    spec::AnUnreadableGuidelineStopsTheRunNamingItsPath,
)]
pub fn coaching_system(project: &Project) -> Result<String, String> {
    let _ = project;
    todo!()
}

/// The conversation's first user message: the slice the document is being
/// written for, and — when a document already exists at that path — that
/// document whole, with the model told it is amending rather than writing.
#[implements(spec::TheOpeningNamesTheSlice, spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment)]
pub fn opening(slice: &str, path: &Path) -> String {
    let _ = (slice, path);
    todo!()
}

/// The loop: a human turn, a model turn driven with this module's own executor,
/// the model's settled answer printed, and the judges' turn landed as the next
/// user message when that turn drafted — at most one drafting turn in a row, so
/// every second one is a decision the human makes. Whether the turn drafted and
/// whether the human ended the conversation inside a tool are read from the
/// [`Noted`] its executor kept, once it has settled: a human who typed `done` at
/// a question has already said they are finished, so the loop ends there rather
/// than reading them a second time. It ends when the human ends it — at a
/// prompt or inside `ask` — or with the halt that reached it.
///
/// It answers with the verdict the ending prints, and is the only thing that
/// can: a conversation is watched from here and nowhere else. Every judging's
/// [`Judging::holds`] replaces the one before it, so the answer is the *last*
/// judging's — the sense in which the ending's verdict is "as the last judging
/// found them" — and it is none exactly when no turn in the whole conversation
/// drafted, which is the case a document's presence at that path cannot tell
/// apart from a run that only amended one. The verdict is carried out rather
/// than recomputed at the end: the checks have already run over those bytes.
/// A halt leaves it unanswered, which costs nothing — a halt ends the run with
/// its own sentence and never reaches the ending — and leaves the drafted
/// document where `draft` put it, on disk rather than in the session, so
/// nothing here unwinds what was written.
#[implements(
    spec::TheModelsSettledAnswerIsPrinted,
    spec::TheConversationEndsAtDoneOrEndOfFile,
    spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    spec::TheJudgesAnswerATurnThatDrafted,
    spec::AFailedDraftIsNotATurnThatDrafted,
    spec::ATurnThatDraftedNothingIsAnsweredByTheHuman,
    spec::TheJudgesAnswerAtMostOneDraftingTurnInARow,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
    spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading,
    spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    spec::TheDraftedDocumentSurvivesAHalt,
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
    spec::ARunThatDraftedNothingEndsSayingSo,
)]
pub fn converse(project: &Project, coach: &mut Coach) -> Result<Option<bool>, Halt> {
    let _ = (project, coach);
    todo!()
}

/// What the human typed, from a blocking read of their terminal; none at `done`
/// alone on a line or at end of file, both of which end the conversation.
#[implements(spec::TheConversationEndsAtDoneOrEndOfFile)]
pub fn human_turn() -> Option<String> {
    todo!()
}

/// The coach's three tools: a set of its own, not the canopy client's five.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools)]
pub enum Tool {
    /// A file's text or a directory's entries, through the canopy client's own
    /// read over its own confinement.
    Read,
    /// The slice's LLD, replaced whole.
    Draft,
    /// One question put to the human, answered with what they typed.
    Ask,
}

/// The coaching session's declarations: `read`, `draft` and `ask`, each with
/// the JSON schema of its arguments — `path` with optional `offset` and
/// `limit`; `content` alone, since the one path `draft` can write is the
/// slice's LLD; `question` with optional `options`.
#[implements(spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools, spec::DraftWritesTheSlicesLldAndNoOtherPath)]
pub fn declarations() -> Vec<ToolDecl> {
    todo!()
}

/// The coach's tool dispatch, handed to [`drive`](crate::headless_canopy_agent::turn::drive)
/// as the turn's executor: an `op` the coach did not declare is refused naming
/// it; `read` is answered by the canopy client's `read_tool` over that client's
/// `confine`, so what a read answers is asserted once, there; `draft` writes
/// `path` — the one document this run has, whatever a call's arguments carry —
/// and is recorded in `noted`, answering with that path and the bytes written
/// rather than with a verdict; `ask` puts its question to the human, and is
/// handed the same record, an ending typed there having nowhere else to go. It
/// takes the project because its `read` route is the canopy client's
/// `read_tool` over that client's `confine`, and a confinement needs the
/// workspace root.
#[implements(
    spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    spec::AReadIsRoutedToTheCanopyClientsReadOverItsConfinement,
    spec::DraftWritesTheSlicesLldAndNoOtherPath,
    spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict,
    spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    spec::AFailedDraftIsNotATurnThatDrafted,
)]
pub fn execute(project: &Project, path: &Path, noted: &mut Noted, op: &str, args: &Value) -> ToolResult {
    let _ = (project, path, noted, op, args);
    todo!()
}

/// `draft` over the run's one document: `content` replaces it whole, its
/// directory created, there being no partial edit of it; the bytes written are
/// the answer. It writes to disk rather than into the session, which is what
/// lets a halt in [`converse`] leave the document standing. Which path it is
/// handed is not its own to show: `content` is the whole of the tool's schema
/// ([`declarations`]) and [`execute`] passes the run's one path, so those are
/// where the path is asserted.
#[implements(
    spec::DraftReplacesTheDocumentWholeCreatingItsDirectory,
    spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict,
)]
pub fn draft(path: &Path, content: &str) -> Result<usize, String> {
    let _ = (path, content);
    todo!()
}

/// `ask` over the human's terminal: the question printed with canopy's
/// fifteen-minute invoke stall beside it, once and at no later moment, its
/// `options` printed numbered beneath it; a bare number answers with the option
/// it names and anything else is answered verbatim, an offer never being a
/// constraint. A human who ends the conversation here — `done` or end of file,
/// typed as the answer — is answered with a tool error saying so, a tool having
/// no other channel to say it through, and is recorded in `noted`, which is how
/// the fact reaches [`converse`]: this answers the provider and returns nothing
/// to the loop.
#[implements(
    spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped,
    spec::TheStallWindowIsPrintedOnceBesideTheQuestion,
    spec::EndingTheConversationInsideAskIsAToolError,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
)]
pub fn ask(question: &str, options: &[String], noted: &mut Noted) -> ToolResult {
    let _ = (question, options, noted);
    todo!()
}

/// What one judging produced: whether the four document checks hold, and the
/// message landed as the next user message. The verdict is the reader's
/// neighbour rather than its subject — a reader that could not be consulted
/// lands a sentence in the message and has no bearing on `holds` — and it is
/// what [`converse`] carries out of the loop for the ending to print, the
/// judging being the only thing that ran those checks over what was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Judging {
    /// Whether the four document checks held over the document as it then
    /// stood.
    pub holds: bool,
    /// The judges' message, landed whole as the next user message.
    pub message: String,
}

/// The judges' message for one drafting turn, under a heading saying what it
/// is: the four document checks of `lld-check` over the path `draft` wrote —
/// rendered as they are rendered for a human, and landed whether or not the
/// reader answers — then the reader's findings. The two artifact checks are not
/// here: they are about the project's synced guideline and reader rather than
/// the document, which is all a model whose only writing tool is `draft` can
/// act on. The judging's heading is printed to the terminal before a reader
/// session is opened, since opening one prints a line of its own. A reader that
/// cannot be consulted lands its sentence in place of its findings and is told
/// to the human; it never fails, so it never ends a conversation the human is
/// in the middle of.
///
/// Beside that message it answers with the verdict, and the verdict is those
/// four checks and nothing else: [`Judging::holds`] is whether they held over
/// the document as this judging found it, not whether the reader answered and
/// not a value carried from anywhere. This is where the verdict the ending
/// prints is decided — [`converse`] carries the last one out and [`owed`]
/// prints it, neither of them looking at the document again — so the checks
/// running here over the path `draft` wrote is what makes the ending's "as the
/// last judging found them" true.
#[implements(
    spec::TheJudgesTurnIsTheDocumentChecksThenTheReader,
    spec::TheDocumentChecksRunOverThePathDraftWrote,
    spec::TheArtifactChecksAreNotInTheJudgesTurn,
    spec::TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens,
    spec::AFailedReadersSentenceIsLandedInPlaceOfItsFindings,
    spec::AFailedReaderIsToldToTheHuman,
    spec::AReaderThatCannotBeConsultedDoesNotEndTheConversation,
    spec::TheDocumentChecksAreLandedWhetherOrNotTheReaderAnswers,
    spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading,
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
)]
pub fn judged(project: &Project, coach: &Coach) -> Judging {
    let _ = (project, coach);
    todo!()
}

/// One reader over the document: a session of its own for this judging, dialled
/// with the body at [`lld_review::READER`](crate::lld_review::READER) — that
/// slice's own name for the synced path, not a second copy of it — as its
/// `system`, the canopy client's three
/// observation tools as its declarations, and no phase — a reading belongs to
/// none whose policy would judge it or whose tally would count it. Its turn is
/// run by that client's own dispatch, it is asked for findings as the reader's
/// definition asks for them, and its session is stopped when it answers.
#[implements(
    spec::TheReadersSystemIsTheSyncedReaderBody,
    spec::AReaderSessionDeclaresTheCanopyClientsObservationTools,
    spec::AReaderSessionCarriesNoPhase,
    spec::AReaderTurnIsRunByTheCanopyClientsOwnDispatch,
    spec::TheReaderIsAFreshSessionForEveryJudging,
    spec::TheReaderIsGivenTheDocumentAndAskedForFindings,
    spec::AReaderSessionIsStoppedWhenItAnswers,
    spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost,
)]
pub fn reader_findings(project: &Project, coach: &Coach) -> Result<Vec<String>, String> {
    let _ = (project, coach);
    todo!()
}

/// The artifact checks, run once before the first question: a guideline that
/// cannot be read is the error, it being half the system prompt; a checklist
/// that has drifted from the tool's checks and a reader declaring more than the
/// observation tools are warnings the human is told, the interview proceeding —
/// they are their project's problem rather than this conversation's.
#[implements(
    spec::TheArtifactChecksRunOnceBeforeTheFirstQuestion,
    spec::AnUnreadableGuidelineStopsTheRunNamingItsPath,
    spec::ADriftedChecklistIsReportedAndTheInterviewProceeds,
    spec::AReaderDeclaringTooManyToolsIsReportedAndTheInterviewProceeds,
)]
pub fn setup(project: &Project) -> Result<Vec<String>, String> {
    let _ = project;
    todo!()
}

/// What the run says on the way out: the document's path and whether the
/// document checks hold as the last judging found them, since nothing has
/// written the document since; and what the human owes — reading it once more
/// and committing it as `phase 1: LLD for <slice>`, which the coach does not do.
/// A conversation in which nothing was ever drafted has no verdict — `holds` is
/// then none — and ends saying so: no document at that path, nothing to check,
/// and nothing to commit.
#[implements(
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
    spec::TheEndingNamesThePhaseOneCommitTheCoachDoesNotMake,
    spec::ARunThatDraftedNothingEndsSayingSo,
)]
pub fn owed(path: &Path, holds: Option<bool>) -> String {
    let _ = (path, holds);
    todo!()
}
