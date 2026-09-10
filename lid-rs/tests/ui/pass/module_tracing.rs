//! `implements_module!` traces a whole module by containment.
//!
//! The claim is marked `#[lid(free)]`: this fixture exists to exercise a
//! module-level citation, not to demonstrate the controlled language.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    #[lid(free)]
    pub struct FixtureClaim;
}

mod machinery {
    lid_rs::implements_module!(crate::spec::FixtureClaim);
}

fn main() {}
