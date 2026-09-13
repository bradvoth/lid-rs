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
run against the thirteen LLDs this workspace held when the slice was first
written, and the two that
refused documents which are not wrong were moved to the reader, where a
false positive costs a sentence of explanation rather than a refused
commit. The document checks that survived fire on nothing the workspace
has written, and on three real gaps in a sibling slice's draft. The checks
that read the slice's code were measured the same way and the answer came
back the other way round: they fire on three rows that disagree with their
own source and four that name another slice's item without linking it, and
every one of the seven is a document that has stopped describing the code
it is about. Correcting them is documentation, a sweep has already made
those corrections, and the Cascade section says whose and when.

The checks that read code read it through one crate. `lid-rs-shape` is this
workspace's syntactic pass over a crate's tokens — every function the crate
declares, with the parameter and return type tokens the source wrote — and
this slice consumes that answer rather than producing one of its own. It is
consumed at the version that carries `owner` on its `Signature`, the self
type of the `impl` block a method was read from: without it a `Self` in
either the row's return or the source's has nothing to stand for. That
amendment is
the shape crate's own, made on its own branch, and it lands on main before
this slice's Phase 3.

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
`lld/` removed, as `phase-check` reads it. **The document is whatever
`layout::lld_path` answers for the slice** — this slice asks and does not
compute. Before the colocation migration that answer is
`docs/intent/<slice>/lld.md` in the workspace package that holds it; after it,
the `lld.md` beside the slice's code, or `<crate>/src/lld.md` for a slice that
is its crate. Either way, or —
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

Each is a property something either has or has not, with no judgment in it:
most of the text alone, two of the text beside what the slice's own
directory holds. A check that needs a reader's opinion is not here; it is
the guideline's, and the agent's.

| Check | What it holds | Why, in evidence |
|---|---|---|
| Decisions exist | The document has a `## Decisions & Alternatives` heading with a table under it | The skill's Phase 1 file demands the table by name; a slice without one recorded no alternatives |
| Alternatives | Every row of that table has four non-empty cells | README §8: a decision with no alternative considered is a decision not yet examined |
| Shape rows | Where a `## Shape` table exists, every row names at least one backticked identifier and gives a non-empty role | A row with no identifier is a note, not a shape |
| Rows agree with the code | Where a `## Shape` row's first cell writes a signature fragment for a function the slice's own source declares, the fragment's argument count is that function's parameter count, and a fragment that writes a return type writes the source's | Measured over this workspace's 322 shape rows, with receivers and `Self` read as below: three rows disagree with the code they describe — two writing a return the source does not have, one naming fewer arguments than its function takes — each one a document that had quietly stopped describing its code |
| Returns are skeletonable | A `## Shape` row's return fragment is a type a layer-0 skeleton can be written for: not an `impl Trait` return, and not a bare `dyn Trait` one | Phase 3 writes every row as a signature with a `todo!()` body. A function returning `impl Trait` does not compile with that body — the hidden type is inferred as `!` and `!` implements nothing — and a bare `dyn Trait` return is unsized. A row in either shape is a row whose skeleton cannot be written, discovered at Phase 3 rather than at Phase 1 |
| A reuse row is linked | A first cell's fragment that names another slice's item in the same crate — a path of two or more segments whose first segment starts with a lowercase letter and is none of the slice's own name in module form, a file stem or directory inside the slice's directory, or a workspace member's crate name — is written as an intra-doc link | Resolution is the compiler's and never this tool's (README constraint 2). A row written as `` [`plan`](crate::phase::plan) `` is resolved by rustdoc at the doc step this phase already runs, and a path that has moved fails there by name |
| Deferred is numbered | Every item under the deferred heading is a numbered list item | An unnumbered deferral cannot be cited by a phase that hits it |
| Guideline names every check | The guideline's checklist names every variant of `Check` | A checklist and an enum that disagree are two sources of truth; the guideline is what a human reads before the code refuses them |
| Reader observes only | The reader's frontmatter declares `Read`, `Grep`, `Glob` and nothing else | The reader is advisory: that it cannot edit is a property to hold, not a sentence to trust. Its two artifacts are prose an agent executes, so this is the one place a gate can hold them |

There are nine of them, and they read three different things. Five read
the document `--slice` names and nothing else — four properties of its
tables, and the skeletonability of the returns its rows write. Two read
something beside the document, through the subsection below: one the
slice's own source, one a listing of the slice's directory. The last two
read the project's **synced** copies —
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

The document checks are this few because they were measured rather than
chosen. Six were prototyped against the thirteen LLDs this workspace held
when the slice was first written — ten in crates and three at the root —
and two of the six fired on
documents that are not wrong. **A required section structure fires on ten
of the thirteen**, which use headings of their own and often name their
shape in prose instead of a table; and "every item the Shape table names
appears in the prose" fires nineteen times across the two largest LLDs,
because prose introduces a type by describing it rather than by spelling
its identifier. Those two figures are the measurement, and everywhere else
in this document and in the guideline that mentions the rate refers back
to them rather than restating it.
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

What the mechanical half gives up is worth saying plainly. Nothing here
weighs a sentence, so a document passes with a Decisions table whose four
filled cells record no real alternative, a Shape table whose rows agree
with their signatures and name items the prose never introduces, and
numbered deferrals that defer nothing. The reader answers the second and
the third; the first is a gap left open on purpose, because the guideline's
question about it found nothing across eighteen decision rows and misled
its reader into a false
positive, and a question that fires only on good documents costs more than
the gap it covers. Two more gaps are left open by measurement rather than
by taste. *Every item the prose names has a row* fires on 224 of 254
identifiers across sixteen documents, because prose names claims, error
codes, agent tools, standard library types and other slices' items, and
five exclusion rules leave 106 of them. *Every public item has a row*
fires on 311 of 576, and on 54 of 221 with this workspace's two largest
slices set aside. Both are the reader's questions below, where a false
positive costs a sentence. Of the twelve amendments the canopy slice paid
for, the mechanical half would have caught two. The rest are the reader's,
and no claim of this slice obliges a host to run it — a project that never
asks the reader passes every gate this slice adds.

