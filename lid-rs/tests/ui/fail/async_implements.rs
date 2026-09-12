//! Citing an `async fn` is refused: the guard would be held across every
//! `.await` inside it, attributing every unrelated future polled in the
//! interval to the cited claim (`lid-rs-macros/src/lld.md`, "Citing an
//! `async fn` is refused").

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    #[lid(free)]
    pub struct FixtureClaim;
}

#[lid_rs::implements(spec::FixtureClaim)]
async fn traced() {}

fn main() {}
