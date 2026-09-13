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
| 1 | `cargo doc --no-deps --document-private-items` with broken intra-doc links denied; `cargo test --doc` | The LLD's links resolve and its code blocks compile — it is wired into its module from the first commit |
| 2 | `cargo check --all-targets` | The claims are documented items that build: `missing_docs` is denied for the workspace by rustc, so an undocumented claim fails without clippy's help, and a claim is a unit struct no configured clippy lint has an opinion about. A reworded claim's old name may still be cited, and those citations are the next phase's work list, not this phase's failure |
| 3, 4 | `cargo check --all-targets` | The skeleton, at this layer, type-checks (check 4) |
| 5 | Every claim in the red set has at least one `#[validates]` test, and every such test **fails** | The validations are red against `todo!()` before implementation exists |
| 7 | README §4.5, in order, first failure named | The slice passes the gate |

There is one doc step, and Phases 1 and 7 share it, so the flag that lets
rustdoc read private items is on at Phase 1 too: an LLD whose links resolve
only into the crate's public surface is not an LLD whose links resolve.
Each of the gate's six cargo steps passes `--locked`, through `args_of`. No
phase may edit a manifest, so no phase can need the lock updated; a step that
would have to update it is a step working against a tree the policy says
cannot exist, and failing there is more useful than rewriting the lock on the
way past. The
same flag refuses a workspace with no `Cargo.lock` at all, at the first
step: a package that never committed its lock fails Phase 7 at `cargo check
--locked`, which is the flag doing its job — a gate that claims a tree
builds reproducibly needs the lock that makes it so. A lock is written the
first time cargo resolves the graph, which `cargo metadata` does and which
`init` runs, so an initialised package holds one; committing it is the
package owner's act and not the tool's.
Phase 5's red run is outside the rule rather than an exception to it: the
per-validation `cargo test` and the registry dump it reads carry no `--locked`,
because the red run is not a gate step and its arguments are not `args_of`'s. Nothing is lost by it. At Phase 5
the workspace root's `Cargo.toml` and `Cargo.lock` are outside the phase's
staged set — the hook writes them at Phase 7 and at no other phase — so a run
that rewrote the lock would be a change outside the staged set, which the
integrity check refuses by name rather than committing.

A step's arguments are data, and `args_of` is where they live: one function
from a `Step` to the argument list `cargo_step` will run. Every rule about
what a step *invokes* — the doc step's `--document-private-items`, the
`--locked` on each of the six, the flags `package` carries — is then a
property of a returned list, which a validation can assert without running
cargo. Inlined in `run_step`'s arms those rules would be reachable only by
running the command and inferring the arguments from its behaviour, which for
a flag that only widens what rustdoc reads is no observation at all.

**A reworded claim.** Phase 2 writes the slice's claims file and nothing
else, while the sites that cite a claim are in the slice's module, so a rename
cannot land in one phase. A reword is therefore the renamed struct plus, beside
it in the slice's own `src/<slice>/spec.rs`, a deprecated alias for the old
name —
`#[deprecated = "replaced by <New>"] pub type <Old> = <slice>::<New>;` —
which registers no claim, so the graph sees only the new one, and makes
every citation of the old name warn with its replacement. Phase 2's check
does not lint, so those warnings do not reach it; the post-edit hook does
lint, and hands the agent every citation of the retired name after each
edit. That list is the cascade in flight — the sites Phases 3 and 4
revisit — and not this phase's to fix: the citing module is outside its
policy, and an attempt to edit it is refused. Phase 7's gate denies them as
it always did, so no citation of a retired name survives the slice, and an
alias no citation names is deleted by the next Phase 2 on the slice.
The alias lives beside the claim it retires because that is where the old
path resolved, and a path that no longer resolves is a compile error rather
than the warning the cascade is made of. The form is a `pub type` and not a
`pub use`: a `pub use` of a deprecated item warns at its own definition site,
so the alias would deny the gate by existing. A `#[deprecated]` `Spec` struct cannot play this part: once its
citations move it is a registered claim with no implementer, which checks
10 and 11 refuse, and only Phase 2 may delete it — the same phase that
cannot move the citations.

