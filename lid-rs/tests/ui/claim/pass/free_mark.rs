//! The ramp's mark: `#[lid(free)]` exempts a claim from every rule of the
//! language but the unit-struct rule, the registry records the exemption, and
//! the trait's `FREE` carries it to every site that cites the claim.
//!
//! `AFreeClaim`'s text is prose, not a claim: no terminator in place, two
//! modals, no link anywhere, and two terms of the vague-term list. It compiles
//! because the mark is read before any rule is applied.
//!
//! `AHeldClaim` carries no mark, so it is written in the language like any
//! claim about the system. It is here for `FREE`: one marked claim alone
//! cannot tell a const that records the mark from a const that is `true` for
//! everything, and the two together can.

use lid_rs::Spec;
use lid_rs::claim::{ClaimMeta, Language, Pattern};

/// this text is quickly and appropriate prose! it shall say shall twice and it
/// ends with no period at all
#[derive(Spec)]
#[lid(free)]
struct AFreeClaim;

/// When the [`Turn`] ends, the run shall stop.
#[derive(Spec)]
struct AHeldClaim;

/// The parts the derive recorded for the claim registered under `name`.
fn claim(name: &str) -> &'static ClaimMeta {
    &lid_rs::SPECS
        .iter()
        .find(|meta| meta.name == name)
        .expect("the derive registers every claim")
        .claim
}

fn main() {
    let free = claim(<AFreeClaim as Spec>::NAME);
    assert_eq!(free.language, Language::Free);
    assert_eq!(free.pattern, Pattern::Ubiquitous);
    assert_eq!(free.trigger, "");
    assert_eq!(free.verb, "");
    assert!(!free.negated);
    assert_eq!(free.object, "");
    assert_eq!(free.owner, "");
    assert!(free.templates.is_empty());

    assert!(<AFreeClaim as Spec>::FREE, "the mark is what the trait's `FREE` carries");
    assert!(
        !<AHeldClaim as Spec>::FREE,
        "a claim with no mark carries no exemption: `FREE` is the presence of `#[lid(free)]`, \
         not a constant"
    );
}
