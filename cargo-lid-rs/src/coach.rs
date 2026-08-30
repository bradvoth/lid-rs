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
//! **The layer beneath the document's shape.** The items that need a person and
//! a terminal keep no decision of their own: each is a printing and a read
//! around a function over plain data that a test can construct. [`typed`]
//! classifies one line from the human as the conversation's end or their answer,
//! for both [`human_turn`] and [`ask`]; [`replied`] turns that classification
//! into what the model is told, so [`ask`] prints, reads and decides nothing;
//! [`answered`] applies `ask`'s rule for its `options`; [`checks_line`] and
//! [`reader_line`] say how a judging went for the human who is watching; and
//! [`next_after`] decides, from the record one turn's executor kept, whether the
//! judges, the human, or nobody answers it. The loop is handed its first user
//! message rather than building one, so a conversation opens, settles a turn and
//! ends with no input at all — which is what makes a printed answer, a halt and
//! the session's stop things a scripted door can show. The tools are a
//! closed set with something derived from it — [`declarations`] and
//! [`declared`] are both built from [`COACH_TOOLS`] — so the set the session
//! declares and the set its dispatch admits cannot drift apart. And a judging's
//! two losses have one shape between them: a reader that could not be consulted
//! and a document that could not be read back are each a `Result` the judging
//! renders rather than propagates ([`reader_section`], [`checks_section`]), so
//! neither can end a conversation the human is in the middle of.
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
use serde::Deserialize;
use serde_json::Value;

use crate::headless_canopy_agent::door::{Door, Settings, ToolDecl};
use crate::headless_canopy_agent::tools::ToolResult;
use crate::headless_canopy_agent::turn::{Halt, Session};
use crate::lld_review::{Failure, Lld};
use crate::project::Project;
use crate::spec;

/// What `coach` prints beside a rejected argument.
pub const COACH_USAGE: &str = "usage: cargo lid-rs coach [--package <name> | --workspace] [--slice <name>] [--door <url>] [--max-cost <amount>]";

/// `coach [--package <name> | --workspace] [--slice <name>] [--door <url>]
/// [--max-cost <amount>]`: the flags, the key from `CANOPY_KEY` — which never
/// appears on a command line — the project, and the conversation
/// ([`coached`]), which is everything a door can be handed to. The key is read
/// here and nowhere else: a run without it stops naming the variable, as
/// [`api_key`](crate::headless_canopy_agent::api_key) stops one, before any
/// session opens.
pub fn run(args: &[String]) -> Result<(), String> {
    let _ = args;
    todo!()
}

/// One coaching run, from the flags to what the human owes, on the door it is
/// handed: the artifact checks and their warnings before the first question
/// ([`setup`]), the slice ([`slice_of`]) and the one document this run writes
/// ([`package_dir`], [`document_path`]), the coaching session dialled
/// ([`coaching_settings`]) and opened with the document's path printed beside
/// it ([`path_line`]) — the one thing a human can get wrong here that no later
/// check would question — the conversation's first user message built here
/// ([`opening`], which needs the slice and the document this run has and the
/// loop does not) and the conversation run on it ([`converse`]), the session
/// stopped through the door however that conversation ended, and the ending
/// ([`owed`]) printed with the verdict [`converse`] answered with, handed
/// straight on rather than recomputed here. A halt reaching this is the run's
/// error, in the sentence
/// [`halt_reason`](crate::headless_canopy_agent::ending::halt_reason) gives it.
///
/// The door is a parameter rather than built here so that a whole run is
/// drivable against one: everything below this line is decided, and the key
/// and the environment are [`run`]'s alone.
#[implements(
    spec::TheArtifactChecksRunOnceBeforeTheFirstQuestion,
    spec::TheDocumentsPathIsPrintedWhenTheSessionOpens,
    spec::TheCoachingSessionIsStoppedBeforeTheClientExits,
    spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
)]
pub fn coached(project: &Project, door: &Door, flags: &Flags) -> Result<(), String> {
    let _ = (project, door, flags);
    todo!()
}

/// What the coach prints as the coaching session opens: the document it is
/// about to write, so that a human who named the wrong package sees it now
/// rather than after the conversation.
#[implements(spec::TheDocumentsPathIsPrintedWhenTheSessionOpens)]
pub fn path_line(path: &Path) -> String {
    let _ = path;
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

/// The arguments `coach` accepts, as a closed set: the five flags, and nothing
/// else. [`Flag::of`] classifies one argument into it and [`applied`] settles
/// what each one means, so an argument outside the set has one place to be
/// refused and a flag has one place to be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// `--package <name>`: the workspace member the document goes under.
    Package,
    /// `--workspace`: the workspace root, for a slice no package holds.
    Workspace,
    /// `--slice <name>`: the slice the document is written for.
    Slice,
    /// `--door <url>`: the door the run's sessions are dialled on.
    Door,
    /// `--max-cost <amount>`: each session's cost ceiling.
    MaxCost,
}

impl Flag {
    /// An argument classified into the closed set — the one decision over its
    /// name: `--package`, `--workspace`, `--slice`, `--door`, `--max-cost`,
    /// and none for anything else, which [`parse_args`] then rejects by name.
    #[implements(spec::AnyOtherArgumentToCoachIsRejectedByName)]
    pub fn of(argument: &str) -> Option<Flag> {
        let _ = argument;
        todo!()
    }

    /// Whether the flag is followed by its value: every flag but
    /// `--workspace`, which stands alone and so consumes no argument after
    /// it. The one predicate [`pairs`] asks of a flag before taking the
    /// argument that follows it.
    pub fn takes_a_value(self) -> bool {
        todo!()
    }

    /// The flag as it is written on a command line, which is how a rejection
    /// and a stop name it.
    pub fn spelling(self) -> &'static str {
        todo!()
    }
}

