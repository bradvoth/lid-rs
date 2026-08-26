# Citation macros — the attribute surface of the intent graph

## Context and Design Philosophy

Slice 1 proved the registration form by hand; this slice makes it the output
of macros so a citation is one attribute, not eleven lines of ceremony. The
macros are syntax sugar over an already-proven expansion — they must reproduce
the hand-expanded form from `lid-rs/docs/intent/registry/lld.md` exactly, and they
add no semantics of their own. Everything they emit is checked by machinery
that already exists: path resolution by the compiler, doc links by rustdoc,
registration by the linker, graph coverage by the (slice 3) intent-graph
tests.

`lid-rs-macros` is a proc-macro crate: it executes at compile time on the host,
links into no target binary, and therefore can neither carry citations nor
register anything itself. Its behaviour is specified by claims in
`lid_rs::spec::citation`, cited by hand-authored edges at the re-export site in
`lid-rs` — the one sanctioned exception to macro-written citations, granted here
because a proc-macro crate has no binary for a citation to register into —
and validated by `lid-rs`'s tests, which are downstream of the macros and can
expand them.

## The five macros

| Macro | Kind | On | Emits |
|---|---|---|---|
| `Spec` | derive | unit struct | `impl lid_rs::Spec` with `NAME = concat!(module_path!(), "::", ident)`; a `SPECS` registration in a `const _` block |
| `implements(path, …)` | attribute | fn (incl. methods), struct, enum, impl-free items generally | per cited spec: a doc line `Implements [\`path\`].` appended to the item's docs, and an `IMPLEMENTATIONS` edge |
| `validates(path, …)` | attribute | `#[cfg(test)]` unit test fns | same shape, doc line `Validates [\`path\`].`, edge into `VALIDATIONS` |
| `implements_module!(path, …)` | function-like | invoked inside a module | `IMPLEMENTATIONS` edges whose `item` is the enclosing `module_path!()` — module-level tracing by containment |
| `spec("FOREIGN-ID")` | attribute | spec struct | re-emits the struct with `#[doc(alias = "FOREIGN-ID")]` |

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
library (`lid-rs/src/spec/mod.rs`), so its edge links into the registry per
README [§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html); only the
fixture files live under `lid-rs/tests/ui/`:

- **fail** (5): citing a path that doesn't resolve; citing a type that isn't
  a `Spec`; empty citation list; generic-argument path; deriving `Spec` on a
  non-unit struct.
- **pass** (7): a fn citing a real spec through a `use` rename (the case grep
  cannot handle and the compiler must); `#[validates]` coexisting with
  `#[test]`; a struct and an enum carrying `#[implements]`; a method carrying
  it inside an impl block; `implements_module!` tracing; `#[spec("…")]` alias
  emission; a `#[deprecated]` spec whose definition site stays warning-free.

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
| Item-kind dispatch | Sequential parse-attempts (`ItemFn`, `ItemStruct`, `ItemEnum`) | `match` over `syn::Item` | `syn::Item` is foreign and `#[non_exhaustive]`, so it cannot be matched without the wildcard arm check 6 denies; caught by our own gate. Reject-by-default for unknown item kinds is the wanted behaviour, and parse-attempts express it without a wildcard. |

## Open Questions & Future Decisions

### Resolved
1. ✅ linkme's *element* expansion does **not** resolve in downstream crates
   that lack a direct linkme dependency — proven when `xtask` failed to
   compile. Every generated registration carries
   `#[linkme(crate = ::lid_rs::__private::linkme)]`; the contract lives in
   `lid-rs/docs/intent/registry/lld.md`.

### Deferred
1. Observing emitted doc lines mechanically (rustdoc JSON is unstable today).

## References

- `lid-rs/docs/intent/registry/lld.md` — the expansion contract this slice must
  reproduce; its doctest keeps the hand form compiling.
- README [§3.3](https://bradvoth.github.io/lid-rs/spec/mapping.html) (citation anatomy), [§5.4](https://bradvoth.github.io/lid-rs/spec/registry.html) (macro error surfacing), [§6.3](https://bradvoth.github.io/lid-rs/spec/traced.html)
  (module-level tracing).
