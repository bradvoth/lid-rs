//! The project as `cargo metadata` describes it: where it is, which members
//! can dump a registry, and how mutation scope is configured
//! (`docs/intent/cargo-lid-rs/lld.md` § Subcommand shell, § Scope).

use std::path::{Path, PathBuf};
use std::process::Command;

use lid_rs::implements;

use crate::spec;

/// How much of the tree to mutate, from configuration or flags.
#[derive(Debug, PartialEq, Eq)]
pub enum Scope {
    /// Only code touched by the diff against a base ref.
    Diff {
        /// The git ref the diff is taken against.
        base: String,
    },
    /// The whole tree.
    Full,
}

/// A project, as described by one `cargo metadata` document fetched once per
/// run.
#[derive(Debug)]
pub struct Project {
    /// The parsed `cargo metadata --format-version 1 --no-deps` document.
    doc: serde_json::Value,
}

impl Project {
    /// Loads the project the current directory belongs to.
    pub fn load() -> Result<Self, String> {
        let out = capture(cargo_command().args(["metadata", "--format-version", "1", "--no-deps"]))?;
        Self::from_json(&out)
    }

    /// Loads the project a manifest belongs to.
    pub fn load_at(manifest: &std::path::Path) -> Result<Self, String> {
        let mut command = cargo_command();
        command
            .args(["metadata", "--format-version", "1", "--no-deps", "--manifest-path"])
            .arg(manifest);
        Self::from_json(&capture(&mut command)?)
    }

    /// Loads the project the current directory belongs to, with its resolved
    /// dependencies — needed to find what `lid-rs` the project uses.
    pub fn load_graph() -> Result<Self, String> {
        let out = capture(cargo_command().args(["metadata", "--format-version", "1"]))?;
        Self::from_json(&out)
    }

    /// Loads the project a manifest belongs to, with its resolved dependencies.
    pub fn load_graph_at(manifest: &std::path::Path) -> Result<Self, String> {
        let mut command = cargo_command();
        command.args(["metadata", "--format-version", "1", "--manifest-path"]).arg(manifest);
        Self::from_json(&capture(&mut command)?)
    }

    /// The directory of the resolved `lid-rs` package, registry or path, if
    /// the metadata includes dependencies and the project has one.
    #[implements(spec::TheSkillComesFromTheResolvedLidRsDependency)]
    pub fn lid_rs_package_dir(&self) -> Option<PathBuf> {
        self.packages()
            .find(|package| package.pointer("/name").and_then(serde_json::Value::as_str) == Some("lid-rs"))
            .and_then(|package| package.pointer("/manifest_path").and_then(serde_json::Value::as_str))
            .and_then(|manifest| Path::new(manifest).parent().map(Path::to_path_buf))
    }

    /// The name of the package whose manifest is at `manifest`, if any
    /// (a virtual workspace root has none).
    #[implements(spec::InitTargetsThePackageInTheCurrentDirectory)]
    pub fn package_at(&self, manifest: &std::path::Path) -> Option<String> {
        self.package_with_manifest(manifest.to_str()?)?
            .pointer("/name")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    }

