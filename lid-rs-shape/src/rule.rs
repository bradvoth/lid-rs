//! F1 through F6, one item per rule, over one function's body.
//!
//! Each item answers one rule and nothing else: whether the body fails it. A
//! body fails a rule or it does not, and the verdict — flow or leaf, and which
//! rule a leaf failed first — is composed from these answers elsewhere, so a
//! report can name the rule rather than the verdict.
//!
//! **The six rules are one definition, and two of them defer to a third.**
//! Read alone, F1 admits no `match` statement and F4 no `if`, whose arms are
//! blocks by Rust's grammar, and F2's allowance of one decision structure could
//! then never be taken. So the statement F2 admits is admitted by F1 and its
//! arms by F4, and the one item that says which statement that is —
//! [`is_decision_structure`] — is read by all three. That is the reading under
//! which all six rules can hold at once.

use lid_rs::implements;

use crate::spec;

/// Whether one statement is the decision structure F2 admits: one `match`, or
/// one two-way `if`.
///
/// The one place the three rules that speak of a decision structure agree on
/// what one is. F2 counts these, F1 admits the one F2 allows as a statement it
/// does not reject, and F4 admits that one's arms as blocks it does not reject
/// — so a wrong answer here moves all three rules at once, which is why it is
/// one item and not three readings.
#[implements(
    spec::ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1,
    spec::ABodyWithMoreThanOneDecisionStructureIsALeafUnderF2,
    spec::AClosureOrANonArmBlockMakesTheBodyALeafUnderF4,
)]
pub(crate) fn is_decision_structure(stmt: &syn::Stmt) -> bool {
    todo!("whether the {:?} statement is the decision structure F2 admits", std::mem::discriminant(stmt))
}

/// Whether one statement is one of the call forms F1 admits: `let pat =
/// call(...)?;`, a bare `call(...)?;`, or the tail call.
///
/// The decision structure F2 admits is not tested here — [`fails_f1`] admits it
/// beside these — so this item answers for the call forms alone.
#[implements(spec::ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1)]
fn is_admitted_statement(stmt: &syn::Stmt) -> bool {
    todo!("whether the {:?} statement is one of F1's call forms", std::mem::discriminant(stmt))
}

/// **F1** — whether the body holds a statement that is neither one of the call
/// forms F1 admits nor the decision structure F2 admits.
#[implements(spec::ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1)]
pub(crate) fn fails_f1(body: &syn::Block) -> bool {
    body.stmts.iter().any(|stmt| !is_admitted_statement(stmt) && !is_decision_structure(stmt))
}

/// **F2, first half** — whether the body holds a second decision structure. One
/// is admitted; two is a body making two flow decisions in one function.
#[implements(spec::ABodyWithMoreThanOneDecisionStructureIsALeafUnderF2)]
pub(crate) fn fails_f2_a_second_decision_structure(body: &syn::Block) -> bool {
    body.stmts.iter().filter(|stmt| is_decision_structure(stmt)).count() > 1
}

/// **F2, second half** — whether an arm of the decision structure the body
/// holds is anything but a single call.
#[implements(spec::ADecisionArmThatIsNoSingleCallMakesTheBodyALeafUnderF2)]
pub(crate) fn fails_f2_an_arm_that_is_no_single_call(body: &syn::Block) -> bool {
    todo!("whether an arm in a body of {} statements is more than a single call", body.stmts.len())
}

/// **F3** — whether a call in the body takes an argument outside the forms F3
/// admits: a path, a field access, a reference, an accessor chain.
#[implements(spec::AnArgumentOutsideTheAdmittedFormsMakesTheBodyALeafUnderF3)]
pub(crate) fn fails_f3(body: &syn::Block) -> bool {
    todo!("whether a call in a body of {} statements takes an argument F3 denies", body.stmts.len())
}

