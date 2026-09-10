//! The fourth corner of the key rule: a verb whose table gives `def` twice
//! fails every derive in the crate, as one that gives it never does, and the
//! message names the file, the verb, and the key.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderATwiceGlossedVerb;
