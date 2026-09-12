//! The two shapes of claim that carry no outcome bound at all.
//!
//! Both name a type that derives nothing and could not satisfy an `Outcome`
//! bound, so each compiles only because the derive emitted none. A derive that
//! bounded every claim's owner, or that read the empty owner as a path, would
//! fail this fixture with `E0277` rather than pass it silently — which is what
//! makes these two claims red before the derive enforces anything, and why
//! they share their harness with the `fail` fixtures beside them.

use lid_rs::Spec;

/// A type no claim may bound: it implements nothing.
#[derive(Debug)]
pub struct Receipt;

/// A type no claim may bound, with a variant to name.
#[derive(Debug)]
pub enum ShippingError {
    /// The variant the event-driven claim below names.
    Lost,
}

/// When the [`Turn`] ends, the run shall report [`ShippingError::Lost`].
// Event-driven, not unwanted: a non-empty owner, and no bound. Written `//`
// and not `///`: a claim is one sentence, and a second doc paragraph would be a
// second terminator.
#[derive(Spec)]
struct AnEventDrivenClaimNamingAVariant;

/// While the [`Session`] is open, the run shall report [`Receipt`].
// State-driven, not unwanted: a non-empty object whose owner is empty.
#[derive(Spec)]
struct AStateDrivenClaimNamingAType;

/// If the [`Store`] is unreachable, then the run shall report [`Receipt`].
// Unwanted, and the object has no variant, so the owner the derive records is
// empty and there is nothing to bound.
#[derive(Spec)]
struct AnUnwantedClaimWhoseObjectHasNoVariant;

fn main() {
    let _ = <AnEventDrivenClaimNamingAVariant as Spec>::NAME;
    let _ = <AStateDrivenClaimNamingAType as Spec>::NAME;
    let _ = <AnUnwantedClaimWhoseObjectHasNoVariant as Spec>::NAME;
}
