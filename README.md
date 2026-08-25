<!-- ANCHOR: header -->
# LID-rs

**Linked-Intent Development, compiled.**
A spec-driven workflow for Rust in which the intent graph is made of Rust items,
so the compiler — not grep, and not a second parser — enforces the links.

LID-rs is an opinionated, Rust-specific implementation of
[Linked-Intent Development (LID)](https://linked-intent.dev/), which is the
source of the idea: link design intent to code through a walkable arrow of
HLDs, LLDs, atomic claims, tests, and citations. Where LID is
language-agnostic and enforces the arrow by convention and tooling, LID-rs
trades that generality for teeth — every edge of the graph becomes something
the Rust compiler, linker, or test harness resolves and gates.

<!-- ANCHOR_END: header -->

---

<!-- ANCHOR: premise -->
## 0. The premise

LID (Linked-Intent Development) links a design document to the code built from
it via greppable requirement IDs: an HLD states the *why*, LLDs state the *how*,
EARS one-liners state atomic claims, tests assert those claims, and code carries
`@spec` annotations citing them. `grep -r AUTH-UI-001` returns the whole arrow.

That linkage is mechanical but *lexical*. An `@spec` comment is a string. It can
cite a requirement that was deleted, describe behaviour the function no longer
has, or be absent entirely from code the agent invented on its own initiative.
LID's own `bidirectional-differential` experiment exists precisely because specs
and code drift apart despite the IDs.

The adjustment: **replace the free-text spec layer with a refinement skeleton
made of real signatures**, and make every edge of the graph something the
compiler resolves. Two additional principles come from stepwise refinement:

- **Dispatch and work are separate functions.** A function either makes one flow
  decision (one `match`, one `if/else` chain, however many arms) or it does one
  unit of work. Never both. Every branch is a decision; every decision should
  have been declared in the design.
- **Refine breadth-first within a vertical slice.** Complete a whole abstraction
  layer for one user-visible operation before descending, so cross-cutting
  corrections land before implementation effort is sunk.

The consequence worth stating up front: **a leaf with a branch in it is a
requirement nobody wrote down.** That turns the complexity rule into a drift
detector you can put in CI.
<!-- ANCHOR_END: premise -->

---

<!-- ANCHOR: purpose -->
## 1. What this is for

### 1.1 The goal is to make semantic drift the only thing left to review

Structural correctness — does this code cite a real claim, does a test depend on
this code, did the design change reach every call site — is mechanical work.
Machines are good at it and humans are terrible at it, especially at review time,
especially across a diff an agent produced in one pass.

Semantic correctness — does this code mean what the claim says — is the opposite.
No tool can check it. It's the entire reason a human is in the loop.

So the design goal is not "catch drift." It's **to make every structural
property hold automatically, so that reviewer attention lands on meaning and
nothing else.** Every check in §4 exists to remove a class of question from
review, not to add a hurdle. When someone reviews a Phase 3 skeleton, the
composition already type-checks, every citation already resolves, and the
cascade has already reached every affected site — so the only remaining question
is *is this the right decomposition of the problem*, which is the question worth
their time.

The iterative refinement flow (§7) serves the same end from the other direction.
By the time an implementation is written, its name, signature, claim, and failing
test are all pinned, so the semantic question at that point is small and local:
*does this body mean what this claim says.* Both halves of the system are aimed
at shrinking the surface a human has to think hard about — not at eliminating
the thinking.

The residual is real and named in §4.4: a test can cite the wrong claim and pass
every gate. That's the differential pass's job, and the system's success
condition is that it's the *only* thing left for it to do.

### 1.2 This is not for prototypes

Everything here is machinery for code that must remain trustworthy over time,
under modification, by people and agents who weren't there when it was written.
That machinery has a real cost: a design document per slice, a claim per
behaviour, a test per claim, and a review gate at every refinement layer.

Exploratory work should skip all of it. Proofs of concept, spikes to answer a
feasibility question, notebooks, one-off scripts, research code whose output is a
number and not a system — the correct amount of LID-rs in those is zero. Adopt at
the point the code stops being disposable, which is usually the moment someone
proposes building on it.

Adopting mid-life is a supported path — see the brownfield note in §11 — and is
better than adopting early. A spike that earned its way into production arrives
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
   source to reconstruct the spec graph will silently diverge from the compiler's
   view — `use` renames, re-exports, `#[cfg]`, macro expansion. Silent divergence
   in a correctness tool is worse than no tool.
3. **Every check gates, or it gets deleted.** A report nobody reads is worse than
   an absent check, because it lets you believe coverage exists. If something
   can't be made to fail the build, remove it and be honest about the gap.

Constraint 3 has a corollary that recurs throughout: **a check built on
enumeration must first prove the enumeration is non-empty.** A registry that
silently fails to populate turns every check over it into a vacuous pass. See
§5.3.
<!-- ANCHOR_END: constraints -->

---

<!-- ANCHOR: mapping -->
## 3. Mapping: LID concept → Rust mechanism

| LID artifact | Rust mechanism | Why this mechanism |
|---|---|---|
| **HLD** — the *why* | `#![doc = include_str!("../docs/intent/hld.md")]` in `lib.rs` | Stays a reviewable markdown file in the repo; renders as the crate's front page in `cargo doc`, directly above the API it governs. |
| **LLD** — the *how*, per slice | `#[doc = include_str!("../docs/intent/auth/lld.md")] pub mod auth;` | One LLD per module. The design and the module boundary become the same boundary. |
| **EARS claim** | `#[derive(Spec)]` on a unit struct; the doc comment *is* the claim | A claim becomes a nameable, linkable, resolvable item. The derive reads the `#[doc]` attributes, so the text is single-sourced. |
| **Spec ID** | The struct's own name, written descriptively | See §3.2. |
| **`@spec` annotation** | `#[implements(spec::ValidCredentialsYieldScopedSession)]` | Emits the doc link, a const type-assertion, and a registry entry. A bad citation is a **type error**, not a broken link. |
| **Test → claim link** | `#[validates(spec::ValidCredentialsYieldScopedSession)]` | Same three effects, on the test side. |
| **Coverage of the graph** | `linkme` distributed slices + an ordinary `#[test]` | Enumeration at link time. No source parsing anywhere. See §5. |
| **Non-vacuous assertion** | Diff-scoped `cargo-mutants`, test subset narrowed by the registry | Proves the test *depends on* the implementation, not merely that it executed it. |
| **Spec → code cascade** | Recompilation; `#[non_exhaustive]`; exhaustive `match` | Adding a case breaks every dispatch site. A compiler error, not an agent pass. |
| **Spec retirement** | `#[deprecated]` on the spec struct | Warns at every citation site through the const assertion. Stable compiler, zero tooling. |
| **Refinement skeleton** | Signatures with `todo!()` bodies | `!` coerces to any type, so a whole layer **type-checks before any leaf exists**. |
| **Complexity budget** | `clippy.toml` thresholds | Enforces dispatch/work separation, which is what makes undeclared decisions detectable. |

### 3.1 Specs are Rust items, authored in `src/`

```rust
//! Atomic claims for the auth slice. Derived from `docs/intent/auth/lld.md`.
//!
//! Each item is one EARS claim. Nothing here has runtime behaviour; these types
//! exist so that citations are resolved by the compiler rather than by grep.

use lid::Spec;

/// When a user submits valid credentials, the authentication service shall
/// return a session scoped to that user.
#[derive(Spec)]
pub struct ValidCredentialsYieldScopedSession;

/// When a user submits credentials that do not match a stored account, the
/// authentication service shall return `AuthError::InvalidCredentials`.
#[derive(Spec)]
pub struct UnknownCredentialsAreRejected;

/// When the credential store is unreachable, the authentication service shall
/// return `AuthError::Backend` and shall not distinguish it from a credential
/// mismatch in any user-facing message.
#[derive(Spec)]
pub struct BackendFailureIsIndistinguishableToUser;
```

A derive rather than a block macro, deliberately. Doc comments stay ordinary doc
comments, so rustdoc, rust-analyzer hover, `missing_docs`, and go-to-definition
all behave normally instead of interacting with a macro arm. Each claim is a real
item at a real source location.

### 3.2 Names, not numbers

`AUTH-UI-001` is a holdover from requirements-document culture, where an ID is a
stable handle for cross-reference in prose. Citations here aren't prose — they're
paths the compiler resolves — so the number buys nothing the name doesn't.
`grep -r ValidCredentialsYieldScopedSession` works exactly as well, and rustdoc
search finds the item by its real name.

More importantly, a descriptive name makes every citation site self-documenting.
`#[implements(spec::BackendFailureIsIndistinguishableToUser)]` states its own
content; `#[implements(spec::AuthUi003)]` requires a lookup. This is the same
verbose-name-as-specification move the refinement practice makes everywhere else,
applied to the spec layer.

The objection is rename cost: rewording a claim should change its name, breaking
every citation. That's the correct behaviour — a reworded claim *should* force
re-review at each implementing site — and `#[deprecated]` provides the migration
path. Numbers make the rename cheap by making it meaningless.

**Foreign keys are the exception.** When claims originate outside the codebase —
a compliance matrix, a customer's numbered spec document, a regulatory
requirement — the dash-case ID is a genuine foreign key:

```rust
/// The system shall log all authentication attempts with a stable subject
/// identifier and retain them for 90 days.
#[derive(Spec)]
#[spec("SOC2-CC6.1-003")]
pub struct AuthAttemptsAreAudited;
```

`#[spec("...")]` emits `#[doc(alias)]` so the foreign ID stays greppable and
rustdoc-searchable. Inert unless there's an external system to key against; don't
reach for it by default.

### 3.3 Anatomy of a citation

```rust
/// Resolves credentials to a session.
///
/// ```
/// # use myapp::auth::authenticate;
/// # use myapp::test_support::valid_creds;
/// let session = authenticate(&valid_creds()).unwrap();
/// assert_eq!(session.user_id(), 42);
/// ```
#[implements(
    spec::ValidCredentialsYieldScopedSession,
    spec::UnknownCredentialsAreRejected,
    spec::BackendFailureIsIndistinguishableToUser,
)]
pub fn authenticate(creds: &Credentials) -> Result<Session, AuthError> {
    todo!()
}
```

The attribute expands to two things:

```rust
// 1. the human-facing doc line, so rustdoc renders and link-checks it
#[doc = "Implements [`crate::spec::ValidCredentialsYieldScopedSession`]."]

