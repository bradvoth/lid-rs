#![doc = include_str!("lld.md")]

pub mod spec;

use std::path::Path;

use lid_rs::implements;
use serde::{Deserialize, Serialize};

use crate::project::Project;

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

/// Every command this workspace does not build, with the reason its entry
/// carries — the twelve rows of the document's table that are not built here,
/// each naming the slice it lands with, the question it waits on, or the thing
/// that owns it where this tool builds it at no point.
///
/// Data rather than a dispatch: which of the three a row names is the table's
/// own answer, so a row that named the wrong one is wrong here and nowhere
/// else.
const UNBUILT: &[(&str, Unbuilt)] = &[
    ("shape", Unbuilt::Slice("slice 18")),
    ("conform", Unbuilt::Slice("slice 19")),
    ("validate", Unbuilt::Slice("slice 20")),
    ("site", Unbuilt::Slice("slice 21")),
    ("examples", Unbuilt::Question("Deferred 1")),
    ("suite", Unbuilt::Question("Deferred 1")),
    ("graph", Unbuilt::Question("Deferred 1")),
    ("regen", Unbuilt::Question("Deferred 1")),
    ("mutants", Unbuilt::Question("Deferred 2")),
    ("pr-body", Unbuilt::Owner("the pipeline")),
    ("status", Unbuilt::Owner("the pipeline")),
    ("commit", Unbuilt::Owner("the pipeline")),
];

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
    let members = publishing.iter().flat_map(|member| ["-p".to_string(), member.clone()]);
    let built = [
        ("check", Invocation::JsonDiagnostics {
            args: ["check", "--all-targets", "--message-format", "json"].map(String::from).to_vec(),
        }),
        ("lint", Invocation::JsonDiagnostics {
            args: ["clippy", "--all-targets", "--message-format", "json", "--", "-D", "warnings"].map(String::from).to_vec(),
        }),
        ("doc", Invocation::StderrDiagnostics {
            args: ["doc", "--no-deps", "--document-private-items"].map(String::from).to_vec(),
            env: vec![("RUSTDOCFLAGS".to_string(), "-D rustdoc::broken_intra_doc_links".to_string())],
        }),
        ("package", Invocation::StderrDiagnostics {
            args: std::iter::once("package".to_string()).chain(members).collect(),
            env: Vec::new(),
        }),
        ("sync", Invocation::SyncComparison),
    ];
    built
        .into_iter()
        .map(|(name, invocation)| Command { name, invocation })
        .chain(UNBUILT.iter().map(|(name, reason)| Command { name, invocation: Invocation::Unbuilt(reason.clone()) }))
        .collect()
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
    let project = Project::load_graph()?;
    let root = project.root()?;
    let table = root.join(crate::sync::SKILL_IN_PROJECT).join("references/gates.md");
    let gates = std::fs::read_to_string(&table).map_err(|why| format!("reading {}: {why}", table.display()))?;
    let publishing = project.publishing_members();
    Ok(run_with(command, &root, &publishing, &gates, &mut |invocation: &Invocation| {
        spawned(&project, command, invocation)
    }))
}

/// One [`Invocation`] spawned: the tool's two streams, or the reason it could
/// not be run at all.
///
/// The spawning half's dispatch, over the four things an invocation can be.
///
/// Two of its arms spawn nothing, and one of those two decides something a
/// claim states. The `sync` comparison is this binary's own work rather than a
/// tool's, so nothing about it is a stream until this item says which one it
/// is: the message goes where a tool with no JSON stream puts its diagnostics,
/// because that is the stream `sync`'s provenance reads. An item answering on
/// stdout would leave a comparison that reported differences with no finding at
/// all, which is
/// [`TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout`](spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout)
/// made false — so that claim is cited here, and observed without spawning
/// anything.
///
/// The two cargo arms carry no claim: what they answer is a real process's
/// output, and a validation of either would be a build.
///
/// An entry with no invocation reaches this at no point — [`run_with`] refuses
/// it before any runner is called — so the arm cites nothing: no answer it
/// could give makes any claim false. It answers the one sentence
/// [`unbuilt_sentence`] composes rather than a second one written here, which
/// is the "six places state a gate" rule and not a claim about a run.
#[implements(spec::TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout)]
fn spawned(project: &Project, command: &str, invocation: &Invocation) -> Result<ToolOutput, String> {
    match invocation {
        Invocation::JsonDiagnostics { args } => cargo_output(project, args, &[]),
        Invocation::StderrDiagnostics { args, env } => cargo_output(project, args, env),
        Invocation::SyncComparison => {
            Ok(ToolOutput { stdout: String::new(), stderr: crate::sync::check(project).err().unwrap_or_default() })
        }
        Invocation::Unbuilt(reason) => Err(unbuilt_sentence(command, reason)),
    }
}

