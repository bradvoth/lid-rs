//! Claims for the `catalog` slice — a check is a command with a finding
//! (`cargo-lid-rs/src/catalog/lld.md`).
//!
//! The slice builds five commands — `check`, `lint`, `doc`, `package` and
//! `sync` — over one shape: a fixed invocation, a report in the finding schema
//! at `target/lid/<command>.json`, a rendering built from that report, and a
//! status. The claims below are that shape, what a diagnostic becomes whichever
//! stream it arrived on, the provenance of each family of findings, the three
//! rows whose invocation the document settles as a behaviour, and the catalog's
//! listing of itself.
//!
//! **Every behavioural claim is seated in `run_with`, never in `run`.** A run
//! decides four things — which invocation it hands the runner, that it writes an
//! entry's report whatever it found, which status it answers with, and what it
//! refuses
//! — and none of them is observable from a signature that must really invoke
//! cargo. `run` spawns and carries no citation; `run_with` takes the runner, so
//! a validation's runner records the `Invocation` it was handed and every one of
//! those decisions has a wrong answer a test can see. A claim naming `run`
//! would be a claim whose only validation is a build.
//!
//! **Which invocation a command runs is the table's data, and not a claim
//! each.** `ACommandRunsTheFixedInvocationItsEntryNames` is the rule over every
//! row of `table`, and a claim per row would assert what the row already is.
//! Three rows are exceptions, because at each of them the document decides a
//! behaviour rather than a string: `package` names every publishing member in
//! one invocation, which is the form that resolves a sibling at a version no
//! registry holds; `doc` both documents private items and denies a broken
//! intra-doc link; and `sync` yields the message of a comparison this slice
//! cannot take apart, or nothing where it reported no difference.
//!
//! **`doc`'s row settles two things, so it is two claims.** The flag decides
//! what is documented and the environment decides what a broken link does:
//! without `--document-private-items` a private item's documentation is never
//! read, and without `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links` rustdoc
//! answers success over a link it could not resolve and the step gates nothing.
//! Neither is implied by the other, and
//! `ACommandRunsTheFixedInvocationItsEntryNames` is satisfied by whatever the
//! entry names — including an empty environment — so the environment is stated
//! here or nowhere. It is stated of the `Invocation` that `run_with` hands the
//! runner rather than of the spawning that applies it, because `run` carries no
//! citation and an environment a validation cannot see is an environment no
//! test can be wrong about.
//!
//! **`sync` is one finding by decision, and none is the other half of the
//! rule.** `sync::check` answers with the differences joined into one string and
//! its three comparison helpers are private to a module no phase of this slice
//! may write, so the granularity a claim can honestly state today is one. A
//! claim of one finding per difference would be validated against a parse of
//! that string — the scrape this slice's own rendering rule forbids. The claim
//! is a mapping from what the comparison reported, for the reason the status is
//! a mapping: a claim conditioned on the `sync` command alone demands a finding
//! of a comparison that reported no difference,
//! `TheStatusIsZeroWithNoFindingAndOneWithAFinding` then makes a clean
//! workspace's `sync` exit 1, and the gate step fails forever on a passing run.
//! Each half alone is satisfied by a constant, so the mapping is the claim.
//!
//! **The status is two claims and not three.** Zero with no finding and one with
//! any finding is a single total function of the finding list: each half alone
//! is satisfied by a constant, so the mapping is the claim and splitting it
//! leaves neither piece asserting anything. The tooling status is not a third
//! case of that function — it is the case where there is no finding list to map,
//! because nothing produced one. So one claim states the function and another
//! states the occasion the function is never reached in. The check number is one
//! claim for the same reason: a mapping from the lint that raised a diagnostic,
//! whose default arm is 0.
//!
//! **The tooling status is one rule and three occasions.**
//! `ARunThatReachesNoAnswerIsTheToolingStatus` is the rule; a command the
//! catalog holds as unbuilt, a name the catalog holds no entry for, and a report
//! directory that cannot be created are the three occasions this slice knows of,
//! and a fourth is judged against the rule. None of the three restates the
//! status: each states the sentence or the finding its refusal carries and then
//! names the rule's condition, so the relation between them is in the text
//! rather than in a reader's head.
//!
//! **A name with no entry is a branch of its own.** It is not the unbuilt case:
//! an unbuilt entry exists and carries a reason to refuse with, and a name with
//! no entry has nothing to read a reason from, so the sentence names the name
//! instead. An unwritten branch is a requirement nobody recorded, and the two
//! arms answer with the same status only because the rule above says so.
//!
//! **The two arms differ in what they write, and that is the second claim about
//! the one with no entry.** The report file belongs to an *entry*:
//! `ARunWritesItsReportEvenWhenItFindsNothing` is scoped to a command `table`
//! holds an entry for and names the file for that entry — an unbuilt entry
//! included, because a pipeline reading `target/lid/<command>.json` for a
//! command the catalog holds is owed the tooling status it would find there. A
//! name with no entry names no file, and `ARefusalOfANameWithNoEntryWritesNoFile`
//! is what makes that written down rather than an exception to "written always"
//! that nobody recorded. It is also what bounds the path: `run_with`'s `command`
//! is arbitrary user input, and
//! `root.join("target/lid").join(format!("{command}.json"))` for `../../foo`
//! writes outside the workspace, so taking the name from the entry makes that
//! traversal unrepresentable instead of sanitised. That the name comes from the
//! entry is no claim of its own — an entry is resolved by equality with the name
//! given, so no test can see the difference for a name that resolves. What a
//! test can see is that a name with no entry writes nothing.
//!
//! **An unbuilt entry names a reason, and a reason is one of three things.**
//! Six of pipeline `§5.1`'s sixteen rows have no slice to name. Four wait on a
//! libtest with machine-readable output and one on four open questions, so a
//! claim about the slice an entry names would be validated against a field
//! filled with a deferral's number. The remaining three — `pr-body`, `status`
//! and `commit` — are the pipeline's own: they wait on no question either, and a
//! claim of a slice or a question would be validated against a field holding
//! "the pipeline" as though that were a question it waits on. The same lie in
//! the other shape, for three of the seventeen rows.
//!
//! **The third case is a distinction and not bookkeeping.** A command waiting on
//! a slice or on a question is one this tool builds one day; a command owned
//! elsewhere is one it never builds. Which of the three a given row names is the
//! table's data, for the reason its invocation is — a claim per row would assert
//! what the row already is — and the claim here is the totality: every unbuilt
//! entry names one of the three, and no case alone holds of the rows the others
//! cover.
//!
//! **What a diagnostic becomes is one rule, and it is `finding_of`'s.** Each
//! provenance answers with a `Diagnostic` and one item turns that into a
//! `Finding`, so the three rules about what a finding carries — the file and
//! line its diagnostic points at, the check number of the lint that raised it,
//! and the `fix` from the `gates.md` row for that check — are stated of
//! `finding_of` and of no other item. The first draft stated two of them of
//! `findings_from_cargo`, which was then the only item that built a finding, and
//! neither reached the stderr path: a `findings_from_stderr` answering
//! `file: None, line: None, check: 0` for every rustdoc diagnostic satisfied
//! both of its claims. It stated the `fix` rule of a `Finding`, which names no
//! item at all — a struct's field cannot be wrong on its own, so the item that
//! could be wrong about it carried no claim.
//!
//! **That a stderr finding carries what a cargo finding carries is no claim of
//! its own.** It is what one item applying one rule means: the three rules above
//! hold of every finding either provenance answers with, because one item builds
//! them all. A claim that the two provenances agree would state those rules a
//! second time, keyed to a coincidence between two outputs rather than to the
//! item that decides — the "six places state a gate" disease this slice exists
//! to cure. What such a claim would buy is an observation, and the observation
//! is had without it: a validation of
//! `ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr` that reads the file
//! and line off a stderr diagnostic's finding sees a provenance that built its
//! findings some other way.
//!
//! **A provenance's claim is about the whole provenance, and stays there.**
//! `EveryDiagnosticOfTheStreamBecomesAFinding` and
//! `ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr` are not claims about
//! parsing. Each says that every diagnostic one stream holds reaches a
//! `Finding`, and neither `diagnostics_from_cargo` nor `diagnostics_from_stderr`
//! answers with a finding at all; a claim seated on a parser would have to be
//! rewritten into a claim about `Diagnostic` values, and the guarantee that
//! nothing is dropped between a tool's output and a report would then be no
//! item's. They are also the pair that draws the distinction between the two
//! streams — which of a tool's two outputs an entry's findings are read from is
//! a dispatch over the provenance, and a parser handed one string cannot be
//! wrong about which string it was handed.
//!
//! **The catalog's claims are about `table` and not about printing it.** The
//! shape table holds the listing and the runner, and nothing in it prints: a
//! claim whose *when* was the printing would name an occasion no item of this
//! slice creates, while what it asserts holds of the listing whether or not
//! anything prints it. What `cargo lid-rs catalog` prints is `table` read aloud,
//! and `table` is what a validation observes.
//!
//! **What the report carries is what a test can observe.** `main` maps every
//! error to exit 1 and sits in no phase's allowed paths, so until that changes a
//! tooling status is visible in the report and not in the process's exit code.
//! `TheReportCarriesTheRunsStatus` is load-bearing for that reason: without it
//! the two status claims assert something nothing in this crate can observe.
//!
//! **What has no claim here.** `examples`, `suite`, `graph`, `regen`,
//! `mutants`, the per-command timeouts, the composites, and the gate's
//! definition are each deferred by the document — to a libtest with
//! machine-readable output on stable, to four open questions, to a manifest no
//! phase may write, and to the `phase` slice. A claim for any of them would be a
//! claim about code this slice does not build, with nothing for a validation to
//! attach itself to.
//!
//! These claims are written in the controlled language, like the `layout`
//! slice's; none is marked free.

