# Mechanics reference

- **New project**: `lid-rs` + workspace lints + `clippy.toml` thresholds ([§7](https://bradvoth.github.io/lid-rs/spec/configuration.html)),
  `[profile.test] opt-level = 0`, `docs/intent/hld.md` included from
  `lib.rs`, and the graph checks:

  ```rust
  #[cfg(test)]
  mod intent_graph {
      //! This crate's instance of the graph checks (README §4.2).
      lid_rs::intent_graph!();
  }
  ```

- **Registry is binary-global, checks are crate-scoped**: `intent_graph!()`
  scopes to the invoking crate's specs automatically; don't hand-write the
  checks.
- **Module-level tracing**: a private-helper cluster implementing one claim
  gets `lid_rs::implements_module!(crate::spec::TheClaim);` inside the module —
  containment, not per-fn ceremony. Public surfaces get per-item citations.
- **Methods** take `#[implements]` like free fns (registrations are
  body-injected). Structs and enums take it too (e.g. a
  `#[non_exhaustive]` closed-set enum implements its closed-set claim).
- **Spec retirement**: rename the claim struct and keep the old name as
  `#[deprecated = "replaced by <New>"] pub type <Old> = <New>;` beside it in
  `src/<slice>/spec.rs` — the alias must sit where the *old* path resolved, or
  it defeats its own purpose. (A crate's `src/spec.rs` holds the aliases from
  renames that predate colocation, for the same reason.) The alias registers no
  claim; every citation site warns, and the gate's `-D warnings` turns each
  into a named work item.
  Never deprecate the struct itself — a registered claim with no
  implementer fails the graph checks. Delete the alias when no citations
  remain.
- **Untraced code** is fine for leaf helpers with no spec-governed behaviour.
  The mutation gate arbitrates empirically: a killed mutant in an untraced fn
  means it *is* participating — trace it or move it behind a traced boundary.
- **proc-macro crates** cannot carry citations (they link into no target
  binary); their implementation edges are hand-authored at the re-export site
  in the runtime crate.
