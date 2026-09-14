//! The detached gate job (`lld.md` § "The gate runs detached"): the key one
//! attempt's result is told from another's by, the record a checkout keeps for
//! a slice, the poll a Phase 7 stop asks instead of holding a call open for the
//! length of the whole plan, and the read-only door an orchestrating session
//! reads instead of a worker's word about what the gate decided.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lid_rs::implements;
use sha2::{Digest, Sha256};

use super::policy::SliceCrates;
use super::spec;
use super::{Phase, Step, bump_workspace_version, integrity, mutation_base, policy};
use crate::mutants;
use crate::project::Project;

/// The gate's own space under a build directory: where every keyed path and
/// every checkout's record hangs.
const GATE_SPACE: &str = "lid-rs/gate";

/// The marker `execute_with`'s own `format!("{step:?} failed: {e}")` writes
/// between a step's name and the failure it carries — the boundary
/// [`bounded_output`] keeps its header up to.
const FAILURE_MARKER: &str = " failed: ";

/// How much of a failing step's own output the bounded stand-in keeps: larger
/// than any single clippy block or survivor summary, and orders of magnitude
/// smaller than the capture that killed a worker.
const TAIL_BYTES: usize = 8 * 1024;

/// What the bounded stand-in's closing line says before the result file's own
/// path, for whatever the bound cut off.
const WHOLE_CAPTURE_AT: &str = "the whole of this step's output is at ";

/// The hex SHA-256 of some bytes — the one digest this module names a
/// directory, a checkout, or a tree by.
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// One file written, its directory made first: what both the running record
/// and the detached child's own outcome reach the disk through.
fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// What a poll must match before it trusts any record: one attempt's identity.
/// A mismatch on any one field is the same as no record, so a result computed
/// for one tree is never read as if it applied to another.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::TheKeyCarriesTheCheckoutTheTipTheBaseTheSliceAndTheFingerprint)]
pub struct JobKey {
    /// The checkout's own canonical path: two worktrees sharing one `target/`
    /// are two checkouts, and differ here.
    pub checkout: PathBuf,
    /// The branch tip's hash, as `tip` reads it.
    pub head: String,
    /// The diff base the mutation step takes, `mutation_base`'s answer.
    pub base: String,
    /// The slice the job gates.
    pub slice: String,
    /// The fingerprint of the phase's own editing set, which a working-tree
    /// edit moves and `HEAD` does not.
    pub fingerprint: String,
}

impl JobKey {
    /// The one name the key's scratch space is addressed by: the directory
    /// `paths_for` and `result_path` derive, so a record and the job it names
    /// always agree without either of them storing a path.
    #[implements(spec::TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone)]
    pub fn hash(&self) -> String {
        digest(format!("{}\0{}\0{}\0{}\0{}", self.checkout.display(), self.head, self.base, self.slice, self.fingerprint).as_bytes())
    }
}

/// The one record a checkout holds for a slice. It only ever names a job as
/// running: a finished outcome is `result_path`'s file, never this one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobRecord {
    /// A job that was started and not yet known to have ended.
    Running {
        /// The key it runs under.
        key: JobKey,
        /// The detached child's process id, which `alive` asks about.
        pid: u32,
        /// The workspace root's `Cargo.toml` and `Cargo.lock` as the bump left
        /// them, each with its bytes, read back by [`running_record`] — what a
        /// later stop holds the tree to rather than recomputing, which
        /// `Cargo.lock` cannot support.
        version_files: Vec<(PathBuf, Vec<u8>)>,
    },
}

/// The whole of what a poll can mean when it does not fail outright: a job
/// just spawned, one already alive under this key, or a finished outcome read
/// back. A live job under a *different* key is no `GatePoll` at all — it is the
/// `Err` `contest` returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatePoll {
    /// A job was started for this key by this call.
    Started,
    /// A job started for this key by an earlier call is still alive.
    Running,
    /// The whole plan finished and wrote this outcome: the pass, or the first
    /// failing step's captured output.
    Done(Result<(), String>),
}

/// What the read-only door can answer. Not `GatePoll` minus `Started`: a live
/// job under another key is something a poll refuses outright, while a status
/// read must name it — there is something to wait for, just not under this key.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::AStatusOverALiveForeignJobCarriesTheOtherKey)]
pub enum GateStatus {
    /// Nothing is running for this slice in this checkout.
    NoJob,
    /// A job under this key is alive with no outcome written.
    Running,
    /// A job under another key is alive, which is named here.
    Foreign(JobKey),
    /// The outcome a finished job wrote.
    Done(Result<(), String>),
}

/// The key a poll holds a record to, computed fresh: the checkout's canonical
/// path, the branch tip's hash, `mutation_base`'s answer, the slice's name, and
/// `fingerprint`'s answer over the entries `policy::workspace_paths` admits.
///
/// It takes the phase and the crates because the editing set is what it
/// fingerprints, and nothing narrower than those two resolves it.
#[implements(
    spec::TheKeyCarriesTheCheckoutTheTipTheBaseTheSliceAndTheFingerprint,
    spec::TheKeyFingerprintsThePhasesOwnEditingSetTakingThePhaseAndTheCrates,
)]
pub fn key(project: &Project, phase: Phase, slice: &str, crates: &SliceCrates) -> Result<JobKey, String> {
    let checkout = project.root()?;
    Ok(JobKey {
        checkout: checkout.canonicalize().map_err(|e| format!("canonicalising {}: {e}", checkout.display()))?,
        head: super::tip(project)?.ok_or("this checkout has no commit, so a job has no tip to key to")?.hash,
        base: super::mutation_base(project)?,
        slice: slice.to_string(),
        fingerprint: fingerprint(project, &policy::workspace_paths(project, phase, crates)?)?,
    })
}

/// The fingerprint of a phase's editing set: every regular file under each
/// admitted entry — a directory walked recursively, a file read once — with its
/// path relative to the project root and a hash of its bytes, sorted by path
/// and hashed as one.
///
/// The walk is over what is on disk, so a leaf the phase has just written and
/// git does not yet track is part of the answer.
#[implements(
    spec::TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath,
    spec::AnUntrackedFileUnderAnAdmittedDirectoryChangesTheFingerprint,
)]
pub fn fingerprint(project: &Project, admitted: &[PathBuf]) -> Result<String, String> {
    let root = project.root()?;
    let mut found = BTreeMap::new();
    admitted.iter().try_for_each(|entry| walk(&root, &root.join(entry), &mut found))?;
    Ok(digest(found.iter().map(|(path, hash)| format!("{}\0{hash}\n", path.display())).collect::<String>().as_bytes()))
}

/// One admitted entry's regular files, each added to `found` under its path
/// relative to the project root: one decision over what lies at the entry — a
/// directory is walked recursively, a regular file is read once, and anything
/// else, a path with nothing at it among them, holds no regular file to add.
#[implements(spec::TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath)]
fn walk(root: &Path, entry: &Path, found: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    match std::fs::metadata(entry) {
        Ok(what) if what.is_dir() => walk_dir(root, entry, found),
        Ok(what) if what.is_file() => hash_file(root, entry, found),
        Ok(_) | Err(_) => Ok(()),
    }
}

/// Every entry of one directory, walked in turn — the recursion the admitted
/// directories are read by.
#[implements(spec::TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath)]
fn walk_dir(root: &Path, dir: &Path, found: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
    let entries = read.collect::<Result<Vec<_>, _>>().map_err(|e| format!("reading {}: {e}", dir.display()))?;
    entries.iter().try_for_each(|entry| walk(root, &entry.path(), found))
}