    /// Parses a metadata document.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let doc = serde_json::from_str(json).map_err(|e| format!("parsing cargo metadata: {e}"))?;
        Ok(Self { doc })
    }

    /// The workspace root cargo reports.
    #[implements(spec::TheProjectRootComesFromCargoMetadata)]
    pub fn root(&self) -> Result<PathBuf, String> {
        self.path_at("/workspace_root")
    }

    /// The build directory cargo reports, where the diff file is written.
    #[implements(spec::TheProjectRootComesFromCargoMetadata)]
    pub fn target_directory(&self) -> Result<PathBuf, String> {
        self.path_at("/target_directory")
    }

    /// The configured mutation scope, with its fallback chain applied.
    #[implements(spec::MutationScopeFallsBackFromWorkspaceToPackageToDiff)]
    pub fn configured_scope(&self) -> Scope {
        let setting = self
            .workspace_setting("mutation_scope")
            .or_else(|| self.root_package_setting("mutation_scope"));
        if setting.as_deref() == Some("full") {
            Scope::Full
        } else {
            Scope::Diff { base: "main".to_string() }
        }
    }

    /// Names of the members that have a library target to dump a registry
    /// from.
    #[implements(spec::MembersWithoutALibraryTargetAreSkipped)]
    pub fn library_members(&self) -> Vec<String> {
        self.packages()
            .filter(|package| has_library_target(package))
            .filter_map(|package| package.pointer("/name").and_then(serde_json::Value::as_str))
            .map(str::to_string)
            .collect()
    }

    /// A cargo invocation rooted at the project.
    #[implements(spec::TheProjectRootComesFromCargoMetadata)]
    pub fn cargo(&self) -> Result<Command, String> {
        let mut command = cargo_command();
        command.current_dir(self.root()?);
        Ok(command)
    }

    /// A git invocation rooted at the project.
    #[implements(spec::TheProjectRootComesFromCargoMetadata)]
    pub fn git(&self) -> Result<Command, String> {
        let mut command = Command::new("git");
        command.current_dir(self.root()?);
        Ok(command)
    }
}

impl Project {
    /// A path-valued field of the metadata document.
    fn path_at(&self, pointer: &str) -> Result<PathBuf, String> {
        self.doc
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| format!("cargo metadata has no {pointer}"))
    }

    /// A `[workspace.metadata.lid_rs]` setting.
    fn workspace_setting(&self, key: &str) -> Option<String> {
        setting_in(&self.doc, key)
    }

    /// A `[package.metadata.lid_rs]` setting of the package whose manifest
    /// is at the workspace root, if there is one.
    fn root_package_setting(&self, key: &str) -> Option<String> {
        let root_manifest = self.root().ok()?.join("Cargo.toml");
        self.package_with_manifest(root_manifest.to_str()?)
            .and_then(|package| setting_in(package, key))
    }

    /// The package whose `manifest_path` is exactly `manifest`.
    fn package_with_manifest(&self, manifest: &str) -> Option<&serde_json::Value> {
        self.packages()
            .find(|package| package.pointer("/manifest_path").and_then(serde_json::Value::as_str) == Some(manifest))
    }

    /// The member packages, in metadata order.
    fn packages(&self) -> impl Iterator<Item = &serde_json::Value> {
        self.doc
            .pointer("/packages")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
    }
}

