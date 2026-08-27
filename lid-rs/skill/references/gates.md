# When a gate fires

| Gate | It means | Correct response |
|---|---|---|
| check 1 — citation fails to resolve | The claim was renamed/deleted, or the path is wrong | Revisit the citation site against the current spec module; this is forced re-review, not breakage |
| check 2/5 — doc link or example broken | Docs/LLD drifted from the API | Fix the doc or the LLD — they are intent, not decoration |
| check 3 — missing docs | An item exists with no stated intent | Write the intent; if you can't state it, the item shouldn't exist yet |
| check 4 — skeleton doesn't type-check | The layer you designed doesn't fit together | Fix the design at this layer before descending — catching it here is why skeletons come first |
| check 6 — non-exhaustive match | An upstream case was added and a dispatch site would swallow it | Handle the new case; never `_ =>` it away |
| check 7 — cognitive complexity | A leaf contains decisions nobody declared | **Return to Phase 1.** Write the claim (making the branch a declared dispatch), or restructure. Never raise the threshold |
| check 8 — bool parameter | Two functions in a trench coat | Split into two leaves; share common structure in a wrapper, never via a flag. The `Option`-parameter variant is your judgment — nothing catches it |
| check 9 — too many lines | An unnamed sub-thought inlined | Extract and name it |
| check 10/11 — uncited/unvalidated spec | The design says it; nothing does/would-notice it | Implement or validate the claim — or if the claim is wrong, cascade its removal from the LLD down |
| check 12 — surviving mutant | A test executes the code but asserts nothing about it — or the test that *would* kill it cites a different claim, so the narrowed test set never ran it | Strengthen the test to assert the claim's observable behaviour, or fix the citation: a killing test must cite the claim the mutated function implements, because narrowing follows citations. A survivor in a `match` arm of a cited function has two fixes — write the claim the arm implements, or delete the arm; moving it to an untraced function is suppression. If the intended test kills the mutant by hand, suspect the engine's narrowing before the planner: `cargo mutants --list -F '<the group's regex>'` must list exactly the group; if it lists more, the engine ignored the filter for that mutant kind — report it with that output |
| canary failure | The registry itself is untrustworthy (stripped sections) | Fix the build/linker configuration first; no other registry result means anything |
