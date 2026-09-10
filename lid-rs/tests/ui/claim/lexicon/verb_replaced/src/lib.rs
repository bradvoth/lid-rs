//! A verb the project's file names that the base also names is the project's,
//! whole: its entry replaces the base's rather than merging with it.
//!
//! The base gives `report` the signature `*`, so a claim taking it needs no
//! response object; this file gives it a template instead. The claim below takes
//! `report` and names no object, and it is the failure that shows whose entry
//! stood: under the base's `*` it would compile.

/// When the [`Turn`] ends, the run shall report.
#[derive(lid_rs::Spec)]
pub struct AClaimTakingARedefinedVerb;
