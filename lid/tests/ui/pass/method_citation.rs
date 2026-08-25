//! `#[implements]` works on methods inside impl blocks (body injection).

mod spec {
    use lid::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    pub struct FixtureClaim;
}

struct Service;

impl Service {
    #[lid::implements(spec::FixtureClaim)]
    fn traced(&self) {}
}

fn main() {
    Service.traced();
}
