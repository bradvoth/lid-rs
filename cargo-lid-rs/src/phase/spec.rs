//! Claims for the `phase` slice (`src/phase/lld.md`, beside this file): a
//! phase is run by an agent that can only edit, and its commit is the check
//! passing.

use lid_rs::Spec;

// ---- phase-check: which phases have a check -----------------------------------

/// When `phase-check` is given a phase with no commit of its own — 0, 6, or
/// any number above 7 — it shall fail naming the phases that have a check.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesWithoutACommitHaveNoCheck;

/// When phase 1 is checked, the tool shall run the LLD's mechanical checks,
/// then rustdoc with broken intra-doc links denied, then the doctests, in
/// that order.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseOneChecksTheDocs;

/// When phase 2 is checked, the tool shall run `cargo check --all-targets`
/// and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoChecksTheClaimsBuild;

/// When the workspace builds with warnings and no error — a skeleton's
/// `todo!()`, a citation of a claim name this phase retired — phase 2's
/// check shall pass, so a claim the design turns out to need is committable
/// while the rest of the slice stands.
#[derive(Spec)]
#[lid(free)]
pub struct WarningsDoNotFailPhaseTwosCheck;

/// When phase 3 or 4 is checked, the tool shall run `cargo check
/// --all-targets` and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourCheckTheSkeletonTypeChecks;

/// When phase 7 is checked, the tool shall run the README §4.5 gate in its
/// order — check, clippy, doc, doctests, lib tests, one `cargo package`
/// naming every package that publishes, `sync --check`, `mutants`.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseSevenRunsTheGateInOrderPackagingEveryPublisherAtOnce;

/// When a step of a phase's sequence fails, the check shall stop there and
/// fail naming that step, running no later step.
#[derive(Spec)]
#[lid(free)]
pub struct ACheckStopsAtTheFirstFailingStep;

// ---- phase-check: what a step invokes -------------------------------------------
//
// A step's arguments are data — one function from a `Step` to the list
// `cargo_step` runs — so every rule about what a step invokes is a property
// of a returned list, which a validation asserts without running cargo. The
// two below are written in the controlled language and carry no mark.

/// When [`args_of`](crate::phase::args_of) is asked for a step that invokes
/// cargo, the argument list it answers with shall carry `--locked`, whichever
/// of the six cargo steps it is, so the check proves the workspace builds from
/// the lock file the commit carries rather than from one cargo rewrote on the
/// way past.
#[derive(Spec)]
pub struct EveryCargoStepIsLocked;

/// When [`args_of`](crate::phase::args_of) is asked for
/// [`Step::Doc`](crate::phase::Step::Doc), the argument list it answers with
/// shall carry `--document-private-items` beside `--no-deps`, so rustdoc reads
/// the private items a slice's claims mostly cite instead of skipping them.
#[derive(Spec)]
pub struct TheDocStepDocumentsPrivateItems;

// ---- phase-check 7: the gate's mutation base ------------------------------------
//
// The first of these is a sibling of `TheBaseIsTheNewestGateCommitReachableFromHead`
// and not a sharer of it: that claim is the red run's rule about which commit
// `gate_base` answers with, and this one is the gate's rule about what the
// mutation step does with the answer, so either could change without the
// other. All three are written in the controlled language and carry no mark.

/// When [`gate_base`](crate::phase::gate_base) answers with a gate commit at
/// the gate's mutation step, [`run_step`](crate::phase::run_step) shall run the
/// mutation engine with `--diff-base` naming that commit and no other base,
/// so check 12 is scoped to what this branch's phases changed since the slice
/// was last whole.
#[derive(Spec)]
pub struct TheGatesMutationStepDiffsAgainstTheGateCommit;

/// When [`mutation_base`](crate::phase::mutation_base) is asked on a history
/// holding no `phase 7:` commit reachable from `HEAD`, the base it answers
/// with shall be the commit `git merge-base main HEAD` names — the point the
/// branch was cut from — and never the trunk itself.
#[derive(Spec)]
pub struct WithoutAGateCommitTheMutationBaseIsTheMergeBaseWithMain;

/// When [`merge_base_with_main`](crate::phase::merge_base_with_main) finds no
/// `main` or no common ancestor of `main` and `HEAD`, the mutation step shall
/// fail naming `main` rather than run against a base that would mean
/// something else.
#[derive(Spec)]
pub struct NoMergeBaseWithMainFailsTheMutationStepNamingTheRef;

// ---- phase-check 7: the workspace's own gate steps ------------------------------
//
// README §4.5's list is the floor; a workspace declares the steps after it as
// `[workspace.metadata.lid_rs] gate_extra`, a list of commands each a list of
// strings, and `plan` carries them as `Step::Extra` after the mutation step.
// The three claims about the value — absent, not a list, an entry that is not
// a non-empty list of strings — are cited by `policy::gate_extra` and never by
// `Project::setting_node`: the raw door is a hand commit no phase can write, so
// a claim implemented only there would have no Phase 3 to leave it `todo!()`
// and no Phase 5 that could make it red. `args_of` answering nothing for an
// extra step is no claim of its own: it joins the arm the library steps and the
// red run already share. All seven are written in the controlled language and
// carry no mark.

/// When [`plan`](crate::phase::plan) is asked for phase 7 with the workspace's
/// configured `gate_extra` entries, the vector it answers with shall carry one
/// `Step::Extra` per entry in the order configured after
/// [`Step::Mutants`](crate::phase::Step::Mutants) — the last step of the floor
/// — so the workspace's own steps run last, against a tree the floor has
/// already accepted, and the floor's cheap and specific steps still fail first.
#[derive(Spec)]
pub struct TheExtraStepsFollowTheMutationStepInTheOrderConfigured;

/// When [`check`](crate::phase::check) builds its plan for a project whose
/// metadata names no `gate_extra` key, the extra steps it hands
/// [`plan`](crate::phase::plan) shall be the empty list — `policy::gate_extra`'s
/// answer for an absent key — so a workspace that configures nothing runs the
/// floor and only the floor, and no consumer's gate changes by upgrading.
#[derive(Spec)]
pub struct AnAbsentGateExtraIsTheEmptyListSoTheFloorAloneRuns;

/// When [`check`](crate::phase::check) reads the workspace's `gate_extra`, the
/// value `policy::gate_extra` parses shall be the `[workspace.metadata.lid_rs]`
/// table's entry as `cargo metadata` reports it — falling back to the
/// `[package.metadata.lid_rs]` table of the package whose manifest is the
/// workspace root's when the workspace table names no such key, as
/// `mutation_scope` is read — and never a manifest the tool parsed itself.
#[derive(Spec)]
pub struct GateExtraIsReadFromTheMetadataCargoReportsNeverFromAManifest;

/// When [`check`](crate::phase::check) builds its plan from a `gate_extra`
/// whose value is not a list — a string, a table, a boolean, a number — the
/// check shall fail naming `gate_extra` and what it found instead, a value that
/// is neither an absent key nor an entry, at whichever phase is running and
/// before any step runs.
#[derive(Spec)]
pub struct AGateExtraThatIsNotAListFailsTheCheckNamingWhatItFound;

/// When [`check`](crate::phase::check) builds its plan from a `gate_extra`
/// holding an entry that is not a non-empty list of strings — an empty list, a
/// bare string, a number among the words — the check shall fail naming
/// `gate_extra` and the entry it could not read, at whichever phase is running
/// and before any step runs, since a gate step that cannot be read is never a
/// gate step silently skipped.
#[derive(Spec)]
pub struct AGateExtraEntryThatIsNotANonEmptyListOfStringsFailsTheCheckNamingTheEntry;

