# The registry is readable as a committed page

## Context and Design Philosophy

HLD row 21 is "The intent graph is readable as a page", and it names two
products in one row (`lid-rs/docs/intent/hld.md:200`):

> `regen` and `docs/intent/trace.md` with check 26; `lid-rs-site`: claim cards,
> the observed flow graph, the trace matrix, the glossary

**The row is split, and this slice is its first half.** `trace` is a module
slice in `lid-rs` at `lid-rs/src/trace/`, delivering the generator, the
committed document, and check 26 — and nothing else. `lid-rs-site` becomes a
slice of its own (Deferred 3).

Two reasons, recorded because the split is a scope decision and not a
discovery. The first is that it unblocks slice 18 soonest: the shape pass ships
with a `warn` ramp whose legitimacy README conditions on committed counts, and
slice 18's own document says the counts have no home
(`lid-rs-shape/src/lld.md:185-193`). The second is that the site half needs
things that do not exist. §11.2 makes a claim card show "the validator's
captured trace as the worked example", which is slice 20's and unbuilt; and the
glossary's vocabulary ledger is circular with this document — the vocab slice
defers it to "the trace document slice 21 builds"
(`lid-rs-macros/src/vocab/lld.md:281-283`) while this document would be reading
a ledger nobody has written.

**What that leaves is a document generated from the registry alone, and the
argument this slice turns on is whether such a document is worth a check.** The
HLD anticipated the objection:

> the page (21) needs everything it renders, and check 26 compares `trace.md`
> against the registry *and* the shape pass, so a registry-only document would
> ship a check gating half of what it names (`hld.md:219-221`)

That is true of a document that *contains* shape counts and gates only the
registry half. It is not true of the document below, which contains no shape
counts: check 26 gates every line of what it holds, and nothing in it is
ungated. The difference is the whole of Decision 1, and README §11.2 gets an
amendment rather than a divergence.

## Facts, verified 2026-09-12

Every row was read from the tree on this branch (`lld/trace`, stacked on
`lld/outcome` at `9527253`), or produced by running the command named.

| Fact | Where |
|---|---|
| `docs/intent/trace.md` does not exist anywhere in the tree; neither does `lid-rs/src/trace/`, nor a `lid-rs-site` crate | `find`, workspace-wide |
| Check 26, Tier 1: "`docs/intent/trace.md` doesn't match the registry and the shape pass" | `README.md:629` |
| §11.2: "one section per slice, one row per claim, with links, plus the shape and conformance `warn` counts — generated from the registry and the shape pass, committed, and freshness-gated (check 26)" | `README.md:1847-1852` |
| The gate runs check 26 under `cargo test --lib` | `README.md:792` |
| The knob `trace = "docs/intent/trace.md"` is declared in `[workspace.metadata.lid_rs]` | `README.md:1200` |
| §3.7 conditions the shape `warn` ramp on counts committed in `trace.md` | `README.md:565-571` |
| §11.1 puts the document under `docs/intent/`, "generated, committed, freshness-gated" | `README.md:1761` |
| §11.1's placement rule: "In a workspace, an intent document lives inside the package root of the crate that includes it" | `README.md:1806-1809` |
| Constraint 3: "Every check gates, or it gets deleted. A report nobody reads is worse than an absent check" | `README.md:160` |
| `regen` is specified as `cargo test --lib -- --ignored regen`, writing `docs/intent/trace.md`, running no check | `lid-rs-pipeline/docs/intent/pipeline.md:226` |
| `graph` is specified as `cargo test --lib intent_graph::graph`, covering "canary, 10, 11, 26", reading `trace.md` | `lid-rs-pipeline/docs/intent/pipeline.md:222` |
| No emitted test's name matches the filter `intent_graph::graph` today: the four are `registry_is_populated`, `every_spec_has_an_implementer`, `every_spec_has_a_validation`, `registry_dump_for_tooling` | `lid-rs/src/graph/mod.rs:79,87,100,117` |
| `intent_graph!()` is a `macro_rules!` in the **`graph`** slice's module, and emits four `#[test]`s | `lid-rs/src/graph/mod.rs:76-131` |
| The graph slice's standing rule: emitted tests are "Plain `#[test]`s, uncited" | `lid-rs/src/graph/lld.md`, Decisions table |
| `graph_orphans` refuses over a stripped registry, returning `Err(CanaryStripped)`; `CanaryStripped` is `pub` | `lid-rs/src/graph/mod.rs:16-42` |
| Checks are crate-scoped by `Spec::NAME` prefix, from `env!("CARGO_CRATE_NAME")` at the invocation site | `lid-rs/src/graph/mod.rs:54,88` |
| Five workspace members invoke `intent_graph!()`: `lid-rs`, `cargo-lid-rs`, `lid-rs-shape`, `lid-rs-pipeline`, `xtask` | `lid-rs/src/lib.rs:22`, `cargo-lid-rs/src/lib.rs:61`, `lid-rs-shape/src/lib.rs:331`, `lid-rs-pipeline/src/lib.rs:638`, `xtask/src/lib.rs:30` |
| `lid-rs-macros` invokes none: a proc-macro crate links into no binary, and its claims register from `lid-rs` | `lid-rs-macros/Cargo.toml:15`, `lid-rs/src/lib.rs:44-75` |
| `#[validates]` edges are `#[cfg(test)]` and exist only in their home crate's test binary; the registry is otherwise binary-global | `README.md:659-666`, `lid-rs/src/graph/lld.md:17-25` |
| Registered claims by crate: `lid-rs` 111, `cargo-lid-rs` 316, `lid-rs-shape` 25, `lid-rs-pipeline` 17, `xtask` 1 | `grep -rc "derive(Spec)"` over each `src` |
| Marked `#[lid(free)]`: `lid-rs` 27, `cargo-lid-rs` 280, `lid-rs-shape` 1, `lid-rs-pipeline` 0, `xtask` 1 | the same, on `lid(free)` |
| `SpecMeta` is `{ name, file, line, claim: ClaimMeta }` — no sentence, no crate field, no free field of its own | `lid-rs/src/registry/mod.rs:5-22` |
| `ClaimMeta` carries `language, pattern, trigger, verb, negated, object, owner, templates` — parts, never the sentence | `lid-rs/src/claim/mod.rs:53-81` |
| A free claim carries `Language::Free` **and empty parts** | `lid-rs/src/registry/mod.rs:16-20`, `lid-rs/src/claim/mod.rs:40-47` |
| The free mark **is** on the registered struct: `is_free(meta)` is `meta.claim.language == Language::Free`, and `claim::free()` already enumerates them | `lid-rs/src/claim/mod.rs:82-96` |
| `SpecMeta.file` is **workspace-root-relative** — `lid-rs/src/graph/mod.rs`, not `src/graph/mod.rs` | live `LID_DUMP=1 cargo test -p lid-rs --lib registry_dump_for_tooling -- --nocapture` |
| `Spec::NAME` is `module_path!()` plus the item's identifier, so a claim's name spells crate, slice and `spec` | `lid-rs/src/lib.rs`, `Spec::NAME` doc; confirmed in the dump (`lid_rs::graph::spec::CoveredGraphsPassTheGraphCheck`) |
| `lid-rs` depends on exactly `lid-rs-macros` and `linkme`, with `trybuild` as a dev-dependency | `lid-rs/Cargo.toml:13-19` |
| `lid-rs-shape` depends on `lid-rs` | `lid-rs-shape/Cargo.toml:30` |
| A normal dependency the other way is refused: `error: cyclic package dependency` | spiked, `cargo metadata` exit 101, scratch workspace |
| A **dev**-dependency cycle is accepted | spiked, `cargo metadata` exit 0, same workspace |
| **Nothing in the workspace depends on `lid-rs-shape`** — no `lid_rs_shape` identifier appears in any file outside that crate | `grep -rn lid_rs_shape` over `*.rs`/`*.toml` |
| `lid-rs-shape` "derives no serialisation of its own — it knows no workspace path, writes no file" | `lid-rs-shape/src/lib.rs:56-63` |
| `cargo lid-rs shape` has no dispatch arm; the arms are `mutants, init, new, sync, phase-check, lld-check, canopy, coach, hook` | `cargo-lid-rs/src/lib.rs:41-56` |
| The catalog's dispatch arm is deferred to the crate-root slice | `cargo-lid-rs/src/catalog/lld.md:271-276` |
| The catalog marks `site` unbuilt on "slice 21", and `regen`/`graph` unbuilt on its own Deferred 1 | `cargo-lid-rs/src/catalog/mod.rs:212-224` |
| `#[flow]` and `#[leaf]` do not exist: no such attribute is exported by `lid-rs-macros` | `grep` over `lid-rs-macros/src` |
| Slice 19 shipped **no** conformance level — "Severity is fixed, not read from `conformance.level`" | `lid-rs/src/outcome/lld.md:260` |
| Phase 2, 3 and 4 run `cargo check --all-targets` only; Phase 5 runs the red check; **Phase 7 runs `cargo test --lib`** | `cargo-lid-rs/src/phase/mod.rs:578-601`, `:631` |
| Phases 5 and 7 may write the slice's module directory and nothing else in the crate; Phases 3 and 4 add `src/lib.rs` | `cargo-lid-rs/src/phase/policy.rs:417-424` |
| `seat_of` refuses every path outside the slice's crates before any table is consulted | `cargo-lid-rs/src/phase/policy.rs:108-118` |
| Phase 2's row is the claims file and `src/spec/mod.rs` | `cargo-lid-rs/src/phase/policy.rs:444-462` |
| The `cargo lid-rs init` claims template holds **no** claims | `cargo-lid-rs/templates/spec.rs` |
| `init` wires `intent_graph!()` into every scaffolded package | `cargo-lid-rs/templates/lib_footer.rs:4-8` |
| `cargo-lid-rs`, `lid-rs-shape` and `xtask` hold **no** `docs/intent/` directory today | `find . -type d -name intent` |
| `cargo-lid-rs` already reaches every crate's registry out-of-process, by parsing `LID-DUMP` lines | `cargo-lid-rs/src/mutants.rs:147,160,174-180` |
| `target/lid/` is "per-run reports and traces, not committed" | `lid-rs-pipeline/docs/intent/pipeline.md:676` |
| `lld-check` requires a `## Decisions & Alternatives` table with four filled cells per row, a `## Shape` table whose rows name a backticked identifier and a role, and numbered `### Deferred` items | `cargo-lid-rs/src/lld_review/mod.rs:17-23`, `:288-360` |

