# phase — a phase is run by an agent that can only edit, and its commit is the check passing

## Context and Design Philosophy

The methodology's phase walk (README §8) ends each phase with a check: the
LLD's code blocks compile, the claims are documented items, the skeleton
type-checks, the validations are red against `todo!()`, the slice passes the
gate. The operating skill (`docs/intent/skill/lld.md`) once made running
those checks the agent's duty, and its evidence table records what that
cost: a red run skipped under a reused waiver, implementation landed in a
Phase 4 skeleton with "tests already pass" noted rather than explained,
"commit anyway" offered at a failed gate. Every one of those is a check that
existed and did not gate, because the thing that had to run it was the
thing being checked.

This slice takes the checks — and every other action the methodology
forbids in a phase — out of the agent's hands by giving each phase to a
subagent that holds no tool but reading and editing. The agent never runs
a command and never commits. Its edits are bounded by a per-phase path
policy enforced before each tool call; after each edit it is handed the
compiler's verdict; and when it stops, the phase's check runs, and if it
passes, the phase's files are committed under the phase tag. A commit is
not something the agent does; it is what a passing check produces. When
the check fails, the agent is kept running and told what failed, what the
skill says to do about it, and what this phase's policy lets it do.

The policy is the same whether the agent is careless or has been talked
into something by a file it read — the confused-deputy case: an LLD that
says "raise the threshold" is text in a file, and the phase that reads it
cannot edit `clippy.toml`, `docs/intent/`, or another slice's module,
whoever asks. What the methodology says must not happen in a phase is made
impossible in that phase, by code the agent cannot change, not by prose
the agent is trusted to follow. What cannot be removed — the agent writes
code that the check then executes — is bounded by environment isolation
(the sandbox, a worktree), which this design assumes rather than replaces.

The same per-phase agents serve both modes. Interactively, the skill's
main session delegates each phase to the phase's agent and presents its
commit to the human at the stop; unattended, the `lid-rs` workflow puts a
clean reviewer agent in that seat and the human reads the branch's phase
commits afterwards. Their output is indistinguishable in git — the same
tags, the same checks, the same instrumented commit bodies — so either
mode can pick up where the other stopped.

Two things the unattended mode deliberately cannot do. It cannot start
without a human-approved LLD: Phase 1 is human-owned (README §8), so the
workflow's input is a branch carrying a `phase 1:` commit, and any event
the methodology routes back to Phase 1 — a check-7 firing, a Phase 8 event
surfacing mid-implementation, a cascade into another slice's LLD, a
`#[mutants::skip]` — ends the run with the decision the human must make.
And it carries no waiver: there is no argument that relaxes a phase,
because the skill's evidence shows a waiver given once is reused.

## Behaviour

### `cargo lid-rs phase-check <n> [--slice <name>]`

Runs the check for phase `n` (1–7) against the project located by `cargo
metadata`, from any directory inside it. Exit status is the verdict; output
names what failed. Phase 6 has no commit of its own (skill, working state),
so `phase-check 6` is an error naming Phase 7. This is the check the stop
hook runs; it also exists on its own for a human, CI, or the Phase 1 commit
the human makes.

| Phase | Check | What it proves |
|---|---|---|
| 1 | `cargo doc --no-deps` with broken intra-doc links denied; `cargo test --doc` | The LLD's links resolve and its code blocks compile — it is wired into its module from the first commit |
| 2 | `cargo check --all-targets`; `cargo clippy --all-targets -- -D warnings` | The claims are documented items (check 3) that build |
| 3, 4 | `cargo check --all-targets` | The skeleton, at this layer, type-checks (check 4) |
| 5 | Every claim in the slice has at least one `#[validates]` test, and every such test **fails** | The validations are red against `todo!()` before implementation exists |
| 7 | README §4.5, in order, first failure named | The slice passes the gate |

Phase 5 is the check no other tool runs. The slice's claims are the `SPEC`
records whose source file is `src/spec/<slice>.rs` (kebab-case slice name
to snake_case module), read from the registry dump the mutation subcommand
already uses — never from Rust source (README constraint 2). The
validations citing them are the `VALID` edges on those claims, each
carrying the test's item path; each test runs alone, `cargo test --lib -p
<package> -- --exact <path>`, and the outcome is its exit status. A claim
with no validation, or a validation that passes, is named in the failure. A
slice whose spec file registers no claims is a failure too ("no claims for
slice `<name>`"), never a vacuous pass.

`--slice` defaults to the current branch's name with the `lld/` prefix
removed, the skill's branch convention; a detached `HEAD` or a branch not of
that form names no slice, which fails phase 5 naming the convention and is
irrelevant to the other phases.

Phase 7 is README §4.5 as the tool runs it: the same commands in the same
order, `cargo package` for every workspace package whose metadata does not
say `publish = false`, `sync --check` and `mutants` invoked through the
library rather than as subprocesses. The README's list stays canonical
(§4.5: "every copy of the list a project keeps must match"); this is one
more copy, held to the same rule. Steps a workspace appends after the floor
(this one's `mdbook build book`) are outside the tool's knowledge and stay
in the project's CI (see Deferred).

### The phase agents

The `lid-rs` crate ships one Claude Code agent definition per phase that
commits — `agent/lid-rs-phase-2.md`, `-3`, `-4`, `-5`, `-7` (Phase 7's
agent does Phase 6, which has no commit of its own) — and one reviewer,
`agent/lid-rs-review.md`. `sync` mirrors them to `.claude/agents/`. An
agent's frontmatter is its whole policy, readable in one screen:

- `tools:` — a phase agent has `Read, Grep, Glob, LSP, Edit, Write` and
  nothing else: no Bash, no git, no network. The reviewer has `Read, Grep,
  Glob, LSP`.
- Three hooks, each one command naming the binary directly — `PreToolUse`
  → `cargo lid-rs hook pre-tool <n>`, `PostToolUse` on `Edit|Write` →
  `cargo lid-rs hook post-edit <n>`, `Stop` → `cargo lid-rs hook stop <n>`
  — with no
  script in between that code could rewrite (Security posture, below). The
  phase number is a literal in the file, so the hook never has to infer
  which phase it is serving; the binary locates the repository root itself
  through `cargo metadata`, as every subcommand does.

The body is the standing instruction: read the skill's dispatcher and the
phase's file, the slice's LLD, and `git log` on the branch; do that phase;
end with either a commit message or numbered decisions (the stop protocol
below). Nothing in it is a rule the hooks do not also enforce.

### `hook pre-tool <n>` — the path policy

Runs before every tool call the phase agent makes; reads Claude Code's
hook JSON (`agent_id`, `tool_name`, `tool_input`). For `Edit`, `Write`, and
any other editing tool, the target path must be in the phase's allowed set,
relative to the slice's crate — the workspace package whose manifest
directory holds `docs/intent/<slice>/lld.md`, found on the filesystem, not
by parsing Rust:

| Phase | May write |
|---|---|
| 2 | `src/spec/<slice>.rs`, `src/spec/mod.rs` |
| 3, 4 | `src/<slice>.rs`, `src/<slice>/**`, `src/lib.rs` |
| 5 | `src/<slice>.rs`, `src/<slice>/**` |
| 7 (with 6) | `src/<slice>.rs`, `src/<slice>/**` |
| reviewer | nothing |

Everything else is refused, in every phase — named here because each is a
rule the skill states and a moment its evidence table records the rule
being dropped: `docs/intent/**` (an LLD change is a Phase 8 event to report,
never to make), `src/spec/**` after Phase 2 (a claim change goes through
the LLD), any other slice's module (untraced helpers belong to the slice
that adds them), `Cargo.toml`, `clippy.toml`, `rust-toolchain.toml`,
`.cargo/`, `.github/`, `.claude/` (a threshold, a lint level, a policy, or
a hook is never the agent's to change). A refusal is a block
whose reason is the skill's own sentence for that moment, quoted from the
synced `references/discipline.md` row, plus what the phase may do instead:
proceed within the allowed paths, or end with the numbered decision.

Reading is never refused: the agent may read anything, which is what makes
the policy a confused-deputy boundary rather than a secrecy one.

The same hook keeps the **tally**: one record per `agent_id` under
`<target>/lid-rs/agents/`, counting tool calls by kind — edits (`Edit`,
`Write`), observations (`Read`, `Grep`, `Glob`, `LSP`), commands (`Bash`,
which the tool list makes impossible and the tally makes visible if a
definition ever drifts), and refusals. `post-edit` and `stop` add their
checks and refusals to it.

### `hook post-edit <n>` — the compiler after every edit

Runs after every `Edit` or `Write`: `cargo clippy --all-targets -- -D
warnings` on the workspace, its output (or "clean") handed back as
additional context on the tool result. It refuses nothing — a mid-refactor
warning is information — and it is the only compile feedback the agent
gets, since the agent cannot run cargo and Claude Code delivers
rust-analyzer's diagnostics to the main session only, never to a subagent
(measured; see Decisions). Incremental clippy costs a third of a second on
this workspace after a small edit; check alone saves a tenth and sees less.

### `hook stop <n>` — the check, then the commit

Runs when the phase agent ends. Claude Code passes the agent's final
message (`last_assistant_message`), its id, and `stop_hook_active`. The
message must carry exactly one of two fenced blocks:

- ```` ```commit ```` — the proposed commit message, subject `phase <n>:
  <what> for <slice>` (Phase 7: `phase 7: <version>: <what and why>`). The
  hook runs `phase-check <n>`; on success it stages the phase's allowed
  paths — exactly the policy's set, nothing else — commits the message with
  the tally appended as trailers (`Lid-Rs-Phase`, `Lid-Rs-Tools`,
  `Lid-Rs-Checks`, `Lid-Rs-Refusals`), and allows the stop. Nothing staged
  is a refusal ("no change to commit"). A subject whose tag is not this
  agent's phase is a refusal.
- ```` ```stop ```` — the numbered decisions that block the phase. The hook
  commits nothing and allows the stop: an honest "this needs the human"
  must always be possible, and the decisions travel to the reviewer's seat
  through the workflow's structured output.

A message with neither block, or both, is refused with the format.

A failed check is a refusal — Claude Code's `{"decision": "block",
"reason": …}` on stdout, which keeps the agent running with the reason as
its next turn. The reason is three parts, in order: the failing step's
output verbatim; the skill's correct response for the check that fired,
quoted from the synced `references/gates.md` row (a clippy lint maps to
its check — `cognitive_complexity` 7, `fn_params_excessive_bools` 8,
`too_many_lines` 9, `wildcard_enum_match_arm` 6, `missing_docs` 3; a red-run
failure to the Phase 5 rule; a survivor to check 12); and what this phase's
policy permits (fix within the allowed paths, or end with a `stop` block).
For check 7 in Phase 6 that spells out that the fix is a claim in an LLD
this phase cannot edit. Claude Code ends the agent after eight consecutive
refusals; the phase is then uncommitted, the tree dirty, and the workflow
reports it as stopped — the next worker's precondition refuses a dirty
tree rather than building on it.

Before the check and again after it, the stop hook verifies integrity:
the synced artifacts match the dependency's (`sync::check`), and nothing
outside the phase's allowed paths has changed (`git status` filtered by the
policy). Either failing is a refusal that names what moved and commits
nothing — the check executed the agent's code, and that code may have
written what the agent could not (Security posture, below).

The stop hook is the only place a phase commit is made. The agent has no
git; `--no-verify` has nothing to bypass.

### The trusted binary

`cargo lid-rs` in a hook is the installed binary. In every consumer that
binary is immutable to the agent. In the tool's own workspace it is not the
working tree: a Phase 6 worker here edits `cargo-lid-rs/src/`, and a hook
that ran from source would run whatever the worker had just written. So
this workspace's hooks, too, run the installed binary, refreshed from
`main` after a merge (`cargo install --path cargo-lid-rs --locked`) — a
policy change is enforced only after it has landed. That is the one place
the workspace's "run from source" rule does not apply, and the reason is
the boundary.

### Installation: `init` and `sync`

`sync` mirrors three artifacts from the resolved `lid-rs` crate under one
any-difference rule: `skill/` → `.claude/skills/lid-rs/`, `workflow/` →
`.claude/workflows/`, `agent/` → `.claude/agents/`. `init` obtains them by
calling `sync`, as it already does for the skill. Nothing is registered
with git and no script is installed; a fresh clone is ready once the
`lid-rs` binary is on the path.

### The workflow: `.claude/workflows/lid-rs.js`

The crate ships `workflow/lid-rs.js`; `sync` mirrors it. It runs as
`Workflow({name: "lid-rs", args: {slice: "<name>"}})` — a Claude Code
workflow needs the user's explicit invocation, which is the human's
decision to run a slice unattended.

- **Precondition** (one reviewer agent, structured output): branch
  `lld/<slice>` exists, its log holds a `phase 1:` commit, the tree is
  clean, which phase commits are present, and the slice's execution class
  (Security posture, below) — a compile-time slice is reported before any
  worker runs, and the run proceeds only if `args.compile_time` was passed,
  the human's explicit acceptance for that slice. The run starts at the first
  phase without a commit — resumption is reading the branch, as the skill
  prescribes. No `phase 1:` commit means the run stops before doing
  anything: the LLD is the human's.
- **Per phase, 2, 3, 4, 5, 7**: the phase's agent (`agentType:
  "lid-rs-phase-<n>"`) works; its stop hook commits or refuses. It returns,
  as structured output, whether it committed, the commit, and the ≤3
  numbered decisions the skill's stop contract requires. A reviewer agent
  reads the commit against the phase's checklist and the `discipline.md`
  rows tagged for that phase, prompted to refute, and returns
  approve-or-findings. Findings go back to a worker once; a second
  rejection ends the run with the findings as the human's decisions.
- **Terminal states**: *PR-ready* — every phase committed and the gate
  passed, the run returns the branch and the accumulated decisions for the
  human's PR review; or *stopped at phase N* — with the decisions that
  stopped it. There is no third state and no waiver argument.

The script sequences and checks the shape of what agents return; it cannot
run `cargo` or `git` itself. Every check that matters runs in a hook, where
the model is not consulted.

### The interactive mode

The skill's main session is an orchestrator: at each phase it spawns the
same phase agent (the Agent tool, `subagent_type: lid-rs-phase-<n>`), then
presents the commit — `git show`, the tally, the decisions — to the human,
who says "continue" or gives findings for a rework worker. The main session
edits no code in a LID project; the phase agent is the only thing that
does, in either mode. This is the cascade into the skill
(`docs/intent/skill/lld.md`): Phases 2–7 become "spawn the phase's agent,
review its commit", and the working-state section says what the tally
trailers are.

### What the commit body carries

Every phase commit ends with trailers the stop hook writes from the tally:

```text
Lid-Rs-Phase: 6
Lid-Rs-Tools: 14 edits, 9 observations, 0 commands
Lid-Rs-Checks: 14 post-edit, 1 stop
Lid-Rs-Refusals: 1 policy, 0 stop
```

The ratio of deterministic steps to agent-chosen ones is then in git for
every phase of every slice — `commands` is structurally zero and the tally
proves it — and refusals per phase is the quality signal: a phase whose
refusals rise is a phase whose skill file or policy is teaching the wrong
thing.

## Security posture

What this design bounds is the agent's **tool calls**: no command it
chooses, no file outside its phase, no commit it makes, no gate it can
skip — and therefore no instruction, from the prompt or from any file it
reads, that could make it do those things. That is the confused-deputy
boundary, and it holds against carelessness and injection alike.

What it does not bound is the **code the agent writes**, which the hooks
execute with the session's privileges. A security-minded reader should
hold these facts:

- **When the agent's code runs.** Phases 2–4 compile only; nothing the
  agent wrote executes. The first stop that executes it is Phase 5 — its
  `#[validates]` tests, each run alone, and with them everything that runs
  when the test binary loads: static constructors (the mechanism the
  registry itself uses) execute for every `--exact` run and for the
  registry dump, whichever test was selected. Phase 7 runs the whole suite,
  the doctests in the slice's `///` comments, and then the mutation engine,
  which builds and runs the suite many times. The agent chooses when it
  stops, so from Phase 5 on this is execution on demand.
- **What that code can do** is what the process can: reach the network,
  read secrets in the environment, write anywhere — including the files
  the policy protects from the *Edit* tool. A test that rewrites
  `clippy.toml`, `.claude/agents/`, or runs `git push` is not an Edit call.
  The integrity check at the stop detects and refuses such writes inside
  the repository; it detects nothing outside it.
- **Compile time.** A new proc macro or build script needs `Cargo.toml`,
  which the policy refuses, so an ordinary slice adds no compile-time
  execution. A slice *in* a proc-macro crate, or in a crate whose build
  script consumes the slice's files, executes the agent's code after every
  edit; such a slice is a **compile-time slice**, reported by the
  precondition from `cargo metadata` (a `proc-macro` or `custom-build`
  target), and the unattended mode refuses it without the human's explicit
  acceptance for that run.
- **Reads are unbounded** by design; confidentiality is not a property of
  this boundary. Nothing the agent reads leaves through a tool — it has no
  network and no command — but what its code reads at Phase 5 or 7 can.

The controls this slice adds are therefore honest about their reach: the
policy bounds the agent, the integrity check bounds persistence inside the
repository, the hooks name the binary directly so no synced script is
left for code to rewrite, and the execution class is disclosed. The
boundary that closes the rest is **environment isolation**: run an
unattended build only where you would let untrusted code run — a container
or VM, or a session whose sandbox denies the network and confines writes —
with no credentials it does not need. README §12 states this; every phase
agent's body restates it. Running each check itself under an OS sandbox is
a control the tool could own later (Deferred), and until it exists this
document does not imply it.

## Shape

| Item | Role |
|---|---|
| `phase::run(args)` | `phase-check` entry: parses `<n>` and `--slice`, dispatches to `check` |
| `phase::hook(args)` | One `match` over the hook kind: `pre-tool <n>`, `post-edit <n>`, `stop <n>`; each reads Claude Code's JSON from stdin |
| `Phase` | Closed set `One`–`Five`, `Seven`; `TryFrom<u8>` refuses 0, 6, 8+ |
| `Step` | Closed set: the gate's steps plus the red run |
| `plan(phase, publishing) -> Vec<Step>` | A phase's steps as data |
| `execute`, `execute_with`, `run_step` | Runs steps in order; the first failure is the result |
| `check_red`, `slice_claims`, `claim_validations`, `run_test`, `unvalidated`, `red_verdict` | The phase 5 red run over the registry dump |
| `slice_of_branch`, `resolve_slice`, `current_branch` | The slice from `lld/<slice>`; a detached `HEAD` names none |
| `HookInput` | The boundary type over the hook JSON: `agent_id`, `tool_name`, `tool_input` path, `last_assistant_message`, `stop_hook_active` |
| `policy::allowed(phase, crate_root, path) -> Verdict` | The path table; `Verdict::Refused(reason)` carries the discipline row |
| `policy::slice_crate(project, slice) -> PathBuf` | The package whose manifest dir holds `docs/intent/<slice>/lld.md` |
| `Tally`, `tally::record(agent_id, kind)`, `tally::trailers` | Counts per agent under `<target>/lid-rs/agents/`; rendered as commit trailers |
| `hook_pre_tool(phase, input)` | Policy verdict for editing tools, tally for every tool |
| `hook_post_edit(project, input)` | Clippy, rendered as `additionalContext` |
| `hook_stop(project, phase, input) -> StopDecision` | Parse the message; `commit` → integrity → check → integrity → stage → commit → allow; `stop` → allow; else refuse |
| `integrity::synced_artifacts_match(project)` | `sync::check`, as a refusal reason |
| `integrity::outside_policy_clean(project, phase, crate_root)` | `git status --porcelain` filtered against the allowed set; anything else is named |
| `ExecutionClass::{Ordinary, CompileTime(reason)}`, `execution_class(project, slice)` | From `cargo metadata` target kinds: `proc-macro`, `custom-build` |
| `Ending::{Commit(message), Stop(decisions)}`, `ending_of(message)` | The stop protocol, parsed from the final message |
| `refusal_for(step_output) -> String` | Output + `gates.md` row for the check that fired + what the phase permits |
| `check_of_lint(name) -> Option<Check>` | The lint → check mapping |
| `stage_and_commit(project, paths, message, trailers)` | `git add -- <paths>`; `git commit -F` |
| `sync::artifacts()` | The mirror table: `skill/`, `workflow/`, `agent/` |
| `lid-rs/agent/lid-rs-phase-{2,3,4,5,7}.md`, `lid-rs-review.md` | The agent definitions: `tools:` and the three hooks with the phase literal |
| `lid-rs/workflow/lid-rs.js` | The unattended build |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Where the phase check attaches | The phase agent's `Stop` hook, which also makes the commit | A git `commit-msg` hook keyed on the `phase N:` tag (the first design of this slice, on PR #2); a `PreToolUse` hook on `git commit`; the agent, as before | A git hook gates a commit but cannot narrow what the committer may do, needs `core.hooksPath` registered per clone (which cost this slice two CI failures), and runs whatever binary the working tree resolves. Attaching to the agent lets the agent hold no git at all, makes the commit an effect of the check, and puts the policy in the same file as the tool list. Its cost is that only Claude Code subagents are gated; a human at a terminal is gated by CI. Spiked 2026-08-26/27: `SubagentStop` receives `last_assistant_message`, exit 2 or a block decision keeps the subagent running with the reason as its next turn, and hooks fire for Workflow-spawned agents (`agent_type: workflow-subagent`). |
| What the agent may run | Nothing: `tools:` has no Bash; compile feedback comes from the post-edit hook | An exact-string allowlist of `cargo check/clippy/test` through a `PreToolUse` hook; the LSP tool for diagnostics | The gate executes the agent's code anyway, so an allowlist adds no capability — but it adds a grammar to reason about and a command the agent chooses to run. A hook that runs clippy after every edit gives the same feedback with no choice involved, and the tally can prove `commands: 0`. The LSP tool has no diagnostics operation, and rust-analyzer's push reaches the main session only (spiked: a subagent saw nothing after a 10 s wait; the main session received the subagent's errors minutes later). |
| Per-edit check | `cargo clippy --all-targets -- -D warnings` | `cargo check` alone; clippy plus the slice's validations | Measured after a small edit here: check 0.19 s, clippy 0.29 s; clippy sees checks 3, 6, 7, 8, 9 that check does not. Running tests per edit is seconds per edit for feedback the stop provides once. |
| Policy enforcement point | `PreToolUse` on the agent, with reasons quoted from `discipline.md` | Prose in the phase files (the 0.2.1 arrangement); a post-hoc diff check at the stop | A rule in prose is dropped exactly when it is inconvenient (skill LLD, evidence table); a diff check at the stop lets the agent spend a phase on work it must then discard. Refusing at the call is immediate, and quoting the discipline row keeps one source of truth for the rule's wording. |
| Confused-deputy scope | Writes are bounded; reads are not | Also restrict what the agent may read | The boundary is about what an instruction — from the prompt or from a file — can make the agent *do*; hiding files would make the reviewer's cold reading impossible and gains nothing once writes are bounded. |
| Runtime tampering | Detected at the stop (synced artifacts and everything outside the policy must be unchanged) and refused; prevented only by isolation | Sandbox every check from the hook (`bwrap`, `sandbox-exec`); ignore it | Detection is cheap, deterministic, and names the event; a sandbox is a control of its own with platform rules, deferred rather than implied. Ignoring it would let a Phase 5 test rewrite the policy the next session loads. |
| Compile-time slices | Disclosed from `cargo metadata`; the unattended mode requires the human's per-run acceptance | Refuse them outright; treat them like any slice | The tool's own `lid-rs-macros` is such a crate and must be workable; the human, not the workflow, decides to run compile-time code unattended. |
| The stop protocol | Fenced ```` ```commit ```` or ```` ```stop ```` in the final message | Structured output only; a marker line; the hook reading the transcript | `last_assistant_message` is what the hook receives; a fenced block is unambiguous to parse and to write, and the refusal teaches the format when it is missing. Whether the final message survives a workflow `schema` is verified at Phase 3 of this slice; if not, the workflow's worker returns plain text and the script parses it. |
| Staging | Exactly the policy's allowed paths | `git add -A`; the agent names files | The set that bounds edits bounds the commit; anything else the agent could not have written. |
| Stop-refusal budget | Refuse while the check fails, up to Claude Code's cap of eight | One refusal then allow (the first design); refuse forever | A failing check is not a reason to let the phase end; eight rounds of clippy output is more than a fixable phase needs, and the cap leaves a dirty, uncommitted tree the next precondition refuses. A `stop` block is always allowed, so an honest stop is never blocked. |
| Trusted binary in the tool's own workspace | Hooks name the installed `cargo-lid-rs` directly, refreshed from `main` after merge; no synced script | A synced `hooks/run` script preferring `cargo run -p cargo-lid-rs` here (the first design); a separate worktree build | A worker in this repository edits the hook's own source; running it from the tree means the policy is whatever the worker last wrote. Enforcing only landed policy is the price of the tool being its own consumer. |
| Instrumentation | A per-agent tally kept by the hooks, written as commit trailers | Parse `agent_transcript_path`; no instrumentation until the design settles | The hooks see every call and refusal; the transcript format is undocumented. Trailers put the measurement where the review already happens, from the first phase this design runs. |
| Phase 7's commit subject | `phase 7: <version>: <what and why>` | The project's convention alone | The tag is what makes the stop hook run the full gate; the changelog-readable part follows it. |
| Phase 7 gate duration in a hook | `timeout` set in the agent's frontmatter to cover a mutation run | Move mutants to CI only | A gate that exists, gates; the hook's timeout ceiling is verified at Phase 3, and mutants moves to CI only if the harness caps below what a slice needs. |
| Worktree per worker | Deferred | The Workflow's `isolation: "worktree"` per phase agent | Which branch a temporary worktree checks out is undocumented; a commit there must land on `lld/<slice>`. The dirty-tree precondition covers the failure the worktree would have contained. |
| Phase 5 test execution | One `cargo test … --exact` run per validation, exit status as verdict | One `cargo test --lib` run with libtest output parsed; `--format json` | One process per test costs seconds on a slice-sized set and needs no parsing of libtest's human-oriented output; JSON output is nightly-only. |
| Phase 5 slice identity | `SPEC` records by source file `src/spec/<slice>.rs`, slice from the branch name | Parse `src/spec/` for the module; a `--claims` list | The registry already carries the file; the branch convention already carries the slice; constraint 2 forbids the parse. |
| Phase 7's list | The tool holds README §4.5 verbatim, in order, as one more copy the README's rule binds | Make `cargo lid-rs gate` canonical and reduce the README to a pointer | Keeping the list canonical in prose is deliberate for now: the spec stays readable without the tool. Promoting the tool is a README change with its own slice. |
| The workflow's input | A branch with a human-approved `phase 1:` commit; no waiver argument | A slice name, with the workflow drafting the LLD; a `--waive` argument | Phase 1 is human-owned; a workflow that drafts it and continues has approved its own LLD. A waiver given once is reused; an argument is a waiver given every time. |
| Reviewer at each stop | One clean agent per phase, prompted to refute, one rework round | No reviewer; a judge panel per phase | A clean reviewer is also the test that the artifact is context-free — the failure interactive mode cannot see. A panel exceeds the cost a slice warrants; one rework round bounds the run. |
| Where the artifacts live | `agent/` and `workflow/` beside `skill/` in the `lid-rs` crate, synced under one rule | Inside `skill/`; a separate crate; the plugin | Claude Code reads agents and workflows from `.claude/agents/` and `.claude/workflows/`; the files are version-coupled to the skill they point at, so they ship with it. |

## Open Questions & Future Decisions

### Deferred
1. Workspace-appended gate steps (`mdbook build book` here): a
   `[workspace.metadata.lid_rs] gate_extra` list `phase-check 7` would run
   after the floor. Until then those steps live in CI only.
2. Worktree isolation per phase worker (see Decisions).
3. The workflow's Phase 8 path: an edited LLD on an existing slice's
   branch re-enters at Phase 2 with renamed claims; the precondition's
   "first phase without a commit" reading needs the `-<what-changed>`
   branch convention settled first.
4. A documentation phase: the cascade a slice causes in README, CLAUDE.md,
   and the skill is no phase agent's to make under the policy; today it is
   the human's, or the main session's outside a LID phase.
5. `rust-analyzer` in `rust-toolchain.toml`'s components, so the LSP tool
   works for the reviewer without a manual install.
6. Running each check under an OS sandbox from the hook — no network,
   writes confined to `target/` — so the residue in Security posture is
   bounded by the tool rather than by the environment it is run in.

## References

- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate this tool runs at Phase 7), [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the phases and who owns each), constraint 2 (no source parsing).
- `docs/intent/skill/lld.md` (workspace) — the working-state convention, the evidence table that says why the agent must not run its own checks, and the interactive mode this slice changes.
- `docs/intent/sync/lld.md` — the strict mirror rule this slice extends to the agents, hooks, and workflow.
- `docs/intent/cargo-lid-rs/lld.md` — the registry dump and the item-path mapping `check_red` reuses.
- Claude Code hooks, subagents, and workflows (`code.claude.com/docs/en/hooks`, `/sub-agents`, `/workflows`) — hook input fields, block semantics, `additionalContext`, frontmatter hook syntax, and the `.claude/agents` and `.claude/workflows` locations.
