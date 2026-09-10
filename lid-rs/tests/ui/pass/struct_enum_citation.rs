//! `#[implements]` works on structs and enums, not only fns.
//!
//! The claim is marked `#[lid(free)]`: this fixture exists to exercise which
//! items may carry a citation, not to demonstrate the controlled language.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    #[lid(free)]
    pub struct FixtureClaim;
}

#[lid_rs::implements(spec::FixtureClaim)]
struct Carrier;

#[lid_rs::implements(spec::FixtureClaim)]
enum Kinds {
    One,
}

fn main() {
    let _ = Carrier;
    let _ = Kinds::One;
}