One fact I could **not** verify: whether `cargo package -p lid-rs` tolerates a
dev-dependency on an unpublished sibling version. It is only reachable under an
alternative this document rejects for other reasons, so it was not spiked.

## Behaviour

### What this slice is not

- **`lid-rs-site`.** Deferred 3, and a slice of its own. Nothing here writes
  HTML, reads a trace, or renders a claim card.
- **The shape `warn` counts.** Deferred 1, with the cause stated below rather
  than promised away.
- **The conformance `warn` counts.** Deferred 2, for a different cause.
- **A `cargo lid-rs regen` subcommand.** The catalog's dispatch is the
  crate-root slice's (`catalog/lld.md:271-276`). `regen` here is a test, run by
  the `cargo test` invocation pipeline §5.1 already specifies.

### Check 26 can compare the registry, and cannot reach the shape pass

README states check 26 over two inputs. The registry half is available where
the check must run — inside a crate's own test binary, which is the only place
`VALIDATIONS` exists (README §5.2). The shape half is not, and the obstruction
is structural rather than a matter of effort.

`lid-rs-shape` depends on `lid-rs` (`lid-rs-shape/Cargo.toml:30`). A normal
dependency the other way is a package cycle, spiked and refused by cargo with
`error: cyclic package dependency` before any build begins. A **dev**-dependency
cycle is accepted — also spiked — but buys nothing, because the check is emitted
by `intent_graph!()` into *consumer* crates: the expansion may only address
`$crate::…`, and a dev-dependency is absent from `lid-rs`'s library. A consumer's
test binary would have no `lid_rs_shape` to reach at all.

The remaining route is a file. Pipeline §5.1 names `shape-classify.json` as the
`shape` command's second output, and `regen` as a reader of it
(`pipeline.md:222-226`). **Nothing writes it.** `lid-rs-shape` writes no file by
design (`lib.rs:56-63`); `cargo lid-rs shape` has no dispatch arm
(`lib.rs:41-56`); and — the fact that settles it — *no crate in the workspace
depends on `lid-rs-shape` at all*, so the pass has no caller of any kind. Even
with a writer, reading JSON from inside `lid-rs` would cost a parser dependency
against tenet 3, and would make a *committed* document's freshness depend on an
uncommitted build artifact under `target/lid/` (`pipeline.md:676`) — so a fresh
clone would compare against an input it does not have, which is constraint 3's
"reports nothing" arriving by the back door.