/// The flags settled from the arguments: `--package` and `--workspace` refused
/// each other first ([`one_target`], a document having one place), then every
/// `(flag, value)` pair the arguments name ([`pairs`]) applied in turn to the
/// defaults ([`applied`]). A flag absent from the arguments keeps what
/// [`Flags::default`] says — canopy's production door, a budget of five, and
/// no target — and absence is what only this sees, the arguments being here
/// and nowhere else. An argument that names no flag is rejected by name.
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

/// The two flags that name a place refuse each other: arguments carrying both
/// `--package` and `--workspace` stop the run naming them, a document having
/// one place. Asked of the arguments rather than of the settled flags, because
/// one target settled from two flags cannot say it came from two.
#[implements(spec::PackageAndWorkspaceRefuseEachOther)]
pub fn one_target(args: &[String]) -> Result<(), String> {
    let _ = args;
    todo!()
}

/// The arguments as the `(flag, value)` pairs they name: each argument
/// classified ([`Flag::of`]) — one that names no flag is rejected by name —
/// and given the argument after it when the flag takes one
/// ([`Flag::takes_a_value`]), so `--workspace` consumes nothing and every
/// other flag consumes what follows it. A flag at the end of the arguments has
/// no value, which is [`applied`]'s to refuse rather than this walk's: what a
/// missing value costs depends on the flag.
#[implements(spec::AnyOtherArgumentToCoachIsRejectedByName)]
pub fn pairs(args: &[String]) -> Result<Vec<(Flag, Option<String>)>, String> {
    let _ = args;
    todo!()
}

/// One `(flag, value)` pair applied to the flags — the one decision over which
/// flag it is: `--package` and `--workspace` name the target, `--slice` the
/// slice, `--door` the door, `--max-cost` the amount ([`amount`]). Every flag
/// but `--workspace` needs its value ([`named`]), and the key is never a flag.
/// It is called once per pair the arguments carry, so it never sees a flag that
/// is absent: what an absent `--door` or `--max-cost` settles on is
/// [`Flags::default`]'s, and that it is left standing is [`parse_args`]'s.
#[implements(spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName)]
pub fn applied(flags: Flags, pair: (Flag, Option<String>)) -> Result<Flags, String> {
    let _ = (flags, pair);
    todo!()
}

/// The value a flag was given; a flag whose value is not there is rejected
/// naming the flag and saying what it needs.
pub fn named(flag: Flag, value: Option<String>) -> Result<String, String> {
    let _ = (flag, value);
    todo!()
}

/// `--max-cost`'s value as an amount in the provider's currency; one that is
/// not a number is rejected, quoting it.
pub fn amount(value: &str) -> Result<f64, String> {
    let _ = value;
    todo!()
}

/// The slice the document is written for: the flag's value, or — absent it —
/// the current branch's name with `lld/` removed, as
/// [`resolve_slice`](crate::phase::resolve_slice) reads it for every other
/// subcommand. A run on no `lld/<slice>` branch and no flag names no slice,
/// and so has no document to write.
#[implements(spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName)]
pub fn slice_of(project: &Project, given: Option<String>) -> Result<String, String> {
    let _ = (project, given);
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

/// One workspace member as this slice needs it: its name, which `--package`
/// gives, beside its manifest directory, which the document goes under. The
/// project reports the two separately, and pairing them is what lets a name
/// be matched and the members be listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    /// The package's name, as `--package` would name it.
    pub name: String,
    /// The package's manifest directory.
    pub dir: PathBuf,
}

/// The workspace's members, each paired with its manifest directory: what
/// [`named_member`] matches a name against and what [`sole_member`] counts.
pub fn members(project: &Project) -> Vec<Member> {
    let _ = project;
    todo!()
}

/// The directory the document goes under — the one decision over what the
/// flags settled: `--workspace` is the workspace root, where a slice whose
/// product is the workspace rather than a crate keeps its LLD and no package
/// holds it; `--package <name>` is that member's directory ([`named_member`]);
/// no target at all is the workspace's own answer ([`sole_member`]). It needs
/// the project the flags do not have, which is why the flags stop at
/// [`Where`] and this goes on from there.
#[implements(spec::TheWorkspaceFlagNamesTheWorkspaceRoot)]
pub fn package_dir(project: &Project, target: Option<&Where>) -> Result<PathBuf, String> {
    let _ = (project, target);
    todo!()
}

/// The named member's manifest directory: the member of that name, and not the
/// first member found to hold a document of the slice's name — a guess no
/// later check would question. A name matching no member stops the run listing
/// the members, so the human can see what they could have named.
#[implements(
    spec::APackageNamingAMemberIsThatMembersManifestDirectory,
    spec::APackageNamingNoMemberStopsTheRunListingTheMembers,
)]
pub fn named_member(members: &[Member], name: &str) -> Result<PathBuf, String> {
    let _ = (members, name);
    todo!()
}

/// The directory a workspace answers with when neither flag named one — the
/// one decision over how many members it has: exactly one, and that member's
/// directory is the only place the document could go; anything else, and the
/// run stops naming the flag, since inferring a package from the directory the
/// command was run in is the guess this slice exists to refuse.
#[implements(
    spec::NoTargetInAOneMemberWorkspaceIsThatMembersDirectory,
    spec::NoTargetInAWorkspaceOfSeveralMembersStopsNamingTheFlag,
)]
pub fn sole_member(members: &[Member]) -> Result<PathBuf, String> {
    let _ = members;
    todo!()
}

/// The document itself: `docs/intent/<slice>/lld.md` under the directory
/// [`package_dir`] settled — the path the walk will look for it at.
#[implements(spec::TheDocumentIsTheSlicesLldUnderThatDirectory)]
pub fn document_path(package_dir: &Path, slice: &str) -> PathBuf {
    let _ = (package_dir, slice);
    todo!()
}

/// The slice a document's path is for: its parent directory's name. What
/// [`document_path`] built the path from, read back out of it, so that the
/// ending and the judging name the slice the path names and cannot disagree
/// with it.
pub fn slice_named(path: &Path) -> String {
    let _ = path;
    todo!()
}

