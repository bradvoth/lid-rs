//! Pure mutant-to-test-set mapping over dumped registry edges
//! (`docs/intent/xtask/lld.md § Mutant → test-set mapping`).

use lid::implements;

use crate::spec;

/// One dumped citation edge, owned: registries are obtained from each
/// crate's test-binary dump (README §5.2), not from statics linked here.
#[derive(Debug, Clone)]
pub struct EdgeRecord {
    /// The cited spec's `NAME`.
    pub spec: String,
    /// Path of the citing item.
    pub item: String,
    /// Source file of the citation site.
    pub file: String,
}

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
    impls: &[EdgeRecord],
    validations: &[EdgeRecord],
) -> TestPlan {
    let direct = specs_for_fn(file, function, impls);
    if !direct.is_empty() {
        return non_empty_or_suite(TestPlan::Traced(tests_validating(&direct, validations)));
    }
    let module = specs_in_file(file, impls);
    if module.is_empty() {
        TestPlan::FullSuite
    } else {
        non_empty_or_suite(TestPlan::ModuleFallback(tests_validating(&module, validations)))
    }
}

/// Guards against an empty narrowed test set: running zero tests would let
/// every mutant survive trivially, so nothing-reachable degrades to the full
/// suite instead.
fn non_empty_or_suite(plan: TestPlan) -> TestPlan {
    let (TestPlan::Traced(tests) | TestPlan::ModuleFallback(tests)) = &plan else {
        return plan;
    };
    if tests.is_empty() { TestPlan::FullSuite } else { plan }
}

/// Spec names cited by implementation edges whose item is `function` in
/// `file`. File equality disambiguates same-named fns in different modules.
fn specs_for_fn<'e>(file: &str, function: &str, impls: &'e [EdgeRecord]) -> Vec<&'e str> {
    let suffix = format!("::{function}");
    impls
        .iter()
        .filter(|e| e.file == file && e.item.ends_with(&suffix))
        .map(|e| e.spec.as_str())
        .collect()
}

/// Distinct spec names implemented anywhere in `file` — the module-fallback
/// spec set for untraced fns, whose module path the mutant list does not
/// carry.
fn specs_in_file<'e>(file: &str, impls: &'e [EdgeRecord]) -> Vec<&'e str> {
    impls
        .iter()
        .filter(|e| e.file == file)
        .map(|e| e.spec.as_str())
        .collect()
}

/// Sorted, deduplicated libtest filters for the tests validating `specs`:
/// each validation edge's item with its leading crate segment stripped, since
/// libtest names are crate-relative.
fn tests_validating(specs: &[&str], validations: &[EdgeRecord]) -> Vec<String> {
    let mut filters: Vec<String> = validations
        .iter()
        .filter(|e| specs.contains(&e.spec.as_str()))
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
    fn edge(spec: &str, item: &str, file: &str) -> EdgeRecord {
        EdgeRecord { spec: spec.into(), item: item.into(), file: file.into() }
    }

    /// The registry used across these tests: `f` in `a.rs` implements S1;
    /// `g` in `b.rs` implements S2; S1 has two validating tests, S2 one,
    /// S3 (implemented by `h` in `c.rs`) none.
    fn registry() -> (Vec<EdgeRecord>, Vec<EdgeRecord>) {
        let impls = vec![
            edge("xtask::spec::S1", "xtask::a::f", "xtask/src/a.rs"),
            edge("xtask::spec::S2", "xtask::b::g", "xtask/src/b.rs"),
            edge("xtask::spec::S3", "xtask::c::h", "xtask/src/c.rs"),
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
    #[validates(spec::TracedMutantsRunOnlyTheirValidatingTests)]
    fn empty_narrowed_sets_degrade_to_the_full_suite() {
        let (impls, validations) = registry();
        let plan = plan_for_mutant("xtask/src/c.rs", "h", &impls, &validations);
        assert_eq!(
            plan,
            TestPlan::FullSuite,
            "zero reachable tests must never mean zero tests run"
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
