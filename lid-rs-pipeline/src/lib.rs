#![doc = include_str!("../docs/intent/pipeline.md")]
#![doc = include_str!("lld.md")]

pub mod spec;

use std::path::{Path, PathBuf};

use cargo_lid_rs::catalog::Finding;
use cargo_lid_rs::headless_canopy_agent::{fork_point, own_commits};
use cargo_lid_rs::layout;
use cargo_lid_rs::phase::{Phase, Tag, tag_of};
use cargo_lid_rs::project::Project;
use lid_rs::implements;

/// The `git log` format one commit of the branch is read in: its object name,
/// its subject, and its message's trailers unfolded onto one line each, the
/// record ended by [`RECORD_SEPARATOR`] rather than by a blank line, so that a
/// trailer value carrying a newline cannot be taken for the next record.
///
/// One wide read rather than two: the branch's own commits are enumerated once
/// and every phase commit's trailers come back with them, which is what makes
/// reporting all of them cost no more than reporting the newest one's.
const LOG_FORMAT: &str = "--format=%H%n%s%n%(trailers:only=true,unfold=true)%x00";

/// The byte each record of a [`LOG_FORMAT`] log ends with — a NUL, which no
/// commit message can hold.
const RECORD_SEPARATOR: char = '\0';

/// What a subject opens with to be this reading's business at all.
///
/// Read here rather than taken from
/// [`tag_of`](cargo_lid_rs::phase::tag_of), which cannot tell `phase foo:`
/// from `chore: x` — both are `Tag::Untagged` — while the first is a finding
/// and the second is a commit the reading has nothing to say about. A
/// seven-character literal in a second crate is the price; the alternative is
/// a `Tag::Malformed` variant on the `phase` slice, which is a third
/// cross-slice branch (`lld.md`, Decisions).
const PHASE_PREFIX: &str = "phase ";

/// The trailer a phase commit records the tool that made it under.
const TOOL_TRAILER: &str = "Lid-Rs-Tool";

/// The version a [`TOOL_TRAILER`] value is compared against: this build's own.
/// The workspace gives every member one version, so the `cargo-lid-rs` a run
/// links and the version it reports cannot differ.
const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// One `Name: value` trailer of a commit message.
///
/// A name and a value rather than a closed set of names: the trailers a phase
/// commit carries grow with what writes them, and a reading that recognised
/// only the names it knew would drop the next one silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trailer {
    /// The trailer's name, as the message spells it — `Lid-Rs-Phase`.
    pub name: String,
    /// The trailer's value, whole.
    pub value: String,
}

/// One `phase N:` commit the branch made, as the reading found it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseCommit {
    /// The commit's object name.
    pub commit: String,
    /// The commit's subject line, whole.
    pub subject: String,
    /// The phase the subject's `phase <N>:` prefix names — a phase with a
    /// check of its own, which is what
    /// [`tag_of`](cargo_lid_rs::phase::tag_of) answers — and none where the
    /// subject names no such phase, whether its prefix carries a number no
    /// phase has (`phase 6:`), a number that will not parse (`phase foo:`),
    /// or no number at all.
    ///
    /// `tag_of` answers this field and cannot answer the finding beside it:
    /// it maps `phase foo:` and `chore: x` alike to `Tag::Untagged`, while
    /// the first is a malformed subject and the second is a commit the
    /// reading has nothing to say about. Which of the two a record is, is
    /// read from the subject's `phase ` opening when the record is taken,
    /// not from the tag.
    pub phase: Option<Phase>,
    /// The commit's trailers, in the order its message carries them, so a
    /// reader sees which agent and which tally produced the phase.
    pub trailers: Vec<Trailer>,
}

/// Why a resume is not a resume.
///
/// The two answers a reading that reached the slice's documents can give. A
/// reading that could not reach them gives neither, which is the absent
/// [`Status::restart`] rather than a third variant here: the variants are the
/// reasons a run resumes or begins again, and "unanswered" is not a reason.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
pub enum Restart {
    /// Neither document moved after the newest phase commit: a run takes the
    /// branch up where it left off.
    Resume,
    /// A document the committed phases were derived from changed after the
    /// newest phase commit, so a run begins again at Phase 2.
    AtPhaseTwo {
        /// The document that changed — the slice's `lld.md` or the
        /// workspace's `hld.md`.
        document: PathBuf,
    },
}

