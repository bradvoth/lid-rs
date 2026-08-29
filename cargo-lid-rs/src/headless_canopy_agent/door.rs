//! The HTTP client over one canopy door
//! (`docs/intent/headless-canopy-agent/lld.md` § A phase is a session, § The
//! door, on the wire): the dial's settings, the routes this client calls,
//! the one boundary over the HTTP library they share, and the boundary types
//! over the door's JSON. Everything past these types takes domain values.

use lid_rs::implements;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tools::Tool;
use crate::spec;

/// The client over one door: its URL and the API key, which is presented on
/// the dial and the refresh and nowhere else. Stateless: every method is one
/// request through [`Door::request`], over one ureq agent that hands every
/// status back as a response.
#[derive(Clone)]
pub struct Door {
    /// The door's base URL.
    url: String,
    /// The API key, bound to the human's config.
    key: String,
    /// The HTTP agent, configured to answer every status rather than turn
    /// `4xx` and `5xx` into errors, so the refusal's body reaches
    /// [`refusal_of`].
    agent: ureq::Agent,
}

impl Door {
    /// A client over the door at `url`, presenting `key`.
    pub fn new(url: &str, key: &str) -> Self {
        let agent = ureq::Agent::new_with_config(ureq::Agent::config_builder().http_status_as_error(false).build());
        Self { url: url.to_string(), key: key.to_string(), agent }
    }

    /// The one boundary over the HTTP library every method shares: the
    /// request goes through [`Door::exchange`], and its status is the one
    /// decision here — a `2xx` yields the answer's JSON ([`answer_json`]);
    /// anything else yields the door's `refused` sentence, or the status
    /// when there is none ([`refusal_of`]), as the error, which stops the
    /// run.
    #[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
    pub fn request(&self, method: &str, path: &str, bearer: &str, body: Option<&Value>, idem: Option<&str>) -> Result<Value, String> {
        let (status, text) = self.exchange(method, path, bearer, body, idem)?;
        if (200..300).contains(&status) { answer_json(method, path, &text) } else { Err(refusal_of(status, &text)) }
    }

    /// The pass-through over ureq, deciding nothing: an `http::Request`
    /// built for `method` at the door's URL joined with `path`, with
    /// `Authorization: Bearer <bearer>`, `Content-Type: application/json`
    /// and the body's JSON when there is one, and `Idempotency-Key` when
    /// there is one, run by `Agent::run`; the answer is its
    /// `status().as_u16()` and its body read whole by
    /// `Body::read_to_string`, whatever the status, since the agent turns
    /// no status into an error. Failing to reach the door is the error.
    fn exchange(&self, method: &str, path: &str, bearer: &str, body: Option<&Value>, idem: Option<&str>) -> Result<(u16, String), String> {
        todo!()
    }

    /// `POST /sessions`: dials a session with the four settings, presenting
    /// the API key; the credential the door issues, or the door's sentence —
    /// which names the setting when the config pins one.
    #[implements(
        spec::TheKeyIsPresentedOnlyToTheDoor,
        spec::AConfigThatPinsADialledSettingStopsTheRunNamingIt,
        spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces,
    )]
    pub fn start(&self, settings: &Settings) -> Result<Started, String> {
        todo!()
    }

    /// `POST /sessions/{id}/events`: lands what the client offers — `kind`
    /// and `body` alone — under an idempotency key when there is one; the
    /// cursor it landed at. The kinds this client lands are
    /// `app.client.user_message` and `app.invoke.completed` — never
    /// `app.policy.configured`.
    #[implements(spec::NoPolicyRecordIsLandedAfterTheDial, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn send(&self, credential: &Started, offered: &Offered, idem: Option<&str>) -> Result<u64, String> {
        todo!()
    }

    /// `GET /sessions/{id}/tail?after=&wait=&envelope=true`: the records
    /// after a cursor, with their envelopes, parked for up to `wait` seconds
    /// when there are none yet.
    #[implements(spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn tail(&self, credential: &Started, after: u64, wait: u64) -> Result<Page, String> {
        todo!()
    }

    /// `POST /sessions/{id}/refresh`: a fresh credential for the same
    /// session, presenting the API key.
    #[implements(spec::TheKeyIsPresentedOnlyToTheDoor, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn refresh(&self, credential: &Started) -> Result<Started, String> {
        todo!()
    }

    /// `POST /sessions/{id}/stop`: ends the session; its log is sealed.
    #[implements(spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn stop(&self, credential: &Started) -> Result<(), String> {
        todo!()
    }
}

