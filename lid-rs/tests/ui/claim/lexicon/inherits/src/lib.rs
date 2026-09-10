//! The project lexicon is the first `docs/intent/lexicon.toml` in the crate's
//! own directory or an ancestor of it.
//!
//! This member carries none, so the file above it — this fixture workspace's —
//! governs. `rootprohibited` is a term only that file supplies, so the failure
//! naming it is the evidence that the walk reached it.

/// When the [`Turn`] ends, the rootprohibited run shall stop.
#[derive(lid_rs::Spec)]
pub struct AClaimUnderTheWorkspacesLexicon;
