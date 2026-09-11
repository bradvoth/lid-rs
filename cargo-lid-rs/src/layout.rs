use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::Project;
use crate::spec;

/// The directory a slice's document sits under before the migration moves it
/// beside the code — and for good, for a slice whose product is the workspace
/// and which therefore has no code to sit beside.
const INTENT_DIR: &str = "docs/intent";

/// Which of the four shapes a slice has where it is asked about — the one
/// place the layout is told apart, so that no resolver invents its own test
/// (§ "Where a slice's `lld.md` goes — four cases, not two").
///
/// The shape is a fact about a directory and the crate it is under, not about
/// the files in it: which shape a slice has decides where its code, its
/// claims, and its document go, and every resolver below reads the answer
/// from here rather than probing the tree again.
///
/// Every shape carries the slice it was resolved for, named as it was asked
/// for. A module's name does not supply it — `lld-review`, not `lld_review` —
/// and the shape of a directory is resolved from a name the caller never had
/// (§ "`Form` carries the name it was resolved for").
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
        /// The slice the shape was resolved for, as it was named.
        slice: String,
    },
    /// The code is the crate itself, so the slice's directory is that
    /// crate's `src` and its document is `lib.rs`'s inner doc. No module is
    /// invented to hold a document.
    CrateRoot {
        /// The crate whose `src` is the slice's directory.
        crate_root: PathBuf,
        /// The slice the shape was resolved for, as it was named — the
        /// crate's package name, which is where a crate-root slice's name is
        /// written.
        slice: String,
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
        /// The slice whose presence the directory is, as its own crate names
        /// it.
        slice: String,
    },
    /// No workspace member holds a document for the slice: it has no crate,
    /// so no directory of its own, and its document stays under the
    /// workspace root — the shape of the slices whose product is the
    /// workspace rather than a crate.
    NoCrate {
        /// The slice the shape was resolved for, as it was named.
        slice: String,
    },
}

impl Form {
    /// The shape the slice has in its own crate: the workspace member whose
    /// directory holds the slice's document, in either layout, and then
    /// whether that crate holds a module named for the slice.
    ///
    /// A crate-root slice's `src/lld.md` names no slice, so the member that
    /// answers for one is the member the slice is named for: the slice is the
    /// crate, and the package names it.
    ///
    /// Never `Companion`. A slice's artifacts are resolved from the crate
    /// its document is in — the crate its phases write — so a directory
    /// named for the slice in the companion that holds its claims is never
    /// taken for the slice's own.
    #[implements(
        spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
        spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
        spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
        spec::ASlicesDocumentIsNeverUnderItsCompanion,
    )]
    pub fn of_slice(project: &Project, slice: &str) -> Self {
        match crate_holding(project, slice) {
            Some(crate_root) => form_in(&crate_root, slice),
            None => Self::NoCrate { slice: slice.to_string() },
        }
    }

    /// The shape of a directory, whose name is the slice's: `Companion` when
    /// the manifest of that slice's own crate names this directory's crate as
    /// its companion, and otherwise the shape that slice has in its own
    /// crate.
    ///
    /// The companion is read from `[package.metadata.lid_rs] companion`,
    /// through the resolution the phase policy already makes, and never from
    /// what the directory holds.
    ///
    /// A directory under no workspace member is a directory no crate holds a
    /// document for, which is the shape a slice with no crate has.
    #[implements(
        spec::ACompanionDirectoryIsTheOneTheManifestNames,
        spec::ACompanionIsNeverReadFromADirectorysShape,
    )]
    pub fn of_directory(project: &Project, dir: &Path) -> Self {
        match member_holding(project, dir) {
            Some(crate_root) => directory_form(project, &crate_root, &name_of(dir)),
            None => Self::NoCrate { slice: name_of(dir) },
        }
    }

    /// The slice's directory — where its code is, and so where its `lld.md`
    /// goes: the module's directory under the crate's `src` for a module
    /// slice and for a companion's presence in another crate, that `src`
    /// itself for a crate-root slice, and none for a slice no member holds a
    /// document for, which has no crate to hold a directory.
    #[implements(
        spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
        spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    )]
    fn dir(&self) -> Option<PathBuf> {
        match self {
            Self::Module { crate_root, module, .. } | Self::Companion { crate_root, module, .. } => {
                Some(module_dir(crate_root, module))
            }
            Self::CrateRoot { crate_root, .. } => Some(crate_src(crate_root)),
            Self::NoCrate { .. } => None,
        }
    }

    /// The slice's own crate: the one its phases write and its document is
    /// under. None for a companion directory — one slice's presence in
    /// another crate is not that slice's crate, and the document is never
    /// there — and none for a slice no member holds a document for.
    #[implements(spec::ASlicesDocumentIsNeverUnderItsCompanion)]
    fn own_crate(&self) -> Option<&Path> {
        match self {
            Self::Module { crate_root, .. } | Self::CrateRoot { crate_root, .. } => Some(crate_root),
            Self::Companion { .. } | Self::NoCrate { .. } => None,
        }
    }

    /// The slice the shape was resolved for, as it was named: the name the
    /// old form of its document is built from, which a module's name cannot
    /// supply.
    fn slice(&self) -> &str {
        match self {
            Self::Module { slice, .. }
            | Self::CrateRoot { slice, .. }
            | Self::Companion { slice, .. }
            | Self::NoCrate { slice } => slice,
        }
    }

    /// The slice's document under its own crate, in whichever layout that
    /// crate holds it; none when the slice has no crate of its own to hold
    /// one.
    #[implements(
        spec::ADocumentBesideTheCodeIsTheSlicesLld,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    )]
    fn own_document(&self) -> Option<PathBuf> {
        Some(document_of(self.own_crate()?, &self.dir()?, self.slice()))
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
    Form::of_slice(project, slice).dir().ok_or_else(|| no_crate_refusal(slice))
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
    match Form::of_slice(project, slice).own_document() {
        Some(document) => Ok(document),
        None => Ok(intent_document(&project.root()?, slice)),
    }
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
    matches!(Form::of_directory(project, dir), Form::Companion { .. })
}

