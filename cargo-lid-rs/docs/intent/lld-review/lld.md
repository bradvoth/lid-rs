# lld-review — an LLD is read before a slice is built

## Context and Design Philosophy

Phase 1 is human-owned, and everything after it is derived from what it
says. When it is silent, or says two things, the cost is not paid at
Phase 1: it is paid three phases later, by a worker that guesses and a
reviewer that rejects, and again by the human who must decide what the
document meant. The methodology has no reader between the human writing
the LLD and the machine building from it.

The canopy-client slice measured that cost. Its LLD was amended twelve
times while the slice was built, and the amendments were not exotic
discoveries — six were something true but unstated, two an external fact
never verified, two prose contradicting the shape table, two process gaps:

| What the LLD did | Found at | Cost |
|---|---|---|
| Said four settings in prose, two in the Shape table | Phase 2's review | a rework |
| Named no item for decisions the Behaviour described (a boundary helper, an offered record, a classifier) | Phase 3's review, twice | two reworks |
| Asserted another system's wire contract without stating where it was verified — and had it backwards | a probe, then Phase 3 | a rewrite of a section |
| Was silent on what an unset environment variable does, and on how a rework commits | Phase 2 | a stop |
| Left a filesystem tool's confinement half-stated | Phase 4's review, twice | two reworks |

Roughly half of those a careful reader could have found in minutes, with
nothing but the document and a checklist. The other half needed the
descent to discover, and belong where they were found. The difference
between the two is the whole design of this slice: **what can be checked
from the text alone should be checked from the text alone, before any
phase runs.**

Two readers, then, and they are deliberately unlike each other. The first
is mechanical and blocking: text checks with no judgment in them — the
document records a decision with its alternatives, its shape rows name
what they are about, its deferrals are numbered, and the guideline and
reader this slice ships say what the code says. These run as a step of
`phase-check 1`, so a `phase 1:` commit cannot be made without them, in
every host and in CI, with no agent consulted. The second is a
reader with judgment: an agent that applies the guideline's questions —
does each behaviour paragraph have an item that performs it, does every
edge path have a stated outcome, is every external contract sourced — and
**reports** what it finds. It cannot block and it cannot approve, because
approving an LLD is the human's act (README §8) and a gate an agent
decides is not a gate (`phase/lld.md`).

The guideline and the agent are one artifact seen twice: the checklist
lives in the skill as `references/lld.md`, and the agent's body is an
instruction to apply that file. Neither can drift from the other because
there is only one of them.

The order in which this arrives is deliberate. It is the first of the
three slices this plan holds — before the walk gains its routing edges
and before a Draft PR starts a container — because every stop it prevents
is a stop the other two never have to recover from.

The checks were chosen by measurement, not by taste: six candidates were
run against the thirteen LLDs this workspace holds, and the two that
refused documents which are not wrong were moved to the reader, where a
false positive costs a sentence of explanation rather than a refused
commit. What survives fires on nothing the workspace has written, and on
three real gaps in a sibling slice's draft.

This document has been its own first customer, and not flatteringly:
three of its amendments were sections contradicting each other after a
change landed in one of them — the shape table naming checks the
behaviour had dropped, a count stated twice and updated once, a
description of the checks left describing the old set. No mechanical
check here would have caught any of them. They are the reader's, and they
are the reason the reader is not decoration.

## Behaviour

### `cargo lid-rs lld-check [--slice <name>]`

Reads one slice's LLD, the slice defaulting to the branch name with
`lld/` removed, as `phase-check` reads it. The document is
`docs/intent/<slice>/lld.md` in the workspace package that holds it, or —
for a slice whose product is the workspace rather than a crate, as `book`,
`publish` and `skill` are here — the same path at the workspace root,
since a virtual manifest holds no package to find. It applies the
mechanical checks below. Any
other argument is rejected by name, as every subcommand of this tool
rejects one. It exits zero when they all hold, and otherwise names every
failure with the file and line it is on, so a human sees the whole list
rather than the first item.

The same checks run as a step of `phase-check 1`, before the doc and
doctest steps, so a `phase 1:` commit is gated on them wherever a phase
commit is made. The subcommand exists so a human can ask the question
while writing, before committing anything.

### The mechanical checks

Each is a property of the text, with no judgment in it. A check that needs
a reader's opinion is not here; it is the guideline's, and the agent's.

| Check | What it holds | Why, in evidence |
|---|---|---|
| Decisions exist | The document has a `## Decisions & Alternatives` heading with a table under it | The skill's Phase 1 file demands the table by name; a slice without one recorded no alternatives |
| Alternatives | Every row of that table has four non-empty cells | README §8: a decision with no alternative considered is a decision not yet examined |
| Shape rows | Where a `## Shape` table exists, every row names at least one backticked identifier and gives a non-empty role | A row with no identifier is a note, not a shape |
| Deferred is numbered | Every item under the deferred heading is a numbered list item | An unnumbered deferral cannot be cited by a phase that hits it |
| Guideline names every check | The guideline's checklist names every variant of `Check` | A checklist and an enum that disagree are two sources of truth; the guideline is what a human reads before the code refuses them |
| Reader observes only | The reader's frontmatter declares `Read`, `Grep`, `Glob` and nothing else | The reader is advisory: that it cannot edit is a property to hold, not a sentence to trust. Its two artifacts are prose an agent executes, so this is the one place a gate can hold them |

The four document checks read the document `--slice` names. The two
artifact checks read the project's **synced** copies —
`.claude/skills/lid-rs/references/lld.md` and
`.claude/agents/lid-rs-lld-review.md` — because those are the files a
consumer has, and the canonical originals exist only in the crate that
ships them. They are the same on every invocation, so they run whatever
slice is named, and a copy that is absent or unreadable is a failure
naming its path, as an unreadable LLD is.

Two headings are named rather than inferred. The decisions table is the
one under `## Decisions & Alternatives`, and the deferred list is the one
under `### Deferred`; a document with no deferred heading has no item to
number, and holds.

The set is this small because it was measured rather than chosen. Six
checks were prototyped against the thirteen LLDs this workspace holds —
ten in crates and three at the root — and two of the six fired on
documents that are not wrong: a required section structure fires on ten
of thirteen, which use headings of their own and often name their shape
in prose instead of a table; and "every item the Shape table names appears in
the prose" fires nineteen times across the two largest LLDs because prose
introduces a type by describing it rather than by spelling its identifier.
Both signals are real — the same prototype found three genuine gaps in a
sibling slice's draft — but neither is a property a human's commit should
be refused for. They are the reader's, below.

A failure names the check, the file, the line, and the sentence the skill
states the rule in. An artifact failure also names what it found — the
`Check` variants the checklist omits, or the tools the reader declares —
and points at the line it read them from: the checklist's heading, or the
frontmatter's `tools:` line, and the file's first line when neither
exists. Nothing here is configurable: a project that wants a different
document shape changes the skill, which is the same discussion in the
right place.

What the mechanical half gives up is worth saying plainly. These are text
properties, so a document passes with a Decisions table whose four filled
cells record no real alternative, a Shape table of well-formed rows naming
items the prose never introduces, and numbered deferrals that defer
nothing. The reader answers the second and the third; the first is a gap
left open on purpose, because the guideline's question about it found
nothing across eighteen decision rows and misled its reader into a false
positive, and a question that fires only on good documents costs more than
the gap it covers. Of the twelve amendments the canopy slice paid for, the
mechanical half would have caught two. The rest are the reader's, and no
claim of this slice obliges a host to run it — a project that never asks
the reader passes every gate this slice adds.

### The guideline

`skill/references/lld.md`, synced into projects like every other reference
file, holds the checklist above and the questions the mechanical checks
cannot ask. The questions are written as a reader's list, each with the
evidence that put it there:

