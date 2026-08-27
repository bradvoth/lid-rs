use std::path::{Path, PathBuf};

use lid_rs::implements;

use crate::project::{Project, capture, cargo_command};
use crate::sync;
use crate::spec;

/// Usage shown for `new` without a name.
const NEW_USAGE: &str = "usage: cargo lid-rs new <name>";

/// The placeholder templates substitute with the package name.
const PACKAGE_NAME: &str = "__LID_PACKAGE_NAME__";

/// The `.gitignore` entry for the mutation engine's output.
const MUTANTS_IGNORE: &str = "mutants.out/";

/// Manifest table headers `init` appends, any of which already present is a
/// conflict.
const APPENDED_TABLES: [&str; 3] = ["[lints", "[package.metadata.lid_rs]", "[profile.test]"];

/// Where the `lid-rs` dependency comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LidRsSource {
    /// A crates.io version — the tool's own by default.
    Version(String),
    /// A checkout, for testing before publication and for contributors.
    Path(PathBuf),
}

/// Options for `init`.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    /// Where to take `lid-rs` from.
    pub lid_rs: LidRsSource,
}

/// The package `init` augments.
#[derive(Debug)]
pub struct Package {
    /// The directory holding its manifest.
    pub dir: PathBuf,
    /// Its name, from `cargo metadata`.
    pub name: String,
    /// Its workspace root, from `cargo metadata`.
    pub root: PathBuf,
}

/// One planned change, computed before anything is written.
#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    /// A new file; the path existing is a conflict.
    CreateFile {
        /// Where to write.
        path: PathBuf,
        /// The rendered content.
        content: String,
    },
    /// New tables appended to the manifest; any of `APPENDED_TABLES` already
    /// present is a conflict.
    AppendManifestTables {
        /// The manifest.
        path: PathBuf,
    },
    /// The library root wired into the graph; `intent_graph!` already present
    /// is a conflict.
    WireLibrary {
        /// `src/lib.rs`.
        path: PathBuf,
    },
    /// A line ensured in a file — appended if absent, left alone if present.
    EnsureLine {
        /// The file, created if absent.
        path: PathBuf,
        /// The line.
        line: String,
    },
    /// The operating skill, synced from the `lid-rs` the manifest resolves
    /// (`docs/intent/sync/lld.md`); the path existing is a conflict.
    SyncSkill {
        /// The manifest whose resolved `lid-rs` supplies the skill.
        manifest: PathBuf,
        /// Where the copy lands: `<workspace_root>/.claude/skills/lid-rs/`.
        path: PathBuf,
    },
    /// `core.hooksPath` pointed at the synced hooks; a repository whose
    /// `core.hooksPath` is already set elsewhere is a conflict
    /// (`docs/intent/phase/lld.md`).
    AssertHooksPath {
        /// The workspace root, the repository the config belongs to.
        root: PathBuf,
    },
    /// The `lid-rs` dependency, added by `cargo add`.
    AddDependency {
        /// The manifest's directory.
        dir: PathBuf,
        /// Where `lid-rs` comes from.
        source: LidRsSource,
    },
}

/// Runs `init` in the current directory.
pub fn run(args: &[String]) -> Result<(), String> {
    let options = parse_options(args)?;
    let dir = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    init_in(&dir, &options)
}

/// Runs `new <name>`: a fresh library package, then `init`.
#[implements(spec::NewCreatesALibraryPackageThenInitialisesIt)]
pub fn run_new(args: &[String]) -> Result<(), String> {
    let Some((name, rest)) = args.split_first() else {
        return Err(NEW_USAGE.to_string());
    };
    let options = parse_options(rest)?;
    let cwd = std::env::current_dir().map_err(|e| format!("current directory: {e}"))?;
    let dir = create_library_package(&cwd, name)?;
    init_in(&dir, &options)
}

/// Augments the package in `dir`: locate, plan, refuse on any conflict,
/// apply.
#[implements(spec::AnInitialisedPackagePassesItsOwnGate)]
pub fn init_in(dir: &Path, options: &Options) -> Result<(), String> {
    let package = Package::locate(dir)?;
    let plan = plan(&package, options)?;
    refuse_conflicts(&plan)?;
    plan.iter().try_for_each(apply)
}

/// Parses `--lid-rs-path <dir>` over the default of the tool's own version.
#[implements(spec::InitAddsLidRsAtTheToolsOwnVersion)]
fn parse_options(args: &[String]) -> Result<Options, String> {
    let mut lid_rs = LidRsSource::Version(env!("CARGO_PKG_VERSION").to_string());
    let mut rest = args;
    while let Some((flag, tail)) = rest.split_first() {
        (lid_rs, rest) = apply_init_flag(flag, tail)?;
    }
    Ok(Options { lid_rs })
}

