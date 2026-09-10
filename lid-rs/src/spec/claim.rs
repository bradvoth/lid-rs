//! Claims for the controlled-language slice. Derived from
//! `lid-rs-macros/docs/intent/claim/lld.md`.
//!
//! The implementing code is the `Spec` derive and the `validates` attribute in
//! `lid-rs-macros`, a proc-macro crate that links into no target binary and
//! therefore cannot carry citations itself; its implementation edges are
//! hand-authored at the re-export site in `lid-rs`'s crate root, and the types
//! its output names live in the companion module `lid_rs::claim`.
//!
//! These claims are the first written in the language they describe: one
//! sentence, one `shall`, the trigger and — for a shape verb — the response
//! object as intra-doc links, and a verb the base lexicon defines. The
//! trigger of a claim about the derive links the derive macro itself,
//! disambiguated from the trait of the same name; the trigger of a claim
//! about check 14 links the `validates` attribute, whose expansion compares
//! the name and emits the assertion that check ends in; the next slice
//! replaces those targets with vocabulary types.

use lid_rs::Spec;

// ---- The language: one sentence, one modal ----------------------------------

/// When a struct deriving [`Spec`](derive@crate::Spec) carries several doc
/// lines, its claim shall be those lines joined by single spaces and trimmed.
#[derive(Spec)]
pub struct TheClaimIsTheDocLinesJoined;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) does not
/// end with a period, compilation shall fail naming the missing period.
#[derive(Spec)]
pub struct AClaimWithoutATerminatorFailsToCompile;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) holds a
/// period or a question mark or an exclamation mark outside backticks before
/// its final period, compilation shall fail naming the second terminator.
#[derive(Spec)]
pub struct ASecondTerminatorFailsToCompile;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) holds
/// `shall` outside backticks a number of times other than once, compilation
/// shall fail naming the count found.
#[derive(Spec)]
pub struct AModalCountOtherThanOneFailsToCompile;

// ---- The pattern, from the opener -------------------------------------------

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// `When` or `If` or `While` or `Where` in any case, its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall carry the
/// [`Pattern`](crate::claim::Pattern) that opener names: event-driven,
/// unwanted, state-driven, or optional.
#[derive(Spec)]
pub struct TheOpenerDecidesThePattern;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// any word other than the four openers, its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall carry
/// [`Pattern::Ubiquitous`](crate::claim::Pattern::Ubiquitous).
#[derive(Spec)]
pub struct AnyOtherOpenerIsUbiquitous;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// `When` or `While` or `Where` in any case, its clause shall be the words from
/// the opener up to the first comma outside backticks.
#[derive(Spec)]
pub struct TheClauseIsTheWordsUpToTheFirstCommaOutsideBackticks;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// `If` in any case, its clause shall be the words from the opener up to the
/// first `, then` outside backticks.
#[derive(Spec)]
pub struct AnIfClauseIsTheWordsUpToCommaThen;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// one of the four openers and does not close its clause as that pattern
/// requires, compilation shall fail naming the opener and the expected `,` or
/// `, then`.
#[derive(Spec)]
pub struct AnUnclosedClauseFailsToCompile;

// ---- The parts, and which of them must be links -----------------------------

/// When a link in the claim of a struct deriving [`Spec`](derive@crate::Spec)
/// carries a parenthesised path, the target the derive records for it shall be
/// that path rather than the bracketed text.
#[derive(Spec)]
pub struct ALinkWithAPathTargetsThePath;

/// When a link in the claim of a struct deriving [`Spec`](derive@crate::Spec)
/// is a backticked name in brackets with no parenthesised path, the target the
/// derive records for it shall be the backticked name.
#[derive(Spec)]
pub struct ABareLinkTargetsItsBacktickedText;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) opens with
/// one of the four openers, the `trigger` of its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall be the target of the first
/// link in the opener's clause.
#[derive(Spec)]
pub struct TheTriggerIsTheFirstLinkInTheClause;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) is
/// ubiquitous, the `trigger` of its [`ClaimMeta`](crate::claim::ClaimMeta)
/// shall be the target of the first link in the subject.
#[derive(Spec)]
pub struct AUbiquitousTriggerIsTheFirstLinkInTheSubject;

