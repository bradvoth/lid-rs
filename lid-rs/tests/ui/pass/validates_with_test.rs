//! `#[validates]` coexists with `#[test]` — and in a fixture, which trybuild
//! compiles as a `[[bin]]` with no `--test`, that is all it can show.
//!
//! rustc's built-in `test` attribute expands before the attributes under it and
//! removes the item it decorates outside a test build, so `validates` never
//! expands here and registers no edge: `fn main` reads that absence back out of
//! `VALIDATIONS`. Inside the library, where the tests are built with `--test`,
//! the same pair registers the edge the registry joins on — which is why the
//! validators this fixture stands for are the ones under `#[cfg(test)]` and why
//! the controlled-language fixtures under `tests/ui/claim/` carry no `#[test]`
//! at all: a check on a validator's name cannot read an item that is gone.
//!
//! The claim is marked `#[lid(free)]`: this fixture exists to exercise how
//! `validates` and `test` compose, not to demonstrate the controlled language.

mod spec {
    use lid_rs::Spec;

    /// When probed, the fixture shall compile.
    #[derive(Spec)]
    #[lid(free)]
    pub struct FixtureClaim;
}

#[test]
#[lid_rs::validates(spec::FixtureClaim)]
fn validated() {}

fn main() {
    assert!(
        lid_rs::VALIDATIONS.iter().all(|edge| !edge.item.ends_with("::validated")),
        "the `#[test]` item survived a build without `--test`, so `validates` expanded after all"
    );
}
