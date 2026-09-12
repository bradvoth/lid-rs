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

pub mod canary;

pub mod spec;
