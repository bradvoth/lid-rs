# A function is flow or leaf by its shape

## Context and Design Philosophy

§0's rule is that dispatch and work are separate functions: a function either
makes one flow decision or does one unit of work, never both, and *a leaf with
a branch in it is a requirement nobody wrote down*. The rule is this project's
premise. Nothing enforces it.

What enforces something adjacent is check 7, `clippy::cognitive_complexity` at
a threshold of 4. README is explicit about what that misses: "What check 7
cannot see — a flat dispatcher whose arms do work — is rule A's job (check
15)." A twelve-arm `match` whose arms each do work has complexity 1 and passes.

The gap is a definition. §0's rule needs a notion of *dispatch* a tool can
apply and nobody can opt out of — and README §3.7 says why a marker cannot be
it: "a marker attribute that declares 'this function is routing' is evaded by
omitting it. So the classification is derived from the body, and the markers
only pin it."

### Constraint 2 permits this, in its own words

This slice parses Rust source, which reads like the thing constraint 2
forbids. It is not, and the constraint says so itself:

> Syntactic passes over one item's tokens are permitted — classifying a body's
> shape, reading a signature's type tokens — because they resolve nothing;
> **the line is resolution, not parsing.**

Those two permitted operations are this slice's two operations. README §3.7
names it again: "The shape pass is a `syn` pass over the source of one crate —
permitted by constraint 2 because it resolves nothing." Two boundaries follow
and bind every decision below: the pass reads **one crate's** source, not a
workspace graph; and rule V compares **written type tokens**, never resolved
paths. A V that chased `String` through a `use` alias would cross the line.

`CLAUDE.md` states the carve-out and names this slice as what it is for
(`CLAUDE.md:93-98`), so this workspace's agents are not misdirected. The file
that states the prohibition without the carve-out is
`cargo-lid-rs/templates/AGENTS.md:63` — "Parse Rust source to reconstruct the
intent graph" — which `cargo lid-rs init` writes into **every scaffolded
downstream project**, where it reads as forbidding a check the tool ships.
Deferred 5.

### Rule V needs no vocabulary, which is why this slice does not wait

README §3.6 states rule V as an **allow-list**: "Parameter types and the `Ok`
type may be vocab types, `Outcome` enums, or those wrapped in `&`, `&mut`,
`Option`, `Result`, `Vec`, `Box`, `Arc`. Not `String`, `&str`, `bool`,
integers, floats, `char`." The distinction is not cosmetic: a deny-list
accepts `PathBuf`, `Duration`, `HashMap` and `serde_json::Value` at a flow
boundary, all of which README's reason for rule V rejects as surely as it
rejects `String`.

The allow-list needs the set of vocabulary type *names* — which a token pass
can compare against without resolving anything, so constraint 2 is not the
obstacle. The obstacle is that nothing holds that set: slice 17 chose not to
build a `TRACEABLES` registry, because its own check needed none.

So check 18 as README states it cannot be built until something enumerates the
nouns, and **this slice narrows rule V to the deny clause** — catching the
primitives README names and missing every other non-vocabulary type. That is a
narrowing of the specification, and under tenet 1 it is recorded as one: the
narrowing is named in the claim itself, so no reader of the claim mistakes it
for README's rule, and README owes either an amendment or a note that the deny
clause is rule V's ramp. Deferred 6.

## Behaviour

### The classification, and what the pins are for

A function is **flow** iff its body satisfies all six of README §3.7's rules,
quoted here because a reader must be able to derive this slice from this
document:

- **F1** every statement is `let pat = call(...)?;`, `call(...)?;`, or the tail call;
- **F2** at most one decision structure — one `match`, or one two-way `if` — and every arm is a single call;
- **F3** arguments are paths, field accesses, references, or accessor chains;
- **F4** no closures or blocks;
- **F5** no literals but `()`;
- **F6** no macros but those in `shape.allow_macros` (`todo!` and `unimplemented!` by default, so a skeleton is flow).

Otherwise it is a leaf. The pass answers, for every function of a crate, flow
or leaf and why — which F-number it failed first, so a report names the rule
rather than the verdict.

`#[flow]` and `#[leaf]` are **pins, not classifications**. `#[flow]` says this
function must stay routing and check 17 fails when it acquires work. `#[leaf]`
on a public function is rule B's escape: a public leaf is allowed and counted,
which is what "counted and reported" means — the mark exists to be visible, the
shape `#[lid(free)]` copied from it. **The pins are not this slice's** — see
"The pins are a slice of another crate" below, which is why check 17 is not in
this slice's row either.

### The rules this slice carries, and the two that are not its own

