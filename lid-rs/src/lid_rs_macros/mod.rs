//! Atomic claims for `lid-rs` itself. The `lid-rs-macros` slice's claims, held in its companion.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these
//! types exist so that citations are resolved by the compiler rather than by
//! grep.

pub mod spec;




#[cfg(test)]
mod tests {
    //! Pin tests: assert the exact observable registry state the citation
    //! macros must produce, established against the hand expansions and kept
    //! green across the swap to macro forms
    //! (`lid-rs-macros/src/lld.md § Equivalence`).

    use crate::{Edge, IMPLEMENTATIONS, SPECS, Spec, VALIDATIONS, validates};

    /// True if `slice` holds an edge joining `spec` to `item`.
    fn has_edge(slice: &[Edge], spec: &str, item: &str) -> bool {
        slice.iter().any(|e| e.spec == spec && e.item == item)
    }

    #[test]
    #[validates(crate::lid_rs_macros::spec::DerivedSpecsCarryTheirDefinitionPath)]
    fn derived_specs_carry_their_definition_path() {
        assert_eq!(
            <crate::registry::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid_rs::registry::spec::CanaryConfirmsRegistryPresence"
        );
        assert_eq!(
            <crate::lid_rs_macros::spec::MalformedCitationsFailToCompile as Spec>::NAME,
            "lid_rs::lid_rs_macros::spec::MalformedCitationsFailToCompile"
        );
    }

    #[test]
    #[validates(crate::lid_rs_macros::spec::DerivedSpecsRegisterIntoSpecs)]
    fn derived_specs_register_into_specs() {
        let expected = [
            "lid_rs::registry::spec::LinkedRegistrationsAreEnumerable",
            "lid_rs::registry::spec::CanaryConfirmsRegistryPresence",
            "lid_rs::registry::spec::CanaryDetectsAStrippedRegistry",
            "lid_rs::lid_rs_macros::spec::DerivedSpecsCarryTheirDefinitionPath",
            "lid_rs::lid_rs_macros::spec::DerivedSpecsRegisterIntoSpecs",
            "lid_rs::lid_rs_macros::spec::ImplementsCitationsRegisterEdges",
            "lid_rs::lid_rs_macros::spec::ValidatesCitationsRegisterEdges",
            "lid_rs::lid_rs_macros::spec::ModuleCitationsTraceByContainment",
            "lid_rs::lid_rs_macros::spec::MalformedCitationsFailToCompile",
        ];
        for name in expected {
            assert!(
                SPECS.iter().any(|s| s.name == name),
                "spec not registered: {name}"
            );
        }
    }

    #[test]
    #[validates(crate::lid_rs_macros::spec::ImplementsCitationsRegisterEdges)]
    fn implements_citations_register_edges() {
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::registry::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid_rs::registry::canary::present"
        ));
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::registry::spec::CanaryDetectsAStrippedRegistry as Spec>::NAME,
            "lid_rs::registry::canary::triple_is_present"
        ));
    }

    #[test]
    #[validates(crate::lid_rs_macros::spec::ValidatesCitationsRegisterEdges)]
    fn validates_citations_register_edges() {
        assert!(has_edge(
            &VALIDATIONS,
            <crate::registry::spec::LinkedRegistrationsAreEnumerable as Spec>::NAME,
            "lid_rs::registry::canary::tests::linked_registrations_are_enumerable"
        ));
        assert!(has_edge(
            &VALIDATIONS,
            <crate::registry::spec::CanaryConfirmsRegistryPresence as Spec>::NAME,
            "lid_rs::registry::canary::tests::canary_confirms_registry_presence"
        ));
        assert!(has_edge(
            &VALIDATIONS,
            <crate::registry::spec::CanaryDetectsAStrippedRegistry as Spec>::NAME,
            "lid_rs::registry::canary::tests::canary_detects_a_stripped_registry"
        ));
    }

    #[test]
    #[validates(crate::lid_rs_macros::spec::ModuleCitationsTraceByContainment)]
    fn module_citations_trace_by_containment() {
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::registry::spec::LinkedRegistrationsAreEnumerable as Spec>::NAME,
            "lid_rs::registry"
        ));
    }

    /// One `TestCases` over the two fixture directories is the whole proof
    /// for every compile-time claim of this slice: the five malformations,
    /// the async refusal (`fail/async_implements.rs`), the pins' argument
    /// refusal (`fail/pin_with_args.rs`), and the pins' unchanged expansion
    /// (`pass/pins.rs`, run as well as compiled). A second harness would
    /// compile every fixture twice and prove nothing this one does not
    /// (`lid-rs-macros/src/lld.md`, "What lands by hand").
    #[test]
    #[validates(
        crate::lid_rs_macros::spec::MalformedCitationsFailToCompile,
        crate::lid_rs_macros::spec::AsyncCitationsFailToCompile,
        crate::lid_rs_macros::spec::PinsExpandToTheirItemUnchanged,
        crate::lid_rs_macros::spec::PinsRefuseArguments
    )]
    fn malformed_citations_fail_to_compile() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail/*.rs");
        t.pass("tests/ui/pass/*.rs");
    }
}
