//! Check 14 admits a suffix and any one cited claim: several validators of one
//! claim in one module cannot share a name, and a validator citing several
//! claims is named after whichever of them its author chose.
//!
//! Every claim here is held — none carries `#[lid(free)]` — because the mark is
//! what switches check 14 off, and marking the claim of a fixture about the
//! name would leave the rule unexercised with nothing going red. Being held is
//! also what makes `the_gate_passes` an observation and not a formality: an
//! assertion over a held claim's `FREE` is false, so a name the rule admits
//! that nonetheless carries an assertion fails to compile here.
//!
//! No fn here carries `#[test]`. trybuild compiles a fixture as a `[[bin]]`
//! with no `--test`, and rustc's built-in `test` attribute expands before the
//! attributes below it and removes the item it decorates outside a test build —
//! so a `#[test]` validator would register no edge and the name it carries
//! would never reach check 14. `fn main` reads the edges back out of
//! `VALIDATIONS`: that each name is there is what says `validates` expanded on
//! it and admitted it, rather than the name never having been read at all.

use lid_rs::Spec;

mod spec {
    /// When the [`Turn`] ends, the run shall stop.
    #[derive(lid_rs::Spec)]
    pub struct TheTurnEnds;

    /// When the [`Log`] closes, the run shall stop.
    #[derive(lid_rs::Spec)]
    pub struct TheLogCloses;

    /// When the [`Gate`] passes, the run shall end.
    #[derive(lid_rs::Spec)]
    pub struct TheGatePasses;
}

#[lid_rs::validates(spec::TheTurnEnds)]
fn the_turn_ends() {}

#[lid_rs::validates(spec::TheTurnEnds)]
fn the_turn_ends_with_an_empty_store() {}

#[lid_rs::validates(spec::TheTurnEnds, spec::TheLogCloses)]
fn the_log_closes() {}

/// An admitted name over a held claim, and so a fixture that compiles only if
/// nothing was asserted. Check 14's exemption is a const assertion over the
/// cited claims' `FREE`, which is false for `TheGatePasses`: emitting the guard
/// under a name the rule admits would fail this compilation with the name it
/// expected.
#[lid_rs::validates(spec::TheGatePasses)]
fn the_gate_passes() {}

/// Whether a validation edge cites `claim` from an item whose path ends in
/// `item` — the registration `validates` leaves behind when it admits a name.
fn cited(claim: &str, item: &str) -> bool {
    lid_rs::VALIDATIONS.iter().any(|edge| edge.spec == claim && edge.item.ends_with(item))
}

fn main() {
    the_turn_ends();
    the_turn_ends_with_an_empty_store();
    the_log_closes();
    the_gate_passes();

    let turn = <spec::TheTurnEnds as Spec>::NAME;
    let log = <spec::TheLogCloses as Spec>::NAME;
    let gate = <spec::TheGatePasses as Spec>::NAME;
    assert!(cited(turn, "::the_turn_ends"), "the claim's own name is admitted");
    assert!(cited(turn, "::the_turn_ends_with_an_empty_store"), "the name with a `_` suffix is admitted");
    assert!(cited(turn, "::the_log_closes"), "a validator citing two claims registers an edge for each");
    assert!(cited(log, "::the_log_closes"), "the name of the second cited claim is admitted");
    assert!(cited(gate, "::the_gate_passes"), "an admitted name over a held claim registers its edge");
    assert!(
        !<spec::TheGatePasses as Spec>::FREE,
        "the claim the admitted name stands over is held: marking it would exempt this validator \
         from check 14 and leave the rule unexercised"
    );
}
