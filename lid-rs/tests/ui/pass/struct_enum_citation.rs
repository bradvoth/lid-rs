//! `#[implements]` works on structs and enums, not only fns.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
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
