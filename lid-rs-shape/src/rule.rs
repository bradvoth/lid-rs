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
    let syn::Stmt::Expr(expression, _) = stmt else { return false };
    if let syn::Expr::If(branch) = expression {
        return branch.else_branch.is_some();
    }
    matches!(expression, syn::Expr::Match(_))
}

/// Whether one expression is a call: a call, a method call, a macro
/// invocation, and any of those with the `?` F1's forms write after it.
///
/// A macro invocation is a call here because F6 is the rule about macros: a
/// body invoking one is a leaf when the set the caller gave does not hold it,
/// and flow when it does — which is what makes a skeleton of `todo!()` flow.
fn is_call(expression: &syn::Expr) -> bool {
    if let syn::Expr::Try(attempted) = expression {
        return is_call(&attempted.expr);
    }
    matches!(expression, syn::Expr::Call(_) | syn::Expr::MethodCall(_) | syn::Expr::Macro(_))
}

/// Whether one statement is one of the call forms F1 admits: `let pat =
/// call(...)?;`, a bare `call(...)?;`, or the tail call.
///
/// The decision structure F2 admits is not tested here — [`fails_f1`] admits it
/// beside these — so this item answers for the call forms alone.
#[implements(spec::ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1)]
fn is_admitted_statement(stmt: &syn::Stmt) -> bool {
    match stmt {
        syn::Stmt::Local(bound) => bound.init.as_ref().is_some_and(|init| init.diverge.is_none() && is_call(&init.expr)),
        syn::Stmt::Macro(_) => true,
        syn::Stmt::Expr(expression, _) => is_call(expression),
        syn::Stmt::Item(_) => false,
    }
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
    body.stmts.iter().filter(|stmt| is_decision_structure(stmt)).any(|stmt| !arms_are_single_calls(stmt))
}

/// Whether every arm of the decision structure one statement holds is the
/// single call F2 admits.
///
/// A statement holding no decision structure has no arm to fail, so it is
/// answered as one whose arms are all single calls: which statements this rule
/// is asked about is the filter above's answer, not this one's.
fn arms_are_single_calls(stmt: &syn::Stmt) -> bool {
    let syn::Stmt::Expr(expression, _) = stmt else { return true };
    if let syn::Expr::Match(routed) = expression {
        return routed.arms.iter().all(|arm| is_single_call(&arm.body));
    }
    if let syn::Expr::If(branch) = expression {
        return is_single_call_block(&branch.then_branch)
            && branch.else_branch.as_ref().is_some_and(|(_, otherwise)| is_single_call(otherwise));
    }
    true
}

/// Whether one arm is the single call F2 admits: the call itself, as a `match`
/// arm writes one, or a block holding one, as an `if` arm does.
fn is_single_call(arm: &syn::Expr) -> bool {
    if let syn::Expr::Block(wrapped) = arm {
        return is_single_call_block(&wrapped.block);
    }
    is_call(arm)
}

/// Whether one block is a single call: one statement, and that statement a
/// call.
fn is_single_call_block(block: &syn::Block) -> bool {
    matches!(&block.stmts[..], [syn::Stmt::Expr(only, _)] if is_call(only))
}

/// **F3** — whether a call in the body takes an argument outside the forms F3
/// admits: a path, a field access, a reference, an accessor chain.
#[implements(spec::AnArgumentOutsideTheAdmittedFormsMakesTheBodyALeafUnderF3)]
pub(crate) fn fails_f3(body: &syn::Block) -> bool {
    body.stmts.iter().flat_map(statement_calls).flat_map(arguments_of).any(|argument| !is_admitted_argument(argument))
}

/// The calls one statement makes at the top of its form: the call a `let`
/// binds, and the calls the expression a statement is makes.
fn statement_calls(stmt: &syn::Stmt) -> Vec<&syn::Expr> {
    if let syn::Stmt::Local(bound) = stmt {
        return bound.init.iter().flat_map(|init| expression_calls(&init.expr)).collect();
    }
    if let syn::Stmt::Expr(expression, _) = stmt {
        return expression_calls(expression);
    }
    Vec::new()
}

/// The calls one expression makes: the expression itself, unwrapped of the `?`
/// a call form writes after it, and the calls of the branches it opens where
/// it opens any.
///
/// An expression that is no call is answered as one all the same. The rules
/// that read this ask what a call takes, holds or invokes, and an expression in
/// a call's place that is not one is exactly what they are looking for.
fn expression_calls(expression: &syn::Expr) -> Vec<&syn::Expr> {
    if let syn::Expr::Try(attempted) = expression {
        return expression_calls(&attempted.expr);
    }
    branched_calls(expression).unwrap_or_else(|| vec![expression])
}