- **A (check 15)** — routing among `shape.dispatch_arms` or more kinds must be flow.
- **B (check 16)** — every `pub fn` in a slice's `mod.rs` is flow or carries `#[leaf]`.
- **V (check 18)** — a flow signature uses no primitive, wrappers unwrapped.
  **Narrowed to README's deny clause by this slice**, per the section above.

**P (check 17) is not in this slice.** It reads `#[flow]`, which lives in
`lid-rs-macros` and which the phase policy will not let this slice's phases
write. It lands with the pins.

README states a fifth rule, **C**: the complexity threshold is uniform,
because flow nodes hold one decision structure by F2. It has no check number
and is not in the HLD's row for this slice. It is not a check but the *reason*
the re-measurement below is meaningful, and it is recorded here so a reader
does not go looking for check 19 in the wrong place.

### What a signature carries, and whose function it is

The pass's second answer is one `Signature` per function: the file and the name
the two answers join on, the type tokens of every parameter in declaration
order, and the whole written return. Every function of the crate has one,
whether the classification called it flow or leaf, because rule V reads these
tokens for the flow nodes and a conformance check reads them for every
implementer of a claim.

A signature also carries **whose function it is**. For a function read from an
`impl` block, the owner is the tokens of the block's self type as the source
wrote them — the bare name for `impl Shape`, and the name with its generic
arguments where the block wrote them. For a function read from a `trait`
block, the owner is the trait's name. For a free function there is none. The
rendering is the one both type positions already use — the tokens through
`quote::ToTokens`, `to_token_stream().to_string()` — so an owner is spelled the
way `parameters` and `returns` are spelled, and a consumer comparing them
compares like with like.

Written tokens, not a resolved type. A block written `impl Report` names the
owner `Report` whether this crate declares that name or reached it through a
`use`, and a `Self` written in a parameter or in the return stays `Self`: what
the owner supplies is the thing `Self` stood for at that declaration, and what
to do with the pair is the consumer's. Nothing is looked up, which is what keeps the addition
inside the carve-out this document's Context section quotes — one item's
tokens, read and not resolved.

The owner is decided where the item kinds are already told apart, in
`functions_of`: the one function that knows whether the tokens it is walking
came from a free function, from an `impl` block or from a `trait` block. Those
three kinds are one decision and it stays in that one body; the helper that
assembles a function's pieces takes the owner as another piece, and
`signature_of` copies it across as it copies the file and the name. Nothing
else about the reading changes — the same functions are found, in the same
order, and a verdict is answered for each of them as before.

### The crate knows nothing about the workspace

Every fact about *this workspace* that a rule needs is **passed in by the
caller**, not read by the crate. That is not tidiness; it is forced. The
accessors that read `[workspace.metadata.lid_rs]` — `Project::setting_in`,
`Project::workspace_setting`, `Project::root_package_setting` — are all
**private** and live in `cargo-lid-rs` (`cargo-lid-rs/src/project.rs:232`,
`:238`, `:261`), and the HLD's graph runs `SHAPE --> CARGO`
(`lid-rs/docs/intent/hld.md:159`): `lid-rs-shape` cannot depend on
`cargo-lid-rs`, so there is no path by which it reads any metadata at all.

So `check` takes its configuration and its workspace knowledge as arguments:

- **`level`**, `shape.dispatch_arms`, `shape.allow_macros` and `shape.wrappers`
  — read from the metadata table by `cargo-lid-rs` and handed over.
- **Which files are slice `mod.rs` files** — rule B's input. "A slice's
  `mod.rs`" is a layout question, and `cargo_lid_rs::layout` is the one place
  that answers it. The pass can see that a file is *named* `mod.rs`; it cannot
  know whether it is a slice's. The caller passes the set.

This also settles a question the crate could not otherwise answer: with the
knobs injected, `lid-rs-shape` has no default to disagree with README about.

### What it runs as, and the dependency direction that decides it

**Stated as a manifest fact, because the obvious design is a cycle.**
`#[implements]` expands to `::lid_rs::IMPLEMENTATIONS`
(`lid-rs-macros/src/expand.rs:35`) and `derive(Spec)` to
`::lid_rs::claim::ClaimMeta`, so **any crate carrying citations depends on
`lid-rs`** — and this slice's phases put citations in `lid-rs-shape`. Meanwhile
`intent_graph!` is `macro_rules!` using `$crate::` throughout
(`lid-rs/src/graph/mod.rs:76-124`), so for its emitted test to call the pass,
`lid-rs` must reach `lid-rs-shape`. `lid-rs` → `lid-rs-shape` → `lid-rs` is a
package cycle: it fails at manifest resolution, before any phase's code exists.

