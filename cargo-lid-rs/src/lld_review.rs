
use std::ops::Range;
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::phase::policy::slice_crate;
use crate::phase::resolve_slice;
use crate::project::Project;
use crate::spec;

/// What `lld-check` prints beside a rejected argument.
const LLD_CHECK_USAGE: &str = "usage: cargo lid-rs lld-check [--slice <name>]";

/// The heading the decisions table is under — named rather than inferred.
const DECISIONS_HEADING: &str = "## Decisions & Alternatives";

/// The heading the shape table is under.
const SHAPE_HEADING: &str = "## Shape";

/// The heading the deferred list is under — named rather than inferred.
const DEFERRED_HEADING: &str = "### Deferred";

/// The start of the heading the guideline's checklist is under; the rest of
/// that heading's sentence is the guideline's to reword.
const CHECKLIST_HEADING: &str = "## The checklist";

/// The start of the line the reader's frontmatter declares its tools on.
const TOOLS_DECLARATION: &str = "tools:";

/// The project's synced copy of the guideline, relative to the workspace root.
const GUIDELINE: &str = ".claude/skills/lid-rs/references/lld.md";

/// The project's synced copy of the reader, relative to the workspace root.
const READER: &str = ".claude/agents/lid-rs-lld-review.md";

/// The tools an advisory reader may declare: it observes, and cannot act.
const OBSERVATION_TOOLS: [&str; 3] = ["Read", "Grep", "Glob"];

/// Every check, in the order the LLD's table states them: the four over the
/// document, then the two over the project's synced artifacts.
const EVERY_CHECK: [Check; 6] = [
    Check::DecisionsExist,
    Check::Alternatives,
    Check::ShapeRows,
    Check::DeferredNumbered,
    Check::GuidelineNamesEveryCheck,
    Check::ReaderObservesOnly,
];

/// The two lines a markdown table spends before its body: the header row and
/// the separator under it.
const HEADER_AND_SEPARATOR: usize = 2;

/// The line a failure points at when its file holds no line to cite.
const FIRST_LINE: usize = 1;

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
    /// The slice's document: the path the slice names, under the package that
    /// holds it or at the workspace root, read into lines.
    pub fn read(project: &Project, slice: &str) -> Result<Self, String> {
        let path = lld_path(project, slice)?;
        Ok(Self { slice: slice.to_string(), lines: read_lines(&path)?, path })
    }
}

/// Where the slice's document is: `docs/intent/<slice>/lld.md` under the
/// workspace package whose manifest directory holds it, or — for a slice whose
/// product is the workspace rather than a crate — the same path at the
/// workspace root, where a virtual manifest holds no package to find. A slice
/// under neither gets the root's path, which is then the one an unreadable
/// document names.
#[implements(
    spec::TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt,
    spec::AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot,
)]
fn lld_path(project: &Project, slice: &str) -> Result<PathBuf, String> {
    match slice_crate(project, slice) {
        Ok(package) => Ok(package.join(document_relative(slice))),
        Err(_) => Ok(project.root()?.join(document_relative(slice))),
    }
}

/// A slice's document, relative to the directory that holds it:
/// `docs/intent/<slice>/lld.md`.
#[implements(spec::TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt)]
fn document_relative(slice: &str) -> PathBuf {
    todo!("docs/intent/{slice}/lld.md, as a relative path")
}

/// A text file's lines, or an error naming the path that was looked for — a
/// file that is absent, a directory, or not text is not read.
#[implements(spec::AnUnreadableLldFailsNamingItsPath)]
fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    todo!("the lines of {path:?}, or an error naming it")
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

/// A failure over the document under check: the check, the file, the line, and
/// the sentence the skill states the rule in. What was found needs no saying —
/// the line says it.
#[implements(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
fn failure(check: Check, path: &Path, line: usize) -> Failure {
    Failure { check, path: path.to_path_buf(), line, message: rule(check).to_string() }
}

/// A failure over one of the project's synced artifacts, which names what it
/// found — the checks the checklist omits, or the tools the reader declares —
/// before the sentence the skill states the rule in.
#[implements(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
fn artifact_failure(check: Check, path: &Path, line: usize, found: &str) -> Failure {
    Failure { check, path: path.to_path_buf(), line, message: format!("{found}: {}", rule(check)) }
}

/// The sentence the skill states a check's rule in, which its failure quotes.
#[implements(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
fn rule(check: Check) -> &'static str {
    todo!("the guideline's sentence for {check:?}")
}

