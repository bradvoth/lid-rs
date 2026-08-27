# Phase 2 — Derive claims (you propose, human approves)

One `#[derive(Spec)]` unit struct per atomic claim in `src/spec/<slice>.rs`,
the EARS-shaped doc comment *is* the claim ("When X, the system shall Y"),
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

Checklist per claim, before proposing it:

- One *when*, one *shall* — no parenthetical alternatives.
- Both halves name something a test can construct or observe.
- An implementer for it exists in the LLD's shape table.
- It asserts something; a claim that only restates the LLD is rejected.
- It is one claim, not two — a claim needing "and" to state usually is.

If the slice's central mechanism has no claim, its tests will end up
attached to the wrong ones — check that before moving on.

Check `references/discipline.md` for this phase's rows (claim reuse across
slices) before stopping.

**STOP for review.** Commit as `phase 2: claims for <slice>` once approved.