/// When [`run_step`](crate::phase::run_step) reaches a `Step::Extra`,
/// `extra_step` shall run the entry's first word as a program with the rest as
/// its arguments, at the workspace root and through no shell, so no quoting
/// grammar, word splitting, or variable expansion stands between the manifest
/// and the process.
#[derive(Spec)]
pub struct AnExtraStepRunsItsEntryAsAProgramAtTheWorkspaceRootThroughNoShell;

/// When [`run_step`](crate::phase::run_step) runs an extra step whose program
/// exits non-zero or is not on the machine at all, the step shall fail naming
/// the entry and carrying the program's output, the two outcomes being one
/// answer because a step that could not run is not a step that passed.
#[derive(Spec)]
pub struct AnExtraStepThatExitsNonZeroOrCannotRunFailsTheGateNamingTheEntry;

// ---- phase-check 5: the red run ----------------------------------------------

/// When the slice's claims are identified, they shall be the registered
/// specs whose source file is the slice's claims file, wherever the layout
/// puts that file, read from the registry dump, never from Rust source.
#[derive(Spec)]
#[lid(free)]
pub struct ASlicesClaimsAreTheSpecsInItsSpecFile;

/// When `--slice` is absent, the slice shall be the current branch's name
/// with the `lld/` prefix removed; a detached `HEAD` or a branch not of that
/// form names no slice, which fails the phase 5 check naming the
/// convention; any other flag shall be rejected by name.
#[derive(Spec)]
#[lid(free)]
pub struct TheSliceComesFromTheBranchName;

/// When the branch's name after `lld/` contains `--`, the slice shall be the
/// part before the first `--` — `lld/phase--companion` is a change to the
/// slice `phase`, made on its own branch because the branch that built the
/// slice is kept and git admits no `lld/phase/companion` beside it.
#[derive(Spec)]
#[lid(free)]
pub struct AChangeBranchNamesItsSliceBeforeTheDoubleDash;

/// When the slice's spec file registers no claims, the phase 5 check shall
/// fail naming the slice, never pass vacuously.
#[derive(Spec)]
#[lid(free)]
pub struct ASliceWithNoClaimsFailsTheRedCheck;

/// When the red run locates its base, the base shall be the newest commit
/// reachable from `HEAD` whose subject starts `phase 7:`, whichever slice
/// that gate was for.
#[derive(Spec)]
#[lid(free)]
pub struct TheBaseIsTheNewestGateCommitReachableFromHead;

/// When a base exists, the red set shall be those of the slice's registered
/// claims whose `struct <Name>` line is an added line of `git diff <base>`
/// taken over the slice's claims file, wherever the layout puts that file.
#[derive(Spec)]
#[lid(free)]
pub struct TheRedSetIsTheClaimsAddedSinceTheBase;

/// When the slice's crate has a companion, the package that holds the
/// slice's claims — where the red run diffs the spec file and runs each
/// validation — shall be the companion, since a proc-macro crate registers
/// no claim and can cite none.
#[derive(Spec)]
#[lid(free)]
pub struct AProcMacroSlicesClaimsAreHeldByItsCompanion;

/// When no gate commit is reachable from `HEAD`, the red set shall be every
/// claim of the slice.
#[derive(Spec)]
#[lid(free)]
pub struct AFreshSliceHasEveryClaimInTheRedSet;

/// When a base exists and the red set is empty, the phase 5 check shall fail
/// naming the base, never pass vacuously.
#[derive(Spec)]
#[lid(free)]
pub struct AnEmptyRedSetAfterAGateFailsTheRedCheck;

/// When a claim in the red set has no validation edge, the phase 5 check
/// shall fail naming the claim.
#[derive(Spec)]
#[lid(free)]
pub struct EveryClaimNeedsAValidationBeforePhaseFivePasses;

/// When the red set's validations are run, each shall run alone as `cargo
/// test --lib -p <package> -- --exact <path>`, with the citing item's path
/// made libtest-relative, and its exit status shall be its outcome.
#[derive(Spec)]
#[lid(free)]
pub struct EachValidationRunsAloneByExactName;

/// When a validation of a red-set claim passes at phase 5, the check shall
/// fail naming the test; when every one fails, the check shall pass, whatever
/// the slice's other validations do.
#[derive(Spec)]
#[lid(free)]
pub struct AGreenValidationFailsTheRedCheck;

// ---- hook pre-tool: the path policy ------------------------------------------

/// When the policy locates the slice's crate, it shall be the workspace
/// package whose manifest directory holds the slice's document, wherever the
/// layout puts that document, found on the filesystem and never by parsing
/// Rust.
#[derive(Spec)]
#[lid(free)]
pub struct TheSlicesCrateIsTheOneHoldingItsLld;

/// When a Phase 2 agent edits or writes under the slice's own crate, the
/// target shall be the slice's claims file — the layout's answer for the
/// slice, placed in that crate — or `src/spec/mod.rs`, and nothing else in
/// that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoMayWriteOnlyTheOwnCratesSpecFiles;

// The four claims below — these two and their companion-seat twins — say what
// a phase may write under the slice's directory rather than naming the
// directory. Colocation puts the slice's document, its claims file, and every
// other file of its intent (a compile-time slice's acceptance) in that same
// directory, and a directory admits everything under it: naming it would hand
// Phases 3, 4, 5 and 7 the design document, the claims, and the human's
// acceptance. A phase's own artifact is Rust source, and of the Rust source
// beside the code only the claims file is another phase's, so that is the rule
// the directory entry was standing in for — and it refuses an intent file
// invented later without the policy learning its name.
//
// The two immediately below carry a second narrowing their companion-seat
// twins further down do not. All four say "under the slice's directory" and
// mean whatever directory the layout answers for the slice, which is the
// question to ask: a directory spelled from the slice's name is one a
// crate-root slice keeps no code in, and Phases 5 and 7, whose row carries no
// `src/lib.rs`, may then write nothing at all. But for a crate-root slice the
// layout's answer is its crate's `src`, and `cargo-lid-rs` holds eight module
// slices under its own — so asking the layout and stopping there would trade a
// lockout for the wider breach. What these two names add is therefore the
// subtraction and not the question: the slice's directory, and no other
// slice's directory in that crate. The companion twins keep their wording,
// a companion being a slice's presence in another crate — always a module
// directory, never a crate root, and so never over another slice's code.

/// When a Phase 3 or 4 agent edits or writes under the slice's own crate, the
/// target shall be the slice's module file, `src/lib.rs`, or a Rust source
/// file under the slice's directory and under no other slice's directory in
/// that crate — wherever the layout puts each of them — other than the slice's
/// claims file, and nothing else in that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourMayWriteTheOwnCratesSliceCodeAndLibraryRootNotItsIntentNorAnotherSlices;

/// When a Phase 5 or 7 agent edits or writes under the slice's own crate, the
/// target shall be the slice's module file or a Rust source file under the
/// slice's directory and under no other slice's directory in that crate —
/// wherever the layout puts each of them — other than the slice's claims file,
/// and nothing else in that crate.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesFiveAndSevenMayWriteOnlyTheOwnCratesSliceCodeNotItsIntentNorAnotherSlices;

/// When a target path contains a parent component or resolves outside both
/// the slice's crate and its companion, the policy shall refuse it before
/// any allowed set is consulted.
#[derive(Spec)]
#[lid(free)]
pub struct PathsOutsideTheSlicesCratesAreRefusedBeforeThePolicy;

// ---- hook pre-tool: a proc-macro crate's companion ---------------------------

/// When the slice's crate declares no `proc-macro` target, its companion
/// shall be none, whatever its package metadata carries.
#[derive(Spec)]
#[lid(free)]
pub struct AnOrdinaryCrateHasNoCompanion;