/// One regular file's bytes, hashed under its path relative to the project
/// root — relative, so the same tree at another root fingerprints the same and
/// the checkout stays the key's own field.
#[implements(spec::TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath, spec::AnUntrackedFileUnderAnAdmittedDirectoryChangesTheFingerprint)]
fn hash_file(root: &Path, file: &Path, found: &mut BTreeMap<PathBuf, String>) -> Result<(), String> {
    let relative = file.strip_prefix(root).map_err(|e| format!("{} is not under {}: {e}", file.display(), root.display()))?;
    let bytes = std::fs::read(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    found.insert(relative.to_path_buf(), digest(&bytes));
    Ok(())
}

/// The keyed scratch paths a detached job hands the mutation step in place of
/// the fixed defaults: the diff file and the output root, in that order, under
/// the key's own directory. Derived, so two jobs never share either path and a
/// stale job's cleanup cannot delete a fresh job's output.
///
/// It takes the project and answers a `Result` because the directory those
/// paths hang under is the build directory, which a `JobKey` does not carry: a
/// key names the checkout, and `CARGO_TARGET_DIR` need not put the build
/// directory anywhere near it. The lookup is `project.target_directory()`, the
/// same door `record_path` and `tally::path` already resolve theirs through,
/// and it fails the same way that lookup can.
#[implements(spec::TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone)]
pub fn paths_for(project: &Project, key: &JobKey) -> Result<(PathBuf, PathBuf), String> {
    let home = gate_home(project, key)?;
    Ok((home.join("diff"), home.join("mutants")))
}

/// The build directory's own gate space, which every keyed path and every
/// checkout's record hangs under: the one lookup through
/// `project.target_directory()` those three share.
fn gate_space(project: &Project) -> Result<PathBuf, String> {
    Ok(project.target_directory()?.join(GATE_SPACE))
}

/// One key's own directory in that space, named by the key's hash alone — so
/// a record and the job it names agree without either of them storing a path.
#[implements(spec::TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone)]
fn gate_home(project: &Project, key: &JobKey) -> Result<PathBuf, String> {
    Ok(gate_space(project)?.join(key.hash()))
}

/// The same keyed pair [`paths_for`] derives, answered only once the key's own
/// directory is on disk: what `keyed_mutants` calls in place of the formula, so
/// the diff file and the output root already have somewhere to be written the
/// moment [`mutants::run_at`] receives them.
///
/// It is a separate door and not a line added to [`paths_for`] because neither
/// `mutants::run_at`'s own diff write nor the mutation engine's output root
/// creates the directory a path it is handed lands in: each writes through the
/// path as given, which is right for a leaf whose whole claim is that a
/// caller's path is the one it writes to and nothing about making it. So
/// [`paths_for`] stays pure and touches no disk — its formula is assertable
/// against a project fixture with no directory ever made — and this is the one
/// that has the effect.
#[implements(spec::TheMutationStepsKeyedHomeExistsBeforeItWritesThere)]
pub fn prepared_paths(project: &Project, key: &JobKey) -> Result<(PathBuf, PathBuf), String> {
    let home = gate_home(project, key)?;
    std::fs::create_dir_all(&home).map_err(|e| format!("creating {}: {e}", home.display()))?;
    paths_for(project, key)
}

/// The file the detached child writes once, when the whole plan ends: a
/// sibling of the mutation step's own outcome under the same keyed directory,
/// holding the plan's `Done` as data.
///
/// It takes the project and answers a `Result` for the same reason
/// [`paths_for`] does: the keyed directory hangs under the build directory,
/// which the key does not name.
#[implements(spec::TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone)]
pub fn result_path(project: &Project, key: &JobKey) -> Result<PathBuf, String> {
    Ok(gate_home(project, key)?.join("result.json"))
}

/// Where a checkout keeps the running record for a slice: nested under the
/// checkout's own canonicalised path, hashed, and named for the slice — the way
/// the tally's own files are nested by agent.
///
/// Two worktrees on one slice therefore hold two files and neither can
/// overwrite the other's, which is what lets each read back the result its own
/// job wrote.
#[implements(
    spec::TheRunningRecordIsNestedUnderTheCheckoutAndNamedForTheSlice,
    spec::TwoCheckoutsOfOneSliceReadAndWriteOnlyTheirOwnRecord,
)]
pub fn record_path(project: &Project, slice: &str) -> Result<PathBuf, String> {
    let root = project.root()?;
    let checkout = root.canonicalize().map_err(|e| format!("canonicalising {}: {e}", root.display()))?;
    Ok(gate_space(project)?.join(digest(checkout.display().to_string().as_bytes())).join(format!("{slice}.json")))
}

/// One file read: the running record this checkout holds for the slice, or
/// none when it holds none.
pub fn read_record(project: &Project, slice: &str) -> Result<Option<JobRecord>, String> {
    std::fs::read_to_string(record_path(project, slice)?).ok().map(|json| record_from_json(&json)).transpose()
}

/// A record as stored, following the tally's own written/read precedent: the
/// key's five fields, the pid, and the bump's two files each with its bytes.
fn record_to_json(record: &JobRecord) -> String {
    let JobRecord::Running { key, pid, version_files } = record;
    let files: Vec<serde_json::Value> =
        version_files.iter().map(|(path, bytes)| serde_json::json!({ "path": path.display().to_string(), "bytes": bytes })).collect();
    serde_json::json!({ "key": key_to_json(key), "pid": pid, "version_files": files }).to_string()
}

/// A stored record, read back whole; a document missing any part of it is a
/// failure naming what it looked for.
fn record_from_json(json: &str) -> Result<JobRecord, String> {
    let doc: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("parsing a gate job record: {e}"))?;
    let stored = doc.pointer("/key").ok_or("a stored gate job record carries no `key`")?;
    let pid = doc.pointer("/pid").and_then(serde_json::Value::as_u64).ok_or("a stored gate job record carries no `pid`")?;
    let files = doc.pointer("/version_files").and_then(serde_json::Value::as_array).ok_or("a stored gate job record carries no `version_files`")?;
    Ok(JobRecord::Running {
        key: key_from_json(stored)?,
        pid: u32::try_from(pid).map_err(|e| format!("a stored gate job record's `pid` is no process id: {e}"))?,
        version_files: files.iter().map(stored_file).collect::<Result<Vec<_>, String>>()?,
    })
}

/// A key as stored: its five fields, the checkout's path as text.
fn key_to_json(key: &JobKey) -> serde_json::Value {
    serde_json::json!({
        "checkout": key.checkout.display().to_string(),
        "head": key.head,
        "base": key.base,
        "slice": key.slice,
        "fingerprint": key.fingerprint,
    })
}

/// A stored key; a document missing one of the five is a failure naming it.
fn key_from_json(stored: &serde_json::Value) -> Result<JobKey, String> {
    let field = |name: &str| {
        stored
            .pointer(&format!("/{name}"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("a stored job key carries no `{name}`"))
    };
    Ok(JobKey {
        checkout: PathBuf::from(field("checkout")?),
        head: field("head")?,
        base: field("base")?,
        slice: field("slice")?,
        fingerprint: field("fingerprint")?,
    })
}

/// One stored `version_files` entry: the path it names and the bytes it holds.
fn stored_file(stored: &serde_json::Value) -> Result<(PathBuf, Vec<u8>), String> {
    let path = stored.pointer("/path").and_then(serde_json::Value::as_str).ok_or("a stored version file carries no `path`")?;
    let stored_bytes = stored.pointer("/bytes").and_then(serde_json::Value::as_array).ok_or("a stored version file carries no `bytes`")?;
    let bytes = stored_bytes
        .iter()
        .map(|byte| byte.as_u64().and_then(|n| u8::try_from(n).ok()).ok_or_else(|| format!("a stored version file holds `{byte}`, which is no byte")))
        .collect::<Result<Vec<u8>, String>>()?;
    Ok((PathBuf::from(path), bytes))
}

/// The detached child's own command line, built and not run: `phase-check <n>
/// --slice <slice> --job-key <the key's hash>` appended to whatever
/// `self_command()` already carries.
///
/// Which program runs a subcommand in a fresh process, and what precedes those
/// six arguments on its line, is `self_command()`'s decision and left
/// unexamined here — an installed binary and a checkout answer it differently
/// without either changing which job this child is being asked to run. It takes
/// no project: nothing in the line it answers with depends on the filesystem or
/// the build directory, which is what makes it a fact about a string, true or
/// false with nothing running.
#[implements(spec::TheChildsLineEndsWithPhaseCheckTheSliceAndTheJobKey)]
pub fn child_command(phase: Phase, slice: &str, key: &JobKey) -> Result<std::process::Command, String> {
    let mut command = super::self_command()?;
    command.args(["phase-check", &policy::number_of(phase).to_string(), "--slice", slice, "--job-key", &key.hash()]);
    Ok(command)
}

/// The `Running` record as a value, built from exactly what it is given: the
/// key and the pid unchanged, and `integrity::contents_of`'s answer for the two
/// root files the bump writes as its `version_files`, in that call's own order.
///
/// It starts no child and asserts nothing about one, so what a record carries
/// is answerable with nothing running — the same kind of fact about a value
/// that [`child_command`]'s line is about a string. The bytes are read back
/// here rather than recomputed at whichever later stop reads `Done`, because
/// `Cargo.lock`'s cannot be derived from `HEAD` without rewriting it.
#[implements(spec::TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder)]
pub fn running_record(project: &Project, key: &JobKey, pid: u32) -> Result<JobRecord, String> {
    Ok(JobRecord::Running {
        key: key.clone(),
        pid,
        version_files: integrity::contents_of(project, &policy::hook_written_paths(Phase::Seven))?,
    })
}

/// One file write: the record it is handed, to this checkout's own record path
/// for the slice.
///
/// It takes a record rather than building one, so a caller — `start`, or a
/// fixture with a record it built by hand — hands it whichever record is to be
/// written, exactly as `act` is handed one it never ran a job to produce.
#[implements(spec::AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord)]
pub fn write_record(project: &Project, slice: &str, record: &JobRecord) -> Result<(), String> {
    write_file(&record_path(project, slice)?, record_to_json(record).as_bytes())
}

/// The one place a job begins, as five moves and no decision: the workspace
/// version and its lock bumped once, so the child gates the bumped tree; the
/// line [`child_command`] answers with, spawned unwaited, so the child outlives
/// this process; and the record [`running_record`] makes of that key and that
/// child's pid, handed to [`write_record`] before returning.
///
/// Spawning a line, and leaving it running, are facts about a process — which
/// is why the two halves that are not are asked of leaves that need none.
#[implements(spec::AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord)]
pub fn start(project: &Project, phase: Phase, slice: &str, key: &JobKey) -> Result<(), String> {
    bump_workspace_version(project)?;
    let child = child_command(phase, slice, key)?.spawn().map_err(|e| format!("spawning the gate job for {slice}: {e}"))?;
    let record = running_record(project, key, child.id())?;
    write_record(project, slice, &record)
}

/// Whether a process is still running, asked of the host through `kill -0`.
///
/// Fallible, never folded into "dead": a host that cannot answer is not a host
/// whose job has ended, and "dead" is the answer that starts a whole gate run
/// again.
#[implements(spec::AlivesOwnFailureFailsThePollAndNeverRestartsTheJob)]
pub fn alive(pid: u32) -> Result<bool, String> {
    liveness_of(pid, kill_zero(pid)?)
}

/// `kill -0 <pid>`, its own output discarded: the exit status the host answers
/// the question with, or the failure of a host that has no `kill` to ask. A
/// bare `Command` and no new dependency, the posture `extra_step` already
/// keeps.
fn kill_zero(pid: u32) -> Result<std::process::ExitStatus, String> {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("asking the host whether {pid} is running: {e}"))
}

