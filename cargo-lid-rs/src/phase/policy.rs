//! The per-phase path policy (`docs/intent/phase/lld.md` § `hook pre-tool`):
//! which files a phase agent may write, where the slice's crate and its
//! companion are, and what kind of execution editing it entails.

use std::path::{Path, PathBuf};

use lid_rs::implements;

use super::Phase;
use crate::layout;
use crate::project::Project;
use super::spec;

/// What a tool call does, for the policy and the tally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    /// `Edit`, `Write`, `MultiEdit`, `NotebookEdit`: subject to the policy.
    Edit,
    /// `Read`, `Grep`, `Glob`, `LSP`, and the workflow's `StructuredOutput`
    /// — the answer a `schema` forces, which reads and writes nothing:
    /// never refused.
    Observation,
    /// `Bash` and anything else that runs: absent from the agents' tools.
    Command,
}

/// The policy's answer for one target path.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Within the phase's allowed set.
    Allowed,
    /// Outside it, with the reason the agent is given.
    Refused(String),
}

/// A policy refusal that is not about one path — the slice's crates
/// themselves cannot be written in any phase — as distinct from a hook that
/// cannot decide, which is an error. The pre-tool hook maps it to a denied
/// edit like any other: tallied, quoting the discipline row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// Why, for the agent: what the policy found and the key it names.
    pub reason: String,
}

/// Whether editing this slice executes the agent's code at compile time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionClass {
    /// Nothing the agent writes runs before Phase 5.
    Ordinary,
    /// The slice's crate has a `proc-macro` or `custom-build` target — named.
    CompileTime(String),
}

/// Which of a slice's crates a path is judged against — one table each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seat {
    /// The slice's own crate, whose manifest directory holds its LLD.
    Own,
    /// The companion a proc-macro crate's manifest names, where the slice's
    /// claims, its citing module, and its fixtures live.
    Companion,
}

/// The crates a phase may write for one slice, resolved once per hook call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceCrates {
    /// The slice, as the branch names it.
    pub slice: String,
    /// The slice's own crate.
    pub own: PathBuf,
    /// Its companion, when the slice's crate is a proc-macro crate.
    pub companion: Option<PathBuf>,
}

impl SliceCrates {
    /// The slice's crates: its own, and the companion its manifest names.
    /// The outer error is a hook that cannot decide — no package holds the
    /// slice's LLD; the inner is the policy's refusal of a proc-macro crate
    /// with no usable companion, which the pre-tool hook denies the edit
    /// with and the stop hook and `phase-check` fail with.
    pub fn resolve(project: &Project, slice: &str) -> Result<Result<Self, Refusal>, String> {
        let own = slice_crate(project, slice)?;
        Ok(companion(project, &own).map(|companion| Self { slice: slice.to_string(), own, companion }))
    }

    /// The crate that holds the slice's claims — where the red run diffs the
    /// spec file: the companion when there is one, else the slice's own.
    #[implements(spec::AProcMacroSlicesClaimsAreHeldByItsCompanion)]
    pub fn claims_crate(&self) -> &Path {
        self.companion.as_deref().unwrap_or(&self.own)
    }

    /// The seats a phase writes in, each with its crate: the slice's own,
    /// then the companion when there is one — the order the stop stages and
    /// the integrity check filters in.
    #[implements(spec::TheStopStagesBothCratesAllowedPaths, spec::IntegrityFiltersAgainstBothCratesAllowedPaths)]
    pub fn seats(&self) -> Vec<(Seat, &Path)> {
        std::iter::once((Seat::Own, self.own.as_path()))
            .chain(self.companion.as_deref().map(|companion| (Seat::Companion, companion)))
            .collect()
    }

    /// The seat a target lies under, with the target relative to that seat's
    /// crate (`within_crate`, tried for the own crate and then the
    /// companion); none when the target has a parent component or lies under
    /// neither, which is refused before any table is consulted.
    #[implements(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy, spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
    pub fn seat_of(&self, target: &Path) -> Option<(Seat, PathBuf)> {
        within_crate(&self.own, target)
            .map(|relative| (Seat::Own, relative))
            .or_else(|| self.companion.as_deref().and_then(|companion| within_crate(companion, target)).map(|relative| (Seat::Companion, relative)))
    }
}

/// The kind of a tool by its name.
pub fn kind_of(tool_name: &str) -> ToolKind {
    match tool_name {
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => ToolKind::Edit,
        "Read" | "Grep" | "Glob" | "LSP" | "StructuredOutput" => ToolKind::Observation,
        _ => ToolKind::Command,
    }
}

/// The slice's crate: the workspace package whose manifest directory holds
/// the slice's document, in whichever layout that package keeps it — the
/// `docs/intent/<slice>/lld.md` it has not moved, or the `lld.md` beside the
/// code of the module or the crate named for the slice once it has. A tree
/// with some slices moved and some not is the migration's normal state, and
/// the policy answers for a slice in either.
///
/// Which package that is is the layout slice's question, and this asks it:
/// `layout::own_crate` is the one place the shapes a slice's directory can
/// have are told apart, and a second reading of the tree here would be that
/// decision in two places — the layout would then move for one caller and not
/// for the other, on exactly the mixed tree the migration presents. The
/// refusal is that door's too, so an operator reads one sentence whichever
/// resolver refused, and it names both forms the document was looked for in
/// rather than only the old one.
#[implements(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
pub fn slice_crate(project: &Project, slice: &str) -> Result<PathBuf, String> {
    layout::own_crate(project, slice)
}

/// The `[package.metadata.lid_rs]` key that names a proc-macro crate's
/// companion.
const COMPANION_KEY: &str = "companion";

/// The companion the slice's crate names, when it is a proc-macro crate —
/// all from the metadata document, never by parsing the manifest. None for
/// an ordinary crate, whatever its metadata carries; for a proc-macro crate,
/// the member its key names, or the refusal for a key that names none, a
/// package that is not a member, or one that is itself a proc-macro crate.
#[implements(spec::AnOrdinaryCrateHasNoCompanion)]
pub fn companion(project: &Project, crate_root: &Path) -> Result<Option<PathBuf>, Refusal> {
    if is_proc_macro(project, crate_root) { named_companion(project, crate_root).map(Some) } else { Ok(None) }
}

/// A proc-macro crate's companion: the package its
/// `[package.metadata.lid_rs] companion` setting names
/// (`Project::package_setting_at`), or the refusal for a crate that names
/// none — no phase of such a slice can produce a claim.
#[implements(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames, spec::AProcMacroCrateNamingNoCompanionRefusesEveryEdit)]
fn named_companion(project: &Project, crate_root: &Path) -> Result<PathBuf, Refusal> {
    match project.package_setting_at(crate_root, COMPANION_KEY) {
        None => Err(companion_refusal("names no package")),
        Some(name) => member_companion(project, &name),
    }
}

