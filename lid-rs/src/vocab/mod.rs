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
    T::NOUN
}

#[cfg(test)]
mod tests {
    //! What [`noun_of`](super::noun_of) answers, what [`Noun`](super::Noun)
    //! admits, and the two refusals only a compile failure can state.
    //!
    //! **Every case that can be read at runtime is read through
    //! [`noun_of`](super::noun_of).** This slice is traits, impls and one
    //! associated const per primitive, and a claim whose only implementer is
    //! data is true from the Phase 3 skeleton onward: a case reading `<u8 as
    //! Traceable>::NOUN` directly would be green before any leaf existed, which
    //! is a validator that can never have been red. The function reads the
    //! const, so the same fact is asserted through the one item of the slice
    //! that is work.
    //!
    //! **The type-level facts are asserted by bounds, not by values.** A type is
    //! a [`Noun`](super::Noun) or it is not, and nothing about that is
    //! observable from a value; [`noun_name`] states the bound in its signature
    //! and the compiler discharges it, while the name it carries back is what
    //! gives the case something to be wrong about.
    //!
    //! **The negatives are trybuild fixtures, because a program that compiles
    //! cannot state them.** There is no way to ask whether `u32: Noun` is false
    //! from a passing test — only a refusal says so. The fixtures under
    //! `lid-rs/tests/ui/vocab/fail/` are a hand commit that precedes this phase,
    //! since a harness whose glob matches nothing records zero failures and is a
    //! vacuous green. There is no `pass` directory to glob: a positive fixture
    //! would want `derive(Traceable)`, which is the other half of HLD row 17,
    //! and the positive case is [`a_hand_declared_type_is_a_noun`] instead.
    //!
    //! **Every assertion names an exact string.** The mutation gate substitutes
    //! `""` for the body of anything returning `&'static str`, so a case
    //! asserting only that a name was non-empty hands it a survivor; the
    //! primitives are asserted one by one and the list of them is the
    //! specification.

    use super::{Declared, Noun, Traceable, noun_of, spec};
    use lid_rs::validates;

    /// The name [`Recorded`] records, deliberately unlike that type's own
    /// spelling.
    ///
    /// What [`noun_of`](super::noun_of) carries is the const the impl declared
    /// and not the type's identifier, and the two are only distinguishable on a
    /// type whose const disagrees with its name.
    const RECORDED: &str = "a name of its own";

    /// A type that records under [`RECORDED`], and carries no seal.
    struct Recorded;

    impl Traceable for Recorded {
        const NOUN: &'static str = RECORDED;
    }

    /// A type declared a noun by hand: it records, it carries the seal, and it
    /// is a [`Noun`](super::Noun).
    ///
    /// The three impls a consumer writes where no derive is available, and the
    /// demonstration that the seal makes a noun a deliberate declaration rather
    /// than an unforgeable one. Its sibling — a type carrying the recording
    /// impl and no seal — is `tests/ui/vocab/fail/no_seal.rs`, which does not
    /// compile.
    struct HandDeclared;

    impl Traceable for HandDeclared {
        const NOUN: &'static str = "HandDeclared";
    }

    impl Declared for HandDeclared {}

    impl Noun for HandDeclared {}

    /// The recorded name of a type the compiler accepted as a
    /// [`Noun`](super::Noun).
    ///
    /// The bound is the assertion: a type satisfying only one of the two
    /// supertraits cannot be passed here at all, so a case that calls this has
    /// already stated that its argument is a noun by compiling. Reading the name
    /// back through [`noun_of`](super::noun_of) is what gives that statement a
    /// reachable red — a bound alone is discharged from the skeleton onward.
    fn noun_name<T: Noun>() -> &'static str {
        noun_of::<T>()
    }

    /// What a type records is the const its impl declared, carried back by
    /// [`noun_of`](super::noun_of) with no value of the type in hand.
    ///
    /// [`Recorded`]'s const is not its identifier, so an answer taken from the
    /// type's spelling rather than from its impl fails here while every
    /// primitive still agreed.
    #[test]
    #[validates(spec::NounOfCarriesTheNameItsTypeRecords)]
    fn noun_of_carries_the_name_its_type_records() {
        assert_eq!(
            noun_of::<Recorded>(),
            RECORDED,
            "the name carried is the one the type's impl declared"
        );
    }

    /// Each of rule V's primitives records the spelling of its own type.
    ///
    /// All eighteen, one exact string each: the list is the decision, and a
    /// pairwise assertion is what makes a single wrong impl name itself.
    /// `&str` is the one entry whose spelling is not a single identifier, and
    /// the one the macro over the list cannot produce.
    #[test]
    #[validates(spec::APrimitiveRecordsTheSpellingOfItsOwnType)]
    fn a_primitive_records_the_spelling_of_its_own_type() {
        let recorded = [
            (noun_of::<String>(), "String"),
            (noun_of::<&str>(), "&str"),
            (noun_of::<bool>(), "bool"),
            (noun_of::<char>(), "char"),
            (noun_of::<u8>(), "u8"),
            (noun_of::<u16>(), "u16"),
            (noun_of::<u32>(), "u32"),
            (noun_of::<u64>(), "u64"),
            (noun_of::<u128>(), "u128"),
            (noun_of::<usize>(), "usize"),
            (noun_of::<i8>(), "i8"),
            (noun_of::<i16>(), "i16"),
            (noun_of::<i32>(), "i32"),
            (noun_of::<i64>(), "i64"),
            (noun_of::<i128>(), "i128"),
            (noun_of::<isize>(), "isize"),
            (noun_of::<f32>(), "f32"),
            (noun_of::<f64>(), "f64"),
        ];
        for (name, spelling) in recorded {
            assert_eq!(name, spelling, "`{spelling}` records the spelling of its own type");
        }
    }

    /// A type carrying the recording impl and a hand-written seal is a
    /// [`Noun`](super::Noun), which is what makes the seal a second thing to
    /// write rather than an impossible one.
    ///
    /// The bound on [`noun_name`] is the whole of the type-level statement; the
    /// name it carries back is what a wrong [`noun_of`](super::noun_of) gets
    /// wrong.
    #[test]
    #[validates(spec::AHandDeclaredTypeIsANoun)]
    fn a_hand_declared_type_is_a_noun() {
        assert_eq!(
            noun_name::<HandDeclared>(),
            "HandDeclared",
            "a type that records and was declared is a noun, and records its own spelling"
        );
    }

    /// The two facts no passing program can state: `String` is refused where a
    /// [`Noun`](super::Noun) is required, and the refusal names the type as not
    /// a vocabulary noun.
    ///
    /// `tests/ui/vocab/fail/primitive.rs` requires the bound of a primitive and
    /// pins the shaped `#[diagnostic::on_unimplemented]` message; `no_seal.rs`
    /// writes `impl Noun` for a type carrying no seal and pins the error that
    /// names the seal. Both fixtures write the bound by hand, because the
    /// emission that would write it for them is the other half of HLD row 17
    /// and this harness is about the trait.
    ///
    /// Named for [`NoPrimitiveIsANoun`](super::spec::NoPrimitiveIsANoun) and
    /// not for what it does: check 14 admits a validator named for the
    /// `snake_case` of any one claim it cites, optionally suffixed, and a
    /// descriptive name is the `snake_case` of no claim of this slice.
    #[test]
    #[validates(
        spec::NoPrimitiveIsANoun,
        spec::TheNounRefusalNamesTheTypeAsNotAVocabularyNoun
    )]
    fn no_primitive_is_a_noun() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/vocab/fail/*.rs");
    }
}

pub mod spec;
