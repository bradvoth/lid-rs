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
    (!shape.flow && shape.dispatch_arity >= dispatch_arms).then(|| Finding::RuleA {
        file: shape.file.clone(),
        function: shape.function.clone(),
        saw: format!(
            "routing among {} kinds as a leaf under F{}",
            shape.dispatch_arity,
            shape.first_failed.unwrap_or_default()
        ),
    })
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
    let violating = shape.public && !shape.flow && !shape.marked_leaf && slice_mods.contains(&shape.file);
    violating.then(|| Finding::RuleB {
        file: shape.file.clone(),
        function: shape.function.clone(),
        saw: format!("a leaf under F{}", shape.first_failed.unwrap_or_default()),
    })
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
    signatures.iter().find(|signature| signature.file == shape.file && signature.function == shape.function)
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
    let (outer, arguments) = returns.trim().strip_suffix('>')?.split_once('<')?;
    (outer.trim() == "Result").then(|| first_argument(arguments))
}

/// The first of the type arguments written between a pair of angle brackets:
/// what precedes the first comma no inner pair encloses, and the whole of them
/// where no comma does.
///
/// The comma is found by balance rather than by counting depth as the tokens
/// are walked: `Result<Vec<T>, E>` holds `Vec<T>` first, and the comma inside
/// that `Vec` is the one that leaves its brackets unbalanced.
fn first_argument(arguments: &str) -> &str {
    arguments
        .match_indices(',')
        .map(|(at, _)| &arguments[..at])
        .find(|held| held.matches('<').count() == held.matches('>').count())
        .unwrap_or(arguments)
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
    let mut name: String = written.chars().filter(|character| !character.is_whitespace()).collect();
    for _ in 0..name.len() {
        match unwrapped_once(&name, wrappers).map(str::to_string) {
            Some(held) => name = held,
            None => break,
        }
    }
    name
}

/// The type one wrapper the caller named holds, and none where the name is
/// written as no wrapper of theirs.
///
/// One layer, so that the loop above is the whole of the unwrapping: a name
/// written as a wrapper of a wrapper is answered a layer at a time, and the
/// tokens are spacing-free by the time they reach here.
fn unwrapped_once<'a>(name: &'a str, wrappers: &[String]) -> Option<&'a str> {
    let (outer, arguments) = name.strip_suffix('>')?.split_once('<')?;
    wrappers.iter().any(|wrapper| wrapper == outer).then(|| first_argument(arguments))
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
    DENIED.contains(&name)
}

/// The names rule V denies, as the design's deny clause writes them: `String`,
/// `&str`, `bool`, `char`, and every integer and float type.
///
/// The clause and nothing beyond it. A name the clause does not hold is not
/// denied however unlike a vocabulary type it is, because the rule this crate
/// implements is the deny clause and not the allow-list it was narrowed from.
const DENIED: [&str; 18] = [
    "String", "&str", "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
    "u64", "u128", "usize",
];

#[cfg(test)]
mod tests {
    //! The three decisions this module makes that no composition above it
    //! could make instead: the mark that is rule B's escape, the set of names
    //! rule V denies, and the name a written type is tested by.
    //!
    //! The rules that the composition could also get wrong — by dropping a
    //! finding it was handed — are validated through [`crate::check`] instead,
    //! where an empty answer is a visible wrong answer.
    //!
    //! Nothing here touches a filesystem. A shape and a signature are plain
    //! data the caller of `check` hands over, so a fixture writes them.

    use lid_rs::validates;

    use super::*;

    /// A leaf as the classification carries one: not public, not marked, and
    /// routing among nothing.
    fn leaf(function: &str, file: &str) -> Shape {
        Shape {
            file: PathBuf::from(file),
            function: function.to_string(),
            public: false,
            flow: false,
            first_failed: Some(1),
            dispatch_arity: 0,
            marked_leaf: false,
        }
    }

    /// Owned names, as the caller hands its wrappers over.
    fn strings(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    /// The `mod.rs` a fixture's slice is in.
    const SLICE_MOD: &str = "src/hello/mod.rs";

    /// The mark is the escape from rule B's finding, and the absence of it is
    /// not: a pass that dropped every marked function would answer none for
    /// both.
    #[test]
    #[validates(spec::APublicLeafMarkedLeafIsNoFindingUnderRuleB)]
    fn a_public_leaf_marked_leaf_is_no_finding_under_rule_b() {
        let mods = vec![PathBuf::from(SLICE_MOD)];
        let marked = rule_b(&Shape { public: true, marked_leaf: true, ..leaf("allowed", SLICE_MOD) }, &mods);
        let unmarked = rule_b(&Shape { public: true, ..leaf("counted", SLICE_MOD) }, &mods);
        assert_eq!(
            (marked, unmarked.is_some()),
            (None, true),
            "`#[leaf]` is rule B's escape, and a public leaf without it is still the rule's finding",
        );
    }

    /// The denied names are README's deny clause and no other name — which is
    /// this crate's narrowing of rule V, stated as an allow-list over
    /// vocabulary types that nothing in reach enumerates.
    #[test]
    #[validates(spec::RuleVDeniesReadmesDenyClauseAndNoOtherName)]
    fn rule_v_denies_readmes_deny_clause_and_no_other_name() {
        let clause = ["String", "&str", "bool", "u8", "usize", "i64", "f32", "f64", "char"];
        let outside = ["PathBuf", "Duration", "Report", "Outcome"];
        let denied: Vec<&str> = clause.into_iter().filter(|name| is_denied_name(name)).collect();
        let also_denied: Vec<&str> = outside.into_iter().filter(|name| is_denied_name(name)).collect();
        assert_eq!(
            (denied, also_denied),
            (clause.to_vec(), Vec::new()),
            "every name README's deny clause holds, and no name outside it — the narrowing, not the allow-list",
        );
    }

    /// A type written as one of the wrappers the caller gave is tested by the
    /// name that wrapper holds, however the tokens are spaced, and a type
    /// written as no wrapper is tested by the name as written.
    ///
    /// The last case is the one that says which way the comparison runs: a
    /// generic type the caller did not name is held whole, so a rule that
    /// unwrapped everything with angle brackets would answer `Kind` where the
    /// source wrote `Registry<Kind>`.
    #[test]
    #[validates(spec::AGivenWrapperIsUnwrappedBeforeRuleVTestsTheName)]
    fn a_given_wrapper_is_unwrapped_before_rule_v_tests_the_name() {
        let wrappers = strings(&["Option", "Vec", "Box", "Arc"]);
        assert_eq!(
            (
                tested_name("Option<String>", &wrappers),
                tested_name("Vec < Box < Report > >", &wrappers),
                tested_name("Report", &wrappers),
                tested_name("Registry < Kind >", &wrappers),
            ),
            ("String".to_string(), "Report".to_string(), "Report".to_string(), "Registry<Kind>".to_string()),
            "the name inside every wrapper the caller named, and the written name where the wrapper is not one of theirs",
        );
    }
}