**This slice therefore has one entry point, not two: `cargo lid-rs shape`.**
`cargo-lid-rs` already depends on `lid-rs`, and the HLD already draws
`SHAPE --> CARGO`, so `cargo-lid-rs` → `lid-rs-shape` → `lid-rs` is acyclic and
needs no new edge anywhere else. **What the dropped entry point costs, stated
exactly:** the pipeline's gate tables run `shape` as a command at phases 3, 4,
5 and 6 (`lid-rs-pipeline/docs/intent/pipeline.md:287-297`) — but **not at
phase 7**, whose row is line 298 and reads `["suite", "mutants", "regen",
"graph", "sync --check", "site"]`. `suite` is `cargo test --lib`, covering
checks "15-26" (`:224`), which is exactly the `intent_graph!()`-emitted test
form this slice declines to build (Deferred 7). So the one phase where the
citation would have to carry the argument is the phase where the dropped
mechanism *is* the mechanism. Neither composite is built, and neither README
§4.5's list nor `CLAUDE.md`'s gate runs `shape` today; adding it to both is
item 6 below.

The cost is honest and recorded: README §3.7 says the pass "runs as a test
(`intent_graph!()` emits it) **and** standalone as `cargo lid-rs shape`". This
slice builds the second and not the first. Under tenet 1 that is a narrowing
the README owes an amendment — Deferred 7 — and it is the reason
`intent_graph!()` is **not** in "What lands by hand".

`lid-rs-shape` is a new **publishable** member — forced, not chosen:
`cargo-lid-rs` depends on it and is published, and a published crate cannot
depend on an unpublished one. It therefore joins the gate's `cargo package`
invocation and the publish slice's set; that is item 6 below. Note the
catalog needs nothing: it derives `-p` from `publishing_members`
(`cargo-lid-rs/src/catalog/mod.rs:241-255`).

### The ramp, and what records it

`level` makes the first landing honest: this slice's three checks denying
against a tree of unclassified functions would block the slice's own gate.
The default when `shape.level` is absent — which is this workspace's state today, since the root
`Cargo.toml` holds only `mutation_scope` and no
`[workspace.metadata.lid_rs.shape]` table at all — is **`warn`**, and the
lookup falls back workspace → root package, as `Project::configured_scope` does
(`cargo-lid-rs/src/project.rs:102-111`).

**What records the counts in the interval: nothing committed, and that is a
real gap.** README makes a `warn` ramp legitimate on a condition — "`warn`
counts are committed in `trace.md` so the ramp is visible in the PR diff"
(README.md:566-571). `trace.md` does not exist anywhere in the tree and is
slice 21's (Deferred 3). In the interval the count is printed by the command
and written to `shape.json`, which is a build artifact and not committed, so
the ramp is **not** visible in the PR diff and tenet 2's "no development window
where the repo would fail its own methodology" is strained rather than
satisfied. This document does not paper over that: the choice of what closes it
is Open Question 1.

### The threshold is a measurement this slice makes and does not act on

README §7: "`cognitive-complexity-threshold = 4` is a starting point, not
scripture… the threshold the design implies is 1 — but this workspace's own
code raised six hundred warnings at 1 before the shape pass existed to say
which were flow nodes, so the number is a measurement to make, not a value to
assert."

The HLD's row for this slice ends "the complexity threshold **re-measured with
the classification in hand**" (`lid-rs/docs/intent/hld.md:197`). That
measurement is the **join of `cargo clippy` at threshold 1 against
`shape-classify.json`** — how many warnings at 1 fall on flow nodes versus
leaves. It is a report, not shipped code, so it produces no Shape row; it is
taken **after Phase 6**, because it needs the real classifier rather than the
throwaway one, and it is **recorded in this slice's Phase 7 commit message**.
Changing `clippy.toml` on that evidence is a separate act — Deferred 1 — because
a threshold change touches every crate and its own template, and because a
measurement that arrives with its conclusion already applied is not evidence.

### The measurement, and why its classification half is withdrawn

The circularity a reviewer named — the instrument is the thing the slice builds
— is broken by a throwaway `syn` pass written outside the repository, where no
phase policy reaches and nothing is committed but its numbers.

**The population must be stated, and the earlier one was wrong.** The
classification figures previously recorded here were taken over 91 parsed
files. The five members' `src/` trees hold **51** `.rs` files; the remainder
came from `lid-rs/tests/ui/**` and `xtask/fixtures/**` (56 `.rs` files between
them) — compile-failure fixtures and deliberately-bad gate self-test crates,
written to violate these very rules and never run over by any check. Those
figures are therefore **withdrawn**, not adjusted: the earlier selection cannot
be reconstructed from the number alone, so the classification must be re-taken
rather than scaled.

