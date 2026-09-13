# A signature keeps the promise its claim makes

## Context and Design Philosophy

Tier 1 conformance (README [§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html))
is four checks that "read the claim's content — `SpecMeta`, extracted by the
derive at compile time — and compare it to signatures and enums found
syntactically by the shape pass. They're the first checks that use what a
claim *says*, not just that it exists."

Two of the four can be built now and two cannot, and the line between them is
drawn by README itself:

> **Mechanics.** `#[derive(Outcome)]` — which every error type at a traced
> boundary must have anyway, for span recording — also registers each variant
> into `OUTCOMES`. So E1 and E2 are registry intersections, no syntax needed.
> S1 and S2 join `SpecMeta` with the shape pass's `signatures()`, which returns
> each function's parameter and return type tokens.

**So this slice is E1 and E2, and it is not S1 and S2.** Checks 19 and 20 need
`lid_rs_shape::signatures()`. When this document was first written slice 18 had
not built it; **it has since** — slice 18 is complete through Phase 7
(`585b212`) and its `signatures` answers every function's parameter and whole
written return type. So S1 and S2 are no longer *blocked*, only *out of scope*:
they are the natural next change on this slice (Deferred 1), and the cut below
is now a scope choice rather than a dependency. Checks 21 and 22 need no syntax at all: one is a
compile-time assertion at the claim, the other an intersection of two
registries. The cut is not a convenience, it is where the dependency actually
falls.

**This slice's checks are inert in this workspace, and that is the decision the
document turns on.** Counted 2026-09-11 over the five members' `src`:

| Fact | Count | How counted |
|---|---|---|
| Registered claims (`derive(Spec)`) | 395 | `grep -rh "derive(Spec)"` |
| Marked `#[lid(free)]` | 309 | `grep -rh "lid(free)"` |
| **Unwanted claims** (`If …, then` opener) | **0** | `grep -rn "/// *If "` over every `spec.rs` |
| Enums named `*Error` | 0 | `grep -rn "enum [A-Za-z]*Error"` |
| `impl std::error::Error` | 0 | `grep -rn "impl .*std::error::Error"` |
| `fn` returning `Result<_, _>` | 295 | `grep -rn "fn .*-> *Result<"` |
| …of which `Result<_, String>` | 261 | the same, filtered on `, String>` |
| **Error enums actually returned** | **1** (`Halt`) | the remaining 34, whose error types are `Halt` (enum), `Stop`, `Refusal` |
| …**owning a claimed variant** | **0** | no claim's `owner` names `Halt` |

The last two rows are the ones that decide this slice, and neither of the two
sweeps above them finds `Halt`. "No error enums" would be the wrong summary;
"one error enum, no claimed variants" is the right one.

Thirty-four of those 295 signatures return an error that is not `String`, and
between them they name three types: `Halt`, `Stop` and `Refusal`. One of the
three is an enum — `Halt`
(`cargo-lid-rs/src/headless_canopy_agent/turn.rs:249`, four variants); the other
two are structs. So the workspace **does** hold an error enum, and a sweep for
`*Error` or `impl std::error::Error` misses it, because it is named for what it
means rather than for its category. But no claim names any of its variants, and
README's scope rule is explicit:

> **On E2's scope.** It applies to enums that own at least one claimed variant —
> `AuthError`, not `StoreError`.

`Halt` owns zero claimed variants, so E2 excludes it by its own rule. With zero
unwanted claims, E1 has nothing to iterate. **Both checks would ship correct and
find nothing here** — not because the workspace is clean, but because it has no
subject matter. The honest statement is "one error enum, no claimed variants,"
not "no error enums."

That is in tension with two standing rules — tenet 2, "a gate that exists,
gates," and README constraint 3, "every check gates, or it gets deleted. A
report nobody reads is worse than an absent check" — and the tension is the
open question below, not something this document resolves.

## Behaviour

### What this slice is not

- **Checks 19 and 20 (S1, S2).** They need `signatures()` from slice 18.
- **`conformance.level` and `aliases`.** HLD row 19 names them; they cannot be
  read from where these checks run (below), so they are deferred with a cause
  rather than a promise.
- **Span recording.** README calls `derive(Outcome)` the thing "every error type
  at a traced boundary must have anyway, for span recording". The recording is
  slice 20's. This slice builds the derive and the registration only, and says
  so in the trait's own documentation so slice 20 inherits a stated surface
  rather than an implied one.

### The two rules, as README states them

> | 21 | **Unwanted claim names a non-variant** (E1) | An *unwanted* claim's response object isn't a variant of an `#[derive(Outcome)]` enum. The claim promises an error that doesn't exist as one. |
> | 22 | **Variant with no claim** (E2) | A variant of a claim-owning error enum is the response object of no *unwanted* claim. **Someone added a way to fail that no requirement asked for.** |

The parts a claim contributes already ship. `ClaimMeta`
(`lid-rs/src/claim/mod.rs`) carries `pattern`, `object` and `owner`, and
`Pattern::Unwanted` is the opener `if` records
(`lid-rs-macros/src/claim/mod.rs:450`). `owner` is documented and implemented as

> The object's owner: the target with its last segment removed, as written,
> when the target ends in two capitalised segments — `AuthError` from
> `AuthError::Backend`. Empty for any other target, which names no variant.

so `owner` keeps the **full remaining path** (`crate::auth::AuthError`), while a
template's `{owner}` binds to its last segment. Both facts matter below: the
path is what resolves, the segment is what a signature spells.

### E1 needs no registry at all

E1 is not an intersection. For an unwanted claim whose object is
`Owner::Variant`, `derive(Spec)` can emit a const block that bounds `Owner` on
`Outcome` and matches the variant. Spiked with `rustc`, 2026-09-11:

- a nonexistent variant gives `E0599: no variant … named X found for enum Owner`;
- an enum without the derive gives `E0277`.

