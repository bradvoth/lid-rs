//! Atomic claims for `lid` itself. Derived from the LLDs under `docs/intent/`.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these
//! types exist so that citations are resolved by the compiler rather than by
//! grep. During the bootstrap window (before `lid-macros` exists) the claims
//! carry hand-expanded registrations in exactly the form the macros will emit.

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
    //! Pin tests: assert the exact observable registry state produced by the
    //! hand expansions, so that swapping in the macro forms is equivalence-
    //! checked field by field (`docs/intent/macros/lld.md § Equivalence`).

    use crate::{Edge, IMPLEMENTATIONS, SPECS, Spec, VALIDATIONS};

    /// True if `slice` holds an edge joining `spec` to `item`.
    fn has_edge(slice: &[Edge], spec: &str, item: &str) -> bool {
        slice.iter().any(|e| e.spec == spec && e.item == item)
    }

    // Hand-expansion of:
    //   #[validates(spec::DerivedSpecsCarryTheirDefinitionPath)]
    #[test]
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
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::DerivedSpecsCarryTheirDefinitionPath as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::derived_specs_carry_their_definition_path"),
            file: file!(),
            line: line!(),
        };
    };

    // Hand-expansion of:
    //   #[validates(spec::DerivedSpecsRegisterIntoSpecs)]
    #[test]
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
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::DerivedSpecsRegisterIntoSpecs as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::derived_specs_register_into_specs"),
            file: file!(),
            line: line!(),
        };
    };

    // Hand-expansion of:
    //   #[validates(spec::ImplementsCitationsRegisterEdges)]
    #[test]
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
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::ImplementsCitationsRegisterEdges as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::implements_citations_register_edges"),
            file: file!(),
            line: line!(),
        };
    };

    // Hand-expansion of:
    //   #[validates(spec::ValidatesCitationsRegisterEdges)]
    #[test]
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
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::ValidatesCitationsRegisterEdges as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::validates_citations_register_edges"),
            file: file!(),
            line: line!(),
        };
    };

    // Hand-expansion of:
    //   #[validates(spec::ModuleCitationsTraceByContainment)]
    #[test]
    fn module_citations_trace_by_containment() {
        assert!(has_edge(
            &IMPLEMENTATIONS,
            <crate::spec::LinkedRegistrationsAreEnumerable as Spec>::NAME,
            "lid::registry"
        ));
    }
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::ModuleCitationsTraceByContainment as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::module_citations_trace_by_containment"),
            file: file!(),
            line: line!(),
        };
    };

    // Hand-expansion of:
    //   #[validates(spec::MalformedCitationsFailToCompile)]
    #[test]
    fn malformed_citations_fail_to_compile() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail/*.rs");
        t.pass("tests/ui/pass/*.rs");
    }
    const _: () = {
        #[allow(missing_docs, clippy::missing_docs_in_private_items)]
        #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
        static EDGE: ::lid::Edge = ::lid::Edge {
            spec: <crate::spec::MalformedCitationsFailToCompile as ::lid::Spec>::NAME,
            item: concat!(module_path!(), "::malformed_citations_fail_to_compile"),
            file: file!(),
            line: line!(),
        };
    };
}