Phase 5 is the check no other tool runs. The slice's claims are the `SPEC`
records whose source file is the slice's claims file — `layout::spec_file`'s
answer, joined onto the claims crate (kebab-case slice name
to snake_case module), read from the registry dump the mutation subcommand
already uses — never from Rust source (README constraint 2). The
validations citing them are the `VALID` edges on those claims, each
carrying the test's item path; each test runs alone, `cargo test --lib -p
<package> -- --exact <path>` in the package that holds the slice's claims,
and the outcome is its exit status. A claim
with no validation, or a validation that passes, is named in the failure. A
slice whose spec file registers no claims is a failure too ("no claims for
slice `<name>`"), never a vacuous pass.

**The red set** is what changed since the slice was last gated. The base
is the newest commit reachable from `HEAD` whose subject starts `phase 7:`
— any slice's, since a gate commit leaves every other slice's spec file as
it was. The red set is the slice's claims, from the registry as above,
whose definition the branch added since that base: the claim's name, as
`struct <Name>`, on an added line of `git diff <base>` over that same claims file
in the crate that holds the slice's claims — the slice's crate, or its
companion (the path policy, below). With no gate commit in the history the whole file is
added and the red set is every claim — a fresh slice. On a Phase 8 edit the
red set is exactly the claims Phase 2 renamed or added, since a reword is a
rename (the `phase-check` section above); the slice's other claims keep
their green validations, which Phase 7's gate runs. An empty red set on a
branch with a gate commit is a failure naming the base ("no claim added
since <base>: nothing for phase 5 to be red about") — a Phase 8 edit that
changes no claim has no Phases 3–7, and the workflow stops with that
decision. The graph still comes from the registry alone; the diff decides
only which of its names are new.

`--slice` defaults to the slice the current branch names: the branch's name
with the `lld/` prefix removed and, when what remains contains `--`, the
part before it — `lld/phase--companion` is a change to the slice `phase`,
made on its own branch because the branch that built the slice is kept, and
git admits no `lld/phase/companion` beside an `lld/phase`. A detached `HEAD`
or a branch not of that form names no slice, which fails phase 5 naming the
convention and is irrelevant to the other phases.

Phase 7 is README §4.5 as the tool runs it: the same commands in the same
order, **one** `cargo package` naming every workspace package whose metadata
does not say `publish = false`, `sync --check` and `mutants` invoked through
the library rather than as subprocesses.

**One invocation, not one per package, and the difference is the whole gate.**
This step ran `cargo package -p <crate>` per publishing member. That form
resolves each crate's dependencies against a registry, so a workspace whose
members depend on each other at a version no registry holds — which is every
workspace between releases, this one included at `0.2.8` — fails on the first
member. And `gate()` stops at the first failure while ordering `Package`
before `SyncCheck` and `Mutants`, so **`sync --check` and check 12 have never
run under this hook, for any slice.** Both were gated by hand for every slice
that shipped, which is the gate this tool exists to make unnecessary.

One invocation naming every member resolves the siblings against each other
and succeeds — verified by hand at slice 22's gate, where it packaged all
three crates that the per-crate form could not. The catalog slice designed
that repair and named this module as the only place that can apply it; this is
the change it was the precondition for. The README's list stays canonical
(§4.5: "every copy of the list a project keeps must match"); this is one
more copy, held to the same rule. Steps a workspace appends after the floor —
this one's `mdbook build book` — are the workspace's to declare and the tool's
to run; see *The floor is the tool's, and the steps after it are the
workspace's* below.

**The doc step documents private items.** The gate's rustdoc step is `cargo
doc --no-deps --document-private-items`, with `RUSTDOCFLAGS` denying
`rustdoc::broken_intra_doc_links` — the form README §4.5 states and the
catalog's `doc` command already runs. Without the flag rustdoc never visits a
private item, so a broken link in a private item's doc comment is not an
error but an item rustdoc did not read: a whole class of the failure the
step exists to catch is invisible to it. A LID slice's items are
predominantly private — every leaf below the module's surface — so the
weaker form checks the smaller half of what the claims cite.

**Check 12's base is the last gate, not the trunk.** The gate's mutation step
carries no base as data; `run_step` resolves one when the step runs, passing
`mutation_base(project)?` into `mutants::run(&["--diff-base", base])`. That
base is the newest commit reachable from `HEAD` whose subject starts
`phase 7:` — the same commit the red set takes as its base, asked of the same
`gate_base` — and, when the history holds no such
commit, `git merge-base main HEAD`, the point the branch was cut from. Both
answers name what *this* branch's phases have changed since the slice was
last whole; neither is the trunk. When the merge base cannot be computed —
no `main`, or no common ancestor — the step fails naming the ref, rather
than falling through to a base that would mean something else.

The scope is the whole difference between a gate that runs under the hook and
one that does not. Measured here on 2026-09-11: with `main` as the base,
check 12 offered **1,390 mutants** and took about seventeen minutes, past the
600 s the harness allows a subagent between stream events; with the newest
gate commit as the base it offered **six**. A gate whose cost grows with the
distance to the trunk is a gate that stops running the moment a branch is a
few slices old, and the phase it protects is then the one committed by hand.

The base is the *gate's*, not the tool's. `cargo lid-rs mutants` run by hand
keeps its own default — the configured scope, `--full` or `--diff-base <ref>`
as the human gives it — and this project's CI keeps `--diff-base
origin/main`. The three ask different questions and deserve different
answers: a phase asks what this phase changed, a human asks what they are
looking at, a release asks what the branch changed against the trunk.

**The floor is the tool's, and the steps after it are the workspace's.**
README §4.5's list is where the gate starts and not where it ends: "This list
is the floor; a workspace appends build-integrity steps of its own after it,
and every copy of the list a project keeps must match." A workspace declares
its own steps in its root manifest, each one a command:

```toml
[workspace.metadata.lid_rs]
gate_extra = [["mdbook", "build", "book"]]
```

An entry is a program and its arguments. [`plan`](crate::phase::plan) appends
one `Step::Extra` per entry, in the order
configured, after [`Step::Mutants`](crate::phase::Step::Mutants) — so the
cheapest and most specific steps still fail first, and the workspace's own
steps run last, against a tree the floor has already accepted. `phase-check 7`
runs them, which is to say the stop hook runs them: a `phase 7:` commit is
made only when the workspace's steps pass too, and a step a project keeps by
hand becomes one line of configuration instead of a line in a runbook.

An entry runs at the workspace root, as a program and not through a shell. The
first word is the program and the rest are its arguments, so no quoting
grammar, word splitting, or variable expansion stands between the manifest and
the process — the same reason a step's cargo arguments are a list and not a
string. Failure is the gate's failure, naming the entry and carrying its
output: the program exiting non-zero and the program not being on the machine
at all are the same answer, because a step that could not run is not a step
that passed. The argument list is the entry's own and
[`args_of`](crate::phase::args_of) answers nothing for such a step: `--locked`
is a rule about what *cargo* is invoked with, and an extra step invokes
whatever the workspace named.

`gate_extra` is read from what `cargo metadata` reports and never by parsing
a manifest: the `[workspace.metadata.lid_rs]` table of the metadata document,
falling back to the `[package.metadata.lid_rs]` table of the package whose
manifest is the workspace root's when the workspace table does not name the
key — the two-step reading `mutation_scope` already has. The fallback is what
makes the key available to a project that is one package and no workspace,
which is what `init` scaffolds and what this slice's fixtures are: such a
project has no `[workspace]` table for the workspace form to sit under, so
its key is the root package's, exactly as `mutation_scope` is there.

The reading is split the way `companion`'s is, and for the same reason.
`Project::setting_node` is the raw door — one key's JSON node from the
metadata document, or none — and it implements no claim, because "which node
the metadata holds" has no wrong answer for a validation to catch.
`policy::gate_extra` is where that node becomes a plan's worth of steps, and
the three rules about the value are its: an absent key is the empty list, so a
workspace that configures nothing runs the floor and only the floor, and no
consumer's gate changes by upgrading; a value that is not a list at all — a
string, a table — fails the check naming `gate_extra` and what it found
instead, since it is neither an absent key nor an entry; and an entry that is
not a non-empty list of strings fails the check naming `gate_extra` and the
entry it could not read. A gate step that cannot be read is not a gate step
that is silently skipped (constraint 3). The split is not tidiness: a claim
whose only implementer is a hand-landed door in `src/project.rs` has no Phase
3 to leave it `todo!()` and therefore no Phase 5 that can make it red, so the
parse belongs in the slice, where every phase of the walk reaches it.

[`check`](crate::phase::check) builds its plan from that answer —
`plan(phase, publishing, policy::gate_extra(project)?)` — which is the one
place the value is read, so a `gate_extra` the tool cannot read stops the
check where the plan is built, at whichever phase is running, rather than
surviving five phases and stopping the gate.

Two hand commits carry this, and their moments differ. `Project::setting_node`
lands in `src/project.rs` between Phase 2 and Phase 3 — the slot this
workspace already uses for landing `pub mod spec;` by hand and for
`package_setting_at` before it — because Phase 3's skeleton calls it, and the
callers that call it cite claims that do not exist until Phase 2 has written
them. The workspace root manifest's `gate_extra = [["mdbook", "build",
"book"]]` lands *after* this change's own Phase 7, once the installed
`cargo-lid-rs` has been refreshed from the merged tree, in the same
documentation commit that changes CLAUDE.md's gate block from `mdbook build
book` "run by hand" to a configured step. The order is forced by the hooks
running the installed binary rather than the working tree: a key added earlier
is a key the old binary does not read, so it would not make `mdbook build
book` run in this change's own gate, and it would sit in the manifest naming a
step nothing runs. Phase 5's fixture takes the same road as `init`'s scaffold:
its key joins the `[package.metadata.lid_rs]` table `init` already wrote to
the `app` package's manifest (beside `mutation_scope`), since a lone package
has no `[workspace]` header the workspace form could sit under, and a second
`[package.metadata.lid_rs]` header would be a table redefinition `cargo
metadata` refuses. That fixture is also what exercises the fallback: a
validation of the read-from-metadata claim takes the package form, so the
door's second step is not left unobserved.

**An extra step may not write into the tree.** The stop hook's integrity pass
reads `git status --porcelain` against `HEAD` and filters it by the staged
set, refusing the stop when anything at all outside that set has changed, and
an extra step runs inside the check — so a step whose output lands on a
tracked or untracked path makes the phase's commit impossible, naming that
path. A step whose output goes under the target directory, or to a path the
project ignores, is unaffected, because `git status` never offers it. (`mdbook
build book` writes `book/book`, which this workspace ignores.)

Two costs stay the workspace's to weigh. The steps run inside the stop hook,
so their time is added to the gate's, against the watchdog Deferred 6 records.
And a step's program is resolved on the machine's `PATH`, so a gate configured
with a tool the machine lacks fails there and passes elsewhere — which is what
an extensible floor means: the tool answers for the floor, and the project
answers for the rest. An extra step is the project's, not the pipeline's, so
it carries no catalog name and `cargo lid-rs catalog` does not list it.

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

The three hooks are functions the binary exposes, and Claude Code's hook
events are one host for them: the canopy client
(`docs/intent/headless-canopy-agent/lld.md`) calls the same three with a
`HookInput` it builds from a session's records, so a phase run there is
gated by the same verdicts and committed with the same trailers.

### `hook pre-tool <n>` — the path policy

Runs before every tool call the phase agent makes; reads Claude Code's
hook JSON (`agent_id`, `tool_name`, `tool_input`). For `Edit`, `Write`, and
any other editing tool, the target path must be in the phase's allowed set,
relative to the slice's crate — the workspace package that holds the slice's
document, found on the filesystem, not by parsing Rust.

**Which package that is, is the layout slice's question, and `slice_crate`
delegates it.** This slice resolved it by looking for
`docs/intent/<slice>/lld.md` and nothing else, which stops working the moment a
slice's document moves beside its code: the colocated layout puts it at
`src/<module>/lld.md`, or at `<crate>/src/lld.md` for a slice that *is* its
crate. `cargo_lid_rs::layout` owns the resolution that admits both forms, and
`policy::slice_crate` calls it. The reading of `[package.metadata.lid_rs]
companion` moves the same way and for the same reason — which crate holds a
slice's claims is a layout fact, not a policy one — so `policy::companion`
calls `layout` too, and the dependency between the two modules runs one way.

This must land **before the first slice's document moves**. The migration is
incremental by design, so a tree with some slices moved and some not is the
normal state throughout it; a `slice_crate` that knows only the old form loses
every slice the migration has already touched, and the phase machinery stops
working on exactly the slices most in need of it.

**Colocation widens these sets unless they are narrowed, and that is a breach
of the boundary this slice exists to build.** `module_and` puts the bare
directory `src/<module>` in the allowed set and `matches_any` admits anything
under it (`relative.starts_with(entry)`). Today that directory holds only the
slice's leaves. Under the colocated layout it also holds `lld.md` and
`spec.rs` — so **Phases 3, 4, 5 and 7 would gain permission to rewrite the
slice's own design document and its own claims**, which §"What a phase may not
do" refuses outright and which is the confused-deputy boundary the whole path
policy exists to draw. Nothing would fail; the refusals would simply stop
happening.

So four more claims were falsified by the migration, beyond the five that name
paths. Each said the target "shall be the slice's module, a file under its
directory, or `src/lib.rs`", and each became a *wider* permission the day the
layout changed under it. All four are narrowed and renamed (the retired names
survive as `#[deprecated]` aliases).

**The rule is positive, not an exclusion list.** A phase may write a **Rust
source file** under the slice's directory, other than the slice's claims file —
plus the module file, `src/lib.rs` for Phases 3 and 4, and `tests/ui/` in the
companion for Phases 5 and 7. The document and the acceptance file are not Rust
source and are refused without the policy learning their names.

That shape was chosen over excluding the two paths this section first named,
because **there is a third**: `compile-time-accepted`. §Security posture calls
it "a file in the human-owned path, so acceptance is a human commit the hooks
verify in both modes, never an argument a model could supply" — and a file an
agent may `Write`, which the stop hook then stages, is not human-owned. The
acceptance gate runs before the path policy, so an unaccepted slice still cannot
self-accept; an accepted one could rewrite the record of what it was accepted
for. An exclusion list would need re-sweeping for every intent file invented
later; a positive rule refuses one without knowing its name.

What it trades away: a non-Rust asset under a slice's module — a fixture, an
included template — becomes unwritable by Phases 3–7. No crate here has one
today; the day one is wanted it is an LLD edit and a claim.

**Staging is the open half of this.** `TheStopStagesExactlyTheStagedSet` has
the stop hook stage the allowed set with `git add -- <paths>`, which cannot
express "everything under this directory except these" without a pathspec
exclusion. If the hook stages the directory as it does now, a human's
in-flight edit to a colocated `lld.md` is swept into a phase commit and the
integrity check that should have named it passes. The refusal and the staging
must narrow together, and only the refusal has.

