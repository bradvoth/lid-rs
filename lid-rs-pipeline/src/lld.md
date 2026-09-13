# The pipeline's state: a slice's phase is what its branch says

The slice is **`lid-rs-pipeline`**. It is a crate-root slice — this document is
`lid-rs-pipeline/src/lld.md`, the crate *is* the slice, and by the layout
slice's rule a crate-root slice's name is its package's — so its branch is
`lld/lid-rs-pipeline` and its phase agents are invoked with that name. Every
other spelling (`pipeline`, "slice 23") names nothing the tooling resolves.

## Context

`lid-rs-pipeline/docs/intent/pipeline.md` is this crate's living specification,
as `README.md` is the methodology's. It describes a harness that builds a slice
unattended from a draft PR: sessions, `fast`/`gate` dispatch, state and
resumption, review, adjudication, publication. This document does not restate
it. It says which part of it this slice builds, and why the rest cannot be
built yet.

**What exists, verified against the tree rather than against the
specification:**

| Fact | Where |
|---|---|
| The crate is a workspace member with **no dependencies** and `publish = false` | `lid-rs-pipeline/Cargo.toml` |
| Its `src/lib.rs` is one line: `#![doc = include_str!("../docs/intent/pipeline.md")]` | `lid-rs-pipeline/src/lib.rs` |
| `cargo-lid-rs` has a **library** target beside its binary, and depends on `lid-rs` | `cargo-lid-rs/src/lib.rs`, `src/main.rs`, `Cargo.toml` |
| The session loop already exists — `Precondition`, `slice_of`, `committed_phases`, `Budget`, `Outcome`, `build`, and the `door`/`turn`/`review`/`tools`/`ending`/`replay` modules | `cargo-lid-rs/src/headless_canopy_agent/` |
| Five commit trailers are written today, on 17 commits: `Lid-Rs-Phase`, `Lid-Rs-Agent`, `Lid-Rs-Tools`, `Lid-Rs-Checks`, `Lid-Rs-Refusals` | `cargo-lid-rs/src/phase/`, `git log` |
| The catalog **already assigns three commands to this slice**: `status`, `commit` and `pr-body` carry `Unbuilt::Owner("the pipeline")` and are refused with "belongs to the pipeline, which this tool is not" | `cargo-lid-rs/src/catalog/mod.rs:222-224`, `:570` |
| `CODEOWNERS` does not exist; `docs/intent/trace.md` does not exist; there is no `[workspace.metadata.lid_rs.pipeline]` table | repository root, `Cargo.toml:36` |

**The dependency direction, as a manifest fact.** `#[implements]` expands to
`::lid_rs::IMPLEMENTATIONS`, so a crate carrying citations needs `lid-rs` in its
extern prelude — a *transitive* dependency does not satisfy that, so this crate
needs a **direct** dependency on `lid-rs` even though it will also depend on
`cargo-lid-rs`, which depends on `lid-rs` itself. `cargo-lid-rs` does not depend
on `lid-rs-pipeline` and must not, so `lid-rs-pipeline → cargo-lid-rs → lid-rs`
is acyclic. This is the direction pipeline §5.4 states — the pipeline is "a
consumer of `cargo-lid-rs`'s library, as the canopy client is today" — and the
canopy client being *inside* `cargo-lid-rs` today is what makes that sentence
describe a move rather than a dependency.

## Behaviour

This slice builds **`status`**: the pipeline's reading of where a slice is, from
its branch alone.

State is the newest `phase N:` subject **made on this branch** — a commit
reachable from HEAD but also from the branch's base belongs to another slice,
and pipeline §4.2 records what reading the whole ancestry once cost: every phase
of a branch cut after a merge was skipped and the branch reported PR-ready with
no session opened. `status` answers, for the branch it is run on:

- the slice, from the branch name;
- the phase last committed on this branch;
- the phase that would run next;
- whether the slice's `lld.md` or the HLD changed after that commit, which under
  §4.2 restarts the pipeline at Phase 2 rather than resuming;
