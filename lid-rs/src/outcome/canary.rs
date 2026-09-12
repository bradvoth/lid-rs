//! The outcome registry's canary: one known enum whose [`Outcome`]
//! implementation and [`OUTCOMES`](crate::OUTCOMES) registrations are written
//! by hand.
//!
//! Everything here is registered in the library proper — not `#[cfg(test)]` —
//! because downstream crates link `lid-rs` compiled without `cfg(test)`, and
//! the canary must reach *their* test binaries for their outcome checks to be
//! non-vacuous. An `OUTCOMES` without it was stripped by the linker; an
//! `OUTCOMES` holding only it is a binary that legitimately registers no
//! outcome of its own. That is the distinction the registry slice's canary
//! draws for the spec/implementation/validation triple, applied to the fourth
//! slice.
//!
//! The implementation is written by hand rather than derived because
//! `derive(Outcome)` expands to an implementation of that trait and so cannot
//! run before the trait exists.

use crate::outcome::Outcome;
use lid_rs::implements;

/// The known outcome this crate ships, in two variants.
///
/// Two rather than one so that a registration per variant is observably
/// different from a registration per enum. No claim names either variant, so
/// the enum owns no claimed variant and the scope rule of the check it guards
/// — "enums that own at least one claimed variant" — excludes it.
#[derive(Debug)]
pub enum CanaryOutcome {
    /// The first known variant.
    First,
    /// The second known variant.
    Second,
}

impl Outcome for CanaryOutcome {
    const NAME: &'static str = concat!(module_path!(), "::", stringify!(CanaryOutcome));
}

/// The identifiers of the variants [`CanaryOutcome`] declares, as the
/// registrations below spell them.
///
/// Written out rather than read back from the slice being examined: what a
/// canary is for is that the thing looked for is known before the registry is
/// consulted, so an `OUTCOMES` the linker stripped answers `false` instead of
/// answering for itself. Adding a variant to the enum adds an entry below and a
/// name here, and a registration that never arrives is the failure this reports.
const VARIANTS: [&str; 2] = ["First", "Second"];

/// Reports whether `outcomes` carries an entry for each variant
/// [`CanaryOutcome`] declares.
///
/// Every check over [`OUTCOMES`](crate::OUTCOMES) asserts this first, as every
/// registry-based check asserts
/// [`registry::canary::present`](crate::registry::canary::present) (README
/// §5.3): if LTO, `--gc-sections`, or an unusual target stripped the linker
/// section, a check over the outcomes would otherwise pass trivially over
/// nothing, and the binary that legitimately registers no outcome of its own
/// would be indistinguishable from the one whose registrations never arrived.
///
/// Parameterized over the slice rather than reading `OUTCOMES` directly, so the
/// stripped case — which cannot be produced at runtime from the real static —
/// is reachable with an empty input, while the validation applies the same
/// function to the real static.
#[implements(crate::outcome::spec::TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked)]
pub fn present(outcomes: &[crate::outcome::OutcomeMeta]) -> bool {
    VARIANTS.into_iter().all(|variant| {
        outcomes
            .iter()
            .any(|entry| entry.owner == CanaryOutcome::NAME && entry.variant == variant)
    })
}

// The registrations, in the shape `derive(Outcome)` emits: the owning key read
// through the enum's own implementation, the variant's identifier, and the
// site. Statics cannot carry `#[implements]`, so they carry no citation of
// their own: the claim they satisfy is cited on `present`, which answers for
// them.
const _: () = {
    /// One `OUTCOMES` entry per variant.
    macro_rules! canary_entry {
        ($variant:ident) => {
            const _: () = {
                /// The canary's registration for one variant.
                #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::OUTCOMES)]
                #[linkme(crate = ::lid_rs::__private::linkme)]
                static ENTRY: ::lid_rs::outcome::OutcomeMeta = ::lid_rs::outcome::OutcomeMeta {
                    owner: <CanaryOutcome as ::lid_rs::outcome::Outcome>::NAME,
                    variant: stringify!($variant),
                    file: file!(),
                    line: line!(),
                };
            };
        };
    }

    canary_entry!(First);
    canary_entry!(Second);
};

#[cfg(test)]
mod tests {
    use super::{CanaryOutcome, present};
    use crate::outcome::{Outcome, OutcomeMeta};
    use lid_rs::validates;

    /// One entry, for the cases the real slice cannot produce.
    fn entry(owner: &'static str, variant: &'static str) -> OutcomeMeta {
        OutcomeMeta { owner, variant, file: file!(), line: line!() }
    }

    /// The canary reaches the registry of a binary that links this crate.
    ///
    /// The first assertion is the claim itself and must be made against the
    /// real [`OUTCOMES`](crate::OUTCOMES): what it asserts is that the
    /// registrations survived linking into *this* binary, and a hand-built
    /// input would pass over a section the linker had stripped, which is the
    /// one thing the canary exists to tell apart from a binary that
    /// legitimately registers no outcome of its own. Registered outside
    /// `cfg(test)`, so the same holds for a consumer's test binary, which links
    /// this crate compiled without it.
    ///
    /// The three that follow are the cases the real slice *cannot* produce, and
    /// they are why [`present`] takes a slice rather than reading the static.
    /// Without them the assertion above holds for a `present` that answers
    /// `true` unconditionally, which is a canary that never sings — check 12
    /// found exactly that. The stripped case pins the answer; the two beside it
    /// pin the two halves of what "an entry for each variant" means, since a
    /// registry carrying the right owner under wrong names, or the right names
    /// under a foreign owner, is not this enum's registration.
    #[test]
    #[validates(crate::outcome::spec::TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked)]
    fn the_canary_outcome_is_enumerable_wherever_the_crate_is_linked() {
        assert!(present(&crate::OUTCOMES), "the canary's registrations were stripped");
        for (outcomes, absent) in registries_without_the_canary() {
            assert!(!present(&outcomes), "{absent}");
        }
    }

    /// The registries the canary is not in, which the real slice cannot be.
    ///
    /// A stripped section; this enum's key under names it does not declare; and
    /// its names under another enum's key. The last two are the two halves of
    /// "an entry for each variant", and each is a registry some other crate
    /// could genuinely produce.
    fn registries_without_the_canary() -> Vec<(Vec<OutcomeMeta>, &'static str)> {
        let mine = CanaryOutcome::NAME;
        vec![
            (vec![], "a stripped section answers for itself"),
            (vec![entry(mine, "Third"), entry(mine, "Fourth")], "not the variants it declares"),
            (vec![entry("other::E", "First"), entry("other::E", "Second")], "not this enum's key"),
        ]
    }
}
