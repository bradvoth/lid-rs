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

This slice is that move, and it has the shape the controlled-language slice
had: **its phases build the mechanism, and its migration is hand-committed
`phase 8:` edits afterward.** Phases 1-7 deliver `cargo_lid_rs::layout` and the
resolvers that read it, inside `cargo-lid-rs`, where phase agents may write.
The migration itself — 15 LLDs, 11 spec files, ~1186 citation sites, four
`lib.rs` files, README, the skill, the book — is outside every phase's policy
and cannot be done by an agent at all: `git mv` is a `ToolKind::Command`,
absent from the agents' tools, and `policy::allowed` refuses every path outside
the slice's own crates. The precedent is exact: the claim slice's 302
`#[lid(free)]` marks were hand-committed the same way, for the same reason.

So the order is: the resolvers land **accepting both layouts**, then the
migration moves slices one commit at a time under a tooling that tolerates a
mixed tree, then a final commit drops the old form.

**How many resolvers, measured rather than guessed: eight claims across two
slices** (Deferred 1). Each names a path where it should ask, and between them
they need three doors on `layout`. One exists:

| Door | Answers | Old form | Asked by |
|---|---|---|---|
| `own_crate` *(built)* | the member holding the slice's document | `docs/intent/<slice>/lld.md` | `policy::slice_crate` |
| `spec_file` | the slice's claims file | `src/spec/<slice>.rs` | the red set's two claims, and Phase 2's two path claims |
| `intent_file(name)` | a named file of the slice's intent directory | `docs/intent/<slice>/<name>` | `policy::compile_time_accepted`, and `lld-review`'s three |

`spec_file` and `intent_file` are two doors and not one because their **old**
forms differ in shape: a spec file embeds the slice's name in the filename
(`src/spec/lld_review.rs`), while an intent file is a named file inside a
directory named for the slice (`docs/intent/lld-review/compile-time-accepted`).
Under colocation both become a named file inside the slice's directory, so the
two collapse — but the migration is exactly the window in which they have not,
and a door that pretends otherwise answers wrongly for every unmoved slice.

**Two crates, not one, and conflating them is a bug this document nearly
shipped.** Answering a slice's claims file needs *two* crates, and they are the
same only for an ordinary slice:

- the **reading** crate — whose directory is checked for `lld.md`, to know
  whether the slice has moved. This is always the slice's **own** crate,
  because `ASlicesDocumentIsNeverUnderItsCompanion`.
- the **joining** crate — which the answer is relative to. For a proc-macro
  slice this is the **companion**, which holds the claims.

An earlier draft had `spec_file` take one crate root and read the layout from
it. `phase.rs`'s red set passes `crates.claims_crate()` there, which for a
proc-macro slice is the companion — and a companion never holds an `lld.md`, so
the door would have answered *"not yet migrated"* for those slices **forever**,
silently, for the whole life of the migration and after it. Caught by the
Phase 3 that seated the door and asked whether each caller could supply the
reading crate; verified at `phase.rs:588`.

So `spec_file` resolves the reading crate itself — the same resolution
`own_crate` performs — and answers a path **relative** to nothing, which the
caller joins onto whichever crate it means. It therefore *can* refuse, for a
slice no member holds a document for, and `ASliceNoMemberHoldsIsRefusedByName`
covers it after all:

```text
spec_file(project: &Project, slice: &str) -> Result<PathBuf, String>
```

The two doors are alike after all: both resolve the slice's own crate to read
its layout. They differ only in what they return — `intent_file` an absolute
path in that crate, `spec_file` a relative one the caller places, because only
the caller knows whether it means the slice's crate or its companion.

`lld_path` stays as it is: it is `intent_file("lld.md")` plus the workspace-root
answer for a slice with no crate, and that special case is real. Whether it
should be rebuilt on `intent_file` once the migration ends is Deferred 7.

**The delegation is four commits, not one, and the companion key is not in
them.** A `phase` Phase 7 agent may write `src/phase.rs` and `src/phase/**`
only, so it can neither expose anything from `layout` nor move a reader into
it. The order that works:

