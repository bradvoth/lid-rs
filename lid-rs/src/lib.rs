#![doc = include_str!("../docs/intent/hld.md")]

// Macro expansions address this crate as `::lid_rs::…`; this makes that path
// resolve inside `lid-rs` itself. Consequence: downstream renames of the
// dependency are unsupported (see the registry LLD).
extern crate self as lid_rs;

pub mod claim;
pub mod graph;
pub mod registry;

// `canary` is a leaf of the `registry` slice and lives beside it, but its path
// is public API: `$crate::canary` is what `intent_graph!` expands to in every
// downstream crate. The re-export keeps that path while the file colocates.
pub use registry::canary;
pub mod lid_rs_macros;

#[cfg(test)]
mod intent_graph {
    //! lid's own instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}

pub use registry::{Edge, IMPLEMENTATIONS, SPECS, SpecMeta, VALIDATIONS};

pub use ::lid_rs_macros::{Spec, implements, implements_module, spec, validates};

// Hand-authored implementation edges for the citation claims: lid-rs-macros is a
// proc-macro crate, which links into no target binary and so can neither
// carry citations nor register anything — its edges live here, at the
// re-export boundary that is its public surface. This is the standing
// exception for proc-macro crates, not bootstrap residue.
#[doc = "Implements [`crate::lid_rs_macros::spec::DerivedSpecsCarryTheirDefinitionPath`], \
[`crate::lid_rs_macros::spec::DerivedSpecsRegisterIntoSpecs`], \
[`crate::lid_rs_macros::spec::ImplementsCitationsRegisterEdges`], \
[`crate::lid_rs_macros::spec::ValidatesCitationsRegisterEdges`], \
[`crate::lid_rs_macros::spec::ModuleCitationsTraceByContainment`], \
[`crate::lid_rs_macros::spec::MalformedCitationsFailToCompile`]."]
const _: () = {
    /// One hand edge per (claim, macro item) pair.
    macro_rules! macro_edge {
        ($spec:path, $item:literal) => {
            const _: () = {
                #[allow(missing_docs, clippy::missing_docs_in_private_items)]
                #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::IMPLEMENTATIONS)]
                #[linkme(crate = ::lid_rs::__private::linkme)]
                static EDGE: ::lid_rs::Edge = ::lid_rs::Edge {
                    spec: <$spec as ::lid_rs::Spec>::NAME,
                    item: $item,
                    file: file!(),
                    line: line!(),
                };
            };
        };
    }
    macro_edge!(crate::lid_rs_macros::spec::DerivedSpecsCarryTheirDefinitionPath, "lid_rs_macros::Spec");
    macro_edge!(crate::lid_rs_macros::spec::DerivedSpecsRegisterIntoSpecs, "lid_rs_macros::Spec");
    macro_edge!(crate::lid_rs_macros::spec::ImplementsCitationsRegisterEdges, "lid_rs_macros::implements");
    macro_edge!(crate::lid_rs_macros::spec::ValidatesCitationsRegisterEdges, "lid_rs_macros::validates");
    macro_edge!(crate::lid_rs_macros::spec::ModuleCitationsTraceByContainment, "lid_rs_macros::implements_module");
    macro_edge!(crate::lid_rs_macros::spec::MalformedCitationsFailToCompile, "lid_rs_macros::expand");
};

// The same exception, for the controlled-language slice: the language is
// executed by `lid_rs_macros::claim`, whose items can carry no citation, so one
// hand edge per (claim, item) pair stands here. The item each names is the
// function or method the claim is kept by, never a module or a bare type: a
// claim is kept by a code path, and the path is what a reader of the edge needs
// to find. `lid_rs::claim::free` is ordinary code and cites itself.
const _: () = {
    /// One hand edge per (claim, item) pair, as above.
    macro_rules! claim_edge {
        ($spec:path, $item:literal) => {
            const _: () = {
                #[allow(missing_docs, clippy::missing_docs_in_private_items)]
                #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::IMPLEMENTATIONS)]
                #[linkme(crate = ::lid_rs::__private::linkme)]
                static EDGE: ::lid_rs::Edge = ::lid_rs::Edge {
                    spec: <$spec as ::lid_rs::Spec>::NAME,
                    item: $item,
                    file: file!(),
                    line: line!(),
                };
            };
        };
    }

    // One sentence, one modal.
    claim_edge!(crate::claim::spec::TheClaimIsTheDocLinesJoined, "lid_rs_macros::claim::sentence");
    claim_edge!(crate::claim::spec::AClaimWithoutATerminatorFailsToCompile, "lid_rs_macros::claim::sentence");
    claim_edge!(crate::claim::spec::ASecondTerminatorFailsToCompile, "lid_rs_macros::claim::sentence");
    claim_edge!(crate::claim::spec::AModalCountOtherThanOneFailsToCompile, "lid_rs_macros::claim::parse");

    // The pattern, from the opener.
    claim_edge!(crate::claim::spec::TheOpenerDecidesThePattern, "lid_rs_macros::claim::pattern");
    claim_edge!(crate::claim::spec::AnyOtherOpenerIsUbiquitous, "lid_rs_macros::claim::pattern");
    claim_edge!(crate::claim::spec::TheClauseIsTheWordsUpToTheFirstCommaOutsideBackticks, "lid_rs_macros::claim::pattern");
    claim_edge!(crate::claim::spec::AnIfClauseIsTheWordsUpToCommaThen, "lid_rs_macros::claim::pattern");
    claim_edge!(crate::claim::spec::AnUnclosedClauseFailsToCompile, "lid_rs_macros::claim::pattern");

    // The parts, and which of them must be links.
    claim_edge!(crate::claim::spec::ALinkWithAPathTargetsThePath, "lid_rs_macros::claim::link");
    claim_edge!(crate::claim::spec::ABareLinkTargetsItsBacktickedText, "lid_rs_macros::claim::link");
    claim_edge!(crate::claim::spec::TheTriggerIsTheFirstLinkInTheClause, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::AUbiquitousTriggerIsTheFirstLinkInTheSubject, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::ATriggerClauseWithoutALinkFailsToCompile, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::TheVerbIsTheFirstWordAfterTheModal, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::NotBeforeTheVerbIsRecordedAsNegation, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::TheObjectIsTheFirstLinkAfterTheVerb, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::NoLinkAfterABehaviourVerbLeavesTheObjectEmpty, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::TheOwnerIsTheObjectTargetWithItsLastSegmentRemoved, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::TheOwnerIsEmptyForAnObjectWithoutAVariant, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::AShapeVerbWithoutAnObjectLinkFailsToCompile, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::AnUndefinedVerbFailsToCompile, "lid_rs_macros::claim::parse");

    // Prohibited terms, and the order the rules are checked in.
    claim_edge!(crate::claim::spec::AProhibitedTermFailsToCompile, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::TheFirstFailingRuleIsReported, "lid_rs_macros::claim::parse");

    // What the derive records.
    claim_edge!(crate::claim::spec::TheRegistrationCarriesTheClaimsParts, "lid_rs_macros::claim::expansion");
    claim_edge!(crate::claim::spec::AParsedClaimIsControlled, "lid_rs_macros::claim::expansion");
    claim_edge!(crate::claim::spec::AShapeVerbsTemplatesAreRecordedAsWritten, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::ABehaviourVerbRecordsTheStarTemplate, "lid_rs_macros::claim::parse");

    // The validator's name — check 14 — and the name it is compared with.
    claim_edge!(crate::claim::spec::AMisnamedValidatorFailsToCompile, "lid_rs_macros::claim::validator_name");
    claim_edge!(crate::claim::spec::ASuffixedValidatorNameIsAdmitted, "lid_rs_macros::claim::validator_name");
    claim_edge!(crate::claim::spec::AValidatorIsNamedForAnyOneCitedClaim, "lid_rs_macros::claim::validator_name");
    claim_edge!(crate::claim::spec::AnAdmittedNameCarriesNoAssertion, "lid_rs_macros::claim::validator_name");
    claim_edge!(crate::claim::spec::AValidatorCitingOnlyFreeClaimsIsNotHeldToTheName, "lid_rs_macros::claim::validator_name");
    claim_edge!(crate::claim::spec::SnakeCaseStartsAWordAtACapitalBeforeALowerCaseLetter, "lid_rs_macros::claim::snake_case");
    claim_edge!(crate::claim::spec::SnakeCaseKeepsARunOfCapitalsAsOneWord, "lid_rs_macros::claim::snake_case");
    claim_edge!(crate::claim::spec::SnakeCaseKeepsADigitWithTheWordBeforeIt, "lid_rs_macros::claim::snake_case");

    // The ramp. `FreeEnumeratesTheMarkedClaims` is cited by `claim::free` itself.
    claim_edge!(crate::claim::spec::AFreeClaimCompilesWhateverItsText, "lid_rs_macros::claim::expansion");
    claim_edge!(crate::claim::spec::AFreeClaimRecordsEmptyParts, "lid_rs_macros::claim::expansion");
    claim_edge!(crate::claim::spec::TheFreeConstRecordsTheMark, "lid_rs_macros::claim::expansion");
    claim_edge!(crate::claim::spec::TheMarkTakesOnlyTheWordFree, "lid_rs_macros::claim::expansion");

    // The lexicon: base, project, and the walk.
    claim_edge!(crate::claim::spec::TheBaseLexiconIsCompiledIntoTheDerive, "lid_rs_macros::claim::lexicon::read");
    claim_edge!(crate::claim::spec::AProjectVerbAddsToTheBase, "lid_rs_macros::claim::lexicon::read");
    claim_edge!(crate::claim::spec::AProjectVerbReplacesTheBasesWhole, "lid_rs_macros::claim::lexicon::read");
    claim_edge!(crate::claim::spec::AProjectExtraFailsToCompileNamingTheLexiconFile, "lid_rs_macros::claim::parse");
    claim_edge!(crate::claim::spec::AMemberFindsItsWorkspacesLexicon, "lid_rs_macros::claim::lexicon::locate");
    claim_edge!(crate::claim::spec::TheWalkStopsAtAWorkspaceManifest, "lid_rs_macros::claim::lexicon::locate");
    claim_edge!(crate::claim::spec::TheWalkStopsBelowADirectoryWithoutAManifest, "lid_rs_macros::claim::lexicon::locate");
    claim_edge!(crate::claim::spec::TheProjectLexiconIsIncludedInTheExpansion, "lid_rs_macros::claim::expansion");

    // The lexicon's format, and what a file outside it does.
    claim_edge!(crate::claim::spec::ALexiconLineOutsideTheSubsetFailsEveryDerive, "lid_rs_macros::claim::lexicon::parse");
    claim_edge!(crate::claim::spec::AVerbMissingOrRepeatingAKeyFailsEveryDerive, "lid_rs_macros::claim::lexicon::parse");
    claim_edge!(crate::claim::spec::AnUnknownLexiconKeyFailsEveryDerive, "lid_rs_macros::claim::lexicon::parse");
    claim_edge!(crate::claim::spec::AVerbNamedTwiceFailsEveryDerive, "lid_rs_macros::claim::lexicon::parse");
    claim_edge!(crate::claim::spec::ExtraGivenTwiceFailsEveryDerive, "lid_rs_macros::claim::lexicon::parse");
    claim_edge!(crate::claim::spec::AMalformedTemplateFailsEveryDerive, "lid_rs_macros::claim::lexicon::Template::parse");
};

/// Trait implemented by every claim item, via `derive(Spec)`.
///
/// The registration statics emitted by `#[implements]` and `#[validates]`
/// produce their join key as `<cited::Path as lid_rs::Spec>::NAME`, which is what
/// turns a citation of a renamed or deleted claim into a compile error (and
/// surfaces `#[deprecated]` on the spec at every citation site).
pub trait Spec {
    /// Canonical name of the claim: definition-site `module_path!()` plus the
    /// item's identifier. The registry join key — single-sourced here, so
    /// every citation agrees on it no matter which path or re-export the
    /// citing site wrote.
    const NAME: &'static str;

    /// Whether the claim carries `#[lid(free)]` — the ramp's mark, exempting
    /// it from the controlled language.
    ///
    /// A citation can project this where it cannot read the mark itself: an
    /// attribute macro sees the paths it was given and nothing about the items
    /// they name. `#[validates]` compares the test's name at expansion and,
    /// when no cited path is admitted, emits a const assertion over the
    /// conjunction of the cited claims' `FREE` — so a misnamed validator of a
    /// held claim fails `cargo check` and one citing only free claims compiles
    /// (check 14).
    const FREE: bool;
}

#[doc(hidden)]
pub mod __private {
    pub use linkme;
}