/// The named companion as a workspace member: its manifest directory
/// (`Project::member_dir_named`), or the refusal for a name that is not a
/// member's.
#[implements(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames, spec::ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit)]
fn member_companion(project: &Project, name: &str) -> Result<PathBuf, Refusal> {
    match project.member_dir_named(name) {
        None => Err(companion_refusal(&format!("names `{name}`, which is not a workspace member"))),
        Some(dir) => usable_companion(project, name, dir),
    }
}

/// The member as a companion that can hold claims: its directory, or the
/// refusal for a member that is itself a proc-macro crate.
#[implements(spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit)]
fn usable_companion(project: &Project, name: &str, dir: PathBuf) -> Result<PathBuf, Refusal> {
    if is_proc_macro(project, &dir) { Err(companion_refusal(&format!("names `{name}`, which is itself a proc-macro crate"))) } else { Ok(dir) }
}

/// Whether the crate at `crate_root` declares a `proc-macro` target
/// (`Project::target_kinds_at`) — the crate that registers no claim and can
/// cite none.
#[implements(spec::AnOrdinaryCrateHasNoCompanion, spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit)]
fn is_proc_macro(project: &Project, crate_root: &Path) -> bool {
    project.target_kinds_at(crate_root).iter().any(|kind| kind == "proc-macro")
}

/// The policy's refusal of a proc-macro crate without a usable companion:
/// names the key, `[package.metadata.lid_rs] companion`, what it does
/// (`what`), and why that refuses every edit — no phase of such a slice can
/// produce a claim.
#[implements(
    spec::AProcMacroCrateNamingNoCompanionRefusesEveryEdit,
    spec::ACompanionThatIsAProcMacroCrateRefusesEveryEdit,
    spec::ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit,
)]
fn companion_refusal(what: &str) -> Refusal {
    Refusal {
        reason: format!(
            "`[package.metadata.lid_rs] {COMPANION_KEY}` {what}. The slice's crate is a proc-macro crate, which registers no claim and \
             can cite none, so no phase of this slice can produce a claim: every edit is refused until the manifest names a workspace \
             member that is not itself a proc-macro crate as the companion."
        ),
    }
}

/// One phase's allowed set for one seat, as paths relative to that seat's
/// crate; a directory entry admits what is under it, and the slice's own
/// directory admits only the files `admits_file` says are the phase's.
/// `Seat::Own` is the LLD's first path table, `Seat::Companion` its second.
#[implements(spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
pub fn allowed_paths(phase: Phase, slice: &str, seat: Seat, claims: &Path) -> Vec<PathBuf> {
    match seat {
        Seat::Own => own_table(phase, slice, claims),
        Seat::Companion => companion_table(phase, slice, claims),
    }
}

/// The LLD's first path table — what a phase may write in the slice's own
/// crate — one row per phase. Phase 1 is the human's: no agent writes in it.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
)]
fn own_table(phase: Phase, slice: &str, claims: &Path) -> Vec<PathBuf> {
    match phase {
        Phase::One => Vec::new(),
        Phase::Two => spec_files(claims),
        Phase::Three | Phase::Four => module_and(slice, &["src/lib.rs"]),
        Phase::Five | Phase::Seven => module_and(slice, &[]),
    }
}

/// The LLD's second path table — what a phase may write in the companion
/// — one row per phase: the first table's rows, and at Phases 5 and 7 the
/// `tests/ui` fixtures too. Phase 1 is the human's here as well.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn companion_table(phase: Phase, slice: &str, claims: &Path) -> Vec<PathBuf> {
    match phase {
        Phase::One => Vec::new(),
        Phase::Two => spec_files(claims),
        Phase::Three | Phase::Four => module_and(slice, &["src/lib.rs"]),
        Phase::Five | Phase::Seven => module_and(slice, &["tests/ui"]),
    }
}

/// Phase 2's row of either table: the slice's claims file as the layout puts
/// it, and `src/spec/mod.rs`.
///
/// The claims file is the layout's answer and not a spelling of the slice's
/// name. Before colocation the two agreed, and the rule was written as the
/// spelling; afterwards they disagree for every migrated slice, and the
/// spelling names a file that does not exist — which refuses Phase 2 its own
/// artifact while admitting a path that, were it written, would put the slice
/// alone in the old layout. The claim this row implements already says "the
/// layout's answer for the slice"; this asks for it.
///
/// `src/spec/mod.rs` stays because a crate that still holds retired names
/// keeps them there, and a rename's alias belongs where the old path
/// resolved.
#[implements(spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles, spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles)]
fn spec_files(claims: &Path) -> Vec<PathBuf> {
    vec![claims.to_path_buf(), PathBuf::from("src/spec/mod.rs")]
}

/// The slice's module — `src/<slice>.rs` and the directory `src/<slice>`,
/// the slice name in snake_case as the spec file's is — followed by
/// `extras`, crate-relative: the rows of Phases 3 to 7 in either table.
///
/// The directory entry is where the code of a slice is, and under the
/// colocated layout it is where the slice's intent is too. This row says
/// which directory a phase writes in; `admits_file` below says which of the
/// files in it are that phase's, and the two are built from the one
/// `module_dir` so that the entry and the rule cannot come apart.
#[implements(
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn module_and(slice: &str, extras: &[&str]) -> Vec<PathBuf> {
    let dir = module_dir(slice);
    let mut paths = vec![dir.with_extension("rs"), dir];
    paths.extend(extras.iter().map(PathBuf::from));
    paths
}

/// The slice's directory, crate-relative: `src/<module>`, the slice name in
/// snake_case as the spec file's is. The entry every row from Phase 3 on
/// carries, and the directory a target is judged against — a wider one hands
/// a phase another slice's code, and a narrower one refuses the phase its
/// own.
#[implements(
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn module_dir(slice: &str) -> PathBuf {
    Path::new("src").join(slice.replace('-', "_"))
}

