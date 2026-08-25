//! Spec-retirement fixture: citing a `#[deprecated]` claim must warn at the
//! citation site, which `-D warnings` turns into a gate failure.

/// Claims for this fixture.
pub mod spec {
    use lid::Spec;

    /// When invoked, the old behaviour shall apply. (Retired.)
    #[deprecated = "superseded by NewClaim; re-review this citation"]
    #[derive(Spec)]
    pub struct RetiredClaim;
}

use lid::implements;

/// Still cites the retired claim.
#[implements(spec::RetiredClaim)]
pub fn old_behaviour() {}
