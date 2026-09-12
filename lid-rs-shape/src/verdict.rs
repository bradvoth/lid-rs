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
/// A reading of the declaration and not a rule over it: no claim of this slice
/// states it, and rule B is given the answer rather than making it.
fn is_public(vis: &syn::Visibility) -> bool {
    todo!("whether the {:?} visibility is public", std::mem::discriminant(vis))
}

/// Whether the declaration carries `#[leaf]`.
///
/// The mark changes no verdict — the classification is derived from the body,
/// and a mark that could change it would be an opt-out of the derivation — but
/// it is carried on the shape, because the public leaves rule B allowed are
/// counted from the classification and nowhere else.
#[implements(spec::TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted)]
fn marked_leaf(attrs: &[syn::Attribute]) -> bool {
    todo!("whether `#[leaf]` is among the {} attributes of the declaration", attrs.len())
}
