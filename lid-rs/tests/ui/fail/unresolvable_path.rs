//! Check 1: citing a spec that does not exist is a compile error.

#[lid_rs::implements(spec::DoesNotExist)]
fn traced() {}

mod spec {}

fn main() {}
