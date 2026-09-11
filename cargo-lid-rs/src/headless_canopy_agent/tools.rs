//! The five tools (`docs/intent/headless-canopy-agent/lld.md` § The five
//! tools): the closed set the model may call, their declarations, the
//! workspace boundary every one applies first, the dispatch that asks the
//! phase library's verdicts around the work, and the work itself.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use glob::Pattern;
use lid_rs::implements;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::door::ToolDecl;
use super::turn::Session;
use crate::phase::{HookInput, HookVerdict, Phase, hook_post_edit, hook_pre_tool};
use crate::project::Project;
use crate::spec;

/// The principal that executes the tools: the requestee every declaration,
/// allowance, payload, and forward names.
pub const REQUESTEE: &str = "lid-rs";

/// The closed set of tools this program defines: the model sees their
/// names, schemas, and descriptions through the session's policy and calls
/// them through canopy's invoke choreography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub enum Tool {
    /// A file's text with line numbers, or a directory's entries.
    Read,
    /// A literal, case-sensitive substring search.
    Grep,
    /// The paths under the root matching a glob.
    Glob,
    /// One exact replacement, or all of them, in an existing file.
    Edit,
    /// A file created or replaced whole.
    Write,
}

impl Tool {
    /// A forward's `op` — `read`, `grep`, `glob`, `edit`, `write` —
    /// classified into the closed set; an unknown `op` is none.
    #[implements(spec::AForwardsOpClassifiesToItsToolOrToNone)]
    pub fn of(op: &str) -> Option<Tool> {
        match op {
            "read" => Some(Tool::Read),
            "grep" => Some(Tool::Grep),
            "glob" => Some(Tool::Glob),
            "edit" => Some(Tool::Edit),
            "write" => Some(Tool::Write),
            _ => None,
        }
    }

    /// The tool's `op` on the wire — `read`, `grep`, `glob`, `edit`,
    /// `write` — the name [`Tool::of`] classifies from.
    #[implements(spec::AForwardsOpClassifiesToItsToolOrToNone)]
    pub fn op(self) -> &'static str {
        match self {
            Tool::Read => "read",
            Tool::Grep => "grep",
            Tool::Glob => "glob",
            Tool::Edit => "edit",
            Tool::Write => "write",
        }
    }

    /// The name the phase library's verdict knows the tool by — `Read`,
    /// `Grep`, `Glob`, `Edit`, `Write` — as `crate::phase::policy::kind_of`
    /// recognises it.
    #[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict)]
    pub fn hook_name(self) -> &'static str {
        match self {
            Tool::Read => "Read",
            Tool::Grep => "Grep",
            Tool::Glob => "Glob",
            Tool::Edit => "Edit",
            Tool::Write => "Write",
        }
    }
}

/// A tool's answer: `Ok` lands as a `success` outcome carrying the text,
/// `Err` as an `error` carrying the message.
pub type ToolResult = Result<String, String>;

/// `read`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReadArgs {
    /// The file or directory, relative to the workspace root.
    pub path: String,
    /// The line to start from, when not the first.
    pub offset: Option<usize>,
    /// How many lines at most, when not all.
    pub limit: Option<usize>,
}

/// `grep`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GrepArgs {
    /// The literal to find.
    pub pattern: String,
    /// The directory to search under, relative to the root; the root when absent.
    pub path: Option<String>,
    /// A glob narrowing which files are searched.
    pub glob: Option<String>,
}

/// `glob`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GlobArgs {
    /// The glob, relative to the root.
    pub pattern: String,
}

/// `edit`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EditArgs {
    /// The existing file, relative to the root.
    pub path: String,
    /// The text to find.
    pub old_string: String,
    /// What replaces it.
    pub new_string: String,
    /// Replace every occurrence rather than requiring exactly one.
    #[serde(default)]
    pub replace_all: bool,
}

/// How many occurrences an `edit` replaces: the closed set the wire's
/// `replace_all` classifies into, once, at the boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replace {
    /// Exactly the one occurrence there is.
    One,
    /// Every occurrence, of which there must be at least one.
    All,
}

impl EditArgs {
    /// `replace_all` classified — the one decision over the wire's flag:
    /// [`Replace::All`] when set, else [`Replace::One`].
    #[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
    pub fn replace(&self) -> Replace {
        if self.replace_all { Replace::All } else { Replace::One }
    }
}

/// `write`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WriteArgs {
    /// The file, relative to the root.
    pub path: String,
    /// Its whole content.
    pub content: String,
}

/// The declarations of `tools` for the requestee `lid-rs`, one
/// [`declaration`] each.
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn declarations(tools: &[Tool]) -> Vec<ToolDecl> {
    tools.iter().map(|tool| declaration(*tool)).collect()
}

/// One tool as the policy declares it: its name — which the model sees and
/// calls it by, and which here is the tool's own `op` — the requestee
/// `lid-rs`, that `op`, and the JSON schema of its arguments
/// ([`schema_of`]).
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn declaration(tool: Tool) -> ToolDecl {
    ToolDecl { name: tool.op().to_string(), requestee: REQUESTEE.to_string(), op: tool.op().to_string(), schema: schema_of(tool) }
}

/// The JSON schema of a tool's arguments, the tool's description in the
/// schema's `description`: `path` with optional `offset` and `limit`;
/// `pattern` with optional `path` and `glob`; `pattern`; `path`,
/// `old_string`, `new_string`, optional `replace_all`; `path`, `content`.
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn schema_of(tool: Tool) -> Value {
    let string = json!({ "type": "string" });
    let count = json!({ "type": "integer" });
    match tool {
        Tool::Read => json!({ "type": "object", "description": "Read a file's text with line numbers, or list a directory's entries. Every path is relative to the workspace root.", "properties": { "path": string, "offset": count.clone(), "limit": count }, "required": ["path"] }),
        Tool::Grep => json!({ "type": "object", "description": "Search files for a literal, case-sensitive substring, one `path:line: text` per match, at most 200 lines.", "properties": { "pattern": string.clone(), "path": string.clone(), "glob": string }, "required": ["pattern"] }),
        Tool::Glob => json!({ "type": "object", "description": "List the paths under the workspace root matching a glob, sorted.", "properties": { "pattern": string }, "required": ["pattern"] }),
        Tool::Edit => json!({ "type": "object", "description": "Replace one exact occurrence of `old_string` in an existing file, or every occurrence when `replace_all` is set. Returns clippy's verdict.", "properties": { "path": string.clone(), "old_string": string.clone(), "new_string": string, "replace_all": json!({ "type": "boolean" }) }, "required": ["path", "old_string", "new_string"] }),
        Tool::Write => json!({ "type": "object", "description": "Create a file, or replace its content whole. Returns clippy's verdict.", "properties": { "path": string.clone(), "content": string }, "required": ["path", "content"] }),
    }
}

