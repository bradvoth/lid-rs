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

/// One function's written signature: its parameter and return type tokens,
/// and the owner it was read under.
///
/// Written, not resolved: the tokens are the ones the source spelled, never
/// what a `use` or a type alias would turn them into. Both positions are kept
/// whole — the return is the whole written type and not the `Ok` type dug out
/// of it — because a consumer that wants the `Ok` type can read it from the
/// whole, and one that wants the whole cannot recover it from a part. The
/// owner is spelled the same way — the `impl` block's self type as the source
/// wrote it, the `trait`'s name, or none for a free function — so a `Self` in
/// either type position stays `Self` and the owner is what explains it.
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
    /// Whose function it is: the self type tokens of the `impl` block it was
    /// read from as the source wrote them, the name of the `trait` block it was
    /// read from, and none for a free function. Written like the two type
    /// positions, so a `Self` in either of them is explained by this and
    /// resolved by nothing.
    pub owner: Option<String>,
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
    /// A source file the pass could not read as Rust: no verdict for any
    /// function it holds, reported rather than skipped, and never a panic.
    ///
    /// A file that could not be read at all is the same gap as one that could
    /// not be parsed — no function of it reaches any rule — so it is the same
    /// finding, and what was seen says which of the two happened.
    Unparsable {
        /// The file that could not be read as Rust.
        file: PathBuf,
        /// What the pass saw — what `syn` said of the tokens, or what the
        /// filesystem said of the file.
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
/// [`Shape`] there. Each entry also carries the owner of its function — the
/// `impl` block's self type as written, the `trait`'s name, or none for a free
/// function — which is what lets a consumer read a written `Self`.
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

#[cfg(test)]
mod tests {
    //! The claims the crate's two entry points answer, put to them there.
    //!
    //! Five of these are claims a leaf below decides and a composition here
    //! could still drop: [`check`] answering an empty vector, and
    //! [`signatures`] answering an empty one, are wrong answers that falsify
    //! the claim as surely as a wrong decision does. They are validated here
    //! rather than at the leaf so that both the decision and its propagation
    //! are covered by the one test the claim is allowed.
    //!
    //! The four claims about what the pass does where it cannot answer are
    //! here for the same reason and one more: a finding raised in place of a
    //! panic, and an empty classification in place of a refusal, are only
    //! observable at the entry point.
    //!
    //! [`check`] takes plain data, so its fixtures are written rather than
    //! parsed; [`classify`] and [`signatures`] take a crate root, so theirs are
    //! files under a scratch directory, made the way this workspace's other
    //! slices make one. No fixture holds a `#[path]` naming another file, so
    //! nothing here walks in a circle.

    use lid_rs::validates;

    use super::*;

    /// The file a fixture's slice keeps its public functions in.
    const SLICE_MOD: &str = "src/hello/mod.rs";

    /// A file of the fixture's slice that is no slice `mod.rs`.
    const WORK: &str = "src/hello/work.rs";

    /// A leaf as the classification carries one: not public, not marked, and
    /// routing among nothing.
    fn leaf(function: &str, file: &str) -> Shape {
        Shape {
            file: PathBuf::from(file),
            function: function.to_string(),
            public: false,
            flow: false,
            first_failed: Some(1),
            dispatch_arity: 0,
            marked_leaf: false,
        }
    }

    /// A flow node as the classification carries one.
    fn flow_node(function: &str, file: &str) -> Shape {
        Shape { public: true, flow: true, first_failed: None, ..leaf(function, file) }
    }

    /// One function's written signature, as a fixture spells it.
    fn signature(function: &str, parameters: &[&str], returns: Option<&str>) -> Signature {
        Signature {
            file: PathBuf::from(WORK),
            function: function.to_string(),
            parameters: parameters.iter().map(|written| (*written).to_string()).collect(),
            returns: returns.map(str::to_string),
            owner: None,
        }
    }

    /// The wrappers README names, as the caller hands them over.
    fn wrappers() -> Vec<String> {
        ["Option", "Result", "Vec", "Box", "Arc"].iter().map(|name| (*name).to_string()).collect()
    }

    /// The rule each finding is under and what it names — a function for the
    /// three rules, and the file or module for the two gaps in coverage, which
    /// name no function at all.
    fn under(finding: &Finding) -> (&'static str, String) {
        match finding {
            Finding::RuleA { function, .. } => ("A", function.clone()),
            Finding::RuleB { function, .. } => ("B", function.clone()),
            Finding::RuleV { function, .. } => ("V", function.clone()),
            Finding::Unparsable { file, .. } => ("unparsable", file.display().to_string()),
            Finding::Unreachable { module, .. } => ("unreachable", module.clone()),
        }
    }

    /// What the findings say, in the order they were answered.
    fn reported(findings: &[Finding]) -> Vec<(&'static str, String)> {
        findings.iter().map(under).collect()
    }

    /// Type tokens without the spacing a printer chooses.
    fn tight(tokens: &str) -> String {
        tokens.replace(' ', "")
    }

    /// A fresh scratch directory, as this workspace's other slices make one.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("lid-rs-shape-tests").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// A crate root holding the files named, each under the directory its path
    /// implies.
    fn crate_with(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let root = scratch(name);
        for (path, source) in files {
            let file = root.join(path);
            std::fs::create_dir_all(file.parent().expect("a fixture path with a directory")).expect("fixture directory");
            std::fs::write(&file, source).expect("fixture source");
        }
        root
    }

    /// A leaf routing among as many kinds as the arm count given is reported
    /// under rule A, as is one routing among more, and one routing among fewer
    /// is not.
    #[test]
    #[validates(spec::ALeafRoutingAmongTheGivenArmCountIsAFindingUnderRuleA)]
    fn a_leaf_routing_among_the_given_arm_count_is_a_finding_under_rule_a() {
        let classification = Classification {
            shapes: vec![
                Shape { dispatch_arity: 3, ..leaf("dispatches", WORK) },
                Shape { dispatch_arity: 1, ..leaf("chooses", WORK) },
                Shape { dispatch_arity: 4, ..leaf("dispatches_wider", WORK) },
            ],
        };
        assert_eq!(
            reported(&check(&classification, &[], &[], 3, &wrappers())),
            vec![("A", "dispatches".to_string()), ("A", "dispatches_wider".to_string())],
            "the arm count given is the count the rule is reached at, not the count it is met at exactly",
        );
        let source = "pub fn routes(kind: Kind) -> Outcome {\n\
                          match kind {\n\
                              Kind::A => first(kind),\n\
                              Kind::B => { let read = source(kind)?; answer(read) }\n\
                              Kind::C => third(kind),\n\
                          }\n\
                      }\n\
                      pub fn chained(kind: Kind) -> Outcome {\n\
                          if first(kind) { one(kind) } else if second(kind) { two(kind) } else { three(kind) }\n\
                      }\n";
        let root = crate_with("rule-a-arity", &[("src/lib.rs", source)]);
        assert_eq!(
            reported(&check(&classify(&root, &[]).0, &[], &[], 3, &wrappers())),
            vec![("A", "routes".to_string()), ("A", "chained".to_string())],
            "the arity the rule is given is the one the pass read from the body, an `if` chain's counted as a `match`'s is",
        );
    }

    /// An unmarked public leaf in one of the slice `mod.rs` files given is
    /// reported under rule B; a public leaf in a file that is no slice's
    /// `mod.rs` is not.
    #[test]
    #[validates(spec::AnUnmarkedPublicLeafInASliceModIsAFindingUnderRuleB)]
    fn an_unmarked_public_leaf_in_a_slice_mod_is_a_finding_under_rule_b() {
        let classification = Classification {
            shapes: vec![
                Shape { public: true, ..leaf("counted", SLICE_MOD) },
                Shape { public: true, ..leaf("elsewhere", WORK) },
            ],
        };
        let mods = vec![PathBuf::from(SLICE_MOD)];
        assert_eq!(
            reported(&check(&classification, &[], &mods, 3, &wrappers())),
            vec![("B", "counted".to_string())],
            "the rule is stated over the slice `mod.rs` files the caller gave, and its finding reaches the caller",
        );
        let held = "pub fn counted() { let total = left + right; answer(total) }\n\
                    fn hidden() { let total = left + right; answer(total) }\n";
        let root = crate_with("rule-b-public", &[("src/lib.rs", "pub mod hello;\n"), (SLICE_MOD, held)]);
        assert_eq!(
            reported(&check(&classify(&root, &[]).0, &[], &[root.join(SLICE_MOD)], 3, &wrappers())),
            vec![("B", "counted".to_string())],
            "the visibility the rule is given is the one the pass read from the declaration, so the private leaf beside it is no finding",
        );
    }

    /// A flow parameter written as a denied name is reported under rule V, and
    /// one written as a name outside the deny clause is not.
    #[test]
    #[validates(spec::AFlowParameterWrittenAsADeniedNameIsAFindingUnderRuleV)]
    fn a_flow_parameter_written_as_a_denied_name_is_a_finding_under_rule_v() {
        let classification = Classification { shapes: vec![flow_node("reads", WORK), flow_node("routes", WORK)] };
        let signatures = vec![signature("reads", &["String"], None), signature("routes", &["Report"], None)];
        assert_eq!(
            reported(&check(&classification, &signatures, &[], 3, &wrappers())),
            vec![("V", "reads".to_string())],
            "the parameter position is read and its finding reaches the caller",
        );
    }

    /// A flow `Ok` type written as a denied name is reported under rule V, and
    /// one written as a name outside the deny clause is not.
    #[test]
    #[validates(spec::AFlowOkTypeWrittenAsADeniedNameIsAFindingUnderRuleV)]
    fn a_flow_ok_type_written_as_a_denied_name_is_a_finding_under_rule_v() {
        let classification = Classification { shapes: vec![flow_node("reads", WORK), flow_node("routes", WORK)] };
        let signatures = vec![
            signature("reads", &[], Some("Result<String, Error>")),
            signature("routes", &[], Some("Result<Report, Error>")),
        ];
        assert_eq!(
            reported(&check(&classification, &signatures, &[], 3, &wrappers())),
            vec![("V", "reads".to_string())],
            "the `Ok` type is read out of the whole written return, and its finding reaches the caller",
        );
    }

    /// The name rule V compares is the one the source wrote: a parameter
    /// written `Text`, which the fixture's crate declares as `type Text =
    /// String;`, is no finding, while one written `String` is.
    #[test]
    #[validates(spec::RuleVTestsTheNameTheSourceWroteAndResolvesNothing)]
    fn rule_v_tests_the_name_the_source_wrote_and_resolves_nothing() {
        let classification = Classification { shapes: vec![flow_node("aliased", WORK), flow_node("written", WORK)] };
        let signatures = vec![signature("aliased", &["Text"], None), signature("written", &["String"], None)];
        assert_eq!(
            reported(&check(&classification, &signatures, &[], 3, &wrappers())),
            vec![("V", "written".to_string())],
            "an alias is not chased to the name it stands for: resolving one is the line the carve-out does not cross",
        );
    }

    /// Every function of the crate carries its parameter tokens, whether the
    /// classification answers flow or leaf for it.
    #[test]
    #[validates(spec::EveryFunctionOfTheCrateHasItsSignatureTokens)]
    fn every_function_of_the_crate_has_its_signature_tokens() {
        let source = "pub fn routes(input: &Report) -> Outcome { answer(input) }\n\
                      pub fn works(left: usize, right: usize) -> usize { left + right }\n";
        let root = crate_with("signature-tokens", &[("src/lib.rs", source)]);
        let carried: Vec<(String, String)> =
            signatures(&root).iter().map(|read| (read.function.clone(), tight(&read.parameters.join(",")))).collect();
        assert_eq!(
            carried,
            vec![("routes".to_string(), "&Report".to_string()), ("works".to_string(), "usize,usize".to_string())],
            "the flow node and the leaf both, each with the parameter tokens its declaration wrote",
        );
    }

    /// Every signature carries the block its function was read from: none for
    /// a free function, the `impl` block's self type as the source wrote it —
    /// generic arguments and all — and the `trait`'s name for a method the
    /// trait gives a body to. A trait `impl`'s owner is its self type, not the
    /// trait, and a trait method declared without a body is no function at
    /// all, so nothing is carried for it.
    #[test]
    #[validates(spec::ASignaturesOwnerIsTheBlockItWasReadFrom)]
    fn a_signatures_owner_is_the_block_it_was_read_from() {
        let source = "pub fn spells(input: &Report) -> Outcome { answer(input) }\n\
                      impl<T> Shape<T> { pub fn method(&self) -> Self { answer(self) } }\n\
                      pub trait Render { fn render(&self) -> String { answer(self) } fn declared(&self); }\n\
                      impl Render for Report { fn render(&self) -> String { answer(self) } }\n";
        let root = crate_with("signature-owner", &[("src/lib.rs", source)]);
        let carried: Vec<(String, Option<String>)> =
            signatures(&root).iter().map(|read| (read.function.clone(), read.owner.as_deref().map(tight))).collect();
        assert_eq!(
            carried,
            vec![
                ("spells".to_string(), None),
                ("method".to_string(), Some("Shape<T>".to_string())),
                ("render".to_string(), Some("Render".to_string())),
                ("render".to_string(), Some("Report".to_string())),
            ],
            "the free function carries no owner, the `impl` method its block's written self type, the trait's bodied method the trait's name, and the trait `impl`'s method the self type rather than the trait",
        );
    }

    /// A file `syn` cannot parse is answered as a finding, the rest of the
    /// pass running on — never a panic and never a silent skip.
    #[test]
    #[validates(spec::AFileSynCannotParseIsAFindingAndNotAPanic)]
    fn a_file_syn_cannot_parse_is_a_finding_and_not_a_panic() {
        let root = crate_with("unparsable", &[("src/lib.rs", "pub fn broken( {\n")]);
        let (classification, findings) = classify(&root, &[]);
        let rules: Vec<&str> = reported(&findings).iter().map(|(rule, _)| *rule).collect();
        assert_eq!(
            (classification.shapes.len(), rules),
            (0, vec!["unparsable"]),
            "a gap in the coverage of every rule at once is reported, and the reading returns",
        );
    }

    /// A crate with no `src` directory is answered with the empty
    /// classification and no finding: a member with nothing to classify is no
    /// violation.
    #[test]
    #[validates(spec::ACrateWithNoSrcDirectoryIsTheEmptyClassification)]
    fn a_crate_with_no_src_directory_is_the_empty_classification() {
        let manifest = ("Cargo.toml", "[package]\nname = \"member\"\n");
        let bare = crate_with("no-src", &[manifest]);
        let holding = crate_with("with-src", &[manifest, ("src/lib.rs", "pub fn works() { let total = left + right; answer(total) }\n")]);
        assert_eq!(
            (classify(&bare, &[]), classify(&holding, &[]).0.shapes.len()),
            ((Classification::default(), Vec::new()), 1),
            "nothing to classify is answered with nothing and with no refusal, which is not how a crate holding a function is answered",
        );
    }

    /// A module the source gates with `#[cfg]` is classified as written,
    /// whether the gate is over a file module or an inline one: the pass reads
    /// tokens and evaluates no gate.
    #[test]
    #[validates(spec::ACfgGatedModuleIsClassifiedAsWritten)]
    fn a_cfg_gated_module_is_classified_as_written() {
        let root = crate_with(
            "cfg-gated",
            &[
                ("src/lib.rs", "#[cfg(test)]\nmod inline { pub fn checks() { answer(read) } }\n\n#[cfg(feature = \"extra\")]\nmod extra;\n"),
                ("src/extra.rs", "pub fn gated() { answer(read) }\n"),
            ],
        );
        let read: Vec<String> = classify(&root, &[]).0.shapes.iter().map(|shape| shape.function.clone()).collect();
        assert_eq!(
            read,
            vec!["checks".to_string(), "gated".to_string()],
            "the gated inline module and the gated file module are both read, the gate evaluated for neither",
        );
    }

    /// A module declaration whose `#[path]` names no string literal is
    /// reported unreachable: coverage the pass cannot read is coverage it must
    /// not claim.
    #[test]
    #[validates(spec::APathAttributeThatIsNoLiteralIsReportedUnreachable)]
    fn a_path_attribute_that_is_no_literal_is_reported_unreachable() {
        let source = "#[path = concat!(\"gener\", \"ated.rs\")]\nmod generated;\n";
        let root = crate_with("no-literal-path", &[("src/lib.rs", source)]);
        assert_eq!(
            reported(&classify(&root, &[]).1),
            vec![("unreachable", "generated".to_string())],
            "the module the pass cannot follow is named, rather than passed over",
        );
    }
}
