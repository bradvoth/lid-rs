# A claim is written in the controlled language

## Context and Design Philosophy

A claim is the doc comment on a `#[derive(Spec)]` unit struct, and until
this slice its text is free prose: the derive reads nothing but the struct's
identifier. Every check that exists today asks whether a claim *is cited*;
none can ask whether the code *keeps what the claim says*, because the claim
has no parts a tool can compare to a signature. README
[§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html) adopts a
controlled language so that a claim has one meaning for its readers and so
that its parts — pattern, trigger, verb, response — become structure the
later checks read (README [§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html),
[§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html)). This slice makes
the derive the place where the language is *executed*: a claim that does not
parse is a compile error at the struct that carries it (check 13), and a
claim that does parse leaves its parts in the registry for every later slice
to read. It also gives validators their claim's name (check 14), so `cargo
test` prints the requirements.

The design has three commitments. **The grammar is checked at the definition
site**, by the derive, on the doc attributes it already reads — no second
tool, no source scan, and the failure names the rule and the word. **The
lexicon is a file the derive reads**, so adding a verb means defining it and
the definition is version-controlled beside the LLDs; the derive emits an
`include_str!` of the file it read so Cargo rebuilds on a lexicon change.
**The language lands strict, with a counted ramp**: a claim may be marked
free of the grammar, the mark is visible in the registry and enumerable by a
test, and the mark's removal is the burn-down this workspace pays slice by
slice — the same shape as `#[leaf]` on a public function
(README [§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)) and the
`warn` levels of the later passes. Nothing is enforced by opt-in; the opt-out
is the one thing the registry counts.

What this slice measured before it was written, at the LLD's first draft
(`bd1d2c7`) unless another figure says otherwise; §"Landing" recounts against a
later tree, which is why its totals are larger. Both were taken by reading the
tree, not by a command that can be re-run — a gap the catalog slice closes when
`cargo lid-rs` grows a claim census. Of the 275 claims in this workspace, every one already opens with *When … shall …* — the event-driven
pattern — and none is more than one sentence; 238 carry exactly one *shall*.
Four link a noun. Their verbs are 64 distinct words, and *be* is a hundred of
the 275. Of 372 validators, 242 already carry their claim's `snake_case` name
and 21 cite more than one claim. So the language's cost here is links and
verbs, not sentence shape; the ramp is what lets the derive land strict while
those are added claim by claim.

This slice's code is a proc macro, and the crate that holds it links into no
binary, so — as the macros slice established — its claims live in `lid-rs`,
in `src/spec/claim.rs`, cited by hand-authored edges at the re-export in
`lid-rs/src/lib.rs`, and validated by `lid-rs`'s tests, which are downstream
of the macros and can expand them. The types the derive's output names live
in `lid-rs/src/claim.rs`, the companion module. Because the derive runs
inside every `cargo check`, this is a compile-time slice
(`cargo-lid-rs/docs/intent/phase/lld.md`): the phases run only after the
human commits `compile-time-accepted` beside this document.

What is out of scope: vocabulary as `Traceable` types and the rule that flow
signatures are made of them (the next slice, README
[§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)); matching a
verb's template against a signature (conformance, README
[§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html)) — this slice
carries the templates and checks their syntax, it does not apply them; the
colocated slice layout (README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html)),
which changes where a claim lives and not what it says, and with it the
uncitable-claim assertion, whose path is the layout's (Deferred 2).

## The language

### One sentence, one modal

The claim is the struct's doc lines joined by single spaces, trimmed. It is
one sentence: it ends with a period, and no other period, question mark, or
exclamation mark occurs outside backticks. Colons, semicolons, and dashes are
punctuation within the sentence and are not constrained. The modal is the
word `shall`, and it occurs exactly once outside backticks; a sentence with
two is two claims, and a sentence with none is prose. `should` and `may` are
not admitted by this slice (Deferred 1).

### The pattern, from the opener

The pattern is decided by the sentence's first word and the clause it opens,
compared without regard to case:

| Opener | Pattern | Clause |
|---|---|---|
| `When …,` | event-driven | the trigger clause, up to the first comma outside backticks |
| `If …, then` | unwanted | the condition clause, up to `, then` |
| `While …,` | state-driven | the state clause, up to the first comma |
| `Where …,` | optional | the feature clause, up to the first comma |
| anything else | ubiquitous | no clause; the subject runs to the modal |

