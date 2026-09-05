# The LID-rs pipeline

**An agentic workflow for the LID-rs process.**
Agents write text. The harness makes truth. Humans decide what to build, and
sign what was built.

This is the living specification of `lid-rs-pipeline`: how the methodology in
`README.md` is *operated* — by which agents, with which tools, gated by which
commands, escalating to which people. `README.md` says what the compiler
checks; this document says how a slice is built without a human at the
keyboard, and what the human sees when it is done. Both are revised under
the workspace HLD's first tenet: when running the pipeline reveals a flaw in
this document, this document changes, and git history is the record.

---

## 0. The vision, in one paragraph

A product owner and an engineer refine an HLD and then an LLD with two
assistant agents — a **product owner assistant** for meaning and an **architect
assistant** for shape. An **LLD reviewer** validates the result against a
guideline and a trial derivation. A valid LLD is committed to a protected
`lld/<slice>` branch with a draft PR. That event runs the LID-rs phases
unattended: one phase agent, one phase reviewer, and one deterministic gate per
phase, with a commit after each gate passes. Agents have only file tools;
every check is a `cargo lid-rs` command the harness runs. When something can't
be resolved within a phase's budget, the two assistants adjudicate; if they
can't, the flow fails and the question is registered as a PR comment. A
passed final phase publishes the PR, whose commit list *is* the development
process. The product owner and the engineer sign it. It squash-merges to
`main`, so `main` reads as one commit per slice and the `lld/*` branch keeps
the refinement narrative forever.

---

## 1. Principles

**P1 — Agents write text; the harness makes truth.** An agent's only effect is
a file change. Whether it compiles, conforms, or kills a mutant is decided by a
`cargo lid-rs` command and reported back. No agent decides whether a check
runs.

**P2 — The command catalog is the implementation.** §5 lists every command the
harness may run. Each has fixed inputs, fixed outputs under `target/lid/`, and
fixed exit codes. Building the pipeline means building those commands;
nothing else is load-bearing.

**P3 — The phases are the state machine.** Commit subjects and trailers persist
state; the branch is resumable from any phase; `git log` is the narrative.

**P4 — Hands-off until it can't be.** No hold points by default. Escalation
goes to the assistants first, and to humans only as a registered question that
fails the flow. Humans are asked exactly twice: at the design loop, and at
sign-off.

**P5 — Round trips are prevented at the LLD.** The heaviest review is the
earliest, and it includes proving the LLD derives.

**P6 — Everything the pipeline does is reviewable.** Prompts, guidelines, the
command catalog, and the branch rules are files, cited by SHA in every commit
the pipeline makes, and the tool version that ran them is cited too.

**P7 — `main` is what; `lld/*` is how.** One squash commit per slice on `main`.
The full phase-by-phase history stays on the protected branch, linked from the
squash commit.

**P8 — A phase's cost is paid once.** Each phase runs in its own session that
starts fresh, works, and ends; deterministic code runs between sessions. An
orchestrating conversation that re-sends its growing context every turn was
measured at 65% of a slice's tokens against 35% for every worker together,
and the split is structural: workers are O(1) each, a conductor is not.

---

## 2. Actors

| Actor | Kind | Owns |
|---|---|---|
| **Product owner** | human | what the slice is for; LLD rationale; **sign-off** |
| **Engineer** | human | vocabulary and signature shape; **sign-off** |
| **APO** — product owner assistant | agent | drafting HLD/LLD with the humans (today: `cargo lid-rs coach`); claim *meaning*; adjudicating design questions |
| **AA** — architect assistant | agent | vocabulary shape, lexicon additions, slice seams, skeleton structure; adjudicating structural questions; the only agent that may edit the protected knobs |
| **LLD reviewer** | agent | the LLD guideline (today: `lid-rs-lld-review`, advisory); the trial derivation |
| **Phase agents 2–7** | agents | one phase's artifacts each (today: `lid-rs-phase-N`) |
| **Phase reviewer** | agent | one phase's rubric, from files alone (today: `lid-rs-review`) |
| **Harness** | service | the tool server, the command catalog, commits, PR lifecycle, state, escalation (today: the `lid-rs` workflow in Claude Code, and `cargo lid-rs canopy`, the headless proof) |

The agents, their guidelines, and the workflow are shipped by the `lid-rs`
crate (`lid-rs/agent/`, `lid-rs/skill/references/`, `lid-rs/workflow/`) and
written into a project by `cargo lid-rs sync`, which also gates that the
copies match. Where this document says "rubric", the file is a skill
reference; where it says "prompt", the file is an agent definition.