### Reading the code the rows describe

**What the checks read, and what they refuse to read.** Two of the nine read
something other than the document's own text, and they read different things.
*Rows agree with the code* reads the slice's own functions through
`lid_rs_shape::signatures`: every function a crate declares, with the parameter
and return type tokens the source wrote and the block a method was written in —
the self type for an `impl`, the trait's name for a method a `trait` gives a
body to. That pass resolves nothing — it is the syntactic reading
over one item's tokens constraint 2 permits, and the names it answers with are
the names the source spelled, never what a `use` or an alias would turn them
into. *A reuse row is linked* reads no code at all: it reads the names of the files
and directories inside the slice's directory and the names of the workspace's
member crates, both of which are listings and neither of which is a
resolution. This slice adds no reading of its own, because a second pass over
the same files would be a second answer to the same question.

**Which source is the slice's.** `lid_rs_shape::signatures` is asked about a
crate: it walks from that crate's `src/lib.rs` and `src/main.rs` through the
module declarations they reach, and cannot be pointed at a directory. So the
reading is the whole of the slice's own crate —
[`layout::own_crate`](crate::layout::own_crate) — narrowed afterwards to the
functions whose file is the slice's, which is the `src/<module>/` that
[`layout::slice_dir`](crate::layout::slice_dir) answers, read through, or the
`src/<module>.rs` file module beside it, since a slice whose code is one file
keeps it there and the directory may not exist at all. The `<module>` is the
slice's name with hyphens read as underscores, which is the conversion the
layout already makes in `layout::module_of` — private today, made `pub` by the
same hand commit that adds the dependency, since `layout/` is another slice's
module and no phase agent may write it — and the only form a path ever spells:
a slice called `lld-review` is the module `lld_review`, and nothing here converts
it a second way. For a slice that *is* its crate, `slice_dir` answers that
crate's `src` and the narrowing keeps every function the crate has, which is
the right answer for a slice that owns all of them.

Two things the reading cannot do are silent rather than fatal. A file it
cannot parse contributes no declaration — the shape pass raises an `Unparsable`
finding for it and `signatures` does not carry findings — so a row about a
function in that file is a row about nothing declared, and it holds. A slice
directory that cannot be read lists nothing, exactly as an absent one does.

Both doors answer a refusal rather than a path for a slice no workspace member
holds a document for — a slice whose product is the workspace, as `book`,
`publish` and `skill` are here — and that refusal is read as *this slice
declares nothing*: the agreement check has nothing to compare and holds for
every row. That refusal is not `check_all`'s error, which stays reserved for a
project whose root cannot be located — a slice with no crate is an ordinary
answer, and a project with no root is not.

**A row is compared only to an item that is there.** The comparison is made per
fragment, and a fragment naming a function the slice does not declare is
compared to nothing and holds: at Phase 1 of a new slice no function is
declared yet, at Phase 3 half of them are, and a reuse row names an item
another slice owns. This is why the check needs no phase to be told to it. It
is silent while the code is being written and it speaks the moment a row and an
item disagree, which is the moment a phase would have to guess which of the two
was meant.

**What agreement is.** A first cell holds one backticked fragment or several,
and each is compared on its own; four is the most any first cell in this
workspace holds today, and the rule sets no limit. A fragment is `name(a, b, c)` or
`Owner::name(a, b, c)`, with an optional `-> Type`. The workspace's shape
tables write argument *names*, not types, in 140 of the 159 rows that write an
argument list at all, and the reading answers with types and not names, so the
argument list is compared by count and the row's return is compared to the
source's token for token. A
fragment that writes no parentheses names a type rather than a function and is
compared to nothing.

The return arrow is the ASCII `->`, and only that. A row that writes the
Unicode `→` writes no return as far as this check is concerned, so it is
compared on its arguments alone and never refused for a return it appears to
have. One arrow is enough: a second spelling would be a second grammar for the
same thing, and the documents that reached for `→` are spelled back to `->`
rather than admitted.

A fragment names a declared function when their names are equal. A qualifier
narrows that only when it is a type: `Lld::read` has an uppercase first segment,
so it is matched against the `read` declared in an `impl Lld` and against no
other. A lowercase first segment is a module — `lld_review::run` says where the
function lives, not what it belongs to — and the reading carries no module for
a function to be matched on, so a module-qualified fragment is matched by name
alone, exactly as a bare `run` is. A fragment that matches several declared
functions fails only when it agrees with none of them: a document that writes a
name the crate declares twice is not thereby wrong, and choosing which of the
two it meant would be the resolution this slice does not do.

A receiver is not an argument, on either side, and each side drops its own
exactly once. The reading carries one entry per parameter *including* the
`self` a method was declared with, as the type tokens that receiver stands for
— `Self`, `&Self`, `&mut Self`, `Box<Self>`, read as everything here is read,
which is what makes a `&'a self` one of those four shapes and not a fifth — so
`Declared.parameters` is that list with a first
parameter of one of those four shapes removed, and every count taken from it
afterwards is already a count of arguments. The row's side is dropped where the
row is read: a first argument written `self`, `&self` or `&mut self` is not
counted. `Budget::spend(&mut self) -> bool` and
`Door::request(method, path, bearer, body, idem)`, the second of which leaves
the `&self` its source declares unwritten, therefore both agree with the method
they name: the tables use both conventions, neither is wrong, and a check that
demanded one of them would be a formatting rule wearing a correctness rule's
clothes.

