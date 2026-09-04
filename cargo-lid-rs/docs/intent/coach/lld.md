# coach — an LLD is written by interview, and checked before it is committed

## Context and Design Philosophy

Everything the walk does is derived from the LLD, and the walk's most
expensive failure is an LLD that turns out to be wrong. The slice that
built the canopy client amended its LLD twelve times while it was built;
each amendment was a phase stopping, a human re-reading a document, and a
worker starting again. Roughly half of those a reader could have found in
minutes from the document alone, which is why the `lld-review` slice put
a checklist and a reader behind `phase-check 1`.

That reader works on a document that already exists. It says what is
wrong with what you wrote. It cannot help you write it, and the failures
it finds most often — a path that can fail with no stated outcome, an
external contract with no provenance, a rule stated at a granularity no
claim can be cut from — are failures of *elicitation* rather than of
prose. They are the questions nobody asked before the writing started.

This slice puts an interviewer in front of the writing. `cargo lid-rs
coach` opens a canopy session whose system prompt is two things: how to
interview — ask before drafting, one decision at a time, surface what the
author has not said rather than confirming what they have — and the
guideline the reader applies, so the questions it asks are the questions
the document will be judged by. The human answers in their terminal. When
the model has enough, it writes the LLD; the client then runs the
document checks and the reader over what was written and lands their
findings as the next turn, so the document is revised against its own
judges rather than against the model's memory of them. The loop ends when
the human ends it.

An interview is watched, which is the other thing that shapes this slice.
Every other host in this crate runs unattended, and silence costs it
nothing; here a person is sitting in front of a terminal deciding whether
the thing is still alive. So the run shows its work — the model's own words
before each tool call, and a line naming every call — and it spends as few
of those calls as it can, by landing what the repository holds in the
opening rather than making the model go and find it one directory listing
at a time.

Two things it deliberately does not do. It does not commit: Phase 1 is
human-owned, and a coach that commits the LLD has approved it. And it
does not decide: every answer it writes into the document came from an
answer the human gave, which is what makes the document theirs to commit
and defend at a review.

The output is the LLD, at the path the walk will look for it, ready for
the human to read once more and commit as `phase 1:`.

## Behaviour

### `cargo lid-rs coach [--package <name> | --workspace] [--slice <name>] [--door <url>] [--max-cost <amount>]`

Opens one canopy session and holds it for as long as the conversation
lasts. The slice defaults to the current branch's name with `lld/`
removed, as every other subcommand reads it; `--door` defaults to
canopy's production door, `https://api.canopyhq.dev`; `--max-cost`
defaults to five. The API key is read from `CANOPY_KEY`, which must hold
the faces `converse`, `execute` and `stop` — the coach lands tool results
as the canopy client does — and never appears on a command line; unset,
the run stops naming the variable. Any other argument is rejected by
name.

### Where the document goes, and why the package is named

The walk finds a slice's LLD by looking for the package whose directory
already holds it, which a slice with no document has no answer for. The
coach could infer a package from the directory the command was run in,
and must not: `cargo metadata` reports no current package, so inferring
one means a new external contract or a walk up the filesystem, and worse,
`slice_crate` takes the *first* member that holds a document of that
name, so a document written under a guessed package is not refused later
— it silently becomes the crate the whole slice is built in.

So the package is named. `--package <name>` is required when the
workspace has more than one member and defaults to the only one when
there is exactly one; a name matching no member stops the run listing the
members. The document is `docs/intent/<slice>/lld.md` under that member's
manifest directory.

A slice whose product is the workspace rather than a crate — `book`,
`publish` and `skill` here — has its document at that path under the
workspace root, where no package holds it. No `--package` value can name
that place, because a virtual manifest defines no package, so `--workspace`
names it instead, and the two flags refuse each other. The path is printed
when the session opens and again at the end, because it is the one thing a
human can get wrong here that no later check would question.

### What the opening carries

The coach reads through a tool, one call at a time, and every call is a
record landed on a tail the client then waits for. A model that must find
the workspace's documents before it can ask anything spends its first
minutes walking directories — `.`, then a member, then `docs`, then
`intent` — because a listing is what `read` answers for a directory and
there is no cheaper way to learn a path. That walk is the same walk on
every run in a given project, and it produces the same answer every time.

So the coach does it once, and the opening carries what it found. Before
what this run is, the first user message carries three things about the
repository the coach is standing in:

1. **The intent index** — every `docs/intent` document in the workspace:
   for the workspace root and for each member, the HLD if one is there and
   each slice's `<slice>/lld.md`. Paths, not documents; the point is that a
   `read` can be aimed rather than searched for. The row naming this run's
   own document is marked as such — it is the one path in the index the
   model is about to write rather than consult.

   Each row names its document **relative to the workspace root**, which is
   the form `read` and `grep` take: `confine` refuses an absolute path as
   written, before it resolves anything. An index in any other form would
   cost the model a refused call per row until it worked out what to strip,
   which is the search this section exists to remove.