1. **A Phase 2 on `layout`**, because the door needs a claim before it can be
   written. Every existing claim names `slice_dir`, `lld_path`,
   `is_companion_dir` or `Form::of_slice` in its *when*, and a wrong answer
   from a free `own_crate` falsifies none of them — it sits **above** them in
   the call graph, not inside one. Mutate it to `Ok(PathBuf::new())` and every
   validator still passes. By README §4.3 the honest citation set is empty, and
   an uncited fn gets check 12's module fallback: the ten validators in
   `layout.rs`, none of which calls it, so both of its body mutants survive.
   §4.3's own reading applies — an uncontradictable citation with a mutant that
   exists to prove it means **a claim is missing from the design**.

   Two claims cover it: one for what the door answers, and a **reword** of
   `ASliceNoMemberHoldsIsRefusedByName` from *when `slice_dir` is asked* to
   *when a resolver in `layout` is asked*, which keeps one *when*, keeps the
   struct's name true, and so needs no rename across its eight citation sites.

   Rejected: citing a claim by association; borrowing `phase`'s
   `TheSlicesCrateIsTheOneHoldingItsLld` (a slice cites its own claims, and at
   this step `slice_crate` does not yet delegate, so that validator never
   reaches the door); adding an assertion about `own_crate` to an existing
   validator, which asserts what that validator's claim does not state —
   README §4.4's uncaught fault, deliberately committed; and an `exclude_re`
   exempting the door from check 12, which is measurement waived on a live
   public function with two observable answers, not the I/O sequencing the
   three existing entries stand on.

2. **A Phase 8 on `layout`** exposing `own_crate(project, slice) ->
   Result<PathBuf, String>`, wrapping the private `Form::own_crate` and
   `no_crate_refusal`. Without it, `policy` would have to match `Form`'s
   variants itself and rebuild the refusal sentence that
   `ASliceNoMemberHoldsIsRefusedByName` owns — one decision duplicated across
   two slices, which is what this slice exists to stop.

Steps 3 and 4 are the `phase` slice's, and **the policy binds a slice to its
branch name** (`TheSliceComesFromTheBranchName`), so they cannot be committed
from `lld/layout` — a Phase 2 there is a Phase 2 on `layout`, whatever its
prompt says. They run on `lld/phase--layout-delegation`, which
`AChangeBranchNamesItsSliceBeforeTheDoubleDash` reads as slice `phase`, branched
at the tip of `lld/layout` and fast-forwarded back into it when they land.
History stays linear because nothing else is on `lld/layout`; the alternative,
hand-committing a claim edit outside the phase machinery, forgoes the check and
the `Lid-Rs-*` trailers on exactly the kind of edit they exist for.

3. **A Phase 2 on `phase`** rewording `TheSlicesCrateIsTheOneHoldingItsLld`,
   which the delegation contradicts: it pins `docs/intent/<slice>/lld.md`, and
   a migrated slice's crate holds no such file. The struct's **name stays
   true** — the slice's crate *is* the one holding its LLD — so this is a
   reword, not a rename, and the `#[implements]` on
   `Project::member_manifest_dirs` needs no cascade.
4. **The Phase 8 on `phase`** delegating `slice_crate` to `layout::own_crate`.

**The companion key stays where it is, for now.** Moving its reading into
`layout` would carry the only implementers of five `phase` claims —
`AnOrdinaryCrateHasNoCompanion` and the four refusal claims — into another
slice's module, against the rule that a slice cites its own claims, and needs
two more Phase 2s. What it buys is direction: `layout::companion_owner` calls
`policy::companion` while `policy::slice_crate` calls `layout`. That is a
module-level cycle, not a recursive one — `Form::of_slice` never reaches
`companion_owner` — so it compiles and is correct, and the objection to it is
architectural rather than a fault. Deferred 6.

**A third site breaks when `claim` moves, and it is not `slice_crate`.**
`policy::compile_time_accepted` hard-codes
`docs/intent/<slice>/compile-time-accepted`, and
`ACompileTimeSliceNeedsTheHumansAcceptance` pins that path. The `claim` slice's
acceptance file moves with its document, so an unrewired policy refuses **every
edit to that slice** the moment it does. It must be rewired before `claim` is
migrated, which is a constraint on the migration order and not only a task:
`claim` moves last, or this is fixed first.