Rendered tokens carry the spacing the printer chose: the source's
`Result<Lld, String>` arrives as `Result < Lld , String >`. So every token
string this slice compares is first *read*, and every one of them is read the
same way: its lifetime arguments and lifetime annotations are erased, and then
its whitespace is removed. How one side was printed and how the other was typed
are never the difference.

That order is forced, not chosen. The space after a lifetime is what ends it,
so removing whitespace first turns `& 'static str` into `&'staticstr`, one word
no erasure can take a lifetime out of, and the row's `&str` could then never be
the same shape as the source's `&'static str` — which is the case the erasure
exists for. Erased first, `&str` and `&'static str` are the same shape,
`Section` and `Section<'a>` are the same shape, and a document that spells the
lifetime is not more correct than one that does not. What an erased
lifetime takes with it is not a character but an argument. The
list arrives already split by the printer's own spacing, never a reader's:
`Map<'a, 'b, K>` is read as `Map < 'a , 'b , K >`, each argument set off by a
space on both sides of its comma and its brackets. Each list is then written
afresh from whichever arguments it still has, joined by commas, inside its
brackets — never edited in place. `Cow<'a, str>` keeps one argument and is
written from it: `Cow<str>`. `Map<'a, 'b, K>` keeps one out of three the same
way: `Map<K>`, with nothing further to say about the second lifetime erased. A
list left with no arguments at all is not written at all, and its brackets go
with it: `Section<'a>` keeps none and is written as `Section`, never
`Section<>`. That is a list *emptied* by erasure; a list *written* empty is a
different case, with no argument there for erasure to take, and keeps its
brackets verbatim — `Section<>` in the source is not the `Section` an erased
`Section<'a>` produces. `&'static str` reads as `&str` because an annotation
sits in no argument list, so there is nothing to re-write.

`Self` is substituted on both sides with the declaration's owner — the self type of the `impl` block the function was read from, or the
name of the `trait` that gave it a body — which the reading carries beside the
signature. The owner is read the same way both sides are before it is
substituted anywhere, because it is tokens that same printer produced: the
owner of an `impl<'a> Returned<'a>` arrives as `Returned < 'a >`, and a row's
`Self` standing for that would refuse a document that wrote `Returned`
correctly. A `Self` in the row's return and a `Self` in the source's each
become that type, and a free function has no owner, so a `Self` in a row about
one stands for nothing and the row disagrees. A trait's own name is what `Self`
stands for in a default method's signature as the source wrote it, which is the
comparison a row about that method is making. The
fragment's own qualifier is never the substitution — a row may write
`Lld::read` against a `read` declared in an `impl` on some other type, and it
is that other type the comparison must use. So
`Lld::read(project, slice) -> Result<Lld, String>` and
`fn read(…) -> Result<Self, String>` are the same signature said twice. Over
this workspace's 106 return fragments those two rules turn ten disagreements
into two, and both are a document claiming a function is infallible where the
code returns a `Result`.

Nothing here compares parameter names or parameter types. The reading answers
with types and the tables write names, so the two never meet; a check that
compared them would need every table rewritten before it could pass, which is
the cost that made this slice measure before it chose. What a wrong argument
*name* costs is a reader's confusion, and that is the reader's to report.

**What a disagreement says.** A failure names the check, the document, and the
row's line, as every failure of this slice does; its message names the item's
file, what the row said, and what the source said, in that order — the row is
what a human is about to fix and the source is the evidence they will fix it
against. The two are in different files and a failure carries one path, so the
document's is the path and the source's is in the sentence, as an artifact
check's failure already names what it found before it quotes the rule. That
message has one maker, `agreement_failure`, beside `failure` and
`artifact_failure`.

**Which rows are about another slice's items.** A reuse row is decided from its
first cell and from names, never from resolution. A backticked fragment is a
reuse row when its path has two or more `::`-separated segments, its first
segment starts with a lowercase letter, and that first segment is neither the
slice's own name in module form nor the name of a sibling inside the slice's
directory, nor the name of a workspace member's crate. A cell writing
`layout::lld_path` is one, because `layout` is a sibling module of the crate
rather than a module under the slice; `lld_review::run` is not, because
`lld_review` *is* the slice, spelled as a path spells it. An uppercase first
segment is never a reuse row: `Lld::read` and `Step::LldChecks` are qualified
by a type, and a type the slice
does not declare is introduced by the role cell that says whose it is.

The rule is about same-crate paths, and a crate name takes a fragment out of
it. `cargo_lid_rs::catalog::…` inside the catalog slice's own document,
`lid_rs::__private::tracing`, `lid_rs_macros::expand::observed` in a companion
crate's document: each is a path into a crate and not into a sibling module, so
no link is asked for and rustdoc would need the crate to be a dependency before
one could resolve. Which names those are is a fact and not a judgment — the
workspace members `cargo metadata` reports, which the project already lists —
read with hyphens as underscores, since `lid-rs-shape` is the package's name
and `lid_rs_shape` is the path's.
The role cell itself is never read — a row is about what its first cell names,
and its prose legitimately names anything, including the crate that does the
reading.

The siblings are a filesystem listing and no more than one, read as module
names: the entries one level inside the slice's directory, a file's stem
without its extension and a directory's own name, with `mod.rs` left out
because it names the slice itself rather than anything under it. The listing is
empty for a slice with no directory to read, whether because no crate holds the
slice at all or because its code is the one file `src/<module>.rs` — one
sentence covers both, and in both a qualified first cell is a reuse row and is
asked for its link, as it would be anywhere else. It tells a submodule of the
slice from a module elsewhere in the crate, which is the whole of the
distinction the rule needs, and it resolves nothing — a name is a name there
whether or not a `mod` declaration mentions it, and whether or not the item
behind it exists.

