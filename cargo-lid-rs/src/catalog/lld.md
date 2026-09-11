# A check is a command with a finding

## Context and Design Philosophy

Every check this project runs is a shell line somewhere, and the copies have
diverged. Six places state a gate: `CLAUDE.md`, README
[§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html),
`lid-rs/skill/references/phase-7.md` (byte-gated against README by
`sync --check`), `.github/workflows/gate.yml`, `phase::plan` in Rust, and the
`phase` slice's own claim `PhaseSevenRunsTheGateInOrder`, which states the step
list *as a claim*. Where they disagree nothing notices: `phase-check 7` runs
`cargo package -p <crate>` per crate, which cannot resolve a sibling at a
version not on crates.io, while `CLAUDE.md`'s single invocation can — and
because `gate()` orders `Package` before `SyncCheck` and `Mutants` and stops at
the first failure, **`sync --check` and check 12 have never run under the
hook**, on any slice.

This slice makes a check a **command with a report**: one fixed invocation, one
finding schema, `target/lid/<command>.json`, and a human rendering generated
from the findings rather than from the tool's output. It does **not** move the
gate's definition into metadata. That is the next change and it is the `phase`
slice's — see §"What this slice is not".

**What a finding buys.** A finding that is data rather than text is what lets
the pipeline adjudicate, what lets a PR body carry a check's output, and what
lets an agent be told which *rule* it broke rather than handed a compiler's
stderr. The hooks already do the last of these by hand, from `refusal_reason`
and the skill's discipline rows; the schema is that generalised and made
readable by something other than a human.

**One correction to the motivation, stated because the draft of this document
got it wrong.** Repairing the `package` form does not make check 12 run under
the hook. `checked()` shells out to `phase-check 7` inside the agent's stop
hook, and the pipeline document's §6.4 gives the binding constraint: the host's
600 s stream watchdog kills a worker blocked in its own stop, and `mutants`
costs seventeen minutes. A check longer than that interval runs *between*
sessions and never inside a worker's stop, whatever this slice does. What the
repair buys is `sync --check` — and an honest gate, run by hand, that reaches
its own last step.

## Behaviour

### What this slice is not

It is not the gate's definition. Moving `phase::plan`'s step lists into
`[workspace.metadata.lid_rs.pipeline]`, renaming `phase-check <n>` to
`gate --phase <n>`, and retiring the prose copies all belong to the **`phase`**
slice: its claims state the step lists, its code runs them, its hook invokes the
name, and the metadata would live in a manifest **no phase of any slice may
write** (`policy::allowed_paths` admits no `Cargo.toml`, and a validator refuses
it by name). A catalog phase cannot write `phase/mod.rs`, `phase/spec.rs`,
`main.rs`, `README.md` or the skill either.

So this slice builds the parts, and a later change on `lld/phase--gate-from-metadata`
composes them — with this slice as its precondition, the same shape slice 16
used for `own_crate` and for its own four-commit delegation. Building both in one
document is what made the first draft unbuildable: three of `phase::plan`'s ten
steps (`LldChecks`, `Red`, `SyncCheck`) had no command in it, so the rename it
promised could not be performed from the parts it built.

### One command, one invocation, one report

`cargo lid-rs <command>`, where `<command>` is atomic. Each of the four
properties below is one rule:

- **One fixed underlying invocation.** No argument passthrough. An agent cannot
  widen `cargo test --lib`; a command's flags are enumerated by that command.
- **A report at `target/lid/<command>.json`**, in the finding schema, written
  always — including the empty list on a pass, so that an empty report and an
  absent one are distinguishable.
- **A human rendering on stdout generated from the findings**, never from the
  tool's output, so that the two cannot disagree.
- **Exit 0 with no findings, 1 with findings, 2 when the tool could not run.**
  1 is the project's answer; 2 is the harness's problem, and a pipeline retries
  one and adjudicates the other.

### The commands, and where their findings come from

The provenance question decides which are buildable, and it splits them:

| Command | Underlying | Findings from |
|---|---|---|
| `check` | `cargo check --all-targets --message-format json` | cargo's JSON diagnostics, mapped |
| `lint` | `cargo clippy --all-targets --message-format json -- -D warnings` | the same |
| `doc` | `cargo doc --no-deps --document-private-items`, `RUSTDOCFLAGS=-D rustdoc::broken_intra_doc_links` | **stderr, parsed** — rustdoc has no JSON diagnostic stream |
| `package` | `cargo package -p <each publishing member>` in **one** invocation | stderr, parsed |
| `sync` | the existing `sync --check` | **one finding carrying the comparison's message** — see below |