**Superseded by the three-commit plan above; kept for the argument it makes.**
`layout` reads
`[package.metadata.lid_rs] companion` through `phase::policy::companion` today,
so `layout` depends on `phase` — and the Phase 8 below makes `phase` depend on
`layout`. Two modules of one crate may depend on each other, but the direction
should be stated rather than left as whichever was written first. The companion
relation is a **layout fact**: which crate holds a slice's claims and module is
the same kind of question as which directory holds its document. So the single
Phase 8 on `phase` does both rewirings in one direction — `slice_crate`
delegates to `layout`, and the reading of the companion key moves into `layout`
with `policy::companion` calling it. Until that Phase 8, the dependency runs
the other way and is a known, bounded inversion.

**`layout` owns the both-layouts resolution; `phase` delegates to it.**
`policy::slice_crate` finds a slice's crate by looking only for
`docs/intent/<slice>/lld.md`, so it cannot find a slice whose document has
already moved. That resolver is the `phase` slice's, which this slice's phases
may not write. The seam: this slice's `layout` module owns the resolution that
admits both forms, and a **Phase 8 on `phase`** rewires `policy::slice_crate`
to delegate to it. That Phase 8 is part of "the resolvers land", not part of the
migration — it must be committed before the first slice moves, or the phase
machinery loses every slice the migration touches. Nothing can move before the
tooling tolerates the move, and a mixed state is guaranteed because the
migration is incremental.

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

A citation that read `crate::spec::Name` reads `crate::s::spec::Name`. One that
read `spec::Name` from inside slice `s`, naming *its own* claim, reads
`spec::Name` still, because `spec` is now a sibling module of the citing code.

**But a cross-slice citation is textually identical to a same-slice one**, and
that is the flat re-export's doing: `cargo-lid-rs/src/sync.rs` writes
`spec::SyncMirrorsEveryArtifactTheDependencyShips` for a claim defined in the
`phase` slice's spec file. After the move that site must read
`crate::phase::spec::…`. So the 1186 sites cannot be partitioned by their text;
each must be resolved **by claim name to its owning spec file**. That resolution
is the migration's real work and the reason it is mechanical only in the sense
that a compiler can check it — not in the sense that a regex can perform it.

`lld.md` in a directory is what marks that directory a slice. Nothing else
does: not a manifest key, not a naming convention, not a registry entry. That
is the rule `lld-check`, the phase policy, the coach's index, and the book all
read, and having exactly one marker is the point of the move.

### Where a slice's `lld.md` goes — four cases, not two

A slice's directory is the directory holding its **code**, and its `lld.md`
goes there. That is one rule, but the repository presents four shapes of it:

| Shape | Slices today | Where `lld.md` goes |
|---|---|---|
| code in `src/<m>.rs` + `src/<m>/` | `phase`, `coach`, `claim`, `sync`, `init`, `registry`, `lld-review`, `headless-canopy-agent`, `macros`… | `src/<m>/lld.md` |
| code **is** the crate root | `cargo-lid-rs`, `macros`, `xtask` | the crate's `src/lld.md`, included by `lib.rs`'s `#![doc]` as it already is |
| no code at all | `book`, `publish`, `skill` | `docs/intent/<slice>/lld.md` at the workspace root, unchanged |
| a companion's presence in another crate | `lid-rs/src/claim/` | no `lld.md` — see below |

The second case is the one the first draft missed, and it is why the rule is
stated as *beside the code* rather than *in `src/<slice>/`*: a crate-root slice
already carries its LLD as `lib.rs`'s inner doc, so it moves from
`docs/intent/<s>/lld.md` to `src/lld.md` and the `#![doc = include_str!]`
shortens. No module is invented to hold a document, which is the thing the
Decisions table refuses.

**A crate-root slice's name is its package's name.** `<crate>/src/lld.md`
records no slice name, so a search that accepts it as a candidate for a slice
resolved *by name* accepts it for every member holding no module of that name —
`of_slice(project, "macros")` would answer `cargo-lid-rs` the moment that crate
migrated, and `ASliceNoMemberHoldsIsRefusedByName` would stop firing at all once
any crate-root slice had moved. The name has to come from somewhere the tree
already states it, and for a crate-root slice that place is `Cargo.toml`: the
slice *is* the crate.

