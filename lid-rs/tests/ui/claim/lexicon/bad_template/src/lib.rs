//! A `signature` entry that is neither `*` nor a template beginning with `->`,
//! with balanced braces holding only `{object}` and `{owner}`, fails every
//! derive in the crate: the message names the file, the verb, and the entry.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAMalformedTemplate;
