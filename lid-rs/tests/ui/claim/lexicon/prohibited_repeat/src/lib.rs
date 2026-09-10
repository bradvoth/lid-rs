//! `[prohibited]` holds `extra` at most once; given twice it fails every derive
//! in the crate, and the message names the file and the key.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderATwiceGivenKey;
