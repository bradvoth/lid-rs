//! Atomic claims for `lid` itself. Derived from the LLDs under `docs/intent/`.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these
//! types exist so that citations are resolved by the compiler rather than by
//! grep.

mod citation;
mod registry;

pub use citation::{
    DerivedSpecsCarryTheirDefinitionPath, DerivedSpecsRegisterIntoSpecs,
    ImplementsCitationsRegisterEdges, MalformedCitationsFailToCompile,
    ModuleCitationsTraceByContainment, ValidatesCitationsRegisterEdges,
};
pub use registry::{
    CanaryConfirmsRegistryPresence, CanaryDetectsAStrippedRegistry,
    LinkedRegistrationsAreEnumerable,
};

#[cfg(test)]
mod tests {
    //! Pin tests: assert the exact observable registry state the citation
    //! macros must produce, established against the hand expansions and kept
    //! green across the swap to macro forms
    //! (`docs/intent/macros/lld.md § Equivalence`).

    use crate::{Edge, IMPLEMENTATIONS, SPECS, Spec, VALIDATIONS, validates};

    /// True if `slice` holds an edge joining `spec` to `item`.
    fn has_edge(slice: &[Edge], spec: &str, item: &str) -> bool {
        slice.iter().any(|e| e.spec == spec && e.item == item)
    }

    #[test]
    #[validates(crate::spec::DerivedSpecsCarryTheirDefinitionPath)]
    fn derived_specs_carry_their_definition_path() {
        assert_eq!(
            <crate::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid::spec::registry::CanaryConfirmsRegistryPresence"
        );
        assert_eq!(
            <crate::spec::MalformedCitationsFailToCompile as Spec>::NAME,
            "lid::spec::citation::MalformedCitationsFailToCompile"
        );
    }

    #[test]
    #[validates(crate::spec::DerivedSpecsRegisterIntoSpecs)]
    fn derived_specs_register_into_specs() {
        let expected = [
            "lid::spec::registry::LinkedRegistrationsAreEnumerable",
            "lid::spec::registry::CanaryConfirmsRegistryPresence",
            "lid::spec::registry::CanaryDetectsAStrippedRegistry",
            "lid::spec::citation::DerivedSpecsCarryTheirDefinitionPath",
            "lid::spec::citation::DerivedSpecsRegisterIntoSpecs",
            "lid::spec::citation::ImplementsCitationsRegisterEdges",
            "lid::spec::citation::ValidatesCitationsRegisterEdges",
            "lid::spec::citation::ModuleCitationsTraceByContainment",
            "lid::spec::citation::MalformedCitationsFailToCompile",
        ];
        for name in expected {
            assert!(
                SPECS.iter().any(|s| s.name == name),
                "spec not registered: {name}"
            );
        }
    }

    #[test]
    #[validates(crate::spec::ImplementsCitationsRegisterEdges)]
    fn implements_citations_register_edges() {
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid::canary::present"
        ));
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::spec::CanaryDetectsAStrippedRegistry as Spec>::NAME,
            "lid::canary::triple_is_present"
        ));
    }

    #[test]
    #[validates(crate::spec::ValidatesCitationsRegisterEdges)]
    fn validates_citations_register_edges() {
        assert!(has_edge(
            &VALIDATIONS,
            <crate::spec::LinkedRegistrationsAreEnumerable as Spec>::NAME,
            "lid::canary::tests::linked_registrations_are_enumerable"
        ));
        assert!(has_edge(
            &VALIDATIONS,
            <crate::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid::canary::tests::canary_confirms_registry_presence"
        ));
        assert!(has_edge(
            &VALIDATIONS,
            <crate::spec::CanaryDetectsAStrippedRegistry as Spec>::NAME,
            "lid::canary::tests::canary_detects_a_stripped_registry"
        ));
    }

    #[test]
    #[validates(crate::spec::ModuleCitationsTraceByContainment)]
    fn module_citations_trace_by_containment() {
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::spec::LinkedRegistrationsAreEnumerable as Spec>::NAME,
            "lid::registry"
        ));
    }

    #[test]
    #[validates(crate::spec::MalformedCitationsFailToCompile)]
    fn malformed_citations_fail_to_compile() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail/*.rs");
        t.pass("tests/ui/pass/*.rs");
    }
}