/// A branch's state as a value: which slice it is for, how far its phases are
/// committed, what a run would do next, and every phase commit it made.
///
/// Three of its fields are places the reading needs and a state made of the
/// newest commit alone would not have: `commits` holds a record per phase
/// commit rather than the newest commit's trailers, `uncommitted` holds work
/// the tree has not committed rather than being a reason to refuse, and
/// `restart` is absent where the reading could not reach that verdict at all.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(
    spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers,
    spec::ADirtyWorkingTreeIsReportedAndNotRefused,
    spec::AnUnresolvableSliceLeavesTheRestartVerdictUnanswered,
)]
pub struct Status {
    /// The slice the branch names.
    pub slice: String,
    /// The commit the branch left its base at: where its own commits begin
    /// ([`fork_point`](cargo_lid_rs::headless_canopy_agent::fork_point)), and
    /// the empty string for a branch sharing no commit with the base.
    pub branch_point: String,
    /// The phase last committed on this branch, and none where the branch made
    /// no phase commit of its own.
    pub phase: Option<Phase>,
    /// The phase a run would take up next, and none where the branch holds a
    /// commit for every phase a run builds.
    pub next: Option<Phase>,
    /// Whether a resume is a resume, and none where the reading could not
    /// reach that verdict — the slice's `lld.md` being a document no workspace
    /// member holds.
    pub restart: Option<Restart>,
    /// Every `phase N:` commit the branch made, newest first, with its
    /// trailers.
    pub commits: Vec<PhaseCommit>,
    /// The paths the working tree holds uncommitted work in, from `git status
    /// --porcelain`, and empty for a clean tree.
    pub uncommitted: Vec<String>,
}

/// What a reading answers with, and what `target/lid/status.json` holds: the
/// branch's state beside the findings the reading itself raised.
///
/// The state is not a defect and does not fit the finding schema; the findings
/// are the three things a reading can raise about one commit or one name. The
/// pair keeps both without making either the other.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::TheReportCarriesTheStateAndTheFindingsTheReadingRaised)]
pub struct Report {
    /// The branch's state.
    pub state: Status,
    /// What the reading raised while reading it — empty where it raised
    /// nothing, which is an answer and not an absence.
    pub findings: Vec<Finding>,
}

/// The branch's state, read from the branch alone.
///
/// The slice is the one `branch` names, or `slice` where a caller gives one,
/// as [`slice_of`](cargo_lid_rs::headless_canopy_agent::slice_of) resolves the
/// pair; a branch that is neither is the one occasion this reading refuses,
/// since there is then no slice to answer about. Everything else it meets is
/// answered: a subject it cannot read the phase of, a commit made by a tool of
/// another version, an unresolvable slice and an uncommitted tree are findings
/// beside the state, because refusing a reading is useless in the case an
/// operator most wants one.
///
/// The state is read from the branch's own commits — those after
/// [`fork_point`](cargo_lid_rs::headless_canopy_agent::fork_point), the range
/// [`own_commits`](cargo_lid_rs::headless_canopy_agent::own_commits) names —
/// and never from the whole ancestry, in which a merged slice leaves every one
/// of its phase subjects. That range is read once, in a format wide enough to
/// carry each commit's trailers as well as its subject, rather than through
/// [`log_subjects`](cargo_lid_rs::headless_canopy_agent::log_subjects), which
/// answers subjects alone and would have to be read a second time per commit.
/// The phase a run would take up next is the first of
/// [`PHASES`](cargo_lid_rs::headless_canopy_agent::PHASES) the branch holds no
/// commit for, which is the order a run itself walks.
///
/// Whether the slice exists at all is
/// [`own_crate`](cargo_lid_rs::layout::own_crate)'s answer: it is the door that
/// refuses, by name, for a slice no workspace member holds a document for, and
/// that refusal is this reading's finding rather than its exit.
/// [`lld_path`](cargo_lid_rs::layout::lld_path) supplies only the path the
/// restart verdict compares against and cannot stand in for it: it answers
/// `Ok` in both arms, falling back to a workspace-root path it never probes.
///
/// It reads and nothing more: it advances no state, writes no commit, and
/// opens no session.
///
/// Every part of the reading below is one of the items this composes: the
/// slice from the branch, the branch's own commits from one wide log, the
/// two phases the state names, the restart verdict, the work the tree has
/// not committed, and the three findings — none of which this function
/// decides, and all of which it puts in one place.
#[implements(
    spec::ABranchThatNamesNoSliceIsRefusedNamingTheConvention,
    spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint,
    spec::TheReportCarriesTheStateAndTheFindingsTheReadingRaised,
)]
pub fn status(project: &Project, branch: &str, slice: Option<&str>) -> Result<Report, String> {
    let slice = slice_named(branch, slice)?;
    let branch_point = fork_point(project)?;
    let commits = phase_commits(&branch_log(project, &own_commits(&branch_point))?);
    let unresolvable = layout::own_crate(project, &slice).err();
    let state = Status {
        phase: newest_phase(&commits),
        next: next_phase(&commits),
        restart: restart_verdict(project, &slice, &since_newest_phase(&commits, &branch_point), unresolvable.as_deref())?,
        uncommitted: uncommitted(project)?,
        commits,
        branch_point,
        slice,
    };
    let findings = [
        malformed_findings(&state.commits),
        tool_findings(&state.commits, TOOL_VERSION),
        unresolvable_findings(&state.slice, unresolvable.as_deref()),
    ]
    .concat();
    Ok(Report { state, findings })
}