/// **F4** — whether the body holds a closure, or a block that is no arm of the
/// decision structure F2 admits.
///
/// The exception is the arm and not its contents: a closure written inside an
/// arm is a closure in the body. Which blocks are arms is
/// [`is_decision_structure`]'s answer, read from inside this rule.
#[implements(spec::AClosureOrANonArmBlockMakesTheBodyALeafUnderF4)]
pub(crate) fn fails_f4(body: &syn::Block) -> bool {
    todo!("whether a body of {} statements holds a closure or a non-arm block", body.stmts.len())
}

/// **F5** — whether the body holds a literal other than `()`.
#[implements(spec::ALiteralOtherThanUnitMakesTheBodyALeafUnderF5)]
pub(crate) fn fails_f5(body: &syn::Block) -> bool {
    todo!("whether a body of {} statements holds a literal but the unit", body.stmts.len())
}

/// **F6** — whether the body invokes a macro outside `allow_macros`.
///
/// The set is the caller's and this rule holds none of its own, so a project
/// that allows a macro is a project that passed it in.
#[implements(spec::AMacroOutsideTheAllowedSetMakesTheBodyALeafUnderF6)]
pub(crate) fn fails_f6(body: &syn::Block, allow_macros: &[String]) -> bool {
    todo!("whether a body of {} statements invokes a macro outside {allow_macros:?}", body.stmts.len())
}

/// How many arms the one decision structure the body holds routes among, and 0
/// for a body holding none.
///
/// This is the arity rule A is given, and it is a reading of the body rather
/// than a rule over it: no claim of this slice states what it counts, which is
/// recorded here so the gap is visible where the count is made.
pub(crate) fn dispatch_arity(body: &syn::Block) -> usize {
    todo!("the arms the decision structure in a body of {} statements routes among", body.stmts.len())
}

#[cfg(test)]
mod tests {
    //! One validator per rule, each over a body that breaks the rule and a
    //! body that does not.
    //!
    //! The pairing is what makes the test falsifiable: a rule answering the
    //! same thing for every body would satisfy half of each claim, and the
    //! bodies here are chosen so that only the rule under test separates them.
    //! They are parsed from source rather than built, because the tokens a
    //! source wrote are the whole of what a rule reads.
    //!
    //! Each rule is put its own body and not a whole verdict, because the
    //! verdict evaluates all six rules at once: aimed through
    //! [`crate::verdict`] every one of these would name the same rule, which is
    //! the opposite of what six claims were cut for.

    use lid_rs::validates;

    use super::*;

    /// The block a fixture wrote, parsed.
    fn block(source: &str) -> syn::Block {
        syn::parse_str(source).expect("a block the fixture wrote")
    }

    /// A body of F1's call forms alone: `let pat = call(...)?;`, a bare call,
    /// and the tail call.
    const CALLS: &str = "{ let read = source(path)?; record(read)?; answer(read) }";

    /// A body whose one decision structure is the `match` F2 admits, every arm
    /// a single call.
    const ONE_MATCH: &str = "{ match kind { Kind::A => first(input), Kind::B => second(input) } }";

    /// A body holding a second decision structure beside the one F2 admits.
    const TWO_MATCHES: &str = "{ match kind { Kind::A => first(input), Kind::B => second(input) } \
                               match other { Kind::A => third(input), Kind::B => fourth(input) } }";

    /// A body whose one decision structure is the two-way `if` F2 admits,
    /// whose arms are blocks by Rust's grammar and so are what F4 must except.
    const ONE_IF: &str = "{ if ready(input) { first(input) } else { second(input) } }";

    /// A statement that is neither a call form nor the decision structure F2
    /// admits makes the body a leaf; the call forms and that structure do not.
    #[test]
    #[validates(spec::ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1)]
    fn a_body_with_a_statement_that_is_no_call_form_is_a_leaf_under_f1() {
        let computes = block("{ let total = left + right; answer(total) }");
        assert_eq!(
            (fails_f1(&computes), fails_f1(&block(CALLS)), fails_f1(&block(ONE_MATCH))),
            (true, false, false),
            "F1 admits its three call forms and the statement F2 allows, and no other statement",
        );
    }