- the trailers the phase commits carry, so a reader sees which agent and which
  tally produced each phase.

It writes `target/lid/status.json` and a human rendering to stdout. Exit 0 when
it answered with no findings, 1 when it answered with findings (a malformed
subject, a tool-version mismatch), 2 when the branch names no slice — §5's codes.
It is a *reading*: it never advances state, never commits, and never opens a
session.

**Its report is the state, not a finding list**, which diverges from §5's
blanket "every command writes a report in the finding schema (§5.3)". A schema
whose fields are `check`, `rule`, `file`, `line`, `claim` has nothing to say
about "Phase 4 was last committed here"; the findings this command can raise are
a second, possibly empty, list beside the state. See Open Questions.

### What this slice is not

**It is not `commit`.** Pipeline §5.1 has `commit --phase N` read
`target/lid/gate-N.json` and refuse if it is absent or failed. That file is a
composite's report, and the composites are `[workspace.metadata.lid_rs.pipeline]`
— which the catalog slice has already assigned elsewhere: moving `phase::plan`'s
step lists into that table and renaming `phase-check <n>` to `gate --phase <n>`
"belong to the **`phase`** slice", on `lld/phase--gate-from-metadata`, because a
manifest is a file no phase of any slice may write. Until that lands there is no
`gate-N.json` for `commit` to refuse on, and a `commit` built against
`phase-check`'s text output would be built against the thing that is being
replaced.

**It is not `fast`/`gate` dispatch.** The HLD's row for this slice names it, and
the naming is ambiguous in a way worth settling: *defining* the composites is
the `phase` slice's, as above; *calling* `gate --phase N` once per phase is this
slice's, and it cannot be written before the thing it calls exists.

**It is not the session loop.** `headless_canopy_agent` already hosts sessions
against a live door, with preconditions, budgets, a tool surface, a reviewer
turn and an ending. Pipeline §5.4 puts that machinery in this crate; its own LLD
defers "the documentation cascade if the POC is promoted". Promotion is a move
of a shipped slice between crates, with its claims, its citations and its
fixtures — a change on that slice, not a phase of this one.

**It is not `pr-body`, publication, `CODEOWNERS`, the rulesets, the squash
message, or adjudication.** `pr-body` reads `trace.md` and the site, both slice
21's and neither built. The rest are the PR lifecycle, which needs a PR the
pipeline opened.

### Failure paths

| When | What `status` does |
|---|---|
| The branch is not `lld/<slice>` and no slice is given | Exits 2 naming the convention — the same refusal `slice_of` already produces |
| The branch has no `phase N:` commit of its own | Answers "no phase committed on this branch", with the branch point named, rather than reporting the base's phases |
| A `phase N:` subject is present but malformed (no parsable `N`) | A finding against that commit, not a panic and not a silent skip — a subject the tool cannot read is the one case where state is genuinely unknown |
| A subject names a phase that has no check of its own — `phase 6: leaves`, which this workspace does produce (`headless_canopy_agent/mod.rs:941`) | Read as no phase, and a finding. `Phase` is the closed set of phases *with a check* (0, 6 and 8 have none), so `Status::phase` cannot represent phase 6 and `tag_of` answers `Tag::Unchecked(6)`. Defensible — the state this command reports is the state a run resumes from, and a run resumes from checked phases — but it means a branch whose newest commit is `phase 6:` reports the phase before it |
| `lld.md` or `hld.md` changed after the newest phase commit | Answers "restarts at Phase 2", per §4.2, and says which document changed |
| A phase commit carries a `Lid-Rs-Tool` version this binary does not match | A finding, not a silent resume — the trailer exists so that a resume across a tool change is visible. Today no commit carries it, so this path is unreachable until `commit` writes it |
| The working tree is dirty | Reported, not refused: `status` reads, and refusing a reading because of uncommitted work would make it useless in the case an operator most wants it |
| The branch's `lld/<slice>` names a slice the layout cannot resolve | Reported, not refused, for the reason the row above gives: `status` reads, and an operator on a malformed branch is exactly who most wants it. The unresolvable slice is a finding and the state is answered as far as it can be read — the restart-at-2 verdict, which needs the slice's `lld.md`, is the part that cannot be. This is *not* the canopy precondition's answer (`an_acceptance_the_layout_cannot_place_is_not_an_acceptance`, `headless_canopy_agent/mod.rs:990`), and the difference is what the two commands do: that one fails closed because it gates running privileged code without the human's acceptance, and a reading gates nothing |

