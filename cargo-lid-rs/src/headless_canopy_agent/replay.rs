//! An in-process door for the slice's validations
//! (`docs/intent/headless-canopy-agent/lld.md` § Decisions, "Validation
//! strategy"): an HTTP/1.1 responder on a loopback `TcpListener` that answers
//! the six requests from a script — sessions to open in order, each with the
//! tail pages it serves and the routes it refuses — and remembers every
//! request it saw and every record the client landed. The record shapes are
//! canopy's own: the policy record and the provider's terminal from a
//! session recorded on the production door, the invoke bodies and the tail
//! order from canopy's pinned goldens.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, PoisonError};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::door::{Door, Started};
use super::tools::{REQUESTEE, Tool};
use super::turn::Session;
use crate::phase::Phase;

/// The halt the replay lands when a session reads past its script: a loop
/// that never settles ends as a halt rather than a hang.
pub const EXHAUSTED: &str = "replay: the script is exhausted";

/// The producer of `app.policy.configured`, as the production door lands it.
pub const EXCHANGE: &str = "exchange";

/// The sentence a shed carries, in the `refused` field the door writes it in:
/// what the client reports when a request is shed out.
pub const SHED_SENTENCE: &str = "shed; retry later";

/// The producer of `inference.requested` and `app.invoke.payload`.
pub const REQUESTOR: &str = "requestor";

/// The producer of `app.invoke.forward` and `app.invoke.denied`.
pub const AUTHORIZER: &str = "authorizer";

/// The producer of `inference.responded`.
pub const DISPATCH: &str = "inference.dispatch";

/// The body of the `app.policy.configured` a production session began with.
pub const POLICY_CONFIGURED: &str = r#"{"allows":[{"op":"read","requestee":"lid-rs"}],"hash":"acaf3cfe931ea99adf32ba579720c99c2cbfa606045e04b775757e2f9722e2cd","principal_digest":"3364063f416d86985e983cf0c2d91e4c491493e5af012a13c6496c8c6921611a","tools":[{"name":"read","op":"read","requestee":"lid-rs","schema":{"description":"Read a file","properties":{"path":{"type":"string"}},"required":["path"],"type":"object"}}],"version":2}"#;

/// The `terminal` sentence of an `inference.responded` recorded on the
/// production door when the provider refused the request.
pub const TERMINAL_SENTENCE: &str = "the provider refused the request: 403 Forbidden {\"error\":{\"message\":\"This model requires you to complete the following before use: 18+ age confirmation. Confirm at https://openrouter.ai/settings/preferences.\",\"code\":403,\"metadata\":{\"missing_attestation_types\":[\"age_18plus\"]}},\"user_id\":\"user_3160oAsw7y3UejF5GafJbwujYEh\"}";

/// The `raw` of that same terminal record.
pub const TERMINAL_RAW: &str = r#"{"error":{"code":403,"message":"This model requires you to complete the following before use: 18+ age confirmation. Confirm at https://openrouter.ai/settings/preferences.","metadata":{"missing_attestation_types":["age_18plus"]}},"user_id":"user_3160oAsw7y3UejF5GafJbwujYEh"}"#;

/// The routes a door answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// `POST /sessions`.
    Start,
    /// `POST /sessions/{id}/events`.
    Send,
    /// `GET /sessions/{id}/tail`.
    Tail,
    /// `POST /sessions/{id}/refresh`.
    Refresh,
    /// `POST /sessions/{id}/stop`.
    Stop,
}

/// One request as the replay door received it.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    /// The method.
    pub method: String,
    /// The path with its query.
    pub path: String,
    /// The bearer presented, without `Bearer `.
    pub bearer: Option<String>,
    /// The `Idempotency-Key` header.
    pub idem: Option<String>,
    /// The body, when it was JSON.
    pub body: Option<Value>,
}

impl Seen {
    /// The route this request took, when it took one.
    pub fn route(&self) -> Option<Route> {
        route_of(&self.method, &self.path).map(|(route, _)| route)
    }

