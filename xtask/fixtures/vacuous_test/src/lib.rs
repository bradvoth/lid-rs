//! Check 12 fixture: the test executes `add` but asserts nothing about it,
//! so replacing the body with a constant must leave a surviving mutant.

/// Adds two numbers.
pub fn add(a: u64, b: u64) -> u64 {
    a + b
}

#[cfg(test)]
mod tests {
    //! The vacuous validation.

    #[test]
    fn calls_but_asserts_nothing() {
        let _ = super::add(2, 2);
    }
}