**So this slice's check 26 is the registry half, and the document holds no
shape section.** That is a narrower check than README's sentence, and it is not
a check gating half of what it names: it gates all of what the document holds.
The distinction matters because the failure mode the HLD warned about is a
document with an ungated region in it, and a document with no such region has
none. When a producer exists, the section is added to the same generator and
check 26 widens with it without changing shape.

### One document per crate, because no binary sees the whole workspace

README shows a single root `trace.md` (`:1761`), and the natural reading is one
document for the repository. It cannot be produced from where the generator has
to run.

`#[validates]` edges are `#[cfg(test)]`. In `cargo-lid-rs`'s test binary,
`VALIDATIONS` holds `cargo-lid-rs`'s validators and nobody else's; `lid-rs`'s
own validators exist only in `lid-rs`'s test binary. This is exactly the fact
README's scoping note is written about (`:659-666`) and the reason checks 10
and 11 filter by crate. A workspace-wide document would need the union of five
binaries' validation edges, and no single run of `cargo test --lib` has it.

**So: one document per library member that invokes `intent_graph!()`, at that
package's `docs/intent/trace.md`.** The path the emitted test uses is
`env!("CARGO_MANIFEST_DIR")` joined with `docs/intent/trace.md` — README's own
spelling, read package-relative, which is what §11.1 already says an intent
document is ("inside the package root of the crate that includes it",
`:1806-1809`). For the single-package project §11.1's tree actually depicts,
and for every project `cargo lid-rs init` creates, the package root *is* the
repository root and README's picture is unchanged. In this workspace it means
five documents, three of which create a `docs/intent/` directory that does not
exist yet.

`env!` is expanded while the *invoking* crate is compiled, which is the same
mechanism `intent_graph!()` already relies on for `env!("CARGO_CRATE_NAME")` —
demonstrated by crate scoping working per-crate today, so this is applied
behaviour rather than an assumption.

### The generator is a pure function; the writing and the comparing are emitted tests

The graph slice's shape transfers unchanged and is followed deliberately: pure
functions over registry slices in `lid-rs/src/trace/`, and a `macro_rules!`
expansion that applies them to the real registries. Every branch is an ordinary
unit test with synthetic inputs; the emitted tests hold no logic worth claiming.

`render` refuses over a stripped registry, returning `Err(CanaryStripped)` the
way `graph_orphans` does. This is not decoration. A stripped registry renders
the empty document, `regen` would then **erase** every row of a committed file,
and check 26 would demand the erasure. The canary is what stands between a
linker section that was stripped and a document that says the project has no
claims. `CanaryStripped` is the graph slice's `pub` type and is reused rather
than duplicated; a second stripped-registry error would be a second answer to
one question.

### What a row can say, and the one thing it cannot

A row's material is what the registry holds, and the registry holds a claim's
*parts*, not its sentence. `ClaimMeta` records language, pattern, trigger,
verb, negation, object, owner and templates (`claim/mod.rs:53-80`) — and for a
claim marked `#[lid(free)]`, every one of those parts is empty
(`registry/mod.rs:16-20`).

That bears directly on two README sentences. §11.2 wants the document "small
enough to belong in PR diffs, where a claim's wording change appears in the
same row as its validators" (`:1847-1852`), and §4.4 leans on the same property
for the residual it declines to gate (`:755-758`). **For a claim held to the
controlled language the property holds**: changing the trigger, the verb, the
object, the owner or the negation changes the row. **For a claim marked free it
does not**, because nothing of its text reaches the registry — and 309 of the
470 claims registered in this workspace are marked free. The document says so
in its own header rather than letting a reader infer a guarantee it does not
give, and the fix (a sentence on `ClaimMeta`) is Deferred 4 on another slice.

### "With links" is refused, and the reason is determinism

§11.2 says "one row per claim, with links". The row carries locations as plain
`file:line` text instead, and this is a deliberate refusal.

`SpecMeta.file` is workspace-root-relative in this workspace — the live dump
prints `lid-rs/src/graph/mod.rs` — and package-relative in a single-package
project, because `file!()` is relative to the directory rustc was invoked from.
The document sits at `<package>/docs/intent/`. The number of `..` segments
between the two is therefore a property of the build layout, not of the
registry. A generator that probed the filesystem to discover it would make the
document's *text* depend on where it was built, and a freshness check requires
that the same registry render the same bytes everywhere. Absolute URLs are the
other candidate and are worse: they pin a crate version and a hosting domain
into a file every consumer of `lid-rs` generates.

Plain `file:line` is also the form every existing report in this tree already
uses — `orphaned_specs` formats `name (file:line)`
(`lid-rs/src/graph/mod.rs:60`), and `unclaimed_variants` copies it. README §11.2
gains an amendment saying so.

The book LLD's three-context rule (`docs/intent/book/lld.md`, Decisions) is what
would otherwise force absolute URLs, and it does not apply: this document is
included by no crate, so rustdoc is not one of its contexts, and Decision 8
below keeps it out of the book. If it ever enters the book, this decision must
be revisited before it is added to `SUMMARY.md`.

### The section per slice comes from the claim's name, not from its path

`Spec::NAME` is `module_path!()` plus the identifier, so
`lid_rs::graph::spec::CoveredGraphsPassTheGraphCheck` spells the crate, the
slice and the `spec` module in order. The section is what stands between the
crate segment and the trailing `spec::<Name>`: `graph` here, `lid_rs_macros`
for the companion module in `lid-rs`, and **empty** for a crate-root slice
whose claims live in `src/spec.rs` — `cargo_lid_rs::spec::X` — which takes the
crate's own name as its section.

Reading it from the name rather than from `SpecMeta.file` keeps the document
free of any assumption about the layout, which the layout slice moved once
already.

### The ramp section this document can honestly carry

Slice 18 handed over a question and named this document as the only one that
can answer it:

> slice 21's LLD must decide what a shape count section looks like, and it is
> the only document that can (`lid-rs-shape/src/lld.md:305-311`)

The answer has two halves. **The shape of a count section** is: one line per
section, under the section's heading, naming each counted category and its
number, with the total; and **the crate's own line** at the top of the
document summing its sections. Not a workspace line: a document is per package
by this document's own second decision, and no crate's test binary can see
another's claims, so a workspace total is not a number this slice can compute. It is a line of text in a diff, which is the entire point — §3.7
wants the ramp "visible in the PR diff" (`:565-571`).