---

## 3. Stage A — the design loop

Interactive, on `lld/<slice>`, with both assistants in the session.

**3.1 HLD.** The APO places the capability: which slice, what crate-wide
nouns, what changes to the HLD's scope. The AA checks the nouns against
existing vocabulary and existing slices for seams.

**3.2 LLD.** The APO drafts the slice's `lld.md` with the humans, applying the
guideline while drafting — the coach's interview is this step, and it opens
with the index of the repository's intent documents so the LLD is placed
before it is written. The AA maintains the **vocabulary candidate list** —
each noun, a definition, a logging policy, and whether it already exists in
another slice. Either assistant may call `trial_derive`.

**3.3 Guideline** (`skill/references/lld.md`, L1–L11): one operation per
slice; every noun is vocabulary; every behaviour derives; failure modes
enumerated with noun and error; invariants stated as invariants; no prohibited
terms; no implementation detail; rationale and rejected alternatives; explicit
out-of-scope; shared nouns link to their owning slice; HLD consistency. The
LLD reviewer applies it and reports what a phase would predictably stop on.
It is advisory: approving an LLD is the human's act.

**3.4 Trial derivation.** The LLD reviewer drafts `spec.rs` and `vocab.rs` in a
scratch worktree from the LLD alone; the harness runs `cargo lid-rs check
--worktree` and `cargo lid-rs doc --worktree`. Failures become findings against
LLD sentences. The trial is retained as a hidden reference for the Phase 2
differential (§7) and never committed.

**3.5 Accept.** On pass: commit the LLD and the HLD diff to `lld/<slice>` as
`phase 1: <slice> — <what the LLD settles>` with trailers (`Lid-Rs-Slice`,
`Lid-Rs-Phase: 1`, `Lid-Rs-Reviewer: lld@<sha>`, `Lid-Rs-Model`), open a draft
PR with the LLD as its body. The `opened` event starts Stage B.

Evidence for P5: the slice that built the headless harness amended its LLD
twelve times while it was built; the LLD reviewer, applied to the next
slice's LLD before its phases ran, rejected a drafted design twice on
contradictions with the gates, and the design that shipped was a fraction of
the draft.

---

## 4. Stage B — the build pipeline

Phases 2–7, unattended. Each phase:

```text
context pack → phase agent edits (cargo lid-rs fast after every edit)
            → cargo lid-rs gate --phase N
            → phase reviewer
            → pass: cargo lid-rs commit --phase N; advance
              fail: retry within the budget; then adjudicate (§8); then fail + comment
```

### 4.1 Phase table

| Phase | Agent produces | `fast` runs | `gate` runs | Reviewer rubric | Commit |
|---|---|---|---|---|---|
| **2 Claims** | `spec.rs`, `vocab.rs`, crate `vocab.rs` additions, lexicon additions | `check`, `doc` | `check`, `doc`, `graph --expect uncited` | R2: every claim traces to an LLD sentence and vice versa; differential vs. trial; lexicon additions have definitions and templates | `phase 2:` |
| **3 Skeleton** | `mod.rs`: pinned entry, dispatch, `#[implements]`, `todo!()`; `#[derive(Outcome)]` error enum | `shape`, `check`, `conform` | `shape`, `check`, `lint`, `doc`, `conform`, `graph --expect unvalidated` | R3: pattern → shape; every claim implemented; no signature the LLD didn't imply; seams | `phase 3:` |
| **4 Descend** | one layer of leaf skeletons, breadth-first | as 3 | as 3 | R4: layer complete; stop condition honest; siblings consistent | `phase 4:`, `Lid-Rs-Layer: n` |
| **5 Validators** | one `#[validates]` per claim | `check` | `check`, `conform`, `validate --red` | R5: assert mirrors response clause; ubiquitous validators generate; state-driven show a transition | `phase 5:` |
| **6 Implement** | leaf bodies, one leaf per agent | `shape`, `check`, `lint`, `validate --claims <leaf's claims>` | `suite` | R6: no behaviour beyond the claim; outcomes read as promised | `phase 6:` |
| **7 Gate** | — | — | `suite`, `mutants`, `regen`, `graph`, `site`, `pr-body` | — | `phase 7:`; PR → ready |

**Phase 2's gate has no lint.** `-D warnings` would turn the deprecation
warning that *is* a Phase 8 cascade into a failure in the one phase that
cannot clear it (only Phase 2 writes the claims; the citations move later).
`check` at Phase 2 is the build with `deprecated` left at warn.

