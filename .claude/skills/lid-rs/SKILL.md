---
name: lid-rs
description: Operate the LID-rs methodology (compiler-enforced linked-intent development) on a Rust codebase. Consult for ALL Rust code changes in a workspace that depends on the `lid-rs` crate or has docs/intent/ — features, refactors, and bug fixes alike. Walks changes through the phase flow (LLD → claims → skeleton → failing tests → leaves → gate), enforces the dispatch/work rule, and prescribes the correct response when a gate fires.
---
<!-- Managed by `cargo lid-rs sync` from the `lid-rs` crate this project depends on.
     Do not edit: the gate's `sync --check` fails on any difference. Project-specific
     guidance belongs in AGENTS.md. -->
<!-- ANCHOR: skill -->

# Operating LID-rs

LID-rs makes the intent graph out of Rust items: claims are `#[derive(Spec)]`
unit structs, citations are `#[implements]`/`#[validates]` attributes the
compiler resolves, and the graph is enumerated at link time and gated in CI.
Your job under this skill is to produce the artifacts in the right order and
treat every gate failure as information about intent, never as an obstacle.
The full rationale lives in the project's `README.md` (LID-rs specification);
section references below point there.

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

## The phases (0–8)

**Phase 0 — Name the slice.** A user-visible operation ("user logs in"), not a
component ("auth module"). One LLD, one module boundary.

**Phase 1 — LLD** *(human-owned; you draft)*. Plain English in
`docs/intent/<slice>/lld.md`, wired into the module:
`#[doc = include_str!("../../docs/intent/<slice>/lld.md")] pub mod <slice>;`.
This layer cannot be recovered from code: rationale, rejected alternatives,
invariants that aren't type-expressible. Rust code blocks in an LLD become
live doctests through the include — write them compilable (hide setup lines
with `# `) or the doc gate fails. **STOP for review.**

**Phase 2 — Derive claims** *(you propose, human approves)*. One
`#[derive(Spec)]` unit struct per atomic claim in `src/spec/<slice>.rs`, the
EARS-shaped doc comment *is* the claim ("When X, the system shall Y"),
re-exported from `src/spec/mod.rs`:

```rust
use lid_rs::Spec;

/// When a user submits valid credentials, the authentication service shall
/// return a session scoped to that user.
#[derive(Spec)]
pub struct ValidCredentialsYieldScopedSession;
```

