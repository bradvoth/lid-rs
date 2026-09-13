//! Claims for `lld-review` (`src/lld_review/lld.md`): one slice's LLD is read
//! mechanically, before the slice is built — its own text, the code its shape
//! rows describe, and the two artifacts, the guideline and the reader, that
//! carry the judgment the mechanical checks cannot.
//!
//! What binds these checks to a phase is not here, because it is not this
//! slice's. The step of `phase-check 1` that runs them before its doc step is
//! the phase slice's and already stands: [`plan`](crate::phase::plan) begins
//! Phase 1 with `Step::LldChecks`, under
//! [`PhaseOneChecksTheDocs`](crate::phase::spec::PhaseOneChecksTheDocs). So no
//! claim below says when or by whom the checks are run: they describe
//! `lld-check` itself, which holds whether it is reached from a phase's check
//! or from a human's command line.

use lid_rs::Spec;

// ---- the subcommand -----------------------------------------------------------

/// When `lld-check` runs, the slice it checks shall be the value of `--slice`,
/// or, absent that flag, the current branch's name with `lld/` removed.
#[derive(Spec)]
#[lid(free)]
pub struct TheSliceIsTheFlagsValueOrTheBranchName;

/// When `lld-check` is given an argument that is not `--slice <name>`, it shall
/// be rejected by name.
#[derive(Spec)]
#[lid(free)]
pub struct AnyOtherArgumentIsRejectedByName;

/// When `lld-check` reads a slice's document, it shall be that slice's
/// `lld.md` under the workspace package whose manifest directory holds it,
/// wherever the layout puts that document, found on the filesystem.
#[derive(Spec)]
#[lid(free)]
pub struct TheDocumentIsTheSlicesLldUnderThePackageThatHoldsIt;

/// When no workspace package holds a document for the slice, the document
/// shall be the layout's answer under the workspace root, where a slice whose
/// product is the workspace rather than a crate keeps it.
#[derive(Spec)]
#[lid(free)]
pub struct AWorkspaceOnlySlicesDocumentIsAtTheWorkspaceRoot;

/// When the document the layout answers for the slice cannot be read,
/// `lld-check` shall fail naming the path it looked for.
#[derive(Spec)]
#[lid(free)]
pub struct AnUnreadableLldFailsNamingItsPath;

/// When every check holds on the document, `lld-check` shall exit zero; when
/// any check fails it shall exit non-zero.
#[derive(Spec)]
#[lid(free)]
pub struct LldCheckExitsZeroOnlyWhenEveryCheckHolds;

/// When more than one check fails, `lld-check` shall report every failure
/// rather than stopping at the first.
#[derive(Spec)]
#[lid(free)]
pub struct EveryFailureIsReportedNotOnlyTheFirst;

/// When a check fails, its failure shall name the check, the file and line it is
/// about, and the sentence the skill states the rule in.
#[derive(Spec)]
#[lid(free)]
pub struct AFailureNamesItsCheckItsFileItsLineAndItsRule;

// ---- the parse the checks share -----------------------------------------------

/// When a table is read under a heading, it shall be the pipe-delimited rows
/// between that heading and the next heading, less the header row and its
/// separator.
#[derive(Spec)]
#[lid(free)]
pub struct ATableIsTheRowsUnderItsHeadingLessHeaderAndSeparator;

// ---- the document checks ------------------------------------------------------

/// When the document has no `## Decisions & Alternatives` heading with a table
/// under it, `lld-check` shall fail naming that heading.
#[derive(Spec)]
#[lid(free)]
pub struct ADocumentWithoutADecisionsTableFails;

/// When a row of the decisions table has fewer than four cells, or leaves one of
/// them empty, `lld-check` shall fail naming that row's line.
#[derive(Spec)]
#[lid(free)]
pub struct EveryDecisionsRowFillsItsFourCells;

/// When a row of the `## Shape` table names no backticked identifier, or gives
/// an empty role, `lld-check` shall fail naming that row's line.
#[derive(Spec)]
#[lid(free)]
pub struct EveryShapeRowNamesAnIdentifierAndARole;

/// When the document has no `## Shape` table, the shape-row check shall hold,
/// since a slice may name its shape in prose instead.
#[derive(Spec)]
#[lid(free)]
pub struct ADocumentWithNoShapeTableHoldsThatCheck;

