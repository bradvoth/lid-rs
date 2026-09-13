#![doc = include_str!("lld.md")]

use std::collections::BTreeSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::layout;
use crate::phase::resolve_slice;
use crate::project::Project;

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
pub const GUIDELINE: &str = ".claude/skills/lid-rs/references/lld.md";

/// The project's synced copy of the reader, relative to the workspace root.
pub const READER: &str = ".claude/agents/lid-rs-lld-review.md";

/// The tools an advisory reader may declare: it observes, and cannot act.
const OBSERVATION_TOOLS: [&str; 3] = ["Read", "Grep", "Glob"];

/// The type tokens a receiver stands for in the reading of a declaration: the
/// four shapes a `self` parameter is answered as, since the reading says what
/// type each parameter is and never which of them was written `self`.
const RECEIVER_TOKENS: [&str; 4] = ["Self", "&Self", "&mut Self", "Box<Self>"];

/// The spellings a shape row writes a receiver as, which are not arguments:
/// the tables use both conventions — a row that writes the receiver and a row
/// that leaves it out — and neither is wrong.
const WRITTEN_RECEIVERS: [&str; 3] = ["self", "&self", "&mut self"];

/// Every check, in the order the LLD's table states them: the seven over the
/// document — five of its text alone, two reading beside it — then the two
/// over the project's synced artifacts.
const EVERY_CHECK: [Check; 9] = [
    Check::DecisionsExist,
    Check::Alternatives,
    Check::ShapeRows,
    Check::ShapeAgrees,
    Check::SkeletonableReturns,
    Check::ReuseRowsLinked,
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

/// The character a markdown span of code is delimited by, which is how a cell
/// says that what lies between two of them is an identifier.
const BACKTICK: char = '`';

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
    /// Where a `## Shape` row's first cell writes a signature fragment for a
    /// function the slice's own source declares, the fragment's argument count
    /// is that function's parameter count, and a return it writes is the
    /// source's.
    ShapeAgrees,
    /// A `## Shape` row's return fragment is a type a layer-0 skeleton can be
    /// written for: not an `impl Trait` return, and not a bare `dyn Trait` one.
    SkeletonableReturns,
    /// A first cell's fragment that names another slice's item in the same
    /// crate is written as an intra-doc link, whose resolution is rustdoc's.
    ReuseRowsLinked,
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

/// A failure over a shape row the source disagrees with — the one shape a
/// source-reading failure takes, beside [`failure`] and [`artifact_failure`]:
/// the check, the document's path and the row's line as every failure carries
/// them, and a message naming the item's file, what the row said and what the
/// source said, in that order, before the sentence the skill states the rule
/// in. The row is what a human is about to fix and the source is the evidence
/// they fix it against; the two are in different files and a failure carries
/// one path, so the document's is the path and the source's is in the sentence.
#[implements(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
fn agreement_failure(check: Check, path: &Path, line: usize, item_file: &Path, row: &str, source: &str) -> Failure {
    todo!("agreement_failure({check:?}, {}, {line}, {}, {row}, {source})", path.display(), item_file.display())
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
        Check::ShapeAgrees => {
            "where a `## Shape` row's first cell writes a signature fragment for a function the slice's own source declares, the fragment's argument count is that function's parameter count, and a return it writes is the one the source wrote"
        }
        Check::SkeletonableReturns => {
            "a `## Shape` row's return fragment is not an `impl Trait` return and not a bare `dyn Trait` one"
        }
        Check::ReuseRowsLinked => {
            "a first cell's fragment that names another slice's item in the same crate — a lowercase-module-qualified path that is not the slice's own, a sibling's, or a crate's — is written as an intra-doc link"
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
pub fn rendered(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(|failure| format!("{}:{}: {:?}: {}", failure.path.display(), failure.line, failure.check, failure.message))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Every check over one document, in the table's order — which is the order
/// [`Check`] declares its variants: the seven document checks against `lld`,
/// two of which read beside it (the slice's own source, and the listing of its
/// directory), then the two artifact checks, which read the project's synced
/// copies and so run whatever slice was named. Every failure is collected — no
/// check is skipped because an earlier one failed. The project is a parameter
/// beside the document because the artifact checks are about files the
/// document does not name, and the source-reading checks about a crate it does
/// not name; the error is reserved for a project whose root cannot be located
/// — an unreadable artifact is a [`Failure`] like any other, and a slice whose
/// crate cannot be located declares nothing and lists nothing.
#[implements(
    spec::EveryFailureIsReportedNotOnlyTheFirst,
    spec::TheArtifactChecksRunWhateverSliceIsNamed,
    spec::TheChecksRunInTheOrderTheTableStatesThem,
)]
pub fn check_all(project: &Project, lld: &Lld) -> Result<Vec<Failure>, String> {
    Ok([
        decisions_exist(lld),
        alternatives(lld),
        shape_rows(lld),
        shape_agrees(project, lld),
        shape_returns(lld),
        reuse_rows_linked(project, lld),
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
    rows_without_identifier_or_role(lld, &shape_table_rows(lld))
}

/// The rows of the `## Shape` table, and none when the document has no such
/// table — the one place that decision is made. Four checks read the shape
/// rows, and a document that names its shape in prose instead holds every one
/// of them for this reason and no other.
#[implements(spec::ADocumentWithNoShapeTableHoldsThatCheck)]
fn shape_table_rows(lld: &Lld) -> Vec<Row> {
    match table_at(lld, SHAPE_HEADING) {
        Some(table) => table.rows,
        None => vec![],
    }
}

/// The shape rows that name no identifier or give no role, each failing on its
/// own line.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
fn rows_without_identifier_or_role(lld: &Lld, rows: &[Row]) -> Vec<Failure> {
    rows.iter()
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

/// The backticked identifiers a cell names: the text of each of its [`spans`],
/// in the order the cell writes them.
#[implements(spec::EveryShapeRowNamesAnIdentifierAndARole)]
pub fn identifiers(cell: &str) -> Vec<String> {
    spans(cell).into_iter().map(|span| cell[span].to_string()).collect()
}

/// The backtick spans of a cell, in document order: the text between the first
/// backtick and the second, the third and the fourth, and so on, as ranges into
/// the cell — a span never closed running to the cell's end.
///
/// The one reading of a cell's backticks. [`identifiers`] answers what is
/// inside them and [`linked`] answers what surrounds them, so the nth
/// identifier and the nth span are the same span rather than two readings
/// agreeing by luck.
#[implements(
    spec::EveryShapeRowNamesAnIdentifierAndARole,
    spec::AFragmentIsLinkedOnlyWhenItsBacktickSpanIsWrappedInAMarkdownLink,
)]
fn spans(cell: &str) -> Vec<Range<usize>> {
    cell.split(BACKTICK)
        .scan(0, |start, part| {
            let span = *start..*start + part.len();
            *start += part.len() + BACKTICK.len_utf8();
            Some(span)
        })
        .skip(1)
        .step_by(2)
        .collect()
}

/// One function the slice's own source declares, as the shape pass read it:
/// the tokens the source wrote, never what a `use` or an alias would turn them
/// into. Data, so no claim is cited here: the claims about this shape are
/// [`declared`]'s, which answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// The function's name, as the source wrote it.
    pub name: String,
    /// For a method, the block it was read from — the self type of an `impl`,
    /// or the name of the `trait` that gave the method a body; none for a free
    /// function, whose `Self` then stands for nothing.
    pub owner: Option<String>,
    /// The parameters the reading answered, as type tokens, less a first one
    /// whose tokens are the receiver's — `Self`, `&Self`, `&mut Self` or
    /// `Box<Self>` — so that every count taken from this list is a count of
    /// arguments.
    pub parameters: Vec<String>,
    /// The return tokens as the source wrote them, whole; none for a function
    /// declared with no `->` at all.
    pub returns: Option<String>,
    /// The file the function's tokens were read from.
    pub file: PathBuf,
}

/// One signature fragment a first cell writes: `name(a, b, c)` or
/// `Owner::name(a, b, c)`, with an optional `-> Type`, or a bare type name.
/// Data, so no claim is cited here: the claims about this shape are
/// [`fragments`]'s, which answers it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    /// The `::`-separated segments of the identifier, in order.
    pub path: Vec<String>,
    /// The arguments between the parentheses when the fragment writes them,
    /// less a first one written `self`, `&self` or `&mut self`; none when it
    /// writes no parentheses, which names a type rather than a function.
    pub arguments: Option<Vec<String>>,
    /// The type after an ASCII `->` when the fragment writes one; a Unicode `→`
    /// writes no return.
    pub returns: Option<String>,
    /// Whether the cell wrapped the backtick span in a markdown link — the span
    /// immediately preceded by `[` and immediately followed by `](`.
    pub linked: bool,
}

/// The functions the slice's own source declares, read through
/// [`lid_rs_shape::signatures`] over the slice's own crate,
/// [`layout::own_crate`](crate::layout::own_crate), and kept where the file is
/// under [`layout::slice_dir`](crate::layout::slice_dir) or is the
/// `src/<module>.rs` file module beside it — the `<module>` being the slice's
/// name as [`layout::module_of`](crate::layout::module_of) spells it, the only
/// conversion a path ever gets. Each is carried with its receiver, if any,
/// dropped from its parameters. No `Result`: a slice no workspace member holds
/// a crate for declares nothing, and a file the reading cannot parse
/// contributes nothing, and neither is a failure.
///
/// The two doors refuse together, which is why one answer stands for both: each
/// is [`Form::of_slice`](crate::layout::Form::of_slice)'s, that shape is
/// documented never to be `Companion` — the one shape with a directory and no
/// crate of its own — and the `NoCrate` it answers for a slice no member holds
/// a document for is exactly the shape with neither. So a refusal here is
/// always *this slice has no crate*, never half an answer.
#[implements(
    spec::TheDeclaredFunctionsAreTheSlicesOwnCratesUnderItsDirectory,
    spec::ASliceNoCrateHoldsDeclaresNoFunction,
    spec::ADeclaredMethodsReceiverIsNotAmongItsParameters,
)]
pub fn declared(project: &Project, slice: &str) -> Vec<Declared> {
    let (Ok(own_crate), Ok(dir)) = (layout::own_crate(project, slice), layout::slice_dir(project, slice)) else {
        return vec![];
    };
    slice_declarations(&own_crate, &dir, &layout::module_of(slice))
}

