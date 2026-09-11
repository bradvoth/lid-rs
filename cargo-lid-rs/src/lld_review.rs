
use std::collections::BTreeSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::layout;
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

/// The character a markdown table delimits its cells with.
const CELL_DELIMITER: char = '|';

/// The character a markdown heading starts with, which is what ends the
/// section above it.
const HEADING_MARK: char = '#';

/// The markers a markdown list item carries when it carries no number.
const BULLETS: [&str; 3] = ["- ", "* ", "+ "];

/// The cells a decisions row fills: the decision, what was chosen, the
/// alternatives considered, and the rationale.
const DECISION_CELLS: usize = 4;

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
    /// The slice's document, wherever the layout puts it: the `lld.md` beside
    /// the slice's code once its directory holds one, the
    /// `docs/intent/<slice>/lld.md` the package that holds the slice keeps
    /// until then, and — for a slice no workspace package holds a document for
    /// — that same path under the workspace root, where a slice whose product
    /// is the workspace rather than a crate keeps it. Read into lines, so that
    /// every failure about the document names a file and a line.
    ///
    /// The layout is asked rather than rebuilt here, and that is the whole of
    /// what this function decides. `lld-check` is a step of `phase-check 1`,
    /// so a document it located by a rule of its own would answer for one
    /// shape of the tree while every other resolver answered for four: the
    /// first slice whose document moved beside its code would fail Phase 1,
    /// for itself and for every slice committed after it, with a sentence
    /// naming a path nothing put a document at.
    #[implements(
        spec::TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt,
        spec::AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot,
        spec::AnUnreadableLldFailsNamingItsPath,
    )]
    pub fn read(project: &Project, slice: &str) -> Result<Self, String> {
        let path = layout::lld_path(project, slice)?;
        Ok(Self { slice: slice.to_string(), lines: read_lines(&path)?, path })
    }
}

/// A text file's lines, or an error naming the path that was looked for — a
/// file that is absent, a directory, or not text is not read.
#[implements(spec::AnUnreadableLldFailsNamingItsPath)]
fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    std::fs::read_to_string(path)
        .map(|text| text.lines().map(str::to_string).collect())
        .map_err(|unreadable| format!("{}: {unreadable}", path.display()))
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
    match check {
        Check::DecisionsExist => "the document has a `## Decisions & Alternatives` heading with a table under it",
        Check::Alternatives => "every row of that table has four non-empty cells",
        Check::ShapeRows => {
            "where a `## Shape` table exists, every row names at least one backticked identifier and gives a non-empty role"
        }
        Check::DeferredNumbered => "every item under `### Deferred` is a numbered list item",
        Check::GuidelineNamesEveryCheck => "the guideline's checklist names every check the tool knows",
        Check::ReaderObservesOnly => "the reader declares `Read`, `Grep`, `Glob` and nothing else",
    }
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
pub fn report(failures: &[Failure]) -> Result<(), String> {
    if failures.is_empty() { Ok(()) } else { Err(rendered(failures)) }
}

/// Every failure, one to a line, each naming its file, its line, its check and
/// its message — the whole list, so a human sees it rather than the first item.
#[implements(spec::EveryFailureIsReportedNotOnlyTheFirst)]
fn rendered(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(|failure| format!("{}:{}: {:?}: {}", failure.path.display(), failure.line, failure.check, failure.message))
        .collect::<Vec<String>>()
        .join("\n")
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
    row.cells.len() == DECISION_CELLS && row.cells.iter().all(|cell| !cell.is_empty())
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
    let names = row.cells.first().is_some_and(|item| !identifiers(item).is_empty());
    let role = row.cells.get(1).is_some_and(|role| !role.is_empty());
    names && role
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
    BULLETS.iter().any(|bullet| line.trim_start().starts_with(bullet))
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
    format!("`{check:?}`")
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
    lines
        .iter()
        .find_map(|line| line.trim_start().strip_prefix(TOOLS_DECLARATION))
        .map_or_else(Vec::new, |declared| declared.split(',').map(|tool| tool.trim().to_string()).collect())
}

/// Whether the declared tools are the observation set and nothing else, in
/// whatever order they were declared.
#[implements(spec::TheReaderDeclaresOnlyTheObservationTools)]
fn observes_only(declared: &[String]) -> bool {
    declared.iter().map(String::as_str).collect::<BTreeSet<&str>>() == OBSERVATION_TOOLS.into_iter().collect()
}