/// The verdict for a target: refused before any table when it has a parent
/// component or lies under neither of the slice's crates; otherwise judged
/// by the table of the seat it lies under, relative to that crate, and — for
/// a target under the slice's directory — by the rule the directory entry
/// stands in for.
///
/// `claims` is the slice's claims file, crate-relative, as the layout puts
/// it: `layout::spec_file`'s answer, asked by the caller that holds the
/// project. One answer judges both seats, which is what the layout answering
/// relative to no crate is for — the crate whose layout is read is the one
/// holding the slice's document, and the companion holds the same file at the
/// same place while holding no document at all. Reading it from the crate a
/// target lies under would make the companion's copy of a colocated slice's
/// claims writable by every phase.
#[implements(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy, spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
pub fn allowed(phase: Phase, crates: &SliceCrates, claims: &Path, target: &Path) -> Verdict {
    match crates.seat_of(target) {
        None => Verdict::Refused(format!("`{}` has a parent component or lies outside the slice's crates", target.display())),
        Some((seat, relative)) => verdict_of(&relative, &allowed_paths(phase, &crates.slice, seat, claims), &module_dir(&crates.slice), claims),
    }
}

/// The phase's allowed paths of both crates — each seat's table relative to
/// its crate, made relative to the workspace root git runs at — which the
/// stop hook filters integrity against and stages.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
pub fn workspace_paths(project: &Project, phase: Phase, crates: &SliceCrates) -> Result<Vec<PathBuf>, String> {
    let root = project.root()?;
    let claims = crate::layout::spec_file(project, &crates.slice)?;
    let per_seat: Vec<Vec<PathBuf>> = crates
        .seats()
        .into_iter()
        .map(|(seat, crate_root)| seat_workspace_paths(&root, phase, &crates.slice, seat, crate_root, &claims))
        .collect::<Result<_, _>>()?;
    Ok(per_seat.concat())
}

/// One seat's table, workspace-relative: each entry under the crate's own
/// prefix.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
fn seat_workspace_paths(
    root: &Path,
    phase: Phase,
    slice: &str,
    seat: Seat,
    crate_root: &Path,
    claims: &Path,
) -> Result<Vec<PathBuf>, String> {
    let prefix = crate_prefix(root, crate_root)?;
    Ok(allowed_paths(phase, slice, seat, claims).into_iter().map(|entry| prefix.join(entry)).collect())
}

/// A crate's manifest directory relative to the workspace root, or the
/// failure naming both when it is not under the root.
#[implements(spec::TheStopStagesBothCratesAllowedPaths)]
fn crate_prefix(root: &Path, crate_root: &Path) -> Result<PathBuf, String> {
    crate_root
        .strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| format!("`{}` is not under the workspace root `{}`, so nothing in it can be staged", crate_root.display(), root.display()))
}

/// Whether a crate-relative path is in the phase's set: an entry admits it,
/// and the file it names is one this phase writes where it lies.
fn verdict_of(relative: &Path, allowed: &[PathBuf], dir: &Path, claims: &Path) -> Verdict {
    if matches_any(relative, allowed) && admits_file(relative, dir, claims, allowed) {
        Verdict::Allowed
    } else {
        Verdict::Refused(format!("`{}` is not in this phase's allowed set", relative.display()))
    }
}

/// Whether the file a target names is one the phase may write where it lies:
/// under the slice's directory the rule below decides, and anywhere else the
/// entry that admitted it has already said all there is to say — the module
/// file and the library root are files of their own, and the companion's
/// `tests/ui` holds a compile-failure fixture and the `.stderr` beside it,
/// which is not Rust source and is not held to being any.
#[implements(
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn admits_file(relative: &Path, dir: &Path, claims: &Path, allowed: &[PathBuf]) -> bool {
    if relative.starts_with(dir) { source_this_phase_may_write(relative, claims, allowed) } else { true }
}

/// Whether a file under the slice's directory is Rust source this phase may
/// write: the claims file is excluded unless the phase's table names it.
///
/// Colocation put the claims file inside the slice's directory, where before
/// it was the crate's `src/spec/<module>.rs` and under no slice's directory
/// at all. The exclusion below exists to keep Phases 3 to 7 out of Phase 2's
/// artifact, and while the file lay outside the directory it never met Phase
/// 2's own row. Now it does, so an exclusion that did not ask whose row it was
/// refuses Phase 2 the one file it exists to write. The table is the authority
/// on what a phase writes; this asks it rather than naming Phase 2, so a
/// later phase that gains the claims file is admitted by saying so in its row
/// and no second place has to agree.
#[implements(
    spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles,
    spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles,
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn source_this_phase_may_write(relative: &Path, claims: &Path, allowed: &[PathBuf]) -> bool {
    if names_claims(allowed, claims) { is_rust_source(relative) } else { rust_source_but_the_claims(relative, claims) }
}

/// Whether the phase's table names the claims file itself — an exact entry,
/// never a directory that happens to contain it, which is how every row from
/// Phase 3 on reaches the file without being entitled to it.
fn names_claims(allowed: &[PathBuf], claims: &Path) -> bool {
    allowed.iter().any(|entry| entry == claims)
}

/// Whether the target is Rust source.
fn is_rust_source(relative: &Path) -> bool {
    relative.extension().is_some_and(|extension| extension == "rs")
}

/// Whether a file under the slice's directory is a phase's to write: Rust
/// source, and not the slice's claims file.
///
/// The rule is positive, so what colocation puts beside a slice's code and is
/// no phase's — the design document Phase 1 settles, and the human's
/// acceptance of running this slice's code at compile time — is refused
/// without the policy learning either name, and so is the intent file
/// invented after this was written. Only the claims file has to be named,
/// because it is Rust source like the code around it, and it is named by the
/// layout rather than by this rule: before the migration it is the crate's
/// `src/spec/<module>.rs`, which is under no slice's directory, and a
/// `spec.rs` beside the code is then nobody's claims file and this phase's
/// source like any other.
#[implements(
    spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent,
    spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent,
    spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims,
    spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims,
)]
fn rust_source_but_the_claims(relative: &Path, claims: &Path) -> bool {
    is_rust_source(relative) && relative != claims
}

/// The target relative to the crate root, or none when it has a parent
/// component or lies outside the crate.
fn within_crate(crate_root: &Path, target: &Path) -> Option<PathBuf> {
    use std::path::Component;
    let clean = target.components().all(|c| !matches!(c, Component::ParentDir));
    (clean && target.is_absolute()).then(|| target.strip_prefix(crate_root).ok().map(Path::to_path_buf)).flatten()
}

/// Whether a relative path is one of, or under, the allowed entries.
fn matches_any(relative: &Path, allowed: &[PathBuf]) -> bool {
    allowed.iter().any(|entry| relative == entry || relative.starts_with(entry))
}