2. **The HLD, whole**, when the index found exactly one. It is the design
   every slice sits inside, it is read on every run that goes well, and it
   is short enough to hand over. When the index found several — a workspace
   whose members each carry one — the opening carries none: which of them
   governs this slice is a design question with a human's answer, and a
   coach that picks one has decided it silently. Named in the index, it is
   one `read` and one `ask` away.
3. **The project's guidance, whole**: `AGENTS.md` at the workspace root, or
   `CLAUDE.md` when there is no `AGENTS.md`, or nothing when there is
   neither. `init` writes the first and a `CLAUDE.md` that imports it, so
   the fallback is for a project that arrived at the methodology by another
   road, this workspace among them.

Nothing else is landed unread. The neighbouring LLDs are named and not
carried: which of them this slice sits beside is exactly the thinking the
interview is for, and a first message carrying twelve documents buries the
one that mattered.

### The conversation

The model speaks first. The opening — what the slice is, and the existing
document whole when there is one — is landed as the conversation's first
user message before the human is ever read, and the turn it settles is
the coach's first question. This is the interview the method describes:
the coach has the guideline, the method and whatever document already
exists, and a coach that waited to be prompted would make the human open
a conversation they came here to be led through.

It is also what makes the loop drivable. A first turn that needs no input
can be scripted, so the printed answer, a halt, and the session's stop can
each be shown without a terminal — and a run whose input is already at end
of file still opens, settles one turn, and ends, rather than returning
before a turn exists.

Thereafter the human types; their text is landed as an
`app.client.user_message`, and the model's settled answer is printed. The loop ends when the human
types `done` alone on a line, or when their input reaches end of file;
both are answers a blocking read returns, and both stop the sessions
before the client exits, because a session left open is an unsealed log.

Between the two, one thing happens the human did not ask for. When a turn
called `draft`, the client runs the document checks and the reader over
what was written and lands their findings as the next user message,
rather than waiting for the human to notice them. A turn that drafted
nothing is answered by the human. A `draft` that failed is not a turn
that wrote the document, and the judges do not run.

The judges answer at most one drafting turn in a row. If the model drafts
again in reply to them, that turn is answered by the human, who can read
what changed and say whether to press on. Without that bound a model and
its judges converse without the human, spending a reader session a turn
at a pace nobody chose; with it, every second drafting turn is a decision
the human makes.

That the turn drafted is known because the coach executes its own tools.
The executor is handed to the turn rather than kept by the session — a
session that owned it could not also be owned by the thing it writes to —
so `drive` takes it for the turn and the coach's closure records that
`draft` was called and whether it succeeded. Nothing is inferred from the
model's words.

The conversation ending inside `ask` is recorded the same way, and for a
sharper reason. A human who types `done` at a question the model asked has
ended the conversation mid-turn: the model is still owed an answer, gets a
tool error, and settles. The loop must then stop without reading the human
again — they have already said they are finished, and asking twice is the
one thing that would make the tool worse than a prose question. Nothing
else in the turn can carry that fact, because `ask` answers the provider
and returns nothing to the loop, so the record the executor already keeps
is where it goes.

### What the human sees while a turn runs

A turn can spend minutes on tool calls, and until it settles the coach has
nothing of its own to print. A human watching an interview that has gone
silent cannot tell reading from a hang, and the honest answer is to show
the work rather than to claim it is happening.

Two things are printed while the turn runs. The model's own text, whenever
it says something before calling a tool — the canopy client's turn loop
hands that text to a narrator, and the coach's narrator prints it — because
that is where a model says what it is about to look for, and it is worth
more to the human than any summary the coach could invent. And one line per
tool call, printed by this module's executor before it routes the call: the
path a `read` names, the pattern a `grep` names, the document a `draft` is
about to replace. `ask` announces nothing; the question it prints is its
announcement, and a line above it would only push the question up the
screen.

A call whose arguments do not carry what its line would name announces
nothing. It is a call about to fail and say so in its own sentence, and a
line guessing at what it meant would be a second, worse account of one
fault.

The reader's session is announced the same way and narrates nothing. Its
calls are printed, because the judging is the other place a run goes quiet
for minutes; its prose is not, because what the reader has to say is landed
whole a moment later, and the human should read the findings rather than a
draft of them.

### The four tools

| Tool | Arguments | Does |
|---|---|---|
| `read` | `path`, optional `offset`, `limit` | The canopy client's own read, over a path its own `confine` has bounded — the same answer a phase worker gets, because there is no reason for two |
| `grep` | `pattern`, optional `path`, `glob` | That client's own grep, under the same confinement: a literal, case-sensitive substring, at most two hundred lines |
| `draft` | `content` | Replaces the slice's LLD with `content`, creating its directory; answers with the path and the bytes written |
| `ask` | `question`, optional `options` | Puts one question to the human and answers with what they typed. Given `options`, they are printed numbered beneath the question and a bare number answers with the option it names; anything else is answered verbatim, because a closed question the human wants to answer differently is a question that was wrong |

