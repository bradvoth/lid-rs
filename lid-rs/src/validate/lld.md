# A validator's trace says which claims it reached, and how they closed

## Context and Design Philosophy

HLD row 20 is "A claim's satisfaction is observable at runtime", and it names
six products in one row (`lid-rs/docs/intent/hld.md:199`):

> `tracing` spans in `#[implements]`, the capturing layer in `#[validates]`,
> `Traceable` recording, checks 23–25, `lid_rs::spawn`, `runtime.level`; the
> test output that reads as the claims exercised

**This is the last of slices 16–23, and it is the one that closes the loop.**
Everything the earlier slices built is static: the graph is real at compile
time, the shape pass reads a function's tokens, the conformance pass reads a
signature's types, the trace document renders the registry. None of them can
say that a claim was *reached*, or that the path it promised was the path
taken. README §6.4 is the answer, and it has been the unbuilt half of the
README since r1 (`README.md:1941-1942`).

**The slice is a module slice in `lid-rs` at `lid-rs/src/validate/`, and the
macro emission lands beside it as a hand cascade.** The split is not the one
slice 17 needed. `expand.rs`, where `#[implements]` and `#[validates]` are
built, sits at `lid-rs-macros/src/` under no module directory at all: it
belongs to the crate-root slice `lid-rs-macros` and to nothing else. Slice 19
landed exactly this file for `derive(Outcome)` as `da047cd`, *"cascade:
derive(Outcome) and E1's bound, which no phase may write"*. That route needs no
`compile-time-accepted` file, because the acceptance gate lives only in the
phase agents' pre-tool hook (`cargo-lid-rs/src/phase/mod.rs:287-296`,
`:326-340`). The alternative — a crate-root slice named `lid-rs-macros` — would
need a human's acceptance file and would point Phase 2 at `lid-rs/src/spec.rs`,
a pre-colocation path that does not exist and that no slice has ever exercised.

**The name is the operation, not the component.** `cargo-lid-rs`'s catalog
already reserves the command `validate` for this slice
(`cargo-lid-rs/src/catalog/mod.rs:215`, `:987`), and the operation a user
performs is running a validator and reading what it reached. `runtime` names a
layer of the system; `validate` names the act. The directory is
`lid-rs/src/validate/` and the branch is `lld/validate`.

**The emission arrives at `warn`, and that is the whole of how this slice
avoids detonating.** Once `#[implements]` opens a span, every cited item in the
workspace opens one — 549 `#[implements]` sites across five members. Once
`#[validates]` captures, every validator runs under the capture and checks 23
and 24 have an opinion about it. If those opinions were errors on arrival, the
slice's own Phase 7 would have to make 549 sites pass checks that were written
three commits earlier. README's answer to exactly this is the ramp: "a
brownfield crate starts at `warn`, commits the counts in `trace.md`, and flips
to `deny` when a slice's count has been zero for a while" (`README.md:1236-1240`).
This slice ships the ramp at `warn` — the finding and the tree are printed, the
test does not fail — and the flip to `deny` is a named later change. That is
also what makes the checks *fed* rather than merely built, which is the gap
slices 18, 19 and 21 each left behind them (`shape`, `conform` and `regen` are
all still `Unbuilt` in the catalog after their slices completed).

**Four things HLD row 20 names are not in this slice, each for a reason
recorded below and none of them for convenience.** `lid_rs::spawn` needs an
async runtime this workspace does not have. `runtime.level` cannot be read from
where this code runs, which is the same wall slices 18, 19 and 21 each hit.
Check 25 needs the union of every validator's trace *and* the flow/leaf
classification, which lives in a crate that depends on `lid-rs`. And a span
records a parameter's *noun*, not its value, because the trait method that
would record a value does not exist and adding it is a breaking change to a
published crate.

## Facts, verified 2026-09-12

Every row was read from the tree on this branch (`lld/validate`, stacked on
`lld/vocab` at `bb73fef`) or produced by running the command named. Spikes were
run with `rustc 1.98.0` against `--edition 2024` and `tracing 0.1.44`.

