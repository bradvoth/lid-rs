# Phase 5 — Failing-first validations

One test per claim, as `#[cfg(test)]` **unit tests inside the library** —
never under `tests/`, where separate binaries never link into the registry
([§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html)):

```rust
#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn wrong_password_is_rejected() { /* … */ }
```

Run them and **confirm they fail** against the `todo!()` skeleton. This
confirmation is not optional and not a formality, even under a "continue
through implementation" waiver from a previous phase — a waiver may not
cover this phase (main `SKILL.md`, Three rules). The `phase 5:` commit's
hook runs the same confirmation (`cargo lid-rs phase-check 5`: every claim
in `src/spec/<slice>.rs` has a `#[validates]` test and every one fails) and
refuses the commit otherwise; paste your own run's output anyway, so the
commit body records what was red. A test that is green before
implementation exists needs an explanation in the commit (usually: its claim
is delivered by data, which skeletons cannot defer).

For a large slice, run the red confirmation as its own step and paste its
output — see `references/discipline.md`. A body written during Phase 4 that
lost its own red run must say so in the commit, not be silently treated as
covered.

**STOP for review.** Commit as `phase 5: failing tests (red) for <slice>`
once approved.