Every sentence has a pattern, since the last row admits anything; what a
sentence can fail is the clause its opener promises. An `If` opener whose
sentence lacks `, then`, or a `When`, `While`, or `Where` opener whose
sentence has no comma before the modal, is malformed, and the message names
the opener and what it expected. The words between the clause (or the
sentence's start) and the modal are the subject, and the subject is
unconstrained: *it*, *the run*, *compilation*, and *the authentication
service* are all admitted, because the subject's identity is carried by the
trigger link.

### The parts, and which of them must be links

A *link* is an intra-doc link as rustdoc reads it: `` [`Name`] ``,
`` [`Name`](path) ``, or `[text](path)`. Its *target* is the path in
parentheses when present and otherwise the backticked text. The derive
extracts:

- the **trigger** — the first link in the pattern's clause; for a ubiquitous
  claim, the first link in the subject;
- the **verb** — the first word after the modal, lower-cased; `not` between
  the modal and the verb is admitted and recorded, so *shall not count* has
  verb `count` negated;
- the **response object** — the first link after the verb;
- the **owner** — when the response object's target ends in two capitalised
  segments, `Enum::Variant`, the target with its last segment removed, as
  written: `AuthError` from `` [`AuthError::Backend`] ``,
  `crate::claim::Language` from `` [`Language::Free`](crate::claim::Language::Free) ``;
  otherwise absent. A template's `{owner}` binds to the owner's last
  segment, which is the name a signature spells.

The trigger must be a link, for every pattern: an unwanted claim whose
condition links nothing — *if the store is unreachable* — is malformed, and
the derive says which clause lacked a link. The response object must be a
link when the verb's template is a shape (below); a behaviour-only verb —
`be`, `stop`, `end` — carries no object requirement, because its meaning is
not a return type. Every link is resolved by rustdoc, not by the derive: the
derive requires that a link *exist* in the slot, and check 2 requires that
it resolve. What the link points at is unconstrained by this slice; the next
slice makes it a vocabulary type, which is the target README §3.5 states.

### The lexicon

The lexicon is the set of admitted verbs, each with a definition and a
signature template, plus the project's additions to the prohibited terms.
It is two files, read by the derive:

- the **base**, `lid-rs-macros/lexicon.toml`, compiled into the derive at
  its own build. It holds exactly the sixteen verbs below and no other: the
  six README [§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)
  names, and ten behaviour-only verbs a claim about a tool needs. A claim
  in one of this workspace's *published* crates uses no verb outside it,
  because a published crate is built by its consumers with the base and
  nothing else; the base is versioned with the crates, so a published crate
  always compiles under the base it was written against.

  | Verb | Definition | `signature` |
  |---|---|---|
  | `return` | produce as the `Ok` value of the implementing function | `["-> Result<{object}, _>", "-> {object}"]` |
  | `reject` | produce as the `Err` value of the implementing function | `["-> Result<_, {owner}>"]` |
  | `emit` | send on the named channel or sink; no value returned | `["-> Result<(), _>", "-> ()"]` |
  | `log` | write to the audit log at the stated level | `["-> Result<(), _>", "-> ()"]` |
  | `retain` | keep available for at least the stated duration | `"*"` |
  | `equal` | compare as identical under `PartialEq` | `"*"` |
  | `be` | hold the stated property; the validator observes it | `"*"` |
  | `fail` | not complete; for a derive, a compile error at the item it decorates | `"*"` |
  | `stop` | end the run at the named point, before what would follow | `"*"` |
  | `refuse` | decline the request, naming why | `"*"` |
  | `end` | reach the state from which nothing follows | `"*"` |
  | `run` | execute the named step | `"*"` |
  | `carry` | hold the named part or value, so a reader of the result finds it there | `"*"` |
  | `name` | state the identifier in the message or output | `"*"` |
  | `report` | state in the output, without stopping | `"*"` |
  | `print` | write to standard output | `"*"` |
- the **project's**, `docs/intent/lexicon.toml`, absent if the project has
  none. Its verbs add to the base, and a verb it names that the base also
  names is the project's, whole. Its prohibited terms add to the built-in
  list. This workspace's file holds prohibited extras and the verbs of its
  unpublished members; it defines no verb a published crate's claim needs.

**Finding the project's file.** The derive starts at `CARGO_MANIFEST_DIR`
and looks for `docs/intent/lexicon.toml` there; if absent it moves to the
parent, and stops — without a file — when the directory it would move to
holds no `Cargo.toml`, or after examining a directory whose `Cargo.toml` has
a line that is exactly `[workspace]`. So a member finds its workspace's
file; a crate built from the registry cache, whose parent holds no
manifest, finds none and uses the base; and a path dependency inside a
consumer's workspace is governed by that consumer's lexicon, which is stated
as a limit rather than prevented — a consumer whose `extra` prohibits a word
a vendored crate's claim uses has broken that crate's build, and the message
names the file that did it.

**The format** is a subset of TOML that the derive parses itself:

- `#` starts a comment to the end of the line; blank lines are ignored;
- `[verbs.<name>]` opens a verb, `<name>` being lower-case letters;
- inside a verb, `def = "…"` and `signature = "*"` or `signature = ["…", …]`,
  each exactly once;
- `[prohibited]` opens the prohibited table, holding `extra = ["…", …]` at
  most once;
- a string is double-quoted and contains no `"` and no backslash.

A verb's `def` is required, and its absence or repetition fails every derive;
but the parsed lexicon carries only the verb's templates. A definition is for
the reader of the file — no message quotes it and no registration records it —
so carrying it would be a field nothing reads. The file is the glossary; the
derive answers only the questions the rules ask of it.

A file outside the subset fails every derive in the crate — loud, since
every claim depends on it — and the message names the file and the line:

| The file | The message names |
|---|---|
| a line that is none of the forms above | the line |
| a verb with `def` or `signature` missing or repeated | the verb and the key |
| a key other than `def` and `signature` under a verb, or other than `extra` under `[prohibited]` | the key |
| a verb named twice in one file | the verb |
| `extra` given twice under `[prohibited]` | the key |
| a template that does not begin with `->`, or whose braces are unbalanced, or whose placeholder is not `{object}` or `{owner}` | the verb and the template |

```toml
# docs/intent/lexicon.toml

[verbs.stage]
def       = "add to the index, so the next commit carries it"
signature = "*"

[verbs.settle]
def       = "reach the state from which no further turn follows"
signature = "*"

[prohibited]
extra = ["promptly", "gracefully"]
```

A template is `->` followed by type tokens in which `{object}` and `{owner}`
are placeholders and `_` stands for any single type: `-> Result<{object}, _>`
for `return`, `-> Result<_, {owner}>` for `reject`. The derive records the
templates on the claim unmatched; matching is conformance's. `*` marks a
verb whose meaning is behaviour rather than shape.

The derive emits, with each claim, `const _: &str = include_str!("<path>")`
for the project lexicon it read, so that editing the lexicon rebuilds every
claim. The base is `include_str!`'d by the derive crate itself.

### Prohibited terms

The built-in list is the INCOSE vague-term list as the README names it:
*appropriate, adequate, as needed, as required, as appropriate, user-friendly,
reasonable, quickly, timely, easy, efficient, robust, approximately, etc.,
and/or, sufficient, normal, minimal, maximise, minimise, support*. Terms are
matched as whole words outside backticks, case-insensitively, multi-word
terms as a phrase. The project's `extra` adds to the list. A match is
malformed, naming the term.

### What the derive rejects — check 13

Each rule is one compile error at the struct, and the message names the rule
and the offending text:

| Rule | Message names |
|---|---|
| a sentence with no terminator, or with a second one | the missing period, or the second terminator |
| no modal, or more than one | the count found |
| an opener whose clause is not closed as the pattern requires | the opener and the expected `,` or `, then` |
| a verb the lexicon does not define | the verb, and the files the lexicon was read from |
| a prohibited term | the term, and the lexicon file when a project's `extra` supplied it |
| a trigger clause, or a ubiquitous subject, with no link | the clause |
| a shape verb with no object link | the verb |

The rules are checked in that order and the first failure is reported, so a
claim with two faults is fixed twice rather than reported once ambiguously.
The unit-struct rule the derive already enforces is unchanged and precedes
them all.

### What the derive records

`SpecMeta` gains one field, `claim: ClaimMeta`, and `ClaimMeta` holds the
extracted parts:

| Field | Type | Holds |
|---|---|---|
| `language` | `Language` | `Controlled` for a parsed claim, `Free` for a marked one |
| `pattern` | `Pattern` | the five patterns; `Ubiquitous` for a free claim |
| `trigger` | `&'static str` | the trigger link's target as written, or empty |
| `verb` | `&'static str` | the verb, lower-cased, or empty |
| `negated` | `bool` | `not` stood between the modal and the verb |
| `object` | `&'static str` | the response object's target, or empty |
| `owner` | `&'static str` | the response object's target with its last segment removed, as written, or empty |
| `templates` | `&'static [&'static str]` | the verb's templates as written; `["*"]` for a behaviour verb; empty for a free claim |

Every field is a literal or an enum, because a registration is a `static`.
The canary's hand-expanded registration in the registry LLD gains the field
with the canary's own parts, so the pin stays exact.

### The validator's name — check 14

`#[validates(path, …)]` on a test fn compares the fn's identifier with the
`snake_case` of each cited path's last segment and the code does not compile
unless, for one of them, the identifier is that name or begins with that name
followed by `_` — **unless every claim it cites is marked `#[lid(free)]`**, in
which case the rule does not apply. The mark exempts a claim from the language,
and a validator's name is a rule of the language about that claim; holding
the name while exempting the sentence would rename a test for a claim whose
words are not yet the ones the name would carry. So the naming lands with
the burn-down, slice by slice: when a slice rewrites its claims into the
language it renames their validators in the same commit, and until then
neither is held. The suffix is how several validators of one claim in one module —
this workspace has eight for `TheSliceComesFromTheBranchName` — stay
distinct while each says which claim it observes. The message names the
expected name for the first cited claim.

**The exemption is a const assertion, not an expansion failure.** An attribute
macro sees the cited paths and the fn's identifier and nothing else: whether
those claims carry `#[lid(free)]` is an attribute on another item, usually in
another file, which no expansion of *this* item can read. The name is therefore
compared at expansion, where the identifier is, and the exemption is deferred to
the compiler, where the claims are. `derive(Spec)` gives the `Spec` trait a
`const FREE: bool` from the mark, and when the identifier admits no cited path
`#[validates]` emits

```text
const _: () = assert!(
    <spec::Claim as ::lid_rs::Spec>::FREE && …,   // every cited claim
    "validator `wrongly_named` should be named `the_claim`",
);
```

one conjunct per cited path, with the expected name baked into the literal at
expansion. A held claim fails `cargo check` with that message; an all-free
citation compiles. When the identifier does admit a path the guard is not
emitted at all, so the common case costs nothing. This is check 1's mechanism —
a fact about the citation, stated at the citing site, resolved by the compiler
at the definition — and it keeps check 14 in Tier 0 where README §4.1 has it,
against the alternatives of dropping the exemption or moving the check to a
registry test (both in Decisions below).

`snake_case` of an identifier: each capital that is followed by a lower-case
letter starts a word, a run of capitals not so followed is one word, and
digits stay with the word before them; words are lower-cased and joined by
`_`. So `GainALibrary` is `gain_a_library`, `HTTPServer` is `http_server`,
and `Phase7Runs` is `phase7_runs`.

A validator that cites several claims is named after one of them; which one
is the author's choice, since the alternative — one validator per claim,
always — is a rule the shape of a test should not have to bend to.

### The ramp

`#[lid(free)]`, a derive helper attribute on the struct, exempts the claim
from every rule of the language except the unit-struct rule; its `ClaimMeta`
is `Free` with empty parts. The registry therefore counts the exemptions,
and `lid_rs::claim::free()` — an iterator over the `SpecMeta` entries whose
language is `Free`, in registry order — enumerates them: the enumeration a
test asserts against, and the number the trace page will show. The attribute
takes exactly the word `free`; any other content is malformed, naming it, so
the exemption cannot grow a second meaning.

The ramp's end is a Phase 8 on this slice: when `free()` is empty in a
project, the attribute is removed from the derive. This workspace's own
burn-down is per slice — each slice's Phase 8 rewrites its claims with links
and lexicon verbs and drops its marks — and the count is reported by the
test that enumerates them, so `cargo test` shows the distance to zero.

### Failure demonstrations

Check 13 and check 14 are demonstrated the way check 1 is: fixtures whose
compilation is the assertion, each failing one pinned by its `.stderr` so the
words the rules name are what the gate holds.

Check 14's pin is the one exception worth naming: its failure is a const-eval
panic, not a `compile_error!`, so the file holds rustc's framing —
`error[E0080]: evaluation panicked:` and `evaluation of `_` failed here` —
around this project's message, and the span is the `#[validates]` attribute
rather than the identifier the rule is about. The rule's own words survive
verbatim, which is what the pin is for; the framing is rustc's to change, and
`rust-toolchain.toml` is what keeps it still. When that file moves, expect this
one `.stderr` to need regenerating and do not read it as a regression. There are two kinds, because
one mechanism cannot reach the project lexicon.

**The base's rules — trybuild fixtures** under `lid-rs/tests/ui/claim/{fail,pass}/`,
run by a `#[validates]` harness in `lid-rs/src/claim.rs`. Every rule the base
lexicon alone can break belongs here: the terminator and the modal count, an
opener whose clause is unclosed or closes after the modal, a trigger clause or
ubiquitous subject with no link, a shape verb with no object, a verb the base
does not define, a built-in prohibited term, the first-failure ordering, a
`#[lid(…)]` that is not `free`, a misnamed validator, and the `snake_case`
rules. Passing fixtures assert the recorded `ClaimMeta` through
`lid_rs::SPECS` in their own `fn main()`, which trybuild runs: one claim of
each pattern, a negated verb, an owner-bearing object, a free claim, a
suffixed validator name, and a validator named for the second of two cited
claims.

**The lexicon's rules — a fixture workspace** under
`lid-rs/tests/ui/claim/lexicon/`, one member per row of the lexicon's failure
table and per rule about which lexicon governs, each member carrying its own
`docs/intent/lexicon.toml` and one claim. A second `#[validates]` harness in
`lid-rs/src/claim.rs` runs `cargo check` over that workspace, with its
own `CARGO_TARGET_DIR` — cargo locks a target directory, so a build inside
`cargo test` must not share the outer one — and asserts that each member's
expected message appears, naming its file and its line.

**Two runs, not one, and the second is forced.** `outer_pkg/inner` demonstrates
that the walk stops at a directory whose manifest opens a workspace: it carries
`[workspace]`, no lexicon of its own, and a claim taking `orbit`, the verb its
parent defines and it therefore cannot reach. A package that opens its own
workspace can be neither a member of the outer one nor a path dependency of a
member — cargo loads a member's path dependencies, finds the second
`[workspace]`, and refuses the whole workspace with *multiple workspace roots
found in the same workspace*, checking nothing at all. `exclude` does not help,
because it governs membership and not the dependency graph. So `inner` is a
path dependency of nothing, and the harness checks it in a run of its own,
merging both reports before it asserts. Its expected message is
`` no lexicon defines the verb `orbit` `` naming the base file, which is the
whole demonstration: the base was the only lexicon read.

This cost the harness its first year of life. Every member's expectation was
asserted against a report that was always empty, and the failure was invisible
because the harness was red for the *other* reason the whole time — no rule was
enforced until the swap, so a red harness was the expected state and nobody
asked which red it was. A test whose redness is expected verifies nothing until
the day it is meant to go green, and that is the day to read its output rather
than its exit code.

The second mechanism exists because the first cannot reach a project lexicon.
trybuild compiles every fixture as a `[[bin]]` of one generated package under
`<target>/tests/trybuild/<crate>/`, whose manifest it writes itself and whose
parent directory holds no `Cargo.toml`. The walk from such a fixture therefore
examines one directory that has no `docs/intent/lexicon.toml` and stops at its
parent for want of a manifest: a lexicon file placed beside a trybuild fixture
is never read, and every trybuild fixture answers to the base alone. A member
of a real workspace finds its own lexicon in its own manifest directory, which
is what the rules are about.

The const-assertion amendment added three behaviours, and each needs a fixture
the enumerations above did not ask for. `pass/free_mark.rs` gains an assertion
on `<C as Spec>::FREE` for a marked and an unmarked claim (`TheFreeConstRecordsTheMark`).
`pass/validator_names.rs` gains a correctly-named validator of a *held* claim,
whose compilation is the demonstration that no assertion was emitted
(`AnAdmittedNameCarriesNoAssertion`) — an emitted assertion over a held claim
would fail it. A new `pass/validator_free.rs` carries a deliberately misnamed
validator citing only free claims (`AValidatorCitingOnlyFreeClaimsIsNotHeldToTheName`);
it is the one fixture whose *passing* is the whole of its point, and it must
not be marked free itself for the reason §"Landing" now gives.

**Their edges are Phase 4's, not Phase 7's.** The hand-authored `claim_edge!`
entries live in `lid-rs/src/lib.rs`, which the policy admits for Phases 3 and 4
and refuses for 5 and 7. A claim added by an amendment after Phase 4 has run
therefore has no phase left that can cite it, and check 10 fails at the gate on
a file the Phase 7 agent may not open. Phase 4 is reopened for the edge, or the
edge is hand-committed with the amendment that added the claim; it is never
Phase 7's to discover. `TheFreeConstRecordsTheMark` is kept by
`claim::expansion`, which reads the mark and emits both; the other two by
`claim::validator_name`, which decides whether a guard is emitted at all.

Each harness cites every claim its fixtures demonstrate, and each is red until
the derive enforces the language — the failing fixtures compile, so the
harness fails. That is also what makes the five claims whose response is that
something *compiles* or *is admitted* red before Phase 6 — `AFreeClaimCompilesWhateverItsText`,
`ASuffixedValidatorNameIsAdmitted`, `AValidatorIsNamedForAnyOneCitedClaim`,
`AnAdmittedNameCarriesNoAssertion` and `AValidatorCitingOnlyFreeClaimsIsNotHeldToTheName`:
they are cited by a harness whose other fixtures are wrong. They are enumerated
rather than counted because the count was left at three across an amendment and
the two it omitted were the two that reached Phase 6 with no validator at all. The cost is that those claims share a
validator with their siblings rather than having one that fails for their own
reason, and check 12 cannot tell them apart.

### Sequence: pin, then swap

The derive runs inside every `cargo check`, so a `todo!()` on its path is a
panic in the compiler on every claim in the workspace. The skeleton therefore
leaves `derive_spec` as it is and adds the new items beside it, unwired:
`claim::parse`, `claim::expansion`, `claim::validator_name` and the lexicon
exist with `todo!()` bodies and nothing calls them.

Phase 5's red is the fixtures: the failing fixtures compile, because no rule is
enforced, and the harness fails; the unit tests in `lid-rs` that call the
companion's `free()` fail because *every* claim reads as free, not because none
does: the unwired derive emits `Language::Free` for all of them, so `free()`
returns every registration and the enumeration goes red on finding this slice's
own claims among the marked.

**The swap is a hand commit, and it comes after Phase 6.** `expand.rs` is in no
phase's path, so the two call sites — `derive_spec` to `expansion`, `citation`
to `validator_name` — can only be wired by hand; and they cannot be wired while
any leaf beneath them is `todo!()`, because that is a compiler panic on every
claim in the workspace. Phase 6 therefore implements the leaves *while they are
still unreachable*: nothing calls `expansion` or `validator_name`, so filling
them in changes no behaviour and cannot break the build. Phase 6 is the one
phase with no check of its own, which is what makes this order possible — there
is no green it is required to reach. The hand commit that follows flips both
call sites at once, and the fixtures and `free()` tests turn green in that
commit. Phase 7 then gates the result.

The pin is the existing registry-content tests of the macros slice, which must
stay green unchanged across the swap.

### Where the phases write

The phase policy resolves a slice's crate from its LLD, and this LLD is in
`lid-rs-macros`, so Phases 3, 4, and 7 edit `lid-rs-macros/src/claim.rs`,
`lid-rs-macros/src/claim/`, and `lid-rs-macros/src/lib.rs`. The rest of the
slice is in `lid-rs`, and the policy must admit it: the claims at
`lid-rs/src/spec/claim.rs` (Phase 2), the companion module
`lid-rs/src/claim.rs` with its types, hand-authored edges, and validators
(Phases 3–7), and the fixtures under `lid-rs/tests/ui/` (Phase 5). That
admission — *a slice of a proc-macro crate keeps its claims, its companion
module, and its fixtures in the crate that re-exports the macros* — is a
rule of the `phase` slice, made before this slice's Phase 2 runs; the
re-exporting crate is named by `[package.metadata.lid_rs] companion` in the
proc-macro crate's manifest, read from `cargo metadata`. Three more slices
in the HLD's map extend the same crate and need the same rule.

Outside every policy, and so the human's, committed by hand as `phase 8:`
edits gated by hand. Numbered in the order they are made, each naming the
phase boundary it sits at — the list has been reordered by amendment twice,
and the numbering is the only thing that says when.

1. **The lexicon files, before Phase 3.** `lid-rs-macros/lexicon.toml` and
   `docs/intent/lexicon.toml` are Shape artifacts that no phase may write —
   for Phases 3 and 4 the policy admits `src/<slice>.rs`, `src/<slice>/` and
   `src/lib.rs`, and nothing else — so they arrive by hand, with the base's
   sixteen verbs as §"The lexicon" enumerates them. The bound is *before Phase
   3*, not before Phase 6: the base's `include_str!` is a `const` in the
   skeleton, so it must resolve for Phase 3's own `cargo check`.
2. **The registry's field, after Phase 3.** `ClaimMeta` is this slice's type
   and Phase 3 is what creates it, so `SpecMeta` cannot carry it before then;
   and `expand.rs`, `graph.rs`, and `registry.rs` are in no phase's
   allowed set. One commit therefore adds `claim: ClaimMeta` to `SpecMeta`,
   has `derive_spec` emit the `Free` literal inline and unwired, fixes the
   `SpecMeta` literal in `graph.rs`'s test helper, and amends the registry
   LLD's `SpecMeta` table and hand expansion — which is a live doctest, so
   the expansion contract stays compiler-checked. A `Free` `ClaimMeta` is
   what an unwired derive means and what the ramp already admits. The field
   is a breaking change to every crate whose registrations the derive writes,
   and `cargo package` is what says so.
3. **The migration, after the lexicon files and before Phase 5**, as below —
   except its last bullet, the marking of fixture claims, which only a phase
   that may write `tests/ui` can make and which is therefore Phase 5's.
4. **The trait's const, after Phase 2's amendment.** `Spec` gains
   `const FREE: bool` and `derive_spec` emits it as `true`, unwired, exactly as
   the `claim` field arrived. It lands here and not with the swap because the
   amended claims link `crate::Spec::FREE`, and an unresolved intra-doc link is
   check 2 failing — a Tier 0 check, which no phase of a slice may leave
   failing. Nothing reads the const yet, so it is inert; and reading `true` for
   every claim is what keeps check 14's misnamed fixture compiling, and so red,
   until the swap. The trait gains a *required* const, so every hand-written
   `impl Spec` moves with it, under the same version discipline as the field
   above.
5. **The swap, after Phase 6**, once no leaf beneath the call sites is
   `todo!()`. It makes `expand::derive_spec` and `expand::citation` delegate
   to `claim::expansion` and `claim::validator_name`, and replaces the inline
   `Free` literal and the `FREE = true` with the two things `expansion` now
   returns. This is the commit the fixtures turn green in.

### Landing in this workspace

The derive lands strict, so the workspace's 275 claims and the validators
whose names differ from their claim's must be brought under it before the
harness first runs green, or the gate fails on the slice that introduces the
gate. Both are mechanical and neither changes a claim's meaning:

- every existing claim is marked `#[lid(free)]`, and this slice's own claims
  are written in the language from the first;
- no validator is renamed now. Recounted at this section's writing — 389
  validators against the 372 above, the tree having grown in between — 133 would need it, and
  every one cites only claims the migration marks free — so check 14 does
  not hold them, and each slice renames its own when it rewrites its claims.
  Renaming them today would have produced names like
  `the_slice_comes_from_the_branch_name_phase_check_rejects_other_flags_by`:
  eight validators of one bundled claim, each forced to carry a name about
  branch names for a test about flag parsing. That the rule produces such
  names against a bundled claim is a signal the claim wants splitting, which
  is that slice's Phase 8 and not this one's;
- the `init` templates' example claim is written in the language, and
  README §3.2's example claim likewise;
- every claim inside a trybuild fixture — this slice's own passing fixtures
  and the macros slice's, which the derive compiles like any other — is
  marked `#[lid(free)]` unless the fixture exists to demonstrate the
  language itself. A fixture's claim is there to exercise a citation or a
  registration; holding it to a grammar it was not written for
  would make every such fixture a claim about the system, which it is not.
  Only a phase that may write `tests/ui` — 5 or 7 — can do this, so it is
  Phase 5's, not Phase 6's.

  **A check-14 fixture is the exception, and its claims stay held.** The mark
  is what switches check 14 off: a validator cited against a free claim is not
  held to its name, so marking the claim of a fixture that demonstrates the
  name disarms the very rule the fixture exists to show. In `fail/validator.rs`
  the fixture would stop failing and the harness would fail instead; in
  `pass/validator_names.rs` the rule would quietly stop being exercised with no
  test going red. Read the rule as: a fixture's claim is marked free unless the
  fixture is *about* the claim — its grammar or its name.

## Shape

| Item | Role |
|---|---|
| `lid_rs::claim::Pattern` | The five EARS patterns, as an enum a registration can name. |
| `lid_rs::claim::Language` | `Controlled` or `Free`. |
| `lid_rs::claim::ClaimMeta` | The extracted parts of one claim; the new field of `SpecMeta`. |
| `lid_rs::claim::free` | The registered claims marked free, from `SPECS`, as an iterator. |
| `lid_rs::Spec::FREE` | Whether the claim carries `#[lid(free)]`, as a const a citation can project. The trait's second associated const, emitted by `derive(Spec)` from the same read of the mark that yields `ClaimMeta.language`. |
| `lid_rs::claim` tests | The harness over `lid-rs/tests/ui/`, and the tests over `free()`. |
| `lid_rs_macros::claim::expansion` | The derive's expansion, in this slice's module: reads `#[lid(free)]`, calls `parse` with the lexicon, and yields **both** what the registration carries — the `ClaimMeta` expression and the `include_str!` of the project lexicon — **and the mark it read, as the `bool` the trait's `FREE` is emitted from**. One read of the mark, two emissions; `derive_spec` names them in one line each. It does not yield the `ClaimMeta` alone, because then `FREE` could only be recovered by re-emitting that expression and matching on its `language`, which duplicates the literal and the `include_str!` in every one of this workspace's claims. |
| `lid_rs_macros::claim::validator_name` | Check 14, in this slice's module: compares the test's identifier with each cited path's `snake_case` and yields the guard the citation emits — nothing when one path is admitted, else the `FREE` assertion naming the expected name. `expand::citation` delegates to it for `#[validates]`; `#[implements]` is untouched. |
| `lid_rs_macros::claim::parse` | The sentence to its parts, or the first rule it fails; pure over the joined doc text and a lexicon. |
| `lid_rs_macros::claim::sentence` | The doc attributes joined, trimmed, and checked for one terminator. |
| `lid_rs_macros::claim::pattern` | The opener to its pattern and clause, or the clause it lacks. |
| `lid_rs_macros::claim::link` | The first link in a span of text, and its target. |
| `lid_rs_macros::claim::snake_case` | The identifier to its name, by the rule check 14 states. |
| `lid_rs_macros::claim::Parts` | The derive side's own reading of one claim — pattern, trigger, verb, negation, object, owner — which `expansion` renders into the registration's literals. `lid-rs` depends on `lid-rs-macros`, not the reverse, so the macro side cannot name `lid_rs::claim::ClaimMeta`. |
| `lid_rs_macros::claim::Pattern` | The five patterns on the derive side, for the same reason. |
| `lid_rs_macros::claim::lexicon::Lexicon` | The admitted verbs with their templates, and the prohibited terms; base merged with project, read through `verb`, `extra`, and `project`. |
| `lid_rs_macros::claim::lexicon::Verb` | One admitted verb's templates, and whether they are a shape. `is_shape` is where that branch lives, and it is the *only* place it lives. |
| `lid_rs_macros::claim::lexicon::read` | The base, `locate`, and `parse` composed into the lexicon a crate answers to; where a project verb is admitted whole. |
| `lid_rs_macros::claim::lexicon::Template::parse` | The template syntax check, taking the verb and the file its message names. |
| `lid_rs_macros::claim::lexicon::BASE_FILE` | The base's name, the other half of "the files the lexicon was read from". |
| `lid_rs_macros::claim::lexicon::locate` | The walk up from `CARGO_MANIFEST_DIR` to the project's file, if any, with its bound. |
| `lid_rs_macros::claim::lexicon::parse` | The TOML subset to a `Lexicon`, or the line and key that are outside it. |
| `lid_rs_macros::claim::lexicon::Template` | One signature template: `Any` for `*`, `Shape` for a template as written. The distinction is data here and a decision only in `Verb::is_shape`, which reads it. |
| `lid-rs-macros/lexicon.toml` | The base lexicon. |
| `docs/intent/lexicon.toml` | This workspace's lexicon: prohibited extras and unpublished members' verbs. |
| `lid-rs/tests/ui/claim/{fail,pass}` | The trybuild fixtures of *Failure demonstrations*, with their `.stderr` pins. |
| `lid-rs/tests/ui/claim/lexicon/` | The fixture workspace: one member per lexicon rule, each with its own `docs/intent/lexicon.toml`. |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Where the grammar is checked | In `derive(Spec)`, at the definition site | A test over the registry (a check 10-style intersection); a `cargo lid-rs` command over source | The derive already holds the doc text and can fail at the struct with the word named; a test can only report a claim after it compiled and registered, and a command over source is a parser the design forbids. |
| Where the slice lives | The LLD in `lid-rs-macros`; claims, companion, and fixtures in `lid-rs` under a `phase` rule that admits them | Two LLDs, one per crate; the slice built by hand as the macros slice was; a phase agent with a widened policy | Two LLDs split one operation in half and each half fails question 8's converse. Building by hand abandons the gate that holds the shape. A widened policy is a hole; a *stated* rule for proc-macro slices is a policy, and three more slices need it. |
| The lexicon's home and discovery | A base file compiled into the derive crate, plus a project file found by walking up from `CARGO_MANIFEST_DIR`, bounded by the manifests | A path in `[workspace.metadata.lid_rs]`; a `lexicon!` macro invoked once per crate; one file per crate with no base; an unbounded walk | A proc macro cannot read workspace metadata without parsing every `Cargo.toml` above it, and cannot see items another macro defined. The bounded walk finds the workspace's file from any member with no manifest parse, and finds nothing from the registry cache, so a published crate's claims answer to the base alone — which is what makes them publishable. |
| Parsing the lexicon | A hand-written parser for the stated TOML subset | The `toml` crate in `lid-rs-macros` | The format is this project's — five forms, enumerated with their failures — and a subset parser is a hundred lines, against a dependency in a proc-macro crate that every consumer compiles. Tenet 3; revisit if the format grows a sixth form. |
| Behaviour-only verbs | First-class, `signature = "*"`, no object link required | Every verb has a template; rewrite claims that use *be* | A hundred of this workspace's claims say *shall be*; a copula has no return shape and forcing one invents structure the claim does not have. `*` is the honest marker README §3.5 already names. |
| Landing strict, with `#[lid(free)]` | Every existing claim marked, marks counted and enumerable, burned down per slice | Land the derive lenient and tighten later; rewrite all 275 claims before the derive lands; a `level = "warn"` a proc macro cannot emit on stable | A lenient derive is a check that does not gate (constraint 3). A prior rewrite is eleven slices' Phase 8 inside one branch with no gate holding the shape until the end. A proc macro has no stable warning channel, so the ramp must be a mark the registry can count — the shape `#[leaf]` already has. |
| The mark is a derive helper attribute | `#[lid(free)]`, declared by `derive(Spec)` | Extending the `#[spec("…")]` attribute macro; a doc-comment marker | A helper attribute is inert until the derive reads it, so it cannot change the item, and it is visible at the struct; the `spec` attribute rewrites the item for a different purpose, and a marker inside the claim's text is text the language would have to parse around. |
| The skeleton leaves the derive unwired | New items beside `derive_spec`, wired at Phase 6; red is the fixtures | Wire at Phase 3 with `todo!()` on the path | A `todo!()` inside a derive is a compiler panic on every claim in the workspace at every `cargo check` from Phase 3 to Phase 6. The macros slice's pin-then-swap is the precedent. |
| The response object is required only for shape verbs | Object link required iff the verb has a template | Always required; never required | *shall end* has no object; *shall return a* `` [`Session`] `` has one and conformance needs it. The template is the thing that says which. |
| Check 14 admits a suffix and any cited claim | The name, or the name followed by `_…`, of any one cited path | Exact equality; equal to the first cited path; one validator per claim, one test module per claim | Eight validators of one claim in one module cannot share a name; a suffix keeps each saying which claim it observes. "The first" makes citation order load-bearing; a module per claim reshapes tests for no structural gain. |
| The free-claim exemption from check 14 | A const assertion over `Spec::FREE` emitted by `#[validates]`, check 14 staying Tier 0 | Drop the exemption and rename the 133 validators now; move check 14 to Tier 1 as a registry test over `SPECS` and `VALIDATIONS` | The macro cannot read another item's attributes, which is what made the first two look like the only options. It can emit a condition the compiler resolves — check 1's mechanism, and Deferred 2's. Renaming now produces names about the wrong claim (see Landing); a registry test moves a check out of the compiler for a fact the compiler already has, and costs README a tier move. |
| Struct names are not checked for a verb | No rule on the identifier | README §3.2's "the derive checks that the name contains a lexicon verb" | Identifiers conjugate — *Stops*, *Refused* — so the check either admits a form list per verb or fires on good names. Constraint 3: a check with that false-positive rate is deleted before it is written; README §3.2 says the name *follows* the discipline. |
| `should` and `may` | Not admitted | Admit and record; admit and relax checks 10 and 11 | Recording them without relaxing 10 and 11 makes a `may` claim fail check 11 for having no validator, which is the relaxation's job — and that is the intent-graph slice's LLD. Deferred with the relaxation, as one change; README §3.5 states the target. |
| The workspace's migration edits | Hand-committed `phase 8:` edits between this slice's Phase 3 and Phase 5 | A migration subcommand; a phase agent with a widened policy | The edits are mechanical, one-time, and touch every slice; a subcommand is code with a lifetime of one use, and a widened policy is a hole in the thing the policy exists for. Precedent: the cross-slice fixes the canopy slice hand-committed. |

## Open Questions & Future Decisions

### Deferred

1. `should` and `may` as modals, with checks 10 and 11 relaxed for
   non-normative claims — one change across this slice and the intent-graph
   slice.
2. The uncitable-claim assertion — a const projection through the claim's
   citable path, emitted by the derive — lands with the layout slice, since
   the citable path is the layout's to define.
3. The end of the ramp: removing `#[lid(free)]` from the derive when
   `free()` is empty here.
4. Matching templates against implementer signatures — the conformance slice.
5. The trace page's column for the free count and the lexicon's verbs — the
   page slice.
6. **Check 12's mapping and cost for this slice, unresolved.** `plan_for_mutant`
   matches an implementation edge by `Edge::file`, and this slice's edges are
   hand-authored in `lid-rs/src/lib.rs`, so their `file!()` is never
   `lid-rs-macros/src/claim.rs`. Every mutant in the macro code therefore falls
   to `TestPlan::FullSuite` and runs the whole workspace suite — which this
   slice has just extended with a nested `cargo check --workspace` over the
   fixture workspace's members and a trybuild run over some forty fixtures.
   With `--baseline skip` cargo-mutants has no measured baseline to scale its
   timeout from, and a timeout is counted a survivor, so the predictable
   failure is a wall of false survivors on a gate that already runs seventeen
   minutes. The fix is a mapping that reaches a hand-authored edge's *subject*
   rather than its file, which is the catalog slice's or the layout's — under
   the colocated layout the edge and the code share a directory and the
   question may dissolve. Named here because it is a cost this slice creates
   and cannot pay.
7. `cargo lid-rs init` writing a project lexicon, once a scaffolded
   project's first claim needs a verb the base lacks.

## References

- README [§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html) — the
  language and the executable lexicon; [§4.1](https://bradvoth.github.io/lid-rs/spec/gates.html)
  checks 13 and 14; [§3.2](https://bradvoth.github.io/lid-rs/spec/mapping.html)
  names.
- `lid-rs/docs/intent/registry/lld.md` — `SpecMeta`, the registration form,
  and the canary expansion this slice extends by one field.
- `lid-rs-macros/docs/intent/macros/lld.md` — the derive this slice extends,
  the pin-then-swap sequence, and the proc-macro-crate exception under which
  its claims live in `lid-rs` and are cited by hand at the re-export.
- `cargo-lid-rs/docs/intent/phase/lld.md` — the path policy and the
  compile-time disclosure this slice is subject to.
- EARS (Mavin et al., 2009), RFC 2119, the INCOSE Guide for Writing
  Requirements (vague terms), ASD-STE100 (a closed verb set) — the four
  standards the profile stacks; each is cited for its idea, none for its
  dictionary.
