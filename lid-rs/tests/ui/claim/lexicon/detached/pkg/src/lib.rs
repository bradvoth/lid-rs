//! The walk stops before a directory holding no `Cargo.toml`.
//!
//! `detached/` is such a directory: it holds this package and no manifest of
//! its own. The workspace lexicon one level above it defines `settle`, and this
//! crate cannot reach it, so `settle` is a verb no lexicon here defines and the
//! message names the base as the only file the lexicon was read from. That is
//! the case a crate built from the registry cache is in.

/// When the [`Turn`] ends, the run shall settle.
#[derive(lid_rs::Spec)]
pub struct AClaimBeyondTheWalksReach;