/// What that exit status means, as one decision: nought is a process still
/// running, one is a process that is not, and any other status is a host that
/// could not be asked — which is never folded into "dead", since dead is the
/// answer that starts a whole gate run again.
#[implements(spec::AlivesOwnFailureFailsThePollAndNeverRestartsTheJob)]
fn liveness_of(pid: u32, status: std::process::ExitStatus) -> Result<bool, String> {
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        Some(_) | None => Err(format!("the host could not say whether {pid} is running: `kill -0 {pid}` answered with {status}")),
    }
}

/// One file read: the outcome at this key's own result path, or none while the
/// child has written none. The outcome is read back as data, so no reader
/// hunts for a marker in captured text.
///
/// It takes the project because the file it reads is [`result_path`]'s, which
/// lives under the build directory a key does not carry.
#[implements(spec::TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText)]
pub fn finished(project: &Project, key: &JobKey) -> Result<Option<Result<(), String>>, String> {
    std::fs::read_to_string(result_path(project, key)?).ok().map(|json| outcome_from_json(&json)).transpose()
}

/// The plan's one outcome as stored: the failing step's own capture under
/// `failure`, and nothing there for a pass.
fn outcome_to_json(outcome: &Result<(), String>) -> String {
    serde_json::json!({ "failure": outcome.as_ref().err() }).to_string()
}

/// A stored outcome, as one decision over the data and not over a marker in
/// text: a capture recorded there is the failing step's, and its absence is
/// the pass.
#[implements(spec::TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText)]
fn outcome_from_json(json: &str) -> Result<Result<(), String>, String> {
    let doc: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("parsing a gate job outcome: {e}"))?;
    match doc.pointer("/failure").and_then(serde_json::Value::as_str) {
        Some(capture) => Ok(Err(capture.to_string())),
        None => Ok(Ok(())),
    }
}

/// The one outcome a detached child writes, once, when the whole plan ends:
/// to this key's own result path, where a later poll reads it back as data.
#[implements(spec::TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText)]
fn write_outcome(project: &Project, key: &JobKey, outcome: &Result<(), String>) -> Result<(), String> {
    write_file(&result_path(project, key)?, outcome_to_json(outcome).as_bytes())
}

/// What a Phase 7 stop asks in place of a blocking check: the key computed
/// fresh and the running record read, both handed to `act`. It never sleeps —
/// a caller wanting a later answer calls again, at the cost of one file read.
#[implements(spec::AKeyDifferingInAnyOnePartIsNeverThisPollsDone)]
pub fn poll(project: &Project, phase: Phase, slice: &str, crates: &SliceCrates) -> Result<GatePoll, String> {
    let fresh = key(project, phase, slice, crates)?;
    act(project, phase, slice, &fresh, read_record(project, slice)?)
}

/// The one decision a poll makes, over what it already read: no record starts a
/// fresh job; a record under this key is `settle`'s; a record under another key
/// is `contest`'s.
#[implements(spec::AKeyDifferingInAnyOnePartIsNeverThisPollsDone)]
pub fn act(project: &Project, phase: Phase, slice: &str, key: &JobKey, record: Option<JobRecord>) -> Result<GatePoll, String> {
    match record {
        None => start(project, phase, slice, key).map(|()| GatePoll::Started),
        Some(JobRecord::Running { key: recorded, pid, .. }) if recorded == *key => settle(project, phase, slice, key, pid),
        Some(JobRecord::Running { key: other, pid, .. }) => contest(project, phase, slice, key, pid, &other),
    }
}

/// One decision, for a record under *this* key: a written outcome answers
/// `Done` before anything asks whether the pid is still alive, and only when
/// none is written does liveness decide.
#[implements(spec::SettleReadsTheWrittenResultBeforeItAsksWhetherThePidIsAlive)]
pub fn settle(project: &Project, phase: Phase, slice: &str, key: &JobKey, pid: u32) -> Result<GatePoll, String> {
    match finished(project, key)? {
        Some(outcome) => Ok(GatePoll::Done(outcome)),
        None => restart_or_running(project, phase, slice, key, pid),
    }
}

/// One decision, reached only once `settle` finds no outcome written: an alive
/// pid answers `Running`; a dead one died before finishing, and a fresh job
/// starts under that same key.
#[implements(spec::RestartOrRunningAnswersRunningForALivePidAndRestartsUnderTheSameKeyForADeadOne)]
pub fn restart_or_running(project: &Project, phase: Phase, slice: &str, key: &JobKey, pid: u32) -> Result<GatePoll, String> {
    if alive(pid)? {
        Ok(GatePoll::Running)
    } else {
        start(project, phase, slice, key).map(|()| GatePoll::Started)
    }
}

/// One decision, for a record under a *different* key: an alive pid is refused
/// outright, naming the other key — neither waited for nor restarted, since
/// waiting is the blocking call this design removes and restarting throws away
/// a run that may be about to pass; a dead one is stale and blocks nothing.
#[implements(
    spec::ALiveForeignRecordRefusesTheStartNamingTheOtherKey,
    spec::ADeadForeignRecordStartsAFreshJobUnderTheCurrentKey,
)]
pub fn contest(project: &Project, phase: Phase, slice: &str, key: &JobKey, pid: u32, other: &JobKey) -> Result<GatePoll, String> {
    if alive(pid)? {
        Err(format!("a gate job is already running in this checkout under another key, as process {pid}: {other:?}"))
    } else {
        start(project, phase, slice, key).map(|()| GatePoll::Started)
    }
}

/// The read-only sibling of `poll`: the same three-way split over the fresh key
/// and the record, mapped to `GateStatus` instead of to a spawn. It reaches
/// `start` on no branch, so a caller that only wants to know starts nothing and
/// is refused nothing.
#[implements(spec::TheStatusDoorSharesActsThreeWaySplitAndReachesStartOnNoBranch)]
pub fn status(project: &Project, phase: Phase, slice: &str, crates: &SliceCrates) -> Result<GateStatus, String> {
    let fresh = key(project, phase, slice, crates)?;
    match read_record(project, slice)? {
        None => Ok(GateStatus::NoJob),
        Some(JobRecord::Running { key: recorded, pid, .. }) if recorded == fresh => matching_status(project, &fresh, pid),
        Some(JobRecord::Running { key: other, pid, .. }) => foreign_status(&other, pid),
    }
}

/// The status-only sibling of `settle` and `restart_or_running`, for a record
/// under *this* key: a written outcome answers `Done`, an alive pid with none
/// written answers `Running`, and a dead one answers `NoJob` — nothing is
/// running, and a poll would start afresh here rather than find anything to
/// report.
///
/// It takes the project for the same reason `finished` does: what it reads
/// lives under the build directory, which a bare key does not carry. The
/// liveness half is `running_or_no_job`'s, so that the result-before-liveness
/// order is the one decision made here — the same split `settle` and
/// `restart_or_running` already are on the poll's side.
#[implements(spec::MatchingStatusAnswersNoJobWhereAPollWouldRestart)]
pub fn matching_status(project: &Project, key: &JobKey, pid: u32) -> Result<GateStatus, String> {
    match finished(project, key)? {
        Some(outcome) => Ok(GateStatus::Done(outcome)),
        None => running_or_no_job(pid),
    }
}

/// What a record under this key means once no outcome is written: `Running`
/// while the pid lives, and `NoJob` once it is dead — where
/// `restart_or_running` would start a fresh job, a status read reports that
/// nothing is running rather than making it so.
#[implements(spec::MatchingStatusAnswersNoJobWhereAPollWouldRestart)]
fn running_or_no_job(pid: u32) -> Result<GateStatus, String> {
    if alive(pid)? { Ok(GateStatus::Running) } else { Ok(GateStatus::NoJob) }
}

/// The status-only sibling of `contest`, for a record under a *different* key:
/// an alive pid answers `Foreign`, naming it, since there is something to wait
/// for even though a start under this key would be refused over it; a dead one
/// answers `NoJob`, the record being stale.
#[implements(
    spec::ForeignStatusAnswersForeignWhileThatPidLivesAndNoJobOnceItIsDead,
    spec::AStatusOverALiveForeignJobCarriesTheOtherKey,
)]
pub fn foreign_status(other: &JobKey, pid: u32) -> Result<GateStatus, String> {
    if alive(pid)? { Ok(GateStatus::Foreign(other.clone())) } else { Ok(GateStatus::NoJob) }
}

