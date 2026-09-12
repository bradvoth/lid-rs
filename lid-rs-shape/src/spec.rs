//! Claims for the `lid-rs-shape` slice — a function is flow or leaf by its
//! shape (`lid-rs-shape/src/lld.md`).
//!
//! The slice builds one `syn` pass over one crate's source: the classification
//! of every function against README `§3.7`'s six rules, the three rule checks
//! (A, B and V) over that classification, the signature tokens rule V reads and
//! slice 19 will consume, and what the pass does at the five places it cannot
//! answer. The claims below are those four groups.
//!
//! **Every claim is written as `Finding` production, never as a gate.** This
//! crate has no `level`: `check` answers findings and takes none, and `cargo
//! lid-rs shape` — in `cargo-lid-rs`, which reads the
//! `[workspace.metadata.lid_rs.shape]` table and owns the exit status —
//! decides what a finding costs. That is the document's own Decisions row, and it is the row
//! that makes this phase possible: a claim reading "shall fail" under one
//! measurement and "shall report" under another cannot be written at all, and
//! check 14's one-validator-per-claim binding would make the wrong choice
//! unrecoverable without a rename cascade. So no claim here says fail, deny,
//! gate or warn; each says what the pass reports, carries or is.
//!
//! **F2 is two claims, because it packs two independently falsifiable
//! statements.** README states F2 as "at most one decision structure — one
//! `match`, or one two-way `if` — and every arm is a single call". A pass that
//! admits a body holding two `match`es falsifies the first half and not the
//! second; a pass that admits an arm holding a block falsifies the second and
//! not the first. As one claim, check 14 would bind both to a single validator
//! and either failure would be reported as the same test failing.
//! [`ABodyWithMoreThanOneDecisionStructureIsALeafUnderF2`] and
//! [`ADecisionArmThatIsNoSingleCallMakesTheBodyALeafUnderF2`] are the two, and
//! the document's "F1–F6 are six claims, not one" is kept whole by them: this
//! is a finer cut of one rule, not a collapse of six into fewer.
//!
//! **Rule B is three claims: the violation, the escape, and the count.** The
//! document says "a public leaf is allowed **and counted**, which is what
//! 'counted and reported' means — the mark exists to be visible". Not being
//! reported and being visible are separately falsifiable: a pass that dropped
//! every `#[leaf]`-marked function on the floor would satisfy
//! [`APublicLeafMarkedLeafIsNoFindingUnderRuleB`] and leave the ramp with
//! nothing to count. [`TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted`] is
//! the count, and it is stated of [`classify`](crate::classify) rather than
//! [`check`](crate::check) because this crate writes no files and `Finding` is
//! "one rule violation": the only place a mark can be visible to the caller that
//! serialises `shape-classify.json` is the `Shape` the classification carries.
//! The Shape table's row for `Shape` lists four things and the mark is not among
//! them, so this claim is the one place this phase proposes a field the document
//! does not enumerate — see the commit's `Review:` block.
//!
//! **Rule V is five claims.** The document's one line — "a flow signature uses
//! no primitive, wrappers unwrapped" — carries the deny set, the unwrapping, and
//! two positions, and each has its own wrong answer. The set is
//! [`RuleVDeniesReadmesDenyClauseAndNoOtherName`], which is also where the
//! **narrowing is named**: README `§3.6` states rule V as an allow-list over
//! vocabulary types, nothing in the tree enumerates those nouns, and this crate
//! implements the deny clause instead — so a reader of that claim cannot mistake
//! it for README's rule. The unwrapping is
//! [`AGivenWrapperIsUnwrappedBeforeRuleVTestsTheName`]; the two positions are
//! [`AFlowParameterWrittenAsADeniedNameIsAFindingUnderRuleV`] and
//! [`AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV`], which are separate
//! because `signatures` is defined over the **whole return** type and rule V
//! over the `Ok` type alone, so a pass can get one position right and the other
//! wrong. [`RuleVTestsTheNameTheSourceWroteAndResolvesNothing`] is the fifth and
//! is constraint 2's line: a V that chased `String` through a `use` or a type
//! alias would be resolving a name, which is the one thing the carve-out this
//! slice lives in does not permit.
//!
//! **F1 and F4 defer to the structure F2 admits, and the claims say so.** Read
//! literally, F1 ("every statement is `let pat = call(...)?;`, `call(...)?;`, or
//! the tail call") admits no `match` statement and F4 ("no closures or blocks")
//! admits no `if`, whose arms are blocks by Rust's grammar — which would make
//! F2's allowance of one decision structure dead. The six rules are one
//! definition, so [`ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1`] and
//! [`AClosureOrANonArmBlockMakesTheBodyALeafUnderF4`] each except what F2
//! admits. That is the minimum reading under which all six rules can hold at
//! once, and it is this phase's, not the document's.
//!
//! **"Which F-number it failed first" is read as the lowest-numbered rule.**
//! [`TheVerdictNamesTheLowestNumberedRuleTheBodyFailed`] is the one claim that
//! states the number the report needs, and it is separate from the seven rule
//! claims because the rules overlap: a body with a literal argument fails F3 and
//! F5 together, and a body binding a closure fails F1 as well as F4. Each rule
//! claim therefore asserts the verdict — leaf — and names its rule in its own
//! text and name, so a failing validator says which rule was got wrong; the
//! report's rule number is the ordering claim's, where a body failing two rules
//! is the fixture that can falsify it.
//!
//! **Four of the five "cannot answer" behaviours are claims; one is not.** A
//! file `syn` cannot parse, a crate with no `src`, a `#[cfg]`-gated module and a
//! `#[path]` the pass cannot read from are each a decision the pass makes, and
//! `§0`'s rule makes a decision a claim. **A macro-generated function is not.**
//! The document's sentence — "invisible, because it does not exist as tokens in
//! the source" — states a consequence of parsing without expanding, and no
//! observation distinguishes a pass that honours it from one that does not:
//! nothing produces the function for a validator to look for. What the pass does
//! see, the macro invocation in the enclosing body, is already
//! [`AMacroOutsideTheAllowedSetMakesTheBodyALeafUnderF6`].
//!
//! **Half of `#[path]` is withheld, and deliberately.** The document says a
//! `#[path]` module is "followed as a file path when the attribute is a literal,
//! and otherwise reported as unreachable". The second half is
//! [`APathAttributeThatIsNoLiteralIsReportedUnreachable`]. The first half is not
//! written, because the document states no bound on the path: taken as written
//! it follows `#[path = "../../../elsewhere.rs"]` out of the one crate this pass
//! is defined to read, and a claim that fixed a bound would be this phase
//! settling a question the LLD left open. It is raised at the stop instead.
//!
//! **What has no claim here, and where it lives.** `level` and the exit status
//! are `cargo-lid-rs`'s; `shape.json` and `shape-classify.json` are written by
//! the same caller, which is why no claim mentions a file; rule P and check 17
//! land with the pins on a `lid-rs-macros` slice, whose paths no phase of this
//! slice may write; rule C is the reason the threshold re-measurement is
//! meaningful and is not a check; and the `intent_graph!()` test form is
//! dropped for the package cycle, so nothing here claims a second entry point.
//! The re-measurement itself is a report taken at Phase 7, not shipped code.
//!
//! **The controlled language was held by hand.** `lid-rs-shape/src/lib.rs`
//! declares no `pub mod spec;` yet — the LLD lands that line between this phase
//! and Phase 3 — so nothing compiles this file and check 13 does not run on it.
//! Every verb below is the base lexicon's (`be`, `carry`, `name`, `report`),
//! which is what a publishable crate is held to whatever a project's file says;
//! every claim is one sentence with one `shall`, one terminator and its
//! terminating periods inside backticks; and every trigger names a Rust item of
//! this crate by intra-doc link, placed first in its clause so the link the
//! derive records is the one intended. None of these claims is marked free.

