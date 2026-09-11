#![doc = include_str!("lld.md")]

pub mod spec;

use std::path::Path;

use lid_rs::implements;
use serde::{Deserialize, Serialize};

/// One finding, in pipeline §5.3's schema: what this project says about one
/// diagnostic, rather than the diagnostic relabelled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The LID-rs check the diagnostic belongs to, and 0 where no check names
    /// the lint that raised it — carried rather than dropped.
    pub check: u32,
    /// The LID-rs rule code, where the check has one.
    pub rule: Option<String>,
    /// How the tool graded the diagnostic.
    pub severity: String,
    /// The file the diagnostic points at, where it points at a place in the
    /// source.
    pub file: Option<String>,
    /// The line the diagnostic points at, where it points at a place in the
    /// source.
    pub line: Option<u32>,
    /// The item the check is about, where it is about one.
    pub item: Option<String>,
    /// The claim the check is about, where it is about one.
    pub claim: Option<String>,
    /// What the diagnostic says.
    pub message: String,
    /// The correct response, read from the skill's `references/gates.md` row
    /// for this finding's check; absent where that table holds no row for it.
    pub fix: Option<String>,
    /// The command whose run produced the finding.
    pub source: String,
}

/// A command's run as it is written to `target/lid/<command>.json`: what the
/// run found, and what those findings amount to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// The command that was run, which the report's file is named for.
    pub command: String,
    /// What the run found — empty where it found nothing, which is written
    /// too, so that an empty report and an absent one are different things.
    pub findings: Vec<Finding>,
    /// What the run amounts to, where a reader finds it though the process's
    /// exit code cannot hold it (Deferred 4).
    pub status: Status,
}

/// The three answers a run can reach, as a closed set whose discriminants are
/// the exit codes — so the numbers the claims name are stated once and a
/// caller cannot read a status without interpreting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
#[implements(spec::TheStatusIsZeroWithNoFindingAndOneWithAFinding, spec::ARunThatReachesNoAnswerIsTheToolingStatus)]
pub enum Status {
    /// The project was asked and there was nothing to find.
    Pass = 0,
    /// The project was asked and the findings are the answer.
    Findings = 1,
    /// No answer about the project was reached at all: the harness's problem,
    /// which a pipeline adjudicates rather than retries.
    Tooling = 2,
}

/// Why an entry is not built: one of the three things a reason can be, so that
/// a command with no slice to name is not made to name one, and a command this
/// tool builds at no point is not made to name a question it never waits on.
///
/// The third case is a distinction and not bookkeeping: a command waiting on a
/// slice or on a question is one this tool builds one day, and a command owned
/// elsewhere is one it never builds. With two cases `pr-body`, `status` and
/// `commit` — the pipeline's own — would carry "the pipeline" in a variant
/// called [`Question`](Unbuilt::Question), which is the lie a field typed
/// `slice` would have told about the rows that wait on a deferral, in the other
/// shape and for three of the seventeen rows.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere)]
pub enum Unbuilt {
    /// The slice the command lands with, named as the design names it.
    Slice(&'static str),
    /// The question the command waits on, named as the design names it.
    Question(&'static str),
    /// The thing that owns the command where this tool builds it at no point,
    /// named as the design names it.
    Owner(&'static str),
}

/// What a command is: the fixed invocation and where its findings come from,
/// or unbuilt and why. One enum, so a built entry has exactly one invocation
/// and an unbuilt one exactly one reason, and the two cannot contradict each
/// other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// A cargo invocation that emits a JSON diagnostic stream, whose findings
    /// are read from that stream.
    JsonDiagnostics {
        /// The arguments cargo is run with, fixed by the entry and taking
        /// nothing from the command line.
        args: Vec<String>,
    },
    /// A cargo invocation whose tool emits no JSON diagnostic stream, whose
    /// findings are read from what it printed to stderr.
    StderrDiagnostics {
        /// The arguments cargo is run with, fixed by the entry and taking
        /// nothing from the command line.
        args: Vec<String>,
        /// The environment the invocation runs under: variable and value.
        env: Vec<(String, String)>,
    },
    /// The `sync --check` comparison, which is this binary's own work rather
    /// than a tool's, and whose message this slice cannot take apart
    /// (Deferred 7).
    SyncComparison,
    /// No invocation: this workspace does not build the command, for the
    /// reason carried.
    Unbuilt(Unbuilt),
}

/// An atomic command: its name, and what it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The name `cargo lid-rs <command>` runs it by, which the report file is
    /// named for.
    pub name: &'static str,
    /// What the command is: a fixed invocation with a finding provenance, or
    /// unbuilt with the reason.
    pub invocation: Invocation,
}

/// A tool's output, captured whole — the answer a [`Runner`] gives.
///
/// Both streams are carried whichever one a command's findings come from, so
/// that the choice between them is made in [`run_with`], where a wrong answer
/// about a command's provenance can be seen, and not in the spawning half,
/// which carries no citation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolOutput {
    /// What the tool wrote to stdout — cargo's JSON diagnostic stream, where
    /// the tool emits one.
    pub stdout: String,
    /// What the tool wrote to stderr — where a tool with no JSON stream
    /// prints its diagnostics, and where the `sync` comparison's message
    /// arrives.
    pub stderr: String,
}

/// One diagnostic as this slice means it: what a tool said about one place in
/// the source, read from whichever stream that tool carries its diagnostics on.
///
/// Both provenances answer with this, so that what a diagnostic *becomes* — the
/// check number of the lint that raised it, the `fix` for that check, the place
/// it points at — is [`finding_of`]'s one rule rather than one rule per stream.
/// A rule written on one stream's mapper holds of the other only by accident of
/// where it was written, and a mapper that answered `file: None, line: None,
/// check: 0` for everything would satisfy the claims of the stream it was not
/// written for.
///
/// It is a domain type and not a `serde_json::Value` because a tool with no
/// JSON stream would otherwise have to *build* JSON in order to be read, and
/// every item below the boundary would depend on the shape of a foreign crate's
/// parse rather than on what this slice means by a diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The lint that raised it, where a lint did, under the name the lint is
    /// declared by — `clippy::cognitive_complexity`, prefix and underscores —
    /// rather than the spelling a particular tool printed. Each stream's reader
    /// answers in that one spelling, so which check a lint belongs to is decided
    /// once for both of them.
    pub lint: Option<String>,
    /// How the tool graded it, in the tool's own words: the grade a
    /// [`Finding`] carries, which nothing below the stream it arrived on knows.
    pub severity: String,
    /// What the tool said about it.
    pub message: String,
    /// The file it points at, where it points at a place in the source.
    pub file: Option<String>,
    /// The line it points at, where it points at a place in the source.
    pub line: Option<u32>,
}

/// An [`Invocation`] to a tool's output: what [`run`] supplies by spawning and
/// a validation supplies by recording.
pub trait Runner {
    /// Runs one invocation, answering the tool's output, or the reason the
    /// tool could not be run at all.
    fn run(&mut self, invocation: &Invocation) -> Result<ToolOutput, String>;
}

