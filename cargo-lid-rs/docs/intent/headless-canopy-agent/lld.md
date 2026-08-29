# headless-canopy-agent — a slice is built unattended by phase sessions on canopy

## Context and Design Philosophy

The phase slice (`docs/intent/phase/lld.md`) settled what an unattended
phase is: an agent that can only read and edit, a path policy in front of
every edit, the compiler's verdict after every edit, and a stop whose
commit is the phase's check passing. It hosted that design on Claude Code:
the agent is a subagent, the policy and the check are hook events, and the
sequence of phases is a workflow script. That host has two costs the
design did not choose. A subagent that makes no stream progress for 600 s
is killed, so the one phase whose check takes longer than that — Phase 7,
whose mutation run takes about 17 minutes here — cannot be committed by
the hook that exists to commit it (phase LLD, Deferred 7). And every
deterministic step between two agent turns is reached through the host's
event vocabulary: a JSON document on stdin, a decision on stdout, a
workflow script that cannot run `cargo` or `git` and so asks a read-only
agent to report whether a branch has a `phase 1:` commit.

This slice hosts the same design on canopy instead. Canopy is a governed
agent platform whose unit is a **session**: one append-only log that every
participant — the client, the model, the platform's own services — reads
from and appends to, and nothing else. The client opens a session with a
system prompt and a policy naming the tools the model may call; the
platform runs inference; the model's tool calls come back to the client
as records, authorized in the cloud against that policy; the client
executes them and appends the results; the model's final text is a record
too. There is no harness in the loop and no watchdog on the client: the
client is an ordinary program that runs a phase's check for as long as the
check takes, between two records of a log it holds open.

What this buys is the separation the phase design asked for, with less
machinery. **Several agent sessions with deterministic code between
them**: each phase is its own session — a fresh context, a system prompt
that is the phase agent's body, a policy admitting five tools — and
everything between one session's last record and the next session's first
is this program: the check, the integrity verification, the staging, the
commit, the reviewer's verdict, the decision to rework or stop. **Only a
few tools, and we define them**: `read`, `grep`, `glob`, `edit`, `write`
exist as functions in this crate, so the path policy is not an
interception in front of a tool but the `edit` and `write` functions
themselves refusing, and clippy's output is not "additional context"
attached by an event but what `edit` and `write` return. The policy is
enforced twice — by the tool's code here, and by canopy's authorizer,
which refuses any call the session's policy does not admit and lands the
refusal as a record. The tally the hooks kept in a file is, on this host,
derivable from the sealed log: the phase commit names its session, and the
session is the audit.

One thing canopy takes away must be said plainly. On Claude Code the
phase design left reads unbounded because nothing the agent read could
leave: it had no network and no command. On canopy **every tool result is
a record on a log the platform stores**; a read is a transmission. So this
host confines reads to the repository, and its Security posture section
says what the log then holds.

This is a proof of concept. It builds on the phase slice's branch, reuses
its library, and touches nothing the interactive mode uses; the Claude
Code agents and the `lid-rs` workflow are untouched. What it exists to
prove is that the phase design survives the change of host — same tags,
same checks, same commit bodies, same terminal states — and that Phase 7
commits under it.

## Behaviour

### `cargo lid-rs canopy [--slice <name>] [--door <url>] [--max-cost <amount>]`

Builds one slice unattended from its approved LLD: Phases 2–7, in order,
one canopy session per phase for the worker and one per phase for the
reviewer, every check and every commit made by this program. The slice
defaults to the current branch's name with the `lld/` prefix removed (the
skill's branch convention, as `phase-check` reads it); `--door` defaults
to canopy's production door, `https://api.canopyhq.dev`. The API key is read from `CANOPY_KEY` and
never appears on a command line.

It prints, per phase, the session ids it opened and either the commit it
made or the decisions that stopped it; its last line is the terminal
state, and its exit status is 0 for *PR-ready* and 1 for *stopped*.

### What the human sets up once

The client brings no provider credential and no model: those live in the
canopy **config** the API key is bound to, which the human creates in the
console (or through canopy's `/configs/<name>` route) before the first
run. The client dials each session with four settings — `system`,
`policy`, `params` (empty: the model's defaults), and `max_cost` (from
`--max-cost`, in the provider's currency, default 5) — so the config must
leave those four overridable or `must_set`; a config that pins one
refuses the dial with the setting's name, and the run stops there. Every
other budget (`max_requests`, `context_limit`) is the config's, the
human's bound on a phase. The key needs the faces `converse`, `execute`,
and `stop`: to land the worker's prompt, to answer the model's tool calls,
and to end the session. A config that refuses the dial ends the run at the
precondition with the door's own sentence.

### The precondition

Before any session opens, this program reads the branch as the workflow's
precondition agent did, but with git rather than a model: the branch is
`lld/<slice>` and checked out; its history holds a `phase 1:` commit; the
tree is clean; the phases already committed are those whose subject tags
`phase-check` recognises (`tag_of`); and if the slice's crate is a
compile-time slice (`execution_class`) the human's acceptance file is
present (`compile_time_accepted`). Any of those failing is the run's first
decision, in the phase LLD's words. The run then starts at the first phase
without a commit; a phase already committed is skipped and said so.

### A phase is a session

For each of Phases 2, 3, 4, 5, and 7 (Phase 7's session also does Phase 6,
which has no commit of its own), the client opens a session whose
`settings` carry the two budgets above and two things of the phase's own:

- `system`: the phase agent's body — the synced
  `.claude/agents/lid-rs-phase-<n>.md` with its frontmatter removed, the
  same text the Claude Code host gives the same phase. Every rule in it
  the tools below also enforce.
- `policy`: an inline policy document admitting the five tools for the
  requestee `lid-rs`, each declared with the JSON schema of its arguments
  and its description in that schema's `description`; `allows` lists the
  same five `(requestee, op)` pairs and nothing else. The door lands the
  resulting `app.policy.configured` — hash, version, principal — as the
  session's first records; the client never lands a policy record itself,
  so a session's tool surface is fixed at the dial.

Then it lands one `app.client.user_message`: the worker prompt — the
slice, the branch, the LLD's path, the branch's `git log --oneline`, the
commit subject this phase must use, and, on a rework, the reviewer's
findings — and drives the turn (below). The worker's final text is the
phase's ending. Every session is stopped when its phase ends, whichever
way it ends; a stopped session is a sealed log.

### The five tools

Each is a function in this crate; the model sees its name, schema, and
description through the policy, and calls it through canopy's invoke
choreography. Every path argument is relative to the workspace root, and
a path that resolves outside the workspace — by `..`, by being absolute,
or through a symlink — is refused by every tool, including `read`: on this
host a read is a transmission (Security posture). What the phase's policy
says about a path is asked afterwards, through the phase library.

| Tool | Arguments | Does | Returns |
|---|---|---|---|
| `read` | `path`, optional `offset`, `limit` (lines) | A file's text, or a directory's entries | The text with line numbers, or the listing |
| `grep` | `pattern`, optional `path` (default the root), `glob` | A literal, case-sensitive substring search over files under `path` | `path:line: text` per match, capped at 200 lines |
| `glob` | `pattern` | Files matching a glob under the root | The paths, sorted |
| `edit` | `path`, `old_string`, `new_string`, optional `replace_all` | Replaces one exact occurrence (or all) in an existing file; a string that matches zero or more than one place is an error naming the count | Clippy's verdict after the edit |
| `write` | `path`, `content` | Creates or replaces a file whole | Clippy's verdict after the write |

The three observation tools ask the phase library's pre-tool verdict with
their name (`Read`, `Grep`, `Glob`) — which never refuses an observation
and tallies it — then do their work. The two editing tools ask it with
`Edit` or `Write` and the target path: a refusal is returned to the model
as the tool's error, wording and all (the discipline row, then what the
phase may do instead), and the file is untouched; an allowed edit is
made, then the post-edit verdict — `cargo clippy --all-targets -- -D
warnings` over the workspace, "clean" or the diagnostics — is the tool's
result. A phase session therefore has, per edit, exactly the feedback the
Claude Code host gave it, arriving as the tool's return value rather than
as context appended to it.

The reviewer's session declares the three observation tools only. Its
policy admits nothing else, so an edit it might be talked into is refused
in the cloud before this program sees it — and if one arrives anyway it is
refused here too.

### Driving a turn

The client holds one session open at a time and has at most one user
message outstanding, which is what makes the log's order sufficient: it
follows the tail by parked reads (`wait` of 25 s, the door's ceiling,
clamped to the credential's remaining life), and acts on records by kind.

- `app.invoke.payload` (`to`, `args`): held until its forward arrives.
- `app.invoke.forward` (`to`, `op`, `payload_digest`): paired with the
  first held payload whose digest — `sha256` of the canonical JSON of
  `{"to", "args"}`, keys in byte order — equals the forward's, and whose
  `to` and the forward's `to` are both `lid-rs`. A forward addressed to
  another principal is not this program's to answer. The paired tool runs;
  its result lands as `app.invoke.completed` with the forward's digest,
  `to` set to the payload record's producer, `outcome` `success` with the
  result or `error` with the message; the idempotency key is
  `:<forward cursor>:65534:0` so a retried append lands once.
- `app.invoke.denied`: the authorizer refused a call the policy does not
  admit. Nothing to execute; it is counted as a refusal and the requestor
  tells the model.
- `inference.responded`: with `tool_uses`, the model has asked for tools
  and the forwards follow; without, the turn is settled and
  `response.text` is the model's final message. A body carrying
  `terminal` is a turn that failed at the provider: the user message is
  landed once more, and a second terminal stops the run naming the
  provider's sentence.
- `app.session.halted`: the session is over — a budget the config set was
  reached, or a stall the platform detected — and the run stops with the
  halt's reason as the decision.

A session whose tail delivers nothing for fifteen minutes (canopy's own
invoke-stall window) is stopped by the client and the run ends the same
way. When the credential is within five seconds of its expiry the client
refreshes it with the API key before the next read; the session and its
cursor are unchanged.

### The door, on the wire

Everything the client says to a door is one of six requests; everything it
hears is JSON. A refusal is `{"refused": "<one sentence>"}` with the
status canopy assigns (401 no credential, 403 the credential's, 409 the
session has stopped, 429 shed with `Retry-After` in seconds), and that
sentence is what the client reports.

| Request | Bearer | Body | Answer |
|---|---|---|---|
| `POST /sessions` | the API key | `{"settings": {"system", "policy": {"allows": [{"requestee", "op"}], "tools": [{"name", "requestee", "op", "schema"}]}, "params": {}, "max_cost"}}` | `{"session", "token", "expires", "stream", "audience"}`; `expires` in seconds since the epoch |
| `POST /sessions/{id}/events` | the session token | `{"kind", "body"}`; header `Idempotency-Key` optional | `{"cursor"}` |
| `GET /sessions/{id}/tail?after=&wait=&envelope=true` | the session token | — | `{"records": [{"cursor", "kind", "producer", "body", "envelope": {"stream", "version", "idem", "size"}}], "through"}`; `wait` is held at most 25 s and an empty page is `200` with `through` unchanged; page on `through` |
| `POST /sessions/{id}/refresh` | the API key | — | as `POST /sessions`: a new token for the same session |
| `POST /sessions/{id}/stop` | the session token | — | `{"cursor"}` |

The tool's description is the `schema`'s own `description`; a tool is
shown to the model only if `allows` also names its `(requestee, op)`. The
door itself lands `app.policy.configured` (with `version`, `hash`,
`principal_digest`) as the session's first readable record, producer
`exchange`.

The records the client acts on, and their bodies:

| Kind | Producer | Body |
|---|---|---|
| `app.client.user_message` | the client | `{"text"}` |
| `inference.requested` | `requestor` | `{"contextContainsUpTo", "params"}` — informational |
| `inference.responded` | `inference.dispatch` | `{"echo": {"for"}, "raw", "response": {"type": "canonical", "text", "tool_uses": [{"name", "requestee", "op", "projections", "args"}], "stop_reason"}, "usage"}`, or `{"echo", "raw", "terminal": "<sentence>"}` with no `response` |
| `app.invoke.payload` | `requestor` | `{"to", "args"}` — `to` is the requestee |
| `app.invoke.forward` | `authorizer` | `{"to", "op", "payload_digest"}` |
| `app.invoke.denied` | `authorizer` | `{"to", "payload_digest"}` — `to` is the requestor |
| `app.invoke.completed` | the client | `{"to", "payload_digest", "outcome": "success", "result"}` or `{…, "outcome": "error", "error"}`; `to` is the payload record's producer |
| `app.session.halted` | the platform | `{"reason"}` |

`payload_digest` is the hex SHA-256 of the canonical JSON
`{"to":<to>,"args":<args>}` — object keys in byte order, numbers written
as `f64` — and canopy pins one vector: the payload with `to` `"executor"`
and `args` `{"A":{},"a":2,"at":1756339200123456800,"b":1,"tie":75251554695404.12,"é":[3,"x"]}`
digests to
`0d0f02537b19d987f967216664c91f7a92dc22364cd4c840d825fbaac5448f10`.

### The ending

The settled turn's text is handed to the phase library's stop verdict,
exactly as the Claude Code host hands it the agent's final message, with
the session's id as the agent id:

- a ```` ```commit ```` block whose subject tags this phase: the phase's
  check runs — `phase-check <n>` in a fresh process, for as long as it
  takes — with the integrity checks around it; on success the phase's
  allowed paths are staged and committed with the tally's trailers, and
  the phase is done. On failure the verdict's reason — the check's output,
  the skill's row for the check that fired, what this phase permits — is
  landed as the next user message in the same session, and the turn is
  driven again; the ninth consecutive refusal ends the run with the
  reason, the tree dirty and uncommitted, as the harness's cap would have.
- a ```` ```stop ```` block: the numbered decisions end the run,
  uncommitted, with those decisions.
- neither, or both: the format is landed as the next user message.

### The reviewer

A committed phase is reviewed before the next opens. A fresh session,
`system` the synced `lid-rs-review.md` body, the three observation tools,
one user message naming the phase, the slice, the commit, the LLD, and
the skill's files for that phase; prompted to refute, as the workflow
prompted it. Its final text must carry one ```` ```review ```` block:

```text
approved: no
1. <finding>
2. <finding>
```

`approved: yes` opens the next phase. `approved: no` with findings opens
one rework session of the phase's worker with the findings in its prompt;
its commit is a second `phase <n>:` commit on the branch — nothing amends
history, since only a passing check commits — and it is reviewed the same
way. A second rejection ends the run with the findings as the decisions.
An unset `CANOPY_KEY` is a precondition stop naming the variable. A
block that is missing or malformed is asked for once more with the
format; a second miss is a rejection whose finding says the reviewer gave
no verdict.

### Terminal states

*PR-ready*: every phase committed and the gate passed; the output ends
with the branch and every decision the phases recorded. *Stopped*: at
the precondition, a phase, or its review — with the decisions.
There is no third state and no waiver.

### What the commit body carries

The trailers the phase library writes, plus the agent's id, which on this
host is the session:

```text
Lid-Rs-Phase: 3
Lid-Rs-Agent: canopy:3f0c1c9a-6f6e-4f0b-9a15-2b6a3f1c77de
Lid-Rs-Tools: 14 edits, 9 observations, 0 commands
Lid-Rs-Checks: 14 post-edit, 1 stop
Lid-Rs-Refusals: 1 policy, 0 stop
```

The session id is the join to the sealed log: every read, every edit,
every refusal, and the model's every word, in order, held by the platform.

## Security posture

The confused-deputy boundary the phase slice drew holds here and is
stronger in one respect: a tool call is refused by this program's code
*and* by canopy's authorizer, which sees only the policy the session
opened with — a policy no model turn can widen, because the client lands
no `app.policy.configured` after the dial and the key's faces do not
include `configure`. The tally proves `commands: 0` as before; the log
proves it independently.

What changes is confidentiality, and it changes materially:

- **Reads leave.** Every tool result is a record the platform stores in
  the session's log and seals when the session stops. The slice's source,
  the LLD, the skill files, and anything else the model reads are then
  held by canopy for as long as it keeps sealed logs, under the tenant's
  account. This is why reads are confined to the workspace: what can be
  transmitted is what the repository already contains, nothing from the
  environment around it. Run this client only in a repository whose
  contents may be sent to that tenant.
- **The API key is a credential the process holds** and presents on every
  dial and refresh; it is not in the log, not on the command line, and
  not written anywhere by this program. It is bound to a config the human
  owns and to faces that cannot configure.
- **Execution residue is unchanged.** The check executes the model's
  code at the Phase 5 and 7 stops, and after every edit for a compile-time
  slice, with this process's privileges; the phase slice's Security
  posture applies verbatim, including its environment-isolation demand.
  What is new is that a test the model wrote at Phase 5 can now read the
  API key from the environment. Run unattended only where untrusted code
  may run, with no credential the run does not need — and this run needs
  one.

## Shape

| Item | Role |
|---|---|
| `headless_canopy_agent::run(args)` | `canopy` entry: parses `--slice`, `--door`, `--max-cost`; reads `CANOPY_KEY`; precondition, then the phases; prints the terminal state |
| `Precondition`, `precondition(project, slice) -> Result<Precondition, Stop>` | Branch, `phase 1:` commit, clean tree, committed phases (`tag_of`), compile-time acceptance; from git and the phase library, no model |
| `Outcome::{PrReady{decisions}, Stopped(Stop)}`, `Stop { at: At, decisions }`, `At::{Precondition, Phase(n), Review(n)}` | The two terminal states, and where a run stopped |
| `build(project, door, state, max_cost) -> Outcome` | Phases 2, 3, 4, 5, 7 in order; skips committed ones; one flow decision per phase result |
| `Door` | The HTTP client over one door's URL and the API key, stateless: `start(settings) -> Started`, `send(credential, offered, idem) -> cursor`, `tail(credential, after, wait) -> Page`, `refresh(credential) -> Started`, `stop(credential)`; every refusal carries the door's sentence |
| `Started`, `Page`, `Record`, `Offered` | The boundary types over the door's JSON: `session`, `token`, `expires`, `stream`; `records`, `through`; a landed record's `cursor`, `kind`, `producer`, `body`, `envelope`; and what the client offers, `kind` and `body` alone |
| `Door::request(method, path, bearer, body, idem) -> Result<Value, String>`, `refusal_of(status, body) -> String` | The one boundary over the HTTP library every method shares: a `2xx` yields the JSON; anything else yields the door's `refused` sentence (or the status when there is none) as the error |
| `Settings`, `policy_for(tools) -> Policy` | The dial: `system`, an inline policy of `allows` and `tools`, empty `params`, and `max_cost` |
| `Tool::{Read, Grep, Glob, Edit, Write}`, `Tool::of(op) -> Option<Tool>` (its own claim: the five names map to their tools, any other `op` to none), `Tool::hook_name(self) -> &str`, `ReadArgs` … `WriteArgs`, `ToolResult`, `declarations(tools) -> Vec<ToolDecl>` | The closed set; the forward's `op` classified into it (an unknown `op` is none); the name the phase library's verdict knows the tool by (`Read`, `Grep`, `Glob`, `Edit`, `Write`); each tool's typed arguments (the boundary over the payload's `args`); their JSON schemas with descriptions |
| `Session`, `Session::open(door, settings, phase, tools) -> Result<Session, String>` | One open session: the door, the `Started` credential, the phase, the tools its policy declared, the cursor, held payloads, the tally key `canopy:<id>`; `open` dials and prints `opened_line(id)`, so every session a phase opens is printed as it opens and `build` prints only the ending — and, for a phase already committed, `skipped_line(phase)`; the two rendering leaves are what a test observes |
| `Payload { to, args }`, `Forward { to, op, payload_digest }`, `Responded { text, tool_uses, terminal }`, `Denied`, `Halted { reason }` | The record bodies the client acts on, decoded from `Record.body` once, by kind, where `drive` classifies the record; nothing past that point indexes JSON |
| `drive(project, session, message) -> Result<Settled, Halt>` | Lands a user message and follows the tail until the turn settles: one `match` over record kinds |
| `Settled { text }`, `Halt::{Halted(reason), Terminal(sentence), Quiet, Refused(sentence)}` | A settled turn, or why the session ended without one: the platform's halt, the provider's second terminal, a quiet tail, the door's refusal |
| `pair(held: &[Held], forward: &Forward) -> Option<Pairing>`, `Held { cursor, producer, payload }` | The first held payload whose digest equals the forward's and whose addressee is this program |
| `payload_digest(to, args) -> String` | `sha256` of canopy's canonical JSON; pinned to canopy's published vector |
| `canonical_json(value) -> String` | Keys in byte order, numbers as `f64` |
| `execute(project, session, op, args) -> ToolResult` | Dispatch over `Tool`: the pre-tool verdict, the work, the post-edit verdict for edits |
| `confine(root, path) -> Result<PathBuf, String>` | The workspace boundary every tool applies first |
| `read_tool`, `grep_tool`, `glob_tool`, `edit_tool`, `write_tool` | The work of each tool, over a confined path |
| `completion(forward_cursor, pairing, result) -> (Offered, String)` | The `app.invoke.completed` record to offer and its idempotency key |
| `worker_session(project, phase, state, findings, max_cost) -> Result<(Settings, String), String>` | The phase agent's body as `system`, the five tools, the worker prompt |
| `worker(project, door, phase, state, findings, max_cost) -> Result<WorkerEnd, String>` | Drives the worker: turn → stop verdict → commit, refusal round, or stop block; ends the session |
| `WorkerEnd::{Committed(hash, decisions), Decisions(Vec<String>), Refused(reason)}` | How a worker session ended |
| `review_session(project, phase, state, commit, max_cost) -> Result<(Settings, String), String>` | The reviewer's body, the three observation tools, the review prompt |
| `review(project, door, phase, state, commit, max_cost) -> Result<Review, String>` | Drives the reviewer; parses the block; one re-ask |
| `Review::{Approved, Rejected(findings)}`, `review_of(text) -> Result<Review, String>` | The review block, parsed |
| `agent_body(project, name) -> Result<String, String>` | A synced agent file with its frontmatter removed |
| `phase::hook_pre_tool`, `hook_post_edit`, `hook_stop` (phase slice, made `pub`) | The three verdicts this host calls with a `HookInput` it builds; `agent_id` is `canopy:<session>` |
| `tally::trailers(tally, phase, agent_id)` (phase slice, gains the agent id) | Renders `Lid-Rs-Agent` between the phase and the tools |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| Language and home | Rust, a `cargo lid-rs canopy` subcommand in this crate | TypeScript on `@canopy/sdk` (bun); a TS shell calling `cargo lid-rs` for every deterministic step; a separate unpublished crate depending on `cargo-lid-rs` | The deterministic code between sessions already exists in this crate as the phase library and stays one implementation; the door is about ten routes with a runtime-served OpenAPI document, and canopy's own reference client is 122 lines of plain `fetch`; a second language adds a toolchain to the gate for a POC. A sibling crate would need the phase module's private paths made public more widely than three functions; the subcommand needs only those. The cost is four dependencies in a published crate, acceptable for a proof of concept that may not merge. |
| Reuse of the phase design | The three hook verdicts, called directly with a `HookInput` this program builds | Reimplement the policy, clippy, check, integrity, and commit path here; shell out to `cargo lid-rs hook …` with hand-made JSON | One policy, one tally, one stop protocol, one commit path — the phase LLD's rules, not a copy of them; the hook functions already take plain values past the JSON boundary. Shelling out would re-encode canopy records as Claude Code events to decode them again. The cascade is three `fn` → `pub fn` and one trailer. |
| HTTP dependency | `ureq` 3, blocking, with its `json` feature; TLS by its default | `reqwest` with `tokio`; hand-written HTTP/1.1 over `std::net` | The loop is sequential — one session, one parked read at a time — so an async runtime buys nothing (tenet 3). A hand-written client would have to do TLS to reach the production door. |
| Other dependencies | `serde` (derive) for the boundary types, `sha2` for the payload digest, `glob` for the `glob` tool | `serde_json::Value` throughout; a hand-rolled SHA-256; a hand-rolled glob matcher | Typed boundary types keep the door's JSON at the edge, as `HookInput` does for the hooks; SHA-256 is not something to write for a POC; `glob` has no dependencies of its own. |
| Where the policy is enforced | In the tool functions here, and by canopy's authorizer through the session's inline policy | Only here; only in the cloud | Here is where the phase library's verdict and wording live; the cloud is a second, independent boundary that a model turn cannot widen, and its refusals are records. Both, because they refuse different things: the path policy needs the repository, the authorizer needs only the policy. |
| Reads | Confined to the workspace root by every tool | Unbounded, as the phase slice leaves them | On this host a read is transmitted to and stored by the platform; confinement bounds what can leave to what the repository holds. The phase slice's "reads are never refused" is a statement about the hook's verdict, which this host still gets — the confinement happens before the verdict is asked, in the tool. |
| Tool set | `read`, `grep`, `glob`, `edit`, `write` | `read`, `write`, `edit` alone, with `read` listing directories; the Claude Code agents' set including LSP | Parity with the phase agents' definitions minus LSP, whose only operations (hover, definition, references) the reviewer can do by reading. Three tools would make search a sequence of reads, each of which is a record. |
| `grep` semantics | Literal, case-sensitive substring; 200-line cap | A regex engine | A dependency for a POC's search tool; the model can read the file when the literal finds it. Revisit if the reviewer's findings show it hunting. |
| Turn settlement | `inference.responded` without `tool_uses`, with one user message outstanding | The idempotency-chain correlation canopy's load driver uses | Canopy lands no "awaiting user" record; the driver's correlation exists because it runs many messages concurrently. One outstanding message per session makes the log's order the correlation. |
| Pairing a forward with its payload | By `payload_digest` computed here, plus the addressee check on both records | By adjacency on the tail; skipping the addressee check as the TS SDK does | The digest is the only correlation canopy defines; the SDK's missing addressee check is a documented defect that lands duplicate completions when another executor serves the session. |
| A failed check | The reason landed as the next user message in the same session, up to eight refusals | A fresh session per attempt; refuse forever | The same session keeps the model's context of what it wrote, which is what the harness's "kept running with the reason" preserved; eight is the harness's cap and the phase LLD's budget. |
| The reviewer's verdict | A fenced ```` ```review ```` block in the final text | JSON in the final text; a structured-output tool the reviewer must call | Canopy forces no schema on a turn; a fenced block is the stop protocol's own convention, and a reviewer given one more tool has one more thing to be talked into. |
| The precondition | Deterministic: git and the phase library | A read-only session reading `.git` from files, as the workflow does | The workflow used a model because its script could not run git; this program can. A model reading packed refs to answer "is the tree clean" is the kind of step the design wanted out of the model's hands. |
| Budgets | `max_cost` per session from `--max-cost` and empty `params`; every other budget the config's | The config's alone, the client dialing `system` and `policy` only; every budget from the client | A config in use leaves `params` and `max_cost` `must_set`, so a dial without them is refused ("params needs a value"); the dial carries what the config demands and nothing else, and a config that pins one of the four refuses with its name. `max_requests` and `context_limit` stay the human's bound, as the acceptance file is for compile-time slices. |
| Provider silence | One retry of the user message, then stop naming the provider's sentence | Stop at once; retry without bound | One retry covers a transient; more hides a configuration problem the human must see. |
| Session lifetime | One per worker attempt and one per review; stopped at its end | One session for the whole slice; one per phase including its review | A fresh context per phase is what the phase design requires ("a clean reviewer is also the test that the artifact is context-free"); a stopped session is a sealed, complete audit of exactly one phase's attempt. |
| The commit's link to the log | `Lid-Rs-Agent: canopy:<session>` as a trailer, via the phase library's `trailers` | The session id in the tally file only; in the commit body's prose | The trailer is where the other measurements are; on the Claude Code host the same trailer carries the subagent's id, which links the commit to its tally file — useful there too. |
| Validation strategy | `#[validates]` tests against an in-process door that replays transcripts: the record sequences and digest vectors canopy pins in its own choreography goldens, and sessions recorded live once a key exists; one `#[ignore]` end-to-end run against a real door when `CANOPY_KEY` is set | Every test against a real door; canopy's in-process fixture via path dependencies on its crates | Canopy's dev door runs no requestor, so no real door short of a deployment lands `inference.responded`; path dependencies on an unpublished tree would put canopy's workspace in this crate's build. Canopy's goldens are its pinned wire records, so replaying them keeps the assertions grounded while the phase-5 red run stays deterministic and offline; the live run is the E2E proof, run by hand. |

## Cascade

Into the phase slice, as a Phase 8 edit whose LLD line is one sentence —
"the three hooks are hosted by Claude Code's hook events or by the canopy
client" — with these consequences: `hook_pre_tool`, `hook_post_edit`, and
`hook_stop` become `pub`; `tally::trailers` takes the agent id and writes
`Lid-Rs-Agent`; the phase LLD's "What the commit body carries" shows the
new line. No claim changes meaning.

Into the skill and the README: none for the POC. The interactive mode,
the workflow, and the agents are unchanged; a proven client would be a
sentence in the skill's working-state section and a row in README §8,
which is a documentation slice of its own (phase LLD, Deferred 4).

## Open Questions & Future Decisions

### Deferred
1. Whether a config with `policy` pinned (`set`) refuses an inline policy
   at the dial, or ignores it; the client assumes refusal and stops with
   the door's sentence. A config that leaves it `must_set` is verified.
2. The `grep` tool as literal substring; a regex engine if the reviewer's
   findings show the model hunting.
3. Reading the sealed log back as the tally — replacing the file the
   hooks keep with a fold over the session — once the client is proven;
   today the trailer names the session and the tally stays where the
   phase slice put it.
4. The documentation cascade (skill, README §8) if the POC is promoted.
5. A second executor on the session (canopy's MCP relay) is out of scope;
   the addressee check is what would make it safe.
6. For the phase slice: a claim added after the skeleton has no phase
   whose check it passes — Phase 2's lint runs over the whole crate, and
   a layer-0 skeleton's `todo!()` parameters are warnings — so a Phase 3
   review that asks for a claim is committed by hand today. Either Phase
   2's lint scopes to `src/spec/`, or the skeleton convention is
   lint-clean; the phase LLD's decision.

## References

- `docs/intent/phase/lld.md` — the design this hosts; its Security
  posture applies verbatim.
- `docs/intent/sync/lld.md` — the agent files this reads as system
  prompts are the synced copies.
- Canopy: `docs/dream-code.md` (the session as the only log), the door's
  routes in `crates/canopy-doors` (`/sessions`, `/events`, `/tail`,
  `/refresh`, `/stop`; `GET /openapi.json` when running), the invoke
  choreography and `payload_digest` in `crates/canopy-invoke` with its
  published golden vector, the policy hash in `crates/canopy-authorizer`,
  the settle rule in `crates/canopy-requestor`, and `tools/load/tools.ts`
  for the SDK's addressee defect.
- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html), [§12](https://bradvoth.github.io/lid-rs/spec/limits.html).
