# init and new — a LID-ready package in one command

## Context and Design Philosophy

README §13 is an eleven-step checklist for making a package LID-ready:
dependency, lint levels, clippy thresholds, mutation profile, an HLD wired
into `lib.rs`, the first slice's document and claims file, the graph checks, a
CI gate, and an agent instruction file. Every step is mechanical and every step is easy to get
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
| `src/lld.md` — the crate-root slice's document: the package's first slice is the crate root until a `src/<slice>/` directory exists | new file | exists → conflict |
| `src/spec.rs` — the crate-root slice's claims file: `//!` doc, no claims | new file | exists, or a `src/spec/` directory exists (the claims file `init` writes would become that directory's parent module: a `mod.rs` inside makes `pub mod spec;` ambiguous, and files no module declares are the user's to resolve first) → conflict |
| `src/lib.rs` — `#![doc = include_str!("../docs/intent/hld.md")]` and `#![doc = include_str!("lld.md")]` prepended; `pub mod spec;` and the `intent_graph!()` test module appended | edit in place; created if absent (a bin-only package gains a library target, which LID needs for `#[validates]` tests) | `intent_graph!` already present → conflict |
| `.github/workflows/gate.yml` — README §4.5 in order, installing `cargo-mutants` and `cargo-lid-rs` | new file | exists → conflict |
| `.gitignore` — `mutants.out/` | appended line | line present → skipped, not a conflict |
| `AGENTS.md` — the eight phases, the dispatch/work rule, the gate, the colocated artifact paths of README §11.1, and where the full skill lives; `CLAUDE.md` importing it (`@AGENTS.md`) | new files | either exists → conflict |
| `.claude/skills/lid-rs/SKILL.md` — the operating skill | `cargo lid-rs sync` from the resolved `lid-rs` dependency (`docs/intent/sync/lld.md`), run after `cargo add` | exists → conflict |

`new <name>` runs `cargo new --lib <name>`, replaces the generated `src/lib.rs` with the documented template
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

## The package's first slice is its crate root

A freshly initialised package has no slice directory, and `init` invents no
name for one: the crate root *is* the first slice, which is README §11.1's
second shape. So the scaffold is a slice in the colocated layout — `src/lld.md`
as the slice's document, `src/spec.rs` as its claims file, both declared from
`src/lib.rs` (`#![doc = include_str!("lld.md")]` beside the HLD include, and
`pub mod spec;`). The HLD stays at `docs/intent/hld.md`, which is where it
belongs in either layout.

The library therefore carries two inner doc attributes, and rustdoc
concatenates them: the package's high-level design, then the first slice's
document. `src/spec.rs` holds its `//!` doc and no claims, because Phase 2 of
the first slice is the human's, not `init`'s.

`src/lld.md` is a short document rather than a stub. It states that the
package's first slice is the crate root until a `src/<slice>/` directory
exists, and it carries a `## Decisions & Alternatives` heading with one
four-cell row — because `cargo lid-rs lld-check` refuses a document without
one (`lld_review/lld.md`), and a scaffold that cannot pass the check the
package's own Phase 1 runs is a scaffold that fails the gate it installed.

The proof that the scaffold *is* a crate-root slice is in-process, not a
subprocess: the validator of
`AnInitialisedPackageIsACrateRootSliceThatPassesItsOwnGate` opens the
scaffolded package as a `Project`, asserts `layout::Form::of_slice` answers
`CrateRoot` for the package's name, and runs `lld_review`'s checks over the
scaffolded `src/lld.md` the way `cargo lid-rs lld-check` does, asserting they
report nothing — beside the gate run it already makes. The template is data
and can be wrong in no other observable way, so the claim on that validator
is what makes the one-row Decisions table a requirement rather than a habit.

### What the layout resolver requires of the scaffold

Verified against a scratch `cargo new` package with the tool built from this
workspace. `layout::Form::of_slice` answers `CrateRoot` for a slice when two
things hold, and only those two:

1. The crate's `src/lld.md` exists — the document's presence is the only thing
   that marks a directory a slice.
2. The slice's name is the manifest's `[package] name`, spelled exactly as the
   manifest spells it, hyphens and all — `src/lld.md` names no slice itself, so
   the package name is where the name comes from.

Nothing reads `src/spec.rs` to decide the form; `layout::spec_file` *derives*
`src/spec.rs` from the form once `CrateRoot` is settled. So `cargo lid-rs
lld-check --slice <package-name>` resolves the scaffold's `src/lld.md`, and
the first slice's phases find their claims file where the resolver says it is.

One shape defeats it, and it is brownfield only: a package that already holds
`src/<package-name-as-a-module>/` — package `demo-pkg` with a `src/demo_pkg/`
— is read as a *module* slice, and its document is looked for at
`src/demo_pkg/lld.md`. `init` writes `src/lld.md` there as everywhere; that
package's crate-root slice is unresolvable until the module is renamed or that
directory carries its own document. `init` neither detects this nor refuses
it: it is the general brownfield bargain — the package is made LID-ready and
the first gate run names what the package already had (Deferred 5).

## The skill

The operating skill is the standing instruction an agent loads to run the
methodology; without it an `AGENTS.md` summary is a table of contents for a
book the agent has not read. `init` does not carry the skill: it ships inside
the `lid-rs` crate, and `init` obtains it the way every later update does —
`sync` from the dependency `cargo add` just resolved
(`docs/intent/sync/lld.md`). The skill a project gets is therefore the one
that matches its `lid-rs`, not the one the installed tool happened to embed.

