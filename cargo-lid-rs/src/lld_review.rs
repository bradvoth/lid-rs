
use std::path::PathBuf;

use lid_rs::implements;

use crate::project::Project;
use crate::spec;

/// One slice's LLD as `lld-check` reads it: the document's lines, with the
/// slice it was asked for and the path it came from, so that every failure
/// about it can name a file and a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lld {
    /// The slice the document is for, as `--slice` or the branch named it.
    pub slice: String,
    /// The file the document was read from.
    pub path: PathBuf,
    /// The document's lines, in order; the line a failure numbers `n` is
    /// `lines[n - 1]`.
    pub lines: Vec<String>,
}

impl Lld {
    /// The slice's document: `docs/intent/<slice>/lld.md` in the workspace
    /// package whose manifest directory holds it, or — for a slice whose
    /// product is the workspace rather than a crate — the same path at the
    /// workspace root, where a virtual manifest holds no package to find. A
    /// document that is under neither, or cannot be read, is the error, and it
    /// names the path that was looked for.
    #[implements(
        spec::TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt,
        spec::AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot,
        spec::AnUnreadableLldFailsNamingItsPath,
    )]
    pub fn read(project: &Project, slice: &str) -> Result<Self, String> {
        todo!("locate {slice}'s document under {project:?}, then read its lines")
    }
}

/// The closed set of mechanical checks — the properties of the text that hold
/// with no reader's judgment in them. A property that needs an opinion is not
/// here; it is the guideline's, and the reader's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// The document has a `## Decisions & Alternatives` heading with a table
    /// under it.
    DecisionsExist,
    /// Every row of that table has four non-empty cells.
    Alternatives,
    /// Every row of a `## Shape` table names at least one backticked
    /// identifier and gives a non-empty role.
    ShapeRows,
    /// Every item under `### Deferred` is a numbered list item.
    DeferredNumbered,
    /// The guideline's checklist names every variant of [`Check`].
    GuidelineNamesEveryCheck,
    /// The reader's frontmatter declares `Read`, `Grep`, `Glob` and nothing
    /// else.
    ReaderObservesOnly,
}

/// One failure: the check that did not hold, and where. The path is carried
/// rather than assumed because an artifact check's failure is about the
/// guideline or the reader, not about the document under check.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
pub struct Failure {
    /// The check that failed.
    pub check: Check,
    /// The file the failure is about.
    pub path: PathBuf,
    /// The line of that file it is on, counted from one.
    pub line: usize,
    /// What was found, and the sentence the skill states the rule in.
    pub message: String,
}

/// A markdown table under a heading, as the rows the checks read: the header
/// row and its separator are not among them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The table's rows, in document order.
    pub rows: Vec<Row>,
}

/// One row of a markdown table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The line of the document the row is on, counted from one.
    pub line: usize,
    /// The row's cells, in order, as the pipes delimit them.
    pub cells: Vec<String>,
}

/// `lld-check [--slice <name>]`: the slice from the flag or, absent it, from
/// the branch; the document that slice names; every check over it; and every
/// failure in the error, which the binary prints before exiting non-zero — so
/// the run exits zero exactly when every check holds. Any other argument is
/// rejected by name, as every subcommand of this tool rejects one.
#[implements(
    spec::TheSliceIsTheFlagsValueOrTheBranchName,
    spec::AnyOtherArgumentIsRejectedByName,
    spec::LldCheckExitsZeroOnlyWhenEveryCheckHolds,
)]
pub fn run(args: &[String]) -> Result<(), String> {
    todo!("parse {args:?}, read the slice's Lld, run check_all, render every failure")
}

/// Every check over one document, in the table's order: the four document
/// checks against `lld`, then the two artifact checks, which read the
/// project's synced copies and so run whatever slice was named. Every failure
/// is collected — no check is skipped because an earlier one failed. The
/// project is a parameter beside the document because the artifact checks are
/// about files the document does not name; the error is reserved for a project
/// whose root cannot be located, an unreadable artifact being a [`Failure`]
/// like any other.
#[implements(spec::EveryFailureIsReportedNotOnlyTheFirst, spec::TheArtifactChecksRunWhateverSliceIsNamed)]
pub fn check_all(project: &Project, lld: &Lld) -> Result<Vec<Failure>, String> {
    todo!("run every check over {lld:?} and {project:?}, collecting their failures")
}