    /// One value of the query string.
    pub fn query(&self, name: &str) -> Option<String> {
        self.path.split_once('?').map(|(_, q)| q).unwrap_or("").split('&').filter_map(|kv| kv.split_once('=')).find(|(k, _)| *k == name).map(|(_, v)| v.to_string())
    }
}

/// One record the client landed in a session.
#[derive(Debug, Clone, PartialEq)]
pub struct Landed {
    /// The cursor the door assigned.
    pub cursor: u64,
    /// The record's kind.
    pub kind: String,
    /// Its body.
    pub body: Value,
    /// The idempotency key it landed under.
    pub idem: Option<String>,
}

/// A route the door sheds before it answers it: `times` answers of `429`,
/// each carrying `retry_after` as its `Retry-After` header when there is one,
/// and then the route's own answer. What a shed needs that a refusal cannot
/// give is an answer that is `429` and then stops being one.
#[derive(Debug, Clone)]
pub struct Shed {
    /// The route shed.
    pub route: Route,
    /// How many of its answers are `429`.
    pub times: u32,
    /// The `Retry-After` those answers carry, when they carry one.
    pub retry_after: Option<String>,
}

impl Shed {
    /// `times` sheds of `route`, each carrying `retry_after`.
    pub fn new(route: Route, times: u32, retry_after: Option<&str>) -> Self {
        Self { route, times, retry_after: retry_after.map(str::to_string) }
    }
}

/// What one session does when opened: the pages its tail serves, in order,
/// and the routes it refuses.
#[derive(Debug, Clone)]
pub struct SessionScript {
    /// The session's id.
    pub id: String,
    /// The tail's pages: each a list of records (`kind`, `producer`, `body`).
    pub pages: VecDeque<Vec<Value>>,
    /// The routes refused, with the status and sentence.
    pub refusals: Vec<(Route, u16, String)>,
    /// How long the credential the dial issues lives, in seconds.
    pub expires_in: u64,
    /// How long a credential a refresh issues lives; `expires_in` when the
    /// script names no other.
    pub refreshed_in: Option<u64>,
}

impl SessionScript {
    /// A session with no pages and no refusals, its credentials good for an hour.
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string(), pages: VecDeque::new(), refusals: Vec::new(), expires_in: 3600, refreshed_in: None }
    }

    /// One more page the tail serves.
    pub fn page(mut self, records: Vec<Value>) -> Self {
        self.pages.push_back(records);
        self
    }

    /// A route this session refuses.
    pub fn refusing(mut self, route: Route, status: u16, sentence: &str) -> Self {
        self.refusals.push((route, status, sentence.to_string()));
        self
    }

    /// Credentials that live this many seconds.
    pub fn expiring_in(mut self, seconds: u64) -> Self {
        self.expires_in = seconds;
        self
    }

    /// Credentials a refresh issues that live this many seconds, whatever the
    /// dial's lived: what a session whose first credential is spent needs to
    /// be asked for a second refresh, or not.
    pub fn refreshed_in(mut self, seconds: u64) -> Self {
        self.refreshed_in = Some(seconds);
        self
    }

    /// The scripted refusal of a route, if any.
    fn refusal(&self, route: Route) -> Option<(u16, Value)> {
        self.refusals.iter().find(|(r, _, _)| *r == route).map(|(_, status, sentence)| refused(*status, sentence))
    }
}

/// A session the replay has opened.
struct Live {
    /// Its script.
    script: SessionScript,
    /// Every token issued for it.
    tokens: Vec<String>,
    /// Its log: every record, scripted or landed, with its envelope.
    log: Vec<Value>,
    /// The records the client landed.
    landed: Vec<Landed>,
    /// Reads past the script's end.
    empty_reads: u32,
    /// Whether it has been stopped.
    stopped: bool,
}

/// What the replay knows.
#[derive(Default)]
struct State {
    /// The sessions still to open, in order.
    scripts: VecDeque<SessionScript>,
    /// The sessions opened.
    sessions: Vec<Live>,
    /// Every request, in order.
    seen: Vec<Seen>,
    /// The API key, as the first dial presented it.
    key: Option<String>,
    /// The sheds still to answer, by route.
    sheds: Vec<Shed>,
}

/// What the replay answers one request with: the status, the JSON body, and
/// the `Retry-After` a shed carries.
type Answer = (u16, Value, Option<String>);

/// The replay door: its URL and what it has seen.
pub struct Replay {
    /// `http://127.0.0.1:<port>`.
    pub url: String,
    /// The state the responder thread shares.
    state: Arc<Mutex<State>>,
}

impl Replay {
    /// Serves the scripts on a loopback port, shedding nothing.
    pub fn serve(scripts: Vec<SessionScript>) -> Self {
        Self::shedding(scripts, Vec::new())
    }

    /// The same, shedding each route the sheds name before answering it.
    pub fn shedding(scripts: Vec<SessionScript>, sheds: Vec<Shed>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let url = format!("http://{}", listener.local_addr().expect("the bound address"));
        let state = Arc::new(Mutex::new(State { scripts: scripts.into(), sheds, ..State::default() }));
        let shared = Arc::clone(&state);
        std::thread::spawn(move || serve_loop(&listener, &shared));
        Self { url, state }
    }

    /// A client over this door, presenting `key`.
    pub fn door(&self, key: &str) -> Door {
        Door::new(&self.url, key)
    }

    /// Every request seen, in order.
    pub fn seen(&self) -> Vec<Seen> {
        self.lock().seen.clone()
    }

    /// The ids of the sessions opened, in order.
    pub fn opened(&self) -> Vec<String> {
        self.lock().sessions.iter().map(|live| live.script.id.clone()).collect()
    }

    /// What the client landed in a session.
    pub fn landed(&self, session: &str) -> Vec<Landed> {
        self.lock().sessions.iter().find(|live| live.script.id == session).map(|live| live.landed.clone()).unwrap_or_default()
    }

    /// Whether a session has been stopped.
    pub fn stopped(&self, session: &str) -> bool {
        self.lock().sessions.iter().any(|live| live.script.id == session && live.stopped)
    }