/// The reason an edit is refused: the discipline rows tagged for the phase,
/// from the synced skill, and what the phase may do instead.
#[implements(spec::ARefusedEditQuotesTheDisciplineRow)]
pub fn refusal_reason(project: &Project, phase: Phase, relative: &Path, allowed: &[PathBuf]) -> String {
    let rows = discipline_rows(project, phase).unwrap_or_default();
    let permitted: Vec<String> = allowed.iter().map(|p| format!("`{}`", p.display())).collect();
    format!(
        "`{}` is not Phase {}'s to write.\n\nThe skill's rule for this moment:\n{}\n\nIn this phase you may write only {}. \
         Proceed within those paths, or end with a ```stop block naming the decision this needs.",
        relative.display(),
        number_of(phase),
        rows.join("\n"),
        permitted.join(", ")
    )
}

/// The `discipline.md` rows whose phase column names this phase.
fn discipline_rows(project: &Project, phase: Phase) -> Result<Vec<String>, String> {
    Ok(table_rows_tagged(&read_synced(project, "references/discipline.md")?, &number_of(phase).to_string()))
}

/// A file of the synced skill, read from the project's copy.
pub(super) fn read_synced(project: &Project, relative: &str) -> Result<String, String> {
    let path = project.root()?.join(crate::sync::SKILL_IN_PROJECT).join(relative);
    std::fs::read_to_string(&path).map_err(|e| format!("reading the synced skill file {}: {e}", path.display()))
}

/// The rows of a Markdown table whose first cell contains `tag` as a
/// whole word, without the header and separator rows.
pub(super) fn table_rows_tagged(markdown: &str, tag: &str) -> Vec<String> {
    markdown
        .lines()
        .filter(|line| line.starts_with('|'))
        .skip(2)
        .filter(|line| first_cell(line).split(',').any(|entry| entry.trim() == tag))
        .map(str::to_string)
        .collect()
}

/// The first cell of a Markdown table row.
fn first_cell(row: &str) -> &str {
    row.trim_start_matches('|').split('|').next().unwrap_or("").trim()
}

/// The number a phase is written as in the skill's tables.
pub(super) fn number_of(phase: Phase) -> u8 {
    match phase {
        Phase::One => 1,
        Phase::Two => 2,
        Phase::Three => 3,
        Phase::Four => 4,
        Phase::Five => 5,
        Phase::Seven => 7,
    }
}

/// The slice's execution class, from the crate's target kinds.
#[implements(spec::ACompileTimeSliceIsDisclosed)]
pub fn execution_class(project: &Project, crate_root: &Path) -> Result<ExecutionClass, String> {
    let kinds = project.target_kinds_at(crate_root);
    Ok(kinds
        .iter()
        .find(|kind| kind.as_str() == "proc-macro" || kind.as_str() == "custom-build")
        .map_or(ExecutionClass::Ordinary, |kind| ExecutionClass::CompileTime(kind.clone())))
}

/// The name the human's acceptance of a compile-time slice is filed under,
/// in whichever layout the slice is in.
pub(crate) const ACCEPTANCE_FILE: &str = "compile-time-accepted";