Compile-time, at the claim, with a diagnostic naming the offence. This is the
shape check 14 already uses — a const assertion emitted beside the claim — and
it costs no registry, no fourth slice, and no test.

It also means **E1 is not a test and cannot be one**. A claim that fails E1 does
not compile, so there is no run in which a check reports it. The claim about E1
is therefore validated the way slice 15 validates the language: by trybuild
fixtures, one `fail` per diagnostic and one `pass`.

### E2 is a registry intersection, and the join key was the hard part

E2 needs both sides enumerable: the unwanted claims (from `SPECS`, already
there) and the variants of claim-owning enums (from `OUTCOMES`, new). The risk
was never the intersection, it was the key. `SpecMeta.name` states the
principle every existing join follows:

> The join key: every `Edge` produces its `spec` field from the same associated
> const, so the two sides of a join can never disagree about naming.

E1/E2 would otherwise be the first join in the system comparing an
author-typed doc-link path (`crate::auth::AuthError`, as the claim spells it)
against a `module_path!()`-derived string (as the enum's registration spells
it) — two independently constructed strings, which is exactly what that
sentence exists to forbid.

**The resolution, spiked 2026-09-11:** `Outcome` carries

```rust
const NAME: &'static str = concat!(module_path!(), "::", stringify!(Enum));
```

and the claim side emits `<Owner as Outcome>::NAME` — resolving the author's
path as a *type* and reading the key off the enum's own definition. Both sides
then come from one const and cannot disagree. It is robust to re-exports and to
path spelling for the same reason the trait bound in E1 is: the compiler
resolves the path, and the constant travels with the definition.

### `OUTCOMES` is a fourth distributed slice, and the canary is built around three

`lid-rs/src/registry/` declares exactly three: `SPECS`, `IMPLEMENTATIONS`,
`VALIDATIONS`. The canary is built around that triple, not around "the
registries" in general — `triple_is_present(specs, impls, validations)` takes
three parameters, `present()` passes the three, and `registry_dump_for_tooling`
emits three line kinds (`SPEC`, `IMPL`, `VALID`) which `cargo-lid-rs`'s mutation
machinery parses.

So adding `OUTCOMES` is not additive-by-default. Three questions it raises,
each belonging to a slice that is not this one:

| Question | Whose |
|---|---|
| Does the canary cover a fourth slice, and does `CanaryStripped` mean anything for a workspace with no outcomes at all? | `registry` — `lld/registry--outcomes` |
| Do the two emitted tests join `intent_graph!()`, and does the dump gain an `OUTCOME` line? | `graph` — `lld/graph--outcome-checks` |
| Where do `derive(Outcome)` and `derive(Spec)`'s E1 emission go? | `claim` — `lld/claim--outcome-derive` |

An empty `OUTCOMES` must be distinguishable from a stripped one, and it is not
obvious that the canary's answer for the triple transfers: a workspace can
legitimately have zero outcomes, which is not true of specs. That is the
registry slice's to answer, and this document only records that it must.

### The checks scope to the invoking crate

The graph slice already settled this and the reasoning transfers unchanged:

> The checks therefore scope to the invoking crate: `Spec::NAME` begins with the
> defining crate's name, and the macro passes `env!("CARGO_CRATE_NAME")` at
> expansion.

`GraphChecksScopeToTheCurrentCrate` is a claim of that slice. E2 must scope the
same way and for the same reason — a consumer's binary links `lid-rs`'s
registrations alongside its own, and an unscoped E2 would report this crate's
unclaimed variants as the consumer's failure. E1 needs no scoping: it fires at
the claim's own definition site, in whichever crate that is.

### Failure paths

Stated because a check is defined as much by what it declines to judge as by
what it fails.

| Input | E1 | E2 |
|---|---|---|
| Unwanted claim, `owner` empty (object names no variant) | Emits nothing. An empty owner means the object was not `X::Y`, so there is no variant to assert. README's rule is about a claim that *names* a non-variant, not one that names no variant at all. | Not a claimed variant; contributes nothing to the claimed set. |
| `owner` names a type that is not an enum | The bound fails: `E0277`, the type does not implement `Outcome`. The diagnostic names the type. | Unreachable — such a type registers nothing into `OUTCOMES`. |
| `owner` names an enum in another crate, without the derive | `E0277` at the claim. A foreign enum cannot be given the derive, so the claim must be reworded or the dependency wrapped; the diagnostic is the signal, not a special case. | The enum registers nothing, so it owns no claimed variant and E2 excludes it. |
| Variant exists, on an enum *without* `derive(Outcome)` | `E0277` — the bound, not the variant match, is what fails, and it fails first. | The enum is absent from `OUTCOMES` entirely, so E2 never considers it. |
| Enum with the derive, zero claimed variants | n/a | **Excluded by README's scope rule.** This is what `Halt` becomes the day it takes the derive, and why taking it costs nothing. |
| Enum with the derive, some claimed and some not | n/a | **Fails**, naming each unclaimed variant. This is the rule's whole point. |

The last two rows are the rule; the middle rows are all the same answer, which
is the argument for E1 being a bound rather than a registry comparison. One
mechanism produces every refusal, and rustc writes the message.

### Where this slice's work can be written, and what it cannot reach

The slice's own crate is the one holding its document. This document is at
`lid-rs/src/outcome/lld.md`, so the own crate is **`lid-rs`** — a plain library,
not a proc-macro crate, so this slice has **no companion** and its phases may
write nothing outside `lid-rs`. Verified against the policy: a companion exists
only when the own crate declares a `proc-macro` target, and `seat_of` refuses
every path outside the slice's crates before any table is consulted.

That places the derive work outside this slice, and it decides the change
branches:

