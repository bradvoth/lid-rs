# Phase 2 — Derive claims (you propose, human approves)

One `#[derive(Spec)]` unit struct per atomic claim in `src/<slice>/spec.rs`,
beside the slice's `lld.md`; the EARS-shaped doc comment *is* the claim ("When
X, the system shall Y"). Nothing re-exports it — a claim's path names the slice
that owns it, so another slice cites it as `<slice>::spec::ClaimName`:

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

If the slice's directory is new, nothing declares `spec.rs` yet — Phase 3
writes the `pub mod spec;`. Until then the claims file **is not compiled**, so
the derive never reads it and the controlled language is not enforced on it.
This phase's check passes either way. Hold the language by hand and say so in
the commit.

**STOP for review.** Commit as `phase 2: claims for <slice>` once approved.