/// One coaching run: the door its sessions are dialled on, the coaching session
/// it holds, the one document it writes, and the budget every session it opens
/// — this one and each judging's reader — is dialled with, so the run is
/// bounded one session at a time rather than in total. The dialling is done by
/// the items that build a dial — [`coaching_settings`] and [`reader_settings`]
/// — which is where that budget is asserted; this only carries it to them.
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
/// learned the conversation is over. The facts [`next_after`] decides from once
/// the turn has settled — held by the turn rather than by the session, and
/// never inferred from the model's words. A turn starts with none of them
/// true, which is what its `Default` is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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

/// The project's synced copy of the interview method, relative to the
/// workspace root: how to interview, which is the first half of the coaching
/// session's system prompt and prose a person maintains, kept honest by the
/// sync rule rather than by a copy in this module.
pub const METHOD: &str = ".claude/skills/lid-rs/references/coach.md";

/// A synced artifact's text, read from `relative` under the workspace root; one
/// that cannot be read is the error naming the path that was looked for, since
/// what a human does about it is put the file back.
pub fn synced_text(project: &Project, relative: &str) -> Result<String, String> {
    let _ = (project, relative);
    todo!()
}

/// The coaching session's system prompt: the synced interview method
/// ([`METHOD`]), then the synced guideline, in that order — how to interview,
/// and the questions the document will be judged by. The guideline is read from
/// [`lld_review::GUIDELINE`](crate::lld_review::GUIDELINE), the path that slice
/// already names, a second copy of a synced path here being two strings nothing
/// keeps in step. A guideline that cannot be read is the error naming the path,
/// it being half of this, so the run stops here rather than opening a session
/// with half a prompt.
#[implements(
    spec::TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline,
    spec::AnUnreadableGuidelineStopsTheRunNamingItsPath,
)]
pub fn coaching_system(project: &Project) -> Result<String, String> {
    let _ = project;
    todo!()
}

/// The coaching session's dial: `system` the two synced prompts
/// ([`coaching_system`]), the policy built from this host's three declarations
/// ([`declarations`]) and nothing else, empty `params`, and the run's
/// `max_cost` as this session's ceiling.
#[implements(
    spec::TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline,
    spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools,
    spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost,
)]
pub fn coaching_settings(project: &Project, max_cost: f64) -> Result<Settings, String> {
    let _ = (project, max_cost);
    todo!()
}

/// The conversation's first user message — the one decision over whether a
/// document is already at that path: none, and the model is told which slice it
/// is writing one for ([`writing`]); one, and it is told it is amending that
/// document, which the message carries whole ([`amending`]).
#[implements(spec::TheOpeningNamesTheSlice, spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment)]
pub fn opening(slice: &str, path: &Path) -> String {
    let _ = (slice, path);
    todo!()
}

/// The document already at that path, whole; none when there is none to read.
/// A path that cannot be read is a slice with no document yet, which is the
/// ordinary case rather than a fault: the coach is about to write one.
pub fn existing(path: &Path) -> Option<String> {
    let _ = path;
    todo!()
}

/// The opening for a slice with no document: what the slice is, and that the
/// model is writing its LLD.
#[implements(spec::TheOpeningNamesTheSlice)]
pub fn writing(slice: &str) -> String {
    let _ = slice;
    todo!()
}

/// The opening for a slice whose document already exists: what the slice is,
/// that document whole, and that the model is amending it rather than writing
/// one — which is how a Phase 8 amendment is drafted, the commit still being
/// the human's.
#[implements(spec::TheOpeningNamesTheSlice, spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment)]
pub fn amending(slice: &str, document: &str) -> String {
    let _ = (slice, document);
    todo!()
}

/// The loop, opened on the message it is handed: `opening` is landed as the
/// conversation's first user message and a turn is settled on it before the
/// human is read for a line, so the coach's first question is that turn's
/// settled answer — the model speaks first, this being an interview — and a run
/// whose input is already at end of file still opens, settles one turn and ends,
/// rather than returning before a turn exists. Thereafter each message is landed
/// ([`next_message`]), a model turn is driven with this module's own executor,
/// the model's settled answer is printed, and what follows that turn is decided
/// from the [`Noted`] its executor kept ([`next_after`]) — never from the
/// model's words. It ends when the human ends it, at a prompt or inside `ask`,
/// or with the halt that reached it.
///
/// The opening is a parameter because the loop has no slice and no document of
/// its own to build one from, and because a first message that can be handed in
/// is a first turn that can be driven with nothing typed: what this prints, the
/// halt it ends on, and the session the caller stops afterwards are each
/// reachable against a scripted door alone. That first turn answers no judging,
/// so a `draft` in it is answered by the judges like any other.
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
    spec::TheFirstTurnSettlesOnTheOpeningBeforeTheHumanIsRead,
    spec::TheModelsSettledAnswerIsPrinted,
    spec::TheConversationEndsAtDoneOrEndOfFile,
    spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading,
    spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    spec::TheDraftedDocumentSurvivesAHalt,
    spec::ARunThatDraftedNothingEndsSayingSo,
)]
pub fn converse(project: &Project, coach: &mut Coach, opening: &str) -> Result<Option<bool>, Halt> {
    let _ = (project, coach, opening);
    todo!()
}

/// Whose message the model's turn is answering. The loop carries it from one
/// turn to the next because the judges answer at most one drafting turn in a
/// row: a turn answering the judges is the human's to answer, whatever it
/// drafts, so every second drafting turn is a decision the human makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answering {
    /// The turn is answering the human: their own message, or the opening the
    /// loop lands on their behalf before they are read.
    TheHuman,
    /// The turn is answering the judges' findings.
    TheJudges,
}

/// What lands the next user message once a turn has settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// The human, who is read for their next line.
    Human,
    /// The judges, whose findings are landed without the human asking.
    Judges,
    /// Nobody: the conversation was ended inside a tool, and the loop ends
    /// as this turn settles rather than reading the human again.
    Nothing,
}

