#![doc = include_str!("lld.md")]
use crate::registry::SpecMeta;
use lid_rs::implements;
use linkme::distributed_slice;

/// An enum whose variants are outcomes a claim may name.
///
/// `derive(Outcome)` implements this and registers one [`OutcomeMeta`] per
/// variant into [`OUTCOMES`]. Two of README §4.7's tier-1 checks read what it
/// carries: E1 bounds the owner of an unwanted claim on this trait, so a claim
/// naming a variant of an enum that is not an outcome does not compile, and E2
/// intersects the registered variants with the registered claims.
///
/// Span recording — the other reason an error type at a traced boundary
/// carries the derive — is not part of this surface. It extends the trait
/// rather than replacing it, so the const below is the whole of what a
/// consumer may rely on today.
pub trait Outcome {
    /// Canonical name of the enum: its definition-site `module_path!()` joined
    /// with the enum's identifier.
    ///
    /// The join key. Both sides of the intersection produce it from this one
    /// const — the registration by reading it through the enum's own
    /// implementation — so they cannot disagree about naming however a path is
    /// spelled or re-exported.
    const NAME: &'static str;
}

/// One registered variant of an enum implementing [`Outcome`].
#[derive(Debug)]
pub struct OutcomeMeta {
    /// The owning enum's [`Outcome::NAME`], read through that enum's own
    /// implementation rather than written a second time.
    pub owner: &'static str,
    /// The identifier of the variant this entry stands for, as the enum
    /// declares it.
    pub variant: &'static str,
    /// Source file of the registration site.
    pub file: &'static str,
    /// Source line of the registration site.
    pub line: u32,
}

/// Every variant of every enum implementing [`Outcome`], gathered at link
/// time.
///
/// The fourth distributed slice, beside [`SPECS`](crate::SPECS),
/// [`IMPLEMENTATIONS`](crate::IMPLEMENTATIONS) and
/// [`VALIDATIONS`](crate::VALIDATIONS).
/// [`CanaryOutcome`](canary::CanaryOutcome) registers into it unconditionally,
/// so a section the linker stripped is distinguishable from a binary that
/// legitimately registers no outcome of its own.
#[distributed_slice]
pub static OUTCOMES: [OutcomeMeta];

/// The owner named by every unwanted claim whose object names a variant.
///
/// README scopes the check to "enums that own at least one claimed variant",
/// and this is the predicate that answers which those are. A claim whose
/// pattern is not [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) puts
/// no enum in scope; neither does an unwanted claim whose object names no
/// variant, which carries the empty owner that
/// [`TheOwnerIsEmptyForAnObjectWithoutAVariant`](crate::claim::spec::TheOwnerIsEmptyForAnObjectWithoutAVariant)
/// records.
///
/// The owners are carried as registered. How one is matched against an enum's
/// [`Outcome::NAME`] is the caller's, so that this function is unaffected by
/// which of the two readings of that join the design settles on.
///
/// A `Vec` and not an `impl Iterator`: nothing here depends on laziness, and
/// `!` coerces to a `Vec` where it does not implement [`Iterator`] — which is
/// what lets this signature stand before its body does.
#[implements(
    spec::AnUnwantedClaimsOwnerIsAClaimedOwner,
    spec::AClaimOutsideTheUnwantedPatternIsNoClaimedOwner,
    spec::AnUnwantedClaimWithAnEmptyOwnerIsNoClaimedOwner,
)]
pub fn claimed_owners(specs: &[SpecMeta]) -> Vec<&'static str> {
    todo!("claimed_owners over {} specs", specs.len())
}

