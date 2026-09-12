//! Claims for the vocab slice — a noun is a type the vocabulary declared, and a
//! primitive only records (`lid-rs/src/vocab/lld.md`).
//!
//! The slice is two traits, the seal between them, rule V's primitive
//! `Traceable` impls, `noun_of`, and the crate-root re-exports — the half of HLD
//! row 17 that is ordinary Rust. `derive(Traceable)`, the target reading and the
//! noun assertion are the other half, `noun-assertion` in `lid-rs-macros`, and
//! nothing below is about them. The five claims fall in two groups: what
//! `noun_of` answers, and what `Noun` admits.
//!
//! **Every claim here is cited on an item a test can catch being wrong.**
//! [`Traceable::NOUN`](crate::vocab::Traceable::NOUN) is data on every impl, and
//! a claim whose only implementer is data is true from the Phase 3 skeleton
//! onward, so no test of it can be red before a leaf exists and `red_verdict`
//! refuses it (`cargo-lid-rs/src/phase/mod.rs:912-919`). Both claims about what a
//! type records are therefore cited on [`noun_of`](crate::vocab::noun_of), which
//! reads the const, and the eighteen primitive impls stay uncited carrying their
//! real values — the escape of skeletoning them with `todo!()` was spiked and
//! refused, because an associated const that panics passes `cargo check` and
//! fails the first real build with `E0080`, taking the whole `lid-rs` test binary
//! out from Phase 3 to Phase 6. This is the answer the trace slice reached for
//! `DOCUMENT` and [`document_path`](crate::trace::document_path).
//!
//! **The three claims about the traits are cited by containment.** A citation
//! applies to a fn, a struct or an enum and to nothing else
//! (`lid-rs-macros/src/expand.rs:205-222`), so a trait declaration and a
//! supertrait list — which is what implements [`NoPrimitiveIsANoun`],
//! [`TheNounRefusalNamesTheTypeAsNotAVocabularyNoun`] and
//! [`AHandDeclaredTypeIsANoun`] — cannot carry `#[implements]`. They are cited
//! with `implements_module!` in `lid-rs/src/vocab/mod.rs`, as the registry
//! slice's presence claim is (`lid-rs/src/registry/mod.rs:54`). Their reds are
//! real and are not the module's: the two refusals are read by the trybuild
//! harness over `lid-rs/tests/ui/vocab/fail/`, where a fixture with no `.stderr`
//! pin makes trybuild write a `.wip` and report failure, and the hand-declared
//! type is read through [`noun_of`](crate::vocab::noun_of), which is `todo!()`
//! until Phase 6.
//!
//! **The seal is one claim, and it says deliberate rather than unforgeable.**
//! Both halves were spiked: `impl Noun` for a type with no seal is an `E0277`
//! naming the seal — `the trait bound Forged: Declared is not satisfied` — so
//! nobody arrives at [`Noun`](crate::vocab::Noun) by accident, and a
//! hand-written `impl Declared`
//! followed by `impl Noun` **compiles**, so a determined consumer can declare a
//! noun without a derive. [`AHandDeclaredTypeIsANoun`] states the second, which
//! is the half a passing program can observe and the half an "unforgeable" claim
//! would get wrong; what makes the act deliberate rather than accidental is that
//! the seal is `#[doc(hidden)]`, outside the documented surface, and that
//! omitting it is the compile failure the second fixture pins. No claim links
//! `Declared`: it is hidden, so the seal is named in backticks and the claim's
//! trigger links [`Traceable`](crate::vocab::Traceable) instead.
//!
//! **The refusal and its wording are two claims.** [`NoPrimitiveIsANoun`] is the
//! fact the whole design turns on — rule V's list gets the recording trait and
//! never the seal — and it is falsified by one stray `impl Declared`.
//! [`TheNounRefusalNamesTheTypeAsNotAVocabularyNoun`] is the
//! `#[diagnostic::on_unimplemented]` message, falsified by dropping the attribute
//! while every other claim stays true, which would leave the common case — a
//! claim naming an ordinary type — reported as a bare unsatisfied trait bound.
//! The claim pins the shaped phrase; the fixture's `.stderr` pins the whole
//! message.
//!
//! **What has no claim here, and where it lives.** The crate-root re-exports and
//! the seal's address in `__private` carry none: a `pub use` is observed only by
//! a path compiling, which is green from Phase 3 onward, and a wrong one is a
//! compile error in its first consumer — which is the other half's derive.
//! Nothing here claims a derive, a target form, an assertion or the burn-down of
//! held claims: they are `noun-assertion`'s, and a claim about them would be an
//! orphan in a crate no phase of this branch may write. No claim states a policy
//! attribute or a span's fields, which describe what a span records and are
//! slice 20's (Deferred 1), and none asks that an LLD link its nouns, which needs
//! `lld-check` to read the document (Deferred 3). No claim fixes whether a
//! *derived* noun's `NOUN` is an identifier or a full path, which is the
//! document's second open question: [`APrimitiveRecordsTheSpellingOfItsOwnType`]
//! settles it for the primitives alone, which have no module path to choose.
//!
//! **The controlled language was held by hand.** `lid-rs/src/lib.rs` declares no
//! `pub mod vocab;` yet — the document lands that line between this phase and
//! Phase 3 — so nothing compiles this file and check 13 does not run on it. Every
//! verb below is the base lexicon's (`carry`, `fail`, `name`, `be`), which is
//! what a published crate answers to whatever a project's file says; every claim
//! is one sentence with one `shall` and one terminator; and every trigger names a
//! Rust item by intra-doc link, placed first in its clause so the link the derive
//! records is the one intended. The items Phase 3 has yet to create are linked by
//! the paths the document's Shape table gives. None of these claims is marked
//! free.

use lid_rs::Spec;

// ---- `noun_of`: the recorded name, and what a primitive records --------------

/// When [`noun_of`](crate::vocab::noun_of) is given a type carrying
/// [`Traceable`](crate::vocab::Traceable), it shall carry that type's
/// [`NOUN`](crate::vocab::Traceable::NOUN).
#[derive(Spec)]
pub struct NounOfCarriesTheNameItsTypeRecords;

/// When [`noun_of`](crate::vocab::noun_of) is given `String` or `&str` or `bool`
/// or `char` or one of Rust's integer or floating-point types, it shall carry
/// the spelling of that type.
#[derive(Spec)]
pub struct APrimitiveRecordsTheSpellingOfItsOwnType;

// ---- `Noun`: what it refuses, how it says so, and what it admits -------------

/// When [`Noun`](crate::vocab::Noun) is required of `String` or `&str` or `bool`
/// or `char` or one of Rust's integer or floating-point types, compilation shall
/// fail.
#[derive(Spec)]
pub struct NoPrimitiveIsANoun;

/// When [`Noun`](crate::vocab::Noun) is required of a type that carries no
/// implementation of it, the compiler error shall name that type as not a
/// vocabulary noun.
#[derive(Spec)]
pub struct TheNounRefusalNamesTheTypeAsNotAVocabularyNoun;

/// When a type carries an implementation of
/// [`Traceable`](crate::vocab::Traceable) and a hand-written implementation of
/// the seal `Declared`, it shall be a [`Noun`](crate::vocab::Noun).
#[derive(Spec)]
pub struct AHandDeclaredTypeIsANoun;