    /// A second decision structure makes the body a leaf; the one F2 admits
    /// does not.
    #[test]
    #[validates(spec::ABodyWithMoreThanOneDecisionStructureIsALeafUnderF2)]
    fn a_body_with_more_than_one_decision_structure_is_a_leaf_under_f2() {
        assert_eq!(
            (
                fails_f2_a_second_decision_structure(&block(TWO_MATCHES)),
                fails_f2_a_second_decision_structure(&block(ONE_MATCH)),
            ),
            (true, false),
            "one decision structure is admitted and a second is not",
        );
    }

    /// An arm that is more than a single call makes the body a leaf; arms that
    /// are each one call do not.
    #[test]
    #[validates(spec::ADecisionArmThatIsNoSingleCallMakesTheBodyALeafUnderF2)]
    fn a_decision_arm_that_is_no_single_call_makes_the_body_a_leaf_under_f2() {
        let working_arm = block("{ match kind { Kind::A => { let read = source(path)?; answer(read) } Kind::B => second(input) } }");
        assert_eq!(
            (fails_f2_an_arm_that_is_no_single_call(&working_arm), fails_f2_an_arm_that_is_no_single_call(&block(ONE_MATCH))),
            (true, false),
            "an arm doing work is a leaf's arm, and an arm that is one call is not",
        );
    }

    /// An argument outside the admitted forms makes the body a leaf; a path, a
    /// field access, a reference and an accessor chain do not.
    #[test]
    #[validates(spec::AnArgumentOutsideTheAdmittedFormsMakesTheBodyALeafUnderF3)]
    fn an_argument_outside_the_admitted_forms_makes_the_body_a_leaf_under_f3() {
        let computed = block("{ record(left + right)? }");
        let admitted = block("{ let read = source(&self.path)?; record(read.name())? }");
        assert_eq!(
            (fails_f3(&computed), fails_f3(&admitted)),
            (true, false),
            "an argument the caller computed is work in a routing node, and a written access is not",
        );
    }

    /// A closure, and a block that is no arm of the decision structure F2
    /// admits, each make the body a leaf; the arms of that structure do not.
    #[test]
    #[validates(spec::AClosureOrANonArmBlockMakesTheBodyALeafUnderF4)]
    fn a_closure_or_a_non_arm_block_makes_the_body_a_leaf_under_f4() {
        let closure = block("{ let answer = |input| respond(input); answer(read) }");
        let bare_block = block("{ { record(read)?; } answer(read) }");
        assert_eq!(
            (fails_f4(&closure), fails_f4(&bare_block), fails_f4(&block(ONE_IF))),
            (true, true, false),
            "F4's exception is the arm of the structure F2 admits, and nothing else is excepted",
        );
    }

    /// A literal other than `()` makes the body a leaf; the unit literal does
    /// not.
    #[test]
    #[validates(spec::ALiteralOtherThanUnitMakesTheBodyALeafUnderF5)]
    fn a_literal_other_than_unit_makes_the_body_a_leaf_under_f5() {
        assert_eq!(
            (fails_f5(&block("{ record(read, 3)? }")), fails_f5(&block("{ record(read, ())? }"))),
            (true, false),
            "F5 admits the unit literal alone",
        );
    }

    /// A macro outside the set the caller gave makes the body a leaf; one
    /// inside it does not — the set being the caller's and this rule holding
    /// none of its own.
    #[test]
    #[validates(spec::AMacroOutsideTheAllowedSetMakesTheBodyALeafUnderF6)]
    fn a_macro_outside_the_allowed_set_makes_the_body_a_leaf_under_f6() {
        let allowed = ["todo".to_string(), "unimplemented".to_string()];
        assert_eq!(
            (fails_f6(&block("{ assert!(ready); }"), &allowed), fails_f6(&block("{ todo!(\"not yet\") }"), &allowed)),
            (true, false),
            "the allowed set is the one passed in, so a skeleton's `todo!` is flow and another macro is not",
        );
    }
}