**Phase 5's gate is inverted**: `validate --red` passes only if every
validator in the red set fails *and* its trace shows the claim reached with
outcome *panicked*. Green at Phase 5 fails the phase. The red set is the
slice's claims whose `struct` line was added since the newest `phase 7:`
commit the branch reaches, less any claim whose implementers are all data — a
`const`, an enum, a value-returning arm — which is green the moment the
skeleton lands and is held to check 24's many-inputs rule instead. A rename
under Phase 8 leaves an alias and is not an addition. An empty red set after a
gate is a finding naming the base.

**Phase 6's fast gate is by claim name**: after each edit to a leaf, the
validators for that leaf's claims run by name. Feedback is outcome-level and
takes seconds.

**Phase 7 publishes.** The only step that changes PR state.

### 4.2 State and resumption

State is the newest `phase N:` subject *made on this branch* — a commit
reachable from HEAD but also from the branch's base is another slice's, and a
precondition that read the whole ancestry once skipped every phase of a
branch cut after a merge and reported it PR-ready having opened no session.
`synchronize` resumes from the state — unless `lld.md` or `hld.md` changed, in
which case the pipeline restarts at Phase 2 (LID-rs Phase 8 as pipeline
behaviour). A human push at any phase is a resume with their edit in place.

---

## 5. The command catalog — `cargo lid-rs`

This is the implementation surface. The harness runs nothing else. `cargo
lid-rs` is the published cargo subcommand; every command below:

- runs a fixed underlying invocation (no argument passthrough from agents),
- writes a report to `target/lid/<command>.json` in the finding schema (§5.3)
  and a human rendering to stdout,
- exits **0** pass, **1** findings, **2** tooling error,
- has a timeout from `[workspace.metadata.lid_rs.pipeline]`.

The subcommands that exist today — `phase-check N`, `mutants`, `sync`,
`init`, `new`, `lld-check`, `coach`, `canopy` — are the catalog's ancestors:
`phase-check N` is `gate --phase N` with a text rendering and no schema;
`mutants` is `mutants` with the file scoping; `lld-check` is the guideline's
mechanical half. The catalog keeps their names where the command is the same
thing.

### 5.1 Atomic commands

| Command | Underlying | LID-rs checks | Reads | Writes | Budget |
|---|---|---|---|---|---|
| `check` | `cargo check --all-targets --message-format json` | 1, 4, 13, 14, `Traceable` bound | source | `check.json` | 10s incremental |
| `lint` | `cargo clippy --all-targets --message-format json -- -D warnings` | 3, 6, 7, 8, 9 | source | `lint.json` | 30s |
| `doc` | `cargo doc --no-deps --document-private-items` with `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links` | 2 | source, `lld.md`, `hld.md` | `doc.json`, `target/doc/` | 60s |
| `examples` | `cargo test --doc` | 5 | source | `examples.json` | 60s |
| `shape` | `lid_rs_shape::check` **directly** — no cargo; a `syn` pass | 15, 16, 17, 18 | `src/`, `[metadata.lid_rs.shape]` | `shape.json`, `shape-classify.json` (every fn → flow/leaf) | 1s |
| `conform` | `cargo test --lib intent_graph::conform` | 19, 20, 21, 22 | registry (`SPECS`, `OUTCOMES`), `lid_rs_shape::signatures`, lexicon templates | `conform.json` | 20s |
| `graph [--expect uncited\|unvalidated\|none]` | `cargo test --lib intent_graph::graph` | canary, 10, 11, 26 | registry, `trace.md` | `graph.json` | 20s |
| `validate [--all \| --claims A B …] [--red]` | `cargo test --lib <validator names>` under the capturing layer | 23, 24 (+ behaviour) | registry (claim → validator names), `SpecMeta` | `validate.json`, `target/lid/traces/<validator>.json` | 5s per claim |
| `suite` | `cargo test --lib` | 10, 11, 15–26, behaviour | everything | `suite.json`, all traces | 5m |
| `mutants` | `cargo mutants` scoped by metadata (`--in-diff` by default) with registry-derived test filters **and `--file` scoping** per group | 12 | registry, `shape-classify.json` (flow nodes fan out) | `mutants.json` | measured, not budgeted |
| `regen` | `cargo test --lib -- --ignored regen` | — (produces the input for 26) | registry, `shape-classify.json` | `docs/intent/trace.md` | 30s |
| `site` | `lid_rs_site::build` | — | `traces/`, `trace.md`, `lld.md`s, JUnit output, `mutants.json`, `git log` | `target/site/` | 60s |
| `pr-body` | `lid_rs_site::pr_body` | — | `trace.md`, `site` URL, phase commits, adjudication log | stdout | 1s |
| `sync [--check]` | as today | — | the resolved `lid-rs` | `.claude/{skills,agents,workflows}` | 1s |
| `status` | reads subjects and trailers | — | git | stdout, `status.json` | 1s |
| `commit --phase N [--layer L]` | `git commit` with generated message and trailers | — | `gate-N.json`, agent and rubric SHAs | git | 1s |