- `derive(Outcome)` and `derive(Spec)`'s E1 emission live in `lid-rs-macros`.
  A slice of that crate is **compile-time** and the policy refuses every edit
  until the human commits `compile-time-accepted` beside its document.
  `lid-rs-macros/src/claim/compile-time-accepted` **already exists**, so
  landing this as a change to the **`claim`** slice needs no new human act;
  a *new* proc-macro slice would need a new acceptance file, and would block.
- The `claim` slice's companion is `lid-rs` (`lid-rs-macros/Cargo.toml`
  `[package.metadata.lid_rs] companion = "lid-rs"`), so a change on that slice
  can write both `lid-rs-macros/src/claim/**` and `lid-rs/src/claim/**` — which
  is where the derive's own claims and fixtures already live.

### `conformance.level` cannot be read from where these checks run

HLD row 19 names `conformance.level` and `aliases`. Both are unreadable today,
and not for the reason a reader would guess. Verified:

- `Project::workspace_setting`, `Project::root_package_setting` and
  `setting_in` are all **private** (`cargo-lid-rs/src/project.rs:232`, `:238`,
  `:261`).
- They live in `cargo-lid-rs`, and `lid-rs` depends on exactly
  `lid-rs-macros` and `linkme`. `lid-rs` cannot depend on `cargo-lid-rs`.
- **No `[workspace.metadata.lid_rs.conformance]` table exists** — not in this
  workspace's root manifest, which holds only `mutation_scope`, and not in
  `cargo-lid-rs/templates/manifest_tables.toml`, which is what `init` writes.

So there is no path by which an emitted test reads a level. This is the same
obstacle slice 18 found for `shape.level`, and the same answer applies: a
setting either arrives as an argument from a caller that can read it, or the
check has no level. Since these checks are emitted by `intent_graph!()` into
crates that link only `lid-rs`, they have no such caller today. Stating the
severity as fixed — E1 a compile error by construction, E2 a failing test — is
the only honest option for this slice, and `aliases` is a Deferred belonging to
S1 rather than to either check here.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| The slice is E1 and E2 only | Checks 21 and 22, `derive(Outcome)`, `OUTCOMES`, and this slice's canary | All four of 19–22; **E1 alone**; wait for subject matter | README draws this line: E1/E2 are "registry intersections, no syntax needed", while S1/S2 need `signatures()` — which slice 18 has now shipped (`585b212`), so S1/S2 are Deferred 1 rather than blocked. **E1 alone was drafted and refuted**: with no E2 the slice holds only a trait, so Phase 3 has no `todo!()`, Phase 5 has no reachable red, and trybuild is closed because `lid-rs` is not a proc-macro crate and so has no companion. E2 is what makes this a slice. |
| E1 is a compile-time bound, not a check | A const block `derive(Spec)` emits, bounding `owner` on `Outcome` and matching the variant | An `OUTCOMES` × `SPECS` intersection like E2; a syntactic pass | Spiked: the bound gives `E0599` for a bad variant and `E0277` for an underived enum, at the claim, naming the offence. An intersection would need the claim's path and the registration's path to agree as strings — the exact hazard `SpecMeta.name`'s join note forbids. It also makes E1 unfailable at runtime, which is why its claims are validated by trybuild rather than by a test. |
| `Outcome::NAME` is the join key for E2 | `concat!(module_path!(), "::", stringify!(Enum))`, with the claim side emitting `<Owner as Outcome>::NAME` | Compare `ClaimMeta.owner` against a `module_path!()` string recorded at the enum; record the enum's path as written at the claim | Both alternatives compare two independently built strings and break on a re-export or a differently spelled path. Reading the const through the resolved type means one const feeds both sides, which is the principle every existing registry join already follows. |
| The derive work lands as a change to the `claim` slice | `lld/claim--outcome-derive` | A new `outcome` slice inside `lid-rs-macros`; widening this slice's path policy | `lid-rs-macros/src/claim/compile-time-accepted` already exists, so a change there needs no new human act, while a new proc-macro slice would need a new acceptance file and block on it. Widening the policy hands this slice a crate its document is not in, which is the boundary the policy exists to draw. |
| Severity is fixed, not read from `conformance.level` | E1 a compile error, E2 a failing test | Read the level and gate accordingly; default to `warn` until the level is readable | Verified: the accessors are private and in `cargo-lid-rs`, `lid-rs` cannot depend on it, and no conformance table exists in the workspace or the `init` template. A level that cannot be read is not a level. A silent `warn` default would also land a check that records nothing, which constraint 3 names as worse than no check. |
| E2 scopes to the invoking crate; E1 does not | `Spec::NAME` prefix filter for E2, as `graph_orphans` does | Scope both; scope neither | The graph slice settled this for checks 10/11 and the reasoning is unchanged — a consumer's binary links this crate's registrations too. E1 fires at its own claim's definition site, so it has no cross-crate surface to scope. |
| The canary's claim is cited on a function, not on the module by containment | `canary::present(outcomes) -> bool`, carrying `#[implements]`; the registrations beside it are no longer traced by `implements_module!` | Keep `implements_module!` over the data; fold the canary into E2's claims and drop it; leave it unvalidated until Phase 6 | Phase 5 requires a **reachable red** for every claim in the red set, and a claim implemented by data alone has none: the registrations exist from Phase 3 onward, so any test of them is green before implementation and `red_verdict` refuses it (`cargo-lid-rs/src/phase/mod.rs:905-919`). The registry slice met this first and answered it the same way — `registry::canary::present()` carries `CanaryConfirmsRegistryPresence` while the statics beside it carry no citation (`lid-rs/src/registry/canary.rs:23-26`). The function is not a contrivance for the gate: it is the predicate a consumer's outcome check must call before trusting `OUTCOMES`, exactly as `triple_is_present` is for the triple. **Resolves Deferred 6.** |
| Refusing to answer over a stripped `OUTCOMES` belongs to the graph change, not here | `unclaimed_variants` keeps `Vec<String>`; the emitted `every_variant_has_a_claim` guards on `canary::present` and reports the stripped case itself | Give `unclaimed_variants` a `Result<_, OutcomesStripped>`, as `graph_orphans` has | The triple's precedent puts the guard in the emitted check and not in the work leaf beneath it: `graph_orphans` returns `Err(CanaryStripped)` while `orphaned_specs` under it returns a plain `Vec` (`lid-rs/src/graph/mod.rs:26-42`). Moving the guard here would change a return type Phase 2 wrote five claims against and Phase 4 has already descended (`b7cad89`), and would need a sixth claim this slice has not derived. It lands with `every_variant_has_a_claim` on `lld/graph--outcome-checks`. |
| The trybuild fixtures land by hand **before** Phase 5 | `lid-rs/tests/ui/outcome/{fail,pass}/`, with no `.stderr` pins | Land them with the derive after Phase 6, as the other three hand items do; let Phase 5 write them under `src/outcome/ui/`, which is in its path row | A harness whose glob matches nothing records **zero failures** and passes (`trybuild-1.0.120/src/run.rs:74`), so nine of eighteen claims would be vacuously green and `red_verdict` would refuse them. `src/outcome/ui/` is legal — trybuild globs from `CARGO_MANIFEST_DIR` and needs no `tests/` — and is refused anyway: every UI fixture in the workspace lives under `lid-rs/tests/ui/`, and a second home for fixtures is a permanent divergence bought with one saved hand commit. |
| E1's four claims and the five `derive(Outcome)` claims are **two harnesses, not nine tests** | One harness per group, each citing every claim its fixtures demonstrate | One validator per claim, as an earlier draft's reading of check 14 required | Check 14 is a naming rule — a validator is admitted when named for **any one** cited claim (`lid-rs/src/claim/spec.rs:219-229`) — so nothing forces the split, and two of E1's four claims are *negative*: their `pass` fixtures compile while the derive is absent, so a harness citing only them is green. They are red by association, which is the cost slice 15 named and took (`lid-rs/src/claim/mod.rs:116-119`). |

