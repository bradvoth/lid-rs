//! Check 14: a validator is named for a claim it cites — the `snake_case` of a
//! cited path's last segment, or that name followed by `_` and a suffix.
//!
//! The three claims after the first are named for the three `snake_case` rules:
//! a word starts at each capital a lower-case letter follows, a run of capitals
//! not so followed is one word, and a digit stays with the word before it. Each
//! validator below is named for no claim it cites, so the message says exactly
//! what the name would have to be; the last cites two claims, and the name the
//! message expects is the first cited claim's.
//!
//! No fn here carries `#[test]`. trybuild compiles a fixture as a `[[bin]]`
//! with no `--test`, and rustc's built-in `test` attribute expands before the
//! attributes below it and removes the item it decorates outside a test build —
//! so a `#[test]` validator would be gone before `validates` expanded, and
//! check 14 would have nothing to read. The rule is about `#[validates]` on a
//! fn, not about `#[test]`.

mod spec {
    /// When the [`Turn`] ends, the run shall stop.
    #[derive(lid_rs::Spec)]
    pub struct FixtureClaim;

    /// When the [`Library`] is opened, the reader shall carry a [`Card`].
    #[derive(lid_rs::Spec)]
    pub struct GainALibrary;

    /// When the [`Request`] arrives, the server shall return a [`Response`].
    #[derive(lid_rs::Spec)]
    pub struct HTTPServer;

    /// When the [`Phase`] is approved, the run shall run the [`Gate`].
    #[derive(lid_rs::Spec)]
    pub struct Phase7Runs;
}

#[lid_rs::validates(spec::FixtureClaim)]
fn validated() {}

#[lid_rs::validates(spec::GainALibrary)]
fn gains() {}

#[lid_rs::validates(spec::HTTPServer)]
fn serves() {}

#[lid_rs::validates(spec::Phase7Runs)]
fn phases() {}

#[lid_rs::validates(spec::GainALibrary, spec::HTTPServer)]
fn neither_cited_name() {}

fn main() {}