/// What follows a settled turn, from the record its executor kept and whose
/// message it was answering — the loop's one decision, over plain data, so
/// that a conversation's rules are tested without a terminal or a door: a turn
/// that learned the conversation is over is followed by nothing, whatever else
/// it did; a turn that wrote the document while answering the human is
/// answered by the judges; a turn that wrote it while answering the judges is
/// answered by the human, who can read what changed and say whether to press
/// on; and a turn that wrote nothing — including one whose `draft` failed, a
/// failed draft not being a turn that wrote the document — is answered by the
/// human.
#[implements(
    spec::TheJudgesAnswerATurnThatDrafted,
    spec::AFailedDraftIsNotATurnThatDrafted,
    spec::ATurnThatDraftedNothingIsAnsweredByTheHuman,
    spec::TheJudgesAnswerAtMostOneDraftingTurnInARow,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
)]
pub fn next_after(noted: Noted, answering: Answering) -> Next {
    let _ = (noted, answering);
    todo!()
}

/// One user message as the loop lands it: its text, whose message the turn it
/// opens is answering, and the verdict a judging reached — none for the opening
/// and none for what the human typed, neither of which is a judging. The first
/// one of these is built from the opening [`converse`] was handed; every one
/// after it comes from [`next_message`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landed {
    /// The text landed as the `app.client.user_message`.
    pub text: String,
    /// Whose message the turn that answers it is answering.
    pub answering: Answering,
    /// The verdict this message's judging reached, if it is a judging's.
    pub holds: Option<bool>,
}

/// The next user message — the one decision over what follows the last turn:
/// the human is read for a line, and the conversation ends when they end it;
/// the judges are run and their findings landed ([`judges_turn`]) without the
/// human asking; nobody, and the loop ends.
#[implements(spec::TheJudgesAnswerATurnThatDrafted, spec::TheConversationEndsAtDoneOrEndOfFile)]
pub fn next_message(project: &Project, coach: &Coach, next: Next) -> Option<Landed> {
    let _ = (project, coach, next);
    todo!()
}

/// The judges' turn as the loop lands it: one judging ([`judged`]) over the
/// document as it now stands, its message landed as the next user message, and
/// its verdict carried beside it for [`converse`] to keep.
#[implements(spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading)]
pub fn judges_turn(project: &Project, coach: &Coach) -> Landed {
    let _ = (project, coach);
    todo!()
}

/// What the human types to end the conversation, alone on a line.
pub const DONE: &str = "done";

/// One line from the human, classified — what a person typed, as [`human_turn`]
/// and [`replied`] each go on to decide over it, kept here so that a test can
/// make one without a terminal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Typed {
    /// [`DONE`] alone on a line, or end of file: the conversation is over.
    Ended,
    /// What they typed, without the newline that ended it.
    Answer(String),
}

/// One line from the human classified — the one decision over what a blocking
/// read returned: nothing at all is end of file and ends the conversation;
/// [`DONE`] alone on a line ends it too; anything else is their answer, as
/// typed.
#[implements(spec::TheConversationEndsAtDoneOrEndOfFile)]
pub fn typed(read: Option<String>) -> Typed {
    let _ = read;
    todo!()
}

/// One line from the human's terminal, from a blocking read; none at end of
/// file. The read itself and nothing else: what the line means is [`typed`]'s.
pub fn read_line() -> Option<String> {
    todo!()
}

/// What the human typed at the loop's prompt, or none when they ended the
/// conversation — `done` alone on a line, or end of file, as [`typed`]
/// classifies them.
#[implements(spec::TheConversationEndsAtDoneOrEndOfFile)]
pub fn human_turn() -> Option<String> {
    todo!()
}

/// The coach's three tools: a set of its own, not the canopy client's five.
/// [`COACH_TOOLS`] is the set itself, and both what the session declares
/// ([`declarations`]) and what its dispatch admits ([`declared`]) are derived
/// from it, so no second list of names can drift from this one.
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

/// The tools a coaching session declares: exactly these three, in the order
/// they are declared. The closed set both [`declarations`] and [`declared`]
/// are built from.
pub const COACH_TOOLS: [Tool; 3] = [Tool::Read, Tool::Draft, Tool::Ask];

impl Tool {
    /// The tool's `op` on the wire — `read`, `draft`, `ask` — which is also
    /// the name the model calls it by and the name [`declared`] matches a
    /// forwarded `op` against.
    #[implements(spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools)]
    pub fn op(self) -> &'static str {
        todo!()
    }

    /// The JSON schema of the tool's arguments, its description in the
    /// schema's `description`: `path` with optional `offset` and `limit`;
    /// `content` alone, since the one path `draft` can write is the slice's
    /// LLD and no argument of its can name another; `question` with optional
    /// `options`.
    #[implements(spec::DraftWritesTheSlicesLldAndNoOtherPath)]
    pub fn schema(self) -> Value {
        todo!()
    }
}

/// The coaching session's declarations: one [`declaration`] for each of
/// [`COACH_TOOLS`], and so exactly `read`, `draft` and `ask` — a set of the
/// coach's own rather than the canopy client's five.
#[implements(spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools)]
pub fn declarations() -> Vec<ToolDecl> {
    todo!()
}

/// One tool as the policy declares it: its name — which the model sees and
/// calls it by, and which here is the tool's own `op` — the requestee this
/// program answers as, that `op`, and the schema of its arguments
/// ([`Tool::schema`]).
#[implements(spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools)]
pub fn declaration(tool: Tool) -> ToolDecl {
    let _ = tool;
    todo!()
}

/// The tool a forwarded `op` names, provided [`COACH_TOOLS`] carries one whose
/// [`Tool::op`] it is; an `op` outside that set — the canopy client's `edit`,
/// or anything else — is the refusal naming it, and nothing runs. A tool set
/// is what a session may call, and a call outside it is a fault to report
/// rather than a request to interpret.
#[implements(
    spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    spec::TheCoachDeclaresExactlyTheReadDraftAndAskTools,
)]
pub fn declared(op: &str) -> Result<Tool, String> {
    let _ = op;
    todo!()
}