use lid_rs::Spec;

// ---- The six rules of README §3.7, and the verdict they compose -------------

/// When [`classify`](crate::classify) reads a function whose body holds a
/// statement outside the forms F1 admits, the [`Shape`](crate::Shape) it answers
/// with shall be a leaf, those forms being `let pat = call(...)?;`, a bare call,
/// the tail call, and the one decision structure F2 admits.
#[derive(Spec)]
pub struct ABodyWithAStatementThatIsNoCallFormIsALeafUnderF1;

/// When [`classify`](crate::classify) reads a function whose body holds more
/// than one decision structure, the [`Shape`](crate::Shape) it answers with
/// shall be a leaf, F2 admitting one `match` or one two-way `if` and no second.
#[derive(Spec)]
pub struct ABodyWithMoreThanOneDecisionStructureIsALeafUnderF2;

/// When [`classify`](crate::classify) reads a function whose decision structure
/// holds an arm that is not a single call, the [`Shape`](crate::Shape) it
/// answers with shall be a leaf, F2 admitting an arm that is a single call and
/// no other.
#[derive(Spec)]
pub struct ADecisionArmThatIsNoSingleCallMakesTheBodyALeafUnderF2;

/// When [`classify`](crate::classify) reads a function whose body holds a call
/// taking an argument outside the forms F3 admits, the [`Shape`](crate::Shape)
/// it answers with shall be a leaf, those forms being a path, a field access, a
/// reference, and an accessor chain.
#[derive(Spec)]
pub struct AnArgumentOutsideTheAdmittedFormsMakesTheBodyALeafUnderF3;

