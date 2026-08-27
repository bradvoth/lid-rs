# Phase 4 — Descend breadth-first

Each layer-0 leaf gets its own skeleton; `cargo check`; review; descend.
Breadth-first, not depth-first: finish one layer across the whole slice
before any leaf goes a level deeper, so the shape at each depth is reviewed
together.

Stop refining when you'd trust the leaf on sight. A branch that can't be
produced from real inputs at test time means the leaf should take its inputs
as plain data arguments so the branch becomes an ordinary unit test instead
of something requiring a mock.

If the slice is large enough that this phase's red run (Phase 5) risks
feeling like a formality, say so now and plan to run it as its own step with
its output pasted — see `references/discipline.md`.

Commit as a single `phase 4: descend for <slice>` once every layer has been
reviewed and approved, even if several layers of internal review happened
along the way — the branch's commit granularity is per numbered phase, not
per layer.
