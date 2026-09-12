//! What `derive(Outcome)` emits for one enum: the key its implementation
//! carries, and one registration per variant.
//!
//! The enum is declared in an inner module so that its definition-site path is
//! observably longer than the fixture's crate root — a derive that recorded the
//! crate name, or the invocation site, would fail the first assertion rather
//! than agree with it by accident.
//!
//! The registrations are read back out of `OUTCOMES` in `fn main()`, which
//! trybuild runs, so this fixture asserts the emission's *content* and not
//! merely that it compiled.

use lid_rs::Outcome;
use lid_rs::outcome::OutcomeMeta;

mod inner {
    use lid_rs::Outcome;

    /// The enum this fixture is about, in two variants so that a registration
    /// per variant is observably different from a registration per enum.
    #[derive(Debug, Outcome)]
    pub enum PaymentOutcome {
        /// The first variant.
        Declined,
        /// The second variant.
        Settled,
    }
}

/// The registrations `OUTCOMES` carries for the enum under test.
fn registered() -> Vec<&'static OutcomeMeta> {
    let key = <inner::PaymentOutcome as Outcome>::NAME;
    lid_rs::OUTCOMES.iter().filter(|meta| meta.owner == key).collect()
}

fn main() {
    // The key is the enum's definition-site module path, joined with its
    // identifier — the inner module included.
    let name = <inner::PaymentOutcome as Outcome>::NAME;
    assert!(
        name.ends_with("::inner::PaymentOutcome"),
        "the name is the definition-site path, not {name}"
    );

    // One entry per declared variant, and no more.
    let entries = registered();
    assert_eq!(entries.len(), 2, "one registration per variant");

    // Each entry is keyed by the name read through the enum's own
    // implementation, so both sides of the join come from one const.
    for entry in &entries {
        assert_eq!(entry.owner, name);
    }

    // Each entry names the identifier of the variant it stands for.
    let mut variants: Vec<&str> = entries.iter().map(|entry| entry.variant).collect();
    variants.sort_unstable();
    assert_eq!(variants, ["Declined", "Settled"]);

    // Each entry carries the site it stands at. The file is asserted by suffix
    // because trybuild compiles a copy of this fixture from a generated
    // package, so `file!()` reports that copy's path and not this one.
    for entry in &entries {
        assert!(entry.file.ends_with("registration.rs"), "the site's file, not {}", entry.file);
        assert!(entry.line > 0, "the site's line");
    }
}