/// When [`classify`](crate::classify) reads a function whose body holds a
/// closure or a block that is no arm of the decision structure F2 admits, the
/// [`Shape`](crate::Shape) it answers with shall be a leaf.
#[derive(Spec)]
pub struct AClosureOrANonArmBlockMakesTheBodyALeafUnderF4;

/// When [`classify`](crate::classify) reads a function whose body holds a
/// literal other than `()`, the [`Shape`](crate::Shape) it answers with shall be
/// a leaf, F5 admitting the unit literal alone.
#[derive(Spec)]
pub struct ALiteralOtherThanUnitMakesTheBodyALeafUnderF5;

/// When [`classify`](crate::classify) reads a function whose body invokes a
/// macro outside the allowed macros it was given, the [`Shape`](crate::Shape) it
/// answers with shall be a leaf, F6 reading that set from its caller and holding
/// none of its own.
#[derive(Spec)]
pub struct AMacroOutsideTheAllowedSetMakesTheBodyALeafUnderF6;

/// When [`classify`](crate::classify) reads a function whose body fails one or
/// more of F1 through F6, the [`Shape`](crate::Shape) it answers with shall name
/// the lowest-numbered rule among them.
#[derive(Spec)]
pub struct TheVerdictNamesTheLowestNumberedRuleTheBodyFailed;

/// When [`classify`](crate::classify) reads a function whose body fails none of
/// F1 through F6, the [`Shape`](crate::Shape) it answers with shall be flow.
#[derive(Spec)]
pub struct ABodyFailingNoneOfTheSixRulesIsFlow;

// ---- Rule A (check 15): routing must be flow --------------------------------

/// When [`check`](crate::check) is given a leaf whose dispatch arity reaches the
/// arm count it was given, it shall report a [`Finding`](crate::Finding) against
/// that function under rule A.
#[derive(Spec)]
pub struct ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA;

// ---- Rule B (check 16): a slice's public functions, and the mark's escape ----

/// When [`check`](crate::check) is given a `pub fn` that is a leaf in one of the
/// slice `mod.rs` files it was given and carries no `#[leaf]` mark, it shall
/// report a [`Finding`](crate::Finding) against that function under rule B.
#[derive(Spec)]
pub struct AnUnmarkedPublicLeafInASliceModIsAFindingUnderRuleB;

/// When [`check`](crate::check) is given a `pub fn` that is a leaf in one of the
/// slice `mod.rs` files it was given and carries the `#[leaf]` mark, it shall
/// report no [`Finding`](crate::Finding) against that function, the mark being
/// rule B's escape.
#[derive(Spec)]
pub struct APublicLeafMarkedLeafIsNoFindingUnderRuleB;