/// The functions one crate's reading declares for the slice: every signature
/// [`lid_rs_shape::signatures`] answers over that crate, kept where its file is
/// the slice's, each translated into this slice's own [`Declared`]. The
/// library's type goes no further than [`declared_from`], so no interior
/// function of this slice takes a `lid_rs_shape::Signature`.
#[implements(spec::TheDeclaredFunctionsAreTheSlicesOwnCratesUnderItsDirectory)]
fn slice_declarations(own_crate: &Path, dir: &Path, module: &str) -> Vec<Declared> {
    lid_rs_shape::signatures(own_crate)
        .into_iter()
        .filter(|signature| in_the_slice(&signature.file, dir, module))
        .map(declared_from)
        .collect()
}

/// Whether a declaration's file is the slice's own: under the slice's
/// directory, read through, or the `src/<module>.rs` file module beside it,
/// since a slice whose code is one file keeps it there and the directory may
/// not exist at all.
#[implements(spec::TheDeclaredFunctionsAreTheSlicesOwnCratesUnderItsDirectory)]
fn in_the_slice(file: &Path, dir: &Path, module: &str) -> bool {
    todo!("in_the_slice({}, {}, {module})", file.display(), dir.display())
}

/// One signature the shape pass read, as the [`Declared`] this slice carries:
/// the reading's own name, owner, return tokens and file, and its parameters
/// less a first one whose type tokens are the receiver's, so that every count
/// taken from them afterwards is a count of arguments. The one place a
/// `lid_rs_shape::Signature` is turned into this slice's own data, which is
/// what keeps the library at the boundary.
#[implements(spec::ADeclaredMethodsReceiverIsNotAmongItsParameters)]
fn declared_from(signature: lid_rs_shape::Signature) -> Declared {
    Declared {
        name: signature.function,
        owner: signature.owner,
        parameters: less_receiver(&signature.parameters),
        returns: signature.returns,
        file: signature.file,
    }
}

/// A declaration's parameters as this slice counts them: the reading's own,
/// less a first one that is a receiver. The reading says what type each
/// parameter stands for and never which of them was written `self`, so the
/// first parameter's tokens are the whole of the evidence.
#[implements(spec::ADeclaredMethodsReceiverIsNotAmongItsParameters)]
fn less_receiver(parameters: &[String]) -> Vec<String> {
    match parameters.first() {
        Some(first) if is_receiver(first) => parameters[1..].to_vec(),
        Some(_) | None => parameters.to_vec(),
    }
}

/// Whether a parameter's type tokens are a receiver's: one of the
/// [`RECEIVER_TOKENS`], both sides [`stripped`] as every comparison of tokens
/// here is — the reading renders them with the spacing its printer chose, and
/// the constant is written the way a human writes them.
///
/// The lifetime half of that reading is not decoration: a receiver written
/// `&'a self` is answered as the type it stands for, `& 'a Self`, and comparing
/// that with whitespace alone taken out would leave a method's receiver counted
/// as one of its arguments.
#[implements(spec::ADeclaredMethodsReceiverIsNotAmongItsParameters)]
fn is_receiver(parameter: &str) -> bool {
    RECEIVER_TOKENS.iter().any(|receiver| stripped(receiver) == stripped(parameter))
}

/// Every signature fragment one cell writes, one per backticked identifier and
/// as many as the cell holds: the identifier's `::`-separated segments; the
/// arguments between its parentheses when it writes them, less a first one
/// written `self`, `&self` or `&mut self`, and none at all when it writes no
/// parentheses; the type after an ASCII `->` when it writes one, a Unicode `→`
/// writing no return; and whether the cell wrapped the span in a markdown link
/// — immediately preceded by `[` and immediately followed by `](` — since any
/// other neighbour means bare.
#[implements(
    spec::EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment,
    spec::AReturnIsWrittenWithTheAsciiArrowAlone,
    spec::AFirstArgumentWrittenAsAReceiverIsNotAFragmentsArgument,
    spec::AFragmentIsLinkedOnlyWhenItsBacktickSpanIsWrappedInAMarkdownLink,
)]
pub fn fragments(cell: &str) -> Vec<Fragment> {
    identifiers(cell).iter().enumerate().map(|(nth, identifier)| fragment_of(cell, nth, identifier)).collect()
}

/// One backticked identifier as the fragment it writes, carrying the cell and
/// which span of it the identifier came from, because the link test is about
/// that span's neighbours rather than about the identifier's text.
#[implements(spec::EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment)]
fn fragment_of(cell: &str, nth: usize, identifier: &str) -> Fragment {
    Fragment {
        path: path_segments(identifier),
        arguments: arguments(identifier),
        returns: written_return(identifier),
        linked: linked(cell, nth),
    }
}

/// The `::`-separated segments of an identifier's name: what it writes before
/// its parentheses, or the whole of it where it writes none.
#[implements(spec::EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment)]
fn path_segments(identifier: &str) -> Vec<String> {
    todo!("path_segments({identifier})")
}

/// The arguments an identifier writes: those between its parentheses, less a
/// first one written as a receiver, and none at all where it writes no
/// parentheses, since such an identifier names a type rather than a function.
#[implements(spec::AFirstArgumentWrittenAsAReceiverIsNotAFragmentsArgument)]
fn arguments(identifier: &str) -> Option<Vec<String>> {
    Some(without_receiver(&argument_list(identifier)?))
}

/// The comma-separated arguments between an identifier's parentheses, each
/// trimmed, and none at all where it writes no parentheses. Empty parentheses
/// hold an empty list and not an absent one: `run()` is a function of no
/// arguments, which agrees with a declaration that takes none, where a
/// fragment that writes no parentheses at all names a type and is compared to
/// nothing.
#[implements(spec::EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment)]
fn argument_list(identifier: &str) -> Option<Vec<String>> {
    todo!("argument_list({identifier})")
}

/// An argument list less a first argument written as a receiver — one of the
/// [`WRITTEN_RECEIVERS`] — so that what is left is a list of arguments however
/// the row's author spelled the method.
#[implements(spec::AFirstArgumentWrittenAsAReceiverIsNotAFragmentsArgument)]
fn without_receiver(written: &[String]) -> Vec<String> {
    match written.first() {
        Some(first) if WRITTEN_RECEIVERS.contains(&first.as_str()) => written[1..].to_vec(),
        Some(_) | None => written.to_vec(),
    }
}

/// The return an identifier writes: the type after an ASCII `->`, and none at
/// all where it writes no such arrow — which is what makes a Unicode `→` a row
/// with no return rather than a row with a return spelled another way.
#[implements(spec::AReturnIsWrittenWithTheAsciiArrowAlone)]
fn written_return(identifier: &str) -> Option<String> {
    todo!("written_return({identifier})")
}

/// Whether the cell wrapped one of its backtick spans in a markdown link: the
/// nth of the cell's [`spans`], immediately preceded by `[` and immediately
/// followed by `](`, any other neighbour meaning bare. The span is taken by
/// position and never by searching the cell for the identifier's text, so a
/// cell that writes one identifier twice — once as a link and once bare —
/// answers for each fragment where that fragment is, and a span that is not
/// there at all is not linked.
///
/// A span can end at the cell's last byte — [`spans`] runs an unclosed one to
/// the end — so the byte after it is a byte that may not exist, and the
/// neighbour after the span is asked for rather than indexed.
#[implements(spec::AFragmentIsLinkedOnlyWhenItsBacktickSpanIsWrappedInAMarkdownLink)]
fn linked(cell: &str, nth: usize) -> bool {
    todo!("linked({cell}, {nth})")
}

/// The first cells whose fragment names a function the slice declares and
/// agrees with none of the declarations it names, each failing on its own
/// row's line. A fragment writing no parentheses names a type and is compared
/// to nothing; a fragment naming no declared function is compared to none and
/// holds — at Phase 1 nothing is declared yet, at Phase 3 half of it is, and a
/// reuse row names an item another slice owns — which is why the check needs
/// no phase told to it. A document with no `## Shape` table has no row to
/// compare, as `shape_table_rows` decides.
#[implements(
    spec::AFragmentWithoutParenthesesIsComparedToNothing,
    spec::AFragmentNamingNoDeclaredFunctionIsComparedToNone,
    spec::AFragmentAgreeingWithAnyDeclarationItNamesHolds,
)]
pub fn shape_agrees(project: &Project, lld: &Lld) -> Vec<Failure> {
    let declarations = declared(project, &lld.slice);
    shape_table_rows(lld).iter().flat_map(|row| row_disagreements(lld, row, &declarations)).collect()
}

/// The first cell of a row, and the empty cell for a row with none: the cell
/// every check of a `## Shape` row reads, since a row is about what its first
/// cell names and its role cell legitimately names anything, including the
/// crate that does the reading.
#[implements(
    spec::AFragmentAgreeingWithAnyDeclarationItNamesHolds,
    spec::AReuseRowNamesItsItemAsAnIntraDocLink,
    spec::ARowWhoseReturnCannotBeSkeletonisedFailsOnItsLine,
)]
fn first_cell(row: &Row) -> &str {
    row.cells.first().map_or("", String::as_str)
}

/// One row's first cell as the pairs an agreement failure needs: each
/// backticked identifier as the cell wrote it, beside the [`Fragment`] read
/// from it — [`fragments`] answers one per identifier in the order the cell
/// holds them, which is the order [`identifiers`] reads them in, so the two
/// readings pair off.
#[implements(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
fn written_fragments(row: &Row) -> Vec<(String, Fragment)> {
    let cell = first_cell(row);
    identifiers(cell).into_iter().zip(fragments(cell)).collect()
}

/// The failures one row earns: each fragment of its first cell that names a
/// declared function and agrees with none of the declarations it names, as the
/// failure on that row's line naming the item's file, what the row said and
/// what the source said.
#[implements(
    spec::AFragmentAgreeingWithAnyDeclarationItNamesHolds,
    spec::AnAgreementFailureNamesTheItemsFileAndBothReadings,
)]
fn row_disagreements(lld: &Lld, row: &Row, declarations: &[Declared]) -> Vec<Failure> {
    written_fragments(row)
        .iter()
        .filter_map(|(written, fragment)| disagreement(fragment, declarations).map(|source| (written, source)))
        .map(|(written, source)| agreement_failure(Check::ShapeAgrees, &lld.path, row.line, &source.file, written, &source_wrote(source)))
        .collect()
}