/// The workspace boundary every tool applies before any verdict is asked.
/// `root` is the workspace root as the caller holds it — the path cargo
/// reports, never canonicalised — and `path` is relative to it. A path
/// that is absolute or climbs with `..` is refused as written
/// ([`relative_only`]); the rest is judged in its canonical form
/// ([`resolved`], then [`within`]), and one that resolves outside the root
/// through a symlink is refused. What comes back is `root.join(path)`: the
/// caller's root with the path under it, in the caller's form — the
/// canonical form serves the judgment and goes no further, so the path
/// every verdict and every tool then sees is on the same root the phase
/// library's policy strips it against.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn confine(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let relative = relative_only(path)?;
    within(root, &resolved(root, relative)?)?;
    Ok(root.join(relative))
}

/// A path as written must be relative and climb nowhere: an absolute path
/// or one with a `..` component is the refusal.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn relative_only(path: &Path) -> Result<&Path, String> {
    let climbs = path.components().any(|component| component == Component::ParentDir);
    (!path.is_absolute() && !climbs)
        .then_some(path)
        .ok_or_else(|| format!("`{}` is outside the workspace: a path is relative to the workspace root and may not climb", path.display()))
}

/// A relative path in its canonical form under the root, symlinks
/// followed: `root.join(relative)`'s deepest ancestor that exists,
/// canonicalised, joined with what remains — so a file about to be written
/// resolves as its directory does, and a symlink out of the root resolves
/// to where it points. `root` is the caller's, in any form; the answer is
/// canonical (`/private/var/…` for `/var/…` on macOS) and is for
/// [`within`] to judge, never for a tool to open or a verdict to strip.
///
/// The deepest ancestor is the deepest that exists *as an entry*, which is
/// what `symlink_metadata` answers and `Path::exists` does not: `exists`
/// follows the link and calls a dangling one absent, so `root/x.rs`
/// pointing at `/outside/new.rs` would resolve as a file not yet written
/// under the root and `write` would create it where it points. Asked about
/// the entry, that link is the deepest ancestor, and a link that leads
/// nowhere cannot be canonicalised: it is the refusal, while a link with a
/// target resolves to it and is judged there.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn resolved(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let full = root.join(relative);
    let deepest = full.ancestors().find(|ancestor| ancestor.symlink_metadata().is_ok()).ok_or_else(|| format!("`{}` has no ancestor that exists", full.display()))?;
    let rest = full.strip_prefix(deepest).map_err(|_| format!("`{}` is not under `{}`", full.display(), deepest.display()))?;
    let canonical = deepest.canonicalize().map_err(|e| format!("`{}` does not resolve ({e}): a link that leads nowhere may lead out of the workspace root", deepest.display()))?;
    Ok(canonical.join(rest))
}

/// The judgment, and nothing else: `resolved`, in the canonical form
/// [`resolved`] produces, must lie under the canonical form of `root` —
/// the caller's root, canonicalised here so canonical is compared with
/// canonical. One that does not is the refusal; one that does is `Ok(())`,
/// and the caller keeps the path it already holds on its own root.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn within(root: &Path, resolved: &Path) -> Result<(), String> {
    let canonical = root.canonicalize().map_err(|e| format!("resolving the workspace root `{}`: {e}", root.display()))?;
    resolved
        .starts_with(&canonical)
        .then_some(())
        .ok_or_else(|| format!("`{}` resolves outside the workspace root `{}`", resolved.display(), canonical.display()))
}

/// One forwarded call, and this host's executor: the function
/// `super::ending::turn` hands each [`super::turn::drive`] as its
/// [`super::turn::Executor`], so a host with tools of its own runs those
/// through the same turn rather than these. The `op` must be a tool the
/// session's declarations carry ([`declared`]) — an edit forwarded to the
/// reviewer is refused here, whatever the authorizer did — then dispatched
/// over [`Tool`] to its call, each of which confines the path, asks the
/// phase library's pre-tool verdict, and does the work. The root every call
/// confines against is the workspace root as cargo reports it,
/// uncanonicalised: the root the phase library's policy strips an edit's
/// path against, and the root the slice's crate is under in the same form.
pub fn execute(project: &Project, session: &Session, op: &str, args: &Value) -> ToolResult {
    let root = project.root()?;
    match declared(session, op)? {
        Tool::Read => read_call(project, session, &root, args),
        Tool::Grep => grep_call(project, session, &root, args),
        Tool::Glob => glob_call(project, session, &root, args),
        Tool::Edit => edit_call(project, session, &root, args),
        Tool::Write => write_call(project, session, &root, args),
    }
}

/// The tool a forward's `op` names, provided the session's own declarations
/// carry it ([`Session::declares`]) — the set its host dialled, not one
/// fixed for this host's five; an unknown `op`, or one those declarations do
/// not name — an `edit` forwarded to the reviewer — is the refusal, and
/// nothing runs.
#[implements(spec::AnEditForwardedToTheReviewerIsRefusedHere, spec::ASessionCarriesTheDeclarationsItsHostDialled)]
pub fn declared(session: &Session, op: &str) -> Result<Tool, String> {
    Tool::of(op)
        .filter(|_| session.declares(op))
        .ok_or_else(|| format!("`{op}` is not a tool this session's policy declared"))
}

/// A payload's `args` as a tool's typed arguments; what does not fit is
/// the error.
pub fn arguments<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    serde_json::from_value(args.clone()).map_err(|e| format!("the call's arguments do not fit the tool: {e}"))
}

/// The phase library's input for one call: the session's agent id
/// (`canopy:<session>`), the tool's hook name, and for an edit its
/// confined path — on the workspace root as cargo reports it, as
/// [`confine`] returns it, since that is the root the policy strips it
/// against.
pub fn tool_input(session: &Session, tool: Tool, path: Option<&Path>) -> HookInput {
    HookInput {
        agent_id: session.agent_id(),
        tool_name: Some(tool.hook_name().to_string()),
        tool_path: path.map(Path::to_path_buf),
        ..HookInput::default()
    }
}

/// The pre-tool verdict for one call — the one decision over whether the
/// session carries a phase: one that does is judged by the phase library
/// ([`judged`]); one that does not asks no verdict and does its work, there
/// being no phase whose policy could bound it and no tally for the call to
/// belong to.
#[implements(spec::ASessionWithoutAPhaseAsksNoPreToolVerdict, spec::ASessionWithoutAPhaseKeepsNoTally)]
pub fn verdict(project: &Project, session: &Session, tool: Tool, path: Option<&Path>) -> Result<(), String> {
    match session.phase {
        Some(phase) => judged(project, session, phase, tool, path),
        None => Ok(()),
    }
}

/// One call judged by the phase library, asked as
/// [`crate::phase::hook_pre_tool`] for `phase` with the session's agent id:
/// an observation is tallied and never refused; an edit or write is
/// allowed, or refused with the verdict's wording as the error, before any
/// file is touched.
#[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict, spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched)]
pub fn judged(project: &Project, session: &Session, phase: Phase, tool: Tool, path: Option<&Path>) -> Result<(), String> {
    match hook_pre_tool(project, phase, &tool_input(session, tool, path))? {
        HookVerdict::Allow => Ok(()),
        HookVerdict::Refuse(reason) | HookVerdict::Context(reason) => Err(reason),
    }
}