/// A `2xx` answer's text as JSON; one that is not is the error naming the
/// request it answered.
pub fn answer_json(method: &str, path: &str, text: &str) -> Result<Value, String> {
    todo!()
}

/// A non-`2xx` answer's sentence: the `refused` field of its JSON body, or
/// the status when the body carries none.
#[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
pub fn refusal_of(status: u16, body: &str) -> String {
    todo!()
}

/// A `2xx` answer as the boundary type it should be — `what` names the
/// answer in the error when it is not.
pub fn decoded<T: DeserializeOwned>(what: &str, answer: Value) -> Result<T, String> {
    todo!()
}

/// The `cursor` of an answer to a send or a stop.
pub fn cursor_of(answer: &Value) -> Result<u64, String> {
    todo!()
}

/// What the door returns from a dial or a refresh: the session's id, the
/// credential for it, when that credential expires (seconds since the
/// epoch), and the stream the session's records are read from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Started {
    /// The session's id — the join to its sealed log.
    pub session: String,
    /// The credential presented on every request after the dial.
    pub token: String,
    /// When the credential expires, in seconds since the epoch.
    pub expires: u64,
    /// The stream the session's records are read from.
    pub stream: String,
}

/// One page of a session's tail.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Page {
    /// The records after the cursor asked for, in log order.
    pub records: Vec<Record>,
    /// The cursor the page reached — where the next read starts.
    pub through: u64,
}

/// One landed record on the log: what the door assigned when it landed
/// (`cursor`, `producer`, `envelope`) and what its producer wrote (`kind`,
/// `body`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Record {
    /// The record's position on the log.
    pub cursor: u64,
    /// The record's kind, `app.invoke.forward` and the like.
    pub kind: String,
    /// The principal that landed it.
    pub producer: String,
    /// The kind's body.
    pub body: Value,
    /// What the door wrapped it in.
    pub envelope: Envelope,
}

/// The door's wrapping of a landed record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Envelope {
    /// The stream it landed on.
    pub stream: String,
    /// The stream's version at the record.
    pub version: u64,
    /// The idempotency key it landed under, when it had one.
    pub idem: Option<String>,
    /// The record's size in bytes.
    pub size: u64,
}

/// What the client offers the door to land: `kind` and `body` alone; the
/// door assigns the rest when it lands.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Offered {
    /// The record's kind: `app.client.user_message` or `app.invoke.completed`.
    pub kind: String,
    /// The kind's body.
    pub body: Value,
}

/// The dial: exactly the four settings the config leaves to the client —
/// the phase agent's body, the inline policy fixing the tool surface, empty
/// `params` for the model's defaults, and the session's `max_cost`. Every
/// other budget is the config's.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[implements(spec::TheDialCarriesExactlyFourSettings)]
pub struct Settings {
    /// The system prompt: the synced agent file's body.
    pub system: String,
    /// The inline policy: the tools the session admits, and nothing else.
    pub policy: Policy,
    /// Empty: the model's defaults.
    pub params: Value,
    /// The session's cost ceiling, in the provider's currency.
    pub max_cost: f64,
}

/// An inline policy document: the `(requestee, op)` pairs it allows and the
/// declarations of those same tools.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Policy {
    /// The pairs the authorizer admits — exactly the declared tools.
    pub allows: Vec<Allow>,
    /// The tools, each with the JSON schema of its arguments.
    pub tools: Vec<ToolDecl>,
}

/// One `(requestee, op)` pair a policy allows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Allow {
    /// The principal that executes the tool: `lid-rs`.
    pub requestee: String,
    /// The tool's name: `read`, `grep`, `glob`, `edit`, or `write`.
    pub op: String,
}

/// One tool as the policy declares it to the model.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolDecl {
    /// The principal that executes it: `lid-rs`.
    pub requestee: String,
    /// The tool's name.
    pub op: String,
    /// The JSON schema of its arguments, its description in the schema's
    /// `description`.
    pub schema: Value,
}

/// The inline policy admitting exactly `tools` for the requestee `lid-rs`:
/// `allows` lists their pairs and `tools` their declarations
/// ([`super::tools::declarations`]), and nothing else.
#[implements(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
pub fn policy_for(tools: &[Tool]) -> Policy {
    todo!()
}