/// `draft`'s arguments: the document's content and nothing else, the one path
/// it can write being the run's own.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DraftArgs {
    /// What the document is to be replaced with, whole.
    pub content: String,
}

/// `ask`'s arguments: the question, and the options it offers when it offers
/// any.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AskArgs {
    /// The one question put to the human.
    pub question: String,
    /// The answers offered, printed numbered beneath the question; none when
    /// the question is open.
    #[serde(default)]
    pub options: Vec<String>,
}

/// The coach's tool dispatch, handed to [`drive`](crate::headless_canopy_agent::turn::drive)
/// as the turn's executor — the one decision over which of its three tools a
/// forward names ([`declared`], which refuses an `op` the coach did not
/// declare): `read` goes to the canopy client's own read over that client's
/// own confinement ([`read_call`]), so what a read answers is asserted once,
/// there; `draft` is given `path` — the one document this run has, whatever a
/// call's arguments carry, which is why no call of it can reach the module,
/// the claims or a manifest ([`draft_call`]); `ask` is given the same record
/// the turn keeps, an ending typed there having nowhere else to go
/// ([`ask_call`]). It takes the project because its `read` route needs the
/// workspace root a confinement is made against.
#[implements(
    spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    spec::AReadIsRoutedToTheCanopyClientsReadOverItsConfinement,
    spec::DraftWritesTheSlicesLldAndNoOtherPath,
)]
pub fn execute(project: &Project, path: &Path, noted: &mut Noted, op: &str, args: &Value) -> ToolResult {
    let _ = (project, path, noted, op, args);
    todo!()
}

/// `read` as the coach routes it: the canopy client's own
/// [`ReadArgs`](crate::headless_canopy_agent::tools::ReadArgs), its path
/// through that client's [`confine`](crate::headless_canopy_agent::tools::confine)
/// against the workspace root, and that client's
/// [`read_tool`](crate::headless_canopy_agent::tools::read_tool) over it — so a
/// file reaches the coach exactly as it reaches a phase worker, and there is
/// no reason for two answers to one question. No verdict is asked: a coaching
/// session carries no phase.
#[implements(spec::AReadIsRoutedToTheCanopyClientsReadOverItsConfinement)]
pub fn read_call(project: &Project, args: &Value) -> ToolResult {
    let _ = (project, args);
    todo!()
}

/// `draft` as the coach routes it: its [`DraftArgs`], the call recorded in
/// `noted` before the write, the run's one document replaced ([`draft`]), and
/// whether that write succeeded recorded too — a failed draft is not a turn
/// that wrote the document, and the judges do not run for it. It answers with
/// the path and the bytes written ([`wrote_line`]) and not with a verdict: the
/// judges' turn runs the same checks a moment later, so they are run once and
/// read once.
#[implements(
    spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    spec::AFailedDraftIsNotATurnThatDrafted,
    spec::DraftWritesTheSlicesLldAndNoOtherPath,
    spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict,
)]
pub fn draft_call(path: &Path, noted: &mut Noted, args: &Value) -> ToolResult {
    let _ = (path, noted, args);
    todo!()
}

/// What a successful `draft` answers the model with: the path it wrote and the
/// number of bytes, and nothing about whether the document holds — the checks
/// belong to the judges' turn.
#[implements(spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict)]
pub fn wrote_line(path: &Path, bytes: usize) -> String {
    let _ = (path, bytes);
    todo!()
}

/// `ask` as the coach routes it: its [`AskArgs`], and the question put to the
/// human ([`ask`]) with the same record the turn keeps.
pub fn ask_call(noted: &mut Noted, args: &Value) -> ToolResult {
    let _ = (noted, args);
    todo!()
}

/// `draft` over the run's one document: `content` replaces it whole, its
/// directory created, there being no partial edit of it; the bytes written are
/// the answer. It writes to disk rather than into the session, which is what
/// lets a halt in [`converse`] leave the document standing. Which path it is
/// handed is not its own to show: `content` is the whole of the tool's schema
/// ([`Tool::schema`]) and [`execute`] passes the run's one path, so those are
/// where the path is asserted.
#[implements(spec::DraftReplacesTheDocumentWholeCreatingItsDirectory, spec::TheDraftedDocumentSurvivesAHalt)]
pub fn draft(path: &Path, content: &str) -> Result<usize, String> {
    let _ = (path, content);
    todo!()
}

/// What is printed beside a question, once and at no later moment: canopy
/// halts a session whose invoke has gone unanswered for its stall window — the
/// same fifteen minutes the canopy client's
/// [`QUIET_TAIL`](crate::headless_canopy_agent::turn::QUIET_TAIL) waits — so a
/// question left that long ends the conversation with the platform's own
/// sentence. A warning that arrived later would need a clock running beside a
/// blocking read, and a conversation is not worth a thread.
pub const STALL_WINDOW: &str = "canopy ends a session whose question has gone unanswered for fifteen minutes; answer within that, or the conversation ends there";

/// What `ask` answers the model with when the human ends the conversation
/// while its question is outstanding: a tool error, a tool having no other
/// channel to say it through. The model's turn then settles, and the loop ends
/// as it would have anyway, so no ending travels out through a channel the
/// door does not have.
pub const CONVERSATION_OVER: &str = "the conversation is over: the human ended it while this question was outstanding";

/// What one question looks like on the human's terminal: the question, the
/// stall window beside it ([`STALL_WINDOW`]) — printed here, once, as the
/// question is asked and at no later moment — and the options numbered
/// beneath it, if there are any, so that a number is an answer.
#[implements(spec::TheStallWindowIsPrintedOnceBesideTheQuestion)]
pub fn asked(question: &str, options: &[String]) -> String {
    let _ = (question, options);
    todo!()
}

/// The option a bare number names — the number read as a place in the list,
/// counted from one as the question printed it; none when what was typed is
/// not a number, or numbers no option there is.
pub fn option_named<'a>(options: &'a [String], answer: &str) -> Option<&'a String> {
    let _ = (options, answer);
    todo!()
}