`read` and `grep` are the client's tools whole, schema included: the
description the model reads comes from that client's `schema_of`, two
descriptions of one tool being two things nothing keeps in step. `draft`
and `ask` are the coach's own and have no other home.

The two the coach must not have are that client's `edit` and `write`.
Everything this run writes is one document, through `draft`; a coach
holding a general write is a coach that can produce work no gate has seen.

`draft` writes one path and no other: not the module, not the claims, not
a manifest. What the coach produces is a document, and the phases produce
everything else. It answers with what it wrote and not with a verdict:
the checks belong to the judges' turn, so that they are run once and read
once.

It replaces the document whole — there is no partial edit — so a slice
whose LLD already exists has it read into the first user message, and the
model is told it is amending rather than writing. A coach run on a
committed LLD is how a Phase 8 amendment is drafted; the commit is still
the human's.

A forward naming an `op` the coach did not declare is refused with that
name, as the canopy client refuses one: a tool set is what a session may
call, and a call outside it is a fault to report rather than a request to
interpret.

`ask` is how the interview happens on the model's initiative rather than
the human's. A question in prose is a turn the human may answer past; a
question through a tool is a prompt they cannot miss and an answer the
sealed log records beside its question.

Its `options` are an offer, never a constraint. A model that has narrowed
a decision to three answers should say so, and a human who wants a fourth
should not have to fight the interface for it — so a bare number picks an
option and anything else is passed through as typed. The alternative,
refusing what is not on the list, would make the tool decide which
answers a design may have.

### What `ask` costs, and its bound

`ask` blocks the turn on a person, and the turn is a forward the door is
waiting to see completed. Canopy halts a session whose invoke has gone
unanswered for fifteen minutes, so a question left that long ends the
conversation with the platform's own sentence. The question is printed
with that fact beside it, once, when it is asked — a warning that arrives
later would need a clock running beside a blocking read, and a
conversation is not worth a thread.

While a question is outstanding no tail read is in flight, so the client
cannot see a halt until it lands the answer. That is the cost of asking a
person inside a tool call. The drafted document survives either way,
because `draft` writes to disk rather than to the session.

A human who ends the conversation while a question is outstanding — `done`
or end of file, typed as the answer — is answered to the model as a tool
error saying the conversation is over, because a tool has no other way to
speak. The model's turn then settles, and the loop ends as it would have
anyway. `ask` therefore fails the way every tool fails, with a sentence,
and no ending travels out through a channel the door does not have.

### What the judges say

After a turn that drafted, in this order:

1. The **four document checks** of `lld-check`, over the path `draft`
   wrote rather than over a path resolved again, every failure rendered
   as it renders them for a human. The two artifact checks are not here:
   they are about the project's synced guideline and reader rather than
   the document, and a model whose only writing tool is `draft` cannot
   act on them.
2. The **reader** — a second canopy session, dialled with the reader's
   synced body as its `system` and the canopy client's three observation
   tools, executed by that client's own dispatch with no phase and no
   tally, given the document and asked for findings as the reader's
   definition asks. It is a fresh session each time, because a reader that
   remembers its last reading reads the diff rather than the document a
   phase will receive, and it is stopped when it answers.

A document that cannot be read back is the same kind of answer. `draft`
reported the bytes it wrote, and the checks read the path again to run
over it; between those two moments a file can go away. When it does, the
judging says so — the read's own sentence stands in place of the checks'
failures — and the verdict it reports is that the checks do not hold,
because a verdict is a statement about checks that ran and none did. It
does not stop the run: the human is mid-interview, the model is owed an
answer, and a judging that could not read what was written must still
say that rather than nothing.

A reader that cannot be read — halted, refused, quiet, or out of budget —
does not end the conversation the human is in the middle of. Its sentence
is landed in place of its findings, saying the reader could not be
consulted, and the human is told; the document checks are landed
regardless.

Both are landed as one user message under a heading that says what it is,
so a reader of the sealed log can tell the judges' turn from the human's.

A reader session prints its own opening line, because opening a session
prints one unconditionally and that is the canopy client's rule rather
than this host's to change. So the coach prints the judging's heading to
the terminal *before* it opens one: the line that follows belongs to the
judging, and is not mistaken for the coaching session reopening.

### Before the interview

The two artifact checks run once, before the first question. A reader
whose frontmatter declares more than the observation tools is reported to
the human and the run continues, because it is their project's problem
and not this conversation's. A guideline that cannot be read is fatal — it is half the system prompt,
so a run without it has nothing to coach with, and it stops naming the
path. A guideline that reads but whose checklist has drifted from the
tool's checks is reported like the reader's: the interview proceeds, and
the human is told which check its checklist omits.