/// The calls the branches of one expression make, and none where it opens no
/// branch: a `match`'s arms beside what it routes on, an `if`'s branches
/// beside what it tests, and the statements of a block.
///
/// The arms are descended into because F2 admits them as arms and no rule
/// admits what they hold: a literal, a closure or a denied argument written
/// inside an arm is written inside the body.
fn branched_calls(expression: &syn::Expr) -> Option<Vec<&syn::Expr>> {
    if let syn::Expr::Match(routed) = expression {
        let arms = routed.arms.iter().flat_map(|arm| expression_calls(&arm.body));
        return Some(expression_calls(&routed.expr).into_iter().chain(arms).collect());
    }
    if let syn::Expr::If(branch) = expression {
        let otherwise = branch.else_branch.iter().flat_map(|(_, alternative)| expression_calls(alternative));
        return Some(expression_calls(&branch.cond).into_iter().chain(block_calls(&branch.then_branch)).chain(otherwise).collect());
    }
    if let syn::Expr::Block(wrapped) = expression {
        return Some(block_calls(&wrapped.block));
    }
    None
}

/// The calls the statements of one block make.
fn block_calls(block: &syn::Block) -> Vec<&syn::Expr> {
    block.stmts.iter().flat_map(statement_calls).collect()
}

/// The arguments one call passes, and none for an expression that is no call.
///
/// The receiver a method call is written on is no argument of it: it is the
/// head of the accessor chain the argument forms admit, and it is read as one
/// where the call is itself an argument.
fn arguments_of(call: &syn::Expr) -> Vec<&syn::Expr> {
    if let syn::Expr::Call(called) = call {
        return called.args.iter().collect();
    }
    if let syn::Expr::MethodCall(called) = call {
        return called.args.iter().collect();
    }
    Vec::new()
}

/// Whether one argument is written in a form F3 admits: a path, a field
/// access, a reference to one of those, or an accessor chain — a method call
/// passing no arguments of its own, on a receiver written the same way.
fn is_admitted_argument(argument: &syn::Expr) -> bool {
    if let syn::Expr::Reference(taken) = argument {
        return is_admitted_argument(&taken.expr);
    }
    if let syn::Expr::Field(accessed) = argument {
        return is_admitted_argument(&accessed.base);
    }
    if let syn::Expr::MethodCall(chained) = argument {
        return chained.args.is_empty() && is_admitted_argument(&chained.receiver);
    }
    matches!(argument, syn::Expr::Path(_))
}

/// **F4** — whether the body holds a closure, or a block that is no arm of the
/// decision structure F2 admits.
///
/// The exception is the arm and not its contents: a closure written inside an
/// arm is a closure in the body. Which blocks are arms is
/// [`is_decision_structure`]'s answer, read from inside this rule.
#[implements(spec::AClosureOrANonArmBlockMakesTheBodyALeafUnderF4)]
pub(crate) fn fails_f4(body: &syn::Block) -> bool {
    body.stmts.iter().any(|stmt| is_non_arm_block(stmt) || holds_a_closure(stmt))
}

/// Whether one statement is a block of the body's own, which is every block a
/// body holds that is not an arm: the arms of the structure F2 admits are
/// written inside that structure and are never statements of the body.
fn is_non_arm_block(stmt: &syn::Stmt) -> bool {
    matches!(stmt, syn::Stmt::Expr(syn::Expr::Block(_), _))
}

/// Whether one statement holds a closure: the value it binds, a call it makes,
/// or an argument one of those passes.
fn holds_a_closure(stmt: &syn::Stmt) -> bool {
    statement_expressions(stmt).into_iter().any(|expression| matches!(expression, syn::Expr::Closure(_)))
}

/// Every expression one statement puts in reach of the rules that read a body
/// for a closure, a literal or a macro: the calls it makes and the arguments
/// they pass.
fn statement_expressions(stmt: &syn::Stmt) -> Vec<&syn::Expr> {
    statement_calls(stmt).into_iter().flat_map(|call| std::iter::once(call).chain(arguments_of(call))).collect()
}

/// **F5** — whether the body holds a literal other than `()`.
#[implements(spec::ALiteralOtherThanUnitMakesTheBodyALeafUnderF5)]
pub(crate) fn fails_f5(body: &syn::Block) -> bool {
    body.stmts.iter().flat_map(statement_expressions).any(|expression| matches!(expression, syn::Expr::Lit(_)))
}

/// **F6** — whether the body invokes a macro outside `allow_macros`.
///
/// The set is the caller's and this rule holds none of its own, so a project
/// that allows a macro is a project that passed it in.
#[implements(spec::AMacroOutsideTheAllowedSetMakesTheBodyALeafUnderF6)]
pub(crate) fn fails_f6(body: &syn::Block, allow_macros: &[String]) -> bool {
    body.stmts.iter().flat_map(macro_names).any(|invoked| !allow_macros.contains(&invoked))
}

/// The macros one statement invokes, each by the name its path writes last —
/// the statement itself where it is an invocation, and the expressions it puts
/// in reach otherwise.
///
/// The name and not the path: a body writing `std::todo!()` invokes the macro
/// a caller allows as `todo`, and this pass resolves no path to find that out.
fn macro_names(stmt: &syn::Stmt) -> Vec<String> {
    if let syn::Stmt::Macro(invoked) = stmt {
        return last_segment(&invoked.mac.path).into_iter().collect();
    }
    statement_expressions(stmt)
        .into_iter()
        .filter_map(|expression| {
            let syn::Expr::Macro(invoked) = expression else { return None };
            last_segment(&invoked.mac.path)
        })
        .collect()
}

