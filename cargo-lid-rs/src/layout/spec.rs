//! Claims for the `layout` slice — a slice's artifacts in one directory
//! (`docs/intent/layout/lld.md`).
//!
//! The slice's phases build the *mechanism* — `cargo_lid_rs::layout` and the
//! resolvers that read it — and the migration of the repository's own tree is
//! hand-committed afterwards. So these claims are about the rule and the
//! resolution and never about the move: what `slice_dir` answers for a slice
//! of each shape, which member `own_crate` answers with, what `lld_path`,
//! `spec_file` and `intent_file` answer while both layouts stand, and what
//! `is_companion_dir` reads to tell a companion's directory from a slice's. A
//! claim that every slice's document sits beside its code would be a claim
//! about the migration, which no phase of this slice performs.
//!
//! `Form` is the one branch the four shapes are told apart at, so the
//! classification half of each claim below is its, and the path half is the
//! resolver's: a claim here holds whichever item Phase 3 seats the branch in.
//!
//! A member's `src/lld.md` records no slice name, so which member answers for a
//! crate-root slice is a claim of its own: that slice is the crate, and the
//! package names it. Read through that claim, a member's `src/lld.md` is the
//! document of the one slice the member is named for and of no other, which is
//! what `ASliceNoMemberHoldsIsRefusedByName` below means by a slice no member
//! holds a document for — so the refusal is one claim and not two.
//!
//! `own_crate` answers that crate to a caller outside this module — the `phase`
//! slice's `policy::slice_crate` resolves a slice's crate through it — so which
//! member it answers with is a claim of its own. Nothing the directory and
//! document claims state is falsified by a wrong crate from it: each of those
//! is asked about what the resolver *builds* from a crate, and a door that
//! answers the crate itself sits above them all.
//!
//! The refusal is stated for a resolver that needs a slice's crate rather than
//! for `slice_dir` alone, because one sentence — `no_crate_refusal` — is what
//! every such resolver gives, and `own_crate` is a second of them. `lld_path`
//! is not one: a slice no member holds a document for still has a document, at
//! the workspace root, which is `ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot`
//! below, so it needs no crate and refuses nothing. `is_companion_dir` is handed
//! a directory and so is never asked to find one.
//!
//! `spec_file` and `intent_file` are two more resolvers that need a slice's
//! crate — neither form of either answer can be built without one — so their
//! refusals are that same claim's, and no sentence below restates them. That is
//! what its *when* was written for: a resolver of `layout` that needs the crate
//! of a slice, which is what each of these two is, and a second refusal claim
//! would be one rule in two places.
//!
//! Which layout each of them answers in is settled by the same fact `lld_path`
//! turns on — whether the slice's directory holds `lld.md` — and never by the
//! presence of the file being asked for. A file a phase is about to *write* does not exist
//! yet: Phase 2 asks where a new slice's claims go, and a human asks where a
//! compile-time slice's acceptance goes, so a door reading its own answer's
//! presence could name the colocated path for neither. The document is what
//! marks a directory a slice's, so it is what says which layout that slice is
//! in.
//!
//! They are two doors and not one because their old forms differ in shape: a
//! spec file's old path names the slice in the *filename*, in a `src/spec`
//! directory shared with every other slice of the crate, while an intent file's
//! names it in a *directory* of its own under `docs/intent`. Under colocation
//! both are a named file in the slice's directory and the two coincide; the
//! migration is the window in which they do not, which is why each form is a
//! claim.
//!
//! Which form of a claims file a slice is in is asked only of a slice whose
//! crate holds a module named for it. A crate-root slice's code is its crate,
//! so its directory is that crate's `src` and its claims are the `src/spec.rs`
//! already beside that code — the one path the two layouts agree on, and so a
//! claim of its own rather than a case inside either of theirs. Neither of the
//! two could hold it: the colocated form names the `spec.rs` in the directory
//! named for the slice, and a crate-root slice has no such directory, while
//! the old form names `src/spec/<module>.rs`, which computes from any name at
//! all and so answers a file the crate does not hold.
//!
//! Answering by what a crate holds rather than by what a slice is called is
//! what ends a fault older than this module. A claims file computed from the
//! slice's name is `src/spec/xtask.rs` for `xtask` and
//! `src/spec/intent_graph.rs` for `intent-graph` — files no crate holds, so
//! those slices' red runs have matched no claim since the slices were built,
//! and an empty red set reads exactly like a slice whose validations all
//! legitimately pass. `xtask` keeps its claims at `src/spec.rs` already; the
//! `cargo-lid-rs` slice keeps them at `src/spec/cargo_lid_rs.rs` and moves
//! that file with the migration, which is the only crate-root claims file that
//! moves at all.
//!
//! The spec-file claims name no crate where the document pair names the slice's,
//! and that is deliberate: which crate holds a slice's *claims* is not decided
//! here — a proc-macro slice's are its companion's, which is the `phase`
//! slice's rule — so these two state the path a crate holds them at, which is
//! the whole of what a mixed tree makes uncertain. The intent-file pair does
//! name the slice's crate, because an intent file has no companion form: a
//! companion directory carries no `lld.md`, and so no acceptance beside one.
//!
//! `intent_file` carries no answer for a slice no member holds a document for.
//! `lld_path`'s workspace-root answer is a special case beside it rather than
//! inside it, so a slice whose product is the workspace reaches the refusal
//! here and its document through `lld_path`.
//!
//! These claims are written in the controlled language from the first, like
//! the claim slice's; none is marked free.