/// The declaration one fragment disagrees with, and none where it has nothing
/// to disagree with: a fragment agreeing with any of the declarations it names
/// holds, and one naming no declaration at all is compared to none, which is
/// the same answer read off the same list. Where it names several and agrees
/// with none, the first is the one a failure quotes: a name the crate declares
/// twice is not thereby wrong, and choosing which of them the row meant would
/// be the resolution this slice does not do.
#[implements(
    spec::AFragmentNamingNoDeclaredFunctionIsComparedToNone,
    spec::AFragmentAgreeingWithAnyDeclarationItNamesHolds,
)]
fn disagreement<'a>(fragment: &Fragment, declarations: &'a [Declared]) -> Option<&'a Declared> {
    let named = compared(fragment, declarations);
    if named.iter().any(|declaration| agrees(fragment, declaration)) { None } else { named.first().copied() }
}

/// The declarations one fragment is compared to: those it [`matches`], and
/// none at all where it writes no parentheses, since a fragment without them
/// names a type rather than a function. Both ways of being compared to nothing
/// are this answer being empty — the fragment that names no declared function
/// as much as the one that names no function at all.
#[implements(
    spec::AFragmentWithoutParenthesesIsComparedToNothing,
    spec::AFragmentNamingNoDeclaredFunctionIsComparedToNone,
)]
fn compared<'a>(fragment: &Fragment, declarations: &'a [Declared]) -> Vec<&'a Declared> {
    match fragment.arguments {
        Some(_) => declarations.iter().filter(|declaration| matches(fragment, declaration)).collect(),
        None => vec![],
    }
}

/// One declaration spelled back as the source wrote it, which is the evidence
/// half of an agreement failure — what a human fixes the row against. The name,
/// qualified by the owner where it has one; the parameter type tokens the
/// reading answered, between parentheses and comma-separated, the receiver
/// already gone; and the return after an ASCII `->` where the source declared
/// one, nothing at all where it did not. It is evidence and not a grammar: a
/// reader is meant to recognise the item in it, so what a message about it must
/// hold is these three parts, not this spelling of them.
#[implements(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
fn source_wrote(declaration: &Declared) -> String {
    format!("{}({}){}", qualified(declaration), declaration.parameters.join(", "), returning(declaration.returns.as_deref()))
}

/// A declaration's name as the source qualifies it: `Owner::name` where the
/// function was read from an `impl` or a `trait` block, and the bare name for a
/// free function.
#[implements(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
fn qualified(declaration: &Declared) -> String {
    todo!("qualified({declaration:?})")
}

/// A return as a signature writes it: `-> Type` where the source declared one,
/// and nothing at all where it did not.
#[implements(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
fn returning(returns: Option<&str>) -> String {
    todo!("returning({returns:?})")
}

/// Whether a fragment names one declared function at all: their names are
/// equal and, where the fragment's first segment starts with an uppercase
/// letter — a type — that segment is the declaration's owner. A lowercase
/// first segment is a module, which the reading carries no function under, so
/// it narrows nothing and the fragment is matched by name alone.
///
/// The qualifier and the owner are compared [`stripped`], as every comparison
/// of tokens here is: the reading prints the self type of an `impl<'_> Fields<'_>`
/// as `Fields < '_ >`, and a row's qualifier `Fields` would otherwise match no
/// declaration at all, so every row about a method of a generic type would be
/// compared to nothing and the check would quietly stop checking.
#[implements(spec::AQualifierNarrowsAMatchOnlyWhenItIsAType)]
pub fn matches(fragment: &Fragment, declared: &Declared) -> bool {
    todo!("matches({fragment:?}, {declared:?})")
}

/// One fragment against one declaration it names: the fragment's argument
/// count is the declaration's parameter count — the fragment's own `self`
/// already dropped where the cell was read, the declaration's receiver already
/// dropped where the source was read, so a row writing `&mut self` and a row
/// leaving it out both agree with a method — and the fragment's return, where
/// it writes one, is the source's as [`same_type`] compares them.
#[implements(spec::AFragmentAgreesWhenItsCountAndAnyReturnItWritesAreTheSources)]
pub fn agrees(fragment: &Fragment, declared: &Declared) -> bool {
    same_arity(fragment, declared) && same_return(fragment.returns.as_deref(), declared)
}

/// Whether a fragment writes as many arguments as the declaration takes
/// parameters. Each side's receiver is already gone — the fragment's where the
/// cell was read, the declaration's where the source was read — so this is a
/// count of arguments against a count of arguments. A fragment that writes no
/// parentheses never reaches here, [`compared`] having left it out, so the
/// arguments it writes are the arguments it has.
#[implements(spec::AFragmentAgreesWhenItsCountAndAnyReturnItWritesAreTheSources)]
fn same_arity(fragment: &Fragment, declared: &Declared) -> bool {
    todo!("same_arity({fragment:?}, {declared:?})")
}

/// Whether the return a row writes is the one the source wrote. A row that
/// writes none is agreeing about arguments alone and holds whatever the source
/// returns, since the check is stated over the return a row writes. A row that
/// writes one against a declaration the source gave no `->` at all disagrees:
/// the row's return must be the source's, and the source's is nothing. Where
/// both write one, [`same_type`] compares them under the declaration's owner,
/// which is the only thing a `Self` on either side can stand for.
#[implements(spec::AFragmentAgreesWhenItsCountAndAnyReturnItWritesAreTheSources)]
fn same_return(row_return: Option<&str>, declared: &Declared) -> bool {
    match (row_return, declared.returns.as_deref()) {
        (None, _) => true,
        (Some(_), None) => false,
        (Some(row), Some(source)) => same_type(row, source, declared.owner.as_deref()),
    }
}

/// The row's return type against the source's, compared as a reader compares
/// them: whitespace stripped from both sides, lifetime arguments and lifetime
/// annotations erased from both, and `Self` substituted on both with `owner`
/// — the declaration's, never the fragment's own qualifier, and none for a
/// free function, whose `Self` then stands for no type and the two disagree.
/// The owner is read the same way the two sides are before it stands anywhere,
/// since it is tokens the same printer wrote.
#[implements(
    spec::WhitespaceIsStrippedFromBothSidesBeforeTwoTypesAreCompared,
    spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared,
    spec::SelfIsTheDeclarationsOwnerOnBothSides,
)]
pub fn same_type(row_return: &str, source_return: &str, owner: Option<&str>) -> bool {
    match (as_compared(row_return, owner), as_compared(source_return, owner)) {
        (Some(row), Some(source)) => row == source,
        (None, _) | (_, None) => false,
    }
}

/// One type as the comparison reads it: [`stripped`], and then its `Self`
/// standing for the declaration's owner — the owner `stripped` the same way,
/// since it is tokens the reading printed like any other and nothing else in
/// this slice normalises it. Without that, the owner of an `impl<'a> Returned<'a>`
/// arrives as `Returned < 'a >` and substituting it into a row's `Self` refuses
/// a document that wrote `Returned` correctly.
///
/// `Self` is substituted last because it is the only step that reads the owner,
/// and the owner has to be read before it can be put anywhere.
#[implements(spec::SelfIsTheDeclarationsOwnerOnBothSides)]
fn as_compared(type_tokens: &str, owner: Option<&str>) -> Option<String> {
    let written = stripped(type_tokens);
    match owner {
        Some(owner) => Some(self_substituted(&written, &stripped(owner))),
        None => ownerless(written),
    }
}

/// One type's tokens as every side of every comparison here is read: its
/// lifetimes erased, and then its whitespace stripped. A row's return, the
/// source's return, and the owner a `Self` stands for all pass through this and
/// through nothing else, which is what "read the same way" means.
///
/// The order is forced. Lifetimes go first because the space after one is what
/// ends it: strip the whitespace out of `& 'static str` first and `'staticstr`
/// is one word no erasure can take a lifetime out of, so the row's `&str` could
/// never be the same shape as the source's `&'static str`, which is the case
/// the rule exists for.
#[implements(
    spec::WhitespaceIsStrippedFromBothSidesBeforeTwoTypesAreCompared,
    spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared,
)]
fn stripped(type_tokens: &str) -> String {
    without_whitespace(&without_lifetimes(type_tokens))
}

/// A type about a free function, as the comparison reads it: itself where it
/// writes no `Self`, and nothing at all where it writes one — a `Self` with no
/// owner stands for no type, so a type that writes one is the same as nothing,
/// not even as another `Self`.
#[implements(spec::SelfIsTheDeclarationsOwnerOnBothSides)]
fn ownerless(written: String) -> Option<String> {
    (!names_self(&written)).then_some(written)
}

/// A type's tokens with every whitespace character removed, which is how one
/// side's printing is kept from being the difference: the reading renders the
/// source's `Result<Lld, String>` as `Result < Lld , String >`, and a row types
/// it as a reader types it.
#[implements(spec::WhitespaceIsStrippedFromBothSidesBeforeTwoTypesAreCompared)]
fn without_whitespace(type_tokens: &str) -> String {
    todo!("without_whitespace({type_tokens})")
}

/// A type's tokens with its lifetime arguments and lifetime annotations
/// erased: `Section<'a>` and `Section` are the same shape, `&'static str` and
/// `&str` are the same shape, and a document that spells the lifetime is not
/// more correct than one that does not.
///
/// What an erased lifetime takes with it is not a character but an argument.
/// Where the tokens hold a top-level generic argument list —
/// [`type_argument_list`] finds it — [`without_lifetimes_around_list`] treats
/// its three pieces differently, because only two of them are guaranteed to
/// hold no list of their own: the head, which is exactly the text before the
/// first top-level `<` and so can never hold one, is
/// [`without_lifetime_tokens`]'s to erase directly, and the list itself is
/// [`rewritten_argument_list`]'s to rewrite; but the tail can still hold a
/// sibling top-level list — a tuple return such as `(Vec<u8>, Section<'a>)`
/// has two, side by side — so the tail is read through this same function
/// again rather than through [`without_lifetime_tokens`], which would erase
/// its lifetime in place and leave exactly the `Section<>` the list-rewriting
/// rule forbids. Tokens with no top-level list at all — `&'static str`, a
/// bare `'a` an argument's whole text once split out of one — have only a
/// lifetime token to erase, which [`without_lifetime_tokens`] does directly.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn without_lifetimes(type_tokens: &str) -> String {
    match type_argument_list(type_tokens) {
        Some((head, list, tail)) => without_lifetimes_around_list(&head, &list, &tail),
        None => without_lifetime_tokens(type_tokens),
    }
}

