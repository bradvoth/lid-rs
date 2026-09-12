//! An unwanted claim whose owner is an enum that does not derive `Outcome`.
//!
//! The bound is on the trait, so the diagnostic is `E0277` — `PaymentError`
//! does not implement `Outcome` — at the claim. This is the case that makes
//! the check total: without the bound, an enum nobody derived would simply
//! never register, and the claim naming its variant would pass unnoticed.

use lid_rs::Spec;

/// The enum the claim below owns. It derives nothing.
#[derive(Debug)]
pub enum PaymentError {
    /// The variant the claim names.
    Declined,
}

/// If the [`Card`] is rejected, then the run shall report
/// [`PaymentError::Declined`].
#[derive(Spec)]
struct AnUnwantedClaimWhoseOwnerDerivesNoOutcome;

fn main() {
    let _ = <AnUnwantedClaimWhoseOwnerDerivesNoOutcome as Spec>::NAME;
}