/// What `ask` answers the model with — the one decision over what the human
/// typed: a bare number naming one of the options answers with that option;
/// anything else is answered verbatim. The options are an offer, never a
/// constraint, because a closed question the human wants to answer differently
/// is a question that was wrong, and refusing what is not on the list would
/// make the tool decide which answers a design may have.
#[implements(spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped)]
pub fn answered(options: &[String], answer: &str) -> String {
    let _ = (options, answer);
    todo!()
}

/// `ask` over the human's terminal: the question printed with its stall window
/// and its options ([`asked`]), one line read from them ([`read_line`]) and
/// classified ([`typed`]), and what the model is answered with decided over that
/// line rather than here ([`replied`]). This is the terminal and nothing else —
/// a print, a read, and a call — so that every answer `ask` can give is
/// reachable from a line a test wrote, and none of them needs a person.
#[implements(
    spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped,
    spec::TheStallWindowIsPrintedOnceBesideTheQuestion,
)]
pub fn ask(question: &str, options: &[String], noted: &mut Noted) -> ToolResult {
    let _ = (question, options, noted);
    todo!()
}

/// What `ask` answers the model with — the one decision over the line the human
/// gave it ([`typed`]): their answer is passed through under `ask`'s own rule
/// for its options ([`answered`]); their ending is the tool error that says the
/// conversation is over, recorded on the way out ([`ended_in_ask`]). The
/// decision is here, over a classified line, because the only way to reach it
/// through [`ask`] is to be a person at a terminal.
#[implements(
    spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped,
    spec::EndingTheConversationInsideAskIsAToolError,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
)]
pub fn replied(options: &[String], line: &Typed, noted: &mut Noted) -> ToolResult {
    let _ = (options, line, noted);
    todo!()
}

/// The conversation ended inside `ask`: the tool error saying so
/// ([`CONVERSATION_OVER`]), because a tool has no other channel to say it
/// through, and the fact recorded in `noted`, because that record is the only
/// thing that carries it back out to [`converse`] — `ask` answers the provider
/// and returns nothing to the loop. Both in one place, so an ending cannot be
/// answered without being recorded, which would leave the loop reading a human
/// who has said they are finished.
#[implements(
    spec::EndingTheConversationInsideAskIsAToolError,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
)]
pub fn ended_in_ask(noted: &mut Noted) -> ToolResult {
    let _ = noted;
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
    /// stood, as [`checks_hold`] reads them. A judging that could not read the
    /// document back ran no check over it, and a verdict is a statement about
    /// checks that ran, so that judging's is that they do not hold.
    pub holds: bool,
    /// The judges' message, landed whole as the next user message.
    pub message: String,
}

/// The heading the judges' message is landed under, and the line printed to
/// the terminal before a reader session is opened: a reader of the sealed log
/// can tell the judges' turn from the human's by it, and a human watching can
/// tell whose opening line follows.
pub const JUDGING_HEADING: &str = "## The judges";

/// What the judges' message says of the document checks when none of them
/// failed.
pub const EVERY_CHECK_HELD: &str = "the four document checks hold over what you wrote";

/// What is said in place of the document checks' failures when the document
/// could not be read back for them to run over — landed to the model and told
/// to the human alike, with the read's own sentence beside it. It says no check
/// ran, because a message that only named a failed read would read as a
/// document that failed one.
pub const DOCUMENT_UNREADABLE: &str = "the document could not be read back, so none of the four document checks ran over it";

/// What is said in place of the reader's findings when the reader could not be
/// answered for — landed to the model and told to the human alike.
pub const READER_UNCONSULTED: &str = "the reader could not be consulted";

/// One judging over the document as it now stands: the four document checks
/// over the path `draft` wrote ([`document_failures`]); the judging's heading
/// printed to the terminal *before* a reader session is opened, since opening
/// one prints a line of its own and that line belongs to the judging; the
/// reader consulted ([`reader_findings`]); the human told how it went
/// ([`judging_line`]); and the two landed as one message ([`judges_message`])
/// with the verdict [`checks_hold`] reads beside it.
///
/// It answers rather than fails, and that is what makes both of a judging's
/// losses cheap. A reader that cannot be consulted lands its sentence in place
/// of its findings; a document that cannot be read back — `draft` reported the
/// bytes it wrote, the checks read the path again, and between those two
/// moments a file can go away — lands the read's own sentence in place of the
/// checks' failures. Neither ends a conversation the human is in the middle of,
/// and neither is a branch here: each is a `Result` the sections and the
/// verdict decide over, so this stays five statements and no branch. And the
/// verdict beside that message is [`checks_hold`]'s reading of those four checks
/// and nothing else — not whether the reader answered, and not a value carried
/// from anywhere — so what [`converse`] carries out of the loop is a statement
/// about the checks this judging ran over those bytes, or about a judging that
/// could run none. It prints no ending: the path and the verdict a human is left
/// with are [`owed`]'s, one conversation later.
#[implements(
    spec::TheJudgesTurnIsTheDocumentChecksThenTheReader,
    spec::TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens,
    spec::TheReaderIsAFreshSessionForEveryJudging,
    spec::TheHumanIsToldHowTheDocumentChecksFoundTheDocument,
    spec::AnUnreadableDocumentIsToldToTheHuman,
    spec::AFailedReaderIsToldToTheHuman,
    spec::AReaderThatCannotBeConsultedDoesNotEndTheConversation,
    spec::ADocumentThatCannotBeReadBackDoesNotEndTheConversation,
)]
pub fn judged(project: &Project, coach: &Coach) -> Judging {
    let _ = (project, coach);
    todo!()
}

