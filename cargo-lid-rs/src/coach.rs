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
//! of its own — `read`, `grep`, `draft`, `ask` — and the dispatch that routes a
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
//! for [`human_turn`] — which the loop hands the line rather than the terminal,
//! [`read_line`] being the whole of what this module does at one — and for
//! [`ask`]; [`replied`] turns that classification
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
//! **What a watched run shows.** An interview is watched, so a turn shows its
//! work rather than claiming it is happening: [`narrate`] prints the model's own
//! words as `drive` hands them over, and [`executed`] announces a call before it
//! routes it, the line itself decided by [`announced`] over the `op` as a string
//! — because a reader session's calls are announced by the same function and it
//! calls tools this host does not declare. What the repository holds is landed
//! once, in [`preamble`], rather than walked for a directory listing at a time
//! on every run.
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
//
// This module's own items are written out below for the same reason. This
// documentation is assembled from two places — the LLD included at `pub mod
// coach;` in `lib.rs`, and the prose above — and a block merged from a
// declaration in another file resolves its links in that file's scope, where
// nothing of this module's is in scope by its bare name.
//
//! [`Noted`]: crate::coach::Noted
//! [`Judging`]: crate::coach::Judging
//! [`typed`]: crate::coach::typed
//! [`human_turn`]: crate::coach::human_turn
//! [`read_line`]: crate::coach::read_line
//! [`ask`]: crate::coach::ask
//! [`replied`]: crate::coach::replied
//! [`answered`]: crate::coach::answered
//! [`checks_line`]: crate::coach::checks_line
//! [`reader_line`]: crate::coach::reader_line
//! [`next_after`]: crate::coach::next_after
//! [`declarations`]: crate::coach::declarations
//! [`declared`]: crate::coach::declared
//! [`COACH_TOOLS`]: crate::coach::COACH_TOOLS
//! [`checks_section`]: crate::coach::checks_section
//! [`reader_section`]: crate::coach::reader_section
//! [`narrate`]: crate::coach::narrate
//! [`executed`]: crate::coach::executed
//! [`announced`]: crate::coach::announced
//! [`preamble`]: crate::coach::preamble

use std::path::{Path, PathBuf};

use lid_rs::implements;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::headless_canopy_agent::door::{Door, Settings, ToolDecl, policy_for};
use crate::headless_canopy_agent::ending::{halt_reason, numbered, without_frontmatter};
use crate::headless_canopy_agent::tools::{
    REQUESTEE, ReadArgs, Tool as CanopyTool, ToolResult, arguments, confine, declarations as canopy_declarations, execute as canopy_execute, read_tool,
    schema_of,
};
use crate::headless_canopy_agent::turn::{Halt, Session, drive, silent};
use crate::headless_canopy_agent::{DEFAULT_MAX_COST, KEY_VARIABLE, PRODUCTION_DOOR, api_key};
use crate::lld_review::{
    Failure, GUIDELINE, Lld, READER, alternatives, decisions_exist, deferred_numbered, guideline_names_every_check, reader_observes_only, rendered, shape_rows,
};
use crate::phase::resolve_slice;
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
    let flags = parse_args(args)?;
    let key = api_key(std::env::var(KEY_VARIABLE).ok()).map_err(|stop| stop.decisions.join("\n"))?;
    let project = Project::load()?;
    coached(&project, &Door::new(&flags.door, &key), &flags)
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
    setup(project)?.iter().for_each(|warning| println!("{warning}"));
    let slice = slice_of(project, flags.slice.clone())?;
    let path = document_path(&package_dir(project, flags.target.as_ref())?, &slice);
    let settings = coaching_settings(project, flags.max_cost)?;
    println!("{}", path_line(&path));
    let session = Session::open(door, &settings, None, declarations())?;
    let opened_with = opening(project, &slice, &path);
    let mut coach = Coach { door: door.clone(), session, path, max_cost: flags.max_cost };
    let ended = converse(project, &mut coach, &opened_with);
    let sealed = coach.session.stop().map_err(halt_reason);
    let holds = ended.map_err(halt_reason).and_then(|verdict| sealed.map(|()| verdict))?;
    println!("{}", owed(&coach.path, holds));
    Ok(())
}

/// What the coach prints as the coaching session opens: the document it is
/// about to write, so that a human who named the wrong package sees it now
/// rather than after the conversation.
#[implements(spec::TheDocumentsPathIsPrintedWhenTheSessionOpens)]
pub fn path_line(path: &Path) -> String {
    format!("this run writes {}", path.display())
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
        Self { target: None, slice: None, door: PRODUCTION_DOOR.to_string(), max_cost: DEFAULT_MAX_COST }
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
        match argument {
            "--package" => Some(Flag::Package),
            "--workspace" => Some(Flag::Workspace),
            "--slice" => Some(Flag::Slice),
            "--door" => Some(Flag::Door),
            "--max-cost" => Some(Flag::MaxCost),
            _ => None,
        }
    }

    /// Whether the flag is followed by its value: every flag but
    /// `--workspace`, which stands alone and so consumes no argument after
    /// it. The one predicate [`pairs`] asks of a flag before taking the
    /// argument that follows it.
    pub fn takes_a_value(self) -> bool {
        self != Flag::Workspace
    }

    /// The flag as it is written on a command line, which is how a rejection
    /// and a stop name it.
    pub fn spelling(self) -> &'static str {
        match self {
            Flag::Package => "--package",
            Flag::Workspace => "--workspace",
            Flag::Slice => "--slice",
            Flag::Door => "--door",
            Flag::MaxCost => "--max-cost",
        }
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
    one_target(args)?;
    pairs(args)?.into_iter().try_fold(Flags::default(), applied)
}

/// The two flags that name a place refuse each other: arguments carrying both
/// `--package` and `--workspace` stop the run naming them, a document having
/// one place. Asked of the arguments rather than of the settled flags, because
/// one target settled from two flags cannot say it came from two.
#[implements(spec::PackageAndWorkspaceRefuseEachOther)]
pub fn one_target(args: &[String]) -> Result<(), String> {
    let named = |flag: Flag| args.iter().any(|argument| argument == flag.spelling());
    let both = named(Flag::Package) && named(Flag::Workspace);
    (!both).then_some(()).ok_or_else(|| {
        format!(
            "`{}` and `{}` refuse each other: the document has one place, so name it once\n{COACH_USAGE}",
            Flag::Package.spelling(),
            Flag::Workspace.spelling()
        )
    })
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
    let mut named = Vec::new();
    let mut rest = args;
    while let Some((argument, after)) = rest.split_first() {
        let flag = Flag::of(argument).ok_or_else(|| format!("unknown argument `{argument}` for coach\n{COACH_USAGE}"))?;
        let takes = flag.takes_a_value();
        named.push((flag, after.first().filter(|_| takes).cloned()));
        rest = after.get(usize::from(takes)..).unwrap_or_default();
    }
    Ok(named)
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
    let (flag, value) = pair;
    match flag {
        Flag::Package => Ok(Flags { target: Some(Where::Package(named(flag, value)?)), ..flags }),
        Flag::Workspace => Ok(Flags { target: Some(Where::Workspace), ..flags }),
        Flag::Slice => Ok(Flags { slice: Some(named(flag, value)?), ..flags }),
        Flag::Door => Ok(Flags { door: named(flag, value)?, ..flags }),
        Flag::MaxCost => Ok(Flags { max_cost: amount(&named(flag, value)?)?, ..flags }),
    }
}

/// The value a flag was given; a flag whose value is not there is rejected
/// naming the flag and saying what it needs.
pub fn named(flag: Flag, value: Option<String>) -> Result<String, String> {
    value.ok_or_else(|| format!("the flag `{}` for coach needs a value\n{COACH_USAGE}", flag.spelling()))
}

/// `--max-cost`'s value as an amount in the provider's currency; one that is
/// not a number is rejected, quoting it.
pub fn amount(value: &str) -> Result<f64, String> {
    value.parse::<f64>().map_err(|_| format!("`{value}` is not an amount for --max-cost, in the provider's currency"))
}

/// The slice the document is written for: the flag's value, or — absent it —
/// the current branch's name with `lld/` removed, as
/// [`resolve_slice`](crate::phase::resolve_slice) reads it for every other
/// subcommand. A run on no `lld/<slice>` branch and no flag names no slice,
/// and so has no document to write.
#[implements(spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName)]
pub fn slice_of(project: &Project, given: Option<String>) -> Result<String, String> {
    resolve_slice(project, given)?.ok_or_else(|| format!("no slice: the branch is not named `lld/<slice>`\n{COACH_USAGE}"))
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
    project
        .member_manifest_dirs()
        .into_iter()
        .filter_map(|dir| project.package_at(&dir.join("Cargo.toml")).map(|name| Member { name, dir }))
        .collect()
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
    match target {
        Some(Where::Workspace) => project.root(),
        Some(Where::Package(name)) => named_member(&members(project), name),
        None => sole_member(&members(project)),
    }
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
    let listed: Vec<&str> = members.iter().map(|member| member.name.as_str()).collect();
    members
        .iter()
        .find(|member| member.name == name)
        .map(|member| member.dir.clone())
        .ok_or_else(|| format!("no workspace member is named `{name}`; the members are {}", listed.join(", ")))
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
    match members {
        [only] => Ok(only.dir.clone()),
        [] | [_, _, ..] => Err(format!(
            "this workspace has {} members: name the one the document goes under with `--package <name>`, or `--workspace` for a slice whose product is the workspace",
            members.len()
        )),
    }
}

/// The document itself: `docs/intent/<slice>/lld.md` under the directory
/// [`package_dir`] settled — the path the walk will look for it at.
#[implements(spec::TheDocumentIsTheSlicesLldUnderThatDirectory)]
pub fn document_path(package_dir: &Path, slice: &str) -> PathBuf {
    package_dir.join("docs/intent").join(slice).join("lld.md")
}

/// The slice a document's path is for: its parent directory's name. What
/// [`document_path`] built the path from, read back out of it, so that the
/// ending and the judging name the slice the path names and cannot disagree
/// with it.
pub fn slice_named(path: &Path) -> String {
    path.parent().and_then(Path::file_name).map(|slice| slice.to_string_lossy().into_owned()).unwrap_or_default()
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
    let path = project.root()?.join(relative);
    std::fs::read_to_string(&path).map_err(|unreadable| format!("reading {}: {unreadable}", path.display()))
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
    let method = synced_text(project, METHOD)?;
    let guideline = synced_text(project, GUIDELINE)?;
    Ok(format!("{method}\n\n{guideline}"))
}

/// The coaching session's dial: `system` the two synced prompts
/// ([`coaching_system`]), the policy built from this host's four declarations
/// ([`declarations`]) and nothing else, empty `params`, and the run's
/// `max_cost` as this session's ceiling.
#[implements(
    spec::TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline,
    spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools,
    spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost,
)]
pub fn coaching_settings(project: &Project, max_cost: f64) -> Result<Settings, String> {
    Ok(Settings { system: coaching_system(project)?, policy: policy_for(&declarations()), params: json!({}), max_cost })
}

/// The conversation's first user message — the one decision over whether a
/// document is already at that path: none, and the model is told which slice it
/// is writing one for ([`writing`]); one, and it is told it is amending that
/// document, which the message carries whole ([`amending`]). Both are handed
/// what the repository holds ([`preamble`]) to lead with, so the walk of
/// directories a model would otherwise spend its first minutes on is done once,
/// by the coach, and the same walk is not repeated a listing at a time on every
/// run.
#[implements(
    spec::TheOpeningNamesTheSlice,
    spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment,
    spec::TheOpeningLeadsWithWhatTheRepositoryHolds,
)]
pub fn opening(project: &Project, slice: &str, path: &Path) -> String {
    let repository = preamble(project, path);
    match existing(path) {
        Some(document) => amending(&repository, slice, &document),
        None => writing(&repository, slice),
    }
}

/// What the repository holds, as the opening carries it: the intent index
/// ([`index_section`]), then the sole HLD ([`hld_section`]), then the project's
/// guidance ([`guidance_section`]), in that order. The index's paths
/// ([`intent_paths`]) are walked once and handed to both the section that lists
/// them and the one that decides whether exactly one HLD is among them, so the
/// two cannot disagree about what the workspace holds.
///
/// The workspace root goes to the index beside them, this being where it is
/// already held: the paths are in the form the walk produced, and each row is
/// named against the root as it is rendered. A project whose root cannot be
/// located stands in the empty path and has no rows to render it against
/// either, `intent_paths` finding nothing under a root it could not find.
#[implements(spec::ThePreambleIsTheIndexTheHldThenTheGuidance)]
pub fn preamble(project: &Project, path: &Path) -> String {
    let paths = intent_paths(project);
    let root = project.root().unwrap_or_default();
    format!("{}\n\n{}\n\n{}", index_section(&root, &paths, path), hld_section(&paths), guidance_section(project))
}

/// The name every HLD is filed under, which is how one is told from an `lld.md`
/// both when the index is walked ([`documents_in`]) and when the HLDs among the
/// index's paths are counted ([`hlds`]) — one spelling, so the document the
/// index names and the one the opening carries cannot be different files.
pub const HLD_FILE: &str = "hld.md";

/// Every `docs/intent` document in the workspace: the HLD at the workspace root
/// and under each member, and every slice's `<slice>/lld.md` under either,
/// sorted, so that one run's index and the next's agree. Sorted also puts a
/// directory named twice — a workspace whose one member sits at its root — next
/// to itself, which is what lets the repeat be dropped.
///
/// The paths are the ones the walk produced, as [`document_path`] holds this
/// run's own: [`hld_section`] opens one of them, and a row compares one against
/// this run's document. What the model is shown is another form — relative to
/// the workspace root, which is the only form `confine` accepts — and that
/// conversion is [`relative_to`]'s, made at the row where its one reader is —
/// not here, where the HLD that must be opened and the comparison that marks a
/// row both want the form the walk answered with.
#[implements(
    spec::TheIntentIndexNamesEveryIntentDocumentInTheWorkspace,
    spec::TheIntentIndexIsSortedSoTwoRunsAgree,
)]
pub fn intent_paths(project: &Project) -> Vec<PathBuf> {
    let mut documents: Vec<PathBuf> = intent_dirs(project).iter().flat_map(|intent| documents_in(intent)).collect();
    documents.sort();
    documents.dedup();
    documents
}

