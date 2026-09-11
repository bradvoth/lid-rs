#![doc = include_str!("lld.md")]
use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::phase::policy;
use crate::project::Project;

/// The directory a slice's document sits under before the migration moves it
/// beside the code — and for good, for a slice whose product is the workspace
/// and which therefore has no code to sit beside.
const INTENT_DIR: &str = "docs/intent";

/// The module a crate's claims register from, in both spellings a module has:
/// the `spec.rs` beside a slice's code, and the `spec` directory a crate holds
/// every slice's claims in until the migration moves them.
const SPEC_MODULE: &str = "spec";

/// The crate a claims file is answered relative to: none at all. Which crate
/// holds a slice's claims — its own, or the companion its manifest names — is
/// the caller's to say, so the paths answered start from a root with no
/// components and the caller joins them onto the crate it means.
const NO_CRATE: &str = "";

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
    ///
    /// Every resolver below answers from this shape, so every claim about a
    /// slice's crate, about its directory and about its document turns on it:
    /// an answer for a slice no member holds is a refusal that never comes, and
    /// a crate answered for a slice that has none is a document placed under a
    /// crate rather than under the workspace root.
    #[implements(
        spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
        spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
        spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
        spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
        spec::ASliceNoMemberHoldsIsRefusedByName,
        spec::ADocumentBesideTheCodeIsTheSlicesLld,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
        spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
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
    ///
    /// It is also the directory a slice's document is looked for in, so which
    /// form of that document is answered turns on it, and the `None` is what
    /// `slice_dir` turns into the refusal.
    #[implements(
        spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
        spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
        spec::ASliceNoMemberHoldsIsRefusedByName,
        spec::ADocumentBesideTheCodeIsTheSlicesLld,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
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
    ///
    /// This is the crate every form of the document is built under, so the
    /// document claims turn on it, and it is the crate the `own_crate` door
    /// below answers with — its `None` being what that door turns into the
    /// refusal, as `dir`'s is for `slice_dir`.
    #[implements(
        spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
        spec::ASliceNoMemberHoldsIsRefusedByName,
        spec::ADocumentBesideTheCodeIsTheSlicesLld,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
        spec::ASlicesDocumentIsNeverUnderItsCompanion,
    )]
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
    /// one — a slice no member holds a document for, and a directory read as
    /// a companion's presence rather than as a slice.
    #[implements(
        spec::ADocumentBesideTheCodeIsTheSlicesLld,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
        spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
        spec::ASlicesDocumentIsNeverUnderItsCompanion,
    )]
    fn own_document(&self) -> Option<PathBuf> {
        Some(document_of(self.own_crate()?, &self.dir()?, self.slice()))
    }

    /// The slice's claims file, relative to no crate: the `spec.rs` in the
    /// slice's directory once that directory holds the document, the
    /// `src/spec/<module>.rs` its crate holds every slice's claims at until
    /// then, and the `src/spec.rs` beside a crate-root slice's code in either
    /// layout, that slice's directory being its crate's `src`.
    ///
    /// The directory whose document is read is under the slice's own crate;
    /// the path answered is under no crate. None where the slice has no crate
    /// of its own to read a layout from — a slice no member holds a document
    /// for, and a directory read as a companion's presence rather than as a
    /// slice — which is what the door turns into the refusal.
    #[implements(
        spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec,
        spec::ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode,
        spec::ASliceNoMemberHoldsIsRefusedByName,
    )]
    fn spec_file(&self) -> Option<PathBuf> {
        match self {
            Self::Module { module, .. } => Some(module_spec_file(&self.dir()?, module)),
            Self::CrateRoot { .. } => Some(claims_in(&crate_src(Path::new(NO_CRATE)))),
            Self::Companion { .. } | Self::NoCrate { .. } => None,
        }
    }

    /// A named file of the slice's intent, under the slice's own crate: the
    /// file of that name in the slice's directory once that directory holds
    /// the document, and the one under `docs/intent/<slice>` in that crate
    /// until it does.
    ///
    /// None where the slice has no crate of its own, which is what the door
    /// turns into the refusal: an intent file has no form under the workspace
    /// root and none under a companion, a companion's directory carrying no
    /// document for one to sit beside.
    #[implements(
        spec::ANamedFileBesideTheDocumentIsTheSlicesIntentFile,
        spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
        spec::ASliceNoMemberHoldsIsRefusedByName,
    )]
    fn intent_file(&self, name: &str) -> Option<PathBuf> {
        Some(intent_file_in(self.own_crate()?, &self.dir()?, self.slice(), name))
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

/// A slice's own crate: the workspace member whose directory holds the slice's
/// document, in either layout — the crate the slice's code is under, whose
/// phases write it — and the refusal naming the slice when no member holds a
/// document for it at all.
///
/// Never the companion crate that holds a slice's claims and the module citing
/// them: a slice has one crate of its own, the one its document is in, and a
/// directory named for the slice elsewhere is that slice's presence in another
/// crate rather than its crate.
///
/// This is the door a caller outside this module asks a slice's crate through.
/// One that matched `Form`'s variants for itself would hold a second copy of
/// the decision the four shapes are told apart by, and would have to rebuild
/// the refusal sentence — one rule in two places, which is what having a single
/// place the layout is computed exists to prevent.
#[implements(
    spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    spec::ASliceNoMemberHoldsIsRefusedByName,
)]
pub fn own_crate(project: &Project, slice: &str) -> Result<PathBuf, String> {
    Form::of_slice(project, slice).own_crate().map(Path::to_path_buf).ok_or_else(|| no_crate_refusal(slice))
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

/// A slice's claims file, as a path relative to no crate: the
/// `src/<module>/spec.rs` beside the slice's document once that document sits
/// beside its code, the `src/spec/<module>.rs` a crate holds it at until then,
/// the `src/spec.rs` a crate-root slice's claims are in under either layout,
/// and the refusal naming the slice when no workspace member holds a document
/// for it.
///
/// The third answer is not a case inside the first two: a crate-root slice's
/// code is its crate, so it has no directory named for it to hold a `spec.rs`
/// beside a document, and the `src/spec/<module>.rs` a slice's name computes
/// is a file its crate does not hold. Which of the two layouts such a slice is
/// in is a question its claims file does not turn on.
///
/// Answering this takes two crates, and they are the same crate only for an
/// ordinary slice. The crate whose layout is *read* — whose directory is asked
/// whether it holds `lld.md` — is always the slice's own, because a slice's
/// document is never under its companion; this door resolves that crate
/// itself, by the resolution `own_crate` makes, and refuses with the same one
/// sentence when no member holds one. The crate the answer is *joined onto* is
/// the one holding the claims, which for a proc-macro slice is the companion,
/// and which crate that is the phase policy's companion rule decides and not
/// this module. So the answer is placed by the caller, which is the only party
/// that knows which of the two it means.
///
/// A door reading the layout from the crate it was handed would take the
/// claims crate for the reading crate wherever a caller has only that one, and
/// a companion holds no `lld.md`: it would answer the pre-migration path for
/// every proc-macro slice, for good and without saying so.
///
/// Which layout the slice is in is read from where its document is, and never
/// from whether the file answered with is there. Phase 2 asks this door where
/// a new slice's claims are to be written, and a file about to be written
/// exists in neither layout.
#[implements(
    spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec,
    spec::ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode,
    spec::ASliceNoMemberHoldsIsRefusedByName,
)]
pub fn spec_file(project: &Project, slice: &str) -> Result<PathBuf, String> {
    Form::of_slice(project, slice).spec_file().ok_or_else(|| no_crate_refusal(slice))
}

