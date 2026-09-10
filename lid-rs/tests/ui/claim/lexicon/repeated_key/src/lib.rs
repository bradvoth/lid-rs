//! The other half of the key rule: a verb whose table gives `def` or
//! `signature` twice fails every derive in the crate, as one that gives neither
//! does, and the message names the file, the verb, and the key.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderATwiceWrittenKey;
