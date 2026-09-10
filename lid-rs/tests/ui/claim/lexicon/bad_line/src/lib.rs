//! A lexicon holding a line the subset does not admit fails every derive in
//! the crate that reads it, and the message names the file and the line.

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderABrokenLexicon;
