#![doc = include_str!("../../docs/intent/xtask/lld.md")]

pub mod mapping;
pub mod mutants;
pub mod selftest;
pub mod spec;

/// Usage shown for unknown or missing subcommands.
const USAGE: &str = "usage: cargo xtask <mutants [--full] [--diff-base <ref>] | gate-selftest>";

/// Entry point for the CLI: dispatches to one task.
pub fn run(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str);
    match command {
        Some("mutants") => mutants::run(&args[1..]),
        Some("gate-selftest") => selftest::run_all(),
        Some(other) => Err(format!("unknown command `{other}`\n{USAGE}")),
        None => Err(USAGE.to_string()),
    }
}

/// The workspace root, one level above xtask's manifest.
pub(crate) fn workspace_root() -> Result<std::path::PathBuf, String> {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| "xtask has no parent directory".to_string())
}

#[cfg(test)]
mod intent_graph {
    //! xtask's own instance of the graph checks (README §4.2).
    lid::intent_graph!();
}
