use crate::canary;
use crate::registry::{Edge, SpecMeta};
use crate::spec;
use lid::implements;
use std::collections::HashSet;

/// Which edge set a graph check runs against.
#[derive(Clone, Copy, Debug)]
pub enum EdgeKind {
    /// Check specs against implementation citations (check 10).
    Implementations,
    /// Check specs against validation citations (check 11).
    Validations,
}

/// The canary triple is absent: the linker sections were stripped or never
/// populated, so the registries cannot be trusted and no orphan claim is
/// made at all (README §5.3).
#[derive(Debug)]
pub struct CanaryStripped;

/// Finds current-crate specs lacking an edge in the chosen set, refusing to
/// run at all over an untrustworthy registry.
///
/// Parameterized over the slices so every failure branch is testable with
/// synthetic inputs; `intent_graph!` applies it to the real registries.
#[implements(spec::GraphChecksRequireThePresentCanary)]
pub fn graph_orphans(
    crate_name: &str,
    specs: &[SpecMeta],
    impls: &[Edge],
    validations: &[Edge],
    kind: EdgeKind,
) -> Result<Vec<String>, CanaryStripped> {
    todo!()
}

/// Specs whose `NAME` begins with `crate_name::` and that no edge cites,
/// formatted `name (file:line)`.
#[implements(
    spec::UncitedSpecsFailTheGraphCheck,
    spec::UnvalidatedSpecsFailTheGraphCheck,
    spec::GraphChecksScopeToTheCurrentCrate,
    spec::CoveredGraphsPassTheGraphCheck,
)]
fn orphaned_specs(crate_name: &str, specs: &[SpecMeta], edges: &[Edge]) -> Vec<String> {
    todo!()
}

/// Expands to the three intent-graph tests (README §4.2): canary presence,
/// every-spec-implemented, every-spec-validated — scoped to the invoking
/// crate. Invoke inside a `#[cfg(test)]` module of the library:
///
/// ```
/// #[cfg(test)]
/// mod intent_graph {
///     lid::intent_graph!();
/// }
/// # fn main() {}
/// ```
#[macro_export]
macro_rules! intent_graph {
    () => {
        #[test]
        fn registry_is_populated() {
            assert!(
                $crate::canary::present(),
                "canary triple missing - registry sections were stripped or never populated (README §5.3)"
            );
        }

        #[test]
        fn every_spec_has_an_implementer() {
            let orphans = $crate::graph::graph_orphans(
                env!("CARGO_CRATE_NAME"),
                &$crate::SPECS,
                &$crate::IMPLEMENTATIONS,
                &$crate::VALIDATIONS,
                $crate::graph::EdgeKind::Implementations,
            )
            .expect("canary triple missing - registry cannot be trusted (README §5.3)");
            assert!(orphans.is_empty(), "specs with no implementation:\n{orphans:#?}");
        }

        #[test]
        fn every_spec_has_a_validation() {
            let orphans = $crate::graph::graph_orphans(
                env!("CARGO_CRATE_NAME"),
                &$crate::SPECS,
                &$crate::IMPLEMENTATIONS,
                &$crate::VALIDATIONS,
                $crate::graph::EdgeKind::Validations,
            )
            .expect("canary triple missing - registry cannot be trusted (README §5.3)");
            assert!(orphans.is_empty(), "specs with no validation:\n{orphans:#?}");
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid::validates;

    /// Synthetic registration at a fixed location.
    fn meta(name: &'static str) -> SpecMeta {
        SpecMeta { name, file: "synthetic.rs", line: 1 }
    }

    /// Synthetic edge citing `spec`.
    fn edge(spec: &'static str) -> Edge {
        Edge { spec, item: "synthetic", file: "synthetic.rs", line: 1 }
    }

    /// The canary join key, so synthetic registries count as trustworthy.
    const CANARY: &str = <crate::spec::CanaryConfirmsRegistryPresence as crate::Spec>::NAME;

    #[test]
    #[validates(spec::UncitedSpecsFailTheGraphCheck)]
    fn uncited_specs_fail_the_graph_check() {
        let specs = [meta(CANARY), meta("lid::synthetic::Fake")];
        let impls = [edge(CANARY)];
        let validations = [edge(CANARY)];
        let orphans =
            graph_orphans("lid", &specs, &impls, &validations, EdgeKind::Implementations)
                .expect("canary is present in the synthetic registry");
        assert_eq!(orphans, ["lid::synthetic::Fake (synthetic.rs:1)"]);
    }

    #[test]
    #[validates(spec::UnvalidatedSpecsFailTheGraphCheck)]
    fn unvalidated_specs_fail_the_graph_check() {
        let specs = [meta(CANARY), meta("lid::synthetic::Fake")];
        let impls = [edge(CANARY), edge("lid::synthetic::Fake")];
        let validations = [edge(CANARY)];
        let orphans =
            graph_orphans("lid", &specs, &impls, &validations, EdgeKind::Validations)
                .expect("canary is present in the synthetic registry");
        assert_eq!(orphans, ["lid::synthetic::Fake (synthetic.rs:1)"]);
    }

    #[test]
    #[validates(spec::GraphChecksRequireThePresentCanary)]
    fn graph_checks_require_the_present_canary() {
        let specs = [meta("lid::synthetic::Fake")];
        let result = graph_orphans("lid", &specs, &[], &[], EdgeKind::Implementations);
        assert!(
            matches!(result, Err(CanaryStripped)),
            "a stripped registry must refuse to make orphan claims"
        );
    }

    #[test]
    #[validates(spec::GraphChecksScopeToTheCurrentCrate)]
    fn graph_checks_scope_to_the_current_crate() {
        let specs = [meta(CANARY), meta("other_crate::Fake")];
        let impls = [edge(CANARY)];
        let validations = [edge(CANARY)];
        let orphans =
            graph_orphans("lid", &specs, &impls, &validations, EdgeKind::Implementations)
                .expect("canary is present in the synthetic registry");
        assert!(orphans.is_empty(), "foreign-crate specs must be ignored: {orphans:?}");
    }

    #[test]
    #[validates(spec::CoveredGraphsPassTheGraphCheck)]
    fn covered_graphs_pass_the_graph_check() {
        let specs = [meta(CANARY), meta("lid::synthetic::Fake")];
        let impls = [edge(CANARY), edge("lid::synthetic::Fake")];
        let validations = [edge(CANARY)];
        let orphans =
            graph_orphans("lid", &specs, &impls, &validations, EdgeKind::Implementations)
                .expect("canary is present in the synthetic registry");
        assert!(orphans.is_empty(), "covered specs must not be reported: {orphans:?}");
    }
}