These four contain none of the strings a path sweep looks for — no
`src/spec/`, no `docs/intent/` — because they name a *directory* that will come
to contain those files rather than the files themselves. A grep over claim text
cannot find that class of breakage; only reading a slice's claims against the
change can.

The allowed sets, relative to that crate:

| Phase | May write |
|---|---|
| 2 | the slice's claims file (`layout::spec_file`), `src/spec/mod.rs` |
| 3, 4 | the slice's directory and module file (`layout::slice_dir`), `src/lib.rs` |
| 5 | the slice's directory and module file (`layout::slice_dir`) |
| 7 (with 6) | the slice's directory and module file (`layout::slice_dir`) |
| reviewer | nothing |

**The directory is the layout's answer, not the slice's name — and asking for
it is not yet enough.** `policy::module_dir` spells `src/<slice_snake>` from
the slice's name. That is right for a module slice and wrong for a
**crate-root** one, whose document is `<crate>/src/lld.md` and whose name is
its package's: `layout::Form::CrateRoot.dir()` is `src` itself, so the
policy names a directory holding none of that slice's code. Phases 3 and 4
survive by accident, their row also carrying `src/lib.rs`; **Phases 5 and 7
carry no such escape and may write nothing at all** for such a slice. The
three that exist — `cargo-lid-rs`, `xtask`, and `macros` — all ran their
phases before colocation, so nothing has reached the hole. Slice 18
(`lid-rs-shape`) is the first that would, and no test covers the form:
`CrateRoot`, `slice_dir` and `rooted` appear nowhere in this slice.

Row 2 already asks the layout for the claims file, and the same threading is
what rows 3 to 7 want — `layout::slice_dir` passed in beside `claims`, never
spelled from the name here. **But for a crate-root slice that answer is `src`,
and `matches_any` admits anything under an allowed directory.** `cargo-lid-rs`
is a crate-root slice *and* holds eight module slices (`phase`, `catalog`,
`coach`, `init`, `layout`, `lld_review`, `sync`, `headless_canopy_agent`), so
handing its phases `src/` hands them all eight — the same confused-deputy
widening colocation already caused once, in the other direction. Trading a
lockout for a breach is not a repair, and this is the Open Question below.

The companion table is unchanged and stays `src/<slice>`: a companion is a
slice's *presence* in another crate, always a module directory and never a
crate root — `Form::Companion`'s directory is `module_dir(crate_root, module)`
whatever form the own crate takes.

**A proc-macro crate's slice.** A proc-macro crate links into no binary,
so a claim defined in it registers nowhere, a `#[validates]` test in it can
cite nothing, and a fixture that expands its macros must live downstream of
it. Such a slice keeps those three things in its **companion**: the
workspace package the proc-macro crate's manifest names under
`[package.metadata.lid_rs] companion = "<package>"`, read from `cargo
metadata`'s package metadata, never by parsing the manifest. The slice's
crate is still the one whose manifest directory holds the LLD; its
`compile-time-accepted` file stays there; and the phase's allowed set is
the union of the table above, relative to the slice's crate, and this one,
relative to the companion:

| Phase | May write, in the companion |
|---|---|
| 2 | `src/<slice>/spec.rs` — the companion directory's claims file — and `src/spec/mod.rs` |
| 3, 4 | `src/<slice>.rs`, `src/<slice>/**`, `src/lib.rs` |
| 5 | `src/<slice>.rs`, `src/<slice>/**`, `tests/ui/**` |
| 7 (with 6) | `src/<slice>.rs`, `src/<slice>/**`, `tests/ui/**` |

So Phase 2 writes the companion's spec files and nothing in the proc-macro
crate; Phases 3 and 4 write the macro's module and the companion's, whose
`src/lib.rs` is where the hand-authored edges citing the slice's claims go;
Phases 5 and 7 write both modules and the companion's `tests/ui`, the one
place a compile-failure fixture can live. A proc-macro crate whose manifest
names no companion is refused at every edit, naming the key, since no phase
of such a slice can produce a claim. A companion that is itself a proc-macro
crate, or is not a workspace member, is refused the same way. Each of those
is a policy refusal like any other — tallied, quoting the discipline row,
naming what the phase may do instead — and not a hook that cannot decide;
the companion is resolved once per hook call, before the acceptance gate.
The stop hook stages the staged set over both crates, the integrity check
filters against it, and a refusal for a failed check names the permitted
paths of both.

**The companion's claims file is the companion directory's.** `layout::spec_file`
answers the *own* crate's claims file, and for a crate-root slice that is the
crate's `src/spec.rs`. That answer is relative to the own crate; placed under
the companion it names `<companion>/src/spec.rs`, a file the layout never
defined and no claim of the slice lives in. A companion is a module directory
whatever form the own crate takes (`Form::Companion`, above), so the claims
file the companion seat is judged against, and stages, is
`module_dir(companion, <slice>)/spec.rs` — for `lid-rs-macros` that is
`lid-rs/src/lid_rs_macros/spec.rs`, where its six claims already are. The
own seat keeps the layout's answer. `seat_claims` is the one place the two
readings meet, so that a row and the verdict judging it cannot part.

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
the policy a confused-deputy boundary rather than a secrecy one. The
workflow's `StructuredOutput` — the final call a `schema` forces, which
reads nothing and writes nothing — is an observation for the same reason:
it is the agent's answer, made after its commit block has already been
judged. A command — impossible by the agents' tool lists — is refused if
one ever arrives, and a hook that cannot decide (no slice on the branch, no
project) refuses rather than letting the call through.