// ---- Which member answers for a slice ----------------------------------------

/// The workspace member whose directory holds the slice's document, in either
/// layout: the member holding a document for a slice of this one's module
/// name. None when no member holds one, which is the slice with no crate.
///
/// The match is made on the module name rather than on the slice's, because
/// the three places a member's documents name a slice — the `docs/intent`
/// directory it has not moved, the module directory it has, and the package a
/// crate-root slice is named for — do not agree on hyphens, and a module's
/// name is the form they all reduce to.
#[implements(spec::ASliceNoMemberHoldsIsRefusedByName)]
fn crate_holding(project: &Project, slice: &str) -> Option<PathBuf> {
    let module = module_of(slice);
    project
        .member_manifest_dirs()
        .into_iter()
        .find(|crate_root| slice_with_module(project, crate_root, &module).is_some())
}

/// The shape a slice has in the crate that holds its document: `Module` when
/// that crate holds a module named for the slice, and `CrateRoot` when it
/// holds none, the slice's code being the crate itself.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
)]
fn form_in(crate_root: &Path, slice: &str) -> Form {
    let module = module_of(slice);
    if holds_module(crate_root, &module) {
        Form::Module { crate_root: crate_root.to_path_buf(), module, slice: slice.to_string() }
    } else {
        Form::CrateRoot { crate_root: crate_root.to_path_buf(), slice: slice.to_string() }
    }
}

/// Whether a crate holds a module of that name, in either spelling: a
/// `src/<module>.rs` beside a `src/<module>/`, or a `src/<module>/mod.rs`
/// inside it.
#[implements(spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc)]
fn holds_module(crate_root: &Path, module: &str) -> bool {
    let _ = (crate_root, module);
    todo!()
}

// ---- Which slice a directory is ----------------------------------------------

/// The shape of a directory named `module` under `crate_root`: `Companion`
/// when a proc-macro member's manifest names this crate as its companion and
/// that member holds a slice of this module's name, and otherwise the shape
/// that crate's own slice of that name has.
#[implements(
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
    spec::ACompanionIsNeverReadFromADirectorysShape,
)]
fn directory_form(project: &Project, crate_root: &Path, module: &str) -> Form {
    match companion_slice(project, crate_root, module) {
        Some(slice) => {
            Form::Companion { crate_root: crate_root.to_path_buf(), module: module.to_string(), slice }
        }
        None => held_slice_form(project, crate_root, module),
    }
}

/// The shape of the slice a crate holds a document for under that module
/// name, and `NoCrate` when it holds none — a directory that is no slice's,
/// whatever files it holds.
#[implements(spec::ACompanionIsNeverReadFromADirectorysShape)]
fn held_slice_form(project: &Project, crate_root: &Path, module: &str) -> Form {
    match slice_with_module(project, crate_root, module) {
        Some(slice) => form_in(crate_root, &slice),
        None => Form::NoCrate { slice: module.to_string() },
    }
}

/// The slice whose presence in this crate a directory of that module name is:
/// the slice of that name held by the member whose manifest names this crate
/// as its companion. None when no member names it, and none when the one that
/// does holds no slice of that name.
#[implements(spec::ACompanionDirectoryIsTheOneTheManifestNames)]
fn companion_slice(project: &Project, crate_root: &Path, module: &str) -> Option<String> {
    slice_with_module(project, &companion_owner(project, crate_root)?, module)
}

/// The member whose `[package.metadata.lid_rs] companion` names this crate:
/// the proc-macro crate whose slice's claims this crate holds. Resolved
/// through the phase policy's own reading of that key, so that one key has one
/// reader, and so that a key naming a crate that cannot hold claims answers
/// for no directory.
#[implements(
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
    spec::ACompanionIsNeverReadFromADirectorysShape,
)]
fn companion_owner(project: &Project, companion_crate: &Path) -> Option<PathBuf> {
    let _ = (project, companion_crate);
    todo!()
}

