<!-- ANCHOR: header -->
# LID-rs

**Linked-Intent Development, compiled.**
A spec-driven workflow for Rust in which the intent graph is made of Rust items,
so the compiler — not grep, and not a second resolver — enforces the links.

LID-rs is an opinionated, Rust-specific implementation of
[Linked-Intent Development (LID)](https://linked-intent.dev/), which is the
source of the idea: link design intent to code through an *arrow* — the
directed chain running HLD → LLD → atomic claim → test and code — walkable in
either direction because every artifact cites its neighbours. Compiled into
Rust items, that arrow is what these pages call the *intent graph*. Where LID is
language-agnostic and enforces the arrow by convention and tooling, LID-rs
trades that generality for teeth — every edge of the graph becomes something
the Rust compiler, linker, or test harness resolves and gates.

<!-- ANCHOR_END: header -->

---

<!-- ANCHOR: premise -->
## 0. The premise

LID (Linked-Intent Development) links a design document to the code built from
it via greppable requirement IDs: an HLD states the *why*, LLDs state the *how*,
EARS one-liners (Easy Approach to Requirements Syntax — "When X, the system
shall Y") state atomic claims, tests assert those claims, and code carries
`@spec` annotations citing them. `grep -r AUTH-UI-001` returns the whole arrow.

That linkage is mechanical but *lexical*. An `@spec` comment is a string. It can
cite a requirement that was deleted, describe behaviour the function no longer
has, or be absent entirely from code the agent invented on its own initiative.
LID's own `bidirectional-differential` experiment — reconstructing the spec
from the code in a fresh session and diffing it against the written one —
exists because specs and code drift apart despite the IDs.

The adjustment: **replace the free-text spec layer with a refinement skeleton
made of real signatures**, and make every edge of the graph something the
compiler resolves. Two additional principles come from stepwise refinement:

- **Dispatch and work are separate functions.** A function either makes one flow
  decision (one `match`, one `if/else` chain, however many arms) or it does one
  unit of work. Never both. Every branch is a decision; every decision should
  have been declared in the design.
- **Refine breadth-first within a vertical slice** — a slice being one
  user-visible operation, end to end. Complete a whole abstraction layer for
  that operation before descending, so cross-cutting corrections land before
  implementation effort is sunk.

**A leaf with a branch in it is a
requirement nobody wrote down.** That turns the complexity rule into a drift
detector you can put in CI.
<!-- ANCHOR_END: premise -->

---

<!-- ANCHOR: purpose -->
## 1. What this is for

### 1.1 Readability, extended from requirements to implementation — and into runtime

Engineers have long held that code is read far more than it is written, and that
readability therefore outranks writing efficiency. That principle has mostly
stopped at the source file. Requirements documents, design documents, and tests
are written to be *produced* — reviewed once, filed, and thereafter consulted
only under duress. And the running system speaks a third language altogether:
logs and metrics named by engineers for engineers, with no path back to the
requirement they serve.

LID-rs extends the readability principle across the whole arrow and out the far
end. The claim is written in a controlled language so it has one meaning — and
because that language is also machine-readable, the claim's parts constrain the
code: its trigger appears in the signature, its verb dictates the return shape,
its named error variant must exist and be the only reason that variant exists
([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)). The claim's
nouns are types, so the design prose is hyperlinked to the code and flow
signatures are made of glossary words ([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)).
The test is named after the claim, so the test runner prints the requirements.
The flow layer is identified by its shape and guaranteed to read as prose
([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)). Every
claim-bearing function opens a span named for its claims, so a test's output —
and a production trace — reads as a sequence of requirements being satisfied
([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)). And a generated
site assembles all of it into one narrative that cannot drift from the code
because it is built from it ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html)).

Every artifact is optimised for the reader who arrives later, including the
person on call.

### 1.2 The goal is to make semantic drift the only thing left to review

The design goal is not "catch drift." It's **to make every structural
property hold automatically, so that reviewer attention lands on meaning and
nothing else.**

Structural correctness — does this code cite a real claim, does the signature
promise what the claim says, does a test depend on this code, did the claim's
promised outcome actually occur, does every error variant trace to a
requirement, did the design change reach every call site — is mechanical work.
Machines are good at it and humans are terrible at it, especially at review time,
especially across a diff an agent produced in one pass. Semantic correctness —
does this code mean what the claim says — is the opposite. No tool can check
it. It's the reason a human is in the loop. Every check in [§4](https://bradvoth.github.io/lid-rs/spec/gates.html) exists
to remove a class of question from review. When someone reviews a refinement skeleton
(Phase 3 of the flow, [§8](https://bradvoth.github.io/lid-rs/spec/flow.html)), the
composition already type-checks, every citation already resolves, every
signature already keeps its claim's promise, and the
cascade has already reached every affected site — so the only remaining question
is *is this the right decomposition of the problem*, which is the question worth
their time.

The iterative refinement flow ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)) serves the same end from the other direction.
By the time an implementation is written, its name, signature, claim, and failing
test are all pinned, so the semantic question at that point is small and local:
*does this body mean what this claim says.* Both halves of the system are aimed
at shrinking the surface a human has to think hard about — not at eliminating
the thinking.

The residual is real and named in [§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html).
The differential pass exists for that case, and the system's success condition
is that it is the only thing left for it to do.

### 1.3 This is not for prototypes

Everything here is machinery for code that must remain trustworthy over time,
under modification, by people and agents who weren't there when it was written.
That machinery has a real cost: a design document per slice, a claim per
behaviour, a test per claim, and a review gate at every refinement layer.

Exploratory work should skip all of it. Proofs of concept, spikes to answer a
feasibility question, notebooks, one-off scripts, research code whose output is a
number and not a system — the correct amount of LID-rs in those is zero. Adopt at
the point the code stops being disposable, which is usually the moment someone
proposes building on it.

Adopting mid-life is a supported path (the brownfield note, [§11](https://bradvoth.github.io/lid-rs/spec/layout.html)), and better
than adopting early. A spike that earned its way into production arrives
with its design decisions already discovered; writing the LLD after the fact is
cheap because you know the answers.
<!-- ANCHOR_END: purpose -->

---

<!-- ANCHOR: constraints -->
## 2. Design constraints

Three constraints shaped every decision below, and they rule out most of the
obvious implementations:

1. **Stable toolchain only.** No nightly, no rustc internals. A methodology that
   forces a toolchain choice on its adopters isn't adoptable.
2. **No second implementation of name resolution.** Any tool that parses Rust
   source to reconstruct the spec graph will diverge from the compiler's
   view — `use` renames, re-exports, `#[cfg]`, macro expansion. Silent divergence
   in a correctness tool is worse than no tool. Syntactic passes over one item's
   tokens are permitted — classifying a body's shape, reading a signature's
   type tokens — because they resolve nothing; **the line is resolution, not
   parsing.**
3. **Every check gates, or it gets deleted.** A report nobody reads is worse than
   an absent check, because it lets you believe coverage exists. If something
   can't be made to fail the build, remove it and be honest about the gap.

Corollaries that recur throughout: **a check built on enumeration must first
prove the enumeration is non-empty** — a registry that silently fails to
populate turns every check over it into a vacuous pass ([§5.3](https://bradvoth.github.io/lid-rs/spec/registry.html));
generated documentation has no content of its own ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html));
a warning is a ramp, not a resting state ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html));
nothing is enforced by opt-in; and in production, the analogue of a gate is an
alert ([§6.7](https://bradvoth.github.io/lid-rs/spec/traced.html)).

One more, which the controlled language ([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html))
makes possible: **a claim's content is structure, not prose.** The language was
adopted to remove ambiguity for readers. It also makes every claim parseable
into trigger, verb, response, and pattern — and each of those parts can be
compared to the code. A grammar that only humans read is half-used.
<!-- ANCHOR_END: constraints -->

---

<!-- ANCHOR: mapping -->
## 3. Mapping: LID concept → Rust mechanism

| LID artifact | Rust mechanism | Why this mechanism |
|---|---|---|
| **HLD** — the *why* | `#![doc = include_str!("../docs/intent/hld.md")]` in `lib.rs` | Stays a reviewable markdown file in the repo; renders as the crate's front page in `cargo doc`, directly above the API it governs. |
| **LLD** — the *how*, per slice | `#[doc = include_str!("lld.md")]` on the slice module | One LLD per module. The design and the module boundary become the same boundary; the LLD's nouns are intra-doc links, so the prose is link-checked ([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)). |
| **EARS claim** | `#[derive(Spec)]` on a unit struct; the doc comment *is* the claim | A claim becomes a nameable, linkable, resolvable item. The derive reads the `#[doc]` attributes, so the text is single-sourced; it enforces the controlled language and extracts the claim's parts into `SpecMeta` ([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)). |
| **Spec ID** | The struct's own name, written descriptively | See [§3.2](https://bradvoth.github.io/lid-rs/spec/mapping.html). |
| **Verb** | An entry in `lexicon.toml` with a definition **and a signature template** | The verb dictates the return shape of every implementer ([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)). |
| **Glossary** | A `vocab` module of `Traceable` newtypes | One word, one meaning, one logging policy — and the only types allowed at flow signatures ([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)). |
| **Error taxonomy** | `#[derive(Outcome)]` enums whose variants are named by *unwanted* claims | The enum is implied by the claims, in both directions ([§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html)). |
| **`@spec` annotation** | `#[implements(spec::ValidCredentialsYieldScopedSession)]` | Emits a doc link and a registry entry whose `NAME` projection is a type assertion — a bad citation is a **type error**, not a broken link — and a span ([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)). |
| **Test → claim link** | `#[validates(spec::ValidCredentialsYieldScopedSession)]`, named after the claim | Same two emissions, on the test side, plus a capturing subscriber. |
| **Flow layer** | Derived from body shape | A function *is* flow if its body reads as prose ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)). |
| **Traceability matrix** | `docs/intent/trace.md`, generated, freshness-gated | LID's grep output as a committed page ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html)). |
| **Coverage of the graph** | `linkme` distributed slices + an ordinary `#[test]` | Enumeration at link time. No source parsing anywhere. See [§5](https://bradvoth.github.io/lid-rs/spec/registry.html). |
| **Reachability and outcome** | Captured span trees | The claim's code ran; its promised response occurred ([§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html)). |
| **Non-vacuous assertion** | Diff-scoped `cargo-mutants`, test subset narrowed by the registry | Proves the test *depends on* the implementation, not merely that it executed it. |
| **Call graph** | The union of validator traces | Observed, exact, no resolution ([§6.6](https://bradvoth.github.io/lid-rs/spec/traced.html)). |
| **Runtime intent** | The same spans, in production | Telemetry speaks the requirements' language ([§6.7](https://bradvoth.github.io/lid-rs/spec/traced.html)). |
| **Spec → code cascade** | Recompilation; `#[non_exhaustive]`; exhaustive `match` | Adding a case breaks every dispatch site. A compiler error, not an agent pass. |
| **Spec retirement** | Rename the struct; keep the old name as a `#[deprecated]` type alias beside the specs | Warns at every citation site through the const assertion; the alias registers no claim, so the graph sees only the new one. Stable compiler, zero tooling. |
| **Refinement skeleton** | Signatures with `todo!()` bodies | `!` coerces to any type, so a whole layer **type-checks before any leaf exists**. |
| **Complexity budget** | `clippy.toml` thresholds | Enforces dispatch/work separation, which is what makes undeclared decisions detectable. |

### 3.1 Specs are Rust items, authored in `src/`

```rust
//! Atomic claims for the auth slice. Derived from the slice's `lld.md`.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these types
//! exist so that citations are resolved by the compiler rather than by grep.

use lid_rs::Spec;

/// When a user submits valid [`Credentials`], the authentication service shall
/// return a [`Session`] scoped to that user.
#[derive(Spec)]
pub struct ValidCredentialsYieldScopedSession;

/// If submitted [`Credentials`] match no stored [`Account`], then the
/// authentication service shall reject them with
/// [`AuthError::InvalidCredentials`].
#[derive(Spec)]
pub struct UnknownCredentialsAreRejected;

/// If the [`CredentialStore`] is unreachable, then the authentication service
/// shall reject the submission with [`AuthError::Backend`].
#[derive(Spec)]
pub struct BackendFailureIsRejected;

/// The [`AuthError`] user-facing message shall equal the same message for
/// every variant.
#[derive(Spec)]
pub struct BackendFailureIsIndistinguishableToUser;
```

A derive rather than a block macro. Doc comments stay ordinary doc
comments, so rustdoc, rust-analyzer hover, `missing_docs`, and go-to-definition
all behave normally instead of interacting with a macro arm. Vocabulary, used
throughout: the *claim* is the sentence in the doc comment; the *spec* is the
Rust item carrying it — what citations resolve to and registries enumerate.
Each spec is a real item at a real source location.

The fourth claim above is the one an agent's first draft usually hides inside
the third: "shall reject with `AuthError::Backend` *and shall not distinguish
it from a mismatch*" is two claims, and the second turns out to be an invariant
on `AuthError`, not a behaviour of `authenticate`. The controlled language
([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)) rejects the compound
form, and the split is what makes the invariant visible.

### 3.2 Names, not numbers

`AUTH-UI-001` is a holdover from requirements-document culture, where an ID is a
stable handle for cross-reference in prose. Citations here aren't prose — they're
paths the compiler resolves — so the number buys nothing the name doesn't.
`grep -r ValidCredentialsYieldScopedSession` works as well, and rustdoc
search finds the item by its real name.

More importantly, a descriptive name makes every citation site self-documenting.
`#[implements(spec::BackendFailureIsIndistinguishableToUser)]` states its own
content; `#[implements(spec::AuthUi003)]` requires a lookup. A descriptive name is itself a small
specification — the move stepwise refinement makes with function names,
applied here to the spec layer. The name follows the claim's own
subject–verb–object discipline; no check reads it, because identifiers
conjugate — *Stops*, *Refused* — and a check that admits every form of every
verb is one that fires on good names.

The objection is rename cost: rewording a claim should change its name, breaking
every citation. That's the correct behaviour: a reworded claim *should* force re-review at
each implementing site. A `#[deprecated]` alias for the old name provides the migration path. Numbers make the rename cheap by making it meaningless.

**Foreign keys are the exception.** When claims originate outside the codebase —
a compliance matrix, a customer's numbered spec document, a regulatory
requirement — the dash-case ID is a genuine foreign key:

```rust
/// The [`AuditLog`] shall retain every [`AuthAttempt`] for 90 days.
#[derive(Spec)]
#[spec("SOC2-CC6.1-003")]
pub struct AuthAttemptsAreAudited;
```

`#[spec("...")]` (spelled `#[lid_rs::spec("…")]` where the bare name would
collide with your `spec` module) emits `#[doc(alias)]` so the foreign ID stays
greppable and rustdoc-searchable. Inert unless there's an external system to key against; don't
reach for it by default.

### 3.3 Anatomy of a citation

```rust
/// Resolves credentials to a session.
///
/// ```
/// # use myapp::auth::authenticate;
/// # use myapp::test_support::{store, valid_creds};
/// let session = authenticate(&store(), &valid_creds()).unwrap();
/// assert_eq!(session.user_id(), 42);
/// ```
#[implements(
    spec::ValidCredentialsYieldScopedSession,
    spec::UnknownCredentialsAreRejected,
    spec::BackendFailureIsRejected,
)]
#[flow]
pub fn authenticate(store: &CredentialStore, creds: &Credentials) -> Result<Session, AuthError> {
    todo!()
}
```

The attribute expands to two things today, and a third once the runtime lands
([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)):

```rust
// 1. the human-facing doc line, so rustdoc renders and link-checks it
#[doc = "Implements [`crate::spec::ValidCredentialsYieldScopedSession`]."]

// 2. a link-time registry entry (see §5). The `<Path as Spec>::NAME`
//    projection doubles as the type assertion: a bad path is a compile
//    error, and a #[deprecated] spec warns right here. The `const _` block
//    scopes the static, so it needs no name-mangling and cannot collide.
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid_rs::__private::linkme::distributed_slice(::lid_rs::IMPLEMENTATIONS)]
    #[linkme(crate = ::lid_rs::__private::linkme)]
    static EDGE: ::lid_rs::Edge = ::lid_rs::Edge {
        spec: <crate::spec::ValidCredentialsYieldScopedSession as ::lid_rs::Spec>::NAME,
        item: concat!(module_path!(), "::authenticate"),
        file: file!(),
        line: line!(),
    };
};