use lid_rs::Spec;

// ---- One command, one invocation, one report ---------------------------------

/// When [`run_with`](crate::catalog::run_with) is given a command, the
/// [`Invocation`](crate::catalog::Invocation) it hands the
/// [`Runner`](crate::catalog::Runner) shall be the fixed one that command's
/// entry in [`table`](crate::catalog::table) names, taking no argument from the
/// command line.
#[derive(Spec)]
pub struct ACommandRunsTheFixedInvocationItsEntryNames;

/// When [`run_with`](crate::catalog::run_with) has finished a command
/// [`table`](crate::catalog::table) holds an entry for, the
/// [`Report`](crate::catalog::Report) it writes shall be at
/// `target/lid/<entry name>.json`, carrying the empty finding list where that
/// command found nothing.
#[derive(Spec)]
pub struct ARunWritesItsReportEvenWhenItFindsNothing;

/// When [`render`](crate::catalog::render) is given a
/// [`Report`](crate::catalog::Report), the text it prints shall be built from
/// that report's findings and never from the output of the tool the command ran.
#[derive(Spec)]
pub struct TheRenderingIsBuiltFromTheFindingsAndNotTheToolsOutput;

/// When [`run_with`](crate::catalog::run_with) has mapped a command's output to
/// findings, the [`Status`](crate::catalog::Status) it answers with shall be 0
/// where that finding list is empty and 1 where it holds a finding.
#[derive(Spec)]
pub struct TheStatusIsZeroWithNoFindingAndOneWithAFinding;