/// One cargo invocation run to completion, both its streams captured whole.
///
/// A tool that exits non-zero is no failure here: a check that found something
/// is the answer the findings carry, and only a cargo that could not be run at
/// all is a run that reached no answer.
fn cargo_output(project: &Project, args: &[String], env: &[(String, String)]) -> Result<ToolOutput, String> {
    let output = project
        .cargo()?
        .args(args)
        .envs(env.iter().cloned())
        .output()
        .map_err(|why| format!("running cargo {}: {why}", args.join(" ")))?;
    Ok(ToolOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
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
    let carried = Finding {
        check: 0,
        rule: None,
        severity: "error".to_string(),
        file: None,
        line: None,
        item: None,
        claim: None,
        message: message.to_string(),
        fix: None,
        source: source.to_string(),
    };
    (!message.trim().is_empty()).then_some(carried).into_iter().collect()
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
    let status = if findings.is_empty() { Status::Pass } else { Status::Findings };
    Report { command: command.to_string(), findings, status }
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
    Report { command: command.to_string(), findings: vec![tooling_finding(command, why)], status: Status::Tooling }
}

/// The finding a run that reached no answer carries: the sentence it refused
/// with, against no check, since nothing about the project was read.
fn tooling_finding(source: &str, why: String) -> Finding {
    Finding {
        check: 0,
        rule: None,
        severity: "error".to_string(),
        file: None,
        line: None,
        item: None,
        claim: None,
        message: why,
        fix: None,
        source: source.to_string(),
    }
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
    let directory = root.join("target").join("lid");
    std::fs::create_dir_all(&directory).map_err(|why| format!("creating {}: {why}", directory.display()))?;
    let path = directory.join(format!("{name}.json"));
    let json = serde_json::to_string_pretty(report).map_err(|why| format!("rendering {}: {why}", path.display()))?;
    std::fs::write(&path, json).map_err(|why| format!("writing {}: {why}", path.display()))
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
    match reason {
        Unbuilt::Slice(slice) => format!("`{name}` is not built here: it lands with {slice}"),
        Unbuilt::Question(question) => format!("`{name}` is not built here: it waits on {question}"),
        Unbuilt::Owner(owner) => format!("`{name}` is not built here: it belongs to {owner}, which this tool is not"),
    }
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
    stream
        .lines()
        .filter_map(|record| serde_json::from_str::<serde_json::Value>(record).ok())
        .filter(|record| record["reason"] == "compiler-message")
        .map(|record| {
            let message = &record["message"];
            let at = message["spans"].as_array().into_iter().flatten().find(|span| span["is_primary"] == true);
            Diagnostic {
                lint: message["code"]["code"].as_str().map(str::to_string),
                severity: message["level"].as_str().unwrap_or_default().to_string(),
                message: message["message"].as_str().unwrap_or_default().to_string(),
                file: at.and_then(|span| span["file_name"].as_str()).map(str::to_string),
                line: at.and_then(|span| span["line_start"].as_u64()).and_then(|line| u32::try_from(line).ok()),
            }
        })
        .collect()
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
    let lines: Vec<&str> = stderr.lines().collect();
    let starts: Vec<usize> =
        lines.iter().enumerate().filter(|(_, line)| starts_a_diagnostic(line)).map(|(at, _)| at).collect();
    starts
        .iter()
        .enumerate()
        .map(|(nth, start)| diagnostic_of(&lines[*start..starts.get(nth + 1).copied().unwrap_or(lines.len())]))
        .collect()
}

/// The grades a tool prints a diagnostic under, at the head of the line the
/// diagnostic starts on and before the sentence it says.
const GRADES: &[&str] = &["warning", "error"];

/// What a tool says at the end about what it already said: a count of the
/// diagnostics above it rather than a diagnostic of its own. A reader that
/// took these for diagnostics would report a finding for every run that found
/// anything, naming no place and saying nothing a reader could act on.
const SUMMARIES: &[&str] = &["aborting due to", "could not compile", "build failed"];

/// Whether a line starts a diagnostic: a tool's own grade at its head, then
/// the sentence — and not one of the lines a tool counts with at the end.
///
/// A grade is matched at the head of the line and not anywhere in it, which is
/// what leaves a tool's progress lines, its `= note:` lines and the indented
/// body of a `Caused by:` block out: each either carries no grade or carries it
/// behind something else.
fn starts_a_diagnostic(line: &str) -> bool {
    line.split_once(": ")
        .is_some_and(|(grade, said)| GRADES.contains(&grade) && !SUMMARIES.iter().any(|end| said.starts_with(end)))
}

/// One block of a tool's stderr — the line a diagnostic starts on and every
/// line under it up to the next diagnostic — read as one [`Diagnostic`].
///
/// The place it points at is the `-->` line tools underline a span with, and
/// the lint is whichever line of the block names one; a block with neither
/// points at nothing and names no lint, which is what a `cargo package`
/// failure looks like.
fn diagnostic_of(block: &[&str]) -> Diagnostic {
    let said = block.first().copied().unwrap_or_default();
    let (severity, message) = said.split_once(": ").unwrap_or(("", said));
    let at: Vec<&str> = block
        .iter()
        .find_map(|line| line.trim_start().strip_prefix("--> "))
        .unwrap_or_default()
        .split(':')
        .collect();
    Diagnostic {
        lint: block.iter().copied().find_map(lint_of),
        severity: severity.to_string(),
        message: message.to_string(),
        file: at.first().filter(|name| !name.is_empty()).map(|name| (*name).to_string()),
        line: at.get(1).and_then(|line| line.parse().ok()),
    }
}

/// The lint a line of a diagnostic's block names, in the spelling that lint is
/// declared by.
///
/// A tool names the lint it raised a diagnostic under in the note that says
/// which flag implied it — `` `-D rustdoc::broken-intra-doc-links` implied by
/// `-D warnings` `` — and spells it with the hyphens a command line takes
/// rather than the underscores the lint is declared with. The declared
/// spelling is what is answered, so that which check a lint belongs to is
/// decided once for both streams.
fn lint_of(line: &str) -> Option<String> {
    line.split('`').nth(1)?.strip_prefix("-D ").map(|lint| lint.replace('-', "_"))
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
    match lint.unwrap_or_default() {
        "rustdoc::broken_intra_doc_links" => 2,
        "missing_docs" | "clippy::missing_docs_in_private_items" => 3,
        "clippy::wildcard_enum_match_arm" => 6,
        "clippy::cognitive_complexity" => 7,
        "clippy::fn_params_excessive_bools" => 8,
        "clippy::too_many_lines" => 9,
        _ => 0,
    }
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
    gates
        .lines()
        .map(|row| row.split('|').map(str::trim).collect::<Vec<&str>>())
        .find(|cells| {
            cells
                .get(1)
                .and_then(|gate| gate.strip_prefix("check "))
                .and_then(|keyed| keyed.split_whitespace().next())
                .is_some_and(|keys| keys.split('/').any(|key| key.parse::<u32>().is_ok_and(|one| one == check)))
        })
        .and_then(|cells| cells.get(3).map(|response| (*response).to_string()))
}

/// A report to the human rendering, built from the findings alone and never
/// from the output of the tool the command ran, so that the two cannot
/// disagree.
#[implements(spec::TheRenderingIsBuiltFromTheFindingsAndNotTheToolsOutput)]
pub fn render(report: &Report) -> String {
    let found: Vec<String> = report
        .findings
        .iter()
        .map(|finding| {
            format!(
                "  {}:{} — check {} — {}\n    fix: {}",
                finding.file.as_deref().unwrap_or("(no place in the source)"),
                finding.line.map_or_else(String::new, |line| line.to_string()),
                finding.check,
                finding.message,
                finding.fix.as_deref().unwrap_or("(this check has no row in the skill's gates table)"),
            )
        })
        .collect();
    format!("{}: {} finding(s)\n{}", report.command, report.findings.len(), found.join("\n"))
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
    //! Five fixtures stand in for what a run would otherwise need on disk:
    //!
    //! | Fixture | Stands for |
    //! |---|---|
    //! | `GATES` | three rows of the synced skill's `references/gates.md`, passed as the text `run_with` takes, so that no test materialises a skill tree to assert a `fix` line |
    //! | `CARGO_STREAM` | cargo's JSON diagnostic stream, holding the records that are no diagnostic among the three that are |
    //! | `RUSTDOC_STDERR` | the other provenance: a tool that prints its diagnostics to stderr, in its own spelling of a lint's name, among its progress and summary lines |
    //! | `PACKAGE_STDERR` | the other *tool* of that provenance: `cargo package`, whose diagnostics point at no place in the source and carry no `-->` line at all, among the lines it prints while it works |
    //! | `blocked_root` | a workspace root whose `target` is a **regular file**, so `create_dir_all` fails on macOS and Linux alike and `path.is_dir()` is false afterwards — a read-only parent is the wrong mechanism, since it silently succeeds for a test run as root |
    //!
    //! `PACKAGE_STDERR` is there because one item reads the stderr of both
    //! tools. A reader keyed to what rustdoc prints — a `warning:` or `error:`
    //! line with a `  --> file:line:col` under it — satisfies `RUSTDOC_STDERR`
    //! exactly and answers nothing at all over a packaging that failed, which is
    //! the `package` step reporting a pass over every reason it could not
    //! publish. What the fixture pins is which of its lines are diagnostics and
    //! what a diagnostic that points at no source carries; whether the `Caused
    //! by:` block below a failure is folded into that diagnostic's message is
    //! not pinned, because the design settles which lines are diagnostics and
    //! does not settle that.
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
    //!
    //! **Why no expectation is read back from the item under test.** The value
    //! a test compares against is the document read into a fixture — the tokens
    //! of each built command's invocation in `fixed`, which of the three things
    //! each unbuilt entry's reason is in `REASONS`, what each stream's
    //! diagnostics become — and never a second call into the code that produced
    //! the observation. An expectation built by calling `table` or
    //! `unbuilt_sentence` holds for every implementation of them, including one
    //! that is wrong consistently, and such a test records only that the code
    //! agrees with itself. Where one item's answer *is* compared with another's
    //! — a refusal's sentence with `unbuilt_sentence`'s — what is asserted is
    //! that the two agree, which is a property of its own, and what either of
    //! them says is pinned against a fixture beside it.
    //!
    //! **Why each provenance is also observed through a run.** Which of a
    //! tool's two streams an entry's findings are read from is `findings_of`'s
    //! dispatch, and a test that calls `findings_from_cargo` or
    //! `findings_from_stderr` itself never reaches it: with those two arms
    //! exchanged, every such test still passes while `check` and `doc` report a
    //! pass over every diagnostic their tools emitted. So each provenance is
    //! observed twice — once on the reader, and once on a run whose
    //! `ToolOutput` carries *both* streams filled, where reading the wrong one
    //! answers the other stream's diagnostics.

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

    /// The other tool of that provenance: `cargo package`, which prints a
    /// diagnostic that points at no place in the source — no `-->` line follows
    /// either of these — among the lines it prints while it works and the block
    /// that details the failure above it.
    const PACKAGE_STDERR: &str = "\
    Packaging cargo-lid-rs v0.2.8 (/w/cargo-lid-rs)
warning: manifest has no documentation, homepage or repository.
See https://doc.rust-lang.org/cargo/reference/manifest.html#package-metadata for more info.
    Updating crates.io index
   Verifying cargo-lid-rs v0.2.8 (/w/cargo-lid-rs)
error: failed to verify package tarball

Caused by:
  no matching package named `lid-rs` found
  location searched: registry `crates.io`
  required by package `cargo-lid-rs v0.2.8 (/w/target/package/cargo-lid-rs-0.2.8)`
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

    /// What the design's table says each unbuilt entry carries: which of the
    /// three things its reason is, and what that reason names.
    ///
    /// Which of the three, and not whether it is a slice: a reason is a slice,
    /// a question or an owner elsewhere, and a boolean leaves the last two
    /// alike — so `pr-body` carrying `Question("the pipeline")` would answer
    /// every question this fixture could ask about it.
    const REASONS: &[(&str, &str, &str)] = &[
        ("shape", SLICE, "18"),
        ("conform", SLICE, "19"),
        ("validate", SLICE, "20"),
        ("site", SLICE, "21"),
        ("examples", QUESTION, "Deferred 1"),
        ("suite", QUESTION, "Deferred 1"),
        ("graph", QUESTION, "Deferred 1"),
        ("regen", QUESTION, "Deferred 1"),
        ("mutants", QUESTION, "Deferred 2"),
        ("pr-body", OWNER, "pipeline"),
        ("status", OWNER, "pipeline"),
        ("commit", OWNER, "pipeline"),
    ];

    /// A reason that names the slice its command lands with.
    const SLICE: &str = "slice";

    /// A reason that names the question its command waits on.
    const QUESTION: &str = "question";

    /// A reason that names what owns its command where this tool builds it at
    /// no point.
    const OWNER: &str = "owner";

    /// What a finding read from a stderr is compared as: where it points, the
    /// check of the lint that raised it, that check's `fix`, and its grade.
    type Read = (Option<String>, Option<u32>, u32, Option<String>, String);

    /// What a finding read from cargo's JSON stream is compared as: the check
    /// of the lint that raised it, where it points, its grade, what it says,
    /// and the command it was recorded against.
    type Mapped = (u32, Option<u32>, String, String, String);

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

    /// Which of the three things an unbuilt reason is, as a word a test
    /// compares: a slice, a question, or what owns the command elsewhere.
    fn case_of(reason: &Unbuilt) -> &'static str {
        match reason {
            Unbuilt::Slice(_) => SLICE,
            Unbuilt::Question(_) => QUESTION,
            Unbuilt::Owner(_) => OWNER,
        }
    }

    /// Whether this workspace builds an entry: an unbuilt one holds no
    /// invocation at all, rather than one nothing would run.
    fn is_built(entry: &Command) -> bool {
        !matches!(entry.invocation, Invocation::Unbuilt(_))
    }

    /// Where an invocation's findings come from, as a word a test compares: an
    /// entry's provenance is half of what its fixed invocation is, and a run
    /// that named the other one would read the wrong stream.
    fn provenance_of(invocation: &Invocation) -> &'static str {
        match invocation {
            Invocation::JsonDiagnostics { .. } => "cargo's JSON stream",
            Invocation::StderrDiagnostics { .. } => "the tool's stderr",
            Invocation::SyncComparison => "the sync comparison",
            Invocation::Unbuilt(_) => "nothing to run",
        }
    }

    /// The arguments an invocation names, whole and in order, where it names
    /// any — compared as arguments rather than searched for as text, since
    /// `doc` is a substring of `--document-private-items` and `lid-rs` of
    /// `cargo-lid-rs`, and a search for either holds where no such argument is.
    fn args_of(invocation: &Invocation) -> Vec<String> {
        match invocation {
            Invocation::JsonDiagnostics { args } | Invocation::StderrDiagnostics { args, .. } => args.clone(),
            Invocation::SyncComparison | Invocation::Unbuilt(_) => Vec::new(),
        }
    }

    /// The fixed invocation the document's table names for each built command:
    /// where that command's findings come from, and the arguments its entry
    /// enumerates. Read from the document into a fixture, so that it is not
    /// `table` compared with itself.
    ///
    /// `package`'s arguments name every publishing member, which is the one
    /// thing here that comes from the workspace rather than from the table.
    fn fixed(publishing: &[String]) -> Vec<(&'static str, &'static str, Vec<String>)> {
        let named: Vec<String> = publishing.iter().flat_map(|member| ["-p".to_string(), member.clone()]).collect();
        vec![
            ("check", "cargo's JSON stream", strings(&["check", "--all-targets", "--message-format", "json"])),
            ("doc", "the tool's stderr", strings(&["doc", "--no-deps", "--document-private-items"])),
            (
                "lint",
                "cargo's JSON stream",
                strings(&["clippy", "--all-targets", "--message-format", "json", "--", "-D", "warnings"]),
            ),
            ("package", "the tool's stderr", [strings(&["package"]), named].concat()),
            ("sync", "the sync comparison", Vec::new()),
        ]
    }

    /// The fixed invocation that fixture names for one built command, which
    /// every command `BUILT` holds must have a row for.
    fn fixed_for(name: &str, publishing: &[String]) -> (&'static str, Vec<String>) {
        fixed(publishing)
            .into_iter()
            .find(|(command, _, _)| *command == name)
            .map(|(_, provenance, args)| (provenance, args))
            .unwrap_or_else(|| panic!("the document names `{name}`'s fixed invocation"))
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

    /// Those five of every finding a stderr was read into, whichever item
    /// answered them — the reader itself, or a run that chose the stream.
    fn read_of(findings: &[Finding]) -> Vec<Read> {
        findings.iter().map(|f| (f.file.clone(), f.line, f.check, f.fix.clone(), f.severity.clone())).collect()
    }

    /// Those five of every finding cargo's stream was mapped into, whichever
    /// item answered them.
    fn mapped_of(findings: &[Finding]) -> Vec<Mapped> {
        findings.iter().map(|f| (f.check, f.line, f.severity.clone(), f.message.clone(), f.source.clone())).collect()
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
        // Each built command hands the runner one invocation, and it is the
        // fixed one its entry names: the provenance the document gives that
        // command, and the arguments the command itself enumerates, whole and
        // in order. Nothing widens them — there is no parameter a command line
        // could reach — and the expectation is the document rather than `table`
        // read back, so a row that names the wrong flags is wrong here.
        let observed: Vec<(&str, usize, &str, Vec<String>)> = BUILT
            .iter()
            .map(|name| {
                let handed = recorded(name, &root, &publishing, ToolOutput::default()).0;
                let one = handed.first();
                let provenance = one.map_or("nothing was handed", provenance_of);
                (*name, handed.len(), provenance, one.map_or_else(Vec::new, args_of))
            })
            .collect();
        let expected: Vec<(&str, usize, &str, Vec<String>)> = BUILT
            .iter()
            .map(|name| {
                let (provenance, args) = fixed_for(name, &publishing);
                (*name, 1, provenance, args)
            })
            .collect();
        assert_eq!(observed, expected, "the one fixed invocation each built command names");
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
        let root = scratch("rendering");
        let answered = Finding {
            fix: Some(FIX_SEVEN.to_string()),
            ..finding(7, "a leaf contains decisions nobody declared")
        };
        let report = Report {
            command: "lint".to_string(),
            findings: vec![answered, finding(0, "an unrecognised one")],
            status: Status::Findings,
        };
        let text = render(&report);
        // Every finding the report carries is in the rendering, with where it
        // points and the correct response this project states for its check —
        // which no tool prints, so a rendering carrying it was built from the
        // findings and not from what the tool said.
        let carried =
            ["lint", "a leaf contains decisions nobody declared", "an unrecognised one", SOURCE_FILE, "42", FIX_SEVEN];
        assert!(carried.iter().all(|needle| text.contains(needle)), "{carried:?} in {text}");
        // A report carrying no finding renders none of them.
        let empty = render(&Report { findings: Vec::new(), status: Status::Pass, ..report });
        assert!(!empty.contains("a leaf contains decisions nobody declared"), "{empty}");
        // And nothing a real run's tool printed reaches the rendering of that
        // run's report: the two cannot disagree, because only one is read.
        let both = ToolOutput { stdout: CARGO_STREAM.to_string(), stderr: RUSTDOC_STDERR.to_string() };
        let rendered = render(&recorded("check", &root, &members(), both).1);
        let printed =
            ["compiler-artifact", "build-finished", "is_primary", "Documenting cargo-lid-rs", "aborting due"];
        assert!(!printed.iter().any(|needle| rendered.contains(needle)), "{printed:?} in {rendered}");
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
        // Four name the slice they land with; five name a question, because
        // those rows have no slice and a field typed `slice` would be filled
        // with a deferral's number; three name what owns them elsewhere,
        // because the pipeline's own commands wait on no question either and
        // `Question("the pipeline")` is that same lie in the other shape.
        let observed: Vec<(&str, &str, bool)> = REASONS
            .iter()
            .map(|(name, _, named)| {
                let reason = reason_of(&entries, name);
                (*name, case_of(&reason), named_by(&reason).contains(named))
            })
            .collect();
        let expected: Vec<(&str, &str, bool)> = REASONS.iter().map(|(name, case, _)| (*name, *case, true)).collect();
        assert_eq!(observed, expected, "each entry as (name, which of the three its reason is, what it names)");
        // And every unbuilt entry is one of those rows, so the three cases are
        // stated of the whole catalog and not of a chosen dozen.
        let unbuilt: Vec<&str> = entries.iter().filter(|entry| !is_built(entry)).map(|entry| entry.name).collect();
        assert_eq!(unbuilt.len(), REASONS.len(), "every unbuilt entry names one of the three: {unbuilt:?}");
    }

    #[test]
    #[validates(spec::AnUnbuiltCommandRefusesWithTheCatalogsReason)]
    fn an_unbuilt_command_refuses_with_the_catalogs_reason() {
        let root = scratch("unbuilt-refusal");
        let entries = table(&members());
        let names = ["shape", "mutants", "pr-body"];
        // The refusal names the command and the reason that entry carries —
        // one of each of the three, so a sentence that named the command and
        // dropped the reason is refused here. What the reason says is read off
        // the entry rather than composed a second time, but it is asserted to
        // be *in* the sentence: an expectation that called `unbuilt_sentence`
        // itself would hold for a sentence that ignored its reason entirely.
        let refused: Vec<(String, bool)> = names
            .iter()
            .map(|name| {
                let (seen, report) = recorded(name, &root, &members(), ToolOutput::default());
                (first_message(&report), seen.is_empty())
            })
            .collect();
        let observed: Vec<(bool, bool, bool)> = names
            .iter()
            .zip(&refused)
            .map(|(name, (sentence, nothing_ran))| {
                let reason = reason_of(&entries, name);
                (sentence.contains(name), sentence.contains(named_by(&reason)), *nothing_ran)
            })
            .collect();
        assert_eq!(observed, vec![(true, true, true); names.len()], "{refused:?}");
        // And it is the sentence composed in one place, which the printer is to
        // call rather than compose a second time.
        let composed: Vec<String> =
            names.iter().map(|name| unbuilt_sentence(name, &reason_of(&entries, name))).collect();
        assert_eq!(refused.iter().map(|(sentence, _)| sentence.clone()).collect::<Vec<String>>(), composed);
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
        //
        // Every lint the gate states is named here, and not the handful a
        // fixture would reach for: a mapping this fixture does not exercise is
        // a mapping nothing would notice the loss of, which is what check 12
        // found for checks 6 and 8 when they were correct and unnamed. The
        // list is README §4.1's tier-0 table read down its lint column, so a
        // check whose lint this map forgets is wrong here.
        let cases = [
            (Some("rustdoc::broken_intra_doc_links"), 2),
            (Some("missing_docs"), 3),
            (Some("clippy::missing_docs_in_private_items"), 3),
            (Some("clippy::wildcard_enum_match_arm"), 6),
            (Some("clippy::cognitive_complexity"), 7),
            (Some("clippy::fn_params_excessive_bools"), 8),
            (Some("clippy::too_many_lines"), 9),
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
        let root = scratch("cargo-provenance");
        // Every compiler message the stream held reaches a finding — the lint
        // no check names among them — while the records that are no diagnostic
        // reach none, and a stream holding none answers with none.
        //
        // Observed on the reader and on a run of `check`, whose output carries
        // a filled stderr beside the stream: a run that read the wrong stream
        // would answer rustdoc's two diagnostics rather than cargo's three, and
        // that is the only place the provenance dispatch can be seen at all.
        let both = ToolOutput { stdout: CARGO_STREAM.to_string(), stderr: RUSTDOC_STDERR.to_string() };
        let ran = recorded("check", &root, &members(), both).1;
        let held = |check: u32, line: u32, severity: &str, message: &str| -> Mapped {
            (check, Some(line), severity.to_string(), message.to_string(), "check".to_string())
        };
        let expected: Vec<Mapped> = vec![
            held(7, 12, "warning", "the function has a cognitive complexity of (5/4)"),
            held(0, 30, "warning", "this expression creates a reference which is immediately dereferenced"),
            held(0, 44, "error", "cannot find value `missing` in this scope"),
        ];
        let read = mapped_of(&findings_from_cargo(CARGO_STREAM, GATES, "check"));
        let none = mapped_of(&findings_from_cargo("", GATES, "check"));
        assert_eq!((read, mapped_of(&ran.findings), none), (expected.clone(), expected, Vec::new()));
    }

    #[test]
    #[validates(spec::ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr)]
    fn a_diagnostic_from_a_tool_with_no_json_stream_is_read_from_stderr() {
        let root = scratch("stderr-provenance");
        // Each diagnostic the tool printed is carried; the progress line above
        // them and the summary line below them are no diagnostics. A finding of
        // this provenance carries what one of the other does: the place it
        // points at, the check of the lint that raised it — in the spelling the
        // lint is declared with, not the tool's — and that check's row.
        //
        // Observed on the reader and on a run of `doc`, whose output carries a
        // full JSON stream on the stdout this provenance does not read: a run
        // that read the wrong stream would answer cargo's three diagnostics.
        let both = ToolOutput { stdout: CARGO_STREAM.to_string(), stderr: RUSTDOC_STDERR.to_string() };
        let ran = recorded("doc", &root, &members(), both).1;
        let expected: Vec<Read> = vec![
            (Some(SOURCE_FILE.to_string()), Some(12), 2, Some(FIX_TWO.to_string()), "warning".to_string()),
            (Some(SOURCE_FILE.to_string()), Some(20), 0, None, "error".to_string()),
        ];
        let read = read_of(&findings_from_stderr(RUSTDOC_STDERR, GATES, "doc"));
        let none = read_of(&findings_from_stderr("", GATES, "doc"));
        assert_eq!((read, read_of(&ran.findings), none), (expected.clone(), expected, Vec::new()));
        // `cargo package` shares this provenance and points at no place in the
        // source at all: a reader keyed to rustdoc's `-->` line answers nothing
        // over a packaging that failed, which is the `package` step passing
        // over every reason it could not publish.
        let packaging = findings_from_stderr(PACKAGE_STDERR, GATES, "package");
        let bare: Vec<Read> =
            vec![(None, None, 0, None, "warning".to_string()), (None, None, 0, None, "error".to_string())];
        let said = ["manifest has no documentation", "failed to verify package tarball"];
        assert_eq!(read_of(&packaging), bare, "{packaging:?}");
        assert!(packaging.iter().zip(said).all(|(found, sentence)| found.message.contains(sentence)), "{packaging:?}");
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
        // The spawning half decides which stream the comparison's message
        // arrives on, and nothing else does: `sync::check` is this binary's own
        // work rather than a tool's, so a `spawned` answering on stdout leaves a
        // comparison that reported differences with no finding at all. Observed
        // without spawning anything, on a project that resolves nothing — what
        // the comparison says then is another slice's wording and is not pinned
        // here, only that it said something, that it arrived on stderr, and that
        // one finding carries it whole.
        let unresolvable = Project::from_json("{}").expect("a project that resolves nothing");
        let spoke = spawned(&unresolvable, "sync", &Invocation::SyncComparison).expect("the comparison answered");
        let said = findings_of(&Invocation::SyncComparison, &spoke, GATES, "sync");
        let observed = (spoke.stdout.as_str(), spoke.stderr.is_empty(), said.len(), said.first().map(|f| f.message.as_str()));
        assert_eq!(observed, ("", false, 1, Some(spoke.stderr.as_str())), "{spoke:?}");
    }

    #[test]
    #[validates(spec::PackageNamesEveryPublishingMemberInOneInvocation)]
    fn package_names_every_publishing_member_in_one_invocation() {
        let root = scratch("package-invocation");
        let publishing = members();
        let (seen, _) = recorded("package", &root, &publishing, ToolOutput::default());
        // One invocation naming them all: the per-crate form cannot resolve a
        // sibling at a version no registry holds. Each member is one whole
        // argument, since `lid-rs` is a substring of the other two names and an
        // invocation naming only those would hold a search for it.
        let args = args_of(seen.first().expect("the package invocation"));
        let named: Vec<bool> = publishing.iter().map(|member| args.contains(member)).collect();
        assert_eq!((seen.len(), named), (1, vec![true; publishing.len()]), "{args:?}");
        // The members are the workspace's answer, not the table's own list.
        let (other, _) = recorded("package", &root, &strings(&["only-me"]), ToolOutput::default());
        let narrowed = args_of(other.first().expect("the package invocation"));
        let held = ["only-me", "lid-rs-macros"].map(|member| narrowed.iter().any(|arg| arg == member));
        assert_eq!(held, [true, false], "{narrowed:?}");
    }

    #[test]
    #[validates(spec::TheDocInvocationNamesTheFlagThatDocumentsPrivateItems)]
    fn the_doc_invocation_names_the_flag_that_documents_private_items() {
        let root = scratch("doc-private-items");
        let (seen, _) = recorded("doc", &root, &members(), ToolOutput::default());
        // Without the flag a private item's documentation is never read, and
        // check 3 and rustdoc stop agreeing about what is documented.
        //
        // Each is one whole argument. `doc` is a substring of
        // `--document-private-items` and of the `RUSTDOCFLAGS` value, so a
        // search of the printed invocation for it holds over an invocation that
        // names neither the command nor the flag.
        let args = args_of(seen.first().expect("the doc invocation"));
        let named = ["doc", "--no-deps", "--document-private-items"].map(|token| args.iter().any(|arg| arg == token));
        assert_eq!(named, [true, true, true], "{args:?}");
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