## Open Questions & Future Decisions

### Open

**Drafted resolution: option 1, E1 *and* E2.** *Human sign-off wanted — this
is the agent's Phase 1 draft.* An earlier draft of this row chose option 3,
E1 only, and it was refuted by review; the reasoning is recorded here because
the refutation is the argument for what replaced it.

**Option 2 is rejected by tenet 2.** "A gate that exists, gates" — waiting
leaves README §4.7 describing checks nobody can run, and makes the first
unwanted claim a retro-fit. The conversion it waits on is itself unscoped
work.

**Option 3 is rejected because it is not a slice.** Constrained-first argues
for it — E1 is a const block, E2 carries the registry, the dump format and
the canary question — and the argument is real but fatal here. Under E1 only,
everything `lid-rs/src/outcome/` holds is `pub trait Outcome`. There is no
function, so there is no `todo!()` for Phase 3, nothing for Phase 6, and
**Phase 5 has no red it can reach**: `check_red` fails on a claim with no
failing validation (`cargo-lid-rs/src/phase/mod.rs:647`, `:783`), and an
associated const in a trait declaration is correct the day it is written.
The escape slice 15 used — trybuild fixtures — is closed too, because that
slice has a **companion** (`policy.rs:434-441`) and a companion exists only
when the own crate is a proc-macro crate. `outcome`'s crate is `lid-rs`, so
no phase of `lld/outcome` may create `lid-rs/tests/ui/**` at all. E1 alone is
a *change to the derive*, not a slice with phases.

**And the question option 3 was chosen to dissolve has an answer already in
this workspace.** "Whether an empty `OUTCOMES` is distinguishable from a
stripped one" is the exact question the **canary** exists to answer, and the
registry slice answered it: `lid_rs::canary` ships a known triple
"registered unconditionally — **not** `#[cfg(test)]` — because downstream
crates link `lid-rs` compiled without `cfg(test)`"
(`lid-rs/src/registry/lld.md:104-107`). A fourth distributed slice takes a
fourth canary: this slice ships one `#[derive(Outcome)]` enum of its own,
registered unconditionally, so `OUTCOMES` **without** it was stripped and
`OUTCOMES` containing only it is a workspace with legitimately zero outcomes.
That is applying a shipped pattern, not answering a new question, and it
lives in this slice's own module rather than on `lld/registry--outcomes`.

So E2 is what makes this a slice: `unclaimed_variants` and `claimed_owners`
are work leaves a Phase 3 can skeleton, a Phase 5 can redden with synthetic
registries, and a Phase 6 can implement — exactly as `lid-rs/src/graph/`
already does. E1 rides along as the derive change it always was.

Constraint 3 — "a report nobody reads is worse than an absent check" — is about
a check that reports without gating, which is not quite this case: these gate,
they simply never fire. The document records that as a distinction and not as a
resolution.

**Whether an empty `OUTCOMES` is distinguishable from a stripped one —
answered by this slice's own canary.** The registry slice met this question
for the triple and settled it by shipping a known entry unconditionally
(`lid-rs/src/registry/lld.md:104-117`). This slice ships the same shape for
the fourth slice: one `#[derive(Outcome)]` enum in `lid-rs/src/outcome/`,
registered outside `cfg(test)` so it reaches every consumer's test binary,
and E2 reads its absence as a stripped section rather than as zero outcomes.
No `lld/registry--outcomes` change is needed; the pattern is the registry
slice's, the instance is this slice's.



### Settled: a claim's `owner` joins an enum's `NAME` through `CLAIMED_OWNERS`

**Decided by the human, 2026-09-12, before Phase 6 ran.** Reading 3 below is
taken; readings 1 and 2 are kept only as the record of what was rejected and
why. The Decisions row that named reading 1 and the Shape row that described
reading 2 are both superseded by this section.

**The shape it is taken in, which is not the only shape it had.** `CLAIMED_OWNERS`
supplies the *resolved name* and nothing else. It does **not** replace
`claimed_owners`'s input, and that distinction is the whole cost difference:

