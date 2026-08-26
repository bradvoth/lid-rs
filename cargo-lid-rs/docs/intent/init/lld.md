# init and new — a LID-ready package in one command

## Context and Design Philosophy

README §13 is an eleven-step checklist for making a package LID-ready:
dependency, lint levels, clippy thresholds, mutation profile, an HLD wired
into `lib.rs`, a `spec` module, the graph checks, a CI gate, and an agent
instruction file. Every step is mechanical and every step is easy to get
subtly wrong — a lint left at `warn`, an `include_str!` one directory off, a
`[profile.test]` forgotten so inlining erases mutation sites. `cargo lid-rs
init` performs the checklist; `cargo lid-rs new <name>` runs `cargo new
--lib` and then performs it.

The design restructures the problem rather than templating it: `cargo new`
already produces a correct package, and `init` *augments* whatever package it
is run in. That makes `init` the brownfield-adoption path README §11
describes in prose (Tier 0 first: the lints apply to existing code
immediately) as well as the greenfield one, and it means the tool never
carries a `Cargo.toml` template that drifts from what cargo emits.

No new dependency is taken. The dependency line is written by `cargo add`
— cargo's own manifest editor, present since 1.62 — and every other manifest
change is a *new table* appended to the file, which is safe by construction
when the table is absent and refused when it is present. Files are emitted
from `include_str!` templates with placeholder markers that cannot occur in
Rust or TOML (`__LID_PACKAGE_NAME__`), substituted with `str::replace`. A
templating engine or a TOML editor is the escalation path, taken only when
append-only demonstrably fails.

`init` is one-shot and all-or-nothing: it computes everything it would
create, checks every target for a conflict, and writes nothing unless every
target is free. A half-initialised package is worse than an untouched one,
and a second run must not silently re-append tables.

## What init emits

Run in a directory holding a package manifest (`Cargo.toml` with
`[package]`). `<name>` is the package name from `cargo metadata`.

| Artifact | Mechanism | Conflict rule |
|---|---|---|
| `lid-rs` dependency | `cargo add lid-rs@<tool version>` (`--path <dir>` when `--lid-rs-path` is given) | cargo's own: an existing entry is updated |
| `[lints.rust]`, `[lints.rustdoc]`, `[lints.clippy]` (README §7 levels) | appended tables | any `[lints` table present → conflict |
| `[package.metadata.lid_rs] mutation_scope = "diff"` | appended table | `[package.metadata.lid_rs]` present → conflict |
| `[profile.test] opt-level = 0` | appended table | `[profile.test]` present → conflict |
| `clippy.toml` (README §7 thresholds) | new file | exists → conflict |
| `docs/intent/hld.md` — HLD skeleton with the section headings this workspace's HLD uses | new file | exists → conflict |
| `src/spec/mod.rs` — `//!` doc, no claims | new file | exists → conflict |
| `src/lib.rs` — `#![doc = include_str!("../docs/intent/hld.md")]` prepended; `pub mod spec;` and the `intent_graph!()` test module appended | edit in place; created if absent (a bin-only package gains a library target, which LID needs for `#[validates]` tests) | `intent_graph!` already present → conflict |
| `.github/workflows/gate.yml` — README §4.5 in order, installing `cargo-mutants` and `cargo-lid-rs` | new file | exists → conflict |
| `.gitignore` — `mutants.out/` | appended line | line present → skipped, not a conflict |
| `AGENTS.md` — the eight phases, the dispatch/work rule, the gate, and where the full skill lives; `CLAUDE.md` importing it (`@AGENTS.md`) | new files | either exists → conflict |
| `.claude/skills/lid-rs/SKILL.md` — the operating skill | new file, from the template copy of this repository's skill | exists → conflict |

`new <name>` runs `cargo new --lib <name>`, empties the generated `src/lib.rs`
(cargo's `add` function carries no doc comment and would fail
`missing_docs = "deny"` on the first gate run; the file itself must stay,
since a package with no target has no metadata), and then performs `init` in
the new directory, whose wiring turns the empty library into the documented
skeleton. Lint levels and thresholds come from the same
constants the emitted files are rendered from, so `new` and `init` cannot
disagree.

The lint levels go in `[lints.*]` on the package, not `[workspace.lints]`:
`cargo new` emits no `[workspace]` table, and a package-level table is what a
single-package project reads. A workspace member is not a target of this
slice (see Deferred).

## The skill in the template