/// The `docs/intent` directories the index is walked from: the workspace
/// root's, and each member's ([`members`], which already answers what the
/// members are). A project whose root cannot be located has none of them,
/// every one being under it.
#[implements(spec::TheIntentIndexNamesEveryIntentDocumentInTheWorkspace)]
pub fn intent_dirs(project: &Project) -> Vec<PathBuf> {
    todo!("the workspace root's `docs/intent` and each member's")
}

/// The intent documents in one `docs/intent` directory: its [`HLD_FILE`], and
/// the `lld.md` of each slice directory under it ([`subdirectories`]) — of
/// those that are there, a directory holding neither being a package with no
/// intent documents rather than a fault.
#[implements(spec::TheIntentIndexNamesEveryIntentDocumentInTheWorkspace)]
pub fn documents_in(intent: &Path) -> Vec<PathBuf> {
    todo!("that directory's HLD and each slice's LLD, of those that are there")
}

/// The directories directly under `dir` — a `docs/intent`'s slice directories —
/// in whatever order the filesystem answers with, [`intent_paths`] being where
/// the order is settled. A directory that cannot be read holds none, which is
/// a package with no documents rather than a fault, as [`members`] treats a
/// member with no manifest name.
pub fn subdirectories(dir: &Path) -> Vec<PathBuf> {
    todo!("the directories directly under it")
}

/// The heading the index is carried under, so that a model reading the opening
/// can tell what the workspace holds from what this run is.
pub const INDEX_HEADING: &str = "## The intent documents this workspace holds";

/// What the row naming this run's own document is marked with: the one path in
/// the index the model is about to write rather than consult.
pub const THIS_RUNS_DOCUMENT: &str = "this run's own document, the one you are writing rather than one to consult";

/// The index as the opening lists it: those paths and not their text — a first
/// message carrying twelve documents buries the one that mattered — each named
/// against `root` and the row naming `path`, this run's own document, marked as
/// the one the model is about to write rather than consult ([`index_row`], where
/// both are settled one row at a time). An index of nothing is its heading and
/// no rows, a workspace with no intent documents being where a first slice
/// starts.
///
/// It takes the root because the paths do not carry their own relation to it:
/// [`intent_paths`] answers in the form the filesystem walk produced, which is
/// what [`hld_section`] opens one of and what a row compares against this run's
/// own document, and the one conversion into the form `confine` accepts sits at
/// the row, where its one reader is.
#[implements(
    spec::EveryIndexRowNamesItsDocumentRelativeToTheWorkspaceRoot,
    spec::ThisRunsOwnDocumentIsMarkedInTheIndex,
    spec::TheDocumentsTheIndexNamesAreNamedAndNotCarried,
)]
pub fn index_section(root: &Path, paths: &[PathBuf], path: &Path) -> String {
    let rows: Vec<String> = paths.iter().map(|document| index_row(root, document, path)).collect();
    format!("{INDEX_HEADING}\n{}", rows.join("\n"))
}

/// One document's row: the document named against the root ([`relative_to`]),
/// and then the one decision over whether it is the document this run writes —
/// it is, and the row says so ([`THIS_RUNS_DOCUMENT`]), since a model asked to
/// consult what it is about to replace would be reading its own draft; it is
/// not, and the row is that name alone, which is the whole of what the index
/// carries about a document. The comparison is between the paths as the walk
/// answered with them, both sides being in that form, so how a row is *named*
/// cannot change which row is marked.
#[implements(
    spec::ThisRunsOwnDocumentIsMarkedInTheIndex,
    spec::TheDocumentsTheIndexNamesAreNamedAndNotCarried,
)]
pub fn index_row(root: &Path, document: &Path, own: &Path) -> String {
    todo!("the document named against the root, marked when it is this run's own")
}

/// A document as a row names it: its path relative to the workspace root, which
/// is the form `read` and `grep` take — [`confine`](crate::headless_canopy_agent::tools::confine)
/// refuses an absolute path as written, before it resolves anything, so a row in
/// any other form is a path the one tool it feeds will not accept.
///
/// A document that is not under the root is named as it stands. No such path can
/// come out of [`intent_paths`], every one of them being built under the root
/// this is handed; if one ever arrives, the row carries what the walk found and
/// the model can see it, which a dropped row or an empty name would not allow.
#[implements(spec::EveryIndexRowNamesItsDocumentRelativeToTheWorkspaceRoot)]
pub fn relative_to(root: &Path, document: &Path) -> String {
    todo!("the document relative to the root, or as it stands when it is not under it")
}

/// The heading the sole HLD is carried under.
pub const HLD_HEADING: &str = "## The HLD every slice sits inside";

/// The HLD the opening carries — the one decision over how many the index
/// found: exactly one, and it is carried whole, being the design every slice
/// sits inside; none or several, and none is carried, which of several governs
/// this slice being a design question with a human's answer rather than one a
/// coach may make silently. The count is the whole of the decision, so it is
/// made where the paths are.
#[implements(
    spec::TheOpeningCarriesTheSoleHldWhole,
    spec::AnIndexWithoutExactlyOneHldCarriesNoHld,
)]
pub fn hld_section(paths: &[PathBuf]) -> String {
    let found = hlds(paths);
    match found.as_slice() {
        [only] => hld_carried(only),
        [] | [_, _, ..] => String::new(),
    }
}

/// Those of the index's paths that are an HLD — the documents filed under
/// [`HLD_FILE`], whichever `docs/intent` they sit in — which is the set
/// [`hld_section`] counts. Read out of the index rather than walked again, so
/// that the HLD the opening carries is one the index named.
pub fn hlds(paths: &[PathBuf]) -> Vec<&Path> {
    todo!("those of the index's paths that are an HLD")
}

/// The HLD as the opening carries it: where it was read from, and its text
/// whole. A document that has gone between the walk that found it and this
/// carries its heading and nothing under it — the index still names it, and a
/// `read` can be aimed at it, which is what the opening is for.
#[implements(spec::TheOpeningCarriesTheSoleHldWhole)]
pub fn hld_carried(hld: &Path) -> String {
    todo!("the HLD's path and its text whole, under the heading")
}

/// The files the project's guidance is looked for in, in the order it prefers
/// them: `AGENTS.md`, which `init` writes, and then `CLAUDE.md`, for a project
/// that arrived at the methodology by another road. The order is the whole of
/// the "else": the first of them that reads is the one carried, so no branch
/// decides between two files a list already ranks.
pub const GUIDANCE_FILES: [&str; 2] = ["AGENTS.md", "CLAUDE.md"];

/// The heading the project's guidance is carried under, naming the file it came
/// from, since which of the two a project keeps is a fact about it.
pub const GUIDANCE_HEADING: &str = "## The project's guidance";

/// The project's guidance, whole: `AGENTS.md` at the workspace root, else its
/// `CLAUDE.md` — for a project that arrived at the methodology by another road —
/// else nothing. The two are [`GUIDANCE_FILES`] in that order and each is read
/// from under the workspace root ([`synced_text`], which is that read and
/// nothing else); a project with neither, or one whose root cannot be located,
/// carries no guidance rather than a sentence about not having any, the opening
/// being what the repository holds and not a report on it.
#[implements(
    spec::TheProjectsGuidanceIsTheWorkspacesAgentsFileWhole,
    spec::ClaudeMdIsTheGuidanceWhenThereIsNoAgentsFile,
    spec::NeitherGuidanceFileCarriesNoGuidance,
)]
pub fn guidance_section(project: &Project) -> String {
    todo!("the first of the guidance files that reads, whole; else nothing")
}

/// The guidance as the opening carries it: the file it came from, and its text
/// whole — [`hld_carried`]'s shape for the other document the opening carries.
#[implements(spec::TheProjectsGuidanceIsTheWorkspacesAgentsFileWhole)]
pub fn guidance_carried(name: &str, guidance: &str) -> String {
    todo!("the file it came from and its text, under the heading")
}

/// The document already at that path, whole; none when there is none to read.
/// A path that cannot be read is a slice with no document yet, which is the
/// ordinary case rather than a fault: the coach is about to write one.
pub fn existing(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// The opening for a slice with no document: what the repository holds, and
/// then what the slice is and that the model is writing its LLD — the
/// repository first, because it is what makes the questions aimable.
#[implements(spec::TheOpeningNamesTheSlice, spec::TheOpeningLeadsWithWhatTheRepositoryHolds)]
pub fn writing(repository: &str, slice: &str) -> String {
    format!(
        "{repository}\n\nYou are writing the LLD for the slice `{slice}`, which has no document yet. Read what the \
         repository already answers, then interview me one decision at a time, and draft when you can write a section \
         without inventing anything."
    )
}

/// The opening for a slice whose document already exists: what the repository
/// holds, then what the slice is, that document whole, and that the model is
/// amending it rather than writing one — which is how a Phase 8 amendment is
/// drafted, the commit still being the human's.
#[implements(
    spec::TheOpeningNamesTheSlice,
    spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment,
    spec::TheOpeningLeadsWithWhatTheRepositoryHolds,
)]
pub fn amending(repository: &str, slice: &str, document: &str) -> String {
    format!(
        "{repository}\n\nYou are amending the LLD for the slice `{slice}`, which already exists. Read what the \
         repository already answers, then interview me one decision at a time, and draft the whole document when you \
         can write the amendment without inventing anything. The document as it stands follows.\n\n{document}"
    )
}

/// The loop, opened on the message it is handed: `opening` is landed as the
/// conversation's first user message and a turn is settled on it before the
/// human is read for a line, so the coach's first question is that turn's
/// settled answer — the model speaks first, this being an interview — and a run
/// whose input is already at end of file still opens, settles one turn and ends,
/// rather than returning before a turn exists. Thereafter each message is landed
/// ([`next_message`]), a model turn is driven with this module's own executor
/// ([`executed`]) and this module's own narrator ([`narrate`]) — an interview
/// being watched, so a turn that spends minutes on tool calls shows its work
/// rather than going silent — the model's settled answer is printed, and what
/// follows that turn is decided
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
    spec::TheCoachingSessionsNarratorPrintsTheModelsText,
    spec::TheConversationEndsAtDoneOrEndOfFile,
    spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred,
    spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading,
    spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence,
    spec::TheDraftedDocumentSurvivesAHalt,
    spec::ARunThatDraftedNothingEndsSayingSo,
)]
pub fn converse(project: &Project, coach: &mut Coach, opening: &str) -> Result<Option<bool>, Halt> {
    let path = coach.path.clone();
    let mut verdict = None;
    let mut message = Some(Landed { text: opening.to_string(), answering: Answering::TheHuman, holds: None });
    while let Some(landed) = message {
        verdict = landed.holds.or(verdict);
        let mut noted = Noted::default();
        let settled = {
            let mut executor = |running: &Project, _: &Session, op: &str, args: &Value| executed(running, &path, &mut noted, op, args);
            drive(project, &mut coach.session, &mut executor, &mut narrate, &landed.text)?
        };
        println!("{}", settled.text);
        message = next_message(project, coach, next_after(noted, landed.answering));
    }
    Ok(verdict)
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
    match (noted.ended, noted.wrote, answering) {
        (true, _, Answering::TheHuman | Answering::TheJudges) => Next::Nothing,
        (false, true, Answering::TheHuman) => Next::Judges,
        (false, true, Answering::TheJudges) => Next::Human,
        (false, false, Answering::TheHuman | Answering::TheJudges) => Next::Human,
    }
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
/// the human is read for a line ([`read_line`]) and what that line means is
/// decided over it ([`human_turn`]), the conversation ending when they end it;
/// the judges are run and their findings landed ([`judges_turn`]) without the
/// human asking; nobody, and the loop ends.
#[implements(
    spec::TheJudgesAnswerATurnThatDrafted,
    spec::TheConversationEndsAtDoneOrEndOfFile,
    spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles,
)]
pub fn next_message(project: &Project, coach: &Coach, next: Next) -> Option<Landed> {
    match next {
        Next::Human => human_turn(read_line()).map(|text| Landed { text, answering: Answering::TheHuman, holds: None }),
        Next::Judges => Some(judges_turn(project, coach)),
        Next::Nothing => None,
    }
}

/// The judges' turn as the loop lands it: one judging ([`judged`]) over the
/// document as it now stands, its message landed as the next user message, and
/// its verdict carried beside it for [`converse`] to keep.
#[implements(spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading)]
pub fn judges_turn(project: &Project, coach: &Coach) -> Landed {
    let judging = judged(project, coach);
    Landed { text: judging.message, answering: Answering::TheJudges, holds: Some(judging.holds) }
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
    let line = read.map(|text| text.trim_end_matches(['\n', '\r']).to_string());
    match line.as_deref() {
        None | Some(DONE) => Typed::Ended,
        Some(answer) => Typed::Answer(answer.to_string()),
    }
}

/// One line from the human's terminal, from a blocking read; none at end of
/// file. The read itself and nothing else: what the line means is [`typed`]'s,
/// and what the loop does with it is [`human_turn`]'s.
///
/// A blocking read of a terminal has no state a test can put it in and still
/// finish, so nothing in these three lines is measurable. Splitting the meaning
/// out of the read is what leaves this holding no branch of the slice's own —
/// the pure I/O sequencing `docs/intent/coach/lld.md` § Decisions &
/// Alternatives exempts from measurement.
pub fn read_line() -> Option<String> {
    let mut line = String::new();
    let read = std::io::stdin().read_line(&mut line).ok()?;
    (read > 0).then_some(line)
}