## Shape

| Item | What it is |
|---|---|
| `lid_rs_pipeline::status(project, branch, slice: Option<&str>) -> Result<Report, String>` | The whole of this slice: the branch's state, and the findings the reading raised, as one value. It answers the **pair**, not the state alone — three of the finding claims are written of what `status` answers with, and a `status` returning `Status` would leave them with no item whose wrong answer could falsify them (README §4.3). The optional slice is the failure table's first row and `ABranchThatNamesNoSliceIsRefusedNamingTheConvention`'s "and no slice beside it": a branch that is not `lld/<slice>` is refused only when nothing else names one |
| `lid_rs_pipeline::Status` | Slice, the phase last committed here, the phase next, the restart-at-2 verdict, and the trailers of **every** phase commit the branch made. No layer field: a Phase 4 subject is exactly `phase 4: descend for <slice>` and carries none, and the layer's specified home is the `Lid-Rs-Layer` trailer (`pipeline.md` §4.1, §5.1), which nothing writes today — so the trailers carry it if `commit --phase N --layer L` ever does, and `Status` needs no second place for it |
| `lid_rs_pipeline::Trailer { name, value }` | One `Name: value` trailer of a commit message, as the message spells it. A name and a value rather than a closed set of names: the trailers a phase commit carries grow with what writes them, and a reading that recognised only the names it knew would drop the next one silently. Six of the names §5.1 specifies are written by nothing today, so the open form is what makes this slice readable when they are |
| `lid_rs_pipeline::PhaseCommit { commit, subject, phase: Option<Phase>, trailers }` | One `phase N:` commit the branch made. `EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers` is a claim about a *record per commit*, not about the newest commit's trailers, so it needs an item of its own; `phase` is optional because a subject the tool cannot read the number of is a finding, not a panic |
| `lid_rs_pipeline::write_report(root, report) -> Result<PathBuf, String>` | Puts the report under `target/lid/status.json`, creating the directory, and answers with the path it wrote. Separate from `status` because a reading that writes is a reading that cannot be called twice safely, and because the exit codes in §5 are about the report, not about the write. The two Decisions rows below say how it serializes a `Phase` and who joins its path; both were Unresolved while this item was unbuildable, and both resolved at `2604ddb` |
| `lid_rs_pipeline::Restart` | Why a resume is not a resume: `Resume`, or `AtPhaseTwo { document }` when `lld.md` or `hld.md` moved after the newest phase commit |
| `lid_rs_pipeline::Report { state: Status, findings: Vec<Finding> }` | What `status.json` holds: the state, and the findings the reading itself raised — a malformed subject, a tool-version mismatch — ordinarily empty. The pair is what makes §5's exit codes (0 / 1 / findings) mean anything for a command whose answer is not a defect |
| `lid_rs_pipeline::rendering(report) -> String` | The human rendering, from the same value the JSON is `serde` over |
| [`cargo_lid_rs::layout::own_crate`](cargo_lid_rs::layout::own_crate) (and `lld_path` beside it) | Whether the branch's slice exists at all, which `slice_of` cannot say — it answers from the branch name alone. `own_crate` is a door that **refuses**, with `no_crate_refusal`'s sentence, for a slice no workspace member holds a document for; that refusal is this slice's finding, not this slice's exit. `lld_path` is only the path for the restart-at-2 comparison and must not be used to decide existence: it answers `Ok` in both arms, falling back to a workspace-root path it never probes (`layout/mod.rs:317-322`) |
| [`cargo_lid_rs::headless_canopy_agent::{slice_of, fork_point, own_commits}`](cargo_lid_rs::headless_canopy_agent) | Reused, not rewritten — this slice's dependency on the `cargo-lid-rs` library is what those are for. `fork_point` and `own_commits` give the range; `slice_of` resolves the branch/slice pair inside `slice_named` |
| `committed_phases` and `log_subjects` are **not** reused | Both were named in an earlier draft of the row above, and Phase 4 found neither usable. `log_subjects` reads `--format=%s`, and `EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers` needs each commit's trailers from the *same* read — using it would mean a second `git log` per commit, so this slice issues one wide-format read (`branch_log`). `committed_phases` flattens subjects to `Vec<Phase>` and drops the per-commit association that `Status::commits` *is*, so its `Tag::Checked` projection is repeated here as `checked_phase`. That repetition is the one this slice accepts: it is three lines over a public enum, not a table or a path, and the alternative is a Phase 8 widening `committed_phases`'s return type on a shipped slice |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| The slice is `status` alone | The branch's state as a value, and its rendering | `status` + `commit`; the whole harness; promote the canopy slice first | `commit` cannot be built before `gate-N.json` exists, and that file is the `phase` slice's to define (`lld/phase--gate-from-metadata`). The whole harness needs eight commands that are four other slices'. A slice that builds what it can gate is worth more than one that gates nothing |
| State is read from commit subjects and trailers, not from a state file | `git log` on the branch | A `target/lid/state.json`; a PR comment; a git note | Pipeline §P3: "commit subjects and trailers persist state; the branch is resumable from any phase". A state file is a second source that can disagree with the branch, and the branch is the one a human pushes to |
| Reuse `cargo-lid-rs`'s session-state functions rather than reimplement them | A direct dependency on the `cargo-lid-rs` library | Reimplement `committed_phases` here; move those functions out of the canopy slice now | Reimplementing puts the same reading in two crates, which is the defect this workspace has found three instances of in path handling alone. Moving them is a change on the canopy slice, not a phase of this one |
| `publish = false` stays | The crate stays out of the registry | Publish it with the others | It is an operator's harness, not a library a consumer links. Keeping it unpublished keeps it out of the gate's `cargo package` step and out of the publish slice's set, and nothing in this slice needs it published. Reversing it is one manifest line — and a Phase 8, since `cargo package` then gates it |
| `lib.rs` includes **both** documents | `#![doc = include_str!("../docs/intent/pipeline.md")]` and `#![doc = include_str!("lld.md")]` | Include only `lld.md`; have `lld.md` quote the spec; move `pipeline.md` | The crate-root form requires `lib.rs` to include `lld.md` so that check 2 resolves this document's links, and the spec's inclusion is the same shape `lid-rs/src/lib.rs` uses for the HLD. Multiple `#![doc]` attributes concatenate in order. **Phase 3 must confirm that concatenation renders and that check 2 passes over both** — this document could not run cargo to verify it |
| How `Phase` reaches the JSON | **Invert the public `TryFrom<u8>` by search** — scan `1..=u8::MAX` for the number whose `Phase::try_from` answers this phase — in one leaf of this crate, and serialize that number | (a) a 6-arm match here; (b) `impl From<Phase> for u8` on the `phase` slice; (c) `impl Serialize for Phase` there | `Status::phase` is `Option<cargo_lid_rs::phase::Phase>` and `Phase` (`phase/mod.rs:34`) derives no `Serialize`, so `#[derive(Serialize)]` on `Status` does not compile. The table is **already written three times** — `TryFrom<u8>` (`phase/mod.rs:51`) plus two byte-identical private copies, `ending::number` (`headless_canopy_agent/ending.rs:65`) and `policy::number_of` (`phase/policy.rs:741`) — so (a) would be the fourth spelling and is rejected on that ground alone. The chosen inversion states **no** table: it is derived from the one public, claimed conversion and cannot drift from it, since a changed `TryFrom` changes the search's answer. Spiked before specifying: it agrees with the table on all six variants. (b) is the tidier end state and would let both private copies collapse into it — a net deletion — but it is a Phase 8 on a shipped slice, and constrained-first forbids escalating there before the local option is shown to fail. Reversing to (b) deletes one leaf and its claim |
| Who writes `target/lid/<name>.json` | **This crate's `write_report` joins the path itself** | Widen `catalog::write_report` to `pub fn write_report<T: Serialize>(root, name: &'static str, report: &T)` and call it | `catalog::write_report` (`catalog/mod.rs:549`) is private and typed to the catalog's own `Report`; this slice's writer has a different signature and answers with the path it wrote, so the shared content is a two-segment join, not a shared reading. The workspace already treats that join as an expected value rather than a reading — `catalog/mod.rs:1167`, a `#[cfg(test)]` helper, spells it literally while the implementation computes it. Widening is a Phase 8 on the gated catalog slice (`8b2c63f`) whose three claims there are written of "one entry's report"; constrained-first forbids that escalation until the local option is shown to fail. Reversing is one call site. Note either way: `TheReportIsWrittenToStatusJsonWhateverItFound` restates `catalog::spec`'s `ARunWritesItsReportEvenWhenItFindsNothing` |
| A malformed `phase ` subject is read in this crate | `subject.starts_with("phase ")` beside `tag_of`, in a leaf of this slice | A `Tag::Malformed` variant on the `phase` slice | `tag_of` (`phase/mod.rs:708`) maps through the private `phase_number`, so `"phase foo:"` and `"chore: x"` both return `Tag::Untagged` — indistinguishable, while `AMalformedPhaseSubjectIsAFindingAgainstItsCommit` must tell them apart. The variant is the tidier fix and removes the duplicated `"phase "` literal, but it is a third cross-slice branch; a 7-character literal in two crates is not the same weight as a 6-entry table or a path, so this one is taken locally. Reversing it is deleting one leaf |
| A direct dependency on `lid-rs` as well as on `cargo-lid-rs` | Both in `[dependencies]` | Rely on the transitive path through `cargo-lid-rs` | `#[implements]` emits `::lid_rs::…`, which needs `lid_rs` in this crate's extern prelude; a transitive dependency does not put it there. Without the direct edge, Phase 3's first citation fails to compile |