What is verified, over the five members' `src/` trees, 2026-09-11:

| | |
|---|---|
| `.rs` files under the members' `src/` | 51 |
| `pub fn` | 408 |
| **`pub fn` in a `mod.rs`** | **200** |

**What this says about rule B, which is the rule the ramp turns on.** Rule B
governs only `pub fn` in a slice's `mod.rs`, so **200 is the hard ceiling** on
rule-B violations, and not all 200 — only the leaves among them, and only those
in a *slice's* `mod.rs` rather than any. The earlier conclusion — "130 rule-B
violations… in the hundreds" — counted public leaves across the whole parsed
population against a rule that reaches a fifth of it, and is arithmetically
impossible. The honest statement is **tens, under a ceiling of 200**.

**Which way that points.** It does not reverse the ramp: any count above zero
blocks a deny-on-landing, because the pass gates at Phase 7 and this slice's
own tree is unclassified. But the ramp decision was previously asserted as
*settled by the measurement*, and the measurement that settled it was wrong.
The Decisions table kept the question open while the prose closed it; the table
was right. It stays open, and the classification is re-taken before Phase 2 —
now over a stated population.

### What the pass does when it cannot answer

One sentence each, because the absence of these cost two earlier slices a phase
stop:

- **A file `syn` cannot parse** — a `Finding` against the file, not a panic and
  not a skip; a source file this pass cannot read is a gap in the check's
  coverage and must be visible.
- **A crate with no `src/`** — no findings and no error; a member with nothing
  to classify is not a violation.
- **A `#[cfg]`-gated module** — classified as written, because the pass reads
  tokens and resolves nothing, including cfg; the gate is not evaluated.
- **A macro-generated function** — invisible, because it does not exist as
  tokens in the source; F6 governs the macro *invocation* in the enclosing
  body, which is the only thing the pass can see.
- **A `mod.rs` reached by `#[path]`** — followed as a file path when the
  attribute is a literal, and otherwise reported as unreachable; the pass walks
  files, and a `#[path]` it cannot read is coverage it must not claim.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| The pass is a new plain library, `lid-rs-shape` | A workspace member depending on `syn`, reading one crate's source | Put it in `cargo-lid-rs`; make it a proc-macro crate | README's layout and the pipeline both name the crate. It is the workspace's first non-proc-macro `syn` dependency, which tenet 3 admits — `syn` is named core. |
