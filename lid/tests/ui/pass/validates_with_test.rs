//! `#[validates]` coexists with `#[test]`.

mod spec {
    use lid::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

#[test]
#[lid::validates(spec::FixtureClaim)]
fn validated() {}

fn main() {}
