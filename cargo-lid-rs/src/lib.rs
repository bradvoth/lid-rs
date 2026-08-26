#![doc = include_str!("../docs/intent/cargo-lid-rs/lld.md")]

pub mod mapping;
pub mod mutants;
pub mod project;
pub mod spec;

use lid_rs::implements;

/// Usage shown for missing or unknown subcommands.
const USAGE: &str = "usage: cargo lid-rs mutants [--full] [--diff-base <ref>]";

/// The name cargo inserts as the first argument when it runs an external
/// subcommand: `cargo lid-rs mutants` arrives as `["lid-rs", "mutants"]`.
const CARGO_SUBCOMMAND: &str = "lid-rs";

/// Entry point: normalises cargo's argument protocol, then dispatches.
pub fn run(args: &[String]) -> Result<(), String> {
    dispatch(without_cargo_subcommand_name(args))
}

/// The arguments with cargo's inserted subcommand name removed, if present.
#[implements(spec::CargoInsertedSubcommandNameIsDiscarded)]
fn without_cargo_subcommand_name(args: &[String]) -> &[String] {
    if args.first().is_some_and(|first| first == CARGO_SUBCOMMAND) {
        &args[1..]
    } else {
        args
    }
}

/// Dispatches to one subcommand.
#[implements(spec::UnknownSubcommandsFailWithUsage)]
fn dispatch(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("mutants") => mutants::run(&args[1..]),
        Some(other) => Err(format!("unknown subcommand `{other}`\n{USAGE}")),
        None => Err(USAGE.to_string()),
    }
}

#[cfg(test)]
mod intent_graph {
    //! This crate's instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid_rs::validates;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_string()).collect()
    }

    #[test]
    #[validates(spec::CargoInsertedSubcommandNameIsDiscarded)]
    fn cargo_inserted_subcommand_name_is_discarded() {
        let cases: [(&[&str], &[&str]); 4] = [
            (&["lid-rs", "mutants", "--full"], &["mutants", "--full"]),
            (&["mutants", "--full"], &["mutants", "--full"]),
            (&[], &[]),
            // Only the leading position is cargo's; elsewhere it is an ordinary argument.
            (&["mutants", "lid-rs"], &["mutants", "lid-rs"]),
        ];
        for (input, expected) in cases {
            assert_eq!(without_cargo_subcommand_name(&args(input)), args(expected).as_slice());
        }
    }

    #[test]
    #[validates(spec::UnknownSubcommandsFailWithUsage)]
    fn unknown_subcommands_fail_with_usage() {
        let unknown = run(&args(&["lid-rs", "bogus"])).expect_err("unknown subcommand must fail");
        assert!(unknown.contains("bogus") && unknown.contains("usage:"), "{unknown}");
        let missing = run(&args(&["lid-rs"])).expect_err("missing subcommand must fail");
        assert!(missing.contains("usage:") && missing.contains("mutants"), "{missing}");
    }
}