/// What `phase::run` calls in place of `check` when `--job-key` is given: the
/// key this child is about to gate under, recomputed here and then held to the
/// hash the flag carried.
///
/// It takes the hash and never a `JobKey`, because nothing reconstructs a key
/// from a one-way digest. So it takes exactly what a bare `phase-check <n>
/// --slice <slice>` already gives it, resolves `crates` the way any other
/// invocation resolves them, and calls the same `key` door `start` called to
/// compute the key it spawned this child under. That is a check and not a
/// transport: a flag wide enough to carry the key's fields verbatim would
/// still have to be trusted, and trusting a value handed across a process
/// boundary is what this design refuses everywhere else.
#[implements(spec::TheJobRunsUnderAKeyItRecomputedAndHeldToTheFlagsHash)]
pub fn run_job(project: &Project, phase: Phase, slice: &str, job_key: &str) -> Result<(), String> {
    let crates = SliceCrates::resolve(project, slice)?.map_err(|refusal| refusal.reason)?;
    let recomputed = key(project, phase, slice, &crates)?;
    run_or_abandon(project, phase, slice, &recomputed, job_key)
}

/// The one decision the detached child makes, before any step of the plan
/// runs: a fresh key whose hash is the one it was spawned under gates the tree
/// it was asked to gate, and anything else is a tree that moved under the job
/// between the spawn and this call.
///
/// A mismatch runs no step and writes to no result path at all — not the given
/// hash's, since this call never held the key that hash names and filing a
/// result there is the one thing recomputing exists to rule out, and not its
/// own fresh key's, since nothing polls a key this call was never asked to run
/// under. The running record still names the original key and this child's
/// pid, so once the child exits the pid answers dead and the ordinary
/// died-before-finishing recovery starts a fresh job under whatever key the
/// tree computes then. From the outside a mismatch looks like a job that never
/// got the chance to run, which is what it is.
#[implements(
    spec::TheJobRunsUnderAKeyItRecomputedAndHeldToTheFlagsHash,
    spec::AKeyMismatchInTheChildRunsNoStepAndWritesNoResult,
)]
fn run_or_abandon(project: &Project, phase: Phase, slice: &str, key: &JobKey, job_key: &str) -> Result<(), String> {
    let hash = key.hash();
    if hash == job_key {
        run_plan(project, phase, slice, key)
    } else {
        Err(format!("this checkout no longer keys to the job it was spawned for: --job-key named {job_key}, the tree now keys to {hash}"))
    }
}

/// The plan itself, run under a key the child has already proved is the one it
/// was spawned under: the same steps `plan` returns, through a runner of its
/// own — every step but the mutation step delegated straight to `run_step`,
/// unchanged, and the mutation step alone diverted to `mutants::run_at` with
/// this key's own scratch paths in place of the fixed defaults — and then its
/// one outcome, written once to this key's result path before the child exits.
#[implements(
    spec::TheJobsRunnerDivertsTheMutationStepAloneToTheKeyedPaths,
    spec::TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText,
)]
fn run_plan(project: &Project, phase: Phase, slice: &str, key: &JobKey) -> Result<(), String> {
    let steps = super::plan(phase, &project.publishing_members(), &policy::gate_extra(project)?);
    let outcome = super::execute_with(&steps, |step| job_step(project, slice, key, step));
    write_outcome(project, key, &outcome).and(outcome)
}

/// The runner `run_plan` hands `execute_with`, and the one decision it holds:
/// the mutation step alone is diverted to this key's own scratch paths, and
/// every other step is delegated straight to `run_step`, unchanged — which is
/// what keeps `run_step` free of a branch nothing claims.
#[implements(spec::TheJobsRunnerDivertsTheMutationStepAloneToTheKeyedPaths)]
fn job_step(project: &Project, slice: &str, key: &JobKey, step: &Step) -> Result<(), String> {
    match step {
        Step::Mutants => keyed_mutants(project, key),
        Step::Check
        | Step::Clippy
        | Step::Doc
        | Step::DocTests
        | Step::LibTests
        | Step::Package(_)
        | Step::SyncCheck
        | Step::Extra(_)
        | Step::Red
        | Step::LldChecks => super::run_step(project, Some(slice), step),
    }
}

/// The mutation step inside a detached job: diverted through
/// [`prepared_paths`] for this key's own diff file and output root —
/// [`paths_for`]'s pair, with the directory they lie in already made —
/// before handing them to [`mutants::run_at`] in place of the fixed defaults
/// `mutants::run` resolves for an ordinary step, so a stale job's cleanup
/// cannot delete a fresh job's still-being-written output. The outcome stays
/// `run_plan`'s to write once; this leaf returns it rather than writing
/// anything itself.
#[implements(spec::TheJobsRunnerDivertsTheMutationStepAloneToTheKeyedPaths)]
fn keyed_mutants(project: &Project, key: &JobKey) -> Result<(), String> {
    let (diff_path, output_root) = prepared_paths(project, key)?;
    mutants::run_at(project, &["--diff-base".to_string(), mutation_base(project)?], &diff_path, &output_root)
}

/// The stand-in a refusal carries for a detached job's whole capture: the
/// failing step's own header, up to and including the `" failed: "` the
/// execution's format string wrote, then the last 8 KiB of the failure itself,
/// then a line naming the result file for whatever the bound cut off.
///
/// The tail and not the head, because every marker the refusal's own reading
/// looks for — a denied lint's name, the red run's and the mutation step's own
/// words — is written where the failing tool reports its verdict, which is the
/// last thing it writes.
/// It takes the project and answers a `Result` for one reason: the closing
/// line names [`result_path`]'s own path, and that resolution reads the build
/// directory through the project and can fail.
#[implements(
    spec::TheBoundedStandInCarriesTheFailingHeaderTheLastTailAndThePath,
    spec::TheBoundedStandInStillNamesTheCheckThatFired,
)]
pub fn bounded_output(project: &Project, key: &JobKey, output: &str) -> Result<String, String> {
    let boundary = output.find(FAILURE_MARKER).map_or(0, |at| at + FAILURE_MARKER.len());
    let path = result_path(project, key)?;
    Ok(format!("{}{}\n{WHOLE_CAPTURE_AT}{}\n", &output[..boundary], last_bytes(&output[boundary..]), path.display()))
}

/// The last [`TAIL_BYTES`] of a string, cut back to a character boundary — the
/// tail, because the verdict a failing tool reports is the last thing it
/// writes and the bound must not cut it off.
fn last_bytes(text: &str) -> &str {
    let start = text.len().saturating_sub(TAIL_BYTES);
    &text[(start..text.len()).find(|at| text.is_char_boundary(*at)).unwrap_or(text.len())..]
}

#[cfg(test)]
mod tests {
    //! What a job's key, its record, its poll and its own runner must answer.
    //!
    //! Two things shape every case here, and neither is a process. A record is
    //! a value `act`, `settle`, `contest` and their status siblings are
    //! *handed*, and `running_record` builds and `write_record` writes, so a
    //! fixture holds one without a job ever having run. And `start` is the one
    //! item that spawns: every branch that ends there is driven in a checkout
    //! whose bump — its first move — cannot succeed, which proves the branch
    //! was taken and pays for no child. What that leaves unobserved is that a
    //! child ran at all and that nothing waited for it, which the design
    //! records as a residue rather than claiming.

    use std::path::Path;

    use lid_rs::validates;

    use super::*;
    use crate::phase::ending::{Check, check_of_output, refusal_for};
    use crate::phase::integrity::contents_of;
    use crate::phase::policy::{hook_written_paths, workspace_paths};
    use crate::phase::{execute, fixture, mutation_base, plan, tip};

    /// This crate as a project: `--no-deps` metadata answers the root, the
    /// build directory and git, which is every question a key asks.
    fn this_project() -> Project {
        Project::load_at(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).expect("cargo metadata")
    }

    /// The `phase` slice's crates: this one, with no companion.
    fn phase_crates() -> SliceCrates {
        SliceCrates { slice: "phase".to_string(), own: PathBuf::from(env!("CARGO_MANIFEST_DIR")), companion: None }
    }