/// The decisions table exists: a `## Decisions & Alternatives` heading with a
/// table under it. A document without one recorded no alternatives.
#[implements(spec::ADocumentWithoutADecisionsTableFails)]
pub fn decisions_exist(lld: &Lld) -> Vec<Failure> {
    todo!("the decisions table of {lld:?}, or the failure naming that heading")
}

/// Every row of the decisions table fills its four cells: a decision with no
/// alternative considered is a decision not yet examined.
#[implements(spec::EveryDecisionsRowFillsItsFourCells)]
pub fn alternatives(lld: &Lld) -> Vec<Failure> {
    todo!("the decisions rows of {lld:?} that leave a cell empty")
}

/// Every row of the `## Shape` table names a backticked identifier and gives a
/// role: a row with no identifier is a note, not a shape. A document with no
/// shape table holds this check, since a slice may name its shape in prose
/// instead.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole, spec::ADocumentWithNoShapeTableHoldsThatCheck)]
pub fn shape_rows(lld: &Lld) -> Vec<Failure> {
    todo!("the shape rows of {lld:?} that name no identifier or no role")
}

/// Every item under `### Deferred` is a numbered list item: an unnumbered
/// deferral cannot be cited by a phase that hits it. A document with no
/// deferred heading holds this check, having nothing to number.
#[implements(spec::EveryDeferredItemIsANumberedListItem, spec::ADocumentWithNoDeferredHeadingHoldsThatCheck)]
pub fn deferred_numbered(lld: &Lld) -> Vec<Failure> {
    todo!("the items under {lld:?}'s deferred heading that are not numbered")
}

/// The checklist in the project's synced `.claude/skills/lid-rs/references/lld.md`
/// names every variant of [`Check`]: a checklist and an enum that disagree are
/// two sources of truth, and the guideline is what a human reads before the
/// code refuses them. The failure names each variant the checklist omits, on
/// the line of that file's checklist heading — or, when it holds no such
/// heading, on its first line; a copy that is absent or unreadable is a failure
/// naming the path that was looked for.
#[implements(
    spec::EveryCheckIsNamedInTheGuidelinesChecklist,
    spec::AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine,
    spec::AnUnreadableSyncedArtifactFailsNamingItsPath,
)]
pub fn guideline_names_every_check(project: &Project) -> Result<Vec<Failure>, String> {
    todo!("read the guideline synced into {project:?} and compare its checklist with Check")
}

/// The frontmatter of the project's synced `.claude/agents/lid-rs-lld-review.md`
/// declares `Read`, `Grep`, `Glob` and nothing else: the reader is advisory,
/// and that it cannot edit is a property to hold rather than a sentence to
/// trust. The failure names the tools it declares, on the line of its `tools:`
/// declaration — or, when it has none, on its first line; a copy that is absent
/// or unreadable is a failure naming the path that was looked for.
#[implements(
    spec::TheReaderDeclaresOnlyTheObservationTools,
    spec::AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine,
    spec::AnUnreadableSyncedArtifactFailsNamingItsPath,
)]
pub fn reader_observes_only(project: &Project) -> Result<Vec<Failure>, String> {
    todo!("read the reader synced into {project:?} and compare its declared tools with the observation set")
}

/// The markdown table under a heading: the pipe-delimited rows between that
/// heading and the next heading, less the header row and its separator. None
/// when the document has no such heading, or no table under it — the one parse
/// the checks share.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
pub fn table_at(lld: &Lld, heading: &str) -> Option<Table> {
    todo!("the rows of {lld:?} under {heading}")
}

/// The backticked identifiers a cell names.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
pub fn identifiers(cell: &str) -> Vec<String> {
    todo!("the backticked spans of {cell}")
}