### The ending

The human ends it. On the way out the client stops the coaching session,
prints the path and whether the document checks hold — as the last
judging found them, since nothing has written the document since — and
says what the human owes: reading it once more, and committing it as
`phase 1: LLD for <slice>`, which the coach does not do.

A conversation in which nothing was ever drafted ends by saying so: there
is no document at that path, nothing to check, and nothing to commit.

The verdict the ending prints is the one the last judging reached, carried
out of the loop rather than recomputed at the end. Recomputing it would be
wrong twice over. A run that amends an existing document and never drafts
finds a document at that path, so a check made at the end would report a
verdict on a document this run did not write — the case
`ARunThatDraftedNothingEndsSayingSo` exists to distinguish, and the one a
file's presence cannot. And the judging already ran: the checks are the
same checks over the same bytes, so running them again is a second answer
to a settled question, which can only ever agree or reveal that something
wrote the document behind the coach's back.

A session that halts, a provider that fails twice, or a door that refuses
ends the run as the canopy client's does, with the sentence it was given,
and the document stays where it was written. A signal that kills the
process is not handled: the coaching session is then left unsealed until
canopy's idle window closes it, and the document on disk is unaffected.

## Security posture

The canopy client bounds what can leave by confining every read to the
workspace: what is transmitted is what the repository already contains.
**This slice breaks that bound deliberately, and saying so is the point.**
What the human types is a design conversation that exists nowhere else —
the alternatives they considered and rejected, what they know about a
system they have not documented, what they are unsure of — and every word
of it is landed as a record in the tenant's log and sealed there when the
session stops, along with the drafted document before anyone has decided
to commit it and the reader's findings about it.

That is the trade the slice asks for: the coach cannot interview without
being told, and canopy cannot host a conversation without storing it. It
is stated here so that a human choosing to run `coach` is choosing it,
and so that a project deciding whether to allow the subcommand has the
fact in front of it rather than in a transcript. The bound that remains
is the client's: `read` and `grep` reach nothing outside the workspace, and
`draft` writes one path inside it.

What the opening carries widens nothing. The index, the HLD and the
project's guidance are files the repository holds and a `read` would have
fetched a turn later; landing them early changes when they are transmitted
and not whether. What is narrated is the model's own words on their way
back to the human, which the log already holds.

## Shape