/// When an item under `### Deferred` is not a numbered list item, `lld-check`
/// shall fail naming that item's line.
#[derive(Spec)]
#[lid(free)]
pub struct EveryDeferredItemIsANumberedListItem;

/// When the document has no `### Deferred` heading, the deferred-numbering check
/// shall hold, since a document that defers nothing has no item to number.
#[derive(Spec)]
#[lid(free)]
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
#[lid(free)]
pub struct EveryCheckIsNamedInTheGuidelinesChecklist;

/// When the frontmatter of `.claude/agents/lid-rs-lld-review.md` declares a set
/// of tools other than exactly `Read`, `Grep` and `Glob`, `lld-check` shall fail
/// naming the tools it declares, on the line that file's `tools:` declaration is
/// at.
#[derive(Spec)]
#[lid(free)]
pub struct TheReaderDeclaresOnlyTheObservationTools;

/// When an artifact check has no such line to point at — the guideline holding
/// no checklist heading, the reader no `tools:` line — its failure shall point
/// at that file's first line.
#[derive(Spec)]
#[lid(free)]
pub struct AnArtifactFailureWithNoLineToCitePointsAtTheFirstLine;

/// When `lld-check` runs, it shall apply both artifact checks whatever slice is
/// named, since the files they read do not depend on the slice.
#[derive(Spec)]
#[lid(free)]
pub struct TheArtifactChecksRunWhateverSliceIsNamed;

/// When either synced artifact is absent or cannot be read, `lld-check` shall
/// fail naming the path it looked for.
#[derive(Spec)]
#[lid(free)]
pub struct AnUnreadableSyncedArtifactFailsNamingItsPath;

// ---- the order of the checks --------------------------------------------------
//
// `Check` is declared in the order the LLD's table states the checks. A
// declaration has no wrong answer a test could catch, so the claim about that
// order is cited by `check_all`, which runs them in it.

/// When [`check_all`](crate::lld_review::check_all) applies the checks, the
/// failures it answers shall be in the order the table of checks states them
/// — decisions exist, alternatives, shape rows, rows agree with the code,
/// returns are skeletonable, a reuse row is linked, deferred is numbered,
/// guideline names every check, reader observes only — which is the order
/// [`Check`](crate::lld_review::Check) declares its variants.
#[derive(Spec)]
#[lid(free)]
pub struct TheChecksRunInTheOrderTheTableStatesThem;

// ---- what the slice declares --------------------------------------------------
//
// The code a row describes is read through `lid_rs_shape::signatures`, a
// syntactic pass over one crate's tokens that resolves nothing. The claims
// about the `Declared` shape are cited by `declared`, since data has no wrong
// answer to go red on.

/// When `declared` reads a slice, the functions it answers shall be those the
/// shape pass reads over the slice's own crate,
/// [`own_crate`](crate::layout::own_crate), whose file is under
/// [`slice_dir`](crate::layout::slice_dir) or is the `src/<module>.rs` file
/// module beside it — and no function from elsewhere in that crate.
#[derive(Spec)]
#[lid(free)]
pub struct TheDeclaredFunctionsAreTheSlicesOwnCratesUnderItsDirectory;

/// When no workspace member holds a crate for the slice, `declared` shall
/// answer no function, and not an error.
#[derive(Spec)]
#[lid(free)]
pub struct ASliceNoCrateHoldsDeclaresNoFunction;

/// When the shape pass answers a function whose first parameter's type tokens
/// are `Self`, `&Self`, `&mut Self` or `Box<Self>` once they are read as every
/// token string here is read — lifetimes erased, then whitespace removed, which
/// is what makes a `&'a self` one of those four shapes and not a fifth — the
/// parameters `declared` carries for it shall be the remaining ones, so that
/// their count is a count of arguments.
#[derive(Spec)]
#[lid(free)]
pub struct ADeclaredMethodsReceiverIsNotAmongItsParameters;

// ---- what a first cell writes -------------------------------------------------
//
// The claims about the `Fragment` shape are cited by `fragments`, since data
// has no wrong answer to go red on.

