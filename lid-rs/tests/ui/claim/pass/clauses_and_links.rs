//! Where a clause ends, and what a link's target is.
//!
//! The first claim's clause holds a comma inside backticks and its only link
//! stands after it, so a clause that ended at the code span would hold no link
//! at all and the claim would not compile. The second's condition holds a plain
//! comma before its `, then`, and its only link stands after that comma, so a
//! clause that ended at the first comma of an `If` claim would hold no link
//! either: an unwanted claim closes at `, then` and nowhere earlier.

use lid_rs::Spec;
use lid_rs::claim::ClaimMeta;

/// When a `list, of names` names the [`Turn`], the run shall stop.
#[derive(Spec)]
struct ACommaInsideBackticks;

/// If the run stalls, and no [`Retry`] is set, then the run shall stop.
#[derive(Spec)]
struct AnIfClauseBeyondAComma;

/// When the [`Turn`](crate::turn::Turn) ends, the run shall stop.
#[derive(Spec)]
struct ALinkWithAPath;

/// When the [`Turn`] ends, the run shall stop.
#[derive(Spec)]
struct ABareLink;

/// When a turn [ends](crate::turn::end), the run shall stop.
#[derive(Spec)]
struct ATextLink;

/// The parts the derive recorded for the claim registered under `name`.
fn claim(name: &str) -> &'static ClaimMeta {
    &lid_rs::SPECS
        .iter()
        .find(|meta| meta.name == name)
        .expect("the derive registers every claim")
        .claim
}

fn main() {
    assert_eq!(claim(<ACommaInsideBackticks as Spec>::NAME).trigger, "Turn");
    assert_eq!(claim(<AnIfClauseBeyondAComma as Spec>::NAME).trigger, "Retry");
    assert_eq!(claim(<ALinkWithAPath as Spec>::NAME).trigger, "crate::turn::Turn");
    assert_eq!(claim(<ABareLink as Spec>::NAME).trigger, "Turn");
    assert_eq!(claim(<ATextLink as Spec>::NAME).trigger, "crate::turn::end");
}