/// When the slice's crate is a proc-macro crate whose package metadata, as
/// `cargo metadata` reports it, names a workspace member under
/// `lid_rs.companion`, the companion shall be that member's manifest
/// directory — read from the metadata, never by parsing the manifest.
#[derive(Spec)]
#[lid(free)]
pub struct TheCompanionIsTheMemberTheProcMacroCratesMetadataNames;

/// When the slice's crate is a proc-macro crate whose package metadata names
/// no `lid_rs.companion`, the policy shall refuse every edit naming
/// `[package.metadata.lid_rs] companion`, since no phase of such a slice can
/// produce a claim.
#[derive(Spec)]
#[lid(free)]
pub struct AProcMacroCrateNamingNoCompanionRefusesEveryEdit;

/// When the named companion is itself a proc-macro crate, the policy shall
/// refuse every edit naming the key, as it does for a missing companion.
#[derive(Spec)]
#[lid(free)]
pub struct ACompanionThatIsAProcMacroCrateRefusesEveryEdit;

/// When the named companion is not a workspace member, the policy shall
/// refuse every edit naming the key, as it does for a missing companion.
#[derive(Spec)]
#[lid(free)]
pub struct ACompanionThatIsNotAWorkspaceMemberRefusesEveryEdit;

/// When the slice's crate has a companion and the target path resolves under
/// the companion, the policy shall judge it by the phase's companion table,
/// relative to the companion — the allowed set is the union of the two
/// tables, each relative to its crate.
#[derive(Spec)]
#[lid(free)]
pub struct APathUnderTheCompanionIsJudgedByTheCompanionsTable;

// The claim below is corrected in place and keeps its name and its mark. Its
// old sentence said the companion seat's claims file was "the same answer the
// layout gives for the slice, placed in the companion", which
// `TheCompanionSeatsClaimsFileIsItsModuleDirectorysSpec` refuses — but that
// behaviour is already delivered and gated under that claim, so a rename
// would put a claim into the red set whose validator is green on arrival,
// which the red check refuses and no phase could make red. A text corrected to
// match behaviour another claim already gates is a documentation defect and
// no rename. The one thing left is Phase 5's: its validator asserts the raw
// claims path through `allowed_paths`, the own-crate answer this claim
// refuses, and is rewritten to reach the row through
// `workspace_paths`/`allowed`, as work on a claim outside the red set that
// must stay green.

/// When a Phase 2 agent edits or writes under the companion, the target
/// shall be the `spec.rs` in the companion's module directory named for the
/// slice — the companion seat's claims file, whatever form the slice's own
/// crate takes, and never the layout's own-crate answer placed under the
/// companion — or `src/spec/mod.rs` there, and nothing else in the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhaseTwoMayWriteOnlyTheCompanionsSpecFiles;

/// When [`seat_claims`](crate::phase::policy::seat_claims) is asked for the
/// companion seat, the claims file it answers with shall be the `spec.rs` in
/// the companion's module directory named for the slice, whatever form the
/// slice's own crate takes, and never the layout's own-crate answer placed
/// under the companion.
#[derive(Spec)]
pub struct TheCompanionSeatsClaimsFileIsItsModuleDirectorysSpec;

/// When a Phase 3 or 4 agent edits or writes under the companion, the target
/// shall be the slice's module file there, `src/lib.rs` — where the
/// hand-authored edges citing the slice's claims go — or a Rust source file
/// under the slice's directory other than its claims file, which the
/// companion is the crate that holds, and nothing else in the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesThreeAndFourMayWriteTheCompanionsSliceCodeAndLibraryRootNotItsClaims;

/// When a Phase 5 or 7 agent edits or writes under the companion, the target
/// shall be the slice's module file there, a Rust source file under the
/// slice's directory other than its claims file, or a file under `tests/ui/`
/// — the one place a compile-failure fixture can live — and nothing else in
/// the companion.
#[derive(Spec)]
#[lid(free)]
pub struct PhasesFiveAndSevenMayWriteTheCompanionsSliceCodeAndUiFixturesNotItsClaims;

/// When an edit or write is refused, the reason shall quote the
/// `discipline.md` row for that moment and name what the phase may do
/// instead: proceed within its allowed paths, or end with the decision.
#[derive(Spec)]
#[lid(free)]
pub struct ARefusedEditQuotesTheDisciplineRow;

/// When a phase agent calls a tool that does not edit — Read, Grep, Glob,
/// LSP, or the workflow's `StructuredOutput` — the hook shall allow it
/// whatever the path.
#[derive(Spec)]
#[lid(free)]
pub struct ReadsAreNeverRefused;

/// When any tool call reaches the hook, the agent's tally shall count it by
/// kind — edit, observation, command — and count every refusal, under the
/// agent's id.
#[derive(Spec)]
#[lid(free)]
pub struct EveryToolCallIsTallied;

// ---- hook post-edit: clippy after every edit ---------------------------------

/// When an Edit or Write completes, the hook shall run clippy on the
/// workspace with warnings denied and hand its output, or "clean", back as
/// additional context, refusing nothing.
#[derive(Spec)]
#[lid(free)]
pub struct EveryEditIsFollowedByClippy;

// ---- hook stop: the check, then the commit -----------------------------------

/// When the agent's final message carries neither a `commit` block nor a
/// `stop` block, or carries both, the stop shall be refused with the
/// format.
#[derive(Spec)]
#[lid(free)]
pub struct AFinalMessageCarriesExactlyOneEnding;

/// When the final message carries a `stop` block, the hook shall commit
/// nothing and allow the stop.
#[derive(Spec)]
#[lid(free)]
pub struct AStopBlockEndsThePhaseWithoutACommit;

/// When a `commit` block's subject does not begin with the agent's own
/// `phase <n>:` tag, the stop shall be refused naming the expected tag.
#[derive(Spec)]
#[lid(free)]
pub struct ACommitSubjectMustCarryThisPhasesTag;

// The claim below is the seventh rewording this file records, and the one that
// belongs to the detached gate's red set rather than to the rework rows' —
// the two counts are disjoint by scope, so a reader counting every rewording
// here finds seven. Its old sentence said the hook "shall run the phase's
// check", which stops being true the moment a Phase 7 stop can instead start,
// or read the outcome of, a check another process ran. A rename with the
// retired name kept beside it as a `#[deprecated]` alias, by the rule the
// earlier six follow; the mark stays because its one validator keeps the
// retired name, and check 14 exempts a validator's name solely while every
// claim it cites is marked free.

/// When the final message carries a `commit` block with this phase's tag,
/// the hook shall settle that phase's check before anything is committed —
/// run inline by `checked` at every phase but the seventh, and at Phase 7
/// read from `gate_job::poll`'s answer for a job another process runs — and a
/// check that fails shall refuse the stop.
#[derive(Spec)]
#[lid(free)]
pub struct ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll;

/// The name [`ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll`]
/// carried while every phase's stop ran its check inline. The alias registers
/// no claim, so the graph sees only the claim it points at; every citation of
/// this name warns with its replacement, and those citations are the later
/// phases' work list.
#[deprecated = "replaced by ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll"]
pub type ACommitBlockRunsThePhasesCheck = ACommitBlockSettlesThePhasesCheckDirectlyOrThroughThePoll;

/// When a check fails, the refusal's reason shall be, in order, the failing
/// step's output, the `gates.md` row for the check that fired, and what the
/// phase's policy permits.
#[derive(Spec)]
#[lid(free)]
pub struct ARefusalCarriesTheOutputTheRuleAndThePermittedMoves;