/// What the human typed at the loop's prompt, or none when they ended the
/// conversation — `done` alone on a line, or end of file, as [`typed`]
/// classifies them. It is handed the line rather than reading it, so that this
/// decision is a function over a value a test supplies and [`read_line`] is the
/// only thing in the pair that no test can reach.
#[implements(spec::TheConversationEndsAtDoneOrEndOfFile)]
pub fn human_turn(line: Option<String>) -> Option<String> {
    match typed(line) {
        Typed::Answer(answer) => Some(answer),
        Typed::Ended => None,
    }
}

/// The coach's four tools: a set of its own, not the canopy client's five,
/// whose `edit` and `write` a coach that writes one document must not hold.
/// [`COACH_TOOLS`] is the set itself, and both what the session declares
/// ([`declarations`]) and what its dispatch admits ([`declared`]) are derived
/// from it, so no second list of names can drift from this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools)]
pub enum Tool {
    /// A file's text or a directory's entries, through the canopy client's own
    /// read over its own confinement.
    Read,
    /// A literal, case-sensitive substring search, through that client's own
    /// grep under the same confinement.
    Grep,
    /// The slice's LLD, replaced whole.
    Draft,
    /// One question put to the human, answered with what they typed.
    Ask,
}

/// The tools a coaching session declares: exactly these four, in the order
/// they are declared. The closed set both [`declarations`] and [`declared`]
/// are built from.
pub const COACH_TOOLS: [Tool; 4] = [Tool::Read, Tool::Grep, Tool::Draft, Tool::Ask];

impl Tool {
    /// The tool's `op` on the wire — `read`, `grep`, `draft`, `ask` — which is
    /// also the name the model calls it by and the name [`declared`] matches a
    /// forwarded `op` against.
    #[implements(spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools)]
    pub fn op(self) -> &'static str {
        match self {
            Tool::Read => "read",
            Tool::Grep => "grep",
            Tool::Draft => "draft",
            Tool::Ask => "ask",
        }
    }

    /// The JSON schema of the tool's arguments, its description in the
    /// schema's `description`. `read` and `grep` take the canopy client's own
    /// [`schema_of`](crate::headless_canopy_agent::tools::schema_of) for that
    /// tool, since they are that client's tools whole and two descriptions of
    /// one tool are two things nothing keeps in step. `draft` and `ask` are the
    /// coach's own and have no other home: `content` alone, since the one path
    /// `draft` can write is the slice's LLD and no argument of its can name
    /// another; `question` with optional `options`.
    #[implements(
        spec::ReadAndGrepAreDeclaredWithTheCanopyClientsSchemas,
        spec::DraftWritesTheSlicesLldAndNoOtherPath,
    )]
    pub fn schema(self) -> Value {
        let string = json!({ "type": "string" });
        match self {
            Tool::Read => schema_of(CanopyTool::Read),
            Tool::Grep => schema_of(CanopyTool::Grep),
            Tool::Draft => json!({ "type": "object", "description": "Replace the slice's LLD whole with `content`, creating its directory. The path is this run's own: no argument of yours names a file.", "properties": { "content": string }, "required": ["content"] }),
            Tool::Ask => json!({ "type": "object", "description": "Put one question to the human and answer with what they typed. With `options` they are printed numbered, and a bare number answers with that option; anything else is answered as typed.", "properties": { "question": string.clone(), "options": json!({ "type": "array", "items": string }) }, "required": ["question"] }),
        }
    }
}

/// The coaching session's declarations: one [`declaration`] for each of
/// [`COACH_TOOLS`], and so exactly `read`, `grep`, `draft` and `ask` — a set of
/// the coach's own rather than the canopy client's five.
#[implements(spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools)]
pub fn declarations() -> Vec<ToolDecl> {
    COACH_TOOLS.map(declaration).to_vec()
}

/// One tool as the policy declares it: its name — which the model sees and
/// calls it by, and which here is the tool's own `op` — the requestee this
/// program answers as, that `op`, and the schema of its arguments
/// ([`Tool::schema`]).
#[implements(spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools)]
pub fn declaration(tool: Tool) -> ToolDecl {
    ToolDecl { name: tool.op().to_string(), requestee: REQUESTEE.to_string(), op: tool.op().to_string(), schema: tool.schema() }
}

/// The tool a forwarded `op` names, provided [`COACH_TOOLS`] carries one whose
/// [`Tool::op`] it is; an `op` outside that set — the canopy client's `edit`,
/// or anything else — is the refusal naming it, and nothing runs. A tool set
/// is what a session may call, and a call outside it is a fault to report
/// rather than a request to interpret.
#[implements(
    spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools,
)]
pub fn declared(op: &str) -> Result<Tool, String> {
    COACH_TOOLS
        .into_iter()
        .find(|tool| tool.op() == op)
        .ok_or_else(|| format!("`{op}` is not a tool this session declared: the coach declares `read`, `grep`, `draft` and `ask`"))
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

/// What [`drive`](crate::headless_canopy_agent::turn::drive) is handed for one
/// coaching turn: the call announced ([`announce`]), then routed
/// ([`execute`]). Two statements rather than one, so that neither the
/// announcing nor the dispatch is the other's condition — a call whose line
/// says nothing is still routed, and a call that fails was still announced.
#[implements(spec::EveryToolCallIsAnnouncedBeforeItIsRouted)]
pub fn executed(project: &Project, path: &Path, noted: &mut Noted, op: &str, args: &Value) -> ToolResult {
    announce(path, op, args);
    execute(project, path, noted, op, args)
}

/// The line a call is announced with, printed when there is one
/// ([`announced`]) — a human watching an interview that has gone silent cannot
/// tell reading from a hang, and a call whose arguments do not carry what its
/// line would name is about to fail and say so in its own sentence. This is
/// where announcing nothing is *nothing*: `announced` gives no line for `ask`
/// or for a call missing its subject, and no line is what is printed for it.
#[implements(
    spec::AskAnnouncesNothing,
    spec::ACallWhoseArgumentsLackItsSubjectAnnouncesNothing,
)]
pub fn announce(path: &Path, op: &str, args: &Value) {
    if let Some(line) = announced(path, op, args) {
        println!("{line}");
    }
}

/// What one call is announced with, or nothing: a `read`'s path, a `grep`'s or
/// a `glob`'s pattern ([`argument_named`]), the document a `draft` is about to
/// replace — which is the coach's own path rather than an argument of the call
/// — and nothing for `ask`, whose printed question is its own announcement, or
/// for a call whose arguments do not carry the subject its line would name.
///
/// It decides which argument a call's line is named by and nothing further: the
/// line itself is [`announcing`]'s, so that each arm here names a subject rather
/// than composing a sentence, and the rule that a missing subject is no line at
/// all is stated once instead of four times.
///
/// It is written over the `op` as a string rather than over [`Tool`], because a
/// reader session's calls are announced by it too and that session calls
/// `glob`, which this host does not declare.
#[implements(
    spec::AReadIsAnnouncedByThePathItNames,
    spec::AGrepOrGlobIsAnnouncedByThePatternItNames,
    spec::ADraftIsAnnouncedByTheDocumentItReplaces,
    spec::AskAnnouncesNothing,
    spec::ACallWhoseArgumentsLackItsSubjectAnnouncesNothing,
)]
pub fn announced(path: &Path, op: &str, args: &Value) -> Option<String> {
    todo!("the line this call is announced with, or none")
}

/// The line a call is announced with once its subject is known: the `op` and
/// that subject, which is what a human watching needs to tell one call from the
/// next. A call whose arguments did not carry a subject has no line — the whole
/// of that rule, kept here rather than in each of [`announced`]'s arms, since
/// what is missing is the same thing however it was named.
#[implements(spec::ACallWhoseArgumentsLackItsSubjectAnnouncesNothing)]
pub fn announcing(op: &str, subject: Option<String>) -> Option<String> {
    todo!("the op and its subject, or no line at all")
}

/// One string argument out of a call's JSON, which is where every
/// announcement's subject comes from; none when the arguments do not carry it,
/// or carry something that is not a string — which is the whole of how a call
/// whose arguments lack its subject comes to be announced with nothing.
#[implements(spec::ACallWhoseArgumentsLackItsSubjectAnnouncesNothing)]
pub fn argument_named(args: &Value, key: &str) -> Option<String> {
    todo!("one string argument of the call")
}

/// The coaching session's narrator: what the model said before calling a tool,
/// printed as it said it. That is where a model says what it is about to look
/// for, and it is worth more to the human than any summary the coach could
/// invent, so the line is the identity of the text `drive` hands over.
#[implements(spec::TheCoachingSessionsNarratorPrintsTheModelsText)]
pub fn narrate(text: &str) {
    todo!("print the model's own words")
}

/// The coach's tool dispatch, reached through [`executed`] — the one decision
/// over which of its four tools a forward names ([`declared`], which refuses an
/// `op` the coach did not declare): `read` goes to the canopy client's own read
/// over that client's own confinement ([`read_call`]) and `grep` to that
/// client's own grep under the same confinement ([`grep_call`]), so what a read
/// or a grep answers is asserted once, there; `draft` is given `path` — the one
/// document this run has, whatever a call's arguments carry, which is why no
/// call of it can reach the module, the claims or a manifest ([`draft_call`]);
/// `ask` is given the same record the turn keeps, an ending typed there having
/// nowhere else to go ([`ask_call`]). It takes the project because its `read`
/// and `grep` routes need the workspace root a confinement is made against.
#[implements(
    spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName,
    spec::AReadIsRoutedToTheCanopyClientsReadOverItsConfinement,
    spec::AGrepIsRoutedToTheCanopyClientsGrepOverItsConfinement,
    spec::DraftWritesTheSlicesLldAndNoOtherPath,
)]
pub fn execute(project: &Project, path: &Path, noted: &mut Noted, op: &str, args: &Value) -> ToolResult {
    match declared(op)? {
        Tool::Read => read_call(project, args),
        Tool::Grep => grep_call(project, args),
        Tool::Draft => draft_call(path, noted, args),
        Tool::Ask => ask_call(noted, args),
    }
}

/// `grep` as the coach routes it: the canopy client's own
/// [`GrepArgs`](crate::headless_canopy_agent::tools::GrepArgs), the directory it
/// searches through that client's
/// [`confine`](crate::headless_canopy_agent::tools::confine) against the
/// workspace root — the same confinement [`read_call`] is bounded by — and that
/// client's [`grep_tool`](crate::headless_canopy_agent::tools::grep_tool) over
/// it, so a search reaches the coach exactly as it reaches a phase worker. No
/// verdict is asked: a coaching session carries no phase.
#[implements(spec::AGrepIsRoutedToTheCanopyClientsGrepOverItsConfinement)]
pub fn grep_call(project: &Project, args: &Value) -> ToolResult {
    todo!("the canopy client's grep, confined to the workspace")
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
    let args: ReadArgs = arguments(args)?;
    let path = confine(&project.root()?, Path::new(&args.path))?;
    read_tool(&path, &args)
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
    let args: DraftArgs = arguments(args)?;
    noted.drafted = true;
    let bytes = draft(path, &args.content)?;
    noted.wrote = true;
    Ok(wrote_line(path, bytes))
}

/// What a successful `draft` answers the model with: the path it wrote and the
/// number of bytes, and nothing about whether the document holds — the checks
/// belong to the judges' turn.
#[implements(spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict)]
pub fn wrote_line(path: &Path, bytes: usize) -> String {
    format!("wrote {bytes} bytes to {}", path.display())
}

/// `ask` as the coach routes it: its [`AskArgs`], and the question put to the
/// human ([`ask`]) with the same record the turn keeps.
pub fn ask_call(noted: &mut Noted, args: &Value) -> ToolResult {
    let args: AskArgs = arguments(args)?;
    ask(&args.question, &args.options, noted)
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
    let directory = path.parent().ok_or_else(|| format!("`{}` names no directory to write the document in", path.display()))?;
    std::fs::create_dir_all(directory).map_err(|e| format!("creating {}: {e}", directory.display()))?;
    std::fs::write(path, content).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok(content.len())
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
    let offered: String = options.iter().enumerate().map(|(at, option)| format!("\n{}. {option}", at + 1)).collect();
    format!("{question}\n({STALL_WINDOW}){offered}")
}

/// The option a bare number names — the number read as a place in the list,
/// counted from one as the question printed it; none when what was typed is
/// not a number, or numbers no option there is.
pub fn option_named<'a>(options: &'a [String], answer: &str) -> Option<&'a String> {
    let place = answer.trim().parse::<usize>().ok()?;
    options.get(place.checked_sub(1)?)
}

/// What `ask` answers the model with — the one decision over what the human
/// typed: a bare number naming one of the options answers with that option;
/// anything else is answered verbatim. The options are an offer, never a
/// constraint, because a closed question the human wants to answer differently
/// is a question that was wrong, and refusing what is not on the list would
/// make the tool decide which answers a design may have.
#[implements(spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped)]
pub fn answered(options: &[String], answer: &str) -> String {
    option_named(options, answer).cloned().unwrap_or_else(|| answer.to_string())
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
    println!("{}", asked(question, options));
    replied(options, &typed(read_line()), noted)
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
    match line {
        Typed::Answer(answer) => Ok(answered(options, answer)),
        Typed::Ended => ended_in_ask(noted),
    }
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
    noted.ended = true;
    Err(CONVERSATION_OVER.to_string())
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
    let checks = document_failures(&coach.path);
    println!("{JUDGING_HEADING}");
    let findings = reader_findings(project, coach);
    println!("{}", judging_line(&checks, &findings));
    Judging { holds: checks_hold(&checks), message: judges_message(&checks, &findings) }
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
    let document = document(path)?;
    Ok([decisions_exist(&document), alternatives(&document), shape_rows(&document), deferred_numbered(&document)].concat())
}

