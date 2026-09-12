//! Claims for `init` and `new` (`src/init/lld.md`).
//!
//! The three claims the colocated-scaffold change added or reworded are
//! written in the controlled language and carry no `#[lid(free)]` mark; the
//! slice's other marks stay until its own burn-down (the ramp's per-slice
//! rule, `lid-rs-macros/src/claim/lld.md`).

use lid_rs::Spec;

/// When `init` runs in a directory that holds no package manifest, it shall
/// fail naming the manifest it expected, and touch nothing.
#[derive(Spec)]
#[lid(free)]
pub struct InitTargetsThePackageInTheCurrentDirectory;

/// When any file or manifest table `init` would create already exists, `init`
/// shall fail naming every conflict and write nothing.
#[derive(Spec)]
#[lid(free)]
pub struct InitWritesNothingWhenAnyTargetConflicts;

/// When [`Change::CreateFile`](crate::init::Change::CreateFile) would put
/// `src/spec.rs` beside an existing `src/spec/` directory, `init` shall name
/// that directory as a conflict, since the claims file would become that
/// directory's parent module.
#[derive(Spec)]
pub struct ASpecDirectoryConflictsWithTheCrateRootClaimsFile;

/// When `init` adds the `lid-rs` dependency, it shall pin the tool's own
/// version, or the checkout given by `--lid-rs-path`; any other flag shall be
/// rejected by name.
#[derive(Spec)]
#[lid(free)]
pub struct InitAddsLidRsAtTheToolsOwnVersion;

/// When `init` edits the manifest, it shall append the lint tables, the
/// `lid_rs` metadata table, and the test profile as new tables, leaving the
/// existing content byte-for-byte intact.
#[derive(Spec)]
#[lid(free)]
pub struct InitAppendsTheManifestTables;

/// When [`Change::WireLibrary`](crate::init::Change::WireLibrary) is applied
/// to `src/lib.rs`, the library shall carry the HLD include and the crate-root
/// slice's `lld.md` include before its existing items, and the sibling `spec`
/// module and the `intent_graph!()` test module after them.
#[derive(Spec)]
pub struct InitWiresTheLibraryAsTheCrateRootSlice;

/// The name [`InitWiresTheLibraryAsTheCrateRootSlice`] carried while the
/// library was wired with one doc include and a `spec` directory rather than
/// two includes and a sibling `spec` module. The alias registers no claim, so
/// the graph sees only the claim it points at; every citation of this name
/// warns with its replacement, and those citations are the later phases' work
/// list.
#[deprecated = "replaced by InitWiresTheLibraryAsTheCrateRootSlice"]
pub type InitWiresTheLibraryIntoTheGraph = InitWiresTheLibraryAsTheCrateRootSlice;

/// When the package has no `src/lib.rs`, `init` shall create one, so the
/// package gains the library test binary that validations link into.
#[derive(Spec)]
#[lid(free)]
pub struct BinOnlyPackagesGainALibrary;

/// When `init` emits a templated file, every placeholder shall be replaced
/// with the package's facts, and no placeholder shall remain in the output.
#[derive(Spec)]
#[lid(free)]
pub struct EmittedFilesCarryThePackageFacts;

/// When `.gitignore` lacks the mutation-output entry, `init` shall append it;
/// when the entry is present, `init` shall leave the file alone rather than
/// report a conflict.
#[derive(Spec)]
#[lid(free)]
pub struct MutationOutputIsIgnoredWithoutConflict;

/// When [`init_in`](crate::init::init_in) succeeds on a package, that package
/// shall be a crate-root slice that passes its own gate:
/// [`Form::of_slice`](crate::layout::Form::of_slice) answers
/// [`CrateRoot`](crate::layout::Form::CrateRoot) for the package's name,
/// [`check_all`](crate::lld_review::check_all) reports no failure over the
/// scaffolded `src/lld.md`, `cargo test --lib` runs the graph checks over a
/// canary-verified registry, and clippy passes at the emitted lint levels.
#[derive(Spec)]
pub struct AnInitialisedPackageIsACrateRootSliceThatPassesItsOwnGate;

/// The name [`AnInitialisedPackageIsACrateRootSliceThatPassesItsOwnGate`]
/// carried while the end-to-end proof ran the gate without resolving what
/// slice the scaffold is or checking its document. The alias registers no
/// claim, so the graph sees only the claim it points at; every citation of
/// this name warns with its replacement, and those citations are the later
/// phases' work list.
#[deprecated = "replaced by AnInitialisedPackageIsACrateRootSliceThatPassesItsOwnGate"]
pub type AnInitialisedPackagePassesItsOwnGate = AnInitialisedPackageIsACrateRootSliceThatPassesItsOwnGate;

/// When `new <name>` runs, it shall create the package with `cargo new
/// --lib`, replace the generated `src/lib.rs` with the documented template,
/// and then perform `init` there; without a name it shall fail with usage.
#[derive(Spec)]
#[lid(free)]
pub struct NewCreatesALibraryPackageThenInitialisesIt;
