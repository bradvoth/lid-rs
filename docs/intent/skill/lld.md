# The operating skill — an agent runs the methodology

## Context and Design Philosophy

The toolchain enforces structure; it cannot make an agent *propose* the right
artifacts in the right order. The skill closes that gap: it is the standing
instruction set under which an agent walks a Rust change through the
phases (README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html)), producing claims as `derive(Spec)` structs, skeletons as
`todo!()` signatures, validations as failing-first `#[validates]` tests, and
running the gate before every commit. Human authorship stays concentrated in
Phases 1 and 3; the skill exists so the agent's share is done inside the
constraints rather than reviewed into them afterwards.

It lives at `.claude/skills/lid-rs/SKILL.md` in this repository (repo-local
now; promotion to a distributable plugin is out of scope — HLD
Non-Goals).

## This slice is untraced, by design

The skill is prose executed by an agent, not code the compiler sees. A spec
claim about it ("when the agent begins a slice, it shall draft an LLD first")
has no implementation edge any build can resolve and no test any gate can
run — and a claim that cannot gate doesn't get written (constraint 3). So
this slice contributes no `derive(Spec)` items. Its validation is the process
evidence the HLD's Goal 5 names: this repository's git history is a sequence
of slices produced under the discipline the skill encodes, and the skill text
is derived from that history's findings rather than from aspiration.

## Content requirements

The skill must carry, in this order of importance:

1. **The phase walk** — every phase with its LID-rs mechanics, each
   phase's exit condition, and the stop discipline (present, get approval,
   proceed). Bugs walk the arrow like any other change.
2. **The non-negotiables** — dispatch/work separation; a leaf with a branch is
   an unwritten requirement; never suppress a firing lint (the lint firing
   is the system working); `#[validates]` only on `#[cfg(test)]` unit tests
   inside the library; claims are EARS-shaped doc comments on descriptive
   names.
3. **Gate-failure remediation** — for each check, what its firing means and
   what the correct response is, with the check-7 rule spelled out: the fix is
   returning to Phase 1 to write the missing claim, never raising the
   threshold.
4. **Mechanics discovered building this workspace** — the crate-scoped graph
   checks, `intent_graph!()` placement, module tracing via
   `implements_module!`, spec retirement via `#[deprecated]` (definition site
   stays clean; citation sites warn), the required linkme wrapper
   attribute, and the bootstrap/brownfield adoption order.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Traceability of this slice | Untraced; process-evidence validation | Ceremonial `Spec` structs for skill behaviour; a skill-lint script | Ceremonial specs make the graph look denser than it is (README [§6](https://bradvoth.github.io/lid-rs/spec/traced.html)). A script checking prose cannot check what matters (did the agent *follow* it). |
| Skill scope | Operates LID-rs on any Rust codebase that adopts it, this repo included | A repo-specific runbook | The skill is a deliverable of the system; this repo is its first consumer, not its subject. |
| Relationship to the installed `linked-intent-dev` skill | This skill owns Rust-artifact mechanics; the generic skill's process discipline (stops, cascade, coherence pre-flight) is incorporated by restatement, not by reference | Deferring to the generic skill plus a delta document | A skill that requires composing two documents at run time is a skill agents will half-follow; restating the process rules keeps one authoritative walk. |
| Failure remediation format | Per-check table with meaning and correct response | Prose guidance | The moment of use is "a gate just fired"; a lookup table matches it. |

## Open Questions & Future Decisions

### Deferred
1. Plugin packaging (marketplace metadata, versioning) — a later slice, after
   the skill has operated on at least one non-self-hosted codebase.
2. A `/lid-differential` companion pass for the semantic residual (README
   [§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)) — scheduled reconstruction of claims from code and diffing against
   the written ones.

## References

- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the flow), [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate), [§11](https://bradvoth.github.io/lid-rs/spec/layout.html) (bootstrap and brownfield).
- This repository's git history — the process evidence the skill distills.
