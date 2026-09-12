# A noun is a type the vocabulary declared; a primitive only records

## Context and Design Philosophy

HLD row 17 is "A slice's nouns are types the LLD links to", and it names four
products in one row (`lid-rs/docs/intent/hld.md:196`):

> `derive(Traceable)` with its policy attributes; `vocab` modules; LLDs
> rewritten with vocabulary links (check 2 enforces them)

**The row is split, and this slice is the half that is ordinary Rust.** `vocab`
is a module slice in `lid-rs` at `lid-rs/src/vocab/`, delivering the two
traits, the seal between them, README §6.5's primitive impls and the crate-root
re-exports — and nothing that runs at compile time. `derive(Traceable)` and the
assertion `derive(Spec)` emits are the other half, `noun-assertion`, at
`lid-rs-macros/src/noun_assertion/`.

**The split is not presentational, and the rename in it is load-bearing.**
`layout::crate_holding` finds a slice's crate with `.find()` over
`Project::member_manifest_dirs()` and returns the *first* member holding a
module of that name (`cargo-lid-rs/src/layout/mod.rs:443-449`,
`cargo-lid-rs/src/project.rs:141-147`). The members list opens `lid-rs`,
`lid-rs-macros` (`Cargo.toml:3`), and a directory counts as a slice's the
moment it holds an `lld.md` (`layout/mod.rs:605-612`). So if both
`lid-rs/src/vocab/` and `lid-rs-macros/src/vocab/` held a document, every
door — the phase policy's crates, the acceptance gate, the claims file, the
red run's diff — would resolve `vocab` to `lid-rs` **silently**, with no
ambiguity error anywhere, and no phase of the macros half could ever write a
line. The macros half is therefore `noun_assertion`, and
`lid-rs-macros/src/vocab/lld.md` is removed in the same commit that adds this
file. Until it is, `crate_holding("vocab")` already answers `lid-rs` and the
old directory is unreachable by name.

**What that leaves is a slice with no proc-macro in it, and that is the whole
of why it is worth splitting.** `lid-rs` is an ordinary library: its execution
class is `Ordinary` (`cargo-lid-rs/src/phase/policy.rs:753-760`), so
`acceptance_gate` returns no reason and no `compile-time-accepted` file is
needed (`cargo-lid-rs/src/phase/mod.rs:334-340`). Its items are types and
functions that a `#[cfg(test)]` unit test in the same crate can name. Every
structural objection recorded against the undivided draft — that its Phase 5
could not exist, that nothing could unit-test a proc-macro crate's function,
that its claims had no reachable red — was an objection to the macros half.
This half is buildable today.

## Facts, verified 2026-09-12

Every row was read from the tree on this branch (`lld/vocab`, stacked on
`lld/claim` at `9699d21`) or produced by running the command named. Spikes were
run with `rustc 1.98.0` against `--edition 2024`.

| Fact | Where |
|---|---|
| Before this document, `lid-rs/src/vocab/` did not exist; `lid-rs-macros/src/vocab/` holds an `lld.md` and nothing else, and no `noun_assertion` directory exists yet | `ls`, workspace-wide |
| No identifier `Traceable` appears in any `.rs` file in the workspace | `grep -rn Traceable lid-rs lid-rs-macros cargo-lid-rs lid-rs-shape` |
| Rule V's primitive list, in full: "Not `String`, `&str`, `bool`, integers, floats, `char`" | `README.md:513-516` |
| §6.5: primitives have `Traceable` impls "with none" — no policy — and that is why rule V exists | `README.md:1136-1145` |
| §3.5's refusal list includes "a trigger or response slot that is not an intra-doc link to a vocabulary type" | `README.md:417-423` |
| Check 13's row names "unlinked noun" among what `derive(Spec)` refuses | `README.md:599` |
| §11.1's tree puts `vocab.rs` — *a slice's own nouns* — beside `lld.md` and `spec.rs` in the slice directory | `README.md:1766` |
| HLD row 17, undelivered, naming the derive, the `vocab` modules and the LLD rewrite in one row | `lid-rs/docs/intent/hld.md:196` |
| `lid-rs` depends on `lid-rs-macros` and `linkme`, with `trybuild` as its only dev-dependency | `lid-rs/Cargo.toml:14-19` |
| `#[doc(hidden)] pub mod __private` already exists in the crate root and holds the `linkme` re-export | `lid-rs/src/lib.rs:245-248` |
| `missing_docs` is `deny` and `missing_docs_in_private_items` is `warn`, workspace-wide | `Cargo.toml:26-36` |
| Phase 2 may write the slice's claims file and `src/spec/mod.rs`, and nothing else | `cargo-lid-rs/src/phase/policy.rs:417-421`, `:455-461` |
| Phases 3 and 4 may write the slice's module directory **and `src/lib.rs`**; Phases 5 and 7 may write the module directory alone | `cargo-lid-rs/src/phase/policy.rs:417-424` |
| `tests/ui` is in the **companion** table's Phase 5/7 row only — a module slice in its own crate cannot write a fixture | `cargo-lid-rs/src/phase/policy.rs:434-441` |
| Phase 7's gate: check, clippy, doc, doc-tests, **lib tests**, package, `sync --check`, mutants — in that order, stopping at the first failure | `cargo-lid-rs/src/phase/mod.rs:578-601`, `:611-616` |
| `red_verdict` fails on any claim with no `#[validates]` test **and on any validator that passes** | `cargo-lid-rs/src/phase/mod.rs:912-919` |
| The red set is every claim on a fresh slice, and after a `phase 7:` commit only the claims added since it | `cargo-lid-rs/src/phase/mod.rs:796-803`, `:808-812` |
| A validator is run as `cargo test -p <package> --lib -- --exact <test>`; a package that fails to build is therefore "not passing", i.e. red | `cargo-lid-rs/src/phase/mod.rs:892-899` |
| `lid-rs/tests/ui/fail/not_a_spec.stderr` pins rustc's alphabetically-truncated eight-name list of `Spec` implementors; its eighth is `ACrateRootClaimsSliceIsEmpty` | `lid-rs/tests/ui/fail/not_a_spec.stderr:12-20` |
| `lld-check` requires a `## Decisions & Alternatives` table with four filled cells per row, `## Shape` rows naming a backticked identifier and a role, and numbered `### Deferred` items | `cargo-lid-rs/src/lld_review/mod.rs:17-23`, `:288-360` |
| Claims registered in `lid-rs` today: 117, of which 19 carry `#[lid(free)]` | the census below |
| Claims registered workspace-wide: 471, of which 294 are free — 177 held | the census below |

