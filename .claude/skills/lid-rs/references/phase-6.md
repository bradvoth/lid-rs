# Phase 6 — Implement leaves

Signature pinned, claim cited, test red: the remaining question is small and
local. Implement; keep each leaf's cognitive complexity within the threshold
without restructuring tricks to dodge it.

Before calling a leaf done: an arm with two statements, or an `if` followed
by more statements, is dispatch and work in one function — split it, or
write the claim the branch implements. The complexity threshold bounds line
counts, not decisions (main `SKILL.md`, rule 3).

If a new type, function, or decision wants to appear while you're here, stop
— that's a Phase 8 event, not a Phase 6 one. See `references/discipline.md`
and `phase-8.md`.

This phase has no independent stop in the walk: its commit merges into
Phase 7's (`phase 7: gate and implementation for <slice>`) once the gate
passes.
