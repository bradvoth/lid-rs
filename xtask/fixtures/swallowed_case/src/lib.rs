//! Check 6 fixture: a dispatch site swallowing future variants under `_`.

/// A closed set expected to grow.
#[non_exhaustive]
pub enum Kind {
    /// First kind.
    A,
    /// Second kind.
    B,
}

/// Labels a kind — with a wildcard that would silently absorb new variants.
pub fn label(k: &Kind) -> &'static str {
    match k {
        Kind::A => "a",
        _ => "other",
    }
}