/// When the clause a pattern requires a link in — the trigger clause or a
/// ubiquitous subject — holds no link in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec), compilation shall fail naming the clause.
#[derive(Spec)]
pub struct ATriggerClauseWithoutALinkFailsToCompile;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) parses,
/// the `verb` of its [`ClaimMeta`](crate::claim::ClaimMeta) shall be the
/// first word after `shall` other than `not`, lower-cased.
#[derive(Spec)]
pub struct TheVerbIsTheFirstWordAfterTheModal;

/// When `not` stands between `shall` and the verb in the claim of a struct
/// deriving [`Spec`](derive@crate::Spec), the `negated` field of its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall be `true`.
#[derive(Spec)]
pub struct NotBeforeTheVerbIsRecordedAsNegation;

/// When a link follows the verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec), the `object` of its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall be the target of the first
/// such link.
#[derive(Spec)]
pub struct TheObjectIsTheFirstLinkAfterTheVerb;

/// When no link follows a behaviour verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec), the `object` of its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall be empty.
#[derive(Spec)]
pub struct NoLinkAfterABehaviourVerbLeavesTheObjectEmpty;

/// When the `object` target in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) ends in two capitalised segments, the `owner`
/// of its [`ClaimMeta`](crate::claim::ClaimMeta) shall be that target with its
/// last segment removed, as written.
#[derive(Spec)]
pub struct TheOwnerIsTheObjectTargetWithItsLastSegmentRemoved;

/// When the `object` target in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) does not end in two capitalised segments, the
/// `owner` of its [`ClaimMeta`](crate::claim::ClaimMeta) shall be empty.
#[derive(Spec)]
pub struct TheOwnerIsEmptyForAnObjectWithoutAVariant;

/// When the verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) has a signature template and no link follows
/// the verb, compilation shall fail naming the verb.
#[derive(Spec)]
pub struct AShapeVerbWithoutAnObjectLinkFailsToCompile;

/// When the verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) is one the lexicon does not define,
/// compilation shall fail naming the verb and the files the lexicon was read
/// from.
#[derive(Spec)]
pub struct AnUndefinedVerbFailsToCompile;

// ---- Prohibited terms -------------------------------------------------------

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) holds a
/// term of the built-in vague-term list as a whole word or phrase outside
/// backticks in any case, compilation shall fail naming the term.
#[derive(Spec)]
pub struct AProhibitedTermFailsToCompile;

// ---- What the derive rejects: the order --------------------------------------

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) breaks
/// more than one rule of the language, compilation shall fail naming only the
/// first broken rule in the order check 13 states.
#[derive(Spec)]
pub struct TheFirstFailingRuleIsReported;

// ---- What the derive records -------------------------------------------------

/// When a unit struct derives [`Spec`](derive@crate::Spec), the derive's
/// expansion shall carry its [`ClaimMeta`](crate::claim::ClaimMeta) inside the
/// [`SpecMeta`](crate::SpecMeta) registration it places in
/// [`SPECS`](crate::SPECS).
#[derive(Spec)]
pub struct TheRegistrationCarriesTheClaimsParts;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) parses,
/// the `language` of its [`ClaimMeta`](crate::claim::ClaimMeta) shall be
/// [`Language::Controlled`](crate::claim::Language::Controlled).
#[derive(Spec)]
pub struct AParsedClaimIsControlled;

/// When the verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) has a signature template list, the
/// `templates` of its [`ClaimMeta`](crate::claim::ClaimMeta) shall be that
/// list as the lexicon writes it, unmatched against any signature.
#[derive(Spec)]
pub struct AShapeVerbsTemplatesAreRecordedAsWritten;

/// When the verb in the claim of a struct deriving
/// [`Spec`](derive@crate::Spec) has the signature `*`, the `templates` of its
/// [`ClaimMeta`](crate::claim::ClaimMeta) shall be `["*"]`.
#[derive(Spec)]
pub struct ABehaviourVerbRecordsTheStarTemplate;

// ---- The validator's name: check 14 -----------------------------------------