/// When a first cell holds several backticked identifiers, `fragments` shall
/// answer one fragment per identifier, as many as the cell holds.
#[derive(Spec)]
#[lid(free)]
pub struct EveryBacktickedIdentifierOfAFirstCellIsItsOwnFragment;

/// When a backticked identifier writes a return after the Unicode `→` rather
/// than the ASCII `->`, the fragment `fragments` answers for it shall carry no
/// return.
#[derive(Spec)]
#[lid(free)]
pub struct AReturnIsWrittenWithTheAsciiArrowAlone;

/// When a backticked identifier's first argument is written `self`, `&self` or
/// `&mut self`, `fragments` shall leave it out of the fragment's arguments.
#[derive(Spec)]
#[lid(free)]
pub struct AFirstArgumentWrittenAsAReceiverIsNotAFragmentsArgument;

/// When a first cell holds a backtick span, the fragment `fragments` answers
/// for it shall be linked exactly when the span is immediately preceded by `[`
/// and immediately followed by `](`.
#[derive(Spec)]
#[lid(free)]
pub struct AFragmentIsLinkedOnlyWhenItsBacktickSpanIsWrappedInAMarkdownLink;

// ---- rows agree with the code -------------------------------------------------

/// When `matches` compares a fragment to a declaration, it shall hold exactly
/// when their names are equal and, where the fragment's first segment starts
/// with an uppercase letter, that segment is the declaration's owner, each read
/// as every token string here is read — lifetimes erased, then whitespace
/// removed — so that a row writing `Fields` narrows to a method declared in an
/// `impl<'_> Fields<'_>`.
#[derive(Spec)]
#[lid(free)]
pub struct AQualifierNarrowsAMatchOnlyWhenItIsAType;

/// When a fragment writes no parentheses, `shape_agrees` shall compare it to
/// nothing, since it names a type rather than a function.
#[derive(Spec)]
#[lid(free)]
pub struct AFragmentWithoutParenthesesIsComparedToNothing;

/// When a fragment matches no function the slice declares, `shape_agrees` shall
/// hold for it.
#[derive(Spec)]
#[lid(free)]
pub struct AFragmentNamingNoDeclaredFunctionIsComparedToNone;

/// When a fragment matches one or more functions the slice declares and agrees
/// with none of them, `shape_agrees` shall fail it on its own row's line.
#[derive(Spec)]
#[lid(free)]
pub struct AFragmentAgreeingWithAnyDeclarationItNamesHolds;

/// When `agrees` compares a fragment against a declaration it names, it shall
/// hold exactly when the fragment's argument count is the declaration's
/// parameter count and the return it writes, if any, is the source's as
/// `same_type` compares them.
#[derive(Spec)]
#[lid(free)]
pub struct AFragmentAgreesWhenItsCountAndAnyReturnItWritesAreTheSources;

/// When `same_type` compares a row's return with the source's, it shall strip
/// whitespace from both sides before comparing, so that `Result<Lld, String>`
/// and `Result < Lld , String >` are the same type.
#[derive(Spec)]
#[lid(free)]
pub struct WhitespaceIsStrippedFromBothSidesBeforeTwoTypesAreCompared;

/// When `same_type` compares a row's return with the source's, it shall erase
/// lifetime arguments and lifetime annotations from both sides before
/// comparing, so that `Section<'a>` and `Section` are the same type, and so are
/// `&'static str` and `&str`.
#[derive(Spec)]
#[lid(free)]
pub struct LifetimeArgumentsAndAnnotationsAreErasedBeforeTwoTypesAreCompared;

/// When erasing a lifetime leaves an argument list holding no argument at all,
/// `same_type` shall read the type as one written with no argument list,
/// brackets included, so that `Section<'a>` is read as `Section` and never as
/// `Section<>`.
#[derive(Spec)]
#[lid(free)]
pub struct AnArgumentListLeftWithNoArgumentIsNotWrittenAtAll;

/// When either the row's return or the source's writes `Self`, `same_type`
/// shall substitute the declaration's owner for it on both sides — never the
/// fragment's own qualifier, and nothing for a free function, whose `Self` then
/// stands for no type and the row disagrees.
#[derive(Spec)]
#[lid(free)]
pub struct SelfIsTheDeclarationsOwnerOnBothSides;

