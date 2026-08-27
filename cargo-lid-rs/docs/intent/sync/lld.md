# sync — a project's skill is the one its `lid-rs` ships

## Context and Design Philosophy

The operating skill has two halves: process guidance an agent could take
from anywhere, and a description of the mechanics the `lid-rs` crate
enforces — `intent_graph!()`, `implements_module!`, retirement via
`#[deprecated]`, the gate. The second half is only right for the crate
version it describes. A skill written into a project once and never
touched, or refreshed from whatever tool binary is installed, drifts from
that crate the first time either moves.

So the skill ships *inside* the `lid-rs` crate (`skill/` in its tarball — a
dispatcher `SKILL.md` plus `references/*.md` the phases, gates, and mechanics
detail live in, split so an agent loads only the phase it's in), and a
project's copy is derived from the `lid-rs` its manifest resolves: `cargo
lid-rs sync` reads `cargo metadata`, finds the resolved `lid-rs` package —
registry or path — and mirrors its `skill/` directory, file for file, to
`<workspace root>/.claude/skills/lid-rs/`. Updating the dependency is the
update; `sync` is how the directory catches up.

The copy is tool-owned, and strictly so: `cargo lid-rs sync --check` fails
when any file is missing, extra, or differs from the dependency's skill
directory, and it sits in every gate (README §4.5, this workspace's, and the
`gate.yml` `init` emits). A project that bumps `lid-rs` without syncing fails
its gate until it does; a project that edits any synced file locally fails
its gate until it reverts. Project-specific guidance belongs in `AGENTS.md`,
which `init` writes once and never touches. Strictness is the point: a gate
that tolerates an edited skill is a gate on nothing, and `SKILL.md`'s first
line says the directory is managed.

This workspace is a consumer like any other: its `.claude/skills/lid-rs/SKILL.md`
is produced by `sync` from the path dependency, and `sync --check` is in its
gate. The canonical file is edited in `lid-rs/skill/`, nowhere else.

## Behaviour

- `cargo lid-rs sync` — locate the project (`cargo metadata`, full, since the
  dependency graph is needed); find the package named `lid-rs` among the
  resolved packages; recursively read every file under
  `<its manifest dir>/skill/`; write each, at the same relative path, under
  `<workspace_root>/.claude/skills/lid-rs/`, creating directories. Idempotent.
  A project with no `lid-rs` dependency, or a `lid-rs` too old to ship a
  skill, fails naming which.
- `cargo lid-rs sync --check` — same lookup; compare the two directories'
  relative-path → content maps instead of writing; exit non-zero naming
  every file that is missing from the project's copy, present in the
  project's copy but not the dependency's, or differs in content. Writes
  nothing.
- `init` calls `sync` after `cargo add lid-rs`, so a new project's skill is
  its dependency's from the first commit; `init`'s conflict rule for the
  skill path stands (an existing `.claude/skills/lid-rs/` directory is a
  conflict, not a sync target).

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Source of truth | `skill/` inside the `lid-rs` crate — a directory, since 0.2.1 | A template in the `cargo-lid-rs` binary (the 0.1 arrangement); a plugin; a URL; a single `skill/SKILL.md` file (0.1–0.2) | The binary's version is independent of the project's `lid-rs`; the skill's mechanics half is not. Shipping with the crate makes "update the dependency" the update, with no second version to track. A single file forces every phase's detail to load whenever the skill triggers; a directory lets `SKILL.md` stay a dispatcher and `references/*.md` load per phase (`docs/intent/skill/lld.md`). The plugin remains the destination for the process half once the seam between the halves is found. |
| Local edits | Refused: `--check` fails on any difference, in any file; no `--force` | Tolerate edits with a marker or a `--force`; three-way merge | A skill that can drift is the failure this slice exists to remove; every relaxation reintroduces it. Project guidance has a home (`AGENTS.md`) that `init` never revisits. |
| Where the copy lives | `<workspace_root>/.claude/skills/lid-rs/` (a directory) | Beside the package that depends on `lid-rs` | Claude Code loads skills from the repository root; a workspace has one skill regardless of how many members depend on `lid-rs`. |
| Dependency lookup | The `lid-rs` package in full `cargo metadata` | Parsing `Cargo.lock`; a fixed registry path | Metadata is the authority for what resolved, handles path and registry sources alike, and is already the tool's only view of the project. The full graph costs a slower call, once. |
| Gate placement | `sync --check` after `cargo test --lib`, before mutation, in all copies of the gate | Only in CI; only in `init`'s emitted workflow | A gate that exists, gates — here, and in every project. Cheap and specific, so it runs before the expensive step. |
| This workspace's copy | Synced from the path dependency, gated like a consumer's | Keep `.claude/skills/lid-rs/` canonical and copy into the crate for packaging | One canonical directory and the tool exercising its own update path on its own repository. The book includes the canonical path. |
| Granularity of the diff | Compare the two directories as whole relative-path → content maps; a missing file, an extra file, and a differing file are all "a difference" | Compare `SKILL.md` only and let `references/*.md` drift silently; per-file flags | Progressive disclosure only works if every reference file stays in lockstep with the dispatcher that points at it; a partial sync (dispatcher current, references stale) is worse than no sync, because nothing signals it. "Any difference" already covered this once the skill is modeled as a directory's contents rather than one file's bytes — no new claim, the existing one generalizes. |

## Open Questions & Future Decisions

### Deferred
1. Syncing project-owned files. `AGENTS.md`, `gate.yml`, and `clippy.toml`
   are project-owned after `init`. (The phase agents and the workflow are
   synced since 0.2.2 — `docs/intent/phase/lld.md` — as further rows of the
   same mirror table, under the same any-difference rule.)
2. The plugin seam (skill LLD, Deferred 1).

## References

- `docs/intent/init/lld.md` — where the skill first lands in a project.
- `docs/intent/skill/lld.md` (workspace) — what the skill must contain and
  why it is evidence-derived.
- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html), [§11](https://bradvoth.github.io/lid-rs/spec/layout.html), [§13](https://bradvoth.github.io/lid-rs/spec/bootstrap.html) — the gate line, the layout row, and the keeping-current step this slice adds.