The same hook keeps the **tally**: one record per `agent_id` under
`<target>/lid-rs/agents/`, counting tool calls by kind — edits (`Edit`,
`Write`), observations (`Read`, `Grep`, `Glob`, `LSP`, `StructuredOutput`),
commands (`Bash`,
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
  hook bumps the version if this is Phase 7, then runs `phase-check <n>`;
  on success it stages the phase's staged set — the allowed paths, plus at
  Phase 7 the two files the bump wrote, and nothing else — commits the
  message with
  the tally appended as trailers (`Lid-Rs-Phase`, `Lid-Rs-Agent`,
  `Lid-Rs-Tools`, `Lid-Rs-Checks`, `Lid-Rs-Refusals`), and allows the stop. Nothing staged
  is a refusal ("no change to commit"). A subject whose tag is not this
  agent's phase is a refusal, as is a Phase 7 subject whose version is not
  the bumped one.
- ```` ```stop ```` — the numbered decisions that block the phase. The hook
  commits nothing and allows the stop: an honest "this needs the human"
  must always be possible, and the decisions travel to the reviewer's seat
  through the workflow's structured output.

A message with neither block, or both, is refused with the format.

The check runs as `phase-check <n>` in a fresh process of the same binary,
its output captured whole: the hook's own stdout is its channel to Claude
Code, and the gate's engines write to theirs. A failed check is a refusal —
Claude Code's `{"decision": "block", "reason": …}` on stdout, which keeps the agent running with the reason as
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

Before the check, the stop hook verifies that the synced artifacts match
the dependency's (`sync::check`). After it, it verifies integrity whole:
the synced artifacts still match, nothing outside the staged set has
changed (`git status` filtered by the policy), and at Phase 7 the two root
files the bump wrote still equal what it wrote. Any of these failing is a
refusal that names what moved and commits nothing — the check executed the
agent's code, and that code may have written what the agent could not
(Security posture, below). The staged-set and bumped-files halves are
asked only after the check because before it nothing but the pre-tool
policy and the bump itself has touched the tree.

The stop hook is the only place a phase commit is made. The agent has no
git; `--no-verify` has nothing to bypass.

**Phase 7 releases, so Phase 7 bumps the version.** A `phase 7:` commit is
where a slice becomes a release candidate: it is the commit `cargo package`
turns into tarballs, and packaging a version a registry already holds is a
publish that cannot happen. The stop hook for Phase 7 therefore raises the
workspace version before it runs the check. It reads the workspace root
`Cargo.toml` **as committed at `HEAD`**, raises the patch level of the
`version` under `[workspace.package]` by one, writes the result to the
working tree, and brings `Cargo.lock` back into agreement with `cargo update
--workspace --offline` — the workspace's own members and nothing else.
`generate-lockfile` would re-resolve the whole graph, so a third-party entry
could move under a bump that changed no requirement, and in a clone whose
registry cache is cold it has nothing to resolve from. `--workspace` names
only the members, and `--offline` is then a statement that it needs no index:
verified 2026-09-12 by bumping this workspace and diffing the lock, whose
every changed line was one of the six members' `version`.

Reading the committed manifest rather than the working one is what makes the
bump idempotent. A refused stop leaves the bumped pair in the tree and keeps
the agent running; the next stop of the same phase reads the same `HEAD` and
computes the same version, so eight refusals raise the version once, not
eight times. That is observable: a validation bumps a scratch repository
twice without committing between and asserts the manifest holds the same
version both times, which a bump reading the working tree fails. No other
phase bumps anything: Phases 2 to 5 commit no release.

**One line, patched, with no TOML parser.** The manifest text is scanned for
the `[workspace.package]` header and, after it, the first line beginning
`version = "`; the quoted value must be three dot-separated numbers, and that
line alone is rewritten with the last of them raised by one. **The scan ends
at the next line beginning `[`** — the header that ends the table — so a
`version` line belonging to `[workspace.dependencies]` or to any later table
is never the one patched, and a `[workspace.package]` table that holds no
`version` line is a failure rather than a patch applied somewhere else.
Anything else —
no such header, no such line, a value that is not three numbers — is a
failure naming what it looked for, never a guess. The `cargo-lid-rs` slice
rejected a TOML dependency for reading this workspace's metadata ("JSON /
metadata parsing"), and one line of one file is not the evidence that would
overturn it. Nothing else in the workspace needs editing: member manifests
take `version.workspace = true`, and the `[workspace.dependencies]` path
entries state a caret requirement that a patch bump still satisfies —
verified 2026-09-12 by bumping the workspace and resolving it.

**Every cargo step is `--locked`.** Each of the gate's six cargo steps —
`check`, `clippy`, `doc`, `test --doc`, `test --lib`, `package` — passes
`--locked`, so the gate proves the workspace builds from the lock file the
commit carries rather than from one cargo silently rewrote while proving it.
A step that would have to update the lock fails instead, naming it (verified
2026-09-12: cargo refuses with "cannot update the lock file … because
--locked was passed"). The bump's own `cargo update --workspace --offline` is
the one gate-side cargo call the stop hook makes without the flag, and the
reason there is one: writing the lock is its purpose. Phase 5's red run is outside the rule
rather than an exception to it — its per-validation `cargo test` and the
registry dump it reads are not gate steps and their arguments are not
`args_of`'s — and the paragraph on the check's `--locked` says why nothing is
lost by that.

`--locked` does not make `package` offline, and the version it packages is one
no registry holds, so the two facts meet at the step this slice already
rewrote. Verified 2026-09-12 at a bumped, unreleased version: `cargo package
-p lid-rs-shape --locked --allow-dirty` fails — "failed to select a version
for the requirement `lid-rs = \"^0.3.0\"`", the sibling being unpublished —
while the one invocation naming every publishing member succeeds, because it
resolves the siblings against each other. `--locked` changes neither outcome;
it only adds the promise that the lock the commit carries is the lock the
tarballs were built from. The per-member form was already refused above, and
the bump is what makes refusing it load-bearing rather than incidental.

**The subject carries the bumped version.** A Phase 7 commit's subject is
`phase 7: <version>: <what and why>`, and `<version>` is the version the bump
produced — the one the tarballs will carry, not the one the branch started
from. The agent can compute it: the manifest is readable and the rule is one
patch level. The hook compares them **after the bump and before the check**,
so a subject naming the wrong version is refused in a second rather than
after a full gate, and refuses naming both versions — the same shape as its
refusal of a subject whose tag is not this agent's phase, and a separate rule
from it.

**What the hook wrote, the hook stages.** The bump writes two workspace-root
files, `Cargo.toml` and `Cargo.lock`, that no phase's path policy admits.
At Phase 7 and at no other phase those two paths join the **staged set** —
what the stop stages and what the integrity check filters against — which is
otherwise the phase's allowed paths of both seats. They do not join the
**editing set**: an `Edit` or `Write` naming either is refused in Phase 7
exactly as in every other phase, and the refusal that tells the agent what it
may do instead quotes the allowed paths, never the staged ones. The two sets
part company here because the writer does: the agent may not touch the
manifest, and the hook that did is not the agent.

**A bump is not a phase.** The bump dirties two files every time, so "nothing
staged" would never be true at Phase 7 if it were asked of the staged set.
It is asked of the **editing** set — the phase's allowed paths, what the agent
could have written — so a Phase 7 whose agent changed nothing is refused as
having nothing to commit, and a bare version bump is never a commit. The
refusal is the same one every other phase gives; only the set it reads is
named differently from the set that is staged.

**The two root files are checked, not exempted.** Admitting them to the staged
set would otherwise make them the one place a Phase 5 or Phase 7 test could
write undetected — and `Cargo.toml` is the file the whole path policy exists
to keep out of the agent's reach. So the integrity check that runs *after* the
check does not merely skip them: it requires each to equal, byte for byte,
what the bump itself wrote, and refuses naming the file when either differs.
Every other path outside the staged set is refused for having changed at all;
these two are refused for having changed *since the bump*. The staged-set
check therefore does not name the root files at Phase 7 — its validation
asserts that a rewritten `Cargo.toml` is *not* what it names there, and is
at Phase 3 — while the bumped-files check's validation rewrites the file
after the bump and asserts the refusal names it.

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
  clean, which phase commits are present, and — read from the slice
  crate's manifest and `docs/intent/<slice>/` — whether the slice is
  compile-time and whether the human has accepted it (Security posture,
  below). A compile-time slice without acceptance stops the run before any
  worker runs; the hooks refuse it regardless. The run starts at the first
  phase without a commit — resumption is reading the branch, as the skill
  prescribes. No `phase 1:` commit means the run stops before doing
  anything: the LLD is the human's.
- **Per phase, 2, 3, 4, 5, 7**: the phase's agent (`agentType:
  "lid-rs-phase-<n>"`) works; its stop hook commits or refuses. It returns,
  as structured output, whether it committed, the commit, and the ≤3
  numbered decisions the skill's stop contract requires. A reviewer agent
  reads the commit against the phase's checklist and the `discipline.md`
  rows tagged for that phase, prompted to refute, and returns
  approve-or-findings. Findings go back to a worker, and the run carries
  a budget of six reworks it spends across every phase: a rejection spends
  one and the phase runs again with the findings; a rejection with the
  budget spent ends the run with the findings as the human's decisions.
  The budget is one pool rather than an allowance per phase, because a
  slice's difficulty is not spread evenly — the run that measured this
  needed three attempts at one phase and four at another, and a per-phase
  allowance generous enough for the second would have been a licence
  everywhere else. Six is that run's five reworks with one to spare. It is
  not an argument, a flag, or a setting a prompt can carry: a bound given
  once is a bound, and one that can be raised is not.
- **Terminal states**: *PR-ready* — every phase committed and the gate
  passed, the run returns the branch and the accumulated decisions for the
  human's PR review; or *stopped at phase N* — with the decisions that
  stopped it, and, when the budget is what stopped it, that fact. There is
  no third state and no waiver argument.

A rejection is the only thing the budget pays for. A failing *check* is
not a rejection: it is refused by the phase's own stop hook, which keeps
the worker running with the reason, up to the harness's cap of eight —
two mechanisms with two bounds, and they do not meet. A finding that
belongs to an earlier phase's artifact still ends the run, as it does
today: the walk is a line, and the budget buys more attempts at a phase
rather than a way back to one.

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
Lid-Rs-Agent: agent-7f3a
Lid-Rs-Tools: 14 edits, 9 observations, 0 commands
Lid-Rs-Checks: 14 post-edit, 1 stop
Lid-Rs-Refusals: 1 policy, 0 stop
```

`Lid-Rs-Agent` is the id the tally was kept under — the subagent's id on
Claude Code, the session's on the canopy client — so a commit names the
record of how it was made. The ratio of deterministic steps to agent-chosen ones is then in git for
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
  the repository; it detects nothing outside it. Phase 7's two root files
  are the one pair that check cannot ask to be unchanged, since the hook
  itself changed them; it asks instead that they still equal what the bump
  wrote, so a test that rewrites `Cargo.toml` or `Cargo.lock` at Phase 7 is
  detected exactly as one that rewrites `clippy.toml` is.
- **Compile time.** A new proc macro or build script needs `Cargo.toml`,
  which the policy refuses, so an ordinary slice adds no compile-time
  execution. A slice *in* a proc-macro crate, or in a crate whose build
  script consumes the slice's files, executes the agent's code after every
  edit; such a slice is a **compile-time slice**, reported by the
  precondition from `cargo metadata` (a `proc-macro` or `custom-build`
  target). Its edits are refused by the policy unless the human has
  accepted it by committing the slice's `compile-time-accepted` intent file
  with the LLD — a file in the human-owned path, so acceptance is a human
  commit the hooks verify in both modes, never an argument a model could
  supply. The companion of a proc-macro slice is an ordinary crate; the
  acceptance covers the macro's edits, which is where compile-time
  execution is.
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
| `Step` | Closed set: the gate's steps, phase 2's lint, and the red run |
| `check(project, phase, slice) -> Result<(), String>` | One phase's check: the plan, built as `plan(phase, publishing, policy::gate_extra(project)?)`, executed in order. The one place `gate_extra` is read, so a value the tool cannot read fails here — at whichever phase is running, not only at phase 7 |
| `plan(phase, publishing: &[String], extra: &[Vec<String>]) -> Vec<Step>` | A phase's steps as data; `publishing` names what one `cargo package` runs for and `extra` the workspace's own steps, which follow the floor in the order configured |
| `execute`, `execute_with`, `run_step` | Runs steps in order; the first failure is the result |
| `args_of(step: &Step) -> Vec<String>` | One step's cargo arguments as data — `--locked` on every one, `--document-private-items` on the doc step, `package`'s member list and `--allow-dirty` — so what a step invokes is assertable without invoking it; a step that invokes no cargo, an extra step included, answers with nothing |
| `cargo_step(project, args, env)` | One cargo invocation of `args_of`'s list, its output captured into the failure |
| `Step::Mutants` | The one step whose argument is not data: `run_step` passes `mutation_base(project)?` into `mutants::run(&["--diff-base", base])` when the step runs |
| `Step::Extra(Vec<String>)` | One entry of the workspace's `gate_extra`: the program and its arguments, carried as data on the variant. `args_of` answers nothing for it, so no rule about a cargo invocation is asserted of a step that invokes no cargo |
| `extra_step(project, entry: &[String]) -> Result<(), String>` | One extra entry run: the entry's first word as a program, the rest as its arguments, at the workspace root and through no shell; a non-zero exit or a program that cannot be run is the failure, naming the entry and carrying its output. A malformed entry never reaches it: `policy::gate_extra` fails the check naming `gate_extra` and the entry it could not read before any step runs |
| `mutation_base(project) -> Result<String, String>` | The gate's diff base: one decision between the gate commit and the merge base |
| `merge_base_with_main(project) -> Result<String, String>` | `git merge-base main HEAD` — where the branch was cut from; a failure names the ref when there is no common ancestor |
| `check_red`, `slice_claims`, `claim_validations`, `run_test`, `unvalidated`, `red_verdict` | The phase 5 red run over the registry dump |
| `gate_base(project) -> Result<Option<String>, String>`, `red_set(project, crate_root, spec_file, claims) -> Result<Vec<String>, String>` | The newest `phase 7:` commit reachable from `HEAD` — the red set's base, and check 12's through `mutation_base`; the slice's claims whose `struct <Name>` line the diff since it added |
| `slice_of_branch`, `resolve_slice`, `current_branch` | The slice from `lld/<slice>` or `lld/<slice>--<change>`; a detached `HEAD` names none |
| `HookInput` | The boundary type over the hook JSON: `agent_id`, `tool_name`, `tool_input` path, `last_assistant_message`, `stop_hook_active` |
| `policy::allowed(phase, crates, claims, own, target) -> Verdict` | The path tables, over the slice's crate and its companion if any; `Verdict::Refused(reason)` carries the discipline row. `claims` and `code` are the layout's answers, resolved by the caller because the policy holds no `Project` where it judges a path |
| `policy::SliceCode { dir, other_slices }` | Where a slice's code is, as one value: its directory, and the directories of other slices of the same crate that lie under it. Empty for a module slice; for a crate-root slice the directory is the crate's `src` and the second half is the module slices it holds. One value rather than two, so that the row and the rule judging it cannot come apart — the half that went missing would silently be the wider permission |
| `policy::SliceCode::of_own_crate(project, crates)` | That value for the own seat, resolved once per hook call beside `layout::spec_file`. The companion's is built from the slice's name: a companion is always a module directory, never a crate root |
| `policy::other_slice_dirs(project, crate_root, dir)` | The subtraction, and the LLD's reading 1 alone in one function — each entry under the slice's directory has its *name* put back to `layout::slice_dir`, and is kept only when that door answers with that very path. The classification is the layout's; this holds no second reading of what a slice is |
| `policy::allowed_paths(phase, seat, claims, code)` | One phase's set for one seat: `Seat::Own` is the first table, `Seat::Companion` the second; `claims` is `seat_claims`'s answer for that seat |
| `policy::seat_claims(seat, claims, code) -> PathBuf` | The claims file one seat is judged against and stages: the layout's answer (`claims`) for the own seat, `code.dir.join("spec.rs")` for the companion seat, whose `code` is the companion's module directory |
| `policy::staged_paths(project, phase, crates) -> Result<Vec<PathBuf>, String>` | The staged set: `workspace_paths` plus `hook_written_paths`. What the stop stages and the integrity check filters against — never what the editing verdict, `permitted_moves`, or the nothing-to-commit test consults, all of which stay `workspace_paths` |
| `policy::hook_written_paths(phase) -> Vec<PathBuf>` | The workspace root's `Cargo.toml` and `Cargo.lock` at Phase 7; empty at every other phase |
| `bump_workspace_version(project) -> Result<String, String>` | The hook's Phase 7 step before the check: the manifest at `HEAD` through `bump_patch_version`, written to the working tree, then `cargo update --workspace --offline`; the new version |
| `bump_patch_version(manifest: &str) -> Result<String, String>` | The manifest text with its `[workspace.package]` version line raised one patch level: `version_line` locates, `next_patch` raises, the line is spliced back |
| `version_line(manifest: &str) -> Result<(usize, String), String>` | The first line beginning `version = "` between `[workspace.package]` and the next line beginning `[`, and its quoted value; a failure names what it looked for |
| `next_patch(version: &str) -> Result<String, String>` | Three dot-separated numbers with the last raised by one; anything else is a failure naming the value |
| `subject_version(subject: &str) -> Option<String>` | The `<version>` field of a `phase 7: <version>: <what and why>` subject, which the hook compares against the bump's answer after the bump and before the check |
| `ending::subject_carries_version(message, version) -> Result<(), String>` | That comparison as a refusal: a Phase 7 subject naming another version than the bump's, or none, is refused naming both — a rule separate from the tag's |
| `integrity::contents_of(project, paths) -> Result<Vec<(PathBuf, Vec<u8>)>, String>` | The bump's two root files read back right after it, each with its bytes — what `bumped_files_untouched` holds the tree to; the bump returns the version, not the bytes, and `Cargo.lock` cannot be recomputed at check time without rewriting it |
| `integrity::bumped_files_untouched(project, version_files)` | The other half of the post-check integrity pass: each of the two root files must still equal, byte for byte, what the bump wrote |
| `policy::slice_crate(project, slice) -> PathBuf` | The package holding the slice's document, in either layout — asked of `layout::own_crate`, which is where the four shapes are told apart |
| `policy::companion(project, crate_root) -> Result<Option<PathBuf>, Refusal>` | The package `[package.metadata.lid_rs] companion` names, from `cargo metadata`; none for an ordinary crate; a refusal naming the key for a proc-macro crate without one, or one whose companion is a proc-macro crate or not a member |
| `policy::gate_extra(project) -> Result<Vec<Vec<String>>, String>` | The workspace's configured steps, parsed from `Project::setting_node("gate_extra")` — the same two-sided shape `companion` has over `package_setting_at`. An absent key is the empty list; a value that is not a list fails the check naming `gate_extra` and what it found; an entry that is not a non-empty list of strings fails the check naming `gate_extra` and the entry it could not read. The four metadata claims — the reading with its fallback, the absent key, the non-list value, the malformed entry — are cited here and not on the raw door, so Phase 3 leaves this a `todo!()` and Phase 5 can redden them |
| `Project::package_setting_at(dir, key)`, `Project::member_dir_named(name)` | What `companion` reads: a package's `[package.metadata.lid_rs]` setting, and a member's manifest directory by package name — both from the metadata document `Project` already holds, in `src/project.rs`, which this slice's phases may not write and the human adds by hand |
| `Project::setting_node(key) -> Option<serde_json::Value>` | One `metadata.lid_rs` key's raw JSON node from the metadata document `Project` already holds: the `[workspace.metadata.lid_rs]` table's, falling back to that of the package whose manifest is the workspace root's, as `configured_scope` reads `mutation_scope` through `setting_in`. It implements no claim — a raw node has no wrong answer — and it lives in `src/project.rs`, which this slice's phases may not write and the human adds by hand between Phases 2 and 3 |
| `SliceCrates { slice, own, companion }`, `SliceCrates::resolve`, `claims_crate()` | The crates a phase may write, resolved once per hook call; the crate that holds the slice's claims, where the red run diffs and tests |
| `Tally`, `tally::record(agent_id, kind)`, `tally::trailers` | Counts per agent under `<target>/lid-rs/agents/`; rendered as commit trailers |
| `hook_pre_tool(project, phase, input)` | Policy verdict for editing tools, tally for every tool |
| `hook_post_edit(project, input)` | Clippy, rendered as `additionalContext` |
| `hook_stop(project, phase, input) -> HookVerdict` | Parse the message; `commit` → sync → bump (Phase 7) → subject version → check → integrity (sync, staged set, bumped files) → stage → commit → allow; `stop` → allow; else refuse |
| `integrity::synced_artifacts_match(project)` | `sync::check`, as a refusal reason |
| `integrity::outside_policy_clean(project, phase, crates)` | `git status --porcelain` filtered against `staged_paths`; anything else is named |
| `ExecutionClass::{Ordinary, CompileTime(reason)}`, `execution_class(project, crate_root)` | From `cargo metadata` target kinds: `proc-macro`, `custom-build` |
| `compile_time_accepted(project, slice)` | Whether the slice's `compile-time-accepted` intent file exists — `layout::intent_file`'s answer |
| `Ending::{Commit(message), Stop(decisions)}`, `ending_of(message)` | The stop protocol, parsed from the final message |
| `refusal_for(project, phase, crates, output) -> String` | Output + `gates.md` row for the check that fired + what the phase permits |
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
| What Phase 2's check runs | `cargo check --all-targets` alone | The build and clippy with warnings denied except `deprecated`, as it was; clippy narrowed to the slice's spec files | A phase must be able to commit its own artifact, and Phase 2 could not: a claim the design turns out to need once a skeleton exists faces a crate-wide lint whose `todo!()` warnings no edit inside `src/spec/` can clear, which is why two such claims in this workspace were committed by hand. Narrowing is not available — clippy lints a crate, not a file, and the mechanism that approximates it parses JSON diagnostics to preserve a step that enforces nothing here: `missing_docs` is a rustc deny, and none of the configured clippy lints can fire on a documented unit struct with a derive. What the step did catch on Phase 2's own two files — `unused_imports`, `non_camel_case_types`, `dead_code`, clippy's doc-comment lints, and `missing_docs_in_private_items`, which is warn-only and so denied by nothing else — now surfaces at Phase 7, on a file that phase may not edit; the remedy is Phase 2 again. |
| Per-edit check | `cargo clippy --all-targets -- -D warnings` | `cargo check` alone; clippy plus the slice's validations | Measured after a small edit here: check 0.19 s, clippy 0.29 s; clippy sees checks 3, 6, 7, 8, 9 that check does not. Running tests per edit is seconds per edit for feedback the stop provides once. |
| Policy enforcement point | `PreToolUse` on the agent, with reasons quoted from `discipline.md` | Prose in the phase files (the 0.2.1 arrangement); a post-hoc diff check at the stop | A rule in prose is dropped exactly when it is inconvenient (skill LLD, evidence table); a diff check at the stop lets the agent spend a phase on work it must then discard. Refusing at the call is immediate, and quoting the discipline row keeps one source of truth for the rule's wording. |
| Confused-deputy scope | Writes are bounded; reads are not | Also restrict what the agent may read | The boundary is about what an instruction — from the prompt or from a file — can make the agent *do*; hiding files would make the reviewer's cold reading impossible and gains nothing once writes are bounded. |
| Runtime tampering | Detected at the stop (synced artifacts and everything outside the policy must be unchanged) and refused; prevented only by isolation | Sandbox every check from the hook (`bwrap`, `sandbox-exec`); ignore it | Detection is cheap, deterministic, and names the event; a sandbox is a control of its own with platform rules, deferred rather than implied. Ignoring it would let a Phase 5 test rewrite the policy the next session loads. |
| Where a proc-macro crate's slice keeps its claims, companion module, and fixtures | A companion crate, named by the proc-macro crate's `[package.metadata.lid_rs] companion`, with its own per-phase path table | Build such slices by hand, outside the phases; a widened policy for compile-time slices; deriving the companion from the dependency graph (the member that depends on the macro crate and re-exports it) | A proc-macro crate cannot register a claim or cite one, so without a second crate no phase of its slice has an artifact; by hand forgoes the gate for the four slices that extend the derive. A widened policy admits everything. The dependency graph names every dependant, and which one re-exports the macros is a question of Rust source; one line of package metadata, read from `cargo metadata`, is the answer the human gives once. |
| The branch a change to a delivered slice is made on | `lld/<slice>--<change>`; the slice is the part before the first `--` | `lld/<slice>/<change>`; `lld/<slice>@<change>`; committing to the original `lld/<slice>` | git refuses `lld/<slice>/<change>` while `lld/<slice>` exists, and the original is kept forever as the slice's origin. `@` is legal in a ref but reads as a revision suffix in every git command line. A double dash cannot occur in a kebab-case slice name, so the slice is the part before the first `--`. The original branch is the slice's story; a change branch is the change's. |
| Shared leaves on a Phase 8 edit | A leaf that also implements a claim outside the red set keeps its body; Phase 3 wipes only leaves whose every claim is in the red set; Phase 5 makes each red-set claim red by validating the delta | Wipe every implementer of a red-set claim; commit every such Phase 5 by hand | Wiping a shared leaf breaks green validations of claims the edit never touched, so Phase 3 could not commit. A reword's delta is observable by construction — it is why the claim was reworded — so a validation of it can be red without un-implementing anything. Hand commits stay for subtractions, whose delta is an absence. |
| Compile-time slices | Disclosed from `cargo metadata`; edits refused unless `docs/intent/<slice>/compile-time-accepted` exists, a file only the human's Phase 1 commit can add | Refuse them outright; treat them like any slice; a workflow argument (`args.compile_time`) | The tool's own `lid-rs-macros` is such a crate and must be workable; the human, not the workflow, decides to run compile-time code unattended. A workflow argument reaches the hook only through a model's prompt, which is exactly the channel the policy must not trust; a file in a path no agent can write is a decision the hook can verify. |
| The stop protocol | Fenced ```` ```commit ```` or ```` ```stop ```` in the final message | Structured output only; a marker line; the hook reading the transcript | `last_assistant_message` is what the hook receives; a fenced block is unambiguous to parse and to write, and the refusal teaches the format when it is missing. Whether the final message survives a workflow `schema` is verified at Phase 3 of this slice; if not, the workflow's worker returns plain text and the script parses it. |
| The workflow's structured answer | `StructuredOutput` is an observation | A fourth tool kind; a command, with the workflow parsing the worker's final message instead of a `schema` | The call reads and writes nothing, and it arrives after the stop hook has already judged the commit block: refusing it there ends the run with the phase committed and the workflow reporting a failure. A tool kind of its own would count something the tally has no question about. |
| The red run on a Phase 8 edit | Scoped to the claims added since the newest `phase 7:` commit, by name in the spec file's diff | Every claim of the slice (the first design); an explicit `--claims` list; the claims the Phase 2 commit's diff touched at all; a registry dump of the base commit | Every claim of the slice can only be red by un-implementing the slice, so an implemented slice's Phase 5 could never pass the hook. A `--claims` list is an argument that reaches the hook through a model's prompt. Any changed line of the Phase 2 diff would sweep in a claim whose doc comment merely mentions another. A base registry dump means building the base commit for every red run. The gate commit is the one moment the slice is known whole, and a renamed struct is exactly one added `struct <Name>` line. |
| A reworded claim under the policy | The renamed struct plus a `#[deprecated]` type alias for the old name beside it in the slice's claims file, wherever `layout::spec_file` puts it — where the old path resolved, which is the point of an alias; Phase 2's check does not lint, so a deprecation reaches the agent only as the post-edit hook's context | `#[deprecated]` on the claim struct itself; a hard rename, with Phase 2's check tolerating unresolved citations; widening Phase 2's policy to the citing module | A deprecated `Spec` struct registers a claim that, once its citations move, has no implementer — checks 10 and 11 refuse it, and only Phase 2 could delete it. Unresolved citations are compile errors no lint level tolerates, and they stop the registry tests compiling too. Widening the policy gives Phase 2 the code it exists to be kept out of. The alias registers nothing, warns at exactly the citation sites, and clippy's `deprecated` is the one lint whose firing at Phase 2 is the methodology's own signal rather than a defect; the gate at Phase 7 still denies it. |
| Staging | The phase's allowed paths of both seats, plus — at Phase 7 only — the two workspace-root files the hook's own bump wrote | `git add -A`; the agent names files; widening Phase 7's editing policy to `Cargo.toml` and `Cargo.lock` so one set still serves both | The set that bounds edits bounds the commit; anything else the agent could not have written — and the two exceptions are exactly the files the agent did *not* write, because the hook did. Widening the editing policy to keep one set would hand the agent the manifest the policy exists to keep it out of, and would make the refusal message offer `Cargo.toml` as somewhere to fix a failing gate. Two sets that differ by what the hook writes is the honest shape; the editing verdict and `permitted_moves` keep reading the narrower one. **Claims:** `OnlyThePoliciesPathsAreStaged` and `TheStopStagesBothCratesAllowedPaths` are reworded to the staged set; `ChangesOutsideThePolicyRefuseTheStop` and `IntegrityFiltersAgainstBothCratesAllowedPaths` are reworded to filter against the staged set; `NothingToCommitIsARefusal` is reworded to say the test reads the *editing* set, so a Phase 7 whose agent changed nothing is refused though the bump dirtied two files; one new claim puts `Cargo.toml` and `Cargo.lock` in the staged set at Phase 7 and no other phase; one new claim requires each of those two to still equal the bump's own output after the check, refusing by name when it does not. |
| Check 12's diff base under the gate | The newest `phase 7:` commit reachable from `HEAD`; `git merge-base main HEAD` when the history holds none | Keep `main`; a `[workspace.metadata.lid_rs] mutation_base` setting; hand the gate to a detached runner and refuse until it reports | Measured here 2026-09-11: `main` as the base offered 1,390 mutants and about seventeen minutes, past the 600 s a subagent is allowed between stream events, so Phase 7 was committed by hand; the newest gate commit offered six. `main` also answers the wrong question — it is what the *branch* changed, and a phase is judged on what the phase changed. A configured base is a value whose wrong setting is a vacuous gate, and it reaches the hook from a file a phase could be argued into rewriting. A detached runner keeps the seventeen minutes, adds a process whose failure the hook cannot see, and leaves the commit waiting on it; making the run proportionate removes the reason for it. The `mutants` subcommand and CI keep their own bases, because a human and a release ask different questions. **Claims:** one new claim gives the gate's mutation step a `--diff-base` of `gate_base`'s commit — a *sibling* of `TheBaseIsTheNewestGateCommitReachableFromHead`, not a sharer of it: that claim is the red run's rule about which commit `gate_base` answers with, and this one is the gate's rule about what the mutation step does with the answer, so either could change without the other. One new claim makes the base `git merge-base main HEAD` when the history holds no gate commit, and one more fails the step naming the ref when no merge base exists. |
| The version bump at Phase 7 | The stop hook raises `[workspace.package] version` one patch level from the manifest at `HEAD`, by a line patch, and regenerates `Cargo.lock`; every cargo step is `--locked` | The `toml` crate; `cargo set-version` from cargo-edit; leave the bump to the human, as it was | Eight hand commits on the record branch were version or lock repairs, and two of them were the same publish trap twice: a `phase 7:` commit that keeps the released version packages a version a registry already holds, and the gate that would have said so is the gate the commit claims to have passed. A TOML dependency is the one the `cargo-lid-rs` slice already rejected ("JSON / metadata parsing"), and one line of one file is not the evidence that overturns it. `cargo set-version` is a third-party binary the hook would have to require installed in every consumer, to edit the same line. Leaving it to the human is what was measured, and it is what failed. **Claims:** one new claim has the Phase 7 stop raise the workspace version one patch level from the manifest at `HEAD` before the check, which is also its idempotence; one for the line patch's target — the first `version = "` line between `[workspace.package]` and the next line beginning `[`, the rule that keeps a later table's `version` safe; one for its failures, naming what was looked for when the header, the line, or three numbers are missing; one for the lock being brought into agreement with `cargo update --workspace --offline`; one for `args_of` putting `--locked` on every cargo step; and one for the subject-version refusal, which is a sibling of `ACommitSubjectMustCarryThisPhasesTag` — that claim is about the tag and says nothing about a version. |
| The gate's doc step | `cargo doc --no-deps --document-private-items` | Keep the weaker `cargo doc --no-deps` | Without the flag rustdoc never visits a private item, so a broken link there is not an error but an item rustdoc did not read — and a LID slice is mostly private items. The weaker form hid a real rustdoc error until it was found by hand (record branch `d3f9040`). README §4.5, CLAUDE.md and the catalog's `doc` command carried the flag; the skill's `references/phase-7.md` and `.github/workflows/gate.yml` did not, and neither did this copy — all three were corrected together when the hook's copy was fixed, which is the drift §4.5's "every copy of the list a project keeps must match" forbids. **Claims:** one new claim gives `args_of`'s doc step `--document-private-items`. `PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce` and `PhaseOneChecksTheDocs` say only "doc" and "rustdoc" — they name the step's place in the order, not its arguments — so neither is reworded, and the flag is a claim of its own that both orders' doc step satisfies. |
| The companion seat's claims file | `seat_claims`: the layout's answer for the own seat, the companion module directory's `spec.rs` for the companion seat | Give `layout` a companion door (`companion_spec_file`) and ask it; move a crate-root proc-macro slice's claims to the companion's `src/spec.rs`; leave the own crate's relative answer in place | Measured 2026-09-12 on `lid-rs-macros`, the first crate-root proc-macro slice under a companion: Phase 2 was admitted only `lid-rs/src/spec.rs`, while the LLD, the six existing claims and their `macro_edge!` lines all live in `lid-rs/src/lid_rs_macros/spec.rs`, so the phase could not commit. Which directory a companion is, the layout already answers (`Form::Companion` is `module_dir`); which file in it holds claims is the colocation rule (`spec.rs` beside the module), not a second reading — so the policy joins the two answers it already holds rather than opening a door for a fact with one possible value. Moving the claims would split a slice's claims across the companion's crate root and its module against README §11.1. **Claims:** add `TheCompanionSeatsClaimsFileIsItsModuleDirectorysSpec`, and correct `PhaseTwoMayWriteOnlyTheCompanionsSpecFiles` in place, whose "the same answer the layout gives for the slice, placed in the companion" is the reading this row replaces and now contradicts the new claim. In place, not by the rename-and-alias rule this document states for a reworded claim: that rule exists so a changed behaviour is a new `struct` line the red run diffs and Phase 5 reddens, and here the behaviour is already delivered and gated under the new claim — a renamed claim would enter the red set with a validator that is green on arrival, which the red check refuses and no phase could make red. A claim whose text is corrected to match behaviour another claim already gates is a documentation defect (tenet 1) and keeps its name. Corrected, Phase 2's companion row is the companion's module directory's `spec.rs`, and its validator reaches that row through `workspace_paths`/`allowed`, which resolve the seat's claims file, rather than by calling `allowed_paths` with a raw claims path of the validation's own choosing — a validation that chose the path would assert the layout's own-crate answer the claim exists to refuse; rewriting that validator is Phase 5's, as work on a claim outside the red set that must stay green. |
| Stop-refusal budget | Refuse while the check fails, up to Claude Code's cap of eight | One refusal then allow (the first design); refuse forever | A failing check is not a reason to let the phase end; eight rounds of clippy output is more than a fixable phase needs, and the cap leaves a dirty, uncommitted tree the next precondition refuses. A `stop` block is always allowed, so an honest stop is never blocked. |
| Trusted binary in the tool's own workspace | Hooks name the installed `cargo-lid-rs` directly, refreshed from `main` after merge; no synced script | A synced `hooks/run` script preferring `cargo run -p cargo-lid-rs` here (the first design); a separate worktree build | A worker in this repository edits the hook's own source; running it from the tree means the policy is whatever the worker last wrote. Enforcing only landed policy is the price of the tool being its own consumer. |
| Instrumentation | A per-agent tally kept by the hooks, written as commit trailers | Parse `agent_transcript_path`; no instrumentation until the design settles | The hooks see every call and refusal; the transcript format is undocumented. Trailers put the measurement where the review already happens, from the first phase this design runs. |
| Phase 7's commit subject | `phase 7: <version>: <what and why>`, `<version>` being the one the bump produced, compared after the bump and before the check | The project's convention alone; the hook rewriting the subject's version itself; comparing after the gate | The tag is what makes the stop hook run the full gate; the changelog-readable part follows it. The version is checked rather than rewritten because the agent can compute it — the manifest is readable and the rule is one patch level — and a subject the hook edits is a subject the reviewer did not read. Comparing before the check costs a second to refuse what would otherwise cost a whole gate. **Claim:** the subject-version refusal, named in the bump row above, belongs here too; `ACommitSubjectMustCarryThisPhasesTag` is unchanged. |
| Phase 7 gate duration in a hook | `timeout` set in the agent's frontmatter to cover a mutation run | Move mutants to CI only | A gate that exists, gates; the hook's timeout ceiling is verified at Phase 3, and mutants moves to CI only if the harness caps below what a slice needs. |
| Worktree per worker | Deferred | The Workflow's `isolation: "worktree"` per phase agent | Which branch a temporary worktree checks out is undocumented; a commit there must land on `lld/<slice>`. The dirty-tree precondition covers the failure the worktree would have contained. |
| Phase 5 test execution | One `cargo test … --exact` run per validation, exit status as verdict | One `cargo test --lib` run with libtest output parsed; `--format json` | One process per test costs seconds on a slice-sized set and needs no parsing of libtest's human-oriented output; JSON output is nightly-only. |
| Phase 5 slice identity | `SPEC` records by source file, which `layout::spec_file` answers for either layout; slice from the branch name | Parse `src/spec/` for the module; a `--claims` list | The registry already carries the file; the branch convention already carries the slice; constraint 2 forbids the parse. |
| Phase 7's list | The tool holds README §4.5 verbatim, in order, as one more copy the README's rule binds | Make `cargo lid-rs gate` canonical and reduce the README to a pointer | Keeping the list canonical in prose is deliberate for now: the spec stays readable without the tool. Promoting the tool is a README change with its own slice. |
| Where a workspace's own gate steps are declared | `[workspace.metadata.lid_rs] gate_extra`, falling back to the root package's `[package.metadata.lid_rs]` as `mutation_scope` does — a list of commands, each a list of strings, read from `cargo metadata` | A shell string per step; a `gate_extra.sh` the tool runs; leaving them in CI, as they were; a `cargo lid-rs gate` the project wraps | A list of strings is the same shape a step's cargo arguments already have, and it has no quoting grammar, no word splitting, and no expansion for a manifest to get wrong. A shell string adds all three and a shell. A script file is a path the tool would have to trust and a project would have to keep executable, and it hides which steps exist from anything reading the manifest. Leaving them in CI is what was measured: a gate that exists only in CI does not gate the commit that claims to have passed it, and this workspace's `mdbook build book` was run by hand for every slice that shipped. The metadata is already where `mutation_scope` and `companion` are read from, so the reading is one more `metadata.lid_rs` key and no new file, and `mutation_scope`'s fallback to the root package's table comes with it, which is what lets a single-package project — `init`'s scaffold, and this slice's fixtures — carry the key at all. **Claims:** one new claim puts the extra steps after the mutation step in `plan`'s answer for phase 7; one makes an absent key the empty list, so the floor alone is what an unconfigured workspace runs; one has an entry run as a program at the workspace root and through no shell; one makes a failing or unrunnable entry the check's failure, naming the entry; one has the list read from the metadata `cargo metadata` reports rather than from a manifest; one fails the check naming `gate_extra` and the entry it could not read when an entry is not a non-empty list of strings; one fails the check naming `gate_extra` and what it found when the value is not a list at all, which is neither an absent key nor an entry. The three about the value are cited by `policy::gate_extra` and not by `Project::setting_node`: the raw door is a hand commit no phase can write, so a claim implemented only there has no Phase 3 to leave it `todo!()` and no Phase 5 that could make it red. |
| How `policy::gate_extra` is skeletoned | Wired into `check` at Phase 3 with a body answering the empty list, so `plan` carries no extra step until Phase 7 and the four metadata claims are red by assertion | Declared unwired with a `todo!()` body until Phase 7; wired with a `todo!()` body; `check` reading the value only at Phase 7 | `check` is reached by every validation of a phase's check outside this change's red set, so a wired `todo!()` turns them red for a reason that is not theirs, and Phase 3's own check could not pass; unwired, the item is dead code in a private module and the post-edit lint refuses it. The empty list is the wrong answer that compiles: a configured key answered as empty fails the read-with-fallback validation, a malformed key answered as empty fails the two refusal validations, and an absent key answered as empty is the one case green early, which Phase 5 states. Reading only at Phase 7 would put a phase decision inside `check` that the malformed-key row says it does not have. `extra_step` keeps the ordinary `todo!()`: nothing green reaches it while the list is empty. |
| Where the extra steps sit in the gate's order | After every step of README §4.5's floor, the mutation step included | Before the mutation step, so the slowest step stays last; interleaved by configuration | The floor's order is cheapest-and-most-specific first, and a workspace's step is neither cheap nor specific to the tool: it is a build-integrity step over a tree the floor has already accepted, so running it before the floor's own steps would spend it on trees the floor rejects. Interleaving by configuration makes the order a value a manifest can get wrong, and README §4.5's list is the order. The cost is that an extra step follows the gate's longest step, so a workspace's step fails late; the alternative is failing the floor's cheap steps late instead. |
| How an extra step is carried through `plan` | `Step::Extra(Vec<String>)`, one variant per entry, with `plan` taking `extra: &[Vec<String>]` as a parameter beside `publishing` | A single `Step::Extras` that reads the metadata when it runs, as `Step::Mutants` resolves its base; a `GateInputs` struct in place of the two parameters | A step whose arguments are data is a step a validation can assert without running it — the reason `args_of` exists — and the extra steps are configuration, which is exactly the shape `publishing` already has as a parameter. `Step::Mutants` is the documented exception because its base is a git fact resolved when the step runs and is one value; a list of commands read at plan time is neither. A `Step::Extras` could be observed only by running the programs, which for "in this order, after the mutation step" is no observation at all. A struct for two lists is a name for the pair and nothing else today; it becomes worth adding at the third configured input. |
| When a malformed `gate_extra` fails | At whichever phase's check builds the plan, failing the check naming `gate_extra` and the entry it could not read | Only at phase 7, where the value is used; ignoring an unreadable entry | The value is read where the plan is built — `check`, through `policy::gate_extra` — which is one place, and a configuration a project cannot have meant is a defect whose earliest naming is its cheapest. Deferring it to phase 7 means a typo survives five phases and stops the gate. Ignoring it is the vacuous pass constraint 3 forbids: a gate step that is silently dropped is worse than one that was never configured. |
| The workflow's input | A branch with a human-approved `phase 1:` commit; no waiver argument | A slice name, with the workflow drafting the LLD; a `--waive` argument | Phase 1 is human-owned; a workflow that drafts it and continues has approved its own LLD. A waiver given once is reused; an argument is a waiver given every time. |
| Reviewer at each stop | One clean agent per phase, prompted to refute, one rework round | No reviewer; a judge panel per phase | A clean reviewer is also the test that the artifact is context-free — the failure interactive mode cannot see. A panel exceeds the cost a slice warrants; one rework round bounds the run. |
| Where the artifacts live | `agent/` and `workflow/` beside `skill/` in the `lid-rs` crate, synced under one rule | Inside `skill/`; a separate crate; the plugin | Claude Code reads agents and workflows from `.claude/agents/` and `.claude/workflows/`; the files are version-coupled to the skill they point at, so they ship with it. |

**How to read the claim notes.** Each Decisions row above ends with the claims
its decision costs: those to be added, and those whose text no longer matches
what this document says. Five are reworded — `OnlyThePoliciesPathsAreStaged`,
`TheStopStagesBothCratesAllowedPaths`,
`IntegrityFiltersAgainstBothCratesAllowedPaths`,
`ChangesOutsideThePolicyRefuseTheStop`, and `NothingToCommitIsARefusal` —
and each rewording is a rename with the old name kept beside it as a
`#[deprecated]` alias, by the rule this document already states for a
reworded claim. `TheBaseIsTheNewestGateCommitReachableFromHead`,
`ACommitSubjectMustCarryThisPhasesTag`,
`PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce` and
`PhaseOneChecksTheDocs` are named in those notes as claims that stay as they
are, each with the reason a sibling was cut instead.

## Open Questions & Future Decisions

### Open

**(Answered below, and the answer is reading 1 — left here with its
alternatives so the choice can be reversed in one function.)**

**What a crate-root slice's directory is, for the path policy.** The policy
must stop spelling `src/<slice>` from the name (above), but `layout::slice_dir`
answers `src` for a crate-root slice, and `matches_any` admits everything under
an allowed directory. For `cargo-lid-rs` that is eight other slices' code. The
readings, none yet chosen, and this is the human's because it moves a
confused-deputy boundary rather than a path:

1. **`src` minus the module-slice directories that crate holds.** The layout
   already knows which members hold which slices, so the subtraction is
   answerable. Truest to "the slice is the crate"; makes the allowed set depend
   on how many slices a crate has, so a new module slice silently narrows a
   crate-root slice's permissions.
2. **Top-level Rust source only** — `src/*.rs`, no recursion — plus
   `src/lib.rs`. Simple, and it is what a crate-root slice's own code actually
   is once its module slices are excluded, but it forbids a crate-root slice
   any subdirectory of its own that is not a slice.
3. **Refuse the form outright**: a crate-root slice may not run phases 3-7
   until it is converted to a module slice. Honest and cheap, and it forces
   slice 18 to a Phase 0 answer (`lid-rs-shape` names a crate, so it would have
   to become a module of one).

**Chosen: reading 1.** It is not a trade against the other two so much as the
faithful definition — a crate-root slice's code is its crate's `src`, and
another slice's directory is that other slice's code, not this one's. The
objection raised against it (a new module slice silently narrowing a
crate-root slice's permissions) is the rule working: when code moves into a
slice of its own, it stops being the crate-root slice's to write. Reading 2
forbids a crate-root slice any subdirectory that is not a slice, which
`cargo-lid-rs` would fail today; reading 3 refuses a form the layout defines
and would force slice 18 and slice 23 to a Phase 0 rename apiece.

The defect blocks **two** of the remaining slices, not one: slice 18
(`lid-rs-shape`) and slice 23, whose document belongs at
`lid-rs-pipeline/src/lld.md`. Both are crate-root.

### Deferred
1. Worktree isolation per phase worker (see Decisions).
2. The workflow's Phase 8 path: with `lld/<slice>--<change>` settled, the
   precondition's "first phase without a commit" reading still counts only
   commits made on the branch itself, and a change branch cut from a merged
   `main` needs that reading to start at the branch point.
3. A documentation phase: the cascade a slice causes in README, CLAUDE.md,
   and the skill is no phase agent's to make under the policy; today it is
   the human's, or the main session's outside a LID phase.
4. A phase with nothing to do: on a Phase 8 edit whose layer 0 is already
   leaves, Phase 4 has no edit to make, and the stop hook's "nothing staged"
   refusal is the right answer to the wrong question. The workflow runs
   every phase; a phase that ends with a stop block saying so is today's
   path, and the session skips it by hand.
5. `rust-analyzer` in `rust-toolchain.toml`'s components, so the LSP tool
   works for the reviewer without a manual install.
6. A gate that outgrows the watchdog again: Claude Code ends a subagent
   that makes no stream progress for 600 s, and the whole of a Phase 7 gate
   runs inside one stop hook with nothing to report until it finishes.
   Scoping check 12 to the phase's own diff brings this workspace's gate
   well inside that bound, but the bound belongs to the slice, not to the
   tool — a phase that adds enough claim groups at once, or a project whose
   suite is slow, reaches it again, and the hook is killed before it can
   commit. Phase 7 is then committed by hand with the gate's run recorded
   in the body. What would remove the bound rather than raise it is a hook
   that emits progress while the gate runs, so the watchdog sees a live run
   instead of a silent one.
7. A Phase 8 edit that subtracts has no red run: the behaviour change
   *is* the shape change, so it lands at Phase 3 and every validation of
   it is green before Phase 5 writes one; such a Phase 5 is committed by
   hand with the reason in its body. A reword whose delta is observable
   has one, by the rule in Decisions: a leaf shared with claims outside
   the red set keeps its body, and Phase 5 writes or rewrites each
   red-set claim's validations to observe the delta — a companion path
   judged by the companion's table, a `--` name cut at the dash — so a
   validation that observed only what the reword kept is rewritten,
   since a red-set claim with a green validation fails the check.
8. Running each check under an OS sandbox from the hook — no network,
   writes confined to `target/` — so the residue in Security posture is
   bounded by the tool rather than by the environment it is run in.
9. The join in `gate_commit` between the check and the commit — the
   nothing-to-commit test over the editing set, then the staged set as what
   is staged — is observed at the two sets on a stopped tree, not as
   `hook_stop`'s verdict: a Phase 7 stop that passes the check is the full
   gate on the fixture, which no unit test carries. A `gate_commit` that read
   the staged set for both is therefore a mutant no test kills and none
   cargo-mutants generates. A seam on `checked` would let a test drive the
   stop past a stubbed check; that is a Shape change, and a later edit's.
10. An extra step has no catalog entry and no report. The pipeline's catalog
    fixes each command's inputs, report, and exit code; a workspace's own step
    is outside that vocabulary, so `cargo lid-rs catalog` neither names it nor
    writes a report for it, and its failure reaches a reader only as the
    check's output. A catalog entry per configured step is the shape that
    would close this, and it needs a name the project supplies.

## References

- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate this tool runs at Phase 7), [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the phases and who owns each), constraint 2 (no source parsing).
- `docs/intent/skill/lld.md` (workspace) — the working-state convention, the evidence table that says why the agent must not run its own checks, and the interactive mode this slice changes.
- `docs/intent/sync/lld.md` — the strict mirror rule this slice extends to the agents, hooks, and workflow.
- `docs/intent/cargo-lid-rs/lld.md` — the registry dump and the item-path mapping `check_red` reuses.
- Claude Code hooks, subagents, and workflows (`code.claude.com/docs/en/hooks`, `/sub-agents`, `/workflows`) — hook input fields, block semantics, `additionalContext`, frontmatter hook syntax, and the `.claude/agents` and `.claude/workflows` locations.
