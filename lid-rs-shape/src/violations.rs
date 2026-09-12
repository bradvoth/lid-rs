//! Rules A, B and V over one function's shape.
//!
//! Three rules, three answers, no level: each says whether what it was given is
//! a violation and what it saw, and nothing here weighs one answer against
//! another or decides what a violation costs.

use std::path::PathBuf;

use lid_rs::implements;

use crate::{Finding, Shape, Signature, spec};

/// Every finding the three rules raise against one function.
///
/// The rules do not order or exclude one another: a function can be a rule A
/// violation and a rule B violation at once, and each is reported.
pub(crate) fn findings_for(
    shape: &Shape,
    signatures: &[Signature],
    slice_mods: &[PathBuf],
    dispatch_arms: usize,
    wrappers: &[String],
) -> Vec<Finding> {
    rule_a(shape, dispatch_arms)
        .into_iter()
        .chain(rule_b(shape, slice_mods))
        .chain(rule_v(shape, signatures, wrappers))
        .collect()
}

/// **A** — a leaf whose dispatch arity reaches `dispatch_arms` is a finding.
///
/// Routing among that many kinds is a flow decision, and a leaf making one is
/// doing work in the same function. The arity and the verdict are both read
/// from the shape: this rule counts nothing and classifies nothing.
#[implements(spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA)]
fn rule_a(shape: &Shape, dispatch_arms: usize) -> Option<Finding> {
    todo!("whether `{}` routes among {dispatch_arms} kinds as a leaf", shape.function)
}

/// **B** — a public leaf in one of `slice_mods` is a finding unless it carries
/// `#[leaf]`.
///
/// One decision with two sides, which is why both of rule B's claims are
/// answered here: the mark is the escape from this finding, and a marked
/// public leaf is a finding for no rule. Whether a `mod.rs` is a *slice's* is a
/// question about a project's layout that this pass cannot answer and is given
/// the answer to.
#[implements(
    spec::AnUnmarkedPublicLeafInASliceModIsAFindingUnderRuleB,
    spec::APublicLeafMarkedLeafIsNoFindingUnderRuleB,
)]
fn rule_b(shape: &Shape, slice_mods: &[PathBuf]) -> Option<Finding> {
    todo!("whether `{}` is an unmarked public leaf in one of {} slice mods", shape.function, slice_mods.len())
}

/// **V** — a flow function whose parameter, or whose `Ok` type, is written as a
/// name the rule denies is a finding, one per denied position.
///
/// Flow functions only: the rule is about the signature of a routing node, and
/// a leaf's signature is not its subject.
fn rule_v(shape: &Shape, signatures: &[Signature], wrappers: &[String]) -> Vec<Finding> {
    shape
        .flow
        .then(|| signature_for(shape, signatures))
        .flatten()
        .into_iter()
        .flat_map(|signature| {
            denied_parameters(signature, wrappers).into_iter().chain(denied_ok_type(signature, wrappers))
        })
        .map(|saw| Finding::RuleV { file: shape.file.clone(), function: shape.function.clone(), saw })
        .collect()
}

/// The signature read from the same declaration as this shape, joined on the
/// file and the name, and none where the two answers hold no such pair.
fn signature_for<'a>(shape: &Shape, signatures: &'a [Signature]) -> Option<&'a Signature> {
    todo!("the signature of `{}` among {} of them", shape.function, signatures.len())
}

/// What the rule saw at each parameter written as a name it denies.
#[implements(spec::AFlowParameterWrittenAsADeniedNameIsAFindingUnderRuleV)]
fn denied_parameters(signature: &Signature, wrappers: &[String]) -> Vec<String> {
    signature
        .parameters
        .iter()
        .filter(|written| is_denied_name(&tested_name(written, wrappers)))
        .map(|written| format!("the parameter written `{written}`, denied as `{}`", tested_name(written, wrappers)))
        .collect()
}

/// What the rule saw where the `Ok` type of the written return is a name it
/// denies.
///
/// The `Ok` type and not the whole return: the return tokens are carried whole
/// and this rule reads the one position out of them.
#[implements(spec::AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV)]
fn denied_ok_type(signature: &Signature, wrappers: &[String]) -> Option<String> {
    let written = ok_type(signature.returns.as_deref()?)?;
    is_denied_name(&tested_name(written, wrappers))
        .then(|| format!("the `Ok` type written `{written}`, denied as `{}`", tested_name(written, wrappers)))
}

/// The `Ok` type of a written return, and none where the return is not a
/// `Result` as the source wrote it.
#[implements(spec::AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV)]
fn ok_type(returns: &str) -> Option<&str> {
    todo!("the `Ok` type written inside `{returns}`")
}

/// The name rule V tests a written type by: the name inside every wrapper the
/// caller named, and the name as written where there is none.
///
/// Written, never resolved. The name tested is the one the source spelled,
/// rather than one a `use` or a type alias would turn it into — which is why
/// this item is given the tokens and nothing to resolve them against.
#[implements(
    spec::AGivenWrapperIsUnwrappedBeforeRuleVTestsTheName,
    spec::RuleVTestsTheNameTheSourceWroteAndResolvesNothing,
)]
fn tested_name(written: &str, wrappers: &[String]) -> String {
    todo!("the name `{written}` is tested by, unwrapping {wrappers:?}")
}

/// Whether a name is one rule V denies: `String`, `&str`, `bool`, the integer
/// types, the float types and `char`, and no other name.
///
/// The deny clause, which is narrower than the rule it comes from: the design
/// states rule V as an allow-list of vocabulary types, an allow-list needs the
/// set of vocabulary names, and no registry in reach holds one. This misses
/// every non-vocabulary type that is not a primitive, and says so here so that
/// no reader takes the set for the rule.
#[implements(spec::RuleVDeniesReadmesDenyClauseAndNoOtherName)]
fn is_denied_name(name: &str) -> bool {
    todo!("whether `{name}` is one of the names rule V denies")
}