| Item | Role |
|---|---|
| `coach::run(args)` | The entry: the flags, the package, the session, the loop, and what the human owes |
| `Flags { target, slice, door, max_cost }`, `parse_args(args)` | The flags and their defaults; `target` is `Option<Where>`, none when neither flag was given, because a flag parser is handed no project and cannot count members. Any other argument is rejected by name, and `--package` and `--workspace` refuse each other |
| `Where::{Package(String), Workspace}`, `package_dir(project, target) -> Result<PathBuf, String>` | Where the document goes, which needs the project the flags do not have: the named member's manifest directory, the workspace root for `--workspace`, the only member when neither flag was given and the workspace has one, a stop naming the flag when neither was given and it has several, and a stop listing the members when a name matches none |
| `document_path(package_dir, slice) -> PathBuf` | `docs/intent/<slice>/lld.md` under that directory |
| `Coach { door, session, path, max_cost }` | One coaching run: the door, the session it holds, the document it writes, and the budget every session it opens is dialled with |
| `Noted { drafted, wrote, ended }` | What one turn's executor records: whether `draft` was called, whether it succeeded, and whether a tool learned the conversation is over — the facts the loop reads after the turn settles, held by the turn rather than by the session |
| `coaching_system(project) -> Result<String, String>` | The system prompt: the synced interview method, then the synced guideline, in that order |
| `opening(project, slice, path) -> String` | The first user message: what the repository holds (`preamble`), then what the slice is, and an existing document read in whole when there is one. The loop is handed it, having no slice of its own to build it from |
| `preamble(project, path) -> String` | The three repository sections in the order the opening carries them: the index, the sole HLD, the project's guidance |
| `intent_paths(project) -> Vec<PathBuf>` | Every `docs/intent` document in the workspace — the root's and each member's HLD, and each slice's `lld.md` — sorted, so one run's index and the next's agree |
| `index_section(root, paths, path) -> String` | Those paths as the opening lists them, each rendered relative to the workspace root, the row naming this run's own document marked as the one it writes. It takes the root because the paths do not carry their own relation to it |
| `hld_section(paths) -> String` | The HLD whole when `paths` holds exactly one, and nothing when it holds none or several — the count is the whole of the decision, so it is made where the paths are |
| `guidance_section(project) -> String` | `AGENTS.md` at the workspace root whole, else `CLAUDE.md` whole, else nothing |
| `converse(project, coach, opening) -> Result<Option<bool>, Halt>` | The loop: a human turn, a model turn, and the judges' turn when the turn drafted. It lands the opening as the first user message and settles a turn on it before reading the human. It answers with the last judging's verdict, or none when the conversation never drafted — the fact the ending needs and only the loop saw. A halt leaves it unanswered and the document where it was written |
| `human_turn(line) -> Option<String>` | What the human typed, or none at `done` or end of file. It is handed the line rather than reading it, so that the classifying is measurable and the read is the only thing that is not |
| `Tool::{Read, Grep, Draft, Ask}`, `declarations()` | The coach's four tools — a set of its own, and not the canopy client's five, whose `edit` and `write` it must not hold. `Read` and `Grep` take that client's `schema_of`; `Draft` and `Ask` carry their own |
| `executed(project, path, noted, op, args) -> ToolResult` | What `drive` is handed for one turn: the call announced (`announce`), then routed (`execute`). Two statements rather than one, so that neither the announcing nor the dispatch is the other's condition |
| `execute(project, path, noted, op, args) -> ToolResult` | The coach's tool dispatch; refuses an `op` it did not declare, and records what the turn did in `noted`. It takes the project because its `read` and `grep` routes are that client's over `confine`, and a confinement needs the workspace root |
| `grep_call(project, args) -> ToolResult` | `grep` routed to the canopy client's `grep_tool` under the same confinement `read` is bounded by |
| `announce(path, op, args)`, `announced(path, op, args) -> Option<String>` | The line a call is announced with, printed when there is one: a `read`'s path, a `grep`'s or a `glob`'s pattern, the document a `draft` replaces, and none for `ask` or for a call whose arguments do not carry what its line would name. It is written over the `op` rather than over `Tool`, because the reader's session is announced by it too and calls tools this one does not declare |
| `argument_named(args, key) -> Option<String>` | One string argument out of a call's JSON, which is where every announcement's subject comes from |
| `narrate(text)` | The coaching session's narrator: what the model said before a tool call, printed as it said it. The reader's session is driven with the canopy client's `silent`, its account being the findings it is about to land |
| `draft(path, content) -> Result<usize, String>` | Replaces the document, creating its directory; the bytes written |
| `ask(question, options, noted) -> ToolResult` | Prints the question with the stall window beside it and reads the answer; a human who ends the conversation is a tool error saying so, since a tool has no other channel, and is recorded in `noted` so the loop can end without reading them again |
| `Judging { holds, message }` | What one judging produced: whether the four document checks hold, and the message landed as the next user message. The verdict is the reader's neighbour, not its subject — a reader that could not be consulted has no bearing on whether the checks held |
| `judged(project, coach) -> Judging` | The four document checks and one reader session, as one message, and the verdict beside it |
| `reader_findings(project, coach) -> Result<Vec<String>, String>` | One fresh reader session over the document, stopped when it answers. It drives its turn with the canopy client's own dispatch behind an announcement, and with that client's `silent`: the judging's calls are printed, its prose is not. The announcing executor is a closure and not an item of its own, as `converse`'s executor is |
| `setup(project) -> Result<Vec<String>, String>` | The artifact checks before the interview: a guideline that cannot be read is the error, and everything else — a drifted checklist, a reader declaring too much — is a warning the human is told |
| `owed(path, holds) -> String` | The path, the verdict, and the commit the human must make. The slice it names in `phase 1: LLD for <slice>` is the document path's parent directory, which `document_path` built from the slice and so cannot disagree with it |

## Decisions & Alternatives

