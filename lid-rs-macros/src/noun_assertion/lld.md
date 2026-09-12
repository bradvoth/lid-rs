# A claim may name only a noun, and the derive says so at the claim

## Context and Design Philosophy

A claim already has to link. Slice 15 made the trigger a link for every pattern
and the response object a link wherever the verb's template is a shape. What a
link *names* is unconstrained: `ClaimMeta` carries the target as written
(`lid-rs/src/claim/mod.rs:57-80`), and a claim may link a function, an
attribute macro, a module, or any path that resolves. Slice 15's own claims
link `derive@crate::Spec` and `crate::validates` — a derive and an attribute.
Nothing objects.

README says something should. §3.5 lists among what `derive(Spec)` refuses "a
trigger or response slot that is **not an intra-doc link to a vocabulary
type**" (`README.md:417-423`), and check 13's row names "unlinked noun"
(`README.md:599`). **The rule is specified and unbuilt.** Slice 15's claims file
names the hand-off: the next slice replaces those targets with vocabulary
types.

**HLD row 17 is split into two slices, and this is the compile-time half.** The
traits the rule is expressed in — `Traceable`, the `Declared` seal, `Noun`, the
primitive impls, the crate-root re-exports — are `vocab`, a module slice in
`lid-rs` at `lid-rs/src/vocab/`, which is an ordinary library slice and is
buildable now. This document covers `derive(Traceable)` and the assertion
`derive(Spec)` emits, both in `lid-rs-macros`.

**This half is blocked, and will stay blocked until a human unblocks it.**
`lid-rs-macros` declares `proc-macro = true` (`lid-rs-macros/Cargo.toml:14-15`),
so `execution_class` classes the slice `CompileTime("proc-macro")`
(`cargo-lid-rs/src/phase/policy.rs:753-760`) and `acceptance_gate` refuses
*every* edit — before the path policy is even consulted
(`cargo-lid-rs/src/phase/mod.rs:326-329`, `:334-340`) — until
`lid-rs-macros/src/noun_assertion/compile-time-accepted` is committed. The
refusal is correct: editing this slice means the model's code runs inside every
`cargo check` in the workspace.

**The rename from `vocab` to `noun_assertion` is load-bearing.**
`layout::crate_holding` finds a slice's crate with `.find()` over the workspace
members in manifest order and returns the *first* member holding a module of
that name (`cargo-lid-rs/src/layout/mod.rs:443-449`), and a directory becomes a
slice's the moment it holds an `lld.md` (`layout/mod.rs:596-612`). The members
list opens `lid-rs`, `lid-rs-macros` (`Cargo.toml:3`). With
`lid-rs/src/vocab/lld.md` in place, a surviving `lid-rs-macros/src/vocab/lld.md`
would resolve `vocab` to `lid-rs` **silently, with no ambiguity error**, and no
phase of this half could write a line: the acceptance gate would read `lid-rs`'s
target kinds and find them ordinary, the phase row would point at
`lid-rs/src/vocab/`, and the red run would diff the wrong file. Hence
`noun_assertion`, and hence `lid-rs-macros/src/vocab/lld.md` is removed in the
same commit that adds this file.

## Facts, verified 2026-09-12

Every row was read from the tree on this branch (`lld/vocab`, stacked on
`lld/claim` at `9699d21`) or produced by running the command named. Spikes were
run with `rustc 1.98.0` against `--edition 2024`.

| Fact | Where |
|---|---|
| `lid-rs-macros/src/vocab/` holds an `lld.md` and nothing else; no `noun_assertion` directory exists yet | `ls`, workspace-wide |
| No identifier `Traceable` appears in any `.rs` file in the workspace | `grep -rn Traceable lid-rs lid-rs-macros cargo-lid-rs lid-rs-shape` |
| `lid-rs-macros` is a proc-macro crate and names `lid-rs` as its companion | `lid-rs-macros/Cargo.toml:14-15`, `:27-28` |
| `lid-rs-macros` has **no** dev-dependencies and **zero** `#[cfg(test)]` items | `lid-rs-macros/Cargo.toml:17-20`; `grep -rn "cfg(test)" lid-rs-macros/` returns only prose in two `lld.md` files |
| The acceptance gate runs inside the pre-tool edit hook, before the path policy, for every edit of a compile-time slice | `cargo-lid-rs/src/phase/mod.rs:288-296`, `:326-329`, `:334-340` |
| The acceptance file's name and placement: `compile-time-accepted`, beside the slice's document once the directory holds one | `cargo-lid-rs/src/phase/policy.rs:764`, `:789-791`; `cargo-lid-rs/src/layout/mod.rs:245-261` |
| A proc-macro slice's claims are held by its companion; `claims_crate()` returns the companion when there is one | `cargo-lid-rs/src/phase/policy.rs:86-91` |
| Phase 2 may write the claims file and `src/spec/mod.rs`, in **either** seat | `cargo-lid-rs/src/phase/policy.rs:444-461` |
| The companion table's Phases 5 and 7 add `tests/ui` to the row; the own table's do not | `cargo-lid-rs/src/phase/policy.rs:417-441` |
| Phases 3 and 4 may write `src/lib.rs` in **both** seats | `cargo-lid-rs/src/phase/policy.rs:421`, `:438` |
| `expand.rs` and `src/claim/` are in **no** phase row of any slice but their own | `cargo-lid-rs/src/phase/policy.rs:417-424`; `lid-rs-macros/src/claim/lld.md:473` |
| `derive(Spec)`'s expansion is assembled in `claim::expansion`, which returns an `Expansion { claim, free, unwanted }` | `lid-rs-macros/src/claim/mod.rs:125-156`; `lid-rs-macros/src/expand.rs:44-48` |
| A link's target is the path in parentheses when it carries one, else the backticked text unbackticked | `lid-rs-macros/src/claim/mod.rs:607-611`, `:621-629` |
| A behaviour verb with no link after it records the **empty** object; a shape verb without one fails to compile | `lid-rs-macros/src/claim/mod.rs:576-584` |
| `owner` is the target minus its last segment, and only when the target ends in **two capitalised segments** — empty for every other target | `lid-rs-macros/src/claim/mod.rs:586-601` |
| `ClaimMeta` carries `trigger` and `object` as `&'static str`, "as written", with the empty string for a part the claim does not carry | `lid-rs/src/claim/mod.rs:50-80` |
| The hand-authored `#[implements]` edges for `lid-rs-macros`'s items stand in `lid-rs/src/lib.rs`, as `macro_edge!`, `claim_edge!` and `outcome_edge!` blocks | `lid-rs/src/lib.rs:44-78`, `:80-172`, `:174-217` |
| The trybuild harness for a proc-macro slice is a `#[cfg(test)]` unit test in the **companion**'s module | `lid-rs/src/lid_rs_macros/mod.rs:105-112`; `lid-rs/src/claim/mod.rs:214-224` |
| Slice 15's real order was Phases 2–5, then a **Phase 6 commit made by the main session** with the leaves unwired (`3775771`), then the hand swap (`678f44e`), then the Phase 7 gate (`7f03252`) | `git log` |
| `Phase` has no variant 6: `number_of` maps One, Two, Three, Four, Five, Seven | `cargo-lid-rs/src/phase/policy.rs:740-750` |
| `red_verdict` fails on any unvalidated claim **and on any validator that passes** | `cargo-lid-rs/src/phase/mod.rs:912-919` |
| `lid-rs/tests/ui/fail/not_a_spec.stderr` pins rustc's alphabetically-truncated eight-name list of `Spec` implementors; its eighth is `ACrateRootClaimsSliceIsEmpty` | `lid-rs/tests/ui/fail/not_a_spec.stderr:12-20` |
| `#[diagnostic::on_unimplemented]` is stable since 1.78; the workspace pins `rust-version = "1.97"` | `Cargo.toml:8` |

