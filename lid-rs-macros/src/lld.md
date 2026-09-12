# Citation macros — the attribute surface of the intent graph

## Context and Design Philosophy

Slice 1 proved the registration form by hand; this slice makes it the output
of macros so a citation is one attribute, not eleven lines of ceremony. The
macros are syntax sugar over an already-proven expansion — they must reproduce
the hand-expanded form from `lid-rs/src/registry/lld.md` exactly, and they
add no semantics of their own. Everything they emit is checked by machinery
that already exists: path resolution by the compiler, doc links by rustdoc,
registration by the linker, graph coverage by the (slice 3) intent-graph
tests.

`lid-rs-macros` is a proc-macro crate: it executes at compile time on the host,
links into no target binary, and therefore can neither carry citations nor
register anything itself. Its behaviour is specified by claims in
`lid_rs::lid_rs_macros::spec`, cited by hand-authored edges at the re-export site in
`lid-rs` — the one sanctioned exception to macro-written citations, granted here
because a proc-macro crate has no binary for a citation to register into —
and validated by `lid-rs`'s tests, which are downstream of the macros and can
expand them.

## The eight macros

| Macro | Kind | On | Emits |
|---|---|---|---|
| `Spec` | derive | unit struct | `impl lid_rs::Spec` with `NAME = concat!(module_path!(), "::", ident)`; a `SPECS` registration in a `const _` block |
| `implements(path, …)` | attribute | fn (incl. methods), struct, enum, impl-free items generally | per cited spec: a doc line `Implements [\`path\`].` appended to the item's docs, and an `IMPLEMENTATIONS` edge |
| `validates(path, …)` | attribute | `#[cfg(test)]` unit test fns | same shape, doc line `Validates [\`path\`].`, edge into `VALIDATIONS` |
| `implements_module!(path, …)` | function-like | invoked inside a module | `IMPLEMENTATIONS` edges whose `item` is the enclosing `module_path!()` — module-level tracing by containment |
| `spec("FOREIGN-ID")` | attribute | spec struct | re-emits the struct with `#[doc(alias = "FOREIGN-ID")]` |
| `Outcome` | derive | enum | `impl lid_rs::Outcome`, and an `OUTCOMES` registration per variant (`lid-rs/src/outcome/lld.md`) |
| `flow` | attribute | fn (free, inherent, trait declaration, trait impl) | the item's tokens unchanged — a pin `lid-rs-shape` reads from source |
| `leaf` | attribute | the same fn positions | the item's tokens unchanged — the same |

### Why module-level tracing is a function-like macro

The natural spelling would be an inner attribute, `#![implements(…)]`, but
custom inner attributes are not stable Rust, and attribute proc-macros on
non-inline `mod foo;` items are not stable either — both forms fail
constraint 1. An ordinary macro invocation inside the module body is stable
and expands to the identical edge, with `module_path!()` supplying
containment.

### Placement of emitted registrations

For fn items the registrations are **injected into the body** as leading
`const _` statements — one per cited spec, since two statics named `EDGE`
cannot share one scope. Body injection is what makes methods work: `impl`
blocks admit no free `const _` siblings, but every fn body admits items. For
non-fn items (structs, enums) the registrations are emitted as siblings, which
is legal at module scope where those items live. The doc lines are appended to
the item's attribute list in both cases, preceded by one empty doc line for a
paragraph break.

### Argument validation

