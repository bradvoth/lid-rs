#![doc = include_str!("lld.md")]

use crate::claim::Pattern;
use crate::registry::SpecMeta;
use lid_rs::implements;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::sync::Mutex;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::subscriber::Interest;
use tracing::{Event, Metadata};

pub mod spec;

/// The span target the emission writes and [`Capture`]'s `enabled` filters on.
///
/// Spelled once. `lid-rs-macros` cannot see this constant — it links into no
/// binary that links this crate — so the emission repeats the literal, and a
/// second spelling anywhere else is a validator whose tree is silently empty.
pub const TARGET: &str = "lid";

/// The unit separator the `lid.claims` field joins on and [`cited_claims`]
/// splits at.
///
/// Spelled once and public, because the emission that writes the field lives in
/// `lid-rs-macros` and cannot see a constant it does not name — and two
/// spellings of it are not a compile error but a silent empty join.
pub const SEPARATOR: char = '\u{1f}';

/// The level the emission opens its spans at: README §7's own default
/// (`level = "debug"`).
///
/// A constant rather than a setting, because the
/// `[workspace.metadata.lid_rs.runtime]` table it belongs in is read by a
/// private reader in `cargo-lid-rs` that handles only flat keys — the LLD's
/// Deferred 5, and the fourth slice to record it.
pub const SPAN_LEVEL: tracing::Level = tracing::Level::DEBUG;

/// The ramp checks 23 and 24 arrive at: a finding prints and the test stands.
///
/// The flip to [`Ramp::Deny`] is the LLD's Deferred 8 and is what makes the two
/// checks gate anything. It cannot happen while the ubiquitous rule wants
/// recorded inputs that no span carries yet.
pub const LEVEL: Ramp = Ramp::Warn;

/// How many citing spans [`enough_spans`] wants of a ubiquitous claim: 16,
/// README's own default for `min_inputs`.
///
/// Fixed here for the same reason as [`SPAN_LEVEL`].
pub const MIN_INPUTS: usize = 16;

/// README §7's ramp: what a finding does to the validator that produced it.
///
/// A type of this slice's own, and deliberately **not** [`tracing::Level`],
/// which has neither variant: a level names a span's verbosity and this names
/// a check's severity. The two stand one line apart in this module, so a shared
/// name would be read as a shared meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ramp {
    /// The finding and the rendered tree are printed; the test does not fail.
    Warn,
    /// The finding fails the test.
    Deny,
}

/// One captured span, as data.
///
/// Parentage is an index rather than a nesting, because a subscriber builds its
/// tree incrementally — a span is recorded when it opens, before any child of
/// it exists — and a nested type cannot be built that way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// The name the span was opened with, from its metadata.
    pub name: String,
    /// The index into [`Trace::spans`] of the span that was entered when this
    /// one was opened, or `None` where none was.
    pub parent: Option<usize>,
    /// The claims the `lid.claims` field named, split at the unit separator
    /// `'\x1f'` — each one a cited claim's `Spec::NAME`.
    pub claims: Vec<String>,
    /// What the `lid.outcome` field recorded before the guard dropped.
    ///
    /// `None` means the body unwound: the emission records the outcome after
    /// the body, so a panic skips the record while the span still closes, and a
    /// unit return records `()` rather than nothing. That is the one inference
    /// in the wire contract, and what [`no_panics`] reads.
    pub outcome: Option<String>,
    /// Every other field the span carried — `lid.noun.<n>`, `lid.feature`,
    /// `lid.state` — as name and value, in the order they were recorded.
    pub fields: Vec<(String, String)>,
}

/// A validator's whole span tree, in creation order.
///
/// The argument every check takes and the value [`render`] prints. Plain data:
/// nothing downstream of [`captured`] reaches for a subscriber, a thread-local
/// or the clock, which is what lets Phase 5 hand every check a tree it built by
/// hand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    /// The spans, in the order they were opened. A [`Span::parent`] is an index
    /// into this vector.
    pub spans: Vec<Span>,
}

/// The [`tracing::Subscriber`] that builds a [`Trace`], and the only item in
/// the slice that touches `tracing`'s API.
///
/// Its recording state sits behind a [`Mutex`] because `Subscriber`'s methods
/// take `&self`, and because [`captured`] reads the tree back out after the
/// dispatcher that owned it is dropped.
#[derive(Debug, Default)]
pub struct Capture {
    /// What the methods below have recorded so far.
    state: Mutex<Recording>,
}

/// [`Capture`]'s state while it is installed: the tree so far, and the stack
/// that gives each new span its parent.
///
/// Private, because it is the one part of the capture that is not a published
/// promise: [`captured`] reads the spans back out and answers a [`Trace`], and
/// nothing outside this module names the stack at all.
#[derive(Debug, Default)]
struct Recording {
    /// The spans recorded so far, in creation order — the future
    /// [`Trace::spans`].
    spans: Vec<Span>,
    /// The indices of the spans currently entered, innermost last. The parent
    /// of a span opened now is the last of these.
    stack: Vec<usize>,
}

/// The [`tracing::field::Visit`] implementation that writes a recorded field
/// onto the [`Span`] it belongs to.
///
/// Both ways a field can arrive end here: the fields a span is opened with,
/// which [`Capture`]'s `new_span` hands over, and the ones recorded afterwards,
/// which [`Capture`]'s `record` does. It is the only place a `tracing` field value
/// becomes a [`String`], so the `Debug`-versus-`str` distinction is decided
/// once rather than at each of those two call sites.
pub struct Fields<'a> {
    /// The span being written to: the one just opened, or the one the recorded
    /// field names.
    span: &'a mut Span,
}

impl Fields<'_> {
    /// Where one recorded field lands on the span, and the only place the wire
    /// contract's three destinations are spelled.
    ///
    /// `lid.claims` is the cited claims, split at the unit separator by
    /// [`cited_claims`]; `lid.outcome` is what the body closed with; and every
    /// other field the contract names — `lid.noun.<n>`, `lid.feature`,
    /// `lid.state` — is kept under the name it was recorded with, for the rules
    /// that read it.
    #[implements(
        spec::TheClaimsFieldIsSplitAtTheUnitSeparator,
        spec::TheOutcomeFieldIsCarriedAsTheSpansOutcome,
        spec::AFieldRecordedAfterCreationReachesItsSpan,
        spec::ARecordedSpanCarriesTheFieldsItWasOpenedWith,
    )]
    fn take(&mut self, name: &str, value: String) {
        match name {
            "lid.claims" => self.span.claims = cited_claims(&value),
            "lid.outcome" => self.span.outcome = Some(value),
            _ => self.span.fields.push((name.to_string(), value)),
        }
    }
}

impl Visit for Fields<'_> {
    /// A field of any other type, through its [`Debug`] rendering — the
    /// fallback every `record_*` the trait provides arrives at.
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.take(field.name(), format!("{value:?}"));
    }

    /// A string field, taken as it stands rather than through [`Debug`], which
    /// would quote it: `lid.claims` reaches [`cited_claims`] as a string, and a
    /// quoted one would split into a first claim carrying a quote and a last
    /// one carrying another.
    fn record_str(&mut self, field: &Field, value: &str) {
        self.take(field.name(), value.to_string());
    }
}

/// The claims a span's `lid.claims` field names, split at the unit separator
/// `'\x1f'`.
///
/// The only place the separator is spelled on this side of the wire. The
/// emission joins the cited claims' `Spec::NAME`s with it — a path may hold no
/// comma, which is why it is not one — and a second spelling here would be a
/// join that silently matches nothing at all. A field naming one claim answers
/// one claim.
#[implements(spec::TheClaimsFieldIsSplitAtTheUnitSeparator)]
pub fn cited_claims(field: &str) -> Vec<String> {
    field.split(SEPARATOR).map(str::to_string).collect()
}

// `register_callsite` can carry no `#[implements]`, because `#[implements]`
// opens a span and `tracing` calls this method *while it holds the global
// callsite registry's lock* — the span the citation would open needs that same
// lock to register its own callsite, and the run stops. Measured: the workspace
// suite completes in seconds single-threaded and never completes in parallel.
// So the one claim about it is cited by containment, as the registry slice's
// enumeration claim is (`lid-rs/src/registry/mod.rs:54`).
lid_rs::implements_module!(spec::TheCapturesInterestInACallsiteIsSometimes);