The operating skill is the standing instruction an agent loads to run the
methodology; without it an `AGENTS.md` summary is a table of contents for a
book the agent has not read. The canonical file is
`.claude/skills/lid-rs/SKILL.md`, where this workspace's own agent loads it —
and that path is outside `cargo-lid-rs`'s package root, so the tool cannot
`include_str!` it (the lesson of `docs/intent/publish/lld.md`). The template
is therefore a copy at `cargo-lid-rs/templates/SKILL.md`, and a unit test
asserts the copy is byte-identical to the canonical file: a moved or edited
skill fails `cargo test --lib` until the copy is refreshed. Drift between two
copies in one repository is gated; drift between a repository and every
project it ever scaffolded is not, and is accepted — an emitted skill is a
snapshot at the tool's version, which is what a snapshot of the toolchain's
dependency is too.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Model | Augment an existing package (`init`); `new` = `cargo new --lib` + `init` | Generate the whole package from an embedded template (loco, cargo-pgrx); cargo-generate as a library over an embedded template | Augmenting reuses cargo's own package shape and serves brownfield adoption for free; a full template carries a `Cargo.toml` that drifts from `cargo new`. cargo-generate is 262 crates including libgit2 and a Rhai interpreter, built around cloning a repository, and blanks undefined placeholders silently. |
| Manifest editing | `cargo add` for the dependency; append-only for new tables; conflict on any existing table | `toml_edit` (8 crates, what `cargo add` uses) | Every table `init` adds is one `cargo new` never emits, so append-only is exact on the greenfield path and safely refusable on brownfield. `toml_edit` is the escalation if a real manifest defeats append-only (tenet 3). |
| Templating | `include_str!` files with `__LID_*__` markers and `str::replace` | `format!` on template text (cargo-pgrx); `minijinja` (4 crates); `tera` (3 crates) | `format!` requires doubling every literal brace in Rust and TOML templates. The markers cannot occur in valid Rust or TOML, so substitution needs no escaping and no engine; an engine is justified only when a template needs a conditional. |
| Dependency source | `lid-rs@<tool version>` by default; `--lid-rs-path <dir>` overrides | Unversioned `lid-rs`; a `--lid-rs-version` flag | The tool and the crate ship from one workspace at one version, so the tool's own version is the right pin. The path override is how this workspace tests `init` end to end before `lid-rs` is on crates.io, and how a contributor works against a checkout. |
| Atomicity | Compute all targets, check all conflicts, then write all | Write as you go and stop at the first conflict; `--force` to overwrite | A partial `init` leaves a package that is neither plain nor LID-ready and cannot be re-run. Overwriting is never right for a file the user wrote: the conflict message names the file and the user decides. |
| Bin-only packages | `init` creates `src/lib.rs`; the existing binary is untouched | Refuse; require `--lib`; document `main.rs` on the user's behalf | `cargo new` defaults to a binary. LID's validations live in the library test binary (§5.2), so a package without a library cannot run checks 10–12; creating the library is the smallest change that makes the package eligible, and cargo discovers it without a manifest edit. The binary is the user's code: the lint levels apply to it immediately (§11, brownfield), so the package's first full gate run names its undocumented `main` — the intended adoption experience, not a defect of `init`. |
| `new`'s `lib.rs` | Emptied, then wired by `init` into the documented skeleton | Keep cargo's `add` function and document it in place; delete the file and let `init` create it | cargo's boilerplate exists to be replaced; documenting a placeholder function is busywork the first slice deletes anyway. Deleting was tried and fails: `cargo metadata` rejects a package with no target, so `init` cannot locate it. `init` on an existing package never replaces `lib.rs`. |
| Agent instructions | `AGENTS.md` with the phases, the rule, and the gate; `CLAUDE.md` = `@AGENTS.md`; the skill emitted verbatim | `CLAUDE.md` only; a URL to the published skill and nothing local; wait for the plugin | `AGENTS.md` is the cross-tool convention and `CLAUDE.md` imports it, so both Claude Code and other agents read one text. A URL alone leaves an agent that does not fetch operating blind. The plugin (HLD non-goal) remains the long-term home; until it exists, the emitted copy is the only way a new project gets the skill at all. |
| Skill drift | Template copy gated byte-equal to the canonical skill by a unit test | `include_str!` of the canonical path (fails from the tarball); making the template the canonical location and the `.claude/` file a copy | The test is the gate; which file is canonical is a naming choice, and the one Claude Code loads should be the one humans edit. |
| Workspace members | Out of scope; `init` targets the package in the current directory | Detect membership and write `[workspace.lints]` + `[lints] workspace = true` | Two manifests, two conflict rules, and a package that inherits lints from a root `init` did not write. Deferred until a member project asks for it. |

## Open Questions & Future Decisions

### Deferred
1. `init` inside a workspace member (`[workspace.lints]` + `[lints] workspace = true`).
2. A thin `src/bin/` alongside the library for `new` (README §11's layout);
   a library-only package is a valid starting point.
3. `--vcs` pass-through to `cargo new`.
4. Skill delivery, to be revisited. Emitting a snapshot couples the skill's
   version to the tool's, which is right for the parts of the skill that
   describe what the toolchain enforces and wrong for the parts that are
   process guidance and could update independently through a plugin. Where
   that seam lies is not yet clear — the coupling between skill and
   implementation is fuzzy — and this design should be broken up once it is:
   likely a plugin for the process half and an emitted, version-pinned
   fragment for the toolchain half.

## References

- README [§7](https://bradvoth.github.io/lid-rs/spec/configuration.html) (configuration), [§11](https://bradvoth.github.io/lid-rs/spec/layout.html) (layout and brownfield adoption), [§13](https://bradvoth.github.io/lid-rs/spec/bootstrap.html) (the checklist this command performs).
- [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html) — the manifest editor relied on.
- `docs/intent/publish/lld.md` — why nothing outside a package root can be `include_str!`'d.