/// When [`run_with`](crate::catalog::run_with) reaches no answer about the
/// project for a command, the [`Status`](crate::catalog::Status) it answers with
/// shall be 2, a status no finding list produces.
#[derive(Spec)]
pub struct ARunThatReachesNoAnswerIsTheToolingStatus;

// ---- The catalog lists itself -------------------------------------------------

/// When [`table`](crate::catalog::table) is read, every command of pipeline
/// `§5.1`, and the `package` step this workspace adds beside them, shall be
/// named there with whether this workspace builds it.
#[derive(Spec)]
pub struct TheCatalogNamesEveryCommandAndWhetherItIsBuilt;

/// When [`table`](crate::catalog::table) holds a command this workspace does not
/// build, the [`Unbuilt`](crate::catalog::Unbuilt) that entry carries shall name
/// the slice the command lands with, the question it waits on, or the thing that
/// owns the command where this tool builds it at no point.
#[derive(Spec)]
pub struct AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere;

/// The name [`AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere`] carried
/// while a reason was two things rather than three. The alias registers no
/// claim, so the graph sees only the claim it points at; every citation of this
/// name warns with its replacement, and those citations are the later phases'
/// work list.
#[deprecated = "replaced by AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere"]
pub type AnUnbuiltEntryNamesTheSliceOrTheQuestionItWaitsOn =
    AnUnbuiltEntryNamesASliceAQuestionOrAnOwnerElsewhere;

/// When [`run_with`](crate::catalog::run_with) is given the name of a command
/// [`table`](crate::catalog::table) holds as unbuilt, it shall refuse with that
/// entry's reason, reaching no answer about the project.
#[derive(Spec)]
pub struct AnUnbuiltCommandRefusesWithTheCatalogsReason;

/// When [`run_with`](crate::catalog::run_with) is given a name
/// [`table`](crate::catalog::table) holds no entry for, it shall refuse naming
/// the name it was given, reaching no answer about the project.
#[derive(Spec)]
pub struct ANameTheCatalogHoldsNoEntryForIsRefusedByName;

/// When [`run_with`](crate::catalog::run_with) refuses a name
/// [`table`](crate::catalog::table) holds no entry for, that refusal shall be
/// carried to the caller in a [`Report`](crate::catalog::Report) and written to
/// no file, since there is no entry whose report it would be.
#[derive(Spec)]
pub struct ARefusalOfANameWithNoEntryWritesNoFile;

// ---- What a diagnostic becomes, whichever stream it arrived on ----------------