/// What an allowed edit or write answers with — the one decision over
/// whether the session carries a phase: one that does answers with the
/// post-edit verdict's text ([`post_edit`]); one that does not asks for no
/// check, since the check the phase library runs writes the tally, and a
/// session with no phase has none for it to be written under.
///
/// What it answers instead is that the change was made and nothing was
/// checked. A phaseless session's edits are unmeasured by design (the LLD's
/// Security posture), and clippy's verdict is not reachable without a
/// phase: the phase library's check and its tally are one call. Saying so
/// is what keeps that plain to whoever reads the answer — an empty result
/// would be indistinguishable from a clean workspace, which is the one
/// thing this answer cannot claim.
#[implements(spec::AnAllowedEditReturnsThePostEditVerdictsText, spec::ASessionWithoutAPhaseKeepsNoTally)]
pub fn checked(project: &Project, session: &Session, tool: Tool, path: &Path) -> ToolResult {
    match session.phase {
        Some(_) => post_edit(project, session, tool, path),
        None => Ok(UNCHECKED.to_string()),
    }
}

/// What an edit or write answers with in a session carrying no phase: the
/// change is on disk and no check was run, so nothing in the answer says
/// the workspace still compiles.
pub const UNCHECKED: &str = "the change was made and nothing was checked: this session carries no phase to run the check under, so nothing here says the workspace still compiles";

/// The post-edit verdict after an allowed edit or write, asked as
/// [`crate::phase::hook_post_edit`] with the session's agent id: its text
/// — clippy's diagnostics, or "clean" — is the tool's result, and the check
/// is tallied under the phase the session carries.
#[implements(spec::AnAllowedEditReturnsThePostEditVerdictsText)]
pub fn post_edit(project: &Project, session: &Session, tool: Tool, path: &Path) -> ToolResult {
    match hook_post_edit(project, &tool_input(session, tool, Some(path)))? {
        HookVerdict::Allow => Ok(String::new()),
        HookVerdict::Refuse(text) | HookVerdict::Context(text) => Ok(text),
    }
}

/// `read` as forwarded: its arguments, the path confined, the verdict as
/// `Read`, then the work.
#[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict)]
fn read_call(project: &Project, session: &Session, root: &Path, args: &Value) -> ToolResult {
    let args: ReadArgs = arguments(args)?;
    let path = confine(root, Path::new(&args.path))?;
    verdict(project, session, Tool::Read, None)?;
    read_tool(&path, &args)
}

/// `grep` as forwarded: its arguments, the directory it searches
/// ([`search_dir`]) confined, the verdict as `Grep`, then the work.
#[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict)]
fn grep_call(project: &Project, session: &Session, root: &Path, args: &Value) -> ToolResult {
    let args: GrepArgs = arguments(args)?;
    let under = confine(root, search_dir(args.path.as_deref()))?;
    verdict(project, session, Tool::Grep, None)?;
    grep_tool(root, &under, &args)
}

/// The directory `grep` searches, relative to the root: `path` when one is
/// given, else the root itself — the path that names the root relative to
/// itself, for [`confine`] to take like any other.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn search_dir(path: Option<&str>) -> &Path {
    Path::new(path.unwrap_or(""))
}

/// `glob` as forwarded: its arguments; the whole pattern judged as the path
/// it is ([`pattern_components_ok`]), then its literal prefix
/// ([`literal_prefix`]) confined — both before any verdict is asked, as
/// every tool's path is; the verdict as `Glob`; then the work under the
/// root.
#[implements(
    spec::EveryToolConfinesItsPathToTheWorkspace,
    spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot,
    spec::ObservationsAreTalliedThroughThePreToolVerdict,
)]
fn glob_call(project: &Project, session: &Session, root: &Path, args: &Value) -> ToolResult {
    let args: GlobArgs = arguments(args)?;
    pattern_components_ok(&args.pattern)?;
    confine(root, literal_prefix(&args.pattern))?;
    verdict(project, session, Tool::Glob, None)?;
    glob_tool(root, &args)
}

/// The whole glob pattern judged as the path it is, before its literal
/// prefix is confined and before any verdict: a pattern that is absolute,
/// or has a `..` component anywhere in it — `*/../../Cargo.toml` climbs
/// past what its prefix says — is the refusal.
#[implements(spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
pub fn pattern_components_ok(pattern: &str) -> Result<(), String> {
    let whole = Path::new(pattern);
    let climbs = whole.components().any(|component| component == Component::ParentDir);
    (!whole.is_absolute() && !climbs)
        .then_some(())
        .ok_or_else(|| format!("`{pattern}` is outside the workspace: a glob is relative to the workspace root and may not climb"))
}

/// The path a glob pattern names literally: its text before the first
/// metacharacter — `*`, `?`, or `[` — which is what [`confine`] judges,
/// since every match lies under it. A pattern that begins with a
/// metacharacter has an empty prefix: the root itself.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn literal_prefix(pattern: &str) -> &Path {
    Path::new(pattern.split(['*', '?', '[']).next().unwrap_or(pattern))
}

/// `edit` as forwarded: its arguments, the path confined, the verdict as
/// `Edit` on that path — a refusal returns before the file is touched —
/// then the work, and the post-edit verdict's text as the result.
#[implements(spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched, spec::AnAllowedEditReturnsThePostEditVerdictsText)]
fn edit_call(project: &Project, session: &Session, root: &Path, args: &Value) -> ToolResult {
    let args: EditArgs = arguments(args)?;
    let path = confine(root, Path::new(&args.path))?;
    verdict(project, session, Tool::Edit, Some(&path))?;
    edit_tool(&path, &args)?;
    checked(project, session, Tool::Edit, &path)
}

/// `write` as forwarded: its arguments, the path confined, the verdict as
/// `Write` on that path — a refusal returns before the file is touched —
/// then the work, and the post-edit verdict's text as the result.
#[implements(spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched, spec::AnAllowedEditReturnsThePostEditVerdictsText)]
fn write_call(project: &Project, session: &Session, root: &Path, args: &Value) -> ToolResult {
    let args: WriteArgs = arguments(args)?;
    let path = confine(root, Path::new(&args.path))?;
    verdict(project, session, Tool::Write, Some(&path))?;
    write_tool(&path, &args)?;
    checked(project, session, Tool::Write, &path)
}

/// `read` over a confined path — the one decision over what the path is: a
/// directory's entries ([`directory_listing`]) or a file's numbered lines
/// ([`numbered_lines`]).
#[implements(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
pub fn read_tool(path: &Path, args: &ReadArgs) -> ToolResult {
    if path.is_dir() { directory_listing(path) } else { numbered_lines(path, args.offset, args.limit) }
}

/// A directory's entries, one name per line, sorted; a directory that
/// cannot be read is the error.
#[implements(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
pub fn directory_listing(path: &Path) -> ToolResult {
    let listed = std::fs::read_dir(path).map_err(|e| format!("listing `{}`: {e}", path.display()))?;
    let mut names: Vec<String> = listed.filter_map(Result::ok).map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    names.sort();
    Ok(names.join("\n"))
}