Notes on specific commands:

- **`check`, `lint`, `doc`** parse `--message-format json` and map rustc/clippy
  diagnostics onto LID-rs check numbers by lint name and by the
  const-assertion and derive spans. Diagnostics that aren't LID-rs checks pass
  through with `check: 0`.
- **`shape`** is the only check that doesn't build anything. It also writes the
  flow/leaf classification that `mutants`, `regen`, and `site` consume, so it
  runs first in every composite that needs it.
- **`conform`** and **`graph`** are tests because they need the linked
  registry. They're filtered by module path so only `intent_graph` runs; the
  cost is the incremental test-binary build.
- **`validate`** resolves claim names to validator names through the registry
  and runs `cargo test --lib` with an exact-name filter. It installs the
  capturing layer, captures traces, and evaluates 23 and 24 per validator.
  `--red` inverts the pass condition over the red set (§4.1): every selected
  validator must fail, and every trace must show *reached* (23 satisfied)
  with outcome *panicked*. Any other kind of failure — a compile error, a
  validator that passes, a trace that never reached the claim — is a finding.
- **`suite`** is `validate --all` plus everything else in the lib, and it's
  the only command that can evaluate 25, because 25 needs the union of all
  traces.
- **`mutants`** derives the per-mutant test filter from the registry: for a
  leaf, its claims' validators; for a flow node (per `shape-classify.json`),
  every validator downstream; for an untraced function, the enclosing module's
  validators, then the suite. It passes `--file` for each distinct file in a
  group, because the engine's name filter alone lets every "delete field"
  mutant of the crate into every run — 62% of the work when measured — and it
  reads each group's verdicts from that run's own `outcomes.json` by name, so
  a stray in someone else's group cannot fail this one. It has no budget
  because its cost is the gate's: seventeen minutes at 51 groups, an hour at
  125; the pipeline runs it between sessions, where no watchdog is waiting.
- **`regen`** is deterministic given the registry and the shape classification.
  It never reads traces, so `trace.md` freshness (26) doesn't depend on test
  execution order.
- **`commit`** reads `target/lid/gate-N.json` and refuses to run if it's absent
  or failed. It stages exactly the phase's policy paths, refuses if anything
  outside them changed, and refuses if a synced artifact differs from what the
  resolved `lid-rs` ships. The message body lists the claims touched; the
  trailers are `Lid-Rs-Slice`, `Lid-Rs-Phase`, `Lid-Rs-Layer`,
  `Lid-Rs-Agent: phase-N@<sha>`, `Lid-Rs-Reviewer: phase-N@<sha>`,
  `Lid-Rs-Model`, `Lid-Rs-Tool: <cargo-lid-rs version>`, `Lid-Rs-Gate: <sha of
  gate-N.json>`, and the tally the hooks keep today — `Lid-Rs-Tools`,
  `Lid-Rs-Checks`, `Lid-Rs-Refusals`.

### 5.2 Composite commands

`fast` and `gate` are sequences of atomic commands, defined per phase in
workspace metadata, stopping at the first failure. They write
`fast-N.json` / `gate-N.json` as the union of their steps' reports.

```toml
[workspace.metadata.lid_rs.pipeline.fast]
2 = ["check", "doc"]
3 = ["shape", "check", "conform"]
4 = ["shape", "check", "conform"]
5 = ["check"]
6 = ["shape", "check", "lint", "validate --claims {leaf_claims}"]

[workspace.metadata.lid_rs.pipeline.gate]
2 = ["check", "doc", "graph --expect uncited"]
3 = ["shape", "check", "lint", "doc", "conform", "graph --expect unvalidated"]
4 = ["shape", "check", "lint", "doc", "conform", "graph --expect unvalidated"]
5 = ["shape", "check", "lint", "conform", "validate --red"]
6 = ["shape", "check", "lint", "doc", "examples", "conform", "suite"]
7 = ["suite", "mutants", "regen", "graph", "sync --check", "site"]

[workspace.metadata.lid_rs.pipeline.worktree]
trial = ["check", "doc"]         # Stage A trial derivation

[workspace.metadata.lid_rs.pipeline.timeouts]
check = "120s"
lint = "180s"
doc = "180s"
examples = "120s"
shape = "10s"
conform = "120s"
graph = "120s"
validate = "300s"
suite = "900s"
regen = "120s"
site = "300s"
```

