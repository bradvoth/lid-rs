//! Retiring a spec must not break its own definition site: only citation
//! sites warn. This compiles under deny(warnings) because the derive's
//! emissions allow(deprecated) on themselves.
//!
//! The claim is marked `#[lid(free)]`: this fixture exists to demonstrate
//! retirement, not the controlled language, and its text is deliberately two
//! sentences — the retirement note is part of what a retired claim reads like.
#![deny(warnings)]

mod spec {
    use lid_rs::Spec;

    /// When invoked, the old behaviour shall apply. (Retired; uncited.)
    #[deprecated = "superseded; do not cite in new code"]
    #[derive(Spec)]
    #[lid(free)]
    pub struct RetiredClaim;
}

fn main() {}