/// When [`validates`](crate::validates) expands on a test fn whose identifier
/// neither is the `snake_case` of any cited path's last segment nor begins
/// with that name followed by `_` while a cited claim carries no
/// `#[lid(free)]`, compilation shall fail naming the expected name for the
/// first cited claim.
#[derive(Spec)]
pub struct AMisnamedValidatorFailsToCompile;

/// When [`validates`](crate::validates) expands on a test fn whose identifier
/// is the `snake_case` of a cited path's last segment followed by `_` and a
/// suffix, compilation shall not fail on the name.
#[derive(Spec)]
pub struct ASuffixedValidatorNameIsAdmitted;

/// When [`validates`](crate::validates) expands on a test fn citing several
/// claims whose identifier is the `snake_case` of one that is not the first,
/// compilation shall not fail on the name.
#[derive(Spec)]
pub struct AValidatorIsNamedForAnyOneCitedClaim;

/// When [`validates`](crate::validates) expands on a test fn whose identifier
/// is the `snake_case` of a cited path's last segment or begins with that name
/// followed by `_`, the expansion shall not carry an assertion over the cited
/// claims' [`FREE`](crate::Spec::FREE).
#[derive(Spec)]
pub struct AnAdmittedNameCarriesNoAssertion;

/// When [`validates`](crate::validates) expands on a test fn whose every cited
/// claim carries `#[lid(free)]`, compilation shall not fail on the name,
/// whatever the fn's identifier.
#[derive(Spec)]
pub struct AValidatorCitingOnlyFreeClaimsIsNotHeldToTheName;

/// When [`validates`](crate::validates) derives the `snake_case` name of a
/// cited path's last segment, the name shall be the identifier's words
/// lower-cased and joined by `_`, a word starting at each capital that a
/// lower-case letter follows, so `GainALibrary` is `gain_a_library`.
#[derive(Spec)]
pub struct SnakeCaseStartsAWordAtACapitalBeforeALowerCaseLetter;

/// When [`validates`](crate::validates) derives the `snake_case` name of a
/// cited path's last segment holding a run of capitals that no lower-case
/// letter follows, that run shall be one word of the name, so `HTTPServer` is
/// `http_server`.
#[derive(Spec)]
pub struct SnakeCaseKeepsARunOfCapitalsAsOneWord;

/// When [`validates`](crate::validates) derives the `snake_case` name of a
/// cited path's last segment holding a digit, that digit shall be part of the
/// word before it, so `Phase7Runs` is `phase7_runs`.
#[derive(Spec)]
pub struct SnakeCaseKeepsADigitWithTheWordBeforeIt;

// ---- The ramp ----------------------------------------------------------------

/// When a struct deriving [`Spec`](derive@crate::Spec) carries `#[lid(free)]`,
/// the derive's expansion shall not fail on its claim, whatever the claim's
/// text.
#[derive(Spec)]
pub struct AFreeClaimCompilesWhateverItsText;

/// When a struct deriving [`Spec`](derive@crate::Spec) carries `#[lid(free)]`,
/// the derive's expansion shall carry a [`ClaimMeta`](crate::claim::ClaimMeta)
/// whose `language` is [`Language::Free`](crate::claim::Language::Free), whose
/// `pattern` is [`Pattern::Ubiquitous`](crate::claim::Pattern::Ubiquitous), and
/// whose every other part is empty.
#[derive(Spec)]
pub struct AFreeClaimRecordsEmptyParts;

/// When a unit struct derives [`Spec`](derive@crate::Spec), the
/// [`FREE`](crate::Spec::FREE) the derive's expansion carries shall be the
/// presence of `#[lid(free)]` on that struct.
#[derive(Spec)]
pub struct TheFreeConstRecordsTheMark;

/// When a struct deriving [`Spec`](derive@crate::Spec) carries a `#[lid(…)]`
/// attribute whose content is anything but the word `free`, the derive's
/// expansion shall fail naming the content.
#[derive(Spec)]
pub struct TheMarkTakesOnlyTheWordFree;

/// When [`free`](crate::claim::free) is called, it shall carry each
/// [`SpecMeta`](crate::SpecMeta) in [`SPECS`](crate::SPECS) whose language is
/// `Free`, in registry order.
#[derive(Spec)]
pub struct FreeEnumeratesTheMarkedClaims;

