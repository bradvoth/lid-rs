# LID-rs — self-hosted workspace

This workspace builds the LID-rs toolchain (`lid`, `lid-macros`, `xtask`, and the
operating skill) **using the LID-rs methodology on itself**. `README.md` is the
methodology's living specification; `docs/intent/hld.md` is this workspace's
implementation design. When implementation reveals a flaw in the README, the
README is revised (revision bump when substantive), not silently diverged from.

## LID Mode: Full

## LID artifact conventions (override skill defaults)

This repo uses LID-rs conventions from README §3/§11, **not** the classic-LID
paths the `linked-intent-dev` skill defaults to:

| Artifact | Location here | Not |
|---|---|---|
| HLD | `docs/intent/hld.md` (included via `#![doc = include_str!]` in `lid/src/lib.rs`) | `docs/high-level-design.md` |
| LLD, per slice | `docs/intent/<slice>/lld.md` (included via `#[doc = include_str!]` on the module) | `docs/llds/` |
| Atomic claims | `#[derive(Spec)]` unit structs in `src/spec/`, doc comment is the claim, **descriptive names not numbered IDs** | `docs/specs/` EARS files |
| Code → claim link | `#[implements(spec::ClaimName)]` | `// @spec AUTH-001` comments |
| Test → claim link | `#[validates(spec::ClaimName)]` on `#[cfg(test)]` unit tests in the lib (never under `tests/`) | `@spec` comments |

The skill's *process* discipline applies unchanged: phase stops, cascade
discipline, coherence pre-flight, context-free docs. Only the artifact formats
differ. Bootstrap window: until `lid-macros` compiles, claims are plain
doc-commented unit structs and one hand-expanded spec/impl/validation triple
(the future canary) stands in for macro output.

## The eight phases (README §8)

0. Name the slice (a user-visible operation, not a component)
1. Write the LLD (human-owned; agent drafts) — `docs/intent/<slice>/lld.md`
2. Derive claims (agent proposes, human approves) — `src/spec/`
3. Layer-0 skeleton: signatures + `#[implements]` + `todo!()`; `cargo check` passes
4. Descend one layer, breadth-first; check and review at each layer
5. Failing-first `#[validates]` tests — confirm red against `todo!()`
6. Implement leaves (agent; review is a small local semantic question)
7. Gate (below), commit the slice
8. Change = LLD edit, cascaded via rename/`#[deprecated]`/`cargo check`

## The dispatch/work rule

A function either makes one flow decision (one `match` / `if-else` chain) or
does one unit of work — never both. Every branch is a decision; every decision
must exist as a claim. **A leaf with a branch in it is a requirement nobody
wrote down.** Do not raise clippy thresholds to make code fit; return to
Phase 1 and write the claim, or restructure.

## The gate (README §4.5) — run before any commit

A gate that exists, gates: run whatever subset currently applies, from the
first commit onward.

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps
cargo test --doc
cargo test --lib
cargo xtask mutants                  # diff scope; --full / --diff-base override
mdbook build book                    # the site is assembled by inclusion; breaks on drift
```

## Tenets (order wins on conflict; full statements in the HLD)

1. **The spec follows reality it failed to predict.** Divergence between README
   and implementation is a doc bug first.
2. **A gate that exists, gates.** No development window where the repo would
   fail its own methodology.
3. **Constrained-first dependencies.** Core deps are `syn`/`quote`/`linkme`;
   escalate only with evidence the constrained option fails.

## Never

- Suppress compiler or linter warnings (`#[allow]`, threshold bumps) to make
  code pass — the firing lint is the system working. (Exception: the specific
  `#[allow]`s inside macro-generated registration statics, per README §5.1.)
- Put `#[validates]` tests under `tests/` — separate binaries never link into
  the registry (README §5.2).
- Parse Rust source to reconstruct the graph (README constraint 2).
