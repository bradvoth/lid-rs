//! `derive(Spec)` applies to unit structs only — a claim has no runtime shape.

#[derive(lid_rs::Spec)]
struct Claim {
    field: u8,
}

fn main() {}