// ---- The lexicon: base, project, and the walk ---------------------------------

/// When a struct deriving [`Spec`](derive@crate::Spec) is built where no
/// project lexicon is found, the lexicon the derive executes shall be the
/// base compiled into `lid_rs_macros` alone.
#[derive(Spec)]
pub struct TheBaseLexiconIsCompiledIntoTheDerive;

/// When the project lexicon read for a struct deriving
/// [`Spec`](derive@crate::Spec) defines a verb the base does not, that verb
/// shall be admitted with the project's templates.
#[derive(Spec)]
pub struct AProjectVerbAddsToTheBase;

/// When the project lexicon read for a struct deriving
/// [`Spec`](derive@crate::Spec) defines a verb the base also defines, the
/// templates admitted for that verb shall be the project's alone.
#[derive(Spec)]
pub struct AProjectVerbReplacesTheBasesWhole;

/// When the claim of a struct deriving [`Spec`](derive@crate::Spec) holds a
/// term the project lexicon's `extra` supplies as a whole word or phrase
/// outside backticks in any case, compilation shall fail naming the term and
/// that lexicon file.
#[derive(Spec)]
pub struct AProjectExtraFailsToCompileNamingTheLexiconFile;

/// When a struct deriving [`Spec`](derive@crate::Spec) is built with a
/// `CARGO_MANIFEST_DIR`, the project lexicon shall be the first
/// `docs/intent/lexicon.toml` found in that directory or its ancestors.
#[derive(Spec)]
pub struct AMemberFindsItsWorkspacesLexicon;

/// When the walk up from `CARGO_MANIFEST_DIR` for a struct deriving
/// [`Spec`](derive@crate::Spec) has examined a directory whose `Cargo.toml`
/// has a line that is exactly `[workspace]`, it shall stop there.
#[derive(Spec)]
pub struct TheWalkStopsAtAWorkspaceManifest;

/// When the walk up from `CARGO_MANIFEST_DIR` for a struct deriving
/// [`Spec`](derive@crate::Spec) would move to a directory holding no
/// `Cargo.toml`, it shall stop there without a project lexicon.
#[derive(Spec)]
pub struct TheWalkStopsBelowADirectoryWithoutAManifest;

/// When a project lexicon is read for a struct deriving
/// [`Spec`](derive@crate::Spec), the derive's expansion shall carry a
/// `const _: &str = include_str!(…)` of that file's path, so that editing the
/// file rebuilds the claim.
#[derive(Spec)]
pub struct TheProjectLexiconIsIncludedInTheExpansion;

// ---- The lexicon: the format, and what a file outside it does -----------------

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) holds a line that is none of the subset's
/// forms, compilation of every claim in the crate shall fail naming the file
/// and the line.
#[derive(Spec)]
pub struct ALexiconLineOutsideTheSubsetFailsEveryDerive;

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) holds a verb with `def` or `signature`
/// missing or repeated, compilation of every claim in the crate shall fail
/// naming the file, the verb, and the key.
#[derive(Spec)]
pub struct AVerbMissingOrRepeatingAKeyFailsEveryDerive;

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) holds a key other than `def` and `signature`
/// under a verb or other than `extra` under `[prohibited]`, compilation of
/// every claim in the crate shall fail naming the file and the key.
#[derive(Spec)]
pub struct AnUnknownLexiconKeyFailsEveryDerive;

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) names one verb twice, compilation of every
/// claim in the crate shall fail naming the file and the verb.
#[derive(Spec)]
pub struct AVerbNamedTwiceFailsEveryDerive;

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) gives `extra` twice under `[prohibited]`,
/// compilation of every claim in the crate shall fail naming the file and the
/// key.
#[derive(Spec)]
pub struct ExtraGivenTwiceFailsEveryDerive;

/// When the lexicon file read for a struct deriving
/// [`Spec`](derive@crate::Spec) holds a template that does not begin with
/// `->` or whose braces are unbalanced or whose placeholder is neither
/// `{object}` nor `{owner}`, compilation of every claim in the crate shall
/// fail naming the file, the verb, and the template.
#[derive(Spec)]
pub struct AMalformedTemplateFailsEveryDerive;