/// A named file of a slice's intent — the human's acceptance of a compile-time
/// slice, and the slice's document itself: the file of that name in the
/// slice's directory once that directory holds `lld.md`, the one under
/// `docs/intent/<slice>` in the slice's crate until it does, and the refusal
/// naming the slice when no workspace member holds a document for it.
///
/// This door resolves the same crate the one above reads — a slice's own, the
/// only crate its document is ever under — and the two differ only in what
/// they answer with. This one answers a path in that crate rather than a
/// relative one for the caller to place, because an intent file has no
/// companion form: a companion directory carries no `lld.md` and so no
/// acceptance beside one, and a slice's intent is in the crate its phases
/// write. There is no second crate for a caller to mean.
///
/// A slice whose product is the workspace reaches the refusal here. The
/// workspace-root document such a slice has is `lld_path`'s answer beside this
/// one and not a case inside it, because no intent file but the document has a
/// workspace-root form to answer with.
///
/// Which layout the slice is in is read from where its document is, and never
/// from whether the file answered with is there: the acceptance a human is
/// asked to write does not exist at the moment the path to it is named.
#[implements(
    spec::ANamedFileBesideTheDocumentIsTheSlicesIntentFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
    spec::ASliceNoMemberHoldsIsRefusedByName,
)]
pub fn intent_file(project: &Project, slice: &str, name: &str) -> Result<PathBuf, String> {
    Form::of_slice(project, slice).intent_file(name).ok_or_else(|| no_crate_refusal(slice))
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
///
/// Which member answers is what every successful resolution is built from, so
/// this decision keeps every claim `Form::of_slice` does: the crate answered
/// for a slice is the member found here, the directory is under it, the
/// document is under it in one of its two forms, and answering none is the
/// refusal and the workspace-root document both.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
    spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    spec::ASliceNoMemberHoldsIsRefusedByName,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
    spec::ASlicesDocumentIsNeverUnderItsCompanion,
)]
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
///
/// The shape chosen here decides which directory of that crate is the slice's,
/// and so which directory its document is looked for beside.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
)]
fn form_in(crate_root: &Path, slice: &str) -> Form {
    let module = module_of(slice);
    if holds_module(crate_root, &module) {
        Form::Module { crate_root: crate_root.to_path_buf(), module, slice: slice.to_string() }
    } else {
        Form::CrateRoot { crate_root: crate_root.to_path_buf(), slice: slice.to_string() }
    }
}

/// Whether a crate holds a module of that name: a `src/<module>.rs` beside a
/// `src/<module>/`, a `src/<module>/mod.rs` inside it, or a
/// `src/<module>/lld.md` marking the directory as that slice's before either
/// file exists.
///
/// This is the whole of the module-versus-crate-root decision: answering
/// `true` for a crate that holds no such module makes that crate's slice a
/// module slice, and answering `false` for one that does makes a module
/// slice's directory its crate's `src`. Both are wrong in the other claim's
/// terms, and both move the directory the slice's document is looked for in.
///
/// The marker is the third spelling because the two code spellings are
/// written by Phase 3, and every phase before it would otherwise read a new
/// module slice as its crate's root: `crate_holding` finds the crate through
/// `slices_of`, which takes a `src/<module>/lld.md` as naming a module slice,
/// and this answering `false` for the same directory made the two halves of
/// one resolver disagree about the slice the other had just found. Phase 2
/// was then offered the crate-root slice's claims file — another slice's —
/// as the only path it could write. Reading the marker here is the same rule
/// `slices_of` already applies, asked at the point the shape is decided.
// Every door routes through the form this decides, so a wrong answer here
// can make any of the three doors' claims false — which is why the list is
// this long. It was the directory's two and the document's one until check 12
// found the `||` survivable: the citation was narrower than the reach, so the
// tests that do exercise this were not in its set (README §4.3).
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec,
    spec::ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode,
    spec::ANamedFileBesideTheDocumentIsTheSlicesIntentFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
)]
fn holds_module(crate_root: &Path, module: &str) -> bool {
    crate_src(crate_root).join(format!("{module}.rs")).is_file()
        || module_dir(crate_root, module).join("mod.rs").is_file()
        || document_in(&module_dir(crate_root, module)).is_file()
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
///
/// This cites no claim. It answers `Module`, `CrateRoot` or `NoCrate` and
/// never `Companion`, so no wrong answer from it can make `is_companion_dir`
/// true, and a citation of `ACompanionIsNeverReadFromADirectorysShape` here
/// could not be contradicted — the decorative kind README §4.3 names. Check 12
/// is silent about it only because `Form` has no `Default` and so no mutant is
/// generated; that silence is not evidence. The claim is kept by
/// `companion_slice`, whose `None` is what a wrong answer would have to change.
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
///
/// Both answers are load-bearing: the `Some` is what makes a directory a
/// companion's, and the `None` is the only thing that keeps a directory of a
/// companion's shape — under a crate no manifest names, or named for a slice
/// the naming member does not hold — from being read as one.
#[implements(
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
    spec::ACompanionIsNeverReadFromADirectorysShape,
)]
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
    project.member_manifest_dirs().into_iter().find(|member| {
        policy::companion(project, member).ok().flatten().as_deref() == Some(companion_crate)
    })
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
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| document_in(&entry.path()).is_file())
        .map(|entry| name_of(&entry.path()))
        .collect()
}