- Does every paragraph of behaviour name the item that performs it, and
  is every item the Shape table names introduced somewhere in the prose?
  *(The canopy LLD failed this twice; both cost a Phase 3 rework. It is
  the reader's question and not a mechanical check because prose
  legitimately introduces a type by describing it — the mechanical form
  fires nineteen times across this workspace's two largest LLDs.)*
- Does the document read as a fresh author's, with no sentence that needs
  the conversation to parse? *(The forbidden phrases are listed in this
  file, which is why the check is the reader's: a mechanical scan flags
  the list itself.)*
- Does the document's structure serve a reader who has never seen it —
  context, then behaviour, then shape, then the decisions and what was
  deferred? *(Prescribing those headings mechanically would refuse seven
  of this workspace's nine LLDs, which are not wrong; a reader can say
  when a structure genuinely hides something.)*
- Does every path that can fail have a stated outcome — what stops the
  run, what is retried, what is reported? *(An unset variable and a
  rework's commit were both unstated; both surfaced as stops.)*
- Is every contract with another system stated with where it was
  verified, or explicitly deferred? *(The policy hash was asserted
  backwards; the wire shapes were absent until Phase 3 needed them.)*
- Does everything that touches the filesystem, the network, or a
  credential state its bound completely? *(Two confinement rules were
  half-stated; both were found by a reviewer reasoning about escapes.)*
- Is each Decisions row a decision the slice actually faced, with an
  alternative that was really considered?
- Could a reader who has never seen this conversation derive the slice
  from this document alone?

### The reader

The `lid-rs` crate ships `agent/lid-rs-lld-review.md`; `sync` mirrors it
beside the phase agents. Its tools are `Read`, `Grep`, `Glob` — the
reviewer's set — and its body is one instruction: read
`references/lld.md` and the slice's LLD, apply the guideline's questions,
and report what a phase would predictably stop on. It has no
`PreToolUse` path policy to enforce, because it writes nothing, and no
`Stop` hook, because there is no commit to make.

Its answer is a list of findings, each naming the question it comes from
and the passage it is about, and a count. It never says approved: an LLD
is approved by the human committing it. It is run:

- by the human, while writing, as the last thing before the `phase 1:`
  commit;
- by an unattended host, on the `phase 1:` commit, before Phase 2 opens —
  where its findings are reported and the run continues, since an
  advisory reader that could stop a run would be a gate an agent decides.

A host that reports the findings and proceeds is doing the right thing:
the mechanical half already refused what can be refused without judgment,
and the rest is information for the human who owns the document.

## Cascade

The claim that binds these checks to a phase belongs to the phase slice,
not this one: `phase-check 1` runs them before its doc step, which is
`PhaseOneChecksTheDocs`'s sentence to widen
(`docs/intent/phase/lld.md`). That rewording lands with the step itself,
at this slice's Phase 7 — reworded earlier, the phase slice's own
validation would fail against a step that does not yet exist. Until then
`lld-check` is reachable as a subcommand and nothing runs it for you.

That cascade is not a phase agent's to make: `src/spec/phase.rs` and
`src/phase/` are another slice's module, which every phase's path policy
refuses. It is the human's commit at Phase 7, alongside the gate they run
there, as every cascade into another slice has been.

## Shape

| Item | Role |
|---|---|
| `lld_review::run(args)` | `lld-check` entry: parses `--slice`, locates the LLD, applies the checks, prints every failure |
| `Lld`, `Lld::read(project, slice) -> Result<Lld, String>` | The document as lines, with the slice and path it came from |
| `Check` | The closed set: `DecisionsExist`, `Alternatives`, `ShapeRows`, `DeferredNumbered`, `GuidelineNamesEveryCheck`, `ReaderObservesOnly` |
| `Failure { check, path, line, message }` | One failure: the file it is about, the line it is on, and the skill's sentence for the rule — the path matters because an artifact check's failure is about the guideline or the reader, not the document under check |
| `check_all(lld) -> Vec<Failure>` | Every check over one document, in the table's order |
| `decisions_exist(lld)`, `alternatives(lld)`, `shape_rows(lld)`, `deferred_numbered(lld)` | One document check each |
| `guideline_names_every_check(project)`, `reader_observes_only(project)` | The two artifact checks: the guideline's checklist names every `Check` variant, and the reader's frontmatter declares `Read`, `Grep`, `Glob` and nothing else |
| `Table`, `table_at(lld, heading) -> Option<Table>`, `Row` | A markdown table under a heading, as rows of cells — the one parse the checks share |
| `identifiers(cell) -> Vec<String>` | The backticked identifiers a cell names |
| `Step::LldChecks` (phase slice) | `phase-check 1` gains this step, before the doc step |
| `lid-rs/skill/references/lld.md` | The guideline: the checklist and the reader's questions |
| `lid-rs/agent/lid-rs-lld-review.md` | The reader, read-only, advisory |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| What blocks | The mechanical checks alone | The agent's findings block too; nothing blocks | A gate an agent decides is not a gate (`phase/lld.md`), and this is the phase whose artifact the human owns — refusing a human's document on a model's opinion inverts that. What can be refused without judgment is refused; the rest is reported. |
| Where the checks run | A step of `phase-check 1`, and a subcommand of their own | The subcommand alone, called by CI; a git hook | As a step they are reached by every host, by CI, and by the phase agent's stop hook, for free. As a subcommand they answer the question while the document is still being written, which is when it is cheapest to fix. |
| What the checks are | Four text properties with no judgment, measured against the workspace's nine existing LLDs before being chosen | Six, including a required section structure and "every Shape item appears in the prose"; a schema the document must validate against | Each surviving check fires on no document that is not wrong. The two that were dropped fire on seven and nineteen passages of documents that are fine: a check with that rate teaches authors to write for the checker, and their signal is kept as the reader's questions instead. |
| The guideline's contents | A checklist plus questions, each with the evidence that put it there | A style guide; a template to fill in | The skill's `discipline.md` is written this way and it is the file that changed behaviour, because a rule with its incident attached is a rule a reader believes. A template produces documents shaped like the template. |
| Guideline and agent | One file, the agent's body pointing at it | The agent's body carries the checklist | Two copies of a checklist drift, and the skill's sync rule already keeps one copy honest. |
| The reader's tools | `Read`, `Grep`, `Glob` | Also the LSP; also the ability to write a report file | It reads one document and the skill; anything it could write is a finding it should say instead. |
| Existing LLDs | The checks were narrowed until every existing LLD passes them | Bring nine documents into conformance with a stricter set; grandfather the old ones | A gate that exempts what already exists never fires, and a gate that fails what is already good is a gate authors learn to route around. Measuring first turned a restructuring of seven documents into a checklist that is right on all nine — and the measurement is itself the answer to whether each check was worth its strictness. |
| Naming | `lld-check` for the mechanical checks | `lint`, `doc-check`, folding it into `phase-check 1` with no subcommand | It says which document and which kind of check; `phase-check 1` remains the gate, and this is the same question asked early. |

## Open Questions & Future Decisions

### Deferred
1. Whether the reader's findings should carry the routing slice's
   categories, so that a finding it makes about a *later* phase's likely
   failure can be routed rather than only reported. Today it reports.
2. Whether the guideline's questions earn their place: the honest measure
   is a table like `discipline.md`'s, recording for each slice which
   questions the reader raised and which of those a later phase would have
   stopped on. Until several slices have run under it, the list is this
   session's evidence and no more.
3. A check that every backticked identifier in the Behaviour prose also
   appears in the Shape table — the converse of `ShapeExplained`. It is
   the same idea, but prose names types and functions from other slices
   too, so it needs a rule for what is out of scope before it can be
   mechanical.

## References

- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (Phase 1 is human-owned; a decision with no alternative is not yet examined).
- `docs/intent/phase/lld.md` — the phase checks this adds a step to, and
  the rule that a gate is never an agent's choice.
- `docs/intent/sync/lld.md` — the strict mirror rule the guideline and the
  reader ship under.
- `.claude/skills/lid-rs/references/phase-1.md` and `discipline.md` — what
  the guideline deepens, and the evidence-table form it follows.
