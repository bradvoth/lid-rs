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
//!
//! [`parse`] is where the rules stand in the order check 13 states, one rule
//! to a line, so that a claim with two faults reports the first. Under it sit
//! the readings each rule needs — the modal, the opener's clause, the verb,
//! the links — and under those three primitives every rule shares:
//! [`in_code`], which is why a backticked span is never punctuation and never
//! a term, [`find_outside`] for the punctuation a pattern turns on, and
//! [`word_matches`] for a word or phrase matched whole and in any case.

mod lexicon;

use lexicon::{Lexicon, Verb};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Attribute, DeriveInput, Ident, LitStr, Path};

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

/// The derive helper attribute the ramp's mark is written as: `#[lid(…)]`,
/// declared by `derive(Spec)` and inert until this module reads it.
const MARK: &str = "lid";

/// The one word the mark takes. Any other content is the failure, so the
/// exemption cannot grow a second meaning.
const FREE: &str = "free";

/// The built-in prohibited terms: the INCOSE vague-term list, matched as whole
/// words or phrases outside backticks and without regard to case.
///
/// A project's `[prohibited] extra` adds to this list and nothing takes from
/// it, so a term here is prohibited wherever a claim is written.
const VAGUE: [&str; 21] = [
    "appropriate",
    "adequate",
    "as needed",
    "as required",
    "as appropriate",
    "user-friendly",
    "reasonable",
    "quickly",
    "timely",
    "easy",
    "efficient",
    "robust",
    "approximately",
    "etc.",
    "and/or",
    "sufficient",
    "normal",
    "minimal",
    "maximise",
    "minimise",
    "support",
];

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
    if free_mark(&item.attrs).map_err(|m| error(item, &m))? {
        return Ok(free_claim());
    }
    let claim = sentence(&item.attrs).map_err(|m| error(item, &m))?;
    let lexicon = lexicon::read(&manifest_dir()).map_err(|m| error(item, &m))?;
    let parts = parse(&claim, &lexicon).map_err(|m| error(item, &m))?;
    let include = lexicon_include(lexicon.project());
    let meta = claim_meta(&parts);
    Ok(quote!({ #include #meta }))
}

/// Whether the struct carries `#[lid(free)]`: the ramp's mark, read before any
/// rule of the language is applied, since a marked claim is exempt from them
/// all.
///
/// A struct carrying no [`MARK`] attribute is unmarked and its claim is parsed.
/// The failure is the message of a broken rule like any other, turned into the
/// compile error at the struct by [`expansion`], because the mark is part of
/// what the derive reads of a claim.
fn free_mark(attrs: &[Attribute]) -> Result<bool, String> {
    match attrs.iter().find(|attr| attr.path().is_ident(MARK)) {
        Some(attr) => marked(attr),
        None => Ok(false),
    }
}

/// The `#[lid(…)]` attribute the struct carries, read: `true` when its content
/// is exactly [`FREE`], and otherwise the failure naming the content.
///
/// The mark takes that one word so the exemption cannot grow a second meaning
/// while still reading as one.
fn marked(attr: &Attribute) -> Result<bool, String> {
    let content = mark_content(attr);
    if content == FREE {
        return Ok(true);
    }
    Err(unknown_mark(&content))
}

/// The content of a `#[lid(…)]` attribute as written: the tokens between its
/// parentheses, or the empty string when the attribute carries none — `#[lid]`
/// names nothing, and nothing is not `free`.
fn mark_content(attr: &Attribute) -> String {
    attr.meta.require_list().map(|list| list.tokens.to_string()).unwrap_or_default()
}

/// The mark's failure, in the shape every failure of this module takes: the
/// rule, and the content that broke it.
///
/// The one word is the whole of the mark's meaning, so the message names what
/// was written instead of it — the author's own text, not a list of what would
/// have been admitted.
fn unknown_mark(content: &str) -> String {
    broken("the mark takes only the word `free`", content)
}

/// The expression a marked claim's registration carries: a `ClaimMeta` whose
/// language is `Free`, whose pattern is `Ubiquitous`, and whose every other
/// part is empty.
///
/// A marked claim reads no lexicon, so it carries no `include_str!` either:
/// nothing about it depends on a file that could change.
fn free_claim() -> TokenStream {
    quote!(::lid_rs::claim::ClaimMeta {
        language: ::lid_rs::claim::Language::Free,
        pattern: ::lid_rs::claim::Pattern::Ubiquitous,
        trigger: "",
        verb: "",
        negated: false,
        object: "",
        owner: "",
        templates: &[],
    })
}

/// The directory of the crate under compilation, where the walk for a project
/// lexicon starts.
///
/// Cargo sets `CARGO_MANIFEST_DIR` for every macro expansion; without it the
/// walk starts nowhere, finds no file, and the base is the whole lexicon —
/// which is the same answer a crate built from the registry cache gets.
fn manifest_dir() -> std::path::PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default().into()
}

/// The `const _: &str = include_str!(…)` of the project lexicon that was read,
/// so that editing the file rebuilds every claim that answered to it; no
/// tokens at all when the walk found no file.
fn lexicon_include(file: Option<&std::path::Path>) -> TokenStream {
    file.map(|path| {
        let path = path.to_string_lossy().into_owned();
        quote!(const _: &str = include_str!(#path);)
    })
    .unwrap_or_default()
}

/// One claim's parts as the `ClaimMeta` literal a `static` registration is
/// made of: every field a literal, an enum path, or a slice of literals.
fn claim_meta(parts: &Parts) -> TokenStream {
    let pattern = pattern_path(&parts.pattern);
    let (trigger, verb) = (&parts.trigger, &parts.verb);
    let (negated, object, owner) = (parts.negated, &parts.object, &parts.owner);
    let templates = &parts.templates;
    quote!(::lid_rs::claim::ClaimMeta {
        language: ::lid_rs::claim::Language::Controlled,
        pattern: #pattern,
        trigger: #trigger,
        verb: #verb,
        negated: #negated,
        object: #object,
        owner: #owner,
        templates: &[#(#templates),*],
    })
}

/// A pattern as the path the registration names it by, in the companion
/// module's enum.
fn pattern_path(pattern: &Pattern) -> TokenStream {
    match pattern {
        Pattern::EventDriven => quote!(::lid_rs::claim::Pattern::EventDriven),
        Pattern::Unwanted => quote!(::lid_rs::claim::Pattern::Unwanted),
        Pattern::StateDriven => quote!(::lid_rs::claim::Pattern::StateDriven),
        Pattern::Optional => quote!(::lid_rs::claim::Pattern::Optional),
        Pattern::Ubiquitous => quote!(::lid_rs::claim::Pattern::Ubiquitous),
    }
}

/// The compile error for a broken rule, reported at the struct that carries
/// the claim — the item the author is looking at when they read it.
fn error(item: &DeriveInput, message: &str) -> syn::Error {
    syn::Error::new_spanned(&item.ident, message)
}

/// Check 14: a validator is named for a claim it cites.
///
/// The test fn's identifier against the [`snake_case`] of each cited path's
/// last segment: admitted when it is that name, or that name followed by `_`
/// and a suffix, for any one of them. An admitted name yields no tokens at all,
/// so the common case costs nothing; any other name yields the guard
/// [`misnamed`] builds, which is the whole of what the citation emits for
/// check 14.
///
/// The name is compared here, where the identifier is, and the exemption is
/// left to the compiler, where the claims are: an attribute macro sees the
/// cited paths and the identifier and nothing else, and whether those claims
/// carry `#[lid(free)]` is an attribute on another item.
pub fn validator_name(ident: &Ident, paths: &[Path]) -> TokenStream {
    if paths.iter().any(|path| admits(ident, path)) {
        TokenStream::new()
    } else {
        misnamed(ident, paths)
    }
}

/// Whether an identifier is the name this cited path admits: the
/// [`snake_case`] of the path's last segment, or that name followed by `_` and
/// a suffix — which is how several validators of one claim in one module stay
/// distinct while each says which claim it observes.
fn admits(ident: &Ident, path: &Path) -> bool {
    let name = expected_name(path);
    let written = ident.to_string();
    written == name || written.strip_prefix(&name).is_some_and(|suffix| suffix.starts_with('_'))
}

/// The name a cited path admits: the [`snake_case`] of its last segment.
///
/// The one reading of a cited claim's name — [`admits`] compares the
/// identifier with it, and [`misnamed`] bakes it into the guard's message — so
/// that the name a validator is held to and the name it is told to take are
/// one rule and not two.
fn expected_name(path: &Path) -> String {
    path.segments.last().map(|segment| snake_case(&segment.ident.to_string())).unwrap_or_default()
}

/// Check 14's guard, emitted when no cited path admits the identifier: a const
/// assertion that every cited claim is marked free of the language, whose
/// message names the expected name for the first cited claim.
///
/// The mark exempts a claim from the language, and a validator's name is a rule
/// of the language about that claim, so a citation of claims that are all free
/// compiles and one holding a claim that is not fails `cargo check` with this
/// message. The expected name is baked into the literal at expansion, because
/// the compiler that resolves the condition cannot derive a name.
///
/// Any one cited claim's name is admitted, so the message names the first
/// rather than all of them — the author renames to one, and citation order
/// decides only which is suggested. A citation with no path does not reach
/// here: `expand::citation` rejects it first.
fn misnamed(ident: &Ident, paths: &[Path]) -> TokenStream {
    let expected = paths.first().map(expected_name).unwrap_or_default();
    let found = format!("expected `{expected}`, found `{ident}`");
    let message = broken("a validator is named for a claim it cites", &found);
    quote!(const _: () = assert!(#(<#paths as ::lid_rs::Spec>::FREE)&&*, #message);)
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
    let (head, response) = modal(claim)?;
    let (opened, clause) = pattern(head)?;
    let (word, negated, after) = verb_word(response);
    let admitted = lexicon.verb(&word).ok_or_else(|| undefined_verb(&word, lexicon))?;
    builtin_term(claim)?;
    project_term(claim, lexicon)?;
    let trigger = link(clause).ok_or_else(|| broken("a claim names its trigger by a link", clause))?;
    let target = object(after, &word, admitted)?;
    Ok(Parts {
        pattern: opened,
        trigger: trigger.to_string(),
        verb: word,
        negated,
        owner: owner(&target),
        object: target,
        templates: admitted.templates.iter().map(|t| t.as_written().to_string()).collect(),
    })
}

/// The claim's sentence: the `#[doc]` attributes' lines joined by single
/// spaces and trimmed, checked for exactly one terminator — a period at the
/// end, and no other period, question mark, or exclamation mark outside
/// backticks.
fn sentence(attrs: &[Attribute]) -> Result<String, String> {
    let claim = joined(attrs);
    terminator(&claim)?;
    Ok(claim)
}

/// The `#[doc]` attributes' lines joined by single spaces and trimmed;
/// anything that is not a doc attribute contributes nothing.
fn joined(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("doc"))
        .filter_map(|attr| attr.meta.require_name_value().ok())
        .filter_map(|line| syn::parse2::<LitStr>(line.value.to_token_stream()).ok())
        .map(|line| line.value().trim().to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

/// The one-terminator rule: the sentence ends with a period and holds no other
/// period, question mark, or exclamation mark outside backticks.
///
/// The message names the missing period, or the second terminator and what
/// follows it — a claim that says two things is two claims.
fn terminator(claim: &str) -> Result<(), String> {
    if !claim.ends_with('.') {
        return Err(broken("a claim is one sentence and ends with a period", claim));
    }
    match extra_terminator(claim) {
        Some(at) => Err(broken("a claim is one sentence and holds one terminator", &claim[at..])),
        None => Ok(()),
    }
}

/// The byte index of a period, question mark, or exclamation mark outside
/// backticks before the sentence's final character, if the sentence holds one.
fn extra_terminator(claim: &str) -> Option<usize> {
    claim
        .char_indices()
        .find(|(at, c)| ".?!".contains(*c) && at + 1 < claim.len() && !in_code(claim, *at))
        .map(|(at, _)| at)
}

/// The opener to its pattern and the span the trigger link is sought in, read
/// from the head — the sentence up to the modal, as [`modal`] divided it.
///
/// The clause is the head up to the first comma outside backticks, up to
/// `, then` for `If`; a ubiquitous claim's clause is the whole head, which is
/// its subject, so the subject needs no second reading of the modal.
///
/// The head, not the sentence, is what the closer is sought in, and that is
/// what makes a `When`, `While`, or `Where` opener whose only comma follows the
/// modal malformed: a clause allowed to span the modal would take its trigger
/// link from the response, and a condition that links nothing would pass while
/// recording a trigger the claim never stated.
///
/// The error names the opener and the `,` or `, then` its clause lacks.
fn pattern(head: &str) -> Result<(Pattern, &str), String> {
    let Some((opened, closer)) = opener(head) else {
        return Ok((Pattern::Ubiquitous, head));
    };
    let end = find_outside(head, closer).ok_or_else(|| unclosed(head, closer))?;
    Ok((opened, &head[..end]))
}

/// The pattern the head's first word — which is the sentence's — names, and the
/// text its clause must close with: `,` for `When`, `While`, and `Where`, and
/// `, then` for `If`. Compared without regard to case.
///
/// `None` for any other first word, which is the ubiquitous pattern: the last
/// row of the table admits anything, so every sentence has a pattern and what
/// a sentence can fail is the clause its opener promised.
fn opener(head: &str) -> Option<(Pattern, &'static str)> {
    match first_word(head).to_lowercase().as_str() {
        "when" => Some((Pattern::EventDriven, ",")),
        "if" => Some((Pattern::Unwanted, ", then")),
        "while" => Some((Pattern::StateDriven, ",")),
        "where" => Some((Pattern::Optional, ",")),
        _ => None,
    }
}

/// The sentence's — or the head's — first word, as written: the text up to the
/// first whitespace, and the empty string when there is no text at all.
///
/// The opener is read from it, and the message of an unclosed clause quotes it,
/// so the word the author wrote is the word the reader is shown.
fn first_word(text: &str) -> &str {
    text.split_whitespace().next().unwrap_or_default()
}

/// The unclosed-clause failure: the opener the head began with, and the `,` or
/// `, then` the pattern it named expects before the modal, which is where the
/// head ends.
fn unclosed(head: &str, closer: &str) -> String {
    let opened = first_word(head);
    let article = if opened.eq_ignore_ascii_case("if") { "an" } else { "a" };
    broken(&format!("{article} `{opened}` claim closes its clause with `{closer}` before the modal"), head)
}

/// The claim's single `shall`, as the two spans it divides the sentence into:
/// the head before it, and the response after it.
///
/// The head is where the opener's clause and a ubiquitous subject are read, and
/// the response is where the verb and the response object are read. The modal
/// is found once and both spans come from that one reading, so no later rule
/// has to find it again — and none can disagree about where it stood.
///
/// The modal is matched as a whole word outside backticks. A sentence with two
/// is two claims and a sentence with none is prose: either is the failure, and
/// the message names the count found.
fn modal(claim: &str) -> Result<(&str, &str), String> {
    let found = word_matches(claim, "shall");
    let [at] = found[..] else {
        return Err(broken(&format!("a claim holds `shall` exactly once, not {}", found.len()), claim));
    };
    Ok((claim[..at].trim(), claim[at + "shall".len()..].trim()))
}

/// The verb, its negation, and the response: the first word after the modal
/// other than a leading `not`, lower-cased; whether that `not` stood there;
/// and the text following the verb, which is where the response object link is
/// sought.
///
/// So *shall not count* has verb `count`, negated. Nothing after the modal
/// yields the empty verb, which no lexicon defines, so the undefined-verb rule
/// reports it.
fn verb_word(after_modal: &str) -> (String, bool, &str) {
    let negated = first_word(after_modal).eq_ignore_ascii_case("not");
    let response = if negated { after_modal["not".len()..].trim_start() } else { after_modal };
    let word: String = response.chars().take_while(|c| c.is_alphabetic()).collect();
    (word.to_lowercase(), negated, &response[word.len()..])
}

/// The undefined-verb failure: the verb, and the files the lexicon was read
/// from — the base always, and the project's file when the walk found one, so
/// the author knows where a verb would have to be defined.
fn undefined_verb(verb: &str, lexicon: &Lexicon) -> String {
    let read = lexicon
        .project()
        .map(|file| format!("{} and {}", lexicon::BASE_FILE, file.display()))
        .unwrap_or_else(|| lexicon::BASE_FILE.to_string());
    broken(&format!("no lexicon defines the verb `{verb}`"), &read)
}

/// The built-in prohibited terms: [`VAGUE`] matched as whole words or phrases
/// outside backticks and in any case. The message names the term.
fn builtin_term(claim: &str) -> Result<(), String> {
    match VAGUE.iter().find(|term| !word_matches(claim, term).is_empty()) {
        Some(term) => Err(broken("a claim uses no term of the vague-term list", term)),
        None => Ok(()),
    }
}

/// The terms the project's lexicon prohibits beyond the built-in list, matched
/// the same way. The message names the term and the lexicon file that supplied
/// it, because that file is where the prohibition can be argued with.
fn project_term(claim: &str, lexicon: &Lexicon) -> Result<(), String> {
    let file = lexicon.project().unwrap_or(std::path::Path::new("")).display();
    match lexicon.extra().iter().find(|term| !word_matches(claim, term).is_empty()) {
        Some(term) => Err(broken("a claim uses no term the lexicon prohibits", &format!("{term}, in {file}"))),
        None => Ok(()),
    }
}

/// The response object's target: the first link after the verb.
///
/// A shape verb without one is the failure, naming the verb — its template is
/// a promise about a return type the claim then never names. A behaviour verb
/// without one records the empty object, because its meaning is not a shape.
fn object(response: &str, verb: &str, admitted: &Verb) -> Result<String, String> {
    match link(response) {
        Some(target) => Ok(target.to_string()),
        None if admitted.is_shape() => {
            Err(broken("a claim whose verb names a return shape names its response object by a link", verb))
        }
        None => Ok(String::new()),
    }
}

/// The object's owner: the target with its last segment removed, as written,
/// when the target ends in two capitalised segments — `AuthError` from
/// `AuthError::Backend`. Empty for any other target, which names no variant.
fn owner(object: &str) -> String {
    object
        .rsplit_once("::")
        .filter(|(head, last)| capitalised(last) && head.rsplit("::").next().is_some_and(capitalised))
        .map(|(head, _)| head.to_string())
        .unwrap_or_default()
}

/// Whether a path segment is capitalised: its first character is upper-case.
/// Two of these at the end of a target is what makes the target a variant of
/// something rather than a type.
fn capitalised(segment: &str) -> bool {
    segment.starts_with(char::is_uppercase)
}

/// The target of the first intra-doc link in a span of text: the path in
/// parentheses when the link carries one, otherwise the backticked text.
/// `None` when the span holds no link.
fn link(text: &str) -> Option<&str> {
    let (bracketed, rest) = brackets(text)?;
    Some(paren(rest).unwrap_or_else(|| unbackticked(bracketed)))
}

/// The first `[…]` span's contents, and the text immediately after its `]`
/// where a path may follow. `None` when the text holds no bracketed span.
fn brackets(text: &str) -> Option<(&str, &str)> {
    text.split_once('[')?.1.split_once(']')
}

/// The contents of a `(…)` that opens at the very start of the text: the
/// link's path. `None` when no parenthesis follows, which is a link that
/// targets its own text.
fn paren(text: &str) -> Option<&str> {
    text.strip_prefix('(')?.split_once(')').map(|(path, _)| path)
}

/// A link's bracketed text with its surrounding backticks removed, which is
/// the target of a link that carries no path.
fn unbackticked(text: &str) -> &str {
    text.trim_matches('`')
}

/// An identifier's `snake_case` name, by the rule check 14 states: a word
/// starts at each capital a lower-case letter follows, a run of capitals not
/// so followed is one word, and a digit stays with the word before it.
fn snake_case(ident: &str) -> String {
    ident.char_indices().map(|(at, c)| letter(ident, at, c)).collect()
}

/// One character of a `snake_case` name: the character lower-cased, preceded
/// by `_` when a word starts at it.
fn letter(ident: &str, at: usize, c: char) -> String {
    let boundary = if starts_word(ident, at) { "_" } else { "" };
    format!("{boundary}{}", c.to_lowercase())
}

/// Whether a word of the `snake_case` name starts at this character: a capital
/// that is not the first character and either follows a lower-case letter or a
/// digit, or is followed by a lower-case letter.
///
/// So `GainALibrary` breaks before `A` and before `Library`, `HTTPServer` only
/// before `S`, and `Phase7Runs` before `R` — the three cases the claims name.
fn starts_word(ident: &str, at: usize) -> bool {
    let mut ahead = ident[at..].chars();
    let here = ahead.next();
    let behind = ident[..at].chars().next_back();
    here.is_some_and(char::is_uppercase)
        && (behind.is_some_and(|c| c.is_lowercase() || c.is_numeric())
            || behind.is_some() && ahead.next().is_some_and(char::is_lowercase))
}

/// A broken rule's message: the rule, and the text that broke it.
///
/// Every check-13 failure takes this shape, so a claim's author reads what was
/// required and what they wrote in one line, and the fixtures pin one form
/// rather than a dozen.
fn broken(rule: &str, offending: &str) -> String {
    format!("lid-rs: {rule}: {offending}")
}

/// The byte indices at which a word or phrase occurs in the text outside
/// backticks, compared without regard to case and matched whole.
///
/// The modal is counted through this, and so is every prohibited term: both
/// rules ask the same question of a claim, and asking it once is what keeps
/// *shall* inside `` `shall` `` from being a modal and *supported* from being
/// *support*.
fn word_matches(text: &str, phrase: &str) -> Vec<usize> {
    let lowered = text.to_ascii_lowercase();
    let sought = phrase.to_ascii_lowercase();
    lowered
        .match_indices(&sought)
        .filter(|(at, found)| bounded(&lowered, *at, found.len()) && !in_code(&lowered, *at))
        .map(|(at, _)| at)
        .collect()
}

/// Whether the span of `len` bytes at `at` is a whole word or phrase: neither
/// the character before it nor the character after it is alphanumeric.
fn bounded(text: &str, at: usize, len: usize) -> bool {
    !text[..at].ends_with(char::is_alphanumeric) && !text[at + len..].starts_with(char::is_alphanumeric)
}

/// The byte index of the first occurrence of `needle` outside backticks, if
/// there is one. The punctuation the patterns turn on is found through this.
fn find_outside(text: &str, needle: &str) -> Option<usize> {
    text.match_indices(needle).map(|(at, _)| at).find(|at| !in_code(text, *at))
}

/// Whether the byte index lies inside a backticked span: an odd number of
/// backticks precedes it.
///
/// This is the whole of "outside backticks", and every rule of the language
/// depends on it: a code span holds no modal, no terminator, no prohibited
/// term, and no clause-closing comma.
fn in_code(text: &str, at: usize) -> bool {
    text[..at].matches('`').count() % 2 == 1
}
