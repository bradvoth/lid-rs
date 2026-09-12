//! A primitive required to be a `Noun`.
//!
//! `String` carries `Traceable` — rule V's list gets the recording trait
//! (`README.md:513-516`) — and never the seal, so it satisfies the recording
//! half of `Noun` and fails the identity half. This is the fact the whole
//! design turns on, and it cannot be observed from a program that compiles:
//! only the refusal states it.
//!
//! The bound is written by hand rather than emitted, because the emission is
//! the other half of HLD row 17 and this fixture is about the trait.

/// The bound, stated once. A wrong answer here is a `Noun` that admits text.
fn assert_noun<T: lid_rs::Noun>() {}

fn main() {
    assert_noun::<String>();
}
