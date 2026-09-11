//! Claims for the `layout` slice — a slice's artifacts in one directory
//! (`docs/intent/layout/lld.md`).
//!
//! The slice's phases build the *mechanism* — `cargo_lid_rs::layout` and the
//! resolvers that read it — and the migration of the repository's own tree is
//! hand-committed afterwards. So these claims are about the rule and the
//! resolution and never about the move: what `slice_dir` answers for a slice
//! of each shape, which member `own_crate` answers with, what `lld_path`
//! answers while both layouts stand, and what `is_companion_dir` reads to tell
//! a companion's directory from a slice's. A claim that every slice's document
//! sits beside its code would be a claim about the migration, which no phase of
//! this slice performs.
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