    /// The state, poisoned or not.
    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The texts of the user messages among landed records, in order.
pub fn user_messages(landed: &[Landed]) -> Vec<String> {
    landed.iter().filter(|l| l.kind == "app.client.user_message").filter_map(|l| l.body["text"].as_str().map(str::to_string)).collect()
}

/// The completions among landed records, in order.
pub fn completions(landed: &[Landed]) -> Vec<Landed> {
    landed.iter().filter(|l| l.kind == "app.invoke.completed").cloned().collect()
}

/// Asserts a text mentions every needle.
pub fn mentions(text: &str, needles: &[&str]) {
    assert!(needles.iter().all(|needle| text.contains(needle)), "expected {needles:?} in: {text}");
}

/// Owned strings.
pub fn strings(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// The user message the client offers.
pub fn user(text: &str) -> super::door::Offered {
    super::door::Offered { kind: "app.client.user_message".to_string(), body: json!({ "text": text }) }
}

/// The hex SHA-256 of a text — canopy's canonical JSON written by hand.
pub fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Seconds since the epoch.
pub fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("after the epoch").as_secs()
}

/// A session over `door` without a dial, for the tools' tests: the
/// credential names `id`, so the tally key is `canopy:<id>`.
pub fn session(door: Door, id: &str, phase: Phase, tools: Vec<Tool>) -> Session {
    let credential = Started { session: id.to_string(), token: format!("tok-{id}"), expires: now() + 3600, stream: format!("t/{id}") };
    Session { door, credential, cursor: 0, held: Vec::new(), phase, tools }
}

// ---- record shapes -------------------------------------------------------------

/// One scripted record.
pub fn rec(kind: &str, producer: &str, body: Value) -> Value {
    json!({ "kind": kind, "producer": producer, "body": body })
}

/// The `app.policy.configured` the door lands first: the replay lands it on
/// every dial, as the production door does, so every session's log begins
/// with it at cursor 1 and the client's first user message lands at 2.
pub fn policy_configured() -> Value {
    rec("app.policy.configured", EXCHANGE, serde_json::from_str(POLICY_CONFIGURED).expect("the recorded policy"))
}

/// An `inference.requested`, informational.
pub fn requested(up_to: u64) -> Value {
    rec("inference.requested", REQUESTOR, json!({ "contextContainsUpTo": up_to, "params": null }))
}

/// An `inference.responded` with the model's text and no tool uses: the
/// gateway's canonical shape.
pub fn responded(text: &str) -> Value {
    rec(
        "inference.responded",
        DISPATCH,
        json!({
            "echo": { "for": "inference.dispatch:4:0:0" },
            "raw": {},
            "response": { "type": "canonical", "text": text, "tool_uses": [], "stop_reason": "end_turn" },
            "usage": { "input_tokens": 11, "output_tokens": 7 },
        }),
    )
}

/// An `inference.responded` asking for tools: one use per `(op, args)`.
pub fn responded_with_tools(text: &str, uses: &[(&str, Value)]) -> Value {
    let tool_uses: Vec<Value> = uses.iter().map(|(op, args)| json!({ "name": op, "requestee": REQUESTEE, "op": op, "projections": {}, "args": args })).collect();
    rec(
        "inference.responded",
        DISPATCH,
        json!({
            "echo": { "for": "inference.dispatch:4:0:0" },
            "raw": {},
            "response": { "type": "canonical", "text": text, "tool_uses": tool_uses, "stop_reason": "tool_calls" },
            "usage": { "input_tokens": 11, "output_tokens": 7 },
        }),
    )
}

/// An `inference.responded` whose turn failed at the provider, as recorded.
pub fn terminal(sentence: &str) -> Value {
    let raw: Value = serde_json::from_str(TERMINAL_RAW).expect("the recorded raw");
    rec("inference.responded", DISPATCH, json!({ "echo": { "for": "inference.dispatch:4:0:0" }, "raw": raw, "terminal": sentence }))
}

/// An `app.invoke.payload` addressed to `to`.
pub fn payload_to(to: &str, args: Value) -> Value {
    rec("app.invoke.payload", REQUESTOR, json!({ "to": to, "args": args }))
}

/// An `app.invoke.payload` addressed to this program.
pub fn payload(args: Value) -> Value {
    payload_to(REQUESTEE, args)
}

/// The `app.invoke.requested` that follows a payload, informational.
pub fn invoke_requested(op: &str, digest: &str) -> Value {
    rec("app.invoke.requested", REQUESTOR, json!({ "requestee": REQUESTEE, "op": op, "projections": {}, "payload_digest": digest }))
}

/// An `app.invoke.forward` addressed to `to`.
pub fn forward_to(to: &str, op: &str, digest: &str) -> Value {
    rec("app.invoke.forward", AUTHORIZER, json!({ "to": to, "op": op, "payload_digest": digest }))
}

/// An `app.invoke.forward` addressed to this program.
pub fn forward(op: &str, digest: &str) -> Value {
    forward_to(REQUESTEE, op, digest)
}

/// An `app.invoke.denied`: the authorizer refused a call.
pub fn denied(digest: &str) -> Value {
    rec("app.invoke.denied", AUTHORIZER, json!({ "to": REQUESTOR, "payload_digest": digest }))
}

/// A `trellis.attempted`, informational.
pub fn attempted() -> Value {
    rec("trellis.attempted", DISPATCH, json!({ "attempt": 0, "request": 4 }))
}

/// An `app.session.halted` with its reason.
pub fn halted(reason: &str) -> Value {
    rec("app.session.halted", "platform", json!({ "reason": reason }))
}

/// A page in the golden tail's order: the model asks for one tool, its
/// payload lands, the authorizer forwards it.
pub fn tool_call_page(op: &str, args: Value, digest: &str) -> Vec<Value> {
    vec![requested(3), responded_with_tools("Let me look.", &[(op, args.clone())]), payload(args), invoke_requested(op, digest), forward(op, digest), attempted()]
}

/// A page that settles the turn with the model's text.
pub fn settling_page(text: &str) -> Vec<Value> {
    vec![requested(9), responded(text)]
}

// ---- the responder -------------------------------------------------------------

/// Accepts connections until the listener is dropped, one thread each.
fn serve_loop(listener: &TcpListener, state: &Arc<Mutex<State>>) {
    for stream in listener.incoming().flatten() {
        let shared = Arc::clone(state);
        std::thread::spawn(move || handle(stream, &shared));
    }
}

/// One connection: one request, one answer — carrying `Retry-After` when it
/// is a shed — then closed.
fn handle(mut stream: TcpStream, state: &Mutex<State>) {
    let Some(request) = read_request(&mut stream) else { return };
    let (status, body, retry_after) = state.lock().unwrap_or_else(PoisonError::into_inner).answer(&request);
    let text = body.to_string();
    let retry = retry_after.map(|seconds| format!("Retry-After: {seconds}\r\n")).unwrap_or_default();
    let response =
        format!("HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\n{retry}Content-Length: {}\r\nConnection: close\r\n\r\n{text}", reason(status), text.len());
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// A reason phrase for the status line.
fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        429 => "Too Many Requests",
        _ => "Status",
    }
}

