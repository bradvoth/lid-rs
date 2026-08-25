//! Pure mutant-to-test-set mapping over registry edges
//! (`docs/intent/xtask/lld.md § Mutant → test-set mapping`).

use lid::Edge;
use lid::implements;

use crate::spec;

/// The tests to run against one mutant.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TestPlan {
    /// The mutated fn is traced: exactly the tests validating its specs.
    Traced(Vec<String>),
    /// Untraced fn: tests validating specs implemented in the same file.
    ModuleFallback(Vec<String>),
    /// Nothing narrower exists: run the whole suite.
    FullSuite,
}

/// Chooses the test plan for a mutant identified by `(file, function)`.
#[implements(
    spec::TracedMutantsRunOnlyTheirValidatingTests,
    spec::UntracedMutantsFallBackToModuleTests,
)]
pub fn plan_for_mutant(
    file: &str,
    function: &str,
    impls: &[Edge],
    validations: &[Edge],
) -> TestPlan {
    let _ = (file, function, impls, validations);
    todo!()
}

/// Spec names cited by implementation edges whose item is `function` in
/// `file`. File equality disambiguates same-named fns in different modules.
fn specs_for_fn(file: &str, function: &str, impls: &[Edge]) -> Vec<&'static str> {
    let _ = (file, function, impls);
    todo!()
}

/// Distinct spec names implemented anywhere in `file` — the module-fallback
/// spec set for untraced fns, whose module path the mutant list does not
/// carry.
fn specs_in_file(file: &str, impls: &[Edge]) -> Vec<&'static str> {
    let _ = (file, impls);
    todo!()
}

/// Sorted, deduplicated libtest filters for the tests validating `specs`:
/// each validation edge's item with its leading crate segment stripped, since
/// libtest names are crate-relative.
fn tests_validating(specs: &[&str], validations: &[Edge]) -> Vec<String> {
    let _ = (specs, validations);
    todo!()
}
