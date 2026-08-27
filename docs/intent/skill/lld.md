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

The skill is evidence-derived. Its first source was this repository's own
history; its second is the first project scaffolded and built under it by an
agent without the toolchain's authors in the loop (`dmdr`, a terminal
Markdown viewer, three slices, August 2026). That deployment passed every
compiler-enforced gate and still produced a module of 62 functions with 49
untraced, three LLDs that name items which do not exist, claims cited across
slice boundaries, a check-12 finding "fixed" by relocation, and a tool bug
misdiagnosed with "full proof" that discriminated nothing. None of that is a
gate failure; all of it is what the skill exists to prevent. So the skill
carries, beside the phase walk, the specific rules those outcomes violated —
each stated as something the agent can check mechanically at the moment it
applies, with the worked example that earned it. A rule without its example
is advice; the example is what makes it recognisable in the moment.

The canonical skill is `lid-rs/skill/SKILL.md`: it ships inside the `lid-rs`
crate, so a project's skill is the one that matches the `lid-rs` it depends
on. Every working copy — this repository's `.claude/skills/lid-rs/SKILL.md`
included — is produced by `cargo lid-rs sync` from the project's resolved
dependency and checked by the gate (`cargo-lid-rs/docs/intent/sync/lld.md`).
There is one file and no copies to keep equal.

## This slice is untraced, by design

The skill is prose executed by an agent, not code the compiler sees. A spec
claim about it ("when the agent begins a slice, it shall draft an LLD first")
has no implementation edge any build can resolve and no test any gate can
run — and a claim that cannot gate doesn't get written (constraint 3). So
this slice contributes no `derive(Spec)` items. Its validation is process
evidence: this repository's git history, and the transcripts and code of the
projects built under it, read against what the skill said to do. The
`dmdr` review is the first such reading; the rules below cite it.

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
5. **Where discipline slips** — the moments at which a principle from 1–3
   is silently abandoned, each paired with the check the agent performs
   then. The skill carries the moment and the check; the evidence that
   earned each rule stays in this table, not in the skill:
6. **Working state** — a slice's branch and its phase-tagged commits are the
   only state the skill keeps between turns. No file records "which phase am
   I on"; that answer comes from `git log` on the current branch, which is
   also what makes an LLD's PR reviewable by walking commits in phase order
   instead of one squashed diff.