/// One HTTP/1.1 request, parsed.
struct Request {
    /// The method.
    method: String,
    /// The path with its query.
    path: String,
    /// The headers, names lowercased.
    headers: Vec<(String, String)>,
    /// The body.
    body: Vec<u8>,
}

impl Request {
    /// The request as the replay remembers it.
    fn seen(&self) -> Seen {
        Seen {
            method: self.method.clone(),
            path: self.path.clone(),
            bearer: header(&self.headers, "authorization").and_then(|v| v.strip_prefix("Bearer ")).map(str::to_string),
            idem: header(&self.headers, "idempotency-key").map(str::to_string),
            body: serde_json::from_slice(&self.body).ok(),
        }
    }
}

/// The request on a stream: its line, its headers, its body.
fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let (method, path) = request_line(&mut reader)?;
    let headers = header_lines(&mut reader)?;
    let body = body_bytes(&mut reader, &headers)?;
    Some(Request { method, path, headers, body })
}

/// The method and path of the request line.
fn request_line(reader: &mut impl BufRead) -> Option<(String, String)> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut words = line.split_whitespace();
    Some((words.next()?.to_string(), words.next()?.to_string()))
}

/// The headers up to the blank line, names lowercased.
fn header_lines(reader: &mut impl BufRead) -> Option<Vec<(String, String)>> {
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let Some((name, value)) = line.trim_end().split_once(':') else { return Some(headers) };
        headers.push((name.trim().to_ascii_lowercase(), value.trim().to_string()));
    }
}

/// One header's value.
fn header<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
}

/// The body, by `Content-Length` or chunked; empty without either.
fn body_bytes(reader: &mut impl BufRead, headers: &[(String, String)]) -> Option<Vec<u8>> {
    let chunked_encoding = header(headers, "transfer-encoding").is_some_and(|v| v.contains("chunked"));
    match header(headers, "content-length").and_then(|v| v.parse::<usize>().ok()) {
        Some(len) => sized(reader, len),
        None if chunked_encoding => chunked(reader),
        None => Some(Vec::new()),
    }
}

/// Exactly `len` bytes.
fn sized(reader: &mut impl BufRead, len: usize) -> Option<Vec<u8>> {
    let mut body = vec![0; len];
    reader.read_exact(&mut body).ok()?;
    Some(body)
}

/// A chunked body, whole.
fn chunked(reader: &mut impl BufRead) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let size = chunk_size(reader)?;
        if size == 0 {
            return Some(body);
        }
        body.extend(sized(reader, size)?);
        sized(reader, 2)?;
    }
}

/// One chunk's size line.
fn chunk_size(reader: &mut impl BufRead) -> Option<usize> {
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    usize::from_str_radix(line.trim().split(';').next()?, 16).ok()
}