/// The slice the branch is for: the one `branch` names, or `slice` where a
/// caller gives one, as
/// [`slice_of`](cargo_lid_rs::headless_canopy_agent::slice_of) resolves the
/// pair — and, for a branch that is neither, the refusal naming the
/// `lld/<slice>` convention, which is the one occasion this reading answers
/// with no state at all.
///
/// The refusal is the sentence `slice_of` already produces rather than a
/// second wording of it: one convention, stated in one place. What differs is
/// its shape — a `Stop` is the canopy run's ending, and a reading has none.
#[implements(spec::ABranchThatNamesNoSliceIsRefusedNamingTheConvention)]
fn slice_named(branch: &str, slice: Option<&str>) -> Result<String, String> {
    todo!("resolve the slice of `{branch}`, against {slice:?}")
}

/// The branch's own commits in [`LOG_FORMAT`], over the range `range` names —
/// the commits after the branch's fork point
/// ([`own_commits`](cargo_lid_rs::headless_canopy_agent::own_commits)), never
/// the whole ancestry, in which a merged slice leaves every one of its phase
/// subjects.
///
/// The range is given rather than computed here so that the one place the
/// branch's own commits are decided stays
/// [`own_commits`](cargo_lid_rs::headless_canopy_agent::own_commits)'s.
#[implements(
    spec::ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase,
    spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint,
    spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers,
)]
fn branch_log(project: &Project, range: &str) -> Result<String, String> {
    todo!("read `git log {LOG_FORMAT} {range}` in {:?}", project.root())
}

/// Every phase commit a [`LOG_FORMAT`] log holds, newest first as `git log`
/// orders them: each record read by [`phase_record`], and the records it
/// answers none for dropped.
#[implements(
    spec::TheStateIsTheNewestPhaseCommitTheBranchMade,
    spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers,
)]
fn phase_commits(log: &str) -> Vec<PhaseCommit> {
    log.split(RECORD_SEPARATOR).filter_map(phase_record).collect()
}

/// One record of a [`LOG_FORMAT`] log as a [`PhaseCommit`] — its object name,
/// its subject, the phase [`checked_phase`] reads from that subject, and the
/// [`trailers`] its message carries — and none for a record whose subject
/// does not open [`PHASE_PREFIX`], which is a commit this reading is not
/// about.
///
/// A subject that opens `phase ` is kept whatever follows it, including the
/// subjects no phase can be read from: they are what
/// [`malformed_findings`] raises a finding against, and a reading that
/// dropped them here could raise none.
#[implements(
    spec::APhaseIsTheNumberOfItsSubjectsPhasePrefix,
    spec::AMalformedPhaseSubjectIsAFindingAgainstItsCommit,
    spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers,
)]
fn phase_record(record: &str) -> Option<PhaseCommit> {
    let (commit, rest) = record.trim_start().split_once('\n')?;
    let (subject, block) = rest.split_once('\n')?;
    subject.starts_with(PHASE_PREFIX).then(|| PhaseCommit {
        commit: commit.to_string(),
        subject: subject.to_string(),
        phase: checked_phase(subject),
        trailers: trailers(block),
    })
}