/// Any closure from an invocation to a tool's output is a runner, so that the
/// spawning half and a recording validation each need a type of their own.
impl<F: FnMut(&Invocation) -> Result<ToolOutput, String>> Runner for F {
    fn run(&mut self, invocation: &Invocation) -> Result<ToolOutput, String> {
        self(invocation)
    }
}

/// Every command the catalog holds — pipeline §5.1's sixteen and the
/// `package` step this workspace adds beside them — built and unbuilt, each
/// unbuilt one with its reason. What `cargo lid-rs catalog` prints, and what a
/// name is refused against.
///
/// `publishing` names the members `package` runs for, which is the workspace's
/// answer rather than the catalog's.
#[implements(
    spec::TheCatalogNamesEveryCommandAndWhetherItIsBuilt,
    spec::AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere,
    spec::PackageNamesEveryPublishingMemberInOneInvocation,
    spec::TheDocInvocationNamesTheFlagThatDocumentsPrivateItems,
    spec::TheDocInvocationCarriesTheEnvironmentThatDeniesABrokenLink,
)]
pub fn table(publishing: &[String]) -> Vec<Command> {
    todo!("the seventeen entries of the document's table, built from {publishing:?}")
}

/// One command by name: resolve the workspace, read the `gates.md` a finding's
/// `fix` comes from, spawn for the invocation the entry names, and answer with
/// the report [`run_with`] made.
///
/// The I/O wrapper — it reads and it spawns, and carries no citation: every
/// decision a run makes is [`run_with`]'s, where a validation can see it
/// without a build. The gate table is read here, from
/// `<root>/<SKILL_IN_PROJECT>/references/gates.md`
/// ([`crate::sync::SKILL_IN_PROJECT`]), and handed on as text, so that
/// [`run_with`] needs no skill tree on disk to answer with.
///
/// The `Err` is the whole error channel and holds only what fails *before* any
/// command is run: a workspace that cannot be resolved, and a `gates.md` that
/// cannot be read. Neither is a run whose report went unwritten — with no root
/// there is no `target/lid` to write one into, and the `fix` of every finding
/// comes from that table, so a run over a table that could not be read would
/// answer with the empty `fix` lines the document amended itself to prevent.
/// Both are the harness's problem rather than an answer about the project, and
/// both collapse to exit 1 in `main` until Deferred 4 is answered.
pub fn run(command: &str) -> Result<Report, String> {
    todo!("read the gate table, spawn for `{command}`'s invocation, and answer `run_with`'s report")
}

/// The same over an injected runner: every decision a run makes, with nothing
/// spawned.
///
/// The dispatch and only the dispatch: which entry `command` resolves to in
/// [`table`], and that the three answers differ.
///
/// A name the catalog holds no entry for is refused by the name it was given
/// and written to no file — there is no entry whose report it would be, and no
/// entry to take a file name from, so the caller's `command` never reaches
/// [`write_report`]. An entry the catalog holds as unbuilt is refused with that
/// entry's own reason and *is* written, because a pipeline reading
/// `target/lid/<command>.json` for a command the catalog holds is owed the
/// tooling status it would find there. Everything a built entry's run decides
/// — the tool that could not run, the stream its provenance reads, the status
/// its findings amount to, the directory its report goes in — belongs to
/// [`run_entry`] and [`written`] below, where a wrong answer about one of them
/// is an answer this item never gave.
///
/// `gates` is the text of the synced skill's `references/gates.md`, read by
/// [`run`] and passed in whole: a validation supplies the rows it wants to
/// assert about as a string, rather than materialising a skill tree under
/// `root` to test a status mapping.
///
/// `root`, `publishing` and `gates` stay three parameters rather than one
/// context struct. Each reaches a different depth — `publishing` no further
/// than [`table`], `gates` no further than the mappers, `root` no further than
/// the write — so one context would hand all three to items that can only be
/// wrong about one, and would be a type the design's shape table does not name.
#[implements(
    spec::ACommandRunsTheFixedInvocationItsEntryNames,
    spec::ANameTheCatalogHoldsNoEntryForIsRefusedByName,
    spec::ARefusalOfANameWithNoEntryWritesNoFile,
    spec::AnUnbuiltCommandRefusesWithTheCatalogsReason,
    spec::ARunThatReachesNoAnswerIsTheToolingStatus,
)]
pub fn run_with(command: &str, root: &Path, publishing: &[String], gates: &str, runner: &mut impl Runner) -> Report {
    match table(publishing).into_iter().find(|entry| entry.name == command) {
        None => refused(command, format!("`{command}` is no command this catalog holds an entry for")),
        Some(Command { name, invocation: Invocation::Unbuilt(reason) }) => {
            written(root, name, refused(name, unbuilt_sentence(name, &reason)))
        }
        Some(entry) => written(root, entry.name, run_entry(&entry, gates, runner)),
    }
}

/// One built entry's run: the [`Runner`] is handed the invocation that entry
/// names, taking nothing from the command line, and what it answers becomes the
/// report.
///
/// The dispatch over the runner's answer. An `Err` is a tool that could not be
/// run at all — a fourth occasion of reaching no answer about the project,
/// beside the three the design names, judged against the rule rather than
/// listed by it. An `Ok` is an output to be read the way the entry's provenance
/// names, whose findings are then what the run amounts to.
#[implements(spec::ACommandRunsTheFixedInvocationItsEntryNames, spec::ARunThatReachesNoAnswerIsTheToolingStatus)]
fn run_entry(entry: &Command, gates: &str, runner: &mut impl Runner) -> Report {
    match runner.run(&entry.invocation) {
        Err(why) => refused(entry.name, why),
        Ok(output) => report_of(entry.name, findings_of(&entry.invocation, &output, gates, entry.name)),
    }
}

/// Which of a tool's two streams an entry's findings are read from, and by
/// which mapper: the entry's provenance decides both.
///
/// The [`Runner`] answers both streams whole, so this is the one item a wrong
/// answer about a command's provenance comes from — reading `cargo doc`'s
/// diagnostics from an empty stdout, or `cargo check`'s from a stderr that
/// holds only its progress lines, loses every diagnostic the stream held.
/// Leaving the choice to the runner would have put it in the spawning half,
/// which carries no citation.
///
/// An entry with no invocation ran no tool, so it has no stream to read; the
/// arm answers the catalog's own refusal, composed by the same
/// [`unbuilt_sentence`] [`run_with`]'s unbuilt arm refuses with, so that this
/// total match has no arm whose answer nobody wrote down. [`run_with`] refuses
/// such an entry before any runner is called, so nothing reaches it here.
#[implements(
    spec::EveryDiagnosticOfTheStreamBecomesAFinding,
    spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr,
    spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout,
)]
fn findings_of(invocation: &Invocation, output: &ToolOutput, gates: &str, source: &str) -> Vec<Finding> {
    match invocation {
        Invocation::JsonDiagnostics { .. } => findings_from_cargo(&output.stdout, gates, source),
        Invocation::StderrDiagnostics { .. } => findings_from_stderr(&output.stderr, gates, source),
        Invocation::SyncComparison => sync_findings(&output.stderr, source),
        Invocation::Unbuilt(reason) => vec![tooling_finding(source, unbuilt_sentence(source, reason))],
    }
}

