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
    todo!("present over {} outcomes", outcomes.len())
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