/// Whether the human has accepted a compile-time slice: whether the file the
/// layout ([`crate::layout::intent_file`]) answers with is there — beside the
/// slice's code once the slice's document has moved there, and under
/// `docs/intent/<slice>` in the slice's own crate until then.
///
/// The layout is asked where that file is rather than told. A door that
/// joined `docs/intent/<slice>` onto the crate itself would, the moment a
/// slice migrated, read a slice the human *has* accepted as unaccepted and
/// refuse every edit to it — and would go on reading a file left at the
/// abandoned path as an acceptance of a slice whose intent no longer lives
/// there. The crate is the layout's to resolve too: a slice's intent is in
/// the crate its phases write, and an intent file has no companion form for a
/// caller to place.
///
/// A layout that can place no file for the slice is the error and never
/// `true`: an acceptance that cannot be located is not an acceptance. The
/// caller's `?` carries that refusal out of the hook to `fail_closed`, which
/// denies the edit — the permissive reading is the one failure mode this gate
/// exists to prevent, a compile-time slice's code run after every edit
/// without the human having said so. The pre-tool hook resolves the slice's
/// crates before it asks this door, so there the refusal has already been
/// made; the answer here is what keeps it a refusal wherever else the door is
/// asked.
#[implements(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
pub fn compile_time_accepted(project: &Project, slice: &str) -> Result<bool, String> {
    Ok(layout::intent_file(project, slice, ACCEPTANCE_FILE)?.exists())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::fixture;
    use lid_rs::validates;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    /// The slice `hello` in an ordinary crate at `/w/app`, with no companion.
    fn own_only() -> SliceCrates {
        SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/app"), companion: None }
    }

    /// The slice `hello` in a proc-macro crate at `/w/mac` whose companion
    /// is `/w/app`.
    fn with_companion() -> SliceCrates {
        SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/mac"), companion: Some(PathBuf::from("/w/app")) }
    }

    /// The claims file of the notional fixtures' slice `hello`. Their crates
    /// are paths and not a tree, so no directory of that slice holds a
    /// document and the layout they stand in is the pre-migration one, whose
    /// claims file is the crate's `src/spec/<module>.rs` — under no slice's
    /// directory, so nothing beside the slice's code is another phase's.
    /// `colocated` below is the tree where it is.
    const NOTIONAL_CLAIMS: &str = "src/spec/hello.rs";

    /// Whether a phase refuses a target for these crates, whose slice keeps
    /// its claims in `claims`.
    fn refused_for(crates: &SliceCrates, claims: &str, phase: Phase, target: &str) -> bool {
        matches!(allowed(phase, crates, Path::new(claims), Path::new(target)), Verdict::Refused(_))
    }

    /// Whether a phase refuses a target under `/w/app`, the one crate.
    fn refused(phase: Phase, target: &str) -> bool {
        refused_for(&own_only(), NOTIONAL_CLAIMS, phase, target)
    }

    /// Asserts whether each target is refused for the notional crates at this
    /// phase.
    fn check_refusals(crates: &SliceCrates, phase: Phase, cases: &[(&str, bool)]) {
        for (target, expected) in cases {
            assert_eq!(refused_for(crates, NOTIONAL_CLAIMS, phase, target), *expected, "{phase:?} {target}");
        }
    }

    /// `check_refusals` for a fixture whose tree is on disk: the targets are
    /// named relative to the workspace root, and the slice's claims file is
    /// the layout's own answer for that tree — the door the hook asks, so the
    /// crate whose layout is read is the fixture's and not this test's idea
    /// of it.
    fn check_refusals_under(project: &Project, crates: &SliceCrates, phase: Phase, cases: &[(&str, bool)]) {
        let root = project.root().expect("the fixture's workspace root");
        let claims = layout::spec_file(project, &crates.slice).expect("the fixture's crate holds the slice's document");
        let claims = claims.display().to_string();
        for (target, expected) in cases {
            let under = root.join(target).display().to_string();
            assert_eq!(refused_for(crates, &claims, phase, &under), *expected, "{phase:?} {target}");
        }
    }

    /// The slice `m` in the colocated layout, in a two-member workspace: its
    /// directory under `owner` is code and intent at once — `lld.md`,
    /// `spec.rs` and the human's `compile-time-accepted` beside the module
    /// and a leaf — and `app`, the companion, holds the same slice's claims
    /// file, the module citing them, and a `tests/ui` fixture. The
    /// pre-migration document is removed, so the slice is in one layout and
    /// not in both.
    ///
    /// The tree is real because which file of a directory is the slice's
    /// claims file is a layout question, and the layout is read from where
    /// the slice's document is. A notional path answers the pre-migration
    /// layout, where no directory of a slice holds its intent at all, and a
    /// test asking about one cannot see the widening colocation causes.
    ///
    /// The project is returned with the crates because it is what the layout
    /// is asked through: the workspace root the targets are named under, and
    /// the slice's claims file, are both read from it rather than assumed
    /// here — the same two answers the hook has when it judges an edit.
    fn colocated(name: &str) -> (Project, SliceCrates) {
        let (root, project) = fixture::two_member_workspace(name, "", "");
        std::fs::remove_dir_all(root.join("owner/docs")).expect("the pre-migration document");
        let tree = [
            ("owner/src/m/lld.md", "# m\n"),
            ("owner/src/m/mod.rs", "//! The m slice.\n"),
            ("owner/src/m/spec.rs", "//! Claims of m.\n"),
            ("owner/src/m/leaf.rs", "//! A leaf of m.\n"),
            ("owner/src/m/compile-time-accepted", ""),
            ("app/src/m/mod.rs", "//! m's presence in app.\n"),
            ("app/src/m/spec.rs", "//! Claims of m.\n"),
            ("app/src/m/leaf.rs", "//! A leaf of m.\n"),
            ("app/tests/ui/fail.rs", "fn main() {}\n"),
        ];
        for (relative, content) in tree {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().expect("the path has a parent")).expect("create the directories");
            std::fs::write(path, content).expect("write the file");
        }
        let crates = SliceCrates { slice: "m".to_string(), own: root.join("owner"), companion: Some(root.join("app")) };
        (project, crates)
    }

    #[test]
    #[validates(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
    fn the_slices_crate_is_the_one_holding_its_lld() {
        let workspace = fixture::workspace();
        let expected = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().expect("dir");
        assert_eq!(slice_crate(&workspace, "phase").expect("found").canonicalize().expect("dir"), expected);
        let macros = slice_crate(&workspace, "macros").expect("found");
        assert!(macros.ends_with("lid-rs-macros"), "{}", macros.display());
        let err = slice_crate(&workspace, "skill").expect_err("a workspace-only slice has no crate");
        assert!(err.contains("skill"), "{err}");
    }

    #[test]
    #[validates(spec::TheSlicesCrateIsTheOneHoldingItsLld)]
    fn the_slices_crate_is_found_wherever_the_layout_puts_its_document() {
        // `owner` holds `docs/intent/m/lld.md`; `app` is given a slice whose
        // document has already moved beside its code. Both stand at once
        // throughout the migration, so a policy that knew only the old form
        // would lose every slice the migration has already touched — and the
        // real workspace above, where nothing has moved yet, cannot show it.
        let (dir, project) = fixture::two_member_workspace("policy-either-layout", "", "");
        std::fs::create_dir_all(dir.join("app/src/moved")).expect("the slice's directory");
        std::fs::write(dir.join("app/src/moved/lld.md"), "# moved\n").expect("the moved document");
        assert_eq!(
            (slice_crate(&project, "m"), slice_crate(&project, "moved")),
            (Ok(dir.join("owner")), Ok(dir.join("app")))
        );
        // And a slice no member holds is refused naming both forms looked for,
        // not only the one the migration is leaving.
        let unheld = slice_crate(&project, "nobody").expect_err("no member holds a document named `nobody`");
        assert!(unheld.contains("docs/intent/nobody/lld.md") && unheld.contains("beside the code"), "{unheld}");
    }

    #[test]
    #[validates(spec::PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles)]
    fn phase_two_may_write_only_the_own_crates_spec_files() {
        assert_eq!(allowed_paths(Phase::Two, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), paths(&["src/spec/hello.rs", "src/spec/mod.rs"]));
        check_refusals(&own_only(), Phase::Two, &[("/w/app/src/spec/hello.rs", false), ("/w/app/src/hello.rs", true)]);
        // With a companion, the own crate is still judged by its own row.
        check_refusals(&with_companion(), Phase::Two, &[("/w/mac/src/spec/mod.rs", false), ("/w/mac/src/lib.rs", true)]);
        // Colocated, the row is the layout's answer and not the slice's name
        // spelled into the old path — and the claims file lies inside the
        // slice's directory, where the rule keeping later phases out of it
        // would refuse Phase 2 the one file it exists to write. Asserting
        // this against a notional path cannot see either: the fixture is a
        // tree so that the layout is read rather than assumed.
        let (project, crates) = colocated("phase_two_colocated");
        assert_eq!(
            allowed_paths(Phase::Two, &crates.slice, Seat::Own, &layout::spec_file(&project, "m").expect("the slice's claims file")),
            paths(&["src/m/spec.rs", "src/spec/mod.rs"])
        );
        check_refusals_under(&project, &crates, Phase::Two, &[("owner/src/m/spec.rs", false), ("owner/src/m/mod.rs", true)]);
        // The companion holds the same file at the same place, and it is
        // Phase 2's there too.
        check_refusals_under(&project, &crates, Phase::Two, &[("app/src/m/spec.rs", false), ("app/src/m/leaf.rs", true)]);
        // And the exclusion still holds where the row does not name the file:
        // Phases 3 to 7 reach it through the directory entry and are refused.
        check_refusals_under(&project, &crates, Phase::Three, &[("owner/src/m/spec.rs", true)]);
        check_refusals_under(&project, &crates, Phase::Five, &[("owner/src/m/spec.rs", true)]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent)]
    fn phases_three_and_four_may_write_the_own_crates_slice_module_and_library_root() {
        let expected = paths(&["src/hello.rs", "src/hello", "src/lib.rs"]);
        assert_eq!(allowed_paths(Phase::Three, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), expected);
        assert_eq!(allowed_paths(Phase::Four, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), expected);
        // What that row admits under the slice's directory is the code in
        // it, never the design document colocation puts there.
        check_refusals(&own_only(), Phase::Three, &[("/w/app/src/hello/lld.md", true)]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent)]
    fn phases_three_and_four_refuse_the_rest() {
        let cases = [
            (Phase::Three, "/w/app/src/hello/policy.rs", false),
            (Phase::Four, "/w/app/src/lib.rs", false),
            (Phase::Three, "/w/app/src/spec/hello.rs", true),
            (Phase::Three, "/w/app/src/hello_extra.rs", true),
            // The rest includes the slice's own intent, wherever the layout
            // puts it: the human's acceptance of a compile-time slice is not
            // a phase's to write once it sits beside the code.
            (Phase::Four, "/w/app/src/hello/compile-time-accepted", true),
            // And only wherever the layout puts it. These crates stand in the
            // pre-migration layout, whose claims file is the crate's
            // `src/spec/<module>.rs`, so a `spec.rs` in the slice's directory
            // is nobody's claims file and is this phase's Rust source like
            // any other — the same path `colocated` below refuses, because
            // there it is the file the layout names.
            (Phase::Three, "/w/app/src/hello/spec.rs", false),
        ];
        for (phase, target, expected) in cases {
            assert_eq!(refused(phase, target), expected, "{phase:?} {target}");
        }
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent)]
    fn phases_five_and_seven_may_write_only_the_own_crates_slice_module() {
        let expected = paths(&["src/hello.rs", "src/hello"]);
        assert_eq!(allowed_paths(Phase::Five, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), expected);
        assert_eq!(allowed_paths(Phase::Seven, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), expected);
        // The code in that directory, never the human's acceptance of
        // running this slice's code, which colocation puts beside it.
        check_refusals(&own_only(), Phase::Five, &[("/w/app/src/hello/compile-time-accepted", true)]);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent)]
    fn phases_five_and_seven_refuse_the_rest() {
        let cases = [
            (Phase::Seven, "/w/app/src/hello.rs", false),
            (Phase::Five, "/w/app/src/lib.rs", true),
            (Phase::Seven, "/w/app/docs/intent/hello/lld.md", true),
        ];
        for (phase, target, expected) in cases {
            assert_eq!(refused(phase, target), expected, "{phase:?} {target}");
        }
        // The own crate's row admits no fixtures, even when the companion's does.
        check_refusals(&with_companion(), Phase::Five, &[("/w/mac/tests/ui/fail.rs", true), ("/w/mac/src/hello/part.rs", false)]);
        // And the document under the slice's own directory is nobody's here.
        check_refusals(&own_only(), Phase::Seven, &[("/w/app/src/hello/lld.md", true)]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntent)]
    fn phases_three_and_four_may_write_the_own_crates_slice_code_and_library_root_not_its_intent() {
        let (project, crates) = colocated("policy-own-code-three-four");
        let own = SliceCrates { companion: None, ..crates };
        let cases = [
            // The slice's intent, in the directory colocation puts it in:
            // the design document Phase 1 settles, the claims Phase 2 owns,
            // and the human's acceptance of running this slice's code at
            // compile time. None of the three is this phase's to write.
            ("owner/src/m/lld.md", true),
            ("owner/src/m/spec.rs", true),
            ("owner/src/m/compile-time-accepted", true),
            // The Rust source beside them, which is these phases' own, and
            // the library root where the module is declared.
            ("owner/src/m/mod.rs", false),
            ("owner/src/m/leaf.rs", false),
            ("owner/src/lib.rs", false),
        ];
        check_refusals_under(&project, &own, Phase::Three, &cases);
        check_refusals_under(&project, &own, Phase::Four, &cases);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntent)]
    fn phases_five_and_seven_may_write_only_the_own_crates_slice_code_not_its_intent() {
        let (project, crates) = colocated("policy-own-code-five-seven");
        let own = SliceCrates { companion: None, ..crates };
        let cases = [
            ("owner/src/m/lld.md", true),
            ("owner/src/m/spec.rs", true),
            ("owner/src/m/compile-time-accepted", true),
            // The library root is Phases 3 and 4's row, not this one's.
            ("owner/src/lib.rs", true),
            ("owner/src/m/mod.rs", false),
            ("owner/src/m/leaf.rs", false),
        ];
        check_refusals_under(&project, &own, Phase::Five, &cases);
        check_refusals_under(&project, &own, Phase::Seven, &cases);
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn paths_outside_the_slices_crates_are_refused_before_the_policy() {
        let targets = ["/w/app/src/hello/../../Cargo.toml", "/w/app/src/hello/../spec/hello.rs", "/etc/passwd", "/w/other/src/hello.rs", "src/hello.rs"];
        for target in targets {
            assert!(refused(Phase::Three, target), "{target}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn a_path_under_neither_crate_is_refused_before_either_table() {
        // Under neither crate, or reached through a parent component from
        // either — even one that resolves inside — is refused as outside.
        let targets = ["/w/other/src/hello.rs", "/w/app/src/hello/../../Cargo.toml", "/w/mac/../app/src/hello.rs", "/w/app/../mac/src/hello.rs"];
        for target in targets {
            let Verdict::Refused(why) = allowed(Phase::Three, &with_companion(), Path::new(NOTIONAL_CLAIMS), Path::new(target)) else {
                panic!("{target} was allowed")
            };
            assert!(why.contains("outside the slice's crates"), "{target}: {why}");
        }
    }

    #[test]
    #[validates(spec::PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy)]
    fn a_target_is_seated_and_made_crate_relative_without_parent_components() {
        let crates = with_companion();
        let cases = [
            ("/w/mac/src/hello.rs", Some((Seat::Own, PathBuf::from("src/hello.rs")))),
            ("/w/app/tests/ui/fail.rs", Some((Seat::Companion, PathBuf::from("tests/ui/fail.rs")))),
            ("/w/mac/src/../Cargo.toml", None),
            ("/w/app/../mac/src/hello.rs", None),
        ];
        for (target, expected) in cases {
            assert_eq!(crates.seat_of(Path::new(target)), expected, "{target}");
        }
        assert_eq!(own_only().seat_of(Path::new("/w/other/src/hello.rs")), None, "no companion: under the own crate or nowhere");
    }

    #[test]
    #[validates(spec::APathUnderTheCompanionIsJudgedByTheCompanionsTable)]
    fn a_path_under_the_companion_is_judged_by_the_companions_table() {
        let crates = with_companion();
        // At Phase 5 the two rows differ: fixtures are the companion's to write.
        check_refusals(&crates, Phase::Five, &[("/w/app/tests/ui/fail.rs", false), ("/w/mac/tests/ui/fail.rs", true)]);
        // Each row is relative to its own crate: the union, not one set.
        check_refusals(&crates, Phase::Two, &[("/w/app/src/spec/hello.rs", false), ("/w/app/src/hello.rs", true)]);
        assert_ne!(allowed_paths(Phase::Five, "hello", Seat::Own, Path::new(NOTIONAL_CLAIMS)), allowed_paths(Phase::Five, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)));
    }

    #[test]
    #[validates(spec::PhaseTwoMayWriteOnlyTheCompanionsSpecFiles)]
    fn phase_two_may_write_only_the_companions_spec_files() {
        assert_eq!(allowed_paths(Phase::Two, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)), paths(&["src/spec/hello.rs", "src/spec/mod.rs"]));
        // The row is the claims file it is given and not the slice's name
        // spelled into a path: a hyphenated slice whose claims the layout
        // puts elsewhere gets that answer, not `src/spec/phase_gate.rs`.
        assert_eq!(
            allowed_paths(Phase::Two, "phase-gate", Seat::Companion, Path::new("src/phase_gate/spec.rs")),
            paths(&["src/phase_gate/spec.rs", "src/spec/mod.rs"])
        );
        let cases = [("/w/app/src/spec/mod.rs", false), ("/w/app/src/lib.rs", true), ("/w/app/tests/ui/fail.rs", true)];
        check_refusals(&with_companion(), Phase::Two, &cases);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims)]
    fn phases_three_and_four_may_write_the_companions_slice_module_and_library_root() {
        let expected = paths(&["src/hello.rs", "src/hello", "src/lib.rs"]);
        assert_eq!(allowed_paths(Phase::Three, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)), expected);
        assert_eq!(allowed_paths(Phase::Four, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)), expected);
        // `src/lib.rs` is where the hand-authored edges go; fixtures and claims are not this row's.
        check_refusals(&with_companion(), Phase::Four, &[("/w/app/src/lib.rs", false)]);
        check_refusals(&with_companion(), Phase::Three, &[("/w/app/tests/ui/fail.rs", true), ("/w/app/src/spec/hello.rs", true)]);
        // And what the row admits under the slice's directory is Rust source:
        // an asset there is refused, the companion holding no row for one.
        check_refusals(&with_companion(), Phase::Three, &[("/w/app/src/hello/fixture.json", true)]);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims)]
    fn phases_five_and_seven_may_write_the_companions_slice_module_and_ui_fixtures() {
        let expected = paths(&["src/hello.rs", "src/hello", "tests/ui"]);
        assert_eq!(allowed_paths(Phase::Five, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)), expected);
        assert_eq!(allowed_paths(Phase::Seven, "hello", Seat::Companion, Path::new(NOTIONAL_CLAIMS)), expected);
        // The fixtures are `tests/ui` alone; the library root is not this row's.
        check_refusals(&with_companion(), Phase::Seven, &[("/w/app/tests/ui/fail.rs", false), ("/w/app/tests/other.rs", true)]);
        check_refusals(&with_companion(), Phase::Five, &[("/w/app/src/hello/part.rs", false), ("/w/app/src/lib.rs", true)]);
        // And what the row admits under the slice's directory is Rust source:
        // an asset there is refused, `tests/ui` being where a fixture lives.
        check_refusals(&with_companion(), Phase::Five, &[("/w/app/src/hello/fixture.json", true)]);
    }

    #[test]
    #[validates(spec::PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims)]
    fn phases_three_and_four_may_write_the_companions_slice_code_and_library_root_not_its_claims() {
        let (project, crates) = colocated("policy-companion-code-three-four");
        let cases = [
            // The claims the companion holds for this slice — the layout's
            // answer for it, placed there rather than in the slice's own
            // crate — are Phase 2's, and the directory entry admitted them.
            ("app/src/m/spec.rs", true),
            // The rule is positive: what these phases write under the
            // slice's directory is Rust source, so an asset is refused
            // without the policy learning its name.
            ("app/src/m/fixture.json", true),
            // Fixtures are Phases 5 and 7's row, not this one's.
            ("app/tests/ui/fail.rs", true),
            ("app/src/m/mod.rs", false),
            ("app/src/m/leaf.rs", false),
            ("app/src/lib.rs", false),
        ];
        check_refusals_under(&project, &crates, Phase::Three, &cases);
        check_refusals_under(&project, &crates, Phase::Four, &cases);
    }

    #[test]
    #[validates(spec::PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims)]
    fn phases_five_and_seven_may_write_the_companions_slice_code_and_ui_fixtures_not_its_claims() {
        let (project, crates) = colocated("policy-companion-code-five-seven");
        let cases = [
            ("app/src/m/spec.rs", true),
            ("app/src/m/fixture.json", true),
            // The library root is Phases 3 and 4's row, not this one's.
            ("app/src/lib.rs", true),
            ("app/src/m/mod.rs", false),
            ("app/src/m/leaf.rs", false),
            // `tests/ui` is the one place a compile-failure fixture lives,
            // and what it admits is not held to being Rust source.
            ("app/tests/ui/fail.rs", false),
            ("app/tests/ui/fail.stderr", false),
        ];
        check_refusals_under(&project, &crates, Phase::Five, &cases);
        check_refusals_under(&project, &crates, Phase::Seven, &cases);
    }

    #[test]
    #[validates(spec::AProcMacroSlicesClaimsAreHeldByItsCompanion)]
    fn a_proc_macro_slices_claims_are_held_by_its_companion() {
        assert_eq!(with_companion().claims_crate(), Path::new("/w/app"), "the companion, not the macro crate");
        let ordinary = SliceCrates { slice: "hello".to_string(), own: PathBuf::from("/w/lib"), companion: None };
        assert_eq!(ordinary.claims_crate(), Path::new("/w/lib"), "an ordinary crate holds its own");
    }

    #[test]
    #[validates(spec::AnOrdinaryCrateHasNoCompanion)]
    fn an_ordinary_crate_has_no_companion() {
        // Both members carry the setting; only `app` declares the target.
        let app = format!("{}{}", fixture::PROC_MACRO_LIB, fixture::companion_setting("owner"));
        let (dir, project) = fixture::two_member_workspace("companion-ordinary", &fixture::companion_setting("app"), &app);
        assert_eq!(companion(&project, &dir.join("owner")), Ok(None), "the setting counts only for a proc-macro crate");
        assert_eq!(companion(&project, &dir.join("app")), Ok(Some(dir.join("owner"))), "none is for the crate without the target, not for every crate");
        let crates = SliceCrates::resolve(&project, "m").expect("the LLD is held").expect("no refusal");
        assert_eq!(crates, SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: None });
    }

    #[test]
    #[validates(spec::TheCompanionIsTheMemberTheProcMacroCratesMetadataNames)]
    fn the_companion_is_the_member_the_proc_macro_crates_metadata_names() {
        let owner = format!("{}{}", fixture::PROC_MACRO_LIB, fixture::companion_setting("app"));
        let (dir, project) = fixture::two_member_workspace("companion-named", &owner, "");
        assert_eq!(companion(&project, &dir.join("owner")), Ok(Some(dir.join("app"))));
        let crates = SliceCrates::resolve(&project, "m").expect("the LLD is held").expect("no refusal");
        assert_eq!(crates, SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: Some(dir.join("app")) });
    }

    #[test]
    #[validates(spec::TheStopStagesBothCratesAllowedPaths)]
    fn the_stop_stages_both_crates_allowed_paths() {
        let (dir, project) = fixture::two_member_workspace("stage-both", "", "");
        let crates = SliceCrates { slice: "m".to_string(), own: dir.join("owner"), companion: Some(dir.join("app")) };
        let both = workspace_paths(&project, Phase::Five, &crates).expect("both under the root");
        assert_eq!(both, paths(&["owner/src/m.rs", "owner/src/m", "app/src/m.rs", "app/src/m", "app/tests/ui"]));
        // What the commit stages is the changes within that set, and nothing else.
        std::fs::write(dir.join("owner/src/m.rs"), "//! m\n").expect("write");
        std::fs::create_dir_all(dir.join("app/tests/ui")).expect("dir");
        std::fs::write(dir.join("app/tests/ui/fail.rs"), "").expect("write");
        std::fs::write(dir.join("app/src/lib.rs"), "// changed\n").expect("write");
        assert_eq!(crate::phase::integrity::changed_within(&project, &both).expect("status"), paths(&["app/tests/ui/fail.rs", "owner/src/m.rs"]));
        let own = SliceCrates { companion: None, ..crates };
        assert_eq!(workspace_paths(&project, Phase::Five, &own).expect("under the root"), paths(&["owner/src/m.rs", "owner/src/m"]));
    }

    #[test]
    #[validates(spec::TheStopStagesBothCratesAllowedPaths)]
    fn a_crate_outside_the_root_cannot_be_staged() {
        assert_eq!(crate_prefix(Path::new("/w"), Path::new("/w/app")).expect("under the root"), PathBuf::from("app"));
        let err = crate_prefix(Path::new("/w"), Path::new("/elsewhere/app")).expect_err("not under the root");
        assert!(err.contains("/elsewhere/app") && err.contains("/w"), "names both: {err}");
    }

    #[test]
    #[validates(spec::ARefusedEditQuotesTheDisciplineRow)]
    fn a_refused_edit_quotes_the_discipline_row() {
        let workspace = fixture::workspace();
        let reason = refusal_reason(&workspace, Phase::Seven, Path::new("docs/intent/phase/lld.md"), &paths(&["src/phase.rs", "src/phase"]));
        let expected = ["Phase 8 event", "src/phase.rs", "```stop"];
        assert!(expected.iter().all(|e| reason.contains(e)), "the row tagged 7 and what the phase may do: {reason}");
    }

    #[test]
    #[validates(spec::ARefusedEditQuotesTheDisciplineRow)]
    fn table_rows_are_selected_by_phase_tag() {
        let rows = table_rows_tagged("| Phase(s) | When | Do this |\n|---|---|---|\n| 6, 7 | a | b |\n| 2, 3 | c | d |\n| 7 | e | f |\n", "7");
        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("| a |") && rows[1].contains("| e |"));
    }

    #[test]
    #[validates(spec::ACompileTimeSliceIsDisclosed)]
    fn a_proc_macro_crate_is_a_compile_time_slice() {
        let workspace = fixture::workspace();
        let macros = Path::new(env!("CARGO_MANIFEST_DIR")).join("../lid-rs-macros");
        assert_eq!(execution_class(&workspace, &macros).expect("class"), ExecutionClass::CompileTime("proc-macro".to_string()));
        let this = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        assert_eq!(execution_class(&workspace, &this).expect("class"), ExecutionClass::Ordinary);
    }

    #[test]
    #[validates(spec::ACompileTimeSliceIsDisclosed)]
    fn a_build_script_makes_a_compile_time_slice() {
        let (dir, _) = fixture::copy("build-script");
        std::fs::write(dir.join("build.rs"), "fn main() {}\n").expect("build.rs");
        let with_build = Project::load_graph_at(&dir.join("Cargo.toml")).expect("metadata");
        assert_eq!(execution_class(&with_build, &dir).expect("class"), ExecutionClass::CompileTime("custom-build".to_string()));
    }

    #[test]
    #[validates(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
    fn acceptance_is_the_intent_file_of_a_slice_that_has_not_migrated() {
        let (dir, project) = fixture::copy("acceptance");
        assert!(!compile_time_accepted(&project, "hello").expect("the layout places the slice's acceptance"));
        std::fs::write(dir.join("docs/intent/hello/compile-time-accepted"), "").expect("accept");
        assert!(compile_time_accepted(&project, "hello").expect("the layout places the slice's acceptance"));
    }

    #[test]
    #[validates(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
    fn a_migrated_slices_acceptance_is_the_file_beside_its_code() {
        // The same slice with its document moved beside its code: the
        // acceptance moves with it. A policy that knew only `docs/intent`
        // would read a slice the human has accepted as unaccepted and refuse
        // every edit to it, and would take a file left behind at the old path
        // for an acceptance of a slice whose intent no longer lives there.
        let (dir, project) = fixture::copy("acceptance-colocated");
        std::fs::create_dir_all(dir.join("src/hello")).expect("the slice's directory");
        std::fs::write(dir.join("src/hello/lld.md"), "# hello\n\nThe hello slice.\n").expect("the document, beside the code");
        std::fs::write(dir.join("docs/intent/hello/compile-time-accepted"), "").expect("the old form of the file");
        assert!(!compile_time_accepted(&project, "hello").expect("placed"), "an acceptance left at the old path accepts nothing");
        std::fs::write(dir.join("src/hello/compile-time-accepted"), "").expect("accept, beside the code");
        assert!(compile_time_accepted(&project, "hello").expect("placed"), "accepted where the layout says the acceptance belongs");
    }

    #[test]
    #[validates(spec::ACompileTimeSliceNeedsTheHumansAcceptance)]
    fn an_acceptance_the_layout_cannot_place_is_not_an_acceptance() {
        // Fail closed: a slice no workspace member holds a document for has
        // nowhere its acceptance could be, so the answer is the layout's
        // refusal — which the hook's `?` turns into a denied edit — and never
        // `true`.
        let (_dir, project) = fixture::copy("acceptance-unresolved");
        let refusal = compile_time_accepted(&project, "no-such-slice").expect_err("no member holds a document for it");
        assert!(refusal.contains("no-such-slice"), "{refusal}");
    }
}