/// The `sync` comparison's message to findings: one finding carrying it whole
/// where the comparison reported differences, and none where it reported none.
///
/// The granularity this slice can state honestly — `sync::check` joins its
/// differences into one string and its three comparison helpers are private to
/// a module no phase of this slice may write, so one finding per difference
/// would need either that module's change or the re-parse this slice's own
/// rendering rule forbids (Deferred 7). A finding on every `sync` run whatever
/// the comparison reported would make a clean workspace's `sync` exit 1 and the
/// gate step fail forever on a passing run.
#[implements(spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout)]
fn sync_findings(message: &str, source: &str) -> Vec<Finding> {
    todo!("`{source}`: one finding carrying {} bytes of comparison message, or none where it found none", message.len())
}

/// The report of a run that reached an answer about the project: what it found,
/// and the [`Status`] that finding list amounts to — 0 where the list is empty,
/// 1 where it holds a finding.
///
/// The status is carried in the report rather than answered alone, because
/// `main` maps every error to exit 1 and sits in no phase's allowed paths, so
/// until that changes the report is where a reader finds it (Deferred 4).
#[implements(spec::TheStatusIsZeroWithNoFindingAndOneWithAFinding, spec::TheReportCarriesTheRunsStatus)]
fn report_of(command: &str, findings: Vec<Finding>) -> Report {
    todo!("the report of `{command}`, whose status is what its {} findings amount to", findings.len())
}

/// The report of a run that reached no answer about the project: one finding —
/// [`tooling_finding`]'s — carrying the sentence it refused with, and the
/// tooling status.
///
/// The answer at every occasion where nothing produced a finding list to map: a
/// name with no entry, an entry the catalog holds as unbuilt, a tool that could
/// not be run, and a report that could not be written. What such a run had
/// found is not carried, because what it found is not the answer.
#[implements(spec::ARunThatReachesNoAnswerIsTheToolingStatus, spec::TheReportCarriesTheRunsStatus)]
fn refused(command: &str, why: String) -> Report {
    todo!("refuse `{command}`, reaching no answer: {why}")
}

/// The finding a run that reached no answer carries: the sentence it refused
/// with, against no check, since nothing about the project was read.
fn tooling_finding(source: &str, why: String) -> Finding {
    todo!("the finding `{source}` refuses with: {why}")
}

/// The report a run answers with, once the file it belongs in has been written:
/// the report itself where it was written, and a refusal naming what could not
/// be done where it was not.
///
/// The dispatch over the write's outcome. A report that could not be written is
/// no answer about the project rather than a silent skip — what the run found
/// persists nowhere, so a reader who has only the file would otherwise be told
/// nothing at all.
#[implements(spec::ARunWritesItsReportEvenWhenItFindsNothing, spec::ADirectoryThatCannotBeCreatedIsNamedByAFinding)]
fn written(root: &Path, name: &'static str, report: Report) -> Report {
    match write_report(root, name, &report) {
        Ok(()) => report,
        Err(why) => refused(name, why),
    }
}

/// Writes one entry's report to `<root>/target/lid/<name>.json`, creating that
/// directory where the workspace holds none — always, whatever the run found,
/// so that an empty report and an absent one stay different things.
///
/// `name` is the entry's own `&'static str` and never the caller's `command`.
/// [`run_with`]'s `command` is arbitrary user input, and
/// `root.join("target/lid").join(format!("{command}.json"))` for `../../foo`
/// writes outside the workspace; taking the name from the table makes that
/// traversal unrepresentable rather than sanitised, and a borrowed `command`
/// cannot reach this parameter at all. The other half of that property — that
/// a name with no entry never reaches this item — is [`run_with`]'s dispatch,
/// and is cited there, since no answer this item could give would change it.
///
/// Creating the directory and writing the file are one fallible sequence: the
/// `Err` names what could not be done and the path it could not be done at,
/// and [`written`] is what turns that into no answer about the project.
#[implements(
    spec::ARunWritesItsReportEvenWhenItFindsNothing,
    spec::TheReportDirectoryIsCreatedOnDemand,
    spec::ADirectoryThatCannotBeCreatedIsNamedByAFinding,
)]
fn write_report(root: &Path, name: &'static str, report: &Report) -> Result<(), String> {
    todo!("write `{name}`'s {} findings under {}", report.findings.len(), root.display())
}

/// Why this workspace does not build a command, as one sentence naming the
/// command and the reason its entry carries.
///
/// Composed here and nowhere else. `cargo lid-rs catalog` prints this for every
/// unbuilt entry it lists, and [`run_with`] refuses an unbuilt name with the
/// same sentence; the printer is another slice's (Deferred 8), so this is `pub`
/// for it to call rather than compose its own — a second composition is the
/// "six places state a gate" disease this slice exists to cure.
#[implements(spec::AnUnbuiltCommandRefusesWithTheCatalogsReason)]
pub fn unbuilt_sentence(name: &str, reason: &Unbuilt) -> String {
    todo!("why `{name}` is not built here: {reason:?}")
}

/// Cargo's JSON diagnostic stream to findings: the stream read into
/// [`Diagnostic`]s, each turned into a [`Finding`] by the rule both provenances
/// share.
///
/// The provenance whole. The parse and the mapping are separate items, and this
/// composition is where "every diagnostic the stream held reaches a finding"
/// is answered: neither half answers it alone, since the reader answers with no
/// finding and the mapper is never handed the stream.
///
/// `gates` is the text of the synced skill's `references/gates.md`, whose rows
/// are keyed by check; `source` is the command the findings are recorded
/// against.
#[implements(spec::EveryDiagnosticOfTheStreamBecomesAFinding)]
pub fn findings_from_cargo(stream: &str, gates: &str, source: &str) -> Vec<Finding> {
    diagnostics_from_cargo(stream).into_iter().map(|diagnostic| finding_of(diagnostic, gates, source)).collect()
}

/// The same for the tools that emit no JSON stream — rustdoc and `cargo
/// package` — whose diagnostics are read from what they printed to stderr:
/// that stderr read into [`Diagnostic`]s, each turned into a [`Finding`] by the
/// same rule, so that a finding of this provenance carries what a finding of
/// the other one does.
///
/// `gates` and `source` are [`findings_from_cargo`]'s.
#[implements(spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr)]
pub fn findings_from_stderr(stderr: &str, gates: &str, source: &str) -> Vec<Finding> {
    diagnostics_from_stderr(stderr).into_iter().map(|diagnostic| finding_of(diagnostic, gates, source)).collect()
}

