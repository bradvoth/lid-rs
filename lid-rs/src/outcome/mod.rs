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

/// One unwanted claim's owner, resolved through the type the claim names.
///
/// `derive(Spec)` registers one of these for each claim whose pattern is
/// [`Pattern::Unwanted`](crate::claim::Pattern::Unwanted) and whose `owner` is
/// non-empty, from the same const block that carries E1's bound — so an entry
/// stands exactly where the compiler has already proved that the owner is an
/// [`Outcome`] and that the object's last segment is one of its variants.
///
/// What it adds to what the claim already registers is the *spelling*: the
/// owner read as [`Outcome::NAME`] rather than as the path the claim's author
/// wrote. Both sides of the intersection then produce their key from one const,
/// which is the join principle [`SpecMeta::name`] states.
///
/// It carries no file or line, unlike [`OutcomeMeta`]. A site is registered so
/// that a report can send a reader to it, and what this join feeds is a report
/// of the *variant's* site, which [`OutcomeMeta`] already carries. The claim's
/// own site is registered once too, on the [`SpecMeta`] this entry joins by
/// [`spec`](ClaimedOwner::spec), so a copy here would answer nothing new and be
/// a second place to keep right.
#[derive(Debug)]
pub struct ClaimedOwner {
    /// The claim's [`Spec::NAME`](crate::Spec::NAME) (joins [`SpecMeta::name`]).
    pub spec: &'static str,
    /// The owning enum's [`Outcome::NAME`], read through the resolved type.
    ///
    /// This may be spelled differently from the same claim's
    /// [`ClaimMeta::owner`](crate::claim::ClaimMeta::owner), which is the path
    /// its author wrote; it is this spelling, not that one, that meets
    /// [`OutcomeMeta::owner`].
    pub owner: &'static str,
    /// The identifier of the variant the claim's `object` names, as the owning
    /// enum declares it.
    pub variant: &'static str,
}

