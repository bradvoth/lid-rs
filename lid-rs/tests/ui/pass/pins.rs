//! `#[flow]` and `#[leaf]` expand to the item's tokens unchanged
//! (`lid-rs-macros/src/lld.md`, "The shape pins"). "Unchanged" is observable
//! only as behaviour, so `main` calls a pinned free fn, a pinned inherent
//! method, and asserts what each returns.

#[lid_rs::flow]
fn routes() -> u8 {
    42
}

#[lid_rs::leaf]
fn works() -> u8 {
    7
}

struct Service;

impl Service {
    #[lid_rs::leaf]
    fn compute(&self) -> u8 {
        13
    }
}

fn main() {
    assert_eq!(routes(), 42);
    assert_eq!(works(), 7);
    assert_eq!(Service.compute(), 13);
}