**The spikes.** Run today against the trait shape the `vocab` half defines
(`pub trait Noun: Traceable + Declared`, with the seal in a public
`#[doc(hidden)]` module). The undivided draft asserted most of these without a
date; slices 19 and 21 have landed since.

| Spiked | Result |
|---|---|
| A noun's own path | compiles |
| A noun's **re-export** path, and its declaration path behind that re-export | both compile — the same type, so the bound holds |
| `String`, `u32`, a type implementing neither trait | `error[E0277]` carrying the `#[diagnostic::on_unimplemented]` message, one error each |
| A hand-written `impl Noun for Forged` with no seal impl | `error[E0277]: the trait bound \`Forged: Declared\` is not satisfied` |
| A hand-written `impl Declared` followed by `impl Noun` | **compiles** — the seal is deliberate, not unforgeable |
| An enum **variant** written as-is | `error[E0573]: expected type, found variant` |
| A **function** path | `error[E0573]: expected type, found function` |
| A **module** path | `error[E0573]: expected type, found module` |
| The **empty** target — `assert_noun::<>` | `error[E0283]: type annotations needed` |
| A generic named without arguments | `error[E0107]: missing generics for struct \`Gen\`` |

Two results decide the design. **The re-export case dissolves**: a bound needs
no path comparison, so a declaration path and a re-export path are the same
type and both pass — there is nothing to compare and no registry to consult,
which was the hard problem when this was drafted as a registry check. And **the
three `E0573` rows are the dominant case in this workspace, not an edge**: they
are what a function or module target produces, and they do **not** carry the
shaped message. See the target table below.

## Behaviour

### What this half delivers

- **`derive(Traceable)`** in `lid-rs-macros/src/noun_assertion/`, emitting three
  impls for the annotated type: the recording impl (`Traceable`, with `NOUN`),
  the seal (`::lid_rs::__private::Declared`), and `::lid_rs::Noun`.
- **`noun_assertion`**: one link target, as `ClaimMeta` records it, to the
  tokens that refuse it unless it names a noun — with a variant or method
  target stripped to its owning type, an empty target answering nothing, and
  the rustdoc disambiguator forms refused by name.
- **The wiring**: one call site inside `derive(Spec)`'s expansion, which no
  phase of this branch may write.
- **Fixtures** under `lid-rs/tests/ui/noun_assertion/`, which — unusually —
  Phase 5 *may* write, because the companion table's Phase 5/7 row includes
  `tests/ui` (`policy.rs:434-441`).

### What `noun_assertion` does with a target, which is all of its behaviour

A target as written is not a Rust path. It is whatever `link` returned — the
parenthesised path if the link carried one, otherwise the backticked text
(`lid-rs-macros/src/claim/mod.rs:607-611`) — and it may be the empty string,
because a behaviour verb with no link after it records an empty object
(`:576-584`).

The table below is the function's whole contract, with each form's **actual
frequency among the 177 held claims in this workspace**, counted today (see
"The census"). The undivided draft's version of this table omitted the three
most common forms, which is how it came to describe the burn-down as a matter
of three hand commits.

| Target as written | Triggers | Objects | What it is | What the emission does, and what the reader sees |
|---|---:|---:|---|---|
| *empty* | 1 | 124 | not a target: no link after a behaviour verb | **Emit nothing.** `assert_noun::<>` is `E0283: type annotations needed`, which names neither the claim nor the rule |
| `crate::layout::slice_dir` | 73 | 5 | a free function, or a module — **the emission cannot tell which** | Used as written. The compiler answers `E0573: expected type, found function` or `found module`, **not** the shaped message |
| `crate::layout`, `crate::classify` | 45 | 1 | a module, or a crate-root function | as above |
| `crate::catalog::Report` | 9 | 36 | a type | Used as written. **The only form that can succeed** |
| `derive@crate::Spec`, `mod@…`, `struct@…` | 46 | 1 | a rustdoc disambiguator, not a Rust path | **Refused by the emission**, naming the disambiguator and what to write instead |
| `crate::claim::Language::Free` | 0 | 8 | a variant — two capitalised trailing segments | Stripped to the owning type |
| `crate::layout::Form::of_slice` | 1 | 1 | a method — a lowercase segment under a capitalised one | Stripped to the owning type |
| `` [`Turn`] `` | 2 | 1 | a bare name with no path | Used as written; it resolves in the claim's own module or it does not |
| `macro!` | 0 | 0 | a `!` suffix | **Refused by the emission** |
| a generic without arguments | 0 | 0 | — | Used as written; the compiler answers `E0107: missing generics` |