/// Variants of claim-owning enums that no unwanted claim names, formatted
/// `name (file:line)`.
///
/// Scoped to `crate_name` by the registered claims' [`Spec::NAME`](crate::Spec::NAME)
/// prefix, as [`graph_orphans`](crate::graph::graph_orphans) is: a consumer's
/// binary links this crate's registrations alongside its own, and an unscoped
/// answer would report one crate's variants as another crate's failure.
///
/// Parameterized over the slices rather than reading the real registries, so
/// that every branch is reachable from synthetic inputs while the emitted test
/// applies the same function to [`SPECS`](crate::SPECS) and [`OUTCOMES`].
///
/// Two steps: the owners the claims of `crate_name` put in scope, and then the
/// registered variants of the enums those owners name, less the ones an
/// unwanted claim names. [`claimed_owners`] is applied one registration at a
/// time so that the crate scope stands in front of it rather than behind it —
/// the scope is a property of the claim, not of the owner it carries — and so
/// that the owners in scope here and the owners [`claimed_owners`] carries are
/// one answer rather than two.
#[implements(
    spec::AVariantOfAnEnumNoClaimOwnsIsNotReported,
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
    spec::AReportedVariantIsNamedWithItsFileAndLine,
    spec::UnclaimedVariantsScopeToTheInvokingCrate,
)]
pub fn unclaimed_variants(
    crate_name: &str,
    specs: &[SpecMeta],
    outcomes: &[OutcomeMeta],
) -> Vec<String> {
    let owners: Vec<&'static str> = specs
        .iter()
        .filter(|spec| registered_by(crate_name, spec))
        .flat_map(|spec| claimed_owners(std::slice::from_ref(spec)))
        .collect();
    outcomes
        .iter()
        .filter(|outcome| owners.iter().any(|owner| owner_names_enum(owner, outcome.owner)))
        .filter(|outcome| !specs.iter().any(|spec| claim_names_variant(spec, outcome)))
        .map(report_variant)
        .collect()
}

/// Whether `crate_name` registered this claim.
///
/// The scope, applied to the claim rather than to the owner it carries: a
/// [`SpecMeta::name`] begins with the name of the crate that registered it,
/// while an owner is the path a claim's author wrote and says nothing about
/// where it was written. A consumer's binary links this crate's registrations
/// beside its own, so an unscoped answer would report one crate's variants as
/// another crate's failure.
///
/// The answer decides both directions: a false one for a claim this crate did
/// register withholds an enum that is in scope, as surely as a true one for a
/// foreign claim admits one that is not.
#[implements(
    spec::UnclaimedVariantsScopeToTheInvokingCrate,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn registered_by(crate_name: &str, spec: &SpecMeta) -> bool {
    todo!("registered_by {crate_name} for {}", spec.name)
}

/// Whether the owner a claim carries names the enum an [`OutcomeMeta`] was
/// registered for.
///
/// The one comparison in the intersection, and the only item that the design
/// document's open question about the join reaches. Under its Decisions row
/// both sides are the enum's own [`Outcome::NAME`], read through the resolved
/// type, and this is an equality; under its Shape row the claim side is the
/// path its author wrote, and this compares last segments. The two need
/// different code here and a different registration at the claim, so the body
/// waits on the document rather than settling it.
#[implements(
    spec::AVariantOfAnEnumNoClaimOwnsIsNotReported,
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn owner_names_enum(owner: &str, enum_name: &str) -> bool {
    todo!("owner_names_enum: {owner} against {enum_name}")
}

/// The owner and the variant identifier an unwanted claim's object names, or
/// `None` where the claim names no variant.
///
/// The claim side of the intersection, read from the registration alone: the
/// owner as [`claimed_owners`] carries it, and the last segment of the object.
/// No comparison against a registered enum happens here, so this is unaffected
/// by which reading of the join the document settles on.
#[implements(
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn claimed_variant(spec: &SpecMeta) -> Option<(&'static str, &'static str)> {
    todo!("claimed_variant of {}", spec.name)
}

/// Whether this claim's object names the variant an [`OutcomeMeta`] stands for.
///
/// Unscoped, unlike the owner side: a claim of any crate that names a variant
/// answers for it. What the scope decides is which enums are looked at, not
/// which of their variants someone asked for.
#[implements(
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn claim_names_variant(spec: &SpecMeta, outcome: &OutcomeMeta) -> bool {
    claimed_variant(spec).is_some_and(|(owner, variant)| {
        variant == outcome.variant && owner_names_enum(owner, outcome.owner)
    })
}

