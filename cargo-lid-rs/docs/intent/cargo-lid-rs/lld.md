# cargo-lid-rs — check 12 for any project that depends on `lid-rs`

## Context and Design Philosophy

Check 12 proves a `#[validates]` test *depends on* the code it cites: mutate
the implementation and the test must fail. Until now the orchestration lived in
this workspace's `xtask`, which is `publish = false` and reads its own
location from `CARGO_MANIFEST_DIR` — a project that depends on `lid-rs` from
crates.io could run eleven of the twelve checks and had no way to run the
last one short of copying `xtask` and maintaining the fork. `cargo-lid-rs`
is that orchestration, published: `cargo install cargo-lid-rs`, then
`cargo lid-rs mutants` in any project whose crates invoke `intent_graph!()`.

The crate is a thin `main` over a library, so its logic sits under
`cargo test --lib` and its doctests run ([§4.1](https://bradvoth.github.io/lid-rs/spec/gates.html)). It depends on `lid-rs`
as an ordinary consumer: its own claims live in `src/spec/`, its code carries
`#[implements]`, and its `intent_graph!()` instance checks the result. That
makes it the *published* downstream-consumer proof — the one that exercises
macro path resolution and linker-section behaviour from outside the
workspace, which `xtask` can only do from inside it.

This workspace runs the same binary from source, `cargo run -p cargo-lid-rs
-- mutants`, so the code under the gate is always the working tree's, never
a stale installed copy; a consumer's gate line is `cargo lid-rs mutants`.

## Subcommand shell

Cargo invokes an external subcommand as `cargo-lid-rs lid-rs <args…>` — the
subcommand name arrives as the first argument — while `cargo run -p
cargo-lid-rs --` and direct invocation deliver `<args…>` alone. The shell discards a leading `lid-rs`
and dispatches on what remains; today the only subcommand is `mutants`.

The project the tool operates on is located through `cargo metadata`, never
through the binary's own location: `workspace_root` from the metadata
document is the root, and the metadata is fetched once per run and threaded
to what needs it. Running from any directory inside the project therefore
behaves the same as running from its root.

## Mutant → test-set mapping

`cargo mutants --list --json` yields `(file, function_name)` per mutant;
edges carry `(file, item)` where `item` is `module::path::fn`. A mutant's
function is matched by `edge.file == mutant.file && edge.item` ending in
`::function_name` — file equality disambiguates same-named functions in
different modules; same-named functions in the same file share a test set,
which over-approximates safely.

- **Traced mutant**: implementation edges found → specs → validation edges →
  test filters (edge `item` minus the leading `crate::` segment, since
  libtest names are crate-relative). Run with `--cargo-test-arg=--lib` plus
  `--exact` filters.
- **Untraced mutant**: the tests validating specs implemented in the same
  file — the mutant list carries no module path, so file identity stands in
  for the enclosing module; when the file implements none, the full suite
  ([§6.1](https://bradvoth.github.io/lid-rs/spec/traced.html)).
- Mutants are grouped by identical test set, one `cargo mutants` run per
  group. `-F` takes a regex, so the group is selected by an anchored
  alternation of its escaped mutant names. `--baseline skip` drops the
  engine's own pre-mutation suite run: the gate already ran the full suite
  earlier ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) order).

## Verdicts

The engine's exit status and its `-F` selection are not trusted to describe
what a group's run proved. Each group runs into a fresh output directory
(`<target>/lid-mutants/<n>`, removed before the run so no earlier verdict can
be read as this run's), and the tool judges the group's mutants — only those
— from the `outcomes.json` the engine writes there: `CaughtMutant` and
`Unviable` are fine, `MissedMutant` and `Timeout` are survivors, any other
summary is an error, and a mutant the engine reports *no* verdict for is a
failure, never a pass. Mutants the engine included that the group did not
ask for are ignored in that run; they are judged in their own group. Every
group runs before survivors are reported, so one run names them all, each
with the tests it survived.

This exists because a consumer found the engine over-including: cargo-mutants'
struct-literal field-deletion mutants are pushed without the `allows_mutant`
check that applies `-F`, so every group's run carried them and the first
group's tests — unrelated to them — "missed" them.

## Registry collection

Each crate's `#[validates]` edges exist only in that crate's own `--lib` test
binary ([§5.2](https://bradvoth.github.io/lid-rs/spec/registry.html) applies recursively: they are absent from this tool's binary
and from `lid-rs` as linked into anything else). `intent_graph!()` therefore
emits an inert `registry_dump_for_tooling` test that prints the registries as
tab-separated lines when `LID_DUMP=1`; the tool runs it per workspace member
and parses the output into edge records. A member without the graph checks
dumps nothing and contributes nothing. A member with no library target is
skipped before the attempt: `cargo test --lib` on a binary-only package is an
error, not an empty run, and downstream workspaces routinely contain such
packages.

## Scope

`mutation_scope` is read from `[workspace.metadata.lid_rs]`; a project that
is a single package without a `[workspace]` table configures it in
`[package.metadata.lid_rs]` instead, and the tool consults the root package
when the workspace table has no entry. Unset means `diff`. Diff scope
generates `git diff <base>` (default base `main`; CI passes
`--diff-base origin/main`; `--full` overrides the scope entirely) and passes
it via `--in-diff`. Diff interpretation is the engine's contract, not
re-implemented here.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Home for check 12 | A published `cargo-lid-rs` crate; `xtask` keeps only the gate self-test | Publish `xtask` whole; have `cargo lid-rs init` emit an `xtask` copy into each project; keep check 12 workspace-only | The self-test's fixtures are this repository's evidence, not a consumer's tool. An emitted copy is a fork in every project. Workspace-only leaves the spec prescribing a gate its consumers cannot run — the spec would be lying ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html)). |
| Project location | `workspace_root` from `cargo metadata` | `CARGO_MANIFEST_DIR` (the `xtask` way); current directory; walking up for `Cargo.toml` | `CARGO_MANIFEST_DIR` is baked in at the tool's own build and points at wherever it was compiled. The metadata document is already fetched for scope and members; it is the authority cargo itself uses, and it makes the tool cwd-independent for free. |
| Invocation in this workspace | `cargo run -p cargo-lid-rs -- mutants` | A cargo alias `lid-rs = "run -p cargo-lid-rs --"` (one gate line everywhere); `cargo install --path` then `cargo lid-rs mutants`; `cargo xtask mutants` delegating to the library | The alias was tried: it shadows an installed `cargo-lid-rs`, and cargo warns that alias-over-external shadowing "will become a hard error in the future" (rust-lang/cargo#10049) — anyone developing the tool also installs it. Installing first runs whatever binary was last installed, not the working tree. Delegation through `xtask` keeps a second entry point alive for no consumer. `cargo run` exercises the shipped binary from source; only cargo's argument prefix is left to its unit test. |
| Cargo's inserted argument | Discard a leading `lid-rs`; otherwise treat the arguments as given | Require the cargo form only; separate binaries for the two shapes | `cargo run` and direct execution deliver the arguments bare; cargo's external-subcommand protocol prefixes the name. Tolerating both is one comparison, and lets the workspace's from-source gate and a consumer's installed binary run the same shell. |
| Scope configuration for single packages | Workspace metadata first, then the root package's metadata, then `diff` | Require a `[workspace]` table; package metadata only | `cargo new` does not emit a `[workspace]` table, and asking every single-crate project to add an empty one to configure a mutation scope is friction with no gate behind it. Package-only would break every existing workspace user, including this one. |
| Members without a library | Skipped at collection | Attempt and tolerate the error; require `--lib` targets | `cargo test --lib` on a binary-only package fails with an error indistinguishable from a broken build; tolerating errors would also hide a real broken build. Membership is visible in the metadata's `targets`, so the decision is made from data, not from a failed subprocess. |
| Argument parsing | `std::env` loop | `clap` | One subcommand and two flags do not justify a dependency tree (tenet 3); `init` will be judged the same way when it arrives. |
| JSON / metadata parsing | `serde_json` untyped `Value` | `toml` crate; typed serde derives | One parsing dependency; `cargo metadata` re-serves `[workspace.metadata.lid_rs]` as JSON. Untyped access keeps the surface small. |
| Registry access | Per-crate dump via the `intent_graph!()`-emitted test, parsed from tab-separated lines | Linking the traced crate into the tool; parsing source | Disproven by the first real run in `xtask`: `#[cfg(test)]` validation edges never link into another binary, so every traced plan came back empty. Line format over JSON: no serializer in `lid-rs`, no escaping surface. Parsing source violates constraint 2. |
| Empty narrowed test set | Degrades to the full suite | Running the (empty) filtered set; erroring out | Running zero tests lets every mutant survive, which is itself a vacuous pass (constraint 3). Erroring would block brownfield crates where check 11 doesn't yet gate. |
| Mutant identity | `(file, ends-with ::fn_name)` join | Qualified names from cargo-mutants (not provided); span-based matching | The JSON provides file + unqualified name; file equality plus suffix match is exact for everything but same-file same-name fns, which merge into one safe over-approximated test set. |
| Verdict source | The group's own mutants, read from the engine's `outcomes.json` for that run | The engine's exit status; trusting `-F` to bound the run | Found in a consumer (`dmdr`): cargo-mutants 27.1.0's struct-field genre bypasses `-F` (upstream `visit.rs`), so stowaways rode along in every group and the exit status reported *their* survival against the wrong tests. Judging by name from the engine's own record is exact whatever the engine included. |
| Output directory | Fresh `<target>/lid-mutants/<n>` per group via `--output` | The engine's default `mutants.out` in the project root | A stale `outcomes.json` from an earlier run would be indistinguishable from this run's; per-group directories keep every group's evidence for inspection and leave the project root alone. |
| No verdict for a selected mutant | Failure naming the mutant | Treat as caught; treat as survivor | An engine that crashed or never built the mutant has proved nothing; silently passing is the vacuous pass constraint 3 forbids, and calling it a survivor misattributes a tooling failure to the tests. |
| Failure timing | All groups run, then every survivor is reported | Stop at the first group with a survivor | One run names every real survivor; stopping early hides the rest behind the first and costs a run per discovery. |
| Baseline handling | `--baseline skip` | Default baseline run per group | The gate runs the full suite before mutation ([§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) order); re-running it per group multiplies wall-clock for no information. |
| Test scope per mutant | `--test-workspace true` | Engine default (test only the mutated package) | Found by a surviving mutant: a broken `derive` in the proc-macro crate is killed only by `lid-rs`'s tests, and the package-scoped default never runs them. Cross-crate kill paths are the norm. |

## Open Questions & Future Decisions

### Deferred
1. `cargo lid-rs init` / `new` — project scaffolding, its own slice and LLD
   beside this one.
2. `--shard` support for large diffs (the engine supports `--sharding`; not
   needed at this repository's scale).
3. Nextest as the test tool once a consumer needs it (`--test-tool` exists).

## References

- README [§4.3](https://bradvoth.github.io/lid-rs/spec/gates.html) (non-vacuity by scoped mutation), [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (gate order),
  [§6.1](https://bradvoth.github.io/lid-rs/spec/traced.html) (untraced fallback).
- [`cargo-mutants` documentation](https://mutants.rs) — flags relied on:
  `--list --json`, `-F`, `--in-diff`, `--cargo-test-arg`, `--baseline`,
  `--test-workspace`.
- [The Cargo Book: external subcommands and aliases](https://doc.rust-lang.org/cargo/reference/config.html#alias) —
  the argument protocol the shell tolerates and the alias precedence the
  workspace relies on.
