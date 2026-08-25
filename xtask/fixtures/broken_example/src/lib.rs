//! Check 5 fixture: a public example that no longer reflects the API.

/// The answer.
///
/// ```
/// assert_eq!(broken_example::answer(), 5);
/// ```
pub fn answer() -> u64 {
    4
}