/// Where a synced artifact is: the path it is at, relative to the workspace
/// root. A project whose root cannot be located is the error both artifact
/// checks reserve theirs for.
#[implements(spec::AnUnreadableSyncedArtifactFailsNamingItsPath)]
fn artifact_path(project: &Project, relative: &str) -> Result<PathBuf, String> {
    Ok(project.root()?.join(relative))
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
    let start = lines.iter().position(|line| line.starts_with(heading))? + 1;
    let end = lines[start..]
        .iter()
        .position(|line| line.starts_with(HEADING_MARK))
        .map_or(lines.len(), |offset| start + offset);
    Some(start..end)
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
    line.trim_start().starts_with(CELL_DELIMITER)
}

/// The cells a pipe-delimited line holds: what lies between its pipes,
/// trimmed, less the empty ends the leading and trailing pipes make.
#[implements(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
fn cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_start_matches(CELL_DELIMITER)
        .trim_end_matches(CELL_DELIMITER)
        .split(CELL_DELIMITER)
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// The backticked identifiers a cell names.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
pub fn identifiers(cell: &str) -> Vec<String> {
    cell.split('`').skip(1).step_by(2).map(str::to_string).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    use std::collections::BTreeSet;

    use crate::phase::fixture;

    /// A document that holds every check: a decisions table whose row fills
    /// its four cells, a shape row naming an identifier and a role, and a
    /// numbered deferral.
    const HOLDS: &str = "\
# s — a slice

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

    /// A document that fails three checks at once: no decisions table, a
    /// shape row naming no identifier, and an unnumbered deferral.
    const FAILS_THREE: &str = "\
# s — a slice

## Shape

| Item | Role |
|---|---|
| no identifier | a note |

### Deferred
- an unnumbered deferral
";

    /// A document with two tables under two headings, and a heading with no
    /// table under it at all.
    const TWO_TABLES: &str = "\
# d

## Behaviour

Prose only, and no table anywhere under this heading.

## Shape

| Item | Role |
|---|---|
| `a` | first |
| `b` | second |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| what | this | that | because |
";

    /// A document whose decisions heading is there but whose decisions are
    /// prose, so there is no table under it.
    const DECISIONS_IN_PROSE: &str = "\
# d

## Decisions & Alternatives

The decisions are told in a paragraph, and there is no table.
";

    /// A decisions table with a filled row, a row of three cells, and a row
    /// whose second cell is empty.
    const DECISIONS_ROWS: &str = "\
# d

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| full | this | that | because |
| three | cells | only |
| empty |  | that | because |
";

    /// A shape table with a row that names an identifier and a role, a row
    /// that names no identifier, and a row that gives no role.
    const SHAPE_ROWS: &str = "\
# d

## Shape

| Item | Role |
|---|---|
| `run(args)` | the entry |
| plain prose | a note |
| `Lld` |  |
";

    /// A document whose shape is named in prose under its heading rather
    /// than in a table.
    const SHAPE_IN_PROSE: &str = "\
# d

## Shape

`run(args)` is the entry, described in a sentence rather than in a table.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| what | this | that | because |
";

    /// A deferred section with a numbered item, its continuation line, and
    /// two unnumbered items — and a bullet under a later heading.
    const DEFERRED_ITEMS: &str = "\
# d

## Open Questions

### Deferred
1. A numbered deferral,
   continued on this line.
- an unnumbered one
* a second unnumbered one

## References

- a bullet outside the deferred section
";

    /// A document with bullets and no deferred heading to number them under.
    const BULLETS_ELSEWHERE: &str = "\
# d

## References

- a bullet, under no deferred heading
- another
";

    /// A guideline whose checklist names four of the six checks, and whose
    /// questions name the other two outside the checklist.
    const PARTIAL_CHECKLIST: &str = "\
# Writing an LLD, and reading one

## The checklist — what the tool refuses

| Check | It holds when |
|---|---|
| `DecisionsExist` | the document records its decisions |
| `Alternatives` | every row fills its four cells |
| `DeferredNumbered` | every deferral is numbered |
| `GuidelineNamesEveryCheck` | this checklist names every check |

## The questions — what a reader asks

Outside the checklist, `ShapeRows` and `ReaderObservesOnly` are named here.
";

    /// The arguments a command line gives, as the strings `run` takes.
    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|item| (*item).to_string()).collect()
    }

    /// One of the fixture documents as the [`Lld`] a slice's document would
    /// have been read into.
    fn document(slice: &str, text: &str) -> Lld {
        Lld {
            slice: slice.to_string(),
            path: PathBuf::from(format!("/w/docs/intent/{slice}/lld.md")),
            lines: text.lines().map(str::to_string).collect(),
        }
    }

    /// The number, counted from one, of a fixture text's line.
    fn line_at(text: &str, line: &str) -> usize {
        text.lines().position(|candidate| candidate == line).expect("the fixture holds that line") + 1
    }

    /// What a failure locates: its check, the file it is about, and the line
    /// it is on.
    fn located(failure: &Failure) -> (Check, &Path, usize) {
        (failure.check, failure.path.as_path(), failure.line)
    }

    /// Every failure, located, in the order they were reported.
    fn all_located(failures: &[Failure]) -> Vec<(Check, &Path, usize)> {
        failures.iter().map(located).collect()
    }

    /// A `cargo metadata` document for a workspace at `root` whose members
    /// are the directories `members`, each relative to it.
    fn metadata(root: &Path, members: &[&str]) -> String {
        let packages: Vec<String> = members
            .iter()
            .map(|member| {
                let manifest = root.join(member).join("Cargo.toml");
                format!(r#"{{"name":"m","manifest_path":"{}","targets":[{{"kind":["lib"],"name":"m"}}]}}"#, manifest.display())
            })
            .collect();
        format!(
            r#"{{"workspace_root":"{}","target_directory":"{}","packages":[{}]}}"#,
            root.display(),
            root.join("target").display(),
            packages.join(",")
        )
    }

    /// A project rooted at `root` with those members, without asking cargo.
    fn project_at(root: &Path, members: &[&str]) -> Project {
        Project::from_json(&metadata(root, members)).expect("the metadata document parses")
    }

    /// This workspace's root: the parent of this crate's manifest directory.
    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("the crate directory has a parent").to_path_buf()
    }

    /// Writes a file, creating the directories above it.
    fn write_at(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().expect("the path has a parent")).expect("create the directories");
        std::fs::write(path, content).expect("write the file");
    }

    /// This workspace's own synced guideline and reader, copied under a
    /// scratch root so that a scratch project's artifact checks read exactly
    /// what this project ships.
    fn install_artifacts(root: &Path) {
        for relative in [GUIDELINE, READER] {
            let content = std::fs::read_to_string(workspace_root().join(relative)).expect("this workspace's synced copy");
            write_at(&root.join(relative), &content);
        }
    }

    /// The reader's file, declaring `tools` in its frontmatter.
    fn reader_declaring(tools: &str) -> String {
        format!("---\nname: lid-rs-lld-review\ndescription: Reads one slice's LLD.\ntools: {tools}\n---\n\nYou read one slice's LLD.\n")
    }

    #[test]
    #[validates(spec::TheSliceIsTheFlagsValueOrTheBranchName)]
    fn the_slice_is_the_flags_value_or_the_branch_name() {
        let root = fixture::scratch("lld-review-slice");
        fixture::git(&root, &["init", "-q", "-b", "lld/hello"]);
        write_at(&root.join("docs/intent/hello/lld.md"), "# hello\n");
        let project = project_at(&root, &[""]);

        // Absent the flag, the branch `lld/hello` names the slice, and it is
        // hello's document that is read.
        let from_branch = slice_of(&project, parse_args(&[]).expect("no argument parses")).expect("the branch names a slice");
        let read = Lld::read(&project, &from_branch).expect("the scratch workspace holds hello's document");
        // The flag wins over the branch: the document looked for is `other`'s.
        let given = slice_of(&project, parse_args(&strings(&["--slice", "other"])).expect("the flag parses")).expect("the flag names the slice");
        let missing = Lld::read(&project, &given).expect_err("the scratch workspace holds no `other`");

        assert_eq!(
            (from_branch.as_str(), given.as_str(), read.slice.as_str(), read.path.as_path()),
            ("hello", "other", "hello", root.join("docs/intent/hello/lld.md").as_path())
        );
        assert_eq!(read.lines, ["# hello"]);
        assert!(missing.contains(&root.join("docs/intent/other/lld.md").display().to_string()), "{missing}");
    }

    #[test]
    #[validates(spec::AnyOtherArgumentIsRejectedByName)]
    fn any_other_argument_is_rejected_by_name() {
        // Through `run`, so the rejection is the one a command line meets.
        let unknown = run(&strings(&["--bogus"])).expect_err("an unknown flag is rejected");
        let stray = run(&strings(&["lld-review"])).expect_err("a bare argument is rejected");
        let nameless = run(&strings(&["--slice"])).expect_err("`--slice` without a name is rejected");
        // A flag with a value is not `--slice <name>` unless the flag is
        // `--slice`: it is rejected by name, not taken for the slice.
        let valued = run(&strings(&["--bogus", "x"])).expect_err("an unknown flag with a value is rejected");
        // And `--slice <name>` is not rejected: the run goes on to look for
        // that slice's document and fails on the document instead.
        let accepted = run(&strings(&["--slice", "no-such-slice"])).expect_err("no such slice has a document");

        let usage = [&unknown, &stray, &nameless, &valued].map(|rejection| rejection.contains(LLD_CHECK_USAGE));

        assert_eq!(
            (usage, unknown.contains("--bogus"), stray.contains("lld-review"), nameless.contains("--slice")),
            ([true; 4], true, true, true),
            "each rejection names the argument, and says how the subcommand is called:\n{unknown}\n{stray}\n{nameless}"
        );
        assert_eq!(
            (
                valued.contains("unknown argument `--bogus`"),
                valued.contains("docs/intent/x/lld.md"),
                nameless.contains("--slice requires a name"),
                nameless.contains("unknown argument"),
            ),
            (true, false, true, false),
            "a flag that is not `--slice` names no slice whatever follows it, and `--slice` itself is the flag, told to name a slice rather than rejected as unknown:\n{valued}\n{nameless}"
        );
        assert_eq!(
            (accepted.contains("unknown argument"), accepted.contains("docs/intent/no-such-slice/lld.md")),
            (false, true),
            "`--slice <name>` is not rejected: the run goes on to the document it names: {accepted}"
        );
    }

    #[test]
    #[validates(spec::TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt)]
    fn the_document_is_the_slices_lld_under_the_package_that_holds_it() {
        // One member, holding a slice in each layout: `inner`'s document is
        // still under `docs/intent`, and `moved-slice`'s sits beside its code.
        // Both are asked, because the document is wherever the layout puts it
        // and a test asking only the first could not tell that from a path
        // built out of the slice's name.
        let root = fixture::scratch("lld-review-document");
        write_at(&root.join("app/docs/intent/inner/lld.md"), "# inner\n\nThe inner slice.\n");
        write_at(&root.join("docs/intent/inner/lld.md"), "# a decoy at the workspace root\n");
        write_at(&root.join("app/src/moved_slice/mod.rs"), "//! The moved slice.\n");
        write_at(&root.join("app/src/moved_slice/lld.md"), "# moved-slice\n\nBeside the code.\n");
        write_at(&root.join("app/docs/intent/moved-slice/lld.md"), "# a decoy the migration left behind\n");
        let project = project_at(&root, &["app"]);

        let read = Lld::read(&project, "inner").expect("the member package holds it");
        let moved = Lld::read(&project, "moved-slice").expect("the member package holds it beside the code");
        // The pre-migration path holds a document of its own, so the answer
        // for `moved-slice` is the layout's and not the only file to be found.
        let left_behind = read_lines(&root.join("app/docs/intent/moved-slice/lld.md")).expect("the old path holds one too");

        assert_eq!(
            [read.path, moved.path],
            [root.join("app/docs/intent/inner/lld.md"), root.join("app/src/moved_slice/lld.md")],
            "the package that holds it and not the workspace root; beside the code and not under `docs/intent`"
        );
        assert_eq!(
            [read.lines, moved.lines, left_behind],
            [
                strings(&["# inner", "", "The inner slice."]),
                strings(&["# moved-slice", "", "Beside the code."]),
                strings(&["# a decoy the migration left behind"]),
            ],
            "each document read is the one at the path the layout answered"
        );
    }

    #[test]
    #[validates(spec::AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot)]
    fn a_workspace_only_slices_document_is_at_the_workspace_root() {
        // The member holds a slice in each layout — one still under
        // `docs/intent`, one already beside its code — because a mixed tree is
        // the migration's normal state, and `skill` is held by neither.
        let root = fixture::scratch("lld-review-workspace-only");
        write_at(&root.join("docs/intent/skill/lld.md"), "# skill\n");
        write_at(&root.join("app/docs/intent/other/lld.md"), "# other\n");
        write_at(&root.join("app/src/moved_slice/mod.rs"), "//! The moved slice.\n");
        write_at(&root.join("app/src/moved_slice/lld.md"), "# moved-slice\n");
        let project = project_at(&root, &["app"]);

        let read = Lld::read(&project, "skill").expect("no member holds it, so the root does");

        assert_eq!((read.slice.as_str(), read.path.as_path()), ("skill", root.join("docs/intent/skill/lld.md").as_path()));
    }

    #[test]
    #[validates(spec::AnUnreadableLldFailsNamingItsPath)]
    fn an_unreadable_lld_fails_naming_its_path() {
        let root = fixture::scratch("lld-review-unreadable");
        let project = project_at(&root, &["app"]);
        std::fs::create_dir_all(root.join("docs/intent/a-directory/lld.md")).expect("a directory where a document should be");
        std::fs::create_dir_all(root.join("docs/intent/not-text")).expect("the directories");
        std::fs::write(root.join("docs/intent/not-text/lld.md"), [0xff_u8, 0xfe, 0xff]).expect("bytes that are not text");
        // A fourth, beside the code in the member: the document the layout
        // answers for a migrated slice is there and is unreadable, so the path
        // the failure names is that one and no path built from the slice's
        // name.
        write_at(&root.join("app/src/moved/mod.rs"), "//! The moved slice.\n");
        std::fs::write(root.join("app/src/moved/lld.md"), [0xff_u8, 0xfe, 0xff]).expect("bytes that are not text");

        let absent = Lld::read(&project, "absent").expect_err("there is no such document");
        let directory = Lld::read(&project, "a-directory").expect_err("a directory is not a document");
        let not_text = Lld::read(&project, "not-text").expect_err("bytes that are not text are not a document");
        let moved = Lld::read(&project, "moved").expect_err("bytes beside the code are not a document either");

        assert_eq!(
            [
                absent.contains(&root.join("docs/intent/absent/lld.md").display().to_string()),
                directory.contains(&root.join("docs/intent/a-directory/lld.md").display().to_string()),
                not_text.contains(&root.join("docs/intent/not-text/lld.md").display().to_string()),
                moved.contains(&root.join("app/src/moved/lld.md").display().to_string()),
                moved.contains("docs/intent"),
            ],
            [true, true, true, true, false],
            "each failure names the path the layout answered, and the migrated slice's names that one alone:\n{absent}\n{directory}\n{not_text}\n{moved}"
        );
    }

    #[test]
    #[validates(spec::LldCheckExitsZeroOnlyWhenEveryCheckHolds)]
    fn lld_check_exits_zero_only_when_every_check_holds() {
        // This slice's own document, under this workspace's own synced
        // artifacts: every check holds, and the run exits zero.
        let project = project_at(&workspace_root(), &["cargo-lid-rs"]);
        let own = Lld::read(&project, "lld-review").expect("this slice's own document");
        let holds = check_all(&project, &own).expect("the root is locatable");
        let failing = check_all(&project, &document("s", FAILS_THREE)).expect("the root is locatable");

        assert!(holds.is_empty(), "this slice's own document holds every check: {holds:?}");
        assert_eq!(
            (report(&holds), failing.len(), report(&failing).is_err()),
            (Ok(()), 3, true),
            "every check holding exits zero; a document that fails three does not: {failing:?}"
        );
    }

    #[test]
    #[validates(spec::EveryFailureIsReportedNotOnlyTheFirst)]
    fn every_failure_is_reported_not_only_the_first() {
        // A document failing three checks, in a project holding neither
        // artifact: five failures, and the report carries all five.
        let root = fixture::scratch("lld-review-every-failure");
        let project = project_at(&root, &[]);
        let doc = document("s", FAILS_THREE);
        let failures = check_all(&project, &doc).expect("the root is locatable");
        let checks: Vec<Check> = failures.iter().map(|failure| failure.check).collect();

        assert_eq!(
            checks,
            [Check::DecisionsExist, Check::ShapeRows, Check::DeferredNumbered, Check::GuidelineNamesEveryCheck, Check::ReaderObservesOnly]
        );
        let reported = report(&failures).expect_err("five failures");
        let named = failures
            .iter()
            .filter(|failure| reported.contains(&format!("{:?}", failure.check)) && reported.contains(&failure.path.display().to_string()))
            .count();
        assert_eq!(named, failures.len(), "every failure is named, not only the first: {reported}");
        assert!(reported.lines().count() >= failures.len(), "one failure to a line: {reported}");
    }

    #[test]
    #[validates(spec::AFailureNamesItsCheckItsFileItsLineAndItsRule)]
    fn a_failure_names_its_check_its_file_its_line_and_its_rule() {
        let doc = document("s", FAILS_THREE);
        let failures = shape_rows(&doc);

        assert_eq!(
            all_located(&failures),
            [(Check::ShapeRows, doc.path.as_path(), line_at(FAILS_THREE, "| no identifier | a note |"))]
        );
        // The message quotes the sentence the skill states that rule in.
        let anchors = [
            (Check::DecisionsExist, "Decisions & Alternatives"),
            (Check::Alternatives, "four non-empty cells"),
            (Check::ShapeRows, "backticked identifier"),
            (Check::DeferredNumbered, "numbered list item"),
            (Check::GuidelineNamesEveryCheck, "every check"),
            (Check::ReaderObservesOnly, "nothing else"),
        ];
        let quoting = anchors.iter().filter(|(check, anchor)| rule(*check).contains(anchor)).count();
        let rules: BTreeSet<&str> = EVERY_CHECK.iter().map(|check| rule(*check)).collect();

        assert_eq!(
            (failures[0].message.contains("backticked identifier"), quoting, rules.len()),
            (true, anchors.len(), EVERY_CHECK.len()),
            "each check's rule is the skill's own sentence for it, and no two checks share one: {rules:?}"
        );
    }

    #[test]
    #[validates(spec::ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator)]
    fn a_table_is_the_rows_under_its_heading_less_header_and_separator() {
        let doc = document("s", TWO_TABLES);
        let shape = table_at(&doc, SHAPE_HEADING).expect("the shape table");
        let decisions = table_at(&doc, DECISIONS_HEADING).expect("the decisions table");

        // The header row and its separator are not rows, and the next
        // heading ends the table: the decisions rows are not the shape's.
        assert_eq!(
            shape.rows,
            [
                Row { line: line_at(TWO_TABLES, "| `a` | first |"), cells: strings(&["`a`", "first"]) },
                Row { line: line_at(TWO_TABLES, "| `b` | second |"), cells: strings(&["`b`", "second"]) },
            ]
        );
        assert_eq!(
            decisions.rows,
            [Row { line: line_at(TWO_TABLES, "| what | this | that | because |"), cells: strings(&["what", "this", "that", "because"]) }]
        );
        assert_eq!(
            (table_at(&doc, "## Behaviour"), table_at(&doc, "## Nowhere")),
            (None, None),
            "a heading with no table under it, and a heading the document does not have"
        );
    }

    #[test]
    #[validates(spec::ADocumentWithoutADecisionsTableFails)]
    fn a_document_without_a_decisions_table_fails() {
        let prose = document("s", DECISIONS_IN_PROSE);
        let missing = document("s", FAILS_THREE);
        let under_the_heading = decisions_exist(&prose);
        let without_a_heading = decisions_exist(&missing);

        assert_eq!(all_located(&under_the_heading), [(Check::DecisionsExist, prose.path.as_path(), FIRST_LINE)]);
        assert_eq!(all_located(&without_a_heading), [(Check::DecisionsExist, missing.path.as_path(), FIRST_LINE)]);
        assert!(decisions_exist(&document("s", HOLDS)).is_empty(), "a heading with a table under it holds");
    }

    #[test]
    #[validates(spec::EveryDecisionsRowFillsItsFourCells)]
    fn every_decisions_row_fills_its_four_cells() {
        let doc = document("s", DECISIONS_ROWS);
        let failures = alternatives(&doc);

        assert_eq!(
            all_located(&failures),
            [
                (Check::Alternatives, doc.path.as_path(), line_at(DECISIONS_ROWS, "| three | cells | only |")),
                (Check::Alternatives, doc.path.as_path(), line_at(DECISIONS_ROWS, "| empty |  | that | because |")),
            ],
            "the filled row holds; a short row and an empty cell each fail on their own line"
        );
        assert!(alternatives(&document("s", HOLDS)).is_empty(), "a table whose every row is filled holds");
        assert!(alternatives(&document("s", FAILS_THREE)).is_empty(), "a document with no such table has no row to fill");
    }

    #[test]
    #[validates(spec::EveryShapeRowNamesAnIdentifierAndARole)]
    fn every_shape_row_names_an_identifier_and_a_role() {
        let doc = document("s", SHAPE_ROWS);
        let failures = shape_rows(&doc);

        assert_eq!(
            all_located(&failures),
            [
                (Check::ShapeRows, doc.path.as_path(), line_at(SHAPE_ROWS, "| plain prose | a note |")),
                (Check::ShapeRows, doc.path.as_path(), line_at(SHAPE_ROWS, "| `Lld` |  |")),
            ],
            "a row with no identifier and a row with no role each fail on their own line"
        );
        assert_eq!(identifiers("`Lld`, `Lld::read(project, slice)`"), ["Lld", "Lld::read(project, slice)"]);
        assert!(identifiers("plain prose").is_empty());
    }

    #[test]
    #[validates(spec::ADocumentWithNoShapeTableHoldsThatCheck)]
    fn a_document_with_no_shape_table_holds_that_check() {
        // No shape heading at all, and a shape heading whose shape is prose:
        // a slice may name its shape either way.
        assert!(shape_rows(&document("s", DECISIONS_ROWS)).is_empty(), "no shape heading");
        assert!(shape_rows(&document("s", SHAPE_IN_PROSE)).is_empty(), "a shape named in prose under its heading");
        assert!(!shape_rows(&document("s", SHAPE_ROWS)).is_empty(), "a shape table is still checked");
    }

    #[test]
    #[validates(spec::EveryDeferredItemIsANumberedListItem)]
    fn every_deferred_item_is_a_numbered_list_item() {
        let doc = document("s", DEFERRED_ITEMS);
        let failures = deferred_numbered(&doc);

        assert_eq!(
            all_located(&failures),
            [
                (Check::DeferredNumbered, doc.path.as_path(), line_at(DEFERRED_ITEMS, "- an unnumbered one")),
                (Check::DeferredNumbered, doc.path.as_path(), line_at(DEFERRED_ITEMS, "* a second unnumbered one")),
            ],
            "the numbered item, its continuation line, and a bullet under a later heading are not failures"
        );
    }

    #[test]
    #[validates(spec::ADocumentWithNoDeferredHeadingHoldsThatCheck)]
    fn a_document_with_no_deferred_heading_holds_that_check() {
        // Bullets under some other heading are nobody's to number.
        assert!(deferred_numbered(&document("s", BULLETS_ELSEWHERE)).is_empty());
        assert!(!deferred_numbered(&document("s", DEFERRED_ITEMS)).is_empty(), "a deferred heading is still checked");
    }

    #[test]
    #[validates(spec::EveryCheckIsNamedInTheGuidelinesChecklist)]
    fn every_check_is_named_in_the_guidelines_checklist() {
        let root = fixture::scratch("lld-review-checklist");
        let project = project_at(&root, &[]);
        install_artifacts(&root);
        let shipped = guideline_names_every_check(&project).expect("the root is locatable");

        // Four of six named in the checklist; the other two named only in
        // the questions, which are not the checklist.
        write_at(&root.join(GUIDELINE), PARTIAL_CHECKLIST);
        let failures = guideline_names_every_check(&project).expect("the root is locatable");
        let guideline = root.join(GUIDELINE);

        assert!(shipped.is_empty(), "this workspace's own guideline names every check: {shipped:?}");
        assert_eq!(
            all_located(&failures),
            [(
                Check::GuidelineNamesEveryCheck,
                guideline.as_path(),
                line_at(PARTIAL_CHECKLIST, "## The checklist — what the tool refuses")
            )]
        );
        assert!(failures[0].message.starts_with("`ShapeRows`, `ReaderObservesOnly`"), "it names what the checklist omits: {}", failures[0].message);
    }

    #[test]
    #[validates(spec::TheReaderDeclaresOnlyTheObservationTools)]
    fn the_reader_declares_only_the_observation_tools() {
        let root = fixture::scratch("lld-review-reader");
        let project = project_at(&root, &[]);
        let reader = root.join(READER);
        install_artifacts(&root);
        let shipped = reader_observes_only(&project).expect("the root is locatable");

        write_at(&reader, &reader_declaring("Glob, Read, Grep"));
        let reordered = reader_observes_only(&project).expect("the root is locatable");

        let extra = reader_declaring("Read, Grep, Glob, Edit");
        write_at(&reader, &extra);
        let with_edit = reader_observes_only(&project).expect("the root is locatable");

        write_at(&reader, &reader_declaring("Read, Grep"));
        let short = reader_observes_only(&project).expect("the root is locatable");

        assert_eq!(
            all_located(&with_edit),
            [(Check::ReaderObservesOnly, reader.as_path(), line_at(&extra, "tools: Read, Grep, Glob, Edit"))]
        );
        assert_eq!(
            (shipped.len(), reordered.len(), short.len(), with_edit[0].message.starts_with("Read, Grep, Glob, Edit")),
            (0, 0, 1, true),
            "the shipped reader holds, and so does a reordered declaration; a tool too many is named, and a tool missing fails too:\n{shipped:?}\n{reordered:?}\n{short:?}\n{}",
            with_edit[0].message
        );
    }

    #[test]
    #[validates(spec::AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine)]
    fn an_artifact_failure_with_no_line_to_cite_points_at_the_first_line() {
        // A guideline with no checklist heading, and a reader with no
        // `tools:` line: neither failure has a line of its own to cite.
        let root = fixture::scratch("lld-review-no-line");
        let project = project_at(&root, &[]);
        write_at(&root.join(GUIDELINE), "# Writing an LLD\n\nNo checklist heading, so no check is named.\n");
        write_at(&root.join(READER), "---\nname: lid-rs-lld-review\n---\n\nIt declares no tools at all.\n");

        let guideline = guideline_names_every_check(&project).expect("the root is locatable");
        let reader = reader_observes_only(&project).expect("the root is locatable");

        // The same two checks over artifacts that do hold such a line: each
        // failure is on the line it read, so the first line is the fallback
        // when there is none rather than the answer either way.
        let declaring = reader_declaring("Read, Grep, Glob, Edit");
        write_at(&root.join(GUIDELINE), PARTIAL_CHECKLIST);
        write_at(&root.join(READER), &declaring);
        let cited = guideline_names_every_check(&project).expect("the root is locatable");
        let cited_reader = reader_observes_only(&project).expect("the root is locatable");

        assert_eq!(
            (all_located(&guideline), all_located(&reader)),
            (
                vec![(Check::GuidelineNamesEveryCheck, root.join(GUIDELINE).as_path(), FIRST_LINE)],
                vec![(Check::ReaderObservesOnly, root.join(READER).as_path(), FIRST_LINE)]
            ),
            "neither failure has a line of its own to cite, so each points at its file's first line"
        );
        assert_eq!(
            (cited[0].line, cited_reader[0].line, cited[0].line == FIRST_LINE, cited_reader[0].line == FIRST_LINE),
            (
                line_at(PARTIAL_CHECKLIST, "## The checklist — what the tool refuses"),
                line_at(&declaring, "tools: Read, Grep, Glob, Edit"),
                false,
                false
            ),
            "a marker that is there is cited on its own line, which is not the first"
        );
        assert!(guideline[0].message.starts_with("`DecisionsExist`"), "a checklist that is not there names none of them: {}", guideline[0].message);
    }

    #[test]
    #[validates(spec::TheArtifactChecksRunWhateverSliceIsNamed)]
    fn the_artifact_checks_run_whatever_slice_is_named() {
        // A project holding neither artifact, and two slices' documents that
        // hold every document check.
        let root = fixture::scratch("lld-review-whatever-slice");
        let project = project_at(&root, &[]);
        let alpha = document("alpha", HOLDS);
        let beta = document("beta", HOLDS);

        let for_alpha = check_all(&project, &alpha).expect("the root is locatable");
        let for_beta = check_all(&project, &beta).expect("the root is locatable");
        let checks: Vec<Check> = for_alpha.iter().map(|failure| failure.check).collect();

        assert_eq!(for_alpha, for_beta, "the two artifact checks do not depend on the slice named");
        assert_eq!(checks, [Check::GuidelineNamesEveryCheck, Check::ReaderObservesOnly]);
        assert!(
            for_alpha.iter().all(|failure| failure.path != alpha.path && failure.path.starts_with(&root)),
            "each names the artifact it read, not the document under check: {for_alpha:?}"
        );
    }

    #[test]
    #[validates(spec::AnUnreadableSyncedArtifactFailsNamingItsPath)]
    fn an_unreadable_synced_artifact_fails_naming_its_path() {
        let root = fixture::scratch("lld-review-unreadable-artifact");
        let project = project_at(&root, &[]);
        let guideline = root.join(GUIDELINE);
        let reader = root.join(READER);
        std::fs::create_dir_all(&reader).expect("a directory where the reader should be");

        // Absent, and unreadable: a failure reported beside the others,
        // never an error that hides the checks after it.
        let absent = guideline_names_every_check(&project).expect("an absent copy is a failure, not an error");
        let unreadable = reader_observes_only(&project).expect("an unreadable copy is a failure, not an error");

        assert_eq!(all_located(&absent), [(Check::GuidelineNamesEveryCheck, guideline.as_path(), FIRST_LINE)]);
        assert_eq!(all_located(&unreadable), [(Check::ReaderObservesOnly, reader.as_path(), FIRST_LINE)]);
        assert_eq!(
            (absent[0].message.contains(&guideline.display().to_string()), unreadable[0].message.contains(&reader.display().to_string())),
            (true, true),
            "each names the path it looked for:\n{}\n{}",
            absent[0].message,
            unreadable[0].message
        );
    }
}
