//! The HTTP client over one canopy door
//! (`docs/intent/headless-canopy-agent/lld.md` § A phase is a session, § The
//! door, on the wire): the dial's settings, the routes this client calls,
//! the one boundary over the HTTP library they share, and the boundary types
//! over the door's JSON. Everything past these types takes domain values.

use std::time::Duration;

use lid_rs::implements;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::tools::{REQUESTEE, Tool, declarations};
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
    /// request is sent, and sent again while the door sheds it — up to
    /// [`SHED_ATTEMPTS`] times ([`Door::sent`]) — and the status of the
    /// answer that stands is the one decision here: a `2xx` yields the
    /// answer's JSON ([`answer_json`]);
    /// anything else yields the door's `refused` sentence, or the status
    /// when there is none ([`refusal_of`]), as the error, which stops the
    /// run.
    #[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
    pub fn request(&self, method: &str, path: &str, bearer: &str, body: Option<&Value>, idem: Option<&str>) -> Result<Value, String> {
        let (status, text) = self.sent(method, path, bearer, body, idem)?;
        if (200..300).contains(&status) { answer_json(method, path, &text) } else { Err(refusal_of(status, &text)) }
    }

    /// The same request until an answer stands: each attempt goes through
    /// [`Door::exchange`], and [`shed`] is the one decision — a `429` this
    /// attempt may wait out yields the pause, which is slept before the
    /// same request is sent again, under the same idempotency key, so a
    /// shed that in fact landed does not land twice; anything else is the
    /// answer [`Door::request`] then judges.
    #[implements(spec::AShedIsWaitedOutAndTheSameRequestSentAgain, spec::ARetriedSendCarriesTheFirstAttemptsIdempotencyKey)]
    fn sent(&self, method: &str, path: &str, bearer: &str, body: Option<&Value>, idem: Option<&str>) -> Result<(u16, String), String> {
        let mut attempt = 0;
        loop {
            let (status, header, text) = self.exchange(method, path, bearer, body, idem)?;
            let Some(pause) = shed(status, header.as_deref(), attempt) else { return Ok((status, text)) };
            attempt += 1;
            std::thread::sleep(Duration::from_secs(pause));
        }
    }

    /// The pass-through over ureq, deciding nothing: an `http::Request`
    /// built for `method` at the door's URL joined with `path`, with
    /// `Authorization: Bearer <bearer>`, `Content-Type: application/json`
    /// and the body's JSON when there is one, and `Idempotency-Key` when
    /// there is one, run by `Agent::run`; the answer is its
    /// `status().as_u16()`, the `Retry-After` its headers carry — the one
    /// header a shed is waited out by, which nothing past this boundary
    /// reads from the library's header map — and its body read whole by
    /// `Body::read_to_string`, whatever the status, since the agent turns
    /// no status into an error. Failing to reach the door is the error.
    fn exchange(&self, method: &str, path: &str, bearer: &str, body: Option<&Value>, idem: Option<&str>) -> Result<(u16, Option<String>, String), String> {
        let headed = ureq::http::Request::builder()
            .method(method)
            .uri(format!("{}{path}", self.url))
            .header("Authorization", format!("Bearer {bearer}"))
            .header("Content-Type", "application/json");
        let request = idem
            .iter()
            .fold(headed, |builder, key| builder.header("Idempotency-Key", *key))
            .body(body.map(Value::to_string).unwrap_or_default())
            .map_err(|e| format!("building {method} {path}: {e}"))?;
        let mut answer = self.agent.run(request).map_err(|e| format!("reaching the door at {}{path}: {e}", self.url))?;
        let status = answer.status().as_u16();
        let header = answer.headers().get(RETRY_AFTER).and_then(|value| value.to_str().ok()).map(str::to_string);
        let text = answer.body_mut().read_to_string().map_err(|e| format!("reading the answer to {method} {path}: {e}"))?;
        Ok((status, header, text))
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
        decoded("the dial", self.request("POST", "/sessions", &self.key, Some(&json!({ "settings": settings })), None)?)
    }

    /// `POST /sessions/{id}/events`: lands what the client offers — `kind`
    /// and `body` alone — under an idempotency key when there is one; the
    /// cursor it landed at. The kinds this client lands are
    /// `app.client.user_message` and `app.invoke.completed` — never
    /// `app.policy.configured`.
    #[implements(spec::NoPolicyRecordIsLandedAfterTheDial, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn send(&self, credential: &Started, offered: &Offered, idem: Option<&str>) -> Result<u64, String> {
        let path = format!("/sessions/{}/events", credential.session);
        cursor_of(&self.request("POST", &path, &credential.token, Some(&json!(offered)), idem)?)
    }

    /// `GET /sessions/{id}/tail?after=&wait=&envelope=true`: the records
    /// after a cursor, with their envelopes, parked for up to `wait` seconds
    /// when there are none yet.
    #[implements(spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn tail(&self, credential: &Started, after: u64, wait: u64) -> Result<Page, String> {
        let path = format!("/sessions/{}/tail?after={after}&wait={wait}&envelope=true", credential.session);
        decoded("a tail read", self.request("GET", &path, &credential.token, None, None)?)
    }

    /// `POST /sessions/{id}/refresh`: a fresh credential for the same
    /// session, presenting the API key.
    #[implements(spec::TheKeyIsPresentedOnlyToTheDoor, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn refresh(&self, credential: &Started) -> Result<Started, String> {
        let path = format!("/sessions/{}/refresh", credential.session);
        decoded("a refresh", self.request("POST", &path, &self.key, None, None)?)
    }

    /// `POST /sessions/{id}/stop`: ends the session; its log is sealed.
    #[implements(spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    pub fn stop(&self, credential: &Started) -> Result<(), String> {
        let path = format!("/sessions/{}/stop", credential.session);
        cursor_of(&self.request("POST", &path, &credential.token, None, None)?).map(|_| ())
    }
}

