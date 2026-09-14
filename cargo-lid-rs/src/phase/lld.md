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
"book"]]` lands *after* the gate_extra change's own Phase 7, once the installed
`cargo-lid-rs` has been refreshed from the merged tree, in the same
documentation commit that changes CLAUDE.md's gate block from `mdbook build
book` "run by hand" to a configured step. The order is forced by the hooks
running the installed binary rather than the working tree: a key added earlier
is a key the old binary does not read, so it would not make `mdbook build
book` run in the gate_extra change's own gate, and it would sit in the manifest naming a
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

Two costs stay the workspace's to weigh: the wall clock, and the `PATH`
dependency. An extra step's time is added to the detached job's own
running time, not to the stop hook's call, so no watchdog bounds it
(Deferred 6). And a step's program is resolved on the machine's `PATH`, so
a gate configured with a tool the machine lacks fails there and passes
elsewhere — which is what an extensible floor means: the tool answers for
the floor, and the project answers for the rest. An extra step is the project's, not the pipeline's, so
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
(`cargo-lid-rs/src/headless_canopy_agent/lld.md`) calls the same three with a
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
  hook undoes a replaceable tip if the phase is being reworked, then checks
  the subject against this phase's tag and, at Phase 7, against the version
  the bump will produce — both before either the bump or the check has run.
  For Phases 1–5 it then runs `phase-check <n>` directly and blocks on it.
  For Phase 7 it asks `gate_job::poll` instead: the first ask bumps the
  version once and starts the whole gate as a detached job; every ask
  thereafter is one file read. A settled `Done` (pass or fail) is what
  every other phase's check answering is: on success it stages the phase's
  staged set — the allowed paths, plus at Phase 7 the two files the bump
  wrote, and nothing else — commits the message with the tally appended as
  trailers (`Lid-Rs-Phase`, `Lid-Rs-Agent`, `Lid-Rs-Tools`, `Lid-Rs-Checks`,
  `Lid-Rs-Refusals`, `Lid-Rs-Reworks`), and allows the stop; on failure it
  refuses through `refusal_for`. A `Started` or `Running` answer — only
  possible at Phase 7, while the detached job has not yet finished — is
  neither: it allows the stop *pending*, staging and committing nothing,
  the tip restored if one was undone (The gate runs detached, below). Any
  failure after the undo restores the tip before the refusal, so the
  attempt's record reaches the next stop. Nothing changed under the
  phase's editing set is a refusal ("no change to commit"), asked before
  the undo and before the bump or the check. A subject whose tag is not
  this agent's phase is a refusal, as is a Phase 7 subject whose version
  is not the bump's.
- ```` ```stop ```` — the numbered decisions that block the phase. The hook
  commits nothing and allows the stop: an honest "this needs the human"
  must always be possible, and the decisions travel to the reviewer's seat
  through the workflow's structured output.

A message with neither block, or both, is refused with the format.

**A rejected phase replaces its commit; it does not stack a second.** A phase
has one commit, whatever it took to make it. A reviewer's rejection sends the
findings back to a worker and the phase runs again; the stop that follows
replaces the commit the rejected attempt made rather than following it. The
branch then reads as the walk reads — one `phase 2:`, one `phase 3:`, one
`phase 4:` — each commit holding its phase whole.

Stacking is what that replaces, and it costs three things. The log shows two
commits with one subject and no way to tell which is the phase. The reviewer,
whose instruction is to read *the newest commit on the branch*, reads a
rework's delta and never sees the phase it is judging. And at Phase 7 the
gate's own base moves: `gate_base` answers the newest `phase 7:` commit
reachable from `HEAD`, which after a stacked rework is the rejected attempt, so
the mutation step diffs against it and covers the rework's few lines while the
commit presents itself as the slice's gate.

**Which tip is replaceable.** The tip is read once — `tip` answers the branch
tip's hash, subject and body from one `git log -1`, or none in a repository
with no commit — and `Replaced::of` turns that reading into the record a
replacement needs, its agents and counts through `trailer_of` and
`tally::from_trailers`. What is left for `replaced_tip` is the decision alone,
over three conditions, every one of them a property of a commit and none of
them an argument:

- **its subject carries this phase's tag** — `tag_of(subject)` is
  `Tag::Checked` of this phase;
- **its body carries a `Lid-Rs-Phase` trailer naming this phase** — the hook's
  own signature, which no agent holds the git to write;
- **it is not reachable from `main`** (`on_the_trunk`), since a commit on the
  trunk is never this branch's to replace.

The first two are separate rules and not one read twice: they fail
independently, and a commit made by hand — a `phase 7:` commit a human wrote
when the watchdog killed the gate — carries the tag and no trailer, which is
exactly the commit the hook must not silently swallow. The branch supplies the
slice: a phase runs on `lld/<slice>` or `lld/<slice>--<change>`, so a tip of
this phase on this branch is this slice's own attempt, and the subject's
`for <slice>` field is never parsed — a Phase 7 subject has no such field to
parse. A repository with no `main` cannot show that a commit is on it, and the
first two conditions carry the decision there. A tip failing any of the three
is left where it is and the stop commits on top of it, exactly as a first
attempt does.

**The tip is undone after the nothing-to-commit test and before the bump and
the check.** `git reset --soft HEAD~1` is the whole mechanism: the commit stops
being a commit, every change it carried stays in the index, and `HEAD` becomes
the commit the replacement will sit on. The position in the order is what makes
every later reading correct without any of them knowing that a rework is in
flight. `manifest_at_head` reads `HEAD:./Cargo.toml`, so after the undo it
reads the manifest below the release and the bump recomputes the version the
replaced commit carried. `gate_base` — and `mutation_base` through it — answers
the newest `phase 7:` commit reachable from `HEAD`, so after the undo it
answers the gate *below* the one being replaced. The red set takes that same
base, so its diff is unmoved. And `changed_within` sees both rounds, which is
what the replacement must carry. Undone after the bump or after the check,
every one of those readings would have to be told which commit to skip, and a
reading that forgot would silently narrow the gate; undone first, none of them
changes at all.

The nothing-to-commit test is the one question that has to be asked ahead of
the undo. Before the undo the phase's editing set holds only what this round
wrote, so a rework that changed nothing is refused as having nothing to commit
and the rejected commit stays where it is. After the undo that set holds both
rounds, and the same question would answer *yes* for an agent that did nothing
at all — having already deleted the tip it would have amended. So the order is
the subject's tag, then the editing set, then the undo, and only then the bump
and the check. Asked that early the test also costs a second rather than a
gate, which is the reason the subject's version is compared before the check
too.

Only half of that order is a claim, because only half of it can be made red.
*Undone before the bump and the check* has a wrong answer a validation
observes: a Phase 7 stop over a replaceable tip, with an edit in the editing
set and a subject naming the wrong version, leaves the tree at the version the
replaced commit carried, and leaves the branch tip a *new hash* carrying that
commit's subject — the restore's signature. An undo placed after the bump
leaves the tree one patch level higher and the tip's hash untouched, so the two
orders are told apart without the check ever running. *Asked after the
nothing-to-commit test* has no second
behaviour to observe on its own; it is observed through
`NothingChangedInTheEditingSetIsARefusal`, whose validation now asserts that
the refusal it already described leaves the replaceable tip standing. So the
claim is the first half, the second half is this paragraph, and the existing
claim carries it.

**This reorders two validations that are green today, and Phase 5 rewrites
them.** Both drive a Phase 7 stop with *nothing* changed in the editing set
and expect the bump to have run —
`phase_sevens_stop_bumps_the_patch_version_from_the_manifest_at_head` in the
slice's module, validating
`PhaseSevensStopBumpsThePatchVersionFromTheManifestAtHead`, and
`nothing_changed_in_the_editing_set_is_a_refusal_read_from_the_changes_within_it`
in `integrity`, validating `NothingChangedInTheEditingSetIsARefusal`, which
asserts the two root files dirty afterwards. Under the new order the stop
refuses before the bump, so both become false about the tool without either
claim changing: each must first make an edit the editing set admits, so the
stop reaches the bump it is asking about. Neither claim is in the rework change's red
set, so this is work on claims that must stay **green**, by the same rule the
companion-seat row states — a validation that has to be rewritten because the
behaviour around it moved, not because its own claim did.

**A reworked gate is a whole gate.** With the tip undone, the mutation step's
base is the newest gate commit *below* the replaced one — the base the rejected
attempt ran against — so check 12 sees everything the slice changed rather than
the difference between two attempts at one phase. It costs a second full
mutation run for every rejected gate, which is the price of the commit meaning
what it says; the base is still a gate commit and not the trunk, so the run
stays the proportionate one. The run itself is detached (below); the undo
puts `HEAD` back at exactly the commit the replaced attempt's own key
already named, not a new one, so `HEAD` is not what tells the two apart.
What does is the fingerprint: the nothing-to-commit test, asked before the
undo, refuses a rework that changed nothing in the editing set, so a
rework that reaches the undo at all has already changed a byte the
fingerprint walks, and the fresh key computed after the undo differs from
the replaced attempt's on that field. The replaced attempt's job or result
therefore fails its key on the next poll and is never mistaken for the
rework's own — the wall-clock cost of the second run is unmoved, but it no
longer happens inside a blocking stop call.

The version follows the same reading. The bump raises the patch level of the
manifest at `HEAD`, and after the undo `HEAD` is the commit before the release
— so a reworked Phase 7 writes the version the replaced commit carried, and a
rejection spends no version number. The subject the agent writes is therefore
the same across attempts, and the comparison of subject to bump is unchanged.

**What the replacement's trailers carry.** The trailers measure how the phase
was made, and a phase made twice was made by one agent or by two. A resumed
worker keeps its `agent_id`, and the tally is filed under that id, so its
counts already include the rejected attempt; the replacement's counts are that
tally alone, and adding the replaced commit's numbers would count every call
twice. A fresh worker — which is what the unattended workflow spawns for a
rework — starts a tally at zero, so the replacement's counts are the replaced
commit's added to this agent's. The discriminator is the replaced commit's own
`Lid-Rs-Agent` trailer: it names this agent, or it does not.

`Lid-Rs-Agent` then names every agent whose work the commit carries, in the
order they worked and none of them twice, so a commit still names the record of
how it was made when that record has two halves. `Lid-Rs-Reworks` carries the
number of commits this one replaced — one more than the replaced commit's own
value, and zero on a first attempt — because the attempt count is the review
signal stacking made visible, and a replacement that dropped it would make a
phase that took four tries read exactly like one that took one.

```text
Lid-Rs-Phase: 4
Lid-Rs-Agent: a580cd3d769f4ec4e, b12f0c94ee3a71d60
Lid-Rs-Tools: 13 edits, 26 observations, 0 commands
Lid-Rs-Checks: 13 post-edit, 3 stop
Lid-Rs-Refusals: 0 policy, 0 stop
Lid-Rs-Reworks: 1
```

**What the undo does not touch.** It writes `.git` and nothing in the working
tree, so no path joins the staged set and what the hook writes is unchanged;
the agent still holds no git, and the one destructive git operation in the
design can remove only a commit the hook itself made, on this branch, for this
phase, with every byte of it kept in the index. The replaced commit's paths are
the same phase's paths in the same crates, so the staged set already covers
them — unless the phase's policy narrowed between the attempts, in which case
the stop is refused naming a path the phase may no longer write, which is the
current policy answering for the commit it is about to make.

**A refused stop puts the attempt back.** The undo runs before the check, so
any failure after it — the check, the integrity pass, a staging that refuses —
would otherwise leave the branch one commit shorter than it was, with the
rejected attempt's subject, body and trailers surviving only in the reflog, and
the next stop, finding no replaceable tip, would stack after all. So any
refusal that follows an undo re-commits the attempt before it returns —
**from the replaced commit's own tree object**, not from the index:
`git commit-tree <hash>^{tree} -p <hash>^ -F <message>`, the message being the
subject and body the `Tip` already holds, then `git update-ref HEAD <new>`.

Building the commit from the stored tree is what makes it exact rather than
approximate. A commit of the *index* would be exact only while nothing had
staged, and something can: `stage_and_commit` runs `git add` before it commits,
so a failure between the two leaves this round's edits in the index, and a
restore reading it would fold them into the attempt and call the result the
attempt. Naming `<hash>^{tree}` asks for the bytes the replaced commit
recorded, whatever the index holds, and `commit-tree` with `update-ref` writes
no index and no working-tree file at all. The result is the same tree, the same
message, the same trailers, a new hash — so the restored commit is not a new
commit of the phase's work: it *is* the attempt that was already there, which
is why "a commit is what a passing check produces" stays true through the
restore. What the branch loses is one hash; what it keeps is the fact that this
phase has been attempted, and how.

This round's edits stay where they were, in the working tree and wherever the
failure left the index. The next stop sees them, passes the nothing-to-commit
test, and runs the check again — and if it too fails, its undo leaves the same
tree to restore, because the tree it undoes is the tree this one restored.
Nothing here bounds a re-stopping agent; the bound is the eight-refusal cap
Claude Code already enforces, after which the phase is uncommitted, the tree
dirty, and the run reported as stopped. A restore that fails is the refusal's
own failure and is reported as one — a stop that cannot put the attempt back
must say so rather than return a refusal that quietly shortened the branch.

Both halves take the record and answer to it: `undo_tip` and `restore_tip` are
each handed the `Option<&Replaced>` and do nothing when it is none, which is
where the "was anything replaced" decision lives for them, as it does for the
three tally leaves. So the stop's chain reads the same whether a phase is on
its first attempt or its fourth, and `gate_commit` holds no branch on
*that* — whether anything was replaced: it undoes, calls `after_undo` for
everything the undo makes safe to ask, and hands that call's `Err` to
`restore_tip` on the way out. (At Phase 7 it holds one branch of a
different kind, over the poll rather than over replacement — The gate runs
detached, below.)

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

**The subject carries the version the bump will write.** A Phase 7 commit's
subject is `phase 7: <version>: <what and why>`, and `<version>` is the
version the bump will write — the one the tarballs will carry, not the one
the branch started from. The agent can compute it: the manifest is readable
and the rule is one patch level. The hook holds them to that version without
running the bump to get it — `bump_patch_version(&manifest_at_head(project)?)`
is a pure read of the manifest committed at `HEAD` — and compares them
**before the poll and so before the bump**, on every Phase 7 stop alike,
whether that stop is the one that starts the job or the tenth that finds it
still running: a subject naming the wrong version is refused in a second
rather than after a full gate, and refuses naming both versions — the same
shape as its refusal of a subject whose tag is not this agent's phase, and a
separate rule from it.

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
named differently from the set that is staged. It is also asked before
`gate_job::start` ever bumps anything on this stop — the rework order above
puts it first — but that is no longer the same as asking it of a tree the
bump has never touched: once a job has started, a bumped, uncommitted tree
is steady state for as long as it keeps running (What a pending ending
leaves behind, above), so a later stop's nothing-to-commit test can meet a
tree the bump already wrote to, on a poll that changes nothing. Either way
the answer is the same, because the set it reads is the reason rather than
the timing: the editing set never admits the two files the bump writes, on
a fresh tree or a bumped one alike.

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

### The gate runs detached

**The rule is not "fit inside a limit."** Two execution substrates that host
a phase agent's stop refuse, independently of each other, to hold a call
open for the length of a slow step. Claude Code ends a subagent that makes
no stream progress for 600 seconds; measured on this workspace, three
consecutive Phase 7 workers died inside the stop hook, one of them
instructed to make no edits, read nothing, and run nothing — which rules
out the worker and leaves the call. Canopy v3, the project's other client
(`cargo-lid-rs/src/headless_canopy_agent/lld.md`), mints the credential
that authorises a tool call's *completion* before the call runs, with a
hardcoded ten-minute lifetime, no renewal, and no way to re-claim it: a
second claim before completion returns "again" with no credential, and the
reference client discards the result silently. Two substrates landing
within two minutes of each other, built by different teams for different
reasons, is evidence about what an execution substrate will hold open for
one tool call — not about one vendor's number. The rule this section states
is the general one: **the gate must not require any caller to hold a call
open for the gate's length**, under any host.

**The whole gate runs detached, not one step of it — measured once, here.**
`phase-check 7` end to end is 1087 seconds (measured 2026-09-13): check 12
— `Step::Mutants` — is 854 of it, across 250 mutants in 33 groups; the
other seven steps together with this workspace's own `gate_extra` book
build are about 233. That smaller figure already fits the 600-second
ceiling on its own, which is enough to detach check 12 alone,
leaving the other seven steps and `gate_extra` inline inside the stop's own
call. That split is unsound, for a reason no wider ceiling and no smarter
wait fixes: splitting the gate needs two mechanisms — one for the steps that
keep running inline and one for the step that does not — and a rule for
telling which is which, while the machinery that makes a poll answer
instantly (the key, the record, the fingerprint, below) has to exist
regardless of how many steps it covers. One job that runs the *whole* plan
`phase-check 7` builds — every step, in the order `plan` already returns,
`gate_extra` included — needs that machinery exactly once. README §4.5's
list still names what runs and in what order; only how the caller learns
whether the run has finished does.

**This is the consequence the narrowed base already paid, made visible.**
The "Check 12's diff base under the gate" decision narrows the mutation
step's base from the trunk to the newest gate commit, and states the
600-second limit as half its reason: measured 2026-09-11, `main` as the
base offered 1,390 mutants and about seventeen minutes; the newest gate
commit offered six. Measured again above, the same narrowed base has grown
to the figures this section opens with — still the phase's own diff, still
not the trunk, and already past what a blocking call can survive. A base
chosen to fit a runner's timeout is a base the methodology checks less
because of the runner, not because of what the phase changed, and a
narrowed base that keeps growing toward the ceiling it was narrowed to fit
will not stay narrow on its own. Detaching the gate is what makes the
base's width a question with an honest answer again: nothing here widens it
back toward the trunk, and whether to is a decision this section does not
make.

**Two cascades, not one.** This section reaches outside its own slice's
files in two places. The first is `mutants.rs`'s own scratch-path
parameters, below. The second is lighter: `cargo lid-rs gate-status` (Who
checks back, and how, below) needs an arm in `dispatch` and a line in
`USAGE`, both in `cargo-lid-rs/src/lib.rs` — the crate-root slice's own
code, even though `src/lib.rs` is already in *this* slice's own Phases 3
and 4 allowed set for an ordinary `pub mod` declaration. It costs no
claim, the same way `Project::setting_node` costs none: a dispatch arm
recognising one more subcommand name has no wrong answer
distinct from what `UnknownSubcommandsFailWithUsage` already refuses,
exactly as none of `canopy`, `coach`, or any other existing arm carries a
claim of its own. What `gate_job::status` answers once dispatched there is
this document's claim, not the arm's.

**The one piece this cascades into another slice's document.**
`mutants::run_at(project, args, diff_path, output_root)` (`src/mutants.rs`,
the crate-root slice's own — `cargo-lid-rs/src/lld.md`) is the door that
takes the diff file path and the output root as parameters rather than
resolving them itself, so a stale job's cleanup cannot delete a fresh
job's still-being-written output (The scratch space, and the result it
reads, below, gives the full account). `mutants::run(args)` stays the
argument-parsing door its two existing callers use, resolving that pair
itself through `mutants::default_paths`; `run_at` is what a caller already
holding a `Project` and two distinct paths of its own — this document's
own detached job — calls directly. What that door must produce is this
document's to say; the door itself, and its claim, belong to that slice's
own document, not this one's.