/// The slice a crate's own `src/lld.md` is the document of: the crate's
/// package name, that file naming no slice itself. None when the crate holds
/// no such file, and none when it is no package of this workspace's.
///
/// The package name is what keeps this document from answering for every
/// slice the crate holds no module for: a migrated crate-root slice read as a
/// candidate by name alone would leave no slice unheld and no refusal to
/// make. And it is the only route by which a migrated crate-root slice is
/// found at all, so its crate, its directory and its document all turn on it.
#[implements(
    spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    spec::ASliceNoMemberHoldsIsRefusedByName,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
)]
fn crate_root_slice(project: &Project, crate_root: &Path) -> Option<String> {
    document_in(&crate_src(crate_root))
        .is_file()
        .then(|| project.package_at(&crate_root.join("Cargo.toml")))
        .flatten()
}

/// The workspace member a directory lies under, if any: the crate whose
/// manifest directory is a prefix of it.
fn member_holding(project: &Project, dir: &Path) -> Option<PathBuf> {
    project.member_manifest_dirs().into_iter().find(|crate_root| dir.starts_with(crate_root))
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

/// A named file of a slice's intent under the slice's crate, while both
/// layouts stand: the file of that name in the slice's directory when that
/// directory holds the document, and the one under `docs/intent/<slice>` when
/// it does not.
///
/// The document is what says which layout the slice is in; the presence of the
/// file named says nothing, and is not looked at. An acceptance a human is
/// asked to write, and a claims file a phase is about to write, exist in
/// neither layout at the moment they are named.
///
/// A slice's document is a named file of its intent like any other, and
/// `document_of` beside this one is this shape with `lld.md` for a name. The
/// two stand apart while the old forms differ and `lld_path` carries the
/// workspace-root answer no other intent file has.
#[implements(
    spec::ANamedFileBesideTheDocumentIsTheSlicesIntentFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
)]
fn intent_file_in(crate_root: &Path, dir: &Path, slice: &str, name: &str) -> PathBuf {
    if document_in(dir).is_file() { dir.join(name) } else { intent_dir(crate_root, slice).join(name) }
}

/// A module slice's claims file, relative to no crate: the `spec.rs` beside
/// the slice's document once its directory holds one, and the
/// `src/spec/<module>.rs` its crate holds every slice's claims at until then.
///
/// The directory is read for the document and for nothing else: it is the
/// slice's own crate's, while the answer is under no crate, because which
/// crate holds a module slice's claims is the caller's to say.
#[implements(
    spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec,
)]
fn module_spec_file(dir: &Path, module: &str) -> PathBuf {
    if document_in(dir).is_file() {
        claims_in(&module_dir(Path::new(NO_CRATE), module))
    } else {
        crate_src(Path::new(NO_CRATE)).join(SPEC_MODULE).join(format!("{module}.rs"))
    }
}

/// The `lld.md` a directory holds: the file whose presence marks that
/// directory a slice's, and the document of the slice whose code is there.
///
/// The marker is the one fact this slice rests on, so it keeps every claim
/// but the two negative ones. Which member holds a slice, which directory is
/// that slice's, and which path each form of its document has are all read
/// through this name; a companion's directory is held apart from a slice's by
/// not holding this file. The refusal and the not-a-companion claim are the
/// exceptions, because a marker that named the wrong file could only find
/// fewer slices, and both of those claims are what is answered when none is
/// found.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
    spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
    spec::ASlicesDocumentIsNeverUnderItsCompanion,
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
)]
fn document_in(dir: &Path) -> PathBuf {
    dir.join("lld.md")
}

/// The claims file a directory holds: the `spec.rs` beside the code that
/// directory is of, which is the slice's directory for a module slice and the
/// crate's `src` for a crate-root slice, whose code is its crate.
///
/// One name answers both because a slice's claims sit beside its code in
/// either shape; what differs is which directory the code is in, and that is
/// `dir`'s question and not this one's.
#[implements(
    spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile,
    spec::ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode,
)]
fn claims_in(dir: &Path) -> PathBuf {
    dir.join(format!("{SPEC_MODULE}.rs"))
}

/// The `docs/intent/<slice>/lld.md` under a directory: the form a slice's
/// document keeps under its crate until the migration moves it, and the form a
/// slice with no crate keeps under the workspace root for good.
#[implements(
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
)]
fn intent_document(under: &Path, slice: &str) -> PathBuf {
    document_in(&intent_dir(under, slice))
}

/// The `docs/intent/<slice>` directory under a directory: the directory a
/// slice's intent files are in until the migration moves them beside its code,
/// and the one a slice with no crate keeps under the workspace root for good.
///
/// The slice is named as it was asked for, hyphens and all, this directory
/// being one of the three places the tree names a slice and the only one that
/// spells it as the slice is spelled.
#[implements(
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath,
    spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot,
    spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent,
)]
fn intent_dir(under: &Path, slice: &str) -> PathBuf {
    under.join(INTENT_DIR).join(slice)
}