/// A file's lines, each prefixed with its number counted from 1, from
/// `offset` (the first line when absent) for at most `limit` lines (all of
/// them when absent); a file that cannot be read as text is the error.
#[implements(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
pub fn numbered_lines(path: &Path, offset: Option<usize>, limit: Option<usize>) -> ToolResult {
    let text = std::fs::read_to_string(path).map_err(|e| format!("reading `{}` as text: {e}", path.display()))?;
    let from = offset.unwrap_or(1).max(1);
    let window = text.lines().enumerate().skip(from - 1).take(limit.unwrap_or(usize::MAX));
    Ok(window.map(|(at, line)| format!("{}\t{line}", at + 1)).collect::<Vec<String>>().join("\n"))
}

/// At most this many `path:line: text` lines from one `grep`.
pub const GREP_CAP: usize = 200;

/// `grep` under a confined directory — `under` on the caller's `root`, as
/// [`confine`] returns it, so the two share one form and every file walked
/// is relative to `root` by prefix: the files under it ([`files_under`])
/// narrowed by the glob ([`narrowed`]), each searched for the pattern
/// ([`matches_in`]), the matches in file order capped ([`capped`]), one per
/// line.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn grep_tool(root: &Path, under: &Path, args: &GrepArgs) -> ToolResult {
    let files = narrowed(root, files_under(under)?, args.glob.as_deref())?;
    let lines = capped(files.iter().flat_map(|file| matches_in(root, file, &args.pattern)));
    Ok(lines.join("\n"))
}

/// The first [`GREP_CAP`] of the matches, in their order; a search past the
/// cap is not read further.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn capped(matches: impl Iterator<Item = String>) -> Vec<String> {
    matches.take(GREP_CAP).collect()
}

/// The two directory names neither `grep` nor `glob` ever enters: build
/// output and git's objects, which no search of the source wants.
pub const SKIPPED: [&str; 2] = ["target", ".git"];

/// Whether one path component names a directory the walks never enter —
/// one of [`SKIPPED`]. The one predicate both tools' walks ask, of every
/// directory before descending into it.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
pub fn skipped(component: &OsStr) -> bool {
    SKIPPED.iter().any(|name| component == OsStr::new(name))
}

/// Every file under a directory, at any depth, in path order, never
/// entering a directory [`skipped`] names and never entering a symlinked
/// directory — a link out of the root would let `grep` read and transmit
/// what confinement refuses — the walk `grep` makes under its directory
/// and `glob` under the root; a directory that cannot be walked is the
/// error. The files are `dir` joined with what the walk found, in `dir`'s
/// own form.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
pub fn files_under(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut pending = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(next) = pending.pop() {
        let listed = std::fs::read_dir(&next).map_err(|e| format!("walking `{}`: {e}", next.display()))?;
        let entries: Vec<(PathBuf, std::fs::FileType)> =
            listed.filter_map(Result::ok).filter_map(|entry| entry.file_type().ok().map(|kind| (entry.path(), kind))).collect();
        pending.extend(entries.iter().filter(|(path, kind)| kind.is_dir() && !path.file_name().is_some_and(skipped)).map(|(path, _)| path.clone()));
        files.extend(entries.iter().filter(|(_, kind)| kind.is_file()).map(|(path, _)| path.clone()));
    }
    files.sort();
    Ok(files)
}

/// The files whose path relative to `root` matches the glob — all of them
/// when there is none; a glob that does not parse is the error. What
/// narrows `grep`'s files, and what `glob`'s pattern selects from the
/// walk.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
pub fn narrowed(root: &Path, files: Vec<PathBuf>, glob: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let pattern = glob.map(Pattern::new).transpose().map_err(|e| format!("`{}` is not a glob: {e}", glob.unwrap_or_default()))?;
    let kept = files.into_iter().filter(|file| {
        pattern.as_ref().is_none_or(|glob| file.strip_prefix(root).is_ok_and(|relative| glob.matches_path(relative)))
    });
    Ok(kept.collect())
}

/// The lines of one file that contain the pattern as a literal,
/// case-sensitive substring, each as `path:line: text` with the path
/// relative to `root` — `file` is on `root` in the same form, as the walk
/// yields it, so the relation is by prefix — and the line counted from 1;
/// a file that is not text yields none.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn matches_in(root: &Path, file: &Path, pattern: &str) -> Vec<String> {
    let shown = file.strip_prefix(root).unwrap_or(file).display().to_string();
    let found = |text: String| -> Vec<String> {
        text.lines().enumerate().filter(|(_, line)| line.contains(pattern)).map(|(at, line)| format!("{shown}:{}: {line}", at + 1)).collect()
    };
    std::fs::read_to_string(file).map(found).unwrap_or_default()
}

/// `glob` under the root, whose pattern [`glob_call`] has already judged
/// whole and by its literal prefix: the files under the root
/// ([`files_under`] — the walk, which never enters a directory
/// [`skipped`] names or a symlinked one) narrowed to those the pattern matches
/// ([`narrowed`]), kept only where they resolve under the root
/// ([`under_root`]), and rendered relative to it, sorted
/// ([`relative_sorted`]). A pattern that does not parse is the error.
#[implements(spec::GlobReturnsMatchingPathsSorted, spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
pub fn glob_tool(root: &Path, args: &GlobArgs) -> ToolResult {
    let matches = under_root(root, narrowed(root, files_under(root)?, Some(&args.pattern))?);
    Ok(relative_sorted(root, &matches).join("\n"))
}

/// The matches that stay under the root: each, relative to `root`, resolved
/// ([`resolved`]) and judged ([`within`]), and kept as the walk yielded it
/// — on the caller's root, the canonical form having served the judgment
/// only — in their order; a match that resolves outside the root through a
/// symlink is omitted from the answer, not an error.
#[implements(spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
pub fn under_root(root: &Path, matches: Vec<PathBuf>) -> Vec<PathBuf> {
    matches
        .into_iter()
        .filter(|path| {
            path.strip_prefix(root).is_ok_and(|relative| resolved(root, relative).and_then(|full| within(root, &full)).is_ok())
        })
        .collect()
}

/// The paths relative to the root, as `glob` answers them, sorted.
#[implements(spec::GlobReturnsMatchingPathsSorted)]
pub fn relative_sorted(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    let mut shown: Vec<String> = paths.iter().map(|path| path.strip_prefix(root).unwrap_or(path).display().to_string()).collect();
    shown.sort();
    shown
}

/// `edit` over a confined existing file: its text read
/// ([`existing_text`]), the occurrences of `old_string` counted and judged
/// against the request ([`replaceable`]) — an error there changes nothing
/// — then every occurrence replaced ([`replaced`]) and the file written
/// back ([`written`]).
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn edit_tool(path: &Path, args: &EditArgs) -> Result<(), String> {
    let text = existing_text(path)?;
    replaceable(text.matches(&args.old_string).count(), args.replace())?;
    written(path, &replaced(&text, &args.old_string, &args.new_string))
}