Each column sums to 177.

**Three consequences the numbers force.**

*The empty target is the common case, not an edge.* 124 of 177 held claims
carry no object link at all, and every one of them must pass. An emission that
wrote a bound for an empty target would break 70% of the workspace's held
claims with a message about type annotations. The empty check is the first line
of the function, not a guard bolted on.

*The dominant non-empty form is not a type, and the shaped message does not
reach it.* 118 of 177 triggers are all-lowercase paths — a function or a module
— and the compiler refuses those at `E0573` before any trait bound is
considered, so `#[diagnostic::on_unimplemented]` never fires. The decision to
shape the message is still right, because it is what a *type* that is not a
noun gets; but the claim "check 13's contract is one compile error naming the
rule and the offending text" is only met for the disambiguator forms (which the
emission refuses by name) and for real types. For a function or module target
the reader gets rustc's own `expected type, found function`, pointing at an
emitted `const _`. **That is a defect of this design and it is not fixed here**
— Open Question 2.

*Stripping is a rare path.* Nine of 354 slots end in a variant or a method.
It is still required, because a variant target is the pattern slice 19's
`CLAIMED_OWNERS` join is built on and an unwanted claim's object routinely
names one. But it is not what this function is mostly for, and the undivided
draft's table — which listed only the stripping cases and the refusals — read
as though it were.

**The stripping rules cannot come from `ClaimMeta.owner`.** Slice 15 records
`owner` only for a target ending in two capitalised segments
(`lid-rs-macros/src/claim/mod.rs:586-601`), so a trigger has none and a method
target has none. `noun_assertion` performs its own reading over the target it
is handed, and `owner` is left to the slice that defined it.

**Refusing the disambiguated forms is not an inconvenience.**
`derive@crate::Spec` and `crate::validates` are the triggers of 46 held claims,
almost all of them slice 15's and slice 19's, and a refusal naming the
disambiguator is the message that tells their author what to write instead.

### What `derive(Traceable)` emits, and why it is three impls

For `#[derive(Traceable)] pub struct Username(String);`:

```rust,ignore
impl ::lid_rs::Traceable for Username { const NOUN: &'static str = "Username"; }
impl ::lid_rs::__private::Declared for Username {}
impl ::lid_rs::Noun for Username {}
```

The seal is addressed through `__private` because that is where the `vocab`
half puts its public address, beside the existing `linkme` re-export
(`lid-rs/src/lib.rs:245-248`); the other two are addressed at the crate root
because that is where the `vocab` half re-exports them, as `lid_rs::Outcome`
already stands beside its own derive (`lid-rs/src/lib.rs:40-42`).

`NOUN` is the type's identifier and not its path. The `vocab` half fixed that
convention for the primitives, which have no module path at all
(`lid-rs/src/vocab/lld.md`, Open Question 2), and disagreeing here would spell a
span's fields two ways.

### Where the red is, and how a proc-macro slice gets one at all

**This is the finding the undivided draft got wrong, and it is worth stating
plainly.** That draft made unit tests over `noun_assertion`'s output the whole
of Phase 5. That cannot work: a proc-macro crate exports only macros, so no
item in `lid-rs` can name a function inside `lid-rs-macros`; `lid-rs-macros`
has no dev-dependencies (`Cargo.toml:17-20`) and zero `#[cfg(test)]` items; and
a `#[validates]` edge written there would register into a test binary that no
`intent_graph!()` invocation reads, so checks 10 and 11 would still name the
claim an orphan. A dev-dependency cycle back onto `lid-rs` is accepted by cargo
(spiked on the trace slice) and buys nothing for the same reason.

**Slice 15's arrangement is the precedent, and it is the one this slice
follows.** It is already in the tree and can be read rather than invented:

- **Implementation edges are hand-authored in `lid-rs/src/lib.rs`**, one per
  (claim, macro item) pair, through the `macro_edge!` / `claim_edge!` /
  `outcome_edge!` blocks (`lid-rs/src/lib.rs:44-78`, `:80-172`, `:174-217`).
  The item each names is the function the claim is kept by, spelled as a
  string — `"lid_rs_macros::noun_assertion::target"` — because a proc-macro
  crate registers nothing and can cite nothing. **`src/lib.rs` is in Phases 3
  and 4's row in both seats** (`policy.rs:421`, `:438`), so these edges are
  *in policy* and need no hand commit; slice 15's were written by hand only
  because the layout at the time gave the phases no route to them.
- **Validation is trybuild**, through a harness that is a `#[cfg(test)]` unit
  test in the companion's module — `lid-rs/src/noun_assertion/mod.rs`, the
  shape `lid-rs/src/lid_rs_macros/mod.rs:105-112` and
  `lid-rs/src/claim/mod.rs:214-224` already have — over fixtures under
  `lid-rs/tests/ui/noun_assertion/{fail,pass}/`.
- **Phase 5's red is the fixtures failing for the honest reason**: the derive
  is unwired, so a `fail/` fixture that ought to be refused compiles, and the
  harness reports the mismatch. This is exactly what slice 15's Phase 5 was
  (`lid-rs-macros/src/claim/lld.md:466-471`), and the fixtures need **no**
  `.stderr` pins to produce it — a `fail` fixture with no pin makes trybuild
  write a `.wip` and report failure. Pins are generated with
  `TRYBUILD=overwrite` after the swap, never hand-edited
  (`docs/intent/publish/lld.md:83`).