/// The text one unclaimed variant is reported as.
///
/// The report is read by whoever has to go and either delete the variant or
/// write the claim nobody wrote, so it carries the site as well as the name.
#[implements(spec::AReportedVariantIsNamedWithItsFileAndLine)]
fn report_variant(outcome: &OutcomeMeta) -> String {
    todo!(
        "report_variant for {}::{} at {}:{}",
        outcome.owner,
        outcome.variant,
        outcome.file,
        outcome.line
    )
}

#[cfg(test)]
mod tests {
    //! What the two work leaves answer over synthetic registries, and the two
    //! trybuild harnesses that stand for the emissions no run can observe.
    //!
    //! E1 is a bound emitted beside a claim, so a claim that breaks it does not
    //! compile and no run reports it; the five things `derive(Outcome)` emits
    //! are read back out of a fixture's own `fn main`, which trybuild runs.
    //! Both groups are therefore fixtures under `tests/ui/outcome/`, and each
    //! harness cites every claim its fixtures demonstrate — so the two claims
    //! whose demonstration is that something *compiles* share a validator with
    //! the siblings that fail. That is the cost the design document names and
    //! takes, and what makes them red before the derive enforces anything
    //! (`lid-rs/src/claim/mod.rs:116-119`).
    //!
    //! Two harnesses and not one: the groups are two independent emissions, and
    //! a single harness over both globs would report a regression in either as
    //! the same failure. Two and not nine: check 14 is a naming rule, admitting
    //! a validator named for any one of the claims it cites.
    //!
    //! Every synthetic owner is spelled from one constant on both sides of the
    //! join. How a claim's `owner` is matched against an enum's
    //! [`NAME`](super::Outcome::NAME) has two readings still open — an equality
    //! of resolved keys, or a comparison of last segments — and test data that
    //! spelled the two sides differently would be positive under one and
    //! negative under the other. Spelled from one constant, no test here
    //! distinguishes them, and the reading the design settles on costs no
    //! rewrite.

    use super::{OutcomeMeta, claimed_owners, spec, unclaimed_variants};
    use crate::claim::{ClaimMeta, Language, Pattern};
    use crate::registry::SpecMeta;
    use lid_rs::validates;

    /// The crate whose claims are in scope for every case below.
    const CRATE: &str = "lid_rs";

    /// The enum the synthetic claims own, spelled once so that both sides of
    /// the join carry the same string whichever reading of it wins.
    const AUTH_ERROR: &str = "crate::auth::AuthError";

    /// The object of the synthetic unwanted claim: a variant of [`AUTH_ERROR`].
    const AUTH_ERROR_REFUSED: &str = "crate::auth::AuthError::Refused";

    /// An enum of another crate's claim, which no claim of [`CRATE`] owns.
    const FOREIGN_ERROR: &str = "other_crate::foreign::ForeignError";

    /// A synthetic registration of one claim, at a fixed site.
    ///
    /// Only the parts these leaves read are given: the registered name the
    /// crate scope is taken from, the pattern, and the object and owner the
    /// intersection joins on.
    fn spec_meta(
        name: &'static str,
        pattern: Pattern,
        object: &'static str,
        owner: &'static str,
    ) -> SpecMeta {
        SpecMeta {
            name,
            file: "synthetic.rs",
            line: 1,
            claim: ClaimMeta {
                language: Language::Controlled,
                pattern,
                trigger: "",
                verb: "report",
                negated: false,
                object,
                owner,
                templates: &["*"],
            },
        }
    }

    /// A synthetic registration of one variant, at the site it is reported by.
    fn outcome_meta(
        owner: &'static str,
        variant: &'static str,
        file: &'static str,
        line: u32,
    ) -> OutcomeMeta {
        OutcomeMeta { owner, variant, file, line }
    }