/// Applies one `init` flag, returning the new source and the unconsumed
/// arguments.
#[implements(spec::InitAddsLidRsAtTheToolsOwnVersion)]
fn apply_init_flag<'a>(flag: &str, tail: &'a [String]) -> Result<(LidRsSource, &'a [String]), String> {
    match flag {
        "--lid-rs-path" => {
            let dir = tail
                .first()
                .ok_or_else(|| "--lid-rs-path requires a directory".to_string())?;
            Ok((LidRsSource::Path(PathBuf::from(dir)), &tail[1..]))
        }
        other => Err(format!("unknown flag `{other}` for init")),
    }
}

impl Package {
    /// The package whose manifest is at `dir/Cargo.toml`.
    #[implements(spec::InitTargetsThePackageInTheCurrentDirectory)]
    pub fn locate(dir: &Path) -> Result<Self, String> {
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            return Err(format!("no package manifest at {}", manifest.display()));
        }
        let canonical = manifest
            .canonicalize()
            .map_err(|e| format!("resolving {}: {e}", manifest.display()))?;
        let project = Project::load_at(&canonical)?;
        let name = project
            .package_at(&canonical)
            .ok_or_else(|| format!("{} is not a package manifest", manifest.display()))?;
        Ok(Self { dir: dir.to_path_buf(), name, root: project.root()? })
    }
}

/// Runs `cargo new --lib <name>` under `parent` and empties the generated
/// `src/lib.rs` (cargo's undocumented placeholder), so `init` wires a
/// library holding nothing but the documented skeleton. The file stays,
/// because a package with no target has no metadata.
#[implements(spec::NewCreatesALibraryPackageThenInitialisesIt)]
fn create_library_package(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let mut command = cargo_command();
    command.current_dir(parent).args(["new", "--lib", name]);
    capture(&mut command)?;
    let dir = parent.join(name);
    let placeholder = dir.join("src/lib.rs");
    std::fs::write(&placeholder, "").map_err(|e| format!("emptying {}: {e}", placeholder.display()))?;
    Ok(dir)
}

/// Every change `init` makes to `package`, in application order.
fn plan(package: &Package, options: &Options) -> Result<Vec<Change>, String> {
    let file = |relative: &str, template: &str| -> Result<Change, String> {
        Ok(Change::CreateFile {
            path: package.dir.join(relative),
            content: render(template, &package.name)?,
        })
    };
    Ok(vec![
        Change::AddDependency { dir: package.dir.clone(), source: options.lid_rs.clone() },
        Change::AppendManifestTables { path: package.dir.join("Cargo.toml") },
        file("clippy.toml", include_str!("../templates/clippy.toml"))?,
        file("docs/intent/hld.md", include_str!("../templates/hld.md"))?,
        file("src/spec/mod.rs", include_str!("../templates/spec_mod.rs"))?,
        Change::WireLibrary { path: package.dir.join("src/lib.rs") },
        file(".github/workflows/gate.yml", include_str!("../templates/gate.yml"))?,
        Change::EnsureLine { path: package.dir.join(".gitignore"), line: MUTANTS_IGNORE.to_string() },
        file("AGENTS.md", include_str!("../templates/AGENTS.md"))?,
        file("CLAUDE.md", include_str!("../templates/CLAUDE.md"))?,
        Change::SyncSkill {
            manifest: package.dir.join("Cargo.toml"),
            path: package.root.join(sync::SKILL_IN_PROJECT),
        },
    ])
}

/// Substitutes every placeholder in `template`; a placeholder left over is an
/// error, never silent output.
#[implements(spec::EmittedFilesCarryThePackageFacts)]
fn render(template: &str, package_name: &str) -> Result<String, String> {
    let rendered = template.replace(PACKAGE_NAME, package_name);
    match rendered.find("__LID_") {
        Some(at) => Err(format!(
            "template placeholder left unsubstituted: {}",
            rendered[at..].lines().next().unwrap_or_default()
        )),
        None => Ok(rendered),
    }
}

/// Fails naming every conflicting change, so nothing is written.
#[implements(spec::InitWritesNothingWhenAnyTargetConflicts)]
fn refuse_conflicts(plan: &[Change]) -> Result<(), String> {
    let conflicts: Vec<String> = plan.iter().filter_map(Change::conflict).collect();
    if conflicts.is_empty() {
        Ok(())
    } else {
        Err(format!("init would overwrite existing work; nothing written:\n  {}", conflicts.join("\n  ")))
    }
}

