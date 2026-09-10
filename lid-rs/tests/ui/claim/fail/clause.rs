//! The opener's clause: a `When`, `While`, or `Where` claim closes its clause
//! with a comma before the modal, and an `If` claim closes it with `, then`.
//!
//! A clause allowed to run past the modal would take its trigger link from the
//! response, so a comma that stands only after the modal closes nothing: the
//! second struct here is that case.

fn main() {}

/// When the [`Turn`] ends the run shall stop.
#[derive(lid_rs::Spec)]
struct AnUnclosedWhen;

/// When the [`Turn`] ends the run shall stop, and then rest.
#[derive(lid_rs::Spec)]
struct ACommaAfterTheModal;

/// If the [`Store`] is unreachable, the run shall stop.
#[derive(lid_rs::Spec)]
struct AnIfWithoutThen;

/// While the [`Turn`] runs the log shall carry the [`Entry`].
#[derive(lid_rs::Spec)]
struct AnUnclosedWhile;

/// Where the [`Feature`] is present the run shall report the [`Count`].
#[derive(lid_rs::Spec)]
struct AnUnclosedWhere;