/// A `2xx` answer's text as JSON; one that is not is the error naming the
/// request it answered.
pub fn answer_json(method: &str, path: &str, text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| format!("the answer to {method} {path} is not JSON: {e}"))
}

/// A non-`2xx` answer's sentence: the `refused` field of its JSON body, or
/// the status when the body carries none.
#[implements(spec::ADoorRefusalStopsTheRunWithItsSentence)]
pub fn refusal_of(status: u16, body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|doc| doc["refused"].as_str().map(str::to_string))
        .unwrap_or_else(|| format!("the door answered {status}"))
}

/// The header a shed's pause is read from; the one header this client reads.
const RETRY_AFTER: &str = "Retry-After";

/// How many times a shed request is sent again before its answer stands: the
/// fourth `429` is a refusal like any other status.
pub const SHED_ATTEMPTS: u32 = 3;

/// The pause, in seconds, a shed is waited out for when its `Retry-After` is
/// missing or is not a whole number of seconds.
pub const SHED_DEFAULT_PAUSE: u64 = 5;

/// Whether this answer is a shed this attempt may wait out, and for how
/// long. `attempt` counts the sheds this request has already waited out —
/// zero on its first answer, one after the first pause — so a `429` is the
/// door saying *later* while `attempt` is below [`SHED_ATTEMPTS`], and the
/// pause is the seconds [`retry_after`] reads from `header`, the
/// `Retry-After` the answer's headers carry. The request's fourth `429` —
/// `attempt` having reached [`SHED_ATTEMPTS`] — is none: a refusal like any
/// other status, waited out no further. Every status but `429` is none on
/// its first answer, whatever `attempt` holds, so nothing else is ever sent
/// twice.
#[implements(
    spec::AShedIsWaitedOutAndTheSameRequestSentAgain,
    spec::TheFourthShedIsARefusalLikeAnyOtherStatus,
    spec::EveryStatusButAShedIsRefusedOnItsFirstAnswer,
)]
pub fn shed(status: u16, header: Option<&str>, attempt: u32) -> Option<u64> {
    todo!()
}

