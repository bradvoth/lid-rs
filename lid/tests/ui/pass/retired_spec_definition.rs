//! Retiring a spec must not break its own definition site: only citation
//! sites warn. This compiles under deny(warnings) because the derive's
//! emissions allow(deprecated) on themselves.
#![deny(warnings)]

mod spec {
    use lid::Spec;

    /// When invoked, the old behaviour shall apply. (Retired; uncited.)
    #[deprecated = "superseded; do not cite in new code"]
    #[derive(Spec)]
    pub struct RetiredClaim;
}

fn main() {}