| Fact | Where |
|---|---|
| Check 23: "The validator's trace has no span citing its claim" | `README.md:886` |
| Check 24: "The citing spans don't exhibit the pattern's required trace" | `README.md:887` |
| Check 25: "Some flow node appears in no validator's trace. The one check that needs the union of all traces, so only the full `--lib` run can evaluate it" | `README.md:888` |
| Check 24's five per-pattern rows, in full | `README.md:890-899` |
| `min_inputs` "defaults to 16 under `[workspace.metadata.lid_rs.runtime]`" | `README.md:906-907` |
| The ramp: "a brownfield crate starts at `warn` … and flips to `deny` when a slice's count has been zero for a while" | `README.md:1236-1240` |
| Check 24 is the answer for a claim implemented only by data, which "is green the moment the skeleton lands" | `README.md:914-918`, restated `:1329-1331` |
| What `#[implements]` must emit: a span at `target = "lid"` carrying `lid.claims`, the parameters through `Traceable`, and `lid.outcome` on exit | `README.md:1111-1116` |
| What `#[validates]` must emit: a capturing subscriber for the test's duration, the tree captured and printed on failure | `README.md:1117-1119` |
| The worked tree output, which carries a `[flow]` marker and a `[redacted]` field | `README.md:1121-1126` |
| §6.8: a "root-keyed global capturing layer rather than a thread-local subscriber", and `lid_rs::spawn` | `README.md:1164-1169` |
| `[workspace.metadata.lid_rs.runtime]` as README §7 spells it: `level`, `capture`, `min_inputs` | `README.md:1213-1216` |
| The HLD's tenet-3 escalation for `tracing`, already argued and accepted | `lid-rs/docs/intent/hld.md:238` |
| The HLD's only stated home for check 25 is beside slice 18, which has shipped without it | `lid-rs/docs/intent/hld.md:219` |
| `expand.rs` emits **no** span today: `cite_fn` inserts one `Edge` registration as the body's first statement and touches nothing else | `lid-rs-macros/src/expand.rs:241-249`, `:357-372` |
| `citation` dispatches over `ItemFn`, `ItemStruct`, `ItemEnum` and refuses everything else | `lid-rs-macros/src/expand.rs:205-222` |
| The existing expansion reaches a dependency through `::lid_rs::__private::linkme`, never by naming the crate | `lid-rs-macros/src/expand.rs:362-363` |
| `__private` today holds exactly `Declared` and `linkme` | `lid-rs/src/lib.rs:254-258` |
| **549** `#[implements]` sites: `cargo-lid-rs` 443, `lid-rs` 41, `lid-rs-shape` 35, `lid-rs-pipeline` 26, `xtask` 4 — and **four of those five crates have no `tracing` dependency and no path to one** | `grep -rn '#\[implements' --include='*.rs'` |
| `acceptance_gate` runs inside `crate_verdict` **before** `path_verdict`, and reads the slice's **own** crate's target kinds | `cargo-lid-rs/src/phase/mod.rs:326-329`, `:335-340`; `policy.rs:753-760` |
| Slice 19's precedent: the macro half of a slice landed as a main-session cascade, with no acceptance file | `da047cd` |
| **`Phase` has no `Six` variant**, there is no `lid-rs-phase-6` agent, and the agents' stop hook rejects `phase 6:` as a commit subject | `cargo-lid-rs/src/phase/policy.rs:741-750`; `ls .claude/agents/`; `cargo-lid-rs/src/phase/ending.rs:254` |
| **There is no `commit-msg` git hook installed**, so `ending.rs` governs the phase agents alone and a main-session `phase 6:` commit is possible — `3775771 phase 6: leaves for claim` is one | `ls .git/hooks/`; `git log -1 3775771` |
| The `lid-rs-phase-7` agent implements the leaves **and** gates in one run, which is why slice 19 recorded "Phase 6 has no commit of its own" as an ordering correction | `lid-rs/src/outcome/lld.md:472-489`; `cargo-lid-rs/src/phase/lld.md:60` |
| No `tracing` dependency exists anywhere: zero hits across all 27 `Cargo.toml` files | `grep -rn tracing --include=Cargo.toml .` |
| Nothing of the runtime story exists in `lid-rs/src`: no `Subscriber`, no `Layer`, no capture, no `spawn` | `grep` over `lid-rs/src` |
| Zero occurrences of `async fn`, `tokio` or `futures` anywhere in the tree | `grep -rn 'async fn\|tokio\|futures' --include='*.rs' --include='*.toml'` |
| `ClaimMeta` carries `pattern`, and also `trigger`, `verb`, `negated`, `object`, `owner`, `templates` — and **no name** | `lid-rs/src/claim/mod.rs:57-78`, `:21-33` |
| A claim's name lives on `SpecMeta`, which is `{ name, file, line, claim: ClaimMeta }` — so `SpecMeta` is the only registered type that carries both an identity and a pattern | `lid-rs/src/registry/mod.rs:6-22` |
| A free claim's `ClaimMeta` is `Language::Free` with `Pattern::Ubiquitous` and every part empty, so **294 of the workspace's 471 claims share one `ClaimMeta` value** | `lid-rs/src/registry/mod.rs:15-22`; `lid-rs/src/claim/mod.rs:61-62` |
| A free claim's `pattern` is `Pattern::Ubiquitous`, so 294 of the workspace's 471 claims are ubiquitous by default | `lid-rs/src/claim/mod.rs:61-62`; the census in `lid-rs/src/vocab/lld.md` |
| `intent_graph!()` emits four tests today and nothing for shape, conformance, runtime or check 26 | `lid-rs/src/graph/mod.rs:76-132` |
| An `intent_graph!()` expansion may address only `$crate`, and `lid-rs-shape` depends on `lid-rs`, so the reverse edge is `error: cyclic package dependency` | `lid-rs/src/trace/lld.md:126`, `:323` |
| Slice 21 recorded that `lid-rs-site` "needs slice 20's captured traces for a claim card's worked example" | `lid-rs/src/trace/lld.md:322` |
| `Project::workspace_setting` and `setting_in` are private to `cargo-lid-rs` and read only flat `/metadata/lid_rs/<key>` keys — a nested `runtime.level` is not expressible | `cargo-lid-rs/src/project.rs:231-233`, `:237-242`, `:261-265` |
| Slice 19's verdict on the same wall: "Severity is fixed, not read from `conformance.level`" | `lid-rs/src/outcome/lld.md:260` |
| `Traceable` today is one associated const and no methods, and its value is the type's **spelling** | `lid-rs/src/vocab/mod.rs:17-25`, `:82-104` |
| vocab's Open 1 hands this slice the trait-method decision by name: "adding a required method to a published trait is a breaking change … named so that slice 20 meets it as a known cost" | `lid-rs/src/vocab/lld.md:302-310` |
| vocab's Open 2 hands this slice the identifier-vs-path convention: "the evidence is slice 20's" | `lid-rs/src/vocab/lld.md:312-323` |
| vocab's Deferred 1: the policy attributes "describe what a **span** records, and there are no spans until slice 20" | `lid-rs/src/vocab/lld.md:327-329` |
| `derive(Traceable)` does not exist and `noun_assertion` has not started, so no user type is a `Noun` | `ls lid-rs-macros/src/noun_assertion/` |
| The catalog reserves `validate` for this slice, unbuilt; `shape` and `conform` are still `Unbuilt` after slices 18 and 19 completed | `cargo-lid-rs/src/catalog/mod.rs:213-215`, `:987` |
| clippy's `cognitive_complexity` threshold is 4, and a six-arm `match` with one call per arm passes it today | `clippy.toml:1`; `cargo-lid-rs/src/phase/policy.rs:741-750` |

**The spikes, run today, that decide the design.** They are what the tenet-3
escalation at `hld.md:238` promised would be measured rather than assumed.

| Spiked | Result |
|---|---|
| A hand-written `impl tracing::Subscriber`, with **no** `tracing-subscriber` | **Captures everything the tree needs.** Span name and target from `Attributes::metadata`; fields at creation through a `Visit` impl; parentage from the subscriber's own enter-stack; a value recorded at exit through `field::Empty` plus `Span::record`; and close from `exit`. The whole capture is 90 lines |
| `tracing = { default-features = false, features = ["std"] }` | Compiles and passes the same spike. Drops `tracing-attributes` and with it `syn`, `quote`, `proc-macro2` from `tracing`'s own tree |
| `cargo tree -e normal` for that dependency | **Four crates total**: `tracing`, `pin-project-lite`, `tracing-core`, `once_cell` |
| `tracing::dispatcher::with_default` to scope a capture to one closure | Works, and is thread-local — which is what a `#[test]` wants and is **not** what README §6.8's async story wants |
| Recording `lid.outcome` after the child call, before the guard drops | The field lands on the parent span and is present in the captured tree. `field::Empty` at creation is what reserves the slot |
| **Whether a default `register_callsite` caching `Interest::never()` can suppress a callsite for a later dispatcher.** A filtering `Capture` rejects an `app`-target callsite; a second, unfiltered subscriber then runs the same code under its own `with_default` | **The hazard did not reproduce.** The second subscriber saw both callsites. Under `with_default` the interest is not cached as `never` across scoped dispatchers. Recorded because it is an undocumented interaction and the mitigation is one method — see the decision below |

## Behaviour

### The tree is data, and everything else reads it

The whole slice is arranged around one type. `Capture` implements
`tracing::Subscriber` and does nothing but build a `Vec<Span>`; `Trace` is that
vector; the checks and the renderer are functions from a `Trace` and some
claims to an answer. Nothing in the slice reaches for a subscriber, a
thread-local, or the clock once the tree exists.

That is deliberate and it is what makes Phase 5 possible. A check written
against a live subscriber can only be tested by running a traced function under
it, which makes every validator an integration test of an emission that does
not exist during Phase 5. A check written against a `Trace` value takes its
input as plain data, which is `references/phase-4.md`'s stated rule and what
slices 19 and 21 both did.

### The wire contract, spelled once because two crates must agree on it

This section is the join between `lid-rs-macros`'s emission and `lid-rs`'s
checks. It is written here, before either exists, because the two are written
by different agents three commits apart and a grammar nobody wrote down is a
grammar each of them invents.

`TARGET` is `"lid"`. `Capture::enabled` answers `false` for every other target,
so an application's own spans never enter a validator's tree.

A span opened by `#[implements]` carries these fields and no others:

| Field | Grammar | Read by |
|---|---|---|
| `lid.claims` | the cited claims' `Spec::NAME`s, `'\x1f'`-separated — the same `<Path as Spec>::NAME` form `Edge.spec` already uses (`expand.rs:365`), so the two sides of the join can never disagree about naming. A unit separator rather than a comma because a `NAME` is a path and may not contain one | check 23, and every rule of check 24 to find its citing spans |
| `lid.outcome` | reserved with `field::Empty` at creation and recorded as `()` when the body returns, by a `Returned` guard whose `Drop` checks `std::thread::panicking()`. **Present means returned, absent means unwound**, and today that is all it means: the `Outcome`-derived rendering README §6.4 asks for needs a bound on every cited fn's return type, which the workspace's 549 citation sites do not carry. Deferred 11 | `no_panics` today; `event_driven` and `unwanted` once the bound exists |
| `lid.noun.<n>` | for each parameter position `n`, `noun_of::<T>()` — the parameter's **noun name**, not its value. See the decision below on why no value is recorded | `distinct_inputs`, once values are recorded |
| `lid.feature` | the `feature` gate the item stands behind, absent when there is none | `optional` |
| `lid.state` | the state the span observed, absent when the claim is not state-driven | `state_driven` |