impl tracing::Subscriber for Capture {
    /// The target filter: `false` for every target that is not [`TARGET`], so
    /// an application's own spans never enter a validator's tree.
    #[implements(
        spec::ASpanAtTheLidTargetEntersTheTrace,
        spec::ASpanAtAnotherTargetIsNotCaptured,
        spec::ASpanOpenedInsideTheCaptureIsNotCaptured,
    )]
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.target() == TARGET && metadata.module_path() != Some(module_path!())
    }

    /// The interest in a callsite, answered explicitly rather than derived from
    /// [`Capture`]'s `enabled`.
    ///
    /// The default caches `Interest::never()` per callsite; the spike could not
    /// make that bite under a scoped dispatcher, and the failure it would cause
    /// — an empty tree in a test that runs after another test — is worth one
    /// method to rule out.
    fn register_callsite(&self, _metadata: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }

    /// Records the span: its metadata's name, its parent from the enter-stack,
    /// and the fields it was opened with, through [`Fields`].
    ///
    /// What it answers for is the name it reads, the parent it takes from the
    /// innermost entry of the enter-stack — none, where nothing is entered —
    /// the position it appends the span at, which is what makes the trace the
    /// order its spans were opened in, and that the attributes are handed to a
    /// [`Fields`] at all. Where each of those fields then lands is [`Fields`]'s
    /// answer, and the split of `lid.claims` is [`cited_claims`]'s.
    ///
    /// The [`Id`] it answers with is what [`record`](Capture::record),
    /// [`enter`](Capture::enter) and [`exit`](Capture::exit) are given back, so
    /// it has to name the entry this call appended.
    #[implements(
        spec::ASpanAtTheLidTargetEntersTheTrace,
        spec::ARecordedSpanCarriesTheNameItWasOpenedWith,
        spec::ARecordedSpanCarriesTheFieldsItWasOpenedWith,
        spec::ARecordedSpansParentIsTheSpanThatWasEntered,
        spec::ASpanOpenedWithNoSpanEnteredHasNoParent,
        spec::ASpanOpenedAfterAnExitTakesTheEnclosingParent,
        spec::TheCapturedTraceIsInTheOrderItsSpansWereOpened,
    )]
    fn new_span(&self, attrs: &Attributes<'_>) -> Id {
        let mut recording = self.state.lock().expect("the recording");
        let mut span = Span {
            name: attrs.metadata().name().to_string(),
            parent: recording.stack.last().copied(),
            claims: Vec::new(),
            outcome: None,
            fields: Vec::new(),
        };
        attrs.record(&mut Fields { span: &mut span });
        recording.spans.push(span);
        Id::from_u64(recording.spans.len() as u64)
    }

    /// Takes the fields recorded after a span was opened, through the same
    /// [`Fields`] the fields at creation go through.
    ///
    /// How `lid.outcome` reaches its span at all: the emission reserves the
    /// slot with `field::Empty` at creation and fills it here, before the
    /// guard drops. This method's own answer is which recorded span the fields
    /// belong to — the [`Id`] names it — and where each of them lands there is
    /// [`Fields`]'s.
    #[implements(spec::AFieldRecordedAfterCreationReachesItsSpan)]
    fn record(&self, span: &Id, values: &Record<'_>) {
        let mut recording = self.state.lock().expect("the recording");
        let recorded = span.into_u64() as usize - 1;
        values.record(&mut Fields { span: &mut recording.spans[recorded] });
    }

    /// Nothing: this slice reads no follows-from edge, and README §6.4's tree
    /// has no place to print one.
    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    /// Nothing: this slice reads no event, only spans. A claim is kept by a
    /// body that ran, and a span is what says one did.
    fn event(&self, _event: &Event<'_>) {}

    /// Pushes the span onto the enter-stack, which is what gives every span
    /// opened under it a parent.
    #[implements(spec::ARecordedSpansParentIsTheSpanThatWasEntered)]
    fn enter(&self, span: &Id) {
        let mut recording = self.state.lock().expect("the recording");
        recording.stack.push(span.into_u64() as usize - 1);
    }

    /// Pops the span off the enter-stack, so that the next span opened takes
    /// the enclosing span as its parent rather than this one.
    #[implements(spec::ASpanOpenedAfterAnExitTakesTheEnclosingParent)]
    fn exit(&self, _span: &Id) {
        let mut recording = self.state.lock().expect("the recording");
        recording.stack.pop();
    }
}

/// Runs `body` under a fresh [`Capture`] and answers its [`Trace`] beside what
/// the body did.
///
/// The seam between the emission and everything this slice can test without it:
/// what `#[validates]` expands to, and the one item here that installs a
/// subscriber.
///
/// The body is run inside `catch_unwind(AssertUnwindSafe(..))`, so a validator
/// that failed still has its tree — README §6.4's deliverable is that the tree
/// is **printed on failure**, and a `captured` that unwound with the body would
/// drop precisely the trace that was worth reading. The panic is not swallowed:
/// it is handed back as the `Err` of a [`std::thread::Result`], and the caller
/// resumes it once the checks have printed.
#[implements(
    spec::TheCapturedTraceCarriesTheSpansOpenedInTheClosure,
    spec::TheCapturedTraceCarriesNoEarlierRunsSpans,
    spec::TheCapturedTraceIsInTheOrderItsSpansWereOpened,
)]
pub fn captured<T>(body: impl FnOnce() -> T) -> (Trace, std::thread::Result<T>) {
    let dispatch = tracing::Dispatch::new(Capture::default());
    let done = tracing::dispatcher::with_default(&dispatch, || {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(body))
    });
    let capture: &Capture = dispatch.downcast_ref().expect("the capture just installed");
    let spans = capture.state.lock().expect("the recording").spans.clone();
    (Trace { spans }, done)
}

/// The spans of the trace that cite the claim: the selection every check makes
/// before it reads anything, spelled once for all eight of them.
///
/// The join is [`SpecMeta::name`] against each [`Span::claims`] entry, both of
/// which are produced from one `<Path as Spec>::NAME`, so the two sides cannot
/// disagree about naming.
///
/// It carries no claim of its own. Every question it is asked belongs to the
/// caller — check 23 is its empty case, and each rule of check 24 reads what it
/// selected — so a wrong selection here is a caller's claim failing, and a
/// claim cited here would be one the same test already answers one level up.
fn citing<'a>(trace: &'a Trace, spec: &SpecMeta) -> Vec<&'a Span> {
    trace.spans.iter().filter(|span| span.claims.iter().any(|cited| cited == spec.name)).collect()
}

/// Check 23: whether no span in the trace cites this claim — the empty case of
/// the selection every check begins with.
///
/// `true` is the finding — the claim was not reached — which is the opposite
/// polarity from the rules below, where `true` is satisfaction. This function's
/// own answer is that polarity.
#[implements(
    spec::NoSpanCitingTheClaimIsAFinding,
    spec::ASpanCitingTheClaimIsNoFinding,
)]
pub fn unreached(trace: &Trace, spec: &SpecMeta) -> bool {
    citing(trace, spec).is_empty()
}

/// Check 24: whether the claim's pattern wants a trace this one is not.
///
/// The slice's one flow node — five arms over
/// [`ClaimMeta::pattern`](crate::claim::ClaimMeta::pattern), one rule each, and
/// no wildcard, since [`Pattern`] has exactly five
/// variants. `true` is the finding, so each arm answers the negation of its
/// rule.
#[implements(
    spec::TheEventDrivenRuleDecidesAnEventDrivenClaim,
    spec::TheUnwantedRuleDecidesAnUnwantedClaim,
    spec::TheOptionalRuleDecidesAnOptionalClaim,
    spec::TheStateDrivenRuleDecidesAStateDrivenClaim,
    spec::TheUbiquitousRuleDecidesAUbiquitousClaim,
    spec::ASatisfiedRuleIsNoFinding,
)]
pub fn wrong_outcome(trace: &Trace, spec: &SpecMeta) -> bool {
    match spec.claim.pattern {
        Pattern::EventDriven => !event_driven(trace, spec),
        Pattern::Unwanted => !unwanted(trace, spec),
        Pattern::Optional => !optional(trace, spec),
        Pattern::StateDriven => !state_driven(trace, spec),
        Pattern::Ubiquitous => !ubiquitous(trace, spec),
    }
}

/// Check 24's event-driven rule: whether a span citing the claim closed `Ok` of
/// the response object the claim names.
///
/// The expected value is [`ClaimMeta::object`](crate::claim::ClaimMeta::object)
/// — the response type as the claim's author wrote it — read against
/// [`Span::outcome`].
#[implements(
    spec::TheEventDrivenRuleIsSatisfiedByAnOkOutcome,
    spec::TheEventDrivenRuleIsNotSatisfiedWithoutAnOkOutcome,
)]
pub fn event_driven(trace: &Trace, spec: &SpecMeta) -> bool {
    let closes = format!("Ok({})", spec.claim.object);
    citing(trace, spec).iter().any(|span| span.outcome.as_deref() == Some(closes.as_str()))
}

/// Check 24's unwanted rule: whether a span citing the claim closed `Err` of
/// the response variant the claim names.
#[implements(
    spec::TheUnwantedRuleIsSatisfiedByAnErrOutcome,
    spec::TheUnwantedRuleIsNotSatisfiedWithoutAnErrOutcome,
)]
pub fn unwanted(trace: &Trace, spec: &SpecMeta) -> bool {
    let closes = format!("Err({})", spec.claim.object);
    citing(trace, spec).iter().any(|span| span.outcome.as_deref() == Some(closes.as_str()))
}

/// Check 24's optional rule: whether a span citing the claim carries a
/// `lid.feature` field naming the feature the claim names.
#[implements(
    spec::TheOptionalRuleIsSatisfiedByASpanCarryingTheFeature,
    spec::TheOptionalRuleIsNotSatisfiedWithoutThatFeature,
)]
pub fn optional(trace: &Trace, spec: &SpecMeta) -> bool {
    citing(trace, spec)
        .iter()
        .flat_map(|span| &span.fields)
        .any(|(name, value)| name == "lid.feature" && value == spec.claim.object)
}

/// Check 24's state-driven rule: whether the spans citing the claim carry at
/// least two distinct `lid.state` values — a transition actually occurred.
///
/// The one rule with no expected value to read, which is why every rule takes
/// the whole [`SpecMeta`] rather than a string extracted for it.
#[implements(
    spec::TheStateDrivenRuleIsSatisfiedByTwoDistinctStates,
    spec::TheStateDrivenRuleIsNotSatisfiedByOneState,
)]
pub fn state_driven(trace: &Trace, spec: &SpecMeta) -> bool {
    let states: BTreeSet<&str> = citing(trace, spec)
        .iter()
        .flat_map(|span| &span.fields)
        .filter(|(name, _)| name == "lid.state")
        .map(|(_, state)| state.as_str())
        .collect();
    states.len() >= 2
}

/// Check 24's ubiquitous rule: [`enough_spans`] and [`distinct_inputs`] and
/// [`no_panics`], together.
///
/// A leaf that calls leaves and decides nothing — it answers the conjunction.
/// README writes the row as one sentence, but a count, a distinctness and an
/// absence of panics are independently falsifiable, and one leaf holding all
/// three hands check 12 two surviving mutants.
#[implements(
    spec::TheUbiquitousRuleIsSatisfiedByItsThreeRulesTogether,
    spec::TheUbiquitousRuleIsNotSatisfiedWhenOneOfItsRulesIsNot,
)]
pub fn ubiquitous(trace: &Trace, spec: &SpecMeta) -> bool {
    enough_spans(trace, spec) && distinct_inputs(trace, spec) && no_panics(trace, spec)
}

/// Whether at least [`MIN_INPUTS`] spans of the trace cite the claim.
#[implements(
    spec::TheEnoughSpansRuleIsSatisfiedByMinInputsCitingSpans,
    spec::TheEnoughSpansRuleIsNotSatisfiedByFewerCitingSpans,
)]
pub fn enough_spans(trace: &Trace, spec: &SpecMeta) -> bool {
    citing(trace, spec).len() >= MIN_INPUTS
}

/// Whether the spans citing the claim carry differing `lid.noun.*` fields.
///
/// It cannot be satisfied by a real trace until a `Traceable` recording method
/// exists — a span records a parameter's noun, not its value, and every call of
/// one signature records the same nouns. That is the LLD's Deferred 7 and the
/// reason [`LEVEL`] arrives at [`Ramp::Warn`].
#[implements(
    spec::TheDistinctInputsRuleIsSatisfiedByDifferingNouns,
    spec::TheDistinctInputsRuleIsNotSatisfiedByRepeatedNouns,
)]
pub fn distinct_inputs(trace: &Trace, spec: &SpecMeta) -> bool {
    let read: Vec<Vec<&str>> = citing(trace, spec)
        .iter()
        .map(|span| {
            span.fields
                .iter()
                .filter(|(name, _)| name.starts_with("lid.noun."))
                .map(|(_, noun)| noun.as_str())
                .collect()
        })
        .collect();
    read.iter().collect::<BTreeSet<_>>().len() == read.len()
}

/// Whether no span citing the claim unwound — which is to say, whether every
/// one of them recorded a [`Span::outcome`].
#[implements(
    spec::TheNoPanicsRuleIsSatisfiedByAnOutcomeOnEveryCitingSpan,
    spec::TheNoPanicsRuleIsNotSatisfiedByASpanThatRecordedNoOutcome,
)]
pub fn no_panics(trace: &Trace, spec: &SpecMeta) -> bool {
    citing(trace, spec).iter().all(|span| span.outcome.is_some())
}

/// The tree as this slice can print it: one line per span, in the order the
/// spans were opened.
///
/// That order is the tree, and no second walk assembles one. A subscriber
/// records a span when it opens, and a span opens inside the span that is
/// entered, so a parent always stands before its children — what makes a child
/// a child on the page is the depth its line is drawn at.
///
/// It takes the claims as well as the trace because the `✓ promised` mark means
/// "[`wrong_outcome`] carries no finding for this claim", which nothing in a
/// [`Trace`] alone can answer.
///
/// The `[flow]` marker and the `[redacted]` field of README §6.4's worked
/// output are the LLD's Deferred 6 — the first needs `lid-rs-shape`'s
/// classification, which is behind a package cycle, and the second needs the
/// policy attributes. The column the marker will stand in is left.
///
/// The test's output *is* the requirements it exercised, so this is a
/// deliverable and not a debugging aid.
#[implements(
    spec::TheRenderedTreeNamesEverySpanOfTheTrace,
    spec::TheRenderedTreeSetsAChildSpanBelowItsParent,
)]
pub fn render(trace: &Trace, specs: &[&SpecMeta]) -> String {
    (0..trace.spans.len()).map(|index| line(trace, index, specs)).collect::<Vec<_>>().join("\n")
}

/// One span's line: where it stands in the tree, its name, each claim it cites
/// with that claim's mark, and the outcome it recorded.
///
/// The line begins with a box-drawing branch, three columns per generation
/// deep, which is what sets a child below its parent. Every line carries one,
/// a span with no parent included: README §6.4's block draws its outermost span
/// that way too, the line above it there being the validator's own name, which
/// is `#[validates]`'s to print and not this function's.
///
/// One branch serves every span rather than a `├─` for a span with a later
/// sibling and a `└─` for the last: no claim of this slice tells the two apart,
/// and a renderer that decided it would be deciding something nobody wrote
/// down.
///
/// A span that recorded no outcome — one whose body unwound — carries no
/// outcome rather than an empty one, which is the same absence [`no_panics`]
/// reads.
#[implements(
    spec::TheRenderedTreeSetsAChildSpanBelowItsParent,
    spec::TheRenderedTreeNamesTheClaimBesideTheSpanThatCitesIt,
    spec::TheRenderedTreeNamesEachSpansRecordedOutcome,
)]
fn line(trace: &Trace, index: usize, specs: &[&SpecMeta]) -> String {
    let span = &trace.spans[index];
    let mut parts = vec![span.name.clone()];
    parts.extend(span.claims.iter().map(|claim| format!("{claim}{}", mark(trace, specs, claim))));
    parts.extend(span.outcome.iter().map(|outcome| format!("→ {outcome}")));
    format!("{}├─ {}", "   ".repeat(depth(trace, index)), parts.join("  "))
}

/// How many spans stand above this one: the length of its chain of parents, and
/// the indentation [`line()`] draws it at.
///
/// The chain is walked at most once per span of the trace, so a
/// [`Span::parent`] that pointed at itself prints a wrong depth rather than
/// hanging the renderer — a wrong answer a test can see, instead of a run that
/// never ends.
///
/// Untraced for the reason [`citing`] is: the claim about a child standing
/// below its parent is answered by the line that carries the indentation and by
/// the walk that orders the lines.
fn depth(trace: &Trace, index: usize) -> usize {
    std::iter::successors(Some(index), |above| trace.spans.get(*above).and_then(|span| span.parent))
        .take(trace.spans.len())
        .count()
        .saturating_sub(1)
}

/// The `✓ promised` mark for one claim named on one line, or nothing.
///
/// A claim is promised when [`wrong_outcome`] carries no finding for it, which
/// is the one thing a [`Trace`] and the validator's claims can say about it. A
/// claim the validator did not cite is named unmarked, there being no
/// [`SpecMeta`] here to ask about it.
///
/// The mark stands beside the claim rather than at the end of the line, because
/// one span cites as many claims as its `#[implements]` names, and a mark at
/// the end of a line naming two claims could not say which of them was kept.
#[implements(
    spec::TheRenderedTreeMarksAKeptClaimAsPromised,
    spec::TheRenderedTreeLeavesAClaimWithAFindingUnmarked,
)]
fn mark(trace: &Trace, specs: &[&SpecMeta], claim: &str) -> &'static str {
    let promised = specs.iter().any(|spec| spec.name == claim && !wrong_outcome(trace, spec));
    if promised { " ✓ promised" } else { "" }
}

/// The guard that records a span's outcome when its body returns, and does not
/// when the body unwinds.
///
/// `lid.outcome` cannot be recorded by a statement after the body, because a
/// body wrapped in `let value = { .. };` is no longer the fn's tail expression
/// and loses the coercion to its return type — which broke three consumer
/// crates the first time the emission was written. Dropping in reverse
/// declaration order puts this after the body's value is produced and before
/// the span's guard, which is exactly the moment the outcome is known.
///
/// It records `()` and not the returned value: a rendering of the value needs a
/// bound on every cited fn's return type, which this workspace's 549 citation
/// sites do not carry. Deferred with the `Traceable` recording method.
#[derive(Debug)]
pub struct Returned<'a> {
    /// The span whose outcome is recorded when this drops without a panic.
    span: &'a tracing::Span,
}

impl<'a> Returned<'a> {
    /// The guard for one span, held for the length of the body it watches.
    pub const fn of(span: &'a tracing::Span) -> Self {
        Self { span }
    }
}

// `Returned::drop` can carry no `#[implements]` either, and for a reason of its
// own: `#[implements]` expands to a span *and a `Returned` guard over it*, so a
// citation here would construct a second guard inside every drop of the first
// and recurse without bound. Measured — `cargo test --lib -p lid-rs` aborts on
// a stack overflow before the third case, with no subscriber installed, because
// the guard is built whether or not the callsite is enabled. The two claims are
// cited by containment, as `register_callsite`'s is above.
lid_rs::implements_module!(
    spec::TheReturnedGuardRecordsAnOutcomeWhenItsBodyReturns,
    spec::TheReturnedGuardRecordsNoOutcomeWhileItsBodyUnwinds,
);

impl Drop for Returned<'_> {
    /// Records the outcome unless the thread is unwinding, which is the whole
    /// of the distinction `no_panics` reads.
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.span.record("lid.outcome", "()");
        }
    }
}

/// What check 23 or check 24 has to say about one claim, or the empty string
/// where neither has anything.
///
/// The dispatch beneath [`report`], and uncited: every question it answers
/// belongs to [`report`], whose claims fail when it answers wrongly — the
/// reading [`citing`] and [`depth`] already take, and the one
/// `claim::is_free` takes one level under `claim::free`.
fn finding(trace: &Trace, spec: &SpecMeta) -> &'static str {
    match (unreached(trace, spec), wrong_outcome(trace, spec)) {
        (true, _) => "unreached",
        (false, true) => "wrong outcome",
        (false, false) => "",
    }
}

/// What a validator prints: the findings against the claims it cites, and the
/// trace that produced them.
///
/// Empty when there is nothing to say, which is the ordinary case. It answers
/// the text rather than printing it, so that a validator can read it; the one
/// `eprintln!` is in `#[validates]`'s expansion, where the I/O belongs.
///
/// A name that [`SPECS`](crate::SPECS) does not register is passed over in
/// silence. The names arrive from an expansion in another crate, and a stale
/// macro, a renamed claim or a partially-linked binary is not a reason to make
/// the runtime checks louder than the code they observe.
#[implements(
    spec::TheReportNamesAnUnreachedClaim,
    spec::TheReportNamesAClaimWithAWrongOutcome,
    spec::TheReportOfAClaimWithNoFindingIsEmpty,
    spec::TheReportCarriesTheRenderedTree,
    spec::TheReportOfAnUnregisteredClaimIsEmpty,
)]
pub fn report(trace: &Trace, claims: &[&str]) -> String {
    let found: Vec<&'static SpecMeta> = crate::SPECS
        .iter()
        .filter(|spec| claims.contains(&spec.name) && !finding(trace, spec).is_empty())
        .collect();
    let lines: Vec<String> = found
        .iter()
        .map(|spec| format!("{}: {}", finding(trace, spec), spec.name))
        .chain(found.first().map(|_| render(trace, &found)))
        .collect();
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    //! What each item answers, over trees this module builds and over one it
    //! records through `tracing` itself.
    //!
    //! **Nothing here runs a traced function.** The span `#[implements]` opens
    //! lands in `lid-rs-macros/src/expand.rs` as a hand cascade after this
    //! slice's leaves, so at this phase there is no emission to observe: a case
    //! that installed a capture and called something would be a case about an
    //! item nobody has written. Every check below is therefore handed a
    //! [`Trace`] assembled from [`span`] and its three modifiers, which is what
    //! taking the tree as plain data was for.
    //!
    //! **[`Capture`] and [`Returned`] are the exceptions, and both are
    //! exercised through the real API.** The guard is the smaller of the two:
    //! it is built over a real span and dropped, once by returning and once by
    //! unwinding, because its subject is what `std::thread::panicking` answers
    //! at drop time and a hand-built tree cannot be wrong about that.
    //!
    //! **[`Capture`]'s case for the real API is stronger.** It is the one item whose subject is `tracing`'s own protocol —
    //! which callsite is enabled, which span is a child of which — and a hand
    //! call of its methods would assert the protocol this module invented
    //! rather than the one the emission will meet. So [`recorded`] installs one
    //! under `dispatcher::with_default`, the case opens spans with the `span!`
    //! macro, and the recording is read back out of the capture the dispatcher
    //! held. Two of its claims cannot be reached that way and have a fixture of
    //! their own: `register_callsite` is given a `&'static Metadata<'static>`,
    //! which no span macro hands a caller, so [`LID_META`] and [`APP_META`] are
    //! declared beside a [`FixtureCallsite`]; and a field recorded after
    //! creation needs `field::Empty` to reserve the slot and a later
    //! `Span::record` to fill it, which is the mechanism `lid.outcome` itself
    //! arrives by.
    //!
    //! **The claims a case is named for.** Check 14 admits a validator named
    //! for the `snake_case` of any one claim it cites, so a rule's two claims —
    //! one per direction, which is what makes the rule falsifiable when it is
    //! inverted — are one case with two assertions, named for the first. The
    //! dispatch's six are six cases instead: five that each hand
    //! [`wrong_outcome`] a trace one pattern's rule is not satisfied by, and one
    //! that hands it all five satisfied traces, which is the only shape in which
    //! "a satisfied rule is no finding" is asserted of every arm rather than of
    //! whichever arm a single case chose.
    //!
    //! **Every fixture claim carries its own name.** A rule selects its spans by
    //! [`SpecMeta::name`], so two fixtures sharing a name would leave every
    //! selection vacuously right; the five below name five different claims, and
    //! [`OTHER`] is the claim no rule here is asked about, so that a body which
    //! read every span rather than the citing ones answers differently.
    //!
    //! **The rendered tree is asserted exactly, and against this slice's
    //! format.** It differs from README §6.4's worked block in four recorded
    //! ways: no `[flow]` marker and no `[redacted]` field, which are the LLD's
    //! Deferred 6; one `├─` branch for every span, because no claim here tells a
    //! last sibling from any other; and the `✓ promised` mark beside the claim it
    //! is about, because a span cites as many claims as its `#[implements]`
    //! names. Two further things the cases fix that no claim states, because an
    //! exact assertion cannot leave them open: the parts of a line are separated
    //! by two spaces, and the answer carries no trailing newline — the line above
    //! the tree and the one below it are `#[validates]`'s to print.

    use super::{
        Capture, MIN_INPUTS, Returned, SPAN_LEVEL, Span, TARGET, Trace, captured, cited_claims,
        distinct_inputs, enough_spans, event_driven, no_panics, optional, render, spec,
        report, state_driven, ubiquitous, unreached, unwanted, wrong_outcome,
    };
    use crate::claim::{ClaimMeta, Language, Pattern};
    use crate::registry::SpecMeta;
    use lid_rs::validates;
    use tracing::callsite::{Callsite, Identifier};
    use tracing::field::{Empty, FieldSet};
    use tracing::metadata::Kind;
    use tracing::subscriber::Interest;
    use tracing::{Metadata, Subscriber};

    /// A claim's registration, as far as any check here reads one: the name the
    /// spans are selected by, the pattern the dispatch reads, and the object the
    /// rules compare a recorded value against.
    ///
    /// A fixture rather than an entry of `SPECS`, because a registered claim's
    /// pattern is whatever its own sentence said and a case resting on another
    /// slice's prose breaks when that prose is edited.
    const fn fixture(name: &'static str, pattern: Pattern, object: &'static str) -> SpecMeta {
        SpecMeta {
            name,
            file: "synthetic.rs",
            line: 1,
            claim: ClaimMeta {
                language: Language::Controlled,
                pattern,
                trigger: "",
                verb: "carry",
                negated: false,
                object,
                owner: "",
                templates: &["*"],
            },
        }
    }

    /// The event-driven fixture: its response object is the type a span must
    /// have closed `Ok` of.
    const EVENT_DRIVEN: SpecMeta =
        fixture("fixture::AnAccountIsLoaded", Pattern::EventDriven, "Account");

    /// The unwanted fixture: its response object is the variant a span must have
    /// closed `Err` of.
    const UNWANTED: SpecMeta =
        fixture("fixture::BadCredentialsAreRefused", Pattern::Unwanted, "InvalidCredentials");

    /// The optional fixture: its response object is the feature a span must
    /// stand behind.
    const OPTIONAL: SpecMeta = fixture("fixture::AuditingIsRecorded", Pattern::Optional, "audit");

    /// The state-driven fixture, whose rule needs no expected value: two
    /// distinct states is the whole of it.
    const STATE_DRIVEN: SpecMeta = fixture("fixture::TheSessionIsOpen", Pattern::StateDriven, "");

    /// The ubiquitous fixture, whose rule is a count, a distinctness and an
    /// absence of panics.
    const UBIQUITOUS: SpecMeta = fixture("fixture::EveryReadIsChecked", Pattern::Ubiquitous, "");

    /// A claim no case asks a rule about, cited by the spans that must not be
    /// selected.
    const OTHER: &str = "fixture::SomeOtherClaimEntirely";

    /// A root span citing `claims`, closing nothing and carrying no field: what
    /// every hand-built tree below is assembled from.
    fn span(name: &str, claims: &[&str]) -> Span {
        Span {
            name: name.to_string(),
            parent: None,
            claims: claims.iter().map(|claim| (*claim).to_string()).collect(),
            outcome: None,
            fields: Vec::new(),
        }
    }

    /// The same span, having recorded `outcome` before its guard dropped.
    fn closed(span: Span, outcome: &str) -> Span {
        Span { outcome: Some(outcome.to_string()), ..span }
    }

    /// The same span, carrying one more recorded field.
    fn carrying(mut span: Span, field: &str, value: &str) -> Span {
        span.fields.push((field.to_string(), value.to_string()));
        span
    }

    /// The same span, opened while the span at `parent` was entered.
    fn under(span: Span, parent: usize) -> Span {
        Span { parent: Some(parent), ..span }
    }

    /// The trace of those spans, in the order they were opened.
    fn trace(spans: Vec<Span>) -> Trace {
        Trace { spans }
    }

    /// The names of the spans, in order — what most of the capture's cases
    /// assert, since a span reaching the tree at all is what they are about.
    fn names(spans: &[Span]) -> Vec<&str> {
        spans.iter().map(|span| span.name.as_str()).collect()
    }

    /// The parent of each span, in order.
    fn parents(spans: &[Span]) -> Vec<Option<usize>> {
        spans.iter().map(|span| span.parent).collect()
    }

    /// The spans a fresh [`Capture`] records while `body` runs under it as the
    /// thread's dispatcher.
    ///
    /// The recording is read out of the capture the [`tracing::Dispatch`] owns
    /// rather than through [`captured`], so that a case about what `Capture`
    /// records is answered by `Capture` alone and a broken seam fails only the
    /// three cases that are about the seam.
    fn recorded(body: impl FnOnce()) -> Vec<Span> {
        let dispatch = tracing::Dispatch::new(Capture::default());
        tracing::dispatcher::with_default(&dispatch, body);
        let capture: &Capture = dispatch.downcast_ref().expect("the capture just installed");
        let recording = capture.state.lock().expect("the recording");
        recording.spans.clone()
    }

    /// A callsite of this module's own, for the one method that is handed a
    /// `&'static Metadata<'static>`.
    ///
    /// The `span!` macros never give a caller their callsite's metadata, and a
    /// borrow of a local cannot be `'static`, so the two the interest case needs
    /// are declared here — still `tracing`'s own types, and still not a mock.
    struct FixtureCallsite {
        /// The metadata this callsite stands for.
        meta: &'static Metadata<'static>,
    }

    impl Callsite for FixtureCallsite {
        fn set_interest(&self, _: Interest) {}

        fn metadata(&self) -> &Metadata<'_> {
            self.meta
        }
    }

    /// The callsite of a span at the target this slice reads.
    static LID_CALLSITE: FixtureCallsite = FixtureCallsite { meta: &LID_META };

    /// The metadata of a span at the target this slice reads.
    static LID_META: Metadata<'static> = Metadata::new(
        "authenticate",
        TARGET,
        SPAN_LEVEL,
        None,
        None,
        None,
        FieldSet::new(&[], Identifier(&LID_CALLSITE)),
        Kind::SPAN,
    );

    /// The callsite of a span at an application's own target.
    static APP_CALLSITE: FixtureCallsite = FixtureCallsite { meta: &APP_META };

    /// The metadata of a span at an application's own target, which the target
    /// filter rejects and the interest does not.
    static APP_META: Metadata<'static> = Metadata::new(
        "authenticate",
        "app",
        SPAN_LEVEL,
        None,
        None,
        None,
        FieldSet::new(&[], Identifier(&APP_CALLSITE)),
        Kind::SPAN,
    );

    /// A span at the `lid` target reaches the trace and one at another target
    /// does not, which is the whole of the filter and the whole of what
    /// [`TARGET`] means.
    ///
    /// Both spans are opened in one run, so a capture that recorded everything
    /// and one that recorded nothing are told apart by the same assertion.
    #[test]
    #[validates(spec::ASpanAtTheLidTargetEntersTheTrace, spec::ASpanAtAnotherTargetIsNotCaptured)]
    fn a_span_at_the_lid_target_enters_the_trace() {
        let spans = recorded(|| {
            let _traced = tracing::span!(target: TARGET, SPAN_LEVEL, "traced");
            let _untraced = tracing::span!(target: "app", SPAN_LEVEL, "untraced");
        });
        assert_eq!(names(&spans), ["traced"]);
    }

    /// The interest is `sometimes` for a callsite the filter admits and for one
    /// it rejects: it is answered explicitly, not derived from the filter.
    ///
    /// The second assertion is the claim. An interest derived from `enabled`
    /// would cache `never` for the application's callsite, and the failure that
    /// causes — an empty tree in a test that runs after another test — is the
    /// hardest kind there is to read.
    #[test]
    #[validates(spec::TheCapturesInterestInACallsiteIsSometimes)]
    fn the_captures_interest_in_a_callsite_is_sometimes() {
        let capture = Capture::default();
        assert!(capture.register_callsite(&LID_META).is_sometimes(), "the lid callsite");
        assert!(
            capture.register_callsite(&APP_META).is_sometimes(),
            "an interest derived from the target filter would be never here"
        );
    }

    /// A recorded span carries the name it was opened with, which is what every
    /// line of the rendered tree is keyed by.
    #[test]
    #[validates(spec::ARecordedSpanCarriesTheNameItWasOpenedWith)]
    fn a_recorded_span_carries_the_name_it_was_opened_with() {
        let spans = recorded(|| {
            let _span = tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate");
        });
        assert_eq!(names(&spans), ["authenticate"]);
    }

    /// The fields a span was opened with reach it under the names they were
    /// recorded with, in the order the callsite declares them.
    ///
    /// Two fields and not one: a body that kept only the last, or only the
    /// first, answers the same as the right one over a single field.
    ///
    /// One of them is not a string, because `tracing` routes a string field to
    /// `Visit::record_str` and everything else to `Visit::record_debug`, and
    /// those are two bodies here — a case passing two strings exercises one of
    /// them twice. The non-string arrives through its `Debug` rendering, which
    /// is why `7` and not `"7"` lands as `"7"` all the same, and why the string
    /// arm exists at all: a `Debug` rendering of `"open"` would be `"\"open\""`,
    /// and `lid.claims` through that arm would split into a first claim
    /// carrying a quote and a last one carrying another.
    #[test]
    #[validates(spec::ARecordedSpanCarriesTheFieldsItWasOpenedWith)]
    fn a_recorded_span_carries_the_fields_it_was_opened_with() {
        let spans = recorded(|| {
            let _span = tracing::span!(
                target: TARGET,
                SPAN_LEVEL,
                "authenticate",
                lid.state = "open",
                lid.feature = 7
            );
        });
        assert_eq!(
            spans[0].fields,
            [
                ("lid.state".to_string(), "open".to_string()),
                ("lid.feature".to_string(), "7".to_string()),
            ]
        );
    }

    /// `lid.claims` is split at the unit separator and lands as the span's
    /// claims rather than among its fields.
    ///
    /// The separator is asserted at both ends of its one spelling: through the
    /// leaf that splits it, and through a span recorded with it, which is where
    /// a routing that kept the field whole would show.
    #[test]
    #[validates(spec::TheClaimsFieldIsSplitAtTheUnitSeparator)]
    fn the_claims_field_is_split_at_the_unit_separator() {
        assert_eq!(cited_claims("fixture::One\u{1f}fixture::Two"), ["fixture::One", "fixture::Two"]);
        let spans = recorded(|| {
            let _span = tracing::span!(
                target: TARGET,
                SPAN_LEVEL,
                "verify",
                lid.claims = "fixture::One\u{1f}fixture::Two"
            );
        });
        assert_eq!(spans[0].claims, ["fixture::One", "fixture::Two"]);
        assert!(spans[0].fields.is_empty(), "the claims are not also a field");
    }

    /// A field reserved with `field::Empty` at creation and recorded afterwards
    /// reaches the span it was recorded on.
    ///
    /// This is the mechanism the wire contract rests on: the emission cannot
    /// know a body's outcome when it opens the span, so every value known only
    /// at the end arrives this way.
    #[test]
    #[validates(spec::AFieldRecordedAfterCreationReachesItsSpan)]
    fn a_field_recorded_after_creation_reaches_its_span() {
        let spans = recorded(|| {
            let span =
                tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate", lid.state = Empty);
            span.record("lid.state", "closed");
        });
        assert_eq!(spans[0].fields, [("lid.state".to_string(), "closed".to_string())]);
    }

    /// `lid.outcome` lands as the span's outcome and not among its fields:
    /// three destinations are what one recorded field is routed between, and
    /// this is the one the checks read as "the body returned".
    #[test]
    #[validates(spec::TheOutcomeFieldIsCarriedAsTheSpansOutcome)]
    fn the_outcome_field_is_carried_as_the_spans_outcome() {
        let spans = recorded(|| {
            let span =
                tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate", lid.outcome = Empty);
            span.record("lid.outcome", "Ok(Account)");
        });
        assert_eq!(spans[0].outcome.as_deref(), Some("Ok(Account)"));
        assert!(spans[0].fields.is_empty(), "the outcome is not also a field");
    }

    /// A span opened while another is entered is recorded as that span's child.
    #[test]
    #[validates(spec::ARecordedSpansParentIsTheSpanThatWasEntered)]
    fn a_recorded_spans_parent_is_the_span_that_was_entered() {
        let spans = recorded(|| {
            let outer = tracing::span!(target: TARGET, SPAN_LEVEL, "outer");
            outer.in_scope(|| {
                let _inner = tracing::span!(target: TARGET, SPAN_LEVEL, "inner");
            });
        });
        assert_eq!(parents(&spans), [None, Some(0)]);
    }

    /// A span opened while nothing is entered names no parent, which is what
    /// makes it a root of the rendered tree.
    #[test]
    #[validates(spec::ASpanOpenedWithNoSpanEnteredHasNoParent)]
    fn a_span_opened_with_no_span_entered_has_no_parent() {
        let spans = recorded(|| {
            let _outer = tracing::span!(target: TARGET, SPAN_LEVEL, "outer");
        });
        assert_eq!(parents(&spans), [None]);
    }

    /// A span opened after an entered span was exited takes the enclosing span,
    /// which is the half of the enter-stack that `exit` answers for.
    ///
    /// A capture that never popped would make the third span a child of the
    /// second, and one that popped everything would leave it a root.
    #[test]
    #[validates(spec::ASpanOpenedAfterAnExitTakesTheEnclosingParent)]
    fn a_span_opened_after_an_exit_takes_the_enclosing_parent() {
        let spans = recorded(|| {
            let outer = tracing::span!(target: TARGET, SPAN_LEVEL, "outer");
            outer.in_scope(|| {
                let middle = tracing::span!(target: TARGET, SPAN_LEVEL, "middle");
                middle.in_scope(|| {});
                let _after = tracing::span!(target: TARGET, SPAN_LEVEL, "after");
            });
        });
        assert_eq!(parents(&spans), [None, Some(0), Some(0)]);
    }

    /// The trace the seam answers with carries the spans the closure opened,
    /// and the closure's own value comes back beside it.
    #[test]
    #[validates(spec::TheCapturedTraceCarriesTheSpansOpenedInTheClosure)]
    fn the_captured_trace_carries_the_spans_opened_in_the_closure() {
        let (tree, result) = captured(|| {
            let _span = tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate");
            7
        });
        assert_eq!(names(&tree.spans), ["authenticate"]);
        assert_eq!(result.expect("the body returned"), 7);
    }

    /// A body that unwinds keeps its trace, and the panic is handed back rather
    /// than swallowed or resumed here.
    ///
    /// This is the case README §6.4's deliverable is about — the tree is printed
    /// *on failure*, and a validator's failure is a panic — and it is the only
    /// one that produces a span with no `lid.outcome` without building one by
    /// hand: the emission records the outcome after the body, so an unwind skips
    /// the record while the span still closes.
    #[test]
    #[validates(spec::TheCapturedTraceCarriesTheSpansOpenedInTheClosure)]
    fn the_captured_trace_carries_the_spans_opened_in_the_closure_that_panicked() {
        let (tree, result): (Trace, std::thread::Result<()>) = captured(|| {
            let span =
                tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate", lid.outcome = Empty);
            let _entered = span.enter();
            panic!("the body under validation failed");
        });
        assert!(result.is_err(), "the panic is handed back, not swallowed");
        assert_eq!(names(&tree.spans), ["authenticate"]);
        assert_eq!(tree.spans[0].outcome, None, "a span that unwound recorded no outcome");
    }

    /// Each run answers with its own tree: a capture is fresh per closure, so no
    /// validator reads the spans of the one that ran before it.
    #[test]
    #[validates(spec::TheCapturedTraceCarriesNoEarlierRunsSpans)]
    fn the_captured_trace_carries_no_earlier_runs_spans() {
        let (first, _) = captured(|| {
            let _span = tracing::span!(target: TARGET, SPAN_LEVEL, "first");
        });
        let (second, _) = captured(|| {
            let _span = tracing::span!(target: TARGET, SPAN_LEVEL, "second");
        });
        assert_eq!(names(&first.spans), ["first"]);
        assert_eq!(names(&second.spans), ["second"]);
    }

    /// The tree is in the order its spans were opened, which is the order the
    /// renderer walks and the only order in which a parent stands before its
    /// children.
    #[test]
    #[validates(spec::TheCapturedTraceIsInTheOrderItsSpansWereOpened)]
    fn the_captured_trace_is_in_the_order_its_spans_were_opened() {
        let (tree, _) = captured(|| {
            let first = tracing::span!(target: TARGET, SPAN_LEVEL, "first");
            first.in_scope(|| {
                let _second = tracing::span!(target: TARGET, SPAN_LEVEL, "second");
            });
            let _third = tracing::span!(target: TARGET, SPAN_LEVEL, "third");
        });
        assert_eq!(names(&tree.spans), ["first", "second", "third"]);
    }

    /// The guard records the outcome when the body it watches returns, and does
    /// not when the body unwinds — the two directions of one `if`, and the
    /// whole of the distinction `no_panics` reads.
    ///
    /// The guard is built and dropped by hand rather than by the emission: the
    /// emission is in another crate, and a case that called a traced function
    /// would answer for the expansion rather than for this body. Both
    /// directions are asserted because either alone is satisfied by a wrong
    /// body — a `drop` that records unconditionally passes the first, and one
    /// that records nothing at all passes the second.
    #[test]
    #[validates(
        spec::TheReturnedGuardRecordsAnOutcomeWhenItsBodyReturns,
        spec::TheReturnedGuardRecordsNoOutcomeWhileItsBodyUnwinds,
    )]
    fn the_returned_guard_records_an_outcome_when_its_body_returns() {
        let returned = recorded(|| {
            let span =
                tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate", lid.outcome = Empty);
            let _guard = Returned::of(&span);
        });
        assert_eq!(returned[0].outcome.as_deref(), Some("()"), "a body that returned");

        let (unwound, result): (Trace, std::thread::Result<()>) = captured(|| {
            let span =
                tracing::span!(target: TARGET, SPAN_LEVEL, "authenticate", lid.outcome = Empty);
            let _guard = Returned::of(&span);
            panic!("the body under validation failed");
        });
        assert!(result.is_err(), "the panic is handed back");
        assert_eq!(unwound.spans[0].outcome, None, "a body that unwound");
    }

    /// Check 23 from both sides: a trace whose spans cite another claim carries
    /// the finding, and one with a span citing this claim does not.
    #[test]
    #[validates(spec::NoSpanCitingTheClaimIsAFinding, spec::ASpanCitingTheClaimIsNoFinding)]
    fn no_span_citing_the_claim_is_a_finding() {
        let elsewhere = trace(vec![span("authenticate", &[OTHER])]);
        let cited = trace(vec![span("authenticate", &[OTHER, EVENT_DRIVEN.name])]);
        assert!(unreached(&elsewhere, &EVENT_DRIVEN), "no span cites the claim");
        assert!(!unreached(&cited, &EVENT_DRIVEN), "a span cites the claim");
    }

    /// The two traces the event-driven rule answers differently: a citing span
    /// that closed `Ok` of the claim's object, and one that closed `Ok` of
    /// something else while another claim's span closed the right thing.
    fn event_driven_traces() -> (Trace, Trace) {
        (
            trace(vec![closed(span("load_account", &[EVENT_DRIVEN.name]), "Ok(Account)")]),
            trace(vec![
                closed(span("load_account", &[EVENT_DRIVEN.name]), "Ok(Session)"),
                closed(span("load_account", &[OTHER]), "Ok(Account)"),
            ]),
        )
    }

    /// The two traces the unwanted rule answers differently, in the same shape.
    fn unwanted_traces() -> (Trace, Trace) {
        (
            trace(vec![closed(span("verify", &[UNWANTED.name]), "Err(InvalidCredentials)")]),
            trace(vec![
                closed(span("verify", &[UNWANTED.name]), "Err(Timeout)"),
                closed(span("verify", &[OTHER]), "Err(InvalidCredentials)"),
            ]),
        )
    }

    /// The two traces the optional rule answers differently: the feature the
    /// claim names stands on a citing span, or on nobody's.
    fn optional_traces() -> (Trace, Trace) {
        (
            trace(vec![carrying(span("audit", &[OPTIONAL.name]), "lid.feature", "audit")]),
            trace(vec![
                carrying(span("audit", &[OPTIONAL.name]), "lid.feature", "metrics"),
                carrying(span("audit", &[OTHER]), "lid.feature", "audit"),
            ]),
        )
    }

    /// The two traces the state-driven rule answers differently: two distinct
    /// states across the citing spans, or one state twice while another claim's
    /// span holds the second.
    fn state_driven_traces() -> (Trace, Trace) {
        (
            trace(vec![
                carrying(span("session", &[STATE_DRIVEN.name]), "lid.state", "open"),
                carrying(span("session", &[STATE_DRIVEN.name]), "lid.state", "closed"),
            ]),
            trace(vec![
                carrying(span("session", &[STATE_DRIVEN.name]), "lid.state", "open"),
                carrying(span("session", &[STATE_DRIVEN.name]), "lid.state", "open"),
                carrying(span("session", &[OTHER]), "lid.state", "closed"),
            ]),
        )
    }

    /// `count` spans citing the ubiquitous fixture, each closing an outcome and
    /// carrying a noun no other one carries.
    fn ubiquitous_spans(count: usize) -> Vec<Span> {
        (0..count)
            .map(|n| {
                carrying(
                    closed(span("read", &[UBIQUITOUS.name]), "Ok(())"),
                    "lid.noun.0",
                    &format!("Noun{n}"),
                )
            })
            .collect()
    }

    /// The two traces the ubiquitous rule answers differently: enough citing
    /// spans with distinct nouns and no unwind, and one span short of enough.
    fn ubiquitous_traces() -> (Trace, Trace) {
        (trace(ubiquitous_spans(MIN_INPUTS)), trace(ubiquitous_spans(MIN_INPUTS - 1)))
    }

    /// The dispatch sends an event-driven claim to the event-driven rule: a
    /// trace that rule is not satisfied by carries check 24's finding.
    #[test]
    #[validates(spec::TheEventDrivenRuleDecidesAnEventDrivenClaim)]
    fn the_event_driven_rule_decides_an_event_driven_claim() {
        let (_, unsatisfied) = event_driven_traces();
        assert!(wrong_outcome(&unsatisfied, &EVENT_DRIVEN), "no citing span closed Ok(Account)");
    }

    /// The dispatch sends an unwanted claim to the unwanted rule.
    #[test]
    #[validates(spec::TheUnwantedRuleDecidesAnUnwantedClaim)]
    fn the_unwanted_rule_decides_an_unwanted_claim() {
        let (_, unsatisfied) = unwanted_traces();
        assert!(
            wrong_outcome(&unsatisfied, &UNWANTED),
            "no citing span closed Err(InvalidCredentials)"
        );
    }

    /// The dispatch sends an optional claim to the optional rule.
    #[test]
    #[validates(spec::TheOptionalRuleDecidesAnOptionalClaim)]
    fn the_optional_rule_decides_an_optional_claim() {
        let (_, unsatisfied) = optional_traces();
        assert!(wrong_outcome(&unsatisfied, &OPTIONAL), "no citing span stood behind the feature");
    }

    /// The dispatch sends a state-driven claim to the state-driven rule.
    #[test]
    #[validates(spec::TheStateDrivenRuleDecidesAStateDrivenClaim)]
    fn the_state_driven_rule_decides_a_state_driven_claim() {
        let (_, unsatisfied) = state_driven_traces();
        assert!(wrong_outcome(&unsatisfied, &STATE_DRIVEN), "the citing spans saw one state");
    }

    /// The dispatch sends a ubiquitous claim to the ubiquitous rule.
    #[test]
    #[validates(spec::TheUbiquitousRuleDecidesAUbiquitousClaim)]
    fn the_ubiquitous_rule_decides_a_ubiquitous_claim() {
        let (_, unsatisfied) = ubiquitous_traces();
        assert!(wrong_outcome(&unsatisfied, &UBIQUITOUS), "one span short of enough");
    }

    /// A trace the claim's own rule is satisfied by carries no finding —
    /// asserted of all five arms, since a dispatch that always found something
    /// would keep every one of the five cases above.
    ///
    /// One assertion over the five rather than five assertions: an `assert!` is
    /// a branch, and five of them in one case is a case more complex than the
    /// dispatch it is about. What it reports is which arms found something,
    /// which is what five separate assertions would have said one at a time.
    #[test]
    #[validates(spec::ASatisfiedRuleIsNoFinding)]
    fn a_satisfied_rule_is_no_finding() {
        let arms = [
            (event_driven_traces().0, &EVENT_DRIVEN),
            (unwanted_traces().0, &UNWANTED),
            (optional_traces().0, &OPTIONAL),
            (state_driven_traces().0, &STATE_DRIVEN),
            (ubiquitous_traces().0, &UBIQUITOUS),
        ];
        let found: Vec<&str> = arms
            .iter()
            .filter(|(tree, spec)| wrong_outcome(tree, spec))
            .map(|(_, spec)| spec.name)
            .collect();
        assert!(found.is_empty(), "a satisfied rule carried a finding: {found:?}");
    }

    /// The event-driven rule from both sides: a citing span that closed `Ok` of
    /// the claim's object satisfies it, and one that closed `Ok` of another type
    /// does not — however many other claims' spans closed the right one.
    #[test]
    #[validates(
        spec::TheEventDrivenRuleIsSatisfiedByAnOkOutcome,
        spec::TheEventDrivenRuleIsNotSatisfiedWithoutAnOkOutcome
    )]
    fn the_event_driven_rule_is_satisfied_by_an_ok_outcome() {
        let (satisfied, unsatisfied) = event_driven_traces();
        assert!(event_driven(&satisfied, &EVENT_DRIVEN));
        assert!(!event_driven(&unsatisfied, &EVENT_DRIVEN), "Ok(Session) is not Ok(Account)");
    }

    /// The unwanted rule from both sides.
    #[test]
    #[validates(
        spec::TheUnwantedRuleIsSatisfiedByAnErrOutcome,
        spec::TheUnwantedRuleIsNotSatisfiedWithoutAnErrOutcome
    )]
    fn the_unwanted_rule_is_satisfied_by_an_err_outcome() {
        let (satisfied, unsatisfied) = unwanted_traces();
        assert!(unwanted(&satisfied, &UNWANTED));
        assert!(!unwanted(&unsatisfied, &UNWANTED), "Err(Timeout) is not the variant claimed");
    }

    /// The optional rule from both sides.
    #[test]
    #[validates(
        spec::TheOptionalRuleIsSatisfiedByASpanCarryingTheFeature,
        spec::TheOptionalRuleIsNotSatisfiedWithoutThatFeature
    )]
    fn the_optional_rule_is_satisfied_by_a_span_carrying_the_feature() {
        let (satisfied, unsatisfied) = optional_traces();
        assert!(optional(&satisfied, &OPTIONAL));
        assert!(!optional(&unsatisfied, &OPTIONAL), "the feature stood on nobody's citing span");
    }

    /// The state-driven rule from both sides: a transition is two states, and
    /// one state recorded twice is not one.
    #[test]
    #[validates(
        spec::TheStateDrivenRuleIsSatisfiedByTwoDistinctStates,
        spec::TheStateDrivenRuleIsNotSatisfiedByOneState
    )]
    fn the_state_driven_rule_is_satisfied_by_two_distinct_states() {
        let (satisfied, unsatisfied) = state_driven_traces();
        assert!(state_driven(&satisfied, &STATE_DRIVEN));
        assert!(!state_driven(&unsatisfied, &STATE_DRIVEN), "two spans, one state");
    }

    /// The ubiquitous rule is the conjunction of its three, and each of the
    /// three can be the one that fails.
    ///
    /// Three unsatisfied traces and not one: a body that dropped a conjunct
    /// would still answer rightly over a trace that broke a different one, which
    /// is the pair of free mutants a single leaf holding all three would hand
    /// check 12.
    #[test]
    #[validates(
        spec::TheUbiquitousRuleIsSatisfiedByItsThreeRulesTogether,
        spec::TheUbiquitousRuleIsNotSatisfiedWhenOneOfItsRulesIsNot
    )]
    fn the_ubiquitous_rule_is_satisfied_by_its_three_rules_together() {
        let (satisfied, few) = ubiquitous_traces();
        let mut repeated = ubiquitous_spans(MIN_INPUTS);
        repeated[MIN_INPUTS - 1] =
            carrying(closed(span("read", &[UBIQUITOUS.name]), "Ok(())"), "lid.noun.0", "Noun0");
        let mut unwound = ubiquitous_spans(MIN_INPUTS);
        unwound[0] = carrying(span("read", &[UBIQUITOUS.name]), "lid.noun.0", "Noun0");
        let broken = [
            ("one span short of enough", few),
            ("two spans carrying one noun", trace(repeated)),
            ("a span that recorded no outcome", trace(unwound)),
        ];
        let kept: Vec<&str> = broken
            .iter()
            .filter(|(_, tree)| ubiquitous(tree, &UBIQUITOUS))
            .map(|(why, _)| *why)
            .collect();
        assert!(ubiquitous(&satisfied, &UBIQUITOUS), "all three rules are satisfied");
        assert!(kept.is_empty(), "the conjunction held despite {kept:?}");
    }

    /// The count rule from both sides, which is where [`MIN_INPUTS`] is
    /// asserted: exactly as many citing spans as it names is enough, and one
    /// fewer is not.
    #[test]
    #[validates(
        spec::TheEnoughSpansRuleIsSatisfiedByMinInputsCitingSpans,
        spec::TheEnoughSpansRuleIsNotSatisfiedByFewerCitingSpans
    )]
    fn the_enough_spans_rule_is_satisfied_by_min_inputs_citing_spans() {
        let (enough, few) = ubiquitous_traces();
        assert!(enough_spans(&enough, &UBIQUITOUS));
        assert!(!enough_spans(&few, &UBIQUITOUS), "one span short of MIN_INPUTS");
    }

    /// The distinctness rule from both sides.
    ///
    /// It is the rule no real trace can satisfy until a parameter's value is
    /// recorded rather than its noun, which is why the ramp arrives at `warn`;
    /// what it answers over a trace that does carry differing nouns is
    /// nonetheless this slice's to get right.
    #[test]
    #[validates(
        spec::TheDistinctInputsRuleIsSatisfiedByDifferingNouns,
        spec::TheDistinctInputsRuleIsNotSatisfiedByRepeatedNouns
    )]
    fn the_distinct_inputs_rule_is_satisfied_by_differing_nouns() {
        let differing = trace(ubiquitous_spans(2));
        let repeated = trace(vec![
            carrying(span("read", &[UBIQUITOUS.name]), "lid.noun.0", "Noun0"),
            carrying(span("read", &[UBIQUITOUS.name]), "lid.noun.0", "Noun0"),
        ]);
        assert!(distinct_inputs(&differing, &UBIQUITOUS));
        assert!(!distinct_inputs(&repeated, &UBIQUITOUS), "two spans read one noun");
    }

    /// The panic rule from both sides: an outcome on every citing span, against
    /// a citing span that recorded none.
    ///
    /// The absent outcome is the one inference in the wire contract — the
    /// emission records after the body, so nothing recorded means the body
    /// unwound, and a unit return records `()` rather than nothing.
    #[test]
    #[validates(
        spec::TheNoPanicsRuleIsSatisfiedByAnOutcomeOnEveryCitingSpan,
        spec::TheNoPanicsRuleIsNotSatisfiedByASpanThatRecordedNoOutcome
    )]
    fn the_no_panics_rule_is_satisfied_by_an_outcome_on_every_citing_span() {
        let closed_spans = trace(ubiquitous_spans(2));
        let unwound = trace(vec![
            closed(span("read", &[UBIQUITOUS.name]), "Ok(())"),
            span("read", &[UBIQUITOUS.name]),
            closed(span("read", &[OTHER]), "Ok(())"),
        ]);
        assert!(no_panics(&closed_spans, &UBIQUITOUS));
        assert!(!no_panics(&unwound, &UBIQUITOUS), "a citing span recorded no outcome");
    }

    /// The rendered tree, exactly: every span on its own line in the order they
    /// were opened, a child three columns in from its parent, the claim beside
    /// the span that cites it, the outcome after it, and `✓ promised` on the
    /// claim check 24 carries no finding for.
    ///
    /// One assertion and not five, because the format is the deliverable — the
    /// test's output *is* the requirements it exercised — and five loose
    /// assertions over substrings would each pass against a tree nobody could
    /// read.
    #[test]
    #[validates(
        spec::TheRenderedTreeNamesEverySpanOfTheTrace,
        spec::TheRenderedTreeSetsAChildSpanBelowItsParent,
        spec::TheRenderedTreeNamesTheClaimBesideTheSpanThatCitesIt,
        spec::TheRenderedTreeNamesEachSpansRecordedOutcome,
        spec::TheRenderedTreeMarksAKeptClaimAsPromised
    )]
    fn the_rendered_tree_names_every_span_of_the_trace() {
        let tree = trace(vec![
            span("authenticate", &[]),
            under(closed(span("load_account", &[]), "Ok(Account)"), 0),
            under(closed(span("verify_password", &[EVENT_DRIVEN.name]), "Ok(Account)"), 0),
        ]);
        let expected = [
            "├─ authenticate",
            "   ├─ load_account  → Ok(Account)",
            "   ├─ verify_password  fixture::AnAccountIsLoaded ✓ promised  → Ok(Account)",
        ]
        .join("\n");
        assert_eq!(render(&tree, &[&EVENT_DRIVEN]), expected);
    }

    /// A claim check 24 carries a finding for is named on its span's line and
    /// left unmarked — the mark means "nothing was found for this claim", which
    /// is the one thing a trace and the validator's claims can say.
    #[test]
    #[validates(spec::TheRenderedTreeLeavesAClaimWithAFindingUnmarked)]
    fn the_rendered_tree_leaves_a_claim_with_a_finding_unmarked() {
        let tree = trace(vec![closed(
            span("verify_password", &[EVENT_DRIVEN.name]),
            "Err(NoSuchAccount)",
        )]);
        assert_eq!(
            render(&tree, &[&EVENT_DRIVEN]),
            "├─ verify_password  fixture::AnAccountIsLoaded  → Err(NoSuchAccount)"
        );
    }

    // ---- `report`: what a validator prints -----------------------------------

    /// A claim of this slice, registered and reachable from here.
    ///
    /// `report` looks names up in [`SPECS`](crate::SPECS), so its cases cannot
    /// use the `SpecMeta` fixtures above — those register nothing. They use this
    /// slice's own claims instead of another slice's, so a sentence edited here
    /// breaks a case in the same file rather than one three directories away.
    const REGISTERED: &str = "lid_rs::validate::spec::TheReportNamesAnUnreachedClaim";

    /// A trace whose one span cites [`REGISTERED`] and satisfies nothing.
    ///
    /// A citing span carrying no outcome, no feature and no state fails every
    /// one of check 24's five rules, so this is the wrong-outcome case whatever
    /// pattern the claim's sentence gave it.
    fn citing_but_unsatisfying() -> Trace {
        Trace { spans: vec![span("authenticate", &[REGISTERED])] }
    }

    #[test]
    #[validates(spec::TheReportNamesAnUnreachedClaim)]
    fn the_report_names_an_unreached_claim() {
        let empty = Trace { spans: vec![] };
        let text = report(&empty, &[REGISTERED]);
        assert!(text.contains(REGISTERED), "the claim is named: {text}");
        assert!(text.contains("unreached"), "the finding is named: {text}");
    }

    #[test]
    #[validates(spec::TheReportNamesAClaimWithAWrongOutcome)]
    fn the_report_names_a_claim_with_a_wrong_outcome() {
        let trace = citing_but_unsatisfying();
        let text = report(&trace, &[REGISTERED]);
        assert!(text.contains(REGISTERED), "the claim is named: {text}");
        assert!(text.contains("wrong outcome"), "the finding is named: {text}");
        assert!(!text.contains("unreached"), "it was reached: {text}");
    }

    #[test]
    #[validates(spec::TheReportOfAClaimWithNoFindingIsEmpty)]
    fn the_report_of_a_claim_with_no_finding_is_empty() {
        let kept = Trace { spans: vec![closed(span("authenticate", &[REGISTERED]), "Ok()")] };
        assert_eq!(report(&kept, &[REGISTERED]), "", "a claim that was kept says nothing");
    }

    #[test]
    #[validates(spec::TheReportCarriesTheRenderedTree)]
    fn the_report_carries_the_rendered_tree() {
        let trace = citing_but_unsatisfying();
        let spec = crate::SPECS.iter().find(|meta| meta.name == REGISTERED).expect("registered");
        let text = report(&trace, &[REGISTERED]);
        assert!(
            text.contains(&render(&trace, &[spec])),
            "the finding carries the trace that produced it: {text}"
        );
    }

    #[test]
    #[validates(spec::TheReportOfAnUnregisteredClaimIsEmpty)]
    fn the_report_of_an_unregistered_claim_is_empty() {
        let trace = citing_but_unsatisfying();
        assert_eq!(
            report(&trace, &["fixture::NoSuchClaimIsRegisteredAnywhere"]),
            "",
            "a name the registry does not hold is passed over in silence"
        );
    }

    #[test]
    #[validates(spec::ASpanOpenedInsideTheCaptureIsNotCaptured)]
    fn a_span_opened_inside_the_capture_is_not_captured() {
        // `cited_claims` is one of the items `Capture::new_span` itself calls,
        // so its span is the one that recurses: new_span -> take ->
        // cited_claims -> enabled -> new_span. The capture declines its own
        // module, and this is that refusal seen from outside.
        let spans = recorded(|| {
            let _ = super::cited_claims("fixture::AClaimName");
        });
        assert!(
            spans.is_empty(),
            "the capture does not observe itself, but recorded {spans:?}"
        );
    }
}