**A row that cannot be skeletonised is refused before the claims are derived.**
Phase 3's skeleton is the Shape table turned into signatures with `todo!()`
bodies, so a row's return type is a promise that such a signature compiles. Two
shapes break it. `-> impl Trait` infers its hidden type from the body, and a
`todo!()` body has type `!`, which implements nothing —
`fn a() -> impl Iterator<Item = u8> { todo!() }` fails to compile, and the same
signature with a real body compiles, so the defect is the skeleton and not the
design. `-> dyn Trait` is unsized in return position and fails for a different
reason with the same consequence. Both are refused at Phase 1, naming the row,
because the alternative is a worker three phases later choosing between writing
a body the phase forbids and amending a document the phase may not touch.

Neither shape appears anywhere inside a type. `-> Box<dyn Trait>` and
`-> Result<Box<dyn Error>, String>` are ordinary skeletonable returns and hold;
the check is about the return's outermost form and nothing deeper. Over this
workspace's 106 return fragments it fires on none, which is what a check
preventing a class of defect looks like before the defect happens.

**Which phase refuses it.** `phase-check 1`, through the step that runs these
checks: a `phase 1:` commit is where a document is changed, and a Phase 8
amendment re-runs Phase 1, so the amendment that moves an item is refused at
the commit that would have left the row behind. Drift the other way — a Phase 4
that changes a signature the document names — is not caught until that slice's
next Phase 1, and closing that is a step in phase 7's plan, which is the phase
slice's to add (see Deferred).

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
  deferred? *(Prescribing those headings mechanically would refuse ten of
  the thirteen documents the Behaviour's measurement covers, which are not
  wrong; a reader can say when a structure genuinely hides something.)*
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

Two questions come from checks that were prototyped and refused. They are
written as questions because their mechanical forms fire on documents that are
not wrong, and the rate is in the question so that a reader knows what they are
being asked to judge rather than to apply:

- Does the Shape table name the items the Behaviour turns on, and does the
  prose introduce the items the table names? *(The mechanical converse fires on
  224 of 254 item-like identifiers across sixteen documents; prose names
  claims, error codes, agent tools and other slices' items, and no exclusion
  rule separates those from the ones that matter.)*
- Is the slice's public surface the surface the document describes? *(Every
  public item having a row fires on 311 of 576 items. The ones worth raising
  are the ones a caller outside the slice would reach for and not find
  described — an exported constant, an entry point the document forgot — and
  telling those from a helper that is public for a citation's sake is
  judgment.)*

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

What binds these checks to a phase is the phase slice's and stands already:
its plan for Phase 1 runs the step that calls them before its doc step, its
claim is stated over the LLD's mechanical checks, and `lld-check` takes the
slice it reads from that slice's `resolve_slice`. Widening any of it was never
a phase agent's to do — `src/phase/spec.rs` and `src/phase/` are another
slice's module, which every phase's path policy refuses — so it was the
human's commit, as every cascade into another slice has been. What this
amendment adds needs none of it again: three more checks behind a step that
already runs.

The guideline and the reader are the human's for the same reason and one
more: they ship from the `lid-rs` crate, a package outside this slice's
path policy, and they are prose a person writes rather than code a phase
derives. They exist before Phase 5, because the two artifact checks have
nothing to read until they do.

Three more things land by hand, and each has a moment.

**The dependency, between Phases 2 and 3.** The same hand commit makes
`layout::module_of` `pub`, the one-word change that lets `declared` and
`is_reuse` ask for the slice's name in module form without a second
conversion. `cargo-lid-rs` does not depend on
`lid-rs-shape` today. The workspace's dependency table already carries that
crate with both a `path` and a `version`, so the change is the single manifest
line `lid-rs-shape.workspace = true` under this crate's `[dependencies]`. No
phase agent may write a manifest, and Phase 3's skeleton does not compile
without it, so it is the main session's commit in the slot between the claims
and the skeleton — the slot this workspace already uses to land a module
declaration by hand. What it must resolve to is the version that carries
`owner` on `lid_rs_shape::Signature`; that is the shape crate's own amendment,
on its own branch, and it is on main before this slice's Phase 3.

**The guideline's checklist, with the Phase 2 claims.** `Check` gains three
variants, and *guideline names every check* fails for every variant the
checklist omits — this slice's own gate pointed at this slice's own amendment.
The canonical file is `lid-rs/skill/references/lld.md`, in another crate and
outside every phase's path policy, so the edit is the main session's, followed
by `cargo lid-rs sync` so that the project's copy matches. It lands with the
Phase 2 claims commit rather than being remembered at the gate, and it may land
early: the check asks only that the checklist name every variant the tool
knows, so a checklist naming a check whose variant Phase 3 has not written yet
passes. The checklist's own counts and its sentence about what is a document
check and what is an artifact check move with it.

**The other documents' rows, before Phase 3.** Two of the new checks fail
documents that already exist, and the handful they fail on has been swept.
Three rows disagreed with the code they describe — `policy::slice_crate` and
`hook_stop` in `cargo-lid-rs/src/phase/lld.md`, each writing a return without
the `Result` the source returns, and `tally::record` in the same document,
naming two arguments where the function takes three — and four rows named
another slice's item in the same crate without linking it, six links in all,
two rows each in `cargo-lid-rs/src/phase/lld.md`,
`cargo-lid-rs/src/headless_canopy_agent/lld.md` and
`lid-rs-pipeline/src/lld.md`. Three more rows wrote a Unicode `→` where the
grammar reads `->`. All of it is documentation the main session commits on a
documentation branch, corrected before this slice's Phase 3, so that the
workspace passes these checks before the skeleton that adds them is written.
None of it is deferred and none of it is exempted: a gate that grandfathers
what already exists never fires.