| Decision | Chosen | Alternatives Considered | Rationale |
|---|---|---|---|
| How the document's package is chosen | Named by `--package`, defaulting to the sole member of a one-member workspace | Inferred from the directory the command was run in; inferred from the slice's name; always the workspace root | `cargo metadata` reports no current package, so inferring means `cargo locate-project` — a contract with no provenance here — or a walk up the filesystem, which is another slice's module. And a guess is not caught later: `slice_crate` takes the first member holding a document of that name, so a document under the wrong package silently becomes the crate the slice is built in. Naming it costs one flag and removes a failure no check would question. |
| What the human sees while a turn runs | The model's text before each tool call, and one line per call | Nothing, as the phase worker prints nothing; a spinner or an elapsed clock; the model's text alone; the tool lines alone | A phase worker runs unattended and its silence costs nobody who is watching; an interview is watched, and a silent minute is indistinguishable from a dead one. A spinner needs a clock running beside a blocking read — the machinery the stall warning was already refused — and says only that time is passing. Either half alone leaves the other silent: the model's text without the calls hides the long part, and the calls without the text say what was read and never why. |
| How the repository reaches the model | An index of paths, the sole HLD and the project's guidance, landed in the opening | Left to `read`, one directory listing at a time; every LLD landed whole; the same three in the system prompt | The walk that finds a path is the same walk on every run and yields the same answer, which makes it the coach's to do once rather than the model's to repeat through a tail. Landing every LLD whole buries the slice that matters and spends the context on documents nobody chose. The system prompt is what the coach *is* — two synced artifacts the sync rule keeps honest — and what the repository holds is a fact about this run, which is the opening's subject. |
| What form the index's paths take | Relative to the workspace root, rendered at the row | Absolute, as the run holds them; workspace-relative throughout, `intent_paths` answering in that form | `confine` refuses an absolute path as written, so an absolute index is a list of paths the tool it feeds will not accept. Relativising at the row rather than at `intent_paths` keeps the paths a filesystem walk answers with in the form the walk produced — which `hld_section` opens and which this run's own document is compared against — and puts the one conversion where the one reader is. |
| Which HLD the opening carries | The one the index found; none when it found several | The first; the one under this run's own package; every one | Which HLD governs a slice in a workspace carrying several is a design question with a human's answer, and a coach that picks one has decided it where nobody can see. Named in the index and unread, it is one `read` and one `ask` away. |
| The tool set | Four: `read`, `grep`, `draft`, `ask` | Three, without `grep`; the canopy client's five; adding its `glob` as well | The method's central question is "what already answers this?", and asking it through `read` alone means walking directories to find the file to read. `grep` is already built, confined and capped at two hundred lines. `glob` is what the index has just answered, so it would arrive with its work done. `edit` and `write` are the two the coach must not hold: what it writes is one document, through `draft`. |
| Where `read` and `grep` get their schemas | The canopy client's `schema_of` | A copy in this module, as `read`'s was | Two descriptions of one tool are two things nothing keeps in step — the failure the sync rule exists to prevent, one layer down. `draft` and `ask` are the coach's own and have no other home. |
| How the loop knows a turn drafted | The coach executes its own tools through a closure that records the call | Scanning the model's settled text; comparing the document's modification time; a flag on the shared session type | The two inferences are guesses about a fact the client already holds — it ran the tool. A flag on the session would widen the canopy client's type for one host's benefit; a closure is the parameter that host already needs. |
| What `draft` answers with | The path and the bytes written | The document checks' verdict | The judges' turn runs the same four checks a moment later; answering with them would run and read them twice per drafting turn, and Phase 2 would have to decide whether that is one rule or two. |
| Whether the coach commits | No | Commit as `phase 1:` when the checks hold | Phase 1 is human-owned (README §8) and a coach that commits it has approved it. The human reading it once more is the only review this document gets. |
| What `draft` may write | The slice's LLD and nothing else | Also the module, the claims, or the branch | Everything else is a phase's artifact with a check behind it. A coach that scaffolds code produces work no gate has seen. |
| How the model asks | An `ask` tool that blocks on the human | A question in prose, answered on the next turn | A question in prose is a turn the human may answer past; a question through a tool is recorded beside its answer in the sealed log, which is what makes the interview auditable. The cost is a blocked turn, bounded by canopy's fifteen-minute invoke stall. |
| When the human is warned about the stall | Once, printed with the question | Again at ten minutes | A later warning needs a clock running beside a blocking read — a thread, or a polled read — which is machinery a conversation does not warrant. |
| The reader's session | Fresh for every judging, stopped when it answers | One reader session for the run | A reader that remembers reads the diff, and the guideline's questions are about the document a phase will receive. |
| When a reader fails | The conversation continues; its sentence is landed instead of findings | End the run, as a failed coaching session does | The human is present and mid-conversation; the document checks still ran, and a lost second opinion is not a reason to end a session they are using. |
| When an artifact check fails before the interview | The guideline's is fatal, the reader's is a warning | Both fatal; both warnings | The guideline is half the system prompt, so a run that cannot read it has nothing to coach with. A reader that declares too many tools is the project's problem to fix and does not stop a conversation. |
| Interruption | `done` and end of file are handled; a killing signal is not | Trap SIGINT and stop the sessions | Trapping needs a dependency or raw `libc` for a case whose only cost is a session canopy's idle window closes anyway. Tenet 3: the constrained option first, and the evidence for escalating is not here. |
| Measuring the terminal read | `read_line` carries `#[mutants::skip]`, and `human_turn` takes the line it returns rather than calling it | Skipping both; threading a `BufRead` through every caller so nothing is skipped | A blocking read of a terminal cannot be driven by a test that must also finish, so nine mutants in these two functions survived the gate with no test able to reach them. Splitting the read from its meaning leaves `read_line` holding no branch of this slice's own — it is the I/O sequencing the skip exception names, and the reason is here as that exception requires. The exemption is written as an `exclude_re` in `.cargo/mutants.toml` rather than as `#[mutants::skip]`, because that attribute resolves only with the `mutants` crate in `[dependencies]` — a published crate would gain a dependency to carry a no-op attribute, which tenet 3 refuses while a constrained option works. The cost of the config file is that the exemption does not sit on the function, so the regex is scoped to this file and this name and `read_line`'s own doc comment states it — while `human_turn`'s decision between an answer and an ended conversation becomes a function over a value a test supplies. Skipping both would exempt that decision rather than the I/O. Threading a reader would measure everything and put a library type in a leaf's signature, which the boundary rule refuses, cascading through four Shape rows to buy three mutants. |
| Where the interview method lives | `lid-rs/skill/references/coach.md`, synced, read beside the guideline | Inline in the client; inside the guideline | The method is prose a person maintains and the sync rule already keeps one copy honest. Beside rather than inside, so the reader's questions have one home and the interviewer's method another. |
| The run's total cost | Bounded per session, unbounded per run | A cap on the number of judgings; a run-wide budget | Every session is dialled with `--max-cost`, and the number of sessions is one plus the drafting turns — which the human chooses, one turn at a time, watching each. A run-wide budget would end a conversation mid-sentence at a moment the human did not choose. |

