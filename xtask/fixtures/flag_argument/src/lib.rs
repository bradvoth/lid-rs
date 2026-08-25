//! Check 8 fixture: two functions in a trench coat.

/// Stores a value, with a branch smuggled in as a `bool`.
pub fn store(value: u32, validate_strictly: bool) -> u32 {
    let _ = validate_strictly;
    value
}