## Deferred

1. **`commit --phase N`**, with the trailers §5.1 specifies. Six of them —
   `Lid-Rs-Slice`, `Lid-Rs-Layer`, `Lid-Rs-Reviewer`, `Lid-Rs-Model`,
   `Lid-Rs-Tool`, `Lid-Rs-Gate` — are written by nothing today. It waits on
   `lld/phase--gate-from-metadata` for `gate-N.json`.
2. **`fast`/`gate` dispatch**, for the same reason.
3. **Promoting the canopy slice** from `cargo-lid-rs` into this crate — pipeline
   §5.4's layout, and that slice's own Deferred 4. A cross-crate move of a
   shipped slice, on `lld/headless-canopy-agent--promote`.
4. **Adjudication and the registered question** (§8). Needs two assistant agents
   that do not exist as files: `lid-rs/agent/` holds the phase agents, the
   reviewer and the LLD reviewer, and neither `apo.md` nor `aa.md`.
5. **Publication, `pr-body`, the squash message** (§9). Needs `trace.md` and the
   site, both slice 21's.
6. **`CODEOWNERS` and the two rulesets** (§9.2, §9.3). `CODEOWNERS` does not
   exist; both are repository configuration no phase of any slice may write, so
   they land by hand as the metadata table does.
7. **The phase reviewers' rubrics** (§4.1's reviewer column, §7). They are
   `lid-rs/skill/references/phase-N.md` — the `skill` slice's files, synced, and
   outside every phase's policy here.

