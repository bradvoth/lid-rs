# xtask — registry-scoped mutation and the gate's self-test

## Context and Design Philosophy

xtask carries two commands: `cargo xtask mutants`, which is check 12, and
`cargo xtask gate-selftest`, which proves every other gate catches its
violation. This document designs both.

Check 12 proves a `#[validates]` test *depends on* the code it cites: mutate
the implementation, and the test must fail. `cargo-mutants` supplies the
mutation engine; what has to be built is the narrowing that keeps it inside a
per-PR budget — for a mutant in function `F`, run only the tests validating
the specs `F` implements. That mapping is a registry join, and the registries
are obtained **from each crate's own `--lib` test binary**: `intent_graph!()`
emits an inert `registry_dump_for_tooling` test that prints the registries as
tab-separated lines when `LID_DUMP=1`, and xtask runs it per workspace crate
and parses the output into owned edge records. A crate's `#[validates]` edges
exist *only* in that crate's test binary (README [§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html) applies recursively:
they are absent from the xtask binary itself, and absent from `lid` as linked
into anything else), so the test binary is the sole honest source.

`xtask` is a thin `main` over a library, so its logic sits under
`cargo test --lib` and its doctests run — the same shape the specification
prescribes for applications ([§4.1](https://bradvoth.github.io/lid-rs/spec/gates.html) toolchain note).

The second command, `gate-selftest`, discharges HLD Goal 3 for the checks no
lib test can demonstrate: each lint-, doc-, and mutation-gate gets a fixture
crate that violates exactly one check, and the self-test asserts the gate
fails on it with the expected diagnostic. A gate that cannot be shown to
catch its violation is presumed vacuous (constraint 3).

## Mutant → test-set mapping

`cargo mutants --list --json` yields `(file, function_name)` per mutant;
edges carry `(file, item)` where `item` is `module::path::fn`. A mutant's
function is matched by `edge.file == mutant.file && edge.item` ending in
`::function_name` — file equality disambiguates same-named functions in
different modules; same-named functions in the same file share a test set, which
over-approximates safely.

- **Traced mutant**: implementation edges found → specs → validation edges →
  test filters (edge `item` minus the leading `crate::` segment, since
  libtest names are crate-relative). Run with `--cargo-test-arg=--lib` plus
  `--exact` filters.
- **Untraced mutant**: the tests validating specs implemented in the same
  file — the mutant list carries no module path, so file identity stands in
  for the enclosing module; when the file implements none, the full suite
  (README [§6.1](https://bradvoth.github.io/lid-rs/spec/traced.html)).
- Mutants are grouped by identical test set, one `cargo mutants` run per
  group. `-F` takes a regex, so the group is selected by an anchored
  alternation of its escaped mutant names. `--baseline skip` drops the
  engine's own pre-mutation suite run: the gate already ran the full suite
  earlier ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) order).

Scope: `mutation_scope = "diff"` generates `git diff <base>` (default base
`main`; CI passes `--diff-base origin/main`; `--full` overrides the scope) and
passes it via `--in-diff`. Diff interpretation is the engine's contract, not
re-implemented here.

## Gate self-test

Fixtures live in `xtask/fixtures/<name>/src/lib.rs`; the self-test synthesizes
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
| Registry access | Per-crate dump via the `intent_graph!()`-emitted test, parsed from tab-separated lines | Linking the traced crate into xtask (the original design); parsing source | Linking was chosen first and **disproven by the first real run**: `#[cfg(test)]` validation edges never link into the xtask binary, so every traced plan came back empty and zero tests ran per mutant. The owning crate's test binary is the only binary that holds its validation edges. Line format over JSON: no serializer needed in `lid`, no escaping surface. Parsing source still violates constraint 2. |
| Empty narrowed test set | Degrades to the full suite | Running the (empty) filtered set; erroring out | Running zero tests lets every mutant survive trivially — the exact vacuous-pass shape constraint 3 exists to kill. Erroring would block brownfield crates where check 11 doesn't yet gate. |
| Mutant identity | `(file, ends-with ::fn_name)` join | Qualified names from cargo-mutants (not provided); span-based matching | The JSON provides file + unqualified name; file equality plus suffix match is exact for everything but same-file same-name fns, which merge into one safe over-approximated test set. |
| Argument parsing | `std::env` loop | `clap` | Two subcommands and three flags do not justify a dependency tree (tenet 3). |
| JSON / metadata parsing | `serde_json` untyped `Value`; config via `cargo metadata` | `toml` crate for Cargo.toml; typed serde derives | One parsing dependency instead of two: `cargo metadata` re-serves `[workspace.metadata.lid]` as JSON. Untyped access keeps the surface small. |
| Selftest fixtures | Synthesized detached crates, one gate each, expected-diagnostic assertions | A single fixture crate tripping all lints; asserting only nonzero exit | One-check-one-fixture keeps a diagnosis readable and proves each gate individually; exit-code-only assertions would let the wrong gate's failure vouch for the right one. |
| Baseline handling | `--baseline skip` | Default baseline run per group | The gate runs the full suite before mutation ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) order); re-running it per group multiplies wall-clock for no information. |
| Test scope per mutant | `--test-workspace true` | Engine default (test only the mutated package) | Found by a surviving mutant: a broken `derive` in the proc-macro crate is killed only by `lid`'s tests, and the package-scoped default never runs them. Cross-crate kill paths are the norm here, not the exception. |

## Open Questions & Future Decisions

### Deferred
1. `--shard` support for large diffs (engine supports `--sharding`; not
   needed at this repo's scale).
2. Nextest as the test tool once a consumer needs it (`--test-tool` exists).

## References

- README [§4.3](https://bradvoth.github.io/lid-rs/spec/gates.html) (non-vacuity by scoped mutation), [§6.1](https://bradvoth.github.io/lid-rs/spec/traced.html) (untraced fallback).
- [`cargo-mutants` documentation](https://mutants.rs) — flags relied on:
  `--list --json`, `-F`, `--in-diff`, `--cargo-test-arg`, `--baseline`.