| One entry point: `cargo lid-rs shape`, not `intent_graph!()` | `cargo-lid-rs` → `lid-rs-shape` → `lid-rs`, acyclic | (a) `lid-rs` takes `lid-rs-shape` as a **dev**-dependency — cargo permits dev cycles, but `$crate::` cannot reach it, so `intent_graph!` must emit an absolute `::lid_rs_shape::` path, making it **every consumer's** dev-dependency and requiring a line in `cargo-lid-rs/templates/manifest_tables.toml`. (b) `lid-rs-shape` carries no citations, so `lid-rs` may depend on it normally — rejected: a slice with no citations fails checks 10 and 11 and is outside the methodology. | Any crate with citations depends on `lid-rs` (`expand.rs:35`), and `intent_graph!`'s emitted test would make `lid-rs` depend back: a package cycle that fails at manifest resolution. The chosen direction adds no edge to any consumer and needs no absolute-path emission. The checks still gate at every phase through the pipeline's command tables. The cost is a README §3.7 narrowing, Deferred 7. |
| Rule V is narrowed to README's **deny clause** | Primitive names and README's wrappers, as injected constants | Implement README's allow-list now; wait for slice 17 | README §3.6 states V as an **allow-list** (README.md:513-516), and an allow-list needs the set of vocabulary names, which nothing in the tree holds — slice 17 chose not to build a `TRACEABLES` registry. Constraint 2 forbids resolving a type token, so the deny clause is what a token pass can decide today. The narrowing is named in the claim and owed a README amendment (Deferred 6). |
| The crate reads no workspace metadata; the caller injects it | `check` takes the three knobs and the set of slice `mod.rs` paths as arguments | Read the metadata in `lid-rs-shape`; hard-code README's defaults | `setting_in`, `workspace_setting` and `root_package_setting` are private and live in `cargo-lid-rs` (`project.rs:232`, `:238`, `:261`), which `lid-rs-shape` cannot depend on (HLD `SHAPE --> CARGO`). Injection is the only direction available, and it also gives rule B the layout knowledge the pass cannot derive. |
| Check 17 (rule P) is **not** in this slice | Rules A, B and V here; P with the pins | Build all four here; defer B as well | `#[flow]` lives in `lid-rs-macros`, and `seat_of` refuses every path under it before any table is consulted: a companion exists only when the own crate is a proc-macro crate (`policy.rs:60-73`), and `lid-rs-shape` is a plain lib. No phase of this slice can write the pins, so a slice claiming check 17 cannot run its own phases. |
| The classification is derived, the marks only pin | F1–F6 from the body; `#[flow]`/`#[leaf]` as pins | Classify by attribute | README §3.7: a marker is "evaded by omitting it". A derived classification cannot be opted out of, which is the whole point of rule A. |
| F1–F6 are six claims, not one | One claim per rule, plus one for the verdict they compose | One claim "a function is flow iff F1–F6", table-driven | Each F-rule is a decision, and §0's rule makes every decision a claim. Check 14 binds one validator per claim, so one claim over six rules forces a single table-driven test whose failure names the table row rather than the rule; six claims give six validators that each name their rule, which is what a report needs. |
| The threshold is measured, not changed | This slice reports the warnings-at-1 join in its Phase 7 commit | Lower it to 1 here; leave it unmeasured | Six hundred warnings at 1 is what made the number a measurement. Applying a conclusion in the commit that produces the evidence leaves no step at which the evidence could have said otherwise. |
| The owner travels on the signature, rather than being worked out by the consumer | `signatures` records the block's written self type beside the parameter and return tokens, as one more written thing | (a) the consumer works out `Self` for itself — it is handed `Signature` values and no tokens, so it would have to read the crate's source a second time; (b) the consumer treats a written `Self` as a wildcard matching whatever a document claims — measured and dropped | The reading holds the tokens at the moment it knows the item kind, and carrying them on costs one field; a consumer may not resolve a name at all (constraint 2), so a `Self` it cannot expand is a comparison it cannot make, and a second reading of the same source would be a second implementation of the same pass. The wildcard was dropped because it goes silent exactly where a written return is most likely wrong: `-> Self` on `impl Report` and on `impl Finding` would both accept a document naming either, so a return that drifted from the block it was written in is the one case a wildcard cannot see. |
| **This crate has no `level`.** Every rule reports; nothing here gates | `check` returns `Finding`s and takes no level | Inject `level` into `check`; a third state for unregistered rules | This is the row that makes Phase 2 possible, and an earlier draft had it the other way. If a rule's claim reads "shall fail" under one measurement and "shall report" under another, Phase 2 cannot write the claim, Phase 5 cannot write its validator, and check 14's one-validator-per-claim binding makes the choice unrecoverable without a rename cascade — so **every claim of this slice is written level-agnostically, as `Finding` production**. That is what the row above already required: a crate that reads no workspace metadata and writes no files has no business knowing whether a finding is fatal. `shape.level` is applied by `cargo lid-rs shape`, in `cargo-lid-rs`, which owns the exit status. The measurement then decides only that caller's default, which is a metadata value and not a claim, and so can wait for the pass that measures it. |

## Open Questions & Future Decisions

1. **Settled: this slice has no `level`, so nothing about the ramp blocks
   it.** The question asked what records the `warn` counts until `trace.md`
   exists, and offered a counts file, a committed `shape.json`, or an
   uncounted interval. All three assume this crate knows about levels. It
   does not: `check` answers `Finding`s, and `cargo lid-rs shape` — in
   `cargo-lid-rs`, which owns exit status and reads the metadata — decides
   what a finding costs. See the Decisions row.

   A previous draft of this row proposed instead that a zero-count rule land
   at `deny` and a nonzero one be "not registered yet". That was wrong twice
   and is withdrawn. README has two levels, `warn` and `deny`
   (`README.md:1204`), so "not registered yet" was invented vocabulary; and
   the proposal contradicted the Decisions row it did not edit, which is this
   repository's most-recorded defect and was its sixth instance.

   **What is still owed to slice 21, stated so it is not discovered.**
   README fixes `trace.md`'s content as one row per claim "plus the shape and
   conformance `warn` counts — generated from the registry and the shape
   pass" (`README.md:1847-1849`). This slice ships no counts document, so
   slice 21's LLD must decide what a shape count section looks like, and it
   is the only document that can. The 21→18 edge is discharged by this
   slice's Phase 7 — check 26 needs the **pass**, which this slice
   delivers — but the 18→21 direction is **renamed, not removed**: a ramp
   still wants committed counts, and the ramp now lives wholly on the caller
   side, in `cargo-lid-rs` and `trace.md`. Neither is this slice's.

