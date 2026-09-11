//! The walk stops after examining a directory whose `Cargo.toml` has a line
//! that is exactly `[workspace]`.
//!
//! This crate carries no lexicon of its own, and the crate one directory above
//! it defines `orbit`. The manifest here opens a workspace, so the walk examines
//! this directory and goes no further: `orbit` is a verb no lexicon this crate
//! answers to defines, and the message names the base as the only file read.
//!
//! Nothing in the workspace above may name this package — a second workspace
//! root reached through a member's path dependencies is one cargo refuses, and
//! then no member is checked at all — so the harness checks this directory in a
//! run of its own and merges the two reports.

/// When the [`Turn`] ends, the run shall orbit.
#[derive(lid_rs::Spec)]
pub struct AClaimBelowANestedWorkspaceRoot;