/// The document line number of a zero-based index into [`Lld::lines`] — the
/// one place the two counts are reconciled.
#[implements(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
fn line_number(index: usize) -> usize {
    index + 1
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
/// the run exits zero exactly when every check holds.
pub fn run(args: &[String]) -> Result<(), String> {
    let project = Project::load()?;
    let slice = slice_of(&project, parse_args(args)?)?;
    let lld = Lld::read(&project, &slice)?;
    report(&check_all(&project, &lld)?)
}

/// The `--slice <name>` flag, if given; any other argument is rejected by
/// name, as every subcommand of this tool rejects one.
#[implements(spec::TheSliceIsTheFlagsValueOrTheBranchName, spec::AnyOtherArgumentIsRejectedByName)]
fn parse_args(args: &[String]) -> Result<Option<String>, String> {
    match args {
        [] => Ok(None),
        [flag, name] if flag == "--slice" => Ok(Some(name.clone())),
        [flag] if flag == "--slice" => Err(format!("--slice requires a name\n{LLD_CHECK_USAGE}")),
        [flag, ..] => Err(format!("unknown argument `{flag}` for lld-check\n{LLD_CHECK_USAGE}")),
    }
}

/// The slice the flag named, or the branch's, as `phase-check` reads it; a run
/// on no `lld/<slice>` branch names no slice, and so has no document to read.
#[implements(spec::TheSliceIsTheFlagsValueOrTheBranchName)]
fn slice_of(project: &Project, given: Option<String>) -> Result<String, String> {
    match resolve_slice(project, given)? {
        Some(slice) => Ok(slice),
        None => Err(format!("no slice: the branch is not named `lld/<slice>`\n{LLD_CHECK_USAGE}")),
    }
}

/// Zero when every check held, and otherwise every failure in the error the
/// binary prints before exiting non-zero.
#[implements(spec::LldCheckExitsZeroOnlyWhenEveryCheckHolds)]
fn report(failures: &[Failure]) -> Result<(), String> {
    if failures.is_empty() { Ok(()) } else { Err(rendered(failures)) }
}

/// Every failure, one to a line, each naming its file, its line, its check and
/// its message — the whole list, so a human sees it rather than the first item.
#[implements(spec::EveryFailureIsReportedNotOnlyTheFirst)]
fn rendered(failures: &[Failure]) -> String {
    todo!("a line for each of {failures:?}")
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
    Ok([
        decisions_exist(lld),
        alternatives(lld),
        shape_rows(lld),
        deferred_numbered(lld),
        guideline_names_every_check(project)?,
        reader_observes_only(project)?,
    ]
    .concat())
}

/// The decisions table exists: a `## Decisions & Alternatives` heading with a
/// table under it. A document without one recorded no alternatives, and its
/// failure has no row to point at, so it points at the document's first line.
#[implements(spec::ADocumentWithoutADecisionsTableFails)]
pub fn decisions_exist(lld: &Lld) -> Vec<Failure> {
    match table_at(lld, DECISIONS_HEADING) {
        Some(_) => vec![],
        None => vec![failure(Check::DecisionsExist, &lld.path, FIRST_LINE)],
    }
}

/// Every row of the decisions table fills its four cells: a decision with no
/// alternative considered is a decision not yet examined. A document with no
/// such table has no row to fill, and [`decisions_exist`] is the check that
/// refuses it.
#[implements(spec::EveryDecisionsRowFillsItsFourCells)]
pub fn alternatives(lld: &Lld) -> Vec<Failure> {
    table_at(lld, DECISIONS_HEADING)
        .map_or_else(Vec::new, |table| table.rows)
        .iter()
        .filter(|row| !fills_four_cells(row))
        .map(|row| failure(Check::Alternatives, &lld.path, row.line))
        .collect()
}

/// Whether a decisions row has four cells with something in each.
#[implements(spec::EveryDecisionsRowFillsItsFourCells)]
fn fills_four_cells(row: &Row) -> bool {
    todo!("whether {row:?} has four cells and none of them is empty")
}

/// Every row of the `## Shape` table names a backticked identifier and gives a
/// role: a row with no identifier is a note, not a shape. A document with no
/// shape table holds this check, since a slice may name its shape in prose
/// instead.
#[implements(spec::ADocumentWithNoShapeTableHoldsThatCheck)]
pub fn shape_rows(lld: &Lld) -> Vec<Failure> {
    match table_at(lld, SHAPE_HEADING) {
        Some(table) => rows_without_identifier_or_role(lld, &table),
        None => vec![],
    }
}

/// The shape rows that name no identifier or give no role, each failing on its
/// own line.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
fn rows_without_identifier_or_role(lld: &Lld, table: &Table) -> Vec<Failure> {
    table
        .rows
        .iter()
        .filter(|row| !names_identifier_and_role(row))
        .map(|row| failure(Check::ShapeRows, &lld.path, row.line))
        .collect()
}

/// Whether a shape row names at least one backticked identifier — the
/// [`identifiers`] its first cell holds — and gives a non-empty role.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
fn names_identifier_and_role(row: &Row) -> bool {
    todo!("whether {row:?} names an identifier and gives a role")
}

/// Every item under `### Deferred` is a numbered list item: an unnumbered
/// deferral cannot be cited by a phase that hits it. A document with no
/// deferred heading holds this check, having nothing to number.
#[implements(spec::ADocumentWithNoDeferredHeadingHoldsThatCheck)]
pub fn deferred_numbered(lld: &Lld) -> Vec<Failure> {
    match section_range(&lld.lines, DEFERRED_HEADING) {
        Some(deferred) => unnumbered_items(lld, deferred),
        None => vec![],
    }
}

/// The lines of the deferred section that are list items without a number,
/// each failing on its own line.
#[implements(spec::EveryDeferredItemIsANumberedListItem)]
fn unnumbered_items(lld: &Lld, deferred: Range<usize>) -> Vec<Failure> {
    deferred
        .filter(|index| is_unnumbered_item(&lld.lines[*index]))
        .map(|index| failure(Check::DeferredNumbered, &lld.path, line_number(index)))
        .collect()
}

/// Whether a line is a list item that carries no number — prose, a blank line
/// and a numbered item alike are not.
#[implements(spec::EveryDeferredItemIsANumberedListItem)]
fn is_unnumbered_item(line: &str) -> bool {
    todo!("whether {line} is a list item without a number")
}

/// The checklist in the project's synced `.claude/skills/lid-rs/references/lld.md`
/// names every variant of [`Check`]: a checklist and an enum that disagree are
/// two sources of truth, and the guideline is what a human reads before the
/// code refuses them. A copy that is absent or unreadable is a failure naming
/// the path that was looked for, on that path's first line; only a project
/// whose root cannot be located is the error.
#[implements(spec::AnUnreadableSyncedArtifactFailsNamingItsPath)]
pub fn guideline_names_every_check(project: &Project) -> Result<Vec<Failure>, String> {
    let path = artifact_path(project, GUIDELINE)?;
    Ok(match read_lines(&path) {
        Ok(lines) => checklist_failures(&path, &lines),
        Err(unreadable) => vec![artifact_failure(Check::GuidelineNamesEveryCheck, &path, FIRST_LINE, &unreadable)],
    })
}

/// One failure naming the checks the guideline's checklist omits, on the line
/// of that checklist's heading — or none, when it names them all.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn checklist_failures(path: &Path, lines: &[String]) -> Vec<Failure> {
    let omitted = checks_omitted(lines);
    if omitted.is_empty() {
        vec![]
    } else {
        vec![artifact_failure(Check::GuidelineNamesEveryCheck, path, line_of(lines, CHECKLIST_HEADING), &spelled(&omitted))]
    }
}

