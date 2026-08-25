//! Check 2 fixture: this crate's docs link to [`Nonexistent`], which rustdoc
//! must report as an unresolved link.

/// A documented item, so only the broken link fires.
pub fn documented() {}