// 3. a `tracing` span around the body, carrying the claims, each parameter
//    through its `Traceable` policy, and the outcome on exit (§6.4).
```

`Spec::NAME` (definition-site `module_path!()` plus the item's identifier) is
generated by the derive, so both sides of every registry join produce the key
from the same single source. (`core::any::type_name` cannot serve here: it is
not const-stable, so it cannot initialize a static on stable Rust.) Effect (2)
is what removes the parser: name resolution is the compiler's, so `use`
renames, re-exports, and `#[cfg]` are handled correctly and for free. It is
also what makes the graph enumerable at runtime without anyone reading source.

`#[implements]` does not inspect body shape or signature conformance; those
are the shape pass's and the conformance checks' jobs
([§4.6](https://bradvoth.github.io/lid-rs/spec/gates.html), [§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html)).
`#[flow]` is a pin, not a classification: it declares that this function is
flow and must stay so ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)).

### 3.4 Anatomy of a validation

```rust
#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn unknown_credentials_are_rejected() {
    // arrange
    let store = store_with(account("brad"));
    // act
    let outcome = authenticate(&store, &wrong_password("brad"));
    // assert — mirrors the claim's response clause
    assert!(matches!(outcome, Err(AuthError::InvalidCredentials)));
}
```

Same two emissions, registering into `VALIDATIONS`. The test is named after
the claim — the claim's name in `snake_case`, which the macro checks
(check 14) — so `cargo test` output is the requirements checklist: every line a
claim, every result a verdict. The body is arrange/act/assert with the
assertion mirroring the claim's *shall* clause, which is the one convention
no check enforces ([§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)).
With the runtime ([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)),
the test runs under a capturing subscriber whose span tree is checked against
the claim.

**These must be `#[cfg(test)]` unit tests inside the library, not files under
`tests/`** — see [§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html) for why.

**Why not doctests?** Doctests compile as separate crates and never link into the
registry, so they cannot register. Keep them — they're the best form of the
"claim and assertion in the same block of text" property, and they're excellent
public-facing documentation — but they are *examples*, not the gate.

### 3.5 The controlled claim language, and the executable lexicon