/// The head, the contents, and the tail of a type's own outermost generic
/// argument list: the text before its first top-level `<`, the text between
/// that `<` and its matching `>`, and whatever follows. None where the tokens
/// hold no top-level `<` at all, and the same none where they hold one that is
/// never closed — a human writes one side of every comparison here, and
/// `Result<Lld, String` is reachable — since an unclosed list is not a list to
/// rewrite: inventing the missing `>` would read `Vec<T` and `Vec<T>` as the
/// same shape, which they are not. "Top-level" is tracked by depth, so the
/// inner list of `Result < Cow < 'a , str > , String >` closes on its own `>`
/// and does not end the outer one early: the head is `Result `, the contents
/// are ` Cow < 'a , str > , String `, and the tail is empty.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn type_argument_list(type_tokens: &str) -> Option<(String, String, String)> {
    todo!("type_argument_list({type_tokens})")
}

/// The text around a type's own outermost generic argument list: the head,
/// which can never hold a list of its own since it is exactly the text before
/// the first top-level `<`, is [`without_lifetime_tokens`]'s to erase
/// directly; the list itself is [`rewritten_argument_list`]'s to rewrite; and
/// the tail, which can still hold a sibling top-level list of its own — the
/// tuple return `(Vec<u8>, Section<'a>)` splits into a head of `(Vec`, a first
/// list of `u8`, and a tail of `, Section<'a>)` that has a second list in it
/// — is read through [`without_lifetimes`] again rather than through
/// [`without_lifetime_tokens`], which would erase that second list's lifetime
/// in place and leave the `Section<>` the list-rewriting rule forbids.
/// Recursing on the tail terminates because the tail excludes the `<...>`
/// this call just consumed and so is always strictly shorter than the tokens
/// read to find it.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn without_lifetimes_around_list(head: &str, list: &str, tail: &str) -> String {
    format!("{}{}{}", without_lifetime_tokens(head), rewritten_argument_list(list), without_lifetimes(tail))
}

/// A run of tokens guaranteed to hold no top-level generic argument list of
/// its own — the head [`type_argument_list`] split off, which can never hold
/// one because it is exactly the text before the first top-level `<`, or the
/// whole of a type that reading found no list in, whether it writes no
/// top-level `<` at all or writes one that is never closed — with every lifetime token
/// in it removed and nothing else touched: an argument `'a`, an annotation
/// `'static` in `&'static str`, an elided `'_` — each token taken out
/// outright, the punctuation around it left exactly where it was. A tail is
/// never handed here, because a tail can hold a sibling list of its own;
/// [`without_lifetimes_around_list`] hands that back to
/// [`without_lifetimes`] instead. This is the only place a lifetime token is
/// ever erased; what becomes of an argument that was nothing but one is
/// [`argument_survives`]'s question, never this function's.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn without_lifetime_tokens(type_tokens: &str) -> String {
    todo!("without_lifetime_tokens({type_tokens})")
}

/// One generic argument list's contents, written afresh from whichever of its
/// own arguments still have something in them: split by
/// [`top_level_arguments`], the survivors [`surviving_arguments`] answers are
/// joined back into the list — or the list is left out entirely, brackets
/// included — by [`argument_list_written`], the one place that decision is
/// made.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn rewritten_argument_list(list_contents: &str) -> String {
    argument_list_written(&surviving_arguments(list_contents))
}

/// A list's contents split into its own top-level arguments — by a comma at
/// this list's own depth, so a nested list's comma never splits it: the two
/// arguments of ` Cow < 'a , str > , String ` are ` Cow < 'a , str > ` and
/// ` String `, not three.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn top_level_arguments(list_contents: &str) -> Vec<String> {
    todo!("top_level_arguments({list_contents})")
}

/// [`top_level_arguments`]'s split, each argument read through
/// [`without_lifetimes`] again — since an argument may hold a generic argument
/// list of its own, as `Cow<'a, str>` does inside
/// `Result<Cow<'a, str>, String>` — less whichever of them
/// [`argument_survives`] says an erased lifetime left holding nothing: a bare
/// `'a` or `'b` erases to nothing and does not survive; `K`, and `Cow<'a,
/// str>` once rewritten to `Cow<str>`, do.
#[implements(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
fn surviving_arguments(list_contents: &str) -> Vec<String> {
    top_level_arguments(list_contents).iter().map(|argument| without_lifetimes(argument)).filter(|rewritten| argument_survives(rewritten)).collect()
}

/// Whether an argument had anything left in it once its own lifetimes were
/// erased: a bare lifetime erases to nothing but whitespace and does not
/// survive to be joined back into the list it came from. This is what decides
/// whether a list is left holding no argument at all, so a wrong answer here
/// is the direct route to the `Section<>` that decision forbids.
#[implements(
    spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared,
    spec::AnArgumentListLeftWithNoArgumentIsNotWrittenAtAll,
)]
fn argument_survives(rewritten: &str) -> bool {
    !rewritten.trim().is_empty()
}

/// The one decision an erased lifetime forces on the list it stood in: a list
/// with nothing left in it is written with no brackets at all, and a list
/// with something left is written afresh from what survived, joined by
/// commas, inside its brackets — a list of one, a list of three, and a list of
/// none are all written the same way, never edited in place. `Section<'a>`
/// keeps no argument and is written `Section`, never `Section<>`; `Map<'a,
/// 'b, K>` keeps one of three and is written `Map<K>`, with nothing further to
/// say about the second lifetime erased.
#[implements(
    spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared,
    spec::AnArgumentListLeftWithNoArgumentIsNotWrittenAtAll,
)]
fn argument_list_written(surviving: &[String]) -> String {
    if surviving.is_empty() { String::new() } else { format!("<{}>", surviving.join(",")) }
}

/// A type's tokens with `Self` standing for the owner it was declared under —
/// the self type of the `impl`, or the name of the `trait` that gave the method
/// a body — which is the substitution both sides get. Both arguments arrive
/// [`stripped`], the owner as much as the type, so this puts one read form
/// inside another and never a printer's spacing inside a stripped type.
#[implements(spec::SelfIsTheDeclarationsOwnerOnBothSides)]
fn self_substituted(type_tokens: &str, owner: &str) -> String {
    todo!("self_substituted({type_tokens}, {owner})")
}

/// Whether a type's tokens write `Self` at all, which is the only thing an
/// ownerless declaration needs to know about them.
#[implements(spec::SelfIsTheDeclarationsOwnerOnBothSides)]
fn names_self(type_tokens: &str) -> bool {
    todo!("names_self({type_tokens})")
}

/// The rows whose first cell names another slice's item as a bare identifier
/// rather than an intra-doc link, each failing on its own line; the link's
/// resolution is rustdoc's, at the doc step Phase 1 already runs. Which rows
/// those are is [`is_reuse`]'s decision, over the slice's own name, its
/// [`siblings`] and the workspace's [`crate_names`]. A document with no
/// `## Shape` table has no row to ask, as `shape_table_rows` decides.
#[implements(spec::AReuseRowNamesItsItemAsAnIntraDocLink)]
pub fn reuse_rows_linked(project: &Project, lld: &Lld) -> Vec<Failure> {
    let inside = siblings(project, &lld.slice);
    let crates = crate_names(project);
    shape_table_rows(lld).iter().filter_map(|row| unlinked_reuse(lld, row, &inside, &crates)).collect()
}

/// The failure one row earns when its first cell names another slice's item as
/// a bare identifier rather than an intra-doc link — one failure for the row
/// however many of its fragments are unlinked, since a second would be the
/// same check quoting the same rule on the same line.
#[implements(spec::AReuseRowNamesItsItemAsAnIntraDocLink)]
fn unlinked_reuse(lld: &Lld, row: &Row, inside: &[String], crates: &[String]) -> Option<Failure> {
    fragments(first_cell(row))
        .iter()
        .any(|fragment| is_reuse(fragment, &lld.slice, inside, crates) && !fragment.linked)
        .then(|| failure(Check::ReuseRowsLinked, &lld.path, row.line))
}

/// The module names one level inside the slice's directory — the directory
/// [`layout::slice_dir`](crate::layout::slice_dir) answers: a file's stem
/// without its extension, `mod.rs` left out since it names the slice itself,
/// and a directory's own name. A listing, which resolves nothing: a name is a
/// name there whether or not a `mod` declaration mentions it. Empty for a slice
/// with no directory to read, whether no crate holds it, its code is the one
/// file `src/<module>.rs`, or the directory cannot be read.
#[implements(
    spec::ASiblingIsAFileStemOrADirectoryNameOneLevelInsideTheSlice,
    spec::ASliceWithNoDirectoryToReadListsNoSibling,
)]
pub fn siblings(project: &Project, slice: &str) -> Vec<String> {
    let Ok(dir) = layout::slice_dir(project, slice) else {
        return vec![];
    };
    entries(&dir).iter().filter_map(|entry| module_name(entry)).collect()
}

/// The paths one level inside a directory — one level and no deeper, a
/// subdirectory being a module name rather than a tree to walk — and none at
/// all for a directory that cannot be read, which is the same answer as an
/// empty one, since a listing nobody can take is a listing with nothing in it
/// and never a failure.
#[implements(
    spec::ASiblingIsAFileStemOrADirectoryNameOneLevelInsideTheSlice,
    spec::ASliceWithNoDirectoryToReadListsNoSibling,
)]
fn entries(dir: &Path) -> Vec<PathBuf> {
    todo!("entries({})", dir.display())
}

