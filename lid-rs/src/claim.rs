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
    crate::SPECS.iter().filter(|_meta| todo!())
}