// 2. a link-time registry entry (see §5). The `<Path as Spec>::NAME`
//    projection doubles as the type assertion: a bad path is a compile
//    error, and a #[deprecated] spec warns right here. The `const _` block
//    scopes the static, so it needs no name-mangling and cannot collide.
const _: () = {
    #[allow(missing_docs, clippy::missing_docs_in_private_items)]
    #[::lid::__private::linkme::distributed_slice(::lid::IMPLEMENTATIONS)]
    static EDGE: ::lid::Edge = ::lid::Edge {
        spec: <crate::spec::ValidCredentialsYieldScopedSession as ::lid::Spec>::NAME,
        item: concat!(module_path!(), "::authenticate"),
        file: file!(),
        line: line!(),
    };
};
```

`Spec::NAME` (definition-site `module_path!()` plus the item's identifier) is
generated by the derive, so both sides of every registry join produce the key
from the same single source. (`core::any::type_name` cannot serve here: it is
not const-stable, so it cannot initialize a static on stable Rust.) Effect (2)
is what removes the parser — name resolution is the compiler's, so `use`
renames, re-exports, and `#[cfg]` are handled correctly and for free — and
what makes the graph enumerable at runtime without anyone reading source.

### 3.4 Anatomy of a validation

```rust
#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn wrong_password_is_rejected() {
    assert!(matches!(
        authenticate(&wrong_password()),
        Err(AuthError::InvalidCredentials)
    ));
}
```