**The census.** Counting `#[derive(Spec)] pub struct X;` items per `spec.rs`
across the workspace and reading `#[lid(free)]` off each: `cargo-lid-rs` 312/274
free, `lid-rs` 117/19, `lid-rs-shape` 24/0, `lid-rs-pipeline` 17/0, `xtask` 1/1.
This differs from the `grep -c "derive(Spec)"` figures the trace document
records (`lid-rs/src/trace/lld.md`, Facts) because that grep also counts the
word where it appears outside a claim item; the item count is the one the
burn-down is about, and the `noun-assertion` document uses it.

**The spikes, run today, that decide the design.** They are recorded here
because the undivided draft asserted most of these results without a date, and
slices 19 and 21 have landed since.

| Spiked | Result |
|---|---|
| `const NOUN: &'static str = todo!();` in a trait impl | `cargo check` **passes** (`--emit=metadata` is clean); the *use* site is clean too under `--emit=metadata`; a real build fails with `error[E0080]: evaluation panicked: not yet implemented … evaluation of <u8 as Tr>::N failed here` |
| `assert_noun::<T>()` for `T` failing `T: Noun`, with `#[diagnostic::on_unimplemented]` on `Noun` | `error[E0277]: \`String\` is not a vocabulary noun: …` — the shaped message, one error, for both a `Traceable` non-noun (`String`) and a type implementing neither (`Plain`, `u32`) |
| A hand-written `impl Noun for Forged` with no seal impl | `error[E0277]: the trait bound \`Forged: lid_rs::vocab::Declared\` is not satisfied` — the seal holds, and **its own name appears in the diagnostic**, spelled by its full path rather than as the fixture writes it |
| A hand-written `impl Declared for Deliberate` followed by `impl Noun for Deliberate` | **compiles.** The seal is public, so it is a second thing to write, not an impossible one |
| A noun named through a `pub use` re-export, and through its declaration path | both compile — the same type, so the bound holds and there is nothing to compare |
| `assert_noun::<a::E::V>` / `::<a::f>` / `::<a>` — a variant, a function, a module | `error[E0573]: expected type, found variant` / `found function` / `found module`. **Not** the shaped message |
| `assert_noun::<>` — the empty target | `error[E0283]: type annotations needed` |
| `assert_noun::<a::Gen>` — a generic named without arguments | `error[E0107]: missing generics for struct \`Gen\`` |

The last three rows belong to the `noun-assertion` half's document and are
recorded here because the spike that produced them also produced the first
four, which are this half's.

## Behaviour

### `Traceable` is not the predicate, and that is the whole design problem

The obvious mechanism — bound a claim's target on `Traceable` — is wrong, and
README says why at §6.5 and §3.6 (`README.md:519-522`):

> `Traceable` has impls for primitives, so a `String` parameter *compiles and
> gets recorded* — as text with no policy. Rule V is what guarantees every
> field on a flow span is a noun whose policy the glossary chose.

`Traceable` is the **recording** trait. A bound on it accepts `String`, `u32`,
`bool`. An earlier draft of this design proposed exactly that bound, and would
have shipped a check that passes for every primitive — the rule stated, the
rule not enforced, which is this workspace's most repeated defect.

