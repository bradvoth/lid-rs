//! `#[validates]` coexists with `#[test]`.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

#[test]
#[lid_rs::validates(spec::FixtureClaim)]
fn validated() {}

fn main() {}