/// A crate's `src`: the directory a crate-root slice's is, and the one every
/// module slice's sits under.
///
/// It is also where a crate's migrated slices are enumerated from — its own
/// and the ones it holds for the proc-macro crate naming it — so which member
/// answers for a slice turns on it as well as where that slice's directory is.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor,
    spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
    spec::ACompanionDirectoryIsTheOneTheManifestNames,
)]
fn crate_src(crate_root: &Path) -> PathBuf {
    crate_root.join("src")
}

/// A module's directory under a crate: `src/<module>`, which is the slice's
/// directory where the module is named for a slice.
///
/// It is also half of the test for whether a crate holds a module at all, so
/// a crate-root slice's directory turns on it too, and it is the directory a
/// module slice's document is looked for beside.
#[implements(
    spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc,
    spec::ACrateRootSlicesDirectoryIsItsCratesSrc,
    spec::ADocumentBesideTheCodeIsTheSlicesLld,
)]
fn module_dir(crate_root: &Path, module: &str) -> PathBuf {
    crate_src(crate_root).join(module)
}

/// A slice's name as a module's: hyphens as underscores. The form the three
/// places that name a slice — its `docs/intent` directory, its module
/// directory, and its package — all reduce to.
fn module_of(slice: &str) -> String {
    slice.replace('-', "_")
}