Same three effects, registering into `VALIDATIONS`. **These must be
`#[cfg(test)]` unit tests inside the library, not files under `tests/`** — see
§5.2 for why.

**Why not doctests?** Doctests compile as separate crates and never link into the
registry, so they cannot register. Keep them — they're the best form of the
"claim and assertion in the same block of text" property, and they're excellent
public-facing documentation — but they are *examples*, not the gate.
<!-- ANCHOR_END: mapping -->

---

<!-- ANCHOR: gates -->
## 4. Validation checks

Twelve gates in two tiers. Both tiers gate; both run on stable; neither parses
Rust source.

### 4.1 Tier 0 — compiler, rustdoc, clippy

| # | Check | How | Failure means |
|---|---|---|---|
| 1 | **Unresolvable citation** | `cargo check` (const assertion) | Code cites a spec that was renamed or deleted. Also warns for `#[deprecated]` specs. |
| 2 | **Broken doc link** | `cargo doc` with `-D rustdoc::broken_intra_doc_links` | A hand-written link in an LLD or doc comment dangles. Belt-and-braces; check 1 is primary for citations. |
| 3 | **Undocumented item** | `missing_docs`, `clippy::missing_docs_in_private_items` | An item exists with no stated intent. |
| 4 | **Skeleton incoherence** | `cargo check` at every refinement layer | The layer you just designed doesn't fit together — wrong error type, unsatisfiable borrow, missing lifetime. Caught before implementation. |
| 5 | **Broken example** | `cargo test --doc` | A public-facing example no longer reflects the API. |
| 6 | **Incomplete cascade** | `#[non_exhaustive]` + exhaustive `match` + `wildcard_enum_match_arm` | A new case was added upstream and a dispatch site swallowed it under `_ =>`. |
| 7 | **Undeclared decision** | `clippy::cognitive_complexity` | A function does dispatch *and* work, or contains a branch that never appeared in the design. **The structural drift detector.** |
| 8 | **Flag argument** | `clippy::fn_params_excessive_bools`, threshold 0 | Two functions in a trench coat. A `bool` parameter is a branch smuggled into a leaf. |
| 9 | **Inlined concept** | `clippy::too_many_lines` | A coherent sub-thought was manually inlined instead of being named. |