## Shape

| Item | Role |
|---|---|
| `lld_review::run(args)` | `lld-check` entry: parses `--slice`, locates the LLD, applies the checks, prints every failure |
| `Lld`, `Lld::read(project, slice) -> Result<Lld, String>` | The document as lines, with the slice and path it came from |
| `Check` | The closed set, declared in the order the table of checks states them: `DecisionsExist`, `Alternatives`, `ShapeRows`, `ShapeAgrees`, `SkeletonableReturns`, `ReuseRowsLinked`, `DeferredNumbered`, `GuidelineNamesEveryCheck`, `ReaderObservesOnly`. A declaration has no wrong answer a test could catch, so the claim about this order is cited by `check_all`, which runs them in it |
| `Failure { check, path, line, message }` | One failure: the file it is about, the line it is on, and the skill's sentence for the rule — the path matters because an artifact check's failure is about the guideline or the reader, not the document under check |
| `check_all(project, lld) -> Result<Vec<Failure>, String>` | Every check in the table's order. The error is reserved for a project whose root cannot be located — an artifact that is absent or unreadable is a `Failure`, and a slice whose crate cannot be located declares nothing and lists nothing, so neither hides the checks after it |
| `decisions_exist(lld)`, `alternatives(lld)`, `shape_rows(lld)`, `deferred_numbered(lld)` | One document check each |
| `guideline_names_every_check(project)`, `reader_observes_only(project)` | The two artifact checks: the guideline's checklist names every `Check` variant, and the reader's frontmatter declares `Read`, `Grep`, `Glob` and nothing else |
| `Table`, `table_at(lld, heading) -> Option<Table>`, `Row` | A markdown table under a heading, as rows of cells — the one parse the checks share |
| `identifiers(cell) -> Vec<String>` | The backticked identifiers a cell names |
| `Declared { name, owner, parameters, returns, file }`, `declared(project, slice) -> Vec<Declared>` | The functions the slice's own source declares: the name; for a method, the block it was read from — the self type of an `impl`, the name of a `trait` that gave it a body — and none for a free function; the parameters the reading answered, less a first one whose type tokens are the receiver's — `Self`, `&Self`, `&mut Self` or `Box<Self>` — so that every count taken from this field afterwards is a count of arguments; the return tokens as the source wrote them; and the file. Read through `lid_rs_shape::signatures` over the slice's own crate, [`layout::own_crate`](crate::layout::own_crate), and kept where the file is under [`layout::slice_dir`](crate::layout::slice_dir) or is the `src/<module>.rs` file module beside it. No `Result`: a slice no workspace member holds a crate for declares nothing, and a file the reading cannot parse contributes nothing, and neither is a failure. The claims about this shape are cited by `declared`, since data has no wrong answer to go red on |
| `Fragment { path, arguments, returns, linked }`, `fragments(cell) -> Vec<Fragment>` | Every signature fragment one cell writes, one per backticked identifier and as many as the cell holds — four is the most any first cell in this workspace holds today, and no limit is stated: the `::`-separated segments of the identifier, the arguments between the parentheses when it writes them less a first one written `self`, `&self` or `&mut self`, the type after an ASCII `->` when it writes one, and whether the cell wrapped it in a markdown link — the backtick span immediately preceded by `[` and immediately followed by `](`, as `` [`plan`](crate::phase::plan) `` is; any other neighbour means bare. The claims about this shape are cited by `fragments`, since data has no wrong answer to go red on |
| `agreement_failure(check, path, line, item_file, row, source) -> Failure` | The one shape a source-reading failure takes, beside `failure` and `artifact_failure`: the check, the document's path and the row's line as every failure carries them, and a message that names the item's file, what the row said and what the source said, in that order, before the rule. The claim about that message is cited here |
| `shape_agrees(project, lld) -> Vec<Failure>` | The first cells whose fragment names a function the slice declares and agrees with none of the declarations it names, each failing on its own row's line |
| `matches(fragment, declared) -> bool` | Whether a fragment names one declared function at all: equal names, and the declaration's owner too where the fragment's qualifier is a type — an uppercase first segment, the qualifier and the owner compared as read, since the owner of an `impl<'_> Fields<'_>` is printed `Fields < '_ >` and a row's `Fields` would otherwise narrow to no declaration at all. A lowercase qualifier is a module and no owner, so it narrows nothing |
| `agrees(fragment, declared) -> bool` | One fragment against one declaration it names: the fragment's argument count is the declaration's parameter count — the fragment's own `self` argument already dropped where the cell was read, the declaration's receiver already dropped where the source was read, so a row writing `&mut self` and a row leaving it out both agree with a method; and the fragment's return, where it writes one, is the source's |
| `same_type(row_return, source_return, owner) -> bool` | The row's return type against the source's, compared as a reader compares them: each side read the same way — lifetime arguments and lifetime annotations erased, then whitespace removed, in that order — and `Self` substituted on both with `owner`, itself read that same way, the declaration's `Declared.owner`, never the fragment's own qualifier, and none for a free function |
| `reuse_rows_linked(project, lld) -> Vec<Failure>` | The rows whose first cell names another slice's item as a bare identifier rather than an intra-doc link, whose resolution is the doc step's |
| `siblings(project, slice) -> Vec<String>` | The module names one level inside the slice's directory: a file's stem without its extension, `mod.rs` left out, and a directory's own name — a listing, which resolves nothing. Empty for a slice with no directory to read, whether no crate holds it, its code is one file module, or the directory cannot be read |
| `is_reuse(fragment, slice, siblings, crates) -> bool` | One decision over one fragment: two or more path segments, a first segment starting with a lowercase letter, and that segment none of the slice's own name in module form, the siblings, or the crate names |
| `crate_names(project) -> Vec<String>` | The workspace members' names as a path spells them: `Project::member_manifest_dirs` for the members, `Project::package_at` for each one's package name, hyphens read as underscores |
| `skeletonable(returns) -> bool` | One decision over a return fragment: whether a signature written with it and a `todo!()` body compiles — false for an `impl Trait` return and for a bare `dyn Trait` one, true for every type that merely contains them |
| `shape_returns(lld) -> Vec<Failure>` | The rows whose return fragment cannot be skeletonised, each failing on its own line |
| `lid-rs/skill/references/lld.md` | The guideline: the checklist and the reader's questions |
| `lid-rs/agent/lid-rs-lld-review.md` | The reader, read-only, advisory |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| What blocks | The mechanical checks alone | The agent's findings block too; nothing blocks | A gate an agent decides is not a gate (`phase/lld.md`), and this is the phase whose artifact the human owns — refusing a human's document on a model's opinion inverts that. What can be refused without judgment is refused; the rest is reported. |
| Where the checks run | A step of `phase-check 1`, and a subcommand of their own | The subcommand alone, called by CI; a git hook | As a step they are reached by every host, by CI, and by the phase agent's stop hook, for free. As a subcommand they answer the question while the document is still being written, which is when it is cheapest to fix. |
| What the checks are | The set the table states, each a property with no judgment in it and each measured over this workspace's documents before it was written down | Six, of which two were a required section structure and "every Shape item appears in the prose"; a schema the document must validate against | No check is here on an author's confidence: what the measurement said is in the table's evidence column, and a candidate that refused documents which are not wrong was moved to the reader rather than softened. The two dropped that way fire at the rate the Behaviour records — ten documents of thirteen, and nineteen passages — and a check with that rate teaches authors to write for the checker. The checks that read the slice's code came back the other way round — three rows that disagree with their own source, and four that name another slice's item in the same crate without a link — which is why they are checks and not questions. |
| The guideline's contents | A checklist plus questions, each with the evidence that put it there | A style guide; a template to fill in | The skill's `discipline.md` is written this way and it is the file that changed behaviour, because a rule with its incident attached is a rule a reader believes. A template produces documents shaped like the template. |
| Guideline and agent | One file, the agent's body pointing at it | The agent's body carries the checklist | Two copies of a checklist drift, and the skill's sync rule already keeps one copy honest. |
| The reader's tools | `Read`, `Grep`, `Glob` | Also the LSP; also the ability to write a report file | It reads one document and the skill; anything it could write is a finding it should say instead. |
| Existing LLDs | The document checks were narrowed until every existing LLD passes them; the checks that read the code fire on the documents that are wrong, and those documents are corrected before this slice is gated | Narrow these two as well, until nothing fires; grandfather the documents that exist; ship the checks and defer the corrections | A gate that exempts what already exists never fires, and a gate that fails what is already *good* is a gate authors learn to route around. The two rules stop being in tension once the measurement is in hand. The document checks that were dropped fired on documents that are fine — a structure of their own choosing, a type introduced by description — so narrowing was right there. The code-reading checks fire on three rows that disagree with their own source and four that name another slice's item in the same crate with no link, and those are defects, so they are fixed rather than exempted: a sweep the main session committed on a documentation branch before this slice's Phase 3. No deferral stands in for them, because a deferral here would be a known divergence hiding behind a numbered list. |
| Naming | `lld-check` for the mechanical checks | `lint`, `doc-check`, folding it into `phase-check 1` with no subcommand | It says which document and which kind of check; `phase-check 1` remains the gate, and this is the same question asked early. |
| Which converse is mechanical | Rows agree with the items the slice declares | Every item the prose names has a Shape row; every public item of the slice has one; both, with an out-of-scope rule | The two that were dropped were prototyped and run over this workspace's sixteen Shape tables before the choice was made. *Prose names an item* fires on 224 of 254 item-like identifiers, and 106 survive five exclusion rules — claim names, error codes, agent tool names, standard library types, other slices' paths — because prose legitimately names all of those. *Every public item has a row* fires on 311 of 576 items, and on 54 of 221 once the two largest slices are set aside; a slice's public surface is public for a citation's sake as often as for a caller's, and the reading cannot tell a test helper from an entry point. The surviving check fires on three rows in this workspace and each of the three is a document that had stopped describing its code. A gate that fails what is already good is a gate authors route around; a gate that fires three times on three defects is the one this slice was measured into. |
| How the code is read | `lid_rs_shape::signatures`, the pass that already reads one crate's tokens and resolves nothing, at the version that carries `owner` | A `syn` pass of this slice's own; `cargo doc --output-format json`; a scratch crate the rows are emitted into and `cargo check` run over | Rustdoc's JSON is nightly-only — spiked on this machine's cargo and rustc, which refuse `--output-format json` and `-Z unstable-options` by name — and README constraint 1 rules out a nightly toolchain. A second `syn` pass would be a second answer to a question one crate in this workspace already answers, and the shape slice's reading is the one the conformance checks join against, so two readings would have to be kept agreeing forever. It answers in 70 ms over this workspace's largest crate, which is below the noise of the step it runs in. One thing it has to answer that it did not is the self type of the `impl` a method was read from, without which `-> Self` cannot be compared to the row that spells the type out; that is an amendment to `lid_rs_shape::Signature`, made on the shape crate's own branch and landed before this slice's Phase 3, and asking for it there is cheaper than a second reading here that would have to agree with the first forever. What it still does not answer is a parameter's name, a struct's fields and an enum's variants, and whether an item is under `#[cfg(test)]`: the first is why arguments are compared by count, the second is why no check here compares a row's members, and the third is why no check here is stated over test-support items. It also does not say which parameters were receivers, only what type each stands for, and it reads a first parameter's type the same way it reads every token here: lifetimes erased, then whitespace removed. So an associated function whose first parameter's tokens, once so read, are one of those four receiver shapes is read here as a method and loses an argument from its count — a plain `Self` by value was always caught this way, and now so is a first parameter genuinely borrowed and lifetime-annotated, `&'a Self` or `&'a mut Self`, which the erasure collapses into the same shape as `&Self` or `&mut Self`. That imprecision is stated rather than guarded against: this workspace declares no such function, and the alternative is a receiver flag on `Signature`, which is a second amendment to another crate for a case nobody has written. |
| Where a reuse row's path is resolved | By rustdoc, at the doc step this phase already runs, over rows written as intra-doc links | By this tool, from the registry; by this tool, by parsing the named crate's source; not at all, as today | A path this tool resolved would be a second implementation of name resolution, which constraint 2 forbids and which would diverge on the first `use` rename. A path written as `` [`plan`](crate::phase::plan) `` is resolved by the compiler's own resolver, and `phase-check 1` already runs `cargo doc` with `-D rustdoc::broken_intra_doc_links` — spiked: an unresolved link inside an LLD included by `#![doc = include_str!("lld.md")]` fails that step by name. So the tool's share is one text property, *a reuse row is linked*, and the resolution is free and correct. The cost is a convention this workspace did not follow: 0 of 322 shape rows were written as links, and four of them are reuse rows under the rule below — six links across three documents, which was one documentation commit. Leaving it unchecked was measured too — a row naming a moved item is exactly the drift the amendment exists to catch. |
| Which rows a link is demanded of | A first cell's fragment whose path has two or more segments, whose first segment starts with a lowercase letter, and whose first segment is none of the slice's own name in module form, a file stem or directory inside the slice's directory, or a workspace member's crate name | Every qualified path, uppercase first segments and crate-qualified paths included; every path this tool cannot find among the functions the slice declares; a list of the slice's own modules kept in the document | The rule has to be decidable without resolving anything, and these four tests are: two read the path's own text, one is a directory listing, and one is the member list `cargo metadata` already reports. A lowercase first segment is a module or a crate, so the only question left is whose, and a name inside the slice's directory is the slice's own — which is what separates `lld_review::run` and a submodule of this slice from `layout::lld_path`, a sibling module of the same crate. The rule is deliberately about same-crate paths: `cargo_lid_rs::catalog::…`, `lid_rs::__private::tracing` and `lid_rs_macros::expand::observed` name other crates, and rustdoc cannot resolve a link into a crate that is not a dependency, so demanding one would demand a link that cannot be written. Demanding a link of every qualified path would also demand one of `Lld::read` and `Step::LldChecks`, whose first segment is a type: a type the slice declares needs no link, and one it does not is introduced by the role cell that says whose it is. Deciding from what the slice declares would make the answer move as the code is written, so a row would be a reuse row at Phase 1 and not at Phase 4. A list kept in the document is a second copy of the directory, which drifts the first time a module is added. |
| What erasing a lifetime does to its argument list | The list is written afresh from the arguments it still has, joined by commas, inside its brackets | Editing the tokens in place — removing the erased lifetime's comma, and then the list's brackets too if it came out empty; erasing lifetimes only after whitespace is already removed | A Phase 4 worker wrote the in-place split: drop the lifetime, drop its comma, then check whether the list came out empty and drop its brackets too. A review rejected it, because that split cannot be one unit of work — removing a comma from a list that still has other arguments and removing a list that has none are two different edits, and something has to decide which applies before either runs, and that decision existed in the code with no claim written for it. It also had no answer once a second lifetime was in the row: `Map<'a, 'b, K>` strands two commas when the first is erased, and whether removing one re-strands the other is a rule the in-place framing needs and never states. Erasing whitespace first was rejected earlier and for a different reason, stated above: the space after a lifetime is what ends it, so stripping it first welds `& 'static str` into `&'staticstr`, a word no erasure reaches. Writing the list afresh from what remains needs neither rule: a list of one, a list of none left by erasure, and a list of three are written the same way, by joining whatever arguments are left — a list written empty to begin with had no argument for erasure to take, so it keeps its brackets verbatim rather than being written afresh at all. |
| What a return fragment is compared as | The tokens the source wrote, read on both sides the same way — lifetimes erased, then whitespace removed — with `Self` substituted on both with the declaration's owner, read that way too | Plain equality over the rendered tokens; resolving both sides to a type; comparing only the outermost constructor | The reading renders tokens with the spacing its printer chooses — the source's `Result<Lld, String>` arrives as `Result < Lld , String >` — so both sides are read the same way before anything is compared, lifetimes erased and then whitespace removed, since the space after a lifetime is what ends it and stripping first would weld `& 'static str` into a word no erasure reaches. How one side was printed is never a difference. Measured after that reading, plain equality fires ten times on this workspace, and eight of the ten are a document spelling `Result<Lld, String>` where the source wrote `Result<Self, String>`, or omitting a lifetime the source annotates — differences with no reader who cares. Resolving both sides is the resolution constraint 2 forbids. Comparing only the outermost constructor would pass `Result<PathBuf, String>` against `Result<String, String>`, which is the class of error the check exists for. With the normalisations the check fires twice, both a document that claims a function is infallible where the code returns a `Result`. |
| Which phase refuses a disagreement | Phase 1's, where the document is committed | Phase 7's, with the gate; every phase's | A document is changed at Phase 1 and at no other phase, so Phase 1 is where a row that stopped describing its item is cheapest to fix and where the human who wrote it is present. A Phase 8 amendment re-runs Phase 1, which is the case this check was measured on. What Phase 1 cannot catch is a signature changed at Phase 4 under a document nobody re-read; catching that is a step in phase 7's plan, which is the phase slice's commit and not this one's — the same boundary this slice's Cascade section already draws. |
| How a row is tested for skeletonability | The return fragment's outermost form, refused for `impl Trait` and bare `dyn Trait` | The rows emitted as `todo!()` stubs into a scratch crate under the target directory and `cargo check` run over it; the fragment parsed as a `syn` signature; the fragment's return parsed as a `syn::Type` | The stub crate was measured and cannot be built from these tables: 13 of 426 shape-row identifiers parse as a Rust signature, because the grammar the workspace writes is a name, *argument names*, and an optional return — 140 of 159 parenthesised cells have no type on any argument. Emitting stubs would mean either rewriting every table into Rust or inventing types for the arguments, and a stub whose arguments were invented type-checks nothing about the row. It also cannot see what it would be for: at Phase 1 the slice's own types do not exist yet, so the crate would have to declare the table's own types to compile at all, and a scratch crate with a path dependency on the slice's crate reaches only its public items — spiked: a private function is `E0603` from outside. And between Phases 3 and 5 the slice's crate does not compile, so the dependency's own check would fail for reasons that have nothing to do with the table. The `syn` alternatives are cheaper and still unnecessary today: all 106 return fragments in this workspace parse as a `syn::Type` and none is malformed, so parsing would buy a dependency to detect a class that has never occurred, while the two unskeletonable forms are a prefix test over the fragment's first word. Start with the test that needs nothing; the `syn` parse is one call away when a malformed return is ever seen. |
| When the refusal happens | Phase 1, with the other document checks | Phase 2, when the claims are derived; Phase 3, when the skeleton is written and fails to compile | Phase 3 is where it is found today, and the cost is the whole point: a worker meets a row it cannot write, and the fix is an amendment to a document its phase policy forbids it to edit, so the phase stops and the human is asked. At Phase 1 the same defect is a sentence and the human is already there. Phase 2 is no better than Phase 3 for this, because a claim derived from an unwritable row is a claim about a function that cannot exist. |

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
3. The agreement checks run at Phase 1 and nowhere else, so a signature
   changed at Phase 4 under a document nobody re-read is not refused
   until that slice's next Phase 1. Closing it is `Step::LldChecks` in
   `plan(Phase::Seven)`, which is the phase slice's commit — and it would
   put every document's agreement inside the gate's longest phase, so it
   is worth measuring the added time before it is added.