/// When [`finding_of`](crate::catalog::finding_of) is given a
/// [`Diagnostic`](crate::catalog::Diagnostic) that points at a place in the
/// source, the [`Finding`](crate::catalog::Finding) it answers with shall name
/// that file and line.
#[derive(Spec)]
pub struct AFindingNamesTheFileAndLineOfItsDiagnostic;

/// When [`finding_of`](crate::catalog::finding_of) sets a
/// [`Finding`](crate::catalog::Finding)'s check number, that number shall be the
/// number of the LID-rs check naming the lint that raised the diagnostic, and 0
/// where no check names it.
#[derive(Spec)]
pub struct TheCheckNumberComesFromTheLintOrIsZero;

/// When [`finding_of`](crate::catalog::finding_of) builds a
/// [`Finding`](crate::catalog::Finding) whose check the skill's
/// `references/gates.md` holds a row for, the `fix` that finding carries shall
/// be that row's correct response.
#[derive(Spec)]
pub struct TheFixLineIsTheGatesRowForTheFindingsCheck;

// ---- The findings of a command whose tool emits JSON diagnostics --------------

/// When [`findings_from_cargo`](crate::catalog::findings_from_cargo) reads
/// cargo's JSON diagnostic stream, every diagnostic that stream holds shall be
/// carried into a [`Finding`](crate::catalog::Finding), whether or not a LID-rs
/// check names the lint that raised it.
#[derive(Spec)]
pub struct EveryDiagnosticOfTheStreamBecomesAFinding;

// ---- The findings of a command whose tool emits none --------------------------

/// When [`findings_from_stderr`](crate::catalog::findings_from_stderr) is given
/// the stderr of a tool that emits no JSON diagnostic stream, each diagnostic
/// printed there shall be carried into a [`Finding`](crate::catalog::Finding).
#[derive(Spec)]
pub struct ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr;

/// When [`run_with`](crate::catalog::run_with) has run the `sync` command's
/// comparison, the findings of that run shall be one
/// [`Finding`](crate::catalog::Finding) carrying the comparison's message whole
/// where that comparison reported differences, and empty where it reported
/// none.
#[derive(Spec)]
pub struct TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout;

/// The name [`TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout`]
/// carried while its condition was the `sync` command rather than what that
/// command's comparison reported. The alias registers no claim, so the graph
/// sees only the claim it points at; every citation of this name warns with its
/// replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout"]
pub type TheSyncCommandYieldsOneFindingCarryingTheComparisonsMessage =
    TheSyncFindingsAreOneMessageWithDifferencesAndNoneWithout;

// ---- The rows whose invocation is a behaviour ---------------------------------

/// When [`run_with`](crate::catalog::run_with) is given the `package` command,
/// the one [`Invocation`](crate::catalog::Invocation) it hands the
/// [`Runner`](crate::catalog::Runner) shall name every member of the workspace
/// that publishes.
#[derive(Spec)]
pub struct PackageNamesEveryPublishingMemberInOneInvocation;

/// When [`run_with`](crate::catalog::run_with) is given the `doc` command, the
/// [`Invocation`](crate::catalog::Invocation) it hands the
/// [`Runner`](crate::catalog::Runner) shall name the flag that documents a
/// crate's private items, so that what a private item's documentation says is
/// read at all.
#[derive(Spec)]
pub struct TheDocInvocationNamesTheFlagThatDocumentsPrivateItems;

/// When [`run_with`](crate::catalog::run_with) is given the `doc` command, the
/// [`Invocation`](crate::catalog::Invocation) it hands the
/// [`Runner`](crate::catalog::Runner) shall carry the `RUSTDOCFLAGS` that denies
/// a broken intra-doc link, without which rustdoc answers success over one.
#[derive(Spec)]
pub struct TheDocInvocationCarriesTheEnvironmentThatDeniesABrokenLink;

// ---- Where the report goes, and what it carries -------------------------------

/// When [`run_with`](crate::catalog::run_with) has a
/// [`Report`](crate::catalog::Report) to write and the workspace root holds no
/// `target/lid` directory, that directory shall be created before the report is
/// written.
#[derive(Spec)]
pub struct TheReportDirectoryIsCreatedOnDemand;

/// When the `target/lid` directory [`run_with`](crate::catalog::run_with) needs
/// cannot be created, that run shall report a
/// [`Finding`](crate::catalog::Finding) naming the directory, reaching no answer
/// about the project.
#[derive(Spec)]
pub struct ADirectoryThatCannotBeCreatedIsNamedByAFinding;

/// When [`run_with`](crate::catalog::run_with) answers with a
/// [`Report`](crate::catalog::Report), that report shall carry the run's
/// [`Status`](crate::catalog::Status), where a reader finds it though the
/// process's exit code cannot hold it.
#[derive(Spec)]
pub struct TheReportCarriesTheRunsStatus;
