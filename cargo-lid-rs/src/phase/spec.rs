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

/// When the final message carries a `commit` block with this phase's tag,
/// the hook shall run the phase's check, and a failing check shall refuse
/// the stop.
#[derive(Spec)]
#[lid(free)]
pub struct ACommitBlockRunsThePhasesCheck;

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
// happen. So the Phase 7 stop raises the workspace version before the check,
// from the manifest at `HEAD` — which is what makes a refused stop's second
// bump answer the same version. Every claim here is written in the
// controlled language and carries no mark. The subject-version refusal is a
// sibling of `ACommitSubjectMustCarryThisPhasesTag`, which is about the tag
// and says nothing about a version.

/// When [`bump_workspace_version`](crate::phase::bump_workspace_version)
/// runs for a Phase 7 stop before the check, the version it writes to the
/// working tree shall be one patch level above the workspace `version` the
/// root manifest holds as committed at `HEAD`, so a second bump before any
/// commit writes the same version again.
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

/// When the version [`subject_version`](crate::phase::subject_version) reads
/// from a Phase 7 `commit` block's subject is not the one the bump produced,
/// the stop shall be refused naming both versions — after the bump and
/// before the check runs.
#[derive(Spec)]
pub struct APhaseSevenSubjectMustCarryTheBumpedVersion;

/// When a phase commit is made, its message shall end with the
/// `Lid-Rs-Phase`, `Lid-Rs-Agent`, `Lid-Rs-Tools`, `Lid-Rs-Checks`, and
/// `Lid-Rs-Refusals` trailers rendered from the agent's tally, the agent
/// being the id the tally was kept under.
#[derive(Spec)]
#[lid(free)]
pub struct TheTallyIsWrittenAsTrailers;

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
