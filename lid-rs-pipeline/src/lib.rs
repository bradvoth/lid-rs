#![doc = include_str!("../docs/intent/pipeline.md")]
#![doc = include_str!("lld.md")]

pub mod spec;

use std::path::{Path, PathBuf};

use cargo_lid_rs::catalog::Finding;
use cargo_lid_rs::phase::Phase;
use cargo_lid_rs::project::Project;
use lid_rs::implements;

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
    /// The phase the subject's `phase <N>:` prefix names
    /// ([`tag_of`](cargo_lid_rs::phase::tag_of)), and none where the subject
    /// carries no prefix a phase can be read from.
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
/// [`own_commits`](cargo_lid_rs::headless_canopy_agent::own_commits) names and
/// [`log_subjects`](cargo_lid_rs::headless_canopy_agent::log_subjects) reads —
/// and never from the whole ancestry, in which a merged slice leaves every one
/// of its phase subjects. The phase a run would take up next is the first of
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
#[implements(
    spec::TheStateIsTheNewestPhaseCommitTheBranchMade,
    spec::ACommitTheBranchsBaseAlsoReachesIsNotThisBranchsPhase,
    spec::APhaseIsTheNumberOfItsSubjectsPhasePrefix,
    spec::ThePhaseNextIsTheFirstOneTheBranchHasNoCommitFor,
    spec::ABranchWithNoPhaseCommitOfItsOwnIsAnsweredWithItsBranchPoint,
    spec::EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers,
    spec::ADocumentChangedAfterTheNewestPhaseCommitRestartsAtPhaseTwo,
    spec::ADirtyWorkingTreeIsReportedAndNotRefused,
    spec::AMalformedPhaseSubjectIsAFindingAgainstItsCommit,
    spec::AToolTrailerTheBinaryDoesNotMatchIsAFinding,
    spec::AnUnresolvableSliceIsAFindingBesideTheStateAndNotARefusal,
    spec::AnUnresolvableSliceLeavesTheRestartVerdictUnanswered,
    spec::ABranchThatNamesNoSliceIsRefusedNamingTheConvention,
    spec::TheReportCarriesTheStateAndTheFindingsTheReadingRaised,
)]
pub fn status(project: &Project, branch: &str, slice: Option<&str>) -> Result<Report, String> {
    todo!("read the state of `{branch}` for the slice {slice:?} in {:?}", project.root())
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
/// and the JSON cannot disagree about what was read.
#[implements(
    spec::TheRenderingIsBuiltFromTheReportAndNeverFromASecondReading,
    spec::TheRenderingNamesEveryFindingTheReportHolds,
)]
pub fn rendering(report: &Report) -> String {
    todo!("render the state of `{}` and its {} finding(s)", report.state.slice, report.findings.len())
}

#[cfg(test)]
mod intent_graph {
    //! This crate's instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}