## What lands by hand, and in what order

Two things no phase of this slice may write. The phase policy's rows for a
module slice name `src/init/`, plus `src/lib.rs` at Phases 3 and 4
(`cargo-lid-rs/src/phase/policy.rs`, `own_table`); nothing under
`cargo-lid-rs/templates/`, which is where every file `plan()` includes lives.

**1. The new and reworded template files — after Phase 1, before Phase 3.**
`templates/lld.md` (the crate-root slice's document, with its one-row
Decisions table), `templates/spec.rs` (the `//!` doc `templates/spec_mod.rs`
carries, reworded for a file beside `lib.rs`), `templates/lib_header.rs` (the
second `#![doc = include_str!("lld.md")]` line), `templates/AGENTS.md` (the
artifact table and the phase list spelling `src/<slice>/lld.md`,
`src/<slice>/spec.rs`, and the crate-root pair), and `templates/hld.md` (its
one mention of a slice document's path). They must precede Phase 3 because
`plan()` reaches them through `include_str!`, so the skeleton's `cargo check`
fails on a missing file. `templates/spec_mod.rs` stays in this commit: `plan()`
still includes it until Phase 3 rewrites the entry, and a deleted include is a
red `cargo check` at the hand commit itself.

**2. Deleting `templates/spec_mod.rs` — after Phase 3.** Once the skeleton's
`plan()` no longer names it, the file is dead and goes. Both commits are the
human's, with this section as their reason.

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
| Agent instructions | `AGENTS.md` with the phases, the rule, and the gate; `CLAUDE.md` = `@AGENTS.md`; the skill synced from the dependency | `CLAUDE.md` only; a URL to the published skill and nothing local; wait for the plugin; a skill template embedded in the tool (the 0.1 arrangement) | `AGENTS.md` is the cross-tool convention and `CLAUDE.md` imports it, so both Claude Code and other agents read one text. A URL alone leaves an agent that does not fetch operating blind. A template in the tool describes the tool's version, not the project's `lid-rs`; syncing from the dependency ties the skill to the crate whose mechanics it documents and gives projects an update path. |
| Scaffold layout | The crate root is the package's first slice: `src/lld.md` and `src/spec.rs`, wired from `src/lib.rs`; the HLD stays at `docs/intent/hld.md` | Keep the pre-colocation `src/spec/mod.rs` re-exporting a file per slice; scaffold a named first slice directory `src/<name>/{lld.md,spec.rs,mod.rs}`; ask `layout` for the paths at scaffold time | README §11.1 makes an `lld.md` in a directory the only marker of a slice, so a scaffold emitting `src/spec/mod.rs` teaches a layout the resolver, the phase policy and the book no longer read — and leaves the package's own first slice with nowhere its document resolves. A named first slice directory would have `init` make Phase 0's decision, which is the decision with the largest downstream cost in the methodology, and would leave a dead directory when the human names the slice something else. Asking `layout` at scaffold time is asking a resolver where a slice is that does not exist yet: every door there answers from a document on disk, so the only answer available before `init` writes one is the refusal — `init` writes the layout, it does not read it, and the check that it wrote the right one is `lld-check` run afterwards. Claims to reword at Phase 2, each renamed with a `#[deprecated]` alias: `InitWiresTheLibraryIntoTheGraph` (the library is wired with two doc includes and a sibling `spec` module, not one include and a `spec` directory) and `AnInitialisedPackagePassesItsOwnGate` (the end-to-end proof now also resolves the scaffold as a crate-root slice). Claims this change adds or rewords are written in the controlled language and carry no `#[lid(free)]` mark; the slice's other marks stay until its own burn-down (the ramp's per-slice rule, `lid-rs-macros/src/claim/lld.md`). |
| Workspace members | Out of scope; `init` targets the package in the current directory | Detect membership and write `[workspace.lints]` + `[lints] workspace = true` | Two manifests, two conflict rules, and a package that inherits lints from a root `init` did not write. Deferred until a member project asks for it. |

## Open Questions & Future Decisions

### Deferred
1. `init` inside a workspace member (`[workspace.lints]` + `[lints] workspace = true`).
2. A thin `src/bin/` alongside the library for `new` (README §11's layout);
   a library-only package is a valid starting point.
3. `--vcs` pass-through to `cargo new`.
4. Skill delivery via a plugin for the process half. The skill now tracks
   the `lid-rs` version, which is right for the parts that describe what the
   toolchain enforces and a constraint on the parts that are process guidance
   and could update independently. Where that seam lies is not yet clear;
   break the skill up once it is.
5. `init` on a package that already holds a module named for the package, whose
   crate-root slice the layout resolver reads as a module slice.

## References

- README [§7](https://bradvoth.github.io/lid-rs/spec/configuration.html) (configuration), [§11.1](https://bradvoth.github.io/lid-rs/spec/layout.html) (the colocated layout the scaffold is in), [§11.3](https://bradvoth.github.io/lid-rs/spec/layout.html) (brownfield adoption), [§13](https://bradvoth.github.io/lid-rs/spec/bootstrap.html) (the checklist this command performs).
- [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html) — the manifest editor relied on.
- `docs/intent/publish/lld.md` — why nothing outside a package root can be `include_str!`'d.
- `cargo-lid-rs/src/layout/lld.md` — the four shapes a slice's directory has, and the doors (`slice_dir`, `lld_path`, `spec_file`) the scaffold must resolve through.
- `cargo-lid-rs/src/lld_review/lld.md` — the checks the scaffolded `src/lld.md` must pass.
