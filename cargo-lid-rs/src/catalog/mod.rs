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

/// Why an entry is not built: one of the two things a reason can be, so that
/// a command with no slice to name is not made to name one.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::AnUnbuiltEntryNamesTheSliceOrTheQuestionItWaitsOn)]
pub enum Unbuilt {
    /// The slice the command lands with, named as the design names it.
    Slice(&'static str),
    /// The question the command waits on, named as the design names it.
    Question(&'static str),
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
    spec::AnUnbuiltEntryNamesTheSliceOrTheQuestionItWaitsOn,
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
/// Resolves `command` against [`table`] — a name with no entry and an entry
/// that is [`Invocation::Unbuilt`] are two different refusals — hands the
/// runner the invocation the entry names, chooses which of the [`ToolOutput`]
/// streams the entry's provenance makes its findings, maps that stream against
/// `gates`, writes `target/lid/<entry name>.json` under `root`, and answers with
/// the [`Report`], which carries the [`Status`] (Deferred 4) rather than the
/// caller reading a status alone.
///
/// `gates` is the text of the synced skill's `references/gates.md`, read by
/// [`run`] and passed in whole: a validation supplies the rows it wants to
/// assert about as a string, rather than materialising a skill tree under
/// `root` to test a status mapping.
#[implements(
    spec::ACommandRunsTheFixedInvocationItsEntryNames,
    spec::ARunWritesItsReportEvenWhenItFindsNothing,
    spec::ARefusalOfANameWithNoEntryWritesNoFile,
    spec::TheStatusIsZeroWithNoFindingAndOneWithAFinding,
    spec::ARunThatReachesNoAnswerIsTheToolingStatus,
    spec::AnUnbuiltCommandRefusesWithTheCatalogsReason,
    spec::ANameTheCatalogHoldsNoEntryForIsRefusedByName,
    spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout,
    spec::EveryDiagnosticOfTheStreamBecomesAFinding,
    spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr,
    spec::TheReportDirectoryIsCreatedOnDemand,
    spec::ADirectoryThatCannotBeCreatedIsNamedByAFinding,
    spec::TheReportCarriesTheRunsStatus,
)]
pub fn run_with(command: &str, root: &Path, publishing: &[String], gates: &str, runner: &mut impl Runner) -> Report {
    match table(publishing).into_iter().find(|entry| entry.name == command) {
        None => todo!(
            "refuse `{command}`, a name the catalog holds no entry for, reaching no answer: answer the caller with the \
             report and write no file, since there is no entry whose report it would be and no entry to take a file \
             name from"
        ),
        Some(Command { invocation: Invocation::Unbuilt(reason), .. }) => {
            todo!("refuse with {reason:?}, the catalog's own, reaching no answer, and write the entry's report")
        }
        Some(entry) => todo!(
            "map {:?} to a report under {}, against {} bytes of gate rows",
            runner.run(&entry.invocation),
            root.display(),
            gates.len()
        ),
    }
}

/// Cargo's JSON diagnostic stream to findings: the check number from the lint
/// that raised each diagnostic, the `fix` from the `gates.md` row for that
/// check, and `check: 0` for a diagnostic no check names, carried rather than
/// dropped.
///
/// `gates` is the text of the synced skill's `references/gates.md`, whose rows
/// are keyed by check; `source` is the command the findings are recorded
/// against.
#[implements(
    spec::EveryDiagnosticOfTheStreamBecomesAFinding,
    spec::AFindingNamesTheFileAndLineOfItsDiagnostic,
    spec::TheCheckNumberComesFromTheLintOrIsZero,
    spec::TheFixLineIsTheGatesRowForTheFindingsCheck,
)]
pub fn findings_from_cargo(stream: &str, gates: &str, source: &str) -> Vec<Finding> {
    todo!("map {} bytes of diagnostics for `{source}` against {} bytes of gate rows", stream.len(), gates.len())
}

/// The same for the tools that emit no JSON stream — rustdoc and `cargo
/// package` — whose diagnostics are read from what they printed to stderr.
///
/// `gates` and `source` are [`findings_from_cargo`]'s.
#[implements(spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr, spec::TheFixLineIsTheGatesRowForTheFindingsCheck)]
pub fn findings_from_stderr(stderr: &str, gates: &str, source: &str) -> Vec<Finding> {
    todo!("map {} bytes of stderr for `{source}` against {} bytes of gate rows", stderr.len(), gates.len())
}

/// A report to the human rendering, built from the findings alone and never
/// from the output of the tool the command ran, so that the two cannot
/// disagree.
#[implements(spec::TheRenderingIsBuiltFromTheFindingsAndNotTheToolsOutput)]
pub fn render(report: &Report) -> String {
    todo!("render {}'s findings", report.command)
}
