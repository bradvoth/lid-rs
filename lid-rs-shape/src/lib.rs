#![doc = include_str!("lld.md")]

pub mod spec;

mod function;
mod rule;
mod signature;
mod source;
mod verdict;
mod violations;

use std::path::{Path, PathBuf};

use lid_rs::implements;

/// One function's verdict, as the source wrote it.
///
/// Four of its fields are the verdict a reader wants — the name, whether it is
/// flow, why not, and its dispatch arity — and three more are what a rule
/// reads. Rule B is stated over a `pub fn` in a slice's `mod.rs`, so the file a
/// function was read from and whether its declaration says `pub` have to travel
/// with the verdict: nothing else the caller of [`check`] hands over could
/// supply them. The mark is the third addition and the one with a claim of its
/// own: a public leaf that carries `#[leaf]` is allowed *and counted*, and the
/// count reaches a reader only through the [`Classification`] this shape sits
/// in.
///
/// Nothing here is resolved. The name is the one the source wrote, the file is
/// the one the tokens came from, and a `#[cfg]` gate over the module is not
/// evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(spec::TheShapeOfAMarkedFunctionCarriesTheMarkSoItIsCounted)]
pub struct Shape {
    /// The file the function's tokens were read from.
    pub file: PathBuf,
    /// The function's name, as the source wrote it.
    pub function: String,
    /// Whether the declaration says `pub`, which is half of what rule B is
    /// stated over.
    pub public: bool,
    /// Whether the body satisfies every one of F1 through F6.
    pub flow: bool,
    /// The lowest-numbered rule of F1 through F6 the body failed, as a number
    /// from 1 to 6, and none for a body that failed none — so a report names
    /// the rule rather than the verdict.
    pub first_failed: Option<u8>,
    /// How many arms the one decision structure the body holds routes among,
    /// and 0 for a body that holds none. This is the arity rule A compares
    /// against the arm count it is given.
    pub dispatch_arity: usize,
    /// Whether the source marks the function `#[leaf]` — rule B's escape,
    /// carried here so that the leaves the rule allowed can be counted.
    pub marked_leaf: bool,
}

/// Every [`Shape`] of one crate: the whole a caller serialises as
/// `shape-classify.json`, and the value [`check`] reads its three rules over.
///
/// Serialisable rather than serialised, and by construction: every field of
/// every part is public and plain, so a caller renders it in whatever schema it
/// writes. This crate derives no serialisation of its own — it knows no
/// workspace path, writes no file, and takes no dependency that would let it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Classification {
    /// One shape per function the pass read, in the order it read them, and
    /// empty for a crate with nothing to classify.
    pub shapes: Vec<Shape>,
}

/// One function's written parameter and return type tokens.
///
/// Written, not resolved: the tokens are the ones the source spelled, never
/// what a `use` or a type alias would turn them into. Both positions are kept
/// whole — the return is the whole written type and not the `Ok` type dug out
/// of it — because a consumer that wants the `Ok` type can read it from the
/// whole, and one that wants the whole cannot recover it from a part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The file the function's tokens were read from.
    pub file: PathBuf,
    /// The function's name, as the source wrote it.
    pub function: String,
    /// One entry per parameter, each the type tokens the source wrote for it,
    /// in declaration order.
    pub parameters: Vec<String>,
    /// The return type tokens the source wrote, whole, and none for a function
    /// declared with no `->` at all.
    pub returns: Option<String>,
}

