# Phase 3 — Layer-0 skeleton (you propose, human approves)

The slice's entry point and its dispatch: signatures, `#[implements(spec::…)]`
citations, `todo!()` bodies. `cargo check` must pass — that verifies the
composition type-checks before any implementation exists.

Review the signatures, not prose: the reviewer is deciding whether this
shape is right before a line of it is filled in.

A library type — enum, struct, error, event stream — entering a leaf's
signature here is a decision: keep it at the boundary (one function
translates it into domain data; interior functions take and return domain
types) rather than threading it through. See `references/discipline.md` for
the full boundary rule and its per-shape corollaries.

Check `references/discipline.md` for this phase's rows (claim reuse, library
types at the boundary) before stopping.

**STOP for review.** Commit as `phase 3: skeleton for <slice>` once approved.
