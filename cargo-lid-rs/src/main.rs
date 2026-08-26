//! Thin CLI over the `cargo-lid-rs` library (README §11: a thin bin over a
//! lib, so logic sits under `cargo test --lib` and doctests run).

use std::process::ExitCode;

/// Parses argv and delegates to the library.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cargo_lid_rs::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
