//! Check 14's exemption: a validator whose every cited claim carries
//! `#[lid(free)]` is not held to its name, whatever its identifier.
//!
//! The mark exempts a claim from the controlled language, and a validator's
//! name is a rule of the language about that claim; holding the name while the
//! sentence is exempt would name a test after words the claim does not yet
//! carry. So the two land together, slice by slice, and this is the fixture
//! where that exemption is observed: `refuses_a_second_turn` is the
//! `snake_case` of neither claim it cites, and it compiles.
//!
//! This is the one fixture whose *passing* is the whole of its point, so it is
//! read together with `fail/validator.rs`, which carries the same misnaming
//! over claims that are held. An implementation that ignored the mark fails
//! here with the name it expected; one that emitted no assertion at all passes
//! here and lets `fail/validator.rs` compile. Neither fixture bounds the rule
//! alone.
//!
//! The mark on the claims below is deliberate and is the condition under
//! demonstration — every other fixture about the name keeps its claims held,
//! because marking them would switch off the rule they exist to show. Their
//! sentences are written in the controlled language even so: what is observed
//! here is the name, and nothing about this fixture should rest on the grammar
//! exemption the mark also grants, which `pass/free_mark.rs` observes on its
//! own.
//!
//! No fn here carries `#[test]`. trybuild compiles a fixture as a `[[bin]]`
//! with no `--test`, and rustc's built-in `test` attribute expands before the
//! attributes below it and removes the item it decorates — so a `#[test]`
//! validator would register no edge and the name it carries would never reach
//! check 14.

use lid_rs::Spec;

mod spec {
    /// When the [`Ledger`] closes, the run shall stop.
    #[derive(lid_rs::Spec)]
    #[lid(free)]
    pub struct TheLedgerCloses;

    /// When the [`Ledger`] opens, the run shall run the [`Audit`].
    #[derive(lid_rs::Spec)]
    #[lid(free)]
    pub struct TheLedgerOpens;
}

/// A validator named for no claim it cites. Every cited claim is marked, so the
/// conjunction the citation would assert over is true and the name is not held.
#[lid_rs::validates(spec::TheLedgerCloses, spec::TheLedgerOpens)]
fn refuses_a_second_turn() {}

/// Whether a validation edge cites `claim` from an item whose path ends in
/// `item` — the registration `validates` leaves behind when it admits a
/// citation.
fn cited(claim: &str, item: &str) -> bool {
    lid_rs::VALIDATIONS.iter().any(|edge| edge.spec == claim && edge.item.ends_with(item))
}

fn main() {
    refuses_a_second_turn();

    let closes = <spec::TheLedgerCloses as Spec>::NAME;
    let opens = <spec::TheLedgerOpens as Spec>::NAME;
    assert!(
        <spec::TheLedgerCloses as Spec>::FREE && <spec::TheLedgerOpens as Spec>::FREE,
        "the exemption is the mark: a claim here that lost its `#[lid(free)]` would take the \
         misnamed validator with it"
    );
    assert!(cited(closes, "::refuses_a_second_turn"), "the unheld name registers its first edge");
    assert!(cited(opens, "::refuses_a_second_turn"), "the unheld name registers its second edge");
}