/// One thing the pass has to say about one place in one crate's source.
///
/// The variant is the rule, and the fields are the function and what the pass
/// saw. Two of the five name no function: a file `syn` cannot parse and a
/// module declaration the pass cannot follow are gaps in the coverage of every
/// rule at once, and a gap a check does not report is coverage it claimed
/// without reading. They are findings because that is the only way the reading
/// reaches a caller — this crate writes no file — and they are variants of
/// their own because there is no function to name them against.
///
/// Nothing here says what a finding costs. Whether one is fatal is read from
/// configuration by the caller that owns the exit status.
#[derive(Debug, Clone, PartialEq, Eq)]
#[implements(
    spec::AFileSynCannotParseIsAFindingAndNotAPanic,
    spec::APathAttributeThatIsNoLiteralIsReportedUnreachable,
)]
pub enum Finding {
    /// Rule A: a leaf that routes among as many kinds as the arm count the
    /// check was given, or more.
    RuleA {
        /// The file the function was read from.
        file: PathBuf,
        /// The function the rule is about.
        function: String,
        /// What the pass saw — the arity it routes among, and the rule of F1
        /// through F6 that made it a leaf.
        saw: String,
    },
    /// Rule B: a public leaf in a slice's `mod.rs` carrying no `#[leaf]` mark.
    RuleB {
        /// The file the function was read from, which is one of the slice
        /// `mod.rs` files the check was given.
        file: PathBuf,
        /// The function the rule is about.
        function: String,
        /// What the pass saw — the rule of F1 through F6 that made it a leaf.
        saw: String,
    },
    /// Rule V: a flow function whose written signature uses a name the rule
    /// denies, in a parameter or in the `Ok` type of its return.
    RuleV {
        /// The file the function was read from.
        file: PathBuf,
        /// The function the rule is about.
        function: String,
        /// What the pass saw — the denied name, the position it was written
        /// in, and the wrappers unwrapped to reach it.
        saw: String,
    },
    /// A source file `syn` could not parse: no verdict for any function it
    /// holds, reported rather than skipped, and never a panic.
    Unparsable {
        /// The file that could not be parsed.
        file: PathBuf,
        /// What the pass saw — the parse error, at the place it was raised.
        saw: String,
    },
    /// A module declaration whose `#[path]` attribute names no string literal,
    /// so the file it stands for cannot be read.
    Unreachable {
        /// The file the module declaration was read from.
        file: PathBuf,
        /// The module the declaration names.
        module: String,
        /// What the pass saw — the attribute, as the source wrote it.
        saw: String,
    },
}

/// Every function of one crate, flow or leaf, beside the findings the reading
/// raised on its way.
///
/// `crate_root` is the directory a crate's `Cargo.toml` sits in; the functions
/// are the ones its `src` holds, so a crate whose `src` does not exist is
/// answered with an empty [`Classification`] and no finding at all — a member
/// with nothing to classify is not a violation.
///
/// `allow_macros` is the set of macro names a body may invoke and stay flow,
/// handed in by the caller. This crate reads no configuration and keeps no
/// default of its own, so there is no default here for a project's settings to
/// disagree with.
///
/// A body is flow when it satisfies all six of F1 through F6, and a leaf
/// otherwise:
///
/// - **F1** every statement is `let pat = call(...)?;`, `call(...)?;`, or the
///   tail call;
/// - **F2** at most one decision structure — one `match`, or one two-way `if` —
///   and every arm a single call;
/// - **F3** arguments are paths, field accesses, references, or accessor
///   chains;
/// - **F4** no closures or blocks;
/// - **F5** no literals but `()`;
/// - **F6** no macros but `allow_macros`.
///
/// The six are one definition, so F1 and F4 admit what F2 admits: the decision
/// structure F2 allows is a statement F1 does not reject and its arms are
/// blocks F4 does not reject, and without that reading F2's allowance could
/// never be taken. A leaf's [`Shape`] names the lowest-numbered rule it failed,
/// which is what a report names instead of the verdict.
///
/// The classification is derived from the body and never declared: a mark that
/// said "this function is routing" would be evaded by omitting it. `#[leaf]` is
/// carried on the [`Shape`] as a mark and changes no verdict.
///
/// Nothing is resolved, including `cfg`: a module the source gates is read for
/// the tokens it wrote, and a function a macro would generate is not there to
/// read. What can be seen of a macro is its invocation in the enclosing body,
/// which is F6's.
///
/// The findings are answered beside the classification rather than inside it.
/// A [`Classification`] is every shape and nothing else, because that is the
/// artifact a caller writes for the consumers that want the verdicts; the
/// findings belong with [`check`]'s, in the findings report the same caller
/// writes.
///
/// The answer is composed of three: which files the crate holds, which
/// functions those files hold, and the verdict for one function's body. This
/// pairing is what the claims below are about — that a reading which cannot be
/// made reaches the caller as a finding rather than as a panic or as silence,
/// and that a crate with nothing to classify is answered with the empty
/// classification. Which rule a body failed is answered a layer down, one item
/// per rule.
#[implements(
    spec::AFileSynCannotParseIsAFindingAndNotAPanic,
    spec::ACrateWithNoSrcDirectoryIsTheEmptyClassification,
    spec::APathAttributeThatIsNoLiteralIsReportedUnreachable,
)]
pub fn classify(crate_root: &Path, allow_macros: &[String]) -> (Classification, Vec<Finding>) {
    let (sources, findings) = source::sources(crate_root);
    let shapes = sources
        .iter()
        .flat_map(|(file, parsed)| function::functions_in(file, parsed))
        .map(|function| verdict::shape_of(&function, allow_macros))
        .collect();
    (Classification { shapes }, findings)
}