/// The last segment of a path, as the source wrote it.
fn last_segment(path: &syn::Path) -> Option<String> {
    path.segments.last().map(|segment| segment.ident.to_string())
}

/// How many arms the one decision structure the body holds routes among, and 0
/// for a body holding none.
///
/// A reading of the body rather than a rule over it — rule A compares this
/// against the arm count it was given rather than counting anything itself —
/// but a wrong count here is rule A's wrong answer: a body routing among a
/// dozen kinds counted as routing among none is a leaf the rule cannot see. The
/// three items that make the count therefore cite the rule they feed.
#[implements(spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA)]
pub(crate) fn dispatch_arity(body: &syn::Block) -> usize {
    body.stmts.iter().filter_map(routed_arms).max().unwrap_or(0)
}

/// The arms the decision structure one statement holds routes among, and none
/// for a statement holding none: the arms of a `match`, and the branches of an
/// `if` chain.
///
/// The structure is read whether the statement is it or a `let` binds its
/// value, because a body routing among a dozen kinds routes among them either
/// way, and rule A is stated over what a function routes among.
#[implements(spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA)]
fn routed_arms(stmt: &syn::Stmt) -> Option<usize> {
    let decided = match stmt {
        syn::Stmt::Local(bound) => bound.init.as_ref().map(|init| &*init.expr),
        syn::Stmt::Expr(expression, _) => Some(expression),
        syn::Stmt::Item(_) | syn::Stmt::Macro(_) => None,
    }?;
    if let syn::Expr::Match(routed) = decided {
        return Some(routed.arms.len());
    }
    if let syn::Expr::If(branch) = decided {
        return Some(branches(branch));
    }
    None
}

/// The branches one `if` chain routes among: one for an `if` with no `else`,
/// two for one with an `else`, and one more for every `else if` beyond it.
///
/// A chain is where the count and the arm count part company — a `match` says
/// how many arms it has and a chain has to be walked — so this is the arity
/// rule A is most easily given wrong.
#[implements(spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA)]
fn branches(branch: &syn::ExprIf) -> usize {
    let Some((_, otherwise)) = &branch.else_branch else { return 1 };
    if let syn::Expr::If(chained) = &**otherwise {
        return 1 + branches(chained);
    }
    2
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
    ///
    /// Both structures F2 admits are put to the rule, and each of the `if`'s
    /// two arms is asked separately: a `match` arm is written as a call and an
    /// `if` arm as a block holding one, so a rule that read only the first
    /// would answer for half of what F2 admits.
    #[test]
    #[validates(spec::ADecisionArmThatIsNoSingleCallMakesTheBodyALeafUnderF2)]
    fn a_decision_arm_that_is_no_single_call_makes_the_body_a_leaf_under_f2() {
        let working_arm = block("{ match kind { Kind::A => { let read = source(path)?; answer(read) } Kind::B => second(input) } }");
        let working_else = block("{ if ready(input) { first(input) } else { let read = source(path)?; answer(read) } }");
        assert_eq!(
            (
                fails_f2_an_arm_that_is_no_single_call(&working_arm),
                fails_f2_an_arm_that_is_no_single_call(&working_else),
                fails_f2_an_arm_that_is_no_single_call(&block(ONE_MATCH)),
                fails_f2_an_arm_that_is_no_single_call(&block(ONE_IF)),
            ),
            (true, true, false, false),
            "an arm doing work is a leaf's arm, wherever it is written, and an arm that is one call is not",
        );
    }

    /// An argument outside the admitted forms makes the body a leaf; a path, a
    /// field access, a reference and an accessor chain do not.
    #[test]
    #[validates(spec::AnArgumentOutsideTheAdmittedFormsMakesTheBodyALeafUnderF3)]
    fn an_argument_outside_the_admitted_forms_makes_the_body_a_leaf_under_f3() {
        let computed = block("{ record(left + right)? }");
        let chained = block("{ answer(read.of(kind))? }");
        let admitted = block("{ let read = source(&self.path)?; record(read.name())? }");
        assert_eq!(
            (fails_f3(&computed), fails_f3(&chained), fails_f3(&admitted)),
            (true, true, false),
            "an argument the caller computed is work in a routing node, as is a method passed one, and a written access is not",
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
        let in_an_arm = block("{ if ready(input) { first(3) } else { second(input) } }");
        assert_eq!(
            (
                fails_f5(&block("{ record(read, 3)? }")),
                fails_f5(&in_an_arm),
                fails_f5(&block("{ record(read, ())? }")),
                fails_f5(&block(ONE_IF)),
            ),
            (true, true, false, false),
            "F5 admits the unit literal alone, and a literal written inside an arm is written inside the body",
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
