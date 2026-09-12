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