8. **An entry point.** This crate is `publish = false` with no `[[bin]]` and no
   arm in `cargo-lid-rs`'s dispatcher, so `status` is a library function and the
   exit codes above (0 / 1 / findings) are realized by nothing this slice
   builds. That is deliberate for now — the exit codes are stated so that
   whoever wires the command has them — but it means no end-to-end test can
   invoke `cargo lid-rs status`, and the slice's validations are library-level
   until it is wired. Wiring it is a manifest edit plus a dispatcher arm on the
   `catalog` slice.

## Open Questions

**Is `status`'s report a finding list or a state record?** §5's preamble says
every command writes the finding schema; §5.1's row for `status` says only
"stdout, `status.json`". The schema's fields describe a defect in a file, and
this command's answer is not one. Either the schema grows a variant for a
reading, or `status` is the documented exception, or its report is a state
object beside an ordinarily-empty finding list. The third is what this document
assumes and the least committing; the choice belongs to whoever builds the
second reading command, and it is a `pipeline.md` amendment either way.

**Settled at Phase 2 (`e75d009`)** in favour of the third:
`TheReportCarriesTheStateAndTheFindingsTheReadingRaised` is committed, so
`status` answers the pair. The `pipeline.md` amendment is still owed.

**Does `status` read the trailers of every phase commit, or only the newest?**
The rendering is more useful with all of them — it is the run's history — but
the state verdict needs only the newest, and reading all of them makes the
command's cost grow with the branch. Settled at Phase 2 (`e75d009`) in favour of **every**
commit: `EveryPhaseCommitOnTheBranchIsReportedWithItsTrailers` is committed, so
`Status` holds a `Vec<PhaseCommit>` and not a single record.

