//! The case grep cannot handle and the compiler must: a citation through a
//! `use` rename resolves to the same spec.

mod spec {
    use lid::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

use spec::FixtureClaim as RenamedClaim;

#[lid::implements(RenamedClaim)]
fn traced() {}

fn main() {
    traced();
}