/// Cargo's JSON diagnostic stream to diagnostics: one [`Diagnostic`] for each
/// compiler message the stream carries.
///
/// The boundary where `serde_json` is used and below which it is not. Cargo's
/// stream arrives as JSON and a library type belongs at the boundary that reads
/// it; nothing this item answers with holds a `Value`, so no item below depends
/// on the shape of a foreign crate's parse.
///
/// A record that is not a compiler message — cargo's artifact and
/// build-finished lines — is no diagnostic and reaches no finding. A compiler
/// message whose lint no check names is a diagnostic all the same and is
/// carried, because a reader that drops what it does not recognise reports
/// success over what it failed to read; the check number that names no check is
/// 0, answered where check numbers are, and not a diagnostic dropped here.
pub fn diagnostics_from_cargo(stream: &str) -> Vec<Diagnostic> {
    todo!("read the diagnostics of {} bytes of cargo's JSON stream", stream.len())
}

/// A tool's stderr to diagnostics: one [`Diagnostic`] for each diagnostic the
/// tool printed there, for rustdoc and `cargo package`, which carry them on no
/// other stream.
///
/// The other reader, answering the same domain type as [`diagnostics_from_cargo`]
/// so that what a diagnostic becomes is not written twice. Where a tool prints a
/// lint's name in a spelling of its own — hyphens for the underscores the lint
/// is declared with — the declared spelling is what is answered, so that the
/// check a lint belongs to is decided for both streams at once and neither
/// reader decides what a diagnostic means.
///
/// What stderr holds besides diagnostics — a tool's progress and summary lines —
/// is not one and reaches no finding.
pub fn diagnostics_from_stderr(stderr: &str) -> Vec<Diagnostic> {
    todo!("read the diagnostics of {} bytes of a tool's stderr", stderr.len())
}

/// One diagnostic to a [`Finding`]: the check number of the lint that raised it,
/// the `fix` from the `gates.md` row for that check, and the file and line it
/// points at.
///
/// The one rule for both provenances, so that a finding read from a stderr
/// carries what a finding read from cargo's JSON carries — not because the two
/// readers agree, but because neither of them decides it.
///
/// `rule`, `item` and `claim` are absent from every finding this item builds.
/// The schema carries them for the checks that are about a citation, a graph
/// item or a claim — `shape`, `conform`, `validate` — and none of those is built
/// here; what a compiler or rustdoc diagnostic names is a place in the source,
/// and `gates.md`'s rows carry no rule code to read one from.
///
/// `gates` is the text of the synced skill's `references/gates.md`; `source` is
/// the command the finding is recorded against.
#[implements(
    spec::AFindingNamesTheFileAndLineOfItsDiagnostic,
    spec::TheCheckNumberComesFromTheLintOrIsZero,
    spec::TheFixLineIsTheGatesRowForTheFindingsCheck,
)]
pub fn finding_of(diagnostic: Diagnostic, gates: &str, source: &str) -> Finding {
    let check = check_of_lint(diagnostic.lint.as_deref());
    Finding {
        check,
        rule: None,
        severity: diagnostic.severity,
        file: diagnostic.file,
        line: diagnostic.line,
        item: None,
        claim: None,
        message: diagnostic.message,
        fix: fix_of_check(check, gates),
        source: source.to_string(),
    }
}

/// The LID-rs check a lint belongs to, and 0 for a diagnostic no check names —
/// one that was raised by a lint this project's gate does not state, and one
/// that no lint raised at all.
///
/// The dispatch over the lint's declared name. It is one decision and not two:
/// a diagnostic with no lint and a diagnostic whose lint no check names are
/// alike in what can be said about them, and both are carried under 0 rather
/// than dropped.
#[implements(spec::TheCheckNumberComesFromTheLintOrIsZero)]
fn check_of_lint(lint: Option<&str>) -> u32 {
    todo!("the check {lint:?} belongs to, or none")
}

/// The correct response the skill's `references/gates.md` states for a check,
/// which is the `fix` a finding of that check carries; absent where that table
/// holds no row for it.
///
/// `gates` is the table's text rather than a path, so that what a finding's
/// `fix` says is decided from rows a caller can hand over and no run of this
/// item needs a skill tree on disk.
#[implements(spec::TheFixLineIsTheGatesRowForTheFindingsCheck)]
fn fix_of_check(check: u32, gates: &str) -> Option<String> {
    todo!("the gates row for check {check}, among {} bytes of table", gates.len())
}

/// A report to the human rendering, built from the findings alone and never
/// from the output of the tool the command ran, so that the two cannot
/// disagree.
#[implements(spec::TheRenderingIsBuiltFromTheFindingsAndNotTheToolsOutput)]
pub fn render(report: &Report) -> String {
    todo!("render {}'s findings", report.command)
}

#[cfg(test)]
mod tests {
    //! What these validations observe, and how they observe it without
    //! spawning a build.
    //!
    //! The design injects the runner for exactly this. A run's decisions —
    //! which invocation it hands the tool, what it writes, what status it
    //! answers with, what it refuses — are seen by handing `run_with` a
    //! closure that *records* the `Invocation` it was given and answers a
    //! canned `ToolOutput`. Nothing here runs cargo, and a wrong answer to the
    //! no-passthrough rule is visible in what the recorder holds.
    //!
    //! Four fixtures stand in for what a run would otherwise need on disk:
    //!
    //! | Fixture | Stands for |
    //! |---|---|
    //! | `GATES` | three rows of the synced skill's `references/gates.md`, passed as the text `run_with` takes, so that no test materialises a skill tree to assert a `fix` line |
    //! | `CARGO_STREAM` | cargo's JSON diagnostic stream, holding the records that are no diagnostic among the three that are |
    //! | `RUSTDOC_STDERR` | the other provenance: a tool that prints its diagnostics to stderr, in its own spelling of a lint's name, among its progress and summary lines |
    //! | `blocked_root` | a workspace root whose `target` is a **regular file**, so `create_dir_all` fails on macOS and Linux alike and `path.is_dir()` is false afterwards — a read-only parent is the wrong mechanism, since it silently succeeds for a test run as root |
    //!
    //! `GATES` deliberately holds no row for check 9, so that a finding whose
    //! check the table holds no row for can be told from one whose row it
    //! holds; check 2's row is keyed `check 2/5`, as the real table keys it.
    //!
    //! **Why each test collects and asserts once.** Check 7 counts every
    //! `assert!` as a branch, so a test of more than three of them is a
    //! function with undeclared decisions in it, threshold or no threshold. The
    //! answer here is the one the rest of this workspace uses: gather what was
    //! observed into one value and compare it with one expected value, so that
    //! a failure prints every case at once instead of the first.

    use super::*;
    use lid_rs::validates;

    use std::path::PathBuf;

    /// Three rows in `references/gates.md`'s shape: the check the row is keyed
    /// by, what the gate means, and the correct response a finding's `fix`
    /// carries.
    const GATES: &str = "\
| Gate | It means | Correct response |
|---|---|---|
| check 2/5 — doc link or example broken | Docs/LLD drifted from the API | Fix the doc or the LLD — they are intent, not decoration |
| check 3 — missing docs | An item exists with no stated intent | Write the intent; if you can't state it, the item shouldn't exist yet |
| check 7 — cognitive complexity | A leaf contains decisions nobody declared | Return to Phase 1. Write the claim, or restructure |
";

    /// The correct response the table above states for check 7.
    const FIX_SEVEN: &str = "Return to Phase 1. Write the claim, or restructure";

