# xtask — registry-scoped mutation and the gate's self-test

## Context and Design Philosophy

Check 12 proves a `#[validates]` test *depends on* the code it cites: mutate
the implementation, and the test must fail. `cargo-mutants` supplies the
mutation engine; what has to be built is the narrowing that keeps it inside a
per-PR budget — for a mutant in function `F`, run only the tests validating
the specs `F` implements. That mapping is exactly a registry join, so `xtask`
reads the registries the same way everything else does: **by linking**. The
xtask library depends on the traced crate, and the linked-in `SPECS` /
`IMPLEMENTATIONS` / `VALIDATIONS` are the target's own graph — no subprocess
dump, no parsing, constraint 2 intact. (In this workspace the traced crate is
`lid`; a consumer workspace's xtask depends on its own traced crates the same
way.)

`xtask` is a thin `main` over a library, so its logic sits under
`cargo test --lib` and its doctests run — the same shape the specification
prescribes for applications (§4.1 toolchain note).

The second command, `gate-selftest`, discharges HLD Goal 3 for the checks no
lib test can demonstrate: each lint-, doc-, and mutation-gate gets a fixture
crate that violates exactly one check, and the selftest asserts the gate
fails on it with the expected diagnostic. A gate that cannot be shown to
catch its violation is presumed vacuous (constraint 3).

## Mutant → test-set mapping

`cargo mutants --list --json` yields `(file, function_name)` per mutant;
edges carry `(file, item)` where `item` is `module::path::fn`. A mutant's
function is matched by `edge.file == mutant.file && edge.item` ending in
`::function_name` — file equality disambiguates same-named fns in different
modules; same-named fns in the same file share a test set, which
over-approximates safely.

- **Traced mutant**: implementation edges found → specs → validation edges →
  test filters (edge `item` minus the leading `crate::` segment, since
  libtest names are crate-relative). Run with `--cargo-test-arg=--lib` plus
  `--exact` filters.
- **Untraced mutant**: per `untraced_fallback = "module"`, the tests
  validating specs implemented by the enclosing module (edges whose item
  shares the module prefix); when none exist, the full suite (README §6.2).
- Mutants are grouped by identical test set; one `cargo mutants` run per
  group, selected by an anchored, escaped `-F` alternation of mutant names,
  `--baseline skip` (the ungated suite already ran earlier in the gate).

Scope: `mutation_scope = "diff"` generates `git diff <base>` (default base
`origin/main`, override `--diff-base`; `--full` overrides the scope) and
passes it via `--in-diff`. Diff interpretation is the engine's contract, not
re-implemented here.

## Gate self-test

Fixtures live in `xtask/fixtures/<name>/src/lib.rs`; the selftest synthesizes
a detached crate per fixture under `target/gate-selftest/<name>` (own
`[workspace]` table, the repo's `clippy.toml`, the workspace lint set inlined,
`lid` as a path dependency where cited) and runs one gate command against it,
asserting failure with the check's signature diagnostic:

| Fixture | Gate | Demonstrates |
|---|---|---|
| `broken_doc_link` | `cargo doc` + `-D rustdoc::broken_intra_doc_links` | check 2 |
| `missing_docs` | clippy `-D warnings` | check 3 |
| `skeleton_incoherence` | `cargo check` | check 4 (E0308 across composed `todo!()` signatures) |
| `broken_example` | `cargo test --doc` | check 5 |
| `swallowed_case` | clippy `-D warnings` | check 6 (`wildcard_enum_match_arm`) |
| `undeclared_decision` | clippy `-D warnings` | check 7 (`cognitive_complexity`) |
| `flag_argument` | clippy `-D warnings` | check 8 (`fn_params_excessive_bools`) |
| `inlined_concept` | clippy `-D warnings` | check 9 (`too_many_lines`) |
| `retired_spec` | clippy `-D warnings` | spec retirement: citing a `#[deprecated]` spec warns at the citation site |
| `vacuous_test` | `cargo mutants` | check 12: a test that executes but asserts nothing leaves a surviving mutant |

Checks 1, 10, 11 are already demonstrated inside `lid` (UI compile-fail
tests; synthetic-registry tests) and are not duplicated here.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Registry access | Link the traced crate into xtask | Dump via a hidden test binary; parse source | Linking is the mechanism the whole system already trusts; a dump adds a subprocess protocol that can silently skew; parsing violates constraint 2. Also makes xtask the first true downstream consumer, settling the deferred linkme-path question with evidence. |
| Mutant identity | `(file, ends-with ::fn_name)` join | Qualified names from cargo-mutants (not provided); span-based matching | The JSON provides file + unqualified name; file equality plus suffix match is exact for everything but same-file same-name fns, which merge into one safe over-approximated test set. |
| Argument parsing | `std::env` loop | `clap` | Two subcommands and three flags do not justify a dependency tree (tenet 3). |
| JSON / metadata parsing | `serde_json` untyped `Value`; config via `cargo metadata` | `toml` crate for Cargo.toml; typed serde derives | One parsing dependency instead of two: `cargo metadata` re-serves `[workspace.metadata.lid]` as JSON. Untyped access keeps the surface small. |
| Selftest fixtures | Synthesized detached crates, one gate each, expected-diagnostic assertions | A single fixture crate tripping all lints; asserting only nonzero exit | One-check-one-fixture keeps a diagnosis readable and proves each gate individually; exit-code-only assertions would let the wrong gate's failure vouch for the right one. |
| Baseline handling | `--baseline skip` | Default baseline run per group | The gate runs the full suite before mutation (§4.5 order); re-running it per group multiplies wall-clock for no information. |

## Open Questions & Future Decisions

### Deferred
1. `--shard` support for large diffs (engine supports `--sharding`; not
   needed at this repo's scale).
2. Nextest as the test tool once a consumer needs it (`--test-tool` exists).

## References

- README r4 §4.3 (non-vacuity by scoped mutation), §6.2 (untraced fallback).
- [`cargo-mutants` documentation](https://mutants.rs) — flags relied on:
  `--list --json`, `-F`, `--in-diff`, `--cargo-test-arg`, `--baseline`.
