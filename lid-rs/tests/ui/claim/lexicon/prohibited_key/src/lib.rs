//! The other half of the unknown-key rule: `[prohibited]` admits `extra` and
//! nothing else, so a second key under that table fails every derive in the
//! crate as an unadmitted key under a verb does, and the message names the file
//! and the key.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAnUnreadableProhibitedKey;
