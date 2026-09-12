//! A type that records but was never declared.
//!
//! `Forged` carries `Traceable` and no seal, and `impl Noun for Forged` is
//! refused by an `E0277` naming the seal. That is what makes a noun a
//! deliberate declaration rather than something a single hand-written impl
//! falls into — and it is the negative side of the one claim this slice makes
//! about the seal, whose positive side is a unit test that writes all three
//! impls.

/// Records a name, and stops there.
struct Forged;

impl lid_rs::Traceable for Forged {
    const NOUN: &'static str = "Forged";
}

impl lid_rs::Noun for Forged {}

fn main() {}
