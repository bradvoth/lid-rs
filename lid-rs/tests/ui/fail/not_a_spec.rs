//! Check 1: citing a type that does not implement `Spec` is a compile error.

struct NotASpec;

#[lid_rs::implements(NotASpec)]
fn traced() {}

fn main() {}
