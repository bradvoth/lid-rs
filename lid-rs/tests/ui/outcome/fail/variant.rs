//! An unwanted claim whose object names a segment the owner enum does not
//! declare as a variant.
//!
//! The bound `derive(Spec)` emits reaches the owner through `Outcome` and then
//! names the object's last segment as one of its variants, so the offence is
//! reported at the claim that wrote it — `E0599`, no variant named `Expired` —
//! and not at the enum. A derive that emitted no bound at all, or one that
//! named the whole object rather than its last segment, would compile this
//! fixture.

use lid_rs::{Outcome, Spec};

/// The enum the claim below owns. It declares `Refused`, not `Expired`.
#[derive(Debug, Outcome)]
pub enum AuthError {
    /// The one variant this enum has.
    Refused,
}

/// If the [`Store`] is unreachable, then the run shall report
/// [`AuthError::Expired`].
#[derive(Spec)]
struct AnUnwantedClaimNamingAVariantTheOwnerDoesNotDeclare;

fn main() {
    let _ = <AnUnwantedClaimNamingAVariantTheOwnerDoesNotDeclare as Spec>::NAME;
}