`lid.feature` and `lid.state` are named here because check 24's optional and
state-driven rows require them and nothing else in the system produces them.
Which item supplies them is Open question 1: they are properties of the *claim*,
which `ClaimMeta` already carries as `object` and `trigger`, and of the *call*,
which only the emission sees.

`render` takes `&[&SpecMeta]` for the same reason the rules do: the `✓ promised`
mark means "`wrong_outcome` carries no finding for this claim", which a `Trace`
alone cannot answer. README shows the mark and never says when it is earned;
this document says it here so that Phase 5 can assert the rendered block
exactly.

**Every rule takes `&SpecMeta`, and that is the whole of how it finds its
spans.** A rule's first act is to select the spans whose `lid.claims` holds the
claim's name, and a name is what `SpecMeta` has and `ClaimMeta` does not
(`registry/mod.rs:6-22` against `claim/mod.rs:57-78`). The first draft of this
document said `&ClaimMeta`, and Phase 3 stopped on it: not only is there no name
to join on, but a free claim's `ClaimMeta` is `Ubiquitous` with every part
empty, so 294 of the workspace's 471 claims share one value and could not be
told apart at all. `SpecMeta` is also the type the `#[validates]` expansion
holds after looking the claim up in `SPECS`, so nothing unpacks it on the way
in.

The expected value each rule compares against comes from `SpecMeta::claim`:
`event_driven` and `unwanted` compare `lid.outcome` against
`ClaimMeta::object` (the response object as written), `optional` compares
`lid.feature` against `ClaimMeta::object`, and `state_driven` needs no expected
value because its rule is "at least two distinct". Every rule therefore has the
signature `fn(&Trace, &SpecMeta) -> bool` — one shape for all seven, one thing
for Phase 3 to get right, and one fixture shape for Phase 5.

**A span that unwound is a span with no `lid.outcome`.** The emission cannot
record it with a statement after the body: a body wrapped in `let value = {
.. };` is no longer its fn's tail expression and loses the coercion to the
return type, which broke three consumer crates the first time this was written.
A `Returned` guard records it on `Drop` instead, which runs after the body's
value is produced and before the span's own guard, and which skips the record
when `std::thread::panicking()` — and a unit return records `()` rather than
nothing, so absence means unwound and never "there was nothing to say". That is
how `no_panics` answers without `std::thread::panicking()`, and it is the one
inference in the contract, so Phase 3 confirms it against the real emission
before the cascade.

### What `#[validates]` does with the answer, and why it warns

`#[validates]` installs a `Capture` for the test's duration, runs the body
through `captured`, hands the `Trace` and the cited claims to `report`, prints
what `report` answers when it is not empty, and then resumes any unwind so the
test still fails on its own assertion. `report` evaluates checks 23 and 24 for
each claim, renders the tree beneath a line naming the claim and the check, and
— at `LEVEL == Ramp::Warn` — that is all it does: the test does not fail for a
finding.

The ramp is the point. 294 of the workspace's 471 claims are free, and a free
claim's pattern is `Pattern::Ubiquitous` (`claim/mod.rs:61-62`), whose rule is
`MIN_INPUTS` spans with distinct recorded inputs. Nothing records inputs yet
(see the `Traceable` decision), so every one of those 294 would fail check 24
on the day the emission lands. A slice that made them errors would be a slice
whose own Phase 7 could not pass, and the honest reading of that is not "write
the checks differently" but "this is a brownfield crate arriving at a new
gate", which is the situation README §7's ramp exists for.

### Check 23 is a join, and check 24 is a dispatch over the pattern

Check 23 asks whether any span in the validator's trace cites the claim. It is
the same shape as checks 10 and 11 — a registry join — with one side a runtime
value rather than a distributed slice.

Check 24 reads `ClaimMeta::pattern` and dispatches to one rule per pattern.
Five arms, one call each: cognitive complexity 2 against a threshold of 4, and
`Pattern` has exactly five variants so no wildcard arm is needed and
`wildcard_enum_match_arm` is not at risk. Each rule below it is a leaf that
branches on nothing.

**The ubiquitous row is three promises, and it gets three leaves.** README
writes "at least `min_inputs` spans with distinct recorded inputs, none
panicking" as one italicised sentence (`README.md:898`), but a count, a
distinctness and an absence of panics are independently falsifiable, and a
single leaf holding all three hands check 12 two surviving mutants for free.
`ubiquitous` is a fourth item that calls `enough_spans`, `distinct_inputs` and
`no_panics` — and it is the one place in the slice where a leaf calls other
leaves rather than a dispatch calling them, because it makes no decision: it
answers the conjunction.

### The renderer prints what this slice can know

README §6.4's worked output is the specification for the format, and two things
in it are not this slice's to produce. The `[flow]` marker needs the flow/leaf
classification, which is `lid-rs-shape`'s and behind a package cycle. The
`[redacted]` field needs the policy attributes, which are vocab's Deferred 1 and
need `derive(Traceable)`. The renderer produces the box-drawing tree, the claim
beside the span that cites it, the outcome, and the `✓ promised` mark — and
leaves a column where the marker will go. That is a reduction against the
README and it is Deferred 6, not a silent difference.

### The clippy window this slice opened, now closed

`Capture`'s recording state is written and read only inside `Subscriber`
methods, and those are `todo!()` from Phase 3 until the leaves land. `dead_code`
fired on the field and on the two inside it, and rustc explicitly does not count
the derived `Debug` as a use. Phase 3's and Phase 4's check is `cargo check
--all-targets` (`phase/mod.rs:581-582`), which passed throughout; `cargo clippy
-- -D warnings` did not, from Phase 3 until Phase 6. **It closed by the leaves
being written, which is how such a window is meant to close.**

**Nothing is suppressed for it.** The alternatives are making the recording
public API of a published crate, or writing `captured`'s leaf three phases
early — both worse than a window that closes on its own. It is the same family
as `cargo test --lib` being necessarily red between Phases 3 and 5, and it is
recorded here because a main session running the full gate in that window will
otherwise read it as a regression.

### What the emission cost, measured rather than predicted

Three things the cascade found that no phase could have, recorded because the
next person to change what `#[implements]` expands to will meet all three.

**The body cannot be bound to a local.** `let value = { .. };` takes the body
out of its fn's tail position and with it the coercion to the return type; three
consumer crates stopped compiling, with errors — `Result<&&str, _>` where a
`Result<&str, _>` was expected, and an `FnMut` that "is not general enough" —
that name nothing about spans. The `Returned` guard exists so the body stays
where it was.

**The observer cannot observe itself, twice over and for two different
reasons.** `Capture::new_span` calls `Fields::take` and `cited_claims`, whose
own spans re-enter the subscriber: an unbounded recursion, and a stack overflow.
`Capture::enabled` declining its own module stops that. It does *not* stop the
second: `tracing` calls `register_callsite` while holding the global callsite
registry's lock, and a span opened there needs that lock to register itself —
`enabled` is never reached, because registration precedes it. That method
carries no citation at all.

**A field value must be written inside the `span!` invocation.** Bound to a
local above it, the join runs on every call of every cited fn whether or not a
subscriber exists. Measured on this workspace: `cargo-lid-rs`'s suite went from
41s to not finishing; written inside the macro, where `tracing` skips a disabled
callsite's field expressions, it is 39.9s — indistinguishable from before the
emission.

### What check 12 found in the cascade, and how its red is shown