/// The route a method and path name, with the session id for the
/// session routes.
fn route_of(method: &str, path: &str) -> Option<(Route, String)> {
    let rest = path.strip_prefix("/sessions")?;
    if rest.is_empty() || rest.starts_with('?') {
        return (method == "POST").then(|| (Route::Start, String::new()));
    }
    let (id, op) = rest.strip_prefix('/')?.split_once('/')?;
    Some((op_route(method, op.split('?').next()?)?, id.to_string()))
}

/// The session route an operation names.
fn op_route(method: &str, op: &str) -> Option<Route> {
    match (method, op) {
        ("POST", "events") => Some(Route::Send),
        ("GET", "tail") => Some(Route::Tail),
        ("POST", "refresh") => Some(Route::Refresh),
        ("POST", "stop") => Some(Route::Stop),
        (_, _) => None,
    }
}

/// A refusal as the door writes it.
fn refused(status: u16, sentence: &str) -> (u16, Value) {
    (status, json!({ "refused": sentence }))
}

/// An answer carrying no `Retry-After`, which every answer but a shed is.
fn plain(answered: (u16, Value)) -> Answer {
    (answered.0, answered.1, None)
}

impl State {
    /// The answer to one request, remembered: a scripted shed of its route
    /// while that route has one left, else the route's own answer.
    fn answer(&mut self, request: &Request) -> Answer {
        let seen = request.seen();
        self.seen.push(seen.clone());
        let shed = self.shed(seen.route());
        match shed {
            Some(answer) => answer,
            None => plain(self.routed(&seen)),
        }
    }

    /// One scripted shed of `route`, spent: `429` with the script's
    /// `Retry-After`, or none when the route has no shed left to answer.
    fn shed(&mut self, route: Option<Route>) -> Option<Answer> {
        let shed = self.sheds.iter_mut().find(|shed| Some(shed.route) == route && shed.times > 0)?;
        shed.times -= 1;
        let retry_after = shed.retry_after.clone();
        let (status, body) = refused(429, SHED_SENTENCE);
        Some((status, body, retry_after))
    }

    /// The answer the route itself gives.
    fn routed(&mut self, seen: &Seen) -> (u16, Value) {
        match route_of(&seen.method, &seen.path) {
            None => refused(404, "no such route"),
            Some((Route::Start, _)) => self.start(seen),
            Some((route, id)) => self.on_session(route, &id, seen),
        }
    }

    /// A dial: the next scripted session opens, unless it refuses, with the
    /// door's own `app.policy.configured` as its first record — cursor 1,
    /// producer `exchange` — as the production door lands it.
    fn start(&mut self, seen: &Seen) -> (u16, Value) {
        let Some(script) = self.scripts.pop_front() else { return refused(500, "the replay has no session scripted for this dial") };
        self.key = seen.bearer.clone();
        if let Some(refusal) = script.refusal(Route::Start) {
            return refusal;
        }
        let mut live = Live { script, tokens: Vec::new(), log: Vec::new(), landed: Vec::new(), empty_reads: 0, stopped: false };
        let started = live.issue();
        live.append_scripted(&policy_configured());
        self.sessions.push(live);
        (200, started)
    }

    /// A session route, on the session it names.
    fn on_session(&mut self, route: Route, id: &str, seen: &Seen) -> (u16, Value) {
        let key = self.key.clone();
        let Some(live) = self.sessions.iter_mut().find(|live| live.script.id == id) else { return refused(404, "no such session") };
        if let Some(refusal) = live.script.refusal(route) {
            return refusal;
        }
        match route {
            Route::Start => refused(404, "no such route"),
            Route::Send => live.send(seen),
            Route::Tail => live.tail(seen),
            Route::Refresh => live.refresh(seen, key.as_deref()),
            Route::Stop => live.stop(seen),
        }
    }
}

impl Live {
    /// A fresh credential for this session, living [`Live::life`] seconds.
    fn issue(&mut self) -> Value {
        let token = format!("tok-{}-{}", self.script.id, self.tokens.len() + 1);
        let expires = now() + self.life();
        self.tokens.push(token.clone());
        json!({ "session": self.script.id, "token": token, "expires": expires, "stream": format!("t/{}", self.script.id), "audience": "replay" })
    }

    /// How long the next credential lives: the script's `expires_in` for the
    /// one the dial issues, and its `refreshed_in` — that same lifetime when
    /// the script names no other — for every one a refresh issues.
    fn life(&self) -> u64 {
        match self.tokens.is_empty() {
            true => self.script.expires_in,
            false => self.script.refreshed_in.unwrap_or(self.script.expires_in),
        }
    }

    /// The refusal for a bearer that is not one of this session's tokens.
    fn authorised(&self, seen: &Seen) -> Option<(u16, Value)> {
        match &seen.bearer {
            Some(bearer) if self.tokens.contains(bearer) => None,
            Some(_) => Some(refused(403, "the API key opens a session; its token drives it")),
            None => Some(refused(401, "no credential")),
        }
    }

    /// The refusal for a token route: the bearer, then a stopped session.
    fn gate(&self, seen: &Seen) -> Option<(u16, Value)> {
        self.authorised(seen).or_else(|| self.stopped.then(|| refused(409, "the session has stopped")))
    }

    /// One record onto the log, with its envelope; its cursor.
    fn append(&mut self, kind: &str, producer: &str, body: Value, idem: Option<String>) -> u64 {
        let cursor = self.log.len() as u64 + 1;
        let size = body.to_string().len();
        let envelope = json!({ "stream": format!("t/{}", self.script.id), "version": 1, "idem": idem, "size": size });
        self.log.push(json!({ "cursor": cursor, "kind": kind, "producer": producer, "body": body, "envelope": envelope }));
        cursor
    }

    /// What the client offers, landed.
    fn send(&mut self, seen: &Seen) -> (u16, Value) {
        if let Some(refusal) = self.gate(seen) {
            return refusal;
        }
        let offered = seen.body.clone().unwrap_or(Value::Null);
        let kind = offered["kind"].as_str().unwrap_or("").to_string();
        let cursor = self.append(&kind, "client", offered["body"].clone(), seen.idem.clone());
        self.landed.push(Landed { cursor, kind, body: offered["body"].clone(), idem: seen.idem.clone() });
        (200, json!({ "cursor": cursor }))
    }

    /// The records after `after`; the next scripted page lands when the log
    /// has none.
    fn tail(&mut self, seen: &Seen) -> (u16, Value) {
        if let Some(refusal) = self.gate(seen) {
            return refusal;
        }
        let after = seen.query("after").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
        if self.log.len() <= after {
            self.next_page();
        }
        let records: Vec<Value> = self.log.iter().skip(after).cloned().collect();
        (200, json!({ "records": records, "through": self.log.len() }))
    }

    /// The next scripted page onto the log, or the script's end noted.
    fn next_page(&mut self) {
        match self.script.pages.pop_front() {
            Some(page) => page.into_iter().for_each(|record| self.append_scripted(&record)),
            None => self.exhausted(),
        }
    }

    /// A scripted record onto the log.
    fn append_scripted(&mut self, record: &Value) {
        let kind = record["kind"].as_str().unwrap_or("").to_string();
        let producer = record["producer"].as_str().unwrap_or("").to_string();
        self.append(&kind, &producer, record["body"].clone(), None);
    }

    /// One read past the script; the third lands the exhausted halt.
    fn exhausted(&mut self) {
        self.empty_reads += 1;
        if self.empty_reads > 2 {
            self.append("app.session.halted", "platform", json!({ "reason": EXHAUSTED }), None);
        }
    }

    /// A refresh presents the API key; a fresh credential.
    fn refresh(&mut self, seen: &Seen, key: Option<&str>) -> (u16, Value) {
        if key.is_none() || seen.bearer.as_deref() != key {
            return refused(401, "a refresh presents the API key");
        }
        (200, self.issue())
    }

    /// The session stopped; its log sealed.
    fn stop(&mut self, seen: &Seen) -> (u16, Value) {
        if let Some(refusal) = self.gate(seen) {
            return refusal;
        }
        self.stopped = true;
        (200, json!({ "cursor": self.log.len() }))
    }
}