/// The document as the four checks read it: the path `draft` wrote, its lines,
/// and the slice its parent directory names ([`slice_named`]). Built here
/// rather than through [`Lld::read`](crate::lld_review::Lld::read), which
/// would resolve a path again and could find another. A path that cannot be
/// read back is the read's own sentence, naming the path it looked for — the
/// file `draft` reported writing can go away between that report and this
/// read, and this is where a judging learns it did.
pub fn document(path: &Path) -> Result<Lld, String> {
    let text = std::fs::read_to_string(path).map_err(|unreadable| format!("reading {}: {unreadable}", path.display()))?;
    Ok(Lld { slice: slice_named(path), path: path.to_path_buf(), lines: text.lines().map(str::to_string).collect() })
}

/// The document checks as the judges' message carries them — the one decision
/// over whether they ran at all: they did, and it is what they found
/// ([`failures_section`]); the document could not be read back, and the read's
/// own sentence stands in place of their failures, under a line saying no check
/// ran ([`DOCUMENT_UNREADABLE`]), so that a model reading the judging is not
/// told a document it can no longer see either holds or fails.
#[implements(spec::AnUnreadableDocumentsSentenceIsLandedInPlaceOfTheChecksFailures)]
pub fn checks_section(checks: &Result<Vec<Failure>, String>) -> String {
    match checks {
        Ok(failures) => failures_section(failures),
        Err(unreadable) => format!("{DOCUMENT_UNREADABLE}: {unreadable}"),
    }
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
    match failures {
        [] => EVERY_CHECK_HELD.to_string(),
        [_, ..] => rendered(failures),
    }
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
    checks.as_ref().is_ok_and(|failures| failures.is_empty())
}

/// The reader's part of the judges' message — the one decision over whether it
/// answered: its findings, landed whole rather than summarised, since which
/// findings matter is the human's judgment; or, when it could not be consulted,
/// that sentence in their place ([`READER_UNCONSULTED`]) with the reason
/// beside it, so the model knows a second opinion was meant to be here and is
/// not.
#[implements(spec::AFailedReadersSentenceIsLandedInPlaceOfItsFindings)]
pub fn reader_section(findings: &Result<Vec<String>, String>) -> String {
    match findings {
        Ok(found) => found.join("\n"),
        Err(lost) => format!("{READER_UNCONSULTED}: {lost}"),
    }
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
    format!(
        "{JUDGING_HEADING}\n\nThe four document checks over what you just wrote:\n\n{}\n\nThe reader:\n\n{}\n",
        checks_section(checks),
        reader_section(findings)
    )
}

/// What the human is told when a judging has run: how the document checks found
/// the document ([`checks_line`]), and what became of the reader
/// ([`reader_line`]), in that order — the order the message the model was given
/// carries them in. Both are decided over the judging's two answers rather than
/// here, so this joins two lines and chooses nothing.
pub fn judging_line(checks: &Result<Vec<Failure>, String>, findings: &Result<Vec<String>, String>) -> String {
    format!("{}\n{}", checks_line(checks), reader_line(findings))
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
    match checks {
        Ok(failures) if failures.is_empty() => "the four document checks hold over what was just written".to_string(),
        Ok(failures) => format!("{} of the four document checks failed over what was just written", failures.len()),
        Err(unreadable) => format!("{DOCUMENT_UNREADABLE}: {unreadable}"),
    }
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
    match findings {
        Ok(_) => String::new(),
        Err(lost) => format!("{READER_UNCONSULTED}: {lost}"),
    }
}

/// The reader's system prompt: the synced
/// [`lld_review::READER`](crate::lld_review::READER) — that slice's own name
/// for the path, not a second copy of it — without its frontmatter, as the
/// canopy client reads an agent's body.
#[implements(spec::TheReadersSystemIsTheSyncedReaderBody)]
pub fn reader_body(project: &Project) -> Result<String, String> {
    Ok(without_frontmatter(&synced_text(project, READER)?))
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
    let observation = canopy_declarations(&[CanopyTool::Read, CanopyTool::Grep, CanopyTool::Glob]);
    Ok(Settings { system: reader_body(project)?, policy: policy_for(&observation), params: json!({}), max_cost })
}

/// The reader's user message: the document's path, and the findings asked for
/// in the form the reader's own definition asks for them, so that what comes
/// back is what a phase would have been given.
#[implements(spec::TheReaderIsGivenTheDocumentAndAskedForFindings)]
pub fn reader_prompt(path: &Path) -> String {
    format!(
        "The slice's LLD is at `{}`. Read it, and the guideline, and answer with your findings as your definition asks \
         for them: one numbered finding to a line, ordered by what they would cost, and say plainly if you find none.",
        path.display()
    )
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
///
/// Its calls are announced ([`announce`]) as the coaching session's are, the
/// judging being the other place a run goes quiet for minutes; the announcement
/// is written over the `op` as a string precisely so that it can carry this
/// session's `glob`, which the coach does not declare. Its prose is not
/// narrated — it is driven with the canopy client's
/// [`silent`](crate::headless_canopy_agent::turn::silent) — because what the
/// reader has to say is landed whole a moment later, and the human should read
/// the findings rather than a draft of them.
#[implements(
    spec::TheReaderIsAFreshSessionForEveryJudging,
    spec::AReaderSessionDeclaresTheCanopyClientsObservationTools,
    spec::AReaderSessionCarriesNoPhase,
    spec::AReaderTurnIsRunByTheCanopyClientsOwnDispatch,
    spec::AReaderSessionsToolCallsAreAnnouncedToo,
    spec::AReaderSessionNarratesNothing,
    spec::TheReaderIsGivenTheDocumentAndAskedForFindings,
    spec::AReaderSessionIsStoppedWhenItAnswers,
)]
pub fn reader_findings(project: &Project, coach: &Coach) -> Result<Vec<String>, String> {
    let settings = reader_settings(project, coach.max_cost)?;
    let mut session = Session::open(&coach.door, &settings, None, settings.policy.tools.clone())?;
    let mut executor = |running: &Project, reading: &Session, op: &str, args: &Value| {
        announce(&coach.path, op, args);
        canopy_execute(running, reading, op, args)
    };
    let read = drive(project, &mut session, &mut executor, &mut silent(), &reader_prompt(&coach.path)).map_err(halt_reason);
    let sealed = session.stop().map_err(halt_reason);
    read.and_then(|settled| sealed.map(|()| numbered(&settled.text)))
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
    let artifacts = [guideline_names_every_check(project)?, reader_observes_only(project)?].concat();
    Ok(artifacts.iter().map(warning).collect())
}

/// One artifact check's failure as the human is told it: the file, the line,
/// and what it found — which check the checklist omits, or what the reader
/// declares — said as a warning the interview goes on past.
pub fn warning(failure: &Failure) -> String {
    format!("warning: {}:{}: {:?}: {}", failure.path.display(), failure.line, failure.check, failure.message)
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
    match holds {
        None => nothing_drafted(path),
        Some(true) => document_owed(path, CHECKS_HOLD),
        Some(false) => document_owed(path, CHECKS_DO_NOT_HOLD),
    }
}

