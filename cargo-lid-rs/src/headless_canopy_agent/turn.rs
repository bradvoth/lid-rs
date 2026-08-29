//! Driving a turn (`docs/intent/headless-canopy-agent/lld.md` § Driving a
//! turn): one open session, the loop that lands a user message and follows
//! the tail until the turn settles, the record bodies it acts on — decoded
//! once, by kind, where the record is classified — and the invoke
//! choreography: pairing a forward with its held payload by digest, and
//! answering it.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use lid_rs::implements;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::door::{Door, Offered, Record, Settings, Started};
use super::tools::{REQUESTEE, Tool, ToolResult, execute};
use crate::phase::Phase;
use crate::phase::tally::{self, Event};
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
    /// for the session's life — and prints [`opened_line`] for the session's
    /// id as it opens, so every session a phase opens is printed before
    /// anything else the phase prints. The session starts at cursor 0 with
    /// nothing held.
    #[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
    pub fn open(door: &Door, settings: &Settings, phase: Phase, tools: Vec<Tool>) -> Result<Self, String> {
        let credential = door.start(settings)?;
        println!("{}", opened_line(&credential.session));
        Ok(Self { door: door.clone(), credential, cursor: 0, held: Vec::new(), phase, tools })
    }

    /// The tally key and the commit's agent: `canopy:<session>`.
    #[implements(spec::TheCommitNamesItsSessionAsTheAgent)]
    pub fn agent_id(&self) -> String {
        format!("canopy:{}", self.credential.session)
    }

    /// Stops the session through the door; its log is sealed.
    #[implements(spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    pub fn stop(self) -> Result<(), String> {
        self.door.stop(&self.credential)
    }
}

/// What the run prints as a session opens: the session's id — the join to
/// its sealed log — on one line.
#[implements(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
pub fn opened_line(id: &str) -> String {
    format!("opened canopy session {id}")
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
        let landed: RespondedBody = serde_json::from_value(body.clone()).map_err(|e| format!("an inference.responded body: {e}"))?;
        match (landed.terminal, landed.response) {
            (Some(sentence), Some(_) | None) => Ok(Responded { text: None, tool_uses: Vec::new(), terminal: Some(sentence) }),
            (None, Some(answer)) => {
                Ok(Responded { text: answer.text, tool_uses: answer.tool_uses.into_iter().map(|use_of| use_of.op).collect(), terminal: None })
            }
            (None, None) => Err("an inference.responded body carries neither a response nor a terminal".to_string()),
        }
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
        match kind {
            "app.invoke.payload" => Kind::Payload,
            "app.invoke.forward" => Kind::Forward,
            "app.invoke.denied" => Kind::Denied,
            "inference.responded" => Kind::Responded,
            "app.session.halted" => Kind::Halted,
            _ => Kind::Other,
        }
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
    Offered { kind: USER_MESSAGE.to_string(), body: json!({ "text": text }) }
}

/// Lands one user message; the door's refusal ends the session with its
/// sentence.
#[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
pub fn land_message(session: &Session, text: &str) -> Result<u64, Halt> {
    session.door.send(&session.credential, &user_message(text), None).map_err(Halt::Refused)
}

/// Seconds since the epoch, as the door writes `expires`.
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| since.as_secs())
}

/// The credential to read with: this one while it has more than five
/// seconds of life at `now`, else a fresh one from the door for the same
/// session; the door's refusal ends the session with its sentence.
#[implements(spec::TheCredentialIsRefreshedBeforeItExpires)]
pub fn usable(door: &Door, credential: &Started, now: u64) -> Result<Started, Halt> {
    if credential.expires.saturating_sub(now) > REFRESH_MARGIN {
        Ok(credential.clone())
    } else {
        door.refresh(credential).map_err(Halt::Refused)
    }
}

/// How long to park the next read: 25 s, the door's ceiling, clamped to the
/// credential's remaining life at `now`.
#[implements(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife)]
pub fn wait_for(credential: &Started, now: u64) -> u64 {
    TAIL_WAIT.min(credential.expires.saturating_sub(now))
}

/// When the tail last delivered — the quiet clock's one decision: now if
/// the page carries records, else as before, so [`quiet_checked`] measures
/// fifteen minutes from the last delivery and not from the last read.
#[implements(spec::AQuietTailForFifteenMinutesStopsTheRun)]
pub fn delivery(previous: Instant, records: &[Record]) -> Instant {
    if records.is_empty() { previous } else { Instant::now() }
}

/// The page's records to act on; an empty page fifteen minutes after the
/// tail last delivered ends the session as quiet.
#[implements(spec::AQuietTailForFifteenMinutesStopsTheRun)]
pub fn quiet_checked(delivered: Instant, records: Vec<Record>) -> Result<Vec<Record>, Halt> {
    if records.is_empty() && delivered.elapsed() >= QUIET_TAIL { Err(Halt::Quiet) } else { Ok(records) }
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
    serde_json::from_value(record.body.clone()).map_err(|e| undecodable(record, &e.to_string()))
}

/// A record whose body is not what its kind promises: the halt naming the
/// record and what failed.
pub fn undecodable(record: &Record, what: &str) -> Halt {
    Halt::Refused(format!("the `{}` record at cursor {} could not be read: {what}", record.kind, record.cursor))
}

/// An `app.invoke.payload` held — its cursor, its producer, its body — until
/// its forward arrives.
#[implements(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest)]
fn held(session: &mut Session, record: &Record) -> Result<Progress, Halt> {
    let payload: Payload = body_of(record)?;
    session.held.push(Held { cursor: record.cursor, producer: record.producer.clone(), payload });
    Ok(Progress::Continue)
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
    tally::record(project, &session.agent_id(), Event::PolicyRefusal).map_err(Halt::Refused)?;
    Ok(Progress::Continue)
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
    let body: Halted = body_of(record)?;
    Err(Halt::Halted(body.reason))
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
    let ours = |waiting: &&Held| forward.to == REQUESTEE && waiting.payload.to == REQUESTEE;
    held.iter()
        .enumerate()
        .find(|(_, waiting)| ours(waiting) && payload_digest(&waiting.payload.to, &waiting.payload.args) == forward.payload_digest)
        .map(|(index, waiting)| Pairing {
            index,
            producer: waiting.producer.clone(),
            digest: forward.payload_digest.clone(),
            args: waiting.payload.args.clone(),
        })
}

/// `sha256` of the canonical JSON of `{"to", "args"}`, hex; reproduces
/// canopy's published vector.
#[implements(spec::ThePayloadDigestReproducesCanopysVector)]
pub fn payload_digest(to: &str, args: &Value) -> String {
    let canonical = format!(r#"{{"to":{},"args":{}}}"#, canonical_json(&Value::String(to.to_string())), canonical_json(args));
    Sha256::digest(canonical.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Canopy's canonical JSON: object keys in byte order at every depth,
/// numbers as [`canonical_number`] renders them, strings and the rest as
/// JSON writes them, no whitespace.
#[implements(spec::CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64)]
pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(fields) => {
            let mut keys: Vec<&String> = fields.keys().collect();
            keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            let pairs: Vec<String> = keys.iter().map(|key| format!("{}:{}", Value::String((*key).clone()), canonical_json(&fields[*key]))).collect();
            format!("{{{}}}", pairs.join(","))
        }
        Value::Array(items) => format!("[{}]", items.iter().map(canonical_json).collect::<Vec<String>>().join(",")),
        Value::Number(number) => canonical_number(number),
        Value::String(_) | Value::Bool(_) | Value::Null => value.to_string(),
    }
}

/// A number as canopy's canonical JSON writes it: as the `f64` it is
/// nearest to, in the shortest digits that read back to the same `f64` —
/// `serde_json`'s own rendering of that `f64`, whose ties go to the even
/// digit as canopy's do, where the standard library's go away from zero —
/// with a whole number's trailing `.0` dropped and a positive exponent
/// written out as the digits it stands for (`written_out`), since canopy
/// writes `1756339200123456800` where `serde_json` would write
/// `1.7563392001234568e18`. The one decision here is whether the rendering
/// carries such an exponent.
#[implements(spec::CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64)]
pub fn canonical_number(number: &serde_json::Number) -> String {
    let shortest = Value::from(number.as_f64().unwrap_or(f64::NAN)).to_string();
    let whole = shortest.strip_suffix(".0").unwrap_or(&shortest);
    match whole.split_once('e').and_then(|(mantissa, exponent)| exponent.parse::<u32>().ok().map(|power| (mantissa, power))) {
        None => whole.to_string(),
        Some((mantissa, power)) => written_out(mantissa, power),
    }
}

/// A mantissa and a positive power as the digits they stand for: the
/// mantissa's digits, its sign kept, padded with zeros to the `power + 1`
/// places the exponent puts before the decimal point — `1.7563392001234568`
/// at power 18 is `1756339200123456800`.
fn written_out(mantissa: &str, power: u32) -> String {
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let zeros = "0".repeat((power as usize + 1).saturating_sub(digits.len()));
    format!("{}{digits}{zeros}", &mantissa[..usize::from(mantissa.starts_with('-'))])
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
    let (outcome, field, text) = match result {
        Ok(answer) => ("success", "result", answer),
        Err(error) => ("error", "error", error),
    };
    let mut body = json!({ "to": pairing.producer, "payload_digest": pairing.digest, "outcome": outcome });
    body[field] = Value::String(text.clone());
    body
}

/// A completion's idempotency key, `:<forward cursor>:65534:0`, so a retried
/// append lands once.
#[implements(spec::ACompletionsIdempotencyKeyIsTheForwardsCursor)]
pub fn idempotency_key(forward_cursor: u64) -> String {
    format!(":{forward_cursor}:65534:0")
}

#[cfg(test)]
mod tests {
    use lid_rs::validates;
    use serde_json::json;

    use super::super::door::{Envelope, policy_for};
    use super::super::ending::WORKER_TOOLS;
    use super::super::replay::{self, REQUESTOR, Replay, Route, Seen, SessionScript, mentions, sha256, strings};
    use super::super::tools::REQUESTEE;
    use super::super::{KEY_VARIABLE, PRODUCTION_DOOR};
    use super::*;
    use crate::phase::fixture;
    use crate::phase::tally;

    /// The variable naming the door the end-to-end run dials; the
    /// production door when unset.
    const DOOR_VARIABLE: &str = "CANOPY_DOOR";

    /// Canopy's canonical JSON for a `read` of `src/hello.rs` by this program.
    const READ_HELLO: &str = r#"{"to":"lid-rs","args":{"path":"src/hello.rs"}}"#;

    /// The same for `src/lib.rs`.
    const READ_LIB: &str = r#"{"to":"lid-rs","args":{"path":"src/lib.rs"}}"#;

    /// A worker's dial.
    fn settings() -> Settings {
        Settings { system: "You run Phase 3.".to_string(), policy: policy_for(&WORKER_TOOLS), params: json!({}), max_cost: 5.0 }
    }

    /// A worker session opened on the replay.
    fn open(replay: &Replay) -> Session {
        Session::open(&replay.door("k"), &settings(), Phase::Three, WORKER_TOOLS.to_vec()).expect("opened")
    }

    /// A held `read` payload addressed to `to`.
    fn held_read(cursor: u64, to: &str, path: &str) -> Held {
        Held { cursor, producer: REQUESTOR.to_string(), payload: Payload { to: to.to_string(), args: json!({ "path": path }) } }
    }

    fn forward_body(to: &str, op: &str, digest: &str) -> Forward {
        Forward { to: to.to_string(), op: op.to_string(), payload_digest: digest.to_string() }
    }

    fn credential(expires: u64) -> Started {
        Started { session: "s".to_string(), token: "t".to_string(), expires, stream: "st".to_string() }
    }

    fn record(cursor: u64) -> Record {
        Record { cursor, kind: "trellis.attempted".to_string(), producer: "y".to_string(), body: json!({}), envelope: Envelope { stream: "t".to_string(), version: 1, idem: None, size: 2 } }
    }

    /// The `after` of every tail read the replay saw.
    fn afters(replay: &Replay) -> Vec<Option<String>> {
        replay.seen().iter().filter(|s| s.route() == Some(Route::Tail)).map(|s| s.query("after")).collect()
    }

    #[test]
    #[validates(spec::EveryPhasePrintsItsSessionsAndItsEnding)]
    fn open_dials_the_session_and_holds_its_credential() {
        let line = opened_line("s-open");
        mentions(&line, &["s-open"]);
        let replay = Replay::serve(vec![SessionScript::new("s-open")]);
        let session = Session::open(&replay.door("k"), &settings(), Phase::Four, WORKER_TOOLS.to_vec()).expect("opened");
        let shape = (session.credential.session.as_str(), session.cursor, session.held.len(), session.phase, session.tools.as_slice());
        assert_eq!(shape, ("s-open", 0, 0, Phase::Four, WORKER_TOOLS.as_slice()));
        assert_eq!(session.agent_id(), "canopy:s-open");
        assert_eq!((replay.opened(), line.lines().count()), (strings(&["s-open"]), 1), "one session opened; what open prints for it is one line");
    }

    #[test]
    #[validates(spec::TheCommitNamesItsSessionAsTheAgent)]
    fn the_agent_id_is_the_session_under_canopy() {
        let session = replay::session(Door::new("http://127.0.0.1:1", "k"), "3f0c1c9a-6f6e", Phase::Three, vec![]);
        assert_eq!(session.agent_id(), "canopy:3f0c1c9a-6f6e");
    }

    #[test]
    #[validates(spec::EverySessionIsStoppedWhenItsPhaseEnds)]
    fn stop_seals_the_session_through_the_door() {
        let replay = Replay::serve(vec![SessionScript::new("s-stop")]);
        open(&replay).stop().expect("stopped");
        assert!(replay.stopped("s-stop"));
        let routes: Vec<Option<Route>> = replay.seen().iter().map(Seen::route).collect();
        assert_eq!(routes, [Some(Route::Start), Some(Route::Stop)]);
    }

    #[test]
    #[validates(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest, spec::ADenialIsCountedAsARefusal, spec::ATurnSettlesOnAResponseWithoutToolUses, spec::AHaltEndsTheRunWithItsReason)]
    fn a_records_kind_is_classified_once() {
        let names = ["app.invoke.payload", "app.invoke.forward", "app.invoke.denied", "inference.responded", "app.session.halted", "inference.requested", "app.policy.configured", "trellis.attempted", "app.invoke.completed", "app.client.user_message"];
        let kinds: Vec<Kind> = names.iter().map(|k| Kind::of(k)).collect();
        assert_eq!(kinds, [Kind::Payload, Kind::Forward, Kind::Denied, Kind::Responded, Kind::Halted, Kind::Other, Kind::Other, Kind::Other, Kind::Other, Kind::Other]);
    }

    #[test]
    #[validates(spec::ThePayloadDigestReproducesCanopysVector)]
    fn the_payload_digest_reproduces_canopys_vector() {
        let args = json!({ "b": 1.0, "a": 2, "é": [3.0, "x"], "A": {}, "tie": 75_251_554_695_404.12_f64, "at": 1_756_339_200_123_456_789_i64 });
        assert_eq!(payload_digest("executor", &args), "0d0f02537b19d987f967216664c91f7a92dc22364cd4c840d825fbaac5448f10");
        assert_eq!(payload_digest(REQUESTEE, &json!({ "path": "src/hello.rs" })), sha256(READ_HELLO));
    }

    #[test]
    #[validates(spec::CanonicalJsonOrdersKeysByByteAndRendersNumbersAsF64)]
    fn canonical_json_orders_keys_by_byte_at_every_depth_and_renders_numbers_as_f64() {
        let value = json!({ "b": 1.0, "a": 2, "é": [3.0, "x", { "z": null, "y": true }], "A": {}, "tie": 75_251_554_695_404.12_f64, "at": 1_756_339_200_123_456_789_i64 });
        assert_eq!(canonical_json(&value), r#"{"A":{},"a":2,"at":1756339200123456800,"b":1,"tie":75251554695404.12,"é":[3,"x",{"y":true,"z":null}]}"#);
        let numbers: Vec<String> = [json!(1.0), json!(2), json!(-0.5), json!(1_756_339_200_123_456_789_i64), json!(10.0)].iter().map(canonical_json).collect();
        assert_eq!(numbers, strings(&["1", "2", "-0.5", "1756339200123456800", "10"]));
        assert_eq!(canonical_number(&serde_json::Number::from_f64(0.1).expect("finite")), "0.1");
    }

    #[test]
    #[validates(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest)]
    fn a_forward_is_paired_with_the_first_held_payload_of_its_digest() {
        let held = [held_read(5, REQUESTEE, "src/lib.rs"), held_read(7, REQUESTEE, "src/hello.rs"), held_read(9, REQUESTEE, "src/hello.rs")];
        let pairing = pair(&held, &forward_body(REQUESTEE, "read", &sha256(READ_HELLO))).expect("paired");
        assert_eq!(pairing, Pairing { index: 1, producer: REQUESTOR.to_string(), digest: sha256(READ_HELLO), args: json!({ "path": "src/hello.rs" }) });
        assert_eq!(pair(&held, &forward_body(REQUESTEE, "read", &sha256(READ_LIB))).expect("paired").index, 0);
        assert_eq!(pair(&held, &forward_body(REQUESTEE, "read", "0000")), None, "no held payload of that digest");
    }

    #[test]
    #[validates(spec::AForwardToAnotherPrincipalIsNotAnswered)]
    fn a_forward_or_payload_to_another_principal_pairs_with_nothing() {
        let ours = [held_read(5, REQUESTEE, "src/hello.rs")];
        assert_eq!(pair(&ours, &forward_body("mcp.relay", "read", &sha256(READ_HELLO))), None, "the forward is another's");
        let theirs = [held_read(5, "mcp.relay", "src/hello.rs")];
        let relay_digest = sha256(r#"{"to":"mcp.relay","args":{"path":"src/hello.rs"}}"#);
        assert_eq!(pair(&theirs, &forward_body(REQUESTEE, "read", &relay_digest)), None, "the payload is another's");
        assert_eq!(pair(&theirs, &forward_body("mcp.relay", "read", &relay_digest)), None, "both are another's");
    }

    #[test]
    #[validates(spec::AForwardToAnotherPrincipalIsNotAnswered)]
    fn a_forward_to_another_principal_lands_no_completion() {
        let (_dir, project) = fixture::copy("canopy-turn-other-principal");
        let relay = json!({ "path": "src/hello.rs" });
        let digest = sha256(r#"{"to":"mcp.relay","args":{"path":"src/hello.rs"}}"#);
        let page = vec![replay::responded_with_tools("Asking the relay.", &[("read", relay.clone())]), replay::payload_to("mcp.relay", relay), replay::forward_to("mcp.relay", "read", &digest)];
        let replay = Replay::serve(vec![SessionScript::new("s-relay").page(page).page(replay::settling_page("done"))]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "go").expect("settled"), Settled { text: "done".to_string() });
        assert!(replay::completions(&replay.landed("s-relay")).is_empty(), "not this program's to answer");
    }

    #[test]
    #[validates(spec::AForwardIsPairedWithTheHeldPayloadOfItsDigest, spec::ACompletionAnswersThePayloadsProducer, spec::ACompletionsIdempotencyKeyIsTheForwardsCursor)]
    fn a_paired_forward_runs_its_tool_and_lands_the_completion_under_the_forwards_cursor() {
        let (_dir, project) = fixture::copy("canopy-turn-completion");
        let digest = sha256(READ_HELLO);
        let script = SessionScript::new("s-read").page(replay::tool_call_page("read", json!({ "path": "src/hello.rs" }), &digest)).page(replay::settling_page("ok"));
        let replay = Replay::serve(vec![script]);
        let mut session = open(&replay);
        assert_eq!((drive(&project, &mut session, "read hello").expect("settled").text, session.held.len()), ("ok".to_string(), 0));
        let completion = replay::completions(&replay.landed("s-read")).remove(0);
        // The door's policy record is 1 and the user message 2; the page's forward is its fifth record: cursor 7.
        assert_eq!(completion.idem, Some(":7:65534:0".to_string()));
        let body = &completion.body;
        assert_eq!((body["to"].as_str(), body["payload_digest"].as_str(), body["outcome"].as_str()), (Some(REQUESTOR), Some(digest.as_str()), Some("success")));
        mentions(body["result"].as_str().expect("the tool's text"), &["The hello slice"]);
    }

    #[test]
    #[validates(spec::ACompletionAnswersThePayloadsProducer)]
    fn a_completions_body_carries_the_digest_the_producer_and_the_outcome() {
        let pairing = Pairing { index: 0, producer: REQUESTOR.to_string(), digest: "digest-7".to_string(), args: json!({}) };
        let success = json!({ "to": "requestor", "payload_digest": "digest-7", "outcome": "success", "result": "1: text" });
        assert_eq!(completed_body(&pairing, &Ok("1: text".to_string())), success);
        let error = json!({ "to": "requestor", "payload_digest": "digest-7", "outcome": "error", "error": "tool failed" });
        assert_eq!(completed_body(&pairing, &Err("tool failed".to_string())), error);
        let (offered, idem) = completion(12, &pairing, &Ok(String::new()));
        assert_eq!((offered.kind.as_str(), idem.as_str(), offered.body["outcome"].as_str()), (COMPLETED, ":12:65534:0", Some("success")));
    }

    #[test]
    #[validates(spec::ACompletionsIdempotencyKeyIsTheForwardsCursor)]
    fn a_completions_idempotency_key_is_the_forwards_cursor() {
        assert_eq!(idempotency_key(6), ":6:65534:0");
        assert_eq!(idempotency_key(65_535), ":65535:65534:0");
    }

    #[test]
    #[validates(spec::ADenialIsCountedAsARefusal)]
    fn a_denial_runs_nothing_and_counts_as_a_refusal() {
        let (_dir, project) = fixture::copy("canopy-turn-denied");
        let args = json!({ "command": "rm -rf /" });
        let digest = sha256(r#"{"to":"lid-rs","args":{"command":"rm -rf /"}}"#);
        let page = vec![replay::responded_with_tools("Trying bash.", &[("bash", args.clone())]), replay::payload(args), replay::denied(&digest)];
        let replay = Replay::serve(vec![SessionScript::new("s-denied").page(page).page(replay::settling_page("fine"))]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "go").expect("settled").text, "fine");
        assert!(replay::completions(&replay.landed("s-denied")).is_empty(), "nothing executed, nothing answered");
        assert_eq!(tally::load(&project, "canopy:s-denied").expect("tally").policy_refusals, 1);
    }

    #[test]
    #[validates(spec::ATurnSettlesOnAResponseWithoutToolUses)]
    fn a_response_is_shaped_from_its_text_and_tool_uses() {
        let asked = replay::responded_with_tools("Let me look.", &[("read", json!({ "path": "a" })), ("grep", json!({ "pattern": "b" }))]);
        let with_tools = Responded::of(&asked["body"]).expect("shaped");
        assert_eq!(with_tools, Responded { text: Some("Let me look.".to_string()), tool_uses: strings(&["read", "grep"]), terminal: None });
        let settled = Responded::of(&replay::responded("Done.")["body"]).expect("shaped");
        assert_eq!(settled, Responded { text: Some("Done.".to_string()), tool_uses: vec![], terminal: None });
        assert!(Responded::of(&json!({ "echo": {}, "raw": {} })).is_err(), "neither response nor terminal");
    }

    #[test]
    #[validates(spec::ATurnSettlesOnAResponseWithoutToolUses)]
    fn the_tail_is_followed_on_past_a_response_with_tool_uses() {
        let (_dir, project) = fixture::copy("canopy-turn-settle");
        let digest = sha256(READ_HELLO);
        let script = SessionScript::new("s-settle").page(replay::tool_call_page("read", json!({ "path": "src/hello.rs" }), &digest)).page(replay::settling_page("The file greets."));
        let replay = Replay::serve(vec![script]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "look").expect("settled"), Settled { text: "The file greets.".to_string() });
        assert_eq!(replay::user_messages(&replay.landed("s-settle")), strings(&["look"]), "one user message outstanding for the turn");
        assert!(afters(&replay).len() >= 2, "the tail was followed on past the tool uses");
    }

    #[test]
    #[validates(spec::AProviderTerminalIsRetriedOnceThenStopsTheRun)]
    fn a_provider_terminal_lands_the_message_once_more() {
        let (_dir, project) = fixture::copy("canopy-turn-terminal-once");
        let shaped = Responded::of(&replay::terminal(replay::TERMINAL_SENTENCE)["body"]).expect("shaped");
        assert_eq!(shaped, Responded { text: None, tool_uses: vec![], terminal: Some(replay::TERMINAL_SENTENCE.to_string()) });
        let first = vec![replay::requested(1), replay::attempted(), replay::terminal(replay::TERMINAL_SENTENCE)];
        let replay = Replay::serve(vec![SessionScript::new("s-terminal").page(first).page(replay::settling_page("second time lucky"))]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "hello").expect("settled").text, "second time lucky");
        assert_eq!(replay::user_messages(&replay.landed("s-terminal")), strings(&["hello", "hello"]));
    }

    #[test]
    #[validates(spec::AProviderTerminalIsRetriedOnceThenStopsTheRun)]
    fn a_second_terminal_stops_the_run_naming_the_providers_sentence() {
        let (_dir, project) = fixture::copy("canopy-turn-terminal-twice");
        let script = SessionScript::new("s-terminal2").page(vec![replay::terminal(replay::TERMINAL_SENTENCE)]).page(vec![replay::terminal(replay::TERMINAL_SENTENCE)]);
        let replay = Replay::serve(vec![script]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "hello").expect_err("stopped"), Halt::Terminal(replay::TERMINAL_SENTENCE.to_string()));
        assert_eq!(replay::user_messages(&replay.landed("s-terminal2")).len(), 2, "landed once more, not twice");
    }

    #[test]
    #[validates(spec::AHaltEndsTheRunWithItsReason)]
    fn a_halt_ends_the_turn_with_its_reason() {
        let (_dir, project) = fixture::copy("canopy-turn-halted");
        let replay = Replay::serve(vec![SessionScript::new("s-halt").page(vec![replay::requested(1), replay::halted("context_limit reached")])]);
        let mut session = open(&replay);
        assert_eq!(drive(&project, &mut session, "go").expect_err("halted"), Halt::Halted("context_limit reached".to_string()));
    }

    #[test]
    #[validates(spec::AQuietTailForFifteenMinutesStopsTheRun)]
    fn a_quiet_tail_for_fifteen_minutes_stops_the_run() {
        let long_ago = Instant::now().checked_sub(QUIET_TAIL + Duration::from_secs(1)).expect("an instant fifteen minutes ago");
        assert_eq!((QUIET_TAIL, quiet_checked(long_ago, vec![]).expect_err("quiet")), (Duration::from_secs(900), Halt::Quiet));
        let just_now = Instant::now();
        let (not_yet, delivered) = (quiet_checked(just_now, vec![]).expect("not yet"), quiet_checked(long_ago, vec![record(1)]).expect("a delivery"));
        assert_eq!((not_yet, delivered), (vec![], vec![record(1)]));
    }

    #[test]
    #[validates(spec::AQuietTailForFifteenMinutesStopsTheRun)]
    fn the_quiet_clock_restarts_on_a_delivery_only() {
        let long_ago = Instant::now().checked_sub(QUIET_TAIL).expect("an instant fifteen minutes ago");
        assert_eq!(delivery(long_ago, &[]), long_ago, "an empty page leaves the clock");
        assert!(delivery(long_ago, &[record(1)]) > long_ago, "a delivery restarts it");
    }

    #[test]
    #[validates(spec::TheCredentialIsRefreshedBeforeItExpires)]
    fn the_credential_is_refreshed_within_five_seconds_of_its_expiry() {
        let replay = Replay::serve(vec![SessionScript::new("s-fresh")]);
        let door = replay.door("key-1");
        let started = door.start(&settings()).expect("dialled");
        assert_eq!((REFRESH_MARGIN, usable(&door, &started, started.expires - 60).expect("still good")), (5, started.clone()));
        let refreshed = usable(&door, &started, started.expires - 4).expect("refreshed");
        assert!(refreshed.session == started.session && refreshed.token != started.token && refreshed.expires > started.expires - 4, "{refreshed:?}");
        let routes: Vec<Option<Route>> = replay.seen().iter().map(Seen::route).collect();
        assert_eq!(routes, [Some(Route::Start), Some(Route::Refresh)], "one refresh, for the same session");
    }

    #[test]
    #[validates(spec::TheCredentialIsRefreshedBeforeItExpires)]
    fn a_refresh_mid_turn_keeps_the_session_and_its_cursor() {
        let (_dir, project) = fixture::copy("canopy-turn-refresh");
        let replay = Replay::serve(vec![SessionScript::new("s-expiring").expiring_in(3).page(replay::settling_page("ok"))]);
        let mut session = open(&replay);
        let first = session.credential.clone();
        assert_eq!(drive(&project, &mut session, "go").expect("settled").text, "ok");
        assert!(session.credential.session == first.session && session.credential.token != first.token, "refreshed: {:?}", session.credential);
        assert_eq!(session.cursor, 4, "the door's policy record, the user message, then the page's two records, read on the refreshed credential");
    }

    #[test]
    #[validates(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife)]
    fn a_read_is_parked_twenty_five_seconds_clamped_to_the_credentials_life() {
        let waits = (wait_for(&credential(1_000), 900), wait_for(&credential(910), 900), wait_for(&credential(925), 900), wait_for(&credential(900), 900));
        assert_eq!((TAIL_WAIT, waits), (25, (25, 10, 25, 0)));
    }

    #[test]
    #[validates(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife)]
    fn the_tail_is_followed_from_the_cursor_the_last_page_reached() {
        let (_dir, project) = fixture::copy("canopy-turn-after");
        let script = SessionScript::new("s-after").page(vec![replay::requested(1), replay::attempted()]).page(replay::settling_page("ok"));
        let replay = Replay::serve(vec![script]);
        let mut session = open(&replay);
        drive(&project, &mut session, "go").expect("settled");
        // The door's policy record is 1 and the message 2; the first read returns both; the next two return the pages through 4 and 6.
        assert_eq!(afters(&replay), [Some("0".to_string()), Some("2".to_string()), Some("4".to_string())]);
        assert!(replay.seen().iter().filter(|s| s.route() == Some(Route::Tail)).all(|s| s.query("wait") == Some("25".to_string())), "every read parked 25 s");
    }

    #[test]
    #[validates(spec::ADoorRefusalStopsTheRunWithItsSentence)]
    fn a_refused_send_or_read_ends_the_turn_with_the_doors_sentence() {
        let (_dir, project) = fixture::copy("canopy-turn-refused");
        let scripts = vec![SessionScript::new("s-nosend").refusing(Route::Send, 409, "the session has stopped"), SessionScript::new("s-notail").refusing(Route::Tail, 429, "shed")];
        let replay = Replay::serve(scripts);
        let mut first = open(&replay);
        assert_eq!(drive(&project, &mut first, "go").expect_err("refused"), Halt::Refused("the session has stopped".to_string()));
        let mut second = open(&replay);
        assert_eq!(drive(&project, &mut second, "go").expect_err("refused"), Halt::Refused("shed".to_string()));
        assert_eq!(land_message(&first, "again").expect_err("refused"), Halt::Refused("the session has stopped".to_string()));
    }

    /// The end-to-end run the LLD's validation strategy names, by hand:
    /// `cargo test -p cargo-lid-rs --lib -- --ignored end_to_end` with
    /// `CANOPY_KEY` set, and `CANOPY_DOOR` when the door is not the
    /// production one. One session with the five-tool policy on the real
    /// door, one user message asking the model to `read` `README.md`, the
    /// turn driven, the session stopped. Not a validation: an ignored
    /// `#[validates]` would count as passing in the red run, and this run
    /// needs a credential the hermetic suite must never have. Without the
    /// key it says so and does nothing.
    #[test]
    #[ignore = "dials a real door with CANOPY_KEY; run by hand"]
    fn end_to_end_one_turn_on_a_real_door() {
        let Ok(key) = std::env::var(KEY_VARIABLE) else {
            println!("{KEY_VARIABLE} is unset: the end-to-end run is skipped");
            return;
        };
        let url = std::env::var(DOOR_VARIABLE).unwrap_or_else(|_| PRODUCTION_DOOR.to_string());
        let project = Project::load_graph().expect("this workspace");
        let system = "You are checking a tool loop. Call the `read` tool on `README.md`, then answer with its first line.".to_string();
        let settings = Settings { system, policy: policy_for(&WORKER_TOOLS), params: json!({}), max_cost: 1.0 };
        let mut session = Session::open(&Door::new(&url, &key), &settings, Phase::Three, WORKER_TOOLS.to_vec()).expect("dialled");
        let turn = drive(&project, &mut session, "Read `README.md` with the `read` tool and tell me its first line.");
        session.stop().expect("stopped");
        match turn {
            Ok(settled) => assert!(!settled.text.trim().is_empty(), "the model answered"),
            Err(Halt::Terminal(sentence) | Halt::Halted(sentence)) => assert!(!sentence.trim().is_empty(), "the provider's or the platform's sentence"),
            Err(halt @ (Halt::Quiet | Halt::Refused(_))) => panic!("the turn ended without a sentence: {halt:?}"),
        }
    }
}