Errors are reported by `syn::Error` at the offending span, before linkme sees
anything malformed (README [§5.4](https://bradvoth.github.io/lid-rs/spec/registry.html)): an empty citation list ("cite at least one
spec, or remove the attribute"), non-path arguments, paths with generic
arguments. Path rendering for doc lines joins segments with `::` from the
parsed `syn::Path`, never from token-stream stringification (which inserts
spaces).

### Citing an `async fn` is refused

Both citation attributes wrap a fn's body, and both wrappings assume a body
that runs to completion where it is written. `#[implements]` opens a span and
holds its guard, `__lid_entered`, inline around the body; `#[validates]` runs
the body inside a closure under a fresh subscriber. On an `async fn` the body
is not run — it becomes a future. The guard would then be held across every
`.await` in it, and every unrelated future polled while the executor has the
thread would be attributed to the cited claim. The trace is not incomplete, it
is wrong, and checks 23–24 read it as fact.

So `cite_fn` refuses before it wraps anything. Its first act is `refuse_async`,
which reads the signature's `asyncness` and, when it is present, answers a
`syn::Error` spanned on the `async` keyword itself, so the diagnostic
underlines the token that has to go. The message is:

```text
lid-rs: spans across `.await` are not supported yet (README §6.8) — cite a synchronous fn
```

`cite_fn` therefore answers `syn::Result<TokenStream>` and `citation`
propagates with `?`, which is where the error becomes
`syn::Error::into_compile_error` at the macro boundary like every other
citation error.

The refusal does not depend on the verb. A `#[validates]` async test whose body
awaits already fails to compile — `.await` inside the capture's non-async
closure — but it fails with a diagnostic about the closure, pointing at the
user's `await` rather than at the citation that caused it, and an async test
body with no `.await` in it wraps correctly and would slip through. One
unconditional refusal gives both verbs the same sentence; a verb-dependent one
would put a decision inside `refuse_async`, and a decision is a claim nobody has
written.

Async tracing itself is not this slice's to build. It stays deferred where
`lid-rs/src/validate/lld.md` keeps it — Deferred 1 (`lid_rs::spawn` and
propagation across `await`) and Deferred 2 (the root-keyed global capturing
layer) — and this refusal is what keeps that absence visible until a slice
closes it, per README [§6.8](https://bradvoth.github.io/lid-rs/spec/traced.html).

### The shape pins

`#[flow]` and `#[leaf]` are the marker attributes README
[§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html) names. `#[flow]`
declares that a function must stay routing — the check that fails when it
acquires work, rule P, is a later change (Deferred 2), so today the mark is a
declaration nothing compares; `#[leaf]` is rule B's escape, marking a public
function whose work is intended, so that it is allowed and counted. Neither mark changes a
classification: the classification is derived from the body, and a mark that
could change it would be an opt-out of the derivation.

They are proc-macro attributes because a mark has to be legal Rust wherever a
function is written and has to survive into the source file `lid-rs-shape`
parses. `passthrough_pin` is their whole expansion: it emits the item's tokens
unchanged, never parsing the item, so what the compiler sees is what the author
wrote. The mark's *meaning* is read syntactically by `lid-rs-shape`, which
matches the attribute path (`lid-rs-shape/src/verdict.rs`, `marked_leaf`) and
never sees an expansion at all.

Neither pin takes arguments. A non-empty argument list is a `syn::Error` at the
arguments' span — "lid-rs: `#[flow]` and `#[leaf]` take no arguments" — because
nothing reads them, and an argument accepted and ignored is a mark whose
meaning the next reader has to guess.

Three answers the pins owe, established by spike and recorded here: they are
re-exported from `lid-rs`'s crate root alongside the citation macros, so
consumer code writes `use lid_rs::{flow, leaf};` and never names
`lid-rs-macros`; they are accepted on free functions, inherent methods, trait
method declarations and trait-impl methods, all four of which compile on stable
with a pass-through attribute macro; and they are not accepted on closures,
where `custom attributes cannot be applied to expressions` (E0658) puts them
outside constraint 1. A closure is classified by the function containing it.

## Equivalence: pin, then swap

The macros' conformance to the hand-expanded form is asserted behaviorally,
in two steps that live in git history:

1. **Pin** (tests-first phase): registry-content tests assert the exact
   observable state produced by the hand expansions — every `Spec::NAME`
   string, every edge's `spec` and `item` field. Green against the
   hand-expanded canary.
2. **Swap** (implementation phase): the hand expansions in `lid-rs` are replaced
   by the macros — canary triple, spec structs, module edge. The pin tests
   must stay green unchanged. Any divergence in what the macros emit is a
   test failure naming the field that moved.

`file`/`line` fields are left outside the pin: they change with the
swap and carry no contract.

## Compile-failure demonstrations (check 1, Goal 3)

The trybuild harness is itself a `#[validates]` unit test inside the
library (`lid-rs/src/lid_rs_macros/mod.rs`), so its edge links into the registry
per README [§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html); only the
fixture files live under `lid-rs/tests/ui/`:

- **fail** (7): citing a path that doesn't resolve; citing a type that isn't
  a `Spec`; empty citation list; generic-argument path; deriving `Spec` on a
  non-unit struct; citing an `async fn` (`async_implements.rs`); giving an
  argument to a pin (`pin_with_args.rs`).
- **pass** (8): a fn citing a real spec through a `use` rename (the case grep
  cannot handle and the compiler must); `#[validates]` coexisting with
  `#[test]`; a struct and an enum carrying `#[implements]`; a method carrying
  it inside an impl block; `implements_module!` tracing; `#[spec("…")]` alias
  emission; a `#[deprecated]` spec whose definition site stays warning-free; a
  `#[flow]` fn and a `#[leaf]` fn compiling unchanged (`pins.rs`) — the
  fixture's `main` calls both and asserts their answers, since a pass fixture
  is run as well as compiled, and "unchanged" is observable only as behaviour.

A fail fixture's `.stderr` is where the refusal messages above are pinned: the
claim is that compilation stops, and the recorded diagnostic is what makes the
sentence and the span it underlines observable.

## Shape

The citation path is described by the behaviour above rather than by row; these
are the items the async refusal and the pins are made of.

| Item | Role |
|---|---|
| `refuse_async(sig: &syn::Signature) -> syn::Result<()>` | Leaf, in `expand.rs`. Answers the refusal error spanned on `sig.asyncness`, or `Ok(())`. |
| `cite_fn(f: ItemFn, paths: &[Path], verb: &Verb) -> syn::Result<TokenStream>` | The cited-fn expansion, answering a `Result` so that `refuse_async`'s error reaches `citation`'s `?` instead of being swallowed at a wrapping that has already happened. |
| `passthrough_pin(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream>` | Leaf, in `expand.rs`. The whole expansion of both pins: the arguments' error when `args` is non-empty, the item's tokens unchanged otherwise. |
| `flow`, `leaf` | The two exported `#[proc_macro_attribute]` entry points in `lib.rs`, each the four-line wrapper over `passthrough_pin` that every macro in this crate is over its expansion. |

## What lands by hand

`lid-rs-macros` is a proc-macro crate, so a phase of this slice executes this
slice's own code at every `cargo check`, with the session's privileges. The
phase policy refuses a compile-time slice until the human has committed an
acceptance file beside its LLD — here `lid-rs-macros/src/compile-time-accepted`,
the crate-root form of the `lid-rs-macros/src/claim/compile-time-accepted` the
claim slice carries (README
[§12](https://bradvoth.github.io/lid-rs/spec/limits.html)). It is a human
commit, never an agent's write; that is the whole of its value.

The trybuild fixtures are the second hand commit, and it precedes Phase 5:
`fail/async_implements.rs` and `fail/pin_with_args.rs` with their `.stderr`
files, and `pass/pins.rs`. The harness globs its two directories, so a citation
whose fixtures are not yet on disk is a vacuous green and Phase 5 could not show
red.

The three new claims are validated by the harness test that already exists,
`malformed_citations_fail_to_compile` in `lid-rs/src/lid_rs_macros/mod.rs`,
whose citation list Phase 5 extends with `AsyncCitationsFailToCompile`,
`PinsRefuseArguments` and `PinsExpandToTheirItemUnchanged`. One `TestCases`
over the two globs is the whole proof; a second harness would compile every
fixture twice and prove nothing the first did not.

## What Phase 3 writes in the companion

The companion's Phase 3 and 4 rows admit `lid-rs/src/lib.rs`
(`cargo-lid-rs/src/phase/policy.rs`, the companion table); Phases 5 and 7 do
not. So two things are Phase 3's writes there, not hand commits: the re-export
`pub use ::lid_rs_macros::{flow, leaf};` beside the citation macros' re-export,
and three `macro_edge!` lines in the block that carries this slice's
implementation edges — `AsyncCitationsFailToCompile` naming
`"lid_rs_macros::expand"`, and `PinsExpandToTheirItemUnchanged` and
`PinsRefuseArguments` each naming `"lid_rs_macros::expand"`, the module whose
`passthrough_pin` and `refuse_async` are the implementers. A claim whose edge is
written later than Phase 4 fails check 10 at Phase 7 on a file that phase may
not open.

## Sequence: pin, then red, then wire

A `todo!()` reachable from a citation's expansion panics every citation in the
workspace, and Phase 3's own `cargo check` stops on the first. And an
unreached function cannot wait either: `expand` is a private module, so a
`refuse_async` nothing calls is dead code, and the lint that denies warnings
refuses the skeleton before the check runs. So `refuse_async` is the one
layer-0 item whose skeleton is not `todo!()`: Phase 3 wires it into `cite_fn`
as that function's first act — `cite_fn` answering `syn::Result` and
`citation` propagating with `?` — with a body that answers `Ok(())` and
refuses nothing, a wrong answer the fail fixture observes. `flow` and `leaf`
are wired to `passthrough_pin` from the start with the ordinary `todo!()`,
because nothing in the workspace carries a pin until the fixtures do.

Phase 5 is then red without a hand swap: `fail/async_implements.rs` compiles
under a refusal that refuses nothing, which the harness reports as a
compile-fail fixture that did not fail; `pass/pins.rs` and
`fail/pin_with_args.rs` reach the `todo!()` in `passthrough_pin` and fail on
the panic rather than on the pinned `.stderr`.

Phase 7 implements the three leaves; `refuse_async`'s body is the refusal
spanned on `sig.asyncness`, and nothing else changes shape. `expand.rs` is
this slice's own row at every phase.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Equivalence assertion | Behavioral pin-then-swap over registry contents | `macrotest`/`cargo-expand` textual expansion comparison | Textual expansion needs nightly `-Zunpretty`, violating constraint 1. The registry is the macros' observable output, so pinning it asserts exactly what matters and nothing incidental. |
| fn registration placement | Body injection | Sibling `const _` after the fn; associated `const _` in impl blocks | Sibling emission breaks for methods (`impl` blocks reject free consts); body injection is uniform everywhere a fn can appear. Slice 1 proved linkme accepts scoped statics. |
| `#[spec]` foreign-key alias | Standalone attribute macro | Derive helper attribute read by `derive(Spec)` | A derive cannot attach `#[doc(alias)]` to its item — derives only append new items. An attribute macro rewrites the item, which is the whole job. |
| Module-level tracing | `implements_module!(…)` function-like macro | `#![implements(…)]` inner attribute; attribute on `mod` declarations | Both alternatives are unstable Rust (custom inner attributes; attributes on non-inline modules). |
| Doc-line emission is design prose, not a spec claim | Un-claimed, documented here | A `CitationsRenderAsDocLinks` claim | No stable mechanism observes an item's rendered docs from a gating test; a claim that cannot gate gets deleted (constraint 3). rustdoc's link check still gates *resolvability* of whatever doc lines exist. Revisit if rustdoc JSON output stabilizes. |
| `#[spec]` searchability un-claimed | UI pass-test for compilation only | A greppability/search claim | Same constraint-3 honesty: `doc(alias)` affects rustdoc search, which no gate can observe on stable. |
| Macro dependencies | `syn` (features `full`), `quote`, `proc-macro2` | Hand-rolled token matching | `full` is needed to parse fn items for body injection; the trio is the floor for attribute macros that rewrite items, not an escalation (tenet 3). `trybuild` added as dev-dependency of `lid-rs` — compile-failure assertion is impossible without it. |
| Citing an `async fn` | Refused at expansion by `refuse_async`, identically for both verbs: a `syn::Error` on the `async` keyword naming README §6.8. Claim Phase 2 adds to `lid-rs/src/lid_rs_macros/spec.rs`: `AsyncCitationsFailToCompile`. | Delegate to `#[tracing::instrument]`, which already handles `async`; keep the citation and enter the span through `tracing::Instrument` on the returned future; leave the expansion silent and the guard held across every `.await`; refuse `#[implements]` only and leave `#[validates]` as it is | The guard is held inline around the body, so on a future it spans every `.await` and attributes every unrelated future polled in the interval to the cited claim — a wrong trace, which checks 23–24 then report as fact, and which no gate can see. A refusal costs the author one keystroke; a wrong trace costs the reader's confidence in every trace. `instrument` restores the `tracing-attributes` feature and moves the field set out of this slice's control, and neither it nor `Instrument` can be reached without the async runtime `lid-rs/src/validate/lld.md` Deferred 1 is waiting on. Refusing one verb only buys a better message for a case the compiler already rejects, at the price of a decision inside `refuse_async` — a branch, therefore a claim, therefore not free. |
| How `refuse_async` is skeletoned | Wired into `cite_fn` at Phase 3 with a body answering `Ok(())`, so the fail fixture is red on a refusal that refuses nothing | Declared unwired with a `todo!()` body and wired at Phase 7; wired with a `todo!()` body; `#[allow(dead_code)]` until it is wired | An unwired function in the private `expand` module is dead code, which the post-edit lint denies, so Phase 3 could not land it. A wired `todo!()` panics every citation in the workspace, so Phase 3's own check could not pass. An `#[allow]` is the suppression this workspace never writes. A wrong answer that compiles is what a skeleton is for: Phase 5's `fail/async_implements.rs` fails to fail against it, and Phase 7 writes the one body. The cost is one layer-0 item whose skeleton is a value rather than a panic, recorded here so the exception is the document's and not the phase's. |
| Where `#[flow]` and `#[leaf]` live | Attribute macros exported by `lid-rs-macros` and re-exported from `lid-rs`'s crate root, each a pass-through over `passthrough_pin` that accepts no arguments and emits its item unchanged. Claims Phase 2 adds to `lid-rs/src/lid_rs_macros/spec.rs`: `PinsExpandToTheirItemUnchanged`, `PinsRefuseArguments`. | Define the pins in `lid-rs` as `macro_rules!`; make `#[leaf]` a `#[cfg_attr(any(), …)]` no-op or a bare `#[doc]` convention; wait for rule P and land the pins with the check that reads them | A `macro_rules!` macro cannot be written in attribute position, so that alternative does not exist in the language. A `cfg_attr` no-op is not a name the compiler resolves, so `#[laef]` is silent — and a pin whose misspelling is silent inverts its own purpose, which is that check 17 fails on what the mark names. Waiting for rule P inverts the dependency instead: `lid-rs-shape` reads `leaf` today (`verdict.rs`, `marked_leaf`) against an attribute nothing exports, so the mark is unwritable until the pins exist, and rule B's escape has no escape. A pass-through is the least that makes the mark legal Rust, keeps it in the source the shape pass parses, and leaves no expansion for a later check to reason around. |
| Item-kind dispatch | Sequential parse-attempts (`ItemFn`, `ItemStruct`, `ItemEnum`) | `match` over `syn::Item` | `syn::Item` is foreign and `#[non_exhaustive]`, so it cannot be matched without the wildcard arm check 6 denies; caught by our own gate. Reject-by-default for unknown item kinds is the wanted behaviour, and parse-attempts express it without a wildcard. |

## Open Questions & Future Decisions

### Resolved
1. ✅ linkme's *element* expansion does **not** resolve in downstream crates
   that lack a direct linkme dependency — proven when `xtask` failed to
   compile. Every generated registration carries
   `#[linkme(crate = ::lid_rs::__private::linkme)]`; the contract lives in
   `lid-rs/src/registry/lld.md`.

### Deferred
1. Observing emitted doc lines mechanically (rustdoc JSON is unstable today).
2. **Rule P (check 17), the pin read against the classification.** The pins
   land inert: nothing compares a `#[flow]` mark to the shape the body derives,
   so a marked function that acquires work is caught by no check. Rule P is a
   later change of *this* slice and not of `lid-rs-shape`, whose phases may
   write no path under `lid-rs-macros` (`lid-rs-shape/src/lld.md`, "The pins
   are a slice of another crate"). It needs two things this document does not
   add, and they are two changes in two slices, in this order: first a
   Phase 8 change of `lid-rs-shape` so its verdict carries `flow` as it
   carries `leaf` (`lid-rs-shape/src/verdict.rs` is that slice's row and no
   phase of this one may write it); then a change of this slice adding rule P's
   claim with an emitted validator — a fixture function pinned `#[flow]` whose
   body fails an F-rule, which the check must report, beside a pinned flow body
   that it must not.

## References

- `lid-rs/src/registry/lld.md` — the expansion contract this slice must
  reproduce; its doctest keeps the hand form compiling.
- README [§3.3](https://bradvoth.github.io/lid-rs/spec/mapping.html) (citation anatomy), [§5.4](https://bradvoth.github.io/lid-rs/spec/registry.html) (macro error surfacing), [§6.3](https://bradvoth.github.io/lid-rs/spec/traced.html)
  (module-level tracing).
