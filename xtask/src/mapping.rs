//! Pure mutant-to-test-set mapping over registry edges
//! (`docs/intent/xtask/lld.md § Mutant → test-set mapping`).

use lid::Edge;
use lid::implements;

use crate::spec;

/// The tests to run against one mutant.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
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
    let direct = specs_for_fn(file, function, impls);
    if !direct.is_empty() {
        return TestPlan::Traced(tests_validating(&direct, validations));
    }
    let module = specs_in_file(file, impls);
    if module.is_empty() {
        TestPlan::FullSuite
    } else {
        TestPlan::ModuleFallback(tests_validating(&module, validations))
    }
}

/// Spec names cited by implementation edges whose item is `function` in
/// `file`. File equality disambiguates same-named fns in different modules.
fn specs_for_fn(file: &str, function: &str, impls: &[Edge]) -> Vec<&'static str> {
    let suffix = format!("::{function}");
    impls
        .iter()
        .filter(|e| e.file == file && e.item.ends_with(&suffix))
        .map(|e| e.spec)
        .collect()
}

/// Distinct spec names implemented anywhere in `file` — the module-fallback
/// spec set for untraced fns, whose module path the mutant list does not
/// carry.
fn specs_in_file(file: &str, impls: &[Edge]) -> Vec<&'static str> {
    impls.iter().filter(|e| e.file == file).map(|e| e.spec).collect()
}

/// Sorted, deduplicated libtest filters for the tests validating `specs`:
/// each validation edge's item with its leading crate segment stripped, since
/// libtest names are crate-relative.
fn tests_validating(specs: &[&str], validations: &[Edge]) -> Vec<String> {
    let mut filters: Vec<String> = validations
        .iter()
        .filter(|e| specs.contains(&e.spec))
        .filter_map(|e| e.item.split_once("::").map(|(_, rest)| rest.to_string()))
        .collect();
    filters.sort();
    filters.dedup();
    filters
}

#[cfg(test)]
mod tests {
    use super::*;
    use lid::validates;

    /// Synthetic edge builder.
    fn edge(spec: &'static str, item: &'static str, file: &'static str) -> Edge {
        Edge { spec, item, file, line: 1 }
    }

    /// The registry used across these tests: `f` in `a.rs` implements S1;
    /// `g` in `b.rs` implements S2; S1 has two validating tests, S2 one.
    fn registry() -> (Vec<Edge>, Vec<Edge>) {
        let impls = vec![
            edge("xtask::spec::S1", "xtask::a::f", "xtask/src/a.rs"),
            edge("xtask::spec::S2", "xtask::b::g", "xtask/src/b.rs"),
        ];
        let validations = vec![
            edge("xtask::spec::S1", "xtask::a::tests::t1", "xtask/src/a.rs"),
            edge("xtask::spec::S1", "xtask::a::tests::t2", "xtask/src/a.rs"),
            edge("xtask::spec::S2", "xtask::b::tests::t3", "xtask/src/b.rs"),
        ];
        (impls, validations)
    }

    #[test]
    #[validates(spec::TracedMutantsRunOnlyTheirValidatingTests)]
    fn traced_mutants_run_only_their_validating_tests() {
        let (impls, validations) = registry();
        let plan = plan_for_mutant("xtask/src/a.rs", "f", &impls, &validations);
        assert_eq!(
            plan,
            TestPlan::Traced(vec![
                "a::tests::t1".to_string(),
                "a::tests::t2".to_string(),
            ])
        );
    }

    #[test]
    #[validates(spec::UntracedMutantsFallBackToModuleTests)]
    fn untraced_mutants_fall_back_to_module_tests() {
        let (impls, validations) = registry();
        let fallback = plan_for_mutant("xtask/src/a.rs", "helper", &impls, &validations);
        assert_eq!(
            fallback,
            TestPlan::ModuleFallback(vec![
                "a::tests::t1".to_string(),
                "a::tests::t2".to_string(),
            ])
        );
        let suite = plan_for_mutant("xtask/src/nowhere.rs", "helper", &impls, &validations);
        assert_eq!(suite, TestPlan::FullSuite);
    }
}