`--document-private-items` is pipeline §5.1's form and is kept: it is what makes
`clippy::missing_docs_in_private_items` (check 3) and rustdoc agree about what
is documented.

**`sync`'s comparison is not structured, and this document said it was.** The
first draft of this table wrote "its own comparison, already structured". It is
not: `sync::check` returns `Result<(), String>` whose error is the differences
joined into one English blob, and `artifact_differences`,
`describe_differences` and `describe_one_difference` are all private to the
`sync` module — which no phase of this slice may write. A finding per
difference therefore needs either a public structured accessor there, which is
the `sync` slice's change, or a re-parse of that blob, which is the scrape this
slice's own rendering rule forbids and which it used to justify deferring
`examples` and `suite`.

So `sync` runs here and yields **one** finding carrying the comparison's
message whole. That is honest about the granularity and buys what this slice's
motivation actually claimed — a gate step that reaches `sync --check` at all.
One finding per difference is Deferred 7, and it is the `sync` slice's.

**`examples` and `suite` are deferred, and the reason is not effort.**
`cargo test --doc` and `cargo test --lib` emit no machine-readable output on
stable. A command whose findings can only come from scraping human test output
would violate this slice's own rendering rule, and the alternative — libtest's
unstable JSON — is a nightly dependency this project's constraints refuse. They
stay as gate steps invoked directly until there is an answer. Deferred 1.

**`mutants` does not fit this shape and is not made to.** Its verdicts are
internal to `mutants.rs`, which belongs to the crate-root `cargo-lid-rs` slice
and which no catalog phase may write; it needs `--full` and `--diff-base <ref>`,
which `gate.yml` uses and which the no-passthrough rule forbids; its runtime is
measured rather than budgeted; and a `Timeout` verdict is neither cleanly a
finding nor cleanly a tooling error. Forcing it in would mean deciding four
things this slice has no basis for. Deferred 2.

### The catalog lists itself

`cargo lid-rs catalog` prints every command the catalog holds, whether it is
built, and for an unbuilt one **why**. An unbuilt command invoked by name
refuses with that same sentence and **exit 2** — a tooling error, not a
finding, because "this check does not exist here" is not an answer about the
project. A name the catalog holds no entry for at all is refused the same way
and with the same status; it is a different sentence and a different branch.

**An unbuilt entry carries a reason, not a slice.** The first draft said "the
slice it lands with", and six of pipeline §5.1's sixteen rows have no slice to
name: `examples`, `suite`, `graph` and `regen` wait on a libtest with
machine-readable output (Deferred 1), `mutants` on four open questions
(Deferred 2), and `pr-body`, `status` and `commit` are the pipeline's own and
were never this slice's. Typing the field as a slice would have forced Phase 4
to invent slice numbers or to write `"Deferred 1"` into a field called `slice`
— and `AnUnbuiltCommandsEntryNames…` would then have been validated against a
lie.

A reason is one of **three** things: the slice a command lands with, the
question it waits on, or the thing that owns it where this tool does not build
it at all. The third case was missed on the first correction and the tests
found it: `pr-body`, `status` and `commit` wait on no question and land with no
slice, so with two cases a phase must write "the pipeline" into a variant
called `Question` — the same lie in the other shape, for three of the
seventeen rows. The distinction is real and not bookkeeping: a command waiting
on a slice or a question will one day be built here, and one owned elsewhere
will not.

The catalog's rows, then, are pipeline §5.1's sixteen plus `package`, which is
this workspace's gate step and **not** a §5.1 row:

| Entry | State |
|---|---|
| `check`, `lint`, `doc`, `sync` | built here |
| `package` | built here; not a §5.1 row, this workspace's own |
| `shape` | waits on slice 18 |
| `conform` | waits on slice 19 |
| `validate` | waits on slice 20, for its runtime half |
| `site` | waits on slice 21 |
| `examples`, `suite`, `graph`, `regen` | wait on Deferred 1 |
| `mutants` | waits on Deferred 2 |
| `pr-body`, `status`, `commit` | the pipeline's, never this slice's |

### The finding schema

Pipeline §5.3, unchanged: `check`, `rule`, `severity`, `file`, `line`, `item`,
`claim`, `message`, `fix`, `source`. `item` and `claim` are present when the
check is about one; `rule` is the LID-rs rule code where there is one.

A finding is not a diagnostic relabelled. Cargo's JSON is the input; the finding
is what this project says about it — including the `fix` line, which is read
from the synced skill's **`references/gates.md`**, whose rows are keyed by
check (`| check 1 — citation fails to resolve | … | Correct response |`).

The first draft named `references/discipline.md` and `policy::discipline_rows`.
Both are the wrong table: `discipline.md` is keyed by *phase* and "when", holds
no check column at all, and `discipline_rows` filters its rows by phase number
— a `Finding` has no phase. Had this shipped, every finding's `fix` would have
been empty, and a validation asserting that emptiness would have been green
over nothing. A diagnostic this project has no check number for is carried
with `check: 0` rather than dropped, because a check that silently discards what
it does not recognise is the pattern this workspace has now met three times.

### A diagnostic is a diagnostic, whichever stream it arrived on

The two provenances differ in how a tool's output is *read*, and in nothing
else. What a diagnostic becomes — its check number from the lint that raised
it, its `fix` from `gates.md`, the file and line it points at — is one rule,
and writing it twice is this slice's own disease.

So each provenance answers with the same domain type, and one item turns that
into a finding:

- cargo's JSON stream is parsed into diagnostics;
- a tool with no JSON stream has its stderr parsed into the same;
- one item maps a diagnostic to a `Finding`.

**This closes a gap the first draft left.** With the rules stated of
`findings_from_cargo`, a stderr mapper answering
`Finding { file: None, line: None, check: 0 }` for every rustdoc diagnostic
satisfied both of its claims — the file-and-line rule, the check-number rule
and the `fix` rule all named the cargo path and none reached the other. The
guarantee was not weaker on purpose; it was weaker because of where the rule
happened to be written.

**Why a domain type and not `serde_json::Value`.** A library type at the
boundary is fine and `Value` is how cargo's stream arrives; a library type in
the interior leaves is not, because every item below then depends on the shape
of a foreign crate's parse rather than on what this slice means by a
diagnostic. `stderr` has no JSON at all, so a shared interior over `Value`
would force the stderr parser to *build* JSON in order to be read — which is
the shape of the problem, not a solution to it.

### Where reports go, and what bounds them

`target/lid/` in the workspace root, created on demand. `target/` is
gitignored, so the stop hook's `outside_policy_clean` is unaffected. A directory
that cannot be created is exit 2 with a finding naming it — not a silent skip.

Timeouts are pipeline §5.1's fourth property and are **not** built here: they
would read `[workspace.metadata.lid_rs.pipeline]`, which is the manifest this
slice cannot write. Deferred 3.

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| The gate's definition is not in this slice | Commands and schema only; metadata, the rename and `phase::plan`'s retirement are a `phase` change with this as precondition | One slice as first drafted; a hand-committed cross-slice change | The metadata lives in a manifest no phase may write, and the step lists are another slice's *claims*. The first draft promised a rename it could not perform from its own parts — three of ten steps had no command. Precedent: slice 16's four-commit delegation. |
| `examples`, `suite`, `graph`, `regen` deferred | Stay as direct invocations | Scrape libtest's human output; take libtest's unstable JSON | Scraping violates this slice's own "rendering from findings, never from output" rule at two of its commands. Unstable JSON is a nightly dependency the constraints refuse. Deferring is honest; building them on a scrape would make the rule a slogan. |
| `mutants` is not a catalog command | Deferred with its four open questions named | Wrap it thinly; force the shape | Its verdicts are another slice's internals, its flags are the passthrough this slice forbids, its runtime is unbudgeted, and its `Timeout` is neither finding nor error. Four decisions with no basis is not a command. |
| An unbuilt command exits 2, not 1 | Tooling error | Exit 1 with a finding; exit 0 as a no-op | "This check does not exist here" is not an answer about the project, and a no-op that exits 0 is the check-that-gates-nothing pattern. |
| An unrecognised diagnostic is `check: 0` | Carried | Dropped | A check that discards what it does not recognise reports success on the thing it failed to read — the pattern behind slice 15's lexicon harness, three empty red runs, and `mdbook build`. |
| `sync` yields one finding, not one per difference | The comparison's message carried whole | A per-difference parse of `sync::check`'s error string; a public structured accessor in `sync`; dropping `sync` from this slice | The accessor is the `sync` slice's file, which no catalog phase may write; the parse is the scrape this slice's own rendering rule forbids and used to defer `examples`. Dropping it would remove the one thing this slice's motivation says the repair buys — a gate that reaches `sync --check`. One honest finding is buildable today; Deferred 7 is the rest. |
| A finding's `fix` comes from `gates.md` | The check-keyed table | `discipline.md` and `policy::discipline_rows`, as the first draft said | `discipline.md` is keyed by phase and holds no check column; `discipline_rows` filters by phase number and a finding has no phase. The first draft's source would have made every `fix` empty, and a test asserting that would have been green over nothing. |
| An unbuilt entry carries a reason, not a slice | Slice, question waited on, or owner elsewhere | A `slice: String` for every unbuilt command; two cases, as first corrected | Six of §5.1's sixteen rows have no slice: four wait on Deferred 1, one on Deferred 2, and three are the pipeline's own. A field typed `slice` would have been filled with `"Deferred 1"` or an invented number, and the claim about it validated against that. |
| `run` delegates to `run_with` over an injected runner | Split, as `phase::execute` splits | One `run` that spawns and carries every claim | Four claims are about what a run decides, and none is observable from a signature that must really invoke cargo. A validation would have to spawn a build; every mutant in the deciding code would survive. The precedent is in the module this document already cites. |
| A name the catalog holds no entry for is its own branch | Refused, with the tooling status | Folded into the unbuilt refusal; left unstated as the first draft left it | `run`'s dispatch has an arm the first draft did not write, and an unwritten branch is a requirement nobody recorded. It is not the unbuilt case: that entry exists and names a reason, and this one has no entry to name anything. |
| Both provenances answer with one `Diagnostic` type | Shared domain type, one `finding_of` | The rules stated of `findings_from_cargo`, as first drafted; `serde_json::Value` as the shared shape | With the rules on the cargo path, a stderr mapper answering `file: None, line: None, check: 0` for everything satisfied both of its claims — a weaker guarantee by accident of placement. `Value` in the interior would make the stderr parser build JSON to be read, and puts a foreign crate's parse shape below the boundary. |
| `package` runs one invocation naming every publishing member | The form that resolves | Per crate, as `phase::plan` does; publish `lid-rs` so the per-crate form works | The per-crate form cannot resolve an unpublished sibling; publishing to make a gate step pass is a release decision driven by a tooling bug. The member list comes from `publishing_members()`. |

## Open Questions & Future Decisions

### Deferred

1. `examples`, `suite`, `graph` and `regen` — all `cargo test` invocations with
   no machine-readable output on stable. They need either a libtest JSON that is
   not nightly-only, or a decision that these four commands' findings may come
   from a parse of human output, which contradicts this slice's rendering rule.
2. `mutants` as a catalog command: its findings' provenance (`mutants.rs` is the
   `cargo-lid-rs` slice's), its `--full`/`--diff-base` flags against the
   no-passthrough rule, its budget, and whether a `Timeout` is exit 1 or 2.
3. Per-command timeouts from `[workspace.metadata.lid_rs.pipeline]` — the
   manifest no phase may write, so they land with the gate-from-metadata change.
4. Exit code 2 itself. `main` maps every `Err` to `ExitCode::FAILURE`, i.e. 1,
   and `main.rs` is in no phase's allowed set. Until that changes, a tooling
   error is reported *in* the report and exits 1, and the distinction this slice
   designs is visible only in the JSON. The fix is one line in another slice's
   file.
5. `fast` — named in pipeline §5.2 and not built here; it belongs with the
   composites.
7. One finding per `sync` difference. It needs `sync` to expose its
   comparison structurally — `artifact_differences` and its two helpers are
   private, and `check` joins them into one string — which is the `sync`
   slice's change and no catalog phase's. Until then a `sync` run is one
   finding carrying the message whole.
8. Printing the catalog, and the `cargo lid-rs <command>` dispatch this
   document opens with. No item here prints and `dispatch` in `src/lib.rs` has
   no arm for these commands; both are the crate-root slice's. The refusal
   sentence is therefore composed in one place today, and when the printer
   lands it must call the same function rather than compose its own — the
   "six places state a gate" disease this slice exists to cure.
6. `init`/`new` scaffolds, which write a project's gate; they change with the
   gate's definition, not with the commands.

### The runner is injected, so a run is observable without spawning

`run` cannot both spawn a tool and be the item every behavioural claim hangs
on. Four of this slice's claims — the fixed invocation, the report written
whatever was found, the status over the finding list, and the unbuilt refusal
— are about what a run *decides*, and none of them is observable from a
signature that only returns a `Report` after really invoking cargo. A
validation would have to spawn a build inside a unit test to make one green,
and every mutant inside the deciding code would survive, which is the pattern
check 12 caught twice while this slice was being built.

This crate already has the shape that works, in the module this document
cites: `phase::execute` carries no citation and delegates to
`execute_with(steps, runner)`, which carries the claim over an injected
runner. The same split here — `run` as the I/O wrapper with no citation, and a
`run_with` taking a runner that maps an `Invocation` to a tool's output — puts
the decisions under test and leaves the spawning uncited, where nothing needs
to falsify it.

The runner is what makes "the fixed invocation that entry names" observable at
all: the test's runner records the `Invocation` it was handed, which is the
only way a wrong answer to the no-passthrough rule can be seen.

## Shape

| Item | Role |
|---|---|
| `cargo_lid_rs::catalog` | This slice's module: the command table, the schema, the runner. |
| `cargo_lid_rs::catalog::Finding` | One finding, in pipeline §5.3's schema. |
| `cargo_lid_rs::catalog::Report` | A command's findings and its status, serialised to `target/lid/<command>.json`. |
| `cargo_lid_rs::catalog::Command` | An atomic command: its name, and what it is — a fixed invocation with a finding provenance, or unbuilt with the reason. |
| `cargo_lid_rs::catalog::Invocation` | What a command is: the fixed invocation and where its findings come from, or unbuilt and why. One enum, so a built entry has exactly one invocation and an unbuilt one exactly one reason, and the two cannot contradict each other. |
| `cargo_lid_rs::catalog::Unbuilt` | Why an entry is not built: the slice it lands with, the question it waits on, or the thing that owns it where this tool does not build it at all. Three cases, because a command owned elsewhere waits on nothing and will never be built here. |
| `cargo_lid_rs::catalog::Status` | 0, 1, 2 as a closed set, its discriminants the exit codes — so the three numbers the claims name are stated once and a caller cannot read a status without interpreting it. |
| `cargo_lid_rs::catalog::table` | Every command the catalog holds — §5.1's sixteen and `package` — built and unbuilt, each unbuilt one with its reason. What `cargo lid-rs catalog` prints and what a name is refused against. |
| `cargo_lid_rs::catalog::run` | One command by name: resolve the entry, invoke, map, write. The I/O wrapper — it spawns, and carries no citation. |
| `cargo_lid_rs::catalog::run_with` | The same over an injected runner: every decision a run makes, with nothing spawned. Where the behavioural claims are seated. Answers with the `Report`, which carries the status (Deferred 4), rather than the status alone. |
| `cargo_lid_rs::catalog::Runner` | An `Invocation` to a tool's output: what `run` supplies by spawning and a validation supplies by recording. |
| `cargo_lid_rs::catalog::ToolOutput` | A tool's two streams, whole. The runner answers both rather than the one the invocation's provenance names — choosing between them is a mapping decision, and leaving it to the runner would put it in the spawning half, which carries no citation. It is the distinction `EveryDiagnosticOfTheStreamBecomesAFinding` and `ADiagnosticFromAToolWithNoJsonStreamIsReadFromStderr` exist to draw. |
| `cargo_lid_rs::catalog::Diagnostic` | One diagnostic as this slice means it: the lint that raised it where there was one, its message, and the file and line it points at where it points at source. What both provenances answer with, so that what a diagnostic becomes is one rule and not two. |
| `cargo_lid_rs::catalog::diagnostics_from_cargo` | Cargo's JSON stream to diagnostics. The boundary where `serde_json` is used and below which it is not. |
| `cargo_lid_rs::catalog::diagnostics_from_stderr` | A tool's stderr to diagnostics, for rustdoc and `cargo package`, which carry them on no other stream. |
| `cargo_lid_rs::catalog::finding_of` | One diagnostic to a `Finding`: the check number from the lint name or 0, the `fix` from the skill's `references/gates.md` row for that check, the file and line the diagnostic points at. One rule for both provenances. |
| `cargo_lid_rs::catalog::findings_from_cargo` | The JSON provenance whole: parse, then map. |
| `cargo_lid_rs::catalog::findings_from_stderr` | The stderr provenance whole: parse, then map. |
| `cargo_lid_rs::catalog::render` | A report to the human rendering, from the findings alone. |

## References

- `lid-rs-pipeline/docs/intent/pipeline.md` §5 — the catalog's specification;
  §6.4 — the watchdog that bounds what can run inside a worker's stop.
- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) — the gate as prose.
- `cargo-lid-rs/src/phase/lld.md` — `phase::plan`, and the slice that will own
  the gate's definition.
