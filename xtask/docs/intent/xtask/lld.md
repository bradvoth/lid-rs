# xtask — the gate's self-test

## Context and Design Philosophy

xtask carries one command: `cargo xtask gate-selftest`, which proves every
gate catches its violation. Check 12's orchestration once lived here too; it
now ships as `cargo-lid-rs` (`cargo-lid-rs/docs/intent/cargo-lid-rs/lld.md`),
because a consumer must be able to run it and a `publish = false` task crate
can only serve this repository. What remains is evidence, not tooling: the
fixtures are this repository's demonstration that its gates gate, which is
HLD Goal 3 for the checks no lib test can demonstrate.

`xtask` is a thin `main` over a library, so its logic sits under
`cargo test --lib` and its doctests run — the same shape the specification
prescribes for applications ([§4.1](https://bradvoth.github.io/lid-rs/spec/gates.html) toolchain note). It depends on
`lid-rs` as an ordinary consumer and carries its own claims, citations, and
`intent_graph!()` instance; it is the in-workspace downstream-consumer proof,
alongside the published one.

## Gate self-test

Fixtures live in `xtask/fixtures/<name>/src/lib.rs`; the self-test synthesizes
a detached crate per fixture under `target/gate-selftest/<name>` (own
`[workspace]` table, the repo's `clippy.toml`, the workspace lint set inlined,
`lid-rs` as a path dependency where cited) and runs one gate command against it,
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

Checks 1, 10, 11 are already demonstrated inside `lid-rs` (UI compile-fail
tests; synthetic-registry tests) and are not duplicated here. The
`vacuous_test` fixture invokes the mutation engine directly on the fixture
crate: it demonstrates the engine's verdict on a vacuous test, which is the
property check 12 rests on, independently of `cargo-lid-rs`'s narrowing.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Selftest fixtures | Synthesized detached crates, one gate each, expected-diagnostic assertions | A single fixture crate tripping all lints; asserting only nonzero exit | One-check-one-fixture keeps a diagnosis readable and proves each gate individually; exit-code-only assertions would let the wrong gate's failure vouch for the right one. |
| Workspace location | `CARGO_MANIFEST_DIR`'s parent | `cargo metadata` | The task crate only ever runs from this repository's checkout, and the fixtures it synthesizes are addressed relative to it; the metadata round-trip buys nothing here. (The published tool makes the opposite choice for the opposite reason.) |
| Argument parsing | `std::env` loop | `clap` | One subcommand does not justify a dependency tree (tenet 3). |
| Cargo invocation | A three-line `cargo_command` helper of its own | Depending on `cargo-lid-rs`'s library for the shared helper | A dependency edge from the repository's evidence crate to its shipped tool, for `Command::new($CARGO)`, would couple the self-test's build to the tool's; the helper is an untraced leaf either way. |

## Open Questions & Future Decisions

### Deferred
1. A fixture per future check, as checks are added — the table above is the
   contract that every gate has a demonstrated failure.

## References

- HLD Goal 3 (every check has a demonstrated failure) and constraint 3 (a
  gate with no demonstrated failure is presumed vacuous).
- `cargo-lid-rs/docs/intent/cargo-lid-rs/lld.md` — where check 12's
  orchestration now lives.