Unlike slice 19, the fixtures do **not** need a hand commit: this slice's
companion seat admits `tests/ui` at Phases 5 and 7 (`policy.rs:434-441`).

### The order the phases run in, and the one place it is genuinely tight

Three constraints pull against each other:

1. **A `todo!()` on `derive(Spec)`'s path is a compiler panic on every claim in
   the workspace.** So the wiring cannot land while `noun_assertion`'s leaves
   are `todo!()`.
2. **Phase 7 runs `cargo test --lib`** (`phase/mod.rs:591-601`), which runs the
   trybuild harness. With the emission unwired, the `fail/` fixtures compile
   and the harness fails — so Phase 7 cannot pass before the swap.
3. **The `lid-rs-phase-7` agent implements the leaves and gates in one run**, so
   there is no agent-made stop between "the leaves work" and "the gate runs".

Slice 19 concluded from (3) that "Phase 6 has no commit of its own"
(`lid-rs/src/outcome/lld.md:472-479`). That is true of the *agent*, and it is
not true of the methodology: slice 15's history holds `3775771 phase 6: leaves
for claim — the language, executed, and still unwired`, a **main-session
commit**, followed by `678f44e` (the swap) and only then `7f03252` (the Phase 7
gate). `Phase` has no variant 6 and `phase-check 6` does not exist
(`policy.rs:740-750`), so that commit is ungated by the tool and made by hand —
which is why it can exist at all.

**So the order is: Phases 2–5 by the phase agents; Phase 6 by the main session,
implementing the leaves while they are still unreachable; the swap by hand;
then the Phase 7 gate, run by hand or by the agent over work already done.**
Filling in an unreachable leaf changes no behaviour and cannot break the build,
which is what makes a main-session Phase 6 safe here where it would be
pointless elsewhere.

**And the swap cannot precede the burn-down.** The moment the wiring is live,
every held claim whose trigger is a function, a module or a disambiguator fails
to compile — 176 of 177 of them, by the table above. `cargo check` fails
workspace-wide, which means the swap commit itself does not build unless the
burn-down landed first. That is the real precondition, and it is Open Question
1.

### The burn-down, in the order the dependency actually runs

Four bodies of work outside this slice's phases. **They are ordered by one
dependency: the wiring (item 4) breaks every held claim that does not already
name a noun, so it is performed last, after 1, 2 and 3 are complete
workspace-wide.** The undivided draft stated that sentence about item 1, which
is the fixture repair — a contradiction of the list it introduced, and the
reason the ordering is spelled out here rather than implied by the numbering.

**1. Nouns for each slice's own claims, per slice, before anything else.** 177
held claims across seven slices in four crates: `lid-rs` `claim` 50, `outcome`
18, `trace` 30; `cargo-lid-rs` `catalog` 22, `layout` 16; `lid-rs-shape` 24
(crate-root); `lid-rs-pipeline` 17 (crate-root). Each slice's nouns are that
slice's to name, and the *scale* is the thing the undivided draft understated:
by the census, **168 of 177 triggers and 141 of 177 objects are not types
today**. This is not a mechanical rewrite of paths. A trigger in this workspace
conventionally links *the function that keeps the claim*
(`crate::layout::slice_dir`), where README's rule wants *the noun the claim is
about* (`crate::layout::Slice`) — and the second is a different sentence, often
about a type that does not exist yet. Expect a change branch with its own
phases per slice, not a hand commit.

For slice 15 specifically this is already known to be new design: a proc-macro
crate exports only macros, so no type in `lid-rs-macros` can be named from
`lid-rs`, and its 50 held claims need **new nouns in `lid_rs::claim`**. Their
*objects* are cheap by comparison — `crate::claim::ClaimMeta` and
`crate::claim::Pattern` are already types and need only the derive.

**2. The fixture repair, before item 4.** 77 held claim items across
`lid-rs/tests/ui/` — `claim/fail` 26, `claim/pass` 26, `claim/lexicon` 20,
`outcome/pass` 3, `outcome/fail` 2 — link types that exist nowhere: `Turn`,
`Store`, `Entry`, `Modal`, `Retry`, `Count`, `Feature`, `Log`, `Report`,
`Card`, `Gate`, `Library`, `Phase`, `Request`, `Response`, `Audit`, `Ledger`,
`Canary`, `Index`, `Mark`. rustdoc never runs on a trybuild fixture and slice
15's rule only required a link to *exist*. Three classes break: `pass` fixtures
stop compiling; `fail` fixtures gain an error their `.stderr` does not pin; and
each of the 20 crates of the lexicon fixture workspace under
`tests/ui/claim/lexicon/` holds one held claim with an asserted message. **The
undivided draft named only `tests/ui/claim/`; `tests/ui/outcome/` landed since
and is in the same position.** They need fixture-local nouns — *not*
`#[lid(free)]`, which disarms the rules they exist to pin, a trap slice 15
documented.

**3. `cargo-lid-rs`, `lid-rs-shape` and `lid-rs-pipeline`'s 79 held claims.**
None of those crates is this slice's own or its companion, so `seat_of` refuses
every path in them before any table is consulted (`policy.rs:109-118`). A
`Slice`, a `Command`, a `Report`, a `Finding`, a `PhaseCommit` already exist as
types in those modules and are the obvious candidates; what each trigger
*becomes* is a question for that slice.

**4. The wiring** — one call site inside `derive(Spec)`'s expansion. The
natural seam is `claim::expansion`, which already assembles the emission and
already carries slice 19's `unwanted` beside the claim
(`lid-rs-macros/src/claim/mod.rs:125-156`); the assertion becomes a third thing
it returns, or a call `expand::derive_spec` makes beside it
(`lid-rs-macros/src/expand.rs:44-48`). Both files are in **no** phase row of
this branch (`lid-rs-macros/src/claim/lld.md:473`), so this is a hand commit
either way, and it is the same hand commit slice 15 made at `678f44e` and slice
19 made at `da047cd`.