Two of the three already satisfy it — slice `cargo-lid-rs` in package
`cargo-lid-rs`, slice `xtask` in package `xtask`. The third does not: slice
`macros` lives in package `lid-rs-macros`. **It is renamed to `lid-rs-macros`**
by the ordinary Phase 8 discipline, as part of the migration and before its own
crate moves — covering `lid-rs-macros/docs/intent/macros/`, its companion
directory in `lid-rs`, and the two mentions of it in this document. The
alternative rules are in Decisions; this one is chosen because it adds no
mechanism and no thing to remember, and because a crate-root slice that is not
named for its package is a name the tree already contradicts.

**The slice's name becomes its directory's name.** Three slices have a name
that is not their module's: `intent-graph` (module `graph`), and the
crate-root slices whose names carry hyphens. This is not cosmetic —
`spec_file_of("intent-graph")` yields `src/spec/intent_graph.rs`, **a file that
does not exist**, so that slice's claims are already unfindable by
`slice_claims` and its red run already finds nothing. The layout surfaces a
defect that predates it. Each such slice is renamed to its module's name by the
ordinary Phase 8 discipline — rename, `#[deprecated]` alias, cascade — as part
of its own migration commit.

### Resolving a directory back to a slice

`is_companion_dir` takes a directory, and mapping a directory back to a slice is
**not an inversion**: `slice.replace('-', "_")` is not injective, so
`headless_canopy_agent` has one preimage today and two in principle. Nothing
guesses. The route is forward: enumerate the slices of the member in question
from the documents it holds, and compare each slice's module name against the
directory's. That rests on the marker rule — a document is what says a slice
exists — and it is a second reason the crate-root naming rule above must hold,
since enumerating a crate's slices from its documents is exactly what an
unnamed `src/lld.md` would defeat.

`coach::slice_named` reads a slice's name out of its document's parent
directory and would answer `src` for a migrated crate-root slice. It is the
`coach` slice's, and its correction is one of the Phase 8s in Deferred 1.

### `Form` carries the name it was resolved for

A `Form` is not enough to build every path the claims ask for: the old-form
answer is `docs/intent/<slice>/lld.md` with the slice's name **as given** —
`lld-review`, not `lld_review` — which no variant's data supplies. The
resolvers each take a slice name and could pass it alongside, but
`Form::of_directory` *derives* a name from a directory and would then discard
it, and that is the one place the fact has no other source. So `Form` carries
the slice name it was resolved for.

### The files with no slice directory

Three files have no destination the target table computes, and each gets one
here rather than at the phase that trips over it:

- **`lid-rs/src/canary.rs`** is a leaf of the `registry` slice sitting at the
  crate root. It moves to `lid-rs/src/registry/canary.rs`. The table's "leaves
  unchanged" row covers leaves already under `src/<m>/`; this one is not, and
  the rule is that a leaf moves into its slice's directory.
- **`lid-rs/src/spec/citation.rs`** is a spec file for a slice of no such name:
  its claims are the `macros` slice's, cited by hand-authored edges in
  `lid-rs/src/lib.rs`. It is the companion shape — `lid-rs` is the companion of
  `lid-rs-macros` — so it becomes `lid-rs/src/macros/spec.rs`, beside
  `lid-rs/src/claim/spec.rs`, and neither carries an `lld.md`.