    /// The correct response it states for the row keyed by checks 2 and 5.
    const FIX_TWO: &str = "Fix the doc or the LLD — they are intent, not decoration";

    /// The file the fixtures' diagnostics point at.
    const SOURCE_FILE: &str = "cargo-lid-rs/src/catalog/mod.rs";

    /// Cargo's JSON diagnostic stream: three compiler messages — a lint the
    /// gate states, a lint it does not, and a diagnostic no lint raised —
    /// among the records that are no diagnostic at all.
    const CARGO_STREAM: &str = r#"{"reason":"compiler-artifact","target":{"name":"cargo-lid-rs"}}
{"reason":"compiler-message","message":{"code":{"code":"clippy::cognitive_complexity"},"level":"warning","message":"the function has a cognitive complexity of (5/4)","spans":[{"file_name":"cargo-lid-rs/src/catalog/mod.rs","line_start":12,"is_primary":true}]}}
{"reason":"compiler-message","message":{"code":{"code":"clippy::needless_borrow"},"level":"warning","message":"this expression creates a reference which is immediately dereferenced","spans":[{"file_name":"cargo-lid-rs/src/catalog/mod.rs","line_start":30,"is_primary":true}]}}
{"reason":"compiler-message","message":{"code":null,"level":"error","message":"cannot find value `missing` in this scope","spans":[{"file_name":"cargo-lid-rs/src/catalog/mod.rs","line_start":44,"is_primary":true}]}}
{"reason":"build-finished","success":false}
"#;

    /// A tool with no JSON stream, printing as rustdoc does: two diagnostics —
    /// one carrying its lint in the tool's own hyphenated spelling, one no
    /// lint raised — among the progress and summary lines that are neither.
    const RUSTDOC_STDERR: &str = "\
 Documenting cargo-lid-rs v0.2.8 (/w/cargo-lid-rs)
warning: unresolved link to `Nowhere`
  --> cargo-lid-rs/src/catalog/mod.rs:12:5
   |
12 |     /// [`Nowhere`]
   |          ^^^^^^^^^ no item named `Nowhere` in scope
   |
   = note: `-D rustdoc::broken-intra-doc-links` implied by `-D warnings`

error: unable to read file `cargo-lid-rs/src/catalog/absent.md`
  --> cargo-lid-rs/src/catalog/mod.rs:20:9

error: aborting due to 1 previous error; 1 warning emitted
";

    /// The `sync` comparison's message, as `sync::check` joins its differences
    /// into one string this slice cannot take apart (Deferred 7).
    const SYNC_MESSAGE: &str =
        ".claude/skills/lid-rs/SKILL.md differs from the resolved lid-rs; .claude/agents/lid-rs-phase-5.md is absent";

    /// Pipeline §5.1's sixteen atomic commands, and `package`, which is this
    /// workspace's gate step and no §5.1 row.
    const EVERY_COMMAND: &[&str] = &[
        "check", "lint", "doc", "examples", "shape", "conform", "graph", "validate", "suite", "mutants", "regen",
        "site", "pr-body", "sync", "status", "commit", "package",
    ];

    /// The five this workspace builds, in the order a sorted comparison puts
    /// them; every other row of the catalog is unbuilt and carries a reason.
    const BUILT: &[&str] = &["check", "doc", "lint", "package", "sync"];

    /// What the design's table says each unbuilt entry waits on: whether its
    /// reason is a slice, and what that reason names.
    const REASONS: &[(&str, bool, &str)] = &[
        ("shape", true, "18"),
        ("conform", true, "19"),
        ("validate", true, "20"),
        ("site", true, "21"),
        ("examples", false, "Deferred 1"),
        ("suite", false, "Deferred 1"),
        ("graph", false, "Deferred 1"),
        ("regen", false, "Deferred 1"),
        ("mutants", false, "Deferred 2"),
        ("pr-body", false, "pipeline"),
        ("status", false, "pipeline"),
        ("commit", false, "pipeline"),
    ];

    /// What a finding read from a stderr is compared as: where it points, the
    /// check of the lint that raised it, that check's `fix`, and its grade.
    type Read = (Option<String>, Option<u32>, u32, Option<String>, String);

