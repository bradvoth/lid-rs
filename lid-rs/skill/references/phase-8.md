# Phase 8 — Change

Every change is an LLD edit, cascaded:

1. Edit the LLD (`phase-1.md`'s discipline applies).
2. Re-derive affected claims.
3. Rename changed claims, keeping each old name as
   `#[deprecated = "replaced by <New>"] pub type <Old> = <New>;` beside it in
   `src/<slice>/spec.rs`, where the old path resolved — every citation of the
   old name *should* start
   warning, that is forced re-review, not friction. Never `#[deprecated]`
   on the struct itself: once its citations move it is a registered claim
   with no implementer, and only Phase 2 may delete it.
4. `cargo check` names every citation site to revisit. Phase 2's check
   leaves them warning; Phases 3 and 4 work through them; Phase 7's gate
   denies any that remain. Delete an alias no citation names at the next
   Phase 2 on the slice.

Bug fixes walk the same arrow: find where behaviour diverged from intent,
decide whether intent was wrong, unexpressed, or misimplemented, and cascade
from there — a bug fix is not a shortcut around Phases 1–7.

Cascade freely within one slice; pause and ask before propagating into
another slice's LLD territory.

On an already-merged slice's branch having closed, a new Phase 8 change gets
its own branch (`lld/<slice-name>-<what-changed>` or similar) and its own
phase-tagged commits, same as a fresh slice — the branch-per-LLD convention
does not exempt maintenance changes.
