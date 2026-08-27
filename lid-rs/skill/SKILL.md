---
name: lid-rs
description: Operate the LID-rs methodology (compiler-enforced linked-intent development) on a Rust codebase. Consult for ALL Rust code changes in a workspace that depends on the `lid-rs` crate or has docs/intent/ — features, refactors, and bug fixes alike. Walks changes through the phase flow (LLD → claims → skeleton → failing tests → leaves → gate), enforces the dispatch/work rule, and prescribes the correct response when a gate fires.
---
<!-- Managed by `cargo lid-rs sync` from the `lid-rs` crate this project depends on.
     Do not edit: the gate's `sync --check` fails on any difference, in any file
     under this directory. Project-specific guidance belongs in AGENTS.md. -->
<!-- ANCHOR: skill -->

# Operating LID-rs

LID-rs makes the intent graph out of Rust items: claims are `#[derive(Spec)]`
unit structs, citations are `#[implements]`/`#[validates]` attributes the
compiler resolves, and the graph is enumerated at link time and gated in CI.
Your job under this skill is to produce the artifacts in the right order and
treat every gate failure as information about intent, never as an obstacle.
The full rationale lives in the project's `README.md` (LID-rs specification);
section references below point there.

This file is a dispatcher, not the whole methodology: each phase's detail
lives in its own file under `references/`, read only when you're in that
phase. Holding every phase's detail in context at once is the failure mode
this split exists to prevent.

## Three rules above everything

1. **Stop at phase boundaries.** After each phase, present the output and
   end with at most three numbered decisions the reviewer must make;
   "continue" approves those and nothing else. A waiver is per slice —
   restate it at Phase 0 — and a directive to a fork may not waive Phase 5.
   Keep self-reviewing under a waiver.
2. **A firing lint is the system working.** Never suppress a warning, raise a
   clippy threshold, add `#[allow]`, or wildcard a match to get past the gate.
   The only sanctioned `#[allow]`s are the ones inside macro-generated
   registration blocks, which you never write by hand. `#[mutants::skip]` is
   a suppression too: only on pure I/O sequencing, only with the reason in
   the LLD's decisions table, only after asking.
3. **A leaf with a branch in it is a requirement nobody wrote down.** A
   function either makes one flow decision (one `match` or one `if/else`
   chain) or does one unit of work — never both. When a decision starts
   creeping into a leaf, the design is missing a claim: go back to Phase 1.

## Coherence pre-flight (before starting or resuming any change)

Verify the slice you're about to touch is internally coherent: the LLD
reflects the HLD, the claims in `src/spec/` trace to the current LLD, and
`cargo test --lib` is green (checks 10/11 prove claims↔tests coherence
mechanically). If docs have drifted from intent, fix the docs first, then
implement. Docs are written fresh-author: no narration of how they changed, no
meaning that needs this conversation, rejected alternatives recorded in the
LLD's Decisions & Alternatives table.

## Working state: a branch per LLD, a commit per phase

A slice's work lives on its own branch, created at Phase 0 and named for the
slice (e.g. `lld/<slice-name>`). Each phase's approval is its own commit —
`phase N: <what was approved>` — so the branch's log is the resumption point:
read it with `git log` before asking where a slice left off, rather than
tracking phase state in a file. Phase 6 has no independent stop in the walk
below, so its commit merges into Phase 7's ("Gate, then commit"); every other
phase that ends in **STOP for review** gets its own commit at approval.

This also makes the slice's eventual PR reviewable by walking the commits in
phase order — the LLD, then the claims it implies, then the skeleton, then
the tests that were red, then the implementation that turned them green —
instead of one squashed diff that hides which order things happened in.
Whether the branch is squash-merged, merged, or rebased into the default
branch afterward is the human's call at PR time; this convention only
prescribes the shape of the working branch.

## The phases (0–8)

Read the linked file when you enter that phase — not before.

- **Phase 0 — Name the slice.** A user-visible operation, not a component.
  One LLD, one module boundary. Create the branch here.
  → `references/phase-0.md`
- **Phase 1 — LLD** *(human-owned; you draft)*. **STOP for review.**
  → `references/phase-1.md`
- **Phase 2 — Derive claims** *(you propose, human approves)*. **STOP for
  review.** → `references/phase-2.md`
- **Phase 3 — Layer-0 skeleton** *(you propose, human approves)*. `cargo
  check` passes. **STOP for review.** → `references/phase-3.md`
- **Phase 4 — Descend breadth-first.** Stop refining when you'd trust the
  leaf on sight. → `references/phase-4.md`
- **Phase 5 — Failing-first validations.** Confirm red. **STOP for review.**
  → `references/phase-5.md`
- **Phase 6 — Implement leaves.** → `references/phase-6.md`
- **Phase 7 — Gate, then commit.** → `references/phase-7.md`
- **Phase 8 — Change.** Every change is an LLD edit, cascaded.
  → `references/phase-8.md`

Before ending any phase's stop, check `references/discipline.md` for the
rows tagged that phase: the gates catch structure, not a phase quietly
merged into the next, a helper dropped into the nearest file, or a claim
borrowed from another slice.

## When a gate fires

Read `references/gates.md` for what each check means and the correct
response. A failed gate never has "commit anyway" as an option: fix it,
prove the tool wrong with a reproducer, or stop. A gate that prints nothing
is a result to report ("0 mutants in scope"), not silence.

## Mechanics reference

New-project scaffolding, the crate-scoped graph checks, `implements_module!`,
method/struct citations, spec retirement via `#[deprecated]`, and
proc-macro-crate limits are in `references/mechanics.md` — read it when
setting up a new project, or when a citation or registration question comes
up in Phase 3 or 6.

## What no gate catches (your residual duties)

- A test citing the **wrong** claim passes every gate ([§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)). Periodically
  reconstruct the claim from the code cold and diff it against the written
  one — that differential pass is scheduled work, not good intentions.
- Shape-coincidence reuse hidden behind an `Option` parameter ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)).
- A module outgrowing its `implements_module!` citation — watch module size.
- Ceremonial claims that restate rather than assert; reject them in Phase 2.

<!-- ANCHOR_END: skill -->