- Taken: `claimed_owners(specs, owners)` keeps deciding *which* claims are
  owners — the `Pattern::Unwanted` and non-empty-`owner` filter stays exactly
  where Phase 4 put it — and consults `owners` only to spell each one as
  `Outcome::NAME` resolves it. It gains a parameter. Its three claims stand
  **verbatim**, still worded about a `SpecMeta`. `owner_names_enum` keeps the
  two-string signature Phase 4 committed and its body collapses to `==`,
  because both sides now come from the same const.
- Rejected: making `CLAIMED_OWNERS` the input, so the derive's registration
  performs the filter. That is cheaper code and a much larger change — the
  three `claimed_owners` claims would no longer be about a function, would move
  from E2's runtime side to E1's compile-time side, and would become
  trybuild-validated. That is a Phase 2 rework, not a cascade, and it was not
  what was chosen.

**One wording note, recorded rather than glossed.**
`AnUnwantedClaimsOwnerIsAClaimedOwner` says the function "shall carry that
owner". Under this reading it carries the owner the claim names *as
`Outcome::NAME` resolves it*, not the string `ClaimMeta.owner` holds. That is
the same owner under a different spelling and the claim is read that way here.
If a reviewer reads "that owner" as "that string", the claim needs a Phase 8
rename and not a reinterpretation.

**What this costs, completely.** A `ClaimedOwner` type and a fifth distributed
slice in `lid-rs`; one added parameter to `claimed_owners`, cascading through
`unclaimed_variants`; `owner_names_enum` becomes `==`; the Phase 5 tests of
those claims gain the argument, their assertions unchanged; and
`derive(Spec)`'s E1 emission — already a hand commit on this slice — grows the
registration beside the bound it was already going to emit. The emission shape
is proven in this tree at `canary.rs:64-84`.

### The record: how a claim's `owner` joins an enum's `NAME`

Phase 2 found this document saying two different things and declined to
choose, wording every claim to hold under either. It bites at Phase 6.

- **The Decisions row** makes the claim side emit
  `<Owner as Outcome>::NAME`, "so both sides of E2 come from one const" — no
  string comparison, and the hazard `SpecMeta.name`'s join note forbids is
  avoided. Cost: `ClaimMeta` has no field to carry a resolved name, so this
  adds one, on the **`claim` slice**.
- **The Shape row** for `claimed_owners` called it — before that row was
  corrected to match the decision — "the one place `ClaimMeta.owner`'s
  full-path-versus-last-segment distinction is resolved",
  which reads as comparing the last segments of two independently written
  strings. Cost: it is exactly the comparison the row above rejects, and
  `Registry::Kind` and `other::Registry::Kind` would join.

- **A third reading, found at Phase 6 and recorded rather than taken.** The
  claim side can carry `<Owner as Outcome>::NAME` without `ClaimMeta` gaining
  a field, by registering it into a distributed slice of its own — a
  `CLAIMED_OWNERS` beside `OUTCOMES`, one entry per unwanted claim with a
  non-empty owner, holding the claim's `NAME`, the owner read through the
  resolved type, and the variant. The emission shape is already proven in this
  tree: the canary registers `owner: <CanaryOutcome as Outcome>::NAME` from
  inside a `const _` block (`canary.rs:64-84`), which is precisely what
  `derive(Spec)`'s E1 bound is already going to emit at the same site. One
  const still feeds both sides, so the join note is satisfied exactly as under
  the first reading. Cost: a fifth distributed slice, and `claimed_owners`
  takes that slice rather than `&[SpecMeta]` — a signature Phases 3 and 4 have
  already committed.

  It is arguably the better shape and that is why it is written down: a field
  on `ClaimMeta` is carried by all 395 registered claims and is empty for
  nearly all of them, whereas this repository already models a per-claim fact
  that only some claims have as an *edge in a distributed slice* — which is
  what `IMPLEMENTATIONS` and `VALIDATIONS` are.

The first is the design this document argued for, the second is what its Shape
row describes, and the third is neither but satisfies the first's principle at
a different cost. They need different code and different registrations, so
**Phase 6 cannot infer it** — the human picks one and the other rows go.
Recorded rather than silently taken: the first adds a field to a delivered
slice, the third adds a distributed slice and changes a committed signature,
and the second ships a join under which `Registry::Kind` and
`other::Registry::Kind` are the same enum.


## What lands by hand, and in what order

Four things no phase of this slice may write. The review that produced this
section verified each against the tree.

1. **`pub mod outcome;` in `lid-rs/src/lib.rs`, and `src/outcome/mod.rs`.**
   Phase 2's path row is the claims file alone (`policy.rs:420`, `:458`), so
   it can write `src/outcome/spec.rs` and cannot declare it. Phase 2's check
   is `cargo check --all-targets`, which passes happily over a file no module
   declares — so check 13 never sees the claims and the failure surfaces at
   Phase 5 as `require_claims`'s "nothing registers". `src/lib.rs` **is** in
   Phase 3/4's row, so the declaration may land there; slice 18 instead landed
   it by hand between Phases 2 and 3 (`4b036c7`) and had all its claims
   accepted first time. Do that.

2. **`derive(Spec)`'s E1 emission, in `lid-rs-macros/src/expand.rs`.** Not a
   phase commit and **not** reachable from `lld/claim--outcome-derive` as an
   earlier draft assumed. The acceptance argument holds — `slice_of_branch`
   maps that branch to `claim` (`phase/mod.rs:693`) and
   `lid-rs-macros/src/claim/compile-time-accepted` exists — but `expand.rs`
   is under `lid-rs-macros/src`, which is in **no** phase row for slice
   `claim` (`Form::Module`, dir `src/claim`; Phase 3/4's only extra is
   `src/lib.rs`). Slice 15 recorded the same fact —
   "`expand.rs` is in no phase's path" (`lid-rs-macros/src/claim/lld.md:473`)
   — and landed its derive change by hand. So does this one.