    /// The bound `derive(Spec)` emits beside an unwanted claim, and the two
    /// shapes of claim that carry none: the fixtures under
    /// `tests/ui/outcome/fail/` and `tests/ui/outcome/pass/no_bound.rs`, whose
    /// compilation is the assertion.
    ///
    /// `fail/variant.rs` names a segment its owner does not declare, which the
    /// bound reports as `E0599` at the claim; `fail/underived.rs` names a
    /// variant of an enum that derives nothing, which the bound reports as
    /// `E0277`. Between them they hold the emission to both halves — a derive
    /// that matched the variant without bounding the owner loses the second,
    /// one that bounded without matching loses the first.
    ///
    /// `pass/no_bound.rs` is the other side: an event-driven claim naming a
    /// variant, a state-driven claim naming a type, and an unwanted claim whose
    /// object names no variant, each owning a type that implements nothing. It
    /// compiles only because no bound was emitted, so a derive that bounded
    /// every pattern or read an empty owner as a path fails it.
    #[test]
    #[validates(
        spec::AnUnwantedClaimNamingAVariantBoundsItsOwnerOnOutcome,
        spec::TheOutcomeBoundNamesTheObjectsLastSegmentAsAVariant,
        spec::AClaimOutsideTheUnwantedPatternCarriesNoOutcomeBound,
        spec::AnUnwantedClaimWithAnEmptyOwnerCarriesNoOutcomeBound
    )]
    fn an_unwanted_claim_naming_a_variant_bounds_its_owner_on_outcome() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/outcome/fail/*.rs");
        t.pass("tests/ui/outcome/pass/no_bound.rs");
    }

    /// What `derive(Outcome)` emits for one enum: `tests/ui/outcome/pass/
    /// registration.rs`, which reads its own registrations back out of
    /// `OUTCOMES` in the `fn main` trybuild runs.
    ///
    /// So this asserts the emission's content and not merely that it compiled:
    /// the key is the enum's definition-site path — the fixture declares the
    /// enum in an inner module, so a derive recording the crate root or the
    /// invocation site disagrees rather than coinciding — there is one entry
    /// per declared variant and no more, every entry is keyed by the name read
    /// through the enum's own implementation, each names the identifier of the
    /// variant it stands for, and each carries the file and line of its site.
    ///
    /// The site is asserted by suffix and by `line > 0`: trybuild compiles a
    /// copy of the fixture from a package it generates, so the `file!()` in the
    /// expansion reports that copy's path.
    #[test]
    #[validates(
        spec::TheOutcomeNameIsTheEnumsDefinitionSitePath,
        spec::EachVariantOfAnOutcomeEnumRegistersIntoOutcomes,
        spec::AnOutcomeRegistrationIsKeyedByTheEnumsOutcomeName,
        spec::AnOutcomeRegistrationNamesTheVariantItStandsFor,
        spec::AnOutcomeRegistrationCarriesTheSiteItStandsAt
    )]
    fn each_variant_of_an_outcome_enum_registers_into_outcomes() {
        let t = trybuild::TestCases::new();
        t.pass("tests/ui/outcome/pass/registration.rs");
    }

    /// The owner of an unwanted claim whose object names a variant is what puts
    /// its enum in scope, and it is carried as registered.
    #[test]
    #[validates(spec::AnUnwantedClaimsOwnerIsAClaimedOwner)]
    fn an_unwanted_claims_owner_is_a_claimed_owner() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR)];
        assert_eq!(claimed_owners(&specs), [AUTH_ERROR]);
    }

    /// A claim of another pattern puts no enum in scope, however its object is
    /// written: the pattern is what says someone asked for this way to fail.
    #[test]
    #[validates(spec::AClaimOutsideTheUnwantedPatternIsNoClaimedOwner)]
    fn a_claim_outside_the_unwanted_pattern_is_no_claimed_owner() {
        let specs = [spec_meta(
            "lid_rs::synthetic::Fake",
            Pattern::EventDriven,
            AUTH_ERROR_REFUSED,
            AUTH_ERROR,
        )];
        assert!(claimed_owners(&specs).is_empty(), "only an unwanted claim owns an enum");
    }

    /// An unwanted claim whose object names no variant carries the empty owner,
    /// and the empty string is not an enum: a filter on the pattern alone would
    /// carry it and put every enum in scope.
    #[test]
    #[validates(spec::AnUnwantedClaimWithAnEmptyOwnerIsNoClaimedOwner)]
    fn an_unwanted_claim_with_an_empty_owner_is_no_claimed_owner() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, "crate::auth::Receipt", "")];
        assert!(claimed_owners(&specs).is_empty(), "an empty owner names no enum");
    }

    /// A variant of an enum that no claim owns is out of scope, so it is not
    /// reported however many variants it has that nobody asked for.
    ///
    /// Written with one *non-matching* claim rather than with none: over an
    /// empty `specs` the owner list is empty and every outcome is filtered out
    /// before the join, so the answer would be the empty report without any of
    /// this reaching an unimplemented leaf.
    #[test]
    #[validates(spec::AVariantOfAnEnumNoClaimOwnsIsNotReported)]
    fn a_variant_of_an_enum_no_claim_owns_is_not_reported() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR)];
        let outcomes = [outcome_meta("crate::store::StoreError", "Backend", "store.rs", 7)];
        let report = unclaimed_variants(CRATE, &specs, &outcomes);
        assert!(report.is_empty(), "an enum no claim owns is out of scope: {report:?}");
    }

    /// The variant an unwanted claim's object names is the one somebody did ask
    /// for, so it is passed over.
    #[test]
    #[validates(spec::AVariantAnUnwantedClaimNamesIsNotReported)]
    fn a_variant_an_unwanted_claim_names_is_not_reported() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR)];
        let outcomes = [outcome_meta(AUTH_ERROR, "Refused", "auth.rs", 12)];
        let report = unclaimed_variants(CRATE, &specs, &outcomes);
        assert!(report.is_empty(), "a claimed variant is not reported: {report:?}");
    }

    /// The rule's whole point: an enum that owns one claimed variant is in
    /// scope, and its other variants are ways to fail that no requirement asked
    /// for.
    #[test]
    #[validates(spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported)]
    fn a_variant_of_a_claimed_owner_that_no_unwanted_claim_names_is_reported() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR)];
        let outcomes = [
            outcome_meta(AUTH_ERROR, "Refused", "auth.rs", 12),
            outcome_meta(AUTH_ERROR, "Backend", "auth.rs", 14),
        ];
        let report = unclaimed_variants(CRATE, &specs, &outcomes);
        assert_eq!(report.len(), 1, "the unclaimed variant and no other: {report:?}");
        assert!(report[0].contains("Backend"), "the variant nobody asked for: {report:?}");
    }

    /// The report is read by whoever has to go and either delete the variant or
    /// write the claim nobody wrote, so it names the variant and the site.
    ///
    /// The name is the owner and the variant together: the identifier alone
    /// does not say which enum declares it, and the reader has to find it.
    #[test]
    #[validates(spec::AReportedVariantIsNamedWithItsFileAndLine)]
    fn a_reported_variant_is_named_with_its_file_and_line() {
        let specs =
            [spec_meta("lid_rs::synthetic::Fake", Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR)];
        let outcomes = [outcome_meta(AUTH_ERROR, "Backend", "auth.rs", 14)];
        let report = unclaimed_variants(CRATE, &specs, &outcomes);
        assert_eq!(report, ["crate::auth::AuthError::Backend (auth.rs:14)"]);
    }

    /// A consumer's binary links this crate's registrations beside its own, so
    /// the enums in scope are the ones this crate's claims own: a foreign
    /// claim's enum is another crate's business, and reporting its variants
    /// would fail the wrong crate.
    ///
    /// The foreign claim owns an enum no claim of [`CRATE`] shares, and the
    /// registered variant is one the foreign claim does not name — so an
    /// unscoped answer reports it and the scoped one does not.
    #[test]
    #[validates(spec::UnclaimedVariantsScopeToTheInvokingCrate)]
    fn unclaimed_variants_scope_to_the_invoking_crate() {
        let specs = [spec_meta(
            "other_crate::Fake",
            Pattern::Unwanted,
            "other_crate::foreign::ForeignError::Refused",
            FOREIGN_ERROR,
        )];
        let outcomes = [outcome_meta(FOREIGN_ERROR, "Backend", "foreign.rs", 3)];
        let report = unclaimed_variants(CRATE, &specs, &outcomes);
        assert!(report.is_empty(), "a foreign crate's claim puts no enum in scope: {report:?}");
    }
}

pub mod canary;

pub mod spec;