**What fills it today** is not the shape pass, which has no producer, and not
conformance, which has no level. It is the `#[lid(free)]` ramp — the one ramp
README already calls counted ("every claim either written in the language or
carrying a counted `#[lid(free)]` mark that the ramp burns down slice by
slice", `:1946-1948`), the one the vocab slice counts *by hand* today
(`lid-rs-macros/src/vocab/lld.md:234`), and the one whose datum is on the
registered struct: `meta.claim.language == Language::Free`, already enumerated
by `lid_rs::claim::free()` (`lid-rs/src/claim/mod.rs:82-96`).

This corrects a standing assumption worth stating plainly, because it changes
the scope: the mark is **not** only on the `Spec::FREE` associated const, which
a registry consumer holding a `SpecMeta` by name cannot read. It is on
`SpecMeta.claim.language`, so the ledger needs no new machinery at all.

So the shape pass fills the section this slice defines when it has a caller,
and until then the section holds the ramp that has data. That discharges slice
18's handover in the direction it was owed — the shape *of* the section — and
records honestly that the shape pass's numbers are not in it.

### The conformance counts are not a section, and that is the answer

§11.2 promises "the shape **and conformance** `warn` counts". Slice 19 shipped
no conformance level: "Severity is fixed, not read from `conformance.level` |
E1 a compile error, E2 a failing test | … A level that cannot be read is not a
level" (`lid-rs/src/outcome/lld.md:260`), and the reason was the same private
accessor obstacle slice 18 hit for `shape.level`. Checks 19 and 20 are not
built at all.

A `warn` count over checks that report no level is a column of zeros or an
invented number. Constraint 3 is about reports nobody reads, and a permanently
empty section is exactly one. **There is no conformance section.** It returns
when there is a conformance level to count against, which is Deferred 2 and
belongs to whichever slice makes a level readable.

### The `trace` knob is not read, for the third time

`README.md:1200` declares `trace = "docs/intent/trace.md"` under
`[workspace.metadata.lid_rs]`. Nothing reads it. The accessors are
`Project::workspace_setting`, `Project::root_package_setting` and `setting_in`,
all private and all in `cargo-lid-rs` (`project.rs:232,238,261`), which `lid-rs`
cannot depend on.

This is the third slice to meet the obstacle and it takes the same answer slices
18 and 19 took: **a setting either arrives as an argument from a caller that can
read it, or there is no setting.** The path is fixed at `docs/intent/trace.md`
relative to the package root. Deferred 5.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Slice 21 is split; this slice is `trace` only | A module slice at `lid-rs/src/trace/`: the generator, the committed document, check 26. `lid-rs-site` becomes a later slice | Build the whole HLD row as one slice; build the site first and derive the document from it; drop check 26 and ship the site | The site needs slice 20's captured traces for a claim card's worked example (§11.2) and a vocabulary ledger that the vocab slice defers *to this document* — a mutual dependency that only breaks by shipping the registry half first. Splitting also unblocks slice 18's `warn` ramp, whose committed-count condition (`README.md:565-571`) has had no home since `585b212`. The HLD row's numbers are stable, so the split renames nothing. |
| Check 26 compares the registry half only, and the document holds no shape section | `render` over `SPECS`/`IMPLEMENTATIONS`/`VALIDATIONS`, compared against the committed bytes | Depend on `lid-rs-shape` from `lid-rs`; read `shape-classify.json` from the emitted test; build `cargo lid-rs shape` first and take the dependency in the caller | Spiked: `lid-rs-shape` depends on `lid-rs`, so the reverse edge is `error: cyclic package dependency` (cargo exit 101); a dev-dependency cycle is accepted (exit 0) but is invisible to `intent_graph!()`, which expands into consumer crates and may address only `$crate`. The JSON route has no writer — the pass writes no file, has no dispatch arm, and has **no dependent crate at all** — and would gate a committed document on an uncommitted `target/lid/` artifact, so a fresh clone would compare against nothing. Building the command first is the crate-root slice's dispatch work (`catalog/lld.md:271-276`). Constraint 3 is satisfied because the document contains no ungated region: the check gates all of what is there, which is not the half-gated document `hld.md:219-221` warns about. |
| One document per library member, at `<package>/docs/intent/trace.md` | Five documents in this workspace, written from `env!("CARGO_MANIFEST_DIR")` | One workspace-root document; one per crate under the workspace root, name-suffixed; a document assembled by `cargo-lid-rs` from the `LID_DUMP` route | `#[validates]` edges are `#[cfg(test)]` and exist only in their home crate's test binary (`README.md:659-666`), so no single `cargo test --lib` run can see the workspace's validators — a root document is **unproducible** from where the generator must run, not merely inconvenient. An emitted test knows only `CARGO_MANIFEST_DIR`, so it cannot address a sibling path reliably. The `LID_DUMP` union is genuinely producible (`mutants.rs:147`) and is rejected because it makes `regen` a subcommand rather than a test, puts check 26 outside `cargo test --lib` where `README.md:792` places it, and is `cargo-lid-rs`'s slice. §11.1's own rule already places an intent document inside its package root. |
| `regen` is `#[ignore]`d; the freshness test is not | Two emitted tests: `regen`, ignored, which writes; and the freshness test, which only compares | One test that writes when `LID_REGEN=1` is set, as `registry_dump_for_tooling` does with `LID_DUMP`; a freshness test that repairs the file and fails | `pipeline.md:226` already specifies `cargo test --lib -- --ignored regen`, and `--ignored` is a libtest flag rather than a private protocol. Splitting keeps `cargo test --lib` incapable of writing into the source tree, which is what makes the document safe to gate. A self-repairing check is a check that never fails twice, so it is not a check. |
| An absent document reads as the empty document | `committed(path)` answers the file's bytes or the empty string, and `stale` compares `render(...)` against that; a crate with no own claims renders the empty string | Absent is a failure naming the regen command; absent is a pass; absent is a failure unless the crate has zero claims | One rule with no exception. A crate with claims and no document fails, which is `cargo fmt --check` semantics and what README asks for. A freshly `cargo lid-rs init`ed package registers no claims — its `spec_mod.rs` template holds none — so it renders the empty document, matches an absent file, and passes its own gate on the first run, keeping HLD row 9's end-to-end proof intact **without a special case for it**. The explicit-exception form was drafted and dropped: it is the same behaviour written twice. |
| A row carries name, pattern, language, implementers and validators; locations are plain `file:line` text | `name`, the pattern, whether the claim is free or controlled, and the citing items with their sites | Markdown links relative to the document; absolute URLs to the published book or docs.rs; the claim's full sentence | `SpecMeta.file` is workspace-relative here and package-relative in a single-package project, so the `..` prefix to the document is a property of the build layout; discovering it by probing the filesystem would make the rendered text environment-dependent, and freshness requires that one registry render one byte sequence. Absolute URLs pin a version and a domain into a file every consumer generates. The sentence is not in the registry at all — `ClaimMeta` carries parts (`claim/mod.rs:53-80`) — so §11.2's "a claim's wording change appears in the same row" holds for controlled claims and not for the 309 free ones, and the document states that rather than implying otherwise. |
| The ramp section is defined here and filled by the `#[lid(free)]` ledger | One count line per section — controlled, free, total — and the crate's own line at the top, summing its sections | No count section until the shape pass has a producer; a shape section of zeros; count the free marks by hand as the vocab slice does | This answers slice 18's handover in the direction it was actually owed (`lid-rs-shape/src/lld.md:305-311`): the *shape* of a count section, which only this document can fix. Filling it with the shape pass's numbers is impossible today and filling it with zeros is constraint 3's empty report. The free ramp is the one README already calls counted (`:1946-1948`) and the one whose datum is on the registered struct — `SpecMeta.claim.language`, enumerated by `claim::free()` — so it needs nothing new. The shape pass fills the same section when it has a caller. |
| No conformance section at all | Omit it, with the cause recorded | A section of zeros; a section counting checks 21 and 22's findings as `warn`s; wait for `conformance.level` | Slice 19 fixed severity instead of reading a level (`outcome/lld.md:260`) and checks 19/20 are unbuilt, so there is no level any count could be `warn` against. A section over that is invented vocabulary — the exact defect slice 18's Open Question 1 was withdrawn for. Waiting blocks the shape ramp for a section that has no producer either. |
| The `trace` knob is not read; the path is fixed | `docs/intent/trace.md`, relative to `CARGO_MANIFEST_DIR` | Read `[workspace.metadata.lid_rs] trace`; pass the path in as a macro argument; add a `lid-rs` settings reader | The accessors are private and in `cargo-lid-rs`, which `lid-rs` cannot depend on — the identical obstacle slices 18 and 19 met for `shape.level` and `conformance.level`, and the same answer: a setting either arrives from a caller that can read it, or there is none. A macro argument would make the path a per-consumer variable that check 26 would then have to agree with, for a knob nobody has changed. |
| The document is not added to the book | `book/src/SUMMARY.md` is untouched | Add each crate's `trace.md` as a chapter; add only `lid-rs`'s | The book is assembled by inclusion and its §-reference rule forces absolute URLs in every artifact it renders (`docs/intent/book/lld.md`, Decisions) — which is the form the row decision above refuses. `SUMMARY.md` is also already stale by several LLDs, and closing that drift is a book-slice change, not a rider on this one (Deferred 7). |
| The emitted tests carry no citation, and no claim of this slice is about them | Two plain `#[test]`s in `intent_graph!()` | Emitted tests carrying `#[validates]`; a claim about the emission cited on the macro | The graph slice settled this and the reasoning is unchanged: "emitted tests are each consumer's enforcement instance, and citing lid's specs from every consumer would scatter cross-crate edges that mean nothing to the consumer's own graph" (`lid-rs/src/graph/lld.md`, Decisions). It also keeps checks 10 and 11 satisfied at this slice's Phase 7, since `intent_graph!()` is in the **graph** slice's module and no phase of `lld/trace` may write it. |
| `render` refuses over a stripped registry, reusing `CanaryStripped` | `render(...) -> Result<String, CanaryStripped>`, the emitted tests `.expect(…)` | A guard only in the emitted test, as the outcome slice put it; a `bool` parameter; a new error type of this slice's | A stripped registry renders the empty document, so an unguarded `regen` would **erase** a committed file and check 26 would demand the erasure — the most damaging failure this slice can have, and the one the canary exists for. It cannot live in the emitted test here, because that test is uncited and its behaviour would be unclaimed. A `bool` is check 8's flag argument. A second stripped-registry type would be a second answer to one question; `CanaryStripped` is `pub` (`graph/mod.rs:16-21`). |
| Check 26 does not land until a phase can regenerate the document | The emission is withheld until the `phase` slice can keep `trace.md` fresh inside a phase | Land check 26 at this slice's Phase 7 and let later slices cope; make the freshness test `#[ignore]`d too | Verified: `plan(Phase::Seven)` runs `Step::LibTests` (`phase/mod.rs:583,591-601,631`), and Phases 5 and 7 may write only the slice's module directory (`policy.rs:417-424`). Any later slice that adds a claim makes its crate's document stale and **cannot write the fix**, so its Phase 7 could never pass — the check would stop the methodology it belongs to. Ignoring the freshness test removes it from `README.md:792`'s gate line, which is constraint 3. See "What lands by hand" and Open Question 1. |

## Open Questions & Future Decisions

### Open

**1. How a phase keeps `trace.md` fresh — and this is a hard precondition, not
a preference.** Once the freshness test is in `intent_graph!()`, every slice's
Phase 7 runs it (`Step::LibTests`), every slice adds claims, and no phase may
write `docs/intent/trace.md`. Without an answer, **no slice after this one can
reach a Phase 7 commit.** Two shapes, both changes to the `phase` slice and
neither this document's to take:

- *Widen the path row.* Add `docs/intent/trace.md` to Phases 3–7 in both
  `own_table` and `companion_table` (`policy.rs:417-440`). One line, and it
  hands an agent a file it should never hand-edit — a hole in the thing the
  policy exists for.
- *Regenerate in the stop hook.* Have the hook run the `--ignored regen`
  invocation and stage the document before the check, the way a formatter
  would. The policy stays closed and the document stays tool output rather than
  agent output. Costs the hook a `cargo test` run per phase commit.

**The second is this document's recommendation**, on the same reasoning that
makes `sync --check` a gate step rather than an editable artifact. It is
recorded as a recommendation because the decision belongs to the slice whose
code it changes.

Cost of getting it wrong: the failure surfaces at the *next* slice's Phase 7,
not this one's, which is precisely the class of ordering defect slice 19 paid
three amendments for.

**2. Whether 316 rows is "small enough to belong in PR diffs".** §11.2's
justification for committing the document is diff size. `cargo-lid-rs` registers
316 claims; its document is the largest and is the one to measure. The rows a
given PR touches are few, so the diff is small even where the file is not, but
the file itself is what a reviewer opens. Measure it once the first documents
exist rather than guessing a column layout now; if it is unreadable, the answer
is likely one document per slice directory, which is a Phase 8 change to
`document_path` and nothing else.

**3. The pipeline's `graph` filter names no test that exists.**
`pipeline.md:222` runs check 26 as `cargo test --lib intent_graph::graph`, and
none of today's four emitted tests match that substring — verified. So the
freshness test's name is unconstrained today, and a later renaming of the
emitted tests to a `graph_` prefix must carry it. Recorded so that a name chosen
here is not mistaken for one the pipeline already depends on. It is a pipeline
document defect and belongs to slice 23.

### Deferred

1. **The shape `warn` count section.** Cause: there is no producer. The pass
   writes no file (`lid-rs-shape/src/lib.rs:56-63`), `cargo lid-rs shape` has no
   dispatch arm (`cargo-lid-rs/src/lib.rs:41-56`), and no crate in the workspace
   depends on `lid-rs-shape`. It lands with whichever slice gives the pass a
   caller, into the section shape this document defines.
2. **The conformance `warn` count section.** Cause: slice 19 shipped no
   conformance level (`lid-rs/src/outcome/lld.md:260`) and checks 19/20 are
   unbuilt, so there is no level to count `warn`s against.
3. **`lid-rs-site`** — claim cards, the observed flow graph, the trace matrix as
   a page, the glossary, `pr-body`. Cause: a claim card's worked example is a
   captured trace, which is slice 20's and unbuilt; and the glossary's
   vocabulary ledger is deferred *to this document* by
   `lid-rs-macros/src/vocab/lld.md:281-283`, which this slice does not resolve.
   A slice of its own, named for its crate under the crate-root naming rule.
4. **The claim's sentence in the row.** Cause: `ClaimMeta` carries parts, never
   the sentence, and a free claim's parts are empty. A `sentence` field is a
   `claim`-slice change carried by all 470 registrations, so it is a cascade
   with its own cost argument. Until then §11.2's wording-change guarantee holds
   for controlled claims only, and the document says so in its header.
5. **The `trace` knob.** Cause: the metadata accessors are private and in
   `cargo-lid-rs`; `lid-rs` cannot depend on it, and an emitted test has no
   caller to hand it a setting. Identical to `shape.level` and
   `conformance.level`.
6. **A workspace-wide union document.** Cause: producible only from
   `cargo-lid-rs`'s `LID_DUMP` route (`mutants.rs:147`), which is a subcommand
   rather than a test and belongs to the catalog's dispatch slice.
7. **`book/src/SUMMARY.md`'s staleness**, and whether the trace documents join
   the book at all. Cause: a book-slice change, and it interacts with the
   plain-text location decision above.
8. **`regen` and `graph` in the catalog's built table.** Cause: they are marked
   unbuilt on the catalog's own Deferred 1, a libtest-JSON question
   (`catalog/mod.rs:219-220`). `regen`'s job is to write a file rather than
   report findings, so the deferral's stated reason may not apply to it — worth
   re-reading when the catalog's dispatch lands, not resolving here.

## What lands by hand, and in what order

Four things no phase of `lld/trace` may write. Each was checked against the
policy and against what the phase after it needs.

**1. `pub mod trace;` in `lid-rs/src/lib.rs`, and `lid-rs/src/trace/mod.rs`
holding `#![doc = include_str!("lld.md")]` and `pub mod spec;` — between Phases
2 and 3.** Phase 2's row is the claims file and `src/spec/mod.rs`
(`policy.rs:452-461`), so it can write `src/trace/spec.rs` and cannot declare
it; Phase 2's check is `cargo check --all-targets`, which passes happily over a
file no module compiles, so **check 13 never sees the claims** and the failure
surfaces at Phase 5 as "nothing registers". `src/lib.rs` is in Phase 3's row, so
the declaration *may* land there — and doing so puts check 13's rejections
inside a phase that may not fix them. Slice 18 (`4b036c7`) and slice 19
(`705ab1b`) both landed it by hand between Phases 2 and 3, and both had every
claim accepted first time. Do that.

**2. The `intent_graph!()` emission — the freshness test and the ignored
`regen` — after Phase 7, not before it.** This is the reverse of slice 19's
order, and the reason is that the dependency runs the other way.

`intent_graph!()` is in `lid-rs/src/graph/mod.rs:76-131`, another slice's
module. For a `Form::Module` slice the phase row is `src/trace` plus, at Phases
3 and 4, `src/lib.rs` (`policy.rs:417-424`), so no phase of this branch may
touch it. That much is the same as slice 19's `every_variant_has_a_claim`.

What differs is that the emission cannot land *earlier* either. Before Phase 6
the generator is `todo!()`, so both emitted tests panic and `cargo test --lib`
is red — tolerable between Phases 3 and 5, fatal at Phase 7. And it cannot land
*at* Phase 7, because the five documents it compares against have to be
generated by the leaves that same phase is implementing, and
`docs/intent/trace.md` is in no phase's row in any crate. Phase 6 has no commit
of its own — the `lid-rs-phase-7` agent implements the leaves and gates in one
run — so there is no stop between "the generator works" and "the gate runs".

**So `lld/trace` runs Phases 2 through 7 as an ordinary library slice with no
emission at all.** Its claims are all about functions in `src/trace/`,
implemented and validated there, so checks 10 and 11 are satisfied and Phase 7
passes. Then one hand commit lands the emission, the phase-policy answer from
Open Question 1, and the first five generated documents together, followed by a
hand gate run. Check 26 gates from that commit onward.

The consequence to state rather than discover: **this slice's Phase 7 gate does
not run check 26.** The check is delivered by the commit after it. That is the
same arrangement `da047cd` used on `lld/outcome` for work no phase could write,
with the order inverted for the reason above.

**3. The five first documents**, generated by
`cargo test --lib -- --ignored regen` and committed in the same hand commit as
item 2. Three of the five create a `docs/intent/` directory that does not exist
yet (`cargo-lid-rs`, `lid-rs-shape`, `xtask`). They must never be hand-edited;
`regen` is the only writer, exactly as `.stderr` pins are only ever written by
`TRYBUILD=overwrite` (`docs/intent/publish/lld.md:83`).

**4. The README and pipeline amendments.** README §11.2 gains the
package-relative path, the plain-text locations in place of links, the
qualification on the wording-change guarantee, and the note that the shape and
conformance counts arrive with their producers; §11.1's tree line and check
26's row follow. `README.md` is at the workspace root, outside every crate, so
`seat_of` refuses it before any table (`policy.rs:106-115`) — a main-session
commit. Tenet 1 makes these doc bugs, and they are not optional: leaving §11.2
describing links this slice does not emit is the divergence tenet 1 exists to
prevent.

**Also worth sequencing:** `lld/graph--outcome-checks` is still pending — slice
19's `every_variant_has_a_claim` has not landed, and `lid-rs/src/graph/mod.rs`
still emits four tests. Item 2 touches the same macro. Whichever lands second
rebases onto the other; neither conflicts semantically.

**And a cascade inside item 2:** `graph/mod.rs:64-66` documents the macro as
expanding to "the three intent-graph tests", and `graph/lld.md`'s Shape row
says "four `#[test]` fns". Both are wrong the moment tests are added, and check
2 does not catch prose. Fix them in the same commit.

## Notes for Phase 5's red set

Five things a red test of this slice gets wrong by default.

1. **A claim implemented by data has no reachable red.** The document's
   location is the obvious candidate: written as `const DOCUMENT: &str`, a claim
   about it is correct from Phase 3 onward, its test is green before
   implementation, and `red_verdict` refuses it
   (`cargo-lid-rs/src/phase/mod.rs:912-919`). The Shape table therefore gives
   the location a function, `document_path`, which the const feeds — the same
   move `registry::canary::present` and `outcome::canary::present` already made
   for their slices. Phase 2 must not derive a claim whose only implementer
   would be the const.

2. **The empty-registry case is green without reaching a `todo!()`.** A test
   written as "no claims, so the empty document" can be satisfied by a `render`
   that returns early, and a `sections` that folds over nothing returns `vec![]`
   without calling anything. Every red test of the generator passes at least one
   `SpecMeta`. The claim most exposed to this is the one about a crate with no
   claims of its own rendering the empty document: write it with a `SpecMeta`
   belonging to *another* crate, so the filter has something to reject.

3. **Determinism is a claim and needs an assertion, not a comment.** Freshness
   is only meaningful if one registry renders one byte sequence, and `linkme`
   gives no ordering guarantee across link units. The generator sorts, and the
   test that says so must compare two renders of the same claims supplied in
   two different orders — not one render against a literal.

4. **The absence rule needs a real path, and only one item may touch the disk.**
   `committed` is the slice's only filesystem reader, so its two branches — a
   path that exists and a path that does not — are the only ones a red test
   cannot build from synthetic values. Redden it against a temporary directory,
   not against a path in the repository, and assert both branches: a missing
   file answers the empty string, and a present one answers its bytes. Every
   other leaf takes its input as a parameter and needs no disk at all.

5. **Check 12 will mutate the formatting leaves.** A row renderer whose test
   asserts only that the output is non-empty hands a surviving mutant to every
   `String::new()` substitution. Assert the row's content — the claim's name,
   the pattern, each citing item — not its length.

## Shape

| Item | Role |
|---|---|
| `lid_rs::trace::DOCUMENT` | The generated document's path relative to a package root: `docs/intent/trace.md`. Data, carrying no citation — the claim about where the document goes is cited on `document_path`, so that Phase 5 has a reachable red. |
| `lid_rs::trace::document_path(manifest_dir) -> PathBuf` | Work leaf: `DOCUMENT` joined onto the invoking package's manifest directory. The emitted tests pass `env!("CARGO_MANIFEST_DIR")`, which is what makes the document per-package without any setting being read. |
| `lid_rs::trace::render(crate_name, specs, impls, validations) -> Result<String, CanaryStripped>` | The whole document, for one crate. Canary-first, reusing the graph slice's [`CanaryStripped`](crate::graph::CanaryStripped): a stripped registry must not render the empty document, because `regen` would then erase a committed file. Below the guard it composes the header, the crate's ledger line and the sections — and holds **one** decision besides the guard: a crate whose own claims are none renders the empty string, not a header and a count line standing over no rows. An earlier draft of this row said it held no decision of its own, which the absence rule in the Decisions table contradicts; two branches is inside check 7's bound. |
| `lid_rs::trace::Section` | One slice's part of the document: the section name as the claim names spell it, **the claims in the order their rows are written**, and the section's `Ledger`. An earlier draft of this row said "the rows in order", which the code cannot honour: `sections` is handed no edge sets, so it can render no row. `render` holds the edges and turns each claim into a row. It borrows the registrations rather than copying them, so it carries a lifetime. |
| `lid_rs::trace::Ledger` | One count line's material: how many of a set of claims are controlled and how many are free. The shape a count section takes, defined here so the shape pass fills the same shape when it has a producer. |
| `lid_rs::trace::sections(crate_name, specs) -> Vec<Section>` | Work leaf: the current crate's claims, grouped by the slice their `Spec::NAME` spells, each group sorted by name and the groups sorted by section. Crate-scoped by `Spec::NAME` prefix, as `orphaned_specs` is. **`Vec`, not `impl Iterator`** — Phase 3 cannot skeleton an `impl Trait` return, since `!` does not implement `Iterator` and the workarounds fire denied lints. |
| `lid_rs::trace::slice_of(name) -> &str` | Work leaf: the segments of a `Spec::NAME` between the crate and the trailing `spec::<Name>` — `graph` from `lid_rs::graph::spec::X`, and empty from `cargo_lid_rs::spec::X`, which is a crate-root slice whose section takes the crate's name. Read from the name and never from `SpecMeta.file`, so the document assumes nothing about the layout. |
| `lid_rs::trace::row(meta, impls, validations) -> String` | Work leaf: one claim's row — its name, its pattern, whether it is controlled or free, its implementing items and its validating items, each with `file:line` as plain text. No links; see the Decisions table. |
| `lid_rs::trace::citations(spec_name, edges) -> Vec<String>` | Work leaf: the items in `edges` citing `spec_name`, formatted `item (file:line)`, sorted. One function for both edge sets, which is why it takes the set rather than an `EdgeKind`. |
| `lid_rs::trace::ledger(specs) -> Ledger` | Work leaf: the controlled and free counts over a set of claims, read from `meta.claim.language`. The one ramp count the registry can answer today. |
| `lid_rs::trace::committed(path) -> String` | Work leaf: the document as it stands, or the empty string where there is none. The **only** item that reads the filesystem, and it exists because the absence rule is a decision of this slice and had nowhere else to live: `read_to_string(path).unwrap_or_default()` written in the caller would put a governing decision inside the emitted test, which is uncited, in the graph slice's module, and lands after this slice's Phase 7 — so checks 10 and 11 would name the claim an orphan. Reading a path that is not there is also the one branch of this slice a synthetic input cannot produce. |
| `lid_rs::trace::stale(generated, committed) -> Option<String>` | Work leaf: check 26's verdict — `None` when the two agree, and otherwise a message naming the first line that differs and the command that repairs it. An absent file is passed as the empty string by the caller, so absence is staleness only when something was generated. |
| `intent_graph!()`'s freshness test | Renders, reads `document_path`, and asserts `stale` is `None`. Plain and uncited, as every emitted test is. Lands in `lid-rs/src/graph/mod.rs` by hand, after this slice's Phase 7. |
| `intent_graph!()`'s `regen` test | `#[ignore]`d, so `cargo test --lib` never writes into a source tree; renders and writes `document_path`. Run as `cargo test --lib -- --ignored regen`, which regenerates every member's document in one invocation. Lands in the same hand commit. |
| `trusted(specs, impls, validations) -> Result<(), CanaryStripped>` | The canary refusal, as a leaf rather than inline. `render` returns what it answers. It is a leaf and not a guard written into `render` because a refusal written inline is implemented before Phase 5 can redden it, and check 12 would have no item to mutate for it. |
| `header() -> &'static str` | The document's opening, including the qualification that a free claim's row holds none of its wording. A function and not a `const` for the reason `document_path` is a function and `DOCUMENT` is not: a claim whose only implementer is data can never be red. |
| `document_text(parts, impls, validations) -> String` | The order the document is read in — the header, the crate's ledger line, then the sections. One of the two placement decisions `render` was carrying. |
| `section_text(part, impls, validations) -> String` | A section's own order — its name, its ledger below that name, its rows below that. The other placement decision, at the other scope. Two items because two claims are about two scopes. |
| `ledger_line(counts) -> String` | The one place a count line is written, so a section's line and the crate's line are one answer. **This is the shape this slice owes the shape ramp**: `Claims: N controlled, N free, N total.` Its total is a sum over the categories the `Ledger` lists rather than an addition of two counts — the binary form leaves check 12 a surviving `+`-to-`*` mutant, because the tests that pin a count line call this function themselves and so mutate both sides of their own comparison. |
| `section_name(crate_name, spec_name) -> &str` | The crate-root branch: a claim whose slice is empty takes the crate's name for its section. |
| `section(name, claims) -> Section` | One group's own order and its ledger. |
| `first_difference(generated, committed) -> Option<usize>` | Whether the two documents differ and where. Its loop is bounded by the lines and not by a counter its own body increments — the other form is a hang rather than a wrong answer, and check 12 reports that as a TIMEOUT an hour later. |
| `repair_message(line) -> String` | What to tell the reader: the line at which they differ and the command that regenerates the document. |

**The last eight rows were added by Phase 4's descent (`6e7945d`) and are private to
the module.** The rows above them are the surface the thirty claims cite; these are what
the descent found beneath that surface. The table names them so that it describes the
slice rather than only its citations — Phase 7 checks that every identifier here resolves
in `src/`, and nothing checks the converse, so an under-specified table is the failure
mode to guard against by hand.

Every function above is parameterized over slices, as the graph slice's are, so
each failure branch is an ordinary unit test with synthetic inputs while the
emitted tests apply the same functions to the real registries. `committed` is
the single exception and the single item that touches the disk, which is what
makes the absence rule reachable from a test at all.

**A note on the public surface, corrected at Phase 3.** Rule B — "every
`pub fn` in a slice's `mod.rs` is flow, or carries `#[leaf]`" — has no escape
hatch available: `#[flow]` and `#[leaf]` do not exist in `lid-rs-macros`, and
the shape checks are emitted by nothing, so rule B does not fire today.

An earlier draft of this note promised the public surface would be "kept
flow-shaped where it is public". **It is not, and it cannot be.** All eleven
items are `pub`, seven of them work leaves, because the thirty claims link to
them from the public `spec` module and a public doc linking a private item is a
`private_intra_doc_links` warning — which this workspace does not suppress. The
same is already true of the graph and outcome slices. So whichever slice turns
rule B on inherits seven public leaves here, and the honest options then are
`#[leaf]` (which must be built) or a rule B that does not fire on an item a
claim links to. Recorded rather than promised away.

**A note on doc links in this document.** The one intra-doc link above,
[`CanaryStripped`](crate::graph::CanaryStripped), resolves only once
`src/trace/mod.rs` includes this file — hand-landing item 1. Until then nothing
includes the document and check 2 does not see it; from that commit onward it
must resolve, so no further link may be written against a type this slice has
not yet created.

## References

- README [§11.2](https://bradvoth.github.io/lid-rs/spec/layout.html) — the five
  reading surfaces, the tool-output paragraph this slice implements, and the
  site it defers.
- README [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) — the
  layout, and the rule that an intent document lives inside its package root.
- README [§3.7](https://bradvoth.github.io/lid-rs/spec/mapping.html) — the shape
  ramp whose committed-count condition this document is the other half of.
- README [§4.2](https://bradvoth.github.io/lid-rs/spec/gates.html) — the scoping
  note, and why a workspace-wide document cannot be produced from one binary.
- `lid-rs/src/graph/lld.md` — the pattern this slice follows in full: pure
  functions over slices, a `macro_rules!` emitter, uncited emitted tests,
  canary-first refusal.
- `lid-rs/src/registry/lld.md` — the triple, the canary, and `SpecMeta.name`'s
  join principle.
- `lid-rs-shape/src/lld.md` — slice 18, which handed this document the shape of
  a count section and is owed the shape pass's numbers back.
- `lid-rs/src/outcome/lld.md` — slice 19, for the settled precedent that a
  setting which cannot be read is not a setting, and for the ordering
  corrections this document tried to anticipate.
- `lid-rs-pipeline/docs/intent/pipeline.md` §5.1 — the `regen`, `graph` and
  `site` rows, and the invocation `regen` is specified as.
