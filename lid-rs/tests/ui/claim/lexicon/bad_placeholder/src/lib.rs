//! A template naming a placeholder that is neither `{object}` nor `{owner}` is
//! malformed as an unbalanced one is: it fails every derive in the crate, and
//! the message names the file, the verb, and the entry. Nothing would ever bind
//! the name, so a template that spells one promises a shape no signature can
//! match.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderAnUnbindablePlaceholder;
