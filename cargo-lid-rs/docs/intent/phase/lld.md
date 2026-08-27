# phase — a phase commit gates itself, so an agent can build a slice unattended

## Context and Design Philosophy

The methodology's phase walk (README §8) ends each phase with a check: the
LLD's code blocks compile, the claims are documented items, the skeleton
type-checks, the validations are red against `todo!()`, the slice passes the
gate. The operating skill (`docs/intent/skill/lld.md`) makes running those
checks the agent's duty, and its evidence table records what that costs: a
red run skipped under a reused waiver, implementation landed in a Phase 4
skeleton with "tests already pass" noted rather than explained, "commit
anyway" offered at a failed gate. Every one of those is a check that existed
and did not gate, because the thing that had to run it was the thing being
checked.

This slice moves the phase checks out of the agent's hands. The skill's
working-state convention already gives each phase exactly one observable
event — a commit tagged `phase N:` on an `lld/<slice>` branch — so the check
attaches there: a git `commit-msg` hook reads the tag and runs the phase's
check through `cargo lid-rs phase-check`, and a commit whose check fails does
not happen. No model decides whether to run a check or reads a passing one;
only a failure returns to whoever typed `git commit`, as the reason it was
refused. The same hook gates a human at a terminal, an agent in an
interactive session, and an agent inside a workflow, which is what lets the
second half of this slice exist.

The second half is the unattended build: a Claude Code workflow that walks a
slice from an approved LLD to a gated, PR-ready branch with a fresh agent per
phase and no human at the stops. It is the skill run with the reviewer's
seat filled differently — a clean agent at each stop, the human afterwards,
reading the branch's phase commits in order — and it exists only because the
phase checks no longer depend on the agent that would otherwise be trusted
to run them. Its phases, artifacts, and commits are those of an interactive
session; the two modes are interchangeable at every phase boundary.

Two things the workflow deliberately cannot do. It cannot start without a
human-approved LLD: Phase 1 is human-owned (README §8), so the workflow's
input is a branch carrying a `phase 1:` commit, and any event the
methodology routes back to Phase 1 — a check-7 firing, a Phase 8 event
surfacing mid-implementation, a cascade into another slice's LLD, a
`#[mutants::skip]` — ends the run with the decision the human must make.
And it carries no waiver: there is no argument that relaxes a phase, because
the skill's evidence shows a waiver given once is reused.

## Behaviour

### `cargo lid-rs phase-check <n> [--slice <name>]`

Runs the check for phase `n` (1–7) against the project located by `cargo
metadata`, from any directory inside it. Exit status is the verdict; output
names what failed. Phase 6 has no commit of its own (skill, working state),
so `phase-check 6` is an error naming Phase 7.

| Phase | Check | What it proves |
|---|---|---|
| 1 | `cargo doc --no-deps` with broken intra-doc links denied; `cargo test --doc` | The LLD's links resolve and its code blocks compile — it is wired into its module from the first commit |
| 2 | `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings` | The claims are documented items (check 3) that build |
| 3, 4 | `cargo check --all-targets` | The skeleton, at this layer, type-checks (check 4) |
| 5 | Every claim in the slice has at least one `#[validates]` test, and every such test **fails** | The validations are red against `todo!()` before implementation exists |
| 7 | README §4.5, in order, first failure named | The slice passes the gate |

Phase 5 is the check no existing tool runs. The slice's claims are the
`SPEC` records whose source file is `src/spec/<slice>.rs` (kebab-case slice
name to snake_case module), read from the registry dump the mutation
subcommand already uses — never from Rust source (README constraint 2). The
validations citing them are the `VALID` edges on those claims, each carrying
the test's item path; each test runs alone, `cargo test --lib -p <package>
-- --exact <path>`, and the outcome is its exit status. A claim with no
validation, or a validation that passes, is named in the failure. A slice
whose spec file registers no claims is a failure too ("no claims for slice
`<name>`"), never a vacuous pass. A test green before implementation exists
is the case the skill requires an explanation for in the commit; this check
makes the explanation happen before the commit rather than after.

`--slice` defaults to the current branch's name with the `lld/` prefix
removed, the skill's branch convention; a branch not of that form and no
`--slice` is an error for phase 5 and irrelevant to the others.

Phase 7 is README §4.5 as the tool runs it: the same commands in the same
order, `cargo package` for every workspace package whose metadata does not
say `publish = false`, `sync --check` and `mutants` invoked through the
library rather than as subprocesses. The README's list stays canonical
(§4.5: "every copy of the list a project keeps must match"); this is one
more copy, held to the same rule. Steps a workspace appends after the floor
(this one's `mdbook build book`) are outside the tool's knowledge and stay
in the project's CI (see Deferred).

### `cargo lid-rs hook commit-msg <file>`

The body of a git `commit-msg` hook. Reads the message from `<file>`; if its
subject line begins `phase N:` (N a digit), runs `phase-check N` and exits
non-zero with the check's output when it fails, so git refuses the commit
and the output is what `git commit` prints. A message without the tag exits
zero: the hook gates phase commits, and only phase commits. A tag naming a
phase with no check (`phase 0:`, `phase 6:`, `phase 8:`) is refused with a
message naming the phases that have one — a mistyped tag must not pass as an
untagged commit.

`--no-verify` bypasses every git hook; the skill names it beside `#[allow]`
and threshold bumps as a suppression (rule 2), and the CI gate remains the
backstop that catches a bypassed phase commit's consequences at PR time.

### Installation: `init` and `sync`

The hook scripts carry no logic: each is one `exec` of the binary, so the
behaviour lives under this crate's own gate and the scripts never need
updating. The `lid-rs` crate ships them as a `hooks/` directory —
`commit-msg`, `subagent-start`, `subagent-stop`, and `run`, the one script
the other three exec through — and `sync` mirrors the directory to
`.lid-rs/hooks/` (executable) like every other artifact the crate ships:
one mirror table, one any-difference rule. `run` resolves the binary: in
any project, `cargo lid-rs`; in the tool's own workspace (recognised by
`cargo-lid-rs/Cargo.toml` at the repository root), `cargo run -q -p
cargo-lid-rs --`, so the hooks here exercise the working tree, as CLAUDE.md
requires of the gate, without the deprecated alias-over-external-subcommand
that `.cargo/config.toml` already declines. `sync` also sets `git config core.hooksPath .lid-rs/hooks` in the project's
repository on every run (the config is per-clone, so a fresh clone needs
it), and `sync --check` fails when it is not set to that. CI is a fresh
clone that never runs `sync`, and a clone where nothing commits has no
"forgot to sync" to detect, so the emitted `gate.yml` registers the hooks
itself with that one `git config` line before the gate; `--check` stays
strict rather than learning to guess where it is running. `init` obtains
both by calling `sync`, as it already does for the skill; a `core.hooksPath`
already set to another value is a conflict under `init`'s all-or-nothing
rule, since the project has a hook arrangement of its own to reconcile.
These are the first non-skill artifacts `sync` owns — the case its LLD
deferred until a `lid-rs` change needed it.

### The phase agent: `lid-rs-phase`, and `cargo lid-rs hook subagent-{start,stop}`

The workflow's workers run as a distributed Claude Code agent type,
`lid-rs-phase`, shipped in the `lid-rs` crate as `agent/lid-rs-phase.md`
and mirrored by `sync` to `<workspace root>/.claude/agents/lid-rs-phase.md`.
Its frontmatter declares two hooks on itself — `SubagentStart` and `Stop`
(which Claude Code fires as `SubagentStop` for a subagent) — each one
command: the synced `.lid-rs/hooks/subagent-start` and
`.lid-rs/hooks/subagent-stop` scripts, which exec `cargo lid-rs hook
subagent-start` and `… subagent-stop`. Declaring the hooks on the agent scopes them to that agent
type by construction: no `settings.json` entry, no matcher, and `sync` owns
the whole arrangement under the same strict rule as the skill directory.

