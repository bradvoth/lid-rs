//! A verb whose table gives `def` or `signature` neither once fails every
//! derive in the crate, and the message names the file, the verb, and the key.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAHalfWrittenVerb;
