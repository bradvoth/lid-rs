//! Claims for the citation-macro slice. Derived from
//! `docs/intent/macros/lld.md`.
//!
//! The implementing code lives in `lid-macros`, a proc-macro crate that links
//! into no target binary and therefore cannot carry citations itself; its
//! implementation edges are hand-authored at the re-export site in `lid`'s
//! crate root (the standing exception for proc-macro crates).

/// When a unit struct derives `Spec`, its `NAME` shall be the definition-site
/// module path joined with the struct identifier.
pub struct DerivedSpecsCarryTheirDefinitionPath;

// Hand-expansion of `#[derive(Spec)]` (bootstrap window; swapped for the
// macro form when lid-macros is implemented).
impl ::lid::Spec for DerivedSpecsCarryTheirDefinitionPath {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(DerivedSpecsCarryTheirDefinitionPath));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <DerivedSpecsCarryTheirDefinitionPath as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When a unit struct derives `Spec`, a `SpecMeta` registration for it shall
/// appear in [`crate::SPECS`].
pub struct DerivedSpecsRegisterIntoSpecs;

impl ::lid::Spec for DerivedSpecsRegisterIntoSpecs {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(DerivedSpecsRegisterIntoSpecs));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <DerivedSpecsRegisterIntoSpecs as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When an item carries `#[implements(...)]`, one [`crate::IMPLEMENTATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing item.
pub struct ImplementsCitationsRegisterEdges;

impl ::lid::Spec for ImplementsCitationsRegisterEdges {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(ImplementsCitationsRegisterEdges));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <ImplementsCitationsRegisterEdges as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When a test fn carries `#[validates(...)]`, one [`crate::VALIDATIONS`]
/// edge per cited spec shall be registered, keyed by the cited spec's `NAME`
/// and naming the citing test.
pub struct ValidatesCitationsRegisterEdges;

impl ::lid::Spec for ValidatesCitationsRegisterEdges {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(ValidatesCitationsRegisterEdges));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <ValidatesCitationsRegisterEdges as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When `implements_module!` is invoked inside a module, the registered
/// edge's item shall be the enclosing module path.
pub struct ModuleCitationsTraceByContainment;

impl ::lid::Spec for ModuleCitationsTraceByContainment {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(ModuleCitationsTraceByContainment));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <ModuleCitationsTraceByContainment as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When a citation is malformed — an unresolvable path, a type that does not
/// implement `Spec`, an empty citation list, a generic path, or a derive
/// target that is not a unit struct — compilation shall fail.
pub struct MalformedCitationsFailToCompile;

impl ::lid::Spec for MalformedCitationsFailToCompile {
    const NAME: &'static str =
        concat!(module_path!(), "::", stringify!(MalformedCitationsFailToCompile));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <MalformedCitationsFailToCompile as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};