/// When a clippy lint in the failing output is one the gate relies on, it
/// shall name its check — `cognitive_complexity` 7,
/// `fn_params_excessive_bools` 8, `too_many_lines` 9,
/// `wildcard_enum_match_arm` 6, `missing_docs` 3 — and a red-run failure
/// shall name the Phase 5 rule, a survivor check 12.
#[derive(Spec)]
#[lid(free)]
pub struct AFailingOutputNamesItsCheck;

/// When the synced artifacts differ from the dependency's, before or after
/// the check, the stop shall be refused naming them, and nothing shall be
/// committed.
#[derive(Spec)]
#[lid(free)]
pub struct SyncedArtifactsMustMatchAtTheStop;

// ---- hook stop: the staged set and the editing set --------------------------
//
// Two sets, because the writers differ. The *editing* set is the phase's
// allowed paths of both seats — what the agent could have written. The
// *staged* set is that plus, at Phase 7 only, the two workspace-root files
// the hook's own bump wrote. The stop stages the staged set and filters
// integrity against it; the editing verdict, the permitted moves a refusal
// quotes, and the nothing-to-commit test keep reading the editing set, so a
// bare version bump is never a commit and the refusal never offers
// `Cargo.toml` as somewhere to fix a gate.
//
// Five claims below are rewordings, each a rename with the retired name kept
// beside it as a `#[deprecated]` alias: the alias registers no claim, so the
// graph sees only the claim it points at, and every citation of the old name
// warns with its replacement — the later phases' work list. Four of the five
// keep `#[lid(free)]` for one reason only: a validator of each is named for
// something other than its claim, and check 14 exempts a validator's name
// solely while every claim it cites is marked free. The alias resolves such a
// citation to the reworded claim, so holding it would fail `cargo check` in a
// file this phase may not edit. The reworded sentences are nevertheless
// written in the controlled language, so the mark is dropped the moment those
// validators are renamed for the new names. The fifth is held: its one
// validator is named for it.

/// When [`outside_policy_clean`](crate::phase::integrity::outside_policy_clean)
/// finds a path outside the phase's staged set changed after the check, the
/// stop shall be refused naming that path, and nothing committed.
#[derive(Spec)]
#[lid(free)]
pub struct ChangesOutsideTheStagedSetRefuseTheStop;

/// The name [`ChangesOutsideTheStagedSetRefuseTheStop`] carried while the set
/// the integrity check filtered against was the phase's allowed paths alone.
/// The alias registers no claim, so the graph sees only the claim it points
/// at; every citation of this name warns with its replacement, and those
/// citations are the later phases' work list.
#[deprecated = "replaced by ChangesOutsideTheStagedSetRefuseTheStop"]
pub type ChangesOutsideThePolicyRefuseTheStop = ChangesOutsideTheStagedSetRefuseTheStop;

/// When [`outside_policy_clean`](crate::phase::integrity::outside_policy_clean)
/// runs for a slice whose crate has a companion, the `git status` it reads
/// shall be filtered against the staged set of both crates — each seat's
/// table relative to its crate — so a change under the companion's table is
/// the phase's own and any other is named.
#[derive(Spec)]
pub struct IntegrityFiltersAgainstBothCratesStagedPaths;

/// The name [`IntegrityFiltersAgainstBothCratesStagedPaths`] carried while
/// the set the integrity check filtered against was the phase's allowed
/// paths alone. The alias registers no claim, so the graph sees only the
/// claim it points at; every citation of this name warns with its
/// replacement, and those citations are the later phases' work list.
#[deprecated = "replaced by IntegrityFiltersAgainstBothCratesStagedPaths"]
pub type IntegrityFiltersAgainstBothCratesAllowedPaths = IntegrityFiltersAgainstBothCratesStagedPaths;

/// When the check and both integrity checks pass at
/// [`hook_stop`](crate::phase::hook_stop), the commit it makes shall carry
/// exactly the changes within the staged set — the phase's allowed paths
/// plus what the hook itself wrote — under the block's message.
#[derive(Spec)]
#[lid(free)]
pub struct TheStopStagesExactlyTheStagedSet;

/// The name [`TheStopStagesExactlyTheStagedSet`] carried while what the stop
/// staged was the phase's allowed paths alone. The alias registers no claim,
/// so the graph sees only the claim it points at; every citation of this
/// name warns with its replacement, and those citations are the later
/// phases' work list.
#[deprecated = "replaced by TheStopStagesExactlyTheStagedSet"]
pub type OnlyThePoliciesPathsAreStaged = TheStopStagesExactlyTheStagedSet;

/// When [`staged_paths`](crate::phase::policy::staged_paths) is asked for a
/// slice whose crate has a companion, the set it answers with shall carry the
/// phase's allowed paths of both crates — each seat's table relative to its
/// crate — beside what the hook wrote, and nothing else.
#[derive(Spec)]
#[lid(free)]
pub struct TheStopStagesBothCratesStagedPaths;

/// The name [`TheStopStagesBothCratesStagedPaths`] carried while what the
/// stop staged was the phase's allowed paths alone. The alias registers no
/// claim, so the graph sees only the claim it points at; every citation of
/// this name warns with its replacement, and those citations are the later
/// phases' work list.
#[deprecated = "replaced by TheStopStagesBothCratesStagedPaths"]
pub type TheStopStagesBothCratesAllowedPaths = TheStopStagesBothCratesStagedPaths;

/// When [`hook_stop`](crate::phase::hook_stop) finds no change under the
/// phase's editing set — its allowed paths, and nothing the bump wrote — it
/// shall refuse the stop as having nothing to commit, so a Phase 7 whose
/// agent changed nothing is refused though the bump dirtied two files.
#[derive(Spec)]
#[lid(free)]
pub struct NothingChangedInTheEditingSetIsARefusal;

/// The name [`NothingChangedInTheEditingSetIsARefusal`] carried while the
/// phase's allowed paths were the only set the stop knew. The alias
/// registers no claim, so the graph sees only the claim it points at; every
/// citation of this name warns with its replacement, and those citations are
/// the later phases' work list.
#[deprecated = "replaced by NothingChangedInTheEditingSetIsARefusal"]
pub type NothingToCommitIsARefusal = NothingChangedInTheEditingSetIsARefusal;

/// When [`staged_paths`](crate::phase::policy::staged_paths) is asked for
/// Phase 7, the set it answers with shall carry the workspace root's
/// `Cargo.toml` and `Cargo.lock` beside the phase's allowed paths, and at
/// every other phase the allowed paths alone.
#[derive(Spec)]
pub struct TheBumpsRootFilesJoinTheStagedSetAtPhaseSevenOnly;

/// When [`bumped_files_untouched`](crate::phase::integrity::bumped_files_untouched)
/// finds `Cargo.toml` or `Cargo.lock` at the workspace root differing after
/// the check from what the bump wrote, the stop shall be refused naming that
/// file, and nothing committed.
#[derive(Spec)]
pub struct TheBumpedRootFilesMustStillEqualWhatTheBumpWrote;

// ---- hook stop 7: the version bump --------------------------------------------
//
// A `phase 7:` commit is where a slice becomes a release candidate, and
// packaging a version a registry already holds is a publish that cannot
// happen. So the workspace version is raised from the manifest at `HEAD` —
// which is what makes a refused stop's second bump answer the same version.
// *Where* that bump runs in a stop's order is `gate_job::start`'s business
// and no claim here: it is the first of that leaf's five moves, one statement
// ahead of the spawn, so a clause about when it runs would be provable only
// by driving a stop through to a real child. What the bump writes needs no
// such drive, and that is what the first claim below says. Every claim here
// is written in the controlled language and carries no mark. The
// subject-version refusal is a sibling of
// `ACommitSubjectMustCarryThisPhasesTag`, which is about the tag and says
// nothing about a version.