Check 7 is load-bearing. Its logic: every branch is a decision, every decision
should be a spec claim, and dispatch nodes are the only place decisions may live.
A leaf whose complexity exceeds 1 is *either* an undeclared dispatch node *or* a
requirement that was never written down — an agent making a judgment call you
never saw, caught mechanically.

> **Toolchain notes.** Intra-doc links are resolved by rustdoc, not `cargo
> build`, so check 2 needs its own `cargo doc` step. Doctests only run for
> library targets, so structure the project as a thin `bin` over a `lib`.

### 4.2 Tier 1 — registry intersection

Enumeration happens at link time (§5). These are ordinary unit tests.

| # | Check | Failure means |
|---|---|---|
| 10 | **Uncited spec** | A claim nothing implements. The design says it; the code doesn't do it. |
| 11 | **Unvalidated spec** | A claim no test cites. Nothing would notice if it broke. |

```rust
// src/intent_graph.rs — compiled into the lib, #[cfg(test)]
use lid::{SPECS, IMPLEMENTATIONS, VALIDATIONS};

#[test]
fn registry_is_populated() {
    // Constraint 3's corollary: prove the enumeration exists before
    // asserting anything over it. See §5.3.
    assert!(lid::canary::present(), "registry empty — see §5.3");
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
design decision, not a gap — see §6.

> **Scoping note.** The registry is binary-global: a consumer's test binary
> links `lid` (and any other traced crate), so `SPECS` contains upstream
> claims whose `#[validates]` edges — being `#[cfg(test)]` in their home
> crates — are absent from this binary. The checks therefore scope to specs
> whose `NAME` begins with the current crate's name; edge sets stay
> unfiltered. `lid::intent_graph!()` expands to the three tests above with
> that scoping applied — invoke it in a `#[cfg(test)]` module rather than
> hand-writing the checks.

### 4.3 Tier 1 — non-vacuity by scoped mutation

`#[validates]` proves a test *claims* a spec. It cannot prove the test would
notice if the implementation were wrong.

Phase 5 already solves this by hand: a `todo!()` body panics, so any test that
genuinely exercises the cited function must fail against the skeleton. "Confirm
red" is a proof of non-vacuity performed when it costs nothing. The problem is
that it's a human ritual done once, and nothing preserves it.

| # | Check | Failure means |
|---|---|---|
| 12 | **Vacuous validation** | Mutating an `#[implements]` function does not fail the tests that `#[validates]` its specs. The arrow is decorative — citation resolves, test passes, test does not depend on the code. |

`cargo-mutants` substitutes plausible return values for function bodies; a test
surviving that mutation is not asserting anything about the function. Strictly
stronger than line coverage, which counts incidental execution as evidence.
Deterministic, stable toolchain, no instrumentation.

Two narrowings compose to keep it inside a per-PR budget:

- `--in-diff` mutates only functions the PR touched.
- The registry supplies the test subset: for a mutant in function `F`, run the
  tests validating the specs `F` implements. The `xtask` obtains each crate's
  `IMPLEMENTATIONS`/`VALIDATIONS` from that crate's **own `--lib` test
  binary** — a dump mode emitted by `intent_graph!()` — because validation
  edges exist in no other binary (§5.2 applies to the tooling too). A mutant
  whose narrowed test set is empty runs the full suite instead: zero reachable
  tests must never mean zero tests run.

Set `[profile.test] opt-level = 0` so inlining can't erase a mutation site. The
registry-driven test filtering is the one piece that has to be built; the rest is
off-the-shelf.

### 4.4 What remains uncaught

A test that cites the *wrong* spec: mechanically sound — it resolves, it's
non-vacuous, it passes — but semantically misaligned with the claim it names. No
structural check can catch this, because every structural property holds.

That is the bidirectional-differential pass's job: reconstruct the claim from the
code in a fresh session and diff it against the written one. Run it periodically,
not per-PR. Per §1.1, this being the *only* residual is the system working as
designed, not a shortfall.

### 4.5 The gate, in order

```bash
cargo check --all-targets                        # 1, 4
cargo clippy --all-targets -- -D warnings        # 3, 6, 7, 8, 9
RUSTDOCFLAGS="-D rustdoc::broken_intra_doc_links" \
  cargo doc --no-deps                            # 2
cargo test --doc                                 # 5
cargo test --lib                                 # 10, 11 + behaviour
cargo xtask mutants                              # 12 (scope from metadata;
                                                 #     --full / --diff-base override)
```