/// When [`classify`](crate::classify) reads a function carrying the `#[leaf]`
/// mark, the [`Shape`](crate::Shape) it answers with shall carry that mark, so a
/// reader of the [`Classification`](crate::Classification) counts the public
/// leaves rule B allowed.
#[derive(Spec)]
pub struct TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted;

// ---- Rule V (check 18): the deny clause, unwrapped, in two positions ---------

/// When [`check`](crate::check) tests a written type under rule V, the names it
/// denies shall be `String`, `&str`, `bool`, the integer types, the float types
/// and `char`, and no other name, this crate narrowing README's allow-list to
/// that deny clause.
#[derive(Spec)]
pub struct RuleVDeniesReadmesDenyClauseAndNoOtherName;

/// When [`check`](crate::check) is given a flow function whose parameter is
/// written as a name rule V denies, it shall report a
/// [`Finding`](crate::Finding) against that function under rule V.
#[derive(Spec)]
pub struct AFlowParameterWrittenAsADeniedNameIsAFindingUnderRuleV;

/// When [`check`](crate::check) is given a flow function whose `Ok` type is
/// written as a name rule V denies, it shall report a
/// [`Finding`](crate::Finding) against that function under rule V.
#[derive(Spec)]
pub struct AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV;

/// When [`check`](crate::check) tests a written type under rule V whose
/// outermost name is one of the wrappers it was given, the name it tests shall
/// be the one that wrapper holds.
#[derive(Spec)]
pub struct AGivenWrapperIsUnwrappedBeforeRuleVTestsTheName;

/// When [`check`](crate::check) tests a written type under rule V, the name it
/// compares shall be the one the source wrote, rather than one a `use` or a type
/// alias would resolve it to.
#[derive(Spec)]
pub struct RuleVTestsTheNameTheSourceWroteAndResolvesNothing;

// ---- The signature tokens rule V reads and slice 19 consumes ----------------

/// For every function of the crate it is given, [`signatures`](crate::signatures)
/// shall carry that function's parameter and return type tokens, whether
/// [`classify`](crate::classify) answers flow or leaf for it.
#[derive(Spec)]
pub struct EveryFunctionOfTheCrateHasItsSignatureTokens;

/// When [`signatures`](crate::signatures) reads a function returning a `Result`,
/// the tokens it carries for that return shall be the whole written type, and
/// not the `Ok` type alone.
#[derive(Spec)]
pub struct TheReturnTokensAreTheWholeWrittenTypeAndNotTheOkTypeAlone;

// ---- What the pass does where it cannot answer ------------------------------

/// When [`classify`](crate::classify) is given a source file `syn` cannot parse,
/// it shall report a [`Finding`](crate::Finding) against that file, rather than
/// ending the pass or passing the file over.
#[derive(Spec)]
pub struct AFileSynCannotParseIsAFindingAndNotAPanic;

/// When [`classify`](crate::classify) is given a crate that has no `src`
/// directory, the [`Classification`](crate::Classification) it answers with
/// shall be empty rather than a refusal, a member with nothing to classify being
/// no violation.
#[derive(Spec)]
pub struct ACrateWithNoSrcDirectoryIsTheEmptyClassification;

/// When [`classify`](crate::classify) reads a module the source gates with
/// `#[cfg]`, the [`Shape`](crate::Shape) of each function that module holds
/// shall be the one its written tokens give, the pass evaluating no gate.
#[derive(Spec)]
pub struct ACfgGatedModuleIsClassifiedAsWritten;

/// When [`classify`](crate::classify) reads a module declaration whose
/// `#[path]` attribute names no string literal, it shall report a
/// [`Finding`](crate::Finding) naming that module unreachable, coverage the pass
/// cannot read being coverage it must not claim.
#[derive(Spec)]
pub struct APathAttributeThatIsNoLiteralIsReportedUnreachable;