So the identity of a noun needs a second trait that primitives do not get. That
is `Noun`, a trait README does not name; under tenet 1 that is a doc bug and
README owes an amendment (Deferred 4).

### The three items, and what the seal is worth — stated once

Three types, in a fixed relation:

- **`lid_rs::vocab::Traceable`** — the recording trait, carrying `const NOUN:
  &'static str`, the type's own name. README §6.5's recording surface is what a
  *span* reads, and there are no spans until slice 20. Adding a method to a
  published trait is a breaking change, and `cargo package` is the only gate
  step that says so; that cost is slice 20's, stated here so it meets it as a
  known one rather than a surprise.
- **`lid_rs::vocab::Declared`** — the seal. `#[doc(hidden)]`, and re-exported
  through the existing `#[doc(hidden)] pub mod __private`
  (`lid-rs/src/lib.rs:245-248`) so that a derive expanding in a consumer's
  crate can emit `impl ::lid_rs::__private::Declared for TheirType`. It must be
  *public*: a `pub(crate)` seal would make `derive(Traceable)` work only inside
  `lid-rs`.
- **`lid_rs::vocab::Noun: Traceable + Declared`** — what a claim may name.

**What the seal guarantees, in one sentence that this document does not then
contradict: the seal makes a noun a *deliberate* declaration, not an
unforgeable one.** The spike above shows both halves of that. `impl Noun for
Forged` without the seal is `E0277: Forged: Declared is not satisfied`, so
nobody arrives at `Noun` by accident; `impl Declared for Deliberate` followed
by `impl Noun for Deliberate` **compiles**, so a determined consumer can
declare a noun without the derive. The seal is `#[doc(hidden)]` — unlisted,
unsearchable, outside the documented surface — which is what makes the second
of those a decision rather than a slip.

The undivided draft said this three times in two different strengths, once
claiming that "nothing outside the derive can be a noun" and once disproving
it. There is one statement now and the Decisions table carries the same one.

**What a consumer cannot do**, and this is the real cost: make a *foreign* type
a noun. `uuid::Uuid` and `chrono::DateTime` cannot carry this crate's derive
(orphan rule), so every claim naming one must link a newtype. That is
consistent with README §3.6 — a `Username`, not a `String` — and it is stated
once here rather than discovered.

**`Traceable`'s primitive impls** are rule V's list exactly (`README.md:515`):
`String`, `&str`, `bool`, `u8 u16 u32 u64 u128 usize`, `i8 i16 i32 i64 i128
isize`, `f32 f64`, `char`. None of them gets `Declared`, and therefore none
gets `Noun`. That is the assertion the whole design turns on and it is one
`grep` away from being checkable by eye.

### Why there is a function in a slice made of traits

Every item above is a type or a `const`. **A claim implemented by data or by a
`const` has no reachable red**: it is correct from the Phase 3 skeleton onward,
its test is green before any leaf exists, and `red_verdict` refuses a passing
validator (`cargo-lid-rs/src/phase/mod.rs:912-919`). The trace slice met this
and answered it by giving `DOCUMENT` a function to be read through
(`lid-rs/src/trace/mod.rs`, `document_path`); the same answer holds here and is
needed harder, because *everything* in this slice is data-shaped.

The obvious escape — skeleton the impls with `const NOUN: &'static str =
todo!();` so the leaves are real work at Phase 6 — is **refused, and the spike
above is why**. `cargo check --all-targets` passes over it, so Phase 3's check
is green; the failure arrives at the first real build as `error[E0080]:
evaluation panicked`, which is a *compile* error, not a runtime panic. That
would take out the whole `lid-rs` test binary — every other slice's tests
included — from Phase 3 until Phase 6, and `red_verdict` would read the
resulting build failure as this slice's tests being red for the right reason
when they are red for no reason at all. So the impls carry their real values
from Phase 3, uncited, exactly as `DOCUMENT` does, and the claims about them
are cited on a function.

That function is **`noun_of<T: Traceable + ?Sized>() -> &'static str`**: the
recorded name of a type, with no value in hand. It is thin, and it is not
invented for the phase machinery alone — it is the surface slice 20's spans
read a noun's name through, and it is where the bound `T: Traceable` is stated
once instead of at each span. It is `todo!()` at Phase 3, which makes every
test of every primitive impl red for a reason a reader can point at.

### Where the red is, and where it is not

Three kinds of claim, and each has an honest red:

- **What a primitive records.** `noun_of::<String>()` is `"String"`. Red
  because `noun_of` is `todo!()`; green when its leaf lands. The impls are the
  implementers; the function is what the test reaches them through.
- **What a primitive is not.** No primitive is a `Noun`. This is a *negative*
  type-level fact and no runtime test can state it: there is no way to ask "is
  `u32: Noun` false" from a passing program. It is pinned by a `trybuild`
  `compile_fail` fixture that writes the bound by hand and asserts the shaped
  `E0277` message. Red at Phase 5 because a `fail` fixture with no `.stderr`
  makes trybuild write a `.wip` and report failure.