/// The checks the guideline's checklist does not name; a guideline with no
/// checklist at all names none of them.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn checks_omitted(lines: &[String]) -> Vec<Check> {
    let checklist = checklist(lines);
    EVERY_CHECK.into_iter().filter(|check| !names_check(checklist, *check)).collect()
}

/// The guideline's checklist: the lines under its heading, and no line at all
/// when it holds no such heading.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn checklist(lines: &[String]) -> &[String] {
    match section_range(lines, CHECKLIST_HEADING) {
        Some(checklist) => &lines[checklist],
        None => &[],
    }
}

/// Whether any of the checklist's lines names a check as the checklist spells
/// one.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn names_check(checklist: &[String], check: Check) -> bool {
    let spelling = spelling(check);
    checklist.iter().any(|line| line.contains(&spelling))
}

/// The checks as the checklist spells them, in a comma-separated list.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn spelled(checks: &[Check]) -> String {
    checks.iter().copied().map(spelling).collect::<Vec<String>>().join(", ")
}

/// How the checklist spells one check: its variant name, in backticks.
#[implements(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
fn spelling(check: Check) -> String {
    todo!("the backticked name of {check:?}")
}

/// The frontmatter of the project's synced `.claude/agents/lid-rs-lld-review.md`
/// declares `Read`, `Grep`, `Glob` and nothing else: the reader is advisory,
/// and that it cannot edit is a property to hold rather than a sentence to
/// trust. A copy that is absent or unreadable is a failure naming the path that
/// was looked for, on that path's first line; only a project whose root cannot
/// be located is the error.
#[implements(spec::AnUnreadableSyncedArtifactFailsNamingItsPath)]
pub fn reader_observes_only(project: &Project) -> Result<Vec<Failure>, String> {
    let path = artifact_path(project, READER)?;
    Ok(match read_lines(&path) {
        Ok(lines) => tool_failures(&path, &lines),
        Err(unreadable) => vec![artifact_failure(Check::ReaderObservesOnly, &path, FIRST_LINE, &unreadable)],
    })
}