impl Change {
    /// What already exists that this change would clobber, if anything.
    #[implements(spec::InitWritesNothingWhenAnyTargetConflicts)]
    fn conflict(&self) -> Option<String> {
        match self {
            Change::CreateFile { path, .. } | Change::SyncSkill { path, .. } => existing_file(path),
            Change::AppendManifestTables { path } => existing_table(path),
            Change::WireLibrary { path } => existing_graph(path),
            Change::AssertHooksPath { root } => foreign_hooks_path(root),
            Change::EnsureLine { .. } | Change::AddDependency { .. } => None,
        }
    }
}

/// A conflict if `path` exists.
fn existing_file(path: &Path) -> Option<String> {
    path.exists().then(|| format!("{} already exists", path.display()))
}

/// A conflict naming the first appended table already in the manifest.
fn existing_table(path: &Path) -> Option<String> {
    let manifest = std::fs::read_to_string(path).ok()?;
    APPENDED_TABLES
        .iter()
        .find(|header| manifest.contains(*header))
        .map(|header| format!("{} already has a `{header}` table", path.display()))
}

/// A conflict if the library already invokes the graph checks.
fn existing_graph(path: &Path) -> Option<String> {
    let library = std::fs::read_to_string(path).ok()?;
    library
        .contains("intent_graph!")
        .then(|| format!("{} already invokes intent_graph!", path.display()))
}

/// Applies one change.
fn apply(change: &Change) -> Result<(), String> {
    match change {
        Change::CreateFile { path, content } => create_file(path, content),
        Change::AppendManifestTables { path } => append_manifest_tables(path),
        Change::WireLibrary { path } => wire_library(path),
        Change::EnsureLine { path, line } => ensure_line(path, line),
        Change::SyncSkill { manifest, .. } => sync::write(&Project::load_graph_at(manifest)?),
        Change::AssertHooksPath { root } => sync::assert_hooks_path(root),
        Change::AddDependency { dir, source } => add_dependency(dir, source),
    }
}

/// Writes a new file, creating its directories.
fn create_file(path: &Path, content: &str) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(path, content).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// Appends the lint, metadata, and profile tables to the manifest.
#[implements(spec::InitAppendsTheManifestTables)]
fn append_manifest_tables(path: &Path) -> Result<(), String> {
    let manifest = std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    create_file(path, &with_manifest_tables(&manifest))
}

/// The manifest text with the tables appended — pure, so the append is
/// unit-testable.
#[implements(spec::InitAppendsTheManifestTables)]
fn with_manifest_tables(manifest: &str) -> String {
    format!("{}\n{}", manifest.trim_end_matches('\n'), include_str!("../templates/manifest_tables.toml"))
}

/// Wires `src/lib.rs` into the graph, creating it when the package has none.
#[implements(spec::BinOnlyPackagesGainALibrary)]
fn wire_library(path: &Path) -> Result<(), String> {
    let existing = match std::fs::read_to_string(path) {
        Ok(library) => library,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("reading {}: {e}", path.display())),
    };
    create_file(path, &wired_library(&existing))
}

/// The library text with the HLD include prepended and the spec module and
/// graph checks appended — pure, so the wiring is unit-testable.
#[implements(spec::InitWiresTheLibraryIntoTheGraph)]
fn wired_library(existing: &str) -> String {
    format!(
        "{}{existing}{}",
        include_str!("../templates/lib_header.rs"),
        include_str!("../templates/lib_footer.rs")
    )
}

/// Appends `line` unless the file already has it.
#[implements(spec::MutationOutputIsIgnoredWithoutConflict)]
fn ensure_line(path: &Path, line: &str) -> Result<(), String> {
    let current = std::fs::read_to_string(path).unwrap_or_default();
    if current.lines().any(|existing| existing == line) {
        return Ok(());
    }
    let appended: String = current.lines().chain(std::iter::once(line)).map(|l| format!("{l}\n")).collect();
    create_file(path, &appended)
}

/// Adds `lid-rs` with `cargo add`.
fn add_dependency(dir: &Path, source: &LidRsSource) -> Result<(), String> {
    let mut command = cargo_command();
    command.current_dir(dir).args(dependency_args(source));
    capture(&mut command).map(drop)
}