The cascade landed two items by hand, and the mutation gate reported three
survivors over them: `<impl Drop for Returned<'_>>::drop` replaced with `()`,
the `!` deleted from that `drop`, and `<impl Visit for Fields<'_>>::record_debug`
replaced with `()`. Both are the same defect and it is the one this workspace
keeps finding: **an item written outside a phase carries no claim, so nothing
was ever obliged to test it.**

`Returned::drop` holds an `if`, and a branch is a decision the dispatch/work
rule says must exist as a claim. It had none: the guard was written to solve a
compile error and its two directions — records on return, records nothing while
unwinding — were prose in this document and nowhere in `spec.rs`. They are two
claims now, cited on the `drop`. `record_debug` needed no new claim; a span
opened with a non-string field is a span opened with fields, which
`ARecordedSpanCarriesTheFieldsItWasOpenedWith` already says, and every case
written for it happened to pass a `&str`, which `record_str` takes instead. The
case now passes one of each.

**The guard's `drop` can carry no `#[implements]`, and this is a third reason
distinct from the other two.** `#[implements]` expands to a span *and a
`Returned` over it*, so a citation on `Returned::drop` builds a second guard
inside every drop of the first and recurses without bound. Measured:
`cargo test --lib -p lid-rs` aborts on a stack overflow before the third case,
with no subscriber installed at all — the guard is constructed whether or not
the callsite is enabled, so this would break every program that links the
crate, not only a validator. The two claims are cited by containment with
`implements_module!`, as `register_callsite`'s one is. Two of the slice's
twenty-odd items cannot be cited per-item, and both are items the emission
itself is built out of: **what the expansion is made of cannot be expanded.**

**The red for the guard's two claims is the mutation gate's, not a `todo!()`
skeleton's.** Phase 3's ordinary red — cite the claim, reduce the body to
`todo!()`, watch the case fail — cannot be run here: a panic in a `Drop` that is
already running because of a panic aborts the process, so the unwinding case
would take the test binary down rather than fail. What Phase 5 wants from red is
proof the case discriminates, and cargo-mutants offers exactly that proof over
this body — `()` for the whole `drop`, and the `!` deleted — with no window in
which the workspace cannot run its own tests. A case that kills both mutants has
shown what a `todo!()` would have shown, and check 12 re-runs it on every later
change.

### Where this slice's work can be written, and what it cannot reach

`validate` is a module slice whose own crate is `lid-rs`, an ordinary library:
`execution_class` is `Ordinary` (`policy.rs:753-760`), the acceptance gate
returns no reason, and Phases 3 and 4 may write `lid-rs/src/validate/` and
`lid-rs/src/lib.rs`, Phases 5 and 7 the module directory alone
(`policy.rs:417-424`).

### What this slice is not

