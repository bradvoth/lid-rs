//! Claims for `init` and `new` (`docs/intent/init/lld.md`).

use lid_rs::Spec;

/// When `init` runs in a directory that holds no package manifest, it shall
/// fail naming the manifest it expected, and touch nothing.
#[derive(Spec)]
pub struct InitTargetsThePackageInTheCurrentDirectory;

/// When any file or manifest table `init` would create already exists, `init`
/// shall fail naming every conflict and write nothing.
#[derive(Spec)]
pub struct InitWritesNothingWhenAnyTargetConflicts;

/// When `init` adds the `lid-rs` dependency, it shall pin the tool's own
/// version, or the checkout given by `--lid-rs-path`; any other flag shall be
/// rejected by name.
#[derive(Spec)]
pub struct InitAddsLidRsAtTheToolsOwnVersion;

/// When `init` edits the manifest, it shall append the lint tables, the
/// `lid_rs` metadata table, and the test profile as new tables, leaving the
/// existing content byte-for-byte intact.
#[derive(Spec)]
pub struct InitAppendsTheManifestTables;

/// When `init` edits `src/lib.rs`, it shall prepend the HLD include and
/// append the `spec` module and the `intent_graph!()` test module, leaving the
/// existing items intact.
#[derive(Spec)]
pub struct InitWiresTheLibraryIntoTheGraph;

/// When the package has no `src/lib.rs`, `init` shall create one, so the
/// package gains the library test binary that validations link into.
#[derive(Spec)]
pub struct BinOnlyPackagesGainALibrary;

/// When `init` emits a templated file, every placeholder shall be replaced
/// with the package's facts, and no placeholder shall remain in the output.
#[derive(Spec)]
pub struct EmittedFilesCarryThePackageFacts;

/// When `.gitignore` lacks the mutation-output entry, `init` shall append it;
/// when the entry is present, `init` shall leave the file alone rather than
/// report a conflict.
#[derive(Spec)]
pub struct MutationOutputIsIgnoredWithoutConflict;

/// When `init` succeeds, the package shall pass its own gate: `cargo test
/// --lib` runs the graph checks over a canary-verified registry, and clippy
/// passes at the emitted lint levels.
#[derive(Spec)]
pub struct AnInitialisedPackagePassesItsOwnGate;

/// When `new <name>` runs, it shall create the package with `cargo new
/// --lib`, replace the generated `src/lib.rs` with the documented template,
/// and then perform `init` there; without a name it shall fail with usage.
#[derive(Spec)]
pub struct NewCreatesALibraryPackageThenInitialisesIt;