- **That the seal is a second thing to write.** A test-local type in this
  slice's `#[cfg(test)]` module implements `Traceable` and `Declared` by hand
  and is then a `Noun`; a sibling that implements only `Traceable` is refused
  by a second `compile_fail` fixture. The positive half needs no derive, which
  is what makes this slice's Phase 5 independent of the other half.

Writing the bound by hand in a fixture pins **the language and this slice's
trait**, which is exactly what this slice owns. It would be the wrong test for
the *emission*, which is the other half's and is validated by fixtures that go
through `derive(Spec)`.

### The one thing this slice's phases cannot write

`lid-rs/tests/ui/` is in the **companion** table's Phase 5/7 row and not in the
own table's (`policy.rs:417-424` against `:434-441`). `vocab` is a module slice
in its own crate, so it takes the own table, and **no phase of this branch may
create a fixture file.** The trybuild *harness* is a `#[cfg(test)]` unit test
in `lid-rs/src/vocab/mod.rs` and is Phase 5's; the fixtures under
`lid-rs/tests/ui/vocab/` are a hand commit.

They must land **before** Phase 5, not after. A harness whose glob matches
nothing does not fail — trybuild calls `message::no_tests_enabled()` and
records zero failures — so a harness written at Phase 5 against absent fixtures
is a *vacuous green*, and `red_verdict` refuses it. Slice 19 paid an amendment
for discovering this at Phase 5 (`lid-rs/src/outcome/lld.md:498-508`); it is
item 2 of "What lands by hand" below.

### What this slice is not

- **Not `derive(Traceable)`, and not the assertion.** Both are
  `noun-assertion`, at `lid-rs-macros/src/noun_assertion/lld.md`. Nothing here
  parses a target, normalises a path, or emits a token.
- **Not the burn-down.** The 177 held claims whose triggers and objects are
  functions, modules and disambiguated paths are broken by the *assertion*, not
  by these traits, and the plan for them is in the other document.
- **Not the policy attributes.** `#[trace(redact)]`, `#[trace(skip)]` and
  §6.5's field selection say what a **span** records; slice 20 builds spans.
  Deferred 1.
- **Not the seventeen LLDs' prose links.** No LLD links a type today. A rewrite
  from nothing, landing per slice as each is next touched. Deferred 2.
- **Not a rule that an LLD must link its nouns.** Check 2 is
  `rustdoc::broken_intra_doc_links` (`README.md:591`) and fires only on a link
  that is *written* and fails to resolve, so a document with no links passes
  it. Enforcing the positive needs `lld-check` to read the document, which is
  slice 13's. Deferred 3.
- **Not a `TRACEABLES` registry.** A trait bound needs no registry, and the
  fourth and fifth distributed slices are already spoken for (`OUTCOMES`,
  `CLAIMED_OWNERS`, `lid-rs/src/lib.rs:33-36`).

### This slice's own claims name no noun, and that is legal today

Its subjects are two traits, a seal and a function. A trait is not a type, so
none of them can satisfy `T: Noun`; this slice is in exactly the position it
puts every other slice in. It is not a contradiction only because **the
assertion is not emitted until the other half's wiring lands** — until then
`derive(Spec)` requires a link that *resolves*, not a link that is a noun.
These claims are therefore in the burn-down like everyone else's, and the
`noun-assertion` document counts them there.

The alternative — inventing nouns in `lid_rs::vocab` for this slice's own
claims to link before anything requires it — was rejected: it would make this
slice define types whose only purpose is to be linked, before the rule that
would need them exists.

### Terms

