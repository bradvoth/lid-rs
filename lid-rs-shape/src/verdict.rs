//! One function's verdict, composed from the six rules' answers.

use lid_rs::implements;

use crate::{Shape, function::Function, rule, spec};

/// One function's [`Shape`]: the verdict the six rules compose, beside the
/// three readings of the declaration a rule is stated over.
///
/// Flow is the absence of a failure and nothing else — a body failing none of
/// the six is flow, and one failing any is a leaf — so the verdict and the rule
/// a report names are one answer read two ways.
#[implements(spec::ABodyFailingNoneOfTheSixRulesIsFlow)]
pub(crate) fn shape_of(function: &Function, allow_macros: &[String]) -> Shape {
    let failed = first_failed(&function.block, allow_macros);
    Shape {
        file: function.file.clone(),
        function: function.sig.ident.to_string(),
        public: is_public(&function.vis),
        flow: failed.is_none(),
        first_failed: failed,
        dispatch_arity: rule::dispatch_arity(&function.block),
        marked_leaf: marked_leaf(&function.attrs),
    }
}

/// The lowest-numbered rule of F1 through F6 the body fails, and none for a
/// body that fails none.
///
/// The order is this list's order, which is the numbers' order: a body failing
/// two rules is named by the first of them, so a report is stable under a body
/// that breaks more than one rule at once. Both halves of F2 answer 2, being
/// one rule cut in two for the two ways it can be broken.
#[implements(spec::TheVerdictNamesTheLowestNumberedRuleTheBodyFailed)]
fn first_failed(body: &syn::Block, allow_macros: &[String]) -> Option<u8> {
    [
        (1, rule::fails_f1(body)),
        (2, rule::fails_f2_a_second_decision_structure(body)),
        (2, rule::fails_f2_an_arm_that_is_no_single_call(body)),
        (3, rule::fails_f3(body)),
        (4, rule::fails_f4(body)),
        (5, rule::fails_f5(body)),
        (6, rule::fails_f6(body, allow_macros)),
    ]
    .into_iter()
    .find_map(|(number, failed)| failed.then_some(number))
}

/// Whether the declaration says `pub`, which is half of what rule B is stated
/// over.
///
/// A reading of the declaration and not a rule over it — rule B is given this
/// answer rather than making it — but a wrong answer here is rule B's wrong
/// answer: a reading that called every declaration public would report the
/// private leaves of a slice's `mod.rs`, and one that called none public would
/// report nothing at all. That is why it cites the rule it feeds.
#[implements(spec::AnUnmarkedPublicLeafInASliceModIsAFindingUnderRuleB)]
fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Whether the declaration carries `#[leaf]`.
///
/// The mark changes no verdict — the classification is derived from the body,
/// and a mark that could change it would be an opt-out of the derivation — but
/// it is carried on the shape, because the public leaves rule B allowed are
/// counted from the classification and nowhere else.
#[implements(spec::TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted)]
fn marked_leaf(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| attribute.path().is_ident("leaf"))
}

#[cfg(test)]
mod tests {
    //! The verdict over functions parsed from source: the rule a report names,
    //! flow as the absence of a failure, and the mark the shape carries.
    //!
    //! The functions are parsed whole rather than assembled field by field, so
    //! what a test states is what a source could write. The file is a fixture's
    //! and is never read: nothing here touches a filesystem, because a verdict
    //! is composed from tokens alone.

    use std::path::PathBuf;

    use lid_rs::validates;

    use super::*;

    /// A function as the pass reads one, from the source a fixture wrote.
    fn function(source: &str) -> Function {
        let parsed: syn::ItemFn = syn::parse_str(source).expect("a function the fixture wrote");
        Function {
            file: PathBuf::from("src/hello/mod.rs"),
            vis: parsed.vis,
            attrs: parsed.attrs,
            sig: parsed.sig,
            block: *parsed.block,
        }
    }

    /// The block a fixture wrote, parsed.
    fn block(source: &str) -> syn::Block {
        syn::parse_str(source).expect("a block the fixture wrote")
    }

    /// A body whose statements are the call forms F1 admits and nothing else.
    const ROUTES: &str = "fn routes() { let read = source(path)?; answer(read) }";

    /// A body doing arithmetic, which fails F1 and is a leaf.
    const WORKS: &str = "fn works() { let total = left + right; answer(total) }";

    /// A body that breaks two rules is reported under the lower-numbered of
    /// them: a literal argument breaks F3 and F5 and is named 3, and a closure
    /// bound by a `let` breaks F1 and F4 and is named 1.
    #[test]
    #[validates(spec::TheVerdictNamesTheLowestNumberedRuleTheBodyFailed)]
    fn the_verdict_names_the_lowest_numbered_rule_the_body_failed() {
        let literal_argument = block("{ record(read, 3)?; }");
        let bound_closure = block("{ let answer = |input| respond(input); answer(read) }");
        assert_eq!(
            (first_failed(&literal_argument, &[]), first_failed(&bound_closure, &[])),
            (Some(3), Some(1)),
            "the lowest-numbered rule among those the body failed, so a report is stable under a body breaking two",
        );
    }

    /// A body failing none of the six is flow and names no rule; one failing
    /// any is a leaf and names the rule.
    #[test]
    #[validates(spec::ABodyFailingNoneOfTheSixRulesIsFlow)]
    fn a_body_failing_none_of_the_six_rules_is_flow() {
        let routing = shape_of(&function(ROUTES), &[]);
        let working = shape_of(&function(WORKS), &[]);
        assert_eq!(
            ((routing.flow, routing.first_failed), (working.flow, working.first_failed)),
            ((true, None), (false, Some(1))),
            "flow is the absence of a failure, and a leaf's shape names the rule a report names",
        );
    }

    /// The mark is read from the declaration's attributes and carried onto the
    /// shape, which is the only place a reader of the classification can count
    /// the public leaves rule B allowed.
    #[test]
    #[validates(spec::TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted)]
    fn the_shape_of_a_marked_function_carries_the_mark_so_it_is_counted() {
        let read_from_the_attributes = marked_leaf(&function("#[leaf] pub fn allowed() { work(input) }").attrs);
        let carried = shape_of(&function("#[leaf] pub fn allowed() { let total = left + right; answer(total) }"), &[]);
        let unmarked = shape_of(&function(WORKS), &[]);
        assert_eq!(
            (read_from_the_attributes, carried.marked_leaf, unmarked.marked_leaf),
            (true, true, false),
            "the mark is read from the attributes and reaches the classification, and an unmarked function carries none",
        );
    }
}
