//! Atomic claims for `__LID_PACKAGE_NAME__`'s crate-root slice: one
//! `#[derive(lid_rs::Spec)]` unit struct per claim, its doc comment the claim
//! ("When X, the system shall Y"), derived from `src/lld.md`. A slice that
//! earns its own `src/<slice>/` directory keeps its claims in that directory's
//! `spec.rs`; this file holds only the crate root's. Code cites
//! `spec::ClaimName`.
