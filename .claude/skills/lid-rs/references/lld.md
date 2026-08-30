<!-- Managed by `cargo lid-rs sync` from the `lid-rs` crate this project depends
     on. Do not edit: the gate's `sync --check` fails on any difference, in any
     file under this directory. Project-specific guidance belongs in AGENTS.md. -->

# Writing an LLD, and reading one

Phase 1 is human-owned, and every phase after it is derived from what the
document says. When it is silent, or says two things, the cost is not paid at
Phase 1: it is paid three phases later by a worker that guesses and a reviewer
that rejects, and again by the human who must decide what the document meant.

This file is both halves of the answer. The checklist is what `cargo lid-rs
lld-check` refuses without judgment, so a `phase 1:` commit cannot be made
against it. The questions are what a reader with judgment asks and *reports* —
they refuse nothing, because approving an LLD is the human's act and a gate an
agent decides is not a gate.

## The checklist — what the tool refuses

Six checks, each a property of the text with no opinion in it. Four are about
the document; two are about the artifacts this file ships with, so that a
guideline and the code cannot drift, and so that the reader's inability to edit
is held rather than promised.

| Check | It holds when |
|---|---|
| `DecisionsExist` | The document has a `## Decisions & Alternatives` heading with a table under it |
| `Alternatives` | Every row of that table has four non-empty cells |
| `ShapeRows` | Where a `## Shape` table exists, every row names at least one backticked identifier and gives a non-empty role |
| `DeferredNumbered` | Every item under `### Deferred` is a numbered list item |
| `GuidelineNamesEveryCheck` | This checklist names every check the tool knows |
| `ReaderObservesOnly` | The reader declares `Read`, `Grep`, `Glob` and nothing else |

The set is small on purpose. Six candidates were measured against thirteen
LLDs, and two were dropped for refusing documents that are not wrong: a
required section structure fires on ten of the thirteen, which use headings of
their own; and "every item the Shape table names appears in the prose" fires
nineteen times across the two largest, because prose introduces a type by
describing it rather than by spelling its identifier. Both signals are real,
and both are questions below.

What the checklist gives up is worth knowing, because the questions are the
only answer to it: a document passes with four filled cells that record no real
alternative, well-formed shape rows naming items the prose never introduces,
and numbered deferrals that defer nothing.

## The questions — what a reader asks

Each carries the incident that put it here. A finding is a passage and the
question it fails, never a verdict on the document.

1. **Does every paragraph of behaviour name the item that performs it, and is
   every item the Shape table names introduced somewhere in the prose?** A
   slice's Shape grew a boundary helper, an offered record and a classifier
   that its prose never mentioned; Phase 3 was rejected twice for inferring
   what they were for.

2. **Does every path that can fail have a stated outcome — what stops the run,
   what is retried, what is reported?** Two documents were silent on what an
   unset environment variable does and on how a rework commits; both surfaced
   as phase stops, and both answers took one sentence.

3. **Is every contract with another system stated, and does the document say
   where its shapes were verified?** A policy hash was asserted backwards, and
   a wire format was absent until Phase 3 had to guess it. Provenance is owed
   at the level the document cites at — a References section naming the other
   system's source files answers for every constant drawn from them; a
   hostname, a timeout, or a magic number with no source anywhere does not.

4. **Does everything that touches the filesystem, the network, or a credential
   state its bound completely?** Two confinement rules were half-stated; a
   reviewer found both by reasoning about how to escape them, one of which
   admitted `*/../..`.

5. **Do the document's sections agree with each other?** Three amendments to
   one slice's LLD were sections contradicting each other after a change landed
   in one of them: a shape table naming what the behaviour had dropped, a count
   stated twice and updated once, a description left describing the old set.
   No mechanical check catches this.

6. **Could a reader who has never seen this conversation derive the slice from
   this document alone?** No narration of how it changed, no meaning that needs
   the discussion that produced it.

7. **Is each rule stated at a granularity a claim can be cut from?** Phase 2
   must derive atomic claims, one *when* and one *shall*. A single table cell
   once packed five independent rules about `glob` — confinement, two refused
   pattern shapes, two directories never entered, matches outside the root
   omitted, sorted output — and every mechanical check passed it. The agent
   then guesses whether that is one claim or five, and whatever it guesses is
   what Phase 5 tests.

8. **Is this one slice?** Phase 0's question, and the decision with the
   largest downstream cost in the methodology, asked by no mechanical check.
   A shape table spanning an HTTP client, a digest, a filesystem sandbox, a
   turn loop, a worker driver and a git precondition is a candidate for two
   documents, and the time to say so is before the claims exist.

9. **Does the document's structure serve a reader who has never seen it?**
   Context, then behaviour, then shape, then decisions and what was deferred.
   Prescribing those headings mechanically would refuse ten of thirteen
   existing documents, so this is a reader's judgment about whether a structure
   hides something, not a rule.

## What this guideline has been measured on

Read once against a large, well-made LLD — the one whose construction
produced the incidents above — a reader applying these questions found one
contradiction that would plausibly have stopped Phase 2, one unstated rule
the phases would have guessed at silently with a security consequence, and
three passages not worth a keystroke. Questions 5, 4 and 1 earned their
place; question 3 produced a near-miss and was reworded; a question asking
whether each decision row records a real alternative found nothing across
eighteen rows, misled the reader into a false positive, and was removed.
Questions 7 and 8 are what that reader said the list was missing.

Keep score the same way. A question that fires only on documents that turn
out to be fine is worse than no question, because it teaches authors to write
for the reader instead of for the next phase.

## Terms

One word, one meaning, within a document. A slice that invents vocabulary —
"shed", "settle", "wound" — defines it where it first appears. Near-synonyms
for one concept are the most common way two sections come to disagree without
either being wrong.