`{leaf_claims}` is the only substitution, filled by the harness from the
registry — never from agent output. The mutation base comes from
`mutation_scope` and the branch, as today.

`graph --expect uncited` at gate 2 means "10 and 11 are *expected* to fail, 26
must pass": the claims exist and nothing implements them yet. `--expect
unvalidated` at gates 3–4 means 10 must pass and 11 is expected to fail.
Without `--expect`, all must pass. Encoding the registry's expected state per
phase is what lets the gate be strict without being wrong about where the
slice is. Until the catalog exists, `phase-check N` encodes the same
expectations as a fixed sequence, and README §4.5's list is the floor `gate
--phase 7` must contain.

### 5.3 The finding schema

Every report is a list of findings:

```json
{
  "check":    19,
  "rule":     "S1",
  "severity": "error",
  "file":     "src/auth/mod.rs",
  "line":     41,
  "item":     "crate::auth::verify_password",
  "claim":    "crate::auth::spec::UnknownCredentialsAreRejected",
  "message":  "return type `bool` matches no template for verb `reject`",
  "fix":      "return `Result<_, AuthError>`; see lexicon `reject`",
  "source":   "conform"
}
```

`item` and `claim` are present when the check is about one; `rule` is the
LID-rs rule code (A, B, P, V, S1, S2, E1, E2, F1–F6) when there is one. The
human rendering is generated from this and is what agents see — with the
skill's rule for that check appended, which is what the hooks return today.
The harness stores the JSON for state, for the PR body, and for adjudication.

### 5.4 What this implies for the workspace

The catalog fixes the crate layout:

```text
lid-rs/            Spec, SpecMeta, Edge, Variant, four slices, canary, Traceable,
                   Outcome, the capturing layer, spawn, trace rendering, config;
                   ships the skill, the agents, the workflow
lid-rs-macros/     derive(Spec, Traceable, Outcome), implements, validates,
                   implements_module!, spec, flow, leaf
lid-rs-shape/      classify, check (A/B/P/V), signatures — pure syn, no cargo
lid-rs-site/       build, pr_body
cargo-lid-rs/      the catalog: atomic commands, composites, finding schema,
                   diagnostic mapping, registry-driven filters, commit, sync,
                   init, new, coach, lld-check
lid-rs-pipeline/   the harness: sessions, fast/gate dispatch, state, review,
                   adjudication, PR lifecycle — a consumer of cargo-lid-rs's
                   library, as the canopy client is today