Counts are `#[derive(Spec)] struct X;` items less `#[lid(free)]` per file, read
2026-09-12; the catalog's census would make them a command rather than a
reading.

### What this half is not

- **Not the traits.** `Traceable`, `Declared`, `Noun`, the primitive impls and
  the crate-root re-exports are the `vocab` slice's
  (`lid-rs/src/vocab/lld.md`). No phase of this branch writes
  `lid-rs/src/vocab/`, and the acceptance gate would refuse it anyway.
- **Not the policy attributes.** `#[trace(redact)]`, `#[trace(skip)]` and
  README §6.5's field selection describe what a **span** records; slice 20
  builds spans. Deferred 1.
- **Not the LLD rewrite.** Deferred 2 on the `vocab` half.
- **Not a `TRACEABLES` registry.** The bound needs none.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| HLD row 17 is split, and this half is `noun-assertion` in `lid-rs-macros` | `lid-rs-macros/src/noun_assertion/`, with `lid-rs/src/noun_assertion/` as its companion for claims, harness and fixtures | One `vocab` slice spanning both crates; put the assertion in `lid-rs` and have the derive call into it; two slices split derive-vs-assertion | One slice spanning both crates is what the undivided draft was: its Phase 5 could not exist, and its acceptance file blocked the traits — which are ordinary Rust — behind a human act they do not need. A proc-macro cannot call into `lid-rs` at expansion time, so the assertion has to be here. Splitting further is Open Question 3, not refused. |
| The directory is `noun_assertion`, not `vocab` | Rename, and delete `lid-rs-macros/src/vocab/lld.md` in the same commit | Keep both named `vocab`; name the `lid-rs` half something else | `crate_holding` takes the **first** member holding a module of that name (`layout/mod.rs:443-449`) over a members list opening with `lid-rs` (`Cargo.toml:3`), with no ambiguity error anywhere. Two `vocab` directories give this half `lid-rs` as its crate: the acceptance gate reads the wrong target kinds, the phase row points at the wrong directory, and the red run diffs the wrong file — all silently. Renaming the reachable half instead would leave the buildable slice mis-named for the sake of the blocked one. |
| A sealed marker separates noun from recordable | `Noun: Traceable + Declared`, all three emitted by `derive(Traceable)` | Bound on `Traceable`; a registry comparison of target paths; leave the predicate to slice 18's rule V | README §6.5 gives `Traceable` primitive impls (`README.md:519-522`), so a bound on it accepts `String` — spiked and confirmed. A registry has no answer for a re-export, where a bound has nothing to compare. Leaving it to slice 18 leaves this slice a component rather than a user-visible operation. The seal is what makes "a noun is declared, not stumbled into" true rather than conventional. |
| The seal is `Declared`, and the guarantee is deliberateness | `::lid_rs::__private::Declared`; the derive is what normally writes it | `Sealed`; claim the seal is unforgeable; a private-witness pattern | The undivided draft wrote `Declared` in its prose and `Sealed` in the *chosen* cell of its decisions table, and Phase 3 reads the table. `Declared` is chosen; the table's `Sealed` was the wrong cell. Spiked: `impl Declared` by hand compiles, so the strong claim is false and this document does not make it. The `vocab` half owns the trait; this row exists so the emission and the trait agree on the name. |
| Enforcement is check 13, at compile time, at the claim | The bound `derive(Spec)` emits at an emitted `const _` | A new numbered check over the registries; a `cargo lid-rs` subcommand | README §3.5 and check 13's row already assign the rule to the derive (`README.md:417-423`, `:599`), and the numbered register runs 13–26 with nothing free. A registry check would run after the fact and could not see a re-export. |
| Both slots, not the trigger alone | Trigger and response object | Trigger only; object only | README §3.5 says "a trigger **or** response slot" (`:417-423`). Slice 15's claims already link `crate::claim::ClaimMeta` and `crate::claim::Pattern` as objects, so those become nouns anyway; excluding the object would leave 36 type-shaped object targets unchecked for no saving. |
| The empty target emits nothing | First line of the function | Refuse an empty object; require every claim to name an object | 124 of 177 held claims record an empty object, because a behaviour verb with no link after it is legal and records the empty string (`lid-rs-macros/src/claim/mod.rs:576-584`). Refusing it would break 70% of the workspace's held claims with `E0283: type annotations needed`, which names neither the claim nor the rule. Requiring an object is a change to slice 15's language, not to this slice's emission. |
| The message is shaped where it can be | `#[diagnostic::on_unimplemented]` on `Noun` (the `vocab` half's item); the emission refuses the disambiguator and `!` forms by name | Accept every raw compiler error; refuse function and module targets in the emission too | Spiked: the attribute produces one error with the shaped message for any *type* failing the bound. It does **not** reach a function or module target, which rustc refuses at `E0573` first — and that is the dominant form today, which Open Question 2 records rather than papers over. Refusing lowercase targets in the emission was considered and rejected: the emission cannot tell a free function from a module from a lower-case type alias, so it would refuse by spelling and be wrong for every project that names a type in `snake_case`. |
| A variant or method target is stripped to its owning type | Reading performed by this slice over the target it is handed | Read `ClaimMeta.owner`; refuse variant targets; strip in `claim::parse` instead | `owner` is recorded only for a target ending in two capitalised segments (`lid-rs-macros/src/claim/mod.rs:586-601`), so a trigger has none and a method target has none — it answers a different question for a different slot. Refusing variants would break slice 19's unwanted-claim convention, which the `CLAIMED_OWNERS` join is built on. Stripping in `parse` would change what the registry records for every claim in the workspace. |
| The burn-down is per slice with its own phases, not a hand commit | Item 1 above: each slice names its own nouns on its own branch | Three numbered hand commits, as the undivided draft planned; a second `#[lid(free)]`-style ramp mark; per-slice as each is next touched, with the check disarmed meanwhile | The census says 168 of 177 triggers and 141 of 177 objects are not types, and a trigger that names a function is a *different sentence* from one that names a noun — so this is design work with claims of its own, not a path rewrite. A second ramp would make the rule aspirational, which is exactly what slice 15's ramp was designed to avoid. Leaving the check disarmed for a long time is constraint 3's "a check that does not gate". |
| Phase 6 is a main-session commit, and the swap follows it | Phases 2–5 by agents; Phase 6 by hand, leaves unwired; the swap by hand; then Phase 7 | The swap before Phase 5, as slice 19 ordered its derive; the swap after Phase 7; make the harness `#[ignore]`d until the swap | A `todo!()` reachable from `derive(Spec)` is a compiler panic on all 471 claims, so the swap cannot precede the leaves; Phase 7 runs `cargo test --lib` and the harness fails while unwired, so it cannot follow Phase 7 either. `Phase` has no variant 6 and `phase-check 6` does not exist (`policy.rs:740-750`), which is precisely why a Phase 6 commit can be made by hand — slice 15's `3775771` is the precedent, and its history (`3775771` → `678f44e` → `7f03252`) is what actually ran, not the order its own LLD predicted. Ignoring the harness removes the only validation this slice has. |

## Open Questions & Future Decisions

### Open

**1. Whether the burn-down completes before this slice runs, or the assertion
ships behind a ramp — and this is a hard precondition, not a preference.** The
swap commit does not compile while 176 of 177 held claims name a non-noun, so
either the burn-down lands first or the emission is conditional. Three shapes:

- *Burn down first.* Seven slices' claims rewritten, each on its own branch,
  before this slice's Phase 6. Honest and slow, and it needs
  `derive(Traceable)` to be usable while this slice is mid-flight — which it is
  not, because the derive's leaves are Phase 6's.
- *A ramp knob.* The derive already walks up to the workspace manifest to read
  `docs/intent/lexicon.toml` (`AMemberFindsItsWorkspacesLexicon`), so a
  per-project `nouns = "off" | "warn" | "error"` key is readable from inside
  the derive by machinery that exists. This is the mechanism README's other
  ramps use, and it lets the assertion ship before the burn-down finishes.
- *Split the derive out.* Ship `derive(Traceable)` as its own slice, burn down
  against it, then ship the assertion. This is Open Question 3.

**This document's recommendation is the ramp**, on the same reasoning that made
`#[lid(free)]` a counted mark rather than a flag day: a rule that lands with
176 violations lands as a revert. It is recorded as a recommendation because the
decision changes the lexicon file's schema, which is slice 15's.

Cost of getting it wrong: the swap commit fails `cargo check` workspace-wide
and the branch cannot be committed at all, which surfaces after Phase 6 rather
than at Phase 1.

**2. What a function or module target should say.** 118 of 177 triggers are
all-lowercase paths, and the reader gets `error[E0573]: expected type, found
function` at an emitted `const _` — no claim name, no rule, no remedy. Check
13's contract is one compile error naming the rule and the offending text, and
this design does not meet it for the most common case. Two candidates, neither
free: refuse lowercase-tailed targets in the emission (wrong for any project
that names a type in `snake_case`, and the emission cannot tell a type alias
from a function), or emit a `#[deprecated]`-style marker alongside so a second
diagnostic names the rule. Not blocking Phase 2 — no claim's truth changes with
the answer — but it should be settled before Phase 6, because it changes what
`noun_assertion` returns.

**3. Whether `derive(Traceable)` and the assertion should be two slices.** They
are two user-visible operations — "a type declares itself a noun" and "a claim
may name only a noun" — and they have opposite ordering needs: the derive must
be usable *before* the burn-down, and the assertion must land *after* it. As one
slice, both are Phase 6 leaves of the same commit and the burn-down has nowhere
to stand. Splitting costs a second acceptance file and a second set of phases;
not splitting costs Open Question 1's ramp. The human's decision, recorded here
because it is a consequence of the split already taken and not a reopening of
it.

**4. Whether a claim item can itself be a noun.** Two held claims in
`cargo-lid-rs` link another claim's struct as their trigger (`catalog`'s
`AnUnbuiltCommandRefusesWithTheCatalogsReason`, `sync`'s
`PackageNamesEveryPublishingMemberInOneInvocation`). A claim struct is a unit
struct and could carry `derive(Traceable)`, which would make "a claim" a noun
of the `claim` slice. Whether that is a vocabulary or a cross-reference is a
question for item 1 of the burn-down, and the answer changes two sentences.