/// The module name one entry carries: a directory's own name, a file's stem
/// without its extension, and none for `mod.rs`, which names the slice itself
/// rather than anything under it.
#[implements(spec::ASiblingIsAFileStemOrADirectoryNameOneLevelInsideTheSlice)]
fn module_name(entry: &Path) -> Option<String> {
    todo!("module_name({})", entry.display())
}

/// The workspace members' names as a path spells them:
/// [`Project::member_manifest_dirs`] for the members, [`Project::package_at`]
/// for each one's package name, hyphens read as underscores — `lid-rs-shape`
/// is the package's name and `lid_rs_shape` is the path's. The first answers
/// directories and the second is asked about a manifest, so the `Cargo.toml`
/// join is this function's and no caller's.
#[implements(spec::ACrateNameIsAMembersPackageNameWithHyphensAsUnderscores)]
pub fn crate_names(project: &Project) -> Vec<String> {
    todo!("crate_names({project:?})")
}

/// One decision over one fragment: whether it names another slice's item in
/// the same crate. It does when its path has two or more segments, its first
/// segment starts with a lowercase letter, and that segment is none of the
/// slice's own name in module form, the siblings listed for it, or the crate
/// names — a bare name names nothing elsewhere, an uppercase first segment is a
/// type the role cell introduces, a name inside the slice's directory is the
/// slice's own, and a crate name is a path into a crate rather than a sibling
/// module.
#[implements(
    spec::AModuleQualifiedPathOutsideTheSlicesDirectoryIsAReuseRow,
    spec::APathThatIsNotModuleQualifiedIsNotAReuseRow,
    spec::APathIntoTheSlicesOwnModulesIsNotAReuseRow,
    spec::APathQualifiedByAWorkspaceMembersCrateNameIsNotAReuseRow,
)]
pub fn is_reuse(fragment: &Fragment, slice: &str, siblings: &[String], crates: &[String]) -> bool {
    todo!("is_reuse({fragment:?}, {slice}, {siblings:?}, {crates:?})")
}

/// One decision over a return fragment: whether a signature written with it
/// and a `todo!()` body compiles. False for an `impl Trait` return, whose
/// hidden type a `todo!()` body infers as `!`, which implements nothing, and
/// for a bare `dyn Trait` one, which is unsized in return position; true for
/// every type that merely contains them, since the test is over the return's
/// outermost form and nothing deeper.
#[implements(spec::AnImplTraitOrBareDynTraitReturnIsNotSkeletonable, spec::AReturnThatMerelyContainsAnImplOrDynTypeIsSkeletonable)]
pub fn skeletonable(returns: &str) -> bool {
    todo!("skeletonable({returns})")
}

/// The rows whose return fragment cannot be skeletonised, as [`skeletonable`]
/// decides, each failing on its own line — refused at Phase 1, where the
/// document is committed and the human who wrote it is present, rather than at
/// Phase 3, where a worker meets a row it cannot write. A document with no
/// `## Shape` table has no row to refuse, as `shape_table_rows` decides.
#[implements(spec::ARowWhoseReturnCannotBeSkeletonisedFailsOnItsLine)]
pub fn shape_returns(lld: &Lld) -> Vec<Failure> {
    shape_table_rows(lld).iter().filter_map(|row| unskeletonable_return(lld, row)).collect()
}

