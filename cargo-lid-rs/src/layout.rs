use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::Project;
use crate::spec;

/// Which of the four shapes a slice has where it is asked about — the one
/// place the layout is told apart, so that no resolver invents its own test
/// (§ "Where a slice's `lld.md` goes — four cases, not two").
///
/// The shape is a fact about a directory and the crate it is under, not about
/// the files in it: which shape a slice has decides where its code, its
/// claims, and its document go, and every resolver below reads the answer
/// from here rather than probing the tree again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Form {
    /// The crate holds a module named for the slice — code in
    /// `src/<module>.rs` and `src/<module>/` — so the slice's directory is
    /// `src/<module>` under that crate.
    Module {
        /// The crate the module is under.
        crate_root: PathBuf,
        /// The module's name: the slice's, with hyphens as underscores.
        module: String,
    },
    /// The code is the crate itself, so the slice's directory is that
    /// crate's `src` and its document is `lib.rs`'s inner doc. No module is
    /// invented to hold a document.
    CrateRoot {
        /// The crate whose `src` is the slice's directory.
        crate_root: PathBuf,
    },
    /// One slice's presence in the companion its own crate's manifest names:
    /// the directory holding that slice's claims and the module citing them,
    /// in a crate that is not the slice's own. It is not a slice — it carries
    /// no `lld.md` — which is the exception `lld-check` must know about
    /// (§ "The companion rule, under colocation").
    Companion {
        /// The companion crate the directory is under.
        crate_root: PathBuf,
        /// The module's name: the slice's, with hyphens as underscores.
        module: String,
    },
    /// No workspace member holds a document for the slice: it has no crate,
    /// so no directory of its own, and its document stays under the
    /// workspace root — the shape of the slices whose product is the
    /// workspace rather than a crate.
    NoCrate,
}

impl Form {
    /// The shape the slice has in its own crate: the workspace member whose
    /// directory holds the slice's document, in either layout, and then
    /// whether that crate holds a module named for the slice.
    ///
    /// Never `Companion`. A slice's artifacts are resolved from the crate
    /// its document is in — the crate its phases write — so a directory
    /// named for the slice in the companion that holds its claims is never
    /// taken for the slice's own.
    #[implements(
        spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
        spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
        spec::ASlicesDocumentIsNeverUnderItsCompanion,
    )]
    pub fn of_slice(project: &Project, slice: &str) -> Self {
        let _ = (project, slice);
        todo!()
    }

    /// The shape of a directory, whose name is the slice's: `Companion` when
    /// the manifest of that slice's own crate names this directory's crate as
    /// its companion, and otherwise the shape that slice has in its own
    /// crate.
    ///
    /// The companion is read from `[package.metadata.lid_rs] companion`,
    /// through the resolution the phase policy already makes, and never from
    /// what the directory holds.
    #[implements(
        spec::ACompanionDirectoryIsTheOneTheManifestNames,
        spec::ACompanionIsNeverReadFromADirectorysShape,
    )]
    pub fn of_directory(project: &Project, dir: &Path) -> Self {
        let _ = (project, dir);
        todo!()
    }
}

/// A slice's directory — the one place the layout is computed: `src/<slice>`
/// under the crate that holds the slice's document when that crate holds a
/// module named for the slice, that crate's `src` when it does not, and the
/// refusal naming the slice when no workspace member holds a document for it
/// at all.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ASliceNoMemberHoldsIsRefusedByName,
)]
pub fn slice_dir(project: &Project, slice: &str) -> Result<PathBuf, String> {
    let _ = (project, slice);
    todo!()
}

/// A slice's `lld.md`, in either layout while both stand: the `lld.md` in the
/// slice's directory once its document sits beside its code, the
/// `docs/intent/<slice>/lld.md` its crate still holds until the migration
/// moves it, and — for a slice no workspace member holds a document for — that
/// same path under the workspace root. A mixed tree is the migration's normal
/// state, not an error, so both forms are answers and neither is a failure;
/// the failure is the one cargo's metadata cannot name a root for.
///
/// The crate is always the slice's own, never the companion that holds its
/// claims: one slice has one document, in the crate its phases write.
#[implements(
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
    spec::ASlicesDocumentIsNeverUnderItsCompanion,
)]
pub fn lld_path(project: &Project, slice: &str) -> Result<PathBuf, String> {
    let _ = (project, slice);
    todo!()
}

/// Whether a directory is a companion's — the `src/<module>` a proc-macro
/// crate's slice has in the crate its `[package.metadata.lid_rs] companion`
/// key names, which holds that slice's claims and the module citing them and
/// carries no `lld.md`.
///
/// Read from the manifest and never from the directory's shape: a `spec.rs`
/// and a `mod.rs` with no `lld.md` is also what a slice whose Phase 1 was
/// skipped looks like, which is the case `lld-check` exists to catch, and the
/// manifest already carries the fact that tells them apart.
#[implements(
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
    spec::ACompanionIsNeverReadFromADirectorysShape,
)]
pub fn is_companion_dir(project: &Project, dir: &Path) -> bool {
    let _ = (project, dir);
    todo!()
}