/// The slice a crate holds a document for whose module is `module`, if it
/// holds one. The route from a directory back to a slice is forward — every
/// slice the crate's documents name, compared by module name — and never an
/// inversion of `module_of`, which is not one to one
/// (§ "Resolving a directory back to a slice").
fn slice_with_module(project: &Project, crate_root: &Path, module: &str) -> Option<String> {
    slices_of(project, crate_root).into_iter().find(|slice| module_of(slice) == module)
}

/// Every slice a crate holds a document for, each named as the document that
/// marks it names it: the `docs/intent` directories it has not moved, whose
/// names are the slices' own; the `src` directories it has, whose names are
/// the slices' modules; and the package name, when the crate's `src/lld.md`
/// makes the crate itself a slice.
fn slices_of(project: &Project, crate_root: &Path) -> Vec<String> {
    let named_by_a_document =
        [slice_names_in(&crate_root.join(INTENT_DIR)), slice_names_in(&crate_src(crate_root))].concat();
    named_by_a_document.into_iter().chain(crate_root_slice(project, crate_root)).collect()
}

/// The names of the directories directly under `dir` that hold an `lld.md` —
/// the marker rule applied to a tree. A directory that cannot be read holds
/// none, which is a crate with no documents of that form rather than a fault.
fn slice_names_in(dir: &Path) -> Vec<String> {
    let _ = dir;
    todo!()
}

/// The slice a crate's own `src/lld.md` is the document of: the crate's
/// package name, that file naming no slice itself. None when the crate holds
/// no such file, and none when it is no package of this workspace's.
#[implements(spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor)]
fn crate_root_slice(project: &Project, crate_root: &Path) -> Option<String> {
    let _ = (project, crate_root);
    todo!()
}

/// The workspace member a directory lies under, if any: the crate whose
/// manifest directory is a prefix of it.
fn member_holding(project: &Project, dir: &Path) -> Option<PathBuf> {
    let _ = (project, dir);
    todo!()
}

// ---- The paths and the names -------------------------------------------------

/// The slice's document while both layouts stand: the `lld.md` in its
/// directory when that file is there, and otherwise the
/// `docs/intent/<slice>/lld.md` its crate still holds. Neither is a failure —
/// a mixed tree is the migration's normal state — and the file's presence, not
/// the crate's, is what tells them apart, because the migration moves one
/// slice at a time.
#[implements(
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
)]
fn document_of(crate_root: &Path, dir: &Path, slice: &str) -> PathBuf {
    let beside_the_code = document_in(dir);
    if beside_the_code.is_file() { beside_the_code } else { intent_document(crate_root, slice) }
}

/// The `lld.md` a directory holds: the file whose presence marks that
/// directory a slice's, and the document of the slice whose code is there.
#[implements(spec::ADocumentBesideTheCodeIsTheSlicesLld)]
fn document_in(dir: &Path) -> PathBuf {
    let _ = dir;
    todo!()
}

/// The `docs/intent/<slice>/lld.md` under a directory: the form a slice's
/// document keeps under its crate until the migration moves it, and the form a
/// slice with no crate keeps under the workspace root for good.
#[implements(
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
)]
fn intent_document(under: &Path, slice: &str) -> PathBuf {
    let _ = (under, slice);
    todo!()
}

/// A crate's `src`: the directory a crate-root slice's is, and the one every
/// module slice's sits under.
#[implements(spec::ACrateRootSlicesDirectoryIsItsCratesSrc)]
fn crate_src(crate_root: &Path) -> PathBuf {
    let _ = crate_root;
    todo!()
}

/// A module's directory under a crate: `src/<module>`, which is the slice's
/// directory where the module is named for a slice.
#[implements(spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc)]
fn module_dir(crate_root: &Path, module: &str) -> PathBuf {
    let _ = (crate_root, module);
    todo!()
}

/// A slice's name as a module's: hyphens as underscores. The form the three
/// places that name a slice — its `docs/intent` directory, its module
/// directory, and its package — all reduce to.
fn module_of(slice: &str) -> String {
    let _ = slice;
    todo!()
}

/// The name a directory carries: its last component, and the empty name for a
/// path with none, which no slice and no module has.
fn name_of(dir: &Path) -> String {
    let _ = dir;
    todo!()
}

/// The refusal for a slice no workspace member holds a document for: it names
/// the slice, both forms a member's document would have been found in — the
/// `docs/intent/<slice>/lld.md` the migration has not moved, and the `lld.md`
/// beside the code of a module or a crate named for it — and what follows from
/// finding neither, which is that the slice has no crate and so no phase agent
/// can run it.
#[implements(spec::ASliceNoMemberHoldsIsRefusedByName)]
fn no_crate_refusal(slice: &str) -> String {
    let _ = slice;
    todo!()
}