2. **Settled: the slice is `lid-rs-shape`, on branch `lld/lid-rs-shape`.**
   Not a preference — the layout slice's rule determines it, and this
   document already stated the derivation while still filing it as open. A
   crate-root slice's name **is its package name**
   (`cargo-lid-rs/src/layout/lld.md:316-323`), which makes this slice
   `lid-rs-shape` on branch `lld/lid-rs-shape`. The catalog's
   `Unbuilt::Slice("slice 18")` is a message and unaffected; the pipeline's
   `shape` is the **command** name and also unaffected. Slice 23 took the
   same rule for `lid-rs-pipeline` and its phases ran on it, so this is
   applied precedent rather than a first reading. The phase agents must be
   invoked with this name; **Open Question 1 is the only one left blocking
   Phase 2.**

### Deferred

1. Changing `cognitive-complexity-threshold`, and
   `cargo-lid-rs/templates/clippy.toml` with it, on this slice's
   warnings-at-1 measurement.
2. Nothing about `[workspace.metadata.lid_rs.shape]` is deferred by this
   slice: all four keys are injected by the caller, and reading them is
   `cargo-lid-rs`'s work under "What lands by hand". What remains deferred is
   **adding the table to this workspace's root `Cargo.toml`**, which holds only
   `mutation_scope` today and which no phase may write.
3. `docs/intent/trace.md`, which README §3.7 says carries the `warn` counts and
   check 26 gates. The document does not exist anywhere in the tree; it is
   slice 21's. See Open Question 1.
4. Classifying test bodies. The pipeline's `shape-classify.json` says "every fn
   → flow/leaf" and §4.6 exempts nothing, but a large share of this
   workspace's functions are in `#[cfg(test)]` modules and no rule of §3.7 was
   written with a test body in mind. The exact share is part of the
   re-measurement, and is deliberately not quoted here until it is re-taken.
5. `cargo-lid-rs/templates/AGENTS.md:63` states constraint 2's prohibition
   without its carve-out, and `cargo lid-rs init` writes that file into every
   scaffolded downstream project. A documentation commit after Phase 7.
   (`CLAUDE.md:93-98` already carries the carve-out and names this slice.)
6. README §3.6 owes either an amendment or a note that the deny clause is rule
   V's ramp, once something enumerates the vocabulary names.
7. README §3.7 says the pass also runs as a test emitted by `intent_graph!()`.
   This slice builds only `cargo lid-rs shape`, for the cycle reason above.
   README owes an amendment, or a note that the test form waits on a
   dev-dependency decision.

### To measure at Phase 7, with the pass this slice builds

The classification over this workspace, and the count of violations per rule.
It decides the **caller's** default `shape.level` — a value in
`[workspace.metadata.lid_rs.shape]`, not a claim — so it does not gate Phase
2, and it is taken with the pass rather than guessed ahead of it. Draft 1's
figure was withdrawn because its population could not be reconstructed, and a
grep cannot replace a `syn` pass: that is what produced a number wrong by
about twice.

**State the population as a command, not as a count.** The replacement figure
in this document says "the five members' `src/`"; globbing `*/src/**/*.rs`
today gives 52 files, not the 51 stated, because this slice's own crate now
has one. A bare number is what made the first measurement unreconstructable,
so Phase 7 records the command it ran beside the result.

Nothing else in this document depends on a fact not already established here
or verified against the source.

## Shape

| Item | Role |
|---|---|
| `lid_rs_shape` | The crate: a `syn` pass over one crate's source, resolving nothing, reading no workspace metadata. |
| `lid_rs_shape::classify` | Every function of a crate to flow or leaf, with the F-number a leaf failed first, and the arm count and kind that rule A reads. |
| `lid_rs_shape::Shape` | One function's verdict: its name, whether it is flow, why not, and its dispatch arity. |
| `lid_rs_shape::check` | Rules A, B and V over a classification, with the knobs and the slice `mod.rs` set as arguments, answering every violation as a `Finding`. **No `level`:** whether a finding is fatal is the caller's, and this crate returns data. Rule P is not here — it lands with the pins. |
| `lid_rs_shape::Finding` | One rule violation: the rule, the function, and what it saw. |
| `lid_rs_shape::signatures` | **Every** function's parameter and **return** type tokens — not only flow nodes and not only the `Ok` type — and the owner of each. Rule V reads the type tokens and slice 19 consumes them; the owner is what lets a consumer read a written `Self`. |
| `lid_rs_shape::Signature` | One function's written signature: the `file` and `function` the two answers join on, `parameters` in declaration order, `returns` whole, and `owner` — the `impl` block's self type as written, the `trait`'s name, or none for a free function. Every field is tokens the source wrote, so a `Self` in either type position stays `Self` and the owner is what explains it. |
| `lid_rs_shape::function::functions_of` | The reading's one dispatch over item kinds — a free function, the methods of an `impl` block, the bodied methods of a `trait`, the functions of an inline module — and so the one body that decides a function's owner, because it is the only place that knows which kind the tokens came from. Three kinds are one decision and it is made here, once. |
| `lid_rs_shape::Classification` | The serialisable whole: every `Shape`, which `cargo-lid-rs` writes as `shape-classify.json`. |