Cheapest and most specific first. Mutation runs last because it's the only step
that rebuilds.
<!-- ANCHOR_END: gates -->

---

<!-- ANCHOR: registry -->
## 5. The registry mechanism

Checks 10 and 11 need to enumerate every spec, every citation, and every
validation. Doing that without parsing source (constraint 2) means collecting
them at link time.

### 5.1 How it works

`linkme` uses linker sections. `lid` declares the slices:

```rust
#[distributed_slice]
pub static SPECS: [SpecMeta] = [..];
#[distributed_slice]
pub static IMPLEMENTATIONS: [Edge] = [..];
#[distributed_slice]
pub static VALIDATIONS: [Edge] = [..];
```

Each macro expansion emits a `static` placed into a specially-named section. The
linker gathers all statics in that section contiguously and emits start/end
symbols; dereferencing the slice yields everything between them. It's a
contiguous array baked into the binary — no initialization, no runtime cost, no
ordering guarantee.

**`linkme` is an implementation detail.** Users depend on `lid`; `lid` depends on
`linkme`. Expansions can't emit `::linkme::...`, since that path only resolves if
the user happens to have linkme as a direct dependency, so `lid` re-exports it:

```rust
#[doc(hidden)]
pub mod __private {
    pub use linkme;
}
```

The re-export alone is not sufficient: linkme's *own* element expansion also
emits `linkme::…` paths. Every generated registration therefore carries
`#[linkme(crate = ::lid::__private::linkme)]`, linkme's wrapper-crate
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

Constraint 3's corollary applies here with force. If LTO, `--gc-sections`, or an
unusual target strips the section, the registry is empty and
`every_spec_has_an_implementer` passes trivially over nothing. Green build, zero
enforcement — the exact failure mode this system exists to prevent, reintroduced
by its own mechanism.

`lid` ships a known spec/implementation/validation triple internally.
`lid::canary::present()` returns false if any of them is missing from the
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

### 6.1 Why there is no syntactic rule

The tempting rule is "an untraced function may not call a traced one." Note the
direction: traced → untraced is fine and expected. Untraced → traced is the
suspicious edge.

But the strict version collapses. `#[implements]` becomes viral upward — every
caller of traced code must be traced, so `main` calls the composition root calls
the entry point, and `main` ends up carrying every spec in the crate. Any
workable version needs a bound, and needs an exemption list: `main`,
`#[cfg(test)]` fixtures, `From`/`Display` forwarding impls. It's also trivially
dodged, since `retry(|| authenticate(&c))` calls nothing traced — the closure
does. And enforcing it needs a call graph, meaning source parsing or rustc
internals: both constraints violated, for a rule that leaks anyway.

### 6.2 What actually holds the line

**Mutation answers the empirical version.** If mutating an untraced function
kills a `#[validates]` test, that function is causally on a spec-governed path
and should carry a citation. If nothing fails, it isn't participating, and
doesn't need one. Check 12 already produces this signal:

- *surviving mutant on untraced function* → not participating, correctly untraced
- *killed mutant on untraced function* → trace it, or move it behind a traced
  boundary

This is strictly better than the syntactic rule, because it tests *participation*
rather than the presence of a call. A function can call traced code incidentally
without carrying its behaviour; it can also carry spec behaviour through a
closure that no syntactic rule would see.

For untraced mutants, the xtask can't narrow the test set through the registry,
so it runs the enclosing module's validating tests, falling back to the full
suite. New untraced code is therefore gated: it may exist only if it either
breaks nothing when mutated, or is traced.

**Privacy does the containment.** The real risk isn't untraced code calling
traced code — it's untraced code *bypassing a dispatch node* to reach a leaf
directly. Rust prevents that already: `apply_display_name` is private, so only
its module can reach it, and that module carries the slice's claims. The version
of the rule that bites is across `pub` boundaries, and public functions should
carry citations anyway.

**Complexity bounds what an untraced function can be.** At complexity 1 it
contains no decision, so it cannot harbour an undeclared requirement by
construction. The explosion worth fearing is untraced *decisions*, and check 7
catches those regardless of tracing.

### 6.3 Module-level tracing

For a cluster of private helpers implementing one claim between them, a
module-level invocation traces by containment rather than per-function
ceremony:

```rust
// src/auth/password.rs
lid::implements_module!(spec::PasswordsAreVerifiedInConstantTime);
```

(A function-like macro rather than an inner attribute, because custom inner
attributes are not stable Rust; the emitted edge is identical, with
`module_path!()` supplying the containment.) Every function in the module
inherits the citation. Use this for a slice's
private machinery; use per-function citations at the public surface where
precision matters.
<!-- ANCHOR_END: traced -->