| Rule | The moment it applies | Evidence (`dmdr`) |
|---|---|---|
| **A change after Phase 3 is a Phase 8 event.** A new type, function, or decision that appears while implementing is an LLD edit *first*, then a claim, then code. At Phase 7, every backticked identifier in the LLD's shape table resolves in `src/`. | Phase 6/7, the instant a helper or enum is about to be written | `NavigationOutcome`, `OpenDocument`, `HeadingAnchor`, `ViewerKey`, `apply_loop_signal` exist in code and in no LLD; every LLD names functions that don't exist. |
| **A slice cites only its own claims.** A method slice *n* adds to slice *n−1*'s type is slice *n*'s code: an `impl` block or free fn in slice *n*'s module, or the type moves. | Phase 2/3, when reuse of an earlier claim looks economical | `ScrollState::resize` in `read_document.rs` cites a `live_reload` claim; the tests that kill `render_block` and `jump_to` mutants sit in another slice's plan, so they survive. |
| **Untraced helpers belong to the slice that adds them.** A later slice adding helpers to an earlier module grows that module's fallback bucket; report each module's untraced count at Phase 7. | Phase 6, when the natural file for a helper is an older one | `read_document.rs`: 962 → 1198 lines across three slices, 49 of 62 fns untraced, 22 with branches; its fallback set now includes a `live_reload` test. |
| **Check 12 on a cited function's branch has two fixes: write the claim, or delete the branch.** Moving the branch into an untraced function is threshold-raising by another name. | Phase 7, when a surviving mutant is in a `match` arm | `handle_key`'s scroll arms survived → extracted to untraced `apply_scroll_key` ("plumbing, not a claim") → `apply_scroll_key → ()` survives in the next full run. |
| **Before diagnosing the planner, test the engine's narrowing.** If a cited test kills a mutant by hand, run `cargo mutants --list -F '<the group's regex>'`; it must list exactly the group. Two hypotheses and the observation that separates them come before any "proof". | Phase 7, a survivor that the intended test kills by hand | Seven struct-field mutants "missed" against `apply_reload`'s tests; declared a plan-grouping bug on an argv diff that discriminated nothing; the engine had ignored `-F` for that mutant kind. |
| **A waiver is per slice, restated at Phase 0; a fork directive may not waive Phase 5.** A large slice makes the red run a separate fork step with its output pasted, not a step to skip. | Phase 0 and at every fork | "continue through implementation" at 19% was reused at 81%; fork 2 was told "you don't need a strict red-confirmation pass"; implementation landed in Phase 4 skeletons and "tests already pass" was noted, not explained. |
| **`#[mutants::skip]` is a suppression.** Allowed only on pure I/O sequencing, only with the reason in the LLD's decisions table, only after asking. | Phase 7, when a survivor is in a function that "just sequences" | Added silently with a new dependency on `run_viewer`, then described to the user as "the tool's own sanctioned escape hatch". |
| **Phase 2 checklist per claim:** one *when*, one *shall*; no parenthetical alternatives; both halves name something a test can construct or observe; an implementer for it exists in the shape table. | Phase 2 | `TabCyclesLinkFocusWithWraparound` ("Tab (or Shift-Tab) … next (or previous)") is two claims; link *position* — slice 3's central mechanism — had no claim, so its tests were hung off list and table claims. |
| **A foreign enum is matched in exactly one function: the boundary that translates it into a domain enum whose variants are the design's own cases.** Interior code matches the domain enum, so `wildcard_enum_match_arm` guards the design's closed set (each variant a claim) instead of forcing every function to enumerate a library's; an upstream variant surfaces at the boundary once (check 6). The boundary's mapping is a set of decisions and gets claims like any other. Count the match sites before answering "one". | Phase 1/3, when a dependency's enum enters the design; Phase 6, when a second match on it is about to be written | Done for crossterm (`ViewerKey`, `translate_key`), declined for `pulldown_cmark::Event` as "one consumer, one match site" — seven exhaustive matches followed, each repeating a 12-arm ignore list. And `translate_key`'s 11 bindings are untraced: the wrapper isolated the enum but not its own decisions, so `apply_scroll_key → ()` survives. |
| **Corollaries of the boundary rule, by shape.** *Structs and events*: the boundary projects a library struct onto the fields the design uses; the interior never reads the library type. *Errors*: a foreign error kind is translated into the design's own error enum, whose variants are the LLD's failure modes. *Open sets*: a string or number the design branches on is classified once, at the boundary, into a domain enum or a named constant. *Shapes*: a foreign structure (an event stream) is converted into the domain structure (a block tree) once; consumers walk the domain shape. *Output*: rendering is a second boundary; interior functions return domain data, one function turns it into library output. *Trait or enum*: an enum for a closed set of decisions (variants are claims), a trait for a capability (implementations are effects). At Phase 3, every skeleton signature is domain-typed except the boundary functions, which the shape table names as such. | Phase 3, reviewing signatures; Phase 6, when a library type or a string test is about to appear inside a leaf | `KeyEvent → ViewerKey` (right); `href.starts_with(...)` classified once into `LinkDestination` (right shape); seven walks of the `Event` stream re-deriving structure (wrong); `3 + index` unnamed in `render_table` (its `+`→`-` mutant survives). |
| **Dispatch/work self-check at Phase 6:** an arm with two statements, or an `if` followed by more statements, is a leaf with an undeclared decision. The complexity threshold shapes line counts, not decisions. | Phase 6, before declaring a leaf done | `tick`, `apply_key`, `check_for_reload` — whose untraced `if` *is* the trigger of `ContentReloadsWhenTheFileChangesOnDisk`, cited on a function that decides nothing. |
| **A stop ends with at most three numbered decisions**, and "continue" approves only those. What the design traded away is one of them. | Every phase stop | Nine bare "continue"s; the whole-line link highlight (a deliberate shortcut recorded as a virtue in the LLD) was caught by the user reading a summary. |
| **A failed gate never offers "commit anyway".** The options are fix, prove the tool wrong with a reproducer, or stop. A gate that prints nothing is reported as a result. | Phase 7 | "commit anyway with this documented" offered twice; an empty mutation run passed in silence. |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Traceability of this slice | Untraced; process-evidence validation | Ceremonial `Spec` structs for skill behaviour; a skill-lint script | Ceremonial specs make the graph look denser than it is (README [§6](https://bradvoth.github.io/lid-rs/spec/traced.html)). A script checking prose cannot check what matters (did the agent *follow* it). |
| Skill scope | Operates LID-rs on any Rust codebase that adopts it, this repo included | A repo-specific runbook | The skill is a deliverable of the system; this repo is its first consumer, not its subject. |
| Relationship to the installed `linked-intent-dev` skill | This skill owns Rust-artifact mechanics; the generic skill's process discipline (stops, cascade, coherence pre-flight) is incorporated by restatement, not by reference | Deferring to the generic skill plus a delta document | A skill that requires composing two documents at run time is a skill agents will half-follow; restating the process rules keeps one authoritative walk. |
| Failure remediation format | Per-check table with meaning and correct response | Prose guidance | The moment of use is "a gate just fired"; a lookup table matches it. |
| Form of the earned rules | `references/discipline.md`, one when → do-this table, read at a phase stop or a gate firing | Inline in `SKILL.md` (the 0.2 arrangement); worked examples inline at each phase | `sync` now carries a directory (below), so "one more thing to carry" no longer argues for staying inline; an always-loaded `SKILL.md` that holds every phase's rules is exactly the context an agent that can't track the whole workflow drops first. One table stays the single source a row's phase tag can point at, instead of duplicating rows across phase files where they'd drift. |
| Where the canonical skill lives | `lid-rs/skill/` — `SKILL.md` a thin dispatcher, `references/*.md` the phase, gate, and mechanics detail — inside the crate that its toolchain-half describes; working copies are synced, never edited | A single `lid-rs/skill/SKILL.md` (the 0.1–0.2 arrangement); `.claude/skills/lid-rs/SKILL.md` as canonical with a template copy in the tool; a plugin | A single file that must state every phase in full is read whole the moment the skill triggers, which is the failure this change exists to fix (weaker agents losing the workflow in a file that large). Splitting by phase means only the current phase's detail loads. The crate-shipped, synced-not-edited arrangement is otherwise unchanged from 0.2's rationale. |
| State between turns | A branch per LLD (`lld/<slice-name>`), one commit per phase at approval | A state file (`.lid-rs-state.json` or similar) naming the current phase and slice | Git already versions the exact artifact each phase produces; a state file is a second source of truth that can drift from what's actually on disk, and is one more file `sync`/the gate would need to know about. `git log` on the branch answers "what phase are we on" from the same evidence a reviewer uses, and turns an LLD's PR into something read commit-by-commit in phase order rather than as one diff. The convention prescribes only the working branch's shape; squash, merge, or rebase into the default branch is the human's call at PR time. |
| Evidence placement | The examples and the project they came from live in this LLD only; the skill states the check | Worked examples in the skill; anonymised examples in the skill | A rule read at the moment it applies needs the check, and a story about another project is noise at that moment; the evidence exists to justify the rule to a reviewer of this design, which is what an LLD is for. Named here, because a named example is checkable. |
| Stop-message contract | ≤3 numbered decisions; "continue" approves those only; waivers per slice | Free-form summary ending "approve or flag" | Nine bare "continue"s in `dmdr` show the free-form stop becomes ceremony; a numbered decision is something a reviewer answers. |
| `#[mutants::skip]` | Permitted only on pure I/O sequencing, reason in the LLD, after asking | Forbidden outright; unregulated | Some functions genuinely only sequence terminal I/O and have no unit-observable behaviour; forbidding the attribute pushes the same escape into `#[cfg(not(test))]` tricks. Unregulated is what happened. |

## Open Questions & Future Decisions

### Deferred
1. Plugin packaging (marketplace metadata, versioning) — the precondition
   ("operated on at least one non-self-hosted codebase") is now met; the
   open question is the seam between the process half of the skill, which
   could update through a plugin, and the toolchain half, which is pinned
   to the tool's version (`cargo-lid-rs/docs/intent/init/lld.md`, Deferred 4).
2. A `/lid-differential` companion pass for the semantic residual (README
   [§4.4](https://bradvoth.github.io/lid-rs/spec/gates.html)) — scheduled reconstruction of claims from code and diffing against
   the written ones. `dmdr`'s tests-cite-the-wrong-claim cases are its first
   test set.
3. A mechanical form for two of the rules: an untraced-fn count per module
   and a shape-table identifier check could be `cargo lid-rs` subcommands
   rather than agent duties. Deferred until the prose rule has been seen to
   fail.

## References

- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the flow), [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate), [§6](https://bradvoth.github.io/lid-rs/spec/traced.html) (traced and untraced), [§11](https://bradvoth.github.io/lid-rs/spec/layout.html) (bootstrap and brownfield).
- This repository's git history — the process evidence the skill distills.
- `dmdr` (commits `be8074d`, `ba1dc6e`, and its agent session transcript of
  2026-08-26) — the first external deployment; source of the rules in
  content requirement 5.
