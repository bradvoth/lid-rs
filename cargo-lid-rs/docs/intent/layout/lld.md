# A slice's artifacts live in one directory

## Context and Design Philosophy

A slice's artifacts are scattered across three trees today. Its design is at
`<crate>/docs/intent/<slice>/lld.md`, its claims at `<crate>/src/spec/<slice>.rs`
behind a re-exporting `mod.rs`, and its code at `<crate>/src/<slice>.rs` with a
`<slice>/` directory beside it. A reader who opens the code sees neither the
design nor the claims; a reader who opens the design sees neither of the others.
README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) states the
target: everything about a slice in `src/<slice>/` — `lld.md`, `vocab.rs`,
`spec.rs`, `mod.rs`, and the leaves — so that opening one directory puts the
whole arrow in front of the reader, design through validations.

This slice is that move. It adds no check and no capability of its own except
one the layout makes possible (below); what it delivers is that every later
slice is cheaper to read and to build.

**Measured before it was written.** Fourteen slices carry an `lld.md`, and
three of them — `book`, `publish`, `skill` — are workspace-only and have no
crate module to move into. There are 344 claims across fourteen `src/spec/`
modules, 1095 `spec::`-prefixed citation paths and 91 written
`crate::spec::`. Fifteen `include_str!` sites reach an intent document by
relative path. So the cascade is large, almost entirely mechanical, and its
risk is concentrated in two places that are not mechanical at all: the three
workspace-only slices, and the companion rule the controlled-language slice
introduced.

The reason this slice is delivered now, immediately after the controlled
language and before the burn-down of that language's `#[lid(free)]` marks, is
stated in the HLD: the language landed by *marking* every claim, not by
rewriting it, so no rename cascade has happened yet. The burn-down that follows
rewrites claims slice by slice, and each rewrite is cheaper once a slice's
claims already sit beside its module. Moving after the burn-down would be two
cascades over the same files.

