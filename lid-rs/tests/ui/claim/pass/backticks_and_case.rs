//! The qualifiers the rules carry: what stands inside backticks is not the
//! sentence's, every word a rule reads is compared without regard to case, and
//! a vague term is a whole word and not a run of letters inside one.
//!
//! Each claim here compiles only under one of those qualifiers, so an
//! implementation that ignored backticks, matched case-sensitively, or looked
//! for a term as a substring would fail this fixture rather than pass it
//! silently. A terminator or a `shall` inside a code span is neither a second
//! sentence nor a second modal; a term of the vague-term list inside a code span
//! is not a term; an opener is an opener lower-cased; the verb is whatever
//! follows the modal, recorded lower-cased, so the lexicon is asked about `stop`
//! when the claim wrote `Stop`; and *supported*, *normalised*, and *robustness*
//! are ordinary words that happen to open with *support*, *normal*, and
//! *robust*, which a substring matcher would reject.

use lid_rs::Spec;
use lid_rs::claim::{ClaimMeta, Language, Pattern};

/// When the [`Turn`] ends, the run shall print `done. ready? go!`.
#[derive(Spec)]
struct TerminatorsInsideBackticks;

/// When the [`Modal`] `shall` is backticked, the run shall stop.
#[derive(Spec)]
struct AModalInsideBackticks;

/// When the [`Turn`] ends, the run shall report the `quickly` flag.
#[derive(Spec)]
struct AVagueTermInsideBackticks;

/// when the [`Turn`] ends, the run shall stop.
#[derive(Spec)]
struct ALowerCaseOpener;

/// if the [`Store`] is unreachable, then the run shall stop.
#[derive(Spec)]
struct ALowerCaseIfOpener;

/// When the [`Turn`] ends, the run shall Stop.
#[derive(Spec)]
struct ACapitalisedVerb;

/// When the [`Turn`] ends, the run shall report the supported and normalised
/// measure of robustness.
#[derive(Spec)]
struct VagueTermsInsideLongerWords;

/// The parts the derive recorded for the claim registered under `name`.
fn claim(name: &str) -> &'static ClaimMeta {
    &lid_rs::SPECS
        .iter()
        .find(|meta| meta.name == name)
        .expect("the derive registers every claim")
        .claim
}

fn main() {
    let terminators = claim(<TerminatorsInsideBackticks as Spec>::NAME);
    assert_eq!(terminators.language, Language::Controlled);
    assert_eq!(terminators.trigger, "Turn");
    assert_eq!(terminators.verb, "print");

    let modal = claim(<AModalInsideBackticks as Spec>::NAME);
    assert_eq!(modal.trigger, "Modal");
    assert_eq!(modal.verb, "stop");

    let vague = claim(<AVagueTermInsideBackticks as Spec>::NAME);
    assert_eq!(vague.verb, "report");

    let opener = claim(<ALowerCaseOpener as Spec>::NAME);
    assert_eq!(opener.pattern, Pattern::EventDriven);
    assert_eq!(opener.trigger, "Turn");

    let unwanted = claim(<ALowerCaseIfOpener as Spec>::NAME);
    assert_eq!(unwanted.pattern, Pattern::Unwanted);
    assert_eq!(unwanted.trigger, "Store");

    let capitalised = claim(<ACapitalisedVerb as Spec>::NAME);
    assert_eq!(capitalised.verb, "stop");
    assert_eq!(capitalised.templates, ["*"].as_slice());

    let longer = claim(<VagueTermsInsideLongerWords as Spec>::NAME);
    assert_eq!(longer.language, Language::Controlled);
    assert_eq!(longer.verb, "report");
}
