//! A verb named twice in one file fails every derive in the crate, and the
//! message names the file and the verb: two tables for one word are two
//! meanings, and the derive will not choose between them.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderATwiceNamedVerb;