**The layout answers a measured fault.** A claim can be registered and yet
uncitable, because `src/spec/mod.rs` is a hand-maintained re-export list and a
claim whose line is missing from it has no path any other module can write. The
registry sees it; the compiler cannot reach it. Under colocation a claim is
uncitable only if its own slice module is, which is a mistake nobody makes
twice — and that is what makes the uncitable-claim assertion
(the claim slice's Deferred 2) implementable here rather than there: the
assertion needs a *citable path* to project through, and this slice is what
defines one.

**This move renames every claim, and the first draft of this document denied
it.** `Spec::NAME` is `concat!(module_path!(), "::", stringify!(ident))` —
the registry join key is the claim's *path*. Moving
`lid-rs/src/spec/registry.rs` to `lid-rs/src/registry/spec.rs` turns
`lid_rs::spec::registry::X` into `lid_rs::registry::spec::X` for all 344
claims, which is what README §8 calls a rename and what Phase 8 exists to
cascade. Check 1 does **not** catch it: a citation that still resolves names a
claim whose key changed underneath it. What catches it is check 5 — nine of
those keys are pinned as string literals in `lid-rs/src/spec/mod.rs` — and
check 12, whose traced mapping joins on `NAME`. Any plan for this slice that
treats the move as path-only is wrong at its foundation.

What is out of scope: rewriting any claim's *text*, which is each slice's own
Phase 8 burn-down; the `vocab.rs` file README §11.1 shows, which is the next
slice's (this slice creates no vocabulary and moves none); and any change to
what the checks check. A claim's *text* and its validators' bodies are
identical before and after; its key, its path, and every citation of it are
not.

## Behaviour

### The target, per slice

For a slice `s` in crate `c` whose code is `c/src/s.rs` and `c/src/s/`:

| Today | After |
|---|---|
| `c/docs/intent/s/lld.md` | `c/src/s/lld.md` |
| `c/src/spec/s.rs` | `c/src/s/spec.rs` |
| `c/src/s.rs` | `c/src/s/mod.rs` |
| `c/src/s/<leaf>.rs` | unchanged |
| `#[doc = include_str!("../docs/intent/s/lld.md")]` on `mod s` | `#![doc = include_str!("lld.md")]` inside `mod.rs` |

`src/spec/mod.rs` is deleted. A citation that read `crate::spec::Name` reads
`crate::s::spec::Name`, and one that read `spec::Name` from inside slice `s`
reads `spec::Name` still, because `spec` is now a sibling module of the citing
code rather than a crate-root one — which is why 1095 of the 1186 citation
sites do not change at all.

`lld.md` in a directory is what marks that directory a slice. Nothing else
does: not a manifest key, not a naming convention, not a registry entry. That
is the rule `lld-check`, the phase policy, the coach's index, and the book all
read, and having exactly one marker is the point of the move.

### The three slices with no module

`book`, `publish`, and `skill` are slices of this *workspace* rather than of a
crate: their artifacts are `book/`, the publish metadata in every manifest, and
`lid-rs/skill/`. They have no `src/<slice>/` to live in and inventing one would
be a module that exists to hold a document.

They keep `docs/intent/<slice>/lld.md` at the workspace root, and the rule
becomes: **a slice's `lld.md` sits beside its code when it has code, and under
`docs/intent/` when it has none.** `lld-check` and the phase policy admit both
forms; the policy already resolves a workspace-only slice differently, because
such a slice has no crate for its paths to be relative to.

This is a two-form rule, which is worse than a one-form rule, and the
alternative is worse still — see Decisions.

### The companion rule, under colocation

A proc-macro crate's slice keeps its claims, its companion module, and its
fixtures in the crate named by `[package.metadata.lid_rs] companion` (the
`phase` slice's rule). Colocation does not change which crate they live in; it
changes where inside it. For the `claim` slice, whose LLD is in
`lid-rs-macros` and whose companion is `lid-rs`:

| Today | After |
|---|---|
| `lid-rs-macros/docs/intent/claim/lld.md` | `lid-rs-macros/src/claim/lld.md` |
| `lid-rs-macros/src/claim.rs` | `lid-rs-macros/src/claim/mod.rs` |
| `lid-rs/src/spec/claim.rs` | `lid-rs/src/claim/spec.rs` |
| `lid-rs/src/claim.rs` | `lid-rs/src/claim/mod.rs` |

So one slice has two `src/claim/` directories in two crates, and the `lld.md`
is in the crate the LLD is in — the one whose code the phases edit. The
companion directory has a `spec.rs` and a `mod.rs` and no `lld.md`, which is
consistent with the marker rule: the companion directory is not a slice, it is
one slice's presence in another crate. `lld-check` must not read it as a slice
missing its document.

### How the move is made

The move is mechanical and the compiler is what verifies it, one slice at a
time rather than all fourteen at once: a slice moved wrongly is a `cargo check`
failure at every citation of it, and moving one slice per commit keeps the
failure set small enough to read. The order is leaves first — a slice nothing
else cites — so that each commit's breakage is confined to the slice being
moved.

`git mv` for every file, then the include path, then the citation prefix. No
file's *content* changes except:

1. the `include_str!` path in the moved `mod.rs`, which becomes `"lld.md"`;
2. the `crate::spec::` prefix at the 91 sites that write it;
3. the deletion of `src/spec/mod.rs`.

This list is known to be incomplete — at minimum it omits 22 `use crate::spec;`
lines (none of which contain the string `crate::spec::`), the `#[doc =
include_str!]` lines in four `lib.rs` files, README's own §11.1 and three
citation sites, seven skill files and their synced `.claude/` copies, `init`'s
templates and assertions, and six test fixtures that build the old tree. One of
the 91 `crate::spec::` sites must *not* change:
`lid-rs/tests/ui/pass/module_tracing.rs` writes the fixture crate's own path, so
a mechanical prefix rewrite breaks it.

Check 1 is **not** sufficient verification, for the reason given in Context: it
passes on a resolved citation whose claim's key has changed.

### What must change in the tooling

- **the phase policy** (`cargo-lid-rs/src/phase/policy.rs`) — its per-phase
  path tables name `src/<slice>.rs`, `src/<slice>/`, `src/spec/<slice>.rs` and
  `src/spec/mod.rs`. Colocation **adds** a Phase 2 path and does not remove
  one: today `pub mod spec;` already exists in `lib.rs`, so a claim Phase 2
  writes is compiled immediately, which is what `PhaseTwoChecksTheClaimsBuild`
  checks and what makes the controlled language's errors land in Phase 2. Under
  colocation nothing declares a new slice's `spec` module — `src/<slice>/mod.rs`
  and `src/lib.rs` are Phases 3-4's — so the file is not compiled, Phase 2's
  check passes vacuously, derive errors surface in a phase that may not fix
  them, and the Phase 5 red set finds nothing. Phase 2 must gain `mod.rs`, and
  `lib.rs` for a new slice. This is the claim slice's lesson in a new shape;
- **`lld-check`** — it locates a slice's LLD; it must find both forms and must
  not read a companion directory as a slice;
- **`init` and `new`** — the scaffolds they write are the layout, so the
  templates move with it;
- **the coach's index** — it opens with the repository's intent documents and
  finds them by path;
- **the book** — assembled by inclusion from paths that move;
- **this LLD's own path.** The layout slice is built under the old layout and
  moves itself last, because the policy resolves a slice's crate from where its
  LLD is. Its document therefore starts at
  `cargo-lid-rs/docs/intent/layout/lld.md` and ends at
  `cargo-lid-rs/src/layout/lld.md`, moved by the cascade it describes.

### The uncitable-claim assertion

With a citable path defined, the derive can emit a projection through it — the
claim slice's Deferred 2. A claim in `c/src/s/spec.rs` is citable as
`c::s::spec::Name`, and the derive knows `module_path!()`, so it can emit a
const that fails to compile when the claim's own module is unreachable from the
crate root. This is the one capability the slice adds, and it is the reason the
claim slice deferred it here rather than solving it against `src/spec/mod.rs`,
where the path a claim *should* have was not derivable from where it sat.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Where a workspace-only slice's LLD lives | `docs/intent/<slice>/lld.md` at the workspace root — a two-form rule | A `src/<slice>/` module that exists only to hold the document; a `book/lld.md` beside each one's artifacts; making them not slices | A module holding no code is a lie the compiler cannot catch, and `lib.rs` would have to include it. Beside their artifacts means three more forms, not one. Making them not slices costs them their phase flow and their place in the coach's index, which is a real loss for the `publish` slice especially. Two forms, both stated, is the least bad. |
| One slice per commit | Move one slice, check, commit; leaves first | One cascade commit for all fourteen; one commit per crate | The compiler verifies each move, and the failure set of a wrong move is every citation of that slice. Fourteen slices at once makes that set unreadable. Per crate is the same problem, smaller. |
| The `lld.md` marks the slice | A file's presence is the marker | A `[package.metadata.lid_rs] slices = [...]` list; a naming convention on the module; a registry entry | A list is a second place to forget, which is the fault the layout exists to fix. A convention cannot be checked. A registry entry is available only after the code compiles, and `lld-check` runs before it. |
| The companion directory carries no `lld.md` | `spec.rs` + `mod.rs` only | An `lld.md` that includes the real one; a symlink | One slice has one document, or the two diverge. A symlink is not portable to a published tarball. `lld-check` learns the exception instead, and states it. |
| Claims stay textually unchanged | Only paths move | Fold the burn-down into this slice, rewriting claims as they move | Two changes in one commit set, one of them 344 claims' meaning, with the compiler unable to distinguish a bad move from a bad rewrite. The HLD's ordering argument is exactly that these are separable. |

## Open Questions & Future Decisions

### Unresolved: this is not one slice

An adversarial read of this document's first draft found it spans at least
three pieces of work with different actors, and the seams fall on its own
section boundaries. Recorded here rather than decided, because re-slicing the
HLD's map is the human's:

- **A — the layout rule and the resolvers that read it.** `cargo_lid_rs::layout`
  and its three functions, `slice_crate`, `lld_path`, the coach's walk,
  `init`/`new` templates. A normal slice, buildable by phase agents inside
  `cargo-lid-rs`, **if** it lands accepting both crate forms — because the
  cascade guarantees a mixed state, and `slice_crate` today looks only for
  `docs/intent/<slice>/lld.md` and errors otherwise. It must land first:
  nothing can move until the tooling tolerates the move.
- **B — the migration itself.** 15 LLDs, 11 spec files, 22 `use` lines, ~1186
  citation sites, four `lib.rs` files, README, seven skill files and their
  synced copies, the book. **No phase agent can perform it**: `git mv` is a
  `ToolKind::Command`, absent from the agents' tools, and `policy::allowed`
  refuses every path outside the slice's own crates. So B is a human-run
  mechanical cascade or a Phase 8 per slice, and either way its commits are not
  `phase N:` commits — `phase-check` and the canopy's subject-tag reader will
  not recognise them. The document must name the actor and the subject
  convention before any phase starts.
- **C — the uncitable-claim assertion.** It belongs to the derive, in
  `lid-rs-macros`, which the policy refuses to a slice seated in
  `cargo-lid-rs`. It is a Phase 8 on `claim`, not part of this slice at all,
  and cutting it out is what takes this work from three crates to one.
- **D — the Phase 8s on other slices whose claim *text* names the old paths**:
  `TheSlicesCrateIsTheOneHoldingItsLld`, the two Phase-2 path claims,
  `lld-review`'s three document-location claims, and the coach's
  `TheIntentIndexNamesEveryIntentDocumentInTheWorkspace`. Phase 2 of this slice
  may not write another slice's spec file, so these are each that slice's own
  Phase 8.

Other findings the draft does not yet answer, each of which would stop a phase:
`xtask`'s claims are a flat `src/spec.rs` with no `mod.rs`, which the target
table does not cover; three more slices (`cargo-lid-rs`, `macros`, `xtask`) have
a crate but no `src/<slice>/` module, so the two-form rule is really four cases;
`intent-graph`'s slice name, module name and spec file all differ, and
`spec_file_of("intent-graph")` already names a file that does not exist;
`lid-rs/src/spec/citation.rs` is a spec file for a slice of no such name;
`lid-rs/src/canary.rs` is a `registry` leaf at the crate root with no stated
destination; deleting `lid-rs/src/spec/mod.rs` deletes the sole `#[validates]`
for six claims **and** the trybuild driver for the crate's whole UI suite;
`cargo-lid-rs/src/spec/mod.rs` holds six live `#[deprecated]` aliases whose
destination is undefined; and `docs/intent/claim/compile-time-accepted` has no
home once `docs/intent/` empties.

### Deferred

1. `vocab.rs` — the file README §11.1 shows in a slice directory is the next
   slice's to create; this slice moves none and creates none.
2. The `phase-check 7` package-step divergence, which this slice inherits and
   does not fix: `CLAUDE.md` runs one `cargo package` naming every publishing
   member, while README §4.5, the skill, and `phase::gate` run one per crate —
   and the per-crate form cannot resolve a sibling at a version not yet on
   crates.io. Every Phase 7 here is therefore finished by hand. It belongs to a
   Phase 8 on the `phase` slice or to the catalog slice, and it is a decision
   the human has not yet taken.
3. Check 12's `FullSuite` mapping for hand-authored edges (the claim slice's
   Deferred 5a). Under colocation a slice's edges and its code share a
   directory, so the mapping that matches an edge by `file!()` may become
   correct for free — measure it here rather than assuming it.

## Shape

| Item | Role |
|---|---|
| `cargo_lid_rs::layout` | This slice's module: the rule that a directory holding `lld.md` is a slice, and the resolution of a slice's artifacts from it. |
| `cargo_lid_rs::layout::slice_dir` | A slice's directory, from its name and crate — the one place the layout is computed. |
| `cargo_lid_rs::layout::lld_path` | A slice's `lld.md`, in either form: beside its code, or under `docs/intent/` when it has none. |
| `cargo_lid_rs::layout::is_companion_dir` | Whether a slice directory is a companion's — `spec.rs` and `mod.rs` with no `lld.md` — so that `lld-check` does not read it as a slice missing its document. |

## References

- README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) — the layout.
- `cargo-lid-rs/docs/intent/phase/lld.md` — the path policy and the companion rule.
- `lid-rs-macros/docs/intent/claim/lld.md` — Deferred 2, the uncitable-claim assertion.