A claim is a doc comment; `#[derive(Spec)]` reads it. Left as free prose, a
claim has as many meanings as readers, and every one of the checks that read a
claim's *content* ([§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html),
[§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html)) needs the content to
have parts. The profile stacks four standards, each fixing a different kind
of ambiguity: EARS (sentence structure), RFC 2119 (modality), the INCOSE
prohibited-term list (vague vocabulary), and ASD-STE100's principle of a closed
verb set with one meaning each (permitted vocabulary — with a project-local
dictionary, since STE's own is for maintenance manuals).

**The five EARS patterns**, with the shape each predicts for the implementation
and for the validator's trace:

| Pattern | Form | Implementing shape | Validator's trace |
|---|---|---|---|
| Ubiquitous | *The X shall Y.* | a type invariant; a constructor | many short spans on the constructor |
| Event-driven | *When T, the X shall Y.* | a function: T in, Y out | one path, closes `Ok(Y)` |
| State-driven | *While S, the X shall Y.* | typestate, or a `match` on state | spans carry `state = S` |
| Unwanted | *If C, then the X shall Y.* | an error variant | one path, closes `Err(Y)` |
| Optional | *Where F, the X shall Y.* | a feature flag or enum variant | spans carry `feature = F` |

The "implementing shape" column began as a Phase 3 heuristic. Through the
verb, it is checked (check 19).

**What the derive extracts** into `SpecMeta`: pattern, modal, trigger noun (the
vocab link in the trigger clause), response verb, response object (the link in
the response clause), and the response object's *owner* when the link is a
variant path (`AuthError` from `AuthError::InvalidCredentials`).

**What the derive rejects** (check 13, a compile error at the spec struct,
naming the offence): more than one sentence, or an opener whose clause is not
closed as its pattern requires; more than one *shall* — a compound claim is two claims; a term from
the INCOSE list or the project's `prohibited.extra`; a verb absent from the
lexicon; a trigger or response slot that is not an intra-doc link to a
vocabulary type; and, for an *unwanted* claim, a condition clause with no
vocabulary link at all. SHOULD and MAY claims are permitted and tagged
non-normative; checks 10 and 11 relax for them — a MAY needs no validator.

**Condition clauses link a noun.** An *unwanted* claim's condition — *if C,
then...* — must contain at least one vocabulary link: *if the
[`CredentialStore`] is unreachable*, not *if the store is unreachable*.
Without this, unwanted claims have no extractable trigger and check 20 is
vacuous for them; with it, the noun the condition is *about* must appear in
an implementer's signature, which also forces the dependency to be explicit
rather than reached through `self`. The predicate — *is unreachable* — stays
prose; that is the residual named in [§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html).

**The lexicon** carries a definition and a signature template per verb:

```toml
# docs/intent/lexicon.toml

[verbs.return]
def       = "produce as the Ok value of the implementing function"
signature = ["-> Result<{object}, _>", "-> {object}"]

[verbs.reject]
def       = "produce as the Err value of the implementing function"
signature = ["-> Result<_, {owner}>"]

[verbs.emit]
def       = "send on the named channel or sink; no value returned"
signature = ["-> Result<(), _>", "-> ()"]

[verbs.retain]
def       = "keep available for at least the stated duration"
signature = "*"                     # constrains behaviour, not shape

[verbs.equal]
def       = "compare as identical under PartialEq"
signature = "*"

[verbs.log]
def       = "write to the audit log at the stated level"
signature = ["-> Result<(), _>", "-> ()"]

[prohibited]
extra = ["promptly", "gracefully", "robust"]
```

A template is matched against the return type's tokens with `{object}` bound to
the response object's type name and `{owner}` to the owning enum. `_` matches
any single type; `*` means the verb constrains behaviour rather than shape. The
match is syntactic — names, not resolution — and deliberately shallow:
`Result<Session, AuthError>` matches `-> Result<{object}, _>` for `Session`;
`Option<Account>` matches nothing a *reject* verb allows.

The definitions were always operational — *reject = produce as the Err value* —
and the template is that definition written down mechanically. A verb whose
meaning can't be expressed as a shape gets `*`; that's the honest marker of a
verb that only a validator can check. The derive `include_str!`s the lexicon
so Cargo tracks it, and the derive is where the lexicon is *executed*: a
claim using a verb the lexicon doesn't define is a compile error, so adding a
verb means defining it.

Six verbs will not survive contact with a real domain, and the list will feel
restrictive during the first slice. The discipline isn't the small list — it's
that a new verb arrives with a definition, and a definition arrives with the
shape it implies.

FRETish (NASA's EARS-shaped grammar that compiles to temporal logic) is the
upgrade path if timing or liveness claims are ever needed; the profile above is
a strict subset of it.

### 3.6 Vocabulary: nouns become types; the LLD links to them; flow speaks only in them

Every LLD has a vocabulary — session, credentials, verifier, schedule. Each
becomes a newtype in the slice's `vocab` module, with a doc comment that is its
definition:

```rust
// src/auth/vocab.rs
#[derive(Traceable)]                   pub struct Credentials { username: Username, secret: Secret }
#[derive(Traceable)] #[trace(redact)]  pub struct Secret(zeroize::Zeroizing<String>);
#[derive(Traceable)]                   pub struct Session { /* ... */ }
```

The LLD is then written with intra-doc links — [`Credentials`] rather than
"credentials" — and because the LLD is included via `include_str!`, rustdoc
resolves those links (check 2). **The design document breaks if the code is
renamed.** Click any noun in the design and land on its definition;
signatures read as domain sentences — `verify_password(&Account, &Secret)`
needs no doc to be understood. `Traceable` declares each noun's logging
policy where the noun is defined ([§6.5](https://bradvoth.github.io/lid-rs/spec/traced.html)).

**Rule V — a flow node's signature is made of vocabulary.** Parameter types
and the `Ok` type may be vocab types, `Outcome` enums, or those wrapped in
`&`, `&mut`, `Option`, `Result`, `Vec`, `Box`, `Arc`. Not `String`, `&str`,
`bool`, integers, floats, `char`. Leaves are unconstrained.

Two reasons. A `String` at a flow boundary could be anything; a `Username` is a
noun the LLD defined and a product manager can read. And `Traceable` has impls
for primitives, so a `String` parameter *compiles and gets recorded* — as text
with no policy. Rule V is what guarantees every field on a flow span is a noun
whose policy the glossary chose.

### 3.7 Flow and leaf, derived from shape

§0's rule — dispatch and work are separate functions — needs a definition of
*dispatch* that a tool can apply, and one that nobody can opt out of: a marker
attribute that declares "this function is routing" is evaded by omitting it.
So the classification is derived from the body, and the markers only pin it.

A function is **flow** iff its body satisfies F1–F6:

- **F1** every statement is `let pat = call(...)?;`, `call(...)?;`, or the tail call;
- **F2** at most one decision structure — one `match`, or one two-way `if` — and every arm is a single call;
- **F3** arguments are paths, field accesses, references, or accessor chains;
- **F4** no closures or blocks;
- **F5** no literals but `()`;
- **F6** no macros but those in `shape.allow_macros` (`todo!` and `unimplemented!` by default, so a skeleton is flow).

Otherwise it's a **leaf**. A flow body is the *no-descent test* made
mechanical: it is readable without opening any callee — `load the account,
verify the password, issue a session` — so a reader descends only for the
*how*, never to learn the *what*. That is the boundary at which a product
manager can stop reading ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)).

Rules over the classification, none opted into:

- **A — dispatch is pure.** A function that routes among three or more kinds
  (`shape.dispatch_arms`) must be flow. A flat twelve-arm `match` passes the
  complexity lint and is still an impure dispatcher if an arm does work.
- **B — the public surface is prose.** Every `pub fn` in a slice's `mod.rs` is
  flow, or carries `#[leaf]`, which is counted and reported — a public leaf
  is allowed, and visible.
- **P — pins hold.** A `#[flow]` function that acquires work fails. The pin is
  how a skeleton's author says "this stays routing" and how a reviewer reads
  the role before the body.
- **V — flow signatures are vocabulary** ([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)).
- **C — the complexity threshold is uniform.** Flow nodes have one decision
  structure by F2, so the same `cognitive-complexity-threshold` applies to
  every function; the threshold is a knob, and lowering it is the ramp
  ([§7](https://bradvoth.github.io/lid-rs/spec/configuration.html)).

The shape pass is a `syn` pass over the source of one crate — permitted by
constraint 2 because it resolves nothing: F1–F6 are token shapes, and rule V's
allowed types are names, not resolved paths. It runs as a test
(`intent_graph!()` emits it) and standalone as `cargo lid-rs shape`, at
`shape.level`; `warn` counts are committed in `trace.md`
([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html)) so the ramp is
visible in the PR diff, and flipping a slice to `deny` is a one-line change
once its count has been zero for a while. A warning is a ramp, not a resting
state.
<!-- ANCHOR_END: mapping -->

---

<!-- ANCHOR: gates -->
## 4. The twenty-six checks

Twenty-six checks in two tiers; run in order they form *the gate* ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html)).
Both tiers gate; both run on stable; neither resolves a name from source.
Checks 1–12 run today; 13–26 are specified here, numbered by seam so that no
existing citation of a check moves, and land in the slices the workspace HLD
names. [§12](https://bradvoth.github.io/lid-rs/spec/limits.html) keeps the
ledger of which are built.

### 4.1 Tier 0 — compiler, rustdoc, clippy

| # | Check | How | Failure means |
|---|---|---|---|
| 1 | **Unresolvable citation** | `cargo check` (const assertion) | Code cites a spec that was renamed or deleted. Also warns for `#[deprecated]` specs. |
| 2 | **Broken doc link** | `cargo doc` with `-D rustdoc::broken_intra_doc_links` | A hand-written link in an LLD or doc comment dangles — an LLD noun, a claim's vocabulary link, a citation's doc line. Redundant with check 1 on purpose; this one also covers hand-written links. |
| 3 | **Undocumented item** | `missing_docs`, `clippy::missing_docs_in_private_items` | An item exists with no stated intent. |
| 4 | **Skeleton incoherence** | `cargo check` at every refinement layer | The layer you just designed doesn't fit together — wrong error type, unsatisfiable borrow, missing lifetime. Caught before implementation. |
| 5 | **Broken example** | `cargo test --doc` | A public-facing example no longer reflects the API. |
| 6 | **Incomplete cascade** | `#[non_exhaustive]` + exhaustive `match` + `wildcard_enum_match_arm` | A new case was added upstream and a dispatch site swallowed it under `_ =>`. |
| 7 | **Undeclared decision** | `clippy::cognitive_complexity` | A function does dispatch *and* work, or contains a branch that never appeared in the design. **The structural drift detector.** |
| 8 | **Flag argument** | `clippy::fn_params_excessive_bools`, threshold 0 | A `bool` parameter is a branch smuggled into a leaf — two functions in a trench coat. |
| 9 | **Inlined concept** | `clippy::too_many_lines` | A coherent sub-thought was manually inlined instead of being named. |
| 13 | **Malformed claim** | `#[derive(Spec)]` ([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)) | Compound sentence, unclosed pattern clause, prohibited term, unknown verb, unlinked noun, unlinked condition. |
| 14 | **Misnamed validator** | `#[validates]` + `cargo check` (const assertion) | The test's name is not its claim's name in `snake_case`. |

Plus the unnumbered type-system property: an untraceable noun at a traced
boundary fails to compile ([§6.5](https://bradvoth.github.io/lid-rs/spec/traced.html)).

Check 7 does the most work: it is §0's dispatch/work rule made mechanical.
A leaf whose complexity exceeds the threshold is *either* an undeclared dispatch node *or* a
requirement that was never written down — an agent making a judgment call you
never saw. Cognitive complexity is the right metric and cyclomatic is not,
because the cognitive metric doesn't penalise flat `match` arms: a twelve-arm
dispatch is fine; three nested `if`s in a leaf is not. What check 7 cannot
see — a flat dispatcher whose arms do work — is rule A's job (check 15).

> **Toolchain notes.** Intra-doc links are resolved by rustdoc, not `cargo
> build`, so check 2 needs its own `cargo doc` step. Doctests only run for
> library targets, so structure the project as a thin `bin` over a `lib`.

### 4.2 Tier 1 — registry intersection

Enumeration happens at link time ([§5](https://bradvoth.github.io/lid-rs/spec/registry.html)). These are ordinary unit tests,
and `lid-rs` ships them: invoke `lid_rs::intent_graph!()` in a `#[cfg(test)]` module
of the library, and the checks below are emitted — crate-scoped, canary-first,
plus an inert registry-dump test the mutation tool reads ([§4.3](https://bradvoth.github.io/lid-rs/spec/gates.html)).
Hand-writing them invites drift in the one place drift-detection lives.

| # | Check | Failure means |
|---|---|---|
| 10 | **Uncited spec** | A normative claim nothing implements. The design says it; the code doesn't do it. |
| 11 | **Unvalidated spec** | A normative claim no test cites. Nothing would notice if it broke. |
| 26 | **Stale trace** | `docs/intent/trace.md` doesn't match the registry and the shape pass ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html)). |

```rust
// what lid_rs::intent_graph!() expands to at its invocation site in lib.rs
use lid_rs::{SPECS, IMPLEMENTATIONS, VALIDATIONS};

#[test]
fn registry_is_populated() {
    // Constraint 3's corollary: prove the enumeration exists before
    // asserting anything over it. See §5.3.
    assert!(lid_rs::canary::present(), "registry empty — see §5.3");
}

#[test]
fn every_spec_has_an_implementer() {
    let implemented: HashSet<_> = IMPLEMENTATIONS.iter().map(|e| e.spec).collect();
    let orphans: Vec<_> = SPECS.iter()
        .filter(|s| !implemented.contains(s.name))
        .map(|s| format!("{} ({}:{})", s.name, s.file, s.line))
        .collect();
    assert!(orphans.is_empty(), "specs with no implementation:\n{orphans:#?}");
}

#[test]
fn every_spec_has_a_validation() { /* same shape against VALIDATIONS */ }
```

Note there is no check for a *function with no citation*. That is a deliberate
design decision, not a gap — see [§6](https://bradvoth.github.io/lid-rs/spec/traced.html).

> **Scoping note** *(uses the linking model of [§5](https://bradvoth.github.io/lid-rs/spec/registry.html) and "traced" from [§6](https://bradvoth.github.io/lid-rs/spec/traced.html);
> return here after those if it reads dense).* The registry is binary-global:
> a consumer's test binary links `lid-rs` (and any other traced crate, meaning
> any crate carrying citations), so `SPECS` contains upstream
> claims whose `#[validates]` edges — being `#[cfg(test)]` in their home
> crates — are absent from this binary. The checks therefore scope to specs
> whose `NAME` begins with the current crate's name; edge sets stay
> unfiltered.

> **Registered but uncitable.** `#[derive(Spec)]` registers a claim whether or
> not the `spec` module re-exports it, and a `mod <slice>;` there is private.
> A claim written and not added to `pub use <slice>::{…}` fails twice from one
> omission: check 10 reports it, and no `#[implements]` can name it because
> `spec::<Name>` does not resolve — a Phase 3 stop that costs a Phase 2
> re-run to diagnose. The derive therefore also emits a const assertion
> through `crate::spec::<Name>`, so the omission fails at the spec's own
> definition site, where it was made.

### 4.3 Tier 1 — non-vacuity by scoped mutation

`#[validates]` proves a test *claims* a spec. It cannot prove the test would
notice if the implementation were wrong.

Phase 5 of the flow ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)) already solves this by hand: a `todo!()` body panics, so any test that
exercises the cited function must fail against the skeleton. "Confirm
red" is a proof of non-vacuity performed when it costs nothing. The problem is
that it's a human ritual done once, and nothing preserves it.

| # | Check | Failure means |
|---|---|---|
| 12 | **Vacuous validation** | Mutating an `#[implements]` function does not fail the tests that `#[validates]` its specs. The arrow is decorative — citation resolves, test passes, test does not depend on the code. |

`cargo-mutants` substitutes plausible return values for function bodies; a test
surviving that mutation is not asserting anything about the function. Strictly
stronger than line coverage, which counts incidental execution as evidence.
Deterministic, stable toolchain, no instrumentation.

**Which claims an item cites decides what check 12 can catch.** The check
narrows a mutant's test set through the registry to the validators of the
claims the mutated item cites, so a citation set that is *too narrow* hands a
mutant a plan of tests that cannot distinguish it: an item deciding between two
claims' worlds while citing one of them survives mutation with its other claim's
test never run. The rule:

> An item cites a claim when a **wrong answer from that item could make the
> claim false** — not when the claim's validator merely executes it.

It cuts both ways. Too broad is decorative in the same sense and worse for
being silent: a citation no wrong answer can contradict adds a test to a plan
that was never going to fail. An item on a claim's execution path that cannot
answer wrongly for it does not cite it.

And check 12's silence is not evidence. `cargo-mutants` generates no mutant for
some items — an enum with no `Default`, for instance — so an uncontradictable
citation there is never reported. When an item's citation cannot be
contradicted and no mutant exists to prove it, the honest reading is that the
citation is decorative or a claim is missing from the design, not that the
arrow holds.

Two narrowings compose to keep it inside a per-PR budget:

- `--in-diff` mutates only functions the PR touched.
- The registry supplies the test subset: for a mutant in function `F`, run the
  tests validating the specs `F` implements; for a mutant in a flow node, every
  validator downstream of it, which the shape pass's classification supplies.
  The tool obtains each crate's
  `IMPLEMENTATIONS`/`VALIDATIONS` from that crate's **own `--lib` test
  binary** — a dump mode emitted by `intent_graph!()` — because validation
  edges exist in no other binary ([§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html) applies to the tooling too). A mutant
  whose narrowed test set is empty runs the full suite instead: zero reachable
  tests must never mean zero tests run.

Set `[profile.test] opt-level = 0` so inlining can't erase a mutation site. The
registry-driven test filtering is the one piece that has to be built; the rest is
off-the-shelf.

**What it costs, measured.** Check 12 dominates the gate: on this workspace,
about seventeen minutes at 51 mutant groups and an hour at 125, and the
per-mutant cost is the *tests*, not the build (≈0.3 s to build, ≈4 s to run,
with an unset timeout defaulting to 300 s so one pathological mutant costs
five minutes). Two things bound it: the engine must be scoped by *file* as
well as by function name — its name filter alone lets every "delete field"
mutant of the crate ride along in every run, which was 62% of the work — and
any check that runs longer than an unattended agent's watchdog cannot run
inside that agent's session at all ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)).

### 4.4 What remains uncaught

Two residuals, named precisely.

**The condition clause.** Every extractable part of a claim is checked
somewhere: pattern (24), trigger noun (20), verb (19), response object
(19, 21, 24), owner (22). The one part that isn't is the *predicate* of an
unwanted claim's condition — *is unreachable*, *matches no stored account*.
The condition must link a noun ([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)),
so its *subject* is structural; its *predicate* is prose. A wording change from
*matches no stored account* to *matches a locked account* keeps the noun, the
pattern, the verb, and the variant, and no check fires. It does appear in the
committed `trace.md` diff, in the same row as the claim's validators, which is
where a reviewer is looking — but it is a reviewer's catch, not a gate's.

**Assertion meaning.** A validator that reaches its claim, produces the promised
outcome, is non-vacuous under mutation, and whose implementer's signature keeps
the promise — and whose `assert!` still checks something the claim didn't mean.
Checks 24 and 12 together make this narrow: 24 is itself a runtime assertion
on the response object, and a weak human assert leaves the validator still
killable through 24, so 12 is satisfied honestly. What survives is an assert
that is *consistent with* the claim without *being* it. Until the runtime
lands, the whole of this residual is the older, wider form: a test that cites
the *wrong* spec — mechanically sound, semantically misaligned.

Both are the bidirectional-differential pass's job — reconstruct the claim
from the code in a fresh session and diff it against the written one — and
both are small enough that the pass can be pointed at exactly them: condition
predicates, and assertion bodies. Run it periodically, not per-PR. Per
[§1.2](https://bradvoth.github.io/lid-rs/spec/purpose.html), this being the
only residual is the system working as designed.

Two candidate checks were evaluated and dropped. *Assertion mentions the
response object* (syntactic) is redundant with 24, which asserts it at
runtime. *Claim edit requires a validator touch* (diff-scoped) is satisfied by
whitespace and adds nothing over the `trace.md` diff for the one case the
structural checks miss. Neither would have moved the residual; both would have
added a gate. Constraint 3 cuts both ways.

### 4.5 The gate, in order

```bash
cargo check --all-targets                        # 1, 4, 13, 14, Traceable
cargo clippy --all-targets -- -D warnings        # 3, 6, 7, 8, 9
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" \
  cargo doc --no-deps                            # 2
cargo test --doc                                 # 5
cargo test --lib                                 # 10, 11, 15–26 + behaviour
cargo package -p <crate> --allow-dirty           # published crates: the tarball
                                                 #     builds standalone
cargo lid-rs sync --check                        # the skill matches the lid-rs
                                                 #     the project depends on
cargo lid-rs mutants                             # 12 (scope from metadata;
                                                 #     --full / --diff-base override)
```

Cheapest and most specific first. Mutation runs last because it's the only step
that rebuilds. `cargo package` applies to crates that publish: it builds the
tarball without the workspace around it, which is what catches an `include_str!`
reaching outside a package root ([§11](https://bradvoth.github.io/lid-rs/spec/layout.html)).
`cargo lid-rs sync --check` fails when any synced artifact — the skill
directory, the phase agents, the workflow — is not byte-identical to what
the `lid-rs` the project resolves ships ([§13](https://bradvoth.github.io/lid-rs/spec/bootstrap.html), keeping current). This list is the floor; a workspace appends
build-integrity steps of its own after it, and every copy of the list a
project keeps must match. Each line is one command of the pipeline's catalog,
which fixes its inputs, its report, and its exit code; the pipeline document
(`lid-rs-pipeline`) is where the catalog lives.

### 4.6 Tier 1 — shape (at `shape.level`)

The rules of [§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html),
as checks. Emitted by `intent_graph!()`, and available standalone as
`cargo lid-rs shape`, which builds nothing and answers in about a second.

| # | Check | Failure means |
|---|---|---|
| 15 | **Impure dispatch** (A) | Routes among three or more kinds and also does work. |
| 16 | **Opaque public surface** (B) | A slice exposes a non-prose function without `#[leaf]`. |
| 17 | **Broken pin** (P) | A `#[flow]` function has acquired work. |
| 18 | **Primitive at the boundary** (V) | A flow signature uses `String`, `bool`, `u64`... where a noun belongs. |

### 4.7 Tier 1 — conformance (at `conformance.level`)

These read the claim's content — `SpecMeta`, extracted by the derive at compile
time — and compare it to signatures and enums found syntactically by the shape
pass. They're the first checks that use what a claim *says*, not just that it
exists.

| # | Check | Failure means |
|---|---|---|
| 19 | **Return shape contradicts the verb** (S1) | An implementer's return type matches none of the verb's templates. *Shall reject with `AuthError::X`* implemented as `-> Option<Account>` fails here. |
| 20 | **Trigger absent from the entry** (S2) | For a claim with a trigger or condition clause, no implementer has the linked noun in its parameter types. The claim says *when `Credentials`...* and nothing that implements it accepts one. |
| 21 | **Unwanted claim names a non-variant** (E1) | An *unwanted* claim's response object isn't a variant of an `#[derive(Outcome)]` enum. The claim promises an error that doesn't exist as one. |
| 22 | **Variant with no claim** (E2) | A variant of a claim-owning error enum is the response object of no *unwanted* claim. **Someone added a way to fail that no requirement asked for.** |

**On S1's scope.** Every implementer of a claim is checked, not just the entry
point. This is right because the verb describes the claim's response, and every
function on the claim's path must be able to carry that response: `load_account`
and `verify_password` both implement *reject with `AuthError::...`* and both
return `Result<_, AuthError>`. A leaf on that path returning `Option<_>` would
have to be adapted at the call site, which is exactly the kind of silent
conversion S1 exists to surface.

**On E2's scope.** It applies to enums that own at least one claimed variant —
`AuthError`, not `StoreError`. An infrastructure error wrapped inside a claimed
variant is not itself a taxonomy the requirements are responsible for. A
brand-new enum with no claims at all can't reach a traced signature without
failing S1 first.

**Why E2 matters more than it looks.** An error variant *is* a requirement about
what can go wrong. Letting engineers add them freely means the failure taxonomy
grows by improvisation, and each unclaimed variant is a user-visible behaviour
nobody specified. E2 makes the error enum something the claims imply, which is
what it should have been all along. It also catches the lockout example
([§9](https://bradvoth.github.io/lid-rs/spec/example-login.html)) at
`cargo test --lib`, before mutation and before the page.

**Mechanics.** `#[derive(Outcome)]` — which every error type at a traced
boundary must have anyway, for span recording — also registers each variant into
`OUTCOMES`. So E1 and E2 are registry intersections, no syntax needed. S1 and S2
join `SpecMeta` with the shape pass's `signatures()`, which returns each
function's parameter and return type tokens.

**Known false positives**, hence the severity level: type aliases
(`type AuthResult<T> = Result<T, AuthError>`) and `impl Trait` returns don't match
templates. Both are loud, not silent. `[workspace.metadata.lid_rs.conformance]
aliases` lets you declare `AuthResult = "Result<_, AuthError>"` and move on.

### 4.8 Tier 1 — runtime (always an error)

These read the span tree a validator captured
([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)).

| # | Check | Failure means |
|---|---|---|
| 23 | **Unreached claim** | The validator's trace has no span citing its claim. |
| 24 | **Wrong outcome** | The citing spans don't exhibit the pattern's required trace (table below). |
| 25 | **Unreached flow node** | Some flow node appears in no validator's trace. The one check that needs the union of all traces, so only the full `--lib` run can evaluate it. |

**What check 24 requires, per pattern.** The two patterns that describe *many*
executions rather than one are held to many:

| Pattern | Required across the validator's citing spans |
|---|---|
| Event-driven | at least one closes `Ok(<response type>)` |
| Unwanted | at least one closes `Err(<response variant>)` |
| Optional | at least one carries `feature = <F>` |
| Ubiquitous | **at least `min_inputs` spans with distinct recorded inputs, none panicking** |
| State-driven | **at least two distinct `state` values — a transition actually occurred** |

The ubiquitous row is what "property test" means operationally: many inputs, the
invariant holds in each. One hand-picked `Username::new("brad")` does not
validate a length invariant. It's checked on the trace, not the syntax, so it's
indifferent to whether the inputs came from `proptest`, `quickcheck`, or a
hand-written loop — and the inputs are already there, because `Traceable`
recorded the constructor's parameter on every span. `min_inputs` defaults to 16
under `[workspace.metadata.lid_rs.runtime]`.

Check 24 and check 19 are the same promise seen twice: 19 says the signature
*can* produce `Err(InvalidCredentials)`; 24 says the validator *did*. A claim
that passes 19 and fails 24 has a correctly-typed implementation that never
takes the promised path under test.

The ubiquitous row is also the answer to a gap Phase 5 found in practice: a
claim implemented only by data — a `const`, an enum, a `match` arm returning a
value — is green the moment the skeleton lands, so the red run has nothing to
fail against. Such a claim is ubiquitous by pattern, and its proof is check
24's many-inputs rule, not a panic.
<!-- ANCHOR_END: gates -->

---

<!-- ANCHOR: registry -->
## 5. The registry mechanism

The registry is how the intent graph becomes enumerable: every spec, citation,
validation, and — with the error taxonomy — every claimed error variant,
collected where checks (10, 11, 21, 22 among them) can iterate
them. Doing that without parsing source (constraint 2) means collecting
them at link time.

### 5.1 How it works

`linkme` uses linker sections. `lid-rs` declares the slices:

```rust
#[distributed_slice]
pub static SPECS: [SpecMeta] = [..];
#[distributed_slice]
pub static IMPLEMENTATIONS: [Edge] = [..];
#[distributed_slice]
pub static VALIDATIONS: [Edge] = [..];
#[distributed_slice]
pub static OUTCOMES: [Variant] = [..];     // registered by #[derive(Outcome)]
```

Each macro expansion emits a `static` placed into a specially-named section. The
linker gathers all statics in that section contiguously and emits start/end
symbols; dereferencing the slice yields everything between them. It's a
contiguous array baked into the binary — no initialization, no runtime cost, no
ordering guarantee. `SpecMeta` carries the claim's extracted parts
([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)) alongside its
name and location.

**`linkme` is an implementation detail.** Users depend on `lid-rs`; `lid-rs` depends on
`linkme`. Expansions can't emit `::linkme::...`, since that path only resolves if
the user happens to have linkme as a direct dependency, so `lid-rs` re-exports it:

```rust
#[doc(hidden)]
pub mod __private {
    pub use linkme;
}
```

The re-export alone is not sufficient: linkme's *own* element expansion also
emits `linkme::…` paths. Every generated registration therefore carries
`#[linkme(crate = ::lid_rs::__private::linkme)]`, linkme's wrapper-crate
override, which redirects those paths through the re-export.

Generated statics are scoped inside `const _` blocks and carry
`#[allow(missing_docs, clippy::missing_docs_in_private_items)]` — without the
allows, check 3 fires on every citation in the crate, catching code the user
didn't write.

### 5.2 Consequence: test placement is not free

Registrations appear in a binary only if the crate containing them was linked
into it. **Each file under `tests/` compiles as a separate binary.** If
`#[validates]` tests live in `tests/auth.rs` and the graph check lives in
`tests/intent_graph.rs`, the graph check sees zero validations and passes
vacuously.

So `#[validates]` tests must be `#[cfg(test)]` unit tests inside the library,
giving one binary from `cargo test --lib` that contains both the registrations
and the checks over them. This is independently correct — leaves are private, and
unit tests can reach them.

Integration tests under `tests/` are still fine for what integration tests are
for. They just can't carry `#[validates]`.

### 5.3 The canary

Constraint 3's corollary applies here. If LTO, `--gc-sections`, or an
unusual target strips the section, the registry is empty and
`every_spec_has_an_implementer` passes trivially over nothing. Green build, zero
enforcement — the exact failure mode this system exists to prevent, reintroduced
by its own mechanism.

`lid-rs` ships a known spec/implementation/validation triple internally.
`lid_rs::canary::present()` returns false if any of them is missing from the
registries. Every registry-based check asserts it first. A stripped section
becomes a named failure instead of an inferred pass.

### 5.4 Where the mechanism leaks

- **Compile errors.** A malformed registration surfaces as a linkme diagnostic
  pointing at expanded code. Mitigate by validating aggressively in the macro and
  emitting `compile_error!` with a LID-rs message before linkme sees bad input.
- **Target support.** On an exotic target the section trick fails. Ship
  `inventory` behind `features = ["inventory"]` — it registers via
  life-before-main constructors, costs a little startup time, and changes only
  macro output, so the escape hatch is a flag rather than a rewrite.
- **LTO and `--gc-sections`.** Aggressive linker settings can strip the section.
  The canary catches it; the fix is a documented profile setting.
- **Dependency audits.** It appears in `cargo tree` and `cargo deny` output.
  Fine — just don't let anyone be surprised by it.
<!-- ANCHOR_END: registry -->

---

<!-- ANCHOR: traced -->
## 6. Traced and untraced code

Not every function should carry a citation. A parsing helper, a `Display` impl, a
private conversion — forcing claims onto those produces ceremonial specs, which
are worse than none because they make the graph look denser than it is.

**Definition: an untraced function is a leaf helper with no spec-governed
behaviour.** If it participates in spec behaviour, it wasn't one.

### 6.1 What holds the line

**Mutation answers the question empirically.** If mutating an untraced function
kills a `#[validates]` test, that function is causally on a spec-governed path
and should carry a citation. If nothing fails, it isn't participating, and
doesn't need one. Check 12 already produces this signal:

- *surviving mutant on untraced function* → not participating, correctly untraced
- *killed mutant on untraced function* → trace it, or move it behind a traced
  boundary

For untraced mutants, the tool can't narrow the test set through the
registry, so it runs the tests validating whatever specs are implemented in
the same file (the mutant list carries no module path), falling back to the
full suite when the file implements none. New untraced code is therefore
gated: it may exist only if it either breaks nothing when mutated, or is
traced.

**Rust's privacy rules do the containment.** The real risk isn't untraced code calling
traced code — it's untraced code *bypassing a dispatch node* to reach a leaf
directly. Rust prevents that already: `apply_display_name` is private, so only
its module can reach it, and that module carries the slice's claims. The
boundary that bites is `pub`, and public functions should carry citations
anyway.

**The complexity threshold bounds what an untraced function can be.** At the
threshold it contains no undeclared decision, so it cannot harbour an
undeclared requirement by construction. The explosion worth fearing is
untraced *decisions*, and check 7 catches those regardless of tracing; the
shape rules ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html))
apply to every function regardless of tracing too.

**Conformance bounds its shape.** An untraced function can't *return* into a
claimed path with the wrong shape, because S1 (check 19) checks every
implementer's return type against the verb — so a leaf that participates has
to conform to the claim it serves even before it's cited.

### 6.2 Why not a syntactic call rule

The tempting rule is "an untraced function may not call a traced one" —
traced → untraced is fine and expected; untraced → traced is the suspicious
edge. But the strict version collapses. `#[implements]` becomes viral upward —
every caller of traced code must be traced, so `main` calls the composition
root calls the entry point, and `main` ends up carrying every spec in the
crate. Any workable version needs a bound, and needs an exemption list:
`main`, `#[cfg(test)]` fixtures, `From`/`Display` forwarding impls. It's
trivially dodged: `retry(|| authenticate(&c))` calls nothing traced —
the closure does. And enforcing it needs a call graph, meaning source parsing
or rustc internals: both constraints violated, for a rule that leaks anyway.
Mutation's participation test ([§6.1](https://bradvoth.github.io/lid-rs/spec/traced.html)) is stronger on both counts: a
function can call traced code incidentally without carrying its behaviour,
and can carry spec behaviour through a closure no call rule would see. And
the call graph the rule would need exists after all — observed, not
resolved ([§6.6](https://bradvoth.github.io/lid-rs/spec/traced.html)).

### 6.3 Module-level tracing

For a cluster of private helpers implementing one claim between them, a
module-level invocation traces by containment rather than per-function
ceremony:

```rust
// src/auth/password.rs
lid_rs::implements_module!(spec::PasswordsAreVerifiedInConstantTime);
```

(A function-like macro rather than an inner attribute, because custom inner
attributes are not stable Rust; the emitted edge is identical, with
`module_path!()` supplying the containment.) Every function in the module
inherits the citation. Use this for a slice's
private machinery; use per-function citations at the public surface where
precision matters.

### 6.4 Runtime: claims as spans

Everything above is static: the graph is real at compile time and in CI.
Claims as spans make it real at runtime — every execution of the code is a
trace of which requirements were satisfied, in what order, with what outcome.

`#[implements]` wraps the body in a `tracing` span (`target = "lid"`, level
configurable) carrying `lid.claims`, each parameter through its `Traceable`
policy, and `lid.outcome` recorded on exit through the return type's
`Outcome` impl. `async fn` delegates to `#[tracing::instrument]`, so span
propagation across awaits is the ecosystem's, not ours.

`#[validates]` installs a capturing subscriber for the test's duration,
captures the span tree, and prints it on failure — the test's output *is* the
requirements it exercised, and the flow they took:

```
unknown_credentials_are_rejected                       UnknownCredentialsAreRejected
└─ authenticate                            [flow]      store = CredentialStore, creds = { username: "brad", secret: [redacted] }
   ├─ load_account                                     → Ok(Account)
   └─ verify_password   UnknownCredentialsAreRejected  → Err(InvalidCredentials)   ✓ promised
```

Checks 23–25 ([§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html)) read
the tree. The pattern predicts the trace's shape
([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)'s last column):
ubiquitous claims on constructors are exercised transitively by the whole
suite, and their proof is many distinct inputs rather than a panic — which is
why a claim implemented by data alone, which can never go red in Phase 5,
still has a check that can fail.

### 6.5 `Traceable`: the logging policy lives with the noun

A traced boundary records its parameters. What is recorded is decided where
the noun is defined, not at every call site: `#[derive(Traceable)]` with
`#[trace(redact)]`, `#[trace(skip)]`, or a field selection. The trait bound
on traced parameters enforces the policy at compile time — an untraceable
noun at a traced boundary does not compile — and rule V
([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)) guarantees every
field on a flow span is a noun with a policy, since primitives have
`Traceable` impls with none.

### 6.6 The observed call graph

The union of validator traces is the call graph — the one constraint 2 said
could not be built statically. It is exact for the paths the validators took,
shows untraced callees, and needs no resolution. Check 25 reads it for flow
nodes nothing reached; the slice page ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html))
renders it.

### 6.7 Production: the same spans, level-gated

Nothing about the span is test-specific. In production the spans are emitted at
`runtime.level` to whatever subscriber the application installs, so claim
names are the observability vocabulary: an alert is per claim, a dashboard is
a slice, and an incident is described in the LLD's language rather than in the
engineer's. The analogue of a gate here is an alert; a claim whose promised
outcome stops occurring is a page.

### 6.8 Async

The macros handle span propagation across `await` and use a root-keyed global
capturing layer rather than a thread-local subscriber, so a validator's tree is
keyed by the test that opened it. An uninstrumented `spawn` orphans its spans
and check 23 fails loudly; `lid_rs::spawn` is the convenience that keeps them.
<!-- ANCHOR_END: traced -->

---

<!-- ANCHOR: configuration -->
## 7. Configuration

Three files, each the canonical home for one kind of setting. None of it lives
scattered through source attributes.

**`Cargo.toml`** — lint levels, workspace-wide (stable since 1.74), and
LID-rs's own knobs:

```toml
[workspace.lints.rust]
missing_docs = "deny"

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"

[workspace.lints.clippy]
cognitive_complexity        = "warn"   # nursery — must be opted in
fn_params_excessive_bools   = "warn"
too_many_lines              = "warn"
wildcard_enum_match_arm     = "deny"
missing_docs_in_private_items = "warn"

[workspace.metadata.lid_rs]
mutation_scope = "diff"          # diff | full
lexicon        = "docs/intent/lexicon.toml"
trace          = "docs/intent/trace.md"
untraced_fallback = "module"     # which tests a mutant in untraced code runs

[workspace.metadata.lid_rs.shape]
level = "warn"                   # warn | deny — the ramp
allow_macros = ["todo", "unimplemented"]
dispatch_arms = 3                # rule A: routing among this many kinds must be pure
wrappers = ["&", "&mut", "Option", "Result", "Vec", "Box", "Arc"]   # rule V

[workspace.metadata.lid_rs.conformance]
level = "warn"
aliases = { AuthResult = "Result<_, AuthError>" }

[workspace.metadata.lid_rs.runtime]
level = "debug"                  # span level in production
capture = "target/lid/traces"    # where validators write their trees
min_inputs = 16                  # check 24, ubiquitous: distinct inputs required
```

Each member crate opts in with:

```toml
[lints]
workspace = true
```

**`clippy.toml`** — thresholds, since clippy reads numeric configuration only
from here:

```toml
cognitive-complexity-threshold = 4
too-many-lines-threshold       = 40
max-fn-params-bools            = 0
```

**`[workspace.metadata.lid_rs]`** — LID-rs's own knobs, read by the tool. The
`level` knobs are ramps: a brownfield crate starts at `warn`, commits the
counts in `trace.md`, and flips to `deny` when a slice's count has been zero
for a while ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)).

`cognitive-complexity-threshold = 4` is a starting point, not scripture. The
shape rules make flow nodes hold exactly one decision structure, so the
threshold the design implies is 1 — but this workspace's own code raised six
hundred warnings at 1 before the shape pass existed to say which were flow
nodes, so the number is a measurement to make, not a value to assert. If
you're raising it, check whether the code is irreducible or whether writing
the claim ([§0](https://bradvoth.github.io/lid-rs/spec/premise.html)'s rule) was merely inconvenient.
<!-- ANCHOR_END: configuration -->

---

<!-- ANCHOR: flow -->
## 8. The flow

Nine phases, 0 through 8: Phases 0–7 deliver a slice, and Phase 8 is the
change loop that re-enters them. Human authorship concentrates in 1 and 3;
agent effort concentrates in 6. Phases 1–3 produce artifacts a product manager
can read and sign — the LLD, the claims in the controlled language, and a
skeleton whose flow bodies read as prose; Phases 4–6 are engineering-owned.
The boundary sits at the dispatch layer because that is where the question
changes: everything above it answers *what* and *where*, everything below it
answers *how*.

**Phase 0 — Name the slice.**
A user-visible operation, not a component. "User logs in", not "auth module".

**Phase 1 — Write the LLD.** *(human, agent drafts)*
Plain English in the slice's `lld.md`, wired into the module docs, its nouns
written as vocabulary links so the design is hyperlinked to the code from the
first commit ([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)). This
is the layer that cannot be recovered from code — rationale, rejected
alternatives, invariants that aren't type-expressible. Your time goes here.

**Spike before you specify a mechanism.** An LLD may assert what the *system*
shall do on the author's judgment; it may not assert what a *tool* can do. A
claim that a macro can read something, that the compiler will reject something,
that a test harness can reach something — anything whose truth belongs to a
toolchain rather than to the design — is not written until a throwaway spike
has compiled and proved it. The spike is deleted; only what it established
enters the document. This is the cheapest rule in the flow and it was learned
the expensive way: of the thirteen amendments the controlled-language slice's
LLD took after its draft, four were mechanism assumptions that a few lines of
`rustc` would have refuted before they were written, and one of those stood for
a week as a design dilemma that dissolved in four lines.

**Read the LLD adversarially before committing it.** `cargo lid-rs lld-check`
gates the document's mechanical properties; the advisory reviewer reads it for
what the mechanics cannot see — a section still describing a mechanism a later
amendment replaced, a rule thin enough that a phase would have to guess, an
obligation the gate will raise that the document never acknowledges. It reports
what a phase would predictably stop on. It cannot approve an LLD, which is the
human's act, and it cannot block a commit; what it produces is a list to decide
about, not a verdict. Run it on the draft and again whenever amendments have
accumulated — a document amended a dozen times is no longer the document that
was reviewed, and accretion, not the original draft, is where inconsistency
collects.

**Phase 2 — Derive claims.** *(agent proposes, human approves)*
Agent emits `#[derive(Spec)]` items in the controlled language; the derive
rejects malformed ones (check 13). Reject claims that restate the LLD rather
than asserting something. Because each claim's verb has a template, writing a
claim *is* writing the implementer's return shape: a reviewer approving *shall
reject with `AuthError::InvalidCredentials`* has approved `-> Result<_,
AuthError>` on every function that will implement it.

**Phase 3 — Layer-0 skeleton.** *(agent proposes, human approves)*
Slice entry point and its dispatch, signatures and `#[implements]` only,
`#[flow]`-pinned, `todo!()` bodies. `cargo check` must pass. **Review the
signatures, not the prose.** Because they type-check together, "these five
functions compose into a login flow" is verified rather than asserted; rule V
forces vocabulary at every flow parameter, and S1 and S2 verify each signature
against its claims before any body exists. **The skeleton fails conformance
before it fails anything else**, which is the cheapest possible time.

**Phase 4 — Descend one layer, breadth-first.** *(repeat)*
Every layer-0 leaf gets its own skeleton. `cargo check`. Review. Descend.
Breadth-first because defining sibling functions routinely reveals that an
earlier sibling's model needs adjusting — and that correction should land before
implementation effort is sunk.

*Stop condition:* refine until you'd trust the leaf on sight. A leaf that's
obviously a fold over a slice needs no further layer. A leaf carrying a tricky
invariant gets one, or gets its invariant lifted into a type.

**Phase 5 — Failing-first validations.** *(agent proposes, human approves)*
One `#[validates]` unit test per claim. Run them and **confirm they fail**
against `todo!()` — with the runtime, the traces show *reached, panicked*.
Check 12 preserves this property for the life of the code; Phase 5 establishes
it. The red set is the slice's claims added since the last gate; a claim
implemented only by data cannot go red and is held to check 24 instead
([§4.8](https://bradvoth.github.io/lid-rs/spec/gates.html)).

**Phase 6 — Implement leaves.** *(agent, minimal review)*
The acceleration. Signature pinned, name pinned, claim cited, test red and
specific. The surrounding skeleton constrains what the function is *allowed to
be*, so the agent has almost no room to invent structure. Per [§1.2](https://bradvoth.github.io/lid-rs/spec/purpose.html), review here
is a small local semantic question; the outcomes on the traces turn to the
promised response.

**Phase 7 — Gate.** Run [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html). Commit the slice.

**Phase 8 — Change.**
Every change is an LLD edit, cascaded: edit `lld.md` → re-derive affected specs →
rename the changed claims, leaving each old name as a `#[deprecated]` alias →
`cargo check` names every citation site to revisit (Phase 2's check leaves them
warning; Phase 7's gate denies them). Deleting a spec breaks the build at every
site that implemented it. Editing a claim's verb or response object changes
its template, and S1 names every implementer whose signature no longer keeps
the promise. A Phase 8 edit that subtracts or rewords has no red run of its
own — the behaviour lands at Phase 3 — which is why the alias, and not a
deprecated struct, is the retirement mechanism: the struct would fail checks
10 and 11 the moment its citations moved.

**Slice seams.** At each boundary, reconcile before Phase 3: slice *n* often
wants something slice *n−1* built. Reuse only when the shared thing is one
concept, not a coincidence of shape — and never by adding a parameter to make two
behaviours fit one body. Check 8 catches the `bool` version; nothing catches the
`Option` version, so that one stays a human judgment.

**A phase is run by an agent that can only edit, and its commit is the
check passing.** A slice's work lives on an `lld/<slice>` branch with one
commit per phase, tagged `phase N:`. Phases 2–7 are each run by a Claude
Code agent that `cargo lid-rs sync` ships (`.claude/agents/lid-rs-phase-N.md`)
whose tools are reading and editing — no shell, no git. Its hooks do the
rest: before every edit a per-phase path policy (Phase 2 writes the
slice's claims, 3 and 4 the slice's module and `lib.rs`, 5 and 7 the module
only; never the LLD, the configuration, or another slice); after every
edit, clippy; and when it ends with a ```` ```commit ```` block, phase N's
check — the docs at 1, the build at 2, the type-check at 3 and 4,
the red run at 5 (every `#[validates]` test on the slice's claims must
fail), the full §4.5 gate at 7 — and, if it passes, the commit, staged from
exactly the policy's paths, with trailers counting the agent's edits,
observations, commands (always zero), checks, and refusals. A failing check
keeps the agent running with the output and the skill's rule for that
check. No agent decides whether a check runs. The interactive skill and
the `lid-rs` workflow use the same agents, so the two modes are
interchangeable at every phase boundary. What the hooks bound and what they
do not — the code the agent writes runs at the Phase 5 and 7 stops with the
session's privileges — is in [§12](https://bradvoth.github.io/lid-rs/spec/limits.html).

Three things running phases this way established. Phase 2's check is the
build alone — no lint — because `-D warnings` turns the deprecation warning
that *is* the Phase 8 cascade into a failure in the one phase that cannot
clear it. Two phases cannot share a worktree: the path policy cannot tell
another phase's in-flight work from an edit the agent should not have made,
which is the check working, so an unattended run gives each phase a fresh
worktree. And a check that runs longer than the agent host's stall watchdog
cannot run inside the agent's stop hook at all — the Phase 7 gate, at check
12's measured cost, is such a check — so the gate is run as deterministic
code *between* sessions, by the pipeline, rather than by a hook the agent is
blocked in. The pipeline document (`lid-rs-pipeline`) carries the rest of
that design: one session per phase, a reviewer per phase, adjudication before
any human is asked, and a pull request whose commit list is the development
process.
<!-- ANCHOR_END: flow -->

---

<!-- ANCHOR: example-login -->
## 9. Worked example A — user login

### Phase 1 — LLD (`src/auth/lld.md`)

> [`Credentials`] enter at the login form, pass to the auth service, and resolve to
> one of two outcomes: a [`Session`] scoped to the user, or a structured
> [`AuthError`]. The UI translates [`AuthError`] into a user-safe message; the
> session rides subsequent requests. The service reads accounts from a
> [`CredentialStore`], which may be unreachable.
>
> Failure modes must be indistinguishable to the caller at the presentation
> layer: a wrong password and an unreachable credential store produce the same
> user-facing message, so account existence is not observable. They remain
> distinguishable in logs.

```rust
// src/lib.rs
#![doc = include_str!("../docs/intent/hld.md")]

#[doc = include_str!("auth/lld.md")]
pub mod auth;

#[cfg(test)]
mod intent_graph;
```

The vocabulary derives `Traceable` at birth: `Credentials`, `Username`,
`Secret` (redacted), `Account`, `Session`, `CredentialStore`. The lexicon
has `return`, `reject`, and `equal` with the templates in
[§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html).

### Phase 2 — claims

The four from [§3.1](https://bradvoth.github.io/lid-rs/spec/mapping.html).
The agent's first draft of the backend claim was compound — *shall reject with
`AuthError::Backend` and shall not distinguish it from a mismatch* — and check
13 rejected it; the split produced `BackendFailureIsIndistinguishableToUser`,
which is an invariant on `AuthError` rather than a behaviour of
`authenticate`, and the pattern (ubiquitous, not unwanted) said so. That
invariant is the kind of thing that otherwise lives only in someone's head
and gets quietly violated by an agent writing a "helpful" error message.

### Phase 3 — the skeleton, and what conformance proves about it

```rust
/// Resolves submitted credentials to a session, or a structured failure.
#[implements(
    spec::ValidCredentialsYieldScopedSession,
    spec::UnknownCredentialsAreRejected,
    spec::BackendFailureIsRejected,
)]
#[flow]
pub fn authenticate(store: &CredentialStore, creds: &Credentials) -> Result<Session, AuthError> {
    let account = load_account(store, creds.username())?;
    verify_password(&account, creds.secret())?;
    issue_session(&account)
}

/// Loads the account record for a username.
#[implements(spec::UnknownCredentialsAreRejected, spec::BackendFailureIsRejected)]
fn load_account(store: &CredentialStore, username: &Username) -> Result<Account, AuthError> { todo!() }

/// Checks a submitted secret against an account's stored verifier.
#[implements(spec::UnknownCredentialsAreRejected)]
fn verify_password(account: &Account, secret: &Secret) -> Result<(), AuthError> { todo!() }

/// Mints a session scoped to the given account.
#[implements(spec::ValidCredentialsYieldScopedSession)]
fn issue_session(account: &Account) -> Result<Session, AuthError> { todo!() }

/// Why authentication did not produce a session.
#[derive(Outcome)]
#[non_exhaustive]
pub enum AuthError {
    /// Introduced by [`spec::UnknownCredentialsAreRejected`].
    InvalidCredentials,
    /// Introduced by [`spec::BackendFailureIsRejected`].
    Backend(StoreError),
}
```

`cargo check` passes. What is already *proven* about a design containing no
implementation whatsoever:

- The three sub-operations compose into `authenticate` with no glue.
- `?` works throughout, so all failures unify into `AuthError` — the LLD's
  "one structured error type" claim is enforced, not aspirational.
- `verify_password` borrows rather than consumes, so ordering with
  `issue_session` is fine.
- `Secret` is distinct from `String`, so a password can't be logged by accident.
  If it can, the type is wrong — a Phase 3 finding, before anyone wrote code.
- Every citation resolves. A typo in a spec path fails here, not in review.

`authenticate` has cognitive complexity 1 and satisfies F1–F6: straight-line
composition, no decision, nothing to declare. Then `cargo test --lib` on the
skeleton, before any body exists:

- **S1 (19).** `ValidCredentialsYieldScopedSession` (verb *return*, object
  `Session`): `authenticate` and `issue_session` both return
  `Result<Session, _>` ✓. `UnknownCredentialsAreRejected` (verb *reject*,
  owner `AuthError`): all three implementers return `Result<_, AuthError>` ✓.
  `BackendFailureIsRejected` ✓.
- **S2 (20).** Trigger `Credentials`: `authenticate` accepts `&Credentials` ✓.
  The backend claim reads *if the [`CredentialStore`] is unreachable, then...*,
  so its condition noun is `CredentialStore`, and `load_account` accepts one ✓.
  Had the store been reached through `self`, S2 would have had nothing to
  check; the condition-noun rule made the dependency a parameter, which is
  better code and a checkable claim at once. The predicate — *is
  unreachable* — remains prose; that's the residual in
  [§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html).
- **E1 (21).** `AuthError::InvalidCredentials` and `AuthError::Backend` are
  both variants of an `Outcome` enum ✓.
- **E2 (22).** `AuthError` has two variants; both are named by unwanted claims ✓.
- **V (18).** `authenticate`'s signature is `&CredentialStore`,
  `&Credentials` → `Result<Session, AuthError>`: all vocabulary ✓.

Nothing has been implemented, and the skeleton already provably promises what
the claims say. "The three sub-operations compose, `?` unifies the errors"
was true of the skeleton before conformance existed — but *inferred by the
reviewer*. Now it's asserted by the gate.

**The mismatch the pattern always predicted.** Suppose the agent had proposed,
for the reject claim:

```rust
#[implements(spec::UnknownCredentialsAreRejected)]
fn verify_password(account: &Account, secret: &Secret) -> bool { todo!() }
```

An *unwanted* claim implemented as a boolean return was always a mismatch a
careful reader could see. S1 fails it: `-> bool` matches no *reject*
template. The heuristic became a check, and it fired at Phase 3.

### Phase 5 — failing-first

```rust
#[test]
#[validates(spec::ValidCredentialsYieldScopedSession)]
fn valid_credentials_yield_scoped_session() {
    let store = store_with(account("brad"));
    assert_eq!(authenticate(&store, &valid_creds("brad")).unwrap().user_id(), 42);
}

#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn unknown_credentials_are_rejected() {
    let store = store_with(account("brad"));
    assert!(matches!(
        authenticate(&store, &wrong_password("brad")),
        Err(AuthError::InvalidCredentials)
    ));
}

#[test]
#[validates(spec::BackendFailureIsRejected)]
fn backend_failure_is_rejected() {
    assert!(matches!(
        authenticate(&unreachable_store(), &valid_creds("brad")),
        Err(AuthError::Backend(_))
    ));
}

#[test]
#[validates(spec::BackendFailureIsIndistinguishableToUser)]
fn backend_failure_is_indistinguishable_to_user() {
    for variant in AuthError::examples() {
        assert_eq!(variant.user_facing_message(), AuthError::InvalidCredentials.user_facing_message());
    }
}
```

Four claims, four registered validations, all red against `todo!()` — the
traces show each claim *reached, panicked*. Checks 10 and 11 now pass; check
12 has a baseline. The fourth is ubiquitous, so its proof is check 24's
many-inputs rule: every variant, not one.

### Phases 6–7 — implement and gate

Green traces close with `Err(InvalidCredentials)` ✓ promised, `Ok(Session)` ✓
promised. Then suppose the agent implements `load_account` as:

```rust
fn load_account(store: &CredentialStore, username: &Username) -> Result<Account, AuthError> {
    match store.fetch(username) {
        Ok(Some(a)) if a.is_active() => Ok(a),
        Ok(Some(a)) if a.locked_until().is_some() => Err(AuthError::Locked),
        Ok(Some(_)) => Err(AuthError::InvalidCredentials),
        Ok(None) => Err(AuthError::InvalidCredentials),
        Err(e) => Err(AuthError::Backend(e)),
    }
}
```

Three layers catch it, in order of cost:

1. **Check 7** fires: a leaf with complexity 5. And it's *correct* to fire —
   account locking and activation status appear nowhere in the LLD and
   nowhere in the spec module. With the shape pass, **rule A (15)** names it
   precisely: dispatch with work. Suppose the agent restructures into a pure
   `match` with named leaves, including `reject_locked`.
2. **E2 (22)** fires — `AuthError::Locked` is a variant of a claim-owning enum
   and no unwanted claim names it. *Someone added a way to fail that no
   requirement asked for.* This stops the merge on `cargo test --lib`, and
   the diagnostic says what to do: write the claim or delete the variant.
3. Had E2 somehow passed, the slice page would show a routing decision about
   locked accounts with no claim, and **check 12** would find `reject_locked`
   a surviving mutant.

`AuthError::Locked` is a fourth outcome the design never authorised, and it
leaks account state to the caller, violating
`BackendFailureIsIndistinguishableToUser`. The fix isn't raising a threshold.
It's returning to Phase 1, deciding whether lockout is in scope, and if so
writing the claim — at which point the variant and the requirement land
together, the branch is declared, `load_account` becomes a dispatch node, and
store access moves down a layer into its own leaf.

**That is the point of the system.** The agent made a judgment call, and a
registry intersection caught it in the same session, without a human reading
the diff carefully enough to notice a fourth error variant.
<!-- ANCHOR_END: example-login -->

---

<!-- ANCHOR: example-settings -->
## 10. Worked example B — applying a settings change

Chosen because it is dispatch-shaped, so it exercises the cascade and
the flag-argument rule.

### Phase 1 — LLD excerpt

> A [`SettingChange`] arrives as one of a closed set of change kinds. Each kind is
> validated against current [`Account`] state, then applied. Validation and
> application are per-kind; nothing is shared between kinds except the
> transaction boundary. New change kinds are expected over time and must not be
> silently ignorable by existing call sites.

That last sentence is a design decision with a direct Rust encoding.

### Phase 3 — layer-0 skeleton

```rust
/// A change a user can make to their own settings.
#[implements(spec::SettingChangeKindsAreAClosedSet)]
#[non_exhaustive]
pub enum SettingChange {
    /// Replace the display name.
    DisplayName(DisplayName),
    /// Replace the notification schedule.
    NotificationSchedule(Schedule),
    /// Begin the email-change handshake.
    EmailAddress(EmailAddress),
}

/// Dispatches a settings change to its per-kind handler.
#[implements(spec::EveryChangeKindHasExactlyOneHandler)]
#[flow]
pub fn apply(account: &mut Account, change: SettingChange) -> Result<Applied, SettingError> {
    match change {
        SettingChange::DisplayName(name) => apply_display_name(account, name),
        SettingChange::NotificationSchedule(s) => apply_schedule(account, s),
        SettingChange::EmailAddress(email) => begin_email_change(account, email),
    }
}

/// Validates and applies a new display name.
#[implements(
    spec::DisplayNamesAreLengthBounded,
    spec::DisplayNamesRejectControlCharacters,
)]
fn apply_display_name(account: &mut Account, name: DisplayName) -> Result<Applied, SettingError> { todo!() }

/// Validates and applies a new notification schedule.
#[implements(spec::SchedulesMustLieInTheAccountTimeZone)]
fn apply_schedule(account: &mut Account, schedule: Schedule) -> Result<Applied, SettingError> { todo!() }

/// Starts the two-step email change handshake.
#[implements(
    spec::EmailChangeRequiresConfirmationAtTheNewAddress,
    spec::TheOldAddressRemainsActiveUntilConfirmation,
)]
fn begin_email_change(account: &mut Account, email: EmailAddress) -> Result<Applied, SettingError> { todo!() }
```

`apply` is a textbook dispatch node: one `match`, three arms, zero work. Its
purpose is expressing where control goes, and the flat arms keep cognitive
complexity low ([§4.1](https://bradvoth.github.io/lid-rs/spec/gates.html)'s metric choice, demonstrated).
Its signature satisfies rule V: `Account`, `SettingChange`, `Applied` are
vocabulary; `SettingError` is an `Outcome` enum. An agent proposing
`apply(account_id: u64, change: SettingChange)` fails V — `u64` where the LLD
said *account*.

### The cascade, concretely

Add a variant:

```rust
    /// Set the account's time zone.
    TimeZone(Tz),
```

`cargo check` fails at `apply` — non-exhaustive match. The build is broken
until the new case is dispatched; no lint or checklist is involved.
`wildcard_enum_match_arm` set to `deny` prevents defusing it with
`_ => Ok(Applied::NoOp)`. `#[non_exhaustive]` extends the same discipline to
downstream crates.

Then check 10 fires on the new spec until a handler implements it, and check 11
until a test validates it. And if the handler adds a
`SettingError::InvalidTimeZone` variant, E2 (22) fires until the unwanted
claim that names it exists — so the error variant and the requirement land
together, or not at all. One design edit propagates to four separate
failures, each naming the next thing to do.

### The anti-pattern, caught

Suppose during Phase 4 the agent notices that `apply_display_name` and
`apply_schedule` share a validate-then-write shape, and proposes:

```rust
fn apply_field(
    account: &mut Account,
    field: Field,
    value: Value,
    validate_strictly: bool,     // ← check 8 fires
) -> Result<Applied, SettingError>
```

Correct on two counts. The `bool` is a branch: it introduces a decision inside
what was supposed to be a leaf, so the function becomes a covert dispatch node
with its dispatch buried mid-body. And the abstraction is a coincidence of shape
— display names and schedules validate against different rules, and
`Field`/`Value` erase the types doing the real work.

Two clean leaves is the answer. The transaction boundary the LLD mentions is
shared and belongs in a wrapper around `apply`, not threaded through
the leaves.
<!-- ANCHOR_END: example-settings -->

---

<!-- ANCHOR: layout -->
## 11. Repo layout and reading surfaces

### 11.1 Layout

Everything about a slice lives in one directory:

```
Cargo.toml                     workspace lints + [workspace.metadata.lid_rs]
.claude/skills/lid-rs/         the operating skill — tool-owned: written by
                               `cargo lid-rs sync` from the lid-rs dependency,
                               never edited (project guidance goes in AGENTS.md)
.claude/agents/lid-rs-*.md     the phase agents (one per committing phase) and
                               the reviewers — edit-only and read-only tool sets,
                               their hooks in the frontmatter; synced the same way
.claude/workflows/lid-rs.js    the unattended build — synced the same way
clippy.toml                    thresholds
docs/
  intent/
    hld.md                     -> included by lib.rs
    lexicon.toml               the verbs, with definitions and templates (§3.5)
    trace.md                   generated, committed, freshness-gated (§11.2)
src/
  lib.rs                       doc includes + #[cfg(test)] intent_graph!()
  auth/
    lld.md                     marks this directory as a slice; included by mod auth
    vocab.rs                   the slice's nouns, each Traceable
    spec.rs                    the slice's claims
    mod.rs                     the slice's public face: flow nodes, error
                               taxonomy, #[cfg(test)] validations
    account.rs                 leaves
  settings/
    lld.md  vocab.rs  spec.rs  mod.rs
  bin/
    app.rs                     thin — doctests don't run in bin targets
tests/
  ...                          integration tests; no #[validates] here (§5.2)
lid-rs/                        support crate: Spec, Edge, SpecMeta, slices, canary
lid-rs-macros/                 derive(Spec, Traceable, Outcome), implements,
                               validates, implements_module!, spec, flow, leaf
lid-rs-shape/                  the shape pass: classify, check (A/B/P/V), signatures
cargo-lid-rs/                  the command catalog: the checks as commands
lid-rs-pipeline/               the unattended build: sessions, gates, review,
                               adjudication, the PR
.github/workflows/gate.yml
```

Specs live in `src/`, not `docs/`, because they must be items the compiler
resolves. The LLD lives beside them because it is prose meant to be diffed and
argued over in a PR *about that slice* — and it renders inside `cargo doc`,
so a reader never goes looking for it. A reader opening `src/auth/` has the
whole arrow in front of them: design, nouns, claims, flow, leaves, validations.

Citations are one segment longer than with a crate-wide `spec` module —
`auth::spec::UnknownCredentialsAreRejected` from another slice — and that is
the point: a claim's path names the slice that owns it, and a claim is
uncitable only if its own module is, which is a mistake nobody makes twice.
The crate-wide `src/spec/<slice>.rs` layout with a re-exporting `mod.rs` is
the one this workspace was built in. The move waited for the
controlled-language slice to rewrite every claim, so the cascade was one rather
than two; it has since been made. A crate keeps a `src/spec.rs` afterwards only
where something outlives the move: its own claims when the crate root *is* a
slice, and retired names whose deprecation windows are still open — an alias
exists so the old path keeps resolving, so moving it would defeat it.

In a workspace, an intent document lives inside the package root of the crate
that includes it. `cargo package` ships only files
under the package root, so an `include_str!` reaching up into a workspace-level
`docs/` builds locally and fails from the tarball. Documents no crate includes
stay at the workspace level.

### 11.2 Reading surfaces

The work product has five reading surfaces, and readability means each one
tells the same story in the order a reader needs it.

**Source, in refinement order.** Rust doesn't care where items sit in a file.
A slice's `mod.rs` starts with the entry point, then its dispatch, then the
leaves in the order they're called: the top of the file is the table of
contents, and scrolling down is descending the refinement. A reader who stops
after the first screen has the design. Flow bodies pass the no-descent test by
construction ([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html)).

**Reference.** `cargo doc`: the HLD as the crate's front page, each LLD on its
module with every noun a link, every citation a doc line, every vocabulary
type's definition one click from the prose that uses it.

**The site.** A generated book with **no content of its own** — every page is
an LLD rendered verbatim or a page generated from the registry, so it cannot
drift from the code because it is made from it. Per slice: the LLD with its
vocabulary links; a *claim card* per spec — the claim text, its pattern, the
verb's template and the implementer signatures matched against it side by
side, its validators, the validator's captured trace as the worked example,
the last test result, the last mutation verdict; the entry point's flow body;
the observed flow graph ([§6.6](https://bradvoth.github.io/lid-rs/spec/traced.html));
the slice's git history. A trace matrix over the whole graph, sortable, with
the error taxonomy as a table — every variant, the unwanted claim that owns
it, the validators that produce it; a variant row with an empty claim column
is check 22 about to fire. A glossary from the vocab modules. Test and
mutation results are display, not checks: the gates already failed if any of
those are red, and the site exists so that "is this claim actually held" is
one click rather than a CI archaeology exercise. The generator runs inside the
test binary, because `VALIDATIONS` exists nowhere else
([§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html)). What to resist:
any hand-written page beyond the LLDs. If a slice needs a better introduction,
the LLD needs a better introduction.

**Tool output.** `docs/intent/trace.md` — one section per slice, one row per
claim, with links, plus the shape and conformance `warn` counts — is
generated from the registry and the shape pass, committed, and
freshness-gated (check 26) the way `cargo fmt --check` works: small enough to
belong in PR diffs, where a claim's wording change appears in the same row as
its validators. And `cargo test`'s output is the requirements checklist,
because validators are named after their claims.

**History.** One commit per phase makes `git log` on a slice read as the
refinement narrative — LLD, claims, skeleton, layers, validators, leaves, gate —
with the skeleton commit showing the shape before any body existed. History
is usually the worst reading surface; this makes it the best.

**Traces.** In production, the same spans ([§6.7](https://bradvoth.github.io/lid-rs/spec/traced.html)).

### 11.3 Brownfield adoption

Layer the tiers in order (`cargo lid-rs init` in the
package performs Tier 0 and the wiring in one step). Tier 0 first — the lints apply
to existing code immediately and will surface every leaf that's secretly a
dispatch node. Then write LLDs for the slices you're actively changing, and let
`#[implements]` spread through the code you touch rather than in a big-bang pass.
Checks 10 and 11 only ever assert over specs that exist, so a partially-traced
codebase gates correctly on the part that's traced. Shape and conformance at
`warn` will, on first run, list every leaf that is secretly a dispatcher,
every implementer whose return shape disagrees with its claim's verb, and
every error variant no requirement owns. All three lists are usually worth
having independently of this system.
<!-- ANCHOR_END: layout -->

---

<!-- ANCHOR: limits -->
## 12. Honest limits

- **The semantic residual.** The condition predicate and the assertion's
  meaning ([§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)); the
  differential pass needs a scheduled owner.
- **Template matching is syntactic and shallow.** Type aliases and `impl Trait`
  need declaring in `conformance.aliases`. Generics need the noun to appear
  literally. Loud, not silent.
- **S2 checks the condition's noun, not its predicate.** *If the
  [`CredentialStore`] is unreachable* puts `CredentialStore` in a signature; it
  says nothing about *unreachable*.
- **`min_inputs` is a floor, not a proof.** Sixteen distinct inputs through a
  constructor is evidence the invariant was exercised, not that the boundary was.
  Property-based generation is still the right way to produce them; the check
  just no longer cares *how* they were produced.
- **E2 covers claim-owning enums only.** Infrastructure errors are not policed
  here, by design.
- **Dynamic evidence is bounded by validator inputs.** Checks 23–25 see the
  paths the validators took and no others.
- **Uninstrumented `spawn` orphans spans** — check 23 fails loudly.
- **`Traceable` is a compile-time obligation** on every traced boundary.
- **The shape grammar is a first cut**; `warn` is the ramp, and the
  complexity threshold is a measurement to make ([§7](https://bradvoth.github.io/lid-rs/spec/configuration.html)).
- **`linkme` has platform edges.** See [§5.4](https://bradvoth.github.io/lid-rs/spec/registry.html). The canary converts silent failure
  into loud failure, but on an unusual target you will be debugging a linker
  mechanism. The `inventory` fallback exists for that case.
- **Module-level `implements_module!` is coarse.** It traces by containment, so a
  module that grows past its original claim will carry a citation that's become
  approximate. Treat module size as the check on this — nothing enforces it.
- **`cognitive_complexity` is a nursery lint.** It has known false positives and
  its behaviour can change between clippy releases. Pin the toolchain in CI.
- **Check 12's cost is the gate's cost.** Measured in [§4.3](https://bradvoth.github.io/lid-rs/spec/gates.html);
  it bounds where the gate can run.
- **The phase agents bound tool calls, not the code they write.** A phase
  agent ([§8](https://bradvoth.github.io/lid-rs/spec/flow.html)) cannot run a
  command, edit outside its phase's paths, or commit;
  no instruction in its prompt or in a file it reads can make it. But the
  checks its hooks run execute the code it wrote — its tests and anything
  that runs when the test binary loads at the Phase 5 and 7 stops, the
  doctests and mutation runs at 7, and, in a slice whose crate is a
  proc-macro crate or has a build script, every edit's clippy — with the
  session's privileges: network, secrets, writes anywhere. The stop hook
  detects and refuses writes inside the repository that the agent could not
  have made (the synced files, anything outside the policy's paths), and a
  compile-time slice is refused unless the human commits
  `compile-time-accepted` beside its LLD; nothing detects
  what that code does outside the repository. Run an unattended build only
  where you would let untrusted code run — a container or VM, or a sandbox
  that denies the network and confines writes — with no credentials it does
  not need. Sandboxing each check from the hook is a control the tool may
  own later; until it does, this document does not imply it.

**What is built, and what is not.** This document is a living design (the
workspace HLD's first tenet), and it describes the target. Of it, the
following is specified here and not yet shipped, each the subject of a slice
in the HLD's map: the uncitable-claim assertion, whose path the colocated
layout defines; vocabulary as `Traceable` types and rule V
([§3.6](https://bradvoth.github.io/lid-rs/spec/mapping.html)); the shape pass
([§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html), checks 15–18);
`#[derive(Outcome)]`, `OUTCOMES`, and conformance
([§4.7](https://bradvoth.github.io/lid-rs/spec/gates.html), checks 19–22); the
runtime ([§6.4](https://bradvoth.github.io/lid-rs/spec/traced.html)–6.8, checks
23–25); `trace.md` and the site ([§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html),
check 26); the colocated layout ([§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html));
and the pipeline as a crate. Checks 1–14, the phase agents, the skill, the
workflow, and the book are built and gate this repository today — the
controlled language and its lexicon
([§3.5](https://bradvoth.github.io/lid-rs/spec/mapping.html)) among them, with
every claim either written in the language or carrying a counted `#[lid(free)]`
mark that the ramp burns down slice by slice.
<!-- ANCHOR_END: limits -->

---

<!-- ANCHOR: bootstrap -->
## 13. Bootstrap checklist

`cargo lid-rs new <name>` performs this list for a new package, and
`cargo lid-rs init` performs it for an existing one; the list remains the
definition of what they do.

1. `cargo new --lib` plus a thin `bin`.
2. `lid-rs` support crate: `Spec` trait, `Edge`, `SpecMeta` (pattern, modal,
   trigger, verb, object, owner), `Variant`, the four
   `#[distributed_slice]` declarations, `canary`, `Traceable`, `Outcome`,
   the capturing layer, `spawn`, `__private` re-export.
3. `lid-rs-macros`: `derive(Spec)` (enforcing the language, extracting
   `SpecMeta`), `derive(Traceable)`, `derive(Outcome)` (registering
   variants), `implements`, `validates`, `implements_module!`, `spec`,
   `flow`, `leaf`.
4. `lid-rs-shape`: `classify`, `check` (A, B, P, V), `signatures`.
5. `Cargo.toml` workspace lints and metadata including `shape`,
   `conformance`, `runtime` ([§7](https://bradvoth.github.io/lid-rs/spec/configuration.html));
   `clippy.toml` thresholds; `[profile.test] opt-level = 0`.
6. `docs/intent/hld.md`, included via `#![doc = include_str!(...)]`;
   `docs/intent/lexicon.toml` with the first verbs **and their templates**,
   and the INCOSE list.
7. The first slice's directory: `lld.md` with its nouns as links, `vocab.rs`
   with every noun `Traceable`, `spec.rs` with `//!` docs explaining what the
   module is for.
8. A `#[cfg(test)] mod intent_graph { lid_rs::intent_graph!(); }` in `lib.rs` —
   canary first, then checks 10, 11, 15–26, and the `regen` that writes
   `trace.md`.
9. Install `cargo-mutants` and `cargo-lid-rs`; the gate's mutation step
   ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html)) scopes by
   registry and by file.
10. CI running [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) in order, all of it gating; the site build as a
    publish step.
11. `AGENTS.md` / `CLAUDE.md` stating the eight phases, the dispatch-vs-work
    rule, the controlled language, pattern → shape → trace, and **"the verb
    is the return type,"** so the agent proposes skeletons rather than
    implementations.
12. First slice end to end before writing a second LLD. The phase boundaries are
    where the tedium hides; find out where it hurts on one slice before
    scaling to ten. Flip `shape.level` and `conformance.level` to `deny` when
    the counts have been stable for the slice.
<!-- ANCHOR_END: bootstrap -->