Names are descriptive sentences, never numbers ([§3.2](https://bradvoth.github.io/lid-rs/spec/mapping.html)) — except genuine
foreign keys, which get `#[lid_rs::spec("SOC2-CC6.1-003")]` as a doc alias.
Reject claims that are two claims; reject claims that restate the LLD without
asserting anything. **STOP for review.**

**Phase 3 — Layer-0 skeleton** *(you propose, human approves)*. The slice's
entry point and its dispatch: signatures, `#[implements(spec::…)]`
citations, `todo!()` bodies. `cargo check` must pass — that verifies the
composition type-checks before any implementation exists. Review the
signatures, not prose. **STOP for review.**

**Phase 4 — Descend breadth-first.** Each layer-0 leaf gets its own skeleton;
`cargo check`; review; descend. Stop refining when you'd trust the leaf on
sight. A branch that can't be produced from real inputs at test time means
the leaf should take its inputs as plain data arguments so the branch
becomes an ordinary unit test.

**Phase 5 — Failing-first validations.** One test per claim, as
`#[cfg(test)]` **unit tests inside the library** — never under `tests/`,
where separate binaries never link into the registry ([§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html)):

```rust
#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn wrong_password_is_rejected() { /* … */ }
```

Run them and **confirm they fail** against the `todo!()` skeleton. A test
that is green before implementation exists needs an explanation in the commit
(usually: its claim is delivered by data, which skeletons cannot defer).
**STOP for review.**

**Phase 6 — Implement leaves.** Signature pinned, claim cited, test red: the
remaining question is small and local. Implement; keep each leaf's cognitive
complexity within the threshold without restructuring tricks.

**Phase 7 — Gate, then commit.** In order, all gating (README §4.5
verbatim — a change to either copy must reach both):

```bash
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" cargo doc --no-deps
cargo test --doc
cargo test --lib
cargo package -p <crate> --allow-dirty   # published crates: tarball builds standalone
cargo lid-rs sync --check  # the skill matches the lid-rs the project depends on
cargo lid-rs mutants       # diff scope; --full / --diff-base <ref> override
```

A failed gate never has "commit anyway" as an option: fix it, prove the tool
wrong with a reproducer, or stop. A gate that prints nothing is a result to
report ("0 mutants in scope"), not silence.

Commit the slice with the phase history legible in the message.

**Phase 8 — Change.** Every change is an LLD edit, cascaded: edit the LLD →
re-derive affected claims → rename or `#[deprecated]` changed claims →
`cargo check` names every citation site to revisit. Renaming a claim
*should* break citations — that is forced re-review, not friction. Bug fixes
walk the same arrow: find where behaviour diverged from intent, decide
whether intent was wrong, unexpressed, or misimplemented, and cascade from
there. Cascade freely within one slice; pause and ask before
propagating into another slice's LLD territory.

## Mechanics reference

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
- **Spec retirement**: add `#[deprecated = "why; what replaces it"]` to the
  claim struct. Its definition site stays clean; every citation site warns,
  and `-D warnings` turns each into a named work item. Delete the struct only
  when no citations remain.
- **Untraced code** is fine for leaf helpers with no spec-governed behaviour.
  The mutation gate arbitrates empirically: a killed mutant in an untraced fn
  means it *is* participating — trace it or move it behind a traced boundary.
- **proc-macro crates** cannot carry citations (they link into no target
  binary); their implementation edges are hand-authored at the re-export site
  in the runtime crate.

## When a gate fires

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

## Where discipline slips

The gates catch structure. They cannot catch a phase quietly merged into the
next, a helper dropped into the nearest file, or a claim borrowed from another
slice — each legal, each compounding into a module nobody can trace. These
are the moments it happens; at each, run the check.

| When | Do this |
|---|---|
| A new type, function, or decision is about to appear while implementing (Phase 6/7) | Stop: it is a Phase 8 event. Edit the LLD, derive or rename the claim, then write the code. At Phase 7, confirm every backticked identifier in the LLD's shape table resolves in `src/`. |
| Reusing an earlier slice's claim looks economical (Phase 2/3) | Don't. A slice cites its own claims. A method this slice adds to an earlier slice's type is this slice's code, in this slice's module — otherwise the tests that kill its mutants sit in another slice's plan and the mutants survive. |
| The natural file for a helper is an older module (Phase 6) | Put it in this slice's module: untraced helpers belong to the slice that adds them. Report each module's untraced-function count at Phase 7; a rising count in an old module is a fallback bucket forming. |
| A library type — enum, struct, error, event stream — is about to be used inside a leaf (Phase 3, reviewing signatures; Phase 6) | Keep it at the boundary. One function translates it into domain data; interior functions take and return domain types; rendering is a second boundary. Classify strings and magic numbers the design branches on once, into an enum or a named constant. Use an enum for a closed set of decisions (its variants are claims) and a trait for a capability. Count the match sites before saying "one". |
| About to call a leaf done (Phase 6) | An arm with two statements, or an `if` followed by more statements, is dispatch and work in one function: split it, or write the claim the branch implements. The complexity threshold bounds line counts, not decisions. |
| Writing a claim (Phase 2) | One *when*, one *shall*; no parenthetical alternatives; both halves name something a test can construct or observe; an implementer for it exists in the shape table. If the slice's central mechanism has no claim, its tests will attach themselves to the wrong ones. |
| The slice is large and the red run feels like a formality (Phase 4/5; any fork directive) | Run it as its own step and paste the output. A body written in Phase 4 has lost its red run, and the commit must say so. Size is a reason to fork the red run, never to skip it. |
| A gate result looks like a tool bug (Phase 7) | Write two hypotheses and name the observation that separates them before concluding; evidence consistent with both is not proof. For check 12, run the narrowing test in the table above. |

## What no gate catches (your residual duties)

- A test citing the **wrong** claim passes every gate ([§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)). Periodically
  reconstruct the claim from the code cold and diff it against the written
  one — that differential pass is scheduled work, not good intentions.
- Shape-coincidence reuse hidden behind an `Option` parameter ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)).
- A module outgrowing its `implements_module!` citation — watch module size.
- Ceremonial claims that restate rather than assert; reject them in Phase 2.

<!-- ANCHOR_END: skill -->