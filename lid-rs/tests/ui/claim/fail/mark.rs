//! The ramp's mark takes exactly the word `free`, so the exemption cannot grow
//! a second meaning; any other content is the failure, and the message names
//! what was written instead.
//!
//! The claim itself is well formed. The mark is read before any rule of the
//! language is applied, because a marked claim is exempt from them all, so a
//! mark that names nothing the derive knows has to fail there.

fn main() {}

/// When the [`Turn`] ends, the run shall stop.
#[derive(lid_rs::Spec)]
#[lid(loose)]
struct AMarkThatIsNotFree;