---

<!-- ANCHOR: configuration -->
## 7. Configuration

Three files, each the canonical home for one kind of setting. None of it lives
scattered through source attributes.

**`Cargo.toml`** — lint levels, workspace-wide (stable since 1.74):

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

[workspace.metadata.lid]
mutation_scope   = "diff"        # diff | full
untraced_fallback = "module"     # module | suite
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

**`[workspace.metadata.lid]`** — LID-rs's own knobs, read by the xtask.

`cognitive-complexity-threshold = 4` is a starting point, not scripture. The
number matters less than the invariant it protects: *a leaf should not branch*.
If you're raising it, check whether the code is genuinely irreducible or whether
writing the claim was merely inconvenient.
<!-- ANCHOR_END: configuration -->

---

<!-- ANCHOR: flow -->
## 8. The flow

Eight phases. Human authorship concentrates in 1 and 3; agent effort concentrates
in 6.

**Phase 0 — Name the slice.**
A user-visible operation, not a component. "User logs in", not "auth module".

**Phase 1 — Write the LLD.** *(human, agent drafts)*
Plain English in `docs/intent/<slice>/lld.md`, wired into the module docs. This
is the layer that cannot be recovered from code — rationale, rejected
alternatives, invariants that aren't type-expressible. Your time goes here.

**Phase 2 — Derive claims.** *(agent proposes, human approves)*
Agent emits `#[derive(Spec)]` items. Reject claims that are really two claims;
reject claims that restate the LLD rather than asserting something. Names should
read as sentences.

**Phase 3 — Layer-0 skeleton.** *(agent proposes, human approves)*
Slice entry point and its dispatch, signatures and `#[implements]` only,
`todo!()` bodies. `cargo check` must pass. **Review the signatures, not the
prose.** Because they type-check together, "these five functions compose into a
login flow" is verified rather than asserted.

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
against `todo!()`. Check 12 preserves this property for the life of the code;
Phase 5 establishes it.

**Phase 6 — Implement leaves.** *(agent, minimal review)*
The acceleration. Signature pinned, name pinned, claim cited, test red and
specific. The surrounding skeleton constrains what the function is *allowed to
be*, so the agent has almost no room to invent structure. Per §1.1, review here
is a small local semantic question.

**Phase 7 — Gate.** Run §4.5. Commit the slice.

**Phase 8 — Change.**
Every change is an LLD edit, cascaded: edit `lld.md` → re-derive affected specs →
rename or `#[deprecated]` the changed claims → `cargo check` names every citation
site to revisit. Deleting a spec breaks the build at every site that implemented
it.

**Slice seams.** At each boundary, reconcile before Phase 3: slice *n* often
wants something slice *n−1* built. Reuse only when the shared thing is one
concept, not a coincidence of shape — and never by adding a parameter to make two
behaviours fit one body. Check 8 catches the `bool` version; nothing catches the
`Option` version, so that one stays a human judgment.
<!-- ANCHOR_END: flow -->

---

<!-- ANCHOR: example-login -->
## 9. Worked example A — user login

### Phase 1 — LLD (`docs/intent/auth/lld.md`)

> Credentials enter at the login form, pass to the auth service, and resolve to
> one of two outcomes: a session scoped to the user, or a structured `AuthError`.
> The UI translates `AuthError` into a user-safe message; the session rides
> subsequent requests.
>
> Failure modes must be indistinguishable to the caller at the presentation
> layer: a wrong password and an unreachable credential store produce the same
> user-facing message, so account existence is not observable. They remain
> distinguishable in logs.

```rust
// src/lib.rs
#![doc = include_str!("../docs/intent/hld.md")]

#[doc = include_str!("../docs/intent/auth/lld.md")]
pub mod auth;
pub mod spec;

#[cfg(test)]
mod intent_graph;
```

### Phase 2 — claims

The three from §3.1. `BackendFailureIsIndistinguishableToUser` captures the
non-observability invariant — exactly the kind of thing that otherwise lives only
in someone's head and gets quietly violated by an agent writing a "helpful" error
message.

### Phase 3 — layer-0 skeleton