    /// Owned strings, as the entries and the workspace hold them.
    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|item| (*item).to_string()).collect()
    }

    /// The members `package` names, as `publishing_members()` answers them.
    fn members() -> Vec<String> {
        strings(&["cargo-lid-rs", "lid-rs", "lid-rs-macros"])
    }

    /// A fresh scratch directory, as this crate's other slices make one.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-catalog-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// A root where `target/lid` cannot be made, because `target` is a regular
    /// file: `create_dir_all` fails and the path is no directory afterwards.
    fn blocked_root(name: &str) -> PathBuf {
        let dir = scratch(name);
        std::fs::write(dir.join("target"), "a regular file, not a directory").expect("the blocking file");
        dir
    }

    /// The invocations a run hands its runner, and the report it answered
    /// with: the recording runner the injected `Runner` exists for.
    fn recorded(command: &str, root: &Path, publishing: &[String], output: ToolOutput) -> (Vec<Invocation>, Report) {
        let mut seen = Vec::new();
        let mut runner = |invocation: &Invocation| -> Result<ToolOutput, String> {
            seen.push(invocation.clone());
            Ok(output.clone())
        };
        let report = run_with(command, root, publishing, GATES, &mut runner);
        (seen, report)
    }

    /// A run whose tool could not be run at all — the fourth occasion of
    /// reaching no answer about the project.
    fn tool_refused(command: &str, root: &Path) -> Report {
        run_with(command, root, &members(), GATES, &mut |_: &Invocation| -> Result<ToolOutput, String> {
            Err("cargo could not be run".to_string())
        })
    }

    /// The entry the catalog holds for a name.
    fn entry_of(entries: &[Command], name: &str) -> Command {
        entries.iter().find(|entry| entry.name == name).cloned().unwrap_or_else(|| panic!("`{name}` is in the catalog"))
    }

    /// The reason an entry the catalog holds as unbuilt carries.
    fn reason_of(entries: &[Command], name: &str) -> Unbuilt {
        match entry_of(entries, name).invocation {
            Invocation::Unbuilt(reason) => reason,
            Invocation::JsonDiagnostics { .. } | Invocation::StderrDiagnostics { .. } | Invocation::SyncComparison => {
                panic!("`{name}` is unbuilt here")
            }
        }
    }

    /// What an unbuilt reason names, whichever of the three it is.
    fn named_by(reason: &Unbuilt) -> &'static str {
        match reason {
            Unbuilt::Slice(named) | Unbuilt::Question(named) | Unbuilt::Owner(named) => named,
        }
    }

    /// Whether a reason names a slice rather than a question.
    fn is_slice(reason: &Unbuilt) -> bool {
        matches!(reason, Unbuilt::Slice(_))
    }

    /// Whether this workspace builds an entry: an unbuilt one holds no
    /// invocation at all, rather than one nothing would run.
    fn is_built(entry: &Command) -> bool {
        !matches!(entry.invocation, Invocation::Unbuilt(_))
    }

    /// The environment an invocation carries, where it carries one.
    fn env_of(invocation: &Invocation) -> Vec<(String, String)> {
        match invocation {
            Invocation::StderrDiagnostics { env, .. } => env.clone(),
            Invocation::JsonDiagnostics { .. } | Invocation::SyncComparison | Invocation::Unbuilt(_) => Vec::new(),
        }
    }

    /// The report a run wrote for an entry, read back from its file.
    fn written_report(root: &Path, name: &str) -> Report {
        let path = root.join("target/lid").join(format!("{name}.json"));
        let text = std::fs::read_to_string(&path).unwrap_or_else(|why| panic!("{}: {why}", path.display()));
        serde_json::from_str(&text).expect("the report parses")
    }

    /// The message of the one finding a refusal carries.
    fn first_message(report: &Report) -> String {
        report.findings.first().expect("a refusal carries a finding").message.clone()
    }

    /// One diagnostic as a tool's reader would answer with.
    fn diagnostic(lint: Option<&str>, file: Option<&str>, line: Option<u32>) -> Diagnostic {
        Diagnostic {
            lint: lint.map(str::to_string),
            severity: "warning".to_string(),
            message: "the tool's sentence".to_string(),
            file: file.map(str::to_string),
            line,
        }
    }

    /// One finding, as `finding_of` answers with for a diagnostic that points
    /// at a place in the source.
    fn finding(check: u32, message: &str) -> Finding {
        Finding {
            check,
            rule: None,
            severity: "warning".to_string(),
            file: Some(SOURCE_FILE.to_string()),
            line: Some(42),
            item: None,
            claim: None,
            message: message.to_string(),
            fix: None,
            source: "lint".to_string(),
        }
    }

    #[test]
    #[validates(spec::ACommandRunsTheFixedInvocationItsEntryNames)]
    fn a_command_runs_the_fixed_invocation_its_entry_names() {
        let root = scratch("fixed-invocation");
        let publishing = members();
        let entries = table(&publishing);
        // Each built command hands the runner exactly one invocation, and it
        // is the one that command's entry names.
        let handed: Vec<Vec<Invocation>> =
            BUILT.iter().map(|name| recorded(name, &root, &publishing, ToolOutput::default()).0).collect();
        let named: Vec<Vec<Invocation>> =
            BUILT.iter().map(|name| vec![entry_of(&entries, name).invocation]).collect();
        assert_eq!(handed, named, "{BUILT:?}");
        // And what the entry names is fixed there: `run_with` takes no
        // argument a command line could widen `cargo check` with.
        let shown = format!("{:?}", handed[0]);
        let fixed = ["check", "--all-targets", "--message-format", "json"];
        assert!(fixed.iter().all(|token| shown.contains(token)), "{fixed:?} in {shown}");
    }

    #[test]
    #[validates(spec::ARunWritesItsReportEvenWhenItFindsNothing)]
    fn a_run_writes_its_report_even_when_it_finds_nothing() {
        let root = scratch("report-written");
        // A run that found nothing writes the empty list all the same, at the
        // file its entry is named for, so that an empty report and an absent
        // one stay different things — and an unbuilt entry is an entry too, a
        // pipeline reading its file being owed what it would find there.
        let nothing = recorded("check", &root, &members(), ToolOutput::default()).1;
        let unbuilt = recorded("shape", &root, &members(), ToolOutput::default()).1;
        assert!(nothing.findings.is_empty(), "{nothing:?}");
        assert_eq!([written_report(&root, "check"), written_report(&root, "shape")], [nothing, unbuilt]);
    }

    #[test]
    #[validates(spec::TheRenderingIsBuiltFromTheFindingsAndNotTheToolsOutput)]
    fn the_rendering_is_built_from_the_findings_and_not_the_tools_output() {
        let report = Report {
            command: "lint".to_string(),
            findings: vec![finding(7, "a leaf contains decisions nobody declared"), finding(0, "an unrecognised one")],
            status: Status::Findings,
        };
        let text = render(&report);
        // Every finding the report carries is in the rendering, with where it
        // points — the report is the only thing the text could be built from.
        let carried = ["lint", "a leaf contains decisions nobody declared", "an unrecognised one", SOURCE_FILE, "42"];
        assert!(carried.iter().all(|needle| text.contains(needle)), "{carried:?} in {text}");
        // And a report carrying no finding renders none of them.
        let empty = render(&Report { findings: Vec::new(), status: Status::Pass, ..report });
        assert!(!empty.contains("a leaf contains decisions nobody declared"), "{empty}");
    }

    #[test]
    #[validates(spec::TheStatusIsZeroWithNoFindingAndOneWithAFinding)]
    fn the_status_is_zero_with_no_finding_and_one_with_a_finding() {
        // One total function of the finding list: 0 where it is empty, 1 where
        // it holds a finding, whatever else was true of the run.
        let cases = [
            (Vec::new(), Status::Pass),
            (vec![finding(7, "one")], Status::Findings),
            (vec![finding(0, "one"), finding(3, "two")], Status::Findings),
        ];
        let mapped: Vec<Status> = cases.iter().map(|(findings, _)| report_of("lint", findings.clone()).status).collect();
        let expected: Vec<Status> = cases.iter().map(|(_, status)| *status).collect();
        assert_eq!(mapped, expected);
        // The numbers the claim names are the discriminants, stated once; and
        // a run answers with that mapping over what it found.
        let root = scratch("status-mapping");
        let run = recorded("check", &root, &members(), ToolOutput::default()).1;
        assert_eq!((Status::Pass as u8, Status::Findings as u8, run.status), (0, 1, Status::Pass));
    }

    #[test]
    #[validates(spec::ARunThatReachesNoAnswerIsTheToolingStatus)]
    fn a_run_that_reaches_no_answer_is_the_tooling_status() {
        let root = scratch("tooling-status");
        let blocked = blocked_root("tooling-status-blocked");
        // Every occasion where nothing produced a finding list to map: the
        // three the design names, and the fourth it judges against the rule.
        let reports = [
            ("a name the catalog holds no entry for", recorded("nonesuch", &root, &members(), ToolOutput::default()).1),
            ("an entry the catalog holds as unbuilt", recorded("shape", &root, &members(), ToolOutput::default()).1),
            ("a report directory that cannot be made", recorded("check", &blocked, &members(), ToolOutput::default()).1),
            ("a tool that could not be run", tool_refused("check", &root)),
        ];
        for (occasion, report) in reports {
            assert_eq!(report.status, Status::Tooling, "{occasion}");
        }
        assert_eq!(Status::Tooling as u8, 2, "a status no finding list produces");
    }

    #[test]
    #[validates(spec::TheCatalogNamesEveryCommandAndWhetherItIsBuilt)]
    fn the_catalog_names_every_command_and_whether_it_is_built() {
        let entries = table(&members());
        let mut named: Vec<&str> = entries.iter().map(|entry| entry.name).collect();
        let mut expected = EVERY_COMMAND.to_vec();
        named.sort_unstable();
        expected.sort_unstable();
        assert_eq!(named, expected, "§5.1's sixteen, and `package` beside them");
        // And each entry says whether this workspace builds it: an unbuilt one
        // holds no invocation at all, rather than one nothing would run.
        let mut built: Vec<&str> = entries.iter().filter(|entry| is_built(entry)).map(|entry| entry.name).collect();
        built.sort_unstable();
        assert_eq!(built, BUILT);
    }

    #[test]
    #[validates(spec::AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere)]
    fn an_unbuilt_entry_names_a_slice_a_question_or_an_owner_elsewhere() {
        let entries = table(&members());
        // Four wait on a slice; the rest wait on a question, because six of
        // §5.1's rows have no slice to name and a field typed `slice` would be
        // filled with a deferral's number.
        let observed: Vec<(&str, bool, bool)> = REASONS
            .iter()
            .map(|(name, _, named)| {
                let reason = reason_of(&entries, name);
                (*name, is_slice(&reason), named_by(&reason).contains(named))
            })
            .collect();
        let expected: Vec<(&str, bool, bool)> =
            REASONS.iter().map(|(name, slice, _)| (*name, *slice, true)).collect();
        assert_eq!(observed, expected, "each entry as (name, waits on a slice, names what the design says)");
    }

    #[test]
    #[validates(spec::AnUnbuiltCommandRefusesWithTheCatalogsReason)]
    fn an_unbuilt_command_refuses_with_the_catalogs_reason() {
        let root = scratch("unbuilt-refusal");
        let entries = table(&members());
        let names = ["shape", "mutants"];
        // The refusal is that entry's own reason, in the sentence composed in
        // one place for the printer to call rather than compose a second time;
        // and no tool ran for it, the refusal coming before any runner.
        let refused: Vec<(String, bool)> = names
            .iter()
            .map(|name| {
                let (seen, report) = recorded(name, &root, &members(), ToolOutput::default());
                (first_message(&report), seen.is_empty())
            })
            .collect();
        let expected: Vec<(String, bool)> =
            names.iter().map(|name| (unbuilt_sentence(name, &reason_of(&entries, name)), true)).collect();
        assert_eq!(refused, expected);
        assert!(refused.iter().zip(names).all(|((sentence, _), name)| sentence.contains(name)), "{refused:?}");
    }

    #[test]
    #[validates(spec::ANameTheCatalogHoldsNoEntryForIsRefusedByName)]
    fn a_name_the_catalog_holds_no_entry_for_is_refused_by_name() {
        let root = scratch("no-entry-refusal");
        let (seen, report) = recorded("nonesuch", &root, &members(), ToolOutput::default());
        // There is no entry to read a reason from, so the refusal names the
        // name it was given; nothing ran for it.
        let observed = (report.command.as_str(), first_message(&report).contains("nonesuch"), seen.is_empty());
        assert_eq!(observed, ("nonesuch", true, true), "{:?}", report.findings);
        // It is not the unbuilt refusal: that entry exists and names a reason.
        let unbuilt = recorded("shape", &root, &members(), ToolOutput::default()).1;
        assert_ne!(first_message(&report), first_message(&unbuilt));
    }

    #[test]
    #[validates(spec::ARefusalOfANameWithNoEntryWritesNoFile)]
    fn a_refusal_of_a_name_with_no_entry_writes_no_file() {
        let root = scratch("no-entry-writes-nothing");
        // The report file belongs to an entry, so a name with none writes no
        // file — which is what makes a traversing name unrepresentable rather
        // than sanitised — while the refusal still reaches the caller.
        let names = ["nonesuch", "../../escape"];
        let refused: Vec<(Status, usize)> = names
            .iter()
            .map(|name| {
                let report = recorded(name, &root, &members(), ToolOutput::default()).1;
                (report.status, report.findings.len())
            })
            .collect();
        assert_eq!(refused, [(Status::Tooling, 1), (Status::Tooling, 1)], "the refusal reaches the caller");
        assert_eq!(std::fs::read_dir(&root).expect("the root").count(), 0, "nothing was written under the root");
    }

    #[test]
    #[validates(spec::AFindingNamesTheFileAndLineOfItsDiagnostic)]
    fn a_finding_names_the_file_and_line_of_its_diagnostic() {
        // Where the diagnostic points at a place in the source, the finding
        // names it; where it points at none, the finding names none.
        let at = finding_of(diagnostic(None, Some(SOURCE_FILE), Some(42)), GATES, "check");
        assert_eq!((at.file.as_deref(), at.line), (Some(SOURCE_FILE), Some(42)));
        let nowhere = finding_of(diagnostic(None, None, None), GATES, "check");
        assert_eq!((nowhere.file, nowhere.line), (None, None));
        // What else the finding carries about the diagnostic comes from it too.
        let carried = (at.severity.as_str(), at.source.as_str(), at.message.as_str());
        assert_eq!(carried, ("warning", "check", "the tool's sentence"));
    }

    #[test]
    #[validates(spec::TheCheckNumberComesFromTheLintOrIsZero)]
    fn the_check_number_comes_from_the_lint_or_is_zero() {
        // The lint that raised it decides the check; a lint no check names and
        // a diagnostic no lint raised are alike carried under 0, never dropped.
        let cases = [
            (Some("clippy::cognitive_complexity"), 7),
            (Some("clippy::too_many_lines"), 9),
            (Some("clippy::missing_docs_in_private_items"), 3),
            (Some("rustdoc::broken_intra_doc_links"), 2),
            (Some("clippy::needless_borrow"), 0),
            (None, 0),
        ];
        for (lint, check) in cases {
            assert_eq!(finding_of(diagnostic(lint, None, None), GATES, "lint").check, check, "{lint:?}");
        }
    }

    #[test]
    #[validates(spec::TheFixLineIsTheGatesRowForTheFindingsCheck)]
    fn the_fix_line_is_the_gates_row_for_the_findings_check() {
        // The `fix` is that check's row in the table, and the row's correct
        // response rather than what the gate means.
        let seven = finding_of(diagnostic(Some("clippy::cognitive_complexity"), None, None), GATES, "lint");
        assert_eq!(seven.fix.as_deref(), Some(FIX_SEVEN));
        // A row keyed by two checks answers for either of them.
        let two = finding_of(diagnostic(Some("rustdoc::broken_intra_doc_links"), None, None), GATES, "doc");
        assert_eq!(two.fix.as_deref(), Some(FIX_TWO));
        // A check the table holds no row for carries none — check 9 has no row
        // above, and check 0 has none in the real table either.
        let nine = finding_of(diagnostic(Some("clippy::too_many_lines"), None, None), GATES, "lint");
        let zero = finding_of(diagnostic(None, None, None), GATES, "lint");
        assert_eq!((nine.fix, zero.fix), (None, None));
    }

    #[test]
    #[validates(spec::EveryDiagnosticOfTheStreamBecomesAFinding)]
    fn every_diagnostic_of_the_stream_becomes_a_finding() {
        let findings = findings_from_cargo(CARGO_STREAM, GATES, "lint");
        // Every compiler message the stream held reaches a finding — the lint
        // no check names among them — while the records that are no diagnostic
        // reach none.
        let observed: Vec<(u32, Option<u32>, &str)> =
            findings.iter().map(|f| (f.check, f.line, f.severity.as_str())).collect();
        let expected = [(7, Some(12), "warning"), (0, Some(30), "warning"), (0, Some(44), "error")];
        assert_eq!(observed, expected, "{findings:?}");
        let third = &findings[2];
        assert!(third.message.contains("cannot find value") && third.source == "lint", "{third:?}");
        // A stream holding no diagnostic answers with no finding.
        assert!(findings_from_cargo("", GATES, "lint").is_empty());
    }

    #[test]
    #[validates(spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr)]
    fn a_diagnostic_from_a_tool_with_no_json_stream_is_read_from_stderr() {
        let findings = findings_from_stderr(RUSTDOC_STDERR, GATES, "doc");
        // Each diagnostic the tool printed is carried; the progress line above
        // them and the summary line below them are no diagnostics.
        //
        // And a finding of this provenance carries what one of the other does:
        // the place it points at, the check of the lint that raised it — in the
        // spelling the lint is declared with, not the tool's — and that row.
        let observed: Vec<Read> =
            findings.iter().map(|f| (f.file.clone(), f.line, f.check, f.fix.clone(), f.severity.clone())).collect();
        let expected: Vec<Read> = vec![
            (Some(SOURCE_FILE.to_string()), Some(12), 2, Some(FIX_TWO.to_string()), "warning".to_string()),
            (Some(SOURCE_FILE.to_string()), Some(20), 0, None, "error".to_string()),
        ];
        assert_eq!(observed, expected, "{findings:?}");
        assert!(findings_from_stderr("", GATES, "doc").is_empty());
    }

    #[test]
    #[validates(spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout)]
    fn the_sync_findings_are_one_message_with_differences_and_none_without() {
        let root = scratch("sync-findings");
        let differences = ToolOutput { stdout: String::new(), stderr: SYNC_MESSAGE.to_string() };
        // The comparison's message is carried whole, as one finding: this
        // slice cannot take it apart, and will not re-parse it (Deferred 7).
        // A comparison that reported no difference yields none, so that a
        // clean workspace's `sync` step does not fail forever on a passing run.
        let reported = recorded("sync", &root, &members(), differences).1;
        let clean = recorded("sync", &root, &members(), ToolOutput::default()).1;
        let carried: Vec<(&str, &str)> =
            reported.findings.iter().map(|f| (f.message.as_str(), f.source.as_str())).collect();
        assert_eq!(carried, [(SYNC_MESSAGE, "sync")], "{:?}", reported.findings);
        assert_eq!((clean.findings.len(), clean.status), (0, Status::Pass), "{:?}", clean.findings);
    }

    #[test]
    #[validates(spec::PackageNamesEveryPublishingMemberInOneInvocation)]
    fn package_names_every_publishing_member_in_one_invocation() {
        let root = scratch("package-invocation");
        let (seen, _) = recorded("package", &root, &members(), ToolOutput::default());
        // One invocation naming them all: the per-crate form cannot resolve a
        // sibling at a version no registry holds.
        let shown = format!("{seen:?}");
        assert!(seen.len() == 1 && members().iter().all(|member| shown.contains(member)), "{shown}");
        // The members are the workspace's answer, not the table's own list.
        let (other, _) = recorded("package", &root, &strings(&["only-me"]), ToolOutput::default());
        let narrowed = format!("{other:?}");
        assert!(narrowed.contains("only-me") && !narrowed.contains("lid-rs-macros"), "{narrowed}");
    }

    #[test]
    #[validates(spec::TheDocInvocationNamesTheFlagThatDocumentsPrivateItems)]
    fn the_doc_invocation_names_the_flag_that_documents_private_items() {
        let root = scratch("doc-private-items");
        let (seen, _) = recorded("doc", &root, &members(), ToolOutput::default());
        // Without the flag a private item's documentation is never read, and
        // check 3 and rustdoc stop agreeing about what is documented.
        let shown = format!("{seen:?}");
        for token in ["doc", "--no-deps", "--document-private-items"] {
            assert!(shown.contains(token), "{token} in {shown}");
        }
    }

    #[test]
    #[validates(spec::TheDocInvocationCarriesTheEnvironmentThatDeniesABrokenLink)]
    fn the_doc_invocation_carries_the_environment_that_denies_a_broken_link() {
        let root = scratch("doc-environment");
        let (seen, _) = recorded("doc", &root, &members(), ToolOutput::default());
        let env = env_of(seen.first().expect("the doc invocation"));
        // Without it rustdoc answers success over a link it could not resolve,
        // and the step gates nothing.
        let denied = env.iter().find(|(name, _)| name == "RUSTDOCFLAGS").map(|(_, value)| value.clone());
        assert_eq!(denied.as_deref(), Some("-D rustdoc::broken_intra_doc_links"), "{env:?}");
    }

    #[test]
    #[validates(spec::TheReportDirectoryIsCreatedOnDemand)]
    fn the_report_directory_is_created_on_demand() {
        let root = scratch("report-directory");
        let report = Report { command: "check".to_string(), findings: Vec::new(), status: Status::Pass };
        // The workspace holds no `target/lid` before the run and holds one
        // after: it is made on demand rather than expected to be there.
        assert!(!root.join("target/lid").exists());
        write_report(&root, "check", &report).expect("the directory is made on demand");
        assert!(root.join("target/lid").is_dir());
        assert_eq!(written_report(&root, "check"), report);
        // And a second run writes into the directory that now exists.
        write_report(&root, "check", &report).expect("the directory is already there");
    }

    #[test]
    #[validates(spec::ADirectoryThatCannotBeCreatedIsNamedByAFinding)]
    fn a_directory_that_cannot_be_created_is_named_by_a_finding() {
        // `target` is a regular file, so `target/lid` cannot be made: the run
        // is no answer about the project rather than a silent skip.
        let root = blocked_root("blocked-report-directory");
        let report = recorded("check", &root, &members(), ToolOutput::default()).1;
        let named = root.join("target/lid");
        assert_eq!(report.findings.len(), 1, "what such a run found is not the answer: {:?}", report.findings);
        assert!(first_message(&report).contains(&named.display().to_string()), "{}", first_message(&report));
        assert!(!named.exists(), "the write really was refused");
    }

    #[test]
    #[validates(spec::TheReportCarriesTheRunsStatus)]
    fn the_report_carries_the_runs_status() {
        let root = scratch("report-status");
        // The status a run reached is in the report its caller is answered
        // with and in the file a reader has, since `main` maps every error to
        // exit 1 and the process's code cannot hold it (Deferred 4).
        let clean = recorded("check", &root, &members(), ToolOutput::default()).1;
        assert_eq!((clean.status, written_report(&root, "check").status), (Status::Pass, Status::Pass));
        let unbuilt = recorded("shape", &root, &members(), ToolOutput::default()).1;
        assert_eq!((unbuilt.status, written_report(&root, "shape").status), (Status::Tooling, Status::Tooling));
    }
}
