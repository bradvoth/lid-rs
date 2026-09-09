//! Check 13 and check 14: the controlled language, executed by the derive.
//!
//! A claim's text is read here — one sentence, one modal, a pattern from its
//! opener, a verb the lexicon defines — and what is read becomes the parts the
//! registration carries. The types those parts land in, this slice's claims,
//! and the fixtures that pin each failure message live in `lid-rs`: a
//! proc-macro crate links into no binary, so it registers nothing and can cite
//! nothing, and its implementation edges are hand-authored at the re-export in
//! `lid-rs`'s crate root.
//!
//! Nothing in `expand` reaches this module yet. The derive runs inside every
//! `cargo check`, so a `todo!()` on its path is a panic in the compiler on
//! every claim in the workspace; the items are therefore built beside the
//! derive and wired to it only once their leaves exist — the LLD's *Sequence:
//! pin, then swap*.

mod lexicon;

use lexicon::Lexicon;
use proc_macro2::TokenStream;
use syn::{Attribute, DeriveInput, Ident, Path};

/// The five patterns a claim's opener names.
///
/// The derive-side mirror of `lid_rs::claim::Pattern`, which this crate cannot
/// name — `lid-rs` depends on this crate, not the other way round. [`expansion`]
/// renders each variant as the path the registration carries.
enum Pattern {
    /// `When …,` — the claim is triggered by an event.
    EventDriven,
    /// `If …, then` — the claim is triggered by an unwanted condition.
    Unwanted,
    /// `While …,` — the claim holds while a state does.
    StateDriven,
    /// `Where …,` — the claim holds where a feature is present.
    Optional,
    /// Any other opener: the claim holds unconditionally, and its trigger link
    /// is sought in the subject.
    Ubiquitous,
}

/// The parts one claim's sentence yields, as [`parse`] extracts them.
///
/// The derive-side mirror of `lid_rs::claim::ClaimMeta`: the same fields, held
/// as owned text before [`expansion`] renders them as the literals a `static`
/// registration is made of.
struct Parts {
    /// The pattern the opener named.
    pattern: Pattern,
    /// The target of the first link in the trigger clause — or, for a
    /// ubiquitous claim, in the subject.
    trigger: String,
    /// The first word after the modal, lower-cased.
    verb: String,
    /// `not` stood between the modal and the verb.
    negated: bool,
    /// The target of the first link after the verb, or empty.
    object: String,
    /// The object's target with its last segment removed, as written, when it
    /// ended in two capitalised segments; otherwise empty.
    owner: String,
    /// The verb's templates as the lexicon writes them, unmatched against any
    /// signature; `["*"]` for a behaviour verb.
    templates: Vec<String>,
}

/// The derive's expansion for one claim struct.
///
/// Reads the `#[lid(free)]` mark, joins and parses the claim against the
/// lexicon the crate under compilation answers to — [`lexicon::read`] of its
/// `CARGO_MANIFEST_DIR` — and yields the expression
/// the registration's `claim` field is initialised with: a block holding the
/// `ClaimMeta` literal and, when a project lexicon was read, the
/// `const _: &str = include_str!(…)` that makes editing that file rebuild the
/// claim. A marked claim yields `Free` parts without reaching [`parse`].
///
/// The error is reported at the struct that carries the claim, naming the rule
/// and the offending text.
pub fn expansion(item: &DeriveInput) -> syn::Result<TokenStream> {
    let _ = item;
    todo!()
}

/// Check 14: a validator is named for a claim it cites.
///
/// The test fn's identifier against the [`snake_case`] of each cited path's
/// last segment: admitted when it is that name, or that name followed by `_`
/// and a suffix, for any one of them. The error is reported at the identifier
/// and names the expected name for the first cited claim.
pub fn validator_name(ident: &Ident, paths: &[Path]) -> syn::Result<()> {
    let _ = (ident, paths);
    todo!()
}

/// The claim to its parts, or the message of the first rule it fails.
///
/// Pure over the joined sentence and the lexicon: the modal is counted, the
/// opener gives the pattern and the clause, the links give the trigger and the
/// response object, and the verb is looked up — the rules in the order check 13
/// states, so a claim with two faults reports the first.
///
/// Everything it asks the lexicon is one of its three queries: `Lexicon::verb`
/// for whether the verb is admitted, whether it is a shape verb and so needs a
/// response-object link, and the templates the parts record; `Lexicon::extra`
/// for the terms the project prohibits beyond the built-in list; and
/// `Lexicon::project` for the file an undefined verb's and a project term's
/// messages name.
fn parse(claim: &str, lexicon: &Lexicon) -> Result<Parts, String> {
    let _ = (claim, lexicon);
    todo!()
}

/// The claim's sentence: the `#[doc]` attributes' lines joined by single
/// spaces and trimmed, checked for exactly one terminator — a period at the
/// end, and no other period, question mark, or exclamation mark outside
/// backticks.
fn sentence(attrs: &[Attribute]) -> Result<String, String> {
    let _ = attrs;
    todo!()
}

/// The opener to its pattern and the span the trigger link is sought in: the
/// opener's clause, up to the first comma outside backticks — up to `, then`
/// for `If` — or, for a ubiquitous claim, the subject up to the modal.
///
/// The error names the opener and the `,` or `, then` its clause lacks.
fn pattern(sentence: &str) -> Result<(Pattern, &str), String> {
    let _ = sentence;
    todo!()
}

/// The target of the first intra-doc link in a span of text: the path in
/// parentheses when the link carries one, otherwise the backticked text.
/// `None` when the span holds no link.
fn link(text: &str) -> Option<&str> {
    let _ = text;
    todo!()
}

/// An identifier's `snake_case` name, by the rule check 14 states: a word
/// starts at each capital a lower-case letter follows, a run of capitals not
/// so followed is one word, and a digit stays with the word before it.
fn snake_case(ident: &str) -> String {
    let _ = ident;
    todo!()
}