3. **The swap, after Phase 6, never before it.** `derive_spec` runs inside
   every `cargo check` for all 395 registered claims, so a `todo!()` on its
   path is a compiler panic across the whole workspace. Slice 15 met this and
   answered *pin, then swap*: build the items beside the derive with their own
   tests, and wire them into the expansion only once their leaves exist
   (`lid-rs-macros/src/claim/lld.md:458-483`). The same sequence holds here.

4. **README §4.7's amendment.** README calls E1 "a registry intersection, no
   syntax needed" and numbers it check 21 with a severity level; this slice
   makes it a compile-time bound, which has no severity and is not a check a
   run reports. Tenet 1 makes that a doc bug, and `README.md` is at the
   workspace root — outside both crates — so `seat_of` refuses it before any
   table (`policy.rs:521-527`). A main-session commit, not a phase's.

**Branch order — the hand commits land after Phase 6 and *before* Phase 7.**
An earlier draft said "after its Phase 7" and that is wrong, in a way Phase 2
caught: **nine of this slice's eighteen claims are implemented in
`lid-rs-macros/src/expand.rs` and validated by trybuild fixtures**, and both
are outside every phase row of `lld/outcome`. Once the module is declared
those nine register, and `lid-rs`'s own `intent_graph!()` — checks 10 and 11
— names them orphans. With the derive landing after Phase 7, **Phase 7 could
never pass**.

So: `lld/outcome` runs Phases 2 through 6, because `derive(Outcome)` emits
`impl ::lid_rs::Outcome for E` and cannot compile before Phase 3 creates the
trait. Then the derive change lands by hand. Then Phase 7 gates the whole of
it.

**Except that "then Phase 7" cannot follow "Phase 6" here, because they are one
commit.** Phase 6 has no commit of its own in this toolchain — the `lid-rs-phase-7`
agent implements the leaves *and* gates, in one run, and its stop hook makes the
commit only when `phase-check 7` passes. There is no invocation that stops after
the leaves. So a derive landing "after Phase 6" would land after a gate that
could never have passed: nine claims are implemented in `expand.rs` and checks
10 and 11 name them orphans until it does.

**The order that runs is: the derive lands by hand *before* the Phase 6/7 agent
is spawned at all.** Nothing prevents it — the derive needs only the trait, which
Phase 3 created at `d2e9bee`, and it touches no file any phase of this branch
may write. The `.stderr` pins are generated in the same hand commit, once the
derive exists to produce the diagnostics. The agent then implements the six E2
leaves and gates in one pass.

This is the third ordering correction on this branch and they share a cause:
the phase sequence in README §8 numbers eight phases, and this toolchain gives
seven of them commits. A document that reasons about "after phase N" has to
check which N have stops. That is also exactly what slice 15 did — "the swap is a hand commit, and
it comes after Phase 6" (`lid-rs-macros/src/claim/lld.md:458-483`) — which
this document already cites for the pin-then-swap rule and should have
followed for the order.

**But the fixtures are not on that schedule, and binding all four hand items
to one sentence was a second ordering error.** The trybuild *harness* is a
`#[cfg(test)]` unit test inside the lib and so is Phase 5's to write
(`lid-rs/src/claim/mod.rs:221`, `lid-rs/src/lid_rs_macros/mod.rs:109`); the
*fixtures* are files under `lid-rs/tests/ui/`, which is in no phase row of this
branch. A harness whose glob matches nothing does **not** fail — trybuild calls
`message::no_tests_enabled()` and records zero failures
(`trybuild-1.0.120/src/run.rs:74`) — so a harness written at Phase 5 against
absent fixtures is a *vacuous green*, and `red_verdict` refuses it. Nine of
eighteen claims would stop the phase.

**The fixtures therefore land by hand *before* Phase 5**, not after Phase 6.
They need no `.stderr` pins to be red and must not be given hand-written ones:
a `fail` fixture with no pin makes trybuild write a `.wip` and report failure,
and a `pass` fixture cannot compile while the derive is absent. The pins are
generated with `TRYBUILD=overwrite` once the derive lands, never hand-edited
(`docs/intent/publish/lld.md:83`). This is the same deadline reasoning this
document already applied to `pub mod outcome;` — landed between Phases 2 and 3
rather than at its nominal phase — applied to the item it was not repeated
for.

**The fixtures live at `lid-rs/tests/ui/outcome/{fail,pass}/`, and `src/` is
refused.** trybuild globs relative to `CARGO_MANIFEST_DIR` and requires no
`tests/` directory, so `t.compile_fail("src/outcome/ui/fail/*.rs")` would be
legal, in-policy for Phase 5, and would dissolve the hand commit entirely. It
is refused anyway: every UI fixture in this workspace lives under
`lid-rs/tests/ui/`, the publish slice's `.stderr` regeneration is written
against that path (`docs/intent/publish/lld.md:83`), and the vocab slice's
fixture repair names it (`lid-rs-macros/src/vocab/lld.md:202-204`). Saving one
hand commit is a process convenience; a second home for fixtures is a
permanent divergence, and hand commits are ordinary on this branch already.
This paragraph exists so that a phase stuck at Phase 5 does not invent the
`src/` route as a way out.

The alternative, if the hand commits are unwelcome on this branch, is to move
those nine claims to the `claim` slice's spec file and deliver them with the
derive. That is a larger change and is not chosen here.

Within `lld/outcome`, the canary enum's `Outcome` impl is hand-written rather
than derived, which is what breaks the circularity — the same move slice 15's
bootstrap window used.

### Open — nothing observable separates a key read through the type from a path written twice