- **`xtask/src/spec.rs`** is flat, with no `src/spec/mod.rs` at all, for a
  crate-root slice. It becomes `xtask/src/spec.rs` — unchanged — because a
  crate-root slice's `spec.rs` is already beside its code.

  **So a crate-root slice's claims file is `src/spec.rs`, in both layouts**, and
  that is what `spec_file` must answer for one. `xtask` already satisfies it;
  `cargo-lid-rs`'s own slice does not — its claims are at
  `src/spec/cargo_lid_rs.rs` — so that file moves to `src/spec.rs` with the
  migration, which is the only crate-root claims file that moves at all.

  **This is a defect that predates the slice, and asked of all fourteen slices
  it is three of them.** `spec_file_of("xtask")` computes `src/spec/xtask.rs`,
  which does not exist, so `xtask`'s red run finds no claims **today** — as for
  `intent-graph`, whose slice name, module name and spec file all differ, and
  `macros`, whose claims are its companion's at `lid-rs/src/spec/citation.rs`
  while `spec_file_of` computes `src/spec/macros.rs`.

  For each, `slice_claims` matches no registered claim, the red set is empty,
  and Phase 5's proof that a test fails before it passes has been proving
  nothing since the slice was built. Nothing failed, because **an empty red set
  and a slice whose tests all legitimately pass look identical to a check that
  only asks whether anything failed** — the same shape as slice 15's lexicon
  harness, which asserted eighteen expectations against an always-empty report
  for its whole life. The canary exists because slice 1 anticipated exactly this
  for `SPECS`; nothing analogous guards the red set.

  `spec_file` repairs them unevenly, and saying "fixes all three" overstates it
  by one: **`xtask`** by construction, since the door resolves what a crate
  holds rather than computing a path from a name; **`intent-graph`** only once
  its migration commit renames it to its module's name; **`macros`** not at all,
  because its claims are its companion's and would sit at
  `lid-rs/src/macros/spec.rs` — a directory named for the slice, which no
  crate-root answer names. Whether `spec_file` should know the joining crate,
  and so repair `macros` too, is Deferred 8.

### What `src/spec/mod.rs` holds besides re-exports

Deleting the two `src/spec/mod.rs` files is not a deletion of re-exports alone,
and the first draft listed it as one of three content-free changes:

- `lid-rs/src/spec/mod.rs` holds a `#[cfg(test)] mod tests` carrying the **sole
  `#[validates]` for six claims** and the trybuild driver for every UI fixture
  in the crate. Deleting it fails check 11 for six claims and silently stops
  running the UI suite. Those tests move to `lid-rs/src/registry/mod.rs` and
  `lid-rs/src/macros/…`, beside the claims they observe — which is what
  colocation is for, and the move is this slice's, not a later slice's.
- `cargo-lid-rs/src/spec/mod.rs` holds **six live `#[deprecated] pub type`
  aliases** from the still-open `phase` and `coach` renames. An alias exists so
  the *old* path keeps resolving; moving it changes the old path and defeats
  it. They stay at `cargo-lid-rs/src/spec/mod.rs`, which therefore is not
  deleted in that crate until those renames' deprecation windows close — a
  file holding aliases and nothing else, with a comment saying so.
- `lid-rs-macros/docs/intent/claim/compile-time-accepted` is read by the policy
  and named in the canopy's stop sentence. It moves to
  `lid-rs-macros/src/claim/compile-time-accepted`, beside the LLD whose
  acceptance it records.

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

**The main session performs the migration by hand**, after this slice's Phase 7,
as `phase 8: layout — <slice> moves` commits. No phase agent can do it: `git mv`
is a `ToolKind::Command`, absent from their tools, and `policy::allowed` refuses
every path outside the slice's crates. These are not `phase N:` commits, so
`phase-check` does not gate them and the canopy's subject-tag reader will not
count them; the gate they answer to is the workspace's, run by hand after each.

One slice per commit, and the compiler verifies each: a slice moved wrongly is a
`cargo check` failure at every citation of it. **The order is not "leaves
first"** — that was the first draft's answer and it is not computable, because a
cross-slice citation is textually indistinguishable (above) and because every
move edits its crate's shared `lib.rs`, including the 54 hand-authored
`macro_edge!`/`claim_edge!` entries in `lid-rs/src/lib.rs`. The order is instead
**stated**: the migration commits are named in the slice's Phase 1, one per
slice, and a mixed tree is expected throughout — which is what the both-forms
resolvers exist for.

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

### The uncitable-claim assertion is not in this slice