```rust
/// Resolves submitted credentials to a session, or a structured failure.
#[implements(
    spec::ValidCredentialsYieldScopedSession,
    spec::UnknownCredentialsAreRejected,
    spec::BackendFailureIsIndistinguishableToUser,
)]
pub fn authenticate(creds: &Credentials) -> Result<Session, AuthError> {
    let account = load_account(creds.username())?;
    verify_password(&account, creds.password())?;
    issue_session(&account)
}

/// Loads the account record for a username.
#[implements(
    spec::UnknownCredentialsAreRejected,
    spec::BackendFailureIsIndistinguishableToUser,
)]
fn load_account(username: &Username) -> Result<Account, AuthError> { todo!() }

/// Checks a submitted password against an account's stored verifier.
#[implements(spec::UnknownCredentialsAreRejected)]
fn verify_password(account: &Account, password: &Secret) -> Result<(), AuthError> { todo!() }

/// Mints a session scoped to the given account.
#[implements(spec::ValidCredentialsYieldScopedSession)]
fn issue_session(account: &Account) -> Result<Session, AuthError> { todo!() }
```

`cargo check` passes. What is already *proven* about a design containing no
implementation whatsoever:

- The three sub-operations compose into `authenticate` with no glue.
- `?` works throughout, so all three failures unify into `AuthError` — the LLD's
  "one structured error type" claim is enforced, not aspirational.
- `verify_password` borrows rather than consumes, so ordering with
  `issue_session` is fine.
- `Secret` is distinct from `String`, so a password can't be logged by accident.
  If it can, the type is wrong — a Phase 3 finding, before anyone wrote code.
- Every citation resolves. A typo in a spec path fails here, not in review.

`authenticate` has cognitive complexity 1: straight-line composition, no `match`,
no decision, nothing to declare.

### Phase 5 — failing-first

```rust
#[test]
#[validates(spec::ValidCredentialsYieldScopedSession)]
fn valid_credentials_yield_scoped_session() {
    assert_eq!(authenticate(&valid_creds()).unwrap().user_id(), 42);
}

#[test]
#[validates(spec::UnknownCredentialsAreRejected)]
fn wrong_password_is_rejected() {
    assert!(matches!(
        authenticate(&wrong_password()),
        Err(AuthError::InvalidCredentials)
    ));
}

#[test]
#[validates(spec::BackendFailureIsIndistinguishableToUser)]
fn backend_failure_is_opaque_downstream() {
    let err = authenticate(&store_down()).unwrap_err();
    assert!(matches!(err, AuthError::Backend(_)));
    assert_eq!(
        err.user_facing_message(),
        AuthError::InvalidCredentials.user_facing_message()
    );
}
```

Three claims, three registered validations, all red against `todo!()`. Checks 10
and 11 now pass; check 12 has a baseline.

### Phase 6–7 — implement and gate

Suppose the agent implements `load_account` as:

```rust
fn load_account(username: &Username) -> Result<Account, AuthError> {
    match self.store.fetch(username) {
        Ok(Some(a)) if a.is_active() => Ok(a),
        Ok(Some(a)) if a.locked_until().is_some() => Err(AuthError::Locked),
        Ok(Some(_)) => Err(AuthError::InvalidCredentials),
        Ok(None) => Err(AuthError::InvalidCredentials),
        Err(e) => Err(AuthError::Backend(e)),
    }
}
```

Check 7 fires: a leaf with complexity 5. And it's *correct* to fire — account
locking and activation status appear nowhere in the LLD and nowhere in the spec
module. `AuthError::Locked` is a fourth outcome the design never authorised, and
it leaks account state to the caller, violating
`BackendFailureIsIndistinguishableToUser`.

The fix isn't raising the threshold. It's returning to Phase 1, deciding whether
lockout is in scope, and if so writing the claim — at which point the branch is
declared, `load_account` becomes a dispatch node, and store access moves down a
layer into its own leaf.

**That is the whole point of the system.** The agent made a judgment call, and a
clippy threshold caught it in the same session, without a human reading the diff
carefully enough to notice a fourth error variant.
<!-- ANCHOR_END: example-login -->

---

<!-- ANCHOR: example-settings -->
## 10. Worked example B — applying a settings change

Chosen because it's genuinely dispatch-shaped, so it exercises the cascade and
the flag-argument rule.

### Phase 1 — LLD excerpt

> A settings change arrives as one of a closed set of change kinds. Each kind is
> validated against current account state, then applied. Validation and
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
entire purpose is expressing where control goes. Complexity stays low because
clippy's cognitive metric doesn't penalise flat `match` arms — which is exactly
why cognitive complexity is the right metric here and cyclomatic is not. A
twelve-arm dispatch is fine; three nested `if`s in a leaf is not.

### The cascade, concretely

Add a variant:

```rust
    /// Set the account's time zone.
    TimeZone(Tz),
```

`cargo check` fails at `apply` — non-exhaustive match. Not a lint, not an agent
pass, not a checklist: the build is broken until the new case is dispatched.
`wildcard_enum_match_arm` set to `deny` prevents defusing it with
`_ => Ok(Applied::NoOp)`. `#[non_exhaustive]` extends the same discipline to
downstream crates.