### Deferred

1. `#[trace(redact)]`, `#[trace(skip)]` and README §6.5's field selection.
   Cause: they describe what a **span** records, and there are no spans until
   slice 20.
2. The seventeen LLDs' prose vocabulary links, and which slices gain a
   vocabulary file. Cause: a rewrite from zero, per slice as each is next
   touched. Held on the `vocab` half (`lid-rs/src/vocab/lld.md`, Deferred 2).
3. A rule that an LLD must link its nouns. Cause: check 2 fires only on a
   written link that fails to resolve (`README.md:591`), so the positive rule
   needs `lld-check` to read the document, which is slice 13's.
4. README's amendment and the HLD's, for this half: §3.5's "vocabulary type"
   needs the marker named and the empty-object exemption stated; check 13's row
   needs the disambiguator refusal. HLD row 17 (`hld.md:196`) owes the split
   into two slices. Cause: no gate step reads README against the code, so
   nothing stops on it. `README.md` is at the workspace root, outside every
   crate, so `seat_of` refuses it before any table (`policy.rs:109-118`) — a
   main-session commit, landing with the `vocab` half's Deferred 4 as one
   documentation commit.
5. The `.stderr` pins for this slice's fixtures. Cause: they cannot be written
   until the swap makes the diagnostics real. Generated with
   `TRYBUILD=overwrite` in or immediately after the swap commit, never
   hand-edited (`docs/intent/publish/lld.md:83`).