/// A `metadata.lid_rs.<key>` string setting on a metadata-bearing node.
fn setting_in(node: &serde_json::Value, key: &str) -> Option<String> {
    node.pointer(&format!("/metadata/lid_rs/{key}"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

/// Whether any of a package's targets is a library kind — something
/// `cargo test --lib` can build.
fn has_library_target(package: &serde_json::Value) -> bool {
    package
        .pointer("/targets")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|target| target.pointer("/kind").and_then(serde_json::Value::as_array).into_iter().flatten())
        .filter_map(serde_json::Value::as_str)
        .any(is_library_kind)
}

/// The target kinds cargo builds as a library target.
fn is_library_kind(kind: &str) -> bool {
    matches!(kind, "lib" | "rlib" | "dylib" | "cdylib" | "staticlib" | "proc-macro")
}

/// The cargo binary this tool was itself invoked through.
pub(crate) fn cargo_command() -> Command {
    Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
}

/// Runs a command, returning stdout or a message including stderr.
pub(crate) fn capture(command: &mut Command) -> Result<String, String> {
    let output = command
        .output()
        .map_err(|e| format!("running {command:?}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{command:?} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("non-utf8 command output: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    /// A metadata document with the fields the project reads, for a root at
    /// `/w` whose only package is the root package.
    fn doc(workspace_metadata: &str, package_metadata: &str, kinds: &[&[&str]]) -> String {
        let packages: Vec<String> = kinds
            .iter()
            .enumerate()
            .map(|(i, kinds)| {
                let targets: Vec<String> =
                    kinds.iter().map(|k| format!(r#"{{"kind":["{k}"],"name":"p{i}"}}"#)).collect();
                let manifest = if i == 0 { "/w/Cargo.toml".to_string() } else { format!("/w/p{i}/Cargo.toml") };
                format!(
                    r#"{{"name":"p{i}","manifest_path":"{manifest}","metadata":{package_metadata},"targets":[{}]}}"#,
                    targets.join(",")
                )
            })
            .collect();
        format!(
            r#"{{"workspace_root":"/w","target_directory":"/w/target","metadata":{workspace_metadata},"packages":[{}]}}"#,
            packages.join(",")
        )
    }

    #[test]
    #[validates(spec::TheProjectRootComesFromCargoMetadata)]
    fn the_project_root_comes_from_cargo_metadata() {
        let project = Project::from_json(&doc("null", "null", &[&["lib"]])).expect("parses");
        assert_eq!(project.root().expect("root present"), PathBuf::from("/w"));
        assert_eq!(project.target_directory().expect("target present"), PathBuf::from("/w/target"));
    }

    #[test]
    #[validates(spec::TheProjectRootComesFromCargoMetadata)]
    fn subprocesses_run_at_the_project_root() {
        let project = Project::from_json(&doc("null", "null", &[&["lib"]])).expect("parses");
        let cargo = project.cargo().expect("cargo");
        let git = project.git().expect("git");
        assert_eq!(
            (cargo.get_current_dir(), git.get_current_dir()),
            (Some(Path::new("/w")), Some(Path::new("/w")))
        );
    }

    #[test]
    #[validates(spec::TheProjectRootComesFromCargoMetadata)]
    fn the_root_is_the_workspace_even_from_a_subdirectory() {
        // Under `cargo test` the current directory is this crate's directory,
        // a subdirectory of the workspace: the root must still be the workspace.
        let live = Project::load().expect("cargo metadata must succeed in this repository");
        let expected = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("crate dir has a parent");
        assert_eq!(live.root().expect("root present"), expected);
    }

    #[test]
    #[validates(spec::MutationScopeFallsBackFromWorkspaceToPackageToDiff)]
    fn mutation_scope_falls_back_from_workspace_to_package_to_diff() {
        let full = r#"{"lid_rs":{"mutation_scope":"full"}}"#;
        let diff = r#"{"lid_rs":{"mutation_scope":"diff"}}"#;
        let against_main = || Scope::Diff { base: "main".to_string() };
        let cases = [
            (full, "null", Scope::Full, "workspace metadata is read"),
            ("null", full, Scope::Full, "root package metadata is the fallback"),
            (diff, full, against_main(), "workspace wins over package"),
            ("null", "null", against_main(), "unset means diff against main"),
        ];
        for (workspace, package, expected, why) in cases {
            let project = Project::from_json(&doc(workspace, package, &[&["lib"]])).expect("parses");
            assert_eq!(project.configured_scope(), expected, "{why}");
        }
    }

    #[test]
    #[validates(spec::InitTargetsThePackageInTheCurrentDirectory)]
    fn the_package_at_a_manifest_is_found_by_exact_path() {
        let project = Project::from_json(&doc("null", "null", &[&["lib"], &["bin"]])).expect("parses");
        assert_eq!(
            (project.package_at(Path::new("/w/Cargo.toml")), project.package_at(Path::new("/w/p1/Cargo.toml")), project.package_at(Path::new("/w/none/Cargo.toml"))),
            (Some("p0".to_string()), Some("p1".to_string()), None)
        );
    }

    #[test]
    #[validates(spec::MembersWithoutALibraryTargetAreSkipped)]
    fn members_without_a_library_target_are_skipped() {
        let project =
            Project::from_json(&doc("null", "null", &[&["lib"], &["bin"], &["proc-macro"], &["bin", "lib"]]))
                .expect("parses");
        assert_eq!(project.library_members(), vec!["p0", "p2", "p3"]);
    }
}
