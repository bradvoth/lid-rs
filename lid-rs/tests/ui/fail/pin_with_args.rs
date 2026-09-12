//! Neither pin takes arguments: an argument accepted and ignored is a mark
//! whose meaning the next reader has to guess (`lid-rs-macros/src/lld.md`,
//! "The shape pins").

#[lid_rs::flow(anything)]
fn traced() {}

fn main() {}