use lid_rs::Spec;

// ---- A slice's directory: the four shapes ------------------------------------

/// When [`slice_dir`](crate::layout::slice_dir) is asked for a slice whose
/// crate holds a module named for that slice, the directory it answers with
/// shall be `src/<slice>` under that crate.
#[derive(Spec)]
pub struct AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc;

/// When [`slice_dir`](crate::layout::slice_dir) is asked for a slice whose
/// crate holds no module named for that slice, the directory it answers with
/// shall be that crate's `src`.
#[derive(Spec)]
pub struct ACrateRootSlicesDirectoryIsItsCratesSrc;

/// When [`Form::of_slice`](crate::layout::Form::of_slice) is asked for a slice
/// whose name is the package name of a workspace member holding `src/lld.md`,
/// the crate it answers with shall be that member.
#[derive(Spec)]
pub struct ACrateRootSlicesCrateIsTheMemberItIsNamedFor;

// ---- A slice's crate, and the refusal a resolver needing one gives ----------

/// When [`own_crate`](crate::layout::own_crate) is asked for a slice a
/// workspace member holds a document for, the crate it answers with shall be
/// that member.
#[derive(Spec)]
pub struct ASlicesOwnCrateIsTheMemberHoldingItsDocument;

/// When a resolver of [`layout`](crate::layout) needs the crate of a slice
/// that no workspace member holds a document for, it shall refuse naming that
/// slice.
#[derive(Spec)]
pub struct ASliceNoMemberHoldsIsRefusedByName;

// ---- A slice's document, while both layouts stand ----------------------------

/// When [`lld_path`](crate::layout::lld_path) is asked for a slice whose
/// directory holds `lld.md`, the document it answers with shall be that file.
#[derive(Spec)]
pub struct ADocumentBesideTheCodeIsTheSlicesLld;

/// When [`lld_path`](crate::layout::lld_path) is asked for a slice whose
/// directory holds no `lld.md`, the document it answers with shall be
/// `docs/intent/<slice>/lld.md` under that slice's crate.
#[derive(Spec)]
pub struct ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath;

/// When [`lld_path`](crate::layout::lld_path) is asked for a slice that no
/// workspace member holds a document for, the document it answers with shall
/// be `docs/intent/<slice>/lld.md` under the workspace root.
#[derive(Spec)]
pub struct ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot;

/// When [`lld_path`](crate::layout::lld_path) is asked for a slice whose
/// companion crate holds a directory named for it, the document it answers
/// with shall be the one under the slice's own crate.
#[derive(Spec)]
pub struct ASlicesDocumentIsNeverUnderItsCompanion;

// ---- A slice's claims file, while both layouts stand -------------------------

/// When [`spec_file`](crate::layout::spec_file) is asked for a slice whose
/// directory holds `lld.md`, the file it answers with shall be the `spec.rs`
/// in the directory named for that slice.
#[derive(Spec)]
pub struct ASpecFileBesideTheDocumentIsTheSlicesClaimsFile;

/// When [`spec_file`](crate::layout::spec_file) is asked for a slice whose
/// directory holds no `lld.md` and whose crate holds a module named for that
/// slice, the file it answers with shall be the `src/spec/<module>.rs` named
/// for that slice's module.
#[derive(Spec)]
pub struct ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec;

/// When [`spec_file`](crate::layout::spec_file) is asked for a slice whose
/// crate holds no module named for that slice, the file it answers with shall
/// be `src/spec.rs`, whether or not that slice's directory holds `lld.md`.
#[derive(Spec)]
pub struct ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode;

// ---- A named file of a slice's intent, while both layouts stand --------------

/// When [`intent_file`](crate::layout::intent_file) is asked for a named file
/// of a slice whose directory holds `lld.md`, the file it answers with shall
/// be the one of that name in that directory.
#[derive(Spec)]
pub struct ANamedFileBesideTheDocumentIsTheSlicesIntentFile;

/// When [`intent_file`](crate::layout::intent_file) is asked for a named file
/// of a slice whose directory holds no `lld.md`, the file it answers with
/// shall be the one of that name under `docs/intent/<slice>` in that slice's
/// crate.
#[derive(Spec)]
pub struct ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent;

// ---- A companion's directory, read from the manifest -------------------------

/// When [`is_companion_dir`](crate::layout::is_companion_dir) is asked about a
/// directory whose name is a slice of the proc-macro crate whose manifest
/// names that directory's crate as its companion, its answer shall be `true`.
#[derive(Spec)]
pub struct ACompanionDirectoryIsTheOneTheManifestNames;

/// When [`is_companion_dir`](crate::layout::is_companion_dir) is asked about a
/// directory whose name is a slice of no proc-macro crate naming that
/// directory's crate as its companion, its answer shall be `false`, whatever
/// files that directory holds.
#[derive(Spec)]
pub struct ACompanionIsNeverReadFromADirectorysShape;
