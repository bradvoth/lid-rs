//! What the derive records: the verb and its negation, the response object and
//! its owner, the verb's templates as the base lexicon writes them, and the
//! language a parsed claim is in.
//!
//! The first claim spans two doc lines: its clause closes only once the lines
//! are joined by a single space, so a claim that reached the derive unjoined
//! would not compile at all.

use lid_rs::Spec;
use lid_rs::claim::{ClaimMeta, Language};

/// When the [`Turn`]
/// ends, the run shall stop.
#[derive(Spec)]
struct JoinedDocLines;

/// When the [`Turn`] ends, the run shall not carry the [`Entry`].
#[derive(Spec)]
struct ANegatedVerb;

/// When the [`Turn`] ends, the run shall return a [`Report`].
#[derive(Spec)]
struct AShapeVerbWithAnObject;

/// When the [`Store`] is unreachable, the run shall reject with
/// [`AuthError::Backend`].
#[derive(Spec)]
struct AnOwnerBearingObject;

/// When the [`Mark`] is read, the run shall return
/// [`Language::Free`](crate::claim::Language::Free).
#[derive(Spec)]
struct AnOwnerFromAPathTarget;

/// When the [`Turn`] ends, the run shall stop.
#[derive(Spec)]
struct ABehaviourVerbWithoutAnObject;

/// The parts the derive recorded for the claim registered under `name`.
fn claim(name: &str) -> &'static ClaimMeta {
    &lid_rs::SPECS
        .iter()
        .find(|meta| meta.name == name)
        .expect("the derive registers every claim")
        .claim
}

fn main() {
    let joined = claim(<JoinedDocLines as Spec>::NAME);
    assert_eq!(joined.trigger, "Turn");
    assert_eq!(joined.verb, "stop");

    let negated = claim(<ANegatedVerb as Spec>::NAME);
    assert_eq!(negated.verb, "carry");
    assert!(negated.negated);
    assert_eq!(negated.object, "Entry");
    assert_eq!(negated.owner, "");

    let shape = claim(<AShapeVerbWithAnObject as Spec>::NAME);
    assert_eq!(shape.language, Language::Controlled);
    assert_eq!(shape.verb, "return");
    assert!(!shape.negated);
    assert_eq!(shape.object, "Report");
    assert_eq!(shape.owner, "");
    assert_eq!(shape.templates, ["-> Result<{object}, _>", "-> {object}"].as_slice());

    let owned = claim(<AnOwnerBearingObject as Spec>::NAME);
    assert_eq!(owned.verb, "reject");
    assert_eq!(owned.object, "AuthError::Backend");
    assert_eq!(owned.owner, "AuthError");
    assert_eq!(owned.templates, ["-> Result<_, {owner}>"].as_slice());

    let pathed = claim(<AnOwnerFromAPathTarget as Spec>::NAME);
    assert_eq!(pathed.object, "crate::claim::Language::Free");
    assert_eq!(pathed.owner, "crate::claim::Language");

    let behaviour = claim(<ABehaviourVerbWithoutAnObject as Spec>::NAME);
    assert_eq!(behaviour.verb, "stop");
    assert_eq!(behaviour.object, "");
    assert_eq!(behaviour.owner, "");
    assert_eq!(behaviour.templates, ["*"].as_slice());
}