xtask/             the gate self-test
```

Two things the catalog decided that the README left open: `shape` runs
standalone rather than as a test (it needs no registry, and one-second
feedback matters in Phases 3–4); and `mutants`' test filtering is in
`cargo-lid-rs`, not in an xtask, because it's the same registry-reading code
`validate` uses — which is where it already is.

---

## 6. The harness

### 6.1 Tool surface for agents

`read`, `grep`, `glob`, `edit`, `write`. Plus `trial_derive` for the two
assistants in Stage A. Paths confined to the workspace. **Read-only to phase
agents:** the synced files, `.github/`, `Cargo.toml`, `clippy.toml`,
`docs/intent/lexicon.toml`, `CODEOWNERS`, and every path outside the phase's
policy — Phase 2 writes the slice's claims, 3 and 4 the slice's module and
`lib.rs`, 5 and 7 the module only; never the LLD, the configuration, or
another slice. The AA edits the knobs, and only during Stage A or
adjudication. Reviewers have `read`, `grep`, `glob` only — they never edit.

After every `write`/`edit`, the harness runs `cargo lid-rs fast --phase N` and
returns the rendering as the tool result. The agent never runs a command.
This is the one deterministic per-edit feedback path: an agent host's own
diagnostics push reaches the main session and never a worker, so a check that
runs *as* the tool result is the only feedback the worker can be sure of.

### 6.2 Sandbox

No network for agents. Fresh worktree per phase — two phases cannot share
one, because the path policy cannot tell another phase's in-flight work from
an edit the agent should not have made, which is the policy working.
Toolchain pinned by `rust-toolchain.toml`. Credentials held by the harness
only. The harness pins the `cargo-lid-rs` it runs and cites the version in
every commit: the hooks that gate a phase today run the *installed* binary,
so a slice that changes its own check cannot use the new check until the tool
is reinstalled — a bootstrap the pipeline makes explicit rather than
accidental.

What the harness bounds and what it does not is README §12's last item: the
checks execute the code the agent wrote, with the harness's privileges. Run
the pipeline where you would run untrusted code.

### 6.3 Limits

```toml
[workspace.metadata.lid_rs.pipeline.limits]
reworks             = 6       # shared budget per run, across phases
max_edits           = 200     # per phase
adjudication_rounds = 1
```

One shared rework budget per run, not a round count per phase. The evidence
is the headless run that built the harness: every rework followed a real
review finding, reworks stayed at the same phase, and the run never rewound
to an earlier one — so a routing design with categories, rewind, and marks
was rejected before it was built, and "one rework then stop" was replaced by
a budget of six. A per-phase cap is a knob the budget makes unnecessary.

### 6.4 The watchdog

An agent host kills a worker that emits nothing for a fixed interval —
600 seconds in the host the phase agents run in today — and a worker blocked
in its own stop hook emits nothing for the whole gate. Raising the hook's
timeout cannot help; the stream watchdog fires first. So a check longer than
the interval runs *between* sessions, as the harness's own code, never inside
the worker's stop: the Phase 7 gate is that check, at check 12's measured
cost, and the headless harness runs it that way. An interactive session
without the harness gates Phase 7 by hand.

---

## 7. Review agents

Rubrics under `skill/references/phase-N.md`, one per reviewer, every rule
answered with evidence. Finding format as §5.3 with `source: "review"` and
`rule: "R3.2"`. `block` returns the phase to the producer; `note` is recorded
and surfaced on the PR. The reviewer reads the phase's *paths*, named in its
prompt — it structurally cannot read a commit.

**The Phase 2 differential.** The reviewer receives Stage A's trial derivation
and diffs it against Phase 2's output as *sentences*, not struct names.
Disagreement is a `block` finding against the LLD sentence — the LLD is
ambiguous there — and it routes to adjudication (§8), not to the Phase 2
agent, because the Phase 2 agent can't fix an LLD.

Reviews are cheap: both reviewer kinds together were 8% of a slice's tokens
against 27% for the phase work they checked, and on that slice every review
found a real fault.

---

## 8. Adjudication

When a run exhausts its rework budget, or a reviewer raises a finding that's
against the LLD or against a protected knob, the harness convenes the two
assistants with: the findings, the LLD, the phase's state files, the last
diagnostics, and the slice's history. They have one round to return a
**decision**:

| Decision | Who | Effect |
|---|---|---|
| **Amend the LLD** | APO | edit `lld.md`; pipeline restarts at Phase 2 |
| **Amend structure** | AA | edit `vocab.rs`, `lexicon.toml`, `conformance.aliases`, `shape.allow_macros`, or the skeleton; pipeline resumes at the affected phase |
| **Accept with note** | both | the finding is downgraded to `note` with a written justification; phase advances |
| **Ask a human** | either | the flow **fails**; the question is registered |

A finding that belongs to *another* slice's LLD — three of them on one walk:
a claim registered but uncitable, two phases in one worktree, a claim
implemented by data that cannot go red — is always **Ask a human**: no agent
propagates into an LLD it was not asked about.

Each decision is committed with trailer `Lid-Rs-Adjudication: <APO|AA>` and the
justification in the message body, so the PR shows *that* the pipeline made a
judgment call and *why*. "Accept with note" is the one that needs watching; the
count of them per slice is on the trace matrix.

**Registering a question.** A failed flow posts one PR comment in a fixed form:

```text
DECISION NEEDED — phase 3, slice auth

Question:   Should account lockout be in scope for login?
Context:    Rule A rejected a five-arm match in load_account; the restructured
            version passes shape but E2 (check 22) fails: AuthError::Locked has
            no claim. The LLD's failure modes don't mention lockout.
Options:    (a) add lockout to the LLD → restart at Phase 2
            (b) remove the variant → resume at Phase 3
Assistants: APO recommends (b): lockout is a separate operation.
            AA concurs; notes a future slice `auth/lockout` would own the variant.