It is not `derive(Traceable)`, which is `noun_assertion`'s and blocked on a
human's acceptance file. It is not the policy attributes. It is not the
`validate` command, which is `cargo-lid-rs`'s. It is not check 25, not `spawn`,
and not the flip from `warn` to `deny` — all in Deferred below.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| HLD row 20 is cut, and this slice is the runtime half in `lid-rs` | `lid-rs/src/validate/`: the tree, the capture, checks 23 and 24, the renderer | One slice spanning both crates as the crate-root slice `lid-rs-macros`; the runtime half alone with the emission left unscheduled; defer the whole row until `noun_assertion` lands | A crate-root slice named `lid-rs-macros` needs `lid-rs-macros/src/compile-time-accepted`, a human commit, and `Form::CrateRoot::spec_file` points its Phase 2 at `lid-rs/src/spec.rs` — a path that does not exist and that no slice has exercised. Leaving the emission unscheduled is what slices 18, 19 and 21 each did, and the result is three checks still `Unbuilt` in the catalog after their slices completed. Deferring the row stops on `noun_assertion`'s acceptance file, which is not this slice's to get. |
| The emission lands after the leaves and before the gate, in a main-session sequence | Phase 5 (agent) → `phase 6:` leaves (main session) → `cascade:` emission + `0.3.0` (main session) → `phase 7:` gate (main session, by hand) | The cascade between Phases 5 and 7, as the first draft of this document said; the cascade before Phase 5; the cascade after the Phase 7 gate | **There is no Phase 6 commit under the phase agents**: `Phase` has no `Six` variant, no `lid-rs-phase-6` agent exists, and `ending.rs:254` rejects the subject — the `lid-rs-phase-7` agent implements the leaves *and* gates in one run, which slice 19 recorded as an ordering correction (`outcome/lld.md:472-489`). So "between 5 and 7" means *before the leaves exist*, and the emission calls `Capture` and `captured`, which are `todo!()` until then: every `#[validates]` test in five crates would panic inside the subscriber, for a reason no check can name. A main-session `phase 6:` commit is available — there is no `commit-msg` hook and `3775771` is one — and this slice's Phase 7 is hand-run anyway, as slices 19, 21, 22 and 17 all were. Before Phase 5 is refused because a red must come from an unimplemented check and not from an absent emission, and `red_verdict` cannot tell those apart. |
| Checks 23 and 24 are asserted by `#[validates]` at a ramp, arriving at `warn` | `LEVEL: Level = Level::Warn`: the finding and the rendered tree are printed, the test does not fail | Assert as errors from the first commit; do not assert at all and ship the checks unwired; assert only for claims in the crate under test | 294 of 471 claims are free, a free claim's pattern is ubiquitous, and the ubiquitous rule needs recorded inputs that this slice deliberately does not record — so errors on arrival means 294 failures the slice cannot fix. Not asserting at all is the gap this slice exists to close, and it is what left `shape`, `conform` and `regen` unbuilt. Scoping by crate hides the same failures behind a narrower window and gives no count. The ramp is README §7's own design for exactly this case (`:1236-1240`). |
| `tracing` only, with a hand-written `Subscriber` | `tracing = { version = "0.1", default-features = false, features = ["std"] }`, and `Capture` implementing `Subscriber` | `tracing-subscriber`'s `Registry` and a `Layer`; a `lid-rs`-owned recorder with no `tracing` at all; `tracing` with default features | Spiked today: the hand-written subscriber captures names, targets, fields, parentage, an exit-recorded value and close, with a four-crate dependency tree. `tracing-subscriber` adds a large tree to do span storage this slice does not need, since the capture is per-test and the tree is kept whole. An own recorder was refused at `hld.md:238` because it proves the checks and gives no production story. Default features pull `tracing-attributes` and with it `syn`, needed only by `#[tracing::instrument]`, which is the async path this slice defers. |
| `Capture` implements `register_callsite` explicitly, returning `Interest::sometimes()` | One extra method, one line | Rely on the default, which derives interest from `enabled` | Spiked today and the hazard **did not** reproduce: a later dispatcher still saw a callsite an earlier filtering `Capture` had rejected. It is recorded as a decision anyway because the default caches `Interest::never()` per callsite, the interaction with scoped dispatchers is undocumented, and the mitigation is one method — where the failure it prevents is a silently empty tree in a test that runs after another test, which is the hardest possible thing to diagnose. |
| The emission reaches `tracing` through `lid_rs::__private::tracing` | A re-export beside `Declared` and `linkme` | Add `tracing` to all five members' manifests; name `::tracing::` in the expansion | Four of the five crates carrying `#[implements]` have no `tracing` dependency and no reason to gain one — `cargo-lid-rs` alone has 443 sites. Naming `::tracing::` in an expansion requires every consumer to depend on it, which is the mistake `__private::linkme` already exists to avoid (`expand.rs:362-363`). Five manifests are five things to keep in step and none of them is in a phase's row. |
| A span records a parameter's **noun**, not its value | `lid.noun.<n> = noun_of::<T>()`, and no `Traceable` method is added | Add a required recording method to `Traceable`; add a provided method with a default; record a `Debug` rendering | vocab's Open 1 handed this slice the decision and named its cost: "adding a required method to a published trait is a breaking change that only `cargo package` reports". There is nothing to enforce a policy *over* until `derive(Traceable)` exists, so the method would ship with no user of it and one breaking release. A provided method with a default is the same breaking change deferred by one version. A `Debug` rendering requires `T: Debug` at every traced boundary, which is a bound on user code this slice has no claim to impose — and it would record exactly the unredacted values README §6.5 exists to prevent. The consequence is stated rather than hidden: `distinct_inputs` cannot be satisfied by a real trace until the method lands, which is why the ramp arrives at `warn`. |
| The checks take a `Trace` and a `SpecMeta` as plain arguments | `fn(&Trace, &SpecMeta) -> bool` for every rule and for `unreached`; `render(&Trace, &[&SpecMeta]) -> String` | `&ClaimMeta`, as the first draft said; `(&Trace, &str, &ClaimMeta)`, the name passed beside the meta; adding `name` to `ClaimMeta`; checks that run a closure under a live capture | `&ClaimMeta` cannot work: it carries no name, so a rule cannot select its citing spans, and a free claim's value is `Ubiquitous` with every part empty so 294 claims share one. Passing the name beside the meta needs no Phase 2 rework, but it is two values that always come from the same `SpecMeta` and may be made to disagree — which is exactly what `Edge.spec` being produced from `<Path as Spec>::NAME` exists to prevent (`registry/mod.rs:10`). Adding `name` to `ClaimMeta` edits slice 15's published type and spells the same string in two places one level apart. A live capture cannot be tested at Phase 5, when the emission does not exist. |
| `captured` catches an unwinding body and resumes the unwind | `std::panic::catch_unwind(AssertUnwindSafe(body))`, then the checks and the rendered tree, then `resume_unwind` | Let the body unwind and lose the trace, as the first draft did; catch and swallow | README §6.4's deliverable is that `#[validates]` "captures the span tree, and **prints it on failure**" — and a validator's failure is a panic, so a `captured` that unwinds with the body drops precisely the trace that was worth printing. Phase 3 found this: it also means `no_panics`, whose whole subject is a span that recorded no `lid.outcome` because something unwound, could be reddened by a hand-built fixture and never fed by the real emission. Swallowing the panic would make a failing test pass, which is the one thing a test harness may never do. `AssertUnwindSafe` is the standard harness pattern; the closure is `FnOnce` and nothing observable is left half-written. |
| The rendered tree draws one branch for every span, and the `✓ promised` mark stands beside the claim | `├─` for every span regardless of position; the mark immediately after the claim it is about | README §6.4's `├─` for a span with a later sibling and `└─` for the last; the mark at the end of the line, as README draws it | No claim of this slice tells a last sibling from any other, and a renderer that decided it would be deciding something nobody wrote down — the dispatch/work rule's "a leaf with a branch in it is a requirement nobody wrote down", met in the renderer. The mark moves because a span cites as many claims as its `#[implements]` names — up to eight in this slice's own code — and one mark at the end of a line naming two claims cannot say which of them was kept. Both are deviations from README §6.4's worked block and both are Phase 5's to pin exactly; they join the `[flow]` marker and `[redacted]` fields in Deferred 6's reconciliation. |
| `min_inputs`, the span level and the ramp level are constants | `MIN_INPUTS = 16`, `SPAN_LEVEL: tracing::Level = DEBUG` (README §7's own default), and `LEVEL: Ramp = Ramp::Warn` | Read them from workspace metadata; read them from an environment variable the gate sets; widen `project.rs` to expose nested settings | `Project::workspace_setting` is private to `cargo-lid-rs`, which `lid-rs` cannot depend on, and `setting_in` reads only flat keys — a nested `runtime.level` is not expressible through it at all. Slices 18, 19 and 21 each met this wall and declined; slice 19's wording is "Severity is fixed, not read from `conformance.level`". 16 is README's own default. An environment variable is a second configuration surface nobody documented. Widening `project.rs` is a second slice's work and would unblock three other slices' levels, which is an argument for doing it deliberately and not inside this one. |
| The capture is thread-local, scoped by `dispatcher::with_default` | One capture per validator, for the closure's duration | README §6.8's root-keyed global layer | The global layer exists to key an *async* validator's tree by the test that opened it. Nothing here is `async`, `spawn` is deferred with it, and a global layer shared across `cargo test`'s parallel threads is a synchronisation problem bought for a case that cannot arise. Deferred 2 records it so the async slice does not rediscover it as a bug. |

## Open Questions & Future Decisions

### Open

**1. Which item supplies `lid.feature` and `lid.state`.** Check 24's optional
row needs the feature the item stands behind and its state-driven row needs the
state a span observed. Neither is derivable from `ClaimMeta` alone: the claim
says *which* feature is promised, the call says whether it was active. The
emission could read a `#[cfg(feature = …)]` on the cited item — a syntactic pass
over one item's tokens, which the parsing carve-out permits — and a state has no
syntactic source at all. **This does not block Phase 2**, because the two rules
are claims about what the *rule* answers given a trace, not about how the trace
was produced, and Phase 5 validates them against fixtures. It blocks the
cascade, which is four commits later.

**2. Whether the emission carries claims of its own.** Slice 19 gave its macro
half nine of eighteen claims. Here the emission's claims would have no
implementer in `lid-rs/src/validate/` and could only be reddened by a validator
that runs a traced function — which the Phase-5 notes forbid, because the
emission does not exist then. The honest options are: no claims, and the
emission is held by the checks' claims alone; or claims derived at Phase 2 and
reddened only after the cascade, which no phase's red set can express. Stated
here rather than left to Phase 2, which would otherwise guess.

**3. Whether `#[implements]` on a struct or an enum should say anything at
runtime.** `citation` admits fns, structs and enums, and a span can only wrap a
body. Those citations produce no span, so check 23 warns for every claim whose
only implementer is a type. That is arguably correct — README's answer for
data-implemented claims is check 24's many-inputs rule reached through the
function that reads the data — and arguably a hole. Under the ramp it is a
counted warning rather than a failure, which is the right place for a question
this size.

**4. Whether check 24's ubiquitous rule can be honest before values are
recorded.** It cannot, and the ramp is the answer for now. Stated as an open
question rather than a deferral because the flip to `deny` cannot happen until
it is closed, and the flip is what makes the slice's checks gate anything.

### Deferred

1. **`lid_rs::spawn` and async span propagation (README §6.8).** Cause: it needs
   an async runtime. Zero occurrences of `async fn`, `tokio` or `futures` exist
   in the tree, so a claim about propagation across `await` could have no
   validator here that was not a mock — which CLAUDE.md forbids. It also
   restores the `tracing-attributes` feature. Its own slice, once something in
   the workspace is async.

2. **The root-keyed global capturing layer (README §6.8).** Cause: same as 1.
   Recorded separately because it is a different piece of machinery, and a
   future reader finding `with_default` here should find the reason rather than
   a divergence from the README.

3. **Check 25, the unreached flow node.** Cause: it needs the union of every
   validator's trace and the flow/leaf classification, which is `lid-rs-shape`'s
   — a crate that depends on `lid-rs`, so the edge is `error: cyclic package
   dependency`, spiked at `trace/lld.md:323`. README's own answer to the union
   is a file, `capture = "target/lid/traces"` (`README.md:1215`), and persisting
   traces is a decision this slice does not take: it would give every test in
   the workspace a filesystem write. **Two consequences that need an HLD
   amendment and are not this document's to make:** the HLD's only stated home
   for check 25 is "beside 18" (`hld.md:219`), and slice 18 has shipped, so on
   this deferral check 25 has no owner slice at all; and `lid-rs-site` was told
   it "needs slice 20's captured traces" (`trace/lld.md:322`), which without
   persistence it will not get. Both belong in the documentation commit named
   below.

4. **Persisting traces to `capture = "target/lid/traces"`.** Cause: separated
   from 3 because it is a behaviour and 3 is a check. It is the precondition for
   both check 25 and `lid-rs-site`'s worked examples, and it is a filesystem
   write on every test in the workspace — which is a decision, not a detail.

5. **`[workspace.metadata.lid_rs.runtime]` as README §7 writes it.** Cause: the
   settings reader is private to `cargo-lid-rs` and handles only flat keys. This
   is the fourth slice to record it, which is the argument for fixing the reader
   as its own work rather than a fifth deviation.

6. **The `[flow]` marker and `[redacted]` fields in the rendered tree.** Cause:
   the marker needs `lid-rs-shape`'s classification, behind the cycle of item 3;
   redaction needs the policy attributes of item 7. The renderer leaves the
   column and the format is otherwise README's.

7. **The policy attributes `#[trace(redact)]`, `#[trace(skip)]` and the field
   selection (README §6.5), and the `Traceable` recording method they need.**
   Cause: vocab's Deferred 1 and Open 1 handed forward. There are no user nouns
   until `derive(Traceable)` exists, which is `noun_assertion`'s and blocked on
   a human's acceptance file.

8. **The flip from `warn` to `deny`.** Cause: it cannot happen until Open
   question 4 is closed — the ubiquitous rule needs recorded inputs, which need
   item 7. It is the change that makes checks 23 and 24 gate, and it is named
   here so that "the checks are shipped" is not read as "the checks are
   enforced".

9. **The `cargo lid-rs validate` command.** Cause: it is `cargo-lid-rs`'s, is
   reserved as `Unbuilt::Slice("slice 20")`, and `cargo-lid-rs` is in no path
   row of this slice. The precedent is that `shape` and `conform` are still
   `Unbuilt` after slices 18 and 19 completed.

10. **The typed outcome and the recorded parameters (README §6.4).** Cause: the
    emission records that a body returned and not what it returned, and records
    no parameters at all. Both need bounds on cited items — `Outcome` on every
    return type, `Traceable` on every parameter — which the workspace's 549
    citation sites do not carry and cannot be given until `derive(Traceable)`
    and `derive(Outcome)` are usable on a consumer's own types. Until then
    `event_driven`, `unwanted`, `optional`, `state_driven` and `distinct_inputs`
    are validated against hand-built traces and warn against real ones, which is
    what the ramp is for.

11. **The README and HLD reconciliation.** README §6.4's parameter recording,
    §6.5's policy, §6.8's global layer and §7's runtime table all now differ
    from what is built, and HLD row 20 owes the cut, the ramp, and check 25's
    lost home. Cause: no gate step reads README against the code. **One
    documentation commit from the main session after this slice's Phase 7**,
    written against what was built, as slice 17's was.

## What lands by hand, and in what order

Six things no phase of `lld/validate` may write. Each was checked against
`policy.rs` and against what the phase after it needs.

**1. The `tracing` dependency, in `Cargo.toml` and `lid-rs/Cargo.toml` — before
Phase 3.** The workspace manifest is at the repository root, outside every
crate, so `seat_of` refuses it before any table is consulted
(`policy.rs:109-118`); a member's `Cargo.toml` is in no phase's row either. It
must precede Phase 3 because the skeleton names `tracing::Subscriber` in an item
signature. The line is
`tracing = { version = "0.1", default-features = false, features = ["std"] }`
in `[workspace.dependencies]` and `tracing = { workspace = true }` in
`lid-rs/Cargo.toml`.

**2. `pub mod validate;` in `lid-rs/src/lib.rs`, and `lid-rs/src/validate/mod.rs`
holding `#![doc = include_str!("lld.md")]` and `pub mod spec;` — between Phases
2 and 3.** Phase 2's row is the claims file and `src/spec/mod.rs`
(`policy.rs:455-461`), so it can write the claims and cannot declare them, and
`cargo check --all-targets` passes happily over a file no module compiles — so
**check 13 never reads the claims** and the failure surfaces at Phase 5 as
"nothing registers". Slices 18 (`4b036c7`), 19 (`705ab1b`), 21 (`04f1388`) and
17 (`a2b44b0`) all landed it by hand between Phases 2 and 3, and all four then
had every claim accepted first time.

**3. `pub use tracing;` in `lid-rs/src/lib.rs`'s `__private` — at Phase 3 or 4,
which is in that row.** Not a hand commit, and listed here because it is easy to
miss and its absence is discovered at the worst moment. Four of the five crates
carrying `#[implements]` have no `tracing` dependency, so the expansion must
reach it the way it already reaches `linkme` (`expand.rs:362-363`). If it is
missing, the cascade fails to compile four crates and the fix is in a file
Phases 5 and 7 may not write.

**4. `phase 6:` — the leaves, from the main session, after Phase 5.** There is
no phase-6 agent and the phase-7 agent implements *and* gates in one run, so the
leaves must exist before the cascade or every validator in the workspace panics
inside a `todo!()` subscriber. `3775771 phase 6: leaves for claim` is the
precedent and there is no `commit-msg` hook to refuse the subject.

**5. The emission in `lid-rs-macros/src/expand.rs`, with the version bump to
`0.3.0` — after the leaves, before the gate.** In no phase's path row; slice
19's `da047cd` is the precedent and the wording. The version goes in the same
commit because changing what `#[implements]` expands to is a breaking change to
a published crate that only `cargo package` reports — the trap that cost
`ce0d86f` and `563e7ab` a bump each.

**6. The claims and cases over what the cascade wrote — after the cascade,
before the gate.** The phase agents cannot run them: Phase 5's check reddens
every claim added since the last gate, and by this point every claim of the
slice is implemented, so the agent's own hook would refuse a run that is
correct. They land from the main session as `phase 1:`, `phase 2:`, `phase 3:`
and `phase 5:` in that order — the `phase 3:` commit is the citation alone,
with no `todo!()` under it for the reason above, and it is a containment
citation for the reason below — and their evidence is check 12: the three
survivors
it reported over `Returned::drop` and `Fields::record_debug` are the red, and
the gate re-runs them. This item exists because the cascade is the one place in
the slice where code is written outside a phase, and a hand-written item with a
branch in it is the defect this workspace keeps finding.

## Notes for Phase 5's red set

Ten things a red test of this slice gets wrong by default.

1. **Do not write a test that runs a traced function.** The emission does not
   exist during Phase 5, deliberately. Every case builds a `Trace` value by hand
   and hands it to the check. A test that installs a capture and calls something
   is testing the emission, which is not written.

2. **`Capture` is the exception, and its red is real.** It is the one item that
   must be exercised through `tracing`'s API, and it can be: open a span by hand
   inside `dispatcher::with_default`, then assert the captured tree. Real
   subscriber, real spans, not a mock — and red while `Capture`'s methods are
   `todo!()`.

   **Two of its claims need more than a `span!`.** `register_callsite` takes a
   `&'static Metadata<'static>`, which the `span!` macros never hand to a test,
   so the interest claim needs a `static impl tracing::Callsite` and a
   `Metadata::new` beside the `Trace` helper of note 6 — still real `tracing`
   API and still not a mock, but a second fixture shape. And the claim that a
   field recorded after creation reaches its span needs `field::Empty` reserved
   at creation and a later `Span::record`, which is the mechanism the spike
   confirmed and the one the emission depends on.

3. **Do not cite a claim on `TARGET`, `MIN_INPUTS`, `LEVEL` or `SPAN_LEVEL`.** A
   const cannot be skeletonised — `const TARGET: &str = todo!();` does not
   compile — so any test of one is green from Phase 3 and `red_verdict` refuses
   it (`phase/mod.rs:912-919`). This is the rule vocab stated for
   `Traceable::NOUN` and slice 21 for `DOCUMENT`: a claim whose only implementer
   is data is cited on the function that reads it. Here that function is
   `wrong_outcome` or the rule that uses the constant.

4. **The red set is about eighteen validators, not eight.** Phase 2 derived
   forty-three claims, and most rules carry two — one per direction, so that a
   rule inverted is falsifiable and check 12 has no free survivor. Roughly:
   sixteen for check 24's seven rules, six for its dispatch, two for check 23,
   thirteen for `Capture` and `captured`, six for `render`. One validator may
   carry both of a rule's claims, because check 14 admits a name matching any
   one cited claim — so plan around eighteen tests rather than one per claim,
   and never around one per leaf. A single test over one pattern leaves the
   other six rules with a `todo!()` nothing reaches.

5. **Assert the rendered tree exactly**, against the format this slice
   produces — which differs from README §6.4's worked block in four ways, all
   deliberate and all recorded: no `[flow]` marker and no `[redacted]` field
   (Deferred 6), one `├─` branch for every span rather than `└─` for a last
   sibling, and the `✓ promised` mark beside the claim it is about rather than
   at the end of the line. Do not paste README's block. `render` returns a `String`, so check 12 substitutes `""` for
   its body and a non-empty assertion is a free survivor.

6. **A `Trace` built by hand must come from a helper the tests share.** Eight
   leaves' worth of cases each constructing spans inline is eight chances for
   the fixture to disagree with itself. Slice 19's `entry(...)` and slice 21's
   section builders are the precedent.

7. **The rules take `&SpecMeta`, so a fixture needs one.** `SpecMeta` is a
   `&'static str` name, a file, a line and a `ClaimMeta` whose own fields are
   all `&'static str` and `&'static [&'static str]` (`registry/mod.rs:6-22`,
   `claim/mod.rs:57-78`), so a `const` fixture is possible and is preferable to
   reaching into `SPECS` for a real claim — a real claim's `pattern` is whatever
   its sentence said, and a fixture that depends on another slice's prose breaks
   when that prose is edited. Give the fixtures distinct `name`s: the whole
   point of `SpecMeta` over `ClaimMeta` is that a rule selects its spans by
   name, and two fixtures sharing one name test nothing.

8. **`captured` catches an unwind, so its own reds are two-sided.** One case for
   a body that returns and one for a body that panics — the second is what
   proves the trace survives, and it is the only case that can produce a span
   with no `lid.outcome` without hand-building one.

9. **Claim names beginning with `A` may re-bless a fixture that has nothing to
   do with this slice.** `lid-rs/tests/ui/fail/not_a_spec.stderr` pins rustc's
   alphabetically-truncated eight-name list of `Spec` implementors, whose eighth
   entry today is `ACrateRootClaimsSliceIsEmpty`. A new `lid-rs` claim sorting
   before it displaces a name and fails that fixture at Phase 7. It is
   re-blessed with `TRYBUILD=overwrite`, never hand-edited; slice 21 did exactly
   this at `2c9c59d`, and slice 17 escaped it only because its `A`-initial
   claims sorted after.

10. **A field case that only ever passes a `&str` leaves half of `Fields`
    untested.** `tracing` routes a string field to `Visit::record_str` and
    everything else to `Visit::record_debug`, and the two are separate bodies
    here because a `Debug` rendering of a string quotes it — which would split
    `lid.claims` into a first claim carrying a quote and a last one carrying
    another. A case asserting a span carries the fields it was opened with must
    pass one of each; passing two strings asserts one body twice.

## Shape

| Item | Role |
|---|---|
| `lid_rs::validate::SEPARATOR` | `'\u{1f}'` — the unit separator `lid.claims` joins on and `cited_claims` splits at. One spelling, reachable from the emission in the other crate, because two spellings are a silent empty join and the emission cannot see a `const` it does not name. |
| `lid_rs::validate::report` | Work leaf: `report(&Trace, &[&str]) -> String` — the text `#[validates]` prints, empty when there is nothing to say. For each named claim it runs `unreached` and `wrong_outcome` and, at `LEVEL == Ramp::Warn`, renders the finding and the tree beneath it. **It is what makes the checks fed rather than merely built**, and it returns the text rather than printing it so that it is a leaf a test can read — the printing is one `eprintln!` in the expansion, where the I/O belongs. Missing from the first six phases: the Behaviour section required it from the first draft and no Shape row had it, which is the defect the emission cascade found. |
| `lid_rs::validate::TARGET` | `"lid"` — the span target the emission writes and `Capture::enabled` filters on. Spelled once, because a crate that cannot see this constant must agree with it and a second spelling is a silently empty tree. |
| `lid_rs::validate::SPAN_LEVEL` | `tracing::Level::DEBUG`, the level the emission opens spans at — README §7's own default (`level = "debug"`). A constant because the metadata table it belongs in cannot be read from this crate; Deferred 5. |
| `lid_rs::validate::Ramp` | `Warn` or `Deny` — README §7's ramp as a type of this slice's own. **Not `tracing::Level`**, which has neither variant and whose values name a span's verbosity rather than a check's severity; the two stand one line apart in this module and a shared name would be read as a shared meaning. |
| `lid_rs::validate::LEVEL` | `Ramp::Warn` today, so a finding prints and does not fail. The flip to `Ramp::Deny` is Deferred 8 and is what makes checks 23 and 24 gate. |
| `lid_rs::validate::MIN_INPUTS` | 16, README's own default (`README.md:906-907`), fixed here for the same reason as `SPAN_LEVEL`. |
| `lid_rs::validate::Span` | One captured span as data: `name`, `parent` as an index into the `Trace`, `claims` split from `lid.claims`, `outcome` as `Option<String>` (absent means the body unwound), and the remaining `fields` as recorded. Parentage as an index rather than nesting, because a subscriber builds incrementally and a nested type cannot be. |
| `lid_rs::validate::Trace` | A validator's whole tree, in creation order. The argument every check takes and the value the renderer prints. |
| `lid_rs::validate::Capture` | The `tracing::Subscriber` implementation, and the only item in the slice that touches `tracing`'s API. Interior mutability behind a `Mutex`, because `Subscriber`'s methods take `&self` and `captured` must read the tree back out after the dispatcher is dropped. |
| `Capture::enabled` | The filter, and it refuses two things. Every target that is not `TARGET`, so an application's own spans never enter a validator's tree; and **every callsite inside this module**, so the capture does not observe itself. The second is not a workaround: `Capture::new_span` calls `Fields::take` and `cited_claims`, whose own `#[implements]` spans re-enter the subscriber and recurse without bound — measured, as `new_span → take → cited_claims → enabled → new_span`. `tracing`'s own guard covers a span opened *directly* inside a callback and not one opened by a function that callback calls. The observer is not part of what it observes. |
| `Capture::register_callsite` | Returns `Interest::sometimes()` explicitly rather than letting the default derive it from `enabled`. **It carries no `#[implements]`, and it is the only item of the slice that cannot.** `tracing` calls this method while it holds the global callsite registry's lock, so the span a citation would open needs that same lock to register its own callsite and the run stops — measured twice: the workspace suite finishes in seconds single-threaded and never finishes in parallel. Its one claim is cited by containment with `implements_module!`, as the registry slice's enumeration claim is. Check 12 still reaches the body through the validator, so only the per-item citation is traded. |
| `Capture::new_span` | Records the span: its metadata's name, the parent from the capture's own enter-stack, and the fields at creation through a `Visit` implementation. |
| `lid_rs::validate::Fields` | The `tracing::field::Visit` implementation `new_span` and `record` both feed. It is the only place a `tracing` field value becomes a `String`, so it is where the `Debug`-versus-`str` distinction is decided once instead of at two call sites. `record_str` takes a string as it stands and `record_debug` renders everything else, both through `take`; a case over the fields a span carries has to pass one of each or it exercises only one of the two. Named in the document's prose from the first draft and given no row until Phase 3 warned that Phase 4 would invent it. |
| `Fields::take` | Where one recorded field lands, and the only place the wire contract's three destinations are spelled: `lid.claims` to the split, `lid.outcome` to `Span::outcome`, every other field to `Span::fields` under the name it was recorded with. **Three destinations are three decisions and each is a claim** — Phase 4 reported that only two of them had one, and that the routing of `lid.outcome` was a branch nobody had written down. |
| `lid_rs::validate::cited_claims` | The split of `lid.claims` at the unit separator into a span's `claims`. One leaf, because the separator is the wire contract's and a second spelling of `'\x1f'` is a silent empty join. |
| `Capture::record` | Takes the fields recorded after creation, which is how `lid.outcome` reaches the span at all: `field::Empty` reserves the slot and this fills it. |
| `Capture::enter`, `Capture::exit` | The enter-stack that gives every span its parent, and the close. Nothing else; `event` and `record_follows_from` are empty because this slice reads neither. |
| `lid_rs::validate::Returned` | The guard that records a span's outcome when its body returns and does not when it unwinds. It exists because the outcome cannot be recorded by a statement after the body without costing the body its tail-expression position, and with it the coercion to the fn's return type — which broke three consumer crates before it was written. `Drop` in reverse declaration order puts it after the value is produced and before the span's guard, which is exactly when the outcome is known. **Its `Drop` holds the slice's other branch, and it arrived in the cascade with no claim over either direction** — check 12 found that, and both directions are claims now, cited by containment because `#[implements]` on this `drop` would construct a `Returned` inside every drop of a `Returned`. |
| `lid_rs::validate::captured` | Work leaf: runs a closure under a fresh `Capture` scoped by `dispatcher::with_default`, **catching an unwinding body with `catch_unwind(AssertUnwindSafe(..))`** so the trace survives the failure that made it worth reading, and answering the `Trace` alongside what the body did. Its caller resumes the unwind after the checks have printed. What `#[validates]` expands to, and the seam between the emission and everything this slice can test without it. |
| `lid_rs::validate::unreached` | Check 23: `unreached(&Trace, &SpecMeta) -> bool` — whether no span in the trace cites the claim's `SpecMeta::name`. A registry join with one runtime side, and the same signature as every rule below it. |
| `lid_rs::validate::wrong_outcome` | Check 24's dispatch over `SpecMeta::claim`'s `pattern` — five arms, one call each, cognitive complexity 2 against a threshold of 4. The slice's one flow node. |
| `lid_rs::validate::event_driven` | Check 24's event-driven rule: at least one citing span's `lid.outcome` closes `Ok(<response type>)`, the type read from `SpecMeta::claim`'s `object`. |
| `lid_rs::validate::unwanted` | Check 24's unwanted rule: at least one citing span's `lid.outcome` closes `Err(<response variant>)`, the variant read from `SpecMeta::claim`'s `object`. |
| `lid_rs::validate::optional` | Check 24's optional rule: at least one citing span carries `lid.feature` matching `SpecMeta::claim`'s `object`. |
| `lid_rs::validate::state_driven` | Check 24's state-driven rule: at least two distinct `lid.state` values across the citing spans — a transition actually occurred. Needs no expected value, which is why every rule takes `&SpecMeta` rather than a pre-extracted string. |
| `lid_rs::validate::ubiquitous` | Check 24's ubiquitous rule, the conjunction of the three below. A leaf that calls leaves and makes no decision: README writes the row as one sentence, but a count, a distinctness and an absence of panics are independently falsifiable and one leaf holding all three is two free mutants. |
| `lid_rs::validate::enough_spans` | At least `MIN_INPUTS` citing spans. |
| `lid_rs::validate::distinct_inputs` | The citing spans' `lid.noun.*` fields are distinct. Cannot be satisfied by a real trace until a `Traceable` recording method exists (Deferred 7), which is why the ramp arrives at `warn`. |
| `lid_rs::validate::no_panics` | No citing span unwound — which is to say, every one of them recorded a `lid.outcome`. The one inference in the wire contract, and the one thing Phase 3 confirms against the real emission. |
| `lid_rs::validate::render` | Work leaf: `render(&Trace, &[&SpecMeta]) -> String`. The tree as this slice can print it: box-drawing, the claim beside the span that cites it, the outcome, and the `✓ promised` mark. **It takes the claims as well as the trace**, because the mark means "`wrong_outcome` carries no finding for this claim" and nothing in a `Trace` alone says that — the first draft of this row named no inputs at all, and Phase 2 reported that two of its claims had no implementer under the reading that omitted them. The test's output *is* the requirements it exercised, so this is a deliverable and not a debugging aid. The `[flow]` marker and `[redacted]` fields are Deferred 6. |
| `lid_rs::__private::tracing` | The re-export the emission reaches `tracing` through, beside `Declared` and `linkme`. Four of the five crates carrying `#[implements]` have no `tracing` dependency; this is what `expand.rs:362-363` already does for `linkme`. Lands in `src/lib.rs` at Phase 3 or 4, which is in that row. |

Every item above is a type, a const, one dispatch, one conjunction, or one
leaf. The dispatch is `wrong_outcome` and it makes exactly one decision; the
conjunction is `ubiquitous` and it makes none. `Capture`'s **eight** trait methods
are listed individually because each is a place a claim would be cited and a
one-line summary of an eight-method impl is how a Phase 3 agent comes to invent
its own shape. Seven are `tracing::Subscriber`'s required set and the eighth is
`register_callsite`, which the decisions table adds; the first draft of this
paragraph said seven, having counted the trait and forgotten its own decision.

**A note on doc links in this document.** It contains no intra-doc link, and
that is deliberate rather than an omission. Nothing includes this file until
`lid-rs/src/validate/mod.rs` lands (item 2 of "What lands by hand"); from that
commit onward every link in it must resolve under `-D
rustdoc::broken_intra_doc_links`, so a link written now against a type Phase 3
has not created yet would break `cargo doc` for two commits. Locations are plain
`file:line` text, which is the form the trace and vocab documents settled on.

## References

- README [§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html) — checks 23,
  24 and 25, and check 24's per-pattern table, which is this slice's
  specification.
- README [§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html) — what each
  attribute emits, and the worked tree output.
- README [§6.5](https://bradvoth.github.io/lid-rs/spec/traced.html) — the
  `Traceable` policy, deferred here and vocab's Deferred 1.
- README [§6.8](https://bradvoth.github.io/lid-rs/spec/traced.html) — async, the
  global layer and `spawn`; Deferred 1 and 2.
- README [§7](https://bradvoth.github.io/lid-rs/spec/configuration.html) —
  `[workspace.metadata.lid_rs.runtime]`, which this slice cannot read, and the
  `warn`-to-`deny` ramp, which it uses.
- `lid-rs/src/outcome/lld.md` — slice 19, for the fixed-severity precedent, for
  `da047cd`'s cascade shape, and for the Phase 6 ordering correction this
  document's first draft repeated.
- `lid-rs/src/vocab/lld.md` — slice 17, for `Traceable`, its Open 1 and its
  Deferred 1.
- `lid-rs/src/trace/lld.md` — slice 21, for the `$crate`-only constraint, the
  spiked package cycle, and `lid-rs-site`'s dependency on captured traces.
- `lid-rs/src/claim/lld.md` — slice 15, for `Pattern` and `ClaimMeta`.
