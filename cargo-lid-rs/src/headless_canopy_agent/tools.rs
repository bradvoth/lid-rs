//! The five tools (`docs/intent/headless-canopy-agent/lld.md` § The five
//! tools): the closed set the model may call, their declarations, the
//! workspace boundary every one applies first, the dispatch that asks the
//! phase library's verdicts around the work, and the work itself.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use lid_rs::implements;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::door::ToolDecl;
use super::turn::Session;
use crate::phase::HookInput;
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
        todo!()
    }

    /// The tool's `op` on the wire — `read`, `grep`, `glob`, `edit`,
    /// `write` — the name [`Tool::of`] classifies from.
    #[implements(spec::AForwardsOpClassifiesToItsToolOrToNone)]
    pub fn op(self) -> &'static str {
        todo!()
    }

    /// The name the phase library's verdict knows the tool by — `Read`,
    /// `Grep`, `Glob`, `Edit`, `Write` — as `crate::phase::policy::kind_of`
    /// recognises it.
    #[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict)]
    pub fn hook_name(self) -> &'static str {
        todo!()
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
        todo!()
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

/// One tool as the policy declares it: the requestee `lid-rs`, its `op`,
/// and the JSON schema of its arguments ([`schema_of`]).
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn declaration(tool: Tool) -> ToolDecl {
    todo!()
}

/// The JSON schema of a tool's arguments, the tool's description in the
/// schema's `description`: `path` with optional `offset` and `limit`;
/// `pattern` with optional `path` and `glob`; `pattern`; `path`,
/// `old_string`, `new_string`, optional `replace_all`; `path`, `content`.
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn schema_of(tool: Tool) -> Value {
    todo!()
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
    todo!()
}

/// A relative path in its canonical form under the root, symlinks
/// followed: `root.join(relative)`'s deepest ancestor that exists,
/// canonicalised, joined with what remains — so a file about to be written
/// resolves as its directory does, and a symlink out of the root resolves
/// to where it points. `root` is the caller's, in any form; the answer is
/// canonical (`/private/var/…` for `/var/…` on macOS) and is for
/// [`within`] to judge, never for a tool to open or a verdict to strip.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn resolved(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    todo!()
}

/// The judgment, and nothing else: `resolved`, in the canonical form
/// [`resolved`] produces, must lie under the canonical form of `root` —
/// the caller's root, canonicalised here so canonical is compared with
/// canonical. One that does not is the refusal; one that does is `Ok(())`,
/// and the caller keeps the path it already holds on its own root.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn within(root: &Path, resolved: &Path) -> Result<(), String> {
    todo!()
}

/// One forwarded call: the `op` must be a tool the session declared
/// ([`declared`]) — an edit forwarded to the reviewer is refused here,
/// whatever the authorizer did — then dispatched over [`Tool`] to its
/// call, each of which confines the path, asks the phase library's
/// pre-tool verdict, and does the work. The root every call confines
/// against is the workspace root as cargo reports it, uncanonicalised: the
/// root the phase library's policy strips an edit's path against, and the
/// root the slice's crate is under in the same form.
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

/// The tool a forward's `op` names, provided the session's policy declared
/// it; an unknown `op`, or one the session did not declare — an `edit`
/// forwarded to the reviewer — is the refusal, and nothing runs.
#[implements(spec::AnEditForwardedToTheReviewerIsRefusedHere)]
pub fn declared(session: &Session, op: &str) -> Result<Tool, String> {
    todo!()
}

/// A payload's `args` as a tool's typed arguments; what does not fit is
/// the error.
pub fn arguments<T: DeserializeOwned>(args: &Value) -> Result<T, String> {
    todo!()
}

/// The phase library's input for one call: the session's agent id
/// (`canopy:<session>`), the tool's hook name, and for an edit its
/// confined path — on the workspace root as cargo reports it, as
/// [`confine`] returns it, since that is the root the policy strips it
/// against.
pub fn tool_input(session: &Session, tool: Tool, path: Option<&Path>) -> HookInput {
    todo!()
}