/// The text of the file an `edit` is over; one that does not exist, or is
/// not text, is the error naming the path.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn existing_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("`{}` cannot be edited: {e}", path.display()))
}

/// The text with every occurrence of `old` replaced by `new` — the one
/// occurrence there is, or all of them, as [`replaceable`] has admitted.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn replaced(text: &str, old: &str, new: &str) -> String {
    text.replace(old, new)
}

/// The edited text written back over the file; a write that fails is the
/// error naming the path.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn written(path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| format!("writing `{}`: {e}", path.display()))
}

/// Whether the occurrences found may be replaced: exactly one for
/// [`Replace::One`], any number above zero for [`Replace::All`]; zero, or
/// more than one for [`Replace::One`], is the error naming the count.
#[implements(spec::AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount)]
pub fn replaceable(count: usize, replace: Replace) -> Result<(), String> {
    match (count, replace) {
        (1, Replace::One) => Ok(()),
        (0, Replace::One | Replace::All) => Err("`old_string` matches 0 places in the file; nothing was changed".to_string()),
        (_, Replace::All) => Ok(()),
        (_, Replace::One) => Err(format!("`old_string` matches {count} places in the file; give a longer string, or set replace_all")),
    }
}

/// `write` over a confined path: the file created, or its content replaced
/// whole.
#[implements(spec::WriteCreatesOrReplacesTheFileWhole)]
pub fn write_tool(path: &Path, args: &WriteArgs) -> Result<(), String> {
    std::fs::write(path, &args.content).map_err(|e| format!("writing `{}` whole: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use lid_rs::validates;
    use serde_json::json;

    use super::super::door::Door;
    use super::super::ending::WORKER_TOOLS;
    use super::super::replay::{self, mentions, strings};
    use super::super::review::REVIEW_TOOLS;
    use super::*;
    use crate::phase::Phase;
    use crate::phase::fixture;
    use crate::phase::tally::{self, Tally};

    /// A fresh scratch directory.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-canopy-tools").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// A file under `root`, its directories made.
    fn file(root: &Path, relative: &str, text: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("dirs");
        std::fs::write(path, text).expect("write");
    }

    /// The text of a file under `root`.
    fn text(root: &Path, relative: &str) -> String {
        std::fs::read_to_string(root.join(relative)).expect("read")
    }

    /// A tree to search: files at three depths, plus `target/` and `.git/`.
    fn search_tree(name: &str) -> PathBuf {
        let root = scratch(name);
        file(&root, "a.rs", "fn main() {}\n// TODO one\n");
        file(&root, "src/z.rs", "// TODO two\nlet todo = 1;\n");
        file(&root, "sub/b.txt", "plain\nTODO three\n");
        file(&root, "target/c.rs", "// TODO built\n");
        file(&root, ".git/d", "TODO object\n");
        root
    }

    /// The files the walk yields under `root`, relative to it.
    fn walked(root: &Path) -> Vec<PathBuf> {
        files_under(root).expect("walk").iter().map(|f| f.strip_prefix(root).expect("under the root").to_path_buf()).collect()
    }

    fn grep_args(pattern: &str, path: Option<&str>, glob: Option<&str>) -> GrepArgs {
        GrepArgs { pattern: pattern.to_string(), path: path.map(str::to_string), glob: glob.map(str::to_string) }
    }

    fn glob_args(pattern: &str) -> GlobArgs {
        GlobArgs { pattern: pattern.to_string() }
    }

    /// `edit`'s arguments, `replace_all` set for [`Replace::All`].
    fn edit_args(path: &str, old: &str, new: &str, replace: Replace) -> EditArgs {
        EditArgs { path: path.to_string(), old_string: old.to_string(), new_string: new.to_string(), replace_all: replace == Replace::All }
    }

    fn read_args(path: &str, offset: Option<usize>, limit: Option<usize>) -> ReadArgs {
        ReadArgs { path: path.to_string(), offset, limit }
    }

    /// Each numbered line as `number:text`: the leading digits, then the
    /// rest past its separator, whatever the separator is.
    fn numbered(output: &str) -> Vec<String> {
        output
            .lines()
            .map(|line| {
                let trimmed = line.trim_start();
                let digits: String = trimmed.chars().take_while(char::is_ascii_digit).collect();
                format!("{digits}:{}", trimmed[digits.len()..].trim_start_matches([':', '\t', ' ', '|']))
            })
            .collect()
    }

    /// A session over no door carrying a phase, for the tools alone: the
    /// tally key is `canopy:<id>`, and its declarations are `tools`.
    fn session(id: &str, phase: Phase, tools: &[Tool]) -> Session {
        replay::session(Door::new("http://127.0.0.1:1", "k"), id, Some(phase), declarations(tools))
    }

    #[test]
    #[validates(spec::AForwardsOpClassifiesToItsToolOrToNone)]
    fn a_forwards_op_classifies_to_its_tool_or_to_none() {
        let mapped: Vec<Option<Tool>> = ["read", "grep", "glob", "edit", "write", "bash", "Read", ""].iter().map(|op| Tool::of(op)).collect();
        assert_eq!(mapped, [Some(Tool::Read), Some(Tool::Grep), Some(Tool::Glob), Some(Tool::Edit), Some(Tool::Write), None, None, None]);
        let round_trip: Vec<Option<Tool>> = WORKER_TOOLS.iter().map(|tool| Tool::of(tool.op())).collect();
        assert_eq!(round_trip, WORKER_TOOLS.map(Some));
    }

    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn a_path_that_climbs_or_is_absolute_is_refused_as_written() {
        let root = scratch("confine-written");
        file(&root, "src/hello.rs", "");
        assert_eq!(confine(&root, Path::new("src/hello.rs")).expect("inside"), root.join("src/hello.rs"));
        let absolute = root.join("src/hello.rs").display().to_string();
        let escaping = ["../etc/passwd", "src/../../x", "/etc/passwd", absolute.as_str()];
        let refused = escaping.iter().all(|p| confine(&root, Path::new(p)).is_err() && relative_only(Path::new(p)).is_err());
        assert!(refused, "each of {escaping:?} is refused as written");
        assert_eq!(relative_only(Path::new("src/hello.rs")).expect("relative"), Path::new("src/hello.rs"));
    }

    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn a_path_is_judged_in_its_canonical_form_under_the_root() {
        let root = scratch("confine-resolved");
        file(&root, "src/hello.rs", "");
        let canonical = root.canonicalize().expect("root");
        assert_eq!(resolved(&root, Path::new("src/hello.rs")).expect("exists"), canonical.join("src/hello.rs"));
        assert_eq!(resolved(&root, Path::new("src/new.rs")).expect("a file about to be written resolves as its directory does"), canonical.join("src/new.rs"));
        within(&root, &canonical.join("src/hello.rs")).expect("under the root");
        assert!(within(&root, &canonical.parent().expect("parent").join("elsewhere.rs")).is_err(), "beside the root");
    }

    #[cfg(unix)]
    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn a_path_through_a_symlink_out_of_the_root_is_refused() {
        let root = scratch("confine-symlink");
        let outside = scratch("confine-symlink-outside");
        file(&outside, "secret.txt", "s");
        file(&root, "inside.txt", "i");
        std::os::unix::fs::symlink(&outside, root.join("link")).expect("dir link");
        std::os::unix::fs::symlink(outside.join("secret.txt"), root.join("leak.txt")).expect("file link");
        std::os::unix::fs::symlink(root.join("inside.txt"), root.join("alias.txt")).expect("inner link");
        assert!(confine(&root, Path::new("link/secret.txt")).is_err(), "through a linked directory");
        assert!(confine(&root, Path::new("leak.txt")).is_err(), "a linked file");
        assert_eq!(confine(&root, Path::new("alias.txt")).expect("a link within the root stays within it"), root.join("alias.txt"));
    }

    #[cfg(unix)]
    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn a_dangling_symlink_out_of_the_root_is_refused_and_creates_nothing_outside() {
        let (dir, project) = fixture::copy("canopy-tools-dangling");
        let outside = scratch("canopy-tools-dangling-outside");
        let target = outside.join("new.rs");
        std::os::unix::fs::symlink(&target, dir.join("src/leak.rs")).expect("a link to a file that does not exist yet");
        assert!(confine(&dir, Path::new("src/leak.rs")).is_err(), "a link that leads nowhere is no file about to be written under the root");
        let session = session("dangling", Phase::Three, &WORKER_TOOLS);
        let refused = execute(&project, &session, "write", &json!({ "path": "src/leak.rs", "content": "//! written through the link\n" })).expect_err("refused");
        mentions(&refused, &["src/leak.rs"]);
        assert!(!target.exists(), "nothing is created outside the root at {}", target.display());
        assert_eq!(tally::load(&project, "canopy:dangling").expect("tally"), Tally::default(), "no verdict was asked");
    }

    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn every_tool_refuses_an_escaping_path_before_any_verdict() {
        let (_dir, project) = fixture::copy("canopy-tools-confine");
        let session = session("confine", Phase::Three, &WORKER_TOOLS);
        let calls = [
            ("read", json!({ "path": "../Cargo.toml" })),
            ("grep", json!({ "pattern": "x", "path": "/etc" })),
            ("glob", json!({ "pattern": "../**/*.rs" })),
            ("edit", json!({ "path": "/etc/hosts", "old_string": "a", "new_string": "b" })),
            ("write", json!({ "path": "../x.rs", "content": "" })),
        ];
        let refused = calls.iter().all(|(op, args)| execute(&project, &session, op, args).is_err());
        assert!(refused, "every tool refuses");
        assert_eq!(tally::load(&project, "canopy:confine").expect("tally"), Tally::default(), "no verdict was asked");
    }

    #[test]
    #[validates(spec::EveryToolConfinesItsPathToTheWorkspace)]
    fn a_glob_patterns_literal_prefix_is_what_confinement_judges() {
        assert_eq!(literal_prefix("src/**/*.rs"), Path::new("src"));
        assert_eq!(literal_prefix("docs/intent/?.md"), Path::new("docs/intent"));
        assert_eq!((literal_prefix("*.rs"), literal_prefix("[ab].rs"), literal_prefix("Cargo.toml")), (Path::new(""), Path::new(""), Path::new("Cargo.toml")));
    }

    #[test]
    #[validates(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
    fn read_returns_numbered_lines_from_offset_for_limit() {
        let root = scratch("read-lines");
        file(&root, "poem.txt", "alpha\nbeta\ngamma\ndelta\n");
        let whole = read_tool(&root.join("poem.txt"), &read_args("poem.txt", None, None)).expect("read");
        assert_eq!(numbered(&whole), strings(&["1:alpha", "2:beta", "3:gamma", "4:delta"]));
        let window = numbered_lines(&root.join("poem.txt"), Some(2), Some(2)).expect("read");
        assert_eq!(numbered(&window), strings(&["2:beta", "3:gamma"]));
        assert!(numbered_lines(&root.join("missing.txt"), None, None).is_err());
    }

    #[test]
    #[validates(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
    fn read_returns_a_directorys_entries() {
        let root = scratch("read-dir");
        file(&root, "src/b.rs", "");
        file(&root, "src/a.rs", "");
        file(&root, "src/sub/c.rs", "");
        let listing = read_tool(&root.join("src"), &read_args("src", None, None)).expect("listed");
        let entries: Vec<&str> = listing.lines().map(str::trim).collect();
        assert_eq!(entries.len(), 3, "{listing}");
        assert!(entries[0].starts_with("a.rs") && entries[1].starts_with("b.rs") && entries[2].starts_with("sub"), "sorted: {listing}");
        assert_eq!(directory_listing(&root.join("src")).expect("listed"), listing);
    }

    #[test]
    #[validates(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
    fn grep_is_a_literal_case_sensitive_substring_search_skipping_target_and_git() {
        let root = search_tree("grep-literal");
        let out = grep_tool(&root, &root, &grep_args("TODO", None, None)).expect("grep");
        assert_eq!(out.lines().collect::<Vec<_>>(), ["a.rs:2: // TODO one", "src/z.rs:1: // TODO two", "sub/b.txt:2: TODO three"]);
        assert_eq!(grep_tool(&root, &root, &grep_args("main(", None, None)).expect("grep"), "a.rs:1: fn main() {}", "a metacharacter is literal");
        assert_eq!(grep_tool(&root, &root, &grep_args("t.do", None, None)).expect("grep"), "", "`.` is no wildcard, and `todo` is not `TODO`");
    }

    #[test]
    #[validates(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
    fn grep_is_narrowed_by_path_and_glob() {
        let root = search_tree("grep-narrowed");
        let root_itself = search_dir(None);
        assert!(root_itself == Path::new("") || root_itself == Path::new("."), "{}", root_itself.display());
        assert_eq!(search_dir(Some("sub")), Path::new("sub"));
        let under = grep_tool(&root, &root.join("sub"), &grep_args("TODO", Some("sub"), None)).expect("grep");
        let by_glob = grep_tool(&root, &root, &grep_args("TODO", None, Some("**/*.rs"))).expect("grep");
        assert_eq!((under.as_str(), by_glob.lines().collect::<Vec<_>>()), ("sub/b.txt:2: TODO three", vec!["a.rs:2: // TODO one", "src/z.rs:1: // TODO two"]));
    }

    #[test]
    #[validates(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
    fn grep_returns_no_more_than_two_hundred_lines() {
        let root = scratch("grep-cap");
        file(&root, "many.txt", &"TODO\n".repeat(350));
        let out = grep_tool(&root, &root, &grep_args("TODO", None, None)).expect("grep");
        assert_eq!((GREP_CAP, out.lines().count()), (200, 200));
        assert_eq!(capped((0..5).map(|i| i.to_string())), strings(&["0", "1", "2", "3", "4"]));
        assert_eq!(matches_in(&root, &root.join("many.txt"), "TODO").len(), 350, "the file's every match; the cap is the search's");
    }

    #[test]
    #[validates(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
    fn the_walk_skips_target_and_git_alone() {
        assert_eq!(SKIPPED, ["target", ".git"]);
        let verdicts: Vec<bool> = ["target", ".git", "src", "targets", ".github", "docs"].iter().map(|c| skipped(OsStr::new(c))).collect();
        assert_eq!(verdicts, [true, true, false, false, false, false]);
        let root = search_tree("walk-skipped");
        assert_eq!(walked(&root), [PathBuf::from("a.rs"), PathBuf::from("src/z.rs"), PathBuf::from("sub/b.txt")]);
    }

    #[cfg(unix)]
    #[test]
    #[validates(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
    fn the_walk_never_enters_a_symlinked_directory_nor_yields_a_symlinked_file() {
        let root = search_tree("walk-symlinks");
        let outside = scratch("walk-symlinks-outside");
        file(&outside, "e.rs", "// TODO outside\n");
        std::os::unix::fs::symlink(&outside, root.join("link")).expect("dir link");
        std::os::unix::fs::symlink(outside.join("e.rs"), root.join("f.rs")).expect("file link");
        assert_eq!(walked(&root), [PathBuf::from("a.rs"), PathBuf::from("src/z.rs"), PathBuf::from("sub/b.txt")]);
        assert_eq!(grep_tool(&root, &root, &grep_args("outside", None, None)).expect("grep"), "");
        assert_eq!(glob_tool(&root, &glob_args("**/*.rs")).expect("glob"), "a.rs\nsrc/z.rs");
    }

    #[test]
    #[validates(spec::GlobReturnsMatchingPathsSorted)]
    fn glob_returns_matching_paths_sorted_never_under_target_or_git() {
        let root = search_tree("glob-sorted");
        file(&root, "src/a.rs", "");
        assert_eq!(glob_tool(&root, &glob_args("**/*.rs")).expect("glob"), "a.rs\nsrc/a.rs\nsrc/z.rs");
        assert_eq!(glob_tool(&root, &glob_args("src/*.rs")).expect("glob"), "src/a.rs\nsrc/z.rs");
        assert_eq!(relative_sorted(&root, &[root.join("src/z.rs"), root.join("a.rs")]), strings(&["a.rs", "src/z.rs"]));
    }

    #[test]
    #[validates(spec::GlobReturnsMatchingPathsSorted)]
    fn a_glob_that_does_not_parse_is_the_error() {
        let root = search_tree("glob-parse");
        assert!(glob_tool(&root, &glob_args("[")).is_err());
        assert!(narrowed(&root, vec![root.join("a.rs")], Some("[")).is_err());
        assert_eq!(narrowed(&root, vec![root.join("a.rs"), root.join("sub/b.txt")], None).expect("all"), [root.join("a.rs"), root.join("sub/b.txt")]);
    }

    #[test]
    #[validates(spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
    fn a_glob_pattern_that_is_absolute_or_climbs_is_refused_before_any_verdict() {
        let refused = ["/etc/*", "*/../../Cargo.toml", "../*.rs", "src/../../*.rs"].iter().all(|p| pattern_components_ok(p).is_err());
        assert!(refused, "absolute, or a `..` component anywhere");
        pattern_components_ok("src/**/*.rs").expect("a pattern under the root");
        pattern_components_ok("*.rs").expect("the root itself");
        let (_dir, project) = fixture::copy("canopy-glob-confined");
        let session = session("glob-confined", Phase::Three, &WORKER_TOOLS);
        execute(&project, &session, "glob", &json!({ "pattern": "*/../../Cargo.toml" })).expect_err("refused before the verdict");
        assert_eq!(tally::load(&project, "canopy:glob-confined").expect("tally").observations, 0, "no verdict was asked");
    }

    #[cfg(unix)]
    #[test]
    #[validates(spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
    fn a_match_that_resolves_outside_the_root_through_a_symlink_is_omitted() {
        let root = scratch("glob-under-root");
        let outside = scratch("glob-under-root-outside");
        file(&outside, "e.rs", "");
        file(&root, "a.rs", "");
        std::os::unix::fs::symlink(outside.join("e.rs"), root.join("leak.rs")).expect("file link");
        assert_eq!(under_root(&root, vec![root.join("leak.rs"), root.join("a.rs")]), [root.join("a.rs")]);
        assert_eq!(glob_tool(&root, &glob_args("*.rs")).expect("glob"), "a.rs");
    }

    #[test]
    #[validates(spec::AGlobPatternIsConfinedAndItsMatchesStayUnderTheRoot)]
    fn matches_under_the_root_are_kept_as_the_walk_yielded_them() {
        let root = scratch("glob-kept");
        file(&root, "a.rs", "");
        file(&root, "src/b.rs", "");
        assert_eq!(under_root(&root, vec![root.join("src/b.rs"), root.join("a.rs")]), [root.join("src/b.rs"), root.join("a.rs")]);
    }

    #[test]
    #[validates(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
    fn edit_replaces_the_one_occurrence_or_all_on_request() {
        let root = scratch("edit-replace");
        file(&root, "f.txt", "one two two three\n");
        edit_tool(&root.join("f.txt"), &edit_args("f.txt", "one", "1", Replace::One)).expect("one occurrence");
        assert_eq!(text(&root, "f.txt"), "1 two two three\n");
        edit_tool(&root.join("f.txt"), &edit_args("f.txt", "two", "2", Replace::All)).expect("all occurrences");
        assert_eq!(text(&root, "f.txt"), "1 2 2 three\n");
        let (one, all) = (edit_args("f.txt", "", "", Replace::One), edit_args("f.txt", "", "", Replace::All));
        assert_eq!((one.replace_all, one.replace(), all.replace_all, all.replace()), (false, Replace::One, true, Replace::All), "the wire's flag classified once");
    }

    #[test]
    #[validates(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
    fn the_edits_leaves_read_replace_and_write_back() {
        let root = scratch("edit-leaves");
        file(&root, "f.txt", "abc");
        assert_eq!(existing_text(&root.join("f.txt")).expect("text"), "abc");
        mentions(&existing_text(&root.join("nope.txt")).expect_err("missing"), &["nope.txt"]);
        assert_eq!(replaced("a-b-a", "a", "x"), "x-b-x");
        written(&root.join("f.txt"), "new").expect("written");
        assert_eq!(text(&root, "f.txt"), "new");
    }

    #[test]
    #[validates(spec::AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount)]
    fn an_ambiguous_or_absent_old_string_is_an_error_naming_the_count() {
        let root = scratch("edit-count");
        file(&root, "f.txt", "two two\n");
        mentions(&edit_tool(&root.join("f.txt"), &edit_args("f.txt", "two", "2", Replace::One)).expect_err("two places"), &["2"]);
        mentions(&edit_tool(&root.join("f.txt"), &edit_args("f.txt", "zero", "0", Replace::One)).expect_err("no place"), &["0"]);
        assert_eq!(text(&root, "f.txt"), "two two\n", "nothing changed");
        let verdicts = (replaceable(1, Replace::One).is_ok(), replaceable(3, Replace::All).is_ok(), replaceable(0, Replace::All).is_err(), replaceable(2, Replace::One).is_err());
        assert_eq!(verdicts, (true, true, true, true));
        mentions(&replaceable(2, Replace::One).expect_err("two"), &["2"]);
    }

    #[test]
    #[validates(spec::WriteCreatesOrReplacesTheFileWhole)]
    fn write_creates_or_replaces_the_file_whole() {
        let root = scratch("write-whole");
        write_tool(&root.join("new.txt"), &WriteArgs { path: "new.txt".to_string(), content: "created\n".to_string() }).expect("created");
        assert_eq!(text(&root, "new.txt"), "created\n");
        file(&root, "old.txt", "a much longer original content\n");
        write_tool(&root.join("old.txt"), &WriteArgs { path: "old.txt".to_string(), content: "short\n".to_string() }).expect("replaced");
        assert_eq!(text(&root, "old.txt"), "short\n");
    }

    #[test]
    #[validates(spec::ObservationsAreTalliedThroughThePreToolVerdict)]
    fn observations_are_tallied_through_the_pre_tool_verdict() {
        assert_eq!(WORKER_TOOLS.map(Tool::hook_name), ["Read", "Grep", "Glob", "Edit", "Write"]);
        let (_dir, project) = fixture::copy("canopy-tools-observe");
        let session = session("observe", Phase::Five, &WORKER_TOOLS);
        mentions(&execute(&project, &session, "read", &json!({ "path": "src/hello.rs" })).expect("read"), &["The hello slice"]);
        mentions(&execute(&project, &session, "grep", &json!({ "pattern": "fn greet" })).expect("grep"), &["src/hello.rs:"]);
        mentions(&execute(&project, &session, "glob", &json!({ "pattern": "src/*.rs" })).expect("glob"), &["src/hello.rs"]);
        let tally = tally::load(&project, "canopy:observe").expect("tally");
        assert_eq!((tally.observations, tally.edits, tally.policy_refusals), (3, 0, 0));
        verdict(&project, &session, Tool::Read, None).expect("an observation is never refused");
        assert_eq!(tally::load(&project, "canopy:observe").expect("tally").observations, 4);
    }

    #[test]
    #[validates(spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched)]
    fn a_refused_edit_is_the_tools_error_and_the_file_is_untouched() {
        let (dir, project) = fixture::copy("canopy-tools-refused");
        let session = session("refused", Phase::Five, &WORKER_TOOLS);
        let (manifest, lib) = (text(&dir, "Cargo.toml"), text(&dir, "src/lib.rs"));
        let edit = execute(&project, &session, "edit", &json!({ "path": "Cargo.toml", "old_string": "[package]", "new_string": "[pkg]" })).expect_err("refused");
        mentions(&edit, &["Cargo.toml", "Phase 5", "```stop"]);
        let write = execute(&project, &session, "write", &json!({ "path": "src/lib.rs", "content": "" })).expect_err("lib.rs is not Phase 5's");
        mentions(&write, &["src/lib.rs", "src/hello.rs"]);
        assert_eq!((text(&dir, "Cargo.toml"), text(&dir, "src/lib.rs")), (manifest, lib), "untouched");
        assert_eq!(tally::load(&project, "canopy:refused").expect("tally").policy_refusals, 2);
        let input = tool_input(&session, Tool::Edit, Some(&dir.join("Cargo.toml")));
        assert_eq!((input.agent_id.as_str(), input.tool_name.as_deref(), input.tool_path.as_deref()), ("canopy:refused", Some("Edit"), Some(dir.join("Cargo.toml").as_path())));
    }

    #[test]
    #[validates(spec::AnAllowedEditReturnsThePostEditVerdictsText)]
    fn an_allowed_edit_returns_the_post_edit_verdicts_text() {
        let (dir, project) = fixture::copy("canopy-tools-allowed");
        let session = session("allowed", Phase::Three, &WORKER_TOOLS);
        let clean = execute(&project, &session, "edit", &json!({ "path": "src/hello.rs", "old_string": "\"hello\"", "new_string": "\"hello there\"" })).expect("allowed");
        assert_eq!((clean.as_str(), text(&dir, "src/hello.rs").contains("hello there")), ("cargo clippy: clean", true));
        let warned_module = "//! The hello slice.\n\n/// Greets.\npub fn greet() -> &'static str {\n    let unused = 1;\n    \"hello\"\n}\n";
        let warned = execute(&project, &session, "write", &json!({ "path": "src/hello.rs", "content": warned_module })).expect("allowed");
        mentions(&warned, &["unused"]);
        let tally = tally::load(&project, "canopy:allowed").expect("tally");
        assert_eq!((tally.edits, tally.post_edit_checks, tally.policy_refusals), (2, 2, 0));
    }

    #[test]
    #[validates(spec::ASessionWithoutAPhaseAsksNoPreToolVerdict)]
    fn a_session_without_a_phase_asks_no_pre_tool_verdict() {
        let (dir, project) = fixture::copy("canopy-tools-phaseless");
        let manifest = dir.join("Cargo.toml");
        let carrying = session("phased", Phase::Five, &WORKER_TOOLS);
        let host = replay::session(Door::new("http://127.0.0.1:1", "k"), "phaseless", None, declarations(&WORKER_TOOLS));
        let bounded = verdict(&project, &carrying, Tool::Write, Some(&manifest));
        let unbounded = verdict(&project, &host, Tool::Write, Some(&manifest));
        assert_eq!((bounded.is_err(), unbounded), (true, Ok(())), "the same write: judged where the session carries a phase whose policy bounds it, asked of nothing where it carries none");
        mentions(&bounded.expect_err("Phase 5's policy bounds its artifact"), &["Cargo.toml", "Phase 5"]);
        verdict(&project, &host, Tool::Read, None).expect("nor is an observation's verdict asked");
        mentions(&execute(&project, &host, "read", &json!({ "path": "src/hello.rs" })).expect("the tool does its work"), &["The hello slice"]);
    }

    #[test]
    #[validates(spec::AnEditForwardedToTheReviewerIsRefusedHere)]
    fn an_edit_forwarded_to_the_reviewer_is_refused_here() {
        let (dir, project) = fixture::copy("canopy-tools-reviewer");
        let session = session("reviewer", Phase::Three, &REVIEW_TOOLS);
        let before = text(&dir, "src/hello.rs");
        let err = execute(&project, &session, "edit", &json!({ "path": "src/hello.rs", "old_string": "\"hello\"", "new_string": "\"hi\"" })).expect_err("refused");
        mentions(&err, &["edit"]);
        execute(&project, &session, "write", &json!({ "path": "src/hello.rs", "content": "" })).expect_err("refused");
        assert_eq!(text(&dir, "src/hello.rs"), before, "untouched");
        assert_eq!((declared(&session, "read").expect("declared"), declared(&session, "edit").is_err(), declared(&session, "bash").is_err()), (Tool::Read, true, true));
        assert_eq!(tally::load(&project, "canopy:reviewer").expect("tally"), Tally::default(), "nothing ran, no verdict was asked");
    }
}