/// When [`bump_workspace_version`](crate::phase::bump_workspace_version)
/// runs, the version it writes to the working tree shall be one patch level
/// above the workspace `version` the root manifest holds as committed at
/// `HEAD`, so a second bump before any commit writes the same version again.
#[derive(Spec)]
pub struct PhaseSevensStopBumpsThePatchVersionFromTheManifestAtHead;

/// When [`version_line`](crate::phase::version_line) scans the root
/// manifest, the line it answers with shall be the first line beginning
/// `version = "` after the workspace package table's header and before the
/// next line that opens a table, so a later table's `version` is never the
/// one patched.
#[derive(Spec)]
pub struct TheVersionLineIsTheFirstUnderWorkspacePackageBeforeTheNextTable;

/// When [`bump_patch_version`](crate::phase::bump_patch_version) is given a
/// manifest with no workspace package header or with no `version = "` line
/// under that header before the next table or with a version that is not
/// three dot-separated numbers, it shall fail naming what it looked for and
/// patch no other line.
#[derive(Spec)]
pub struct AManifestTheBumpCannotReadFailsNamingWhatItLookedFor;

/// When [`bump_workspace_version`](crate::phase::bump_workspace_version) has
/// written the raised manifest, it shall run `cargo update --workspace
/// --offline` and nothing wider, so `Cargo.lock` follows the members' new
/// version while no third-party entry moves.
#[derive(Spec)]
pub struct TheBumpUpdatesTheLockForTheMembersAloneOffline;

// The claim below is the ninth rewording this file records, and the only one
// belonging to neither red set. Its old sentence ended "after the bump and
// before the check runs" — an order the code reverses rather than narrows,
// since `gate_commit` asks this check before the poll, and so before the
// bump `gate_job::start` runs — which is what makes it a rename with the
// retired name kept beside it as a `#[deprecated]` alias, by the rule the
// earlier eight follow, rather than a clause dropped in place. No reddening
// case is owed for it: `subject_version_matches` and
// `version_the_bump_will_write` both predate the detached gate, are
// implemented, and are already validated against the order the sentence now
// states, so no skeleton stands under them to answer it wrongly.

/// When the version [`subject_version`](crate::phase::subject_version) reads
/// from a Phase 7 `commit` block's subject is not the one
/// `version_the_bump_will_write` computes from the manifest at `HEAD` without
/// running the bump, the stop shall be refused naming both versions — before
/// the poll, and so before any job begins.
#[derive(Spec)]
pub struct APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll;

/// The name [`APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll`]
/// carried while the subject was compared after the bump had already run. The
/// alias registers no claim, so the graph sees only the claim it points at;
/// every citation of this name warns with its replacement, and those
/// citations are the later phases' work list.
#[deprecated = "replaced by APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll"]
pub type APhaseSevenSubjectMustCarryTheBumpedVersion =
    APhaseSevenSubjectIsHeldToTheVersionTheBumpWillWriteBeforeThePoll;

// The claim below is the sixth rewording this file records — a rename with the
// retired name kept beside it as a `#[deprecated]` alias, by the rule the
// earlier five follow. The old sentence named five trailers where the block
// now has six, and "the agent" where a replacement's `Lid-Rs-Agent` may name
// two. Its one validator is named for it, so no mark is needed; the sentence
// is written in the controlled language. Its validation is stop-driven, since
// the renderer decides nothing and would satisfy any assertion made of it
// alone: a stop that replaces a tip and reads `Lid-Rs-Reworks: 1` off the
// commit that comes out is what makes it red.

/// When [`hook_stop`](crate::phase::hook_stop) commits a phase, its message
/// shall end with the six trailers `tally::trailers` renders — `Lid-Rs-Phase`,
/// `Lid-Rs-Agent`, `Lid-Rs-Tools`, `Lid-Rs-Checks`, `Lid-Rs-Refusals`, and
/// `Lid-Rs-Reworks` — `Lid-Rs-Agent` naming every agent whose work the commit
/// carries with two or more separated by `", "`, and `Lid-Rs-Reworks` the
/// number of commits this one replaced, so a phase made twice still names the
/// whole record of how it was made.
#[derive(Spec)]
pub struct APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks;

/// The name [`APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks`]
/// carried while the block had five trailers and one agent. The alias
/// registers no claim, so the graph sees only the claim it points at; every
/// citation of this name warns with its replacement, and those citations are
/// the later phases' work list.
#[deprecated = "replaced by APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks"]
pub type TheTallyIsWrittenAsTrailers = APhaseCommitEndsWithTheSixTrailersNamingEveryAgentAndTheReworks;

// ---- hook stop: a rework replaces its phase's commit ----------------------------
//
// A phase has one commit, whatever it took to make it: a stop over a tip this
// phase made and a reviewer rejected replaces that tip rather than following
// it. `tip` reads the branch tip once and `Replaced::of` turns the reading into
// the record a replacement needs; `replaced_tip` holds the decision alone, over
// three conditions; `undo_tip` runs before the bump and the check, so every
// reading after it sees the commit below the replaced one without being told a
// rework is in flight; `restore_tip` puts the attempt back when anything after
// the undo refuses; and the three tally leaves merge the two rounds' record.
// The three conditions are three claims and not one, because they fail
// independently — a `phase 7:` commit a human made when the watchdog killed a
// gate carries the tag and no trailer, and swallowing it is the worst thing
// this decision could do. "After the nothing-to-commit test" is no claim of
// its own: it is observed through `NothingChangedInTheEditingSetIsARefusal`,
// whose validation asserts the refusal leaves the replaceable tip standing.
// Every claim here is written in the controlled language and carries no mark.

/// When [`gate_commit`](crate::phase::gate_commit) runs over a branch tip that
/// `replaced_tip` answers as replaceable, the commit the stop makes shall be
/// that tip's replacement and not a second commit stacked after it — the
/// rejected attempt gone from the branch and its work carried whole — so the
/// branch holds one commit for the phase whatever it took to make it.
#[derive(Spec)]
pub struct AReworkedPhaseReplacesItsCommitRatherThanStackingASecond;

/// When [`gate_commit`](crate::phase::gate_commit) asks `replaced_tip` whether
/// the branch tip is this phase's attempt to replace, the tip it answers with
/// shall carry this phase's `phase <n>:` tag in its subject — `tag_of`
/// answering `Tag::Checked` of this phase — a tip tagged for another phase
/// being none, left where it is and committed on top of as a first attempt's
/// tip is.
#[derive(Spec)]
pub struct AReplacedTipsSubjectTagNamesThisPhase;

/// When [`gate_commit`](crate::phase::gate_commit) asks `replaced_tip` whether
/// the branch tip is this phase's attempt to replace, the tip it answers with
/// shall carry a `Lid-Rs-Phase` trailer naming this phase — `trailer_of`'s
/// answer for the body, the hook's own signature that no agent holds the git
/// to write — so a commit made by hand under the tag and carrying no trailer
/// is left standing and never swallowed.
#[derive(Spec)]
pub struct AReplacedTipCarriesTheHooksPhaseTrailerForThisPhase;

/// When [`gate_commit`](crate::phase::gate_commit) asks `replaced_tip` about a
/// branch tip that `on_the_trunk` finds reachable from `main`, the answer
/// shall be none whatever tag and trailer the tip carries — a commit the trunk
/// holds is never this branch's to replace — while a repository with no
/// `main` reaches nothing from it and leaves the first two conditions to
/// decide.
#[derive(Spec)]
pub struct ATipReachableFromMainIsNeverReplaced;

