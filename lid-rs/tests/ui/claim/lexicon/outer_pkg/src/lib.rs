//! The control for the nested workspace below this crate: `orbit` is this
//! crate's own verb, and a claim of this crate takes it and compiles.
//!
//! `inner/` is a package with a `[workspace]` table of its own. Its claim takes
//! the same verb and does not compile, because the walk stops at a directory
//! whose manifest opens a workspace and never reaches this file.

/// When the [`Turn`] ends, the run shall orbit.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderThisCratesOwnLexicon;
