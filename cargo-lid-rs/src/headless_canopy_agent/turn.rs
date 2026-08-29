//! Driving a turn (`docs/intent/headless-canopy-agent/lld.md` § Driving a
//! turn): one open session, the loop that lands a user message and follows
//! the tail until the turn settles, the record bodies it acts on — decoded
//! once, by kind, where the record is classified — and the invoke
//! choreography: pairing a forward with its held payload by digest, and
//! answering it.

use std::time::{Duration, Instant};

use lid_rs::implements;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::door::{Door, Offered, Record, Settings, Started};
use super::tools::{Tool, ToolResult, execute};
use crate::phase::Phase;
use crate::project::Project;
use crate::spec;

/// How long a tail read is parked, in seconds: the door's ceiling.
pub const TAIL_WAIT: u64 = 25;

/// How close to its expiry, in seconds, a credential is refreshed.
pub const REFRESH_MARGIN: u64 = 5;

/// How long a tail may deliver nothing before the client stops the session:
/// canopy's own invoke-stall window.
pub const QUIET_TAIL: Duration = Duration::from_secs(15 * 60);

/// The kind of the record a user message lands as.
pub const USER_MESSAGE: &str = "app.client.user_message";

/// The kind of the record a tool's answer lands as.
pub const COMPLETED: &str = "app.invoke.completed";

/// One open session: the door it was dialled on, the credential, the cursor
/// the tail has reached, the payloads held until their forwards arrive, the
/// phase it runs, and the tools its policy declared.
pub struct Session {
    /// The client the session was dialled through.
    pub door: Door,
    /// The session's id and credential, refreshed before it expires.
    pub credential: Started,
    /// Where the next tail read starts.
    pub cursor: u64,
    /// `app.invoke.payload` records awaiting their forwards.
    pub held: Vec<Held>,
    /// The phase the session runs, for the phase library's verdicts.
    pub phase: Phase,
    /// The tools the session's policy declared; a forward for any other is
    /// refused here.
    pub tools: Vec<Tool>,
}

impl Session {
    /// Dials a session with `settings` on `door` — its policy is fixed there
    /// for the session's life — and prints the session's id as it opens, so
    /// every session a phase opens is printed before anything else the phase
    /// prints.
    #[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
    pub fn open(door: &Door, settings: &Settings, phase: Phase, tools: Vec<Tool>) -> Result<Self, String> {
        todo!()
    }

    /// The tally key and the commit's agent: `canopy:<session>`.
    #[implements(spec::TheCommitNamesItsSessionAsTheAgent)]
    pub fn agent_id(&self) -> String {
        todo!()
    }

    /// Stops the session through the door; its log is sealed.
    #[implements(spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    pub fn stop(self) -> Result<(), String> {
        todo!()
    }
}

/// The body of an `app.invoke.payload`: the requestee and the call's
/// arguments. `args` stays JSON — it is opaque until the tool's own typed
/// arguments decode it — and is what [`payload_digest`] takes.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Payload {
    /// The requestee the call is addressed to; this program answers `lid-rs`.
    pub to: String,
    /// The call's arguments, as the model wrote them.
    pub args: Value,
}

/// The body of an `app.invoke.forward`: the authorizer's admission of a
/// payload, named by its digest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Forward {
    /// The requestee the forward is addressed to; this program answers
    /// `lid-rs`.
    pub to: String,
    /// The tool's name: what [`Tool::of`] classifies.
    pub op: String,
    /// The digest of the payload this forward admits.
    pub payload_digest: String,
}

/// The body of an `inference.responded`, shaped from its `response` or its
/// `terminal`: the model's text and the tools it asked for when the provider
/// answered, or the provider's sentence when the turn failed there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Responded {
    /// `response.text`: the model's message, when the provider answered.
    pub text: Option<String>,
    /// The `op` of each tool use in `response.tool_uses`; empty when the
    /// model asked for none, which settles the turn.
    pub tool_uses: Vec<String>,
    /// The provider's sentence, when the turn failed at the provider.
    pub terminal: Option<String>,
}

impl Responded {
    /// The body shaped, decoded through [`RespondedBody`] — the one
    /// decision over what it carries: a `terminal` is the provider's
    /// sentence and nothing else; otherwise `response.text` — none when
    /// the model wrote no text, which [`responded`] settles as an empty
    /// string — and the `op` of each of `response.tool_uses`. A body with
    /// neither is the error.
    #[implements(spec::ATurnSettlesOnAResponseWithoutToolUses, spec::AProviderTerminalIsRetriedOnceThenStopsTheRun)]
    pub fn of(body: &Value) -> Result<Responded, String> {
        todo!()
    }
}

/// An `inference.responded` body as the door lands it, the boundary type
/// [`Responded::of`] decodes through: `response` when the provider
/// answered, `terminal` when the turn failed there; `echo`, `raw`, and
/// `usage` are not read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct RespondedBody {
    /// The provider's answer, when there is one.
    response: Option<Response>,
    /// The provider's sentence, when the turn failed there.
    terminal: Option<String>,
}

/// `inference.responded`'s `response`: the model's text and its tool uses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct Response {
    /// The model's message, when it wrote one.
    text: Option<String>,
    /// The tools the model asked for; none when absent.
    #[serde(default)]
    tool_uses: Vec<ToolUse>,
}

/// One of `response.tool_uses`; only its `op` is read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ToolUse {
    /// The tool's name, as the forward will name it.
    op: String,
}

/// An `app.invoke.denied`: the authorizer refused a call the policy does not
/// admit. Nothing in its body is acted on; it is counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Denied;

/// The body of an `app.session.halted`: why the platform ended the session.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Halted {
    /// A budget the config set was reached, or a stall the platform detected.
    pub reason: String,
}

/// A payload held until its forward arrives: where it landed, who landed it
/// (the completion's addressee), and its body.
#[derive(Debug, Clone, PartialEq)]
pub struct Held {
    /// The payload record's position on the log.
    pub cursor: u64,
    /// The payload record's producer.
    pub producer: String,
    /// The payload's body.
    pub payload: Payload,
}

/// A settled turn: the model's final text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settled {
    /// `response.text` of the `inference.responded` without `tool_uses`.
    pub text: String,
}

/// Why a session ended without a settled turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Halt {
    /// `app.session.halted`: a budget the config set was reached, or a
    /// stall the platform detected — its reason.
    Halted(String),
    /// A second `terminal` in one turn: the provider's sentence.
    Terminal(String),
    /// The tail delivered nothing for fifteen minutes.
    Quiet,
    /// The door refused a request, or answered with something this program
    /// could not use — a record whose body does not decode, a tally it
    /// could not write while acting on one: its sentence.
    Refused(String),
}

/// The record kinds the client acts on, classified once from
/// `Record.kind`; every other kind — `inference.requested`,
/// `app.policy.configured`, its own records — is informational.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `app.invoke.payload`: held until its forward arrives.
    Payload,
    /// `app.invoke.forward`: paired with a held payload and answered.
    Forward,
    /// `app.invoke.denied`: counted as a refusal.
    Denied,
    /// `inference.responded`: settles the turn, or asks for tools, or is
    /// the provider's terminal.
    Responded,
    /// `app.session.halted`: ends the session.
    Halted,
    /// Anything else: read past.
    Other,
}

impl Kind {
    /// A record's kind classified — the one decision over `Record.kind`,
    /// which [`acted`] dispatches on: `app.invoke.payload`,
    /// `app.invoke.forward`, `app.invoke.denied`, `inference.responded`,
    /// `app.session.halted`, and anything else as [`Kind::Other`].
    #[implements(
        spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest,
        spec::ADenialIsCountedAsARefusal,
        spec::ATurnSettlesOnAResponseWithoutToolUses,
        spec::AHaltEndsTheRunWithItsReason,
    )]
    pub fn of(kind: &str) -> Kind {
        todo!()
    }
}

/// What acting on one record does to the turn.
enum Progress {
    /// The tail is followed on.
    Continue,
    /// The turn settled with the model's final text.
    Settled(Settled),
}

/// Whether the turn's one retry of a provider terminal has been spent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Retry {
    /// No terminal yet: the next one lands the message once more.
    Untried,
    /// The message has been landed once more: the next terminal stops the run.
    Retried,
}

/// One turn's state while its tail is followed.
struct Turn<'a> {
    /// The user message, landed once more on a provider terminal.
    message: &'a str,
    /// Whether that retry has been spent.
    retry: Retry,
    /// When the tail last delivered a record.
    delivered: Instant,
}

/// Lands `message` as one `app.client.user_message` and follows the tail
/// page by page ([`next_page`]), acting on each record by kind ([`acted`])
/// until one settles the turn: the model's final text. Everything that
/// ends the session without one — the platform's halt, the provider's
/// second terminal, a quiet tail, the door's refusal — is the [`Halt`].
#[implements(spec::ATurnSettlesOnAResponseWithoutToolUses)]
pub fn drive(project: &Project, session: &mut Session, message: &str) -> Result<Settled, Halt> {
    land_message(session, message)?;
    let mut turn = Turn { message, retry: Retry::Untried, delivered: Instant::now() };
    loop {
        for record in next_page(session, &mut turn)? {
            if let Progress::Settled(settled) = acted(project, session, &mut turn, &record)? {
                return Ok(settled);
            }
        }
    }
}

/// The user message to offer: `app.client.user_message` with `text`.
pub fn user_message(text: &str) -> Offered {
    todo!()
}

/// Lands one user message; the door's refusal ends the session with its
/// sentence.
#[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
pub fn land_message(session: &Session, text: &str) -> Result<u64, Halt> {
    todo!()
}

/// Seconds since the epoch, as the door writes `expires`.
pub fn now() -> u64 {
    todo!()
}

/// The credential to read with: this one while it has more than five
/// seconds of life at `now`, else a fresh one from the door for the same
/// session; the door's refusal ends the session with its sentence.
#[implements(spec::TheCredentialIsRefreshedBeforeItExpires)]
pub fn usable(door: &Door, credential: &Started, now: u64) -> Result<Started, Halt> {
    todo!()
}

/// How long to park the next read: 25 s, the door's ceiling, clamped to the
/// credential's remaining life at `now`.
#[implements(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife)]
pub fn wait_for(credential: &Started, now: u64) -> u64 {
    todo!()
}

/// When the tail last delivered — the quiet clock's one decision: now if
/// the page carries records, else as before, so [`quiet_checked`] measures
/// fifteen minutes from the last delivery and not from the last read.
#[implements(spec::AQuietTailForFifteenMinutesStopsTheRun)]
pub fn delivery(previous: Instant, records: &[Record]) -> Instant {
    todo!()
}

/// The page's records to act on; an empty page fifteen minutes after the
/// tail last delivered ends the session as quiet.
#[implements(spec::AQuietTailForFifteenMinutesStopsTheRun)]
pub fn quiet_checked(delivered: Instant, records: Vec<Record>) -> Result<Vec<Record>, Halt> {
    todo!()
}

/// The next page of the tail: the credential refreshed within five seconds
/// of its expiry, the read parked for 25 s clamped to the credential's
/// life, the cursor advanced to where the page reached, the quiet clock
/// restarted by any delivery, and the records — or the halt: fifteen quiet
/// minutes, or the door's refusal of the read or the refresh.
#[implements(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife, spec::ADoorRefusalStopsTheRunWithItsSentence)]
fn next_page(session: &mut Session, turn: &mut Turn) -> Result<Vec<Record>, Halt> {
    let now = now();
    session.credential = usable(&session.door, &session.credential, now)?;
    let page = session.door.tail(&session.credential, session.cursor, wait_for(&session.credential, now)).map_err(Halt::Refused)?;
    session.cursor = page.through;
    turn.delivered = delivery(turn.delivered, &page.records);
    quiet_checked(turn.delivered, page.records)
}

/// One record acted on by its kind — the one `match` the turn is driven by:
/// a payload is held; a forward is paired and answered; a denial is counted;
/// a response settles the turn, asks for tools, or is the provider's
/// terminal; a halt ends the session; anything else is read past.
#[implements(
    spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest,
    spec::ADenialIsCountedAsARefusal,
    spec::ATurnSettlesOnAResponseWithoutToolUses,
    spec::AHaltEndsTheRunWithItsReason,
)]
fn acted(project: &Project, session: &mut Session, turn: &mut Turn, record: &Record) -> Result<Progress, Halt> {
    match Kind::of(&record.kind) {
        Kind::Payload => held(session, record),
        Kind::Forward => answered(project, session, record),
        Kind::Denied => denied(project, session),
        Kind::Responded => responded(session, turn, record),
        Kind::Halted => halted(record),
        Kind::Other => Ok(Progress::Continue),
    }
}

/// A record's body as the type its kind promises; one that does not decode
/// is the door's answer failing ([`undecodable`]).
pub fn body_of<T: DeserializeOwned>(record: &Record) -> Result<T, Halt> {
    todo!()
}

/// A record whose body is not what its kind promises: the halt naming the
/// record and what failed.
pub fn undecodable(record: &Record, what: &str) -> Halt {
    todo!()
}

/// An `app.invoke.payload` held — its cursor, its producer, its body — until
/// its forward arrives.
#[implements(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest)]
fn held(session: &mut Session, record: &Record) -> Result<Progress, Halt> {
    todo!()
}

/// An `app.invoke.forward`: paired with the held payload of its digest and
/// answered; one this program is not the addressee of, or whose payload it
/// does not hold, is read past unanswered.
#[implements(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest, spec::AForwardToAnotherPrincipalIsNotAnswered)]
fn answered(project: &Project, session: &mut Session, record: &Record) -> Result<Progress, Halt> {
    let forward: Forward = body_of(record)?;
    match pair(&session.held, &forward) {
        None => Ok(Progress::Continue),
        Some(pairing) => completed(project, session, record.cursor, &forward, &pairing),
    }
}

/// A paired forward answered: its tool run, the completion landed under the
/// forward's idempotency key — the door's refusal ends the session with its
/// sentence — and the payload released.
#[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
fn completed(project: &Project, session: &mut Session, forward_cursor: u64, forward: &Forward, pairing: &Pairing) -> Result<Progress, Halt> {
    let result = execute(project, session, &forward.op, &pairing.args);
    let (offered, idem) = completion(forward_cursor, pairing, &result);
    session.door.send(&session.credential, &offered, Some(&idem)).map_err(Halt::Refused)?;
    session.held.remove(pairing.index);
    Ok(Progress::Continue)
}

/// An `app.invoke.denied`: nothing runs; it is counted as a policy refusal
/// in the session's tally ([`crate::phase::tally::record`]).
#[implements(spec::ADenialIsCountedAsARefusal)]
fn denied(project: &Project, session: &Session) -> Result<Progress, Halt> {
    todo!()
}

/// An `inference.responded`, shaped by [`Responded::of`]: a terminal lands
/// the message once more the first time and stops the run the second;
/// tool uses mean the forwards follow; neither settles the turn with the
/// model's text.
#[implements(spec::ATurnSettlesOnAResponseWithoutToolUses, spec::AProviderTerminalIsRetriedOnceThenStopsTheRun)]
fn responded(session: &mut Session, turn: &mut Turn, record: &Record) -> Result<Progress, Halt> {
    let body = Responded::of(&record.body).map_err(|why| undecodable(record, &why))?;
    match (body.terminal, body.tool_uses.is_empty(), turn.retry) {
        (Some(_), _, Retry::Untried) => retried(session, turn),
        (Some(sentence), _, Retry::Retried) => Err(Halt::Terminal(sentence)),
        (None, false, Retry::Untried | Retry::Retried) => Ok(Progress::Continue),
        (None, true, Retry::Untried | Retry::Retried) => Ok(Progress::Settled(Settled { text: body.text.unwrap_or_default() })),
    }
}

/// The turn's one retry of a provider terminal: spent, and the message
/// landed once more.
#[implements(spec::AProviderTerminalIsRetriedOnceThenStopsTheRun)]
fn retried(session: &mut Session, turn: &mut Turn) -> Result<Progress, Halt> {
    turn.retry = Retry::Retried;
    land_message(session, turn.message)?;
    Ok(Progress::Continue)
}

/// An `app.session.halted`: the session is over, with the halt's reason.
#[implements(spec::AHaltEndsTheRunWithItsReason)]
fn halted(record: &Record) -> Result<Progress, Halt> {
    todo!()
}

/// A forward matched to its payload: which held payload, who produced it
/// (the completion's addressee), the digest both carry (the completion
/// carries it too), and the call's arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct Pairing {
    /// The payload's index in the held list.
    pub index: usize,
    /// The payload record's producer.
    pub producer: String,
    /// The forward's `payload_digest`, which the payload reproduces.
    pub digest: String,
    /// The payload's `args`, for the tool's typed arguments to decode.
    pub args: Value,
}

/// The first held payload whose digest equals the forward's and whose `to`,
/// like the forward's, is `lid-rs`; none when the forward is another
/// principal's to answer.
#[implements(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest, spec::AForwardToAnotherPrincipalIsNotAnswered)]
pub fn pair(held: &[Held], forward: &Forward) -> Option<Pairing> {
    todo!()
}

/// `sha256` of the canonical JSON of `{"to", "args"}`, hex; reproduces
/// canopy's published vector.
#[implements(spec::ThePayloadDigestReproducesCanopysVector)]
pub fn payload_digest(to: &str, args: &Value) -> String {
    todo!()
}

/// Canopy's canonical JSON: object keys in byte order at every depth,
/// numbers as [`canonical_number`] renders them, strings and the rest as
/// JSON writes them, no whitespace.
#[implements(spec::CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64)]
pub fn canonical_json(value: &Value) -> String {
    todo!()
}

/// A number as canopy's canonical JSON writes it: as the `f64` it is
/// nearest to, in the shortest form that reads back to the same `f64`.
#[implements(spec::CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64)]
pub fn canonical_number(number: &serde_json::Number) -> String {
    todo!()
}

/// The `app.invoke.completed` record to offer in answer to a paired forward
/// — its body [`completed_body`] — and its idempotency key
/// ([`idempotency_key`]).
pub fn completion(forward_cursor: u64, pairing: &Pairing, result: &ToolResult) -> (Offered, String) {
    (Offered { kind: COMPLETED.to_string(), body: completed_body(pairing, result) }, idempotency_key(forward_cursor))
}

/// A completion's body: the pairing's digest as `payload_digest`, `to` the
/// payload record's producer, and `outcome` `success` with `result` or
/// `error` with `error`, as the tool answered.
#[implements(spec::ACompletionAnswersThePayloadsProducer)]
pub fn completed_body(pairing: &Pairing, result: &ToolResult) -> Value {
    todo!()
}

/// A completion's idempotency key, `:<forward cursor>:65534:0`, so a retried
/// append lands once.
#[implements(spec::ACompletionsIdempotencyKeyIsTheForwardsCursor)]
pub fn idempotency_key(forward_cursor: u64) -> String {
    todo!()
}