/// What the four document checks of `lld-check` — `DecisionsExist`,
/// `Alternatives`, `ShapeRows`, `DeferredNumbered` — answered for one judging:
/// every failure they found over the path `draft` wrote ([`document`]) rather
/// than over a path resolved again; or, when that path could not be read back,
/// the read's own sentence, no check having run over anything. That is the one
/// thing a judging's checks can fail to have, and it is a `Result` for the same
/// reason [`reader_findings`] is one — the caller renders it and carries on.
/// The two artifact checks are not among them: they are about the project's
/// synced guideline and reader rather than the document, which is all a model
/// whose only writing tool is `draft` can act on.
#[implements(spec::TheDocumentChecksRunOverThePathDraftWrote, spec::TheArtifactChecksAreNotInTheJudgesTurn)]
pub fn document_failures(path: &Path) -> Result<Vec<Failure>, String> {
    let _ = path;
    todo!()
}

/// The document as the four checks read it: the path `draft` wrote, its lines,
/// and the slice its parent directory names ([`slice_named`]). Built here
/// rather than through [`Lld::read`](crate::lld_review::Lld::read), which
/// would resolve a path again and could find another. A path that cannot be
/// read back is the read's own sentence, naming the path it looked for — the
/// file `draft` reported writing can go away between that report and this
/// read, and this is where a judging learns it did.
pub fn document(path: &Path) -> Result<Lld, String> {
    let _ = path;
    todo!()
}

/// The document checks as the judges' message carries them — the one decision
/// over whether they ran at all: they did, and it is what they found
/// ([`failures_section`]); the document could not be read back, and the read's
/// own sentence stands in place of their failures, under a line saying no check
/// ran ([`DOCUMENT_UNREADABLE`]), so that a model reading the judging is not
/// told a document it can no longer see either holds or fails.
#[implements(spec::AnUnreadableDocumentsSentenceIsLandedInPlaceOfTheChecksFailures)]
pub fn checks_section(checks: &Result<Vec<Failure>, String>) -> String {
    let _ = checks;
    todo!()
}

/// What the checks found, as the judges' message carries it — the one decision
/// over whether any failed: those that did, rendered as `lld-check` renders
/// them for a human, file and line and rule apiece; and a line saying they
/// hold ([`EVERY_CHECK_HELD`]) when none did, since a message that said
/// nothing of them would read as a message that ran none of them.
#[implements(
    spec::TheDocumentChecksRunOverThePathDraftWrote,
    spec::AJudgingWhoseChecksAllHeldSaysSo,
)]
pub fn failures_section(failures: &[Failure]) -> String {
    let _ = failures;
    todo!()
}

/// The verdict one judging reached, and the whole of what [`Judging::holds`]
/// carries: the four document checks ran, and none of them failed. A judging
/// that could not read the document back ran none, and a verdict is a statement
/// about checks that ran, so an unreadable document does not hold. The reader
/// has no part in this — a lost second opinion is the verdict's neighbour, not
/// its subject — and neither has anything else: the checks' own answer is the
/// whole of what is read here, which is what makes the verdict the ending prints
/// a statement about the four document checks and nothing besides.
#[implements(
    spec::AJudgingsVerdictIsTheFourDocumentChecksAndNothingElse,
    spec::AnUnreadableDocumentsVerdictIsThatTheChecksDoNotHold,
)]
pub fn checks_hold(checks: &Result<Vec<Failure>, String>) -> bool {
    let _ = checks;
    todo!()
}

/// The reader's part of the judges' message — the one decision over whether it
/// answered: its findings, landed whole rather than summarised, since which
/// findings matter is the human's judgment; or, when it could not be consulted,
/// that sentence in their place ([`READER_UNCONSULTED`]) with the reason
/// beside it, so the model knows a second opinion was meant to be here and is
/// not.
#[implements(spec::AFailedReadersSentenceIsLandedInPlaceOfItsFindings)]
pub fn reader_section(findings: &Result<Vec<String>, String>) -> String {
    let _ = findings;
    todo!()
}

/// The judges' message, landed whole as the next user message: the heading
/// that says what it is ([`JUDGING_HEADING`]), then the document checks
/// ([`checks_section`]), then the reader ([`reader_section`]) — in that order,
/// and the checks' part in it whether or not the reader answered, since it is
/// what the model can act on either way, and whether or not the document could
/// be read back, since a judging that could read nothing must still say that
/// rather than nothing.
#[implements(
    spec::TheJudgesTurnIsTheDocumentChecksThenTheReader,
    spec::TheDocumentChecksAreLandedWhetherOrNotTheReaderAnswers,
    spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading,
)]
pub fn judges_message(checks: &Result<Vec<Failure>, String>, findings: &Result<Vec<String>, String>) -> String {
    let _ = (checks, findings);
    todo!()
}

/// What the human is told when a judging has run: how the document checks found
/// the document ([`checks_line`]), and what became of the reader
/// ([`reader_line`]), in that order — the order the message the model was given
/// carries them in. Both are decided over the judging's two answers rather than
/// here, so this joins two lines and chooses nothing.
pub fn judging_line(checks: &Result<Vec<Failure>, String>, findings: &Result<Vec<String>, String>) -> String {
    let _ = (checks, findings);
    todo!()
}

/// How the document checks found the document, as the human watching is told it
/// — the one decision over what those checks answered: none of them failed, and
/// they are told it holds; some did, and they are told how many, the failures
/// themselves having gone to the model; the document could not be read back, and
/// they are told that no check ran over it ([`DOCUMENT_UNREADABLE`]), since they
/// are the only one who can put back a document that has gone from under the
/// run. A judging says how it found the document either way: this line is where
/// a run tells the human whether what was just written holds.
#[implements(
    spec::TheHumanIsToldHowTheDocumentChecksFoundTheDocument,
    spec::AnUnreadableDocumentIsToldToTheHuman,
)]
pub fn checks_line(checks: &Result<Vec<Failure>, String>) -> String {
    let _ = checks;
    todo!()
}

