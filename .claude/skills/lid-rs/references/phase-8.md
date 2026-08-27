# Phase 8 — Change

Every change is an LLD edit, cascaded:

1. Edit the LLD (`phase-1.md`'s discipline applies).
2. Re-derive affected claims.
3. Rename or `#[deprecated]` changed claims — renaming *should* break
   citations, that is forced re-review, not friction.
4. `cargo check` names every citation site to revisit; work through them.

Bug fixes walk the same arrow: find where behaviour diverged from intent,
decide whether intent was wrong, unexpressed, or misimplemented, and cascade
from there — a bug fix is not a shortcut around Phases 1–7.

Cascade freely within one slice; pause and ask before propagating into
another slice's LLD territory.

On an already-merged slice's branch having closed, a new Phase 8 change gets
its own branch (`lld/<slice-name>-<what-changed>` or similar) and its own
phase-tagged commits, same as a fresh slice — the branch-per-LLD convention
does not exempt maintenance changes.