## Cascade

**Into the canopy client: already made, and this slice depends on it.**
That host's turn loop was built for a phase — `Session::open` took a
`Phase` and the five-tool set, `execute` matched that closed set,
`policy_for` was typed on that enum, and every call was judged by
`hook_pre_tool` and counted in a phase's tally — none of which is true of
a coaching run, which also holds a second session with a *different* tool
set, the reader's. That generalisation landed as a Phase 8 edit on that
slice's branch and is gated below this one.

What it gives this slice: `policy_for(declarations)` builds a policy from
the declarations a host hands it; `Session::open(door, settings, phase,
declarations)` takes an optional phase and a host's declarations;
`Session::declares(op)` judges a forwarded `op` against them; and
`drive(project, session, executor, message)` takes `Executor<'a> = &'a mut
dyn FnMut(&Project, &Session, &str, &Value) -> ToolResult`, so a host runs
its own tools and an executor can hold the state that records what it ran.
A session carrying no phase asks no pre-tool verdict and keeps no tally.
The canopy client passes its five tools, its `execute` and its phase, so
what it asserts is unchanged: `TheWorkerPolicyAdmitsExactlyTheFiveTools`,
`AForwardsOpClassifiesToItsToolOrToNone`,
`AnEditForwardedToTheReviewerIsRefusedHere` and
`TheReviewerPolicyAdmitsOnlyTheObservationTools` and
`TheDialCarriesExactlyFourSettings` were all checked against the code
when that edit was made: four were widened at their implementers, and
three whose sentences said *every* observation asks the verdict were
reworded to name a session carrying a phase, since a phaseless session
contradicts them. `EveryPhasePrintsItsSessionsAndItsEnding` is the one to watch: `Session::open`
prints unconditionally, so a coach's reader sessions print an opening line
too, which that claim does not cover and which the coach's own output must
account for rather than contradict.

**Into the canopy client, a second time: `drive` gains a narrator.** A
response that asks for tools carries text as well, and that text is read
past where the record is classified — so the model's account of what it is
about to do exists on the tail and reaches no host. `drive` takes
`Narrator<'a> = &'a mut dyn FnMut(&str)` beside its executor and hands it
that text when there is any; the client passes its own `silent` at both of
its call sites, so what a phase prints is unchanged and
`EveryPhasePrintsItsSessionsAndItsEnding` still holds. A parameter rather
than a field on `Session`, for the reason the executor is one: how a host
shows its work is not the session's to hold. That edit is this slice's to
make, its LLD and its claims cascaded with it.

The coach reuses `Door`, `Session`, `drive`, the pairing, the digest,
`confine`, `read_tool`, `grep_tool` and `schema_of` unchanged — its `read`
and `grep` are that client's, over that client's confinement and under that
client's descriptions, so a file reaches the coach exactly as it reaches a
phase worker, and nothing here restates what those answer. The coach's own
dispatch is what routes a call to them, which is why this slice's Shape is
what it is rather than twice the size. What a phaseless session gives up —
no path policy in front of its edits, no measurement of them — that host's
Security posture states, and this slice's says what bounds its own four
tools instead.

**Into the skill**: `skill/references/coach.md`, the interview method, is
a file under an already-mirrored directory — the sync rule covers it
without changing. It is prose in the `lid-rs` crate, outside this slice's
path policy, so it is the human's to write, and it must exist before
Phase 5, because `coaching_system` has nothing to read until it does.

Its reading section is the half this slice's opening has changed. A method
that says "read the neighbouring slices' LLDs" to a model with no index was
asking for the walk; with the index, the HLD and the guidance in front of
it, the instruction is to aim rather than to search — the index says what
exists, `grep` says which of them mentions the thing, and the first `ask`
should arrive within a few calls rather than after a survey. The method
still owns *what* to read for; only the cost of finding it has changed.