/// The `cargo add` arguments for a source — pure, so the pin is unit-testable.
#[implements(spec::InitAddsLidRsAtTheToolsOwnVersion)]
fn dependency_args(source: &LidRsSource) -> Vec<String> {
    match source {
        LidRsSource::Version(version) => vec!["add".to_string(), format!("lid-rs@{version}")],
        LidRsSource::Path(dir) => {
            ["add", "lid-rs", "--path"].map(String::from).into_iter().chain([dir.display().to_string()]).collect()
        }
    }
}

/// The conflict for `core.hooksPath`: set, and not to the synced hooks.
#[implements(spec::AForeignHooksPathIsAnInitConflict)]
fn foreign_hooks_path(root: &Path) -> Option<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    /// A fresh, empty scratch directory outside any cargo workspace.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-init-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// This checkout's `lid-rs`, for `--lid-rs-path`.
    fn lid_rs_checkout() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../lid-rs")
    }

    /// Options pointing `lid-rs` at this checkout.
    fn local_options() -> Options {
        Options { lid_rs: LidRsSource::Path(lid_rs_checkout()) }
    }

    /// `cargo new <kind> <name>` under `parent`; returns the package dir.
    fn cargo_new(parent: &Path, kind: &str, name: &str) -> PathBuf {
        let status = cargo_command()
            .current_dir(parent)
            .args(["new", kind, name, "--vcs", "git"])
            .status()
            .expect("cargo new runs");
        assert!(status.success(), "cargo new {kind} {name} must succeed");
        parent.join(name)
    }

    /// Runs a cargo command in `dir` with a shared target directory, returning
    /// success and the combined output.
    fn cargo_in(dir: &Path, args: &[&str]) -> (bool, String) {
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/init-tests-target");
        let output = cargo_command()
            .current_dir(dir)
            .args(args)
            .env("CARGO_TARGET_DIR", &target)
            .output()
            .expect("cargo runs");
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        (output.status.success(), text)
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_string()).collect()
    }

    #[test]
    #[validates(spec::InitTargetsThePackageInTheCurrentDirectory)]
    fn init_targets_the_package_in_the_current_directory() {
        let empty = scratch("locate-empty");
        let missing = Package::locate(&empty).expect_err("no manifest must be an error");
        assert!(missing.contains("Cargo.toml"), "{missing}");
        let parent = scratch("locate-package");
        let dir = cargo_new(&parent, "--lib", "located");
        let package = Package::locate(&dir).expect("a package manifest is located");
        assert_eq!((package.name.as_str(), package.dir.as_path()), ("located", dir.as_path()));
    }

    #[test]
    #[validates(spec::InitWritesNothingWhenAnyTargetConflicts)]
    fn init_writes_nothing_when_any_target_conflicts() {
        let parent = scratch("conflicts");
        let dir = cargo_new(&parent, "--lib", "conflicted");
        std::fs::write(dir.join("clippy.toml"), "# mine\n").expect("write");
        std::fs::write(dir.join("AGENTS.md"), "# mine\n").expect("write");
        let manifest_before = std::fs::read_to_string(dir.join("Cargo.toml")).expect("read");
        let error = init_in(&dir, &local_options()).expect_err("conflicts must fail init");
        assert!(error.contains("clippy.toml") && error.contains("AGENTS.md"), "{error}");
        let untouched = (
            std::fs::read_to_string(dir.join("Cargo.toml")).expect("read") == manifest_before,
            !dir.join("docs/intent/hld.md").exists(),
            !dir.join("src/spec/mod.rs").exists(),
        );
        assert_eq!(untouched, (true, true, true), "nothing may be written on conflict");
    }

    #[test]
    #[validates(spec::InitAddsLidRsAtTheToolsOwnVersion)]
    fn init_adds_lid_rs_at_the_tools_own_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(
            dependency_args(&LidRsSource::Version(version.to_string())),
            strings(&["add", &format!("lid-rs@{version}")])
        );
        assert_eq!(
            dependency_args(&LidRsSource::Path(PathBuf::from("/c/lid-rs"))),
            strings(&["add", "lid-rs", "--path", "/c/lid-rs"])
        );
    }

    #[test]
    #[validates(spec::InitAddsLidRsAtTheToolsOwnVersion)]
    fn init_options_default_to_the_tools_version() {
        let expected = Options { lid_rs: LidRsSource::Version(env!("CARGO_PKG_VERSION").to_string()) };
        assert_eq!(parse_options(&[]), Ok(expected));
        let path = Options { lid_rs: LidRsSource::Path(PathBuf::from("/c/lid-rs")) };
        assert_eq!(parse_options(&strings(&["--lid-rs-path", "/c/lid-rs"])), Ok(path));
    }

    #[test]
    #[validates(spec::InitAddsLidRsAtTheToolsOwnVersion)]
    fn init_options_reject_unknown_and_incomplete_flags() {
        let incomplete = parse_options(&strings(&["--lid-rs-path"]));
        let unknown = parse_options(&strings(&["--bogus"]));
        // `run` rejects the flag before touching the working directory.
        let via_run = run(&strings(&["--bogus"]));
        assert_eq!(
            (incomplete.is_err(), unknown.is_err_and(|e| e.contains("--bogus")), via_run.is_err()),
            (true, true, true)
        );
    }

    #[test]
    #[validates(spec::InitAppendsTheManifestTables)]
    fn init_appends_the_manifest_tables() {
        let manifest = "[package]\nname = \"x\"\n\n[dependencies]\nlid-rs = \"0.1\"\n";
        let appended = with_manifest_tables(manifest);
        let shape = (
            appended.starts_with(manifest),
            APPENDED_TABLES.iter().all(|header| appended.contains(header)),
        );
        assert_eq!(shape, (true, true), "existing content intact and every table present:\n{appended}");
    }

    #[test]
    #[validates(spec::InitAppendsTheManifestTables)]
    fn init_appends_the_tables_to_the_real_manifest() {
        let dir = fresh_package("new-manifest");
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).expect("manifest");
        let shape = (
            manifest.starts_with("[package]"),
            APPENDED_TABLES.iter().all(|header| manifest.contains(header)),
            manifest.contains("lid-rs"),
        );
        assert_eq!(shape, (true, true, true), "{manifest}");
    }

    #[test]
    #[validates(spec::InitAppendsTheManifestTables)]
    fn an_existing_table_is_a_conflict() {
        let dir = scratch("tables");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(&manifest, with_manifest_tables("[package]\nname = \"x\"\n")).expect("write");
        let conflict = existing_table(&manifest);
        std::fs::write(&manifest, "[package]\nname = \"x\"\n").expect("write");
        assert_eq!(
            (conflict.is_some_and(|c| c.contains("[lints")), existing_table(&manifest)),
            (true, None)
        );
    }

    #[test]
    #[validates(spec::InitWiresTheLibraryIntoTheGraph)]
    fn init_wires_the_library_into_the_graph() {
        let existing = "//! Mine.\n\n/// Adds.\npub fn add(a: u8, b: u8) -> u8 {\n    a + b\n}\n";
        let wired = wired_library(existing);
        let shape = (
            wired.starts_with("#![doc = include_str!(\"../docs/intent/hld.md\")]"),
            wired.contains(existing),
            wired.contains("pub mod spec;"),
            wired.trim_end().ends_with("lid_rs::intent_graph!();\n}"),
        );
        assert_eq!(shape, (true, true, true, true), "{wired}");
        let dir = scratch("wire");
        std::fs::write(dir.join("lib.rs"), &wired).expect("write");
        assert!(existing_graph(&dir.join("lib.rs")).is_some(), "an already-wired library is a conflict");
        std::fs::write(dir.join("lib.rs"), existing).expect("write");
        assert_eq!(existing_graph(&dir.join("lib.rs")), None);
    }

    #[test]
    #[validates(spec::BinOnlyPackagesGainALibrary)]
    fn bin_only_packages_gain_a_library() {
        let parent = scratch("bin-only");
        let dir = cargo_new(&parent, "--bin", "binary");
        assert!(!dir.join("src/lib.rs").exists(), "cargo new --bin has no library");
        init_in(&dir, &local_options()).expect("init on a bin-only package");
        let lib = std::fs::read_to_string(dir.join("src/lib.rs")).expect("a library was created");
        assert!(lib.contains("lid_rs::intent_graph!();"), "{lib}");
        // The library's graph checks run; the user's undocumented `main` is
        // theirs to fix — the lints apply to existing code immediately (§11).
        let (ok, out) = cargo_in(&dir, &["test", "--lib"]);
        assert!(ok && out.contains("registry_is_populated"), "the new library's graph checks must run:\n{out}");
    }

    #[test]
    #[validates(spec::BinOnlyPackagesGainALibrary)]
    fn an_unreadable_library_is_an_error_not_a_replacement() {
        // Only a *missing* library is created; any other read failure must
        // surface, never be treated as "absent" and overwritten.
        let dir = scratch("unreadable");
        let path = dir.join("lib.rs");
        std::fs::create_dir_all(&path).expect("a directory where the file should be");
        let error = wire_library(&path).expect_err("reading a directory as the library must fail");
        assert!(error.starts_with("reading "), "{error}");
    }

    #[test]
    #[validates(spec::EmittedFilesCarryThePackageFacts)]
    fn emitted_files_carry_the_package_facts() {
        assert_eq!(render("# __LID_PACKAGE_NAME__ is __LID_PACKAGE_NAME__", "pkg"), Ok("# pkg is pkg".to_string()));
        let leftover = render("__LID_UNKNOWN__", "pkg").expect_err("an unknown placeholder must not be emitted");
        assert!(leftover.contains("__LID_UNKNOWN__"), "{leftover}");
    }

    #[test]
    #[validates(spec::MutationOutputIsIgnoredWithoutConflict)]
    fn mutation_output_is_ignored_without_conflict() {
        let dir = scratch("gitignore");
        let path = dir.join(".gitignore");
        ensure_line(&path, MUTANTS_IGNORE).expect("created when absent");
        let created = std::fs::read_to_string(&path).expect("read");
        ensure_line(&path, MUTANTS_IGNORE).expect("left alone when present");
        let unchanged = std::fs::read_to_string(&path).expect("read");
        std::fs::write(&path, "/target\n").expect("write");
        ensure_line(&path, MUTANTS_IGNORE).expect("appended when missing");
        let appended = std::fs::read_to_string(&path).expect("read");
        assert_eq!(
            (created.as_str(), unchanged == created, appended.as_str()),
            ("mutants.out/\n", true, "/target\nmutants.out/\n")
        );
    }

    /// A `new`-style package: created, emptied, initialised against this checkout.
    fn fresh_package(scratch_name: &str) -> PathBuf {
        let parent = scratch(scratch_name);
        let dir = create_library_package(&parent, "fresh").expect("cargo new --lib");
        init_in(&dir, &local_options()).expect("init on the new package");
        dir
    }

    #[test]
    #[validates(spec::AnInitialisedPackagePassesItsOwnGate, spec::NewCreatesALibraryPackageThenInitialisesIt)]
    fn a_new_package_passes_its_own_gate() {
        let dir = fresh_package("new-gate");
        let (tests_ok, tests_out) = cargo_in(&dir, &["test", "--lib"]);
        let (clippy_ok, clippy_out) = cargo_in(&dir, &["clippy", "--all-targets", "--", "-D", "warnings"]);
        assert_eq!(
            (tests_ok && tests_out.contains("registry_is_populated"), clippy_ok),
            (true, true),
            "graph checks must run and clippy must pass at the emitted levels:\n{tests_out}\n{clippy_out}"
        );
    }

    #[test]
    #[validates(spec::NewCreatesALibraryPackageThenInitialisesIt)]
    fn new_empties_the_placeholder_library_and_needs_a_name() {
        let parent = scratch("new-placeholder");
        let dir = create_library_package(&parent, "fresh").expect("cargo new --lib");
        let placeholder = std::fs::read_to_string(dir.join("src/lib.rs")).expect("lib.rs stays");
        assert_eq!((placeholder.as_str(), run_new(&[]).is_err_and(|e| e.contains("usage"))), ("", true));
    }

    #[test]
    #[validates(spec::TheSkillComesFromTheResolvedLidRsDependency)]
    fn an_initialised_package_carries_its_dependencys_skill() {
        let dir = fresh_package("new-skill");
        let synced = sync::read_relative_files(&dir.join(sync::SKILL_IN_PROJECT)).expect("the skill was synced");
        let canonical = sync::read_relative_files(&lid_rs_checkout().join("skill")).expect("canonical skill");
        assert!(synced == canonical, "init syncs the skill of the lid-rs it just added, every file");
    }

    #[test]
    #[validates(spec::EmittedFilesCarryThePackageFacts)]
    fn an_initialised_package_has_no_placeholders_left() {
        let dir = fresh_package("new-placeholders");
        let leftovers: Vec<PathBuf> = ["src/lib.rs", "src/spec/mod.rs", "docs/intent/hld.md", "AGENTS.md", ".claude/skills/lid-rs/SKILL.md"]
            .iter()
            .map(|f| dir.join(f))
            .filter(|f| std::fs::read_to_string(f).expect("emitted file").contains("__LID_"))
            .collect();
        assert!(leftovers.is_empty(), "placeholders left in {leftovers:?}");
    }

}