/// What the human is told of the reader — the one decision over whether it
/// answered: it did, and there is nothing here to tell them, its findings having
/// gone to the model, which is the empty text this gives back; it could not be
/// consulted ([`READER_UNCONSULTED`]), and they are told so with the reason
/// beside it, because the second opinion they were about to be given did not
/// arrive and a conversation carrying on without a word about it is the one way
/// a lost reader passes unnoticed.
#[implements(spec::AFailedReaderIsToldToTheHuman)]
pub fn reader_line(findings: &Result<Vec<String>, String>) -> String {
    let _ = findings;
    todo!()
}

/// The reader's system prompt: the synced
/// [`lld_review::READER`](crate::lld_review::READER) — that slice's own name
/// for the path, not a second copy of it — without its frontmatter, as the
/// canopy client reads an agent's body.
#[implements(spec::TheReadersSystemIsTheSyncedReaderBody)]
pub fn reader_body(project: &Project) -> Result<String, String> {
    let _ = project;
    todo!()
}

/// A reader session's dial: `system` the reader's synced body
/// ([`reader_body`]), the policy built from the canopy client's declarations
/// of its three observation tools — a reading observes and cannot act — empty
/// `params`, and the run's `max_cost`, which every session the coach opens is
/// dialled with.
#[implements(
    spec::TheReadersSystemIsTheSyncedReaderBody,
    spec::AReaderSessionDeclaresTheCanopyClientsObservationTools,
    spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost,
)]
pub fn reader_settings(project: &Project, max_cost: f64) -> Result<Settings, String> {
    let _ = (project, max_cost);
    todo!()
}

/// The reader's user message: the document's path, and the findings asked for
/// in the form the reader's own definition asks for them, so that what comes
/// back is what a phase would have been given.
#[implements(spec::TheReaderIsGivenTheDocumentAndAskedForFindings)]
pub fn reader_prompt(path: &Path) -> String {
    let _ = path;
    todo!()
}

/// One reader over the document: a session of its own for this judging —
/// fresh, because a reader that remembers its last reading reads the diff
/// rather than the document a phase will receive — dialled by
/// [`reader_settings`] and carrying no phase, a reading belonging to none
/// whose policy would judge it or whose tally would count it. Its turn is run
/// by the canopy client's own dispatch rather than by the coach's, `drive`
/// taking the executor for a turn as a parameter; it is asked for findings
/// ([`reader_prompt`]); its numbered findings are the answer; and its session
/// is stopped when it answers. Anything that stops it short — a halt, a
/// refusal, a quiet tail, a budget — is the error [`judged`] lands in place of
/// findings rather than one that ends the conversation.
#[implements(
    spec::TheReaderIsAFreshSessionForEveryJudging,
    spec::AReaderSessionDeclaresTheCanopyClientsObservationTools,
    spec::AReaderSessionCarriesNoPhase,
    spec::AReaderTurnIsRunByTheCanopyClientsOwnDispatch,
    spec::TheReaderIsGivenTheDocumentAndAskedForFindings,
    spec::AReaderSessionIsStoppedWhenItAnswers,
)]
pub fn reader_findings(project: &Project, coach: &Coach) -> Result<Vec<String>, String> {
    let _ = (project, coach);
    todo!()
}

/// The artifact checks, run once before the first question: `lld-check`'s two
/// checks over the project's synced guideline and reader, each failure told to
/// the human as a warning ([`warning`]) and none of them stopping the
/// interview — a checklist that has drifted from the tool's checks and a reader
/// declaring more than the observation tools are their project's problem rather
/// than this conversation's. The error is reserved for a project whose root
/// cannot be located; a guideline that cannot be *read* stops the run in
/// [`coaching_system`], which is what needs it.
#[implements(
    spec::TheArtifactChecksRunOnceBeforeTheFirstQuestion,
    spec::ADriftedChecklistIsReportedAndTheInterviewProceeds,
    spec::AReaderDeclaringTooManyToolsIsReportedAndTheInterviewProceeds,
)]
pub fn setup(project: &Project) -> Result<Vec<String>, String> {
    let _ = project;
    todo!()
}

/// One artifact check's failure as the human is told it: the file, the line,
/// and what it found — which check the checklist omits, or what the reader
/// declares — said as a warning the interview goes on past.
pub fn warning(failure: &Failure) -> String {
    let _ = failure;
    todo!()
}

/// What the ending says of the document checks when the last judging found
/// them holding.
pub const CHECKS_HOLD: &str = "the document checks hold";

/// What it says when the last judging found them failing.
pub const CHECKS_DO_NOT_HOLD: &str = "the document checks do not hold";

/// What the run says on the way out — the one decision over whether anything
/// was ever drafted: nothing was, and it says so ([`nothing_drafted`]); or
/// something was, and it says where the document is, whether the checks hold as
/// the last judging found them — nothing having written the document since —
/// and what the human owes ([`document_owed`]).
#[implements(spec::TheEndingPrintsThePathAndWhetherTheChecksHold, spec::ARunThatDraftedNothingEndsSayingSo)]
pub fn owed(path: &Path, holds: Option<bool>) -> String {
    let _ = (path, holds);
    todo!()
}

/// The ending for a conversation in which nothing was ever drafted: there is
/// no document at that path, nothing to check, and nothing to commit.
#[implements(spec::ARunThatDraftedNothingEndsSayingSo)]
pub fn nothing_drafted(path: &Path) -> String {
    let _ = path;
    todo!()
}

/// The ending for a conversation that drafted: the document's path, the
/// verdict the last judging reached ([`CHECKS_HOLD`], [`CHECKS_DO_NOT_HOLD`]),
/// and what the human owes — reading it once more, and committing it as
/// `phase 1: LLD for <slice>`, which the coach does not do, Phase 1 being the
/// human's. The slice it names is the path's own ([`slice_named`]), which
/// [`document_path`] built from the slice and so cannot disagree with it.
#[implements(
    spec::TheEndingPrintsThePathAndWhetherTheChecksHold,
    spec::TheEndingNamesThePhaseOneCommitTheCoachDoesNotMake,
)]
pub fn document_owed(path: &Path, verdict: &str) -> String {
    let _ = (path, verdict);
    todo!()
}