The claim slice deferred it here (its Deferred 2) on the reasoning that the
assertion needs a citable path and the layout defines one. The path half is
right; the placement is not. The assertion is emitted by the **derive**, in
`lid-rs-macros/src/claim/`, and `policy::allowed` refuses that crate to a slice
seated in `cargo-lid-rs` — so no phase of this slice can build it. It is a
Phase 8 on `claim`, taken once this slice's migration has landed.

Two things that Phase 8 must state, because they decide what the assertion is
worth, and which this document records so they are not rediscovered:

- it fires on a module *declared but unreachable*; it cannot see a `spec.rs`
  that nothing declares, which is the likelier fault under colocation;
- §Context's account of the old fault holds only for a module declared
  privately (`mod x;` rather than `pub mod x;`). If the line is missing
  altogether the file is not compiled and the registry never sees the claim
  either. The same private-vs-public mistake is available in the new `mod.rs`,
  so "a mistake nobody makes twice" is asserted rather than shown.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| The phases build the mechanism; the migration is hand-committed | Phases 1-7 deliver the rule and the resolvers in `cargo-lid-rs`; the main session moves the tree as `phase 8:` commits | A phase agent performing the move; one cascade commit; a `cargo lid-rs migrate` subcommand | No agent *can* move it — `git mv` is a `ToolKind::Command` and the policy refuses every path outside the slice's crates — so the only question is whether that is stated or discovered at the first refusal. The precedent is the claim slice's 302 marks, hand-committed for the same reason. A subcommand is code with a lifetime of one use. |
| The resolvers accept both layouts until the migration ends | Both forms admitted, the old dropped in a final commit | Switch the tooling first; switch it last; a feature flag | The migration is incremental by design, so a mixed tree is guaranteed, not a risk. Switching first breaks `slice_crate` for every unmoved slice — including `layout` itself, whose own stop hook then errors. Switching last means every moved slice is unresolvable in between. |
| A slice's directory is where its **code** is | Four shapes, one rule: beside the code, or `docs/intent/` when there is none | `src/<slice>/` always; a manifest list of slices | "Always `src/<slice>/`" invents a module to hold a document for the three crate-root slices, which is a lie the compiler cannot catch. A manifest list is a second place to forget, which is the fault the layout exists to fix. |
| A crate-root slice's name is its package's name | The rule, with `macros` renamed to `lid-rs-macros` | Crate-root slices keep `docs/intent/<slice>/lld.md` permanently; a `[package.metadata.lid_rs]` key naming the crate's slice | `src/lld.md` names no slice, and a by-name search that accepts it matches every member without that module — which silently retires `ASliceNoMemberHoldsIsRefusedByName`. Keeping the old path for three slices abandons colocation for them, which is the promise §Context makes. A manifest key is a fourth thing a new crate-root slice must remember, and the package name is a fact the tree already states. The cost is one rename under the discipline this project already uses for renames. |
| `Form::of_slice` never answers `Companion`, as a constructor invariant | The invariant, with the unreachable arm carried at every match | A split type — `OwnForm` of three variants, wrapped by `Form` — making it unrepresentable | `ASlicesDocumentIsNeverUnderItsCompanion` is a **claim**, and a claim the type makes unrepresentable is one no `#[validates]` test can turn red at Phase 5 and no mutant can kill at check 12. Making it impossible would delete the evidence that it holds. The unreachable arm is what that evidence costs. |
| Where a workspace-only slice's LLD lives | `docs/intent/<slice>/lld.md` at the workspace root | A `src/<slice>/` module that exists only to hold the document; a `book/lld.md` beside each one's artifacts; making them not slices | A module holding no code is a lie the compiler cannot catch, and `lib.rs` would have to include it. Beside their artifacts means three more forms, not one. Making them not slices costs them their phase flow and their place in the coach's index, which is a real loss for the `publish` slice especially. Two forms, both stated, is the least bad. |
| One slice per commit | Move one slice, check, commit; leaves first | One cascade commit for all fourteen; one commit per crate | The compiler verifies each move, and the failure set of a wrong move is every citation of that slice. Fourteen slices at once makes that set unreadable. Per crate is the same problem, smaller. |
| The `lld.md` marks the slice | A file's presence is the marker | A `[package.metadata.lid_rs] slices = [...]` list; a naming convention on the module; a registry entry | A list is a second place to forget, which is the fault the layout exists to fix. A convention cannot be checked. A registry entry is available only after the code compiles, and `lld-check` runs before it. |
| The companion directory carries no `lld.md` | `spec.rs` + `mod.rs` only | An `lld.md` that includes the real one; a symlink | One slice has one document, or the two diverge. A symlink is not portable to a published tarball. `lld-check` learns the exception instead, and states it. |
| Claims stay textually unchanged | Only paths move | Fold the burn-down into this slice, rewriting claims as they move | Two changes in one commit set, one of them 344 claims' meaning, with the compiler unable to distinguish a bad move from a bad rewrite. The HLD's ordering argument is exactly that these are separable. |

