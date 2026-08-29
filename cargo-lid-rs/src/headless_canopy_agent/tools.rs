//! The five tools (`docs/intent/headless-canopy-agent/lld.md` § The five
//! tools): the closed set the model may call, their declarations, the
//! workspace boundary every one applies first, the dispatch that asks the
//! phase library's verdicts around the work, and the work itself.

use std::path::{Path, PathBuf};

use lid_rs::implements;
use serde::Deserialize;
use serde_json::Value;

use super::door::ToolDecl;
use super::turn::Session;
use crate::project::Project;
use crate::spec;

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

/// `write`'s arguments.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WriteArgs {
    /// The file, relative to the root.
    pub path: String,
    /// Its whole content.
    pub content: String,
}

/// The declarations of `tools` for the requestee `lid-rs`: each with the
/// JSON schema of its arguments, the tool's description in that schema's
/// `description`.
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn declarations(tools: &[Tool]) -> Vec<ToolDecl> {
    todo!()
}

/// The workspace boundary every tool applies before any verdict is asked:
/// `path`, relative to `root`, resolved to an absolute path inside it; a
/// path that resolves outside — by `..`, by being absolute, or through a
/// symlink — is the refusal.
#[implements(spec::EveryToolConfinesItsPathToTheWorkspace)]
pub fn confine(root: &Path, path: &Path) -> Result<PathBuf, String> {
    todo!()
}

/// One forwarded call, dispatched over `Tool`: the path confined, the
/// phase library's pre-tool verdict asked under the session's agent id —
/// an observation is tallied and never refused; an edit or write is refused
/// with the verdict's wording, the file untouched — then the work, and for
/// an allowed edit the post-edit verdict's text as the result. An `op` the
/// session did not declare (an edit forwarded to the reviewer) is refused
/// here, whatever the authorizer did.
#[implements(
    spec::ObservationsAreTalliedThroughThePreToolVerdict,
    spec::ARefusedEditIsTheToolsErrorAndTheFileIsUntouched,
    spec::AnAllowedEditReturnsThePostEditVerdictsText,
    spec::AnEditForwardedToTheReviewerIsRefusedHere,
)]
pub fn execute(project: &Project, session: &Session, op: &str, args: &Value) -> ToolResult {
    todo!()
}

/// `read` over a confined path: a file's lines numbered, from `offset` for
/// at most `limit` lines; a directory's entries.
#[implements(spec::ReadReturnsNumberedLinesOrADirectorysEntries)]
pub fn read_tool(path: &Path, args: &ReadArgs) -> ToolResult {
    todo!()
}

/// `grep` under a confined directory: the pattern as a literal,
/// case-sensitive substring in the files under it narrowed by the glob,
/// `path:line: text` per match with paths relative to `root`, at most 200
/// lines.
#[implements(spec::GrepIsALiteralSubstringSearchCappedAtTwoHundredLines)]
pub fn grep_tool(root: &Path, under: &Path, args: &GrepArgs) -> ToolResult {
    todo!()
}

/// `glob` under the root: the matching paths, relative to it, sorted.
#[implements(spec::GlobReturnsMatchingPathsSorted)]
pub fn glob_tool(root: &Path, args: &GlobArgs) -> ToolResult {
    todo!()
}

/// `edit` over a confined existing file: `old_string` replaced where it
/// occurs exactly once, or everywhere when `replace_all`; zero occurrences,
/// or more than one without `replace_all`, changes nothing and is an error
/// naming the count.
#[implements(spec::EditReplacesTheOneOccurrenceOrAllOnRequest, spec::AnAmbiguousOrAbsentOldStringIsAnErrorNamingTheCount)]
pub fn edit_tool(path: &Path, args: &EditArgs) -> ToolResult {
    todo!()
}

/// `write` over a confined path: the file created, or its content replaced
/// whole.
#[implements(spec::WriteCreatesOrReplacesTheFileWhole)]
pub fn write_tool(path: &Path, args: &WriteArgs) -> ToolResult {
    todo!()
}