/// Every function of one crate with the type tokens its signature wrote —
/// every function, whether [`classify`] answered flow or leaf for it, and both
/// positions of each, whole.
///
/// `crate_root` is the directory a crate's `Cargo.toml` sits in, as it is for
/// [`classify`], and the functions are the same ones: a reader that joined the
/// two answers on a file and a name would find one entry here for every
/// [`Shape`] there.
///
/// The breadth is the point. Rule V reads these tokens for the flow functions
/// alone, but a conformance check reads them for every implementer of a claim,
/// and a return narrowed to the `Ok` type of a `Result` would leave a consumer
/// no way back to what the source actually wrote.
///
/// The crate is read the way [`classify`] reads it and the functions are found
/// the way it finds them, which is what makes the two answers joinable at all.
/// A file the reading raises a finding against is a file neither answer has an
/// entry from; the findings themselves are [`classify`]'s to answer, because a
/// caller asking for tokens is not asking a second time about the reading.
#[implements(spec::EveryFunctionOfTheCrateHasItsSignatureTokens)]
pub fn signatures(crate_root: &Path) -> Vec<Signature> {
    let (sources, _) = source::sources(crate_root);
    sources
        .iter()
        .flat_map(|(file, parsed)| function::functions_in(file, parsed))
        .map(|function| signature::signature_of(&function))
        .collect()
}

/// Rules A, B and V over one crate's classification: every violation as a
/// [`Finding`], and nothing else.
///
/// **No level, and no exit status.** Nothing here decides whether a finding is
/// fatal, counts one against a threshold, or grades one over another. The rules
/// report and the caller judges: it is the caller that reads a project's
/// configuration, and this crate that reads none.
///
/// - **A** — a leaf whose dispatch arity reaches `dispatch_arms` is a finding.
///   Routing among that many kinds is a flow decision, and a leaf that makes
///   one is doing work in the same function.
/// - **B** — a public leaf in one of `slice_mods` is a finding unless it
///   carries `#[leaf]`. The mark is the escape, and a marked leaf is a finding
///   for no rule.
/// - **V** — a flow function whose parameter, or whose `Ok` type, is written as
///   a name the rule denies is a finding. The denied names are `String`,
///   `&str`, `bool`, the integer types, the float types and `char`, and no
///   other name; a type written as one of `wrappers` is unwrapped and the name
///   it holds is the one tested. The name tested is the one the source wrote:
///   following a `use` or a type alias to another name would be resolution,
///   which this pass does not do.
///
/// The rule V clause is narrower than the design it comes from, which states
/// rule V as an allow-list of vocabulary types and admits nothing else. An
/// allow-list needs the set of vocabulary names, which no registry in reach
/// holds; a deny-list of the primitives is what a pass that resolves nothing
/// can decide today, and it misses every non-vocabulary type that is not a
/// primitive.
///
/// The workspace knowledge is all injected, none of it read: `slice_mods` is
/// the set of files that are a *slice's* `mod.rs`, which is a question about a
/// project's layout that this pass cannot answer — it can see that a file is
/// named `mod.rs` and no more — and `dispatch_arms` and `wrappers` are settings
/// of the project being checked.
///
/// `classification` and `signatures` are two answers about one crate, joined on
/// a file and a function name.
///
/// Every shape is put to every rule, and the findings of all three rules over
/// all of them are the answer: no rule is tried only where another was silent,
/// and one function can be a finding under more than one rule. The claims here
/// are the four that say a violation *shall be reported* — dropping one is this
/// composition's wrong answer — while what makes a violation, and what name a
/// rule tests, is answered a layer down, one item per decision.
#[implements(
    spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA,
    spec::AnUnmarkedPublicLeafInASliceModIsAFindingUnderRuleB,
    spec::AFlowParameterWrittenAsADeniedNameIsAFindingUnderRuleV,
    spec::AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV,
)]
pub fn check(
    classification: &Classification,
    signatures: &[Signature],
    slice_mods: &[PathBuf],
    dispatch_arms: usize,
    wrappers: &[String],
) -> Vec<Finding> {
    classification
        .shapes
        .iter()
        .flat_map(|shape| violations::findings_for(shape, signatures, slice_mods, dispatch_arms, wrappers))
        .collect()
}

#[cfg(test)]
mod intent_graph {
    //! This crate's instance of the graph checks (README §4.2).
    lid_rs::intent_graph!();
}