`signatures`' breadth is README's, not a choice: §4.7 says it "returns **each
function's** parameter and **return** type tokens" (README.md:870-872), and
"**Every implementer** of a claim is checked, not just the entry point"
(README.md:846-852). Narrowing it to flow nodes and `Ok` types would break
slice 19, its only named consumer (`hld.md:214`).

### The two artifacts, and who writes them

Pipeline §5.1 (`pipeline.md:220`) has the `shape` command writing `shape.json`
and `shape-classify.json`, the latter consumed by `mutants` (`:225`), `regen`
(`:226`) and the site. **`lid-rs-shape` returns the data and writes no files**
— it knows no workspace paths. `cargo-lid-rs` serialises them, as it already
does for every other command's report:

- **`shape-classify.json`** — `Classification`: every function to flow/leaf
  with its first failed F-rule and its dispatch arity.
- **`shape.json`** — the catalog's ordinary findings report over `Finding`,
  plus the per-rule `warn` counts Open Question 1 is about.

### The cascade a field added to `Signature` makes

`Signature` has public fields and is not `#[non_exhaustive]`, so a field added
to it breaks every struct literal that constructs one. Inside this crate that
is two — `signature_of`, which is the item that fills the field, and the
fixture helper that spells a signature for a test. Outside it there are none:
`Signature {` matches in no other member's source, and no member's
`Cargo.toml` names `lid-rs-shape` as a dependency at all, the root manifest's
`[workspace.dependencies]` entry standing ahead of the first consumer rather
than behind one.

So nothing in this workspace fails on the addition, and that is the reason to
record it rather than the reason not to. `lid-rs-shape` is a published crate,
and a consumer outside this repository that built a `Signature` by struct
literal is broken by the new field without any gate step here being able to
see it. The version the Phase 7 gate writes is one patch level above the
workspace version, for this change as for every other: this workspace has a
single consumer of its own crates, and takes a breaking change at a patch bump
rather than holding it back for a release that would serve nobody.

## The pins are a slice of another crate

`#[flow]` and `#[leaf]` live in `lid-rs-macros`. They are **not** items of this
slice and carry no row in the Shape table above, because no phase of this slice
can write them: `seat_of` refuses every path under `lid-rs-macros` before any
table is consulted, a companion existing only when the own crate is a
proc-macro crate (`cargo-lid-rs/src/phase/policy.rs:60-73`), and
`lid-rs-shape` is a plain lib.

They are therefore a slice of `lid-rs-macros` with `lid-rs` as companion,
exactly as `claim` is — with its own `lld.md` at
`lid-rs-macros/src/<slice>/lld.md`, its own Phase 1, its own claims, and its
own human-committed `compile-time-accepted`. **Check 17 lands with it.** That
slice owes three answers this one does not: where `#[flow]`/`#[leaf]` are
re-exported from so user code can `use` them; whether they are accepted on
`impl` methods, trait methods and closures; and what they expand to, which is
their input unchanged.

## What lands by hand

Seven things no phase of this slice may write, stated as a plan:

1. **The crate's membership.** `lid-rs-shape` added to the root `Cargo.toml`'s
   `members`, and to `[workspace.dependencies]`; plus
   `lid-rs-shape/Cargo.toml` itself. `policy::allowed_paths` admits no
   `Cargo.toml` in any phase. A hand commit before Phase 3.
2. **The metadata table, in two parts, at two moments.**
   `[workspace.metadata.lid_rs.shape]` in the root `Cargo.toml`, which holds
   only `mutation_scope` today. Its keys do not all land together, and an
   earlier draft asked for them in item 1's commit while Deferred 2 called the
   whole table deferred — they now agree here. `allow_macros`,
   `dispatch_arms` and `wrappers` are the knobs `check` is handed and must
   exist **before Phase 6** can run the command against this workspace.
   `level` is the caller's and is written **after Phase 7**, from the
   measurement, because it is the one key this slice's own pass supplies the
   evidence for.