Then check 10 fires on the new spec until a handler implements it, and check 11
until a test validates it. One design edit propagates to three separate failures,
each naming the next thing to do.

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
— display names and schedules validate against completely different rules, and
`Field`/`Value` erase the types doing the real work.

Two clean leaves is the answer. The transaction boundary the LLD mentions is
genuinely shared and belongs in a wrapper around `apply`, not threaded through
the leaves.
<!-- ANCHOR_END: example-settings -->

---

<!-- ANCHOR: layout -->
## 11. Repo layout

```
Cargo.toml                     workspace lints + [workspace.metadata.lid]
clippy.toml                    thresholds
docs/
  intent/
    hld.md                     -> included by lib.rs
    auth/lld.md                -> included by mod auth
    settings/lld.md            -> included by mod settings
src/
  lib.rs                       doc includes
  intent_graph.rs              #[cfg(test)] — checks 10, 11, canary
  spec/
    mod.rs                     re-exports
    auth.rs                    claims for the auth slice
    settings.rs                claims for the settings slice
  auth/
    mod.rs                     dispatch + entry points + #[cfg(test)] validations
    account.rs                 leaves
  settings/
    mod.rs
  bin/
    app.rs                     thin — doctests don't run in bin targets
tests/
  ...                          integration tests; no #[validates] here (§5.2)
lid/                           support crate: Spec, Edge, slices, canary
lid-macros/                    derive(Spec), implements, validates, spec
xtask/                         registry-driven mutation scoping
.github/workflows/gate.yml
```

Specs live in `src/`, not `docs/`, because they must be items the compiler
resolves. LLDs live in `docs/` because they're prose meant to be diffed and
argued over in a PR — but they render inside `cargo doc`, so a reader never goes
looking for them.

**Brownfield adoption.** Layer the tiers in order. Tier 0 first — the lints apply
to existing code immediately and will surface every leaf that's secretly a
dispatch node. Then write LLDs for the slices you're actively changing, and let
`#[implements]` spread through the code you touch rather than in a big-bang pass.
Checks 10 and 11 only ever assert over specs that exist, so a partially-traced
codebase gates correctly on the part that's traced.
<!-- ANCHOR_END: layout -->

---

<!-- ANCHOR: limits -->
## 12. Honest limits

- **The semantic residual.** Per §4.4, a test can cite the wrong claim and pass
  every gate. This is by design (§1.1), but it means the differential pass is
  load-bearing and needs a scheduled owner, not good intentions.
- **`linkme` has platform edges.** See §5.4. The canary converts silent failure
  into loud failure, but on an unusual target you will be debugging a linker
  mechanism. The `inventory` fallback exists for that case.
- **Two pieces are unbuilt.** The `lid`/`lid-macros` crates, and the xtask that
  reads the registry to narrow the mutation test subset. That filtering is what
  keeps check 12 inside a per-PR budget; without it, mutation runs the full suite
  per mutant.
- **Module-level `implements_module!` is coarse.** It traces by containment, so a
  module that grows past its original claim will carry a citation that's become
  approximate. Treat module size as the check on this — nothing enforces it.
- **`cognitive_complexity` is a nursery lint.** It has known false positives and
  its behaviour can change between clippy releases. Pin the toolchain in CI.
<!-- ANCHOR_END: limits -->

---

<!-- ANCHOR: bootstrap -->
## 13. Bootstrap checklist

1. `cargo new --lib` plus a thin `bin`.
2. `lid` support crate: `Spec` trait, `Edge`, `SpecMeta`, three
   `#[distributed_slice]` declarations, `canary`, `__private` re-export.
3. `lid-macros`: `derive(Spec)`, `implements`, `validates`,
   `implements_module!`, `spec`.
4. `Cargo.toml` workspace lints + `clippy.toml` thresholds (§7).
5. `docs/intent/hld.md`, included via `#![doc = include_str!(...)]`.
6. `src/spec/mod.rs` with `//!` docs explaining what the module is for.
7. `src/intent_graph.rs` under `#[cfg(test)]` — canary first, then checks 10, 11.
8. `[profile.test] opt-level = 0`; install `cargo-mutants`; `xtask` for
   registry-driven test filtering.
9. CI running §4.5 in order, all of it gating.
10. `AGENTS.md` / `CLAUDE.md` stating the eight phases and the dispatch-vs-work
    rule, so the agent proposes skeletons rather than implementations.
11. First slice end to end before writing a second LLD. The phase boundaries are
    where the tedium hides; find out where it bites on one slice before
    committing to the shape.
<!-- ANCHOR_END: bootstrap -->