/// When [`gate_commit`](crate::phase::gate_commit) holds a replaceable tip, it
/// shall run `undo_tip` — `git reset --soft HEAD~1`, the commit gone and every
/// change it carried kept in the index — before the Phase 7 bump and before
/// the check, so the bump and every reading after it see the commit below the
/// one being replaced without being told a rework is in flight.
#[derive(Spec)]
pub struct TheReplacedTipIsUndoneBeforeTheBumpAndTheCheck;

/// When `undo_tip` has undone a replaceable `phase 7:` tip and
/// [`mutation_base`](crate::phase::mutation_base) is asked for the gate's diff
/// base, the base it answers with shall be the newest gate commit reachable
/// from the `HEAD` that remains — the gate below the one being replaced, the
/// base the rejected attempt ran against — so a reworked gate's mutation step
/// covers everything the slice changed and not the difference between two
/// attempts at one phase.
#[derive(Spec)]
pub struct AReworkedGateDiffsAgainstTheGateBelowTheReplacedCommit;

/// When `undo_tip` has undone a replaceable `phase 7:` tip and
/// [`bump_workspace_version`](crate::phase::bump_workspace_version) then runs,
/// the version it writes shall be the one the replaced commit carried — one
/// patch level above the manifest at the `HEAD` that remains, which is the
/// commit below the release — so a rejection spends no version number and the
/// subject the agent writes is the same across attempts.
#[derive(Spec)]
pub struct AReworkedPhaseSevenBumpsToTheVersionTheReplacedCommitCarried;

/// When `tally::merged` is handed a replaced commit whose `Lid-Rs-Agent`
/// already names this agent and this agent's own
/// [`Tally`](crate::phase::tally::Tally), the tally it answers with shall be
/// this agent's alone — a resumed worker keeps its id and its tally counted
/// the rejected attempt too — so none of the replaced commit's counts is added
/// to a record that already covers it and no call is counted twice.
#[derive(Spec)]
pub struct AResumedAgentsCountsAreNotAddedToTheCommitThatAlreadyCoversThem;

/// When `tally::merged` is handed a replaced commit whose `Lid-Rs-Agent` does
/// not name this agent and this agent's own
/// [`Tally`](crate::phase::tally::Tally), the tally it answers with shall be
/// the replaced commit's counts — read back from its trailers by
/// `tally::from_trailers` — added count for count to this agent's, since a
/// fresh worker's tally started at zero and the commit is the durable record
/// of the attempt before it.
#[derive(Spec)]
pub struct AFreshAgentsCountsAreAddedToTheReplacedCommits;

/// When `tally::agents` is asked which agents the commit
/// [`gate_commit`](crate::phase::gate_commit) is about to make carries, the
/// list it answers with shall be the replaced commit's agents in the order
/// they worked with this agent appended when it is not already among them —
/// every agent whose work the commit carries, in order, none of them twice,
/// and this agent alone when nothing was replaced — which is the
/// `Lid-Rs-Agent` line the trailers render.
#[derive(Spec)]
pub struct LidRsAgentNamesEveryAgentThatMadeTheCommitInOrderNoneTwice;

/// When `Replaced::next_reworks` is asked for the `Lid-Rs-Reworks` the commit
/// [`gate_commit`](crate::phase::gate_commit) is about to make carries, the
/// number it answers with shall be one more than the replaced commit's own
/// value — which `Replaced::of` reads as zero when that commit carries no such
/// line, as every phase commit made before the trailer existed does — and zero
/// when nothing is replaced, so a phase that took four tries never reads like
/// one that took one.
#[derive(Spec)]
pub struct LidRsReworksIsOneMoreThanTheReplacedCommitsValueAndZeroWhenNothingIsReplaced;

/// When a failure after the undo turns the stop
/// [`gate_commit`](crate::phase::gate_commit) is making into a refusal, it
/// shall run `restore_tip` before the refusal returns — for a failing check, a
/// failing integrity pass, and a staging that refuses alike — re-creating the
/// undone attempt from the replaced commit's own tree object with the subject
/// and body the `Tip` holds, `git commit-tree <hash>^{tree} -p <hash>^ -F
/// <message>` then `git update-ref HEAD <new>`, so the branch tip is the same
/// tree and the same record under a new hash, this round's edits stay where
/// the failure left them, and the next stop finds the attempt replaceable
/// rather than stacking after all.
#[derive(Spec)]
pub struct ARefusedStopRestoresTheUndoneAttemptFromTheReplacedCommitsTree;

/// When `tally::from_trailers` reads a commit body holding a trailer line
/// [`tally::trailers`](crate::phase::tally::trailers) could not have written,
/// it shall fail naming that line — an absent `Lid-Rs-Agent` or count line, a
/// count that is not a number — and never read it as zero, so the replaced
/// commit's work is never filed under counts nobody kept.
#[derive(Spec)]
pub struct ATrailerLineTheRendererCouldNotHaveWrittenFailsNamingTheLine;

// ---- the gate runs detached: one job, one record, one poll, one key ------------
//
// No execution substrate holds a call open for the length of the Phase 7 plan:
// Claude Code ends a subagent that makes no stream progress for 600 seconds,
// the canopy client's tool credential lasts ten minutes with no renewal, and
// `phase-check 7` end to end is 1087 seconds here. So the whole plan — every
// step `plan` returns, in the order it returns them, `gate_extra` included —
// runs as one detached job, and a Phase 7 stop either reads a finished result
// under a key it computes fresh and commits exactly as it always did, or
// starts the job and ends `Pending`, a third ending that commits nothing and
// refuses nothing. The claims below are that job's key and fingerprint, its
// record and its scratch paths, its own runner, the decisions a poll makes
// over what the record says, the read-only door an orchestrating session
// reads instead of a worker's word about the gate, and the bounded stand-in a
// failing job's refusal carries in place of a capture that once reached
// 1.3 MB. Every one is written in the controlled language and carries no mark.
//
// Three things this design records are stated limits and no claim of any of
// these: a bump left behind by a job nobody polls again, a published-version
// collision the key does not close, and a result file the code the check
// executes could write for itself.

/// When [`gate_commit`](crate::phase::gate_commit) reaches the check at a
/// Phase 7 stop, the answer it acts on shall be `gate_job::poll`'s over the
/// whole plan — asked before `after_undo` is ever called — while at every
/// other phase it goes on to `after_undo` and the inline `checked` call
/// unchanged.
#[derive(Spec)]
pub struct APhaseSevenStopActsOnThePollsAnswerWhileEveryOtherPhaseChecksInline;

/// When [`gate_commit`](crate::phase::gate_commit) reads a settled `Done` from
/// the poll at a Phase 7 stop, the tally that stop leaves shall carry one
/// `Event::StopCheck` for the verdict it read and none for a stop whose poll
/// answered `Started` or `Running`, so `Lid-Rs-Checks` names how many times a
/// phase's check ran to a finished verdict.
#[derive(Spec)]
pub struct TheStopCheckEventIsTalliedAtThePollThatSettlesAndAtNoOther;

/// When [`poll`](crate::phase::gate_job::poll) answers `Started` or `Running`
/// at a Phase 7 stop, [`gate_commit`](crate::phase::gate_commit) shall end
/// with `GateCommit::Pending` carrying that key — the tip it undid put back,
/// nothing staged and nothing committed — so a job still running leaves the
/// branch as the stop found it.
#[derive(Spec)]
pub struct AStartedOrRunningPollEndsTheStopPendingWithTheTipPutBack;