**A noun** is a type implementing `lid_rs::vocab::Noun`, which in practice
means a type carrying `derive(Traceable)`. **This slice** is called `vocab` and
is a directory, `lid-rs/src/vocab/`. README §11.1's per-slice
`src/<slice>/vocab.rs` — the file a slice keeps *its own* nouns in
(`README.md:1766`) — is the **vocabulary file**, and no slice in this workspace
has one; which slices gain one lands with Deferred 2. The two spellings
collide, and the collision is inherited from README rather than introduced
here; where this document means the file it says "vocabulary file".

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| HLD row 17 is split, and this half is a module slice in `lid-rs` | `lid-rs/src/vocab/`: the traits, the seal, the primitive impls, the re-exports | Keep one `vocab` slice spanning both crates; put the traits in `lid-rs-macros`; make `vocab` a crate-root slice of a new crate | One slice spanning both crates is what the undivided draft was, and its Phase 5 could not exist: a proc-macro crate exports only macros, has no dev-dependencies (`lid-rs-macros/Cargo.toml:17-20`) and holds zero `#[cfg(test)]` items, so nothing could name the function under test. Splitting makes this half `Ordinary` (`policy.rs:753-760`), which needs no acceptance file and can be built today. The traits cannot live in `lid-rs-macros` because a proc-macro crate's types are unreachable from consumers. A new crate would add a publishable member for three items. |
| The macros half is renamed `noun_assertion` | `lid-rs-macros/src/noun_assertion/`, with `lid-rs-macros/src/vocab/lld.md` removed in the same commit | Leave both directories named `vocab`; name this half `nouns` and leave the macros half `vocab` | `crate_holding` takes the **first** member holding a module of the name (`layout/mod.rs:443-449`) over a members list that opens with `lid-rs` (`Cargo.toml:3`), and a directory is a slice's as soon as it holds `lld.md` (`layout/mod.rs:605-612`). Two `vocab` directories would silently give both slices `lid-rs` as their crate, with no ambiguity error and no phase of the macros half able to write anything. Renaming this half instead would put the collision on the half that is already blocked and leave the reachable one mis-named. |
| The seal is called `Declared` | `lid_rs::vocab::Declared`, `#[doc(hidden)]`, re-exported at `lid_rs::__private::Declared` | `Sealed`; a private module `sealed`; no seal, with `Noun` a plain sub-trait of `Traceable` | The undivided draft named it `Declared` in its prose and `Sealed` in the *chosen* cell of its own decisions table, and Phase 3 reads the table as the decision. `Declared` is chosen and the table's `Sealed` was the wrong cell. The name is user-visible in one place — the spike gives `E0277: the trait bound \`Forged: Declared\` is not satisfied` — and "declared" says what the author did, where "sealed" says what the library did. No seal at all makes `Noun` reachable by writing one impl, which is the accident the trait exists to prevent. |
| The seal is public, and this document claims deliberateness rather than impossibility | One statement: the seal makes a noun a deliberate declaration, not an unforgeable one | Claim the seal is unforgeable; make the seal `pub(crate)`; a sealed-trait pattern with a private witness type in the method signature | `pub(crate)` makes `derive(Traceable)` work only inside `lid-rs`, which is the whole consumer story. The spike shows a hand-written `impl Declared` compiles, so "nothing outside the derive can be a noun" is false as written and the undivided draft disproved it three paragraphs after asserting it. A private-witness pattern would hold, and it would put a hidden parameter on the recording trait that slice 20 has to carry through every span. |
| The primitive impls carry real values from Phase 3, uncited, and the claims about them are cited on `noun_of` | `impl Traceable for u8 { const NOUN: &'static str = "u8"; }` at Phase 3; `noun_of` `todo!()` until Phase 6 | Skeleton the impls with `const NOUN = todo!()`; cite the claims on the impl blocks and accept a green Phase 5; a `primitives()` leaf returning the list | Spiked today: a `todo!()` associated const passes `cargo check --all-targets` and fails the first real build with `error[E0080]`, taking the whole `lid-rs` test binary — every other slice's tests included — out from Phase 3 to Phase 6, and leaving `red_verdict` reading a build failure as a correct red. A claim whose only implementer is data can never be red (`phase/mod.rs:912-919`), which is precisely what `document_path` was introduced for on the trace slice. `noun_of` is also the surface slice 20 reads `NOUN` through, so it is not an artefact of the machinery alone. |
| The negative facts are pinned by `trybuild` fixtures that write the bound by hand | `lid-rs/tests/ui/vocab/fail/` — a primitive failing `T: Noun`, and a type with `Traceable` but no seal failing `impl Noun` | Assert the negative at runtime; defer every negative claim to the `noun-assertion` half; a `static_assertions`-style const trick | "No primitive is a `Noun`" cannot be observed from a passing program; only a compile failure states it. Deferring it leaves the trait's `#[diagnostic::on_unimplemented]` message — this slice's item — unpinned by this slice, and check 13's contract is that a rule is one compile error naming the rule. A hand-written bound is the *wrong* test for the emission and the *right* one for the trait, which is what this half owns. A new dependency is refused under tenet 3. |
| The message is shaped, not left to a bare `E0277` | `#[diagnostic::on_unimplemented]` on `Noun`, reading *"`{Self}` is not a vocabulary noun: a claim may name only a type carrying `derive(Traceable)`"* | Accept the raw trait error; put the message on `Declared` instead | Spiked today: the attribute on `Noun` produces exactly one error with that message for a `Traceable` non-noun (`String`), for a type implementing neither (`Plain`), and for a primitive (`u32`). Putting it on `Declared` would leave the common case — a claim naming an ordinary type — reporting a `#[doc(hidden)]` trait the author has never heard of. The attribute is stable since 1.78 and this workspace pins `rust-version = "1.97"` (`Cargo.toml:8`). |
| The re-exports stand at the crate root beside `SPECS` | `pub use vocab::{Noun, Traceable};` in `lid-rs/src/lib.rs` | `lid_rs::vocab::Traceable` only; a prelude module | `derive(Traceable)`'s expansion addresses `::lid_rs::…` in every consumer crate, and `lid_rs::Outcome` already stands at the root beside its derive for the same reason (`lid-rs/src/lib.rs:40-42`). Without the re-export the derive is unusable from `lid-rs` itself, which is where its first noun will be. A prelude is a second import surface for two names. |
| A noun's visibility is not this slice's rule | No rule | Require `pub`; deny `private_intra_doc_links` workspace-wide | The bound is a type error and not a lint, so the check holds for a private noun either way. What warns is the *document link* beside it, and `private_intra_doc_links` is warn-by-default and denied nowhere in this workspace. Denying it edits the workspace manifest, changes every crate, and fires on existing private doc links. No claim of this slice changes with the answer, so it does not block Phase 2. |