/// The failure one row earns when a return its first cell writes is one
/// [`skeletonable`] refuses — one failure for the row however many of its
/// fragments write such a return, since a second would be the same check
/// quoting the same rule on the same line. A fragment writing no return has
/// none to refuse.
#[implements(spec::ARowWhoseReturnCannotBeSkeletonisedFailsOnItsLine)]
fn unskeletonable_return(lld: &Lld, row: &Row) -> Option<Failure> {
    fragments(first_cell(row))
        .iter()
        .filter_map(|fragment| fragment.returns.as_deref())
        .any(|returns| !skeletonable(returns))
        .then(|| failure(Check::SkeletonableReturns, &lld.path, row.line))
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

    /// A guideline whose checklist names four of the nine checks, and whose
    /// questions name two of the five it omits, outside the checklist — where
    /// naming a check is not naming it in the checklist.
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

    /// A `cargo metadata` document for a workspace at `root` whose members are
    /// the directories `members`, each relative to it and each carrying the
    /// package name it is paired with — the name being what [`crate_names`]
    /// reads, and nothing else here.
    fn metadata(root: &Path, members: &[(&str, &str)]) -> String {
        let packages: Vec<String> = members
            .iter()
            .map(|(member, name)| {
                let manifest = root.join(member).join("Cargo.toml");
                format!(r#"{{"name":"{name}","manifest_path":"{}","targets":[{{"kind":["lib"],"name":"{name}"}}]}}"#, manifest.display())
            })
            .collect();
        format!(
            r#"{{"workspace_root":"{}","target_directory":"{}","packages":[{}]}}"#,
            root.display(),
            root.join("target").display(),
            packages.join(",")
        )
    }

    /// A project rooted at `root` with those members, without asking cargo,
    /// every package named `m`: which reading a member's directory is enough
    /// for, and which is every check here but [`crate_names`].
    fn project_at(root: &Path, members: &[&str]) -> Project {
        project_naming(root, &members.iter().map(|member| (*member, "m")).collect::<Vec<(&str, &str)>>())
    }

    /// A project rooted at `root` whose members carry the package names they
    /// are paired with, for the one check that reads a package's name.
    fn project_naming(root: &Path, members: &[(&str, &str)]) -> Project {
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

    /// A slice's source holding the three declarations whose owners differ: a
    /// free function, which has none; a method of an `impl Shape<T>`, whose
    /// owner is the block's self type as the source wrote it; and a function a
    /// `trait` gives a body to, whose owner is the trait's name.
    const SLICE_SOURCE: &str = "\
pub mod inner;

pub fn run(args: &[String], project: &Project) -> Result<(), String> {
    todo!()
}

pub struct Shape<T> {
    pub held: T,
}

impl<T> Shape<T> {
    pub fn of(&self, held: T) -> Shape<T> {
        todo!()
    }
}

pub trait Read {
    fn read(&self, path: &Path) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}
";

    /// A scratch workspace whose member `app` holds the slice `thing`: that
    /// source as its `mod.rs`, a submodule file under it, the document beside
    /// them, and a module of the same crate that is no part of the slice.
    fn slice_with_source(name: &str, source: &str, document: &str) -> (PathBuf, Project) {
        let root = fixture::scratch(name);
        write_at(&root.join("app/src/lib.rs"), "pub mod elsewhere;\npub mod thing;\n");
        write_at(&root.join("app/src/elsewhere.rs"), "pub fn far_away(only: u8) -> u8 { only }\n");
        write_at(&root.join("app/src/thing/mod.rs"), source);
        write_at(&root.join("app/src/thing/inner.rs"), "pub fn deeper(one: u8) -> u8 { one }\n");
        write_at(&root.join("app/src/thing/lld.md"), document);
        (root.clone(), project_at(&root, &["app"]))
    }

    /// One fragment as a first cell writes it, for the comparisons that are
    /// about a path and its arguments rather than about how a cell is read.
    fn fragment(path: &[&str], arguments: Option<&[&str]>, returns: Option<&str>) -> Fragment {
        Fragment { path: strings(path), arguments: arguments.map(strings), returns: returns.map(str::to_string), linked: false }
    }

    /// One fragment spelled back as a line a test reads: the path, the
    /// arguments where it writes parentheses, and the return where it writes
    /// one — the three answers [`fragments`] gives, in one string.
    fn as_written(fragment: &Fragment) -> String {
        let arguments =
            fragment.arguments.as_ref().map_or_else(|| ", no parentheses".to_string(), |written| format!("({})", written.join(", ")));
        let returns = fragment.returns.as_deref().map_or_else(|| ", no return".to_string(), |written| format!(" -> {}", written.trim()));
        format!("{}{arguments}{returns}", fragment.path.join("::"))
    }

    /// One declaration as this slice carries one, its receiver already dropped
    /// from its parameters, as [`declared`] answers it.
    fn declaration(name: &str, owner: Option<&str>, parameters: &[&str], returns: Option<&str>) -> Declared {
        Declared {
            name: name.to_string(),
            owner: owner.map(str::to_string),
            parameters: strings(parameters),
            returns: returns.map(str::to_string),
            file: PathBuf::from("/w/app/src/thing/mod.rs"),
        }
    }

    /// One signature as the shape pass answers one, the receiver still among
    /// the parameters, as the type tokens it stands for.
    fn signature(function: &str, owner: Option<&str>, parameters: &[&str], returns: Option<&str>) -> lid_rs_shape::Signature {
        lid_rs_shape::Signature {
            file: PathBuf::from("/w/app/src/thing/mod.rs"),
            function: function.to_string(),
            parameters: strings(parameters),
            returns: returns.map(str::to_string),
            owner: owner.map(str::to_string),
        }
    }

    /// A document whose shape rows meet that source: a row agreeing with the
    /// entry, one an argument short of it, one qualified by a type the method
    /// is not declared under, one naming no declared function, and one naming
    /// a type rather than a function.
    const AGREEMENT_ROWS: &str = "\
# thing — a slice

## Shape

| Item | Role |
|---|---|
| `run(args, project) -> Result<(), String>` | the entry, as the source declares it |
| `run(args)` | the same entry, one argument short |
| `Shape::of(&self, held) -> Shape<T>` | the method, its receiver written |
| `Lld::of(project)` | `of` qualified by a type that is not its owner |
| `nowhere(a, b)` | a fragment naming no declared function |
| `Shape` | a type, with no parentheses to compare |
";

    /// A document failing eight of the nine checks at once, the ninth being
    /// the one a document with a decisions table cannot fail.
    const ORDERED_FAILURES: &str = "\
# thing — a slice

## Shape

| Item | Role |
|---|---|
| a row that names no identifier | a note rather than a shape |
| `run(args)` | the entry, one argument short of the source |
| `hidden() -> impl Iterator<Item = u8>` | a return no layer-0 skeleton can be written for |
| `layout::lld_path(project, slice)` | another slice's item, named bare |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| three | cells | only |

### Deferred
- an unnumbered deferral
";

    /// A document whose rows name items of every kind the link rule tells
    /// apart, one of them bare where the rule asks for a link.
    const REUSE_ROWS: &str = "\
# thing — a slice

## Shape

| Item | Role |
|---|---|
| `layout::lld_path(project, slice)` | another slice's item in the same crate, named bare |
| [`layout::slice_dir`](crate::layout::slice_dir) | the same kind of item, written as the link the rule asks for |
| `thing::run(args)` | the slice's own name, spelled as a path spells it |
| `inner::deeper(one)` | a module inside the slice's own directory |
| `m::far_away(only)` | a workspace member's crate name |
| `Lld::read(project, slice)` | a type, which the role cell introduces |
";

    /// A document whose rows write returns of both unskeletonable shapes,
    /// beside returns that merely contain one.
    const RETURN_ROWS: &str = "\
# thing — a slice

## Shape

| Item | Role |
|---|---|
| `hidden() -> impl Iterator<Item = u8>` | a hidden type a `todo!()` body infers as `!` |
| `erased() -> dyn Error` | unsized in return position |
| `boxed() -> Box<dyn Error>` | a type that merely contains one |
| `fallible() -> Result<Box<dyn Error>, String>` | deeper still |
| `Section` | a type, writing no return at all |
";

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
        // artifacts: every check holds, and the run exits zero. The members
        // are named as this workspace names them, because the reuse check
        // excludes a first cell qualified by a crate's name, and a fabricated
        // member list would exclude a name no row here could write.
        let project = project_naming(
            &workspace_root(),
            &[
                ("lid-rs", "lid-rs"),
                ("lid-rs-macros", "lid-rs-macros"),
                ("cargo-lid-rs", "cargo-lid-rs"),
                ("lid-rs-shape", "lid-rs-shape"),
                ("lid-rs-pipeline", "lid-rs-pipeline"),
                ("xtask", "xtask"),
            ],
        );
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
            (Check::ShapeAgrees, "argument count"),
            (Check::SkeletonableReturns, "bare `dyn Trait`"),
            (Check::ReuseRowsLinked, "intra-doc link"),
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

        // Four of nine named in the checklist; two of the five it omits are
        // named only in the questions, which are not the checklist.
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
        assert!(
            failures[0].message.starts_with("`ShapeRows`, `ShapeAgrees`, `SkeletonableReturns`, `ReuseRowsLinked`, `ReaderObservesOnly`"),
            "it names what the checklist omits, in the order the checks are declared: {}",
            failures[0].message
        );
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

    #[test]
    #[validates(spec::TheChecksRunInTheOrderTheTableStatesThem)]
    fn the_checks_run_in_the_order_the_table_states_them() {
        // Eight of the nine at once, in a project holding neither synced
        // artifact. The ninth cannot fail beside `Alternatives`: a decisions
        // table that is not there has no row to leave a cell empty.
        let (_, project) = slice_with_source("lld-review-order", SLICE_SOURCE, ORDERED_FAILURES);
        let doc = Lld::read(&project, "thing").expect("the member holds the slice's document");
        let checks: Vec<Check> = check_all(&project, &doc).expect("the root is locatable").iter().map(|failure| failure.check).collect();
        let without_a_table: Vec<Check> =
            check_all(&project, &document("thing", FAILS_THREE)).expect("the root is locatable").iter().map(|failure| failure.check).collect();

        assert_eq!(
            checks,
            [
                Check::Alternatives,
                Check::ShapeRows,
                Check::ShapeAgrees,
                Check::SkeletonableReturns,
                Check::ReuseRowsLinked,
                Check::DeferredNumbered,
                Check::GuidelineNamesEveryCheck,
                Check::ReaderObservesOnly,
            ],
            "the order the table of checks states them, which is the order `Check` declares its variants"
        );
        assert_eq!(
            without_a_table,
            [Check::DecisionsExist, Check::ShapeRows, Check::DeferredNumbered, Check::GuidelineNamesEveryCheck, Check::ReaderObservesOnly],
            "and where the decisions table is absent altogether, that check is the first of all — ahead of the shape rows, not behind them"
        );
    }

    #[test]
    #[validates(spec::TheDeclaredFunctionsAreTheSlicesOwnCratesUnderItsDirectory, spec::ASliceNoCrateHoldsDeclaresNoFunction)]
    fn the_declared_functions_are_the_slices_own_crates_under_its_directory() {
        let (root, project) = slice_with_source("lld-review-declared", SLICE_SOURCE, HOLDS);

        // No workspace member holds a document for `skill`, so it has no
        // crate: it declares nothing, and that is an answer and not an error.
        assert!(declared(&project, "skill").is_empty(), "a slice no crate holds declares no function");

        let read = declared(&project, "thing");
        let mut named: Vec<(&str, Option<&str>, usize)> =
            read.iter().map(|one| (one.name.as_str(), one.owner.as_deref(), one.parameters.len())).collect();
        named.sort_unstable();

        assert_eq!(
            named,
            [("deeper", None, 1), ("of", Some("Shape < T >"), 1), ("read", Some("Read"), 1), ("run", None, 2)],
            "the submodule's function, the method of an `impl`, the trait's own and the free one — and never `far_away`, which is the same crate but not the slice"
        );
        assert!(read.iter().all(|one| one.file.starts_with(root.join("app/src/thing"))), "each was read from a file of the slice's own");
    }

    #[test]
    #[validates(spec::ADeclaredMethodsReceiverIsNotAmongItsParameters)]
    fn a_declared_methods_receiver_is_not_among_its_parameters() {
        // The four shapes a receiver is answered as, each also written with a
        // lifetime, which is what makes a `&'a self` one of the four and not a
        // fifth. That a first parameter genuinely typed `&'a Self` is read the
        // same way is the imprecision the document states, so nothing here
        // asserts against it.
        let receivers = ["Self", "& Self", "& mut Self", "Box < Self >", "& 'a Self", "& 'a mut Self"]
            .map(|first| less_receiver(&strings(&[first, "u8"])).len());
        let method = declared_from(signature("held", Some("Returned < 'a >"), &["& 'a Self", "& str"], Some("& 'a str")));

        assert_eq!(receivers, [1; 6], "each of the four leaves a count that is a count of arguments");
        assert_eq!(
            less_receiver(&strings(&["& 'a str", "u8"])),
            strings(&["& 'a str", "u8"]),
            "a lifetime-annotated reference to something that is not `Self` is an argument, and stays"
        );
        assert_eq!(
            (method.name.as_str(), method.owner.as_deref(), method.parameters.as_slice(), method.returns.as_deref()),
            ("held", Some("Returned < 'a >"), strings(&["& str"]).as_slice(), Some("& 'a str")),
            "the reading's own name, owner and return are carried across; the receiver alone is dropped"
        );
    }

    #[test]
    #[validates(spec::EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment)]
    fn every_backticked_identifier_of_a_first_cell_is_its_own_fragment() {
        let cell = "`Lld`, `Lld::read(project, slice) -> Result<Lld, String>`, `run()`";
        let read: Vec<String> = fragments(cell).iter().map(as_written).collect();

        assert_eq!(
            read,
            [
                "Lld, no parentheses, no return",
                "Lld::read(project, slice) -> Result<Lld, String>",
                "run(), no return",
            ],
            "one fragment per identifier, as many as the cell holds; empty parentheses are a function of no arguments, and none at all name a type"
        );
        assert!(fragments("plain prose, naming nothing").is_empty(), "a cell with no backtick span writes no fragment");
    }

    #[test]
    #[validates(spec::AReturnIsWrittenWithTheAsciiArrowAlone)]
    fn a_return_is_written_with_the_ascii_arrow_alone() {
        let ascii = fragments("`skeletonable(returns) -> bool`");
        let unicode = fragments("`skeletonable(returns) → bool`");

        assert_eq!(
            (ascii[0].returns.as_deref().map(str::trim), ascii[0].arguments.clone()),
            (Some("bool"), Some(strings(&["returns"]))),
            "the ASCII arrow writes the return"
        );
        assert_eq!(
            (unicode[0].returns.as_deref(), unicode[0].arguments.clone()),
            (None, Some(strings(&["returns"]))),
            "a row that reaches for the Unicode arrow writes no return, and is compared on its arguments alone"
        );
    }

    #[test]
    #[validates(spec::AFirstArgumentWrittenAsAReceiverIsNotAFragmentsArgument)]
    fn a_first_argument_written_as_a_receiver_is_not_a_fragments_argument() {
        // Both conventions the tables use: a row that writes the receiver, and
        // a row that leaves the one its source declares out.
        let written = ["Budget::spend(&mut self) -> bool", "Shape::of(&self, held)", "of(self)", "Door::request(method, path, bearer, body, idem)"]
            .map(arguments);

        assert_eq!(
            written,
            [
                Some(strings(&[])),
                Some(strings(&["held"])),
                Some(strings(&[])),
                Some(strings(&["method", "path", "bearer", "body", "idem"])),
            ],
            "what is left is a list of arguments however the row's author spelled the method"
        );
        assert_eq!(
            (arguments("Lld"), arguments("run()")),
            (None, Some(strings(&[]))),
            "no parentheses names a type; empty ones are a function of no arguments"
        );
    }

    #[test]
    #[validates(spec::AFragmentIsLinkedOnlyWhenItsBacktickSpanIsWrappedInAMarkdownLink)]
    fn a_fragment_is_linked_only_when_its_backtick_span_is_wrapped_in_a_markdown_link() {
        // The same identifier twice, once linked and once bare; a bracket that
        // opens no link; and a span the cell never closes, which runs to the
        // end and so has no byte after it to ask about.
        let cell = "[`layout::lld_path`](crate::layout::lld_path), `layout::lld_path`, [`bracketed`] and `trailing";
        let by_position: Vec<bool> = (0..5).map(|nth| linked(cell, nth)).collect();

        assert_eq!(by_position, [true, false, false, false, false], "linked only where `[` and `](` are the span's neighbours, and never where there is no span");
        assert_eq!(fragments(cell).iter().map(|fragment| fragment.linked).collect::<Vec<bool>>(), [true, false, false, false]);
    }

    #[test]
    #[validates(spec::AQualifierNarrowsAMatchOnlyWhenItIsAType)]
    fn a_qualifier_narrows_a_match_only_when_it_is_a_type() {
        let free = declaration("read", None, &["& Project"], None);
        let method = declaration("read", Some("Lld"), &["& Project"], None);
        let bare = fragment(&["read"], Some(&["project"]), None);
        let typed = fragment(&["Lld", "read"], Some(&["project"]), None);

        assert_eq!(
            (matches(&bare, &free), matches(&bare, &method), matches(&typed, &method), matches(&typed, &free)),
            (true, true, true, false),
            "names alone where the fragment writes no type; the owner too where it writes one"
        );
        assert!(!matches(&typed, &declaration("read", Some("Other"), &["& Project"], None)), "a type that is not the owner narrows to nothing");
        assert!(
            matches(&fragment(&["Fields", "of"], Some(&["held"]), None), &declaration("of", Some("Fields < '_ >"), &["u8"], None)),
            "a row's `Fields` is the owner an `impl<'_> Fields<'_>` is printed as, both sides read the same way"
        );
        assert!(
            matches(&fragment(&["layout", "lld_path"], Some(&["p"]), None), &declaration("lld_path", Some("Layout"), &["p"], None)),
            "a lowercase qualifier is a module, which narrows nothing"
        );
        assert!(!matches(&fragment(&["write"], Some(&["p"]), None), &free), "different names never name each other");
        // A path of one segment writes no qualifier at all — its one segment is
        // the name — so there is nothing to narrow with whatever case that
        // segment is in. The reading answers whatever the source spelled, and a
        // free function declared under an uppercase name is one a row may write
        // bare.
        let uppercase = [declaration("Build", None, &["u8"], None)];
        let bare_uppercase = fragment(&["Build"], Some(&["held"]), None);

        assert!(
            matches(&bare_uppercase, &uppercase[0]),
            "a one-segment path names a function and no owner, so an uppercase name is not thereby a qualifier the declaration must be owned by"
        );
        assert_eq!(
            compared(&bare_uppercase, &uppercase).len(),
            1,
            "so the row is compared to the declaration it names, rather than narrowed against an owner it never wrote and compared to none"
        );
    }

    #[test]
    #[validates(spec::AFragmentWithoutParenthesesIsComparedToNothing)]
    fn a_fragment_without_parentheses_is_compared_to_nothing() {
        let declarations = [declaration("run", None, &["& [String]"], None)];
        let names_a_type = fragment(&["run"], None, None);

        assert!(compared(&names_a_type, &declarations).is_empty(), "a fragment writing no parentheses names a type, whatever the crate declares under that name");
        assert_eq!(disagreement(&names_a_type, &declarations), None, "so it has nothing to disagree with");
        assert_eq!(
            compared(&fragment(&["run"], Some(&["args"]), None), &declarations).len(),
            1,
            "a fragment that writes them is compared to the function it names"
        );
    }

    #[test]
    #[validates(spec::AFragmentNamingNoDeclaredFunctionIsComparedToNone)]
    fn a_fragment_naming_no_declared_function_is_compared_to_none() {
        let declarations = [declaration("run", None, &["& [String]", "& Project"], None)];

        assert_eq!(
            disagreement(&fragment(&["absent"], Some(&["a"]), None), &declarations),
            None,
            "at Phase 1 no function is declared yet, and a reuse row names an item another slice owns"
        );
        assert_eq!(
            disagreement(&fragment(&["run"], Some(&["args"]), None), &declarations),
            Some(&declarations[0]),
            "a fragment that names one and agrees with none of them has that one to disagree with"
        );
    }

    #[test]
    #[validates(spec::AFragmentAgreeingWithAnyDeclarationItNamesHolds)]
    fn a_fragment_agreeing_with_any_declaration_it_names_holds() {
        let (_, project) = slice_with_source("lld-review-agreement", SLICE_SOURCE, AGREEMENT_ROWS);
        let doc = Lld::read(&project, "thing").expect("the member holds the slice's document");
        let twice = [declaration("run", None, &["u8"], None), declaration("run", None, &["u8", "u8"], None)];

        assert_eq!(
            all_located(&shape_agrees(&project, &doc)),
            [(Check::ShapeAgrees, doc.path.as_path(), line_at(AGREEMENT_ROWS, "| `run(args)` | the same entry, one argument short |"))],
            "the agreeing row, the row whose qualifier names no declaration, the row naming nothing declared and the row naming a type all hold"
        );
        assert_eq!(
            disagreement(&fragment(&["run"], Some(&["a", "b"]), None), &twice),
            None,
            "a name the crate declares twice is not thereby wrong: agreeing with either of them holds"
        );
        assert_eq!(disagreement(&fragment(&["run"], Some(&["a", "b", "c"]), None), &twice), Some(&twice[0]), "and the first is the one a failure quotes");
    }

    #[test]
    #[validates(spec::AFragmentAgreesWhenItsCountAndAnyReturnItWritesAreTheSources)]
    fn a_fragment_agrees_when_its_count_and_any_return_it_writes_are_the_sources() {
        let source = declaration("read", Some("Lld"), &["& Project", "& str"], Some("Result < Self , String >"));
        let infallible = declaration("run", None, &[], None);

        assert!(agrees(&fragment(&["Lld", "read"], Some(&["project", "slice"]), None), &source), "a row writing no return agrees about its arguments alone");
        assert!(agrees(&fragment(&["read"], Some(&["project", "slice"]), Some("Result<Lld, String>")), &source), "and one that writes the source's return agrees");
        assert!(!agrees(&fragment(&["read"], Some(&["project"]), None), &source), "one argument short of the source disagrees");
        assert!(!agrees(&fragment(&["read"], Some(&["project", "slice"]), Some("Result<PathBuf, String>")), &source), "a return that is not the source's disagrees");
        assert!(!agrees(&fragment(&["run"], Some(&[]), Some("bool")), &infallible), "a row writing a return where the source declared none disagrees");
        assert!(agrees(&fragment(&["run"], Some(&[]), None), &infallible), "and a row that writes neither agrees");
    }

    #[test]
    #[validates(spec::WhitespaceIsStrippedFromBothSidesBeforeTwoTypesAreCompared)]
    fn whitespace_is_stripped_from_both_sides_before_two_types_are_compared() {
        // The reading renders the source's `Result<Lld, String>` with the
        // spacing its printer chose; the row types it as a reader types it.
        assert!(same_type("Result<Lld, String>", "Result < Lld , String >", None), "how one side was printed is never the difference");
        assert!(same_type("Result < Lld , String >", "Result<Lld,String>", None), "and neither is how the other was typed");
        assert!(!same_type("Result<Lld, PathBuf>", "Result < Lld , String >", None), "what is left once the spacing is gone is compared");
        assert_eq!(stripped("  Result < Lld , String >  "), "Result<Lld,String>");
    }

    #[test]
    #[validates(spec::LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared)]
    fn lifetime_arguments_and_annotations_are_erased_before_two_types_are_compared() {
        // What an erased lifetime takes with it is an argument and not a
        // character: a list that loses one of two, or one of three, still
        // writes the arguments it has, brackets and all.
        let read = [
            "Cow < 'a , str >",
            "Map < 'a , 'b , K >",
            "& 'static str",
            "Result < Cow < 'a , str > , String >",
            "( Vec < u8 > , Section < 'a > )",
            "Vec < u8 >",
            "Result < Lld , String",
        ]
        .map(stripped);

        assert_eq!(
            read,
            ["Cow<str>", "Map<K>", "&str", "Result<Cow<str>,String>", "(Vec<u8>,Section)", "Vec<u8>", "Result<Lld,String"],
            "an argument the erasure does not touch survives, at any depth and in a sibling list; and one side of every comparison is written by a human, so an unclosed `<` is no list to rewrite"
        );
        // An outer argument that erases away entirely, beside an inner list
        // that survives: the outer list is written afresh from the survivor
        // alone, so the comma that separated the two goes with the argument it
        // separated. This is what the split into top-level arguments is for,
        // and the only reading that shows it — where every argument of the
        // outer list survives, a split that never comes back to the list's own
        // depth reads the whole contents as one argument, finds the inner list
        // inside it anyway, rewrites that, and joins back to the same
        // characters. Here it cannot: the comma it leaves standing is
        // `Map<,Cow<str>>` when the erased argument came first and
        // `Map<Cow<str>,>` when it came last, and both orders are asked because
        // a split that fails only after the first inner list closes agrees with
        // the first of them.
        let stranded = ["Map < 'a , Cow < 'b , str > >", "Map < Cow < 'b , str > , 'a >"].map(stripped);

        assert_eq!(
            stranded,
            ["Map<Cow<str>>", "Map<Cow<str>>"],
            "the surviving argument is written on its own, wherever in the list the erased lifetime stood, and never beside the comma that stood between them"
        );
        assert_eq!(
            (same_type("Cow<str>", "Cow < 'a , str >", None), same_type("&str", "& 'static str", None), same_type("Map<K>", "Map < 'a , 'b , K >", None)),
            (true, true, true),
            "a document that spells the lifetime is not more correct than one that does not"
        );
        assert_eq!(
            (same_type("Cow<String>", "Cow < 'a , str >", None), same_type("Map", "Map < 'a , 'b , K >", None), same_type("Vec<T>", "Vec < T", None)),
            (false, false, false),
            "erasing a lifetime never erases the arguments beside it, and `Vec<T` is not the shape `Vec<T>` is"
        );
    }

    #[test]
    #[validates(spec::AnArgumentListLeftWithNoArgumentIsNotWrittenAtAll)]
    fn an_argument_list_left_with_no_argument_is_not_written_at_all() {
        // The list is written afresh from whatever survived, so a list of
        // none, a list of one and a list of two are written the same way.
        let emptied = stripped("Section < 'a >");

        assert_eq!(emptied, "Section", "the brackets go with the only argument the list had");
        assert_ne!(emptied, "Section<>", "which is the shape this decision exists to forbid");
        assert_eq!(
            [stripped("Cow < 'a , str >"), stripped("Map < 'a , 'b , K >"), stripped("( Vec < u8 > , Section < 'a > )")],
            ["Cow<str>", "Map<K>", "(Vec<u8>,Section)"],
            "a list that keeps an argument keeps its brackets, and a sibling list is written on its own"
        );
        assert_eq!(
            (same_type("Section", "Section < 'a >", None), same_type("Section<>", "Section < 'a >", None)),
            (true, false),
            "a row writing the type without its erased lifetime agrees; one writing empty brackets does not"
        );
    }

    #[test]
    #[validates(spec::SelfIsTheDeclarationsOwnerOnBothSides)]
    fn self_is_the_declarations_owner_on_both_sides() {
        let owned_elsewhere = declaration("read", Some("Other"), &["& Project", "& str"], Some("Result < Self , String >"));
        let says_lld = fragment(&["Lld", "read"], Some(&["project", "slice"]), Some("Result<Lld, String>"));

        assert_eq!(
            (
                same_type("Result<Lld, String>", "Result < Self , String >", Some("Lld")),
                same_type("Returned", "Self", Some("Returned < 'a >")),
                same_type("Self", "Lld", Some("Lld")),
            ),
            (true, true, true),
            "the owner stands for `Self` on whichever side writes it, itself read as every token string here is read"
        );
        assert_eq!(
            (same_type("Lld", "Self", None), same_type("Self", "Self", None), same_type("Lld", "Lld", None)),
            (false, false, true),
            "a free function has no owner, so a `Self` in a row about one stands for no type and the row disagrees"
        );
        assert!(!agrees(&says_lld, &owned_elsewhere), "the substitution is the declaration's owner and never the fragment's own qualifier");
    }

    #[test]
    #[validates(spec::AnAgreementFailureNamesTheItemsFileAndBothReadings)]
    fn an_agreement_failure_names_the_items_file_and_both_readings() {
        let doc = PathBuf::from("/w/app/src/thing/lld.md");
        let source = declaration("read", Some("Lld"), &["& Project", "& str"], Some("Result < Self , String >"));
        let wrote = source_wrote(&source);
        let failure = agreement_failure(Check::ShapeAgrees, &doc, 12, &source.file, "Lld::read(project)", &wrote);
        let places = ["/w/app/src/thing/mod.rs", "Lld::read(project)", wrote.as_str(), rule(Check::ShapeAgrees)].map(|part| failure.message.find(part));

        assert_eq!(located(&failure), (Check::ShapeAgrees, doc.as_path(), 12), "the document's path and the row's line, as every failure carries them");
        assert!(
            places.iter().all(Option::is_some) && places.windows(2).all(|pair| pair[0] < pair[1]),
            "the item's file, what the row said, what the source said, and then the rule: {}",
            failure.message
        );
        assert_eq!(
            (qualified(&source).as_str(), returning(None).as_str(), returning(Some("u8")).contains("-> u8")),
            ("Lld::read", "", true),
            "the source is spelled back qualified by its owner, with the return it declared and nothing where it declared none"
        );
        // What the source said is the declaration's own three parts, and the
        // message has to hold them: the name its owner qualifies, the parameter
        // tokens the reading answered, and the return the source declared, in
        // the order a signature writes them. The spelling that joins them is
        // evidence rather than a grammar, so nothing here asks for its
        // punctuation — but a message that carries none of the item's parts
        // cites the row against nothing, and a reader has no evidence to fix it
        // against.
        let spelled = ["Lld::read", "& Project", "& str", "Result < Self , String >"].map(|part| failure.message.find(part));

        assert!(
            spelled.iter().all(Option::is_some) && spelled.windows(2).all(|pair| pair[0] < pair[1]),
            "the source's own name, its parameters and its return are in the message, in the order a signature writes them: {}",
            failure.message
        );
        // What the row said is the identifier as the cell wrote it, which
        // `written_fragments` pairs with the fragment read from that same
        // identifier — so a cell naming two items has a pair for each, and a
        // failure quotes the row's own text rather than a spelling this slice
        // invented for it.
        let row = Row {
            line: 12,
            cells: strings(&["`Lld::read(project, slice) -> Result<Lld, String>` and `run(args)`", "two items in one cell"]),
        };
        let paired: Vec<String> =
            written_fragments(&row).iter().map(|(written, fragment)| format!("{written} => {}", as_written(fragment))).collect();

        assert_eq!(
            paired,
            [
                "Lld::read(project, slice) -> Result<Lld, String> => Lld::read(project, slice) -> Result<Lld, String>",
                "run(args) => run(args), no return",
            ],
            "one pair per identifier the cell holds, each the text the row wrote beside the fragment read from it"
        );
    }

    #[test]
    #[validates(spec::AReuseRowNamesItsItemAsAnIntraDocLink)]
    fn a_reuse_row_names_its_item_as_an_intra_doc_link() {
        let (_, project) = slice_with_source("lld-review-reuse", SLICE_SOURCE, REUSE_ROWS);
        let doc = Lld::read(&project, "thing").expect("the member holds the slice's document");

        assert_eq!(
            all_located(&reuse_rows_linked(&project, &doc)),
            [(
                Check::ReuseRowsLinked,
                doc.path.as_path(),
                line_at(REUSE_ROWS, "| `layout::lld_path(project, slice)` | another slice's item in the same crate, named bare |")
            )],
            "the linked row, the slice's own name, a module inside its directory, a member's crate name and a type all hold"
        );
    }

    #[test]
    #[validates(
        spec::AModuleQualifiedPathOutsideTheSlicesDirectoryIsAReuseRow,
        spec::APathThatIsNotModuleQualifiedIsNotAReuseRow,
        spec::APathIntoTheSlicesOwnModulesIsNotAReuseRow,
        spec::APathQualifiedByAWorkspaceMembersCrateNameIsNotAReuseRow,
    )]
    fn a_module_qualified_path_outside_the_slices_directory_is_a_reuse_row() {
        let inside = strings(&["spec", "lld"]);
        let crates = strings(&["cargo_lid_rs", "lid_rs_shape"]);
        let asked = |path: &[&str]| is_reuse(&fragment(path, Some(&["a"]), None), "lld-review", &inside, &crates);

        assert!(asked(&["layout", "lld_path"]) && asked(&["phase", "plan"]), "a sibling module of the crate is another slice's, and its row is asked for a link");
        assert!(!asked(&["run"]), "a bare name names nothing elsewhere");
        assert!(!asked(&["Lld", "read"]) && !asked(&["_private", "held"]), "a first segment that does not start with a lowercase letter is no module");
        assert!(!asked(&["lld_review", "run"]) && !asked(&["spec", "ClaimName"]), "the slice's own name in module form, and a sibling inside its directory");
        assert!(
            !asked(&["cargo_lid_rs", "catalog", "entry"]) && !asked(&["lid_rs_shape", "signatures"]),
            "a path into a crate is not a path into a sibling module, and rustdoc could not resolve a link into a crate that is no dependency"
        );
    }

    #[test]
    #[validates(spec::ASiblingIsAFileStemOrADirectoryNameOneLevelInsideTheSlice, spec::ASliceWithNoDirectoryToReadListsNoSibling)]
    fn a_sibling_is_a_file_stem_or_a_directory_name_one_level_inside_the_slice() {
        let (root, project) = slice_with_source("lld-review-siblings", SLICE_SOURCE, HOLDS);
        write_at(&root.join("app/src/thing/nested/mod.rs"), "//! A module one level inside the slice.\n");
        write_at(&root.join("app/src/thing/nested/deeper.rs"), "//! Two levels in, and no sibling of the slice.\n");

        assert!(siblings(&project, "skill").is_empty(), "a slice with no directory to read lists nothing, and not an error");

        let mut listed = siblings(&project, "thing");
        listed.sort();

        assert_eq!(
            listed,
            strings(&["inner", "lld", "nested"]),
            "a file's stem without its extension and a directory's own name, one level in, with `mod.rs` left out and nothing deeper listed"
        );
    }

    #[test]
    #[validates(spec::ACrateNameIsAMembersPackageNameWithHyphensAsUnderscores)]
    fn a_crate_name_is_a_members_package_name_with_hyphens_as_underscores() {
        let root = fixture::scratch("lld-review-crate-names");
        let project = project_naming(&root, &[("app", "app"), ("shape", "lid-rs-shape")]);

        assert_eq!(crate_names(&project), strings(&["app", "lid_rs_shape"]), "`lid-rs-shape` is the package's name and `lid_rs_shape` is the path's");
        assert!(crate_names(&project_naming(&root, &[])).is_empty(), "a workspace with no member names no crate");
    }

    #[test]
    #[validates(spec::AnImplTraitOrBareDynTraitReturnIsNotSkeletonable, spec::AReturnThatMerelyContainsAnImplOrDynTypeIsSkeletonable)]
    fn an_impl_trait_or_bare_dyn_trait_return_is_not_skeletonable() {
        let refused = ["impl Iterator<Item = u8>", "impl Trait", "dyn Error", "dyn Fn(u8) -> u8"].map(skeletonable);
        let held = ["Box<dyn Error>", "Result<Box<dyn Error>, String>", "Result<Lld, String>", "implementation::Handle", "dynamics::Handle"].map(skeletonable);

        assert_eq!(refused, [false; 4], "a `todo!()` body infers an `impl Trait`'s hidden type as `!`, which implements nothing, and a bare `dyn Trait` is unsized");
        assert_eq!(held, [true; 5], "the test is over the return's outermost form and nothing deeper, and over its first word and not a prefix of one");
    }

    #[test]
    #[validates(spec::ARowWhoseReturnCannotBeSkeletonisedFailsOnItsLine)]
    fn a_row_whose_return_cannot_be_skeletonised_fails_on_its_line() {
        let doc = document("thing", RETURN_ROWS);

        assert_eq!(
            all_located(&shape_returns(&doc)),
            [
                (Check::SkeletonableReturns, doc.path.as_path(), line_at(RETURN_ROWS, "| `hidden() -> impl Iterator<Item = u8>` | a hidden type a `todo!()` body infers as `!` |")),
                (Check::SkeletonableReturns, doc.path.as_path(), line_at(RETURN_ROWS, "| `erased() -> dyn Error` | unsized in return position |")),
            ],
            "each unskeletonable row fails on its own line; the rows that merely contain one, and the row writing no return at all, hold"
        );
        assert!(shape_returns(&document("thing", HOLDS)).is_empty(), "a row writing no return has none to refuse");
    }
}

pub mod spec;
