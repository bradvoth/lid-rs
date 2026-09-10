//! A verb whose table gives no `def` fails every derive in the crate, and the
//! message names the file, the verb, and the key — the half of the key rule
//! that no later reader would miss, because nothing later reads a definition.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAnUnglossedVerb;