## Open Questions & Future Decisions

### Deferred

1. **The Phase 8s on other slices whose claim *text* names the old paths.**
   **Measured, not estimated**: all 344 claims were swept for text naming
   `src/spec/`, `docs/intent/`, `spec/mod.rs` or `<slice>.rs`. Thirteen name the
   old layout; two are this slice's own and correctly describe the old form as
   the fallback the resolvers accept; **eleven are falsified by the migration**,
   where this item previously listed six.

   **Eight of the eleven are preconditions** — they break on the *first* slice
   to move, not eventually:

   | Claim | Slice | What breaks |
   |---|---|---|
   | `ASlicesClaimsAreTheSpecsInItsSpecFile` | phase | the red set reads `src/spec/<slice>.rs`; a moved slice's Phase 5 finds no claims and fails |
   | `TheRedSetIsTheClaimsAddedSinceTheBase` | phase | the red set is `git diff <base> -- src/spec/<slice>.rs`; same break, second site |
   | `PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles` | phase | Phase 2's allowed *target* is that path, so Phase 2 cannot write a moved slice's claims at all |
   | `PhaseTwoMayWriteOnlyTheCompanionsSpecFiles` | phase | the same, companion seat |
   | `ACompileTimeSliceNeedsTheHumansAcceptance` | phase | acceptance read from `docs/intent/<slice>/compile-time-accepted`; once it moves the policy refuses every edit to that slice |
   | `TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt` | lld-review | `lld-check` resolves the old path, and it is a **Phase 1 gate step** |
   | `AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot` | lld-review | same resolver, workspace-only branch |
   | `AnUnreadableLldFailsNamingItsPath` | lld-review | names the path it failed on, which is the old form |

   The other three are follow-ups: the `coach`'s two index claims degrade an
   interview's opening rather than failing a gate, and `claim`'s
   `AMemberFindsItsWorkspacesLexicon` is about the lexicon walk and may not be
   affected at all — **verify before assuming it is**.

   **So the precondition is eight claims across two slices, not the three
   rewirings §Context says.** Each needs its resolution to ask `layout` rather
   than name a path, and two of the doors they would ask — a spec-file resolver
   and an acceptance-file resolver — do not exist yet. That is a slice's worth
   of work standing between here and the first `git mv`, and calling it "the
   migration" hides that the tree-wide move cannot start until it is done.

   Superseded text, kept for the record: two of them are preconditions,
   because the first slice to move breaks them:
   `ASlicesClaimsAreTheSpecsInItsSpecFile` pins the red set to
   `src/spec/<slice>.rs`, so a migrated slice's red run finds no claims and
   `ASliceWithNoClaimsFailsTheRedCheck` fails its Phase 5; and
   `ACompileTimeSliceNeedsTheHumansAcceptance` pins
   `docs/intent/<slice>/compile-time-accepted`, so an unrewired policy refuses
   every edit to a compile-time slice once its acceptance file moves. Both are
   `spec_file_of`-shaped and `slice_crate`-shaped respectively: the resolution
   must move to `layout` before any document does. The rest of this item —
   `TheSlicesCrateIsTheOneHoldingItsLld` and the two Phase-2 path claims in the
   `phase` slice, `lld-review`'s three document-location claims, and the coach's
   `TheIntentIndexNamesEveryIntentDocumentInTheWorkspace`. Phase 2 of this slice
   may not write another slice's spec file, so each is that slice's own Phase 8,
   taken with the migration commit that invalidates it. Listed here because a
   reader of this document would otherwise find four claims that contradict it
   and no record of why.