## What lands by hand, and in what order

Five things, and the order matters more here than on any slice so far. Each was
checked against `policy.rs` and against what the phase after it needs.

**0. `lid-rs-macros/src/noun_assertion/compile-time-accepted`, before Phase 2 —
the human's, and this slice does not start without it.** The pre-tool hook runs
`acceptance_gate` before the path policy for every edit
(`phase/mod.rs:326-329`, `:334-340`), so a phase agent spawned without this file is refused
its first edit and can do nothing but stop. The path is the layout's answer, not
a spelling: `layout::intent_file` places it beside the slice's document once the
directory holds one (`layout/mod.rs:245-261`), which it does as soon as this
file exists.

**1. `pub mod noun_assertion;` in `lid-rs/src/lib.rs`, and
`lid-rs/src/noun_assertion/mod.rs` holding `pub mod spec;` — between Phases 2
and 3.** Phase 2's row is the claims file and `src/spec/mod.rs` in either seat
(`policy.rs:444-461`), so it can write `lid-rs/src/noun_assertion/spec.rs` and
cannot declare it; Phase 2's check is `cargo check --all-targets`, which passes
happily over a file no module compiles, so **check 13 never reads the claims**
and the failure surfaces at Phase 5 as "nothing registers". `src/lib.rs` *is* in
Phase 3's row, so the declaration could land there — and doing so puts check
13's rejections inside an agent that may not open the claims file to fix them.
Slices 18 (`4b036c7`), 19 (`705ab1b`) and 21 (`04f1388`) all landed it by hand
between Phases 2 and 3 and all three then had every claim accepted first time.
Do that.

The companion module carries no `lld.md` — a slice's document is never under
its companion (`ASlicesDocumentIsNeverUnderItsCompanion`) — so
`lid-rs/src/noun_assertion/mod.rs` opens with a `//!` summary and a pointer to
this file, as `lid-rs/src/lid_rs_macros/mod.rs:1-13` does.

**2. The burn-down, items 1–3 of the list above, complete workspace-wide —
before Phase 6.** Or Open Question 1's ramp, in which case this item becomes
"the ramp knob, in slice 15's lexicon reader". One of the two must happen; the
swap does not compile otherwise.

**3. Phase 6: the leaves, still unwired — a main-session commit.** `phase-check
6` does not exist (`policy.rs:740-750`) and the `lid-rs-phase-7` agent fuses
Phase 6 and Phase 7 into one run, so this commit is made by hand and gated by
nothing but `cargo check`. Filling in a leaf nothing calls changes no behaviour.
Slice 15's `3775771` is the precedent.

**4. The swap: one call site in `lid-rs-macros/src/claim/` or
`src/expand.rs`.** Both are in no phase row of this branch
(`lid-rs-macros/src/claim/lld.md:473`). The `.stderr` pins are generated in this
same commit with `TRYBUILD=overwrite`, once the diagnostics exist to be pinned.
Slice 15's `678f44e` and slice 19's `da047cd` are the precedents.

**Then the Phase 7 gate**, over work already done. `phase-check 7` is the floor;
this workspace adds `mdbook build book` by hand.

**One cascade to expect rather than rediscover.** A claim's trigger clause must
contain an intra-doc link (`README.md:417-423`), and the link must resolve from
the public `spec` module — so every item this slice's claims name ends up `pub`,
and a public doc linking a private item is a `private_intra_doc_links` warning
this workspace does not suppress. Between item 1 landing and Phase 3 creating
the items, `cargo doc` is **broken**: the claims file compiles and its links
point at nothing. That clears at Phase 3, and Phase 2's check does not run
rustdoc, so nothing stops. Note also that `mod claim;` is private in
`lid-rs-macros/src/lib.rs:5` and `mod noun_assertion;` will be too, so the
items the claims link are in `lid-rs`, not here.

**And one fixture that is nobody's slice.** `lid-rs/tests/ui/fail/not_a_spec.stderr`
pins rustc's alphabetically-truncated eight-name list of `Spec` implementors,
whose eighth entry today is `ACrateRootClaimsSliceIsEmpty` (`:12-20`). A new
`lid-rs` claim sorting before it displaces a name and fails that fixture at
Phase 7; claim names beginning `A` followed by an early letter are the ones to
watch. It is re-blessed with `TRYBUILD=overwrite`, never hand-edited — slice 21
did exactly this at `2c9c59d`.

## Notes for Phase 5's red set

Six things a red test of this slice gets wrong by default.

1. **There is no unit test of `noun_assertion`.** Nothing outside
   `lid-rs-macros` can name it, and `lid-rs-macros` has no test harness of any
   kind. Every validation is a trybuild fixture through `derive(Spec)`, and the
   `#[validates]` attribute goes on the harness in
   `lid-rs/src/noun_assertion/mod.rs`. A Phase 5 that writes unit tests here
   will find no way to compile them.

2. **The red is the unwired derive, and it is honest.** With no wiring, a
   `fail/` fixture that ought to be refused compiles, and the harness reports
   the mismatch. Do not hand-write `.stderr` pins to manufacture a red: the pin
   would be a guess at a diagnostic that does not exist, and it is regenerated
   in the swap commit anyway.

3. **A harness whose glob matches nothing does not fail.** trybuild calls
   `message::no_tests_enabled()` and records zero failures, so a harness written
   before its fixtures is a vacuous green that `red_verdict` refuses
   (`phase/mod.rs:912-919`). Slice 19 paid an amendment for this
   (`lid-rs/src/outcome/lld.md:498-508`). Here the fixtures are in Phase 5's own
   row (`policy.rs:434-441`), so write them in the same commit as the harness.