/// When [`commit_phase`](crate::phase::commit_phase) is handed a `Pending`
/// ending, the verdict it answers with shall be `HookVerdict::Allow` — the
/// value a `Committed` ending produces — with no numbered decisions, no
/// proposed message, no refusal reason, and neither `Event::StopCheck` nor
/// `Event::StopRefusal` tallied for it.
#[derive(Spec)]
pub struct APendingEndingIsAllowedAndTalliedAsNeitherACheckNorARefusal;

/// When [`key`](crate::phase::gate_job::key) computes the key a poll holds a
/// record to, the value it answers with shall carry the checkout's canonical
/// path, the branch tip's hash, `mutation_base`'s answer, the slice's name and
/// `fingerprint`'s answer, so one attempt's result is told from another's.
#[derive(Spec)]
pub struct TheKeyCarriesTheCheckoutTheTipTheBaseTheSliceAndTheFingerprint;

/// When [`act`](crate::phase::gate_job::act) is handed a record whose key
/// differs from the fresh key in any one part — the checkout, the tip, the
/// base, the slice or the fingerprint — the outcome written under that other
/// key shall be no `Done` of this poll's, so an edit made after a job started
/// is never settled by the result that job was started for.
#[derive(Spec)]
pub struct AKeyDifferingInAnyOnePartIsNeverThisPollsDone;

/// When [`key`](crate::phase::gate_job::key) fingerprints the phase's editing
/// set, the entries it hands `fingerprint` shall be
/// `policy::workspace_paths(project, phase, crates)`'s answer — which is why
/// `key` takes the phase and the crates beside the project and the slice — so
/// two phases whose editing sets differ hold one tree to two keys.
#[derive(Spec)]
pub struct TheKeyFingerprintsThePhasesOwnEditingSetTakingThePhaseAndTheCrates;

/// When [`fingerprint`](crate::phase::gate_job::fingerprint) walks the entries
/// `policy::workspace_paths` admits, the value it answers with shall be a hash
/// of every regular file's path relative to the project root and that file's
/// bytes — a directory walked recursively and a file read once — sorted by
/// path and concatenated.
#[derive(Spec)]
pub struct TheFingerprintHashesEveryFileUnderTheAdmittedEntriesSortedByPath;

/// When a file git does not track is written under a directory
/// [`fingerprint`](crate::phase::gate_job::fingerprint) admits, the value it
/// answers with shall be other than the one it answered before that file
/// existed, so a leaf the phase wrote and has not staged is part of the key.
#[derive(Spec)]
pub struct AnUntrackedFileUnderAnAdmittedDirectoryChangesTheFingerprint;

/// When [`record_path`](crate::phase::gate_job::record_path) is asked where a
/// checkout keeps the running record for a slice, the path it answers with
/// shall be `<target>/lid-rs/gate/<the checkout's canonicalised path,
/// hashed>/<slice>.json` — nested by checkout as the tally's own
/// `<target>/lid-rs/agents/` is nested by agent.
#[derive(Spec)]
pub struct TheRunningRecordIsNestedUnderTheCheckoutAndNamedForTheSlice;

/// When two checkouts of one slice ask [`poll`](crate::phase::gate_job::poll)
/// at the same time, the record each of them reads and writes shall be its own
/// checkout's alone, neither overwriting the other's `Running` entry and
/// neither left unable to read back the result its own job wrote.
#[derive(Spec)]
pub struct TwoCheckoutsOfOneSliceReadAndWriteOnlyTheirOwnRecord;

// The claim below is the eighth rewording this file records, and the second
// of the detached gate's own — the seventh being the commit block's, above.
// Its old sentence had `start` capturing the two files' bytes "at the moment
// it bumps them", which is only true of a leaf that bumps and captures in one
// breath; the bump and the capture are now two leaves' work at two moments,
// and the record is a value `running_record` builds from what it is given. A
// rename with the retired name kept beside it as a `#[deprecated]` alias, by
// the rule the earlier seven follow. No mark: the one validator citing it is
// named for the claim beside it, which check 14 admits.

/// When [`running_record`](crate::phase::gate_job::running_record) is asked
/// for the record a key and a pid make, the `Running` value it answers with
/// shall carry that key and that pid unchanged, and
/// `integrity::contents_of`'s answer for `Cargo.toml` and `Cargo.lock` — in
/// that call's own order — as its `version_files`, read back after the bump
/// with nothing spawned.
#[derive(Spec)]
pub struct TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder;

/// The name [`TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder`]
/// carried while `start` captured those bytes at the moment it bumped them,
/// before the record was a value built where nothing runs. The alias
/// registers no claim, so the graph sees only the claim it points at; every
/// citation of this name warns with its replacement, and those citations are
/// the later phases' work list.
#[deprecated = "replaced by TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder"]
pub type TheRunningRecordCarriesTheBytesTheBumpWrote =
    TheRunningRecordCarriesTheKeyThePidAndTheBumpsBytesInReadOrder;

/// When [`start`](crate::phase::gate_job::start) begins a job under a key, it
/// shall run `bump_workspace_version` once for that job — never for a key
/// whose job is already running or already read as `Done` — then spawn the
/// whole plan as an unwaited child and write, through `write_record`, the
/// record `running_record` built for it, before returning.
#[derive(Spec)]
pub struct AJobBeginsWithOneBumpAnUnwaitedChildAndTheRunningRecord;

/// When [`child_command`](crate::phase::gate_job::child_command) is asked for
/// a phase, a slice and a key, the command it answers with shall carry
/// `phase-check`, the phase's number, `--slice`, the slice, `--job-key` and
/// the key's hash as the last six arguments of its line — appended to whatever
/// `self_command()` already carries, whose program and leading arguments this
/// claim leaves unexamined.
#[derive(Spec)]
pub struct TheChildsLineEndsWithPhaseCheckTheSliceAndTheJobKey;

/// When [`paths_for`](crate::phase::gate_job::paths_for) and `result_path` are
/// asked for a key, the paths they answer with shall be derived from that key
/// alone — the diff file, the output root, and `result.json`, under
/// `<target>/lid-rs/gate/<the key's hash>/` — so a record and the job it names
/// agree without either of them storing a path.
#[derive(Spec)]
pub struct TheJobsScratchPathsAndItsResultPathAreDerivedFromTheKeyAlone;

/// When [`prepared_paths`](crate::phase::gate_job::prepared_paths) is asked
/// for the mutation step's keyed diff file and output root, the key's own
/// directory shall be on disk by the time the pair
/// [`paths_for`](crate::phase::gate_job::paths_for) derives is answered, so
/// `keyed_mutants` hands `mutants::run_at` a home that already exists rather
/// than one whose absence only `write_diff_file`'s own plain write, under
/// `Scope::Diff`, would otherwise discover as a failure.
#[derive(Spec)]
pub struct TheMutationStepsKeyedHomeExistsBeforeItWritesThere;

/// When [`run_job`](crate::phase::gate_job::run_job) runs the plan for a
/// `--job-key` hash, the key it runs under shall be one it recomputed itself
/// through `gate_job::key` from `project`, `phase`, `slice` and the `crates`
/// any other invocation resolves the same way, its own hash held to the flag's
/// before any step of `plan` runs.
#[derive(Spec)]
pub struct TheJobRunsUnderAKeyItRecomputedAndHeldToTheFlagsHash;

/// When the hash [`run_job`](crate::phase::gate_job::run_job) recomputes
/// differs from the one `--job-key` carried, it shall run no step of `plan`
/// and write to neither the given hash's own directory nor the fresh key's,
/// leaving the recorded pid to answer dead and the next poll to start a fresh
/// job under whatever key the tree computes then.
#[derive(Spec)]
pub struct AKeyMismatchInTheChildRunsNoStepAndWritesNoResult;

