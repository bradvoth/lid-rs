# __LID_PACKAGE_NAME__ — operated under LID-rs

This package is developed with LID-rs: design intent is linked to code
through Rust items the compiler resolves, and every structural property of
that linkage is gated. The operating skill at `.claude/skills/lid-rs/SKILL.md`
is the full instruction set; the specification is published at
<https://bradvoth.github.io/lid-rs/>. This file is the summary an agent must
not work without.

## Artifacts

| Artifact | Location |
|---|---|
| HLD | `docs/intent/hld.md`, included via `#![doc = include_str!]` in `src/lib.rs` |
| LLD, per slice | `docs/intent/<slice>/lld.md`, included via `#[doc = include_str!]` on the slice's module |
| Atomic claims | `#[derive(Spec)]` unit structs in `src/spec/`; the doc comment is the claim; descriptive names, never numbered IDs |
| Code → claim | `#[implements(spec::ClaimName)]` |
| Test → claim | `#[validates(spec::ClaimName)]` on `#[cfg(test)]` unit tests inside the library — never under `tests/`, which never links into the registry |

## The eight phases (README §8)

0. Name the slice: a user-visible operation, not a component.
1. Write the LLD (human-owned; the agent drafts) — `docs/intent/<slice>/lld.md`.
2. Derive claims (the agent proposes, the human approves) — `src/spec/`.
3. Layer-0 skeleton: signatures + `#[implements]` + `todo!()`; `cargo check` passes.
4. Descend one layer, breadth-first; check and review at each layer.
5. Failing-first `#[validates]` tests — confirm red against `todo!()`.
6. Implement leaves; review is a small local semantic question.
7. Gate (below), commit the slice.
8. Change = LLD edit, cascaded via rename / `#[deprecated]` / `cargo check`.

Stop at every phase boundary for review. Work on an `lld/<slice>` branch;
phases 2–7 are run by the phase agents `sync` installs
(`.claude/agents/lid-rs-phase-N.md`), whose hooks bound their edits, run
clippy after each, and turn their final message into the `phase N:` commit
when the phase's check (`cargo lid-rs phase-check N`) passes. The main
session spawns them and reviews their commits; it does not edit code.

## The dispatch/work rule

A function either makes one flow decision (one `match` / `if-else` chain) or
does one unit of work — never both. Every branch is a decision; every
decision must exist as a claim. **A leaf with a branch in it is a requirement
nobody wrote down.** Never raise a clippy threshold to make code fit; return
to Phase 1 and write the claim, or restructure.

## The gate (README §4.5) — run before any commit, in order, all gating

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps
cargo test --doc
cargo test --lib
cargo lid-rs sync --check   # the synced skill, agents, and workflow match the lid-rs this package depends on
cargo lid-rs mutants        # diff scope; --full / --diff-base <ref> override
```

## Never

- Suppress compiler or linter warnings (`#[allow]`, threshold bumps) to make
  code pass — the firing lint is the system working.
- Put `#[validates]` tests under `tests/`.
- Parse Rust source to reconstruct the intent graph.
- Edit anything under `.claude/skills/lid-rs/`, `.claude/workflows/lid-rs.js`,
  or `.claude/agents/lid-rs-*.md` — they are written by `cargo lid-rs sync`
  from the `lid-rs` dependency; project-specific guidance goes here instead.