/// The resolved owner of every unwanted claim whose object names a variant,
/// gathered at link time.
///
/// The fifth distributed slice, beside [`OUTCOMES`] and the triple. It supplies
/// a spelling and nothing else: which claims put an enum in scope is still
/// [`claimed_owners`]'s answer, read from the registered claims themselves.
#[distributed_slice]
pub static CLAIMED_OWNERS: [ClaimedOwner];

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
/// Each owner is carried as [`Outcome::NAME`] resolves it, which is what
/// `owners` is for: the claim registers the path its author wrote, and
/// [`CLAIMED_OWNERS`] registers the same enum's own key beside it. The two
/// sides of the intersection then come from one const and cannot disagree
/// however a path is spelled or re-exported.
///
/// `owners` supplies that spelling and decides nothing. Which claims are owners
/// is answered here and from `specs` alone — the pattern and the emptiness of
/// the owner are properties of the claim — so an entry for a claim this
/// function passes over is passed over with it.
///
/// A `Vec` and not an `impl Iterator`: nothing here depends on laziness, and
/// `!` coerces to a `Vec` where it does not implement [`Iterator`] — which is
/// what lets this signature stand before its body does.
#[implements(
    spec::AnUnwantedClaimsOwnerIsAClaimedOwner,
    spec::AClaimOutsideTheUnwantedPatternIsNoClaimedOwner,
    spec::AnUnwantedClaimWithAnEmptyOwnerIsNoClaimedOwner,
)]
pub fn claimed_owners(specs: &[SpecMeta], owners: &[ClaimedOwner]) -> Vec<&'static str> {
    todo!("claimed_owners over {} specs and {} owners", specs.len(), owners.len())
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
///
/// `owners` is passed through whole to everything that has to spell an owner,
/// because it answers *how* an owner is written and never *whether* a claim has
/// one: the claims decide that, and the crate scope applies to them.
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
    owners: &[ClaimedOwner],
    outcomes: &[OutcomeMeta],
) -> Vec<String> {
    let in_scope: Vec<&'static str> = specs
        .iter()
        .filter(|spec| registered_by(crate_name, spec))
        .flat_map(|spec| claimed_owners(std::slice::from_ref(spec), owners))
        .collect();
    outcomes
        .iter()
        .filter(|outcome| in_scope.iter().any(|owner| owner_names_enum(owner, outcome.owner)))
        .filter(|outcome| !specs.iter().any(|spec| claim_names_variant(spec, owners, outcome)))
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
/// The one comparison in the intersection, and both sides of it are the enum's
/// own [`Outcome::NAME`]: the registration reads it through the enum, and the
/// claim side reads it through the owner the claim's author named, by way of
/// [`CLAIMED_OWNERS`]. So what this reconciles is one const against itself, and
/// not two paths that were written independently — which is what keeps
/// `Registry::Kind` and `other::Registry::Kind` two enums rather than one.
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
/// The claim side of the intersection, read from the registrations alone: the
/// owner as [`claimed_owners`] carries it — resolved through `owners`, so that
/// what is put beside a registered enum is that enum's own key — and the
/// identifier the object's last segment named. No enum is looked at here, so
/// the comparison stays in one place.
///
/// A claim that names no variant has no entry in `owners`, which is what `None`
/// reports: the derive emits one exactly where E1's bound proved there was a
/// variant to name.
#[implements(
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn claimed_variant(
    spec: &SpecMeta,
    owners: &[ClaimedOwner],
) -> Option<(&'static str, &'static str)> {
    todo!("claimed_variant of {} over {} owners", spec.name, owners.len())
}

/// Whether this claim's object names the variant an [`OutcomeMeta`] stands for.
///
/// Unscoped, unlike the owner side: a claim of any crate that names a variant
/// answers for it. What the scope decides is which enums are looked at, not
/// which of their variants someone asked for. `owners` is carried through for
/// the spelling of the claim's owner and for nothing else.
#[implements(
    spec::AVariantAnUnwantedClaimNamesIsNotReported,
    spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported,
)]
fn claim_names_variant(spec: &SpecMeta, owners: &[ClaimedOwner], outcome: &OutcomeMeta) -> bool {
    claimed_variant(spec, owners).is_some_and(|(owner, variant)| {
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
    //! join: the [`ClaimedOwner`](super::ClaimedOwner) a claim registers and the
    //! [`OutcomeMeta`](super::OutcomeMeta) it must meet carry the same string,
    //! because in the tree they both read one enum's
    //! [`NAME`](super::Outcome::NAME). No case here turns on two spellings of a
    //! resolved key disagreeing, which is a state the derive cannot produce.
    //!
    //! The `owner` the *claim itself* carries is deliberately a different
    //! spelling of that same enum — the path its author wrote, which is shorter
    //! than the one `module_path!()` resolves to. That is the one difference the
    //! registry does produce, and spelling it out is what lets these cases tell
    //! an answer read through the resolved type from one copied out of the
    //! claim.

    use super::{ClaimedOwner, OutcomeMeta, claimed_owners, spec, unclaimed_variants};
    use crate::claim::{ClaimMeta, Language, Pattern};
    use crate::registry::SpecMeta;
    use lid_rs::validates;

    /// The crate whose claims are in scope for every case below.
    const CRATE: &str = "lid_rs";

    /// The claim of [`CRATE`] every case registers, spelled once because a
    /// `ClaimedOwner` finds the claim it stands for by this name.
    const FAKE: &str = "lid_rs::synthetic::Fake";

    /// The same, for the claim another crate registered.
    const FOREIGN_FAKE: &str = "other_crate::Fake";

    /// The enum the synthetic claims own, as [`Outcome::NAME`](super::Outcome::NAME)
    /// resolves it: `module_path!()` rooted at the crate.
    ///
    /// Both sides of the join carry this one string — the `ClaimedOwner` the
    /// claim registers and the `OutcomeMeta` the enum registers — because in
    /// the tree both read it off the same const.
    const AUTH_ERROR: &str = "lid_rs::auth::AuthError";

    /// The same enum as the claim's author wrote it, which is what the claim's
    /// own `owner` carries.
    ///
    /// Shorter than [`AUTH_ERROR`] on purpose: `crate::` is what an author
    /// writes and `lid_rs::` is what the path resolves to. Were the two spelled
    /// alike, an answer that copied the claim's string and an answer that read
    /// the enum's own key would be the same answer, and no case here could tell
    /// them apart.
    const AUTH_ERROR_AS_WRITTEN: &str = "crate::auth::AuthError";

    /// The object of the synthetic unwanted claim: a variant of
    /// [`AUTH_ERROR_AS_WRITTEN`], an object being as its author wrote it.
    const AUTH_ERROR_REFUSED: &str = "crate::auth::AuthError::Refused";

    /// An enum of another crate's claim, which no claim of [`CRATE`] owns.
    ///
    /// Written crate-rooted, so its author's spelling and its resolved one
    /// coincide; nothing the scope case asks turns on their difference.
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

    /// A synthetic registration of one claim's resolved owner, in the shape
    /// `derive(Spec)` emits beside the bound it already puts on an unwanted
    /// claim's owner.
    ///
    /// `spec` is the claim this entry stands for, and `owner` is that claim's
    /// enum as its own `NAME` spells it.
    fn claimed_owner(
        spec: &'static str,
        owner: &'static str,
        variant: &'static str,
    ) -> ClaimedOwner {
        ClaimedOwner { spec, owner, variant }
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
    /// its enum in scope, and it is carried as that enum's own `NAME` spells it.
    ///
    /// The claim carries the shorter path its author wrote, so an answer taken
    /// from the claim's string is a different string from the one asserted: the
    /// owner a registered enum can be met with is the resolved one.
    #[test]
    #[validates(spec::AnUnwantedClaimsOwnerIsAClaimedOwner)]
    fn an_unwanted_claims_owner_is_a_claimed_owner() {
        let specs =
            [spec_meta(FAKE, Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR_AS_WRITTEN)];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        assert_eq!(claimed_owners(&specs, &owners), [AUTH_ERROR]);
    }

    /// A claim of another pattern puts no enum in scope, however its object is
    /// written: the pattern is what says someone asked for this way to fail.
    ///
    /// An owner *is* registered for this claim, which the derive would not do
    /// for an event-driven one. Given on purpose: the filter is this function's,
    /// so a body that only spelled out what it was handed would answer the same
    /// for both patterns and this case would not notice.
    #[test]
    #[validates(spec::AClaimOutsideTheUnwantedPatternIsNoClaimedOwner)]
    fn a_claim_outside_the_unwanted_pattern_is_no_claimed_owner() {
        let specs = [spec_meta(
            FAKE,
            Pattern::EventDriven,
            AUTH_ERROR_REFUSED,
            AUTH_ERROR_AS_WRITTEN,
        )];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        assert!(claimed_owners(&specs, &owners).is_empty(), "only an unwanted claim owns an enum");
    }

    /// An unwanted claim whose object names no variant carries the empty owner,
    /// and the empty string is not an enum: a filter on the pattern alone would
    /// carry it and put every enum in scope.
    ///
    /// An owner is registered for this claim too — what a derive reading an
    /// empty owner as a path would emit — so that the answer is the claim's
    /// emptiness and not the registry's.
    #[test]
    #[validates(spec::AnUnwantedClaimWithAnEmptyOwnerIsNoClaimedOwner)]
    fn an_unwanted_claim_with_an_empty_owner_is_no_claimed_owner() {
        let specs = [spec_meta(FAKE, Pattern::Unwanted, "crate::auth::Receipt", "")];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Receipt")];
        assert!(claimed_owners(&specs, &owners).is_empty(), "an empty owner names no enum");
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
            [spec_meta(FAKE, Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR_AS_WRITTEN)];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        let outcomes = [outcome_meta("lid_rs::store::StoreError", "Backend", "store.rs", 7)];
        let report = unclaimed_variants(CRATE, &specs, &owners, &outcomes);
        assert!(report.is_empty(), "an enum no claim owns is out of scope: {report:?}");
    }

    /// The variant an unwanted claim's object names is the one somebody did ask
    /// for, so it is passed over.
    #[test]
    #[validates(spec::AVariantAnUnwantedClaimNamesIsNotReported)]
    fn a_variant_an_unwanted_claim_names_is_not_reported() {
        let specs =
            [spec_meta(FAKE, Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR_AS_WRITTEN)];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        let outcomes = [outcome_meta(AUTH_ERROR, "Refused", "auth.rs", 12)];
        let report = unclaimed_variants(CRATE, &specs, &owners, &outcomes);
        assert!(report.is_empty(), "a claimed variant is not reported: {report:?}");
    }

    /// The rule's whole point: an enum that owns one claimed variant is in
    /// scope, and its other variants are ways to fail that no requirement asked
    /// for.
    #[test]
    #[validates(spec::AVariantOfAClaimedOwnerThatNoUnwantedClaimNamesIsReported)]
    fn a_variant_of_a_claimed_owner_that_no_unwanted_claim_names_is_reported() {
        let specs =
            [spec_meta(FAKE, Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR_AS_WRITTEN)];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        let outcomes = [
            outcome_meta(AUTH_ERROR, "Refused", "auth.rs", 12),
            outcome_meta(AUTH_ERROR, "Backend", "auth.rs", 14),
        ];
        let report = unclaimed_variants(CRATE, &specs, &owners, &outcomes);
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
            [spec_meta(FAKE, Pattern::Unwanted, AUTH_ERROR_REFUSED, AUTH_ERROR_AS_WRITTEN)];
        let owners = [claimed_owner(FAKE, AUTH_ERROR, "Refused")];
        let outcomes = [outcome_meta(AUTH_ERROR, "Backend", "auth.rs", 14)];
        let report = unclaimed_variants(CRATE, &specs, &owners, &outcomes);
        assert_eq!(report, ["lid_rs::auth::AuthError::Backend (auth.rs:14)"]);
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
            FOREIGN_FAKE,
            Pattern::Unwanted,
            "other_crate::foreign::ForeignError::Refused",
            FOREIGN_ERROR,
        )];
        let owners = [claimed_owner(FOREIGN_FAKE, FOREIGN_ERROR, "Refused")];
        let outcomes = [outcome_meta(FOREIGN_ERROR, "Backend", "foreign.rs", 3)];
        let report = unclaimed_variants(CRATE, &specs, &owners, &outcomes);
        assert!(report.is_empty(), "a foreign crate's claim puts no enum in scope: {report:?}");
    }
}

pub mod canary;

pub mod spec;
