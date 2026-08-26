//! `implements_module!` traces a whole module by containment.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

mod machinery {
    lid_rs::implements_module!(crate::spec::FixtureClaim);
}

fn main() {}