**The shape.** `after_undo`'s Phase 7 arm no longer calls `checked` (which
spawns `phase-check <n> --slice <slice>` and blocks on it) — every other
phase's arm still does, unchanged. What replaces it is `gate_job::poll(project,
phase, slice, crates)` — work, not decision, composed of small leaves that
each do one thing: `gate_job::key` (the fresh key, below), `read_record`
(one file read), and `act`, the one leaf that actually dispatches — the
three-way decision over what those two answer. `act` in turn hands the
matching- and foreign-key cases to `settle`/`restart_or_running` and
`contest` respectively (deciding, for a record under this key, whether a
result is already written and, if not, whether the pid is still alive; and
the same for a record under a foreign key), and to `start` for the no-record
case (spawn the whole plan as a detached child, then hand `running_record`'s
answer to `write_record`) — no leaf writes a finished outcome back into the *running*
record, since that record only ever names a job as running; a leaf's own
`Err` is reserved for the leaf's own failure, never for "still running."
`GatePoll` is the whole of what a call that does not fail can mean:
`Started`, `Running`, or `Done(Result<(), String>)` — three variants
answering three of `act`'s four branches, the fourth being the `Err` a
live, *differently keyed* job returns outright (below) rather than
anything `GatePoll` names, since that case is not this key's job to report
on — four branches, not a two-versus-five mismatch.
`gate_job::poll` runs only inside the process the stop hook itself is; a
bare `phase-check 7` — what a human or CI runs — never calls it, and
`Step::Mutants` there still calls `mutants::run` directly with its
ordinary fixed paths and blocks, exactly as today.

**Where the third ending is decided, and where it is not.** `after_undo`
gains no third return: its answer stays the two it always had, the staged
set or a failure, because the call that can produce `Started` or `Running`
never happens inside it. `gate_commit` asks the poll itself, once the tip
is undone and before `after_undo` is ever called for Phase 7: a `Done`
answer — pass or fail — goes on to `after_undo` exactly as `checked`
succeeding or failing always did, so the staging, the integrity pass, and
a failure's restore are unchanged code paths reached by a new door. A
`Started` or `Running` answer never reaches `after_undo` at all: `gate_commit`
restores the tip — exactly as it does for a refusal (A pending gate is not
a refusal, below) — and returns the third ending directly, in a small enum
of its own, `GateCommit::{Committed(String), Pending(gate_job::JobKey)}`.
The gate's result is therefore read once and never re-run: because
`after_undo` calls `checked` for no phase but the five that always did,
a `Done(Ok(()))` poll cannot be followed by a second, synchronous run of
the check — there is no code path left that would make one. `checked`
records `Event::StopCheck`, and it is `checked` alone that does: at Phase
7, `gate_commit` tallies the same event itself, once, at the stop that
reads a settled `Done` — never at a `Started` or `Running` poll. So
`Lid-Rs-Checks`' stop count still names how many times a phase's check ran
to a finished verdict, at every phase, and not how many times a worker or
an orchestrating session polled for one.

**Why the poll moves ahead of the bump, and why that is still the same
key.** Asking the poll ahead of `after_undo` is what keeps the bump to
once per job: `gate_job::start`, which raises the workspace version and
runs `cargo update --workspace --offline`, is reached only from the
poll's own no-record branch, so on every stop while a job is `Running`,
`act` never reaches `start` at all — nothing re-bumps and nothing
re-runs `cargo update` alongside a detached child that may itself be
mid-`cargo check` or mid-`cargo package` on the same tree. The bump
happens exactly once, inside `gate_job::start`, immediately before the
child is spawned —
which is also what lets the child test the *bumped* tree, the whole reason
Phase 7 bumps before it gates. Moving the bump earlier does not move the
key: `gate_job::key`'s fingerprint is `policy::workspace_paths`'s answer —
the phase's editing set — and the bump writes the workspace root's
`Cargo.toml` and `Cargo.lock`, which no phase's editing set ever admits
(The two root files are checked, not exempted, above). A key computed
before the bump and one computed after it are therefore the same key, so
asking the poll first costs nothing a later ask would have answered
differently. The subject-version check does not wait on the bump to stay
this cheap either: `bump_patch_version(&manifest_at_head(project)?)` is a
pure read of the committed manifest, so `gate_commit` can check the
subject against it — and refuse in a second, exactly as today — on every
Phase 7 stop, whether that stop is the one that starts the job or the
tenth that finds it still running. And once the poll finally answers
`Done`, `bumped_files_untouched` is asked against the bytes `gate_job::running_record`
read back once the bump has run — not recomputed. `Cargo.toml`'s new
line is a pure function of `HEAD` and could be recomputed; `Cargo.lock`'s
is not: nothing short of running `cargo update` again — a write, not a
check — derives what it must contain, and comparing the working tree's
lock to itself proves nothing. So once `start` has bumped and spawned the
child, it asks `running_record` for both files' bytes — untouched since
the bump, since nothing between the two touches either — packaged with
`key` and `pid` into the record `write_record` then writes, and the stop
that later reads `Done` reads them back from there rather than asking any
question of `HEAD` a second time.

**What a pending ending leaves behind, and why that is safe.** Moving the
bump into `gate_job::start` means every ending this section adds has
already written it by the time that ending is returned: `Started` bumps
before it answers, and `Running` — and a later `Started` under the same
key — finds the bump already there, since nothing between one poll and
the next touches those two files again. A pending ending is no longer the
rare case; it is the ordinary shape of a Phase 7 stop for as long as the
job it started keeps running, so a bumped, uncommitted working tree is
now steady state and not a symptom. Measured here, 2026-09-13: three
Phase 7 workers were killed inside the stop hook in one day, each leaving
its own bump behind; one of those leftover bumps sat at a version `main`
had already published, and a naive retry from it would have packaged that
same version a second time — recovery needed `git checkout --
Cargo.toml Cargo.lock` before anything else could proceed. Under the
design this section replaces, a leftover bump was diagnostic: finding one
meant a worker had died mid-gate. Under this one it means nothing has
failed yet: a bumped, uncommitted tree left by a job still running and one
left by a worker that died gathering the gate look identical from the
tree alone, and only the record — never the working tree — says which.

Two of the three facts this document already holds do close part of this.
`bump_workspace_version` reads `manifest_at_head`, never the working tree
it writes to, so a leftover bump and the bump a fresh job would write from
the same `HEAD` are byte-identical — which is why `bumped_files_untouched`
holds the tree to the bytes `gate_job::running_record` read back, not to
bytes recomputed later (Why the poll moves ahead of the bump, above): there is
nothing for a stale bump to *disagree* with while `HEAD` has not moved.
And the fingerprint never sees the bump at all (below), so a
leftover bump can neither invalidate a key nor be caught by one.

**The collision itself is not closed, and saying it is was backwards.**
The published-version collision measured here happened because `HEAD` had
**not** moved, not because it had. `bump_workspace_version` reads the
manifest at `HEAD` and raises its patch level by one; as long as `HEAD`
sits still — which is exactly the state a repeatedly-restarted leftover
bump produces — every fresh `start` recomputes the identical version.
When `main` has, meanwhile, published exactly that number (another
slice's `phase 7:` landed one patch ahead of what this branch's `HEAD`
still names), the recomputed bump collides with it every time, not once.
The key's inclusion of `HEAD` (The key, and what it must include, below)
decides whether to trust an *old record* — it discards a stale job or
result the moment `HEAD` changes — but it says nothing about the *version*
a correctly, freshly keyed job goes on to compute: a start under a brand
new key, on a `HEAD` that has not moved, recomputes the same colliding
number as cleanly as a stale one did. What actually resolved the measured
incident, 2026-09-13, was a human merging `main` into the branch — the one
operation that moves `HEAD` past the version `main` had already taken, so
`manifest_at_head` reads a higher base and the next bump no longer
collides. This is a residue this section does not close: its resolution
is merging `main` (or otherwise moving `HEAD`) before the gate is spent
again, not a mechanism this design supplies.

**A second residue this section does not close: noticing.** Nothing in this design watches a
checkout for a bump left by a job nobody polls again — a worktree
abandoned outright, or one whose `gate-status` nobody runs again after
its worker dies — and `gate_job::alive`'s own fallibility cuts both ways:
a pid can go on answering alive long after the process it named is gone,
if the number is reused or inherited by an unrelated one, so "abandoned
permanently" is a state this design can reach, not only one it could in
principle. Nothing here clears a bump on that path; it is left to
whoever next runs `git status` on the checkout, or to the orchestrating
session's own judgment about a worktree it has stopped polling, exactly
as a human already discards one by hand today. Saying so plainly is the
honest answer; a sweeper this section does not otherwise need would be a
mechanism with no claim to attach it to. And the bump is invisible to the
key in both directions regardless of who notices it: `gate_job::fingerprint`
walks only `policy::workspace_paths`'s answer, which no phase's editing
set ever admits the workspace root's `Cargo.toml` or `Cargo.lock` into
(The two root files are checked, not exempted, above), so a bump can
neither invalidate a key it plays no part in nor be caught by one that
was never asked to cover it.

No branch turns on any of this: the bump's idempotence, the key's
inclusion of `HEAD`, and the fingerprint's exclusion of the root manifest
are each already a fact this document's existing claims establish, and
this passage draws the conclusion rather than adding a new one. It costs
no claim of its own.

**A read-only door onto the same state.** Nothing above lets an
orchestrating session learn any of this except by starting a job — poll's
`act` always spawns one when it finds none, and refuses outright over a
live job under a different key. A caller that only wants to know, never to
start and never to be refused, needs its own door: `gate_job::status`.
It shares `act`'s top-level, three-way classification of what `read_record`
and the fresh key say — no record, a matching key, a foreign key — but the
classification is factored out of the starting rather than threaded
through it: `matching_status` and `foreign_status` make the same
finished/alive reads `settle`/`restart_or_running` and `contest` already
make, and answer `GateStatus` instead of spawning anything. A situation
`poll` would answer `Started` for instead answers `NoJob`; a situation
`poll` would refuse outright — a live job under a different key — answers
`Foreign(other key)` instead of failing, because there genuinely is
something to wait for, even though it is not this key's own job. `cargo
lid-rs gate-status` puts that on the command line (Who checks back, and
how, below); it is the deterministic read surface this section needs so
that "the orchestrator sees a Phase 7 that ended pending" is answered by a
file, not by an agent's word about one.

**The detached child's own command line.** Building it is a leaf of its
own, not work folded into `start`: `gate_job::child_command(phase, slice,
key)` appends `phase-check <n> --slice <slice> --job-key <key's hash>` —
`n` `policy::number_of(phase)`'s answer — to whatever `self_command()`
already carries, and hands back the whole `Command`, built and not run.
Which program runs a subcommand in a fresh process, and what (if
anything) already precedes these six arguments on its line, is
`self_command()`'s own decision, made elsewhere: an installed binary and
a checkout answer it differently without either changing which job this
child is being asked to run, and `child_command` leaves that choice
unexamined. `start` composes five moves, not three: bump, ask
`child_command` for the line, spawn what it returns unwaited, ask
`running_record` for the record that key and that child's pid make, and
hand it to `write_record`. The line is the part of a detached child a
caller can be held to — a fact about a string, true or false with nothing
running — where spawning it, and leaving it running, are facts about a
process, a different kind of question this leaf does not ask and `start`
alone answers.

**The record is a value too, built where nothing runs.** The line was
split out of `start` because which command a child runs is a fact about a
string, answerable without starting a process; the record's contents are
the same kind of fact about a value, and the split is the same one.
`gate_job::running_record(project, key: &JobKey, pid: u32) ->
Result<JobRecord, String>` builds the `Running` record from exactly what
it is given — the key and the pid unchanged, and
`integrity::contents_of(project, &policy::hook_written_paths(Phase::Seven))`'s
answer for `Cargo.toml` and `Cargo.lock`, in that call's own order, as
`version_files` — and asserts nothing about a child, because it starts
none. `gate_job::write_record(project, slice, record: &JobRecord) ->
Result<(), String>` is the other half: one file write, of the record it is
handed, to `record_path(project, slice)?` — a caller's record, not one it
builds itself, so a fixture can hand it any record, `running_record`'s
included, exactly as a fixture already hands `act` a `JobRecord` it never
ran a job to produce. `start` composes both in turn, once the child is
spawned: it asks `running_record` for the record this key and this
child's pid make, and hands it to `write_record` before returning.

Splitting the record out is what makes it assertable on its own terms, the
same reason splitting the line out was: a value built from a key, a pid,
and two files' bytes is checkable by handing `running_record` a project
whose `Cargo.toml` and `Cargo.lock` are known and asking what comes back,
with nothing spawned — where the only claim naming what the record
carries could otherwise be reached solely by driving `start` through a
real spawn, which no fixture in this module does (A residue the split
does not close, above). It also gives the poll's own paths something to
be exercised against: `settle`, `contest`, and their status siblings are
already handed a `JobRecord` a fixture built by hand rather than one a job
produced, and `running_record` is the door that shows such a fixture is
not invented out of nothing — it is the same value the real leaf would
answer.

Two of this document's own claims are reworded for it, a third correction
runs alongside them, and a fourth claim's citation moves with no word of
its own changed.
`TheRunningRecordCarriesTheBytesTheBumpWrote` names `start` as the one
that captures the bytes "at the moment it bumps them" — a sentence that
is only true of a leaf that bumps and captures in the same breath, and
`start` no longer does: the bump and the capture are two different
leaves' work, asked at two different moments. Renamed to
`TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder`, the
retired name kept beside it as a `#[deprecated]` alias, by the rule this
document already states for a reworded claim, its subject is
`running_record`, asked for the record after the spawn and answering from
nothing but what it is given. A validation of it needs no spawn: a
fixture calls `running_record` directly, with a key, a pid, and a project
whose two files are known, and reads back what it answers.

`AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord` keeps its name
and most of its sentence: `start` still bumps once, still spawns unwaited,
and a record still gets written before it returns, composed now through
`running_record` and `write_record` rather than inline. What the sentence
drops is "naming the key, the child's pid and the captured bytes" — a
clause no validation of `start` could ever reach, since reaching it needs
a completed spawn, and the claim above now carries exactly that content
where a fixture can reach it directly. Dropping a clause nothing here
could prove, in favour of a claim that can, is a correction and not a
rename (A reworded claim under the policy, Decisions) — the sentence that
remains was already true and stays true, narrower only because the wider
clause moved to where it is provable.

`PhaseSevensStopBumpsThePatchVersionFromTheManifestAtHead` is corrected
the same way, and its citation settles here rather than on the writes
this section retires. Its sentence names `bump_workspace_version` as
running "for a Phase 7 stop before the check" — a clause about *when*
the bump runs relative to a stop, provable only by driving a real stop
through to a real spawn, the same residue this section already leaves
open for `start` itself (A residue the split does not close: that the
child ran, and that nothing waits for it, below). The clause is
dropped. What remains — one patch level above the workspace `version`
the root manifest holds as committed at `HEAD`, read from `HEAD` and
never from the working tree, so a second bump before any commit writes
the same version again — was already true of `bump_workspace_version`
alone, and stays true with the stop context gone: a fixture calls it
directly, with no stop and no spawn, exactly as its own reordered
validation already does (Shared leaves on a Phase 8 edit, Decisions).
Dropping a clause nothing here could prove, in favour of the sentence
that can, is a correction and not a rename, by the same test the claim
above was judged against. Its citation belongs to
`bump_workspace_version` itself, reached on the live path only through
`gate_job::start`, immediately before the child is spawned.

`APhaseSevenSubjectMustCarryTheBumpedVersion` is renamed, not merely
recited: its sentence ends "after the bump and before the check runs,"
an order the code reverses rather than narrows, so the test this
document applies to a stale sentence gives a rename and not a
correction (Where the subject-version check runs, relative to the
bump, Decisions). Renamed to
`APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll`,
the retired name kept beside it as a `#[deprecated]` alias, reading:
"When the version `subject_version` reads from a Phase 7 `commit`
block's subject is not the one `version_the_bump_will_write` computes
from the manifest at `HEAD` — without running the bump — the stop
shall be refused naming both versions, before the poll and so before
any job begins." Its subject is `subject_version_matches`, and beneath
it `version_the_bump_will_write` — the pure read of the manifest at
`HEAD` that `gate_commit` runs ahead of the poll (Why the poll moves
ahead of the bump, above), on every Phase 7 stop alike, whether that
stop is the one that starts the job or the tenth that finds it still
running.
Both leaves already exist and are already validated against this
order — neither is work this section's Phase 3 skeleton answers
wrongly — so the rename carries no reddening case of its own. Its two
validators are renamed with it, unchanged in body:
`a_phase_seven_subject_must_carry_the_bumped_version_before_the_check`
and `a_phase_seven_subject_must_carry_the_bumped_version` each cite
this claim and nothing else, and check 14 holds a test to being named
for a claim it cites — a naming rule, which is exactly why the name
must move once the citation does, not a reason it can stay behind.
Marking the claim `#[lid(free)]` to spare the rename is the wrong
instrument: `free` exempts a claim from the controlled-language check
along with check 14, so reaching for it here would buy two untouched
test names by quietly dropping a real check on the sentence. Renamed,
both still assert a mismatch refused with no gate spent on either
version and nothing committed, which is exactly the order the
corrected sentence now states.

`phase::run` gains an optional `--job-key <hash>` beside `<n>` and
`--slice`: absent, it calls `check` and blocks exactly as it always has —
the human fallback, below, and every call `checked` still makes for
Phases 1–5; present, it calls
`gate_job::run_job` instead. The flag carries a hash, not a key: a `JobKey`
is not a value a command line can round-trip, since nothing reconstructs
one from the one-way digest `--job-key` names. `run_job` does not try to.
It takes exactly what a bare `phase-check <n> --slice <slice>` already
gives it — `project`, `phase`, `slice` — resolves `crates` the same way
any other invocation resolves them (`SliceCrates::resolve`), and calls
`gate_job::key(project, phase, slice, &crates)` itself: the same door
`gate_job::start` called to compute the key it is spawning this child
under. Hashing that fresh key and checking it against the hash
`--job-key` carried is not a narrower stand-in for a wider flag that would
have passed the key's fields directly — it is a check, not a transport.
A matching hash is the proof that the child is about to gate the same
tree the parent keyed when it spawned it; a flag wide enough to carry
`checkout`, `head`, `base`, `slice`, and `fingerprint` verbatim would still
have to be *trusted*, and trusting a value handed across a process
boundary is exactly what a confused-deputy design refuses everywhere else
in this document. Recomputing costs the child three arguments it already
holds and nothing else.

A mismatch is a real condition, not a corner nothing reaches: between
`gate_job::start` computing the key and this call recomputing it, an edit
can land in the fingerprint's own walk — the same window "what stops a
worker from starting the job, editing, and stopping" (The key, and what
it must include, above) already reasons about, reached here from the
other side of the process boundary instead of the same one. A child that
ran the plan anyway would be gating a tree nobody asked it to gate. So a
mismatch runs no step of `plan` and writes to no result path at all:
not the path the given hash would derive, because `run_job` never held
the `JobKey` that hash names and writing there would be filing a result
under a key it did not compute — the one thing recomputing exists to
rule out; and not the path its own fresh key would derive, because
nothing polls a key this call was never asked to run under. The running
record `gate_job::start` wrote still names the original key and this
child's own pid, and once the child exits having written nothing, that
pid answers dead: the next poll's `settle` finds no outcome at the
recorded key, `restart_or_running` finds the pid gone, and starts a fresh
job under whatever key the tree computes then. That is the ordinary
died-before-finishing recovery every other crash already gets, not a
second mechanism built to answer a question the first one already
answers — a mismatch costs nothing more than looking, from the outside,
like a job that never got the chance to run.

On a match, every step but `Step::Mutants` is still delegated straight to
`run_step`, unchanged, and `Step::Mutants` alone is still diverted to
call `mutants::run_at` with `gate_job::prepared_paths(project, key)?`'s
paths in place of the fixed defaults — `execute_with` already takes a runner as a
closure, so `run_job` supplies its own rather than widening `run_step`'s
own dispatch with a branch nothing claims — then writes its one outcome —
`Done(Ok(()))` or `Done(Err(output))` — to
`gate_job::result_path(project, key)?` before exiting. `gate_job::start`
is what sets `--job-key`; a human or CI never does, so bare use of every
existing flag is untouched by this section.

**A residue the split does not close: that the child ran, and that
nothing waits for it.** `child_command`'s answer is a fact about a string
— a program and an argument list — true or false with nothing running,
and a fact about a string is exactly what the split bought: nothing here
needs to spawn a process just to ask it. That `start` spawns the command
that leaf returned, and leaves it unwaited rather than blocked on, is
carried by `AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord`
itself, not left outside it. What no validation in this crate reaches is
that clause alone: driving `start` far enough to observe a real spawn
needs a checkout whose bump succeeds, and the validation of that claim
drives `start` in a checkout whose bump fails instead, so the clause is
asserted and unobserved here, not unmade — an acknowledged gap, not a
denial that the claim makes it. What stands in its place is what already
stands for the rest of this section's reach beyond its claims (A passing
gate is not evidence the detached path ran, below): mutation testing over
the diff, check 12 among it, and a real `hook stop <n>` a phase agent
actually ends against a real checkout — the only thing that exercises the
detached path at all.

**A `Done(Err(..))` must not carry what the file already holds.** A
refusal built from a check's failure has always spliced the failing
step's whole captured output into the message `refusal_for` composes,
because that output had no other home. Measured here, 2026-09-13, one
such refusal reached 1.3 MB and killed the Phase 7 worker it was handed
to. Detaching the whole plan, and not only `Step::Mutants`, makes this
worse rather than better: the step that fails and supplies that output
can now be any of `plan`'s steps — the floor's build, test, doc, and
package steps included, none of them as compact as the mutation step's
own report — so the worst case this path can produce has grown, not
shrunk: the same whole-plan job "The whole gate runs detached, not one
step of it — measured once, here" (above) argues for on every other
ground costs this one. What removes the reason to splice the *whole*
capture is the file this section already writes: a `Done(Err(output))`
lands at `gate_job::result_path(project, key)?` whether or not a poll ever
reads it back, so by the time one does, the whole capture already has a
permanent address a refusal can name for anything past its own bound.