**Is "the newest `phase N:` subject made on this branch" one claim or three?**
It has three independently falsifiable parts: that the newest is taken, that a
commit also reachable from the base is excluded, and that the subject form is
`phase N:`. Check 14 binds one validator per claim and check 7's threshold is 4,
so as one claim it must be a table-driven validator. As three claims each gets
its own, and the middle one is the one §4.2 says was actually got wrong. This is
Phase 2's cut to make, and it is the only claim here with the problem.

**Answered, by a rule this document already stated.** *What does `status` do on
a branch whose `lld/<slice>` names a slice the layout cannot resolve?*
`slice_of` answers from the branch name alone and does not ask the layout
whether that slice exists, so resolving it needs the layout — and the
restart-at-2 verdict needs it too. It reports; it does not refuse. That is not
a new judgement: the failure table's dirty-tree row already gives the reason —
"`status` reads, and refusing a reading … would make it useless in the case an
operator most wants it" — and an operator on a malformed branch is exactly that
case. The canopy precondition's opposite answer for the same unresolvable slice
is not a counter-example: it fails closed because it gates running privileged
code without the human's acceptance, and a reading gates nothing. The failure
table now carries the row, and the Shape table names the resolver.

**Should the pipeline's own slices be built with the pipeline?** Every other
slice in this workspace was built by phase agents under the hooks. This one
builds the thing that would replace them, and until it can run a slice
end-to-end it cannot build itself. The bootstrap order in §14 puts this crate
fifth of seven; nothing says whether the last two steps are run by hand.

## References

- `lid-rs-pipeline/docs/intent/pipeline.md` — the specification, especially §4.2
  (state and resumption), §5.1 (`status`, `commit`), §5.4 (the crate layout),
  §14 (bootstrap order).
- `cargo-lid-rs/src/catalog/lld.md` — "What this slice is not", which assigns
  the gate's definition to the `phase` slice and names this slice the owner of
  `status`, `commit` and `pr-body`.
- `cargo-lid-rs/src/headless_canopy_agent/lld.md` — the session loop this slice
  reuses and does not rewrite.
- `lid-rs/docs/intent/hld.md` row 23, and its dependency note: "the pipeline
  (23) runs the catalog, and only its `fast`/`gate` dispatch waits on the
  composites slice 22 deferred."