To resume:  push to lld/auth, or comment `/lid decide a|b`.
```

The pipeline stops. The branch stays. A `/lid decide` comment or a push resumes
it. The humans were asked one question, framed, with a recommendation.

---

## 9. Publication, sign-off, and merge

### 9.1 Publication

Gate 7 passes → `regen`, `site`, `pr-body` → the PR flips from draft to ready.
The PR body is the trace matrix, the site link, the adjudication log, and the
phase commit list. **The PR is reviewable as the development process**: its
commits are LLD → claims → skeleton → layers → validators → leaves → gate, each
carrying the SHA of the prompt and rubric that produced it and the version of
the tool that gated it.

### 9.2 Sign-off

`main` requires two approvals through `CODEOWNERS`:

```text
# CODEOWNERS
src/**/lld.md        @product-owners
src/**/spec.rs       @product-owners
src/**/vocab.rs      @engineers
src/**               @engineers
docs/intent/hld.md   @product-owners @engineers
```

The product owner reviews what the slice *means* — the LLD, the claims, the
site's slice page. The engineer reviews what it *is* — the vocabulary, the flow
nodes, the leaves, the traces. Both are reviewing the one thing the pipeline
can't check: whether the leaves mean what the claims say.

### 9.3 Branch protection

```text
Ruleset: lld/**
  restrict deletions:        yes
  require linear history:    no   (phase commits are the point)
  block force-push:          yes

Ruleset: main
  require pull request:      yes
  required approvals:        2, via CODEOWNERS
  required status checks:    lid-pipeline / gate-7
  allowed merge method:      squash only
  restrict deletions:        yes
```

### 9.4 Squash merge

One commit on `main` per slice. The squash message is generated:

```text
auth: user login

<first paragraph of lld.md>

Claims: ValidCredentialsYieldScopedSession, UnknownCredentialsAreRejected,
        BackendFailureIsRejected, BackendFailureIsIndistinguishableToUser

Phases: 2 ×1, 3 ×2, 4 ×1, 5 ×1, 6 ×3, 7 ×1   (attempts)
Adjudications: 0 accepted-with-note, 0 questions

Lid-Rs-Slice: auth
Lid-Rs-Branch: lld/auth
Lid-Rs-Tip: <sha>
Lid-Rs-Site: <url>
Signed-off-by: <product owner>
Signed-off-by: <engineer>
```

`git log main` reads as a list of slices in the language of the HLD, and each
squash carries the rolled-up summary — attempts per phase, LLD amendments,
refusals — so `main` is the retrospective dataset. `git log lld/auth` reads as
how each one was built. The branch is never deleted and its tip is tagged
`slice/<name>`, so the `Lid-Rs-Branch` trailer always resolves.

### 9.5 Subsequent changes

A change to a merged slice is a new branch `lld/<slice>/<change>` from `main`,
with an LLD edit as its first commit. Stage A applies; Stage B runs from Phase
2; the same protections apply. The original `lld/<slice>` stays as the slice's
origin story. A Phase 8 edit that subtracts or rewords has no red run of its
own — README §8 — so its Phase 5 is the reviewer's, not the gate's, and the
reviewer's rubric says so.

---

## 10. Phase agents

**Context packs:** the minimum per phase, assembled by the harness — Phase 2
gets the HLD, the LLD, the lexicon, existing `vocab.rs` files, and the
controlled-language rules; Phase 3 adds `spec.rs`, `vocab.rs`, the shape and
conformance rules, and neighbouring slices' `mod.rs` for seams; Phase 6 gets one
leaf's signature, its claims' text, its validators, and their latest traces.
Grep for the rest. Paths, not documents: a first message carrying twelve
documents buries the one that mattered. The Phase 2 pack **excludes the trial
derivation**.

**Output contract:** each agent ends with a ```` ```commit ```` block — what it
produced, which claims each artifact serves, and what it was unsure about —
which becomes the commit body when the gate passes. Reviewers read it first.

**Prohibitions:** no scope widening; no weakening a check to satisfy it (the
knobs are read-only); no bodies in Phases 3–4; no green in Phase 5; no edits
outside the leaf in Phase 6; no propagation into another slice's LLD (§8).

**Phase 6 fan-out:** one agent per leaf in parallel worktrees, each gated by
`validate --claims`, merged by the harness in leaf order.

---

## 11. Example run — the login slice, hands-off

**Stage A.** The APO and AA draft `lld.md` with the humans. `trial_derive`
fails check 13 on a compound sentence; the APO splits it. Second trial passes.
Draft PR on `lld/auth`.

**Phase 2.** `fast` green on every write. `gate.2` passes (`graph --expect
uncited`: four claims, nothing implements them, as expected). Reviewer diffs
against the trial: identical. Commit `phase 2:`.

**Phase 3.** First `load_account` draft takes `&Username` only; `fast.3` returns
S2 (check 20): condition noun `CredentialStore` in no signature. Agent adds the
parameter. `gate.3` passes. Commit.

**Phase 4.** One layer. Commit `phase 4:`, `Lid-Rs-Layer: 1`.

**Phase 5.** Four validators. `gate.5`: `validate --red` — three failures,
three traces *reached, panicked*; the fourth claim is ubiquitous and
data-implemented, excluded from the red set and held to check 24. Commit.

**Phase 6.** Three leaf agents. `verify_password` returns `Err(Backend)` on
mismatch; `fast.6` runs `validate --claims UnknownCredentialsAreRejected` and
returns *Err(Backend) where Err(InvalidCredentials) promised*. Fixed. `gate.6`:
`suite` green, 25 satisfied. Commit.

**Phase 7.** `mutants`: none survive. `regen`, `site`, `pr-body`. PR ready. No
human was asked anything.

**Sign-off.** The PO reads four sentences and a slice page. The engineer reads
three leaves and four traces. Two approvals. Squash merge: `main` gains
`auth: user login`.

**A month later.** Lockout is requested. `lld/auth/lockout` from `main`; the
APO adds a failure mode to the LLD; the AA adds `Locked` to the candidate list
with its claim; Stage B runs; E2 passes because the variant and the claim
arrive together. `main` gains `auth: account lockout`.

---

## 12. Repo artifacts

```text
lid-rs/agent/            apo.md, aa.md, lid-rs-phase-N.md, lid-rs-review.md,
                         lid-rs-lld-review.md          — synced to .claude/agents/
lid-rs/skill/references/ lld.md (the guideline), phase-N.md (the rubrics),
                         gates.md, discipline.md       — synced to .claude/skills/
lid-rs/workflow/         lid-rs.js                     — synced to .claude/workflows/
Cargo.toml               [workspace.metadata.lid_rs.pipeline] — §5.2, §6.3
CODEOWNERS               §9.2
.github/
  rulesets/              §9.3, if managed as code
  workflows/gate.yml     the gate; lid-pipeline.yml — the draft-PR trigger
target/lid/              per-run reports and traces, not committed
```

---

## 13. Limits

- **Stage A is where judgment lives.** The assistants help, the guideline
  constrains, the trial proves derivability. None of it decides whether the
  slice is the right thing to build.
- **Adjudication can be wrong.** "Accept with note" is a judgment call by two
  agents. It's committed, counted, and on the PR; the humans see every one at
  sign-off. If the count is routinely nonzero, the guideline or the LLD process
  needs work, not the assistants.
- **Fast-gate latency bounds agent quality.** `shape` is a second; `check` should
  be ten. A crate where it isn't needs splitting.
- **The differential needs independence.** The Phase 2 pack must exclude the
  trial. Audit it.
- **Squash merge loses per-phase authorship on `main`.** By design. The branch
  keeps it, protected, linked.
- **The harness runs the code the agent wrote.** §6.2; README §12.
- **The pipeline is a design with a proof, not a product.** The Claude Code
  workflow and hooks run phases today; the headless harness proved the session
  loop against a live door up to the completion it was not permitted to land;
  the catalog, the finding schema, adjudication, and publication are unbuilt
  and are slices in the workspace HLD's map.

---

## 14. Bootstrap

1. README through its bootstrap; then the catalog in `cargo-lid-rs`, in this
   order: `shape`, `check`, `doc`, `graph`, `validate`, `conform`, `suite`,
   `regen`, `commit`, `lint`, `examples`, `mutants`, `site`, `pr-body`. Each is
   usable alone before the harness exists, and `phase-check N` becomes `gate
   --phase N` when its steps are all commands.
2. `[workspace.metadata.lid_rs.pipeline]` from §5.2; measure each budget in
   this repo.
3. The phase rubrics from §4.1's reviewer column, into the skill's references.
4. `lid-rs/agent/`: APO, AA — the coach and the LLD reviewer are their
   ancestors.
5. `lid-rs-pipeline`: sessions (from the canopy client), `fast`/`gate`
   dispatch, the state machine, adjudication, the PR lifecycle.
6. `CODEOWNERS` and the two rulesets.
7. Run the login slice once with a hold after Phase 2 to watch it; then
   remove the hold. Read every `note`. Fix rubrics before prompts, prompts
   before models.
