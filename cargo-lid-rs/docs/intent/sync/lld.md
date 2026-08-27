# sync — a project's skill is the one its `lid-rs` ships

## Context and Design Philosophy

The operating skill has two halves: process guidance an agent could take
from anywhere, and a description of the mechanics the `lid-rs` crate
enforces — `intent_graph!()`, `implements_module!`, retirement via
`#[deprecated]`, the gate. The second half is only right for the crate
version it describes. A skill written into a project once and never
touched, or refreshed from whatever tool binary is installed, drifts from
that crate the first time either moves.

So the skill ships *inside* the `lid-rs` crate (`skill/SKILL.md` in its
tarball), and a project's copy is derived from the `lid-rs` its manifest
resolves: `cargo lid-rs sync` reads `cargo metadata`, finds the resolved
`lid-rs` package — registry or path — and writes its skill to
`<workspace root>/.claude/skills/lid-rs/SKILL.md`. Updating the dependency is
the update; `sync` is how the file catches up.

The copy is tool-owned, and strictly so: `cargo lid-rs sync --check` fails
when the file differs from the dependency's skill, and it sits in every gate
(README §4.5, this workspace's, and the `gate.yml` `init` emits). A project
that bumps `lid-rs` without syncing fails its gate until it does; a project
that edits the file locally fails its gate until it reverts. Project-specific
guidance belongs in `AGENTS.md`, which `init` writes once and never touches.
Strictness is the point: a gate that tolerates an edited skill is a gate on
nothing, and the file says in its first line that it is managed.

This workspace is a consumer like any other: its `.claude/skills/lid-rs/SKILL.md`
is produced by `sync` from the path dependency, and `sync --check` is in its
gate. The canonical file is edited in `lid-rs/skill/`, nowhere else.

## Behaviour

- `cargo lid-rs sync` — locate the project (`cargo metadata`, full, since the
  dependency graph is needed); find the package named `lid-rs` among the
  resolved packages; read `<its manifest dir>/skill/SKILL.md`; write it to
  `<workspace_root>/.claude/skills/lid-rs/SKILL.md`, creating directories.
  Idempotent. A project with no `lid-rs` dependency, or a `lid-rs` too old to
  ship a skill, fails naming which.
- `cargo lid-rs sync --check` — same lookup; compare instead of write; exit
  non-zero naming the file when it is absent or differs. Writes nothing.
- `init` calls `sync` after `cargo add lid-rs`, so a new project's skill is
  its dependency's from the first commit; `init`'s conflict rule for the
  skill path stands (an existing file is a conflict, not a sync target).

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Source of truth | `skill/SKILL.md` inside the `lid-rs` crate | A template in the `cargo-lid-rs` binary (the 0.1 arrangement); a plugin; a URL | The binary's version is independent of the project's `lid-rs`; the skill's mechanics half is not. Shipping with the crate makes "update the dependency" the update, with no second version to track. The plugin remains the destination for the process half once the seam between the halves is found. |
| Local edits | Refused: `--check` fails on any difference; no `--force` | Tolerate edits with a marker or a `--force`; three-way merge | A skill that can drift is the failure this slice exists to remove; every relaxation reintroduces it. Project guidance has a home (`AGENTS.md`) that `init` never revisits. |
| Where the copy lives | `<workspace_root>/.claude/skills/lid-rs/SKILL.md` | Beside the package that depends on `lid-rs` | Claude Code loads skills from the repository root; a workspace has one skill regardless of how many members depend on `lid-rs`. |
| Dependency lookup | The `lid-rs` package in full `cargo metadata` | Parsing `Cargo.lock`; a fixed registry path | Metadata is the authority for what resolved, handles path and registry sources alike, and is already the tool's only view of the project. The full graph costs a slower call, once. |
| Gate placement | `sync --check` after `cargo test --lib`, before mutation, in all copies of the gate | Only in CI; only in `init`'s emitted workflow | A gate that exists, gates — here, and in every project. Cheap and specific, so it runs before the expensive step. |
| This workspace's copy | Synced from the path dependency, gated like a consumer's | Keep `.claude/skills/lid-rs/SKILL.md` canonical and copy into the crate for packaging | One canonical file and the tool exercising its own update path on its own repository. The book includes the canonical path. |

## Open Questions & Future Decisions

### Deferred
1. Syncing anything besides the skill. `AGENTS.md`, `gate.yml`, and
   `clippy.toml` are project-owned after `init`; if a future `lid-rs`
   changes the gate itself, that is a `sync` candidate with its own
   conflict rule.
2. The plugin seam (skill LLD, Deferred 1).

## References

- `docs/intent/init/lld.md` — where the skill first lands in a project.
- `docs/intent/skill/lld.md` (workspace) — what the skill must contain and
  why it is evidence-derived.
- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html), [§11](https://bradvoth.github.io/lid-rs/spec/layout.html), [§13](https://bradvoth.github.io/lid-rs/spec/bootstrap.html) — the gate line, the layout row, and the keeping-current step this slice adds.
