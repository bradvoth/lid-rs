//! Claims for `lld-review` (`docs/intent/lld-review/lld.md`): one slice's LLD
//! is read mechanically, before the slice is built, and so are the two
//! artifacts — the guideline and the reader — that carry the judgment the
//! mechanical checks cannot.
//!
//! What binds these checks to a phase is deliberately absent here. `phase-check
//! 1` gains a step that runs them before its doc step, and that step is the
//! phase slice's claim to widen — `spec::PhaseOneChecksTheDocs` — when it lands
//! (the LLD's Cascade section). So no claim below says when or by whom the
//! checks are run: they describe `lld-check` itself, which holds whether it is
//! reached from a phase's check or from a human's command line.

use lid_rs::Spec;

// ---- the subcommand -----------------------------------------------------------

/// When `lld-check` runs, the slice it checks shall be the value of `--slice`,
/// or, absent that flag, the current branch's name with `lld/` removed.
#[derive(Spec)]
pub struct TheSliceIsTheFlagsValueOrTheBranchName;

/// When `lld-check` is given an argument that is not `--slice <name>`, it shall
/// be rejected by name.
#[derive(Spec)]
pub struct AnyOtherArgumentIsRejectedByName;

/// When `lld-check` reads a slice's document, it shall be that slice's
/// `lld.md` under the workspace package whose manifest directory holds it,
/// wherever the layout puts that document, found on the filesystem.
#[derive(Spec)]
pub struct TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt;

/// When no workspace package holds a document for the slice, the document
/// shall be the layout's answer under the workspace root, where a slice whose
/// product is the workspace rather than a crate keeps it.
#[derive(Spec)]
pub struct AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot;

/// When the document the layout answers for the slice cannot be read,
/// `lld-check` shall fail naming the path it looked for.
#[derive(Spec)]
pub struct AnUnreadableLldFailsNamingItsPath;

/// When every check holds on the document, `lld-check` shall exit zero; when
/// any check fails it shall exit non-zero.
#[derive(Spec)]
pub struct LldCheckExitsZeroOnlyWhenEveryCheckHolds;

/// When more than one check fails, `lld-check` shall report every failure
/// rather than stopping at the first.
#[derive(Spec)]
pub struct EveryFailureIsReportedNotOnlyTheFirst;

/// When a check fails, its failure shall name the check, the file and line it is
/// about, and the sentence the skill states the rule in.
#[derive(Spec)]
pub struct AFailureNamesItsCheckItsFileItsLineAndItsRule;

// ---- the parse the checks share -----------------------------------------------

/// When a table is read under a heading, it shall be the pipe-delimited rows
/// between that heading and the next heading, less the header row and its
/// separator.
#[derive(Spec)]
pub struct ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator;

// ---- the document checks ------------------------------------------------------

/// When the document has no `## Decisions & Alternatives` heading with a table
/// under it, `lld-check` shall fail naming that heading.
#[derive(Spec)]
pub struct ADocumentWithoutADecisionsTableFails;

/// When a row of the decisions table has fewer than four cells, or leaves one of
/// them empty, `lld-check` shall fail naming that row's line.
#[derive(Spec)]
pub struct EveryDecisionsRowFillsItsFourCells;

/// When a row of the `## Shape` table names no backticked identifier, or gives
/// an empty role, `lld-check` shall fail naming that row's line.
#[derive(Spec)]
pub struct EveryShapeRowNamesAnIdentifierAndARole;

/// When the document has no `## Shape` table, the shape-row check shall hold,
/// since a slice may name its shape in prose instead.
#[derive(Spec)]
pub struct ADocumentWithNoShapeTableHoldsThatCheck;

/// When an item under `### Deferred` is not a numbered list item, `lld-check`
/// shall fail naming that item's line.
#[derive(Spec)]
pub struct EveryDeferredItemIsANumberedListItem;

/// When the document has no `### Deferred` heading, the deferred-numbering check
/// shall hold, since a document that defers nothing has no item to number.
#[derive(Spec)]
pub struct ADocumentWithNoDeferredHeadingHoldsThatCheck;

// ---- the artifact checks ------------------------------------------------------
//
// These two hold a property of the project's own synced copies of the guideline
// and the reader — `.claude/skills/lid-rs/references/lld.md` and
// `.claude/agents/lid-rs-lld-review.md`, the files a consumer has — not of the
// document named by `--slice`.

/// When the checklist in `.claude/skills/lid-rs/references/lld.md` does not name
/// every variant of `Check`, `lld-check` shall fail naming each variant the
/// checklist omits, on the line that file's checklist heading is at.
#[derive(Spec)]
pub struct EveryCheckIsNamedInTheGuidelinesChecklist;

/// When the frontmatter of `.claude/agents/lid-rs-lld-review.md` declares a set
/// of tools other than exactly `Read`, `Grep` and `Glob`, `lld-check` shall fail
/// naming the tools it declares, on the line that file's `tools:` declaration is
/// at.
#[derive(Spec)]
pub struct TheReaderDeclaresOnlyTheObservationTools;

/// When an artifact check has no such line to point at — the guideline holding
/// no checklist heading, the reader no `tools:` line — its failure shall point
/// at that file's first line.
#[derive(Spec)]
pub struct AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine;

/// When `lld-check` runs, it shall apply both artifact checks whatever slice is
/// named, since the files they read do not depend on the slice.
#[derive(Spec)]
pub struct TheArtifactChecksRunWhateverSliceIsNamed;

/// When either synced artifact is absent or cannot be read, `lld-check` shall
/// fail naming the path it looked for.
#[derive(Spec)]
pub struct AnUnreadableSyncedArtifactFailsNamingItsPath;