4. Parameter names are not compared, because the reading answers with
   types and the tables write names. A row whose argument names have
   drifted from the source's passes every check here. What would close it
   is a name alongside each type in `lid_rs_shape::Signature`, which is
   the shape slice's amendment and not this one's.
5. An item under `#[cfg(test)]` is indistinguishable from a production one
   in the reading, which is why no check here is stated over test-support
   items. The reading answers 71 functions for this slice's directory, of
   which 58 are the tests and their helpers. A `#[cfg]` fact on
   `lid_rs_shape::Shape` would separate them.
6. Only the return is tested for skeletonability. An argument's type is
   not tested because the tables do not write argument types — 140 of 159
   parenthesised rows write names alone — and a row whose *arguments*
   cannot be written is not a shape this workspace has produced. If the
   tables ever carry argument types, the same test applies to each of
   them and the check is one loop wider.
7. A row that writes a type's members — `Failure { check, path, line, message }`,
   `Halt::{Quiet, Refused}` — is not compared to the type. The reading
   answers functions and their signature tokens, and carries no struct
   field, no enum variant and no `impl` block of its own, so there is
   nothing for a member to be compared against. The measurement says
   there is no hurry either: 0 of this workspace's 28 member rows name a
   member the type does not declare, so the check would fire on nothing
   today. What would close it is fields and variants carried alongside
   the signatures in `lid-rs-shape`, which is that crate's amendment and
   not this one's.
8. A qualifier is matched to the declaration's owner as both are read, and
   reading erases lifetimes only — never a type argument. So `impl<'_>
   Fields<'_>` reads as `Fields` and a row's `Fields` matches it, which is
   the case `matches` is written for; but `impl<T> Shape<T>` reads as
   `Shape<T>`, and a row's bare `Shape` never equals that, so the fragment
   matches no declaration, is compared to nothing, and the row holds
   whether or not it agrees. This slice's own fixture carries the case:
   the row `Shape::of(&self, held) -> Shape<T>` against `impl<T>
   Shape<T>` holds by matching nothing, not by agreeing. Closing it would
   mean erasing type arguments from the owner comparison too, which is a
   design change and not a code fix. Impact today is nil: this workspace
   declares only one lifetime-generic `impl` and one blanket `impl`.

## References

- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (Phase 1 is human-owned; a decision with no alternative is not yet examined).
- `docs/intent/phase/lld.md` — the phase checks this adds a step to, and
  the rule that a gate is never an agent's choice.
- `docs/intent/sync/lld.md` — the strict mirror rule the guideline and the
  reader ship under.
- `.claude/skills/lid-rs/references/phase-1.md` and `discipline.md` — what
  the guideline deepens, and the evidence-table form it follows.