## Open Questions & Future Decisions

### Open

**1. Whether `Traceable` should carry a method now rather than only
`const NOUN`.** Slice 20 records a span's fields through this trait, and adding
a required method to a published trait is a breaking change that only `cargo
package` reports. Shipping the const alone is the constrained-first choice and
is what this document assumes. The cost of getting it wrong is one breaking
release at slice 20, on a crate with one consumer; the cost of guessing the
method's shape now is a signature written against spans that do not exist. Not
a blocker for any phase of this slice, and named so that slice 20 meets it as a
known cost.

**2. Whether the `NOUN` const should be the type's identifier or its full
path.** `Spec::NAME` is `module_path!()` plus the identifier
(`lid-rs/src/lib.rs:229-233`), and a span field named `Username` is more
readable than `myapp::auth::vocab::Username` while being ambiguous across
slices. The derive is what would produce either, so the decision is
`noun-assertion`'s to *implement* — but the primitive impls in this slice fix
the convention for `String` and `u8`, which have no module path at all, so this
slice must pick one and say so. **Chosen for the primitives: the identifier**,
because there is no alternative for `u8`. If `noun-assertion` chooses the full
path for derived nouns the two disagree, and the cost is a span whose fields
are spelled two ways. Recorded rather than settled because the evidence is
slice 20's.

### Deferred

1. `#[trace(redact)]`, `#[trace(skip)]` and README §6.5's field selection.
   Cause: they describe what a **span** records, and there are no spans until
   slice 20.
2. The seventeen LLDs' prose vocabulary links, and which slices gain a
   vocabulary file. Cause: a rewrite from zero, per slice as each is next
   touched. The ledger is the trace document
   (`lid-rs/src/trace/lld.md`, Deferred 3 names this dependency in the other
   direction and is discharged by the split: the glossary is `lid-rs-site`'s
   and needs the vocabulary files, not this slice).
3. A rule that an LLD must link its nouns. Cause: check 2 fires only on a
   written link that fails to resolve (`README.md:591`), so the positive rule
   needs `lld-check` to read the document, which is slice 13's.
4. README's amendment, and the HLD's. §3.5's "vocabulary type"
   (`README.md:417-423`) needs the marker named; §6.5 (`:1136-1145`) needs to
   say that `Traceable` is recording while `Noun` is identity; §11.1's tree
   (`:1766`) needs the collision between the slice named `vocab` and the file
   called `vocab.rs` resolved in words. HLD row 17 (`hld.md:196`) owes the
   split into two slices, the move of the policy attributes to slice 20, and
   the move of the LLD rewrite to per-slice. Cause: no gate step reads README
   against the code, so nothing stops on it — which is why it needs a named
   boundary. **One documentation commit from the main session after this
   slice's Phase 7**, written against what was built. `README.md` is at the
   workspace root, outside every crate, so `seat_of` refuses it before any
   table is consulted (`policy.rs:109-118`).

## What lands by hand, and in what order

Two things no phase of `lld/vocab` may write. Each was checked against
`policy.rs` and against what the phase after it needs.

**1. `pub mod vocab;` in `lid-rs/src/lib.rs`, and `lid-rs/src/vocab/mod.rs`
holding `#![doc = include_str!("lld.md")]` and `pub mod spec;` — between Phases
2 and 3.** Phase 2's row is the claims file and `src/spec/mod.rs`
(`policy.rs:455-461`), so it can write `lid-rs/src/vocab/spec.rs` and cannot
declare it; Phase 2's check is `cargo check --all-targets`
(`phase/mod.rs:580-582`), which passes happily over a file no module compiles,
so **check 13 never reads the claims** and the failure surfaces at Phase 5 as
"nothing registers". `src/lib.rs` *is* in Phase 3's row, so the declaration
could land there — and doing so puts check 13's rejections inside an agent that
may not open the claims file to fix them. Slice 18 (`4b036c7`), slice 19
(`705ab1b`) and slice 21 (`04f1388`) all landed it by hand between Phases 2 and
3, and all three then had every claim accepted first time. Do that.

**2. The `trybuild` fixtures under `lid-rs/tests/ui/vocab/fail/` — before Phase
5.** `tests/ui` is in the companion table's Phase 5/7 row and not the own
table's (`policy.rs:417-424`, `:434-441`), and this slice takes the own table,
so no phase of this branch may create a fixture file. They must precede Phase 5
rather than follow it because a harness whose glob matches nothing reports zero
failures, which is a vacuous green that `red_verdict` refuses — slice 19's
recorded amendment (`lid-rs/src/outcome/lld.md:498-508`). They need **no**
`.stderr` pins to be red: a `fail` fixture with no pin makes trybuild write a
`.wip` and report failure. The pins are generated with `TRYBUILD=overwrite`
once the traits exist, in the same commit as Phase 7 or immediately after, and
are **never** hand-edited (`docs/intent/publish/lld.md:83`).