2b. **`pub mod spec;` in `lid-rs-shape/src/lib.rs`, immediately after Phase
   2 commits and before Phase 3 starts.** Not cosmetic: a claims file that no
   module declares is compiled by nothing, so **check 13 does not run on
   Phase 2's claims at all** and its gate passes over them vacuously. Slice 23
   paid this — Phase 3's declaration rejected six of seventeen claims at once,
   four for an undefined verb and five for triggers naming no Rust item — and
   recovering cost a re-run of Phase 2 with Phase 3's skeleton stashed around
   it, because Phase 2 may not touch `lib.rs` and its hook refuses to commit
   while a file outside its paths is dirty. Landing the one line by hand
   between the two phases makes the rejections surface while Phase 2 can still
   be re-run cleanly. It cannot land earlier: `pub mod spec;` before
   `spec.rs` exists fails `cargo check`.
3. **Discharged — the crate-root write policy is fixed.** An earlier draft
   asked for a change branch `lld/phase--crate-root-directory` because
   `policy::module_dir` hard-coded `src/<slice_snake>` and would forbid this
   slice's Phases 5 and 7. That defect was fixed at `cd76407` and the
   installed binary carries it: `SliceCode::of_own_crate` routes through
   `layout::slice_dir` (`cargo-lid-rs/src/phase/policy.rs:265`), `module_file`
   answers `None` when the directory is the crate's `src` (`:312-314`), and
   `module_dir` — now at `:497-499` — is the companion seat only, its own doc
   saying it is "the only place the policy still spells one from the slice's
   name, because a companion's is a module's". Crate-root coverage for Phases
   5 and 7 exists at `:1134-1156`. Kept as a numbered item rather than
   deleted, so that the branch it asks for is not opened by someone reading an
   older copy.

4. **The `shape` command.** Not "one arm of `cargo-lid-rs`'s dispatch":
   commands live in the **catalog** slice's table
   (`cargo-lid-rs/src/catalog/mod.rs:213` holds
   `("shape", Unbuilt::Slice("slice 18"))`). A change branch on that slice,
   `lld/catalog--shape`, owing: a **new `Invocation` variant** for an
   in-process pass that spawns nothing — the enum has `JsonDiagnostics`,
   `StderrDiagnostics`, `SyncComparison` and `Unbuilt`
   (`catalog/mod.rs:100-124`) — removal of the `UNBUILT` row and the test
   asserting it (`catalog/mod.rs:985`), a mapping from `lid_rs_shape::Finding`
   to the catalog's `Finding` schema, the two artifact writes above, and the
   reading of `[workspace.metadata.lid_rs.shape]` this slice's crate cannot do.
5. **Rows for checks 15–18 in `lid-rs/skill/references/gates.md`**, which stops
   at check 12. The catalog reads that table for every finding's `fix`, and the
   catalog slice already recorded what a missing row costs: "every finding's
   `fix` would have been empty, and a validation asserting that emptiness would
   have been green over nothing" (`catalog/lld.md:179-186`). It is the `skill`
   slice's file, so a change branch there, before the command lands.

Publication is not a hand commit but is an obligation of the same set:
`lid-rs-shape` is a new publishable member and joins both the gate's single
`cargo package` invocation and the publish slice's crate set.


6. **The three places `shape` must be named as a gate step, and the two that
   already name the package.** README §4.5's check list and `CLAUDE.md`'s gate
   block must gain `cargo lid-rs shape`, since the dropped `intent_graph!()`
   entry point means nothing else runs it at gate phase 7. The packaging half
   is already done — `CLAUDE.md`'s and `.github/workflows/gate.yml`'s
   `cargo package` lines both name `-p lid-rs-shape` as of the membership
   commit — and is recorded here because CI and the local gate silently
   disagreed for the length of one commit, which is the drift this item
   exists to prevent.

7. **The pipeline's check assignment.** `pipeline.md:220` gives the `shape`
   command checks **15, 16, 17 and 18**; this slice excludes 17, which lands
   with the pins on a `lid-rs-macros` slice. That document is owed the
   amendment, and no Deferred entry carries it — Deferred 7 covers only README
   §3.7's test form.

## References

- README [§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html) — F1–F6,
  rules A/B/P/V/C, the `warn` ramp's `trace.md` condition, and why the
  classification is derived rather than marked.
- README [§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html) — rule V
  as an allow-list, and the two reasons for it.
- README [§2](https://bradvoth.github.io/lid-rs/spec/constraints.html)
  constraint 2 — the carve-out this slice lives in.
- README [§4.7](https://bradvoth.github.io/lid-rs/spec/conformance.html) —
  what `signatures()` returns, and S1's scope over every implementer.
- README [§7](https://bradvoth.github.io/lid-rs/spec/configuration.html) — the
  shape table's four keys, the ramp, and the threshold as a measurement.
- `lid-rs-pipeline/docs/intent/pipeline.md` §5.1 — the `shape` command's row,
  its two artifacts, and the phase gate tables that run it.
