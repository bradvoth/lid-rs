//! Claims for the registry slice. Derived from `docs/intent/registry/lld.md`.

/// When a binary links a crate containing registration statics, iterating
/// [`crate::SPECS`], [`crate::IMPLEMENTATIONS`], and [`crate::VALIDATIONS`]
/// shall yield those registrations.
pub struct LinkedRegistrationsAreEnumerable;

// Hand-expansion of `#[derive(Spec)]` (bootstrap window; the lid-macros
// derive must reproduce this form exactly).
impl ::lid::Spec for LinkedRegistrationsAreEnumerable {
    const NAME: &'static str = concat!(module_path!(), "::", stringify!(LinkedRegistrationsAreEnumerable));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <LinkedRegistrationsAreEnumerable as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When the canary spec, implementation edge, and validation edge are all
/// enumerable in the registries, the canary's presence check shall report
/// `true`.
pub struct CanaryConfirmsRegistryPresence;

impl ::lid::Spec for CanaryConfirmsRegistryPresence {
    const NAME: &'static str = concat!(module_path!(), "::", stringify!(CanaryConfirmsRegistryPresence));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <CanaryConfirmsRegistryPresence as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};

/// When any entry of the canary triple is missing from its registry, the
/// canary's presence check shall report `false`.
pub struct CanaryDetectsAStrippedRegistry;

impl ::lid::Spec for CanaryDetectsAStrippedRegistry {
    const NAME: &'static str = concat!(module_path!(), "::", stringify!(CanaryDetectsAStrippedRegistry));
}
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::SPECS)]
    static META: ::lid::SpecMeta = ::lid::SpecMeta {
        name: <CanaryDetectsAStrippedRegistry as ::lid::Spec>::NAME,
        file: file!(),
        line: line!(),
    };
};
