//! A template whose braces do not balance is malformed as one that begins with
//! no `->` is: it fails every derive in the crate, and the message names the
//! file, the verb, and the entry.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAnUnbalancedTemplate;