2. **The uncitable-claim assertion**, a Phase 8 on `claim` — see above.

3. `vocab.rs` — the file README §11.1 shows in a slice directory is the next
   slice's to create; this slice moves none and creates none.
4. The `phase-check 7` package-step divergence, which this slice inherits and
   does not fix: `CLAUDE.md` runs one `cargo package` naming every publishing
   member, while README §4.5, the skill, and `phase::gate` run one per crate —
   and the per-crate form cannot resolve a sibling at a version not yet on
   crates.io. Every Phase 7 here is therefore finished by hand. It belongs to a
   Phase 8 on the `phase` slice or to the catalog slice, and it is a decision
   the human has not yet taken.
5. Check 12's `FullSuite` mapping for hand-authored edges (the claim slice's
   Deferred 5a). Under colocation a slice's edges and its code share a
   directory, so the mapping that matches an edge by `file!()` may become
   correct for free — measure it here rather than assuming it.

6. The companion key's reading moving into `layout`, with `policy::companion`
   calling it — deferred above, because it carries five `phase` claims'
   implementers into another slice's module for an architectural gain rather
   than a correctness one. The module-level cycle it would remove compiles and
   is not recursive.

7. Rebuilding `lld_path` on `intent_file` once the migration ends and the two
   old forms no longer differ — until then its workspace-root answer for a
   slice with no crate is a special case `intent_file` does not carry.

8. Whether `spec_file` should know the joining crate. It answers a relative path
   the caller places, which is right for an ordinary slice and for a crate-root
   one, and leaves `macros` — whose claims are its companion's, in a directory
   named for the slice — with a red run that still finds nothing. Repairing it
   means the door knowing which crate joins its answer, which is the distinction
   §"Two crates, not one" deliberately kept out of it.

## Shape

| Item | Role |
|---|---|
| `cargo_lid_rs::layout` | This slice's module: the rule that a directory holding `lld.md` is a slice, and the resolution of a slice's artifacts from it. |
| `cargo_lid_rs::layout::own_crate` | The member whose directory holds a slice's document, in either layout, and the refusal naming the slice when none does. The one door a caller outside this module asks a slice's crate through — `phase`'s `policy::slice_crate` delegates to it, so the four shapes are told apart here and not a second time there. |
| `cargo_lid_rs::layout::spec_file` | A slice's claims file as a **relative** path: `src/<module>/spec.rs` once that slice's own crate holds the document beside its code, `src/spec/<module>.rs` until it does. Reads the layout from the slice's own crate, which it resolves, and refuses for a slice no member holds; the caller joins the answer onto whichever crate it means — its own, or its companion. See §"Two crates, not one". |
| `cargo_lid_rs::layout::intent_file` | A named file of a slice's intent — `compile-time-accepted`, and `lld.md` — beside the slice's code once moved, under `docs/intent/<slice>` in the slice's own crate until then. Resolves that crate itself, so it refuses for a slice no member holds. |
| `cargo_lid_rs::layout::slice_dir` | A slice's directory, from its name and crate — the one place the layout is computed. |
| `cargo_lid_rs::layout::lld_path` | A slice's `lld.md`, in either form: beside its code, or under `docs/intent/` when it has none. |
| `cargo_lid_rs::layout::is_companion_dir` | Whether a slice directory is a companion's, read from `[package.metadata.lid_rs] companion` in the manifest — **not** from the directory's shape. A shape heuristic ("`spec.rs` and `mod.rs` with no `lld.md`") cannot distinguish a companion from a slice whose Phase 1 was genuinely skipped, which is the case `lld-check` exists to catch; the manifest already carries the fact. |
| `cargo_lid_rs::layout::Form` | Which of the four shapes a slice has, so that every resolver branches once, here, rather than each inventing its own test. |

## References

- README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) — the layout.
- `cargo-lid-rs/docs/intent/phase/lld.md` — the path policy and the companion rule.
- `lid-rs-macros/docs/intent/claim/lld.md` — Deferred 2, the uncitable-claim assertion.