/// The ending for a conversation in which nothing was ever drafted: there is
/// no document at that path, nothing to check, and nothing to commit.
#[implements(spec::ARunThatDraftedNothingEndsSayingSo)]
pub fn nothing_drafted(path: &Path) -> String {
    format!("nothing was drafted: there is no document at {}, nothing to check, and nothing for you to commit", path.display())
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
    format!(
        "the document is at {}, and {verdict}. Read it once more, and commit it as `phase 1: LLD for {}`, which the \
         coach does not do.",
        path.display(),
        slice_named(path)
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use lid_rs::validates;
    use serde_json::json;

    use super::*;
    use crate::headless_canopy_agent::PRODUCTION_DOOR;
    use crate::headless_canopy_agent::door::policy_for;
    use crate::headless_canopy_agent::ending::without_frontmatter;
    use crate::headless_canopy_agent::replay::{self, Replay, Route, SessionScript, mentions, strings};
    use crate::headless_canopy_agent::tools::{REQUESTEE, ReadArgs, Tool as CanopyTool, declarations as canopy_declarations, read_tool};
    use crate::headless_canopy_agent::turn::{QUIET_TAIL, payload_digest};
    use crate::lld_review::{Check, GUIDELINE, READER, check_all, rendered};
    use crate::phase::{fixture, tally};

    /// The budget every session these tests dial is dialled with.
    const MAX_COST: f64 = 2.5;

    /// The conversation's first user message, as the loop is handed it — a
    /// literal, so that what the coaching session's log carries first can be
    /// compared with what [`converse`] was given.
    const OPENING: &str = "You are writing the LLD for the slice `hello`.";

    /// What the model settles the drafting turn on.
    const FIRST_ANSWER: &str = "I have written a first draft; read the judges' findings.";

    /// The sentence the platform halts the driven conversation's second turn
    /// with — the turn answering the judges, which the human would otherwise
    /// be read for.
    const HALTED: &str = "max_cost reached";

    /// The reader's answer: one numbered finding.
    const READER_ANSWER: &str = "1. The Shape table names no failure path.\n";

    /// That finding without its number, as [`reader_findings`] yields it.
    const FINDING: &str = "The Shape table names no failure path.";

    /// A document that holds all four of `lld-check`'s document checks.
    const HOLDS: &str = "\
# hello — a slice

## Shape

| Item | Role |
|---|---|
| `run(args)` | the entry |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| what | this | that | because |

### Deferred
1. Something later.
";

    /// A document that fails three of them at once: no decisions table, a
    /// shape row naming no identifier, and an unnumbered deferral.
    const FAILS: &str = "\
# hello — a slice

## Shape

| Item | Role |
|---|---|
| no identifier | a note |

### Deferred
- an unnumbered deferral
";

    /// This workspace's root: the parent of this crate's manifest directory.
    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("the crate directory has a parent").to_path_buf()
    }

    /// Writes a file, creating the directories above it.
    fn write_at(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().expect("the path has a parent")).expect("create the directories");
        std::fs::write(path, content).expect("write the file");
    }

    /// A `cargo metadata` document for a workspace at `root` whose members are
    /// those `(name, directory)` pairs, each directory relative to it.
    fn metadata(root: &Path, members: &[(&str, &str)]) -> String {
        let packages: Vec<String> = members
            .iter()
            .map(|(name, dir)| {
                let manifest = root.join(dir).join("Cargo.toml");
                format!(r#"{{"name":"{name}","manifest_path":"{}","targets":[{{"kind":["lib"],"name":"{name}"}}]}}"#, manifest.display())
            })
            .collect();
        format!(r#"{{"workspace_root":"{}","target_directory":"{}","packages":[{}]}}"#, root.display(), root.join("target").display(), packages.join(","))
    }

    /// A project rooted at `root` with those members, without asking cargo.
    fn project_at(root: &Path, members: &[(&str, &str)]) -> Project {
        Project::from_json(&metadata(root, members)).expect("the metadata document parses")
    }

    /// A scratch workspace carrying this project's own synced artifacts — the
    /// interview method, the guideline and the reader — so that a scratch run
    /// reads exactly what this workspace ships, with one member at its root.
    fn scratch_project(name: &str) -> (PathBuf, Project) {
        let root = fixture::scratch(name);
        for relative in [METHOD, GUIDELINE, READER] {
            let text = std::fs::read_to_string(workspace_root().join(relative)).expect("this workspace's synced copy");
            write_at(&root.join(relative), &text);
        }
        let project = project_at(&root, &[("app", "")]);
        (root, project)
    }

    /// One workspace member, named, at its own directory under `root`.
    fn member(root: &Path, name: &str) -> Member {
        Member { name: name.to_string(), dir: root.join(name) }
    }

    /// One failure of a document check, as `lld-check` reports one.
    fn failure_at(check: Check, path: &Path, line: usize) -> Failure {
        Failure { check, path: path.to_path_buf(), line, message: "the rule it states".to_string() }
    }

    /// A coach over `replay`'s door whose document is `path`; its coaching
    /// session is a placeholder that never dials, a judging touching none.
    fn judging_coach(replay: &Replay, path: &Path) -> Coach {
        let door = replay.door("k");
        let session = replay::session(door.clone(), "s-coach", None, vec![]);
        Coach { door, session, path: path.to_path_buf(), max_cost: MAX_COST }
    }

    /// A reader session that answers with one numbered finding.
    fn answering_reader(id: &str) -> SessionScript {
        SessionScript::new(id).page(replay::settling_page(READER_ANSWER))
    }

    /// A reader session whose dial the door refuses.
    fn refused_reader(id: &str, sentence: &str) -> SessionScript {
        SessionScript::new(id).refusing(Route::Start, 403, sentence)
    }

    /// A reader session that reads the document through its own dispatch,
    /// tries the coach's `draft`, and then answers.
    fn observing_reader(id: &str, relative: &str) -> SessionScript {
        let (read, write) = (json!({ "path": relative }), json!({ "content": "the reader cannot draft" }));
        SessionScript::new(id)
            .page(replay::tool_call_page("read", read.clone(), &payload_digest(REQUESTEE, &read)))
            .page(replay::tool_call_page("draft", write.clone(), &payload_digest(REQUESTEE, &write)))
            .page(replay::settling_page(READER_ANSWER))
    }

    /// The driven conversation's coaching session: a first turn that calls
    /// `draft` and settles, and a second — the one answering the judges — that
    /// the platform halts. Every turn is followed by the judges or by a halt,
    /// so the loop runs from its opening to its end with nothing typed.
    fn drafting_then_halted(id: &str, content: &str) -> SessionScript {
        let args = json!({ "content": content });
        SessionScript::new(id)
            .page(replay::tool_call_page("draft", args.clone(), &payload_digest(REQUESTEE, &args)))
            .page(replay::settling_page(FIRST_ANSWER))
            .page(vec![replay::halted(HALTED)])
    }

    /// One conversation driven with nothing on the terminal.
    struct Driven {
        /// The door it was driven against.
        replay: Replay,
        /// The document `draft` wrote.
        path: PathBuf,
        /// What the loop answered.
        ended: Result<Option<bool>, Halt>,
    }

    /// Drives that conversation: a scratch workspace, a replay serving the
    /// coaching session and the judging's one reader, a coach opened on it,
    /// and [`converse`] run on the opening it is handed. Nothing is read from
    /// the terminal: the first turn drafts, the judges answer it, and the turn
    /// that answers them is halted.
    fn driven(name: &str) -> Driven {
        let (root, project) = scratch_project(name);
        let replay = Replay::serve(vec![drafting_then_halted("s-coach", FAILS), answering_reader("s-reader")]);
        let path = root.join("docs/intent/hello/lld.md");
        let door = replay.door("k");
        let settings = coaching_settings(&project, MAX_COST).expect("the coaching dial");
        let session = Session::open(&door, &settings, None, declarations()).expect("the coaching session");
        let mut coach = Coach { door, session, path: path.clone(), max_cost: MAX_COST };
        let ended = converse(&project, &mut coach, OPENING);
        Driven { replay, path, ended }
    }

    // ---- the subcommand and its flags ------------------------------------------

    #[test]
    #[validates(spec::TheCoachsSliceIsTheFlagsValueOrTheBranchName)]
    fn the_coachs_slice_is_the_flags_value_or_the_branch_name() {
        let root = fixture::scratch("coach-slice");
        fixture::git(&root, &["init", "-q", "-b", "lld/hello"]);
        let project = project_at(&root, &[("app", "")]);
        let flagged = parse_args(&strings(&["--slice", "login"])).expect("the flag").slice;
        let absent = parse_args(&[]).expect("no flag").slice;
        assert_eq!((flagged, absent), (Some("login".to_string()), None), "absent the flag, only a project can settle it");
        // The flag's value survives a fold that applies three others beside it,
        // which is only visible if what each of those settled is named too: a
        // fold arm that dropped a field would leave the slice standing and the
        // flag beside it silently defaulted.
        let every = parse_args(&strings(&["--slice", "login", "--package", "app", "--door", "http://127.0.0.1:1", "--max-cost", "0.25"]));
        let beside_workspace = parse_args(&strings(&["--workspace", "--slice", "login"]));
        let standing = Flags {
            target: Some(Where::Package("app".to_string())),
            slice: Some("login".to_string()),
            door: "http://127.0.0.1:1".to_string(),
            max_cost: 0.25,
        };
        let alongside = Flags { target: Some(Where::Workspace), slice: Some("login".to_string()), ..Flags::default() };
        let folded = (every.expect("every flag"), beside_workspace.expect("a slice beside the workspace"));
        assert_eq!(folded, (standing, alongside), "each flag applied to what the ones before it settled, and not one of them dropped");
        let from_branch = slice_of(&project, None).expect("the branch names it");
        let given = slice_of(&project, Some("login".to_string())).expect("the flag wins over the branch");
        assert_eq!((from_branch.as_str(), given.as_str()), ("hello", "login"));
        fixture::git(&root, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        slice_of(&project, None).expect_err("a run on no `lld/<slice>` branch and no flag names no slice");
    }

    #[test]
    #[validates(spec::TheDoorDefaultsToCanopysProductionDoor)]
    fn the_door_defaults_to_canopys_production_door() {
        let defaulted = (Flags::default().door, parse_args(&[]).expect("no arguments").door);
        assert_eq!(defaulted, (PRODUCTION_DOOR.to_string(), PRODUCTION_DOOR.to_string()));
        assert_eq!(PRODUCTION_DOOR, "https://api.canopyhq.dev");
        assert_eq!(parse_args(&strings(&["--door", "http://127.0.0.1:1"])).expect("the flag").door, "http://127.0.0.1:1");
    }

    #[test]
    #[validates(spec::TheMaxCostDefaultsToFive)]
    fn the_max_cost_defaults_to_five() {
        let defaulted = (Flags::default().max_cost, parse_args(&[]).expect("no arguments").max_cost);
        assert_eq!(defaulted, (5.0, 5.0));
        let given = parse_args(&strings(&["--max-cost", "0.25"])).expect("the flag").max_cost;
        assert_eq!((given, amount("12").expect("an amount")), (0.25, 12.0));
        mentions(&amount("lots").expect_err("not a number"), &["lots"]);
    }

    #[test]
    #[validates(spec::AnyOtherArgumentToCoachIsRejectedByName)]
    fn any_other_argument_to_coach_is_rejected_by_name() {
        let named = ["--package", "--workspace", "--slice", "--door", "--max-cost", "--bogus"];
        let classified: Vec<Option<Flag>> = named.iter().map(|argument| Flag::of(argument)).collect();
        let closed = [Some(Flag::Package), Some(Flag::Workspace), Some(Flag::Slice), Some(Flag::Door), Some(Flag::MaxCost), None];
        assert_eq!(classified, closed, "the five flags, and none for anything else");
        mentions(&parse_args(&strings(&["--bogus"])).expect_err("rejected"), &["--bogus", COACH_USAGE]);
        mentions(&parse_args(&strings(&["coach"])).expect_err("a bare argument"), &["coach"]);
        mentions(&pairs(&strings(&["--bogus", "x"])).expect_err("no flag of that name"), &["--bogus"]);
        mentions(&parse_args(&strings(&["--slice", "hello", "--door"])).expect_err("a flag with no value"), &["--door"]);
        // Which arguments are read as names is what `--workspace` standing
        // alone decides: a flag that consumed the argument after it would take
        // the next flag for its value and reject that flag's value by name.
        let stands_alone = pairs(&strings(&["--workspace", "--slice", "hello"])).expect("`--workspace` consumes nothing after it");
        assert_eq!(stands_alone, vec![(Flag::Workspace, None), (Flag::Slice, Some("hello".to_string()))], "each argument after it still a name");
        mentions(&run(&strings(&["--bogus"])).expect_err("the entry rejects it before it reads the environment"), &["--bogus"]);
    }

    #[test]
    #[validates(spec::PackageAndWorkspaceRefuseEachOther)]
    fn package_and_workspace_refuse_each_other() {
        let both = strings(&["--package", "app", "--workspace"]);
        mentions(&one_target(&both).expect_err("a document has one place"), &["--package", "--workspace"]);
        mentions(&parse_args(&both).expect_err("refused before either is applied"), &["--package", "--workspace"]);
        one_target(&strings(&["--package", "app"])).expect("one flag names one place");
        one_target(&strings(&["--workspace"])).expect("one flag names one place");
        one_target(&[]).expect("neither names one, which a project settles");
    }

    #[test]
    #[validates(spec::NeitherFlagSettlesOnNoTarget)]
    fn neither_flag_settles_on_no_target() {
        let another = parse_args(&strings(&["--slice", "hello"])).expect("another flag").target;
        let settled = (Flags::default().target, parse_args(&[]).expect("no arguments").target, another);
        assert_eq!(settled, (None, None, None), "a flag parser is handed no project and counts no workspace's members");
        let workspace = parse_args(&strings(&["--workspace"])).expect("a target").target;
        let package = parse_args(&strings(&["--package", "app"])).expect("a target").target;
        assert_eq!((workspace, package), (Some(Where::Workspace), Some(Where::Package("app".to_string()))));
    }

    // ---- where the document goes -----------------------------------------------

    #[test]
    #[validates(spec::APackageNamingAMemberIsThatMembersManifestDirectory)]
    fn a_package_naming_a_member_is_that_members_manifest_directory() {
        let root = fixture::scratch("coach-named-member");
        let project = project_at(&root, &[("first", "first"), ("app", "app")]);
        assert_eq!(members(&project), vec![member(&root, "first"), member(&root, "app")]);
        assert_eq!(named_member(&members(&project), "app").expect("a member of that name"), root.join("app"));
        // `first` holds a document of this slice's name; the member named is
        // still `app`, which is the guess no later check would question.
        write_at(&root.join("first/docs/intent/hello/lld.md"), HOLDS);
        let named = package_dir(&project, Some(&Where::Package("app".to_string()))).expect("the member named");
        assert_eq!(named, root.join("app"), "the member of that name, not the first found to hold the slice's document");
    }

    #[test]
    #[validates(spec::APackageNamingNoMemberStopsTheRunListingTheMembers)]
    fn a_package_naming_no_member_stops_the_run_listing_the_members() {
        let root = fixture::scratch("coach-no-such-member");
        let project = project_at(&root, &[("first", "first"), ("app", "app")]);
        mentions(&named_member(&members(&project), "missing").expect_err("no member"), &["missing", "first", "app"]);
        let stop = package_dir(&project, Some(&Where::Package("missing".to_string()))).expect_err("no member");
        mentions(&stop, &["missing", "first", "app"]);
    }

    #[test]
    #[validates(spec::TheWorkspaceFlagNamesTheWorkspaceRoot)]
    fn the_workspace_flag_names_the_workspace_root() {
        let root = fixture::scratch("coach-workspace-root");
        let project = project_at(&root, &[("first", "first"), ("app", "app")]);
        let named = package_dir(&project, Some(&Where::Workspace)).expect("the workspace root");
        assert_eq!(named, root, "a virtual manifest defines no package, so no `--package` value could name it");
        assert_eq!(document_path(&named, "book"), root.join("docs/intent/book/lld.md"));
    }

    #[test]
    #[validates(spec::NoTargetInAOneMemberWorkspaceIsThatMembersDirectory)]
    fn no_target_in_a_one_member_workspace_is_that_members_directory() {
        let root = fixture::scratch("coach-sole-member");
        let project = project_at(&root, &[("app", "app")]);
        assert_eq!(sole_member(&members(&project)).expect("exactly one member"), root.join("app"));
        assert_eq!(package_dir(&project, None).expect("no target"), root.join("app"), "the only place the document could go");
    }

    #[test]
    #[validates(spec::NoTargetInAWorkspaceOfSeveralMembersStopsNamingTheFlag)]
    fn no_target_in_a_workspace_of_several_members_stops_naming_the_flag() {
        let root = fixture::scratch("coach-several-members");
        let project = project_at(&root, &[("first", "first"), ("app", "app")]);
        mentions(&sole_member(&members(&project)).expect_err("several members"), &["--package"]);
        mentions(&sole_member(&[]).expect_err("no member at all"), &["--package"]);
        let stop = package_dir(&project, None).expect_err("several members and no flag");
        mentions(&stop, &["--package"]);
    }

    #[test]
    #[validates(spec::TheDocumentIsTheSlicesLldUnderThatDirectory)]
    fn the_document_is_the_slices_lld_under_that_directory() {
        let under = document_path(Path::new("/w/app"), "login");
        assert_eq!(under, Path::new("/w/app/docs/intent/login/lld.md"));
        assert_eq!(document_path(Path::new("/w"), "book"), Path::new("/w/docs/intent/book/lld.md"));
        assert_eq!(slice_named(&under), "login", "the slice the path was built from, read back out of it");
    }

    #[test]
    #[validates(spec::TheDocumentsPathIsPrintedWhenTheSessionOpens)]
    fn the_documents_path_is_printed_when_the_session_opens() {
        // Asserted at the line, as `EveryPhasePrintsItsSessionsAndItsEnding` is
        // asserted through `opened_line`: what a print puts on the terminal has
        // no in-process seam, and the line it is given does.
        let line = path_line(Path::new("/w/app/docs/intent/hello/lld.md"));
        mentions(&line, &["/w/app/docs/intent/hello/lld.md"]);
        assert_eq!(line.lines().count(), 1, "one line, as the session opens");
    }

    // ---- before the interview --------------------------------------------------

    #[test]
    #[validates(spec::TheArtifactChecksRunOnceBeforeTheFirstQuestion)]
    fn the_artifact_checks_run_once_before_the_first_question() {
        let (root, project) = scratch_project("coach-artifact-checks");
        assert_eq!(setup(&project).expect("healthy artifacts"), Vec::<String>::new(), "this project's own copies hold both");
        let guideline = std::fs::read_to_string(root.join(GUIDELINE)).expect("the synced guideline");
        write_at(&root.join(GUIDELINE), &guideline.replace("`DecisionsExist`", "the decisions rule"));
        let reader = std::fs::read_to_string(root.join(READER)).expect("the synced reader");
        write_at(&root.join(READER), &reader.replace("tools: Read, Grep, Glob", "tools: Read, Grep, Glob, Write"));
        let warnings = setup(&project).expect("their project's problem, not this conversation's");
        assert_eq!(warnings.len(), 2, "both artifact checks ran: {warnings:?}");
        mentions(&warnings.join("\n"), &["DecisionsExist", "Write"]);
    }

    #[test]
    #[validates(spec::AnUnreadableGuidelineStopsTheRunNamingItsPath)]
    fn an_unreadable_guideline_stops_the_run_naming_its_path() {
        let (root, project) = scratch_project("coach-guideline-unreadable");
        std::fs::remove_file(root.join(GUIDELINE)).expect("take the guideline away");
        let stopped = coaching_system(&project).expect_err("half the system prompt is gone");
        mentions(&stopped, &[&root.join(GUIDELINE).display().to_string()]);
        synced_text(&project, METHOD).expect("the interview method is still there");
        assert!(!setup(&project).expect("the artifact checks answer").is_empty(), "setup reports it; coaching_system stops on it");
    }

    #[test]
    #[validates(spec::ADriftedChecklistIsReportedAndTheInterviewProceeds)]
    fn a_drifted_checklist_is_reported_and_the_interview_proceeds() {
        let (root, project) = scratch_project("coach-checklist-drift");
        let guideline = std::fs::read_to_string(root.join(GUIDELINE)).expect("the synced guideline");
        write_at(&root.join(GUIDELINE), &guideline.replace("`ShapeRows`", "the shape rule"));
        let warnings = setup(&project).expect("a drifted checklist does not stop the interview");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        mentions(&warnings[0], &["ShapeRows", &root.join(GUIDELINE).display().to_string()]);
        coaching_system(&project).expect("a guideline that reads is half the system prompt whatever its checklist omits");
    }

    #[test]
    #[validates(spec::AReaderDeclaringTooManyToolsIsReportedAndTheInterviewProceeds)]
    fn a_reader_declaring_too_many_tools_is_reported_and_the_interview_proceeds() {
        let (root, project) = scratch_project("coach-reader-declares-too-much");
        let reader = std::fs::read_to_string(root.join(READER)).expect("the synced reader");
        write_at(&root.join(READER), &reader.replace("tools: Read, Grep, Glob", "tools: Read, Grep, Glob, Edit"));
        let warnings = setup(&project).expect("the project's problem, not this conversation's");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        mentions(&warnings[0], &["Edit", &root.join(READER).display().to_string()]);
    }

    // ---- what the sessions are opened with -------------------------------------

    #[test]
    #[validates(spec::TheSystemPromptIsTheSyncedInterviewMethodThenTheGuideline)]
    fn the_system_prompt_is_the_synced_interview_method_then_the_guideline() {
        let (root, project) = scratch_project("coach-system-prompt");
        let method = std::fs::read_to_string(root.join(METHOD)).expect("the synced method");
        let guideline = std::fs::read_to_string(root.join(GUIDELINE)).expect("the synced guideline");
        let system = coaching_system(&project).expect("both synced copies");
        let (at_method, at_guideline) = (system.find(&method), system.find(&guideline));
        assert!(at_method.is_some() && at_method < at_guideline, "how to interview, then the questions it will be judged by");
        let named = (METHOD, synced_text(&project, METHOD).expect("read under the workspace root"));
        assert_eq!(named, (".claude/skills/lid-rs/references/coach.md", method));
        assert_eq!(coaching_settings(&project, MAX_COST).expect("the dial").system, system);
    }

    #[test]
    #[validates(spec::TheOpeningNamesTheSlice)]
    fn the_opening_names_the_slice() {
        let (root, project) = scratch_project("coach-opening-names-the-slice");
        let path = root.join("docs/intent/login/lld.md");
        let repository = preamble(&project, &path);
        mentions(&writing(&repository, "login"), &["login"]);
        mentions(&amending(&repository, "login", HOLDS), &["login"]);
        assert_eq!(opening(&project, "login", &path), writing(&repository, "login"), "a slice with no document is told it is writing one");
        mentions(&opening(&project, "login", &path), &["login"]);
    }

    #[test]
    #[validates(spec::AnExistingDocumentIsReadWholeIntoTheOpeningAsAnAmendment)]
    fn an_existing_document_is_read_whole_into_the_opening_as_an_amendment() {
        let (root, project) = scratch_project("coach-opening-amends");
        let path = root.join("docs/intent/hello/lld.md");
        assert_eq!(existing(&path), None, "a slice with no document yet is the ordinary case, not a fault");
        write_at(&path, HOLDS);
        let repository = preamble(&project, &path);
        let amendment = opening(&project, "hello", &path);
        let read_whole = (existing(&path), amendment.clone());
        assert_eq!(read_whole, (Some(HOLDS.to_string()), amending(&repository, "hello", HOLDS)));
        mentions(&amendment, &[HOLDS, "amend"]);
        assert!(!writing(&repository, "hello").contains(HOLDS), "and a slice with no document carries none");
    }

    #[test]
    #[validates(spec::TheCoachDeclaresExactlyTheReadGrepDraftAndAskTools)]
    fn the_coach_declares_exactly_the_read_grep_draft_and_ask_tools() {
        let (_root, project) = scratch_project("coach-declares-four");
        let coachs = declarations();
        let ops: Vec<&str> = coachs.iter().map(|tool| tool.op.as_str()).collect();
        let set = (COACH_TOOLS, ops.clone(), coachs.clone());
        let four = ([Tool::Read, Tool::Grep, Tool::Draft, Tool::Ask], vec!["read", "grep", "draft", "ask"], COACH_TOOLS.map(declaration).to_vec());
        assert_eq!(set, four, "a set of the coach's own, not the canopy client's five");
        let admitted: Vec<Tool> = ops.iter().map(|op| declared(op).expect("declared")).collect();
        let named = coachs.iter().all(|tool| tool.requestee == REQUESTEE && tool.name == tool.op);
        let outside = ["glob", "edit", "write"].iter().all(|op| declared(op).is_err());
        assert_eq!((admitted, named, outside), (COACH_TOOLS.to_vec(), true, true), "each under its own `op`; the client's five are another set");
        assert_eq!(coaching_settings(&project, MAX_COST).expect("the dial").policy, policy_for(&coachs));
    }

    #[test]
    #[validates(spec::EverySessionTheCoachOpensIsDialledWithTheMaxCost)]
    fn every_session_the_coach_opens_is_dialled_with_the_max_cost() {
        let (_root, project) = scratch_project("coach-max-cost");
        let coaching = coaching_settings(&project, MAX_COST).expect("the coaching dial");
        let reading = reader_settings(&project, MAX_COST).expect("a reader's dial");
        assert_eq!((coaching.max_cost, reading.max_cost), (MAX_COST, MAX_COST), "the run is bounded one session at a time");
        assert_eq!(coaching_settings(&project, 0.5).expect("another budget").max_cost, 0.5);
        assert_eq!(reader_settings(&project, 0.5).expect("another budget").max_cost, 0.5);
    }

    // ---- the loop --------------------------------------------------------------

    #[test]
    #[validates(spec::TheFirstTurnSettlesOnTheOpeningBeforeTheHumanIsRead)]
    fn the_first_turn_settles_on_the_opening_before_the_human_is_read() {
        let driven = driven("coach-first-turn");
        let messages = replay::user_messages(&driven.replay.landed("s-coach"));
        let opened = (messages.first().map(String::as_str), messages.len(), driven.path.is_file());
        assert_eq!(opened, (Some(OPENING), 2, true), "the opening is the first user message, and the turn it opened settled: it drafted");
        assert_eq!(driven.ended, Err(Halt::Halted(HALTED.to_string())), "nothing typed: a run at end of file still opens, settles a turn and ends");
    }

    #[test]
    #[validates(spec::TheModelsSettledAnswerIsPrinted)]
    fn the_models_settled_answer_is_printed() {
        // What `println!` puts on the terminal has no in-process seam, and the
        // model's settled answer is its verbatim text rather than a line this
        // module composes — so what is asserted is the driven path: each turn is
        // driven to a settled answer and the loop carries on from it.
        let driven = driven("coach-settled-answer");
        let messages = replay::user_messages(&driven.replay.landed("s-coach"));
        assert_eq!(messages.len(), 2, "the first turn settled — the judges answered it — and the second was driven on that");
        assert!(messages[1].starts_with(JUDGING_HEADING), "what followed the settled answer is the judges', not the model's");
        assert_eq!(driven.ended, Err(Halt::Halted(HALTED.to_string())), "and the second turn ended the loop rather than settling");
    }

    #[test]
    #[validates(spec::TheConversationEndsAtDoneOrEndOfFile)]
    fn the_conversation_ends_at_done_or_end_of_file() {
        // Asserted at `typed` and at `human_turn`, each of which is handed the
        // line rather than reading it; only `read_line` needs a terminal, and
        // `ask` reaches this through `replied`.
        let ended = (typed(None), typed(Some(DONE.to_string())), typed(Some(format!("{DONE}\n"))));
        assert_eq!(ended, (Typed::Ended, Typed::Ended, Typed::Ended), "end of file, and `done` alone on a line with or without its newline");
        let answers = (typed(Some("not yet".to_string())), typed(Some("done for now".to_string())));
        let theirs = (Typed::Answer("not yet".to_string()), Typed::Answer("done for now".to_string()));
        let at_the_prompt = (human_turn(None), human_turn(Some(format!("{DONE}\n"))), human_turn(Some("not yet\n".to_string())));
        let read_back = (None, None, Some("not yet".to_string()));
        assert_eq!((answers, at_the_prompt), (theirs, read_back), "alone on a line and not merely first on it, at the prompt as in the tool");
        assert_eq!(DONE, "done");
    }

    #[test]
    #[validates(spec::TheCoachingSessionIsStoppedBeforeTheClientExits)]
    fn the_coaching_session_is_stopped_before_the_client_exits() {
        let (root, project) = scratch_project("coach-session-stopped");
        let replay = Replay::serve(vec![drafting_then_halted("s-coach", FAILS), answering_reader("s-reader")]);
        let flags = Flags { target: None, slice: Some("hello".to_string()), door: replay.url.clone(), max_cost: MAX_COST };
        let ended = coached(&project, &replay.door("k"), &flags).expect_err("the turn answering the judges is halted");
        mentions(&ended, &[HALTED]);
        assert!(replay.stopped("s-coach"), "however the conversation ended, a session left open is an unsealed log");
        assert_eq!(replay.opened(), strings(&["s-coach", "s-reader"]));
        assert!(root.join("docs/intent/hello/lld.md").is_file(), "under the sole member, where the flags named no other");
    }

    #[test]
    #[validates(spec::ThatATurnDraftedIsRecordedByTheExecutorNotInferred)]
    fn that_a_turn_drafted_is_recorded_by_the_executor_not_inferred() {
        let (root, project) = scratch_project("coach-drafted-recorded");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&root.join("src/lib.rs"), "//! the module\n");
        let mut drafting = Noted::default();
        execute(&project, &path, &mut drafting, "draft", &json!({ "content": HOLDS })).expect("the document is written");
        assert_eq!(drafting, Noted { drafted: true, wrote: true, ended: false }, "the executor ran the tool, and recorded that it did");
        let mut reading = Noted::default();
        execute(&project, &path, &mut reading, "read", &json!({ "path": "src/lib.rs" })).expect("the read");
        assert_eq!(reading, Noted::default(), "a turn that only read drafted nothing");
        assert_eq!(next_after(reading, Answering::TheHuman), Next::Human, "and what the model said about it is never read");
    }

    #[test]
    #[validates(spec::TheJudgesAnswerATurnThatDrafted)]
    fn the_judges_answer_a_turn_that_drafted() {
        let (root, project) = scratch_project("coach-judges-answer");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![answering_reader("s-reader")]);
        let coach = judging_coach(&replay, &path);
        let wrote = Noted { drafted: true, wrote: true, ended: false };
        assert_eq!(next_after(wrote, Answering::TheHuman), Next::Judges);
        let landed = next_message(&project, &coach, Next::Judges).expect("the judges land the next message");
        assert_eq!((landed.answering, landed.holds), (Answering::TheJudges, Some(true)), "rather than the run waiting for the human");
        mentions(&landed.text, &[JUDGING_HEADING, EVERY_CHECK_HELD, FINDING]);
    }

    #[test]
    #[validates(spec::AFailedDraftIsNotATurnThatDrafted)]
    fn a_failed_draft_is_not_a_turn_that_drafted() {
        let root = fixture::scratch("coach-draft-failed");
        write_at(&root.join("src/hello.rs"), "//! a file, and not a directory\n");
        let mut noted = Noted::default();
        let under_a_file = root.join("src/hello.rs/lld.md");
        draft_call(&under_a_file, &mut noted, &json!({ "content": HOLDS })).expect_err("no directory can be made under a file");
        assert_eq!(noted, Noted { drafted: true, wrote: false, ended: false }, "called, and recorded as not having written");
        assert_eq!(next_after(noted, Answering::TheHuman), Next::Human, "so the judges do not run for it");
    }

    #[test]
    #[validates(spec::ATurnThatDraftedNothingIsAnsweredByTheHuman)]
    fn a_turn_that_drafted_nothing_is_answered_by_the_human() {
        assert_eq!(next_after(Noted::default(), Answering::TheHuman), Next::Human);
        assert_eq!(next_after(Noted::default(), Answering::TheJudges), Next::Human);
        let failed = Noted { drafted: true, wrote: false, ended: false };
        assert_eq!(next_after(failed, Answering::TheHuman), Next::Human, "a failed draft wrote nothing");
    }

    #[test]
    #[validates(spec::TheJudgesAnswerAtMostOneDraftingTurnInARow)]
    fn the_judges_answer_at_most_one_drafting_turn_in_a_row() {
        let wrote = Noted { drafted: true, wrote: true, ended: false };
        assert_eq!(next_after(wrote, Answering::TheHuman), Next::Judges, "the first drafting turn is the judges' to answer");
        assert_eq!(next_after(wrote, Answering::TheJudges), Next::Human, "the one that drafts in reply to them is the human's");
    }

    #[test]
    #[validates(spec::AHaltReachingTheLoopEndsTheConversationWithItsSentence)]
    fn a_halt_reaching_the_loop_ends_the_conversation_with_its_sentence() {
        let driven = driven("coach-halted");
        assert_eq!(driven.ended, Err(Halt::Halted(HALTED.to_string())), "the halt's own sentence, unanswered by a verdict");
    }

    #[test]
    #[validates(spec::TheDraftedDocumentSurvivesAHalt)]
    fn the_drafted_document_survives_a_halt() {
        let driven = driven("coach-halt-leaves-the-document");
        assert!(driven.ended.is_err(), "the conversation ended on the platform's halt");
        assert_eq!(std::fs::read_to_string(&driven.path).expect("the document"), FAILS, "`draft` wrote to disk, not into the session");
    }

    // ---- the three tools -------------------------------------------------------

    #[test]
    #[validates(spec::AnOpTheCoachDidNotDeclareIsRefusedWithItsName)]
    fn an_op_the_coach_did_not_declare_is_refused_with_its_name() {
        let (root, project) = scratch_project("coach-undeclared-op");
        let path = root.join("docs/intent/hello/lld.md");
        let mut noted = Noted::default();
        let args = json!({ "path": "src/lib.rs", "old_string": "a", "new_string": "b" });
        mentions(&execute(&project, &path, &mut noted, "edit", &args).expect_err("the coach declares no `edit`"), &["edit"]);
        mentions(&declared("edit").expect_err("outside the set"), &["edit"]);
        assert_eq!((noted, path.exists()), (Noted::default(), false), "a call outside the set runs nothing and writes nothing");
    }

    #[test]
    #[validates(spec::AReadIsRoutedToTheCanopyClientsReadOverItsConfinement)]
    fn a_read_is_routed_to_the_canopy_clients_read_over_its_confinement() {
        let (root, project) = scratch_project("coach-read-routed");
        let (path, module) = (root.join("docs/intent/hello/lld.md"), root.join("src/lib.rs"));
        write_at(&module, "//! the module\n");
        let mut noted = Noted::default();
        let answer = execute(&project, &path, &mut noted, "read", &json!({ "path": "src/lib.rs" })).expect("the read");
        let clients = ReadArgs { path: "src/lib.rs".to_string(), offset: None, limit: None };
        assert_eq!(answer, read_tool(&module, &clients).expect("the canopy client's own read"), "the answer a phase worker gets");
        assert_eq!(answer, "1\t//! the module");
        let climbing = execute(&project, &path, &mut noted, "read", &json!({ "path": "../elsewhere" })).expect_err("confined");
        mentions(&climbing, &["outside the workspace"]);
        assert_eq!(noted, Noted::default(), "a read is neither a draft nor an ending");
    }

    #[test]
    #[validates(spec::DraftReplacesTheDocumentWholeCreatingItsDirectory)]
    fn draft_replaces_the_document_whole_creating_its_directory() {
        let root = fixture::scratch("coach-draft-whole");
        let path = root.join("docs/intent/hello/lld.md");
        assert!(!path.parent().expect("a parent").exists(), "the directory is not there yet");
        let first = draft(&path, HOLDS).expect("the directory is created with it");
        assert_eq!((first, std::fs::read_to_string(&path).expect("the document")), (HOLDS.len(), HOLDS.to_string()));
        let second = draft(&path, FAILS).expect("replaced");
        let replaced = (second, std::fs::read_to_string(&path).expect("the document"));
        assert_eq!(replaced, (FAILS.len(), FAILS.to_string()), "whole: there is no partial edit of it");
    }

    #[test]
    #[validates(spec::DraftWritesTheSlicesLldAndNoOtherPath)]
    fn draft_writes_the_slices_lld_and_no_other_path() {
        let (root, project) = scratch_project("coach-draft-one-path");
        let (path, module) = (root.join("docs/intent/hello/lld.md"), root.join("src/lib.rs"));
        write_at(&module, "//! the module\n");
        let mut noted = Noted::default();
        let args = json!({ "content": HOLDS, "path": "src/lib.rs" });
        execute(&project, &path, &mut noted, "draft", &args).expect("the run's one document");
        let written = (std::fs::read_to_string(&path).expect("the document"), std::fs::read_to_string(&module).expect("the module"));
        assert_eq!(written, (HOLDS.to_string(), "//! the module\n".to_string()), "the run's one path; no call of it reaches the module");
        let schema = Tool::Draft.schema();
        assert_eq!((&schema["required"], &schema["properties"]["path"]), (&json!(["content"]), &Value::Null), "`content` is the whole schema");
    }

    #[test]
    #[validates(spec::DraftAnswersWithThePathAndTheBytesWrittenNotAVerdict)]
    fn draft_answers_with_the_path_and_the_bytes_written_not_a_verdict() {
        let root = fixture::scratch("coach-draft-answer");
        let path = root.join("docs/intent/hello/lld.md");
        let mut noted = Noted::default();
        let answer = draft_call(&path, &mut noted, &json!({ "content": FAILS })).expect("written");
        assert_eq!(answer, wrote_line(&path, FAILS.len()));
        mentions(&answer, &[&path.display().to_string(), &FAILS.len().to_string()]);
        let verdicts = [EVERY_CHECK_HELD, CHECKS_HOLD, CHECKS_DO_NOT_HOLD, "DecisionsExist"];
        assert!(!verdicts.iter().any(|verdict| answer.contains(verdict)), "the checks belong to the judges' turn: {answer}");
    }

    #[test]
    #[validates(spec::AskPutsItsQuestionToTheHumanAndAnswersWithWhatTheyTyped)]
    fn ask_puts_its_question_to_the_human_and_answers_with_what_they_typed() {
        // Asserted at `answered` and `replied`, which carry this over a line a
        // test wrote: `ask` itself reaches them only through a blocking read.
        let options = strings(&["a closed set", "another", "a third"]);
        let picked = answered(&options, "2");
        let fourth = answered(&options, "a fourth answer");
        let open = answered(&[], "free text");
        let past_the_end = answered(&options, "9");
        let given = (picked.as_str(), fourth.as_str(), open.as_str(), past_the_end.as_str());
        assert_eq!(given, ("another", "a fourth answer", "free text", "9"), "a bare number picks; anything else is answered verbatim");
        let by_number = (option_named(&options, "1").map(String::as_str), option_named(&options, "0").map(String::as_str));
        assert_eq!(by_number, (Some("a closed set"), None), "counted from one, as the question printed it");
        let mut noted = Noted::default();
        let through_reply = (replied(&options, &Typed::Answer("2".to_string()), &mut noted), noted);
        assert_eq!(through_reply, (Ok("another".to_string()), Noted::default()), "an answer is not an ending");
    }

    #[test]
    #[validates(spec::TheStallWindowIsPrintedOnceBesideTheQuestion)]
    fn the_stall_window_is_printed_once_beside_the_question() {
        let options = strings(&["one", "two"]);
        let question = "Which package holds this slice?";
        let open = asked(question, &[]);
        let closed = asked(question, &options);
        mentions(&open, &[question, STALL_WINDOW]);
        mentions(&closed, &[question, STALL_WINDOW, "one", "two"]);
        assert_eq!((open.matches(STALL_WINDOW).count(), closed.matches(STALL_WINDOW).count()), (1, 1), "once, as the question is asked");
        let mut noted = Noted::default();
        let answer = replied(&options, &Typed::Answer("a fourth".to_string()), &mut noted).expect("the answer");
        assert!(!answer.contains(STALL_WINDOW), "and at no later moment");
        assert!(STALL_WINDOW.contains("fifteen minutes") && QUIET_TAIL == Duration::from_secs(15 * 60), "canopy's own invoke stall");
    }

    #[test]
    #[validates(spec::EndingTheConversationInsideAskIsAToolError)]
    fn ending_the_conversation_inside_ask_is_a_tool_error() {
        let mut noted = Noted::default();
        let directly = (ended_in_ask(&mut noted), noted);
        let recorded = (Err(CONVERSATION_OVER.to_string()), Noted { drafted: false, wrote: false, ended: true });
        assert_eq!(directly, recorded, "a tool has no other channel to say it through");
        let mut typed_as_the_answer = Noted::default();
        let through_reply = (replied(&[], &Typed::Ended, &mut typed_as_the_answer), typed_as_the_answer.ended);
        assert_eq!(through_reply, (Err(CONVERSATION_OVER.to_string()), true), "`done` or end of file, typed at an outstanding question");
    }

    #[test]
    #[validates(spec::AConversationEndedInsideAskEndsTheLoopWhenTheTurnSettles)]
    fn a_conversation_ended_inside_ask_ends_the_loop_when_the_turn_settles() {
        let (root, project) = scratch_project("coach-ended-in-ask");
        let mut noted = Noted { drafted: true, wrote: true, ended: false };
        replied(&[], &Typed::Ended, &mut noted).expect_err("the conversation is over");
        let after = next_after(Noted { ended: true, ..Noted::default() }, Answering::TheJudges);
        let follows = (noted.ended, next_after(noted, Answering::TheHuman), after);
        assert_eq!(follows, (true, Next::Nothing, Next::Nothing), "the record carries the ending out of `ask`, whatever else the turn did");
        let replay = Replay::serve(vec![]);
        let coach = judging_coach(&replay, &root.join("docs/intent/hello/lld.md"));
        assert_eq!(next_message(&project, &coach, Next::Nothing), None, "the loop ends rather than reading them again");
    }

    // ---- what the judges say ---------------------------------------------------

    #[test]
    #[validates(spec::TheJudgesTurnIsTheDocumentChecksThenTheReader)]
    fn the_judges_turn_is_the_document_checks_then_the_reader() {
        let failures = vec![failure_at(Check::DecisionsExist, Path::new("/w/docs/intent/hello/lld.md"), 1)];
        let findings = strings(&[FINDING]);
        let message = judges_message(&Ok(failures.clone()), &Ok(findings.clone()));
        let at_checks = message.find(&checks_section(&Ok(failures))).expect("the document checks' part");
        let at_reader = message.find(&reader_section(&Ok(findings))).expect("the reader's part");
        assert!(at_checks < at_reader, "the four document checks first, the reader's findings second: {message}");
        assert!(message.starts_with(JUDGING_HEADING));
    }

    #[test]
    #[validates(spec::TheDocumentChecksRunOverThePathDraftWrote)]
    fn the_document_checks_run_over_the_path_draft_wrote() {
        let root = fixture::scratch("coach-checks-over-the-path");
        let (resolved, drafted) = (root.join("docs/intent/hello/lld.md"), root.join("elsewhere/docs/intent/hello/lld.md"));
        write_at(&resolved, HOLDS);
        write_at(&drafted, FAILS);
        let failures = document_failures(&drafted).expect("the document reads back");
        let over_it = failures.iter().all(|failure| failure.path == drafted);
        let elsewhere = document_failures(&resolved).expect("reads back").is_empty();
        assert_eq!((failures.is_empty(), over_it, elsewhere), (false, true, true), "the path it wrote, not one resolved again: {failures:?}");
        assert!(failures_section(&failures).contains(&rendered(&failures)), "rendered as `lld-check` renders them for a human");
    }

    #[test]
    #[validates(spec::AJudgingWhoseChecksAllHeldSaysSo)]
    fn a_judging_whose_checks_all_held_says_so() {
        let held = failures_section(&[]);
        mentions(&held, &[EVERY_CHECK_HELD]);
        assert_eq!(checks_section(&Ok(vec![])), held);
        let failed = failures_section(&[failure_at(Check::Alternatives, Path::new("/w/docs/intent/hello/lld.md"), 9)]);
        assert!(!failed.contains(EVERY_CHECK_HELD), "a message that said nothing of them would read as one that ran none");
    }

    #[test]
    #[validates(spec::TheArtifactChecksAreNotInTheJudgesTurn)]
    fn the_artifact_checks_are_not_in_the_judges_turn() {
        let (root, project) = scratch_project("coach-no-artifact-checks");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, FAILS);
        let guideline = std::fs::read_to_string(root.join(GUIDELINE)).expect("the synced guideline");
        write_at(&root.join(GUIDELINE), &guideline.replace("`ShapeRows`", "the shape rule"));
        let every = check_all(&project, &document(&path).expect("the document reads back")).expect("every check");
        let judged = document_failures(&path).expect("the judges' four");
        assert!(every.iter().any(|f| f.check == Check::GuidelineNamesEveryCheck), "lld-check sees the drifted guideline");
        let four = [Check::DecisionsExist, Check::Alternatives, Check::ShapeRows, Check::DeferredNumbered];
        assert!(judged.iter().all(|f| four.contains(&f.check)), "the judges' turn carries the document's checks alone: {judged:?}");
        assert!(!judges_message(&Ok(judged), &Ok(vec![])).contains("GuidelineNamesEveryCheck"));
    }

    #[test]
    #[validates(spec::AJudgingsVerdictIsTheFourDocumentChecksAndNothingElse)]
    fn a_judgings_verdict_is_the_four_document_checks_and_nothing_else() {
        let (root, project) = scratch_project("coach-verdict-checks-only");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        assert!(checks_hold(&Ok(vec![])));
        assert!(!checks_hold(&Ok(vec![failure_at(Check::ShapeRows, &path, 5)])));
        let replay = Replay::serve(vec![answering_reader("s-answers"), refused_reader("s-refuses", "the tenant has no budget")]);
        let coach = judging_coach(&replay, &path);
        let (with_reader, without) = (judged(&project, &coach), judged(&project, &coach));
        assert_eq!((with_reader.holds, without.holds), (true, true), "the reader is the verdict's neighbour, not its subject");
        mentions(&without.message, &[READER_UNCONSULTED, EVERY_CHECK_HELD]);
    }

    #[test]
    #[validates(spec::TheHumanIsToldHowTheDocumentChecksFoundTheDocument)]
    fn the_human_is_told_how_the_document_checks_found_the_document() {
        let path = Path::new("/w/docs/intent/hello/lld.md");
        let held = checks_line(&Ok(vec![]));
        let failed = checks_line(&Ok(vec![failure_at(Check::DecisionsExist, path, 1), failure_at(Check::ShapeRows, path, 7)]));
        assert!(!held.is_empty() && !failed.is_empty(), "a judging says how it found the document either way");
        mentions(&failed, &["2", "failed"]);
        assert_ne!(held, failed, "where a run tells the human whether what was just written holds");
        // A line that counted nothing failing is not the line a document that
        // held is told about: a human reading "0 of the four failed" has been
        // told a count where they were owed a verdict.
        assert!(held.contains("hold") && !held.contains("failed"), "the checks held, and the human is told that rather than a count: {held}");
    }

    #[test]
    #[validates(spec::AnUnreadableDocumentsSentenceIsLandedInPlaceOfTheChecksFailures)]
    fn an_unreadable_documents_sentence_is_landed_in_place_of_the_checks_failures() {
        let root = fixture::scratch("coach-unreadable-document");
        let path = root.join("docs/intent/hello/lld.md");
        let gone = document(&path).expect_err("the file `draft` reported writing has gone");
        mentions(&gone, &[&path.display().to_string()]);
        let section = checks_section(&Err(gone.clone()));
        mentions(&section, &[DOCUMENT_UNREADABLE, &gone]);
        assert!(!section.contains(EVERY_CHECK_HELD), "a document nobody can see neither holds nor fails");
        mentions(&judges_message(&Err(gone), &Ok(strings(&[FINDING]))), &[DOCUMENT_UNREADABLE, FINDING]);
    }

    #[test]
    #[validates(spec::AnUnreadableDocumentIsToldToTheHuman)]
    fn an_unreadable_document_is_told_to_the_human() {
        let gone = "reading `docs/intent/hello/lld.md`: no such file or directory".to_string();
        let line = checks_line(&Err(gone.clone()));
        mentions(&line, &[DOCUMENT_UNREADABLE]);
        assert_ne!(line, checks_line(&Ok(vec![])), "they are the only one who can put back a document that has gone");
        mentions(&judging_line(&Err(gone), &Ok(strings(&[FINDING]))), &[DOCUMENT_UNREADABLE]);
    }

    #[test]
    #[validates(spec::AnUnreadableDocumentsVerdictIsThatTheChecksDoNotHold)]
    fn an_unreadable_documents_verdict_is_that_the_checks_do_not_hold() {
        assert!(!checks_hold(&Err("no such file".to_string())), "a verdict is a statement about checks that ran, and none did");
        assert!(checks_hold(&Ok(vec![])), "and a document that read back with nothing against it holds");
    }

    #[test]
    #[validates(spec::ADocumentThatCannotBeReadBackDoesNotEndTheConversation)]
    fn a_document_that_cannot_be_read_back_does_not_end_the_conversation() {
        let (root, project) = scratch_project("coach-unreadable-continues");
        let never_written = root.join("docs/intent/hello/lld.md");
        let replay = Replay::serve(vec![answering_reader("s-reader")]);
        let judging = judged(&project, &judging_coach(&replay, &never_written));
        assert!(!judging.holds, "a judging that could run no check does not say the document holds");
        mentions(&judging.message, &[JUDGING_HEADING, DOCUMENT_UNREADABLE, FINDING]);
        assert_eq!(replay.opened(), strings(&["s-reader"]), "the model is still owed an answer, and the human is mid-interview");
    }

    #[test]
    #[validates(spec::TheReadersSystemIsTheSyncedReaderBody)]
    fn the_readers_system_is_the_synced_reader_body() {
        let (root, project) = scratch_project("coach-reader-body");
        let text = std::fs::read_to_string(root.join(READER)).expect("the synced reader");
        let body = reader_body(&project).expect("the reader's body");
        assert_eq!(body, without_frontmatter(&text), "as the canopy client reads an agent's body");
        assert!(!body.contains("tools: Read, Grep, Glob"), "its frontmatter is not its body");
        assert_eq!(reader_settings(&project, MAX_COST).expect("a reader's dial").system, body);
    }

    #[test]
    #[validates(spec::AReaderSessionDeclaresTheCanopyClientsObservationTools)]
    fn a_reader_session_declares_the_canopy_clients_observation_tools() {
        let (_root, project) = scratch_project("coach-reader-observes");
        let observation = canopy_declarations(&[CanopyTool::Read, CanopyTool::Grep, CanopyTool::Glob]);
        let dial = reader_settings(&project, MAX_COST).expect("a reader's dial");
        assert_eq!(dial.policy, policy_for(&observation), "a reading observes and cannot act");
        let ops: Vec<&str> = dial.policy.tools.iter().map(|tool| tool.op.as_str()).collect();
        assert_eq!(ops, ["read", "grep", "glob"]);
    }

    #[test]
    #[validates(spec::AReaderSessionCarriesNoPhase)]
    fn a_reader_session_carries_no_phase() {
        let (root, project) = scratch_project("coach-reader-phaseless");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![observing_reader("s-reader", "docs/intent/hello/lld.md")]);
        reader_findings(&project, &judging_coach(&replay, &path)).expect("the reader answered");
        let counted = tally::load(&project, "canopy:s-reader").expect("the tally");
        assert_eq!(counted, tally::Tally::default(), "a reading belongs to no phase whose tally would count it");
    }

    #[test]
    #[validates(spec::AReaderTurnIsRunByTheCanopyClientsOwnDispatch)]
    fn a_reader_turn_is_run_by_the_canopy_clients_own_dispatch() {
        let (root, project) = scratch_project("coach-reader-dispatch");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![observing_reader("s-reader", "docs/intent/hello/lld.md")]);
        let findings = reader_findings(&project, &judging_coach(&replay, &path)).expect("the reader answered");
        assert_eq!(findings, strings(&[FINDING]));
        let answers = replay::completions(&replay.landed("s-reader"));
        mentions(answers[0].body["result"].as_str().expect("the read's text"), &["## Shape"]);
        assert_eq!(answers[1].body["outcome"].as_str(), Some("error"), "the coach's own `draft` is no tool of that dispatch");
        mentions(answers[1].body["error"].as_str().expect("the refusal"), &["draft"]);
        assert_eq!(std::fs::read_to_string(&path).expect("the document"), HOLDS, "and nothing wrote it");
    }

    #[test]
    #[validates(spec::TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens)]
    fn the_judgings_heading_is_printed_before_a_reader_session_opens() {
        // The order of two prints — this heading's and `Session::open`'s — has
        // no in-process seam, as `EveryPhasePrintsItsSessionsAndItsEnding` is
        // asserted through `opened_line` rather than through the print. What is
        // asserted is the heading and that a judging opens one reader under it.
        let (root, project) = scratch_project("coach-judging-heading");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![answering_reader("s-reader")]);
        let judging = judged(&project, &judging_coach(&replay, &path));
        assert_eq!(JUDGING_HEADING, "## The judges");
        assert!(judging.message.starts_with(JUDGING_HEADING), "the line a reader session's own opening follows");
        assert_eq!(replay.opened(), strings(&["s-reader"]), "one reader session, opened by this judging");
    }

    #[test]
    #[validates(spec::TheReaderIsAFreshSessionForEveryJudging)]
    fn the_reader_is_a_fresh_session_for_every_judging() {
        let (root, project) = scratch_project("coach-fresh-reader");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![answering_reader("s-reader-1"), answering_reader("s-reader-2")]);
        let coach = judging_coach(&replay, &path);
        judged(&project, &coach);
        judged(&project, &coach);
        assert_eq!(replay.opened(), strings(&["s-reader-1", "s-reader-2"]), "a session of its own for each reading");
    }

    #[test]
    #[validates(spec::TheReaderIsGivenTheDocumentAndAskedForFindings)]
    fn the_reader_is_given_the_document_and_asked_for_findings() {
        let (root, project) = scratch_project("coach-reader-prompt");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        mentions(&reader_prompt(&path), &[&path.display().to_string(), "findings"]);
        let replay = Replay::serve(vec![answering_reader("s-reader")]);
        reader_findings(&project, &judging_coach(&replay, &path)).expect("the reader answered");
        let asked_for = replay::user_messages(&replay.landed("s-reader"));
        assert_eq!(asked_for, vec![reader_prompt(&path)], "what a phase would have given it");
    }

    #[test]
    #[validates(spec::AReaderSessionIsStoppedWhenItAnswers)]
    fn a_reader_session_is_stopped_when_it_answers() {
        let (root, project) = scratch_project("coach-reader-stopped");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let replay = Replay::serve(vec![answering_reader("s-reader")]);
        reader_findings(&project, &judging_coach(&replay, &path)).expect("the reader answered");
        assert!(replay.stopped("s-reader"), "a session left open is an unsealed log");
    }

    #[test]
    #[validates(spec::AFailedReadersSentenceIsLandedInPlaceOfItsFindings)]
    fn a_failed_readers_sentence_is_landed_in_place_of_its_findings() {
        let refused = "the door refused the dial: 403".to_string();
        let landed = reader_section(&Err(refused.clone()));
        mentions(&landed, &[READER_UNCONSULTED, &refused]);
        let answered_whole = reader_section(&Ok(strings(&[FINDING, "and a second finding"])));
        mentions(&answered_whole, &[FINDING, "and a second finding"]);
        assert!(!answered_whole.contains(READER_UNCONSULTED), "which findings matter is the human's judgment, so they land whole");
    }

    #[test]
    #[validates(spec::AFailedReaderIsToldToTheHuman)]
    fn a_failed_reader_is_told_to_the_human() {
        let halted = "the reader session halted: max_cost reached".to_string();
        mentions(&reader_line(&Err(halted.clone())), &[READER_UNCONSULTED, &halted]);
        assert!(reader_line(&Ok(strings(&[FINDING]))).is_empty(), "its findings went to the model; there is nothing here to tell them");
        mentions(&judging_line(&Ok(vec![]), &Err(halted)), &[READER_UNCONSULTED]);
    }

    #[test]
    #[validates(spec::AReaderThatCannotBeConsultedDoesNotEndTheConversation)]
    fn a_reader_that_cannot_be_consulted_does_not_end_the_conversation() {
        let (root, project) = scratch_project("coach-reader-lost");
        let path = root.join("docs/intent/hello/lld.md");
        write_at(&path, HOLDS);
        let refused = "the tenant has no budget left";
        let replay = Replay::serve(vec![refused_reader("s-first", refused), refused_reader("s-second", refused)]);
        let coach = judging_coach(&replay, &path);
        mentions(&reader_findings(&project, &coach).expect_err("the dial was refused"), &[refused]);
        let judging = judged(&project, &coach);
        assert!(judging.holds, "the document's checks held, whatever became of the second opinion");
        mentions(&judging.message, &[READER_UNCONSULTED, refused, EVERY_CHECK_HELD]);
    }

    #[test]
    #[validates(spec::TheDocumentChecksAreLandedWhetherOrNotTheReaderAnswers)]
    fn the_document_checks_are_landed_whether_or_not_the_reader_answers() {
        let path = Path::new("/w/docs/intent/hello/lld.md");
        let failures = vec![failure_at(Check::DeferredNumbered, path, 12)];
        let lost = "the reader could not be reached".to_string();
        mentions(&judges_message(&Ok(failures), &Err(lost.clone())), &["DeferredNumbered", READER_UNCONSULTED]);
        mentions(&judges_message(&Ok(vec![]), &Err(lost)), &[EVERY_CHECK_HELD, READER_UNCONSULTED]);
    }

    #[test]
    #[validates(spec::TheJudgesAreLandedAsOneUserMessageUnderAHeading)]
    fn the_judges_are_landed_as_one_user_message_under_a_heading() {
        let driven = driven("coach-judges-one-message");
        let messages = replay::user_messages(&driven.replay.landed("s-coach"));
        assert_eq!(messages.len(), 2, "the opening, then the judges' one message: {messages:?}");
        assert_eq!(messages[0], OPENING);
        assert!(messages[1].starts_with(JUDGING_HEADING), "so a reader of the sealed log tells the judges' turn from the human's");
        mentions(&messages[1], &["DecisionsExist", FINDING]);
    }

    // ---- the ending ------------------------------------------------------------

    #[test]
    #[validates(spec::TheEndingPrintsThePathAndWhetherTheChecksHold)]
    fn the_ending_prints_the_path_and_whether_the_checks_hold() {
        let path = Path::new("/w/app/docs/intent/hello/lld.md");
        let holds = owed(path, Some(true));
        let does_not = owed(path, Some(false));
        mentions(&holds, &["/w/app/docs/intent/hello/lld.md", CHECKS_HOLD]);
        mentions(&does_not, &["/w/app/docs/intent/hello/lld.md", CHECKS_DO_NOT_HOLD]);
        assert_eq!(holds, document_owed(path, CHECKS_HOLD), "the verdict the last judging reached, carried out of the loop");
        assert!(!holds.contains(CHECKS_DO_NOT_HOLD));
    }

    #[test]
    #[validates(spec::TheEndingNamesThePhaseOneCommitTheCoachDoesNotMake)]
    fn the_ending_names_the_phase_one_commit_the_coach_does_not_make() {
        let under_a_package = Path::new("/w/app/docs/intent/hello/lld.md");
        mentions(&document_owed(under_a_package, CHECKS_HOLD), &["phase 1: LLD for hello"]);
        let at_the_root = Path::new("/w/docs/intent/book/lld.md");
        mentions(&document_owed(at_the_root, CHECKS_DO_NOT_HOLD), &["phase 1: LLD for book"]);
    }

    #[test]
    #[validates(spec::ARunThatDraftedNothingEndsSayingSo)]
    fn a_run_that_drafted_nothing_ends_saying_so() {
        // Asserted at `owed`: the loop answers `None` only when the human has
        // ended a conversation that never drafted, which no driven run reaches.
        let path = Path::new("/w/app/docs/intent/hello/lld.md");
        let nothing = owed(path, None);
        assert_eq!(nothing, nothing_drafted(path));
        mentions(&nothing, &["/w/app/docs/intent/hello/lld.md"]);
        let said = [CHECKS_HOLD, CHECKS_DO_NOT_HOLD, "phase 1:"];
        assert!(!said.iter().any(|verdict| nothing.contains(verdict)), "nothing to check, and nothing to commit: {nothing}");
        assert_ne!(nothing, owed(path, Some(false)), "a run that drafted a failing document drafted something");
    }
}
