//! The companion of the controlled-language slice: the types a claim's
//! registration names, and the tests that exercise the derive. The design is
//! `lid-rs-macros/docs/intent/claim/lld.md`.
//!
//! A claim is written in a controlled language — one sentence, one `shall`, a
//! pattern from its opener, a verb the lexicon defines — and `derive(Spec)`
//! executes that language at the struct that carries it. What the derive reads
//! lands here, in [`ClaimMeta`], so that a later check can ask whether the code
//! keeps what the claim says rather than only whether the claim is cited.
//!
//! The derive itself is in `lid-rs-macros`, which links into no binary: it
//! registers nothing and can cite nothing, so its implementation edges are
//! hand-authored at the re-export in this crate's root.

use crate::registry::SpecMeta;
use crate::spec;
use lid_rs::implements;

/// The five patterns a claim's opener names, as an enum a registration can
/// name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// `When …,` — the claim is triggered by an event.
    EventDriven,
    /// `If …, then` — the claim is triggered by an unwanted condition.
    Unwanted,
    /// `While …,` — the claim holds while a state does.
    StateDriven,
    /// `Where …,` — the claim holds where a feature is present.
    Optional,
    /// Any other opener: the claim holds unconditionally. Also the pattern
    /// recorded for a claim marked free of the language, which has no opener to
    /// read.
    Ubiquitous,
}

/// Whether a claim's parts were read from the controlled language, or the claim
/// was marked free of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// The claim parsed: every part of its [`ClaimMeta`] is the derive's
    /// reading of the sentence.
    Controlled,
    /// The claim carries `#[lid(free)]`, and no rule of the language but the
    /// unit-struct rule was applied to it. The registry counts these, and
    /// [`free`] enumerates them: they are the ramp, and the burn-down is per
    /// slice.
    Free,
}

/// The parts `derive(Spec)` extracted from one claim: the `claim` field of
/// every [`SpecMeta`].
///
/// Every field is a literal or an enum, because a registration is a `static`.
/// A part the claim does not carry — the object of a behaviour verb, the owner
/// of an object that names no variant — is empty rather than absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClaimMeta {
    /// [`Language::Controlled`] for a parsed claim, [`Language::Free`] for a
    /// marked one.
    pub language: Language,
    /// The pattern the opener named; [`Pattern::Ubiquitous`] for a free claim.
    pub pattern: Pattern,
    /// The trigger link's target as written, or empty.
    pub trigger: &'static str,
    /// The verb, lower-cased, or empty.
    pub verb: &'static str,
    /// Whether `not` stood between the modal and the verb.
    pub negated: bool,
    /// The response object's target as written, or empty.
    pub object: &'static str,
    /// The response object's target with its last segment removed, as written,
    /// or empty. A template's `{owner}` binds to its last segment, which is the
    /// name a signature spells.
    pub owner: &'static str,
    /// The verb's templates as the lexicon writes them, unmatched against any
    /// signature; `["*"]` for a behaviour verb, empty for a free claim.
    pub templates: &'static [&'static str],
}

/// The registered claims marked `#[lid(free)]`, from [`SPECS`](crate::SPECS),
/// in registry order.
///
/// The exemption from the controlled language is the one thing the registry
/// counts: this is the enumeration a test asserts against, and the number that
/// says how far a project is from having none.
#[implements(spec::FreeEnumeratesTheMarkedClaims)]
pub fn free() -> impl Iterator<Item = &'static SpecMeta> {
    crate::SPECS.iter().filter(|meta| is_free(meta))
}

/// Whether a registration's claim was marked free of the controlled language.
///
/// The one question [`free`] asks of each entry, and the one the burn-down is
/// counted by: a claim is either the derive's reading of a sentence or an
/// exemption from reading it, and [`Language`] is where the derive says which.
fn is_free(meta: &SpecMeta) -> bool {
    let _ = meta;
    todo!()
}

