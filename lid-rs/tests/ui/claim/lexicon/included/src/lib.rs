//! The expansion carries an `include_str!` of the project lexicon it read, so
//! that editing the file rebuilds every claim that answered to it.
//!
//! This member's claim compiles: `gather` is its own file's verb, defined
//! nowhere else in the workspace. What the harness reads is the dep-info cargo
//! wrote for this crate, which lists every file the compilation read — the
//! lexicon among them, if the expansion included it.

/// When the [`Turn`] ends, the run shall gather.
#[derive(lid_rs::Spec)]
pub struct AClaimWhoseLexiconIsIncluded;
