//! `#[flow]` and `#[leaf]` expand to the item's tokens unchanged
//! (`lid-rs-macros/src/lld.md`, "The shape pins"). "Unchanged" is observable
//! only as behaviour, so `main` calls a pinned free fn, a pinned inherent
//! method, a pinned trait-impl method whose trait declaration is pinned too,
//! and asserts what each returns — the four positions the pins accept.

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

trait Answers {
    #[lid_rs::leaf]
    fn answer(&self) -> u8;

    #[lid_rs::flow]
    fn twice(&self) -> u8 {
        self.answer() * 2
    }
}

impl Answers for Service {
    #[lid_rs::leaf]
    fn answer(&self) -> u8 {
        21
    }
}

fn main() {
    assert_eq!(routes(), 42);
    assert_eq!(works(), 7);
    assert_eq!(Service.compute(), 13);
    assert_eq!(Service.answer(), 21);
    assert_eq!(Service.twice(), 42);
}
