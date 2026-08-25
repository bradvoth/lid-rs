//! Check 1: citing a spec that does not exist is a compile error.

#[lid::implements(spec::DoesNotExist)]
fn traced() {}

mod spec {}

fn main() {}