That bound cannot be a bare header, though. `refusal_for` derives the
skill's response to quote from the text it is handed, through
`check_of_output`, which is `lint_names` — every `clippy::` name the text
contains — falling back to `marker_check`, which looks for the literal
substrings `"is not red"` and `"survived"` (`ending.rs`). A header naming
the step plus a path contains none of those, so every detached Phase 7
failure — check 12's own survivors are the commonest — would fall through
`refusal_for`'s own fallback: "No gate row names this failure: it is a
plain compile or test error," which is false, and drops the second of the
three parts this document's own refusal promises ("the failing step's
output verbatim; the skill's correct response for the check that fired").
`gate_job::bounded_output(project, key, output)` therefore carries the
**failing block**, not a bare header: `execute_with`'s own failure is
`format!("{step:?} failed: {e}")`, and `bounded_output` keeps that header —
up to and including the literal `" failed: "` marker — followed by the
**last 8 KiB of `{e}` itself**, with a closing line naming
`result_path(project, key)?`'s own path for whatever the bound cut off,
`bounded_output` itself answering a `Result` for that one reason. Chosen, not
measured: 8 KiB is comfortably larger than any single clippy diagnostic
block or cargo-mutants' own survivor summary and orders of magnitude
smaller than the 1.3 MB capture that killed a worker. The tail, not the
head, is what is kept, because every marker `check_of_output` looks for is
emitted at the point the failing tool reports its own verdict — clippy's
denied lint, cargo-mutants' own "N survived" line — which is the last
thing that tool writes, never the first; a bound taken from the front
would keep the "Compiling…" noise and cut the verdict. `hook_stop`'s
Phase 7 arm hands `bounded_output`'s result to `refusal_for` as its
`output` argument in place of the detached job's raw capture, and
`check_of_output` runs on that same bounded string — the one `refusal_for`
already receives — so the check that fired is still named in the refusal:
a claim to that effect belongs beside `bounded_output`'s own. `refusal_for`
itself changes nothing about how it composes a message — only what it is
handed does. A synchronous check's own failure, at every phase but this
poll, still hands `refusal_for` its output whole, exactly as it always
has: a tighter bound that scopes every refusal to the failing diff rather
than to a fixed-size tail is not built here (Deferred 11), and this
passage decides only what the detached path forces — that a poll's own
failing answer must not reproduce the 1.3 MB refusal it stands in for,
while still naming the check that fired.

Whether `" failed: "` is matched at its first occurrence: yes, and that is
safe. The format string `"{step:?} failed: {e}"` always places the literal
`" failed: "` immediately after the step's own `Debug` text, which is a
short enum name or a handful of quoted words — never a string containing
that literal itself — so the *first* occurrence of `" failed: "` in the
whole formatted string is always the header's own, regardless of whether
`{e}` goes on to contain the same substring later, which cargo's own
output routinely does (`error: linking with \`cc\` failed: exit status: 1`
is an ordinary rustc message). Searching from the first occurrence finds
the boundary the format string actually wrote; searching for a *later* one
would not, since nothing later is guaranteed to exist at all.

**The verdict stays out of an LLM's hands.** Moving the gate off the stop
hook's own call must not move the *verdict* off it too. Three Phase 7
workers were killed inside the stop hook in one day (above); two of them
had already written a commit message asserting the gate had **passed**,
when a hand-run of the same gate showed it had **failed** check 12 with
seven survivors. Whatever runs detached, the thing that decides pass or
fail — and that stages and commits when the answer is pass — stays the
same deterministic code this document already trusted with that decision:
the detached job runs `check`'s own plan, unchanged, and the stop hook's
own staging and commit logic runs, unchanged, once a passing `Done` is
read back. Neither the orchestrating session nor the phase agent is ever
asked what the gate decided; both only start a job and read a record —
the phase agent's own stop hook for a `Done` it can act on, and every
other reader, human or orchestrating session alike, through `cargo lid-rs
gate-status`, never through an agent's structured output. Tenet 2 — a gate
that exists, gates — holds under this section with the same non-LLM hand
on the trigger it always had.

**A pending gate is not a refusal.** Every ending this design had before
was one of two: `commit` — the check passed — or `stop` — the phase agent
names the decision. `Started` and `Running` are neither: the phase agent
proposed a commit, and the hook is not refusing it, only reporting that
the deterministic step that would prove or disprove it has not yet run to
completion. Answering these as a `block` would spend the worker's own
eight-refusal cap waiting for a job nothing about the worker can make
finish sooner, and would re-ask the worker, turn after turn, a question
whose answer has not changed since the last turn asked it. This design
answers instead with a **third ending**, `pending`: the hook allows the
stop — the phase agent's turn ends exactly as a `stop` block's does —
commits nothing, refuses nothing, and carries the key rather than numbered
decisions. No refusal is spent, and the eight-refusal cap this document
relies on elsewhere is untouched by a gate that has not yet answered. A
Phase 7 that ends this way is the one shape a reader of this design will
find surprising — nothing committed, and no error — and what makes it safe
is what already made the key safe for check 12 alone (below): the
fingerprint changes the moment the editing set does, so nothing about
ending here silently accepts a tree that moved after the job was asked
about it.

A pending ending still restores the tip when there was one to restore.
When the Phase 7 stop being polled is a rework's, `undo_tip` has already
removed the rejected attempt before `gate_commit` ever asks the poll, and
a `Started` or `Running` answer must not leave the branch sitting one
commit short while the job runs: `gate_commit` calls `restore_tip` on the
pending path exactly as it does on a refusal's, before returning the third
ending, so the rejected attempt comes back — same tree, same trailers, new
hash — and the next poll's eventual `Done` still has a `Lid-Rs-Reworks` and
a `Lid-Rs-Agent` to read from and add to. All three endings — committed,
refused, pending — leave the replaced attempt intact; that is what keeps
those two trailers honest across a poll, not only across a refusal. A
first attempt has nothing to restore, and `restore_tip` already does
nothing when nothing was replaced, so the same call is correct whether the
stop it is closing out is a fresh Phase 7 or a reworked one, without
asking which.

**Who checks back, and how.** The poller is the orchestrating session —
the workflow, or the interactive main session — whichever spawned the
worker whose stop answered `pending`, never the phase worker itself: an
orchestrating session has no inter-turn watchdog counting silence against
it, spends no worker's refusal cap by asking again, and re-runs no part of
the plan by asking, because a poll is one file read whichever session
makes it. It does not learn what happened from the worker's own report —
whether it committed, the ≤3 numbered decisions, any of it — because a
worker's word about the gate is precisely the channel this section removes
from the verdict (The verdict stays out of an LLM's hands, above), and a
pending stop carries none of that anyway, since nothing was decided. It
calls `cargo lid-rs gate-status` for the slice instead: `NoJob` means there
is nothing to wait for — the worker's own stop already committed or was
refused, by the ordinary path, and the branch already reads as it would
without any of this section; `Running` means resume or respawn the phase-7
worker later, whose own next stop — reached without a fresh edit — passes
the nothing-to-commit test exactly as an unedited rework's would and calls
`gate_job::poll` again, which, still running, answers `pending` again at
the cost of one more file read rather than a re-paid floor; `Foreign(other
key)` also means wait and check back later — a job is genuinely running,
just not this key's own, and the next poll answers `Started` under the
current key the moment it dies, or is refused again while it lives, so
resuming the worker later is the same move as for `Running`, not a third
thing to handle; `Done` means
the same resume, whose stop this time reads the settled result and reaches
the ending `Done` was always going to reach — a pass commits, following
the staging this document already describes, exactly as an ordinary Phase
7 does; a fail is handed to `refusal_for` through `gate_job::bounded_output`
(A `Done(Err(..))` must not carry what the file already holds, above), on
whichever worker's turn asks next. Nothing here counts how many times a
worker asked, or how
many times an orchestrating session polled; the record answers, not the
asking. Phase 7's own worker keeps doing what it always did — implementing
the leaves, then asking the gate about them — and remains the agent
`lid-rs/skill/SKILL.md` and the four slices that reference
`lid-rs-phase-7` expect it to be. What leaves the worker is not the phase;
it is the blocking call, and the commit's timing with it.

**The key, and what it must include.** A record answers a poll only when
its key matches one computed fresh, `gate_job::key(project, phase, slice,
crates)`: the checkout's own canonical path (`project.root()?`,
canonicalised), the branch tip's hash (`tip`'s `hash`), the same
`mutation_base(project)?` `Step::Mutants` already passes as `--diff-base`,
the slice's name, and the fingerprint below — over
`policy::workspace_paths(project, phase, crates)`'s answer, which is why
`key` takes `phase` and `crates` rather than resolving the phase's editing
set some other way: a two-argument `key(project, slice)` has no path to
`workspace_paths` at all, and cannot compute what it is asked to fingerprint.
The checkout is the addition a shared `target/` forces: `CARGO_TARGET_DIR`
need not be per-checkout, and this project runs phase agents in worktrees,
so two worktrees at the same HEAD and slice with an identical editing-set
fingerprint would otherwise compute one key and accept each other's result
for what are, on disk, two different builds. HEAD alone already fails a
narrower case: Phase 7's leaves are working-tree edits until the commit
that only happens once the gate has passed, so an edit after the job
starts changes nothing HEAD-shaped — which is why the fingerprint exists
at all, and is also the answer to "what stops a worker from starting the
job, editing, and stopping": the edit changes the fingerprint, so a done
result computed before it is never read as if it applied after. A
rework's undo (`undo_tip`) does *not* move `HEAD` to anywhere new — it puts
`HEAD` back at exactly the value the replaced attempt's own key already
named, since that attempt's job started from the commit below the one the
undo removes. What discards the stale job or result is the fingerprint:
the nothing-to-commit test, asked before the undo, refuses a rework that
changed nothing in the editing set, so a rework never reaches the undo
without having already changed a byte the fingerprint walks, and the key
computed afterward differs from the replaced attempt's on that field alone
— the same mechanism that catches an edit made after a job starts catches
a rework for free, because by the time it reaches this point a rework *is*
such an edit. The wall-clock cost of the rework's own fresh gate run is
unchanged, only no longer inside a blocking call.

The key is honest about what it does not cover. The fingerprint is built
from the phase's own editing set — `workspace_paths`' answer, the
directory and module file Phase 7 may write. Check 12's own subject is the
*whole workspace's* diff against the base, wider than that set. A write
outside it — which no phase's `Edit` tool can make, but the code the check
executes can (Security posture) — moves what check 12 sees without moving
the key. This document does not ask the key to catch that:
`outside_policy_clean`'s integrity pass already refuses the stop for any
such write, on every phase, key or no key. The key's job is narrower and
already stated: tell one attempt's result from another's, for the tree the
phase itself is allowed to have changed.

**The fingerprint, decided now.** `workspace_paths`' answer is a directory
and a module file, matched by prefix — not a list of files, so nothing
about it is hashable as given. `gate_job::fingerprint(project, admitted:
&[PathBuf])` walks each admitted entry — recursively for a directory, once
for a file — collects every regular file's path relative to the project
root together with a hash of its bytes, sorts the pairs by path, and hashes
the concatenation. Decided here, not deferred to Phase 5: a walk over what
is actually on disk includes a leaf the agent just wrote and git does not
yet track, which is exactly the case both rejected alternatives fail —
`git stash create` reads the index and the worktree but not untracked files
by default, and "tracked changes only" fails identically, since a new leaf
has no tracked change to be part of. Walking the filesystem directly needs neither git's index
nor a guess about a leaf's tracked state. **Guessed, not measured:** the
cost of walking and hashing a whole editing set is not yet measured against
a large slice's directory; a narrower fingerprint is the fallback if it
proves slow, and settling that is Phase 5's, not Phase 1's.

**A live, differently keyed job is refused, not waited for and not
restarted.** A start is refused outright — naming the other key — while a
poll finds a *live* job recorded under a *different* key: not because the
scratch space would collide (the keyed paths, below, already stop that),
but because two 1087-second gate runs on one machine is a cost nobody
asked for, and neither waiting for the other to finish nor restarting it
under the current key is `contest`'s to do — waiting would be the
same blocking call this section exists to remove, moved one level up, and
restarting would throw away a run that may itself be about to pass. Only
once that other job's process is confirmed dead does a fresh one start
under the current key. The two controls — the keyed scratch space and
this refusal — hold for different reasons and neither substitutes for the
other.

**The scratch space, and the result it reads.** The diff file and the
output root `keyed_mutants` hands `mutants::run_at` — `prepared_paths`'s
own pair — are derived from the key and the project's own build directory
(`<target>/lid-rs/gate/<key's hash>/…`) rather than resolved by it, and
independently of the refusal above: keying them is what stops a stale
job's cleanup step from deleting a fresh job's still-being-written output
even in the narrow window before either process's liveness has been asked
about. `<target>` is `project.target_directory()`'s own answer: a `JobKey`
carries the checkout's canonical path, never its build directory, which
`CARGO_TARGET_DIR` need not put anywhere near it (The key, and what it
must include, above), so `paths_for`, `prepared_paths`, and `result_path`
take `project` and resolve it exactly as `record_path` already does for
the running record and `tally::path` already does for the tally — four
keyed-path functions sharing one shape for the same reason. `paths_for`
answers the formula alone, touching no disk, so a validation can assert
it against a project fixture without a directory ever existing;
`prepared_paths` makes the key's own directory before answering the same
pair, which is what `keyed_mutants` calls: `write_diff_file`, the first of
`mutants::run_at`'s writes, plain-writes to the path it is given and fails
if the parent is missing, under `Scope::Diff` alone; `run_group` does make
its own output directory, and everything above it, through
`create_dir_all`, but only once its own turn comes, after `write_diff_file`
has already needed somewhere to land. Two files answer a poll, and they
are not the same file.

The **running** record is one per **checkout and slice**,
`<target>/lid-rs/gate/<checkout's canonicalised path, hashed>/<slice>.json`
— nested under the checkout, not named by slice alone — holding
`Running { key, pid }` for as long as a job is alive and nothing else; no
leaf of `poll` ever overwrites it with a finished outcome. Nesting by
checkout is not decoration: the key's own rationale for carrying the
checkout is that `CARGO_TARGET_DIR` need not be per-checkout and this
project runs phase agents in worktrees, and that is exactly the case a
record named by slice alone breaks. `lld/foo` and `lld/foo--change` are one
slice, `foo`; two worktrees on them, sharing a `target/`, would write
`Running { key, pid }` for two different checkouts to the one file
`<target>/lid-rs/gate/foo.json`, each overwriting the other's. Because a
poll only ever consults `result_path` once its own key matches what the
*current* record names, a checkout whose job finishes and passes can have
its own record clobbered by the other checkout before either ever reads it
back — its result sits at its own key's `result_path`, finished and
correct, and nothing asks for it, since the record that would have named
that key is gone. That is the bug nesting the record closes: **the record
is addressed by key** — the checkout, its one stable part — **so that a
result is readable by key independently of whichever record is current**.
Nested under the checkout, each worktree's poll reads and writes only its
own file, so this failure has nowhere left to happen.

The **result** the detached child writes, once, when the whole plan ends,
lives at the full key's own derived `result_path` — checkout, `HEAD`, base,
slice, and fingerprint together, not the checkout alone. It is the only
outcome file this design writes: `mutants::run_at` returns its verdict as
an ordinary `Result` from the call, under the same keyed output root
`paths_for` derives, and `run_job` is what turns that `Result` into the
`Done(...)` written here.
**The bound this design holds is per checkout, not per slice.** A slice
runs at most one job at a time *per checkout*, which the checkout-nested
record path is what makes true, not an assumption behind it. Concurrent,
differently keyed attempts *within one checkout* are the refusal above;
two checkouts of the same slice are simply two files, never
sharing a slot to contend over.

**Liveness must be fallible.** `gate_job::alive(pid) -> Result<bool,
String>` is fallible, not `-> bool`: a host that cannot ask — `kill` not on
`PATH`, a permission the sandbox denies — is a host this call cannot answer
for, and "dead" is exactly the answer that makes `gate_job::poll` restart
a 1087-second job. Answering `false` when the true answer is "cannot
tell" would restart it every poll, forever — the same failure this section
exists to remove, reintroduced at one call. So `alive`'s `Err` is
`gate_job::poll`'s own failure, in the words this document already uses
for `extra_step`: a step that could not run is not a step that passed.

**What survives the caller dying, and what does not.** The job is detached
from the calling process's lifetime by construction — an unwaited `Child`
— so it survives the phase agent's own process ending, the workflow's
session ending, and the stop hook's own single invocation returning: none
of them ever held it open to begin with. Only `target/` being removed — a
clean checkout, `cargo clean` — loses it, at the cost of the run in flight
and nothing else: the next poll finds no record, and no live process under
any key, and starts fresh.

**What a human running the gate by hand gets.** `cargo lid-rs phase-check
<n>` and `cargo lid-rs mutants`, run directly, keep calling `check` and
`mutants::run` in-process and blocking on them exactly as today — a
terminal has no 600-second watchdog, and the documented fallback (run the
whole gate in one sitting, commit by hand when the hook cannot) needs
nothing new to keep working. Neither ever passes `--job-key`, so `phase::run`
takes the unconditional branch that calls `check` and blocks; only
`gate_job::start`, from inside the stop hook's own Phase 7 arm, ever sets
that flag. `cargo lid-rs gate-status` is new, but it only reads; running it
starts nothing and blocks on nothing either.

**A passing gate is not evidence the detached path ran.** The gate is
`plan`'s own steps — `Check`, `Clippy`, `Doc`, `DocTests`, `LibTests`,
`Package`, `SyncCheck`, `Mutants`, and whatever `Extra` a workspace
configures — carried out in order by `execute`/`execute_with`/`run_step`.
`run_job`, `poll`, `act`, `settle`, `contest`, and the spawn `start` wraps
are none of them a step: `plan` never returns one for them, so no run of
`phase-check <n>`, whoever calls it and on whichever slice, ever reaches
them by running the gate. What exercises them is `cargo test --lib`
against this section's own fixtures,
and check 12's mutation of the source those fixtures cover, both driving
`run_job`'s dispatch and `poll`'s three-way read at the level of values
and fixtures, never a live child spawned and waited on for real. A
slice's own gate passing green is a claim about the code a detached job
would run, not a claim that a detached job, spawned and polled against a
real checkout, reaches the same result — that is tested only by a `hook
stop <n>` a phase agent actually ends, on a tree where `gate_job::start`
actually spawns a child and a later poll actually reads back what it
wrote.