The pair enforces one thing the `commit-msg` hook cannot: *a phase worker
does not end silently.* Claude Code passes each hook the agent's `agent_id`
and, on stop, `stop_hook_active` and the agent's last message, as JSON on
stdin. `subagent-start` records the branch's `HEAD` under
`<target>/lid-rs/agents/<agent_id>`; `subagent-stop` compares. If `HEAD`
has moved, the worker committed and the stop is allowed. If it has not and
this is the first stop attempt (`stop_hook_active` false), the stop is
refused — a `{"decision": "block", "reason": …}` on stdout, the form Claude
Code accepts beside exit 2, so a hook that itself fails (exit 1) is shown to
the user rather than mistaken for a refusal — and the reason is delivered to
the worker as its next turn: *no `phase N:` commit was made since you
started; either commit it, or end with the numbered decisions that block
it.* The second attempt is allowed regardless: a worker that has hit a
Phase 1 event has nothing to commit and must be able to say so, and the
refusal exists to catch a forgotten commit, not to hold an agent hostage —
Claude Code's own eight-refusal cap is never reached. Both commands exit
zero when the agent record is missing (a `subagent-stop` with no matching
`subagent-start` is not the worker's fault).

A newly synced `lid-rs-phase.md` is not usable the instant it is written:
in the session that wrote it, the agent type was reported unavailable at
first and available some minutes later, so discovery is delayed rather
than immediate; a fresh session is the reliable path. Whether an agent's
own frontmatter `SubagentStart` fires for itself is not documented; if it
does not, no record is written and every stop is allowed — the hook fails
open, by the no-record claim — and the next phase's precondition still
reads the log. The first workflow run settles it.

### The workflow: `cargo lid-rs sync` ships `.claude/workflows/lid-rs.js`

The `lid-rs` crate ships `workflow/lid-rs.js` beside `skill/` and
`agent/`; `sync` mirrors it to `<workspace root>/.claude/workflows/lid-rs.js`
under the same strict rule (any difference fails `--check`). It runs as
`Workflow({name: "lid-rs", args: {slice: "<name>"}})` — a Claude Code
workflow needs the user's explicit invocation, which is the human's
decision to run a slice unattended.

Its shape:

- **Precondition** (one agent, structured output): branch `lld/<slice>`
  exists, its log contains a `phase 1:` commit, and `git log` shows which
  phase commits are already present. The run starts at the first phase
  without a commit — resumption is reading the branch, exactly as the skill
  prescribes for a session. No `phase 1:` commit means the run stops before
  doing anything: the LLD is the human's.
- **Per phase, 2 through 7**: a *worker* agent of type `lid-rs-phase`,
  with the phase's file (`.claude/skills/lid-rs/references/phase-N.md`),
  the slice's LLD, and the branch — nothing from any other agent's context
  — produces the phase's artifact and commits it under the tag; the
  `commit-msg` hook gates the commit and the stop hook refuses a silent
  end. It returns, as structured output, the commit it made and the ≤3 numbered
  decisions the skill's stop contract requires. A *reviewer* agent, equally
  clean, reads the artifact against the phase's checklist and the
  `discipline.md` rows tagged for that phase, prompted to refute, and returns
  approve-or-findings. The reviewer commits nothing, so it runs as an
  ordinary agent, outside the stop hook. Findings go back to a worker once;
  a second rejection ends the run with the findings as the human's
  decisions. Phase 6 and 7 are one worker and one reviewer, since they
  share a commit.
- **Terminal states**: *PR-ready* — every phase committed and the gate
  passed, the run returns the branch and the accumulated decisions for the
  human's PR review; or *stopped at phase N* — with the decisions that
  stopped it. There is no third state and no waiver argument.

The script sequences and checks the shape of what agents return; it cannot
run `cargo` or `git` itself (a workflow script has no filesystem or shell).
Every check that matters runs in a hook, where the model is not consulted.

### Modes are interchangeable

A session under the skill and a workflow run produce the same branch: the
same tags, the same artifacts, the same gate at each commit. Either can pick
up where the other stopped by reading `git log`. If the two modes' output
could be told apart in git, one of them has left the methodology.

## Shape

| Item | Role |
|---|---|
| `phase::run(args)` | Subcommand entry: parses `<n>` and `--slice`, dispatches to `check` |
| `phase::hook(args)` | One `match` over the hook kind: `commit-msg <file>`, `subagent-start`, `subagent-stop` |
| `hook_commit_msg(project, file)` | Reads the message, `tag_of`, then `check` or exit zero |
| `hook_subagent_start(project, stdin)` | Records `HEAD` under `<target>/lid-rs/agents/<agent_id>` |
| `hook_subagent_stop(project, stdin)` | `HEAD` moved, or second attempt, or no record → allow; else refuse with the instruction |
| `HookInput` | The fields read from the hook's stdin JSON: `agent_id`, `stop_hook_active` — a boundary type; interior code takes plain values |
| `lid-rs/agent/lid-rs-phase.md` | The phase agent: frontmatter hooks naming the two commands; body is the worker's standing instruction |
| `Phase` | Closed set `One`–`Five`, `Seven`; `TryFrom<u8>` refuses 0, 6, 8+ with the message above |
| `tag_of(subject) -> Option<Phase>` | The `phase N:` prefix, or none |
| `slice_of_branch(name) -> Option<String>` | `lld/<slice>` → `<slice>` |
| `check(project, phase, slice) -> Result<(), String>` | One `match` over `Phase`: phase 5 → `check_red`; every other phase → `execute(plan(phase))` |
| `plan(phase, project) -> Vec<Step>` | The phase's command sequence as data — phases 1, 2, 3/4, 7 — so the sequence is unit-testable and mutation-covered without running cargo |
| `execute(project, steps) -> Result<(), String>` | Runs steps in order; the first failure is the result, named |
| `check_red(project, slice)` | Phase 5: `slice_claims` → `claim_validations` → `run_test` each; collects the unvalidated and the green |
| `slice_claims(registry, slice) -> Vec<String>` | `SPEC` records whose file is the slice's spec module |
| `claim_validations(registry, claims) -> Vec<TestPath>` | `VALID` edges on those claims, item paths made libtest-relative via `mapping` |
| `run_test(project, package, path) -> bool` | One `cargo test --lib -p … -- --exact` run; exit status |
| `init` additions | A foreign `core.hooksPath` joins the conflict set |
| `sync::artifacts()` | The mirror table: `skill/` → `.claude/skills/lid-rs/`, `workflow/` → `.claude/workflows/`, `agent/` → `.claude/agents/`, `hooks/` → `.lid-rs/hooks/` (executable) |
| `sync::assert_hooks_path`, `sync::check_hooks_path` | `core.hooksPath` set to, and verified as, `.lid-rs/hooks` |
| `lid-rs/hooks/{run,commit-msg,subagent-start,subagent-stop}` | The shipped hook scripts; `run` resolves the binary, the others exec through it |
| `lid-rs/workflow/lid-rs.js` | The workflow, shipped in the crate's tarball |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Where the phase check attaches | A git `commit-msg` hook keyed on the `phase N:` tag | A Claude Code `PreToolUse` hook on `Bash` matching `git commit`; a `SubagentStop` hook on a distributed agent type used by the workflow; the workflow script itself; the agent, as today | The tag is the one event every phase already has, and git's hook gates everyone who commits — human at a terminal, agent in a session, agent in a workflow — with no shell-command parsing (a `PreToolUse` hook must recover `-m` from a command line) and no harness coupling. A stop hook cannot know which phase the agent was expected to commit; a workflow script cannot run anything; the agent running its own check is the failure mode on record. The stop hook is kept for the one thing it can know — whether a commit happened at all — below. |
| Where the stop hook is declared | Frontmatter of the distributed `lid-rs-phase` agent (`SubagentStart` + `Stop`) | A `SubagentStop` entry in `.claude/settings.json` with a matcher on `workflow-subagent`; no stop hook (the next phase's precondition reads the log) | Declared on the agent, the hook is scoped to phase workers by construction and `sync` owns the file whole; a settings entry gates every workflow agent (reviewers included), lives in a file `sync` must not own, and needs a matcher that the 2026-08-26 spike showed filters on `agent_type` exactly — workable, but two files where one will do. Spiked the same day: settings-level hooks fire for workflow agents, exit 2 reaches the subagent as its next instruction, and agent definitions load at session start; frontmatter hooks themselves are verified at Phase 5. |
| Stop refusal budget | One refusal, then allow | Refuse until a phase commit exists (Claude Code caps at eight); refuse unless the last message carries a stop marker | A worker at a Phase 1 event has nothing to commit and must be able to stop with its decisions; a marker in prose is a protocol the model can forget. One refusal catches a forgotten commit — the only case the hook is for — and never makes an honest stop impossible. |
| `cargo lid-rs` from a hook | The synced `hooks/run` script: `cargo lid-rs` in any project, `cargo run -q -p cargo-lid-rs --` when the repository root holds `cargo-lid-rs/Cargo.toml` | A cargo alias `lid-rs = "run -p cargo-lid-rs --"` in this workspace; an installed binary refreshed by the gate; hooks invoking `cargo run` everywhere | The synced files are byte-identical in every project, so the choice has to live inside them. The alias is deprecated when it shadows an installed external subcommand (rust-lang/cargo#10049, already declined in `.cargo/config.toml`); an installed binary is stale by exactly the change under test. The workspace test is precise — only the tool's own repository has that manifest — and the consumer path stays the plain command. |
| Check logic location | In the `cargo-lid-rs` binary; the hook script is one `exec` line, shipped and mirrored like the skill | A shell script carrying the phase table, synced from the crate; `init` writing the one-liner from a template and `sync` checking it separately | The logic gets claims, tests, and check 12 like any other code; the script never changes. Shipping it in the crate keeps one mirror table instead of a mirror plus a special case. The binary's phase table is version-coupled to the skill's phase walk the same way `mutants` is coupled to the registry format — the seam the skill LLD defers is unchanged by this. |
| Phase 5 test execution | One `cargo test … --exact` run per validation, exit status as verdict | One `cargo test --lib` run with libtest output parsed; `--format json` | One process per test costs seconds on a slice-sized set and needs no parsing of libtest's human-oriented output; JSON output is nightly-only. |
| Phase 5 slice identity | `SPEC` records by source file `src/spec/<slice>.rs`, slice from the branch name | Parse `src/spec/` for the module; a `--claims` list; a `#[lid_rs::slice]` attribute | The registry already carries the file; the branch convention already carries the slice; constraint 2 forbids the parse. |
| Phase 7's list | The tool holds README §4.5 verbatim, in order, as one more copy the README's rule binds | Make `cargo lid-rs gate` the canonical gate and reduce the README to a pointer; read the list from a config table | Keeping the list canonical in prose is a deliberate choice for now: the spec stays readable without the tool, and the copies-must-match rule is already the discipline. Promoting the tool to canonical is a README change with its own slice when the copies are seen to drift. |
| Untagged commits | Not gated; the Phase 7 slice commit is tagged (`phase 7: <version>: <what and why>`) so the hook runs the full gate on it | Gate every commit on an `lld/*` branch with the full gate; leave the slice commit untagged and have the Phase 7 worker run `phase-check 7` itself | The full gate rebuilds under mutation; running it on every fixup commit makes people bypass hooks. An untagged slice commit would put the gate back in the agent's hands, which is the failure this slice removes — so the skill's Phase 7 convention gains the tag, and the changelog-readable subject follows it. |
| Mistyped tags | `phase 0/6/8+:` refused, not ignored | Treat as untagged | A tag the hook silently ignores is a phase that silently escapes its check. |
| Hook installation | `init` writes `.lid-rs/hooks/commit-msg` and sets `core.hooksPath`; `sync` re-asserts and `--check` verifies | Write into `.git/hooks/` (unversioned, lost on clone); a `pre-commit`-framework config; a Claude Code hook in `.claude/settings.json` | `core.hooksPath` is the only git mechanism that makes a versioned hook directory authoritative; it is per-clone config, which is why `sync` owns re-asserting it. `.claude/settings.json` is user territory `sync` must not own whole, and gates only Claude Code. |
| The workflow's input | A branch with a human-approved `phase 1:` commit; no waiver argument | A slice name, with the workflow drafting the LLD; a `--waive` argument mirroring the skill's per-slice waiver | Phase 1 is human-owned; a workflow that drafts it and continues has approved its own LLD. The skill's evidence shows a waiver given once is reused; an argument is a waiver given every time. |
| Reviewer at each stop | One clean agent per phase, prompted to refute, one rework round | No reviewer (worker commits, human reviews the PR); a judge panel per phase | A clean reviewer is also the test that the artifact is context-free: a reviewer who cannot follow the LLD to the claims has found a doc bug, which is the failure the interactive mode cannot see. A panel per phase exceeds the cost a slice warrants; one rework round bounds the run. |
| Where the workflow lives | `lid-rs/workflow/lid-rs.js`, synced to `.claude/workflows/lid-rs.js` | Inside `skill/` (synced under `.claude/skills/lid-rs/`, where Claude Code does not look for workflows); a separate crate; the plugin | The workflow must be at `.claude/workflows/` to be invoked by name, and it is version-coupled to the skill files it points agents at, so it ships with them. |
| Workflow prompts | Point agents at the synced skill files by path; restate nothing | Inline the phase text in the script | The skill LLD rejected composing two documents at run time; a script that paraphrases the skill is that failure. |

## Open Questions & Future Decisions

### Deferred
1. Workspace-appended gate steps (`mdbook build book` here): a
   `[workspace.metadata.lid_rs] gate_extra` list `phase-check 7` would run
   after the floor. Until then those steps live in CI only.
2. A `PreToolUse` hook on the `lid-rs-phase` agent refusing `git commit
   --no-verify` — closes the bypass for phase workers with no new file; a
   human's bypass stays a CI matter.
3. The workflow's Phase 8 path: an edited LLD on an existing slice's branch
   re-enters at Phase 2 with renamed claims; the precondition agent's
   "first phase without a commit" reading needs the `-<what-changed>`
   branch convention to be settled first.

## References

- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate this tool runs at Phase 7), [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the phases and who owns each), constraint 2 (no source parsing).
- `docs/intent/skill/lld.md` (workspace) — the working-state convention this slice gates, and the evidence table that says why the agent must not run its own checks.
- `docs/intent/sync/lld.md` — the strict mirror rule this slice extends to the workflow and the hook; its Deferred 1 anticipated the case.
- `docs/intent/init/lld.md` — the all-or-nothing conflict rule the hook installation joins.
- `docs/intent/cargo-lid-rs/lld.md` — the registry dump and the item-path mapping `check_red` reuses.
- Claude Code hooks, subagents, and workflows (`code.claude.com/docs/en/hooks`, `/sub-agents`, `/workflows`) — the stop-hook input fields, exit-2 semantics, frontmatter hook syntax, and the `.claude/workflows/` and `.claude/agents/` locations this slice relies on.
