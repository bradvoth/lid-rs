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

#[cfg(test)]
mod tests {
    //! What these validations observe, and where each of them observes it.
    //!
    //! Most of the reading's items take plain data — a subject, a log, a slice
    //! of [`PhaseCommit`]s, a tool version — so most of these tests construct
    //! the case rather than a repository. The rest need one, because what they
    //! observe is a range of commits, a document that moved, a tree holding
    //! uncommitted work, or a slice no workspace member holds a document for;
    //! each of those builds one with [`repository`].
    //!
    //! **Three claims are observed through a composition and not through an
    //! item that names them**, because Phase 4 wrote that item's body and a
    //! test aimed at it would answer correctly before anything is implemented:
    //!
    //! | Claim | Observed through | The written body it would otherwise reach |
    //! |---|---|---|
    //! | [`APhaseIsTheNumberOfItsSubjectsPhasePrefix`](spec::APhaseIsTheNumberOfItsSubjectsPhasePrefix) | [`phase_commits`] | [`checked_phase`], [`phase_record`] |
    //! | [`ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo`](spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo) | [`reached_verdict`] | [`restart_of`], [`since_newest_phase`] |
    //! | [`TheReportCarriesTheStateAndTheFindingsTheReadingRaised`](spec::TheReportCarriesTheStateAndTheFindingsTheReadingRaised) | [`status`] | a [`Report`] built in the test |
    //!
    //! [`AnUnresolvableSliceLeavesTheRestartVerdictUnanswered`](spec::AnUnresolvableSliceLeavesTheRestartVerdictUnanswered)
    //! is that hazard in its sharpest form: [`restart_verdict`] answers
    //! `Ok(None)` for an unresolvable slice without reaching any unimplemented
    //! item at all, so it is observed through [`status`] on a branch naming a
    //! slice no member holds a document for — where the slice must be resolved
    //! before any verdict is reached.
    //!
    //! **The branch point is asserted deliberately.**
    //! [`ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint`](spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint)
    //! and [`ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase`](spec::ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase)
    //! share [`branch_log`], and one test of the range would satisfy both
    //! while telling neither apart. The first is asserted over [`status`], of
    //! the commit the branch left its base at and the phase it does *not* take
    //! from the base; the second over [`branch_log`], of the range itself.
    //!
    //! **No expectation is read back from the code under test.** The commit a
    //! fixture's branch left its base at is read from git and never from
    //! [`fork_point`](cargo_lid_rs::headless_canopy_agent::fork_point); the
    //! phases and trailers a log is expected to yield are the ones its records
    //! were written with; and the tool version a mismatch is judged against is
    //! handed to [`tool_findings`] as a parameter, which is what that
    //! parameter is for — a mismatch is a case a test constructs, not one only
    //! a differently-built binary produces.

    use std::process::Command;

    use lid_rs::validates;

    use super::*;

    /// The manifest of the fixture workspace: one member, `app`.
    const WORKSPACE_MANIFEST: &str = "[workspace]\nresolver = \"2\"\nmembers = [\"app\"]\n";

    /// The manifest of that member, which depends on nothing.
    const MEMBER_MANIFEST: &str = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    /// The fixture slice's document, which is what makes `app` the member that
    /// holds a document for the slice `hello` — and so what makes `hello`
    /// resolvable and every other name not.
    const SLICE_DOCUMENT: &str = "# hello\n\nThe hello slice.\n";