`AnOutcomeRegistrationIsKeyedByTheEnumsOutcomeName` (`spec.rs:121-126`)
requires the registration's key be "the `NAME` read through that enum's own
implementation, **rather than a path written a second time**". Where the derive
emits the registration — in the enum's own module, as the hand-written canary
shows (`canary.rs:70-77`) — a literal `module_path!()` produces the *identical*
string. So the claim and its negation have the same output, and no fixture
written from this document distinguishes them.

Phase 5 is unaffected: with no derive, the harness is red either way. The cost
lands at Phase 6, where the test must separate two indistinguishable outputs,
and at check 12, where the mutant that re-spells the path survives. The
candidates are a fixture that *re-exports* the enum, or one that registers from
a module the enum is not declared in, so that the two spellings diverge — both
need the derive to exist before they can be tried. Recorded rather than guessed;
it is the one claim of the nine whose fixture cannot be written from this
document alone.

### Deferred

1. **Checks 19 and 20 (S1, S2).** They need `lid_rs_shape::signatures()`.
   **That dependency is now discharged**: slice 18 is complete through Phase 7
   (`585b212`), and its `signatures` answers every function's parameter and
   **whole written return** type — not only flow nodes and not only the `Ok`
   type — which is the widening this slice needed and README §4.7 requires.
   S1 and S2 are therefore buildable, and are the natural next change on this
   slice rather than a blocked item.
2. **`conformance.level`.** Unreadable from an emitted test; see above. It needs
   either a settings path into `lid-rs` or a caller that reads the manifest,
   neither of which exists.
3. **`aliases`.** README introduces `[workspace.metadata.lid_rs.conformance]
   aliases` for S1's known false positives — type aliases and `impl Trait`
   returns. It belongs to S1, and lands with it.
4. **Span recording on `Outcome`.** Slice 20's, and the reason the trait exists
   at a traced boundary at all. This slice ships the trait with the registration
   and `NAME` only.
5. **The `OUTCOME` dump line.** `registry_dump_for_tooling` emits three kinds,
   parsed by `cargo-lid-rs`'s mutation machinery. Whether outcomes need to reach
   that tooling is unanswered, and belongs with the `graph` change.


6. ~~**The canary's own claim, if it needs one beyond E2's.**~~ **Resolved.**
   Phase 2 cut it as a claim of its own,
   `TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked`. Phase 5 then showed
   that citing it by containment over the registrations leaves it with no
   reachable red, so it is cited on `canary::present` instead — see the
   Decisions table. The registry slice's precedent is followed in full, minus
   the sentinel validation edge, which the triple needs because `VALIDATIONS`
   is one of the three slices it guards and `OUTCOMES` is not.

## Shape

| Item | Role |
|---|---|
| `lid_rs::outcome::Outcome` | The trait `derive(Outcome)` implements. Carries `const NAME: &'static str` — the join key, `module_path!()`-rooted, read through the resolved type so both sides of E2 come from one const. Documented as the surface slice 20 will extend for span recording. |
| `lid_rs::outcome::OutcomeMeta` | One registered variant: the owning enum's `NAME`, the variant identifier, and the registration site's file and line — the same shape as `SpecMeta`, for the same reporting reasons. |
| `lid_rs::OUTCOMES` | The fourth distributed slice. Declared beside the triple, and the subject of the canary question above. |
| `lid_rs::outcome::ClaimedOwner` | One unwanted claim's owner, joined to the enum through the type the claim named: the claim's `Spec::NAME`, the owner's `Outcome::NAME` read through that type, and the variant identifier. No file and no line — a site is registered so a report can send a reader somewhere, and this join's report names the *variant's* site, which `OutcomeMeta` holds; the claim's own site is on the `SpecMeta` this entry joins by name. |
| `lid_rs::CLAIMED_OWNERS` | The fifth distributed slice, gathering those at link time. Declared as `OUTCOMES` is and re-exported from the crate root beside it, because `derive(Spec)` addresses it as `$crate::CLAIMED_OWNERS` from every downstream crate. Filled by the derive's E1 emission — until that hand commit lands it is empty, and an emitted check over it would report every variant of every claimed enum. |
| `unclaimed_variants(crate_name, specs, owners, outcomes) -> Vec<String>` | Work leaf for E2: variants of claim-owning enums that no unwanted claim names, formatted `name (file:line)`. Crate-scoped by `Spec::NAME` prefix, as `orphaned_specs` is. |
| `claimed_owners(specs, owners) -> Vec<&'static str>` | Work leaf: for every `Pattern::Unwanted` claim with a non-empty owner, that owner **as `Outcome::NAME` spells it**, looked up in `owners`. The predicate E2's scope rule turns on — "enums that own at least one claimed variant". The filter stays here; the spelling comes from the registration, which is what makes the join an equality of two values from one const rather than a comparison of two written strings. An earlier version of this row called this function "the one place `ClaimMeta.owner`'s full-path-versus-last-segment distinction is resolved", which described the rejected reading 2; there is no such distinction to resolve now.  **`Vec`, not `impl Iterator`** — the LLD asked for the latter and Phase 3 could not skeleton it: `!` does not implement `Iterator`, so a bare `todo!()` body is `E0277`, and the workaround (`let v: Vec<_> = todo!(); v.into_iter()`) fires `unreachable_code`, `unused_variables` and `clippy::diverging_sub_expression`, all denied. Spiked to confirm. Nothing here needs laziness and no claim's wording changes. |
| `derive(Outcome)` (in `lid-rs-macros`) | Emits the `Outcome` impl with `NAME`, and one `OUTCOMES` registration per variant. **Lands as a hand commit, not a phase commit** — see "What lands by hand" below. |
| `lid_rs::outcome::canary::CanaryOutcome` | One enum this slice ships — **the name is load-bearing**, because Phase 2's trigger links it and a differently named enum breaks that link at Phase 7's doc step. Its `Outcome` impl is hand-written, not derived, which breaks the circularity with `derive(Outcome)`, registered **unconditionally** — not `#[cfg(test)]` — so it reaches a consumer's test binary. `OUTCOMES` without it was stripped; `OUTCOMES` holding only it is a workspace with legitimately zero outcomes. The shape is the registry slice's (`lid-rs/src/registry/lld.md:104-117`); the instance is this slice's. |
| `lid_rs::outcome::canary::present(outcomes) -> bool` | True when `outcomes` carries an entry for each variant `CanaryOutcome` declares — the fourth slice's analogue of `registry::canary::triple_is_present`, and the item [`TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked`](spec::TheCanaryOutcomeIsEnumerableWhereverTheCrateIsLinked) cites. Parameterized over the slice rather than reading `OUTCOMES` directly, so the stripped case is testable with an empty input while the validation applies it to the real static. It is the only work in this slice that is *not* E2's. |
| `derive(Spec)`'s E1 emission (in `lid-rs-macros`) | For an unwanted claim with a non-empty `owner`: a const block bounding the owner on `Outcome` and matching the variant. Lands on the same change. |
| `intent_graph!()`'s new test (in `lid-rs/src/graph/`) | `every_variant_has_a_claim`, calling `unclaimed_variants` against the real registries. Lands on `lld/graph--outcome-checks`. There is no emitted test for E1: a claim that fails it does not compile. |