#[cfg(test)]
mod tests {
    //! The demonstrations of check 13 and check 14, and the enumeration of the
    //! ramp.
    //!
    //! Two mechanisms, because one cannot reach a project lexicon. trybuild
    //! compiles every fixture as a bin of one package it generates, whose parent
    //! directory holds no `Cargo.toml`: the walk from such a fixture examines one
    //! directory that has no `docs/intent/lexicon.toml` and stops at its parent
    //! for want of a manifest, so a lexicon file placed beside a trybuild fixture
    //! is never read and every trybuild fixture answers to the base alone. Every
    //! rule the base can break is therefore demonstrated by
    //! `tests/ui/claim/{fail,pass}`, and every rule about which lexicon governs
    //! by the fixture workspace under `tests/ui/claim/lexicon`, whose members are
    //! real crates in real directories.
    //!
    //! Each harness cites every claim its fixtures demonstrate, so the claims
    //! whose response is that something compiles share a validator with the
    //! siblings that fail — the cost the LLD names, and what makes them red
    //! before the derive enforces anything.

    use super::{Language, free};
    use crate::{SPECS, Spec, spec, validates};
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// Check 13 and check 14 against the base lexicon: the fixtures under
    /// `tests/ui/claim/`, whose compilation is the assertion and whose `.stderr`
    /// files pin the words each rule names.
    ///
    /// The failing fixtures break one rule apiece — a missing terminator and a
    /// second one that is a period, a question mark, and an exclamation mark in
    /// turn, the modal count, an opener whose clause is unclosed or closes after
    /// the modal, a trigger clause or ubiquitous subject with no link, a
    /// ubiquitous subject whose only link stands after the modal, a shape verb
    /// with no object, a verb the base does not define, a vague term of the
    /// built-in list plain and capitalised and two words long, the first-failure
    /// ordering, a `#[lid(…)]` that is not `free`, and five misnamed
    /// validators, the last of them citing two claims so that the name the
    /// message expects is the first cited claim's.
    ///
    /// Two of those `.stderr` files hold more than the rule's name. A claim
    /// spanning two doc lines is quoted in its message as the lines trimmed and
    /// joined by single spaces, which is where the joining rule is observed
    /// rather than only relied on; and the ubiquitous subject whose link stands
    /// after the modal is where the trigger's *in the subject* is separated from
    /// *anywhere in the sentence*, since taking the first link of the whole
    /// sentence would compile it.
    ///
    /// The passing ones assert what the derive recorded in their own `fn main`,
    /// which trybuild runs: the `ClaimMeta` of each claim through `SPECS`, and —
    /// for the validator names — the edges `validates` left in `VALIDATIONS`,
    /// which is what says a name was read and admitted rather than never read.
    /// Among them is a claim whose words open with terms of the vague-term list
    /// without being them, which is the whole-word qualifier: a matcher looking
    /// for a term as a substring rejects it and fails here.
    ///
    /// Three of the passing fixtures are about the const assertion check 14's
    /// exemption is made of. One reads `Spec::FREE` for a marked claim and an
    /// unmarked one, which is what tells a const that records the mark from a
    /// const that is `true` for every claim. One carries a correctly named
    /// validator of a *held* claim, where the assertion is not emitted at all:
    /// its compilation is the observation, since an assertion over a held
    /// claim's `FREE` is false and would fail it. One carries a deliberately
    /// misnamed validator citing only claims that carry `#[lid(free)]`, and is
    /// the fixture whose passing is the whole of its point — read against the
    /// same misnaming over held claims in `fail/validator.rs`, which is what
    /// bounds the rule from the other side. The claims of every fixture about
    /// the name stay held, except those the exemption is demonstrated over: the
    /// mark is what switches the rule off, so marking them would leave it
    /// unexercised with nothing going red.
    ///
    /// No fixture fn carries `#[test]`: trybuild compiles a fixture as a
    /// `[[bin]]` with no `--test`, where rustc's built-in `test` attribute
    /// removes the item it decorates before the attributes under it expand.
    #[test]
    #[validates(
        spec::TheRegistrationCarriesTheClaimsParts,
        spec::TheClaimIsTheDocLinesJoined,
        spec::AClaimWithoutATerminatorFailsToCompile,
        spec::ASecondTerminatorFailsToCompile,
        spec::AModalCountOtherThanOneFailsToCompile,
        spec::TheOpenerDecidesThePattern,
        spec::AnyOtherOpenerIsUbiquitous,
        spec::TheClauseIsTheWordsUpToTheFirstCommaOutsideBackticks,
        spec::AnIfClauseIsTheWordsUpToCommaThen,
        spec::AnUnclosedClauseFailsToCompile,
        spec::ALinkWithAPathTargetsThePath,
        spec::ABareLinkTargetsItsBacktickedText,
        spec::TheTriggerIsTheFirstLinkInTheClause,
        spec::AUbiquitousTriggerIsTheFirstLinkInTheSubject,
        spec::ATriggerClauseWithoutALinkFailsToCompile,
        spec::TheVerbIsTheFirstWordAfterTheModal,
        spec::NotBeforeTheVerbIsRecordedAsNegation,
        spec::TheObjectIsTheFirstLinkAfterTheVerb,
        spec::NoLinkAfterABehaviourVerbLeavesTheObjectEmpty,
        spec::TheOwnerIsTheObjectTargetWithItsLastSegmentRemoved,
        spec::TheOwnerIsEmptyForAnObjectWithoutAVariant,
        spec::AShapeVerbWithoutAnObjectLinkFailsToCompile,
        spec::AnUndefinedVerbFailsToCompile,
        spec::AProhibitedTermFailsToCompile,
        spec::TheFirstFailingRuleIsReported,
        spec::AParsedClaimIsControlled,
        spec::AShapeVerbsTemplatesAreRecordedAsWritten,
        spec::ABehaviourVerbRecordsTheStarTemplate,
        spec::TheBaseLexiconIsCompiledIntoTheDerive,
        spec::AMisnamedValidatorFailsToCompile,
        spec::ASuffixedValidatorNameIsAdmitted,
        spec::AValidatorIsNamedForAnyOneCitedClaim,
        spec::AnAdmittedNameCarriesNoAssertion,
        spec::AValidatorCitingOnlyFreeClaimsIsNotHeldToTheName,
        spec::SnakeCaseStartsAWordAtACapitalBeforeALowerCaseLetter,
        spec::SnakeCaseKeepsARunOfCapitalsAsOneWord,
        spec::SnakeCaseKeepsADigitWithTheWordBeforeIt,
        spec::AFreeClaimCompilesWhateverItsText,
        spec::AFreeClaimRecordsEmptyParts,
        spec::TheFreeConstRecordsTheMark,
        spec::TheMarkTakesOnlyTheWordFree
    )]
    fn the_registration_carries_the_claims_parts() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/claim/fail/*.rs");
        t.pass("tests/ui/claim/pass/*.rs");
    }

    /// Each member of the fixture workspace, as the source file its diagnostics
    /// point at, with the words those diagnostics must hold.
    ///
    /// A member's lexicon is its own and no other's, so the words are what only
    /// that member's lexicon could have produced: the verb it alone defines, the
    /// term it alone prohibits, the file it alone names. `included` is absent
    /// because it compiles — what it demonstrates is read from cargo's dep-info,
    /// not from a message.
    ///
    /// A rule stating a disjunction gets one member per branch, because a member
    /// demonstrates the branch it carries and no other: `signature` missing and
    /// `signature` repeated, `def` missing and `def` repeated, a key the subset
    /// does not admit under a verb and one under `[prohibited]`, a template that
    /// begins with no `->` and one whose braces do not balance and one whose
    /// placeholder binds nothing.
    ///
    /// The two `def` members are the corners nothing else would find: a parsed
    /// lexicon carries only the templates, so an implementation that never reads
    /// a definition passes every other member here while admitting a verb the
    /// glossary never glossed. No expected word of theirs occurs in their own
    /// path, so each assertion is the message and not the path.
    const LEXICON_FIXTURES: [(&str, &[&str]); 18] = [
        ("bad_line/src/lib.rs", &["bad_line/docs/intent/lexicon.toml", "this line belongs to no form"]),
        ("missing_key/src/lib.rs", &["missing_key/docs/intent/lexicon.toml", "partial", "signature"]),
        ("repeated_key/src/lib.rs", &["repeated_key/docs/intent/lexicon.toml", "doubled", "signature"]),
        ("gloss_missing/src/lib.rs", &["gloss_missing/docs/intent/lexicon.toml", "unglossed", "def"]),
        ("gloss_repeated/src/lib.rs", &["gloss_repeated/docs/intent/lexicon.toml", "restated", "def"]),
        ("unknown_key/src/lib.rs", &["unknown_key/docs/intent/lexicon.toml", "remark"]),
        ("prohibited_key/src/lib.rs", &["prohibited_key/docs/intent/lexicon.toml", "banned"]),
        ("verb_twice/src/lib.rs", &["verb_twice/docs/intent/lexicon.toml", "duplicated"]),
        ("prohibited_repeat/src/lib.rs", &["prohibited_repeat/docs/intent/lexicon.toml", "extra"]),
        ("bad_template/src/lib.rs", &["bad_template/docs/intent/lexicon.toml", "shaped", "Result<{object}, _>"]),
        ("unbalanced_template/src/lib.rs", &[
            "unbalanced_template/docs/intent/lexicon.toml",
            "lopsided",
            "-> Result<{object, _>",
        ]),
        ("bad_placeholder/src/lib.rs", &[
            "bad_placeholder/docs/intent/lexicon.toml",
            "mistaken",
            "-> Result<{subject}, _>",
        ]),
        ("project_verb/src/lib.rs", &["stage", "response object"]),
        ("verb_replaced/src/lib.rs", &["report", "response object"]),
        ("project_extra/src/lib.rs", &["project_extra/docs/intent/lexicon.toml", "promptly"]),
        ("inherits/src/lib.rs", &["rootprohibited"]),
        ("detached/pkg/src/lib.rs", &["settle", "lid-rs-macros/lexicon.toml"]),
        ("inner/src/lib.rs", &["orbit", "lid-rs-macros/lexicon.toml"]),
    ];

    /// The path the `include_str!` of a read project lexicon must put into the
    /// dep-info of the crate that read it.
    const INCLUDED_LEXICON: &str = "included/docs/intent/lexicon.toml";

    /// Which lexicon governs a crate, and what a file outside the format's
    /// subset does: one `cargo check` over the fixture workspace, whose members
    /// differ only in where their lexicon stands and what it says.
    ///
    /// Every assertion here is a message that could come from no other member's
    /// lexicon, so the absence of one is the absence of the rule. The check runs
    /// with a target directory of its own: cargo locks a target directory, and a
    /// build inside `cargo test` must not share the outer one.
    #[test]
    #[validates(
        spec::AMemberFindsItsWorkspacesLexicon,
        spec::TheWalkStopsAtAWorkspaceManifest,
        spec::TheWalkStopsBelowADirectoryWithoutAManifest,
        spec::AProjectVerbAddsToTheBase,
        spec::AProjectVerbReplacesTheBasesWhole,
        spec::AProjectExtraFailsToCompileNamingTheLexiconFile,
        spec::TheProjectLexiconIsIncludedInTheExpansion,
        spec::ALexiconLineOutsideTheSubsetFailsEveryDerive,
        spec::AVerbMissingOrRepeatingAKeyFailsEveryDerive,
        spec::AnUnknownLexiconKeyFailsEveryDerive,
        spec::AVerbNamedTwiceFailsEveryDerive,
        spec::ExtraGivenTwiceFailsEveryDerive,
        spec::AMalformedTemplateFailsEveryDerive
    )]
    fn a_member_finds_its_workspaces_lexicon() {
        let report = check_fixture_workspace();
        for (source, expected) in LEXICON_FIXTURES {
            let said = diagnostics_for(&report, source);
            for word in expected {
                assert!(said.contains(word), "{source}: no diagnostic names `{word}`\n{report}");
            }
        }
        assert!(
            dep_info_mentions(&fixture_target(), INCLUDED_LEXICON),
            "the expansion includes no `{INCLUDED_LEXICON}`, so editing it rebuilds nothing\n{report}"
        );
    }

    /// The fixture workspace, checked whole, with its diagnostics captured.
    ///
    /// `--keep-going` is what makes one run enough: without it cargo stops
    /// scheduling after the first member fails, and the members it never checked
    /// would look like rules that never fired.
    fn check_fixture_workspace() -> String {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
        let output = Command::new(cargo)
            .current_dir(fixture_workspace())
            .args(["check", "--workspace", "--keep-going"])
            .env("CARGO_TARGET_DIR", fixture_target())
            .output()
            .expect("cargo check over the fixture workspace");
        String::from_utf8_lossy(&output.stderr).into_owned()
    }

    /// The fixture workspace's root: a virtual manifest whose members are the
    /// crates one lexicon rule apiece is demonstrated in.
    fn fixture_workspace() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ui/claim/lexicon")
    }

    /// The target directory the fixture workspace is checked into, beside the
    /// outer one and locked separately from it.
    fn fixture_target() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/claim-lexicon")
    }

    /// The diagnostics of one check that point at `source`, joined.
    ///
    /// A lexicon failure fails every claim in the crate that read the file, and
    /// each crate is one member, so the diagnostics naming a member's source are
    /// what its own lexicon produced and nothing else's.
    fn diagnostics_for(report: &str, source: &str) -> String {
        report.split("\nerror").filter(|block| block.contains(source)).collect::<Vec<&str>>().join("\n")
    }

    /// Whether any dep-info cargo wrote under `target` names `needle`.
    ///
    /// rustc records every file a compilation read, `include_str!`ed files among
    /// them, and cargo rebuilds when one of them changes. So the lexicon's path
    /// standing here is the whole of "editing the file rebuilds the claim".
    fn dep_info_mentions(target: &Path, needle: &str) -> bool {
        std::fs::read_dir(target.join("debug/deps"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|kind| kind == "d"))
            .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
            .any(|text| text.contains(needle))
    }

    /// The ramp, enumerated: `free` yields the registrations marked
    /// `#[lid(free)]`, in registry order, and nothing else.
    ///
    /// A claim written in the controlled language — every claim of this slice —
    /// is not among them, and a claim marked free is. That is the number the
    /// burn-down is counted by, so the test says how far this workspace is from
    /// having none.
    #[test]
    #[validates(spec::FreeEnumeratesTheMarkedClaims)]
    fn free_enumerates_the_marked_claims() {
        let marked: Vec<&str> = free().map(|meta| meta.name).collect();
        let registered: Vec<&str> =
            SPECS.iter().filter(|meta| meta.claim.language == Language::Free).map(|meta| meta.name).collect();
        assert_eq!(marked, registered, "`free` is the marked registrations, in registry order");
        assert!(
            !marked.contains(&<spec::FreeEnumeratesTheMarkedClaims as Spec>::NAME),
            "a claim written in the controlled language is not an exemption from it"
        );
        assert!(
            marked.contains(&<spec::LinkedRegistrationsAreEnumerable as Spec>::NAME),
            "a claim marked `#[lid(free)]` is one: {} claims are still marked",
            marked.len()
        );
    }
}