/// One failure naming the tools the reader declares, on the line of its
/// declaration — or none, when they are the observation set exactly.
#[implements(spec::TheReaderDeclaresOnlyTheObservationTools)]
fn tool_failures(path: &Path, lines: &[String]) -> Vec<Failure> {
    let declared = declared_tools(lines);
    if observes_only(&declared) {
        vec![]
    } else {
        vec![artifact_failure(Check::ReaderObservesOnly, path, line_of(lines, TOOLS_DECLARATION), &declared.join(", "))]
    }
}

/// The tools the reader's `tools:` declaration names; none when it has no such
/// line, which declares no tool.
#[implements(spec::TheReaderDeclaresOnlyTheObservationTools)]
fn declared_tools(lines: &[String]) -> Vec<String> {
    todo!("the comma-separated tools the `tools:` line of {lines:?} names")
}

/// Whether the declared tools are the observation set and nothing else, in
/// whatever order they were declared.
#[implements(spec::TheReaderDeclaresOnlyTheObservationTools)]
fn observes_only(declared: &[String]) -> bool {
    todo!("whether {declared:?} is exactly {OBSERVATION_TOOLS:?}, in any order")
}

/// Where a synced artifact is: the path it is at, relative to the workspace
/// root. A project whose root cannot be located is the error both artifact
/// checks reserve theirs for.
#[implements(spec::AnUnreadableSyncedArtifactFailsNamingItsPath)]
fn artifact_path(project: &Project, relative: &str) -> Result<PathBuf, String> {
    todo!("{relative} under the root of {project:?}")
}

/// The number of the first line that starts with `marker`, and the file's
/// first line when no line does — an artifact failure always cites a line.
#[implements(spec::AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine)]
fn line_of(lines: &[String], marker: &str) -> usize {
    lines
        .iter()
        .position(|line| line.trim_start().starts_with(marker))
        .map_or(FIRST_LINE, line_number)
}

/// The markdown table under a heading: the pipe-delimited rows between that
/// heading and the next heading, less the header row and its separator. None
/// when the document has no such heading, or no table under it — the one parse
/// the checks share.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
pub fn table_at(lld: &Lld, heading: &str) -> Option<Table> {
    let rows = pipe_rows(lld, section_range(&lld.lines, heading)?);
    Some(Table { rows: rows.get(HEADER_AND_SEPARATOR..)?.to_vec() })
}

/// The lines under a heading: those between the first line that starts with it
/// and the next line that starts a heading, as indices into `lines`. None when
/// no line starts with the heading.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
fn section_range(lines: &[String], heading: &str) -> Option<Range<usize>> {
    todo!("the lines of {lines:?} under {heading}, up to the next heading")
}

/// The pipe-delimited lines of a section, in order, each as the row it is:
/// the line it is on, and the cells between its pipes.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
fn pipe_rows(lld: &Lld, section: Range<usize>) -> Vec<Row> {
    section
        .filter(|index| is_pipe_row(&lld.lines[*index]))
        .map(|index| Row { line: line_number(index), cells: cells(&lld.lines[index]) })
        .collect()
}

/// Whether a line is a row of a markdown table — prose and a blank line are
/// not, and the separator under a header is.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
fn is_pipe_row(line: &str) -> bool {
    todo!("whether {line} is a pipe-delimited row")
}

/// The cells a pipe-delimited line holds: what lies between its pipes,
/// trimmed, less the empty ends the leading and trailing pipes make.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
fn cells(line: &str) -> Vec<String> {
    todo!("the cells of {line}")
}

/// The backticked identifiers a cell names.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
pub fn identifiers(cell: &str) -> Vec<String> {
    todo!("the backticked spans of {cell}")
}