/// When [`run_job`](crate::phase::gate_job::run_job) runs the plan for a key,
/// the runner it hands `execute_with` shall run every step but `Step::Mutants`
/// through `run_step` unchanged and `Step::Mutants` through `mutants::run_at`
/// with `prepared_paths`'s diff and output-root paths, so the keyed paths are the
/// one decision `run_job` makes and `run_step` gains no branch.
#[derive(Spec)]
pub struct TheJobsRunnerDivertsTheMutationStepAloneToTheKeyedPaths;

/// When [`alive`](crate::phase::gate_job::alive) cannot ask the host whether a
/// pid is running, the poll shall fail naming the question it could not ask,
/// never reading a host that cannot answer as a job that is dead, since dead
/// is the answer that starts the whole gate again.
#[derive(Spec)]
pub struct AlivesOwnFailureFailsThePollAndNeverRestartsTheJob;

/// When [`settle`](crate::phase::gate_job::settle) is handed a record under
/// this key, it shall run `finished` first — a written outcome answering
/// `Done` before anything asks whether the pid is alive — and leave that pid's
/// liveness to `restart_or_running` only when no outcome is written.
#[derive(Spec)]
pub struct SettleReadsTheWrittenResultBeforeItAsksWhetherThePidIsAlive;

/// When [`restart_or_running`](crate::phase::gate_job::restart_or_running) is
/// reached for a record under this key whose result `settle` found unwritten,
/// the answer it gives shall be `Running` while that pid is alive and
/// `Started` once it is dead — the attempt having died before finishing, a
/// fresh job runs under that same key.
#[derive(Spec)]
pub struct RestartOrRunningAnswersRunningForALivePidAndRestartsUnderTheSameKeyForADeadOne;

/// When [`contest`](crate::phase::gate_job::contest) finds the pid of a record
/// under another key alive, it shall refuse the start outright naming that
/// other key, neither waiting for that job to finish nor restarting it under
/// the current key, since two gate runs at once on one machine is a cost
/// nobody asked for.
#[derive(Spec)]
pub struct ALiveForeignRecordRefusesTheStartNamingTheOtherKey;

/// When [`contest`](crate::phase::gate_job::contest) finds the pid of a record
/// under another key dead, it shall run a fresh job under the current key and
/// answer `Started`, a stale record blocking nothing.
#[derive(Spec)]
pub struct ADeadForeignRecordStartsAFreshJobUnderTheCurrentKey;

/// When [`status`](crate::phase::gate_job::status) answers over a live job
/// recorded under another key, the answer it gives shall carry that other key
/// — `GateStatus::Foreign`, a fourth variant beside `NoJob`, `Running` and
/// `Done` — so a reader learns which attempt it waits on rather than that
/// nothing is running.
#[derive(Spec)]
pub struct AStatusOverALiveForeignJobCarriesTheOtherKey;

/// When [`status`](crate::phase::gate_job::status) classifies what the record
/// and the fresh key say, it shall run the three-way split `act` runs — no
/// record, a matching key, a foreign key — and reach `start` on no branch of
/// it, spawning nothing and writing nothing.
#[derive(Spec)]
pub struct TheStatusDoorSharesActsThreeWaySplitAndReachesStartOnNoBranch;

/// When [`matching_status`](crate::phase::gate_job::matching_status) is handed
/// a record under this key, the answer it gives shall be `Done` for an outcome
/// already written, `Running` for a pid still alive with none written, and
/// `NoJob` where `restart_or_running` would start a fresh job.
#[derive(Spec)]
pub struct MatchingStatusAnswersNoJobWhereAPollWouldRestart;

/// When [`foreign_status`](crate::phase::gate_job::foreign_status) is handed a
/// record under another key, the answer it gives shall be `Foreign` naming
/// that key while the pid is alive and `NoJob` once it is dead, a stale record
/// being nothing for a caller to wait on.
#[derive(Spec)]
pub struct ForeignStatusAnswersForeignWhileThatPidLivesAndNoJobOnceItIsDead;

/// When [`status_line`](crate::phase::status_line) is handed a `GateStatus`,
/// the answer it gives shall be that variant's own line — one apiece for
/// `NoJob`, `Running`, `Foreign(other)` naming that other key, and
/// `Done(Ok(()))`, no two of the four alike — and, for `Done(Err(output))`,
/// this call's own `Err` carrying `output` unchanged, so a finished failure is
/// the one status this door cannot report as a pass.
#[derive(Spec)]
pub struct StatusLineAnswersEveryGateStatusItsOwnLineAndOnlyADoneFailureExitsNonZero;

/// When [`run_job`](crate::phase::gate_job::run_job) reaches the end of the
/// plan, the outcome it writes once to `result_path` shall be
/// `Done(Result<(), String>)` as structured data — the pass, or the first
/// failing step's captured output — and no marker a reader must find in
/// captured text.
#[derive(Spec)]
pub struct TheDetachedChildWritesItsOutcomeAsStructuredDataNotAMarkerInText;

/// When [`bounded_output`](crate::phase::gate_job::bounded_output) stands in
/// for a failing step's whole capture, the text it answers with shall carry
/// that step's own header up to and including `" failed: "`, the last 8 KiB of
/// the output after it, and a closing line naming `result_path`'s path for
/// what the bound cut off.
#[derive(Spec)]
pub struct TheBoundedStandInCarriesTheFailingHeaderTheLastTailAndThePath;

/// When a Phase 7 stop reads `Done(Err(output))` from
/// [`poll`](crate::phase::gate_job::poll), the text it hands `refusal_for`
/// shall be `bounded_output`'s stand-in and never the raw capture, so the
/// refusal a poll produces is bounded however large the failing step's own
/// output is.
#[derive(Spec)]
pub struct ADoneErrReachesRefusalForThroughTheBoundedStandIn;

/// When [`refusal_for`](crate::phase::ending::refusal_for) runs
/// `check_of_output` over that bounded stand-in, the refusal it composes shall
/// name the check that fired — a clippy lint name, or the red-run and check-12
/// markers the kept tail holds — rather than falling through to its
/// unnamed-failure answer.
#[derive(Spec)]
pub struct TheBoundedStandInStillNamesTheCheckThatFired;

/// When [`run`](crate::phase::run) parses a `phase-check` invocation, the
/// branch it takes shall be `check` called in-process and blocked on while
/// `--job-key` is absent — a human's or CI's own call — and
/// `gate_job::run_job` for the key given while it is present, a flag only
/// `gate_job::start` sets.
#[derive(Spec)]
pub struct AnAbsentJobKeyRunsTheCheckInlineAndAPresentOneRunsTheJob;

// ---- execution class -----------------------------------------------------------

/// When the slice's crate declares a `proc-macro` or `custom-build` target,
/// its execution class shall be compile-time, naming which; otherwise it
/// shall be ordinary.
#[derive(Spec)]
#[lid(free)]
pub struct ACompileTimeSliceIsDisclosed;

/// When the slice is compile-time and the `compile-time-accepted` file of
/// its intent — wherever the layout puts that file in the slice's own crate
/// — does not exist, the policy shall refuse every edit, naming the file the
/// human commits to accept it.
#[derive(Spec)]
#[lid(free)]
pub struct ACompileTimeSliceNeedsTheHumansAcceptance;

// ---- sync: the mirrored artifacts ------------------------------------------------

/// When `sync` runs, it shall mirror each artifact the resolved `lid-rs`
/// ships — the `skill/`, `workflow/`, and `agent/` directories — to its
/// place in the project, and `--check` shall hold every one to the skill's
/// any-difference rule.
#[derive(Spec)]
#[lid(free)]
pub struct SyncMirrorsEveryArtifactTheDependencyShips;