`unclaimed_variants` and `claimed_owners` are parameterized over slices, as the
graph slice's functions are, so every branch is an ordinary unit test with
synthetic inputs while the emitted test applies the same function to the real
registries.

**A note on granularity, corrected after Phase 2 cut the claims.** E2's rule
decomposes into at least three independently failing statements — which enums
are in scope, which variants count as claimed, and what the check reports —
and check 7's `cognitive-complexity-threshold` is 4, so each wants its own
one-assertion test. Phase 2 cut them that way.

An earlier draft of this note said "check 14 binds one validator to one claim".
**That is wrong, and it would have sized Phase 5's red set wrongly.** Check 14
is a *naming* rule: a validator is admitted when its name is the `snake_case`
of **any one** of the claims it cites, optionally with a `_suffix`
(`AValidatorIsNamedForAnyOneCitedClaim` and `ASuffixedValidatorNameIsAdmitted`,
`lid-rs/src/claim/spec.rs:219-229`; implemented at
`lid-rs-macros/src/claim/mod.rs:278-330`). One test may cite many claims —
`the_registration_carries_the_claims_parts` cites roughly forty
(`lid-rs/src/claim/mod.rs:176-224`). No exemption is involved; the rule never
bound one-to-one.

**So E1 is one harness, not four, and it must be.** Its four claims are not
four diagnostics — an earlier draft said three (`E0599`, `E0277`, and "the
compiling case"), and Phase 2 split the compiling case in two
(`spec.rs:19-34`). Two of the four —
`AClaimOutsideTheUnwantedPatternCarriesNoOutcomeBound` and
`AnUnwantedClaimWithAnEmptyOwnerCarriesNoOutcomeBound` — are *negative*: their
demonstration is a `pass` fixture, which **compiles** while the derive is
absent. A harness citing only those two is green at Phase 5 and refused. They
are red only by association with the siblings that fail, which is exactly the
cost slice 15 named and accepted: "Each harness cites every claim its fixtures
demonstrate, so the claims whose response is that something compiles share a
validator with the siblings that fail — the cost the LLD names, and what makes
them red before the derive enforces anything" (`lid-rs/src/claim/mod.rs:116-119`).
The five `derive(Outcome)` claims share a harness for the same reason.

## Notes for Phase 5's red set

Three things a red test of this slice gets wrong by default.

1. **Every E2 test passes at least one `SpecMeta`.** `unclaimed_variants`
   returns `vec![]` without reaching any `todo!()` when `specs` is empty: the
   owner list is then empty and every outcome is filtered out before the join
   is called (`mod.rs:108-124`). A test written as "no claims, so no report" is
   **green** and refused. The claim this traps is
   `AVariantOfAnEnumNoClaimOwnsIsNotReported`, which reads most naturally as
   "no claims at all"; written with one *non-matching* claim it is red. The
   mirror case is safe — non-empty `specs` with empty `outcomes` still panics,
   because the filter forces `registered_by` before the outcomes are touched.

2. **Both sides of every synthetic owner are spelled identically.** The join
   between a claim's `owner` and an enum's `NAME` has two readings still open
   for Phase 6 (above). Phase 2 wrote every *claim* to hold under either and
   Phase 4's signatures hold under either, but the *test data* is not protected
   by that: a case pairing a claim owner `crate::auth::AuthError` with a
   registration owner `app::auth::AuthError` is positive under one reading and
   negative under the other, and Phase 6 would have to rewrite the tests it was
   meant to satisfy. Spell them the same and no test distinguishes the
   readings.

3. **`file!()` inside a trybuild fixture is not the fixture's path.** trybuild
   copies each fixture into `target/tests/trybuild/lid-rs/…` and compiles it
   there, so an expansion's `file!()` reports the generated path.
   `AnOutcomeRegistrationCarriesTheSiteItStandsAt` must assert a suffix, or
   `line > 0`, not an equality.

## References

- README [§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html) — checks
  19–22, E2's scope rule, the mechanics note, and the worked example at §9.
- README [§4.2](https://bradvoth.github.io/lid-rs/spec/gates.html) — the
  crate-scoping rule these checks inherit.
- `lid-rs/src/registry/lld.md` — the triple, the canary, and `SpecMeta.name`'s
  join principle.
- `lid-rs/src/graph/lld.md` — the pattern this slice follows: pure functions
  over slices, a `macro_rules!` emitter, synthetic-input tests.
- `lid-rs-macros/src/claim/lld.md` — `ClaimMeta`, `Pattern::Unwanted`, and the
  `owner` rule; the slice the derive changes land on.
- `lid-rs-shape/src/lld.md` — slice 18, which owes `signatures()` to S1/S2.