**Into the lld-review slice**: its `GUIDELINE` and `READER` path
constants become public, and so does `rendered`, the renderer that puts
every failure on a line naming its file, line, check and message. The
judges land the document checks "as `lld-check` renders them for a
human", and that sentence is only true if it is the same function: the
alternative reachable today is `report`'s *error*, which renders only by
way of failing and answers `Ok(())` when nothing failed, so a caller
wanting the text would have to unwrap an error that may not be there. `coaching_system` reads the first and
`reader_findings` opens the second, and a second copy of a synced path in
this module is two strings nothing keeps in step — the failure the sync
rule exists to prevent, reintroduced one layer up. Its checks and its
reader are otherwise used as they are.

**Into this crate's subcommand table**: `dispatch` gains a `coach` arm and
`USAGE` gains its line. The table is another slice's module, and its
`dispatch` cites that slice's claim, so the arm is the human's commit
rather than a phase agent's — as every cascade into another slice has
been.

## Open Questions & Future Decisions

### Deferred
1. Whether the coach should read a sibling slice's LLD unprompted. The
   opening names every one of them and carries none, and nothing tells the
   model which to open; a coach that read the whole workspace before its
   first question would be expensive and probably worse at listening. What
   would force the question is an interview whose first `ask` is still
   preceded by a survey, which the index was meant to remove.
2. Whether the reader's findings should be summarised before they are
   landed. Today they are landed whole, because a summary is a judgment
   about which findings matter and that judgment is the human's.
3. Whether an interview transcript belongs beside the LLD. The sealed
   canopy log already holds it; a copy in the repository would be a second
   source with no reader.
4. Whether a coach run should produce the branch and the slice's naming as
   well as the document. It would make the front of the walk one command,
   and it would also make the coach the thing that decides a slice's
   boundary, which is the human's first decision.
5. Whether the index should carry more than a path — each document's title
   line, say, so that "what already answers this?" could be asked of the
   index rather than of `grep`. Today it is paths alone, because a title is
   a read of every document in the workspace on every run, and `grep`
   answers the same question against the whole text rather than the
   heading. What would force it is an interview that greps for a slice by
   name and finds it filed under another.
6. Whether the claims about printing should gain seams a test can
   observe. `TheModelsSettledAnswerIsPrinted` prints the model's text
   verbatim, so the line function that would carry it is the identity, and
   the narrator prints what it is handed for the same reason;
   `TheJudgingsHeadingIsPrintedBeforeAReaderSessionOpens` is an ordering
   against a print `Session::open` makes unconditionally, which nothing
   in-process can see. Both are asserted as far as they can be — the
   heading's constant, the message landed under it, one reader session per
   judging, and the driven path around each — and no further, because a
   test that pretends to observe a print is worse than one that admits it
   does not. This is the workspace's existing practice, where
   `EveryPhasePrintsItsSessionsAndItsEnding` is asserted through
   `opened_line` and `ending_line` and never through the print. What would
   force the question is a surviving mutant at the gate: check 12 mutating
   either print has nothing to fail, and a survivor there is the evidence
   that the seam is worth its ceremony.

   The announcements join this company rather than escaping it. `announced`
   decides what a call is announced with and answers with it, so what a line
   says is measurable; the printing of it is not, and neither is
   `EveryToolCallIsAnnouncedBeforeItIsRouted`, which is an ordering inside
   `executed` between a print and a call and has no value for a test to
   observe. Both are asserted as far as they honestly can be — the line
   `announced` answers with, and the dispatch `execute` performs — and the
   ordering between them is left to the reviewer's eye, as the ending's and
   the heading's are. A mutant surviving at any of these print sites is the
   evidence this question asks for, and never a reason to suppress the
   mutant, raise a threshold, or work the gate around.

   Four more claims rest on the same limit in a milder form, and the
   accounting should say so rather than name only the two.
   `TheEndingPrintsThePathAndWhetherTheChecksHold` and
   `ARunThatDraftedNothingEndsSayingSo` both turn on what `converse`
   answers, and no test reaches that answer: every exit but a halt goes
   through a blocking read. They are asserted at `owed` and
   `nothing_drafted` over values a test supplies, which is honest —
   `owed(path, holds)` taking the verdict as a parameter structurally
   forbids recomputing it anywhere but the one line that calls it — but it
   observes the rendering rather than the carry.
   `TheDocumentsPathIsPrintedWhenTheSessionOpens` and
   `TheArtifactChecksRunOnceBeforeTheFirstQuestion` are the same shape:
   what is printed and what is checked are observed at `path_line` and
   `setup`, and when each happens is not.

## References

- `docs/intent/lld-review/lld.md` — the checks and the reader this uses,
  and the guideline whose questions it asks.
- `docs/intent/headless-canopy-agent/lld.md` — the door client, the tool
  machinery and the turn loop this reuses, and the Security posture this
  one departs from.
- `docs/intent/sync/lld.md` — the mirror rule the interview method ships
  under.
- README [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) — Phase 1 is human-owned, which is why this does not commit.
