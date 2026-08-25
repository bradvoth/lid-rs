//! `implements_module!` traces a whole module by containment.

mod spec {
    use lid::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

mod machinery {
    lid::implements_module!(crate::spec::FixtureClaim);
}

fn main() {}
