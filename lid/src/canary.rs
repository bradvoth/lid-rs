//! The registry canary: a known spec/implementation/validation triple shipped
//! unconditionally, so a stripped or silently-empty registry becomes a named
//! failure instead of a vacuous pass. Design in [`crate::registry`]'s LLD.
//!
//! Everything here is registered in the library proper — not `#[cfg(test)]` —
//! because downstream crates link `lid` compiled without `cfg(test)`, and the
//! canary must reach *their* test binaries for their registry checks to be
//! guarded.

use crate::registry::{Edge, SpecMeta};
use crate::spec;
use lid::implements;

/// The join key the canary triple is looked up by: the type name of
/// [`crate::spec::CanaryConfirmsRegistryPresence`].
const CANARY_SPEC: &str =
    <crate::spec::CanaryConfirmsRegistryPresence as ::lid::Spec>::NAME;

/// Reports whether the canary triple survived linking into this binary.
///
/// Every registry-based check asserts this first (README §5.3): if LTO,
/// `--gc-sections`, or an unusual target stripped the linker sections, checks
/// over the registries would otherwise pass trivially over nothing.
#[implements(spec::CanaryConfirmsRegistryPresence)]
pub fn present() -> bool {
    triple_is_present(&crate::SPECS, &crate::IMPLEMENTATIONS, &crate::VALIDATIONS)
}

/// Checks that each canary entry appears in its slice.
///
/// Parameterized over the slices rather than reading the real registries so
/// the stripped case — which cannot be produced at runtime from the real
/// statics — is testable with empty inputs.
#[implements(spec::CanaryDetectsAStrippedRegistry)]
pub(crate) fn triple_is_present(specs: &[SpecMeta], impls: &[Edge], validations: &[Edge]) -> bool {
    specs.iter().any(|s| s.name == CANARY_SPEC)
        && impls.iter().any(|e| e.spec == CANARY_SPEC)
        && validations.iter().any(|e| e.spec == CANARY_SPEC)
}

// The sentinel validation edge (see the registry LLD): proves the VALIDATIONS
// section survives linking in every binary that links lid. It names no
// runnable test — the runnable validation of the canary claim is the
// `#[cfg(test)]` unit test below.
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::VALIDATIONS)]
    static EDGE: ::lid::Edge = ::lid::Edge {
        spec: <crate::spec::CanaryConfirmsRegistryPresence as ::lid::Spec>::NAME,
        item: concat!(module_path!(), "::sentinel"),
        file: file!(),
        line: line!(),
    };
};

#[cfg(test)]
mod tests {
    use super::*;
    use lid::validates;

    #[test]
    #[validates(spec::LinkedRegistrationsAreEnumerable)]
    fn linked_registrations_are_enumerable() {
        assert!(lid::SPECS.iter().any(|s| s.name == CANARY_SPEC));
        assert!(lid::IMPLEMENTATIONS.iter().any(|e| e.spec == CANARY_SPEC));
        assert!(lid::VALIDATIONS.iter().any(|e| e.spec == CANARY_SPEC));
    }

    #[test]
    #[validates(spec::CanaryConfirmsRegistryPresence)]
    fn canary_confirms_registry_presence() {
        assert!(present());
    }

    #[test]
    #[validates(spec::CanaryDetectsAStrippedRegistry)]
    fn canary_detects_a_stripped_registry() {
        assert!(!triple_is_present(&[], &lid::IMPLEMENTATIONS, &lid::VALIDATIONS));
        assert!(!triple_is_present(&lid::SPECS, &[], &lid::VALIDATIONS));
        assert!(!triple_is_present(&lid::SPECS, &lid::IMPLEMENTATIONS, &[]));
    }
}
