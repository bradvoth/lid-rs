//! One claim of each pattern, with the trigger its clause yields.
//!
//! trybuild runs a passing fixture, so `fn main` is where the recorded parts
//! are asserted: the derive put them in `SPECS` at compile time and the
//! assertions read them back.

use lid_rs::Spec;
use lid_rs::claim::{ClaimMeta, Language, Pattern};

/// When the [`Turn`] ends, the run shall stop.
#[derive(Spec)]
struct AnEventDrivenClaim;

/// If the [`Store`] is unreachable, then the run shall stop.
#[derive(Spec)]
struct AnUnwantedClaim;

/// While the [`Turn`] runs, the log shall carry the [`Entry`].
#[derive(Spec)]
struct AStateDrivenClaim;

/// Where the [`Feature`] is present, the run shall report the [`Count`].
#[derive(Spec)]
struct AnOptionalClaim;

/// The [`Canary`] shall be present.
#[derive(Spec)]
struct AUbiquitousClaim;

/// The parts the derive recorded for the claim registered under `name`.
fn claim(name: &str) -> &'static ClaimMeta {
    &lid_rs::SPECS
        .iter()
        .find(|meta| meta.name == name)
        .expect("the derive registers every claim")
        .claim
}

fn main() {
    let event = claim(<AnEventDrivenClaim as Spec>::NAME);
    assert_eq!(event.pattern, Pattern::EventDriven);
    assert_eq!(event.trigger, "Turn");
    assert_eq!(event.language, Language::Controlled);

    let unwanted = claim(<AnUnwantedClaim as Spec>::NAME);
    assert_eq!(unwanted.pattern, Pattern::Unwanted);
    assert_eq!(unwanted.trigger, "Store");

    let state = claim(<AStateDrivenClaim as Spec>::NAME);
    assert_eq!(state.pattern, Pattern::StateDriven);
    assert_eq!(state.trigger, "Turn");

    let optional = claim(<AnOptionalClaim as Spec>::NAME);
    assert_eq!(optional.pattern, Pattern::Optional);
    assert_eq!(optional.trigger, "Feature");

    let ubiquitous = claim(<AUbiquitousClaim as Spec>::NAME);
    assert_eq!(ubiquitous.pattern, Pattern::Ubiquitous);
    assert_eq!(ubiquitous.trigger, "Canary");
}