    /// A fresh scratch directory, canonical as the paths `cargo metadata`
    /// reports are.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-pipeline-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir.canonicalize().expect("scratch dir")
    }

    /// Runs git in `dir` under an identity of its own, asserting it succeeded.
    fn git(dir: &Path, args: &[&str]) {
        let ran = Command::new("git")
            .args(["-c", "user.email=pipeline@tests", "-c", "user.name=pipeline tests"])
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git");
        assert!(ran.success(), "git {args:?} in {}", dir.display());
    }

    /// Stages everything and commits it under `subject`, empty commits and all.
    fn commit_all(dir: &Path, subject: &str) {
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "--allow-empty", "-m", subject]);
    }

    /// The fixture's current `HEAD`, read from git.
    fn head_of(dir: &Path) -> String {
        let shown = Command::new("git").args(["rev-parse", "HEAD"]).current_dir(dir).output().expect("git rev-parse");
        String::from_utf8_lossy(&shown.stdout).trim().to_string()
    }

    /// A workspace whose one member holds the slice `hello`'s document, with
    /// two phase commits on `main` and `subjects` committed after it on
    /// `lld/hello`, oldest first.
    ///
    /// The commits on `main` are the ancestry a branch cut after a merge
    /// carries and the range it does not (pipeline `§4.2`): a reading that
    /// took the whole ancestry would answer phase 3 for every branch built
    /// here, whatever the branch itself committed.
    ///
    /// Answers the directory, the project, and the commit the branch left its
    /// base at — read from git, so that what the fixture pins is the
    /// repository rather than this crate's reading of it.
    fn repository(name: &str, subjects: &[&str]) -> (PathBuf, Project, String) {
        let dir = scratch(name);
        std::fs::write(dir.join("Cargo.toml"), WORKSPACE_MANIFEST).expect("the workspace manifest");
        std::fs::write(dir.join(".gitignore"), "/target\nCargo.lock\n").expect("the ignore file");
        std::fs::create_dir_all(dir.join("app/src/hello")).expect("the member's source");
        std::fs::write(dir.join("app/Cargo.toml"), MEMBER_MANIFEST).expect("the member's manifest");
        std::fs::write(dir.join("app/src/lib.rs"), "").expect("the member's library");
        std::fs::write(dir.join("app/src/hello/lld.md"), SLICE_DOCUMENT).expect("the slice's document");
        git(&dir, &["init", "-q", "-b", "main"]);
        commit_all(&dir, "phase 1: LLD for hello");
        commit_all(&dir, "phase 3: skeleton for hello");
        let base = head_of(&dir);
        git(&dir, &["checkout", "-q", "-b", "lld/hello"]);
        for subject in subjects {
            git(&dir, &["commit", "-q", "--allow-empty", "-m", subject]);
        }
        let project = Project::load_at(&dir.join("Cargo.toml")).expect("cargo metadata");
        (dir, project, base)
    }

    /// One record of a [`LOG_FORMAT`] log, as `git log` writes it: the commit,
    /// its subject, its trailer block, and the separator that ends it.
    fn record(commit: &str, subject: &str, trailers: &str) -> String {
        format!("{commit}\n{subject}\n{trailers}{RECORD_SEPARATOR}")
    }

    /// One `Name: value` trailer.
    fn trailer(name: &str, value: &str) -> Trailer {
        Trailer { name: name.to_string(), value: value.to_string() }
    }

    /// One phase commit as the reading holds it.
    fn commit_of(commit: &str, subject: &str, phase: Option<Phase>, trailers: Vec<Trailer>) -> PhaseCommit {
        PhaseCommit { commit: commit.to_string(), subject: subject.to_string(), phase, trailers }
    }

    /// A finding of the reading's own shape, carrying `message`.
    fn finding(message: &str) -> Finding {
        Finding {
            check: 0,
            rule: None,
            severity: "warning".to_string(),
            file: None,
            line: None,
            item: None,
            claim: None,
            message: message.to_string(),
            fix: None,
            source: "status".to_string(),
        }
    }

    /// A report of a branch at phase 4 carrying `findings`, built here and
    /// backed by no repository — so that a rendering agreeing with it is a
    /// rendering built from the report.
    fn report_of(findings: Vec<Finding>) -> Report {
        let state = Status {
            slice: "hello".to_string(),
            branch_point: "b0b0b0b".to_string(),
            phase: Some(Phase::Four),
            next: Some(Phase::Five),
            restart: Some(Restart::Resume),
            commits: vec![commit_of("c1", "phase 4: descend for hello", Some(Phase::Four), vec![trailer("Lid-Rs-Phase", "4")])],
            uncommitted: Vec::new(),
        };
        Report { state, findings }
    }

    /// Whether a finding names `needle` anywhere in what it carries.
    fn names(raised: &[Finding], needle: &str) -> bool {
        raised.iter().any(|found| format!("{found:?}").contains(needle))
    }

    /// The phase the state names is the newest phase commit's, and not an
    /// older commit's — including where the newest phase subject names a phase
    /// with no check of its own.
    #[test]
    #[validates(spec::TheStateIsTheNewestPhaseCommitTheBranchMade)]
    fn the_state_is_the_newest_phase_commit_the_branch_made() {
        let descended = commit_of("c3", "phase 4: descend for hello", Some(Phase::Four), Vec::new());
        let skeleton = commit_of("c2", "phase 3: skeleton for hello", Some(Phase::Three), Vec::new());
        let leaves = commit_of("c1", "phase 6: leaves for hello", None, Vec::new());
        let newest_first = [descended, skeleton.clone()];
        let over_an_uncheckable = [leaves, skeleton];
        assert_eq!(
            (newest_phase(&newest_first), newest_phase(&over_an_uncheckable)),
            (Some(Phase::Four), Some(Phase::Three)),
            "the newest of the branch's phase commits that names a phase",
        );
    }

    /// The range read is the branch's own commits, so the phase subject its
    /// base also reaches is no part of the reading.
    #[test]
    #[validates(spec::ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase)]
    fn a_commit_the_branchs_base_also_reaches_is_not_this_branchs_phase() {
        let (dir, project, base) = repository("base-reaches", &["phase 4: descend for hello"]);
        let own = head_of(&dir);
        let log = branch_log(&project, &own_commits(&base)).expect("the branch's own commits");
        assert_eq!(
            (log.contains(&own), log.contains(&base)),
            (true, false),
            "the commit the branch made, and not the `phase 3:` commit its base reaches",
        );
    }

    /// A record's phase is its subject's `phase <N>:` number, and none where
    /// the subject carries no such prefix — observed over a whole log, since
    /// the item that reads one subject was written at Phase 4.
    #[test]
    #[validates(spec::APhaseIsTheNumberOfItsSubjectsPhasePrefix)]
    fn a_phase_is_the_number_of_its_subjects_phase_prefix() {
        let log = [
            record("c3", "phase 4: descend for hello", "Lid-Rs-Phase: 4\n"),
            record("c2", "chore: no business of this reading", ""),
            record("c1", "phase foo: a subject naming no number", ""),
        ]
        .concat();
        let read = phase_commits(&log);
        let phases: Vec<Option<Phase>> = read.iter().map(|found| found.phase).collect();
        assert_eq!(
            (read.len(), phases),
            (2, vec![Some(Phase::Four), None]),
            "the number of each `phase ` subject's prefix, and no record for the subject that opens with none",
        );
    }

    /// The phase next is the first the branch holds no commit for, and not the
    /// one after the newest: a branch holding 2, 3 and 5 is taken up at 4.
    #[test]
    #[validates(spec::ThePhaseNextIsTheFirstOneTheBranchHasNoCommitFor)]
    fn the_phase_next_is_the_first_one_the_branch_has_no_commit_for() {
        let five = commit_of("c3", "phase 5: failing tests (red) for hello", Some(Phase::Five), Vec::new());
        let three = commit_of("c2", "phase 3: skeleton for hello", Some(Phase::Three), Vec::new());
        let two = commit_of("c1", "phase 2: claims for hello", Some(Phase::Two), Vec::new());
        let with_a_gap = [five, three.clone(), two.clone()];
        let in_order = [three, two];
        assert_eq!(
            (next_phase(&with_a_gap), next_phase(&in_order)),
            (Some(Phase::Four), Some(Phase::Four)),
            "the first phase a run builds that the branch holds no commit for",
        );
    }

    /// A branch that committed no phase of its own is answered with the commit
    /// it left its base at, and with no phase — not with the base's.
    #[test]
    #[validates(spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint)]
    fn a_branch_with_no_phase_commit_of_its_own_is_answered_with_its_branch_point() {
        let (_dir, project, base) = repository("no-phase-commit", &["chore: a note", "docs: another note"]);
        let report = status(&project, "lld/hello", None).expect("a reading of a branch that committed no phase");
        assert_eq!(
            (report.state.branch_point, report.state.phase),
            (base, None),
            "the commit the branch left its base at, and no phase read from the base",
        );
    }

    /// Every phase commit is reported with its own trailers, in the order its
    /// message carries them — not the newest commit's alone.
    #[test]
    #[validates(spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers)]
    fn every_phase_commit_on_the_branch_is_reported_with_its_trailers() {
        let log = [
            record("c2", "phase 4: descend for hello", "Lid-Rs-Phase: 4\nLid-Rs-Agent: lid-rs-phase-4\n"),
            record("c1", "phase 3: skeleton for hello", "Lid-Rs-Phase: 3\n"),
        ]
        .concat();
        let carried: Vec<Vec<Trailer>> = phase_commits(&log).into_iter().map(|found| found.trailers).collect();
        assert_eq!(
            carried,
            vec![
                vec![trailer("Lid-Rs-Phase", "4"), trailer("Lid-Rs-Agent", "lid-rs-phase-4")],
                vec![trailer("Lid-Rs-Phase", "3")],
            ],
            "each phase commit's trailers, with the older commit's kept",
        );
    }

    /// A document changed after the newest phase commit restarts the pipeline
    /// at Phase 2, naming that document.
    #[test]
    #[validates(spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo)]
    fn a_document_changed_after_the_newest_phase_commit_restarts_at_phase_two() {
        let (dir, project, _base) = repository("document-changed", &["phase 4: descend for hello"]);
        let newest = head_of(&dir);
        std::fs::write(dir.join("app/src/hello/lld.md"), "# hello\n\nThe hello slice, amended.\n").expect("the amended document");
        commit_all(&dir, "docs: amend the LLD");
        let verdict = reached_verdict(&project, "hello", &own_commits(&newest)).expect("a verdict for a slice the layout placed");
        assert_eq!(
            verdict,
            Restart::AtPhaseTwo { document: dir.join("app/src/hello/lld.md") },
            "the restart the moved document earns, naming it",
        );
    }

    /// A tree holding uncommitted work is read and reported, not refused.
    #[test]
    #[validates(spec::ADirtyWorkingTreeIsReportedAndNotRefused)]
    fn a_dirty_working_tree_is_reported_and_not_refused() {
        let (dir, project, _base) = repository("dirty-tree", &["phase 4: descend for hello"]);
        std::fs::write(dir.join("app/src/lib.rs"), "// work the tree has not committed\n").expect("the uncommitted work");
        let reported = uncommitted(&project).expect("a reading of a tree that holds uncommitted work");
        assert_eq!(reported, vec!["app/src/lib.rs".to_string()], "the path the work is in, reported rather than refused");
    }

    /// A subject that opens `phase ` and names no number a phase can be read
    /// from raises a finding against that commit, and the readable subject
    /// beside it raises none.
    #[test]
    #[validates(spec::AMalformedPhaseSubjectIsAFindingAgainstItsCommit)]
    fn a_malformed_phase_subject_is_a_finding_against_its_commit() {
        let commits = [
            commit_of("c2", "phase foo: a subject naming no number", None, Vec::new()),
            commit_of("c1", "phase 4: descend for hello", Some(Phase::Four), Vec::new()),
        ];
        let raised = malformed_findings(&commits);
        assert_eq!(
            (raised.len(), names(&raised, "c2")),
            (1, true),
            "one finding, against the commit whose subject the reading cannot read",
        );
    }

    /// A `Lid-Rs-Tool` trailer whose value is not the running binary's version
    /// raises a finding naming that commit; one that matches raises none.
    #[test]
    #[validates(spec::AToolTrailerTheBinaryDoesNotMatchIsAFinding)]
    fn a_tool_trailer_the_binary_does_not_match_is_a_finding() {
        let commits = [
            commit_of("c2", "phase 4: descend for hello", Some(Phase::Four), vec![trailer(TOOL_TRAILER, "9.9.9")]),
            commit_of("c1", "phase 3: skeleton for hello", Some(Phase::Three), vec![trailer(TOOL_TRAILER, "0.1.0")]),
        ];
        let raised = tool_findings(&commits, "0.1.0");
        assert_eq!(
            (raised.len(), names(&raised, "c2")),
            (1, true),
            "one finding, naming the commit a tool of another version made",
        );
    }

    /// A branch naming a slice no member holds a document for is read, and the
    /// slice is a finding beside the state rather than a refusal.
    #[test]
    #[validates(spec::AnUnresolvableSliceIsAFindingBesideTheStateAndNotARefusal)]
    fn an_unresolvable_slice_is_a_finding_beside_the_state_and_not_a_refusal() {
        let (_dir, project, _base) = repository("unresolvable-finding", &["phase 4: descend for ghost"]);
        let report = status(&project, "lld/ghost", None).expect("a reading, not a refusal");
        assert!(names(&report.findings, "ghost"), "the finding names the slice no member holds a document for: {report:?}");
    }

    /// That reading answers no restart verdict: the `lld.md` the verdict is
    /// made against is a document no member holds.
    #[test]
    #[validates(spec::AnUnresolvableSliceLeavesTheRestartVerdictUnanswered)]
    fn an_unresolvable_slice_leaves_the_restart_verdict_unanswered() {
        let (_dir, project, _base) = repository("unresolvable-verdict", &["phase 4: descend for ghost"]);
        let report = status(&project, "lld/ghost", None).expect("a reading, not a refusal");
        assert_eq!(report.state.restart, None, "no verdict at all, rather than the resume a missing document cannot earn");
    }

    /// A branch that is not `lld/<slice>` is refused naming the convention —
    /// and only when no slice is given beside it.
    #[test]
    #[validates(spec::ABranchThatNamesNoSliceIsRefusedNamingTheConvention)]
    fn a_branch_that_names_no_slice_is_refused_naming_the_convention() {
        let beside_it = slice_named("feature/status", Some("hello")).expect("the slice given beside the branch");
        let refusal = slice_named("feature/status", None).expect_err("a branch naming no slice, with none given");
        assert_eq!(
            (beside_it.as_str(), refusal.contains("feature/status"), refusal.contains("lld/")),
            ("hello", true, true),
            "the branch named, and the convention it does not follow",
        );
    }

    /// The report is the pair: the branch's state, and the findings the
    /// reading raised — the empty list where it raised none.
    #[test]
    #[validates(spec::TheReportCarriesTheStateAndTheFindingsTheReadingRaised)]
    fn the_report_carries_the_state_and_the_findings_the_reading_raised() {
        let (_dir, project, _base) = repository("the-pair", &["phase 4: descend for hello"]);
        let report = status(&project, "lld/hello", None).expect("a reading of the branch");
        assert_eq!(
            (report.state.slice.as_str(), report.state.phase, report.findings.is_empty()),
            ("hello", Some(Phase::Four), true),
            "the state beside the finding list a reading that raised nothing carries",
        );
    }

    /// The report reaches `target/lid/status.json` under the root, and the
    /// empty finding list — the half of "whatever it found" that a write can
    /// silently suppress — does not stop it being written.
    ///
    /// The expected path is spelled here rather than asked of the code that
    /// writes it: a test that derived its expectation from the writer would
    /// hold for a writer that chose any path at all. This is how the catalog's
    /// own tests pin the same directory (`catalog/mod.rs`, `written_report`),
    /// and it settles neither of the LLD's Unresolved rows — what is asserted
    /// is a file at a path, not a route to it.
    #[test]
    #[validates(spec::TheReportIsWrittenToStatusJsonWhateverItFound)]
    fn the_report_is_written_to_status_json_whatever_it_found() {
        let root = scratch("written-report");
        let expected = root.join("target/lid/status.json");
        let written = write_report(&root, &report_of(Vec::new())).expect("the path it wrote");
        assert_eq!(
            (written.as_path(), expected.is_file()),
            (expected.as_path(), true),
            "the file a reading that raised no finding still writes",
        );
    }

    /// The rendering carries what the report holds, including a branch point
    /// and a slice no repository the run could read would answer.
    #[test]
    #[validates(spec::TheRenderingIsBuiltFromTheReportAndNeverFromASecondReading)]
    fn the_rendering_is_built_from_the_report_and_never_from_a_second_reading() {
        let text = rendering(&report_of(Vec::new()));
        assert_eq!(
            (text.contains("hello"), text.contains("b0b0b0b")),
            (true, true),
            "the state the report carries, which no repository backs: {text}",
        );
    }

    /// Every finding the report holds is named in the rendering.
    #[test]
    #[validates(spec::TheRenderingNamesEveryFindingTheReportHolds)]
    fn the_rendering_names_every_finding_the_report_holds() {
        let held = vec![finding("a subject naming no number"), finding("a tool of another version")];
        let text = rendering(&report_of(held));
        assert_eq!(
            (text.contains("a subject naming no number"), text.contains("a tool of another version")),
            (true, true),
            "both findings, named: {text}",
        );
    }
}
