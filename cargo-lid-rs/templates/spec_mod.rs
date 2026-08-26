//! Atomic claims for `__LID_PACKAGE_NAME__`: one `#[derive(lid_rs::Spec)]`
//! unit struct per claim, its doc comment the claim ("When X, the system
//! shall Y"), derived from the LLD of the slice it belongs to. Each slice's
//! claims live in their own file here and are re-exported from this module,
//! so code cites `spec::ClaimName`.