/// The phase a subject's `phase <N>:` prefix names, and none where it names
/// no phase with a check of its own — the one decision over what
/// [`tag_of`](cargo_lid_rs::phase::tag_of) answered.
#[implements(spec::APhaseIsTheNumberOfItsSubjectsPhasePrefix)]
fn checked_phase(subject: &str) -> Option<Phase> {
    match tag_of(subject) {
        Tag::Checked(phase) => Some(phase),
        Tag::Untagged | Tag::Unchecked(_) => None,
    }
}

/// The `Name: value` trailers of one commit's message, in the order the
/// message carries them, from the block a [`LOG_FORMAT`] record ends with:
/// one [`Trailer`] a line, unfolded, and none for a message that carries
/// none.
#[implements(spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers)]
fn trailers(block: &str) -> Vec<Trailer> {
    todo!("read the trailers of {block:?}")
}

/// The phase the branch last committed: the phase of the newest of its
/// [`PhaseCommit`]s that names one, and none where the branch made no phase
/// commit of its own, or none of the ones it made names a phase with a check.
#[implements(
    spec::TheStateIsTheNewestPhaseCommitTheBranchMade,
    spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint,
)]
fn newest_phase(commits: &[PhaseCommit]) -> Option<Phase> {
    todo!("take the newest phase among {} commit(s)", commits.len())
}

/// The phase a run would take up next: the first of the phases a run builds
/// ([`PHASES`](cargo_lid_rs::headless_canopy_agent::PHASES)) that the
/// branch's own commits hold no commit for, and none where they hold one for
/// every phase.
#[implements(spec::ThePhaseNextIsTheFirstOneTheBranchHasNoCommitFor)]
fn next_phase(commits: &[PhaseCommit]) -> Option<Phase> {
    todo!("take the first phase none of {} commit(s) holds", commits.len())
}

/// The paths the working tree holds uncommitted work in, from
/// `git status --porcelain`, and empty for a clean tree.
///
/// Read here rather than through the canopy slice's precondition: that
/// reading is private to it and answers the opposite question — it refuses a
/// dirty tree, and this one reports it.
#[implements(spec::ADirtyWorkingTreeIsReportedAndNotRefused)]
fn uncommitted(project: &Project) -> Result<Vec<String>, String> {
    todo!("read `git status --porcelain` in {:?}", project.root())
}

/// The commits the restart verdict reads over: those after the newest
/// [`PhaseCommit`] the branch made, and the branch's own commits where it
/// made none — a branch with no phase commit has nothing for a document to
/// have moved *after*, so everything it committed is the range.
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
fn since_newest_phase(commits: &[PhaseCommit], branch_point: &str) -> String {
    own_commits(commits.first().map_or(branch_point, |newest| newest.commit.as_str()))
}

/// Whether a resume is a resume, and none where the reading could not reach
/// that verdict at all: the one decision over whether the layout placed the
/// slice, `unresolvable` carrying the refusal it placed none with.
///
/// The verdict needs the slice's `lld.md`, so a slice no workspace member
/// holds a document for leaves it unanswered rather than answered "resume" —
/// which would tell a run to carry on from a check nothing made.
#[implements(spec::AnUnresolvableSliceLeavesTheRestartVerdictUnanswered)]
fn restart_verdict(project: &Project, slice: &str, range: &str, unresolvable: Option<&str>) -> Result<Option<Restart>, String> {
    match unresolvable {
        Some(_) => Ok(None),
        None => reached_verdict(project, slice, range).map(Some),
    }
}

/// The verdict for a slice the layout placed: whether a commit in `range`
/// changed one of the documents the committed phases were derived from
/// ([`watched_documents`], [`document_changed`]), as a [`Restart`].
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
fn reached_verdict(project: &Project, slice: &str, range: &str) -> Result<Restart, String> {
    let documents = watched_documents(project, slice)?;
    Ok(restart_of(document_changed(project, range, &documents)?))
}

/// The documents a branch's committed phases were derived from: the slice's
/// `lld.md` ([`lld_path`](cargo_lid_rs::layout::lld_path)) and the HLDs the
/// workspace's intent index names
/// ([`hlds`](cargo_lid_rs::coach::hlds) of
/// [`intent_paths`](cargo_lid_rs::coach::intent_paths)).
///
/// Both are asked of the crate that already answers them rather than spelled
/// here, so that this crate holds no second account of where a document
/// lives.
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
fn watched_documents(project: &Project, slice: &str) -> Result<Vec<PathBuf>, String> {
    todo!("collect the documents `{slice}` was derived from in {:?}", project.root())
}

