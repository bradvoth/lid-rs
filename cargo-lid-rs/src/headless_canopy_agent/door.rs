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

#[cfg(test)]
mod tests {
    use lid_rs::validates;
    use serde_json::json;

    use super::super::ending::WORKER_TOOLS;
    use super::super::replay::{self, Landed, Replay, Route, Seen, SessionScript, mentions, strings, user};
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
        let tools: Vec<(&str, &str)> = policy.tools.iter().map(|t| (t.requestee.as_str(), t.op.as_str())).collect();
        assert_eq!(tools, allows, "the declarations are the same five pairs");
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