/// A shed's pause in seconds: `header` — the `Retry-After` the shed's answer
/// carries, `None` when it carries none — read whole as a decimal number of
/// seconds. Anything else the header may hold is unreadable and pauses
/// [`SHED_DEFAULT_PAUSE`]: an empty value, the HTTP-date form the standard
/// also allows, a fraction, a negative, or a number with anything around it.
/// The pause is taken as given; the door's own ceiling is the door's to
/// impose.
#[implements(spec::AShedsPauseIsItsRetryAfterSeconds, spec::AMissingOrUnreadableRetryAfterPausesFiveSeconds)]
pub fn retry_after(header: Option<&str>) -> u64 {
    todo!()
}

/// A `2xx` answer as the boundary type it should be — `what` names the
/// answer in the error when it is not.
pub fn decoded<T: DeserializeOwned>(what: &str, answer: Value) -> Result<T, String> {
    serde_json::from_value(answer).map_err(|e| format!("{what} answered something this client cannot use: {e}"))
}

/// The `cursor` of an answer to a send or a stop.
pub fn cursor_of(answer: &Value) -> Result<u64, String> {
    answer["cursor"].as_u64().ok_or_else(|| format!("the door's answer carries no cursor: {answer}"))
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

/// One tool as the policy declares it to the model: the name it is shown
/// by, the principal that executes it, the `op` a forward names, and the
/// schema its description lives in.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ToolDecl {
    /// The name the model sees and calls the tool by.
    pub name: String,
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
    let allows = tools.iter().map(|tool| Allow { requestee: REQUESTEE.to_string(), op: tool.op().to_string() }).collect();
    Policy { allows, tools: declarations(tools) }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use lid_rs::validates;
    use serde_json::json;

    use super::super::ending::WORKER_TOOLS;
    use super::super::replay::{self, Landed, Replay, Route, SHED_SENTENCE, Seen, SessionScript, Shed, mentions, strings, user};
    use super::super::review::REVIEW_TOOLS;
    use super::super::tools::{REQUESTEE, schema_of};
    use super::*;

    /// A worker's dial with this `max_cost`.
    fn settings(max_cost: f64) -> Settings {
        Settings { system: "You run Phase 3.".to_string(), policy: policy_for(&WORKER_TOOLS), params: json!({}), max_cost }
    }

    /// The requests the replay saw on one route.
    fn on(replay: &Replay, route: Route) -> Vec<Seen> {
        replay.seen().into_iter().filter(|seen| seen.route() == Some(route)).collect()
    }

    /// Asserts every request in a run of them is the same request: the same
    /// method, path, bearer, idempotency key, and body.
    fn same_request(sent: &[Seen]) {
        let each: Vec<String> = sent.iter().map(|seen| format!("{} {} {:?} {:?} {:?}", seen.method, seen.path, seen.bearer, seen.idem, seen.body)).collect();
        assert!(each.windows(2).all(|pair| pair[0] == pair[1]), "the same request, sent again: {sent:?}");
    }

    /// A schema's `properties` keys and `required` names, both sorted.
    fn arguments_of(tool: Tool) -> (Vec<String>, Vec<String>) {
        let schema = schema_of(tool);
        let mut properties: Vec<String> = schema["properties"].as_object().expect("properties").keys().cloned().collect();
        let mut required: Vec<String> = schema["required"].as_array().expect("required").iter().filter_map(Value::as_str).map(str::to_string).collect();
        properties.sort();
        required.sort();
        (properties, required)
    }

    #[test]
    #[validates(spec::TheDialCarriesExactlyFourSettings)]
    fn the_dial_carries_exactly_four_settings() {
        let value = serde_json::to_value(settings(5.0)).expect("serialises");
        let mut keys: Vec<&str> = value.as_object().expect("an object").keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(keys, ["max_cost", "params", "policy", "system"]);
        let replay = Replay::serve(vec![SessionScript::new("s1")]);
        replay.door("key-1").start(&settings(5.0)).expect("dialled");
        let body = on(&replay, Route::Start).remove(0).body.expect("json");
        assert_eq!(body, json!({ "settings": value }), "the dial's body is `settings` alone: the four keys, `params` empty");
        let declared: Vec<(String, String)> = body["settings"]["policy"]["tools"]
            .as_array()
            .expect("the declarations")
            .iter()
            .map(|tool| (tool["name"].as_str().expect("a name on the wire").to_string(), tool["op"].as_str().expect("an op").to_string()))
            .collect();
        let named: Vec<(String, String)> = strings(&["read", "grep", "glob", "edit", "write"]).into_iter().map(|op| (op.clone(), op)).collect();
        assert_eq!(declared, named, "the policy the model is shown carries each tool's name beside its op");
    }

    #[test]
    #[validates(spec::MaxCostIsTheFlagsAmountOrFive)]
    fn the_dial_carries_the_max_cost_given() {
        let replay = Replay::serve(vec![SessionScript::new("s1")]);
        replay.door("key-1").start(&settings(2.5)).expect("dialled");
        assert_eq!(on(&replay, Route::Start).remove(0).body.expect("json")["settings"]["max_cost"], json!(2.5));
    }

    #[test]
    #[validates(spec::TheKeyIsPresentedOnlyToTheDoor, spec::TheClientCallsOnlyTheConverseExecuteAndStopFaces)]
    fn the_key_opens_and_refreshes_and_the_token_drives() {
        let replay = Replay::serve(vec![SessionScript::new("s1").page(vec![replay::requested(1)])]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        door.send(&started, &user("hi"), None).expect("sent");
        door.tail(&started, 0, 1).expect("read");
        let refreshed = door.refresh(&started).expect("refreshed");
        door.stop(&refreshed).expect("stopped");
        let bearers: Vec<(Option<Route>, Option<String>)> = replay.seen().iter().map(|seen| (seen.route(), seen.bearer.clone())).collect();
        let (key, token) = (Some("key-1".to_string()), Some(started.token.clone()));
        let expected = [(Some(Route::Start), key.clone()), (Some(Route::Send), token.clone()), (Some(Route::Tail), token), (Some(Route::Refresh), key), (Some(Route::Stop), Some(refreshed.token))];
        assert_eq!(bearers, expected, "the key on the dial and the refresh; the session's token on the rest; every request one of the five routes");
        assert!(replay.seen().iter().all(|seen| !seen.body.as_ref().is_some_and(|body| body.to_string().contains("key-1"))), "the key is in no body");
    }

    #[test]
    #[validates(spec::AConfigThatPinsADialledSettingStopsTheRunNamingIt)]
    fn a_config_that_pins_a_dialled_setting_refuses_the_dial_naming_it() {
        let replay = Replay::serve(vec![SessionScript::new("s1").refusing(Route::Start, 403, "the config pins `max_cost`")]);
        let err = replay.door("key-1").start(&settings(5.0)).expect_err("refused");
        assert_eq!(err, "the config pins `max_cost`");
    }

    #[test]
    #[validates(spec::ADoorRefusalStopsTheRunWithItsSentence)]
    fn a_refusal_is_the_doors_sentence_or_its_status() {
        assert_eq!(refusal_of(409, r#"{"refused":"the session has stopped"}"#), "the session has stopped");
        assert_eq!(refusal_of(401, r#"{"refused":"no credential"}"#), "no credential");
        mentions(&refusal_of(429, ""), &["429"]);
        mentions(&refusal_of(502, "<html>bad gateway</html>"), &["502"]);
    }

    #[test]
    #[validates(spec::ADoorRefusalStopsTheRunWithItsSentence)]
    fn every_route_yields_the_doors_sentence_when_refused() {
        let script = SessionScript::new("s1")
            .refusing(Route::Tail, 409, "the session has stopped")
            .refusing(Route::Stop, 429, "shed; retry later")
            .refusing(Route::Send, 403, "the credential lacks the converse face")
            .refusing(Route::Refresh, 401, "no such key");
        let replay = Replay::serve(vec![script]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let refusals = [
            door.tail(&started, 0, 1).expect_err("tail"),
            door.send(&started, &user("hi"), None).expect_err("send"),
            door.refresh(&started).expect_err("refresh"),
            door.stop(&started).expect_err("stop"),
        ];
        assert_eq!(refusals, ["the session has stopped", "the credential lacks the converse face", "no such key", "shed; retry later"]);
        assert_eq!(door.request("GET", "/nowhere", "key-1", None, None).expect_err("404"), "no such route");
    }

    #[test]
    #[validates(spec::AShedIsWaitedOutAndTheSameRequestSentAgain)]
    fn a_shed_is_waited_out_and_the_same_request_sent_again() {
        let waited = (shed(429, Some("0"), 0), shed(429, Some("0"), 1), shed(429, Some("0"), 2));
        assert_eq!(waited, (Some(0), Some(0), Some(0)), "a 429 is the door saying later, for each of the three sheds a request may wait out");
        let script = SessionScript::new("s-shed").page(vec![replay::requested(1)]);
        let replay = Replay::shedding(vec![script], vec![Shed::new(Route::Tail, 1, Some("0"))]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let page = door.tail(&started, 0, 25).expect("the answer that stands, after the shed was waited out");
        let reads = on(&replay, Route::Tail);
        assert_eq!((reads.len(), page.records[0].kind.as_str()), (2, "app.policy.configured"), "shed once and sent again, and what stands is the request's own answer");
        same_request(&reads);
    }

    #[test]
    #[validates(spec::AShedsPauseIsItsRetryAfterSeconds)]
    fn a_sheds_pause_is_its_retry_after_seconds() {
        let read = [retry_after(Some("0")), retry_after(Some("1")), retry_after(Some("7")), retry_after(Some("120"))];
        let judged = (read, shed(429, Some("3"), 0), shed(429, Some("0"), 2));
        assert_eq!(judged, ([0, 1, 7, 120], Some(3), Some(0)), "the pause is the header's whole seconds, taken as given: no ceiling");
        let replay = Replay::shedding(vec![SessionScript::new("s-pause")], vec![Shed::new(Route::Send, 1, Some("1"))]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let began = Instant::now();
        door.send(&started, &user("hi"), None).expect("sent again after the shed");
        let paused = began.elapsed();
        let its_own = Duration::from_secs(1)..Duration::from_secs(SHED_DEFAULT_PAUSE);
        assert!(its_own.contains(&paused), "the shed's one second was waited out, and it was that second and not the default five: {paused:?}");
    }

    #[test]
    #[validates(spec::AMissingOrUnreadableRetryAfterPausesFiveSeconds)]
    fn a_missing_or_unreadable_retry_after_pauses_five_seconds() {
        assert_eq!((SHED_DEFAULT_PAUSE, retry_after(None)), (5, 5), "no header at all pauses five seconds");
        let unreadable = ["", " ", "Wed, 21 Oct 2015 07:28:00 GMT", "2.5", "-1", "3s", " 3", "3 ", "five"];
        let pauses: Vec<u64> = unreadable.iter().map(|header| retry_after(Some(header))).collect();
        assert_eq!(pauses, vec![SHED_DEFAULT_PAUSE; unreadable.len()], "empty, an HTTP-date, a fraction, a negative, or a number with anything around it: {unreadable:?}");
        assert_eq!((shed(429, None, 0), shed(429, Some("later"), 1)), (Some(SHED_DEFAULT_PAUSE), Some(SHED_DEFAULT_PAUSE)), "a shed the client cannot read a pause from waits five seconds");
    }

    #[test]
    #[validates(spec::TheFourthShedIsARefusalLikeAnyOtherStatus)]
    fn the_fourth_shed_is_a_refusal_like_any_other_status() {
        let out = (SHED_ATTEMPTS, shed(429, Some("0"), SHED_ATTEMPTS), shed(429, Some("0"), SHED_ATTEMPTS + 1));
        assert_eq!(out, (3, None, None), "the fourth 429 is waited out no further");
        let replay = Replay::shedding(vec![SessionScript::new("s-shed-out")], vec![Shed::new(Route::Send, 9, Some("0"))]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let refusal = door.send(&started, &user("hi"), None).expect_err("shed out");
        let stood = (refusal.as_str(), on(&replay, Route::Send).len(), replay.landed("s-shed-out").len());
        assert_eq!(stood, (SHED_SENTENCE, 4, 0), "the first answer and the three sheds waited out, and then the door's own sentence, as any other refusal");
    }

    #[test]
    #[validates(spec::EveryStatusButAShedIsRefusedOnItsFirstAnswer)]
    fn every_status_but_a_shed_is_refused_on_its_first_answer() {
        let statuses = [200, 201, 400, 401, 403, 404, 409, 428, 430, 500, 502, 503];
        let judged: Vec<Option<u64>> = statuses.iter().flat_map(|status| [shed(*status, Some("0"), 0), shed(*status, Some("0"), 2)]).collect();
        assert_eq!(judged, vec![None; statuses.len() * 2], "nothing but a 429 is a shed, whatever attempt holds: {statuses:?}");
        let replay = Replay::serve(vec![SessionScript::new("s-once").refusing(Route::Send, 503, "the door is down")]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        assert_eq!(door.send(&started, &user("hi"), None).expect_err("refused"), "the door is down");
        assert_eq!(on(&replay, Route::Send).len(), 1, "refused on its first answer: nothing but a shed is ever sent twice");
    }

    #[test]
    #[validates(spec::ARetriedSendCarriesTheFirstAttemptsIdempotencyKey)]
    fn a_retried_send_carries_the_first_attempts_idempotency_key() {
        let replay = Replay::shedding(vec![SessionScript::new("s-idem")], vec![Shed::new(Route::Send, 1, Some("0"))]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let key = ":7:65534:0";
        let cursor = door.send(&started, &user("hi"), Some(key)).expect("sent again after the shed");
        let sends = on(&replay, Route::Send);
        let keys: Vec<Option<String>> = sends.iter().map(|seen| seen.idem.clone()).collect();
        let both = vec![Some(key.to_string()), Some(key.to_string())];
        assert_eq!((cursor, keys, replay.landed("s-idem").len()), (2, both, 1), "the retry carries the first attempt's key, so a shed that in fact landed does not land twice");
        same_request(&sends);
    }

    #[test]
    #[validates(spec::TheTailIsFollowedByParkedReadsWithinTheCredentialsLife)]
    fn a_tail_read_asks_after_the_cursor_parked_for_wait_with_envelopes() {
        let replay = Replay::serve(vec![SessionScript::new("s1").page(vec![replay::requested(1), replay::attempted()])]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        let first = door.tail(&started, 0, 25).expect("read");
        let opening = (first.records.len(), first.through, first.records[0].kind.as_str(), first.records[0].producer.as_str());
        let page = door.tail(&started, first.through, 25).expect("read");
        let shape = (page.records.len(), page.through, page.records[1].kind.as_str(), page.records[0].envelope.stream.as_str());
        let expected = ((1, 1, "app.policy.configured", replay::EXCHANGE), (2, 3, "trellis.attempted", "t/s1"));
        assert_eq!((opening, shape), expected, "the door's own policy record is the session's first; the page follows it");
        let reads = on(&replay, Route::Tail);
        let queries = (reads[0].query("after"), reads[0].query("wait"), reads[0].query("envelope"), reads[1].query("after"));
        assert_eq!(queries, (Some("0".to_string()), Some("25".to_string()), Some("true".to_string()), Some("1".to_string())), "the second read asks after the cursor the first reached");
    }

    #[test]
    #[validates(spec::NoPolicyRecordIsLandedAfterTheDial)]
    fn send_lands_what_is_offered_kind_and_body_alone() {
        let replay = Replay::serve(vec![SessionScript::new("s1")]);
        let door = replay.door("key-1");
        let started = door.start(&settings(5.0)).expect("dialled");
        assert_eq!(door.send(&started, &user("hi"), Some(":7:65534:0")).expect("sent"), 2, "after the door's policy record at 1");
        let landed = Landed { cursor: 2, kind: "app.client.user_message".to_string(), body: json!({ "text": "hi" }), idem: Some(":7:65534:0".to_string()) };
        assert_eq!(replay.landed("s1"), [landed]);
        let sent = on(&replay, Route::Send).remove(0).body.expect("json");
        assert_eq!(sent, json!({ "kind": "app.client.user_message", "body": { "text": "hi" } }), "kind and body alone; never app.policy.configured");
    }

    #[test]
    #[validates(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
    fn the_worker_policy_admits_exactly_the_five_tools() {
        let policy = policy_for(&WORKER_TOOLS);
        let allows: Vec<(&str, &str)> = policy.allows.iter().map(|a| (a.requestee.as_str(), a.op.as_str())).collect();
        assert_eq!(allows, [("lid-rs", "read"), ("lid-rs", "grep"), ("lid-rs", "glob"), ("lid-rs", "edit"), ("lid-rs", "write")]);
        let tools: Vec<(&str, &str, &str)> = policy.tools.iter().map(|t| (t.name.as_str(), t.requestee.as_str(), t.op.as_str())).collect();
        let declared = [("read", "lid-rs", "read"), ("grep", "lid-rs", "grep"), ("glob", "lid-rs", "glob"), ("edit", "lid-rs", "edit"), ("write", "lid-rs", "write")];
        assert_eq!(tools, declared, "the declarations are the same five pairs, each carrying the name the model calls it by");
        let described = policy.tools.iter().all(|t| t.schema["description"].as_str().is_some_and(|d| !d.is_empty()) && t.schema["type"] == json!("object"));
        assert!(described, "each schema is an object carrying its description: {:?}", policy.tools);
    }

    #[test]
    #[validates(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
    fn the_observation_schemas_name_their_arguments() {
        assert_eq!(arguments_of(Tool::Read), (strings(&["limit", "offset", "path"]), strings(&["path"])));
        assert_eq!(arguments_of(Tool::Grep), (strings(&["glob", "path", "pattern"]), strings(&["pattern"])));
        assert_eq!(arguments_of(Tool::Glob), (strings(&["pattern"]), strings(&["pattern"])));
    }

    #[test]
    #[validates(spec::TheWorkerPolicyAdmitsExactlyTheFiveTools)]
    fn the_editing_schemas_name_their_arguments() {
        assert_eq!(arguments_of(Tool::Edit), (strings(&["new_string", "old_string", "path", "replace_all"]), strings(&["new_string", "old_string", "path"])));
        assert_eq!(arguments_of(Tool::Write), (strings(&["content", "path"]), strings(&["content", "path"])));
    }

    #[test]
    #[validates(spec::TheReviewerPolicyAdmitsOnlyTheObservationTools)]
    fn the_reviewer_policy_admits_only_the_observation_tools() {
        let policy = policy_for(&REVIEW_TOOLS);
        let allows: Vec<&str> = policy.allows.iter().map(|a| a.op.as_str()).collect();
        let tools: Vec<&str> = policy.tools.iter().map(|t| t.op.as_str()).collect();
        assert_eq!((allows, tools), (vec!["read", "grep", "glob"], vec!["read", "grep", "glob"]));
        let for_lid_rs = policy.allows.iter().all(|a| a.requestee == REQUESTEE) && policy.tools.iter().all(|t| t.requestee == REQUESTEE);
        assert!(for_lid_rs, "every pair and declaration names the requestee `lid-rs`: {policy:?}");
    }
}
