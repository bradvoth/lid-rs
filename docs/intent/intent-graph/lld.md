# Intent-graph checks — uncited and unvalidated claims fail the build

## Context and Design Philosophy

Checks 10 and 11 close the loop the registry opened: a claim nothing
implements, or a claim no test would notice breaking, becomes a named test
failure. The checks are ordinary unit tests over the registries — no source
parsing, no tooling beyond `cargo test --lib` — and every one of them asserts
the canary before asserting anything else, because a check over an enumeration
must first prove the enumeration exists.

`lid` ships the checks themselves, not a recipe for writing them: pure
functions over registry slices, plus an `intent_graph!()` macro that expands
to the test fns a consuming crate needs. Consumers invoke one macro in a
`#[cfg(test)]` module; `lid` invokes the same macro on itself.

## The registry is binary-global; the checks are crate-scoped

A consumer's test binary links `lid`, so its registries contain `lid`'s specs
alongside its own. `lid`'s implementation edges ride along in the library, but
its `#[validates]` edges are `#[cfg(test)]` and exist only in `lid`'s own test
binary — in a consumer's binary, upstream specs would look unvalidated and
fail an unscoped check 11 vacuously-in-reverse.

The checks therefore scope to the invoking crate: `Spec::NAME` begins with the
defining crate's name, and the macro passes `env!("CARGO_CRATE_NAME")` at
expansion. Specs defined by other crates in the same binary are ignored;
edges are not filtered, since any crate may legitimately cite any spec.

## Shape

| Item | Role |
|---|---|
| `EdgeKind` (`Implementations` / `Validations`) | Selects the edge set a check runs against — an enum, not a `bool`, because check 8 is right about flag arguments in our code too. |
| `orphaned_specs(crate_name, specs, edges) → Vec<String>` | Work leaf: own-crate specs with no edge, formatted `name (file:line)`. |
| `graph_orphans(crate_name, specs, impls, validations, kind) → Result<Vec<String>, CanaryStripped>` | Dispatch node: canary-first guard over all three slices, then one `match` on `EdgeKind` delegating to the leaf. |
| `intent_graph!()` | `macro_rules!` emitting four `#[test]` fns: `registry_is_populated`, `every_spec_has_an_implementer`, `every_spec_has_a_validation` — each calling `graph_orphans` against the real registries via `$crate` — plus the inert `registry_dump_for_tooling` test the mutation xtask reads (`docs/intent/xtask/lld.md`). |
| `CanaryStripped` | Error carried when the canary triple is absent: the registry cannot be trusted, so no orphan claim is made at all. |

Everything is parameterized over slices (the `triple_is_present` trick from
the registry slice), so every failure branch — orphaned spec, foreign-crate
spec, stripped canary — is an ordinary unit test with synthetic inputs, while
the emitted tests apply the same functions to the real registries.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Crate scoping | Filter specs by `crate_name::` prefix of `Spec::NAME` | A `crate` field in `SpecMeta` from `env!("CARGO_CRATE_NAME")` at registration; no scoping at all | `NAME` already begins with the defining crate (it is `module_path!()`-rooted), so a second field would duplicate the datum. No scoping is wrong once two traced crates link together — found by reasoning through the consumer binary, recorded in the README's [§4.2](https://bradvoth.github.io/lid-rs/spec/gates.html) scoping note. |
| Checks shipped as | Pure fns + `macro_rules!` test emitter | Hand-written per-crate tests (the README [§4.2](https://bradvoth.github.io/lid-rs/spec/gates.html) sketch); a proc-macro emitter in lid-macros | Hand-copying three tests into every consumer invites drift in the one place drift-detection lives. `macro_rules!` suffices — the expansion is three plain fns calling `$crate` paths; no parsing, no proc-macro build cost. |
| Canary-first enforcement | `graph_orphans` returns `Err(CanaryStripped)` before any orphan logic | A `canary_ok: bool` parameter; asserting the canary only in `registry_is_populated` | The bool is check 8's flag argument verbatim. Canary-only-in-one-test leaves checks 10/11 trusting an enumeration they didn't verify — each emitted test must be independently safe against a stripped registry. |
| Emitted tests | Plain `#[test]`s, uncited | Emitted tests carrying `#[validates]` of the graph claims | The claims about check *behaviour* are validated by synthetic-input tests in `lid`; emitted tests are each consumer's enforcement instance, and citing lid's specs from every consumer would scatter cross-crate edges that mean nothing to the consumer's own graph. |

## Open Questions & Future Decisions

### Deferred
1. Lint-based gate-failure demonstrations (checks 3, 6–9) need a fixture crate
   driven by `cargo clippy` — slice 4's `xtask gate-selftest`, alongside the
   mutation machinery.
2. A `#[deprecated]` spec-retirement demonstration (spec retirement path,
   README [§3](https://bradvoth.github.io/lid-rs/spec/mapping.html) table) — candidate for the same fixture set.

## References

- README [§4.2](https://bradvoth.github.io/lid-rs/spec/gates.html) (registry intersection checks), [§5.3](https://bradvoth.github.io/lid-rs/spec/registry.html) (canary).
- `docs/intent/registry/lld.md` — slice parameterization and the canary.