    /// A project rooted where the caller says, keeping its build directory
    /// where the caller says — the two things a keyed path hangs under, and
    /// nothing a manifest would have to hold.
    fn project_at(root: &Path, target: &Path) -> Project {
        Project::from_json(&format!(r#"{{"workspace_root":"{}","target_directory":"{}","packages":[]}}"#, root.display(), target.display()))
            .expect("parses")
    }

    /// A checkout no job can begin in: a repository whose root manifest holds
    /// no `[workspace.package]` table, so `start`'s own first move — the bump —
    /// fails and no child is ever spawned. What a case drives here observes is
    /// that the branch reached `start` at all.
    fn a_checkout_no_job_can_begin_in(name: &str) -> Project {
        let root = fixture::scratch(name);
        std::fs::write(root.join("Cargo.toml"), "[package]\nname = \"a\"\nversion = \"0.1.0\"\n").expect("manifest");
        fixture::git(&root, &["init", "-q"]);
        fixture::git(&root, &["add", "-A"]);
        fixture::git(&root, &["commit", "-q", "-m", "a manifest the bump cannot read"]);
        project_at(&root, &root.join("target"))
    }

    /// What the bump names when it cannot read a manifest — the failure that
    /// tells a branch reaching `start` from one that answered without it.
    const NO_BUMP_HERE: &str = "[workspace.package]";

    /// A key whose every field is spelled out, so a case can change one.
    fn a_key(fingerprint: &str) -> JobKey {
        JobKey {
            checkout: PathBuf::from("/one/checkout"),
            head: "a tip".to_string(),
            base: "a base".to_string(),
            slice: "hello".to_string(),
            fingerprint: fingerprint.to_string(),
        }
    }

    /// The key with one field changed, once for each of the five it carries.
    fn differing_in_one_part(key: &JobKey) -> Vec<JobKey> {
        vec![
            JobKey { checkout: key.checkout.join("elsewhere"), ..key.clone() },
            JobKey { head: format!("{} moved", key.head), ..key.clone() },
            JobKey { base: format!("{} moved", key.base), ..key.clone() },
            JobKey { slice: format!("{}-other", key.slice), ..key.clone() },
            JobKey { fingerprint: format!("{} edited", key.fingerprint), ..key.clone() },
        ]
    }

    /// The pid of a process that has run and been reaped: dead, and not handed
    /// to anything else while this test runs.
    fn a_dead_pid() -> u32 {
        let mut child = std::process::Command::new("git")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("git");
        let pid = child.id();
        child.wait().expect("git exits");
        pid
    }

    /// A pid wider than any the host has: the question itself is ill-formed,
    /// so `alive` cannot ask it — which is not the answer that a process is
    /// not running.
    const UNASKABLE_PID: u32 = u32::MAX;

    /// Copies a tree, as the fixture's own copy does.
    fn copy_tree(from: &Path, to: &Path) {
        let status = std::process::Command::new("cp")
            .args(["-R", &format!("{}/.", from.display()), &to.display().to_string()])
            .status()
            .expect("cp");
        assert!(status.success(), "cp -R {} {}", from.display(), to.display());
    }

    /// The fixture's slice, as a phase resolves it.
    fn hello_crates(dir: &Path) -> SliceCrates {
        SliceCrates { slice: "hello".to_string(), own: dir.to_path_buf(), companion: None }
    }

    /// A key whose job has already finished, and the outcome it wrote: the
    /// Phase 1 plan, run here through `run_job` — the child's own door — over
    /// an LLD that cannot pass its mechanical checks, so the whole plan ends
    /// in the time a file read takes and no cargo step is ever reached.
    fn a_finished_job(dir: &Path, project: &Project) -> JobKey {
        std::fs::write(dir.join("docs/intent/hello/lld.md"), "").expect("an LLD the checks reject");
        let job = key(project, Phase::One, "hello", &hello_crates(dir)).expect("the key");
        let _ = run_job(project, Phase::One, "hello", &job.hash());
        assert!(finished(project, &job).expect("read").is_some(), "the child wrote its outcome before exiting");
        job
    }

    #[test]
    #[validates(spec::TheKeyCarriesTheCheckoutTheTipTheBaseTheSliceAndTheFingerprint)]
    fn the_key_carries_the_checkout_the_tip_the_base_the_slice_and_the_fingerprint() {
        let project = this_project();
        let crates = phase_crates();
        let job = key(&project, Phase::Seven, "phase", &crates).expect("the key");
        assert_eq!(job.checkout, project.root().expect("the root").canonicalize().expect("canonical"), "the checkout's own canonical path");
        assert_eq!(job.head, tip(&project).expect("git").expect("a commit").hash, "the branch tip's hash");
        assert_eq!(job.base, mutation_base(&project).expect("git"), "the base the mutation step takes");
        assert_eq!(job.slice, "phase", "the slice the job gates");
        let admitted = workspace_paths(&project, Phase::Seven, &crates).expect("the editing set");
        assert_eq!(job.fingerprint, fingerprint(&project, &admitted).expect("the fingerprint"), "and the fingerprint of that editing set");
    }

    #[test]
    #[validates(spec::TheKeyFingerprintsThePhasesOwnEditingSetTakingThePhaseAndTheCrates)]
    fn the_key_fingerprints_the_phases_own_editing_set_taking_the_phase_and_the_crates() {
        let project = this_project();
        let crates = phase_crates();
        // Phase 3 may write the library root beside the slice's own code and
        // Phase 7 may not: one tree, two editing sets, two keys.
        let at_three = key(&project, Phase::Three, "phase", &crates).expect("the key");
        let at_seven = key(&project, Phase::Seven, "phase", &crates).expect("the key");
        let three = workspace_paths(&project, Phase::Three, &crates).expect("the editing set");
        let seven = workspace_paths(&project, Phase::Seven, &crates).expect("the editing set");
        assert_ne!(three, seven, "the two phases' editing sets differ, which is what this asks about");
        assert_eq!(at_three.fingerprint, fingerprint(&project, &three).expect("the fingerprint"), "each is `workspace_paths`' own answer");
        assert_eq!(at_seven.fingerprint, fingerprint(&project, &seven).expect("the fingerprint"));
        assert_ne!(at_three.fingerprint, at_seven.fingerprint, "so one tree holds to two keys");
    }

    #[test]
    #[validates(spec::TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath)]
    fn the_fingerprint_hashes_every_file_under_the_admitted_entries_sorted_by_path() {
        let root = fixture::scratch("fingerprint-files");
        let project = project_at(&root, &root.join("target"));
        std::fs::create_dir_all(root.join("src/deep")).expect("dirs");
        std::fs::write(root.join("src/a.rs"), "a").expect("write");
        std::fs::write(root.join("src/deep/b.rs"), "b").expect("write");
        std::fs::write(root.join("module.rs"), "m").expect("write");
        std::fs::write(root.join("outside.rs"), "o").expect("write");
        let admitted = [PathBuf::from("src"), PathBuf::from("module.rs")];
        let first = fingerprint(&project, &admitted).expect("the fingerprint");
        assert_eq!(first, fingerprint(&project, &admitted).expect("again"), "one tree hashes the same twice");
        let reordered = [PathBuf::from("module.rs"), PathBuf::from("src"), PathBuf::from("src/nothing-is-here")];
        assert_eq!(first, fingerprint(&project, &reordered).expect("the fingerprint"), "sorted by path, and an entry with no file under it adds nothing");
        std::fs::write(root.join("outside.rs"), "changed").expect("write");
        assert_eq!(first, fingerprint(&project, &admitted).expect("the fingerprint"), "a file under no admitted entry is no part of it");
        std::fs::write(root.join("src/deep/b.rs"), "changed").expect("write");
        let edited = fingerprint(&project, &admitted).expect("the fingerprint");
        assert_ne!(first, edited, "a byte under an admitted directory is");
        std::fs::rename(root.join("src/deep/b.rs"), root.join("src/deep/c.rs")).expect("rename");
        assert_ne!(edited, fingerprint(&project, &admitted).expect("the fingerprint"), "and so is the path it lies at, the same bytes under another name");
        // The paths hashed are relative to the project root: the same tree at
        // another root is the same fingerprint, and the checkout is the key's
        // own field rather than something the fingerprint smuggles in.
        let twin = fixture::scratch("fingerprint-twin");
        copy_tree(&root, &twin);
        assert_eq!(
            fingerprint(&project_at(&twin, &twin.join("target")), &admitted).expect("the fingerprint"),
            fingerprint(&project, &admitted).expect("the fingerprint"),
            "relative to the project root, never to the host"
        );
        // `walk`'s own third case: an entry that is on disk but is neither a
        // directory to recurse into nor a regular file to hash. A Unix domain
        // socket is exactly that — `is_dir()` and `is_file()` both false —
        // and it is what tells that case apart from the one just above it,
        // where a leaf is a file worth hashing. Reading it as a file would
        // fail outright (a socket cannot be opened for `read`), so the
        // assertion below is not merely "the digest is unchanged": a
        // `fingerprint` that tried to hash it would have returned `Err` here
        // instead of the same answer as before.
        //
        // `UnixListener` does not exist off Unix, so this block is Unix-only.
        // It is also rooted directly under `/tmp`, not under
        // `fixture::scratch`'s own directory: an `AF_UNIX` path is capped
        // around 104 bytes on macOS, and `std::env::temp_dir()`'s per-user
        // directory there is already close enough to that limit
        // (`/var/folders/<..>/T/lid-rs-phase-tests/<name>/...` routinely
        // runs past 70 bytes on its own) that nesting the socket under it
        // would make `bind` flaky rather than reliably short.
        #[cfg(unix)]
        {
            let socket_root = PathBuf::from("/tmp").join(format!("lid-rs-gate-job-socket-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&socket_root);
            std::fs::create_dir_all(socket_root.join("src")).expect("dirs");
            std::fs::write(socket_root.join("src/a.rs"), "a").expect("write");
            let socket_project = project_at(&socket_root, &socket_root.join("target"));
            let socket_admitted = [PathBuf::from("src")];
            let before_the_socket = fingerprint(&socket_project, &socket_admitted).expect("the fingerprint");
            let socket = socket_root.join("src/s");
            let listener = std::os::unix::net::UnixListener::bind(&socket).expect("bind a socket under an admitted directory");
            let kind = std::fs::metadata(&socket).expect("metadata");
            assert!(!kind.is_dir() && !kind.is_file(), "a socket is neither, which is the case this asks about: {kind:?}");
            assert_eq!(
                fingerprint(&socket_project, &socket_admitted).expect("walking it neither errors nor tries to hash it"),
                before_the_socket,
                "traversed as nothing and hashed as nothing, a socket under an admitted directory changes the fingerprint not at all"
            );
            drop(listener);
            let _ = std::fs::remove_dir_all(&socket_root);
        }
    }

    #[test]
    #[validates(spec::AnUntrackedFileUnderAnAdmittedDirectoryChangesTheFingerprint)]
    fn an_untracked_file_under_an_admitted_directory_changes_the_fingerprint() {
        let root = fixture::scratch("fingerprint-untracked");
        let project = project_at(&root, &root.join("target"));
        std::fs::create_dir_all(root.join("src")).expect("dirs");
        std::fs::write(root.join("src/a.rs"), "a").expect("write");
        fixture::git(&root, &["init", "-q"]);
        fixture::git(&root, &["add", "-A"]);
        fixture::git(&root, &["commit", "-q", "-m", "the tree git tracks"]);
        let admitted = [PathBuf::from("src")];
        let tracked = fingerprint(&project, &admitted).expect("the fingerprint");
        // The leaf a phase has just written and not staged: git's index does
        // not hold it, and the walk this reads is the disk's.
        std::fs::write(root.join("src/leaf.rs"), "what the phase wrote").expect("write");
        assert_ne!(tracked, fingerprint(&project, &admitted).expect("the fingerprint"), "an untracked file under an admitted directory is part of the key");
        std::fs::remove_file(root.join("src/leaf.rs")).expect("remove");
        assert_eq!(tracked, fingerprint(&project, &admitted).expect("the fingerprint"), "and the tree without it keys as it did before");
    }

    #[test]
    #[validates(spec::TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone)]
    fn the_jobs_scratch_paths_and_its_result_path_are_derived_from_the_key_alone() {
        let root = fixture::scratch("keyed-paths");
        let target = root.join("build");
        let project = project_at(&root, &target);
        let (first, same, other) = (a_key("one tree"), a_key("one tree"), a_key("another tree"));
        let (diff, output) = paths_for(&project, &first).expect("the scratch paths");
        let result = result_path(&project, &first).expect("the result path");
        let home = target.join("lid-rs/gate").join(first.hash());
        assert!(diff.starts_with(&home), "the diff file under the key's own directory: {diff:?}");
        assert!(output.starts_with(&home), "and the output root: {output:?}");
        assert!(result.starts_with(&home), "and the result: {result:?}");
        assert!(diff != output && result != diff && result != output, "three paths, not one: {diff:?} {output:?} {result:?}");
        assert_eq!(paths_for(&project, &same).expect("the scratch paths"), (diff, output), "an equal key derives equal paths, neither storing one");
        assert_eq!(result_path(&project, &same).expect("the result path"), result);
        assert_ne!(other.hash(), first.hash(), "a key differing in one field is another job");
        assert_ne!(result_path(&project, &other).expect("the result path"), result, "with a scratch space of its own");
    }

    #[test]
    #[validates(spec::TheMutationStepsKeyedHomeExistsBeforeItWritesThere)]
    fn the_mutation_steps_keyed_home_exists_before_it_writes_there() {
        // The same fixture the formula's own case uses — a project rooted in a
        // scratch directory, with its build directory under it — and a key
        // whose home nothing has made yet.
        let root = fixture::scratch("prepared-paths");
        let target = root.join("build");
        let project = project_at(&root, &target);
        let job = a_key("one tree");
        let home = target.join("lid-rs/gate").join(job.hash());
        assert!(!home.exists(), "nothing is on disk under the key's own directory before the call: {home:?}");
        let prepared = prepared_paths(&project, &job).expect("the prepared paths");
        // The pair answered is the one `paths_for` derives for this key, and
        // not some other pair that happens to lie under a directory this made.
        assert_eq!(prepared, paths_for(&project, &job).expect("the scratch paths"), "the pair `paths_for` derives for this key, unchanged");
        // And that key's own directory — the one the diff file lies in, which
        // is where the mutation engine's output root hangs too — is on disk by
        // the time the pair is answered, since neither `mutants::run_at`'s own
        // diff write nor the engine creates a directory for a path it is handed.
        let (diff, output) = prepared;
        let made = diff.parent().expect("the diff file lies in a directory");
        assert_eq!(made, home, "the key's own directory under the build directory's gate space: {diff:?}");
        let kind = std::fs::metadata(made).expect("the key's own directory is on disk by the time the pair is answered");
        assert!(kind.is_dir(), "and it is a directory, not a file left where one is about to be written: {kind:?}");
        assert!(output.starts_with(made), "which is the home the output root hangs under as well: {output:?}");
    }

    #[test]
    #[validates(spec::TheRunningRecordIsNestedUnderTheCheckoutAndNamedForTheSlice)]
    fn the_running_record_is_nested_under_the_checkout_and_named_for_the_slice() {
        // One build directory, two checkouts: what `CARGO_TARGET_DIR` need not
        // separate, and this project's own worktrees do not.
        let shared = fixture::scratch("record-build-directory");
        let (one, two) = (fixture::scratch("record-checkout-one"), fixture::scratch("record-checkout-two"));
        let (first, second) = (project_at(&one, &shared), project_at(&two, &shared));
        let (mine, theirs) = (record_path(&first, "hello").expect("the record"), record_path(&second, "hello").expect("the record"));
        let gate = shared.join("lid-rs/gate");
        assert!(mine.starts_with(&gate) && theirs.starts_with(&gate), "under the build directory's own gate space: {mine:?} {theirs:?}");
        assert!(mine.ends_with("hello.json") && theirs.ends_with("hello.json"), "named for the slice: {mine:?} {theirs:?}");
        assert_ne!(mine.parent(), theirs.parent(), "and nested by checkout, so neither can overwrite the other");
        assert_ne!(record_path(&first, "greet").expect("the record"), mine, "one checkout's two slices are two files as well");
    }

    #[test]
    #[validates(spec::TwoCheckoutsOfOneSliceReadAndWriteOnlyTheirOwnRecord)]
    fn two_checkouts_of_one_slice_read_and_write_only_their_own_record() {
        let shared = fixture::scratch("two-checkouts-build-directory");
        let (one, two) = (fixture::scratch("two-checkouts-one"), fixture::scratch("two-checkouts-two"));
        let (first, second) = (project_at(&one, &shared), project_at(&two, &shared));
        assert_eq!(read_record(&first, "hello").expect("read"), None, "neither checkout holds a record for the slice yet");
        assert_eq!(read_record(&second, "hello").expect("read"), None);
        // What the first checkout's poll wrote: the second reads its own file
        // and finds nothing, rather than reading or replacing this one.
        let mine = record_path(&first, "hello").expect("the record");
        std::fs::create_dir_all(mine.parent().expect("a directory to hold it")).expect("dirs");
        std::fs::write(&mine, "the first checkout's running record").expect("write");
        assert_eq!(read_record(&second, "hello").expect("read"), None, "the other checkout's record is not this one's to read");
        assert_eq!(
            std::fs::read_to_string(&mine).expect("read"),
            "the first checkout's running record",
            "and the first checkout's own is still there to read back"
        );
    }

    #[test]
    #[validates(spec::AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord)]
    fn a_job_begins_with_one_bump_an_unwaited_child_and_the_running_record() {
        // The bump is the first of the five moves: where it cannot run, the
        // job never begins — no line is built, nothing is spawned, and nothing
        // is recorded.
        let cannot_bump = a_checkout_no_job_can_begin_in("job-start-no-bump");
        let job = a_key("one tree");
        let refused = start(&cannot_bump, Phase::Seven, "hello", &job).expect_err("the bump cannot read this manifest");
        assert!(refused.contains(NO_BUMP_HERE), "the bump is what failed, and it ran before anything else: {refused}");
        assert_eq!(read_record(&cannot_bump, "hello").expect("read"), None, "a job that never began records nothing");
        // Once for that job, and never for a key whose job is already running:
        // a live pid answers `Running`, and the second bump — which in this
        // checkout could only fail — is never reached.
        let running = restart_or_running(&cannot_bump, Phase::Seven, "hello", &job, std::process::id()).expect("a job already running is not one to begin");
        assert_eq!(running, GatePoll::Running, "no bump is paid for a job already running");
        // Nor for a key already read as `Done`: the written outcome answers,
        // and a `start` in that checkout would have failed at the bump instead.
        let (dir, project) = fixture::copy("job-start-already-done");
        let settled = settle(&project, Phase::One, "hello", &a_finished_job(&dir, &project), a_dead_pid()).expect("the outcome already written answers");
        assert!(matches!(settled, GatePoll::Done(_)), "no bump is paid for a job already read as `Done`: {settled:?}");
        // And the last of the five: the record reaches the file before a
        // beginning returns. Asked of the leaf that writes it, which takes the
        // record rather than making one, so no job need have run to have one.
        let record = JobRecord::Running { key: job, pid: 4242, version_files: Vec::new() };
        write_record(&project, "hello", &record).expect("the record is written");
        assert_eq!(read_record(&project, "hello").expect("read"), Some(record), "read back from this checkout's own record for the slice");
        assert!(record_path(&project, "hello").expect("the record path").exists(), "which is the file it lies in");
    }

    #[test]
    #[validates(spec::TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder)]
    fn the_running_record_carries_the_key_the_pid_and_the_bumps_bytes_in_read_order() {
        // A checkout whose two root files are known bytes, and no job of any
        // kind near it: what a record carries is a fact about a value.
        let root = fixture::scratch("running-record");
        let project = project_at(&root, &root.join("target"));
        let (manifest, lock) = ("[workspace.package]\nversion = \"0.4.1\"\n", "# the lock the bump brought into agreement\n");
        std::fs::write(root.join("Cargo.toml"), manifest).expect("the manifest");
        std::fs::write(root.join("Cargo.lock"), lock).expect("the lock");
        let (job, pid) = (a_key("one tree"), 4242);
        let JobRecord::Running { key, pid: carried, version_files } = running_record(&project, &job, pid).expect("the record");
        assert_eq!(key, job, "the key it was given, unchanged");
        assert_eq!(carried, pid, "and the pid it was given");
        // The bump's two files as `contents_of` answers for them, in that
        // call's own order — the order a later stop holds the tree to.
        let named: Vec<PathBuf> = version_files.iter().map(|(path, _)| path.clone()).collect();
        assert_eq!(named, hook_written_paths(Phase::Seven), "`Cargo.toml` and `Cargo.lock`, in read order");
        assert_eq!(version_files[0].1, manifest.as_bytes(), "the manifest's own bytes, read back after the bump");
        assert_eq!(version_files[1].1, lock.as_bytes(), "and the lock's, which nothing can recompute from `HEAD`");
        assert_eq!(version_files, contents_of(&project, &hook_written_paths(Phase::Seven)).expect("the two files"), "which is `contents_of`'s answer entire");
    }

    #[test]
    #[validates(spec::TheChildsLineEndsWithPhaseCheckTheSliceAndTheJobKey)]
    fn the_childs_line_ends_with_phase_check_the_slice_and_the_job_key() {
        let job = a_key("one tree");
        let seven = child_command(Phase::Seven, "hello", &job).expect("the child's line");
        let args: Vec<String> = seven.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
        let last_six = ["phase-check", "7", "--slice", "hello", "--job-key", &job.hash()].map(String::from);
        assert!(args.ends_with(&last_six), "the six arguments the line ends with: {args:?}");
        // A suffix, and only a suffix: which program runs a subcommand in a
        // fresh process, and what it puts before these six, is `self_command()`'s
        // own decision — an installed binary and a checkout answer it
        // differently, and this leaf examines neither.
        let five = child_command(Phase::Five, "greet", &job).expect("the child's line");
        let other: Vec<String> = five.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
        let five_six = ["phase-check", "5", "--slice", "greet", "--job-key", &job.hash()].map(String::from);
        assert!(other.ends_with(&five_six), "the phase's own number and the slice it was asked for: {other:?}");
    }

    #[test]
    #[validates(spec::AKeyDifferingInAnyOnePartIsNeverThisPollsDone)]
    fn a_key_differing_in_any_one_part_is_never_this_polls_done() {
        let project = a_checkout_no_job_can_begin_in("act-foreign-key");
        let fresh = a_key("this tree");
        for other in differing_in_one_part(&fresh) {
            let record = JobRecord::Running { key: other.clone(), pid: std::process::id(), version_files: Vec::new() };
            let answer = act(&project, Phase::Seven, "hello", &fresh, Some(record));
            assert!(answer.is_err(), "the outcome written under another key is no `Done` of this poll's: {answer:?} for {other:?}");
        }
        // The other side of the same guard: a record whose key matches
        // exactly is `settle`'s, not `contest`'s, and a finished job's
        // outcome is `Done` — which a guard permanently `false` could never
        // answer, since every record would fall to `contest` regardless of
        // whether its key matched. The pid is one the host cannot be asked
        // about, the same device `settle`'s own test uses to show that a
        // written outcome is read before any pid is: under the real guard
        // that never matters here, and under the `false` mutant `contest`
        // would have to ask it and fail outright, which is not this poll.
        let (dir, matching_project) = fixture::copy("act-matching-key");
        let matching_key = a_finished_job(&dir, &matching_project);
        let matching_record = JobRecord::Running { key: matching_key.clone(), pid: UNASKABLE_PID, version_files: Vec::new() };
        let settled = act(&matching_project, Phase::One, "hello", &matching_key, Some(matching_record)).expect("a matching key settles rather than contesting");
        assert!(matches!(settled, GatePoll::Done(_)), "the finished job's own outcome, reached only because the recorded key matched the fresh one: {settled:?}");
    }

    #[test]
    #[validates(spec::AlivesOwnFailureFailsThePollAndNeverRestartsTheJob)]
    fn alives_own_failure_fails_the_poll_and_never_restarts_the_job() {
        assert!(alive(std::process::id()).expect("this process is running"), "a live pid");
        assert!(!alive(a_dead_pid()).expect("a reaped child is not"), "a dead one");
        let unaskable = alive(UNASKABLE_PID).expect_err("a host that cannot answer is not a job that has ended");
        assert!(unaskable.contains(&UNASKABLE_PID.to_string()), "naming the question it could not ask: {unaskable}");
        // And that failure is the poll's own: nothing beneath it reads "cannot
        // tell" as "dead", which is the answer that starts a whole gate again.
        let project = a_checkout_no_job_can_begin_in("alive-cannot-ask");
        let failed = restart_or_running(&project, Phase::Seven, "hello", &a_key("this tree"), UNASKABLE_PID).expect_err("the poll fails with it");
        assert!(!failed.contains(NO_BUMP_HERE), "and no fresh job is begun over it: {failed}");
    }

    #[test]
    #[validates(spec::SettleReadsTheWrittenResultBeforeItAsksWhetherThePidIsAlive)]
    fn settle_reads_the_written_result_before_it_asks_whether_the_pid_is_alive() {
        let (dir, project) = fixture::copy("settle-written-first");
        let job = a_finished_job(&dir, &project);
        // The pid is one the host cannot be asked about, so a `settle` that
        // asked about it first would fail here instead of answering.
        let settled = settle(&project, Phase::One, "hello", &job, UNASKABLE_PID).expect("a written outcome answers before any pid is asked about");
        assert!(matches!(settled, GatePoll::Done(_)), "the outcome the child wrote: {settled:?}");
        // Only where none is written does liveness decide, and that same pid
        // is the failure there.
        let nothing_ran = a_key("a key no job has finished under");
        assert!(settle(&project, Phase::One, "hello", &nothing_ran, UNASKABLE_PID).is_err(), "with nothing written, the pid is what settle asks about");
    }

    #[test]
    #[validates(spec::RestartOrRunningAnswersRunningForALivePidAndRestartsUnderTheSameKeyForADeadOne)]
    fn restart_or_running_answers_running_for_a_live_pid_and_restarts_under_the_same_key_for_a_dead_one() {
        let project = a_checkout_no_job_can_begin_in("restart-or-running");
        let job = a_key("this tree");
        let alive_answer = restart_or_running(&project, Phase::Seven, "hello", &job, std::process::id()).expect("a live pid");
        assert_eq!(alive_answer, GatePoll::Running, "a job started by an earlier call is still alive");
        // A dead pid died before finishing: the answer is a fresh job under
        // that same key, which in this checkout can only fail at the bump.
        let dead = restart_or_running(&project, Phase::Seven, "hello", &job, a_dead_pid()).expect_err("the fresh job cannot begin here");
        assert!(dead.contains(NO_BUMP_HERE), "a dead pid restarts rather than answering `Running`: {dead}");
    }

    #[test]
    #[validates(spec::ALiveForeignRecordRefusesTheStartNamingTheOtherKey)]
    fn a_live_foreign_record_refuses_the_start_naming_the_other_key() {
        let project = a_checkout_no_job_can_begin_in("contest-live");
        let (fresh, other) = (a_key("this tree"), a_key("another attempt's tree"));
        let refused = contest(&project, Phase::Seven, "hello", &fresh, std::process::id(), &other).expect_err("a live job under another key refuses the start");
        assert!(refused.contains("another attempt's tree"), "naming that other key: {refused}");
        assert!(!refused.contains(NO_BUMP_HERE), "and refusing outright: neither waiting for that job nor starting one here: {refused}");
    }

    #[test]
    #[validates(spec::ADeadForeignRecordStartsAFreshJobUnderTheCurrentKey)]
    fn a_dead_foreign_record_starts_a_fresh_job_under_the_current_key() {
        let project = a_checkout_no_job_can_begin_in("contest-dead");
        let (fresh, other) = (a_key("this tree"), a_key("a stale attempt's tree"));
        let started = contest(&project, Phase::Seven, "hello", &fresh, a_dead_pid(), &other).expect_err("the fresh job cannot begin in this checkout");
        assert!(started.contains(NO_BUMP_HERE), "a stale record blocks nothing: a fresh job begins, and begins with the bump: {started}");
        assert!(!started.contains("a stale attempt's tree"), "never the refusal naming the other key: {started}");
    }

    #[test]
    #[validates(spec::TheStatusDoorSharesActsThreeWaySplitAndReachesStartOnNoBranch)]
    fn the_status_door_shares_acts_three_way_split_and_reaches_start_on_no_branch() {
        let (dir, project) = fixture::copy("status-no-job");
        let answer = status(&project, Phase::Seven, "hello", &hello_crates(&dir)).expect("the status");
        assert_eq!(answer, GateStatus::NoJob, "nothing is running for this slice in this checkout");
        // Where a poll would have spawned one, a status read starts nothing
        // and writes nothing.
        assert_eq!(read_record(&project, "hello").expect("read"), None, "no record was written");
        assert!(!record_path(&project, "hello").expect("the record").exists(), "not even the file");
    }

    #[test]
    #[validates(spec::MatchingStatusAnswersNoJobWhereAPollWouldRestart)]
    fn matching_status_answers_no_job_where_a_poll_would_restart() {
        let (dir, project) = fixture::copy("matching-status");
        let finished_job = a_finished_job(&dir, &project);
        let done = matching_status(&project, &finished_job, a_dead_pid()).expect("the status");
        assert!(matches!(done, GateStatus::Done(_)), "an outcome already written, whatever the pid says: {done:?}");
        let running = a_key("a key no job has finished under");
        assert_eq!(matching_status(&project, &running, std::process::id()).expect("the status"), GateStatus::Running, "a pid still alive with none written");
        assert_eq!(
            matching_status(&project, &running, a_dead_pid()).expect("the status"),
            GateStatus::NoJob,
            "and where `restart_or_running` would begin a fresh job, a reader is told nothing is running"
        );
    }

    #[test]
    #[validates(spec::ForeignStatusAnswersForeignWhileThatPidLivesAndNoJobOnceItIsDead, spec::AStatusOverALiveForeignJobCarriesTheOtherKey)]
    fn foreign_status_answers_foreign_while_that_pid_lives_and_no_job_once_it_is_dead() {
        let other = a_key("the attempt this reader waits on");
        let live = foreign_status(&other, std::process::id()).expect("the status");
        assert_eq!(live, GateStatus::Foreign(other.clone()), "the other key itself, so a reader learns which attempt it waits on");
        assert_eq!(foreign_status(&other, a_dead_pid()).expect("the status"), GateStatus::NoJob, "a stale record is nothing for a caller to wait on");
    }

    #[test]
    #[validates(spec::TheJobRunsUnderAKeyItRecomputedAndHeldToTheFlagsHash, spec::TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText)]
    fn the_job_runs_under_a_key_it_recomputed_and_held_to_the_flags_hash() {
        let (dir, project) = fixture::copy("run-job-match");
        // The key the flag's hash is held to is one `run_job` computes for
        // itself from the project, the phase and the slice: this is the same
        // door, asked the same question, and the hash is all that crosses.
        let job = a_finished_job(&dir, &project);
        assert!(result_path(&project, &job).expect("the result").exists(), "the outcome is filed under the key the child recomputed");
        let outcome = finished(&project, &job).expect("read").expect("the outcome, read back as data");
        assert_eq!(outcome, execute(&project, Some("hello"), &plan(Phase::One, &[], &[])), "the plan's own answer, and no marker to find in captured text");
    }

    #[test]
    #[validates(spec::AKeyMismatchInTheChildRunsNoStepAndWritesNoResult)]
    fn a_key_mismatch_in_the_child_runs_no_step_and_writes_no_result() {
        let (dir, project) = fixture::copy("run-job-mismatch");
        // An LLD the first step of the plan rejects, so a step that did run
        // would leave an outcome behind to find.
        std::fs::write(dir.join("docs/intent/hello/lld.md"), "").expect("an LLD the checks reject");
        let fresh = key(&project, Phase::One, "hello", &hello_crates(&dir)).expect("the key this tree computes");
        let given = "a hash this tree cannot key to";
        let abandoned = run_job(&project, Phase::One, "hello", given).expect_err("the tree no longer keys to the job this child was spawned for");
        assert!(abandoned.contains(given) && abandoned.contains(&fresh.hash()), "naming the hash it was given and the one the tree computes: {abandoned}");
        assert_eq!(finished(&project, &fresh).expect("read"), None, "no result under the fresh key, which nothing polls");
        assert!(!result_path(&project, &fresh).expect("the result").exists(), "no step of the plan ran");
        let gate = project.target_directory().expect("the build directory").join("lid-rs/gate");
        assert!(!gate.join(given).exists(), "and nothing under the given hash's own directory, which this call never held the key for");
    }

    #[test]
    #[validates(spec::TheJobsRunnerDivertsTheMutationStepAloneToTheKeyedPaths)]
    fn the_jobs_runner_diverts_the_mutation_step_alone_to_the_keyed_paths() {
        let (dir, project) = fixture::copy("run-job-runner");
        let job = a_finished_job(&dir, &project);
        // Every step but the mutation step is `run_step`'s, unchanged: the
        // plan's answer through the job's own runner is the answer `execute`
        // gives for the same steps.
        let outcome = finished(&project, &job).expect("read").expect("the outcome");
        assert_eq!(outcome, execute(&project, Some("hello"), &plan(Phase::One, &[], &[])), "delegated unchanged");
        // And the mutation step alone is diverted: what it is handed is this
        // key's own scratch space, never the fixed defaults a bare
        // `phase-check 7` leaves the engine to resolve.
        let (diff, output) = paths_for(&project, &job).expect("the keyed paths");
        let target = project.target_directory().expect("the build directory");
        assert!(diff != target.join("lid-mutants.diff"), "the diff file is the key's: {diff:?}");
        assert!(output != target.join("lid-mutants"), "and so is the output root: {output:?}");
    }

    #[test]
    #[validates(spec::TheBoundedStandInCarriesTheFailingHeaderTheLastTailAndThePath)]
    fn the_bounded_stand_in_carries_the_failing_header_the_last_tail_and_the_path() {
        let root = fixture::scratch("bounded-output");
        let project = project_at(&root, &root.join("target"));
        let job = a_key("one tree");
        // What the execution writes when a step fails, with a capture far past
        // the bound and a second ` failed: ` of the kind cargo routinely
        // prints: the header is the first occurrence, which is the one the
        // format string wrote.
        let head = "the first thing the failing tool wrote";
        let capture =
            format!("Mutants failed: {head}\n{}\nerror: linking with `cc` failed: exit status: 1\n16 mutant(s) survived their validating tests\n", "filler ".repeat(4000));
        let bounded = bounded_output(&project, &job, &capture).expect("the stand-in");
        assert!(bounded.starts_with("Mutants failed: "), "the step's own header, up to and including the marker: {bounded}");
        assert!(!bounded.contains(head), "the head is what the bound cuts off");
        assert!(bounded.contains("16 mutant(s) survived"), "and the last of it is what it keeps");
        let whole = result_path(&project, &job).expect("the result").display().to_string();
        assert!(bounded.contains(&whole), "with the address the whole capture is at, for what the bound cut off");
        assert!(bounded.len() < capture.len() / 2, "bounded, however large the failing step's own output is: {} of {}", bounded.len(), capture.len());
    }

    #[test]
    #[validates(spec::TheBoundedStandInStillNamesTheCheckThatFired)]
    fn the_bounded_stand_in_still_names_the_check_that_fired() {
        let root = fixture::scratch("bounded-output-check");
        let project = project_at(&root, &root.join("target"));
        let job = a_key("one tree");
        let filler = "filler ".repeat(4000);
        let survivors = format!("Mutants failed: cargo mutants said:\n{filler}\n16 mutant(s) survived their validating tests\n");
        let bounded = bounded_output(&project, &job, &survivors).expect("the stand-in");
        assert_eq!(check_of_output(&bounded), Some(Check::Twelve), "the marker the kept tail holds: {bounded}");
        let denied = format!("Clippy failed: clippy said:\n{filler}\nerror: the function has a cognitive complexity of (5/4)\n = note: `-D clippy::cognitive-complexity`\n");
        let bounded = bounded_output(&project, &job, &denied).expect("the stand-in");
        assert_eq!(check_of_output(&bounded), Some(Check::Seven), "and a denied lint's own name: {bounded}");
        // So the refusal a poll composes names the check, rather than falling
        // through to the answer a bare header would have left it with.
        let refusal = refusal_for(&this_project(), Phase::Seven, &phase_crates(), &bounded);
        assert!(!refusal.contains("No gate row names this failure"), "{refusal}");
    }
}
