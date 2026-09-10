//! `#[implements]` works on methods inside impl blocks (body injection).
//!
//! The claim is marked `#[lid(free)]`: this fixture exists to exercise a
//! citation, not to demonstrate the controlled language, and its claim is a
//! stand-in for whatever the citing code implements.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    #[lid(free)]
    pub struct FixtureClaim;
}

struct Service;

impl Service {
    #[lid_rs::implements(spec::FixtureClaim)]
    fn traced(&self) {}
}

fn main() {
    Service.traced();
}