**This section's own red set.** Wired into `gate_commit`'s Phase 7 arm at
Phase 3, the dispatch answers `Ok(GatePoll::Done(Ok(())))` unconditionally
— the wrong answer that compiles, the same shape `policy::gate_extra` and
`replaced_tip` are skeletoned with — so a Phase 7 stop proceeds straight to
`after_undo` exactly as it does today. Every validation that reaches
`hook_stop` for a claim outside this red set already expects a Phase 7
stop to either commit or refuse, never to end pending, so answering `Done`
keeps every one of those green, while every claim this section adds is
false against it: `key`'s widened signature, `fingerprint`'s walk,
`paths_for`'s and `result_path`'s derivation, `record_path`,
`child_command`'s own command line, `running_record` (building the
`Running` record from the key and the pid it is given, plus
`integrity::contents_of`'s answer for the bump's two files),
`write_record` (the one file write), `start` (bumping once, asking
`child_command` for that line, spawning it unwaited, then handing
`running_record`'s answer to `write_record`), `alive`, `read_record`,
`finished`, `act`, `settle`, `restart_or_running`, `contest`, `status`,
`matching_status`, `foreign_status`, `run_job`, and `bounded_output`
(carrying the failing block, not merely a header) keep the ordinary
`todo!()` beneath that answer, as the section's other new leaves do. The
pending ending is red the way `replaced_tip`'s claims were: a validation
drives a Phase 7 stop with a job
recorded `Running` under the fresh key and asserts the tip is left
standing, nothing committed, and no refusal returned — which
`Done(Ok(()))` cannot produce, since a skeleton answering the common case
is never asked to leave a tip alone. The differently keyed refusal is red
the ordinary way a leaf's `Err` always is: a fixture holding one record
under a foreign key, its pid alive, drives `poll`'s foreign branch to the
`todo!()` under it. The matching-key restart is red the same ordinary way:
a fixture holding one record under the current key, its pid dead and no
result written, drives `settle`'s call into `restart_or_running`'s own
`todo!()`, which the skeleton's blanket `Done(Ok(()))` never reaches.
`run_job`'s recomputation and its mismatch outcome are red the same
ordinary way, both against `run_job`'s own single `todo!()` rather than
against `poll`'s blanket answer, since neither is a call `poll` makes: a
fixture calls `run_job` with a hash equal to what `gate_job::key` computes
from the same `project`, `phase`, and `slice`, and asserts the mutation
step receives `prepared_paths`'s keyed paths and a `Done` lands at
`result_path` — an assertion the `todo!()` cannot satisfy any more than
any other leaf's does. The mismatch is red as its own case and not folded
into that one: a fixture calls `run_job` with a hash that disagrees, and
asserts that no step of `plan` runs and neither the given hash's own
directory nor the fresh key's holds a file afterward — an assertion the
`todo!()` also fails, by panicking before either path is ever touched, so
the claim is red for the same reason its sibling is and not by accident of
the panic alone. `child_command`'s own claim reddens the same ordinary
way, driven directly against its own `todo!()` and not through `poll`'s
blanket answer, since nothing on `poll`'s own path calls it: a fixture
calls it with a phase, a slice, and a key, and asserts the returned
command's argument list ends with `phase-check`, the phase's number,
`--slice`, the slice, and `--job-key`, the key's hash, appended to
whatever `self_command()` already carries — an assertion the `todo!()`
cannot satisfy any more than any other leaf's does.

`running_record`'s own claim reddens the same ordinary way, driven
directly against its own `todo!()` and not through `poll`'s blanket
answer, since nothing on `poll`'s own path calls it: a fixture calls it
with a key, a pid, and a project whose `Cargo.toml` and `Cargo.lock` are
known bytes, and asserts the returned record's key and pid equal what it
was given and its `version_files` equal `integrity::contents_of`'s own
answer for those same two files, in that call's own order — an assertion
the `todo!()` cannot satisfy any more than any other leaf's does, and one
no spawn is needed to drive.

`write_record`'s share of `AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord`
reddens independently of `start`'s own path through it: a fixture hands it
a record built by hand — the same shape a foreign-key fixture already
constructs elsewhere in this module, with nothing run to produce it — and
asserts `read_record` reads back exactly that record afterward, which the
`todo!()` beneath it cannot produce any more than any other leaf's can.

`ACommitBlockRunsThePhasesCheck` is reworded here — to
`ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll`, the retired
name kept beside it as a `#[deprecated]` alias, by the rule this document
already states for a reworded claim — because its old sentence says the
hook "shall run the phase's check," which stops being true the moment a
Phase 7 stop can instead start, or read the outcome of, a check some other
process ran. It belongs to this section's red set and not to "How to read
the claim notes" above, since the behaviour it now names is this section's
to redden, and Phase 5 rewrites its validation to drive three cases, not
two: a Phases-1–5 stop, unchanged (`checked` runs and blocks); a Phase 7
stop against the Phase 3 skeleton's own stub, `Ok(GatePoll::Done(Ok(())))`
— which is **green on arrival**, since a passing Phase 7 stop under that
stub commits exactly as it always did, and proves nothing this rewording
did not already have from today's `checked`; and a Phase 7 stop with the
poll stubbed to answer `Ok(GatePoll::Done(Err(output)))` for the fresh key
— which the stub cannot produce, since it is wired to the passing case
only — asserting the stop *refuses*, through `refusal_for`, rather than
committing. That third case is what makes the reworded claim red, the same
way the pending and foreign-key claims above are: named by the case the
skeleton gets wrong, not merely by the rename. Its validator,
`a_commit_block_runs_the_phases_check`, keeps the retired name because
the claim carries `#[lid(free)]`: check 14 holds a test to being named
for a claim it cites — one of several, when it cites several — and
exempts a validator's name from that only while every claim it cites is
marked free, so the name holds exactly as long as the mark does, and
renaming the validator is what the mark's own removal waits on.
`TheTallyIsWrittenAsTrailers` is the other path through the same rule,
not a precedent for keeping this one: its one validator is already named
for its claim, so no mark was ever needed to hold it.

**Recount.** "How to read the claim notes" names five reworded claims and,
with the rework rows, a sixth (`TheTallyIsWrittenAsTrailers`), whose red
set is fourteen; that count is still correct for the rework rows and does
not include either of this section's own. This section adds a
**seventh** rewording to the whole document — `ACommitBlockRunsThePhasesCheck`,
above — and an **eighth** — `TheRunningRecordCarriesTheBytesTheBumpWrote`,
renamed to `TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder`
(The record is a value too, built where nothing runs, above) — both
belonging to this section's own red set, not to the rework rows'
fourteen: the two counts are disjoint by scope, not in competition, and
neither needs the other's number changed. A **ninth** rewording belongs to
neither red set: `APhaseSevenSubjectMustCarryTheBumpedVersion`, renamed to
`APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll` (Why the
poll moves ahead of the bump, above; Where the subject-version check runs,
relative to the bump, Decisions), corrects a sentence that named the
comparison as running after the bump rather than before it — false, not
merely narrower, by the same test the eighth was judged against — but names
leaves, `subject_version_matches` and `version_the_bump_will_write`, that
predate this section and are already validated against the true order, so
nothing about it is new work for a Phase 3 skeleton to answer wrongly, and
its two validators, unchanged in body, are renamed alongside it rather
than kept — each cites this claim alone, and check 14 holds a test to
being named for a claim it cites, so nothing needs to redden for either
to land under the new name. A reader counting every rewording in this
document finds nine,
in two red sets that never share a claim, plus this one that belongs to no
red set at all.

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
  "lid-rs-phase-<n>"`) works; its stop hook commits, refuses, or — at
  Phase 7 alone, while `gate_job::poll` answers `Started` or `Running` —
  ends the turn pending, neither. It returns, as structured output,
  whether it committed, the commit if so, and the ≤3 numbered decisions
  the skill's stop contract requires; none of the three for a pending
  stop, since nothing was decided. The workflow never reads that field to
  learn whether Phase 7's gate has passed — a worker's word about the gate
  is exactly the channel this document keeps out of the verdict (The
  verdict stays out of an LLM's hands) — it calls `cargo lid-rs gate-status`
  for the slice instead, and resumes or respawns the Phase 7 worker to ask
  again, or to let its next stop read a settled result and commit or be
  refused by it. A reviewer agent reads the commit against the phase's
  checklist and the `discipline.md` rows tagged for that phase, prompted to
  refute, and returns approve-or-findings; a pending stop has no commit yet
  for a reviewer to read, so none runs until one exists. Findings go back
  to a worker, and the run carries
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
  no third state and no waiver argument: a pending Phase 7 stop is not a
  third outcome of the *run* — it ends no run and decides nothing — it is
  what the workflow polls through, cheaply, for as long as the
  orchestrating session keeps going, on its way to one of the two. Nothing
  about a poll in flight spends the six-rework budget, which is a
  reviewer's tool and pays only for rejections, never for a check still
  running; a session that ends before the job does leaves polling for
  whoever resumes it, and resuming just polls once more.

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
same phase agent (the Agent tool, `subagent_type: lid-rs-phase-<n>`). At
Phase 7 alone, a pending stop is not a commit to present: the session runs
`cargo lid-rs gate-status` for the slice, and while it answers `Running`
simply asks the same worker to stop again later, cheaply, rather than
presenting anything. Once a phase commits, the session presents that
commit — `git show`, the tally, the decisions — to the human, who says
"continue" or gives findings for a rework worker. The main session edits
no code in a LID project; the phase agent is the only thing that does, in
either mode. This is the cascade into the skill
(`docs/intent/skill/lld.md`): Phases 2–7 become "spawn the phase's agent,
review its commit", and the working-state section says what the tally
trailers are.

### What the commit body carries

Every phase commit ends with trailers the stop hook writes from the tally:

```text
Lid-Rs-Phase: 4
Lid-Rs-Agent: a580cd3d769f4ec4e, b12f0c94ee3a71d60
Lid-Rs-Tools: 13 edits, 26 observations, 0 commands
Lid-Rs-Checks: 13 post-edit, 3 stop
Lid-Rs-Refusals: 0 policy, 0 stop
Lid-Rs-Reworks: 1
```

`Lid-Rs-Agent` is the id, or the ids, the tally was kept under — the
subagent's id on Claude Code, the session's on the canopy client — so a commit
names the record of how it was made, and names every agent when a rejection
sent the phase back to a second one. Two or more ids are separated by `", "` —
a comma and a space, which is the one form written and the one form read back,
since `Replaced::of` splits the trailer on it to learn whose work the commit it
is replacing carries. `Lid-Rs-Reworks` is how many commits this one replaced,
and a commit that carries no such line replaced none: every phase commit made
before the rework change lacks it, and reading its absence as zero is what lets those
commits be reworked at all. The ratio of deterministic steps to agent-chosen ones is then in git for
every phase of every slice — `commands` is structurally zero and the tally
proves it — and refusals per phase is the quality signal: a phase whose
refusals rise is a phase whose skill file or policy is teaching the wrong
thing.

The phase in the example is 4 and not 6 because `Lid-Rs-Phase` is rendered
from the phase the hook is serving, and `Phase` is the closed set `One`–`Five`,
`Seven`: Phase 6 has no commit and no hook, so no commit can carry it.

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
- **The verdict is now a file.** `gate_job::result_path` lives under
  `target/`, which is git-ignored and therefore outside
  `outside_policy_clean`'s `git status --porcelain` filter — the integrity
  pass that catches every other write this section's boundary does not
  prevent. And the key that names it is computed from values the same
  process can read: the branch tip, the mutation base, the fingerprint. So
  source a Phase 5 or Phase 7 test writes, executed by the gate's own
  `cargo test`, could compute that key and write a passing `Done(Ok(()))`
  to `result_path` itself, and a poll that finds it would stage and commit
  — carrying the hook's genuine `Lid-Rs-*` trailers on a `phase 7:` commit
  whose gate never ran. This document does not close that. What bounds it
  is what already bounds every other case in this section: a phase agent's
  tools are `Read`, `Grep`, `Glob`, `LSP`, `Edit`, `Write` — no shell — so
  the path runs through source the agent writes being *executed later*, by
  the check itself, never through the agent forging a verdict directly.
  The exposure is not new in kind: the tally already holds it today — code
  that runs at Phase 5 or 7 can invoke `git` directly, exactly as it could
  write a fake outcome file — and Deferred 8's OS sandbox is the control
  that would close both at once. What is new is the consequence: before
  this section, a test's own writes could at worst corrupt a check that
  still had to report back through the very process the stop hook was
  blocking on; now a forged `result_path` is read by a later, separate
  process invocation, with no gate having run in between, and it is read
  as the verdict.

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
| `phase::run(args)` | `phase-check` entry: parses `<n>`, `--slice`, and an optional `--job-key <hash>`; absent, dispatches to `check` and blocks, exactly as before this section (the human fallback, below); present — set only by `gate_job::start`, never by a human or CI — dispatches to `gate_job::run_job` instead |
| `phase::hook(args)` | One `match` over the hook kind: `pre-tool <n>`, `post-edit <n>`, `stop <n>`; each reads Claude Code's JSON from stdin |
| `Phase` | Closed set `One`–`Five`, `Seven`; `TryFrom<u8>` refuses 0, 6, 8+ |
| `Step` | Closed set: the gate's steps, the LLD checks, the red run, and one `Extra` per configured `gate_extra` entry |
| `check(project, phase, slice) -> Result<(), String>` | One phase's check: the plan, built as `plan(phase, publishing, policy::gate_extra(project)?)`, executed in order. The one place `gate_extra` is read, so a value the tool cannot read fails here — at whichever phase is running, not only at phase 7 |
| `plan(phase, publishing: &[String], extra: &[Vec<String>]) -> Vec<Step>` | A phase's steps as data; `publishing` names what one `cargo package` runs for and `extra` the workspace's own steps, which follow the floor in the order configured |
| `execute`, `execute_with`, `run_step` | Runs steps in order; the first failure is the result |
| `args_of(step: &Step) -> Vec<String>` | One step's cargo arguments as data — `--locked` on every one, `--document-private-items` on the doc step, `package`'s member list and `--allow-dirty` — so what a step invokes is assertable without invoking it; a step that invokes no cargo, an extra step included, answers with nothing |
| `cargo_step(project, args, env)` | One cargo invocation of `args_of`'s list, its output captured into the failure |
| `Step::Mutants` | The one step whose argument is not data. Never itself the thing a poll waits on — the *whole* plan is what runs detached (The gate runs detached, below), not this step alone. `run_step`'s own arm is unchanged by detachment: it always calls `mutants::run(&["--diff-base", mutation_base(project)?])` with the fixed default paths and blocks — a human's or CI's own bare `phase-check 7` reaches only this arm. The keyed paths belong to `gate_job::run_job` alone (below), which diverts this one step to `mutants::run_at` with `gate_job::prepared_paths(project, key)?`'s paths instead of calling `run_step` for it |
| `Step::Extra(Vec<String>)` | One entry of the workspace's `gate_extra`: the program and its arguments, carried as data on the variant. `args_of` answers nothing for it, so no rule about a cargo invocation is asserted of a step that invokes no cargo |
| `extra_step(project, entry: &[String]) -> Result<(), String>` | One extra entry run: the entry's first word as a program, the rest as its arguments, at the workspace root and through no shell; a non-zero exit or a program that cannot be run is the failure, naming the entry and carrying its output. A malformed entry never reaches it: `policy::gate_extra` fails the check naming `gate_extra` and the entry it could not read before any step runs |
| `mutation_base(project) -> Result<String, String>` | The gate's diff base: one decision between the gate commit and the merge base |
| `merge_base_with_main(project) -> Result<String, String>` | `git merge-base main HEAD` — where the branch was cut from; a failure names the ref when there is no common ancestor |
| `gate_job::JobKey { checkout, head, base, slice, fingerprint }`, `gate_job::key(project, phase, slice, crates) -> Result<JobKey, String>` | What a poll must match before trusting any record: the checkout's canonical path, `tip`'s hash, `mutation_base`'s answer, the slice's name, and `fingerprint`'s answer over `policy::workspace_paths(project, phase, crates)` — which is why `key` takes `phase` and `crates` rather than resolving the editing set itself. A mismatch on any one field is the same as no record |
| `gate_job::fingerprint(project, admitted: &[PathBuf]) -> Result<String, String>` | Walks each path `policy::workspace_paths` admits — recursively for a directory, once for a file — and hashes every regular file's relative path and bytes, sorted and concatenated: decided in Decisions, not deferred, because a walk (unlike `git stash create` or a tracked-changes diff) includes a leaf the phase just wrote and git does not yet track |
| `gate_job::paths_for(project, key: &JobKey) -> Result<(PathBuf, PathBuf), String>` | The formula for the keyed scratch pair — the diff file first, the output root second — at `<target>/lid-rs/gate/<key's hash>/…` — derived, not stored, so a record and the job it names always agree. `<target>` is `project.target_directory()?`, not a fixed root: a `JobKey` carries the checkout's canonical path, not its build directory, which `CARGO_TARGET_DIR` can put anywhere, so the path is resolved through `project` exactly as `tally::path` already resolves the tally's own directory, and fails the same way that lookup can. It touches no disk, so a validation can assert the formula against a project fixture with no directory ever made; `prepared_paths` is what a detached job actually calls |
| `gate_job::prepared_paths(project, key: &JobKey) -> Result<(PathBuf, PathBuf), String>` | What `keyed_mutants` calls in place of `paths_for`: makes the key's own directory before answering the same pair `paths_for` derives, so the diff file and the output root already have somewhere to be written the moment `mutants::run_at` receives them. Needed because `mutants::run_at`'s own `write_diff_file` writes through the path it is given with a plain `std::fs::write` and fails if the parent is missing — under `Scope::Diff` alone, since `Scope::Full` returns `None` and writes nothing — while `run_group` makes its own output directory, and everything above it, through `create_dir_all`, but only once the engine's own turn comes; the diff file is the write with nowhere else to get a home made for it first. `paths_for` itself stays pure and unwritten to |
| `gate_job::result_path(project, key: &JobKey) -> Result<PathBuf, String>` | The keyed file the detached child writes once, on its own, when the *whole plan* finishes — `<target>/lid-rs/gate/<key's hash>/result.json`, holding `Done(Result<(), String>)`, following `tally::to_json`/`tally::from_json`'s written/read precedent. Distinct from the mutation engine's own per-group `outcomes.json` evidence under `paths_for`'s output root, which no poll of this file reads. `<target>` is `project.target_directory()?`, the same door `record_path` already resolves the running record's directory through and `tally::path` resolves the tally's — a `JobKey` names the checkout, but not the build directory a shared `CARGO_TARGET_DIR` can put anywhere |
| `gate_job::JobRecord::Running { key: JobKey, pid: u32, version_files: Vec<(PathBuf, Vec<u8>)> }`, `gate_job::record_path(project, slice) -> Result<PathBuf, String>` | The one record a *checkout* holds for a slice, at `<target>/lid-rs/gate/<checkout's canonicalised path, hashed>/<slice>.json` — nested by checkout, the way the tally's own `<target>/lid-rs/agents/<agent_id>.json` is nested by agent, so two worktrees on one slice hold two files and never clobber each other's `Running { key, pid, version_files }`. It only ever names a job as *running* — no leaf of `poll` writes a finished outcome back into it, since the finished outcome is `result_path`'s, not this file's. `version_files` is `integrity::contents_of`'s answer for `Cargo.toml` and `Cargo.lock`, built into the record by `running_record` from the key and the pid it is given, and carried on the record `write_record` writes; `bumped_files_untouched` reads it back from there at whichever later stop finds `Done`, rather than recomputing it — the one recompute `Cargo.lock` cannot support |
| `gate_job::read_record(project, slice) -> Result<Option<JobRecord>, String>` | One file read: the current running record, or none |
| `gate_job::child_command(phase, slice, key: &JobKey) -> Result<std::process::Command, String>` | The detached child's own command line, built and not run: `phase-check <n> --slice <slice> --job-key <key's hash>` — `n` `policy::number_of(phase)`'s answer — appended to whatever `self_command()` already carries, its own program and any leading arguments left unexamined. A fact about a string, holding no process and needing none to check it — the half of a detached child a caller can be held to |
| `gate_job::running_record(project, key: &JobKey, pid: u32) -> Result<JobRecord, String>` | The `Running` record as a value, built from exactly what it is given: the key and the pid unchanged, and `integrity::contents_of(project, &policy::hook_written_paths(Phase::Seven))`'s answer as `version_files`, in that call's own order. Assertable with nothing running — no child, no spawn — the same shape `child_command`'s own line already is |
| `gate_job::write_record(project, slice, record: &JobRecord) -> Result<(), String>` | The other half of a job's beginning: one file write, of the record it is handed, to `record_path(project, slice)?`. Takes the record rather than building one, so `start` — or a fixture — can hand it any record, `running_record`'s included |
| `gate_job::start(project, phase, slice, key: &JobKey) -> Result<(), String>` | The one place a job begins: bumps the workspace version and its lock (`bump_workspace_version`) — once, since no other leaf calls `start` for a job already running or already read as `Done` — then asks `gate_job::child_command` for the line this job's child runs and spawns it, unwaited, through `Command::spawn`, so the child outlives this process under ordinary Unix reparenting; then asks `running_record` for the record this key and this child's pid make, and hands it to `write_record` before returning |
| `gate_job::alive(pid: u32) -> Result<bool, String>` | `kill -0 <pid>` through a bare `Command` — no new dependency, the posture `extra_step` already keeps; an `Err` when the host cannot answer, never folded into "dead" |
| `gate_job::finished(project, key: &JobKey) -> Result<Option<Result<(), String>>, String>` | One file read: the outcome at this key's `result_path(project, key)?`, or none when the child has not written one yet |
| `gate_job::run_job(project, phase, slice: &str, job_key: &str) -> Result<(), String>` | What `phase::run` calls instead of `check` when `--job-key` is given: takes the hash `--job-key` carried, not a `JobKey` — nothing reconstructs one from a one-way digest — and recomputes the key itself, `crates` resolved the same way any other invocation resolves them and handed with `project`, `phase`, `slice` to `gate_job::key`; a hash that disagrees with `job_key` runs no step of `plan` and writes to no result path at all, leaving the running record's pid to answer dead so the next poll restarts under whatever key the tree computes then (The detached child's own command line, above). A match builds the same steps `plan` returns and runs them through its own runner passed to `execute_with` — every step but `Step::Mutants` delegated straight to `run_step`, unchanged, `Step::Mutants` alone diverted to call `mutants::run_at` with `gate_job::prepared_paths(project, key)?`'s diff and output-root paths — then writes its one outcome — `Done(Ok(()))` or `Done(Err(output))` — to `gate_job::result_path(project, key)?` once, before exiting. `run_step` and `execute_with` are untouched: `execute_with` already takes a runner as a closure, so the one place that needs the key supplies its own rather than threading a new parameter through either. The writer this cascade needs (The one piece this cascades into another slice's document, above) |
| `GatePoll::{Started, Running, Done(Result<(), String>)}` | The whole of what `gate_job::poll` can mean when it does not fail outright: a job just spawned, one already alive under this key, or a finished outcome read back from the record. The fourth thing a poll can answer — a live job under a *different* key — is not a `GatePoll` at all; it is the `Err` `act` — the one leaf in `poll`'s pipeline that actually dispatches — returns through `contest` |
| `gate_job::poll(project, phase, slice, crates) -> Result<GatePoll, String>` | Work, not decision: computes the fresh key and reads the running record, then hands both to `gate_job::act`. Never sleeps; a caller wanting a later answer calls again |
| `gate_job::act(project, phase, slice, key: &JobKey, record: Option<JobRecord>) -> Result<GatePoll, String>` | One decision, three-way, over what `poll` already read: no record starts a fresh job and answers `Started`; a record whose key matches goes to `gate_job::settle`; a record under a different key goes to `gate_job::contest` |
| `gate_job::settle(project, phase, slice, key: &JobKey, pid: u32) -> Result<GatePoll, String>` | One decision, for a record under *this* key: `finished(project, key)?` first — a written outcome answers `Done` before anything asks whether the pid is still alive — and only when none is written does `gate_job::restart_or_running` decide from liveness |
| `gate_job::restart_or_running(project, phase, slice, key: &JobKey, pid: u32) -> Result<GatePoll, String>` | One decision, reached only once `settle` finds no result written: an alive pid answers `Running`; a dead one — the attempt died before finishing — starts a fresh job under the *same* key and answers `Started` |
| `gate_job::contest(project, phase, slice, key: &JobKey, pid: u32, other: &JobKey) -> Result<GatePoll, String>` | One decision, for a record under a *different* key: an alive pid is refused outright, naming the other key; a dead one is stale and starts a fresh job under *this* key, answering `Started` |
| `GateStatus::{NoJob, Running, Foreign(JobKey), Done(Result<(), String>)}` | What a read-only status can answer. Not simply `GatePoll` minus `Started`: a live job recorded under a *different* key is something a poll refuses outright, but a status read must not fail over it — there is something to wait for, just not under this key — so `Foreign(JobKey)` names that case rather than folding it into `NoJob`, which would tell a caller there is nothing running when there is |
| `gate_job::status(project, phase, slice, crates) -> Result<GateStatus, String>` | The read-only sibling of `poll`: computes the fresh key and reads the record, the same three-way split `act` makes (no record, a matching key, a foreign key), but never reaches `start` because the classification is factored out of the starting rather than threaded through it — `matching_status` and `foreign_status` (below) make the same finished/alive reads `settle`/`restart_or_running` and `contest` make, mapped to `GateStatus` instead of to a spawn. Spawns nothing, writes nothing: the deterministic read surface an orchestrating session consults instead of a worker's own report of what happened (Who checks back, and how, below) |
| `gate_job::matching_status(project, key: &JobKey, pid: u32) -> Result<GateStatus, String>` | The status-only sibling of `settle`/`restart_or_running`, for a record under *this* key: `finished(project, key)?` first — `Some(r)` answers `Done(r)` — and only when none is written does `alive(pid)` decide: alive answers `Running`; dead answers `NoJob`, since nothing is actually running and a poll would restart here rather than find anything to report. It takes `project` for the same reason `finished` does: what it reads lives under the build directory, which a bare key does not carry |
| `gate_job::foreign_status(other: &JobKey, pid: u32) -> Result<GateStatus, String>` | The status-only sibling of `contest`, for a record under a *different* key: `alive(pid)` decides — alive answers `Foreign(other)`, since there is something to wait for even though a fresh start of this key would be refused over it; dead answers `NoJob`, since the record is stale and a poll would start fresh under the current key |
| `cargo lid-rs gate-status [--slice <name>]` | The CLI door onto `gate_job::status` (always asked of Phase 7, the only phase with jobs): `status_flag` reads the optional `--slice <name>`, `gate_status` resolves the slice and asks `gate_job::status`, and `report_status` (below) turns the answer into what the process prints and exits with; never spawns a job and never blocks |
| `report_status(status: &GateStatus) -> Result<(), String>` | Work, not decision, over what `gate_status` already read: `status_line(status)?`'s line handed to `announce`, so its own `Err` is exactly `status_line`'s — a finished failure exits non-zero without this call inspecting which variant produced it |
| `status_line(status: &GateStatus) -> Result<String, String>` | One decision, five-way, over what a `GateStatus` prints and whether it fails: `NoJob` answers `no job`, `Running` answers `running`, `Foreign(other)` answers `blocked by another attempt (<other>)`, `Done(Ok(()))` answers `done: the gate passed`, and `Done(Err(output))` answers this call's own `Err`, carrying `output` unchanged — pure, so a validation calls it directly and asserts the string or the error, with no process to run and no stdout to capture |
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
| `bump_workspace_version(project) -> Result<String, String>` | Phase 7's one bump: the manifest at `HEAD` through `bump_patch_version`, written to the working tree, then `cargo update --workspace --offline`; the new version. Run once, by `gate_job::start`, immediately before the detached child is spawned — never by `after_undo`, and never again while that job is running or once it is read `Done` — so the child gates the bumped tree and the bump itself never runs once per poll |
| `bump_patch_version(manifest: &str) -> Result<String, String>` | The manifest text with its `[workspace.package]` version line raised one patch level: `version_line` locates, `next_patch` raises, the line is spliced back |
| `version_line(manifest: &str) -> Result<(usize, String), String>` | The first line beginning `version = "` between `[workspace.package]` and the next line beginning `[`, and its quoted value; a failure names what it looked for |
| `next_patch(version: &str) -> Result<String, String>` | Three dot-separated numbers with the last raised by one; anything else is a failure naming the value |
| `subject_version(subject: &str) -> Option<String>` | The `<version>` field of a `phase 7: <version>: <what and why>` subject, which the hook holds to the version the bump will write — computed from the manifest at `HEAD`, not run — comparing before the poll and so before the bump |
| `ending::subject_carries_version(message, version) -> Result<(), String>` | That comparison as a refusal: a Phase 7 subject naming another version than the bump's, or none, is refused naming both — a rule separate from the tag's |
| `integrity::contents_of(project, paths) -> Result<Vec<(PathBuf, Vec<u8>)>, String>` | The bump's two root files read back right after it, each with its bytes — what `bumped_files_untouched` holds the tree to; the bump returns the version, not the bytes, and `Cargo.lock` cannot be recomputed at check time without rewriting it. `gate_job::running_record` is the caller, asked by `start` once the bump has run, and its answer is what the running record carries as `version_files` for a later stop to read back |
| `integrity::bumped_files_untouched(project, version_files)` | The other half of the post-check integrity pass: each of the two root files must still equal, byte for byte, what the bump wrote. Takes bytes, not a recipe for producing them: at Phases 1 to 5, where nothing bumps, `version_files` is empty and there is nothing to hold the tree to; at Phase 7 it is the running record's own carried answer, read back rather than recomputed |
| `policy::slice_crate(project, slice) -> Result<PathBuf, String>` | The package holding the slice's document, in either layout — asked of `layout::own_crate`, which is where the four shapes are told apart |
| `policy::companion(project, crate_root) -> Result<Option<PathBuf>, Refusal>` | The package `[package.metadata.lid_rs] companion` names, from `cargo metadata`; none for an ordinary crate; a refusal naming the key for a proc-macro crate without one, or one whose companion is a proc-macro crate or not a member |
| `policy::gate_extra(project) -> Result<Vec<Vec<String>>, String>` | The workspace's configured steps, parsed from `Project::setting_node("gate_extra")` — the same two-sided shape `companion` has over `package_setting_at`. An absent key is the empty list; a value that is not a list fails the check naming `gate_extra` and what it found; an entry that is not a non-empty list of strings fails the check naming `gate_extra` and the entry it could not read. The four metadata claims — the reading with its fallback, the absent key, the non-list value, the malformed entry — are cited here and not on the raw door; Phase 3 wires it answering the empty list, and Phase 5 reddens the other three through `extra_entry` and the non-list arm (the Decisions row "How `policy::gate_extra` is skeletoned") |
| `policy::extra_entry(value: &serde_json::Value) -> Result<Vec<String>, String>` | One entry read, `gate_extra`'s descent: a list of one or more strings is the entry's words; anything else — not a list, an empty list, a non-string among the words — fails naming `gate_extra` and the entry as its JSON, never the whole configured value. The first entry that cannot be read is the failure, since the entries are collected in order |
| `Project::package_setting_at(dir, key)`, `Project::member_dir_named(name)` | What `companion` reads: a package's `[package.metadata.lid_rs]` setting, and a member's manifest directory by package name — both from the metadata document `Project` already holds, in `src/project.rs`, which this slice's phases may not write and the human adds by hand |
| `Project::setting_node(key) -> Option<serde_json::Value>` | One `metadata.lid_rs` key's raw JSON node from the metadata document `Project` already holds: the `[workspace.metadata.lid_rs]` table's, falling back to that of the package whose manifest is the workspace root's, as `configured_scope` reads `mutation_scope` through `setting_in`. It implements no claim — a raw node has no wrong answer — and it lives in `src/project.rs`, which this slice's phases may not write and the human adds by hand between Phases 2 and 3 |
| `SliceCrates { slice, own, companion }`, `SliceCrates::resolve`, `claims_crate()` | The crates a phase may write, resolved once per hook call; the crate that holds the slice's claims, where the red run diffs and tests |
| `Tally`, `tally::record(project, agent_id, event)`, `tally::trailers(tally, phase, agents, reworks)` | Counts per agent under `<target>/lid-rs/agents/`; rendered as commit trailers, `Lid-Rs-Agent` naming every agent whose work the commit carries and `Lid-Rs-Reworks` the number `Replaced::next_reworks` answers. The renderer is handed the agents and the number, so neither the agent list's order nor the reworks' arithmetic is decided here |
| `Tip { hash, subject, body }`, `tip(project) -> Result<Option<Tip>, String>` | The branch tip as it reads, from one `git log -1`: the hash, the subject line, and the body the trailers are in; none in a repository with no commit. The one read of the commit, so the decision below re-reads none of it — `on_the_trunk` asks git one further question, about the hash and not about the commit's content |
| `Replaced { commit, agents, tally, reworks }`, `Replaced::of(tip) -> Result<Replaced, String>` | What a stop replacing the branch tip needs from it: the `Tip` itself — the hash to parent the restore on and the subject and body to re-commit, since the commit is read once and `restore_tip` is handed nothing else — the agents its `Lid-Rs-Agent` names split on `", "`, the counts its trailers carry, and how many commits it already replaced — built from one `Tip` through `trailer_of` and `tally::from_trailers`, whose failure is this one's. An absent `Lid-Rs-Reworks` is zero, because every commit made before the rework change carries none; an absent `Lid-Rs-Agent` or count line is not, and fails naming the line, since a commit the hook wrote carried the five lines of its day |
| `Replaced::next_reworks(replaced: Option<&Replaced>) -> u32` | The `Lid-Rs-Reworks` a commit carries: zero when it replaces none, one more than the replaced commit's value when it does. The trailer's whole arithmetic in one place, so `tally::trailers` is handed the number rather than the record |
| `replaced_tip(project, phase) -> Result<Option<Replaced>, String>` | The decision alone, over `tag_of(subject)`, `trailer_of(body, "Lid-Rs-Phase")` and `on_the_trunk(hash)`: replaceable when the tag names this phase, the trailer names this phase, and the commit is not reachable from `main`; none otherwise, and the stop then commits on top of it. The slice is the branch's, so no subject field is parsed for it |
| `trailer_of(body, key) -> Option<String>` | One trailer's value in a commit body — the last line beginning `<key>: `, trimmed — none for a body that carries none |
| `on_the_trunk(project, commit) -> Result<bool, String>` | Whether a commit is reachable from `main` — `git merge-base --is-ancestor <commit> main`, true on exit 0 and false on anything else — which a repository with no `main` answers with false, since a ref that does not exist reaches nothing |
| `undo_tip(project, replaced: Option<&Replaced>) -> Result<(), String>` | Nothing when nothing is replaceable; otherwise `git reset --soft HEAD~1`: the tip stops being a commit and every change it carried stays in the index, so the replacement carries both rounds and every reading after it sees the commit below the one replaced |
| `restore_tip(project, replaced: Option<&Replaced>) -> Result<(), String>` | Nothing when nothing was replaced; otherwise the undo's other half, run when any failure after the undo turns the stop into a refusal: `git commit-tree <hash>^{tree} -p <hash>^ -F <message>` with the message the `Tip` already holds, then `git update-ref HEAD <new>`. It reads the replaced commit's own tree object, so it touches neither the index nor the working tree and the attempt comes back byte for byte — same tree, same message, new hash. A failure here is the refusal's own failure |
| `after_undo(project, phase, input, message, plan) -> Result<Vec<PathBuf>, String>` | Everything the stop does once the tip is undone and, for Phase 7, once `gate_commit` has already read a `Done` poll: sync, the check (`hook_writes` then `checked`, for Phases 1–5 — Phase 7's arm calls neither, since `gate_job::start` already bumped and the detached job already ran), integrity (sync again, staged set unchanged, and — Phase 7 only — the bump's files still equal what `gate_job::start` wrote, checked against the bytes the running record carries, read back by `running_record` once the bump has run rather than recomputed — `Cargo.lock`'s bytes cannot be recomputed at check time without rewriting it), and the staged set it answers with. `gate_commit` hands its `Err` to `restore_tip` and returns it unchanged; its own answer stays two-valued — staged or failed — because the call that can produce the third ending never happens inside it. The Phase 7 subject-version check moves out of this item entirely, to `gate_commit`, ahead of the poll (Why the poll moves ahead of the bump, above) |
| `GateCommit::{Committed(String), Pending(gate_job::JobKey)}` | The third ending's home: not `after_undo`'s, since the poll that can produce `Pending` is asked before `after_undo` is ever called for Phase 7, and `after_undo` keeps the two answers it always had |
| `gate_commit(project, phase, input, message, plan) -> Result<GateCommit, String>` | Everything that must hold before a phase commits, in order, and then the commit — as it always has for Phases 1–5, straight to `after_undo`. At Phase 7, once the tip is undone: the subject carries the version `bump_patch_version(&manifest_at_head(project)?)` computes (a pure check, asked before the poll on every stop, whether this is the one that starts the job or the tenth that finds it running), then the undone tip is polled: `Done` goes on to `after_undo` and `commit_now` exactly as every other phase's success path, mapped to `GateCommit::Committed`; `Started`/`Running` restores the tip — exactly as a refusal does — and returns `GateCommit::Pending` directly; any other poll failure, a failed subject check, or a failure from `after_undo`, restores the tip and returns `Err`, unchanged from today |
| `commit_phase(project, phase, input, message) -> Result<HookVerdict, String>` | `gate_commit`'s three outcomes become two verdicts, not three: `Committed` and `Pending` both map to `HookVerdict::Allow` — the stop is allowed either way, and `HookVerdict` gains no variant for the difference; `Err` refuses, tallied and worded as always. The tally is what keeps them apart: `Pending` is never recorded as `Event::StopCheck` and never as a refusal, so a `Committed` stop and a `Pending` one leave different marks there though the wire verdict they render is the same value |
| `HookVerdict::{Allow, Refuse(String), Context(String)}` | Unchanged by this section: the three sites this slice's own phases may not edit — `headless_canopy_agent/tools.rs`'s pre-tool and post-edit dispatch, `headless_canopy_agent/ending.rs`'s stop dispatch — match it exhaustively today with no wildcard arm (`wildcard_enum_match_arm` is denied), so a fourth variant fails `cargo check --all-targets` inside modules this phase's own policy refuses to let a worker touch. The wire is blind to `Committed` versus `Pending`; `GateCommit` (below), the tally, and `cargo lid-rs gate-status` are the channel that carries the distinction instead |
| `tally::from_trailers(body) -> Result<Tally, String>` | The counts a commit's trailer block carries, read back from the lines `tally::trailers` wrote; a line that renderer could not have written is a failure naming it |
| `tally::merged(replaced: Option<&Replaced>, this: &Tally, agent: &str) -> Tally` | One decision: this agent's tally alone when nothing was replaced, or when the replaced commit's agents already name `agent` — that tally counts both rounds — otherwise the replaced commit's counts added to this agent's. The id is a parameter because the decision is about *which* agent, and the counts do not carry one; `tally::agents` takes it for the same reason. The `Option` is read here and not in `commit_now`, which holds no decision |
| `tally::agents(replaced: Option<&Replaced>, this: &str) -> Vec<String>` | The agents whose work the commit carries, in the order they worked, none of them twice: this agent alone when nothing was replaced, otherwise the replaced commit's agents with this one appended if it is not already among them |
| `hook_pre_tool(project, phase, input)` | Policy verdict for editing tools, tally for every tool |
| `hook_post_edit(project, input)` | Clippy, rendered as `additionalContext` |
| `hook_stop(project, phase, input) -> Result<HookVerdict, String>` | Parse the message; `commit` → `commit_phase` → `gate_commit`: subject tag → nothing changed in the editing set → undo a replaceable tip → for Phases 1–5, straight to `after_undo` (sync → `hook_writes` (no-op) → `checked` → integrity → staged set) → stage → commit → allow; for Phase 7, the subject-version check (pure, against the manifest at `HEAD`) then `gate_job::poll` — `Done(Ok(()))` folds into `after_undo`'s own path (its bump already `gate_job::start`'s and its check already the detached job's) → stage → commit → allow, `Done(Err(output))` refuses through `refusal_for`, handed `gate_job::bounded_output(project, key, output)?` in place of `output` itself (A `Done(Err(..))` must not carry what the file already holds, above), `Started`/`Running` restores the tip and allows the stop `pending` (The gate runs detached) with integrity, staging, and commit all skipped — and any other failure after the undo → restore the tip → refuse; `stop` → allow; else refuse |
| `integrity::synced_artifacts_match(project)` | `sync::check`, as a refusal reason |
| `integrity::outside_policy_clean(project, phase, crates)` | `git status --porcelain` filtered against `staged_paths`; anything else is named |
| `ExecutionClass::{Ordinary, CompileTime(reason)}`, `execution_class(project, crate_root)` | From `cargo metadata` target kinds: `proc-macro`, `custom-build` |
| `compile_time_accepted(project, slice)` | Whether the slice's `compile-time-accepted` intent file exists — `layout::intent_file`'s answer |
| `Ending::{Commit(message), Stop(decisions)}`, `ending_of(message)` | The stop protocol, parsed from the final message |
| `refusal_for(project, phase, crates, output) -> String` | Output + `gates.md` row for the check that fired + what the phase permits |
| `gate_job::bounded_output(project, key: &JobKey, output: &str) -> Result<String, String>` | The header `execute_with`'s own `format!("{step:?} failed: {e}")` writes — up to and including the literal `" failed: "` marker, matched at its first occurrence, which is always the header's own — followed by the **last 8 KiB of `{e}`** and a closing line naming `result_path(project, key)?`'s own path for the rest. The tail, not the head, because every substring `check_of_output` looks for (`clippy::…`, `"is not red"`, `"survived"`) is emitted where the failing tool reports its own verdict — the last thing it writes. What `hook_stop`'s Phase 7 arm hands `refusal_for` in place of a `Done(Err(..))`'s raw capture, so `check_of_output`, run on this same bounded string inside `refusal_for`, still names the check that fired, and a pending poll's own refusal never reproduces what the keyed file already holds. It takes `project` and answers a `Result` for the one reason: the closing line names `result_path`'s own path, and that resolution is fallible now that it reads the build directory through `project` |
| `check_of_lint(name) -> Option<Check>` | The lint → check mapping |
| `stage_and_commit(project, paths, message, trailers)` | `git add -- <paths>`; `git commit -F` |
| [`sync::artifacts()`](crate::sync::artifacts) | The mirror table: `skill/`, `workflow/`, `agent/` |
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
| Shared leaves on a Phase 8 edit | A leaf that also implements a claim outside the red set keeps its body; Phase 3 wipes only leaves whose every claim is in the red set; Phase 5 makes each red-set claim red by validating the delta | Wipe every implementer of a red-set claim; commit every such Phase 5 by hand | Wiping a shared leaf breaks green validations of claims the edit never touched, so Phase 3 could not commit. A reword's delta is observable by construction — it is why the claim was reworded — so a validation of it can be red without un-implementing anything. Hand commits stay for subtractions, whose delta is an absence. The same rule reaches validations whose *claim* did not change and whose behaviour moved under them: the rework change reorders the stop, and `phase_sevens_stop_bumps_the_patch_version_from_the_manifest_at_head` (`PhaseSevensStopBumpsThePatchVersionFromTheManifestAtHead`) and `nothing_changed_in_the_editing_set_is_a_refusal_read_from_the_changes_within_it` in `integrity` (`NothingChangedInTheEditingSetIsARefusal`) both drive a Phase 7 stop with nothing in the editing set and expect the bump to have run, which the nothing-to-commit test now forestalls. Each must make an edit the editing set admits before the stop, and the second must also assert the tip it did not replace is still there. Neither claim is in the red set, so Phase 5 rewrites them as green work, exactly as the companion-seat row prescribes.

The subject-check reorder in this section breaks the same two tests a **second** time, for a different reason, and both still encode the old order in their own comments and assertions: `phase_sevens_stop_bumps_the_patch_version_from_the_manifest_at_head` drives a stop with a *wrong-version* subject and asserts the bump ran anyway ("a subject naming the wrong version is refused after the bump and before the check"); `nothing_changed_in_the_editing_set_is_a_refusal_read_from_the_changes_within_it`'s second half drives a wrong-version stop with an edit present and asserts the staged set carries the bump's two files beside it. Under this section's order the subject check runs before the poll, so before anything is bumped — a wrong-version stop now refuses with the tree exactly as it started, and neither assertion holds. Both are green work again, not red-set work: `phase_sevens_stop_bumps_the_patch_version_from_the_manifest_at_head` drives its bump-observing stop with a subject naming the *correct* bumped version, and moves the wrong-version case to its own assertion that the tree is untouched; `nothing_changed_in_the_editing_set_is_a_refusal_read_from_the_changes_within_it`'s second half does the same. Neither test can still reach the bump by routing a *correct*-subject, fresh-key stop through `hook_stop`, though: once Phase 7's leaves are real, that call reaches `gate_job::start`, which spawns an actual detached child (`Command::spawn`) — a real subprocess, even against a scratch fixture — which a fast unit test should not depend on finishing. So Phase 5 observes the bump the way the first test already does for one of its own assertions: by calling `bump_workspace_version` directly, never through `hook_stop`. The reordered *refusal* needs no fixture-level stop at all either — `ending::subject_carries_version` is a pure leaf, taking no key and spawning nothing, and is what Phase 5 drives instead to show a wrong-version subject is refused before the bump has run. This is a stated limit of testing through `hook_stop` at Phase 7 from here on: a correct-subject, fresh-key stop is no longer a cheap thing to drive end to end in a unit test, and Phase 5's validations route around it rather than pretend otherwise. |
| Compile-time slices | Disclosed from `cargo metadata`; edits refused unless `docs/intent/<slice>/compile-time-accepted` exists, a file only the human's Phase 1 commit can add | Refuse them outright; treat them like any slice; a workflow argument (`args.compile_time`) | The tool's own `lid-rs-macros` is such a crate and must be workable; the human, not the workflow, decides to run compile-time code unattended. A workflow argument reaches the hook only through a model's prompt, which is exactly the channel the policy must not trust; a file in a path no agent can write is a decision the hook can verify. |
| The stop protocol | Fenced ```` ```commit ```` or ```` ```stop ```` in the final message | Structured output only; a marker line; the hook reading the transcript | `last_assistant_message` is what the hook receives; a fenced block is unambiguous to parse and to write, and the refusal teaches the format when it is missing. Whether the final message survives a workflow `schema` is verified at Phase 3 of this slice; if not, the workflow's worker returns plain text and the script parses it. |
| The workflow's structured answer | `StructuredOutput` is an observation | A fourth tool kind; a command, with the workflow parsing the worker's final message instead of a `schema` | The call reads and writes nothing, and it arrives after the stop hook has already judged the commit block: refusing it there ends the run with the phase committed and the workflow reporting a failure. A tool kind of its own would count something the tally has no question about. |
| The red run on a Phase 8 edit | Scoped to the claims added since the newest `phase 7:` commit, by name in the spec file's diff | Every claim of the slice (the first design); an explicit `--claims` list; the claims the Phase 2 commit's diff touched at all; a registry dump of the base commit | Every claim of the slice can only be red by un-implementing the slice, so an implemented slice's Phase 5 could never pass the hook. A `--claims` list is an argument that reaches the hook through a model's prompt. Any changed line of the Phase 2 diff would sweep in a claim whose doc comment merely mentions another. A base registry dump means building the base commit for every red run. The gate commit is the one moment the slice is known whole, and a renamed struct is exactly one added `struct <Name>` line. |
| A reworded claim under the policy | The renamed struct plus a `#[deprecated]` type alias for the old name beside it in the slice's claims file, wherever `layout::spec_file` puts it — where the old path resolved, which is the point of an alias; Phase 2's check does not lint, so a deprecation reaches the agent only as the post-edit hook's context | `#[deprecated]` on the claim struct itself; a hard rename, with Phase 2's check tolerating unresolved citations; widening Phase 2's policy to the citing module | A deprecated `Spec` struct registers a claim that, once its citations move, has no implementer — checks 10 and 11 refuse it, and only Phase 2 could delete it. Unresolved citations are compile errors no lint level tolerates, and they stop the registry tests compiling too. Widening the policy gives Phase 2 the code it exists to be kept out of. The alias registers nothing, warns at exactly the citation sites, and clippy's `deprecated` is the one lint whose firing at Phase 2 is the methodology's own signal rather than a defect; the gate at Phase 7 still denies it. |
| Staging | The phase's allowed paths of both seats, plus — at Phase 7 only — the two workspace-root files the hook's own bump wrote | `git add -A`; the agent names files; widening Phase 7's editing policy to `Cargo.toml` and `Cargo.lock` so one set still serves both | The set that bounds edits bounds the commit; anything else the agent could not have written — and the two exceptions are exactly the files the agent did *not* write, because the hook did. Widening the editing policy to keep one set would hand the agent the manifest the policy exists to keep it out of, and would make the refusal message offer `Cargo.toml` as somewhere to fix a failing gate. Two sets that differ by what the hook writes is the honest shape; the editing verdict and `permitted_moves` keep reading the narrower one. **Claims:** `OnlyThePoliciesPathsAreStaged` and `TheStopStagesBothCratesAllowedPaths` are reworded to the staged set; `ChangesOutsideThePolicyRefuseTheStop` and `IntegrityFiltersAgainstBothCratesAllowedPaths` are reworded to filter against the staged set; `NothingToCommitIsARefusal` is reworded to say the test reads the *editing* set, so a Phase 7 whose agent changed nothing is refused though the bump dirtied two files; one new claim puts `Cargo.toml` and `Cargo.lock` in the staged set at Phase 7 and no other phase; one new claim requires each of those two to still equal the bump's own output after the check, refusing by name when it does not. |
| Check 12's diff base under the gate | The newest `phase 7:` commit reachable from `HEAD`; `git merge-base main HEAD` when the history holds none | Keep `main`; a `[workspace.metadata.lid_rs] mutation_base` setting; hand the gate to a detached runner and refuse until it reports | Measured here 2026-09-11: `main` as the base offered 1,390 mutants and about seventeen minutes, past the 600 s a subagent is allowed between stream events, so Phase 7 was committed by hand; the newest gate commit offered six. `main` also answers the wrong question — it is what the *branch* changed, and a phase is judged on what the phase changed. A configured base is a value whose wrong setting is a vacuous gate, and it reaches the hook from a file a phase could be argued into rewriting. A detached runner keeps the seventeen minutes, adds a process whose failure the hook cannot see, and leaves the commit waiting on it; making the run proportionate removes the reason for it. The `mutants` subcommand and CI keep their own bases, because a human and a release ask different questions. (This row's objection to a detached runner is answered, not reversed, by "The gate runs detached" below: what made that alternative unworkable here was a result the hook could not observe and a commit that still waited on it in the same call: a job record the next poll reads answers the first, and a commit decided on a later call rather than the same one answers the second. Narrowing the base is what bought time before either was designed; it stopped being enough once the narrowed base itself reached the cost measured at that section's opening.) **Claims:** one new claim gives the gate's mutation step a `--diff-base` of `gate_base`'s commit — a *sibling* of `TheBaseIsTheNewestGateCommitReachableFromHead`, not a sharer of it: that claim is the red run's rule about which commit `gate_base` answers with, and this one is the gate's rule about what the mutation step does with the answer, so either could change without the other. One new claim makes the base `git merge-base main HEAD` when the history holds no gate commit, and one more fails the step naming the ref when no merge base exists. |
| The version bump at Phase 7 | The stop hook raises `[workspace.package] version` one patch level from the manifest at `HEAD`, by a line patch, and regenerates `Cargo.lock`; every cargo step is `--locked` | The `toml` crate; `cargo set-version` from cargo-edit; leave the bump to the human, as it was | Eight hand commits on the record branch were version or lock repairs, and two of them were the same publish trap twice: a `phase 7:` commit that keeps the released version packages a version a registry already holds, and the gate that would have said so is the gate the commit claims to have passed. A TOML dependency is the one the `cargo-lid-rs` slice already rejected ("JSON / metadata parsing"), and one line of one file is not the evidence that overturns it. `cargo set-version` is a third-party binary the hook would have to require installed in every consumer, to edit the same line. Leaving it to the human is what was measured, and it is what failed. **Claims:** one new claim has `bump_workspace_version` write a version one patch level above the workspace `version` the root manifest holds as committed at `HEAD`, which is also its idempotence — where in a stop's order the bump runs is `gate_job::start`'s and no claim's; one for the line patch's target — the first `version = "` line between `[workspace.package]` and the next line beginning `[`, the rule that keeps a later table's `version` safe; one for its failures, naming what was looked for when the header, the line, or three numbers are missing; one for the lock being brought into agreement with `cargo update --workspace --offline`; one for `args_of` putting `--locked` on every cargo step; and one for the subject-version refusal, which is a sibling of `ACommitSubjectMustCarryThisPhasesTag` — that claim is about the tag and says nothing about a version. |
| The gate's doc step | `cargo doc --no-deps --document-private-items` | Keep the weaker `cargo doc --no-deps` | Without the flag rustdoc never visits a private item, so a broken link there is not an error but an item rustdoc did not read — and a LID slice is mostly private items. The weaker form hid a real rustdoc error until it was found by hand (record branch `d3f9040`). README §4.5, CLAUDE.md and the catalog's `doc` command carried the flag; the skill's `references/phase-7.md` and `.github/workflows/gate.yml` did not, and neither did this copy — all three were corrected together when the hook's copy was fixed, which is the drift §4.5's "every copy of the list a project keeps must match" forbids. **Claims:** one new claim gives `args_of`'s doc step `--document-private-items`. `PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce` and `PhaseOneChecksTheDocs` say only "doc" and "rustdoc" — they name the step's place in the order, not its arguments — so neither is reworded, and the flag is a claim of its own that both orders' doc step satisfies. |
| The companion seat's claims file | `seat_claims`: the layout's answer for the own seat, the companion module directory's `spec.rs` for the companion seat | Give `layout` a companion door (`companion_spec_file`) and ask it; move a crate-root proc-macro slice's claims to the companion's `src/spec.rs`; leave the own crate's relative answer in place | Measured 2026-09-12 on `lid-rs-macros`, the first crate-root proc-macro slice under a companion: Phase 2 was admitted only `lid-rs/src/spec.rs`, while the LLD, the six existing claims and their `macro_edge!` lines all live in `lid-rs/src/lid_rs_macros/spec.rs`, so the phase could not commit. Which directory a companion is, the layout already answers (`Form::Companion` is `module_dir`); which file in it holds claims is the colocation rule (`spec.rs` beside the module), not a second reading — so the policy joins the two answers it already holds rather than opening a door for a fact with one possible value. Moving the claims would split a slice's claims across the companion's crate root and its module against README §11.1. **Claims:** add `TheCompanionSeatsClaimsFileIsItsModuleDirectorysSpec`, and correct `PhaseTwoMayWriteOnlyTheCompanionsSpecFiles` in place, whose "the same answer the layout gives for the slice, placed in the companion" is the reading this row replaces and now contradicts the new claim. In place, not by the rename-and-alias rule this document states for a reworded claim: that rule exists so a changed behaviour is a new `struct` line the red run diffs and Phase 5 reddens, and here the behaviour is already delivered and gated under the new claim — a renamed claim would enter the red set with a validator that is green on arrival, which the red check refuses and no phase could make red. A claim whose text is corrected to match behaviour another claim already gates is a documentation defect (tenet 1) and keeps its name. Corrected, Phase 2's companion row is the companion's module directory's `spec.rs`, and its validator reaches that row through `workspace_paths`/`allowed`, which resolve the seat's claims file, rather than by calling `allowed_paths` with a raw claims path of the validation's own choosing — a validation that chose the path would assert the layout's own-crate answer the claim exists to refuse; rewriting that validator is Phase 5's, as work on a claim outside the red set that must stay green. |
| Stop-refusal budget | Refuse while the check fails, up to Claude Code's cap of eight | One refusal then allow (the first design); refuse forever | A failing check is not a reason to let the phase end; eight rounds of clippy output is more than a fixable phase needs, and the cap leaves a dirty, uncommitted tree the next precondition refuses. A `stop` block is always allowed, so an honest stop is never blocked. |
| Trusted binary in the tool's own workspace | Hooks name the installed `cargo-lid-rs` directly, refreshed from `main` after merge; no synced script | A synced `hooks/run` script preferring `cargo run -p cargo-lid-rs` here (the first design); a separate worktree build | A worker in this repository edits the hook's own source; running it from the tree means the policy is whatever the worker last wrote. Enforcing only landed policy is the price of the tool being its own consumer. |
| Instrumentation | A per-agent tally kept by the hooks, written as commit trailers | Parse `agent_transcript_path`; no instrumentation until the design settles | The hooks see every call and refusal; the transcript format is undocumented. Trailers put the measurement where the review already happens, from the first phase this design runs. |
| Phase 7's commit subject | `phase 7: <version>: <what and why>`, `<version>` being the one the bump will write, compared before the poll and so before the bump | The project's convention alone; the hook rewriting the subject's version itself; comparing after the gate | The tag is what makes the stop hook run the full gate; the changelog-readable part follows it. The version is checked rather than rewritten because the agent can compute it — the manifest is readable and the rule is one patch level — and a subject the hook edits is a subject the reviewer did not read. Comparing before the poll costs a second to refuse what would otherwise cost a whole gate, and refuses a wrong subject before the bump spends anything on it too. **Claim:** the subject-version refusal, named in the bump row above, belongs here too; `ACommitSubjectMustCarryThisPhasesTag` is unchanged. |
| Where the subject-version check runs, relative to the bump | Ahead of `gate_job::poll`, and so ahead of `gate_job::start`'s own bump: `subject_version_matches` holds the subject to `version_the_bump_will_write`'s answer, a pure read of the manifest committed at `HEAD`, and refuses a mismatch before any job begins — on every Phase 7 stop alike, whether that stop is the one that starts the job or the tenth that finds it still running | Comparing after the bump, as the retired claim's own sentence had it; folding the comparison inside the detached job itself, so a wrong subject surfaces only once the whole plan has run | The comparison costs nothing to ask early: reading the manifest at `HEAD` runs no command and starts nothing, so asking it ahead of the poll costs a second rather than a gate, exactly as the tag check in the row above already does. Asking it after the bump would raise the workspace version and regenerate the lock — a write, not a read — for a stop that was going to be refused anyway, and pay that cost on every stop a long-running job spans rather than once. Folding it inside the detached job answers it only after `phase-check 7`'s own measured cost has run (The whole gate runs detached, not one step of it, above), spending on a wrong subject exactly the gate it never needed. **Claims:** `APhaseSevenSubjectMustCarryTheBumpedVersion` is renamed to `APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll`, the retired name kept beside it as a `#[deprecated]` alias, by the rule this document states for a reworded claim: its old sentence names the comparison as running "after the bump and before the check runs," an order the code reverses rather than narrows, which is false and not merely imprecise — the test this document uses to tell a rewording from a correction in place. |
| Phase 7 gate duration in a hook | **Retracted.** `timeout: 3600` in the agent's frontmatter bounds only how long Claude Code lets the Stop hook *command* run; it does nothing about the separate no-stream-progress watchdog that ends the whole *agent* at 600 seconds regardless of any hook's own timeout — measured against three Phase 7 workers killed inside the stop hook in one day. "The gate runs detached" is the actual answer, for the whole gate and not only check 12 | Move mutants to CI only | A gate that exists, gates, which is why a frontmatter setting looked like an answer before it was measured against what it actually bounds: ordinary configuration, not a new mechanism, and configuration was the wrong shape for this problem. It does not hold, and did not. The floor's own cost already fits inside 600 seconds without any mechanism in this row, so nothing here moves mutants to CI either. |
| What runs detached | The whole Phase 7 plan, as one job | Only `Step::Mutants`, leaving the other seven steps and `gate_extra` inline | Detaching only the slow step is sound under one assumption: it adds no start-and-wait shape to steps that already fit inside 600 seconds, for no benefit, so long as whatever asks the question is the same worker that would otherwise have blocked. That assumption does not survive an orchestrator that polls across turns rather than a worker that waits within one: `checked` re-running `phase-check <n>` fresh on every retry re-paying the floor, and a worker spending its own refusal cap on an answer that has not changed, are costs of *who* asks and *how*, not of how many steps are slow. Once the poller is the orchestrating session rather than the phase worker (below), detaching only the slow step buys nothing a whole-plan job does not already buy more simply — one key, one record, one poll for every step, rather than a rule for which steps need the machinery and which do not. **Claims:** one claim has `gate_commit`'s Phase 7 arm call `gate_job::poll` for the whole plan in place of going straight to `after_undo`, while every other phase goes straight to `after_undo` unchanged — one claim for both arms of the same dispatch, not two: the Phase 7 arm is what a Phase 3 skeleton answers wrongly and Phase 5 reddens, and the other arms are pre-existing code this row does not touch, which is the companion-seat row's green-early case and not a second claim. This phase decision is `gate_commit`'s alone: it is asked in `detached_gate`, before any check has settled, over which phase's stop can be answered by a poll rather than run inline — for a phase that need not be the one whose bump lives in `gate_job::start`, if a later slice ever gave another phase a detachable check of its own. Folding it into `after_undo` would need `after_undo` itself to return the third ending, which is exactly the return `after_undo` — and everything that calls it — is kept free of. One more claim belongs here and is easy to miss: `gate_commit` — not `checked` — tallies `Event::StopCheck` for Phase 7, and it does so exactly once, at the stop that reads a settled `Done`, never at a stop whose poll answers `Started` or `Running`. `checked` still records the event for every other phase, unchanged; the claim is that `gate_commit`'s own Phase 7 arm carries the same recording duty `checked` cannot, since `checked` is never called there, so `Lid-Rs-Checks`' stop count keeps naming how many times a phase's check ran to a finished verdict rather than how many times anything polled for one. |
| Where the detached job's state lives | `<target>/lid-rs/gate/<checkout's canonicalised path, hashed>/<slice>.json` for the running record, `<target>/lid-rs/gate/<key's hash>/…` for the keyed result and scratch space | A dotfile outside `target/`; the OS temp directory; the `lid-rs/mutants/` path used while only check 12 was detached; a running record named by slice alone, with no checkout in its path | `target/` is already untracked, git-ignored, per-checkout scratch this slice already uses this way for the tally (`<target>/lid-rs/agents/<agent_id>.json`); a dotfile needs its own gitignore entry and a path some phase's policy would then have to admit; the OS temp directory is not scoped to a checkout and is not cleared by the `cargo clean` that already clears a stale gate cache. A path under `lid-rs/mutants/` would misname what is now kept there — the whole gate's record, not one step's — so it is `lid-rs/gate/` instead. A record named by slice alone collides under exactly the shared-`CARGO_TARGET_DIR` case the key's own checkout field exists for: two worktrees on one slice — `lld/foo` and `lld/foo--change` are both slice `foo` — would share one file and overwrite each other's `Running { key, pid, version_files }`, and since a poll only ever consults `result_path` once its own key matches what the record currently names, an overwritten record can make a finished, passing job's result unreachable to the checkout that ran it. Nesting the record under the checkout closes that: each checkout's poll reads and writes only its own file. The bound this design holds is per checkout, not per slice — it runs at most one job at a time per checkout, and the checkout-scoped path is what makes that true rather than assumed. **Claims:** one claim gives the running record's path as `<target>/lid-rs/gate/<checkout, hashed>/<slice>.json`, a sibling of the tally's own directory; one asserts that two checkouts of the same slice, polling concurrently, read and write only their own record and never see the other's; one — `TheRunningRecordCarriesTheBytesTheBumpWrote`, renamed to `TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder` since its old sentence names `start` as the one that bumps and captures in the same breath, which is no longer a leaf that exists — has `gate_job::running_record` carry the key and the pid it is given, and `integrity::contents_of`'s answer for `Cargo.toml` and `Cargo.lock` — in that call's own order — as `version_files`, checkable by handing it a key and a pid over a project whose two files are known, with nothing spawned; one — `AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord`, corrected in place rather than renamed, its "naming the key, the child's pid and the captured bytes" clause dropped now that the claim above carries it where a fixture can reach it — has `gate_job::start` bump the workspace version exactly once per job — never on an already-running or already-`Done` key — spawn the whole plan as an unwaited child, and then write, through `write_record`, the record `running_record` built for it, before returning. |
| What the record is keyed to | The checkout's canonical path, the branch tip's hash, `mutation_base`'s answer, the slice's name, and a fingerprint over the phase's editing set; `gate_job::key` takes `phase` and `crates` as well as `project` and `slice`, since the fingerprint's input is `policy::workspace_paths(project, phase, crates)`'s answer | HEAD alone; a `git stash create` tree hash; no key, trusting call order; the fingerprint alone, without the checkout; a two-parameter `key(project, slice)` | HEAD alone misses an edit made after the job starts, which is exactly the case this key exists to catch. `git stash create` and a tracked-changes-only fingerprint both miss an untracked new leaf (the fingerprint row, below, settles which form is used instead). Trusting call order is what "an edit invalidates the result, and the stop must catch that rather than trust ordering" refuses by name. Without the checkout, two worktrees at the same HEAD and slice with an identical editing-set fingerprint — routine under a shared `CARGO_TARGET_DIR`, and this project runs phase agents in worktrees — would compute one key and accept each other's result for what are, on disk, two different builds. A two-parameter `key(project, slice)` has no path to `workspace_paths` at all and cannot compute what it is asked to fingerprint — the real signature was never buildable, not merely under-specified. The key is honest about its reach: it fingerprints the phase's own editing set, not the whole workspace diff check 12 actually runs against; a write outside that set is caught by `outside_policy_clean`'s integrity pass on every phase, key or no key, and this key does not try to catch it twice. **Claims:** one new claim names the key as the checkout, HEAD, base, and slice, plus the fingerprint; one asserts a mismatch in any one part answers exactly as no record at all; one gives `key` its four-parameter signature. |
| The fingerprint's form | A walk of every path `workspace_paths` admits — recursive for a directory, once for a file — hashing each regular file's relative path and bytes, sorted and concatenated | `git stash create`'s tree hash; a diff of tracked changes against HEAD | Both alternatives read git's index, which does not include a file the phase just wrote and has not yet staged — exactly a Phase 7 leaf before `stage_and_commit` runs. A filesystem walk reads what is actually on disk and needs neither git's index nor a guess about a leaf's tracked state. Settled here, not deferred to Phase 5: `workspace_paths`' answer is a directory and a module file matched by prefix, not a file list, so nothing about it is hashable without first walking it, and that walk is the whole of this decision. **Claims:** one new claim gives `gate_job::fingerprint` this walk-and-hash form over `workspace_paths`' admitted entries; one asserts a new, untracked file under an admitted directory changes the fingerprint. |
| The job's scratch space | The diff file and the output root are derived from the key and the project's own build directory (`<target>/lid-rs/gate/<key's hash>/…`), with the key's own directory made before either path is handed to `mutants::run_at(project, args, diff_path, output_root)` rather than resolved or created by it; independently, a start is refused outright while a poll finds a live job recorded under a different key | Fixed paths, relying on the key alone to keep results from crossing; fixed paths with a lock file; a wrapper in `phase` that shells out to `mutants::run` and writes the record itself, without changing `mutants.rs`; leaving the directory for `mutants::run_at` or the mutation engine to create | `run_group` empties its output directory before every group and reads `outcomes.json` back from it, and `write_diff_file` writes one fixed diff file; two jobs sharing either path do not merely collide — a stale job's own cleanup deletes a fresh job's still-being-written output, and a fresh job can read outcomes a stale job left behind, which is the exact guarantee that removal exists to give, defeated silently, so a phase would not stop on this, it would pass, which is worse than a crash. Keying the paths closes that regardless of what starts when; refusing a second start while a different key's job lives is a separate, resource-driven rule on top of it — two 1087-second gate runs on one machine is a cost nobody asked for — and neither makes the other unnecessary. A lock file changes nothing about the deletion itself. A wrapper in `phase` that shelled out to `mutants::run` and watched from outside would have to re-derive when the mutation engine had actually finished and what it decided; calling `mutants::run_at` in-process instead is what lets `run_job` capture that `Result` directly and write it itself (Crossing the process boundary, below), without a second process guessing at it. `write_diff_file` writes through the diff path as given, with a plain `std::fs::write`, and fails if the parent is missing — under `Scope::Diff` alone — before `run_group` ever runs; `run_group` does make its own output directory, and everything above it, through `create_dir_all`, but only once its own turn comes, which is too late for the diff file already written or already failed to write. So the diff file is the write with nowhere else to get a home made ahead of it, and a home that does not yet exist there is a write that fails, naming the missing directory, unless something makes it first. **Claims: none here for the cascade itself** — the two path parameters `mutants::run_at` accepts belong to `mutants.rs`'s own LLD; the claims for that half are that slice's own Phase 2, against its own document, not this one's. **This slice's own half does have claims:** one gives `gate_job::paths_for(project, key)`'s two paths and `gate_job::result_path(project, key)`'s one, each a derivation from the key and `project.target_directory()` alone (`<target>/lid-rs/gate/<key's hash>/…`) — the same door `record_path` and `tally::path` already resolve the build directory through, since a `JobKey` carries the checkout and not the directory a shared `CARGO_TARGET_DIR` can put anywhere — so a validation can assert the formula against a project fixture without running `mutants::run_at` at all; one has `gate_job::run_job`'s own runner send every step but `Step::Mutants` straight to `run_step`, unchanged, and divert `Step::Mutants` alone to call `mutants::run_at` with `prepared_paths(project, key)?`'s paths — the one decision `run_job` makes, over which step it is running, rather than a second reading inside `run_step` itself. One more claim is `contest`'s own: a pid recorded under a foreign key that answers alive refuses the start outright, naming the other key, rather than being waited for or restarted. One more claim is `prepared_paths`'s own — `TheMutationStepsKeyedHomeExistsBeforeItWritesThere` — that it makes the key's own directory before answering `paths_for`'s pair, so `keyed_mutants` hands `mutants::run_at` a home that already exists rather than one whose absence only `write_diff_file`'s own plain write, under `Scope::Diff`, would otherwise discover as a failure; `paths_for` keeps its claim exactly as it was, over the formula alone, so a validation can still assert it with nothing on disk. |
| Detecting an abandoned job | The spawning call records the child's pid; `gate_job::poll` tests the written result *before* liveness — `settle`/`gate_job::finished` first, `alive` (`kill -0`, fallibly) only when nothing is written yet — and finding a pid dead **with no result written** restarts under the same key, whichever key that pid was recorded under: this key's own dead-and-resultless record restarts under the same key (`restart_or_running`), and a *different* key's dead record — stale, and blocking nothing — also starts a fresh job under the current key (`contest`) rather than being refused | A wall-clock timeout on the running record; a lock file (`flock`) the child holds; an infallible `alive: bool`; testing liveness before the result | A timeout needs a threshold nothing here has measured. A lock file needs the child to release it on its own crash, which is the one case this check exists for. An infallible `alive` cannot distinguish "cannot tell" from "dead," and "dead" means restart — so a host lacking the mechanism would restart a 1087-second job every poll, forever, the exact failure this section removes, reintroduced at one call; this document already holds that a step which could not run is not a step that passed, and applies it here. Testing liveness first would call a finished job's pid dead a moment before checking whether it left a result, and a live-but-just-finished process cannot be told apart from one that finished and exited on its own — the result is the fact that does not race, so it is asked first. **Claims:** one claim makes `alive` fallible and a query failure a step failure, never a restart; one has `settle` read the result before `restart_or_running` asks about the pid; one has a dead pid under a foreign key start a fresh job under the current key rather than being refused — the refusal in the row above is for a *live* foreign pid alone; one has `restart_or_running` itself answer `Running` for a live pid and, for a dead one, restart under the same key and answer `Started`, once `settle` has found no result written. |
| The read-only status door's answer over a live foreign job | `GateStatus` gains a fourth variant, `Foreign(JobKey)`, answered by a status-only sibling of `contest` (`foreign_status`) that never starts anything; the same three-way classification `act` makes is repeated in `status` rather than shared by call, because `act`, `restart_or_running`, and `contest` all call `start` in some arm and `status` must never reach it | Fold a live foreign job into `NoJob`, as a first draft did; give `act` a `starts: bool` parameter threaded through `settle`/`restart_or_running`/`contest` so one set of functions serves both callers | Folding a live foreign job into `NoJob` is actively wrong, not merely imprecise: an orchestrating session reading `NoJob` as "nothing to wait for" would resume a worker whose next `start` is refused outright by the very job `status` failed to mention, wasting a turn on an answer the record already had. A `starts: bool` parameter threaded through three functions whose `Done`/`Running` arms need no starting decision at all would carry that flag into arms that never look at it, for the sake of arms that do; two small status-only siblings built on the same primitive reads (`finished`, `alive`, `read_record`, `key`) cost less than the threading and need no arm to ignore a parameter meant for another. **Claims:** one claim adds `GateStatus::Foreign(JobKey)`; one has `gate_job::status` share `act`'s top-level three-way split (no record, matching key, foreign key) without ever calling `start`; one gives `matching_status` the same finished-then-alive decision `settle`/`restart_or_running` make, answering `NoJob` where they would restart; one gives `foreign_status` the same alive decision `contest` makes, answering `Foreign(other)` where `contest` refuses and `NoJob` where `contest` would restart. |
| What the status door's own line and exit status mean | `status_line(status: &GateStatus) -> Result<String, String>` answers each `GateStatus` variant with its own line — `NoJob`, `Running`, `Foreign(other)`, `Done(Ok(()))` — and answers `Done(Err(output))` with this call's own `Err`, carrying `output` unchanged; `report_status` hands that answer straight to `announce` and returns it unchanged, so the process's exit status is `status_line`'s decision and nothing else's | Leave the `match` inside `report_status`, whose own branch already chooses the line and, for `Done(Err(output))`, returns `output` as the process's own failure; assert the printed line by capturing stdout | `report_status`'s `match` prints through `announce`, which returns `()` on every line it prints, so the only fact a test can hold that `match` to is whether the call returned `Ok` or `Err` — not which line a `GateStatus` produced, which is the actual rule this door must keep: `NoJob` must never print as `running`, and a `Foreign` job must never print as `no job`. Splitting the decision into `status_line` gives it a return value a validation calls directly, one case per variant, with no process to spawn and no stdout to capture, the same reasoning `default_paths` and `prepared_paths` already carry for a pure decision beside a door that touches the filesystem; `report_status` is left holding one call handed to another, with no branch of its own. The variant that matters is `Done(Err(output))`: an orchestrating session reads this door's own exit status to decide whether to resume a worker (Who checks back, and how, above), and a `status_line` that answered a printable line for a finished failure would let `report_status` return `Ok(())` for a failed gate — the one outcome an orchestrator must never read as a pass. **Claims:** one new claim has `status_line` answer `NoJob`, `Running`, `Foreign(other)`, and `Done(Ok(()))` each with their own line, and `Done(Err(output))` with this call's own `Err` carrying `output` unchanged. `report_status` handing that answer to `announce` and returning it unchanged is not a second claim: a call passed straight through to another has no wrong answer distinct from the one `status_line` already names. |
| The pending ending, and who polls it | A `pending` result `gate_commit` returns and `commit_phase` maps to an allowed stop — commits nothing, refuses nothing, ends the worker's turn — for `Started` or `Running`; the orchestrating session (the workflow, or the interactive main session — whichever spawned the worker) is what checks back later, by resuming or respawning the phase-7 worker, never the phase worker looping on its own stop | Answer `Started`/`Running` as a refusal, sleeping and re-checking up to a bounded ceiling before returning still-running; answer immediately as a refusal with no wait at all; have the phase worker's own tool set gain a wait | Sleeping and re-checking is unsound: `checked` re-running `phase-check <n>` fresh on every retry re-pays the floor, so even a bounded wait needs two attempts to clear a check-12-sized job — 466 seconds of duplicated floor work (two attempts at the ~233 s floor), and two of Claude Code's eight-consecutive-refusal cap. Refusing with no wait at all is worse, not better: each retry re-pays the floor before it can even ask about the mutants step, so clearing the same 854-second mutants run this way costs four attempts and about 932 seconds of duplicated floor work — twice the bounded wait's cost and twice its share of the cap, for the same answer. A wait needs a tool the phase agent's tool list does not have and should not gain: `tools:` is deliberately Bash-less (What the agent may run, above), and a sleep is exactly the kind of thing an agent should not be trusted to run for itself. Ending the worker's turn cleanly and letting a different session — one with no inter-turn watchdog, no refusal cap of its own to spend, and no reason to re-run anything, because the record it reads is a file, not a re-execution — check back is the shape that survives review. Because a pending stop is never a refusal, there is nothing here to exempt from the eight-refusal cap or from `Lid-Rs-Refusals`: the carve-out a still-running wait would need disappears with the mechanism it would have been carved out of, rather than being answered. `HookVerdict` itself gains no variant, and cannot: three sites this slice's own phases may not edit — `headless_canopy_agent/tools.rs`'s pre-tool and post-edit dispatch, `headless_canopy_agent/ending.rs`'s stop dispatch — match it exhaustively with no wildcard arm today, and a fourth variant fails `cargo check --all-targets` inside those modules, which a Phase 3 worker on this slice cannot fix and would end its run on a dirty tree over. `GateCommit::{Committed, Pending}` carries the distinction instead, internal to this slice and never crossing the wire; `commit_phase` maps both to `HookVerdict::Allow`. There is no breaking change to `cargo-lid-rs`'s public enum. **Claims:** one claim has `gate_commit` restore the tip and return the third ending, with nothing staged and nothing committed, when `gate_job::poll` answers `Started` or `Running`; one asserts `commit_phase` maps a `Pending` ending to `HookVerdict::Allow` — the same value a `Committed` ending produces — while recording neither `Event::StopCheck` nor a tallied refusal for it: no numbered decisions, no proposed message, no refusal reason, and no `Event::StopRefusal`, so the wire is silent about the difference and only the tally and the record carry it. That this ending is never chosen by the agent's own message needs no claim of its own: `AFinalMessageCarriesExactlyOneEnding` already leaves `ending_of` no third block to parse, so nothing but the poll can produce it. |
| The detached child's own command line | `gate_job::child_command(phase, slice, key)` builds the line `start` spawns — `phase-check <n> --slice <slice> --job-key <key's hash>` appended to whatever `self_command()` already carries — and returns it unrun; `phase::run` gains an optional `--job-key <hash>` beside `<n>` and `--slice`, carrying a hash rather than a `JobKey`; present, `run_job` resolves `project`, `phase`, `slice`, and `crates` exactly as any other invocation does, recomputes `gate_job::key` itself, and checks the result's hash against the one `--job-key` carried before running any step of `plan` | A flag wide enough to carry `checkout`, `head`, `base`, `slice`, and `fingerprint` verbatim, trusted as given rather than recomputed; the line assembled inline inside `start` rather than answered by a leaf of its own | A `JobKey` is not a value a command line can round-trip — nothing reconstructs one from the one-way digest `--job-key` names — so recomputing from the same three arguments `gate_job::start` already resolved is a check, not a transport: a matching hash is the proof that the child is about to gate the same tree the parent keyed when it spawned it. A flag wide enough to carry the key's fields verbatim would still have to be trusted across the process boundary, exactly what a confused-deputy design refuses everywhere else in this document; recomputing costs the child three arguments it already holds and nothing else. A mismatch is a real condition and not a corner nothing reaches — an edit can land in the fingerprint's own walk between `gate_job::start` computing the key and this call recomputing it — so it must not gate a tree nobody asked it to gate: it runs no step of `plan` and writes to no result path at all, not the path the given hash would derive, because `run_job` never held the `JobKey` that hash names, and not the path its own fresh key would derive, because nothing polls a key this call was never asked to run under. The running record still names the original key and this child's own pid, so once it exits having written nothing, that pid answers dead, and the next poll's `restart_or_running` starts a fresh job under whatever key the tree computes then — the ordinary died-before-finishing recovery every other crash already gets, not a second mechanism. Answering the line as its own leaf rather than assembling it inline inside `start` is what makes it assertable on its own terms: a program and an argument list are a fact about a string, checkable without spawning anything, the same reason a matching hash proves the tree without trusting a value handed across the process boundary. **Claims:** one has `run_job` recompute the key from `project`, `phase`, `slice`, and `crates` resolved the same way any other invocation resolves them, and check the result's hash against the one `--job-key` carried before running any step of the plan; one has a mismatch run no step and write to neither the given hash's own directory nor the fresh key's, leaving the recorded pid to answer dead and the next poll to restart under whatever key the tree computes then, exactly as any other attempt that died before writing a result already does; one more gives `gate_job::child_command`'s own answer — for a phase, a slice, and a key, the returned command's argument list ends with `phase-check`, the phase's number, `--slice`, the slice, and `--job-key`, the key's hash, appended to whatever `self_command()` already carries; the program and any leading arguments are `self_command()`'s own, unexamined by this claim — the parent's half of the same contract the other two are the child's: what `run_job` recomputes and holds `--job-key` to is exactly what `child_command` appended to spawn it under. |
| Crossing the process boundary | The detached child writes its raw result — pass, or the first failing step's captured output — to the slice's keyed `result_path` as structured data (`Done(Ok(()))` or `Done(Err(output))`), which `gate_job::poll` reads back and returns to its caller as an ordinary `Result`; whichever `hook_stop` invocation reads a `Done(Err(output))` builds the refusal, since only that invocation — never the detached child — holds `phase` and `crates`, but it hands `refusal_for` `gate_job::bounded_output(project, key, output)?` in place of `output` itself, so the message never splices the captured text a `Done(Err(..))` already wrote to `result_path` | A fixed marker line in captured text, read the way `marker_check` already reads `"is not red"`/`"survived"`, from when only check 12 crossed the boundary; have the child call `refusal_for` itself; have the parent guess from the exit code alone; hand `refusal_for` the raw `output` unbounded, as a synchronous check's own failure always has | A marker line was a text convention standing in for a value the record can just hold structurally now that the whole plan, not one step's stdout, is what the child produces; reading structured data back needs no substring convention and cannot collide with a step's own output the way a fixed string could. The child holds neither `phase` nor `crates` — both `CommitPlan`-derived, computed only in the parent — so it cannot build the message `refusal_for` builds; only the parent can. An exit code alone cannot distinguish "still running" from every other reason a child might not have written a result yet. Handing the raw capture straight through is what a synchronous failure has always done because the capture had no other home; here it does — `result_path` — and splicing all of it anyway duplicates what the file already holds and, measured here 2026-09-13, once reached 1.3 MB and killed the worker reading it, so naming the file for the remainder costs one path instead. A bare header plus that path would go too far the other way: `refusal_for` identifies which `gates.md` row applies by searching the text it is handed for a clippy lint name or the red-run/check-12 markers, and a header contains none of them, so a header-only stand-in would misreport every detached failure as an unnamed compile error. **Claims:** one new claim has the detached child write its outcome as `Done(Result<(), String>)` to `result_path` rather than relying on any convention read from captured text; one gives `gate_job::bounded_output` the header-plus-tail form — the failing step's header, the last 8 KiB of its own output, and `result_path`'s path for the rest (A `Done(Err(..))` must not carry what the file already holds, above); one asserts a `Done(Err(..))` reaches `refusal_for` through that bounded stand-in rather than through the raw capture, so the refusal this poll produces is bounded regardless of how large the failing step's own output is; one asserts `check_of_output`, run on that same bounded stand-in inside `refusal_for`, still names the check that fired for a clippy lint or a red-run/check-12 marker within the kept tail. |
| The human fallback | `cargo lid-rs phase-check <n>` and `cargo lid-rs mutants`, run directly, keep calling `check` and `mutants::run` in-process and blocking on them exactly as today; `--job-key` is never set by a human or CI, only by `gate_job::start`; only the stop hook's own Phase 7 arm calls `gate_job::poll` | Make polling the default for every caller; a separate subcommand a human must learn | A terminal has no 600-second watchdog, so the documented fallback — run the whole gate in one sitting, commit by hand when the hook cannot — needs nothing new to keep working, and README §4.5's list stays one list a human, CI, and the hook all read the same way. Defaulting to polling would ask a human at a terminal to check a record file instead of watching cargo's own output, for a limit that is not theirs. A separate subcommand is one more name to keep in sync with the six the gate already runs by number. **Claims:** one claim gives `phase::run`'s handling of `--job-key` both its arms — absent, `check` and block exactly as before this section; present, `gate_job::run_job` instead. The absent arm is green the moment the flag exists to be absent, the same way an unconfigured `gate_extra` is green early; the claim is red through the arm that is new. |
| Worktree per worker | Deferred | The Workflow's `isolation: "worktree"` per phase agent | Which branch a temporary worktree checks out is undocumented; a commit there must land on `lld/<slice>`. The dirty-tree precondition covers the failure the worktree would have contained. |
| Phase 5 test execution | One `cargo test … --exact` run per validation, exit status as verdict | One `cargo test --lib` run with libtest output parsed; `--format json` | One process per test costs seconds on a slice-sized set and needs no parsing of libtest's human-oriented output; JSON output is nightly-only. |
| Phase 5 slice identity | `SPEC` records by source file, which `layout::spec_file` answers for either layout; slice from the branch name | Parse `src/spec/` for the module; a `--claims` list | The registry already carries the file; the branch convention already carries the slice; constraint 2 forbids the parse. |
| Phase 7's list | The tool holds README §4.5 verbatim, in order, as one more copy the README's rule binds | Make `cargo lid-rs gate` canonical and reduce the README to a pointer | Keeping the list canonical in prose is deliberate for now: the spec stays readable without the tool. Promoting the tool is a README change with its own slice. |
| Where a workspace's own gate steps are declared | `[workspace.metadata.lid_rs] gate_extra`, falling back to the root package's `[package.metadata.lid_rs]` as `mutation_scope` does — a list of commands, each a list of strings, read from `cargo metadata` | A shell string per step; a `gate_extra.sh` the tool runs; leaving them in CI, as they were; a `cargo lid-rs gate` the project wraps | A list of strings is the same shape a step's cargo arguments already have, and it has no quoting grammar, no word splitting, and no expansion for a manifest to get wrong. A shell string adds all three and a shell. A script file is a path the tool would have to trust and a project would have to keep executable, and it hides which steps exist from anything reading the manifest. Leaving them in CI is what was measured: a gate that exists only in CI does not gate the commit that claims to have passed it, and this workspace's `mdbook build book` was run by hand for every slice that shipped. The metadata is already where `mutation_scope` and `companion` are read from, so the reading is one more `metadata.lid_rs` key and no new file, and `mutation_scope`'s fallback to the root package's table comes with it, which is what lets a single-package project — `init`'s scaffold, and this slice's fixtures — carry the key at all. **Claims:** one new claim puts the extra steps after the mutation step in `plan`'s answer for phase 7; one makes an absent key the empty list, so the floor alone is what an unconfigured workspace runs; one has an entry run as a program at the workspace root and through no shell; one makes a failing or unrunnable entry the check's failure, naming the entry; one has the list read from the metadata `cargo metadata` reports rather than from a manifest; one fails the check naming `gate_extra` and the entry it could not read when an entry is not a non-empty list of strings; one fails the check naming `gate_extra` and what it found when the value is not a list at all, which is neither an absent key nor an entry. The three about the value are cited by `policy::gate_extra` and not by `Project::setting_node`: the raw door is a hand commit no phase can write, so a claim implemented only there has no Phase 3 to leave it `todo!()` and no Phase 5 that could make it red. |
| How `policy::gate_extra` is skeletoned | Wired into `check` at Phase 3 with a body answering the empty list, so `plan` carries no extra step until Phase 7 and the four metadata claims are red by assertion | Declared unwired with a `todo!()` body until Phase 7; wired with a `todo!()` body; `check` reading the value only at Phase 7 | `check` is reached by every validation of a phase's check outside the gate_extra change's red set, so a wired `todo!()` turns them red for a reason that is not theirs, and Phase 3's own check could not pass; unwired, the item is dead code in a private module and the post-edit lint refuses it. The empty list is the wrong answer that compiles: a configured key answered as empty fails the read-with-fallback validation, a malformed key answered as empty fails the two refusal validations, and an absent key answered as empty is the one case green early, which Phase 5 states. Reading only at Phase 7 would put a phase decision inside `check` that the malformed-key row says it does not have. `extra_step` keeps the ordinary `todo!()`: nothing green reaches it while the list is empty. |
| Where the extra steps sit in the gate's order | After every step of README §4.5's floor, the mutation step included | Before the mutation step, so the slowest step stays last; interleaved by configuration | The floor's order is cheapest-and-most-specific first, and a workspace's step is neither cheap nor specific to the tool: it is a build-integrity step over a tree the floor has already accepted, so running it before the floor's own steps would spend it on trees the floor rejects. Interleaving by configuration makes the order a value a manifest can get wrong, and README §4.5's list is the order. The cost is that an extra step follows the gate's longest step, so a workspace's step fails late; the alternative is failing the floor's cheap steps late instead. |
| How an extra step is carried through `plan` | `Step::Extra(Vec<String>)`, one variant per entry, with `plan` taking `extra: &[Vec<String>]` as a parameter beside `publishing` | A single `Step::Extras` that reads the metadata when it runs, as `Step::Mutants` resolves its base; a `GateInputs` struct in place of the two parameters | A step whose arguments are data is a step a validation can assert without running it — the reason `args_of` exists — and the extra steps are configuration, which is exactly the shape `publishing` already has as a parameter. `Step::Mutants` is the documented exception because its base is a git fact resolved when the step runs and is one value; a list of commands read at plan time is neither. A `Step::Extras` could be observed only by running the programs, which for "in this order, after the mutation step" is no observation at all. A struct for two lists is a name for the pair and nothing else today; it becomes worth adding at the third configured input. |
| When a malformed `gate_extra` fails | At whichever phase's check builds the plan, failing the check naming `gate_extra` and the entry it could not read | Only at phase 7, where the value is used; ignoring an unreadable entry | The value is read where the plan is built — `check`, through `policy::gate_extra` — which is one place, and a configuration a project cannot have meant is a defect whose earliest naming is its cheapest. Deferring it to phase 7 means a typo survives five phases and stops the gate. Ignoring it is the vacuous pass constraint 3 forbids: a gate step that is silently dropped is worse than one that was never configured. |
| The workflow's input | A branch with a human-approved `phase 1:` commit; no waiver argument | A slice name, with the workflow drafting the LLD; a `--waive` argument | Phase 1 is human-owned; a workflow that drafts it and continues has approved its own LLD. A waiver given once is reused; an argument is a waiver given every time. |
| Reviewer at each stop | One clean agent per phase, prompted to refute, one rework round | No reviewer; a judge panel per phase | A clean reviewer is also the test that the artifact is context-free — the failure interactive mode cannot see. A panel exceeds the cost a slice warrants; one rework round bounds the run. |
| Where the artifacts live | `agent/` and `workflow/` beside `skill/` in the `lid-rs` crate, synced under one rule | Inside `skill/`; a separate crate; the plugin | Claude Code reads agents and workflows from `.claude/agents/` and `.claude/workflows/`; the files are version-coupled to the skill they point at, so they ship with it. |
| What a rework's commit does to the rejected attempt's | Replaces it: the hook undoes the tip it made for this phase and commits in its place, so a phase has one commit | Stacking a second `phase <n>:` commit, as it was; squashing at the end of the slice; a `fixup!` commit left for the human's rebase | A phase's commit is what its review reads, and a review's instruction is to read the newest commit on the branch: under stacking the second review reads the rework's delta and never sees the phase whole. The log loses the walk it exists to show — two commits, one subject, no way to tell which is the phase. And at Phase 7 stacking moves the gate's own base onto the rejected attempt, so the mutation step covers the delta while the commit claims the slice. Squashing later needs a step nothing in the design has, on a branch the human may already have pushed; a `fixup!` leaves the branch wrong until someone rebases, and the rebase is exactly the git the agents do not hold. **Claims:** one new claim replaces the tip rather than following it; one holds the replacement to a tip whose *subject tag* names this phase; one to a tip whose *`Lid-Rs-Phase` trailer* names this phase — two claims and not one, because the two fail independently and the commit that separates them is real: a `phase 7:` commit a human made when the watchdog killed a gate carries the tag and no trailer, and swallowing it is the worst thing this decision could do. One more refuses to replace a tip already reachable from `main`. |
| How a rejected tip is told from an accepted one | It is not on `main`: a tip reachable from `main` is accepted and never replaced, and a tip not on `main` carrying this phase's tag and the hook's `Lid-Rs-Phase` trailer is this phase's attempt being reworked | A `Lid-Rs-Rejected` trailer the reviewer's seat writes; a marker file the workflow keeps; comparing the tip to the branch point instead of to `main` | Nothing in a commit says it was rejected, and nothing can: rejection is a verdict the reviewer reaches after the commit was made, and the only writer at that moment is a reviewer with no git and no tools but reading. So the question is turned around — what marks a commit as *accepted* — and landing is the answer the design already has. This is a **hand seam**: the rule is sound only while accepted phase commits reach `main`, and under this workspace's per-phase-squash landing every accepted phase commit does, so a tip that is not on `main` is by construction an attempt this branch has not yet landed. A trailer written after the fact needs a writer that holds git, which is the one thing the agents are denied; a marker file is state outside the commit, which a fresh clone and the canopy client would both lose. Comparing to the branch point rather than to `main` fails the case the guard exists for — a branch cut from a `main` that already holds this slice's phases — and `merge_base_with_main` already fails naming `main` when there is none, which must not become every stop's failure. **Claim:** the on-`main` refusal is the claim named in the row above; this row records what makes it the right test rather than a second one. |
| How the replacement is made | `git reset --soft HEAD~1`, after the nothing-to-commit test and before the bump and the check, then the ordinary commit | `git commit --amend` at the end, with `gate_base` and `manifest_at_head` told which commit to skip; `git commit` then `git reset --soft HEAD~2` and commit again; undoing before the nothing-to-commit test | Undoing first is what makes every later reading correct without knowing a rework is in flight: `manifest_at_head` reads the manifest below the release, `gate_base` and `mutation_base` answer the gate below the replaced commit, the red set's diff is unmoved, and the staging sees both rounds. Amending at the end would thread "the commit being replaced" into two independent readings, and a reading that forgot it would silently narrow the gate — the failure mode this slice exists to remove. The nothing-to-commit test is the one thing that must come *before* the undo: after it the editing set holds both rounds, so a rework that changed nothing would pass the test and would already have deleted the tip it meant to amend. The undo's cost is that a failing check finds the branch a commit shorter, which the row below pays for with the restore. Committing then resetting twice runs the check against the wrong `HEAD` and gains nothing. **Claims:** one new claim puts the undo before the bump and the check — the half a validation can redden, since an undo placed later leaves the tree at the unreplaced version and `HEAD` unmoved; the "after the nothing-to-commit test" half is prose, observed through `NothingChangedInTheEditingSetIsARefusal`, whose rewritten validation asserts the refusal leaves the replaceable tip standing. One new claim makes a reworked gate's mutation base the gate commit below the one being replaced; one has a reworked Phase 7 bump to the version the replaced commit carried. |
| What a refused stop does with the attempt it undid | Re-creates it from the replaced commit's own tree object before the refusal returns — `git commit-tree <hash>^{tree} -p <hash>^ -F <message>`, then `git update-ref HEAD <new>` — the same tree and the same record under a new hash | Leave the branch short and the record in the reflog (the first design); keep the record in a file under the target directory until the next stop; read it back out of the reflog | The undo runs before the check so that every later reading is correct, and the price is that a refusal lands on a branch whose tip the hook removed. Left there, the phase's attempt survives only in the reflog, and the next stop finds no replaceable tip and stacks after all. Re-creating it pays that back with two plumbing calls, and it is exact rather than approximate because it names the stored tree: `git commit -C` over the index would be exact only while nothing had staged, and `stage_and_commit` runs `git add` before it commits, so a failure between the two leaves this round's edits in the index and a restore reading it would fold them into the attempt. `<hash>^{tree}` asks for the bytes the replaced commit recorded whatever the index holds, and `commit-tree` with `update-ref` writes neither the index nor a working-tree file, so the restored commit *is* the attempt that was already there. This round's edits stay where the failure left them, so the next stop passes the nothing-to-commit test and runs the check again. Nothing here bounds a re-stopping agent and nothing needs to: that is Claude Code's eight-refusal cap, which this document already relies on. A file under the target directory is a second place for one fact, in a directory that is scratch and may be cleaned between attempts — the same objection the trailers row makes to reading tallies from disk. The reflog is not a record a tool should read: it is local, expiring, and absent from a fresh clone, so a rule resting on it holds only where nobody cleaned up. **Claim:** one new claim re-commits the undone attempt with the replaced commit's message before the stop refuses. |
| How `replaced_tip` and the tally leaves are skeletoned | Wired into `gate_commit` at Phase 3 with a body answering `Ok(None)` — never replace — so the branch stacks exactly as it does today until Phase 7; the three tally leaves are wired too, each with a stated wrong-but-compiling answer | Declared unwired with a `todo!()` body; a wired `todo!()`; wiring them only at Phase 5 | Five existing validations drive `hook_stop` past the nothing-to-commit test, so a wired `todo!()` in `replaced_tip` turns them red for a reason that is not theirs and Phase 3's own check could not pass; unwired, the item is dead code in a private module and the post-edit lint refuses it. `Ok(None)` is the wrong answer that compiles, and it is wrong *observably*: stacking is what the branch does today, so a validation that asserts one commit where two would stand is red against it, and the replaceability claims are red by assertion rather than by panic. The tally leaves are reached on **every** commit, not only a rework's — `the_stop_stages_exactly_the_staged_set` drives a Phase 3 stop all the way to a commit — so a `todo!()` in any of them fails a validation of a claim outside the red set. Each therefore lands at Phase 3 answering the first-attempt case: `Replaced::next_reworks` answers `0`; `tally::merged(replaced, this, agent)` answers this agent's tally; `tally::agents(replaced, this)` answers `[this]`; and `tally::trailers(tally, phase, agents, reworks)` is written **whole**, because it decides nothing — it renders the six lines from what it is handed. All three of the first take `Option<&Replaced>`, so the dispatch on "was anything replaced" lives in them and not in `commit_now`, which stays a work item. `undo_tip`, `restore_tip`, `on_the_trunk`, `trailer_of`, `tip`, `Replaced::of` and `tally::from_trailers` keep the ordinary `todo!()`: nothing green reaches them while the decision answers none. |
| Which halves are green at Phase 3, and what makes each claim red anyway | Every claim's validation carries the case `Ok(None)` gets wrong, beside the case it gets right | One validation per half; accepting that some claims arrive green and stating it at Phase 5 | The precedent is the `policy::gate_extra` row above: a skeleton that answers the common case leaves each claim half-green, and the claim is red only if its validation also drives the case the skeleton answers wrongly. Here the negative halves are all green against `Ok(None)` — a tip whose tag is another phase's, a tip with no `Lid-Rs-Phase` trailer, a tip already on `main` — because "the stop commits on top of it" is exactly what never replacing does. So each of those three validations asserts the *contrast*: the negative case leaving two commits, and beside it the positive case — tag, trailer, and off `main` — leaving one. The restore claim's validation asserts that after a refused stop the tip's hash has changed while its subject and trailers have not, which `Ok(None)` leaves untouched, so it is red on its own. The mutation-base claim is not driven through a stop at all: it is observed at leaf level, over a fixture whose history carries two `phase 7:` commits — `mutation_base` answers the newer, `undo_tip` runs, `mutation_base` answers the older — and it is red the ordinary way, because `undo_tip` is a `todo!()`. The two agent-count claims part the same way: `tally::merged` answering this agent's tally is already right for a resumed agent, which Phase 5 states as green early, and wrong for a fresh one, which is where its validation drives it; `tally::agents` answering `[this]` is right for a first attempt and wrong for a replacement by a second agent. And `TheTallyIsWrittenAsTrailers`, reworded, is red through the sixth line, but only because its validation is **stop-driven**: the renderer is written whole at Phase 3 and would satisfy any assertion made of it directly, so the validation drives a stop that replaces a tip and reads `Lid-Rs-Reworks: 1` off the commit that comes out, which `next_reworks` answering `0` cannot produce. A validation of the renderer alone would be green on arrival and the claim would enter the red set unreddenable. |
| What the replacement's trailers count | The agent's own tally when the replaced commit already names that agent; the replaced commit's counts added to this agent's when it does not | Always the agent's tally (restart); always the sum (accumulate); summing the on-disk tallies of every named agent | The two cases are both real and they differ: an interactive session resumes the worker, which keeps its `agent_id` and whose tally already counts the rejected attempt — measured on this slice's own branch, where a Phase 4 rework's trailers read 8 edits where the replaced commit read 5 — while the unattended workflow spawns a fresh worker for the rework, whose tally starts at zero. Restarting always loses the first agent's work; summing always double-counts the resumed one. Summing the on-disk tallies would be simpler, but they live under the target directory, which is scratch and may be cleaned between attempts, so a cleaned tree would silently undercount; the commit is the durable record and its own trailers are what the replacement reads back. **Claims:** one new claim keeps a resumed agent's counts from being added to a commit that already covers them; one adds a new agent's counts to the replaced commit's; one has `Lid-Rs-Agent` name every agent whose work the commit carries, in order and none twice. And one more for the reading itself, since the commit is the durable record only if a record it cannot read is a failure and not a zero: a trailer line the renderer could not have written — an absent `Lid-Rs-Agent` or count line, a count that is not a number — fails naming that line, cited by `tally::from_trailers`. Silently defaulting there would file the replaced commit's work under counts nobody kept. |
| Whether the attempt count survives the replacement | `Lid-Rs-Reworks`, the number of commits this one replaced | No new trailer — the agent list is the only record; a line in the commit body | The attempt count is a quality signal stacking made visible by accident, and a design that replaces commits loses it unless it is written down: a phase that took four tries would read exactly like one that took one. The agent list carries it only when the reworks were done by different agents, which is the workflow's case and not the interactive one. A body line is prose a reader must parse; a trailer is what `git log --format=%(trailers:key=…)` already answers with. **Claims:** one new claim makes the trailer one more than the value the replaced commit carries, and zero when that commit carries none or when nothing is replaced — the two zeroes are one rule, because a phase commit made before the rework change carries no such line and reading its absence as anything else would refuse to rework it. Cited by `Replaced::next_reworks`, which is the whole of that arithmetic, and by `Replaced::of`, which is where the absent line becomes the zero; the renderer is handed a number and decides nothing. And `TheTallyIsWrittenAsTrailers` is **reworded**, the sixth rewording this document records: it names five trailers where the block now has six, and says "the agent being the id the tally was kept under" where a replacement's `Lid-Rs-Agent` may name two. A rename with the retired name kept beside it as a `#[deprecated]` alias, by the rule this document states for a reworded claim; its one validator is named for it, so no mark is needed. The rewording is what puts it in the rework change's red set, which is therefore **fourteen** claims — the thirteen added here and this one renamed. |

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

The rework rows add a sixth rewording, recorded in the `Lid-Rs-Reworks` row:
`TheTallyIsWrittenAsTrailers` names five trailers where there are now six, and
says "the agent" where a replaced commit's record may name two; it is renamed
with the retired name kept beside it as a `#[deprecated]` alias, by the same
rule. That rename is what puts it in the red set beside the thirteen claims the
rework rows add, so the rework change's red set is fourteen. Three neighbouring claims are
*not* reworded, and the rows above say why:
`NothingChangedInTheEditingSetIsARefusal` describes the test it always
described — only its place in the stop's order moves, which is the undo's rule
and not its own, and its *validation* is rewritten to observe the new order and
the tip left standing, which is Phase 5's green work and not a rewording;
`TheBaseIsTheNewestGateCommitReachableFromHead` still says
which commit `gate_base` answers with for a given `HEAD`; and
`PhaseSevensStopBumpsThePatchVersionFromTheManifestAtHead` still reads the
manifest at `HEAD`. What the undo changes in the last two is what `HEAD` *is*,
which is the undo's claim to make.

A seventh and an eighth rewording live outside these rows entirely, in
"The gate runs detached" section: `ACommitBlockRunsThePhasesCheck` →
`ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll`, and
`TheRunningRecordCarriesTheBytesTheBumpWrote` →
`TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder` (Recount,
above). Both belong to that section's own red set, not to the fourteen
above — the two counts are disjoint by scope — so this section's five-plus-
sixth tally and that section's fourteen both stand unchanged. A ninth,
`APhaseSevenSubjectMustCarryTheBumpedVersion` →
`APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll` (Recount,
above), belongs to neither red set: the leaves it names predate this
section and are already validated against the order its corrected sentence
states, so nothing here needs to redden for it to land. A reader counting
every reworded claim in the whole document finds nine, split across two
red sets that never share a member, plus this one member of neither.

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
6. A floor that outgrows the watchdog: **closed**, not merely narrowed.
   "The gate runs detached" now detaches the *whole* plan — the floor,
   check 12, and `gate_extra` together, behind one job, one record, and one
   poll — which closes the concrete case this item recorded: three Phase 7
   workers killed inside the stop hook in one day, one of them making no
   edits, reading nothing, and running nothing. Detaching only check 12
   would have left a floor that grew past 600 seconds on its own — a phase
   landing enough claim groups at once, a slower test suite, a heavier
   `gate_extra` step — free to reach the same wall by a different step;
   that concern no longer applies, because no step of
   `phase-check 7`, however slow, ever runs inside a call either substrate
   is timing. What remains is observability, not correctness: a human or an
   orchestrating session polling mid-run sees only `Started` or `Running`,
   never which step is executing, and a hook that emitted progress while a
   step runs — so a live job reads as progress rather than silence to
   whatever is watching it — remains undone; it would now be a convenience
   for a session waiting on the result, not the fix this item was written
   to get. A rejected gate still runs a second full plan at the same
   wall-clock cost, but no longer inside a blocking call, so a rework no
   longer meets either substrate's limit on that account.
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
9. The join in `gate_commit` between its two sets — the nothing-to-commit test
   over the editing set, now asked before the undo, and the staged set as
   what a passing check stages — is observed at the two sets on a stopped
   tree, not as `hook_stop`'s verdict, for Phases 1–5: a stop that passes
   `checked` is the full gate on the fixture, which no unit test carries,
   so a `gate_commit` that read the staged set for both is a mutant no test
   kills and none cargo-mutants generates there. **Closed for Phase 7**:
   `gate_job::poll` is exactly the seam this item was waiting for — a
   fixture writes a `Running` or `Done` record under the key a Phase 7 stop
   will compute, and the stop is driven past a stubbed check without
   running the gate at all, which is how this section's own validations
   work (This section's own red set, above). What still stands is Phases
   1–5, where `checked` remains an unstubbed subprocess call and the join
   is observed only at leaf level.
10. An extra step has no catalog entry and no report. The pipeline's catalog
    fixes each command's inputs, report, and exit code; a workspace's own step
    is outside that vocabulary, so `cargo lid-rs catalog` neither names it nor
    writes a report for it, and its failure reaches a reader only as the
    check's output. A catalog entry per configured step is the shape that
    would close this, and it needs a name the project supplies.
11. Scoping a detached refusal to the failing diff rather than a fixed-size
    tail. `gate_job::bounded_output`'s last-8-KiB bound (A `Done(Err(..))`
    must not carry what the file already holds) is deliberately generic
    across every step of the plan; a bound tied to what actually changed —
    the newly surviving mutants alone, the specific denied lint's own block —
    rather than a fixed byte window is undesigned, and this section does not
    build it.

## References

- README [§4.5](https://bradvoth.github.io/lid-rs/spec/gates.html) (the gate this tool runs at Phase 7), [§8](https://bradvoth.github.io/lid-rs/spec/flow.html) (the phases and who owns each), constraint 2 (no source parsing).
- `docs/intent/skill/lld.md` (workspace) — the working-state convention, the evidence table that says why the agent must not run its own checks, and the interactive mode this slice changes.
- `cargo-lid-rs/src/sync/lld.md` — the strict mirror rule this slice extends to the agents, hooks, and workflow.
- `cargo-lid-rs/src/lld.md` — the registry dump and the item-path mapping `check_red` reuses.
- Claude Code hooks, subagents, and workflows (`code.claude.com/docs/en/hooks`, `/sub-agents`, `/workflows`) — hook input fields, block semantics, `additionalContext`, frontmatter hook syntax, and the `.claude/agents` and `.claude/workflows` locations.