/// Which of `documents` a commit in `range` changed, and none where no commit
/// in it changed any of them.
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
fn document_changed(project: &Project, range: &str, documents: &[PathBuf]) -> Result<Option<PathBuf>, String> {
    todo!("find which of {documents:?} a commit in {range} changed in {:?}", project.root())
}

/// What a changed document means: a restart at Phase 2 naming it, and a
/// resume where none changed.
#[implements(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
fn restart_of(changed: Option<PathBuf>) -> Restart {
    match changed {
        Some(document) => Restart::AtPhaseTwo { document },
        None => Restart::Resume,
    }
}

/// The findings the branch's unreadable phase subjects raise: one against
/// each [`PhaseCommit`] whose subject opens [`PHASE_PREFIX`] and names no
/// phase with a check, and none for a branch every one of whose phase
/// subjects names one.
#[implements(spec::AMalformedPhaseSubjectIsAFindingAgainstItsCommit)]
fn malformed_findings(commits: &[PhaseCommit]) -> Vec<Finding> {
    todo!("raise a finding against each unreadable subject among {} commit(s)", commits.len())
}

/// The findings the branch's [`TOOL_TRAILER`] trailers raise: one naming each
/// [`PhaseCommit`] carrying that trailer with a value other than `version`,
/// so that a resume across a tool change is visible.
///
/// The version is handed in rather than read, so that the mismatch is a case
/// a test constructs from plain data rather than one only a differently-built
/// binary could produce.
#[implements(spec::AToolTrailerTheBinaryDoesNotMatchIsAFinding)]
fn tool_findings(commits: &[PhaseCommit], version: &str) -> Vec<Finding> {
    todo!("raise a finding for each {TOOL_TRAILER} among {} commit(s) that is not {version}", commits.len())
}

/// The finding an unresolvable slice raises: one naming `slice` and carrying
/// the layout's refusal, and none where `unresolvable` is absent because a
/// workspace member holds the slice's document.
#[implements(spec::AnUnresolvableSliceIsAFindingBesideTheStateAndNotARefusal)]
fn unresolvable_findings(slice: &str, unresolvable: Option<&str>) -> Vec<Finding> {
    todo!("raise the finding `{slice}` earns from {unresolvable:?}")
}

/// Writes a reading's report to `target/lid/status.json` under `root`,
/// answering with the path it wrote, and creating the directory where the
/// project has none.
///
/// It writes whatever the reading found: an empty finding list is written too,
/// so that a report holding no finding and no report at all are different
/// things to whoever reads the file next.
#[implements(spec::TheReportIsWrittenToStatusJsonWhateverItFound)]
pub fn write_report(root: &Path, report: &Report) -> Result<PathBuf, String> {
    todo!("write the state of `{}` and its {} finding(s) under {}", report.state.slice, report.findings.len(), root.display())
}

/// The human rendering of a reading: the state a reader reads on stdout, and
/// every finding the report holds beside it.
///
/// It is given the report and nothing to read the repository with, so the text
/// and the JSON cannot disagree about what was read. Both halves of the pair
/// the report carries are rendered, in the order the report holds them: the
/// state, then every finding.
#[implements(
    spec::TheRenderingIsBuiltFromTheReportAndNeverFromASecondReading,
    spec::TheRenderingNamesEveryFindingTheReportHolds,
)]
pub fn rendering(report: &Report) -> String {
    format!("{}\n{}", rendered_state(&report.state), rendered_findings(&report.findings))
}

/// The state a reader reads: the slice, the phase last committed here, the
/// phase a run would take up next, the restart verdict, every phase commit
/// the branch made with its trailers, and the work the tree has not
/// committed.
///
/// Given the state and nothing to read the repository with, for the reason
/// [`rendering`] is: what a reader is shown and what `status.json` holds are
/// two renderings of one value.
#[implements(spec::TheRenderingIsBuiltFromTheReportAndNeverFromASecondReading)]
fn rendered_state(state: &Status) -> String {
    todo!("render the state of `{}`", state.slice)
}

/// Every finding the report holds, one a line, and nothing at all for a
/// report that holds none — the ordinary case, where a reading found the
/// branch as it expected to.
#[implements(spec::TheRenderingNamesEveryFindingTheReportHolds)]
fn rendered_findings(findings: &[Finding]) -> String {
    todo!("render {} finding(s)", findings.len())
}

#[cfg(test)]
mod intent_graph {
    //! This crate's instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}
