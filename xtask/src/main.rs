//! Thin CLI over the xtask library (README §11: a thin bin over a lib, so
//! logic sits under `cargo test --lib` and doctests run).

use std::process::ExitCode;

/// Parses argv and delegates to the library.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match xtask::run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