4. **One harness cannot be red for twelve reasons at once.** Every claim of this
   slice will cite the same trybuild test, and trybuild reports the whole glob
   as one pass or fail. That satisfies `red_verdict`, which only asks whether
   each cited claim has a validator and whether that validator passes — but it
   means check 12 has one test to kill mutants with. Slice 15 met this and
   answered it by making each fixture *file* about one rule
   (`clause.rs`, `links.rs`, `mark.rs`, `order.rs`, `sentence.rs`,
   `validator.rs`) so a mutant that breaks one rule breaks one file's expected
   output; follow that, one file per row of the target table.

5. **A claim implemented by data has no reachable red.** The refusal messages
   are the obvious trap: written as `const`s, a claim about the wording of the
   disambiguator refusal is true from Phase 3 onward. Give each such claim a
   function to cite — the same move `document_path` made on the trace slice
   (`lid-rs/src/trace/lld.md`, Notes 1).

6. **The red set is scoped to claims added since the last `phase 7:` commit**
   (`phase/mod.rs:796-812`). On a branch that has already gated once — which
   this one will have, if Open Question 3 splits it — a Phase 5 that adds no
   claim fails with an empty red set rather than passing vacuously.

## Shape

| Item | Role |
|---|---|
| `lid_rs_macros::noun_assertion` | The module: `derive(Traceable)`'s expansion and the assertion. Private, as `mod claim;` is (`lid-rs-macros/src/lib.rs:5`). |
| `lid_rs_macros::noun_assertion::derive_traceable` | Flow: the three impls for one annotated item — the recording impl, the seal, and `Noun` — or the refusal for an item that cannot carry them. |
| `lid_rs_macros::noun_assertion::noun_impls` | Work leaf: the three `impl` blocks for a named type, addressed as `::lid_rs::Traceable`, `::lid_rs::__private::Declared` and `::lid_rs::Noun`. |
| `lid_rs_macros::noun_assertion::noun_name` | Work leaf: the `NOUN` const's value — the type's identifier, matching the convention the `vocab` half fixed for the primitives. |
| `lid_rs_macros::noun_assertion::assertion` | Flow: one target as `ClaimMeta` records it, to the tokens that refuse it unless it names a noun. The one dispatch of this slice, over the rows of the target table. |
| `lid_rs_macros::noun_assertion::target` | Work leaf: the target reduced to the path the bound is written against — stripped of a trailing variant or method segment, and unchanged otherwise. |
| `lid_rs_macros::noun_assertion::disambiguated` | Work leaf: whether a target carries a rustdoc disambiguator, and which — the refusal names it, because `derive@crate::Spec` is 46 of the 177 held triggers and its author needs to be told what to write instead. |
| `lid_rs_macros::noun_assertion::bound` | Work leaf: the `const _: () = { fn assert_noun<T: ::lid_rs::Noun + ?Sized>() {} let _ = assert_noun::<#path>; };` for one resolved path. |
| `lid_rs::noun_assertion` | The companion module in `lid-rs`: this slice's claims, its trybuild harness, and nothing with runtime behaviour — the shape `lid-rs/src/lid_rs_macros/mod.rs` already has. |
| `lid_rs::noun_assertion::spec` | This slice's claims. In the companion because a proc-macro crate registers nothing (`policy.rs:86-91`). |
| `lid_rs::noun_assertion::tests::a_claim_naming_a_non_noun_fails_to_compile` | The trybuild harness over `lid-rs/tests/ui/noun_assertion/{fail,pass}/`, a `#[cfg(test)]` unit test in the lib as README §5.2 requires. Every claim of this slice cites it. |
| `lid_rs::lib`'s `noun_edge!` block | The hand-authored `#[implements]` edges, one per (claim, macro item) pair, as `macro_edge!` and `outcome_edge!` already are (`lid-rs/src/lib.rs:44-78`, `:174-217`). In Phases 3 and 4's row in the companion seat (`policy.rs:438`), so in policy and not a hand commit. |
| `lid_rs::Traceable` (the derive) | The re-export beside `lid_rs::Spec` and `lid_rs::Outcome` (`lid-rs/src/lib.rs:42`), occupying the macro namespace where the `vocab` half's trait occupies the type namespace — so one `use lid_rs::Traceable` reaches both. |

The `#[proc_macro_derive(Traceable)]` entry point in
`lid-rs-macros/src/lib.rs` and the re-export in `lid-rs/src/lib.rs` are both in
Phases 3 and 4's row in their respective seats (`policy.rs:421`, `:438`), so
neither is a hand commit. The **wiring into `derive(Spec)`** is the only
emission-side edit that is not, and it is item 4 of "What lands by hand".

**A note on doc links in this document.** It contains no intra-doc link. This
file is included by `#![doc = include_str!("lld.md")]` in
`lid-rs-macros/src/noun_assertion/mod.rs` from Phase 3 onward, and from that
commit every link in it must resolve under `-D
rustdoc::broken_intra_doc_links`. Locations are plain `file:line` text, which is
the convention the trace document settled on for the same class of reason.

## References

- README [§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html) — the rule,
  what the derive extracts, and check 13's "unlinked noun".
- README [§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html) — rule V,
  and why a bound on `Traceable` is not the predicate.
- README [§6.5](https://bradvoth.github.io/lid-rs/spec/traced.html) — the
  recording trait, and the primitives that have it with no policy.
- `lid-rs/src/vocab/lld.md` — the other half of HLD row 17: the traits, the
  seal, the primitive impls and the crate-root re-exports this slice's emission
  addresses.
- `lid-rs-macros/src/claim/lld.md` — slice 15: the targets, the ramp, the
  fixture trap, the *pin, then swap* sequence, and the fact that `expand.rs` is
  in no phase's path.
- `lid-rs/src/outcome/lld.md` — slice 19: the fixtures-before-Phase-5 deadline,
  and the ordering corrections a proc-macro slice's hand commits attract.
- `lid-rs/src/trace/lld.md` — slice 21: the plain-`file:line` convention, and
  the precedent for giving a data-shaped claim a function to be cited on.
