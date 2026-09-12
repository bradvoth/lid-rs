#![doc = include_str!("lld.md")]
use lid_rs::implements;

/// The recording trait: what a type is recorded under when a span carries it.
///
/// It is not the predicate a claim's noun is checked against. Rule V's
/// primitives carry it (`README.md:513-516`), so a bound on this trait accepts
/// `String`, `u32` and `bool` — text with no policy — which is the whole reason
/// identity is a second trait, [`Noun`], and this one only says how a recorded
/// value is named.
///
/// Its recording surface is one associated const today. Slice 20 builds the
/// spans that read it, and adding a required method to a published trait is a
/// breaking change that only `cargo package` reports; shipping the const alone
/// is the constrained-first choice and names that cost where slice 20 will meet
/// it.
pub trait Traceable {
    /// The name this type is recorded under: the type's own spelling.
    ///
    /// Data on every impl, and uncited. A claim whose only implementer is data
    /// is true from the skeleton onward, so no test of it could be red before a
    /// leaf exists (`cargo-lid-rs/src/phase/mod.rs:912-919`) — the claims about
    /// what a type records are cited on [`noun_of`], which reads this.
    const NOUN: &'static str;
}

/// The seal: implementing it is the deliberate act that makes a type eligible
/// to be a [`Noun`].
///
/// Hidden from the documented surface and re-exported at
/// `lid_rs::__private::Declared`, because a derive expanding inside a
/// consumer's crate must be able to name it while nothing a reader browses
/// points at it. Public rather than `pub(crate)`, because a crate-local seal
/// would make the derive work only inside `lid-rs`.
///
/// What it guarantees is deliberateness and not impossibility: `impl Noun` for
/// a type carrying no seal is refused by an error naming this trait, and a
/// hand-written `impl Declared` followed by `impl Noun` compiles. The second is
/// a decision a consumer can take, not an accident one can fall into.
#[doc(hidden)]
pub trait Declared {}

/// What a claim may name: a type that both records ([`Traceable`]) and carries
/// the hidden seal `Declared`.
///
/// Empty, and implemented item by item — by hand, or by `derive(Traceable)`
/// once the other half of this design lands. There is no blanket
/// implementation, so the supertrait list is what refuses a primitive and what
/// names the missing seal when only one of the two was written.
///
/// The shaped message is this slice's to own: a claim naming an ordinary type
/// is the common failure, and a bare unsatisfied trait bound would leave its
/// author reading about a trait they have never heard of.
///
/// The seal is named in backticks above and not linked, as this slice's claims
/// name it: it is `#[doc(hidden)]`, so a link from a rendered page would point
/// at a page that is not rendered.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a vocabulary noun: a claim may name only a type carrying `derive(Traceable)`"
)]
pub trait Noun: Traceable + Declared {}

// The three claims cited here are kept by the trait declarations above and by
// the supertrait list, and `#[implements]` applies to a fn, a struct or an enum
// and to nothing else (`lid-rs-macros/src/expand.rs:205-222`). So the module
// holding them is cited by containment, as the registry slice's enumeration
// claim is (`lid-rs/src/registry/mod.rs:54`).
lid_rs::implements_module!(
    spec::NoPrimitiveIsANoun,
    spec::TheNounRefusalNamesTheTypeAsNotAVocabularyNoun,
    spec::AHandDeclaredTypeIsANoun,
);

/// One [`Traceable`] impl per primitive named, each recording that type's own
/// identifier and none of them carrying the seal.
///
/// A macro over the list rather than seventeen hand-written impls: rule V's
/// list is one decision (`README.md:513-516`), and seventeen copies of it are
/// seventeen chances to disagree. The recorded name is the identifier, which is
/// the only spelling `u8` has — the convention for a *derived* noun's name is
/// the other half's to settle.
macro_rules! records_its_own_spelling {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Traceable for $ty {
                const NOUN: &'static str = stringify!($ty);
            }
        )+
    };
}

records_its_own_spelling!(
    String, bool, char, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64,
);

/// The one primitive of rule V's list whose spelling is not a single
/// identifier, recorded as the list writes it (`README.md:513-516`).
///
/// Outside the macro because `stringify!` over a reference type reproduces a
/// token sequence rather than the source spelling, and this const is what a
/// test of a primitive's recorded name compares against.
impl Traceable for &str {
    const NOUN: &'static str = "&str";
}

/// The name a type is recorded under, read from the type with no value of it in
/// hand.
///
/// The surface a noun's name is reached through: slice 20's spans state the
/// bound `T: Traceable` here once instead of at every field, and this slice's
/// claims about what a type records are cited here rather than on the impls,
/// because [`Traceable::NOUN`] is data and a claim implemented by data is true
/// from the skeleton onward. That is the answer the trace slice reached for
/// `DOCUMENT` and [`document_path`](crate::trace::document_path).
///
/// `?Sized`, because the name is read from the type and never from a value:
/// nothing here needs a size.
#[implements(
    spec::NounOfCarriesTheNameItsTypeRecords,
    spec::APrimitiveRecordsTheSpellingOfItsOwnType,
)]
pub fn noun_of<T: Traceable + ?Sized>() -> &'static str {
    todo!()
}

pub mod spec;
