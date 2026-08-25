//! Atomic claims for `lid` itself. Derived from the LLDs under `docs/intent/`.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these
//! types exist so that citations are resolved by the compiler rather than by
//! grep. During the bootstrap window (before `lid-macros` exists) the claims
//! carry hand-expanded registrations in exactly the form the macros will emit.

mod registry;

pub use registry::{
    CanaryConfirmsRegistryPresence, CanaryDetectsAStrippedRegistry,
    LinkedRegistrationsAreEnumerable,
};