/// The pre-tool verdict for one call, asked as
/// [`crate::phase::hook_pre_tool`] with the session's agent id: an
/// observation is tallied and never refused; an edit or write is allowed,
/// or refused with the verdict's wording as the error, before any file is
/// touched.
#[implements(spec::ObservationsAreTalliedThroughThePreToolVerdict, spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched)]
pub fn verdict(project: &Project, session: &Session, tool: Tool, path: Option<&Path>) -> Result<(), String> {
    todo!()
}

/// The post-edit verdict after an allowed edit or write, asked as
/// [`crate::phase::hook_post_edit`] with the session's agent id: its text
/// — clippy's diagnostics, or "clean" — is the tool's result.
#[implements(spec::AnAllowedEditReturnsThePostEditVerdictsText)]
pub fn checked(project: &Project, session: &Session, tool: Tool, path: &Path) -> ToolResult {
    todo!()
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
    todo!()
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
    todo!()
}

/// The path a glob pattern names literally: its text before the first
/// metacharacter — `*`, `?`, or `[` — which is what [`confine`] judges,
/// since every match lies under it. A pattern that begins with a
/// metacharacter has an empty prefix: the root itself.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn literal_prefix(pattern: &str) -> &Path {
    todo!()
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
    todo!()
}

/// A file's lines, each prefixed with its number counted from 1, from
/// `offset` (the first line when absent) for at most `limit` lines (all of
/// them when absent); a file that cannot be read as text is the error.
#[implements(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
pub fn numbered_lines(path: &Path, offset: Option<usize>, limit: Option<usize>) -> ToolResult {
    todo!()
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
    todo!()
}

/// The two directory names neither `grep` nor `glob` ever enters: build
/// output and git's objects, which no search of the source wants.
pub const SKIPPED: [&str; 2] = ["target", ".git"];

/// Whether one path component names a directory the walks never enter —
/// one of [`SKIPPED`]. The one predicate both tools' walks ask, of every
/// directory before descending into it.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
pub fn skipped(component: &OsStr) -> bool {
    todo!()
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
    todo!()
}

/// The files whose path relative to `root` matches the glob — all of them
/// when there is none; a glob that does not parse is the error. What
/// narrows `grep`'s files, and what `glob`'s pattern selects from the
/// walk.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines, spec::GlobReturnsMatchingPathsSorted)]
pub fn narrowed(root: &Path, files: Vec<PathBuf>, glob: Option<&str>) -> Result<Vec<PathBuf>, String> {
    todo!()
}

/// The lines of one file that contain the pattern as a literal,
/// case-sensitive substring, each as `path:line: text` with the path
/// relative to `root` — `file` is on `root` in the same form, as the walk
/// yields it, so the relation is by prefix — and the line counted from 1;
/// a file that is not text yields none.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn matches_in(root: &Path, file: &Path, pattern: &str) -> Vec<String> {
    todo!()
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
    todo!()
}

/// The paths relative to the root, as `glob` answers them, sorted.
#[implements(spec::GlobReturnsMatchingPathsSorted)]
pub fn relative_sorted(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    todo!()
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
    todo!()
}

/// The text with every occurrence of `old` replaced by `new` — the one
/// occurrence there is, or all of them, as [`replaceable`] has admitted.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn replaced(text: &str, old: &str, new: &str) -> String {
    todo!()
}

/// The edited text written back over the file; a write that fails is the
/// error naming the path.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest)]
pub fn written(path: &Path, text: &str) -> Result<(), String> {
    todo!()
}

/// Whether the occurrences found may be replaced: exactly one for
/// [`Replace::One`], any number above zero for [`Replace::All`]; zero, or
/// more than one for [`Replace::One`], is the error naming the count.
#[implements(spec::AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount)]
pub fn replaceable(count: usize, replace: Replace) -> Result<(), String> {
    todo!()
}

/// `write` over a confined path: the file created, or its content replaced
/// whole.
#[implements(spec::WriteCreatesOrReplacesTheFileWhole)]
pub fn write_tool(path: &Path, args: &WriteArgs) -> Result<(), String> {
    todo!()
}