/// When `agreement_failure` builds a failure, it shall carry the document's
/// path and the row's line, and a message naming the item's file, what the row
/// said and what the source said, in that order, before the rule.
#[derive(Spec)]
#[lid(free)]
pub struct AnAgreementFailureNamesTheItemsFileAndBothReadings;

// ---- a reuse row is linked ----------------------------------------------------
//
// Whether a row is about another slice's item is decided from its first cell
// and from names — the path's own text, a listing of the slice's directory, and
// the workspace's member list — never from resolution, which is rustdoc's.

/// When a first cell's fragment is a reuse row and is not linked,
/// `reuse_rows_linked` shall fail naming that row's line.
#[derive(Spec)]
#[lid(free)]
pub struct AReuseRowNamesItsItemAsAnIntraDocLink;

/// When a fragment's path has two or more segments and a first segment
/// starting with a lowercase letter that is none of the slice's own name in
/// module form, a sibling inside the slice's directory, or a workspace member's
/// crate name, `is_reuse` shall hold.
#[derive(Spec)]
#[lid(free)]
pub struct AModuleQualifiedPathOutsideTheSlicesDirectoryIsAReuseRow;

/// When a fragment's path has one segment, or a first segment that does not
/// start with a lowercase letter, `is_reuse` shall not hold — a bare name
/// names nothing elsewhere, and a type the slice does not declare is
/// introduced by the role cell that says whose it is.
#[derive(Spec)]
#[lid(free)]
pub struct APathThatIsNotModuleQualifiedIsNotAReuseRow;

/// When a fragment's first segment is the slice's own name with hyphens read as
/// underscores, or a sibling listed for the slice, `is_reuse` shall not hold,
/// since the path names the slice or a module under it.
#[derive(Spec)]
#[lid(free)]
pub struct APathIntoTheSlicesOwnModulesIsNotAReuseRow;

/// When a fragment's first segment is a workspace member's crate name,
/// `is_reuse` shall not hold, since the path is into a crate and not into a
/// sibling module.
#[derive(Spec)]
#[lid(free)]
pub struct APathQualifiedByAWorkspaceMembersCrateNameIsNotAReuseRow;

/// When `siblings` lists a slice's directory, it shall answer each file's stem
/// without its extension and each directory's own name, one level inside, with
/// `mod.rs` left out.
#[derive(Spec)]
#[lid(free)]
pub struct ASiblingIsAFileStemOrADirectoryNameOneLevelInsideTheSlice;

/// When the slice has no directory to read — no crate holds it, its code is
/// the one file `src/<module>.rs`, or the directory cannot be read — `siblings`
/// shall list nothing, and not an error.
#[derive(Spec)]
#[lid(free)]
pub struct ASliceWithNoDirectoryToReadListsNoSibling;

/// When `crate_names` lists the workspace, it shall answer each member's
/// package name —
/// [`Project::member_manifest_dirs`](crate::project::Project::member_manifest_dirs)
/// for the members, [`Project::package_at`](crate::project::Project::package_at)
/// for each one's name — with hyphens read as underscores.
#[derive(Spec)]
#[lid(free)]
pub struct ACrateNameIsAMembersPackageNameWithHyphensAsUnderscores;

// ---- returns are skeletonable -------------------------------------------------

/// When a return fragment's outermost form is `impl Trait` or a bare
/// `dyn Trait`, `skeletonable` shall not hold — a `todo!()` body infers the
/// hidden type of the first as `!`, which implements nothing, and the second
/// is unsized in return position.
#[derive(Spec)]
#[lid(free)]
pub struct AnImplTraitOrBareDynTraitReturnIsNotSkeletonable;

/// When a return fragment merely contains an `impl` or `dyn` type — `Box<dyn
/// Trait>`, `Result<Box<dyn Error>, String>` — `skeletonable` shall hold, since
/// the test is over the return's outermost form and nothing deeper.
#[derive(Spec)]
#[lid(free)]
pub struct AReturnThatMerelyContainsAnImplOrDynTypeIsSkeletonable;

/// When a `## Shape` row's fragment writes a return `skeletonable` refuses,
/// `shape_returns` shall fail naming that row's line.
#[derive(Spec)]
#[lid(free)]
pub struct ARowWhoseReturnCannotBeSkeletonisedFailsOnItsLine;