Refused, and stated so that a phase stuck at Phase 5 does not invent it as a
way out: trybuild globs relative to `CARGO_MANIFEST_DIR` and needs no `tests/`
directory, so `t.compile_fail("src/vocab/ui/fail/*.rs")` would be legal and
in-policy. Every UI fixture in this workspace lives under `lid-rs/tests/ui/`,
the publish slice's `.stderr` regeneration is written against that path, and a
second home for fixtures is a permanent divergence bought for one hand commit.

**And one cascade to expect rather than rediscover.** A claim's trigger clause
must contain an intra-doc link (`README.md:417-423`, enforced by
`ATriggerClauseWithoutALinkFailsToCompile`), and the link must resolve from the
public `spec` module — so every item this slice's claims name ends up `pub`,
and a public doc linking a private item is a `private_intra_doc_links` warning
this workspace does not suppress. Between item 1 landing and Phase 3 creating
the items, `cargo doc` is therefore **broken**: the claims file is compiled, its
links point at types that do not exist yet, and `-D
rustdoc::broken_intra_doc_links` denies them. This is expected and clears at
Phase 3. Phase 2's check is `cargo check` and does not run rustdoc, so nothing
stops; the main session should not read the broken doc build between those two
commits as a defect.

## Notes for Phase 5's red set

Six things a red test of this slice gets wrong by default.

1. **Every claim here is about data, and data is never red.** The whole slice
   is traits, impls and one `const`-shaped fact per primitive. Route every test
   through `noun_of`, which is `todo!()` until Phase 6. A test that reads
   `<u8 as Traceable>::NOUN` directly is green the moment Phase 3 writes the
   impl, and `red_verdict` refuses it (`phase/mod.rs:912-919`).

2. **Do not skeleton an associated `const` with `todo!()` to manufacture a
   red.** Spiked today: it passes `cargo check` and fails the first real build
   with `error[E0080]`, which takes out the entire `lid-rs` test binary. Every
   other slice's `#[validates]` test then fails too, the red run reports
   nonsense, and the cause is three phases away from the symptom.

3. **The negative claims need a fixture, and the fixture must exist before the
   harness.** "No primitive is a `Noun`" and "the seal is a second impl" are
   compile failures. A trybuild harness with no matching fixture files reports
   zero failures and is a vacuous green. Item 2 of "What lands by hand" is the
   deadline.

4. **A `pass` fixture is not available to this half.** A positive fixture would
   want `derive(Traceable)`, which is the other slice's. The positive case is a
   test-local type in `lid-rs/src/vocab/mod.rs`'s `#[cfg(test)]` module that
   implements `Traceable`, `Declared` and `Noun` by hand — which is also the
   demonstration that the seal is deliberate rather than impossible, so it
   earns its place twice.

5. **Check 12 will mutate `noun_of`.** A test asserting only that the result is
   non-empty hands a surviving mutant to every `""` substitution. Assert the
   exact string each primitive records, primitive by primitive; the list is
   short and the assertions are the specification.

6. **The harness cannot be named for what it is about.** Check 14 admits a
   validator named for the `snake_case` of any one claim it cites, optionally
   `_suffix`ed (`lid-rs/src/claim/spec.rs:219-229`). The trybuild harness cites
   both negative claims, so it is `no_primitive_is_a_noun` — or
   `the_noun_refusal_names_the_type_as_not_a_vocabulary_noun` — and nothing
   descriptive of its own.

7. **Claim names beginning with `A` may re-bless a fixture that has nothing to
   do with this slice.** `lid-rs/tests/ui/fail/not_a_spec.stderr` pins rustc's
   alphabetically-truncated eight-name list of `Spec` implementors, whose
   eighth entry today is `ACrateRootClaimsSliceIsEmpty`
   (`not_a_spec.stderr:12-20`). A new `lid-rs` claim sorting before it displaces
   a name and fails that fixture at Phase 7. It is re-blessed with
   `TRYBUILD=overwrite`, never hand-edited; slice 21 did exactly this at
   `2c9c59d`.

## Shape