/// The name a directory carries: its last component, and the empty name for a
/// path with none, which no slice and no module has.
fn name_of(dir: &Path) -> String {
    dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The refusal for a slice no workspace member holds a document for: it names
/// the slice, both forms a member's document would have been found in — the
/// `docs/intent/<slice>/lld.md` the migration has not moved, and the `lld.md`
/// beside the code of a module or a crate named for it — and what follows from
/// finding neither, which is that the slice has no crate and so no phase agent
/// can run it.
#[implements(spec::ASliceNoMemberHoldsIsRefusedByName)]
fn no_crate_refusal(slice: &str) -> String {
    format!(
        "no workspace package holds `{INTENT_DIR}/{slice}/lld.md`, and none holds an `lld.md` beside the code of a \
         module or a crate named `{slice}`: `{slice}` has no crate, so no phase agent can run it"
    )
}

#[cfg(test)]
mod tests {
    //! One fixture workspace holds a slice of every shape in both layouts, so
    //! that each resolver is asked the tree the migration actually presents:
    //! some slices moved beside their code, some not.
    //!
    //! | Member | Slice | Shape | Where its document is |
    //! |---|---|---|---|
    //! | `app` | `alpha` | module | old — `app/docs/intent/alpha/lld.md` |
    //! | `app` | `lld-review` | module, hyphenated | old |
    //! | `app` | `beta` | module | new — `app/src/beta/lld.md` |
    //! | `mac` | `two-phase` | module, companion in `app` | old |
    //! | `mac` | `delta` | module, companion in `app` | new |
    //! | `tool` | `tool` | crate root | old — `tool/docs/intent/tool/lld.md` |
    //! | `rooted` | `rooted` | crate root | new — `rooted/src/lld.md` |
    //! | — | `book` | no crate | `docs/intent/book/lld.md` at the root |
    //!
    //! `mac` is the proc-macro member whose manifest names `app` as its
    //! companion, so `app/src/two_phase` and `app/src/delta` are companion
    //! directories: a `spec.rs`, a `mod.rs`, and no `lld.md`. Two directories
    //! of exactly that shape are no companion's — `app/src/skipped`, which
    //! `mac` holds no slice for, and `tool/src/two_phase`, under a crate no
    //! manifest names — which is what holds the manifest apart from the shape.
    //!
    //! Two of the fixture's members carry facts a test would otherwise be
    //! silently vacuous without, and both are asserted where they matter:
    //! `rooted` has already migrated, so a resolver taking an unnamed
    //! `src/lld.md` for any slice's would answer for a slice no member holds;
    //! and the companion directories hold no `lld.md`, which is the whole
    //! reason a slice's document is never found under its companion.
    //!
    //! `mac` and `app` are also what make the two crates a claims file takes
    //! tellable apart. `delta`'s document is `mac`'s and its claims are
    //! `app`'s, and `mac/src/delta` holds an `lld.md` where `app/src/delta`
    //! holds none — so a door reading the layout from the crate it is joined
    //! onto answers the pre-migration form for every proc-macro slice, for
    //! good, and a test asked only about an ordinary slice cannot see it.
    //!
    //! The manifests exist only in the metadata document — nothing here parses
    //! one and cargo is never run — while every path the resolvers read is on
    //! disk.

    use super::*;
    use lid_rs::validates;

    use crate::phase::fixture;

    /// The fixture's members, as `(package, target kind, companion)`, in the
    /// order the resolvers search them. `app` comes first: a resolver that
    /// took a companion directory for a slice's own would answer `app` for
    /// `two-phase` and `delta` before reaching `mac`.
    const MEMBERS: &[(&str, &str, Option<&str>)] =
        &[("app", "lib", None), ("mac", "proc-macro", Some("app")), ("tool", "lib", None), ("rooted", "lib", None)];

    /// Every file of the fixture tree, each with the content its kind would
    /// carry. A directory is made by the file written into it.
    const TREE: &[(&str, &str)] = &[
        // The slice whose product is the workspace: no member holds it.
        ("docs/intent/book/lld.md", "# book\n"),
        // `app`: two slices in the old layout, one moved, and the companion
        // directories of `mac`'s two.
        ("app/docs/intent/alpha/lld.md", "# alpha\n"),
        ("app/src/alpha.rs", "//! The alpha slice.\n"),
        ("app/src/alpha/leaf.rs", "//! A leaf of alpha.\n"),
        ("app/docs/intent/lld-review/lld.md", "# lld-review\n"),
        ("app/src/lld_review.rs", "//! The lld-review slice.\n"),
        ("app/src/lld_review/leaf.rs", "//! A leaf of lld-review.\n"),
        ("app/src/beta/lld.md", "# beta\n"),
        ("app/src/beta/mod.rs", "//! The beta slice.\n"),
        ("app/src/two_phase/spec.rs", "//! Claims of two-phase.\n"),
        ("app/src/two_phase/mod.rs", "//! Two-phase's presence in app.\n"),
        ("app/src/delta/spec.rs", "//! Claims of delta.\n"),
        ("app/src/delta/mod.rs", "//! Delta's presence in app.\n"),
        // A companion's shape under no companion relation: `mac` holds no
        // slice `skipped`, so this is a slice whose Phase 1 was skipped.
        ("app/src/skipped/spec.rs", "//! Claims of skipped.\n"),
        ("app/src/skipped/mod.rs", "//! The skipped slice.\n"),
        // `mac`: the proc-macro crate, one slice in each layout.
        ("mac/docs/intent/two-phase/lld.md", "# two-phase\n"),
        ("mac/src/two_phase.rs", "//! The two-phase slice.\n"),
        ("mac/src/two_phase/leaf.rs", "//! A leaf of two-phase.\n"),
        ("mac/src/delta/lld.md", "# delta\n"),
        ("mac/src/delta/mod.rs", "//! The delta slice.\n"),
        // `tool`: a crate-root slice that has not moved, and a companion's
        // shape in a crate no manifest names as one.
        ("tool/docs/intent/tool/lld.md", "# tool\n"),
        ("tool/src/lib.rs", "//! The tool crate.\n"),
        ("tool/src/two_phase/spec.rs", "//! Not two-phase's claims.\n"),
        ("tool/src/two_phase/mod.rs", "//! Not two-phase's module.\n"),
        // `rooted`: a crate-root slice that has, its document naming no slice.
        ("rooted/src/lld.md", "# rooted\n"),
        ("rooted/src/lib.rs", "//! The rooted crate.\n"),
    ];

    /// A named file of a slice's intent that no layout of the fixture holds:
    /// the human's acceptance of a compile-time slice, which is asked for
    /// before it is written and so is never there to be found.
    const ACCEPTANCE: &str = "compile-time-accepted";

    /// The fixture tree at a scratch root of its own, with the project whose
    /// metadata describes it.
    fn workspace(name: &str) -> (PathBuf, Project) {
        let root = fixture::scratch(name).canonicalize().expect("the scratch root");
        for (relative, content) in TREE {
            write_at(&root, relative, content);
        }
        (root.clone(), Project::from_json(&metadata(&root, MEMBERS)).expect("the metadata document parses"))
    }

    /// Writes one of the fixture's files, creating the directories above it.
    fn write_at(root: &Path, relative: &str, content: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("the path has a parent")).expect("create the directories");
        std::fs::write(path, content).expect("write the file");
    }

    /// The `cargo metadata` document for a workspace at `root` with those
    /// members, each rooted at the directory its package is named for.
    fn metadata(root: &Path, members: &[(&str, &str, Option<&str>)]) -> String {
        let packages: Vec<String> = members.iter().map(|member| package(root, member)).collect();
        format!(
            r#"{{"workspace_root":"{}","target_directory":"{}","packages":[{}]}}"#,
            root.display(),
            root.join("target").display(),
            packages.join(",")
        )
    }

    /// One member's package node: its manifest directory, its one target's
    /// kind, and the `[package.metadata.lid_rs] companion` key when it names
    /// one.
    fn package(root: &Path, member: &(&str, &str, Option<&str>)) -> String {
        let (name, kind, companion) = *member;
        let metadata = companion.map_or_else(
            || "null".to_string(),
            |companion| format!(r#"{{"lid_rs":{{"companion":"{companion}"}}}}"#),
        );
        format!(
            r#"{{"name":"{name}","manifest_path":"{}","metadata":{metadata},"targets":[{{"kind":["{kind}"],"name":"{name}"}}]}}"#,
            root.join(name).join("Cargo.toml").display()
        )
    }

    #[test]
    #[validates(spec::AModuleSlicesDirectoryIsTheOneNamedForItUnderSrc)]
    fn a_module_slices_directory_is_the_one_named_for_it_under_src() {
        let (root, project) = workspace("layout-module-directory");
        let found = ["alpha", "beta", "lld-review", "delta"].map(|slice| slice_dir(&project, slice));
        assert_eq!(
            found,
            [
                // `app/src/alpha.rs` beside `app/src/alpha/`, and the
                // migrated spelling, `app/src/beta/mod.rs`.
                Ok(root.join("app/src/alpha")),
                Ok(root.join("app/src/beta")),
                // The directory is the module's name, not the slice's.
                Ok(root.join("app/src/lld_review")),
                // Under the slice's own crate, never under its companion,
                // although `app/src/delta` exists.
                Ok(root.join("mac/src/delta")),
            ]
        );
    }

    #[test]
    #[validates(spec::ACrateRootSlicesDirectoryIsItsCratesSrc)]
    fn a_crate_root_slices_directory_is_its_crates_src() {
        let (root, project) = workspace("layout-crate-root-directory");
        // Neither crate holds a module named for its slice, in either layout:
        // the directory is the crate's `src`, and no module is invented.
        assert_eq!(
            (slice_dir(&project, "tool"), slice_dir(&project, "rooted")),
            (Ok(root.join("tool/src")), Ok(root.join("rooted/src")))
        );
    }

    #[test]
    #[validates(spec::ACrateRootSlicesCrateIsTheMemberItIsNamedFor)]
    fn a_crate_root_slices_crate_is_the_member_it_is_named_for() {
        let (root, project) = workspace("layout-crate-root-crate");
        // `rooted/src/lld.md` records no slice name, so the member that
        // answers for it is the one the slice is named for — the package.
        assert_eq!(
            Form::of_slice(&project, "rooted"),
            Form::CrateRoot { crate_root: root.join("rooted"), slice: "rooted".to_string() }
        );
        // And answers for no other name: a member holding an unnamed
        // `src/lld.md` is not a candidate for every slice it has no module for.
        assert_eq!(Form::of_slice(&project, "nobody"), Form::NoCrate { slice: "nobody".to_string() });
    }

    #[test]
    #[validates(spec::ASlicesOwnCrateIsTheMemberHoldingItsDocument)]
    fn a_slices_own_crate_is_the_member_holding_its_document() {
        let (root, project) = workspace("layout-own-crate");
        // The door is asked directly, for a slice of every shape and in both
        // layouts: what it answers is observable nowhere else.
        let found = ["alpha", "beta", "lld-review", "tool", "rooted", "two-phase", "delta"]
            .map(|slice| own_crate(&project, slice));
        assert_eq!(
            found,
            [
                // A module slice's crate, its document in either layout, and
                // whether or not the slice's name is its module's.
                Ok(root.join("app")),
                Ok(root.join("app")),
                Ok(root.join("app")),
                // A crate-root slice's crate is the member it is named for.
                Ok(root.join("tool")),
                Ok(root.join("rooted")),
                // The member holding the document, never the companion that
                // holds the slice's claims — and `app` is searched first.
                Ok(root.join("mac")),
                Ok(root.join("mac")),
            ]
        );
    }

    #[test]
    #[validates(spec::ASliceNoMemberHoldsIsRefusedByName)]
    fn a_slice_no_member_holds_is_refused_by_name() {
        let (root, project) = workspace("layout-refusal");
        // The fixture holds a member that has already migrated. A resolver
        // accepting a non-matching `src/lld.md` would answer `rooted` below,
        // and this refusal would stop firing for any slice at all.
        assert!(root.join("rooted/src/lld.md").is_file(), "the fixture holds a migrated crate-root slice");

        let unheld = slice_dir(&project, "nobody").expect_err("no member holds a document named `nobody`");
        let workspace_only = slice_dir(&project, "book").expect_err("`book`'s document is the workspace root's");
        assert_eq!(
            (unheld.contains("nobody"), unheld.contains("docs/intent/nobody/lld.md"), unheld.contains("no crate")),
            (true, true, true),
            "the refusal names the slice, the form its document would have had, and what follows: {unheld}"
        );
        assert!(workspace_only.contains("book"), "{workspace_only}");
    }

    #[test]
    #[validates(spec::ASliceNoMemberHoldsIsRefusedByName)]
    fn a_slice_no_member_holds_is_refused_by_name_at_own_crate() {
        let (_root, project) = workspace("layout-own-crate-refusal");
        // The second resolver that needs a slice's crate refuses in the same
        // one sentence, and the refusing is observed at the door rather than
        // through another resolver that reaches it.
        let unheld = own_crate(&project, "nobody").expect_err("no member holds a document named `nobody`");
        let workspace_only = own_crate(&project, "book").expect_err("`book`'s document is the workspace root's");
        assert_eq!(
            (unheld.contains("nobody"), unheld.contains("docs/intent/nobody/lld.md"), unheld.contains("no crate")),
            (true, true, true),
            "the refusal names the slice, the form its document would have had, and what follows: {unheld}"
        );
        assert!(workspace_only.contains("book"), "{workspace_only}");
    }

    #[test]
    #[validates(spec::ADocumentBesideTheCodeIsTheSlicesLld)]
    fn a_document_beside_the_code_is_the_slices_lld() {
        let (root, project) = workspace("layout-document-beside-the-code");
        // The three shapes that have moved: a module slice, a module slice
        // whose claims are a companion's, and a crate-root slice.
        let found = ["beta", "delta", "rooted"].map(|slice| lld_path(&project, slice));
        assert_eq!(
            found,
            [
                Ok(root.join("app/src/beta/lld.md")),
                Ok(root.join("mac/src/delta/lld.md")),
                Ok(root.join("rooted/src/lld.md")),
            ]
        );
    }

    #[test]
    #[validates(spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsTheOldPath)]
    fn a_slice_whose_directory_holds_no_document_keeps_the_old_path() {
        let (root, project) = workspace("layout-document-old-path");
        let found = ["alpha", "lld-review", "tool", "two-phase"].map(|slice| lld_path(&project, slice));
        assert_eq!(
            found,
            [
                Ok(root.join("app/docs/intent/alpha/lld.md")),
                // The slice as it was named, not as its module is.
                Ok(root.join("app/docs/intent/lld-review/lld.md")),
                Ok(root.join("tool/docs/intent/tool/lld.md")),
                Ok(root.join("mac/docs/intent/two-phase/lld.md")),
            ]
        );
    }

    #[test]
    #[validates(spec::ASliceWithNoCrateKeepsItsDocumentAtTheWorkspaceRoot)]
    fn a_slice_with_no_crate_keeps_its_document_at_the_workspace_root() {
        let (root, project) = workspace("layout-workspace-root-document");
        // No member holds `book`, in either layout, so its document is the
        // workspace root's and not any crate's.
        assert_eq!(lld_path(&project, "book"), Ok(root.join("docs/intent/book/lld.md")));
    }

    #[test]
    #[validates(spec::ASlicesDocumentIsNeverUnderItsCompanion)]
    fn a_slices_document_is_never_under_its_companion() {
        let (root, project) = workspace("layout-document-not-under-companion");
        let companion_dir = root.join("app/src/two_phase");
        // The companion directory is built as the rule has it: a `spec.rs`, a
        // `mod.rs`, and no `lld.md`. That absence is why the claim holds — the
        // directory is never enumerated as a slice — so a fixture without the
        // directory would hold it for the wrong reason.
        assert_eq!(
            (
                companion_dir.join("spec.rs").is_file(),
                companion_dir.join("mod.rs").is_file(),
                companion_dir.join("lld.md").exists()
            ),
            (true, true, false),
            "the companion directory carries claims and a module and no document"
        );
        // `app` is searched before `mac`, and answers for neither slice.
        assert_eq!(
            (lld_path(&project, "two-phase"), lld_path(&project, "delta")),
            (Ok(root.join("mac/docs/intent/two-phase/lld.md")), Ok(root.join("mac/src/delta/lld.md")))
        );
        // Read as a directory, a companion's is a shape with no document.
        assert_eq!(Form::of_directory(&project, &companion_dir).own_document(), None);
    }

    #[test]
    #[validates(spec::ASpecFileBesideTheDocumentIsTheSlicesClaimsFile)]
    fn a_spec_file_beside_the_document_is_the_slices_claims_file() {
        let (root, project) = workspace("layout-spec-file-beside-the-document");
        // A slice between Phase 1 and Phase 3: its directory holds the
        // document and no code, the module file being Phase 3's to write.
        // The marker is then the only evidence the directory is a slice's,
        // and `crate_holding` already reads it through `slices_of` — so the
        // form has to read it too. While it did not, every new module slice
        // answered as its crate's root, and Phase 2 was offered the
        // crate-root slice's claims file as the only path it could write.
        let fresh = root.join("app/src/gamma");
        std::fs::create_dir_all(&fresh).expect("the new slice's directory");
        std::fs::write(fresh.join("lld.md"), "# gamma\n").expect("the new slice's document");
        // The tree the three answers below are read from: `delta`'s claims
        // are the companion's, beside its module and at no old-form path;
        // `beta` has no claims file at all, this door being where Phase 2 is
        // told to write one; and `gamma` has neither claims nor code.
        assert_eq!(
            [
                root.join("app/src/delta/spec.rs").is_file(),
                root.join("app/src/spec/delta.rs").exists(),
                root.join("app/src/beta/spec.rs").exists(),
                fresh.join("mod.rs").exists(),
                root.join("app/src/gamma.rs").exists(),
            ],
            [true, false, false, false, false],
            "the fixture: a migrated claims file, no pre-migration one, and two slices with none"
        );
        // Each answer is relative to no crate — `src/<module>/spec.rs` — so
        // that the caller places it under the crate it means. `delta`'s form
        // is read from `mac`, where its document is, and joined by the caller
        // onto `app`, where its claims are.
        let answers = ["beta", "delta", "gamma"].map(|slice| spec_file(&project, slice));
        let expected = ["src/beta/spec.rs", "src/delta/spec.rs", "src/gamma/spec.rs"].map(|p| Ok(PathBuf::from(p)));
        assert_eq!(answers, expected);
    }

    #[test]
    #[validates(spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsClaimsUnderSpec)]
    fn a_slice_whose_directory_holds_no_document_keeps_its_claims_under_spec() {
        let (root, project) = workspace("layout-spec-file-under-spec");
        // None of these slices' directories holds a document, so each keeps
        // the path its crate holds its claims at until the migration moves
        // them — `two-phase`'s directory in the crate its document is in.
        assert_eq!(
            (root.join("app/src/alpha/lld.md").exists(), root.join("mac/src/two_phase/lld.md").exists()),
            (false, false),
            "the unmigrated slices' directories hold no document"
        );
        let found = ["alpha", "lld-review", "two-phase"].map(|slice| spec_file(&project, slice));
        assert_eq!(
            found,
            [
                Ok(PathBuf::from("src/spec/alpha.rs")),
                // The file is named for the slice's module, not the slice.
                Ok(PathBuf::from("src/spec/lld_review.rs")),
                // Read from `mac`, whose `docs/intent` holds the document,
                // and joined by the caller onto the companion that holds the
                // claims: the form is the reading crate's answer either way.
                Ok(PathBuf::from("src/spec/two_phase.rs")),
            ]
        );
    }

    #[test]
    #[validates(spec::ACrateRootSlicesClaimsFileIsTheSpecBesideItsCode)]
    fn a_crate_root_slices_claims_file_is_the_spec_beside_its_code() {
        let (root, project) = workspace("layout-spec-file-crate-root");
        // The fixture's two crate-root slices are in opposite layouts:
        // `tool`'s document is still the one under `docs/intent`, `rooted`'s
        // is already beside its code. Both are asked here because the answer
        // not depending on the layout is what the claim states, and a test
        // asking one of them could not show it.
        assert_eq!(
            (
                root.join("tool/src/lld.md").exists(),
                root.join("tool/docs/intent/tool/lld.md").is_file(),
                root.join("rooted/src/lld.md").is_file()
            ),
            (false, true, true),
            "one crate-root slice has migrated and the other has not"
        );
        // Neither crate holds a module named for its slice, in either
        // spelling: the code is the crate, so there is no directory named for
        // the slice to hold a `spec.rs` beside a document. And nothing probes
        // for the file answered with — neither crate holds one, at the path
        // this door answers or at the one a slice's *name* computes, which is
        // a file no crate holds at all and the fault older than this door.
        let modules = ["tool/src/tool.rs", "tool/src/tool/mod.rs", "rooted/src/rooted.rs", "rooted/src/rooted/mod.rs"]
            .map(|path| root.join(path).exists());
        let claims_files =
            ["tool/src/spec.rs", "rooted/src/spec.rs", "tool/src/spec/tool.rs"].map(|path| root.join(path).exists());
        assert_eq!(
            (modules, claims_files),
            ([false; 4], [false; 3]),
            "neither crate-root slice's crate holds a module named for it, nor a claims file at either path"
        );
        let found = ["tool", "rooted"].map(|slice| spec_file(&project, slice));
        // The one path the two layouts agree on, answered relative to no
        // crate as every answer of this door is.
        assert_eq!(found, [Ok(PathBuf::from("src/spec.rs")), Ok(PathBuf::from("src/spec.rs"))]);
    }

    #[test]
    #[validates(spec::ASliceNoMemberHoldsIsRefusedByName)]
    fn a_slice_no_member_holds_is_refused_by_name_at_spec_file() {
        let (_root, project) = workspace("layout-spec-file-refusal");
        // A third resolver needing a slice's crate: it resolves the crate
        // whose layout it reads, so it can fail to find one, and it refuses
        // in the same one sentence — observed at the door rather than
        // inferred from another resolver that reaches it.
        let unheld = spec_file(&project, "nobody").expect_err("no member holds a document named `nobody`");
        let workspace_only = spec_file(&project, "book").expect_err("`book`'s document is the workspace root's");
        assert_eq!(
            (unheld.contains("nobody"), unheld.contains("docs/intent/nobody/lld.md"), unheld.contains("no crate")),
            (true, true, true),
            "the refusal names the slice, the form its document would have had, and what follows: {unheld}"
        );
        assert!(workspace_only.contains("book"), "{workspace_only}");
    }

    #[test]
    #[validates(spec::ANamedFileBesideTheDocumentIsTheSlicesIntentFile)]
    fn a_named_file_beside_the_document_is_the_slices_intent_file() {
        let (root, project) = workspace("layout-intent-file-beside-the-document");
        // Nothing probes for the file answered with: the acceptance a human
        // is asked to write does not exist at the moment it is named.
        assert!(!root.join("app/src/beta/compile-time-accepted").exists(), "the fixture holds no acceptance file");
        let found = ["beta", "delta", "rooted"].map(|slice| intent_file(&project, slice, ACCEPTANCE));
        assert_eq!(
            found,
            [
                Ok(root.join("app/src/beta/compile-time-accepted")),
                // Under the slice's own crate, although `app` holds a
                // directory named for `delta` and is searched first: an
                // intent file has no companion form, and the crate whose
                // layout says the slice has migrated is `mac` either way.
                Ok(root.join("mac/src/delta/compile-time-accepted")),
                // A crate-root slice's directory is its crate's `src`.
                Ok(root.join("rooted/src/compile-time-accepted")),
            ]
        );
        // The document is a named file of the slice's intent like any other.
        assert_eq!(intent_file(&project, "beta", "lld.md"), lld_path(&project, "beta"));
    }

    #[test]
    #[validates(spec::ASliceWhoseDirectoryHoldsNoDocumentKeepsItsIntentFilesUnderDocsIntent)]
    fn a_slice_whose_directory_holds_no_document_keeps_its_intent_files_under_docs_intent() {
        let (root, project) = workspace("layout-intent-file-under-docs-intent");
        let found = ["alpha", "lld-review", "two-phase", "tool"].map(|slice| intent_file(&project, slice, ACCEPTANCE));
        assert_eq!(
            found,
            [
                Ok(root.join("app/docs/intent/alpha/compile-time-accepted")),
                // The directory is the slice as it was named, hyphens and
                // all, and not its module.
                Ok(root.join("app/docs/intent/lld-review/compile-time-accepted")),
                // In the crate holding the document, never in the companion
                // that holds the slice's claims.
                Ok(root.join("mac/docs/intent/two-phase/compile-time-accepted")),
                Ok(root.join("tool/docs/intent/tool/compile-time-accepted")),
            ]
        );
    }

    #[test]
    #[validates(spec::ASliceNoMemberHoldsIsRefusedByName)]
    fn a_slice_no_member_holds_is_refused_by_name_at_intent_file() {
        let (_root, project) = workspace("layout-intent-file-refusal");
        // The fourth resolver needing a slice's crate. A slice whose product
        // is the workspace reaches the refusal here and not a workspace-root
        // answer: that answer is `lld_path`'s, beside this door, because no
        // intent file but the document has a form there to answer with.
        let unheld = intent_file(&project, "nobody", ACCEPTANCE).expect_err("no member holds a document named `nobody`");
        let workspace_only =
            intent_file(&project, "book", ACCEPTANCE).expect_err("`book` has no crate to hold an intent file");
        assert_eq!(
            (unheld.contains("nobody"), unheld.contains("docs/intent/nobody/lld.md"), unheld.contains("no crate")),
            (true, true, true),
            "the refusal names the slice, the form its document would have had, and what follows: {unheld}"
        );
        assert!(workspace_only.contains("book"), "{workspace_only}");
    }

    #[test]
    #[validates(spec::ACompanionDirectoryIsTheOneTheManifestNames)]
    fn a_companion_directory_is_the_one_the_manifest_names() {
        let (root, project) = workspace("layout-companion-named");
        // `mac` is the proc-macro member whose manifest names `app`, and it
        // holds both slices — in either layout.
        let found = ["app/src/two_phase", "app/src/delta"].map(|dir| is_companion_dir(&project, &root.join(dir)));
        assert_eq!(found, [true, true]);
        // The shape carries the slice as the crate holding its document names
        // it, hyphens and all: the route from a directory back to a slice is
        // forward, and never an inversion of the module's name.
        assert_eq!(
            Form::of_directory(&project, &root.join("app/src/two_phase")),
            Form::Companion {
                crate_root: root.join("app"),
                module: "two_phase".to_string(),
                slice: "two-phase".to_string(),
            }
        );
    }

    #[test]
    #[validates(spec::ACompanionIsNeverReadFromADirectorysShape)]
    fn a_companion_is_never_read_from_a_directorys_shape() {
        let (root, project) = workspace("layout-companion-not-a-shape");
        // Both carry a companion's shape exactly — a `spec.rs` and a `mod.rs`
        // and no `lld.md` — and neither is one: `mac` holds no slice
        // `skipped`, and no manifest names `tool` as a companion.
        assert_eq!(
            (root.join("app/src/skipped/spec.rs").is_file(), root.join("app/src/skipped/lld.md").exists()),
            (true, false),
            "the shaped directories carry claims and no document"
        );
        let shaped = ["app/src/skipped", "tool/src/two_phase"].map(|dir| is_companion_dir(&project, &root.join(dir)));
        // A slice's own module directory, the same module in the proc-macro
        // crate that owns the slice, and a directory under no member at all.
        let others =
            ["app/src/alpha", "mac/src/two_phase", "docs"].map(|dir| is_companion_dir(&project, &root.join(dir)));
        assert_eq!((shaped, others), ([false, false], [false, false, false]));
    }
}

pub mod spec;