| Item | Role |
|---|---|
| `lid_rs::vocab::Traceable` | The recording trait: `const NOUN: &'static str`, the type's own name. What a span will read (slice 20); what a primitive gets and a policy does not come with. |
| `lid_rs::vocab::Traceable::NOUN` | The recorded name. Data on every impl, uncited: the claims about what a primitive records are cited on `noun_of`, which reads it, so that Phase 5 has a reachable red. |
| `lid_rs::vocab::Declared` | The seal: `#[doc(hidden)]`, empty, public because a consumer's expansion must be able to name it. Implementing it is the deliberate act that makes a type eligible to be a noun. |
| `lid_rs::vocab::Noun` | `Traceable + Declared`, empty, carrying `#[diagnostic::on_unimplemented]`. What a claim may name, and the one item whose failure message is this slice's to shape. |
| `lid_rs::implements_module!` in `lid-rs/src/vocab/mod.rs` | The citation site for the three claims about the traits. `#[implements]` dispatches over a fn, a struct or an enum and nothing else (`lid-rs-macros/src/expand.rs:205-222`), so a trait declaration and a supertrait list cannot carry one; the module is cited by containment, as the registry slice's presence claim is (`lid-rs/src/registry/mod.rs:54`). The hand commit writes a bare `mod.rs`; **Phase 3 adds this invocation to it**, `mod.rs` being inside the module directory that Phase 3's row grants. |
| `lid_rs::vocab::noun_of` | Work leaf: `noun_of::<T>() -> &'static str` for `T: Traceable + ?Sized`, answering `T::NOUN` with no value in hand. The item every data-shaped claim of this slice is cited on, and the surface slice 20 reads a noun's name through. |
| `impl Traceable for String`, `for &str`, `for bool`, `for char` | Rule V's non-numeric primitives (`README.md:515`), each recording its own type's spelling and none of them `Declared`. |
| `impl Traceable for u8 … u128`, `usize`, `i8 … i128`, `isize` | Rule V's integers, same rule. Written as a `macro_rules!` over the list so the twelve impls are one statement of one decision rather than twelve chances to disagree. |
| `impl Traceable for f32`, `f64` | Rule V's floats, same rule. |
| `lid_rs::Traceable`, `lid_rs::Noun` | The crate-root re-exports, as `lid_rs::Outcome` is (`lid-rs/src/lib.rs:40-42`): without them `derive(Traceable)` is unusable from inside `lid-rs`. Land in `src/lib.rs` at Phase 3 or 4, which is in that row. |
| `lid_rs::__private::Declared` | The seal's public address, beside the existing `linkme` re-export (`lid-rs/src/lib.rs:245-248`). A consumer's expansion writes `impl ::lid_rs::__private::Declared for TheirType`; nothing else names this path. |
| `lid_rs::vocab::tests::a_hand_declared_type_is_a_noun` | Phase 5's positive case: a test-local type implementing all three traits by hand. Also the demonstration that the seal is deliberate and not unforgeable, which is the one claim about the seal this slice makes. |
| `lid_rs::vocab::tests::no_primitive_is_a_noun` | The trybuild harness over `lid-rs/tests/ui/vocab/fail/`, a `#[cfg(test)]` unit test in the lib as check 14 and README §5.2 require. It validates both negative claims, and check 14 is a **naming** rule: a validator is admitted when named for the `snake_case` of any one claim it cites, optionally `_suffix`ed (`lid-rs/src/claim/spec.rs:219-229`). `not_a_noun`, which this row said in its first draft, is the `snake_case` of no claim of this slice and would be refused. Its fixtures are a hand commit; see "What lands by hand". |

Every item above is either a type, an impl, or one function. There is no
dispatch anywhere in the slice, so the dispatch/work rule has nothing to
separate — which is worth saying explicitly, because a Phase 4 descent that
finds no layer beneath `noun_of` has found the truth and not a gap.

**A note on doc links in this document.** It contains no intra-doc link, and
that is deliberate rather than an omission. Nothing includes this file until
`lid-rs/src/vocab/mod.rs` lands (item 1 of "What lands by hand"); from that
commit onward every link in it must resolve under `-D
rustdoc::broken_intra_doc_links`, so a link written now against a type Phase 3
has not created yet would break `cargo doc` for two commits. Locations are
plain `file:line` text, which is the form the trace document settled on for the
same class of reason.

## References

- README [§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html) — rule V's
  primitive list, and why a bound on `Traceable` is not the predicate.
- README [§6.5](https://bradvoth.github.io/lid-rs/spec/traced.html) — the
  recording trait, and the sentence that says primitives have impls with no
  policy.
- README [§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html) — the rule
  this slice makes expressible, and check 13's "unlinked noun".
- README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) — the
  slice directory, and the `vocab.rs` file whose name collides with this
  slice's.
- `lid-rs-macros/src/noun_assertion/lld.md` — the other half of HLD row 17: the
  derive, the assertion, the target forms, and the burn-down of 177 held
  claims.
- `lid-rs/src/trace/lld.md` — slice 21, for the precedent of giving a
  data-shaped claim a function to be cited on, and for the plain-`file:line`
  convention.
- `lid-rs/src/outcome/lld.md` — slice 19, for the fixtures-before-Phase-5
  deadline and the ordering corrections that produced it.
- `lid-rs-macros/src/claim/lld.md` — slice 15, for what a claim's trigger and
  object are, and for the ramp this slice's own claims join.
