// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Session hub for real-time collaboration: the authoritative sequencer behind the studio server (R-111).
//!
//! Pure and deterministic like `collab` (no sockets, no clock: the caller passes milliseconds). A
//! `Hub` holds
//! * `main`: the server's replica; every accepted envelope is applied to it in one total order, and
//!   that order (the log, numbered from 0 for the whole session) is what clients see as `index`;
//! * one `User` per participant. A *thin* user (a browser page) has no replica of its own: the hub keeps
//!   a **proxy replica** for it and translates the user's pointer edits (`Op` with array indices, made
//!   against the view the user had at log position `base`) into identity-based envelopes in that
//!   view, then merges them into `main`. This is what makes a stale index land on the right element.
//!   A *replica* user (another process that runs `collab::Replica` itself) sends finished envelopes.
//! * presence (`PresenceBoard`) and an outbox of `Event`s for the transport (server-sent events).
//!
//! Every accepted change is first validated by a caller-supplied closure on the merged JSON (the studio
//! passes one that compiles the project), so a merge that would produce an invalid project is rejected
//! as a whole and nothing changes.

use crate::collab::{colour_for, Envelope, Presence, PresenceBoard, Replica, ReplicaId, Snapshot, VClock};
use crate::edit::Op;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub type Validate<'a> = &'a mut dyn FnMut(&Value) -> Result<(), Vec<String>>;

const MAIN_ID: ReplicaId = 1;
/// The studio host's own editing (and any request that names no user) is attributed to this user.
pub const LOCAL_ID: ReplicaId = 2;
const MAX_USERS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// The hub keeps a proxy replica; the user sends pointer ops.
    Thin,
    /// The user runs its own replica and sends envelopes.
    Replica,
}

struct User {
    name: String,
    colour: String,
    mode: Mode,
    proxy: Option<Replica>,
    /// Log position (absolute) up to which the proxy has been brought forward.
    seen: usize,
    last_ms: u64,
}

#[derive(Clone, Debug)]
pub enum Event {
    Ops { index: usize, by: ReplicaId, envelope: Envelope },
    Presence(Presence),
    Users(Vec<Value>),
    Left(ReplicaId),
}

impl Event {
    /// One server-sent-events frame.
    pub fn to_sse(&self) -> String {
        let (name, id, data) = match self {
            Event::Ops { index, by, envelope } => ("ops", Some(*index + 1), json!({"index": index + 1, "by": by, "envelope": envelope})),
            Event::Presence(p) => ("presence", None, serde_json::to_value(p).unwrap_or(Value::Null)),
            Event::Users(u) => ("users", None, json!({"users": u})),
            Event::Left(r) => ("left", None, json!({"replica": r})),
        };
        match id {
            Some(i) => format!("id: {i}\nevent: {name}\ndata: {data}\n\n"),
            None => format!("event: {name}\ndata: {data}\n\n"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct JoinInfo {
    pub replica: ReplicaId,
    pub colour: String,
    pub name: String,
    pub index: usize,
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub struct Submitted {
    /// Log length after the change (clients use it as their next `base`).
    pub index: usize,
    /// The envelope that was merged, `None` when the edit changed nothing.
    pub envelope: Option<Envelope>,
}

#[derive(Clone, Debug)]
pub struct SubmitError {
    pub errors: Vec<String>,
    /// The caller's view is older than the compacted log: it must reload the state.
    pub resync: bool,
}

fn fail(e: impl Into<String>) -> SubmitError {
    SubmitError { errors: vec![e.into()], resync: false }
}

pub struct Hub {
    main: Replica,
    log_base: usize,
    users: BTreeMap<ReplicaId, User>,
    next_id: ReplicaId,
    pub presence: PresenceBoard,
    outbox: Vec<Event>,
    /// Users idle longer than this are dropped by `expire`.
    pub idle_ms: u64,
}

impl Hub {
    pub fn new(doc: &Value) -> Hub {
        let main = Replica::new(MAIN_ID, doc).expect("replica id 1 is valid");
        let mut h = Hub { main, log_base: 0, users: BTreeMap::new(), next_id: LOCAL_ID + 1, presence: PresenceBoard::new(15_000), outbox: vec![], idle_ms: 10 * 60_000 };
        let proxy = h.main.fork_as(LOCAL_ID).ok();
        h.users.insert(LOCAL_ID, User { name: "local".into(), colour: colour_for(LOCAL_ID), mode: Mode::Thin, proxy, seen: 0, last_ms: 0 });
        h
    }

    /// Number of envelopes accepted so far (positions `0..index` of the log).
    pub fn index(&self) -> usize {
        self.log_base + self.main.log_entries().len()
    }
    pub fn doc_json(&self) -> Value {
        self.main.json()
    }
    /// The id the next participant will get (persisted so a resumed session never reuses an id whose
    /// envelopes are already in the log).
    pub fn next_id(&self) -> ReplicaId {
        self.next_id
    }
    /// Rebuild the server state from a checkpoint (`snapshot` taken when the log had `log_base` entries).
    /// Participants are not part of the state: they join again and get fresh ids above `next_id`.
    pub fn restore(snapshot: Snapshot, log_base: usize, next_id: ReplicaId) -> Result<Hub, String> {
        if next_id <= LOCAL_ID {
            return Err(format!("next participant id {next_id} is not above the reserved ids"));
        }
        let main = Replica::from_snapshot(MAIN_ID, snapshot)?;
        let mut h = Hub { main, log_base, users: BTreeMap::new(), next_id, presence: PresenceBoard::new(15_000), outbox: vec![], idle_ms: 10 * 60_000 };
        let proxy = h.main.fork_as(LOCAL_ID).ok();
        h.users.insert(LOCAL_ID, User { name: "local".into(), colour: colour_for(LOCAL_ID), mode: Mode::Thin, proxy, seen: log_base, last_ms: 0 });
        Ok(h)
    }
    /// Call once after the last `replay`: the host's proxy is rebuilt from the final state (it cannot replay
    /// its own earlier envelopes, which it would take for forged ones).
    pub fn finish_restore(&mut self) {
        let end = self.index();
        let proxy = self.main.fork_as(LOCAL_ID).ok();
        if let Some(u) = self.users.get_mut(&LOCAL_ID) {
            u.proxy = proxy;
            u.seen = end;
        }
    }
    /// Re-apply one logged envelope at absolute position `index` while resuming (no events, no validation
    /// beyond causal delivery: it was validated when it was first accepted).
    pub fn replay(&mut self, index: usize, env: Envelope) -> Result<(), String> {
        if index != self.index() {
            return Err(format!("log entry {index} found where entry {} was expected", self.index()));
        }
        if env.replica >= self.next_id {
            self.next_id = env.replica + 1;
        }
        let label = format!("{}:{}", env.replica, env.seq);
        self.main.receive_checked(env, &mut |_| Ok(())).map_err(|e| format!("envelope {label} does not fit the state before it: {}", e.join("; ")))
    }
    pub fn clock(&self) -> &VClock {
        self.main.clock()
    }
    pub fn lamport(&self) -> u64 {
        self.main.lamport()
    }
    pub fn snapshot(&self) -> Snapshot {
        self.main.snapshot()
    }
    pub fn tombstones(&self) -> usize {
        self.main.tombstones()
    }
    pub fn log_base(&self) -> usize {
        self.log_base
    }
    pub fn has_user(&self, id: ReplicaId) -> bool {
        self.users.contains_key(&id)
    }
    /// Participants other than the implicit host user.
    pub fn joined_count(&self) -> usize {
        self.users.keys().filter(|k| **k != LOCAL_ID).count()
    }
    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.outbox)
    }

    pub fn users_view(&self) -> Vec<Value> {
        self.users
            .iter()
            .filter(|(id, _)| **id != LOCAL_ID)
            .map(|(id, u)| json!({"replica": id, "name": u.name, "colour": u.colour, "mode": u.mode}))
            .collect()
    }

    fn announce(&mut self) {
        let u = self.users_view();
        self.outbox.push(Event::Users(u));
    }

    pub fn join(&mut self, name: &str, mode: Mode, now_ms: u64) -> Result<JoinInfo, String> {
        if self.users.len() >= MAX_USERS {
            return Err(format!("session is full ({MAX_USERS} participants)"));
        }
        let id = self.next_id;
        self.next_id += 1;
        let p = Presence::new(id, name);
        let proxy = if mode == Mode::Thin { Some(self.main.fork_as(id)?) } else { None };
        let info = JoinInfo { replica: id, colour: p.colour.clone(), name: p.name.clone(), index: self.index(), mode };
        self.users.insert(id, User { name: p.name.clone(), colour: p.colour.clone(), mode, proxy, seen: self.index(), last_ms: now_ms });
        self.presence.update(p, now_ms);
        self.announce();
        Ok(info)
    }

    pub fn leave(&mut self, id: ReplicaId) -> bool {
        if id == LOCAL_ID || self.users.remove(&id).is_none() {
            return false;
        }
        self.presence.remove(id);
        self.outbox.push(Event::Left(id));
        self.announce();
        true
    }

    /// Mark a user as alive (any request from it).
    pub fn touch(&mut self, id: ReplicaId, now_ms: u64) {
        if let Some(u) = self.users.get_mut(&id) {
            u.last_ms = now_ms;
        }
        self.presence.touch(id, now_ms);
    }

    /// Drop presence that was not refreshed (TTL) and users idle too long. Returns the ids removed.
    pub fn expire(&mut self, now_ms: u64) -> Vec<ReplicaId> {
        let mut gone = Vec::new();
        for id in self.presence.expire(now_ms) {
            self.outbox.push(Event::Left(id));
            gone.push(id);
        }
        let idle: Vec<ReplicaId> = self.users.iter().filter(|(id, u)| **id != LOCAL_ID && now_ms.saturating_sub(u.last_ms) > self.idle_ms).map(|(id, _)| *id).collect();
        for id in idle {
            self.users.remove(&id);
            if !gone.contains(&id) {
                self.outbox.push(Event::Left(id));
                gone.push(id);
            }
        }
        if !gone.is_empty() {
            self.announce();
        }
        gone
    }

    pub fn set_presence(&mut self, id: ReplicaId, mut p: Presence, now_ms: u64) -> Result<(), String> {
        let u = self.users.get_mut(&id).ok_or_else(|| format!("unknown participant {id}: join first"))?;
        u.last_ms = now_ms;
        p.replica = id;
        p.name = u.name.clone();
        p.colour = u.colour.clone();
        if self.presence.update(p, now_ms) {
            if let Some(q) = self.presence.get(id) {
                self.outbox.push(Event::Presence(q.clone()));
            }
        }
        Ok(())
    }

    pub fn can_undo(&self, id: ReplicaId) -> bool {
        self.users.get(&id).and_then(|u| u.proxy.as_ref()).map(|p| p.can_undo()).unwrap_or(false)
    }
    pub fn can_redo(&self, id: ReplicaId) -> bool {
        self.users.get(&id).and_then(|u| u.proxy.as_ref()).map(|p| p.can_redo()).unwrap_or(false)
    }

    /// Bring a user's proxy forward to log position `to` (absolute).
    fn advance(&mut self, id: ReplicaId, to: usize) -> Result<(), SubmitError> {
        let end = self.index();
        if to > end {
            return Err(fail(format!("base {to} is ahead of the log ({end})")));
        }
        let base = self.log_base;
        if to < base {
            return Err(SubmitError { errors: vec!["the history this view was based on was compacted; reload the state".into()], resync: true });
        }
        let u = self.users.get_mut(&id).ok_or_else(|| fail(format!("unknown participant {id}: join first")))?;
        let proxy = u.proxy.as_mut().ok_or_else(|| fail("this participant sends envelopes, not edits"))?;
        if to > u.seen {
            let from = u.seen.max(base);
            let entries = &self.main.log_entries()[from - base..to - base];
            for e in entries {
                if proxy.receive(e.clone()).is_err() {
                    return Err(SubmitError { errors: vec!["internal: a proxy could not follow the log; reload the state".into()], resync: true });
                }
            }
            u.seen = to;
        }
        Ok(())
    }

    /// Merge an envelope produced by a user's proxy into `main`.
    fn commit(&mut self, by: ReplicaId, env: Envelope, validate: Validate) -> Result<Submitted, Vec<String>> {
        self.main.receive_checked(env.clone(), validate)?;
        let index = self.index();
        self.outbox.push(Event::Ops { index: index - 1, by, envelope: env.clone() });
        Ok(Submitted { index, envelope: Some(env) })
    }

    fn with_proxy<F>(&mut self, id: ReplicaId, base: Option<usize>, validate: Validate, f: F) -> Result<Submitted, SubmitError>
    where
        F: FnOnce(&mut Replica) -> Result<Option<Envelope>, String>,
    {
        let to = base.unwrap_or_else(|| self.index());
        self.advance(id, to)?;
        // Edits are made in the view at `base` (plus the user's own earlier edits, which the proxy has).
        let backup = self.users.get(&id).and_then(|u| u.proxy.clone());
        let proxy = self.users.get_mut(&id).and_then(|u| u.proxy.as_mut()).ok_or_else(|| fail("no proxy"))?;
        let made = match f(proxy) {
            Ok(m) => m,
            Err(e) => {
                if let (Some(u), Some(b)) = (self.users.get_mut(&id), backup) {
                    u.proxy = Some(b);
                }
                return Err(fail(e));
            }
        };
        let Some(env) = made else { return Ok(Submitted { index: self.index(), envelope: None }) };
        match self.commit(id, env, validate) {
            Ok(s) => Ok(s),
            Err(errors) => {
                if let (Some(u), Some(b)) = (self.users.get_mut(&id), backup) {
                    u.proxy = Some(b);
                }
                Err(SubmitError { errors, resync: false })
            }
        }
    }

    /// A pointer edit made in the user's view at log position `base` (`None` = the current end).
    pub fn submit_ops(&mut self, id: ReplicaId, base: Option<usize>, ops: &[Op], validate: Validate, now_ms: u64) -> Result<Submitted, SubmitError> {
        self.touch(id, now_ms);
        let op = if ops.len() == 1 { ops[0].clone() } else { Op::Batch { ops: ops.to_vec() } };
        self.with_proxy(id, base, validate, |p| p.local_op(&op))
    }

    /// The user's new complete document (for edits that were computed server-side on the editor).
    pub fn submit_doc(&mut self, id: ReplicaId, doc: &Value, validate: Validate, now_ms: u64) -> Result<Submitted, SubmitError> {
        self.touch(id, now_ms);
        let end = self.index();
        self.with_proxy(id, Some(end), validate, |p| p.local_doc(doc))
    }

    /// Undo or redo the user's own last edit against the CURRENT state.
    pub fn undo_redo(&mut self, id: ReplicaId, undo: bool, validate: Validate, now_ms: u64) -> Result<Submitted, SubmitError> {
        self.touch(id, now_ms);
        let end = self.index();
        self.with_proxy(id, Some(end), validate, |p| if undo { p.undo() } else { p.redo() })
    }

    /// Envelopes from a participant that runs its own replica. They must arrive in causal order (a
    /// missing predecessor is an error naming the server clock, so the sender can resend from there).
    pub fn submit_envelopes(&mut self, id: ReplicaId, envs: Vec<Envelope>, validate: Validate, now_ms: u64) -> Result<Submitted, SubmitError> {
        self.touch(id, now_ms);
        match self.users.get(&id) {
            Some(u) if u.mode == Mode::Replica => {}
            Some(_) => return Err(fail("this participant is a thin client: send ops, not envelopes")),
            None => return Err(fail(format!("unknown participant {id}: join first"))),
        }
        let mut last = None;
        for e in envs {
            if e.replica != id {
                return Err(fail(format!("envelope from replica {} sent by participant {id}", e.replica)));
            }
            if e.seq <= self.main.clock().get(&id).copied().unwrap_or(0) {
                continue; // duplicate: idempotent
            }
            let by = id;
            match self.commit(by, e, validate) {
                Ok(s) => last = Some(s),
                Err(errors) => {
                    let have = self.main.clock().get(&id).copied().unwrap_or(0);
                    let mut errors = errors;
                    errors.push(format!("server has {have} envelopes from replica {id}; resend from {}", have + 1));
                    return Err(SubmitError { errors, resync: false });
                }
            }
        }
        Ok(last.unwrap_or(Submitted { index: self.index(), envelope: None }))
    }

    /// Envelopes at log positions `since..` for a replica-mode participant polling for changes.
    pub fn log_since(&self, since: usize) -> Result<Vec<(usize, Envelope)>, String> {
        if since < self.log_base {
            return Err(format!("log starts at {}: fetch the state instead", self.log_base));
        }
        let skip = (since - self.log_base).min(self.main.log_entries().len());
        Ok(self.main.log_entries()[skip..].iter().enumerate().map(|(i, e)| (since + i, e.clone())).collect())
    }

    /// Events a (re)connecting event stream missed: operations from position `since`, current users and presence.
    pub fn catch_up(&self, since: usize) -> Result<Vec<Event>, String> {
        let mut out = Vec::new();
        for (i, e) in self.log_since(since)? {
            out.push(Event::Ops { index: i, by: e.replica, envelope: e });
        }
        out.push(Event::Users(self.users_view()));
        for p in self.presence.all() {
            out.push(Event::Presence(p));
        }
        Ok(out)
    }

    /// Drop history every participant has, and (only if nobody lags and nobody runs their own replica)
    /// purge tombstones and unreachable nodes in the server and in all proxies.
    pub fn compact(&mut self) -> Value {
        let end = self.index();
        let all_thin = self.users.values().all(|u| u.mode == Mode::Thin);
        for id in self.users.keys().copied().collect::<Vec<_>>() {
            let _ = self.advance(id, end);
        }
        let mut clocks: Vec<VClock> = self.users.values().filter_map(|u| u.proxy.as_ref().map(|p| p.clock().clone())).collect();
        if !all_thin {
            // a participant with its own replica may lag: do not purge, only keep the log it might need
            clocks.push(VClock::new());
        }
        let st = self.main.compact(&clocks);
        self.log_base += st.log_dropped;
        if st.purged {
            let main_clock = self.main.clock().clone();
            for u in self.users.values_mut() {
                if let Some(p) = u.proxy.as_mut() {
                    p.compact(std::slice::from_ref(&main_clock));
                }
            }
        }
        json!({"log_dropped": st.log_dropped, "tombstones_purged": st.tombstones_purged, "nodes_collected": st.nodes_collected, "purged": st.purged, "log_base": self.log_base, "index": self.index()})
    }

    pub fn status(&self, id: Option<ReplicaId>) -> Value {
        json!({
            "index": self.index(), "log_base": self.log_base, "clock": self.main.clock(), "lamport": self.main.lamport(),
            "users": self.users_view(), "presence": self.presence.all(), "tombstones": self.main.tombstones(), "nodes": self.main.node_count(),
            "can_undo": id.map(|i| self.can_undo(i)).unwrap_or(false), "can_redo": id.map(|i| self.can_redo(i)).unwrap_or(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collab::{Delivery, Replica};

    fn base() -> Value {
        json!({"frames": 24, "layers": [{"name": "a", "visible": true}, {"name": "b", "visible": true}, {"name": "c", "visible": true}], "meta": {"k": 1}})
    }
    fn ok() -> impl FnMut(&Value) -> Result<(), Vec<String>> {
        |_| Ok(())
    }
    fn set(p: &str, v: Value) -> Op {
        Op::Set { path: p.into(), value: v }
    }

    #[test]
    fn stale_thin_edit_lands_on_the_element_the_user_saw() {
        let mut h = Hub::new(&base());
        let a = h.join("ana", Mode::Thin, 0).unwrap();
        let b = h.join("bo", Mode::Thin, 0).unwrap();
        assert_eq!((a.replica, b.replica), (3, 4));
        assert_eq!(a.index, 0);
        // ana inserts at the front
        let s = h.submit_ops(a.replica, Some(0), &[Op::Insert { path: "/layers/0".into(), value: json!({"name": "front"}) }], &mut ok(), 1).unwrap();
        assert_eq!(s.index, 1);
        // bo still sees the old list (base 0): "/layers/1" is layer "b"
        let s = h.submit_ops(b.replica, Some(0), &[set("/layers/1/visible", json!(false))], &mut ok(), 2).unwrap();
        assert_eq!(s.index, 2);
        let j = h.doc_json();
        assert_eq!(j["layers"][0]["name"], "front");
        assert_eq!(j["layers"][2]["name"], "b");
        assert_eq!(j["layers"][2]["visible"], json!(false));
        assert_eq!(j["layers"][1]["visible"], json!(true));
        // after catching up, bo's next edit uses the new indices
        let s = h.submit_ops(b.replica, Some(2), &[set("/layers/3/visible", json!(false))], &mut ok(), 3).unwrap();
        assert_eq!(s.index, 3);
        assert_eq!(h.doc_json()["layers"][3]["name"], "c");
        assert_eq!(h.doc_json()["layers"][3]["visible"], json!(false));
    }

    #[test]
    fn rejected_edits_change_nothing_and_a_validator_veto_rolls_the_proxy_back() {
        let mut h = Hub::new(&base());
        let a = h.join("ana", Mode::Thin, 0).unwrap().replica;
        let before = h.doc_json();
        assert!(h.submit_ops(a, None, &[Op::Remove { path: "/layers/9".into() }], &mut ok(), 1).is_err());
        let mut veto = |_: &Value| -> Result<(), Vec<String>> { Err(vec!["nope".into()]) };
        let e = h.submit_ops(a, None, &[set("/frames", json!(5))], &mut veto, 2).unwrap_err();
        assert_eq!(e.errors, vec!["nope".to_string()]);
        assert_eq!((h.doc_json(), h.index()), (before.clone(), 0));
        // the proxy was restored: the same edit now works and has sequence number 1
        let s = h.submit_ops(a, None, &[set("/frames", json!(5))], &mut ok(), 3).unwrap();
        assert_eq!(s.envelope.unwrap().seq, 1);
        assert!(h.submit_ops(99, None, &[set("/frames", json!(5))], &mut ok(), 3).is_err());
        assert!(h.submit_ops(a, Some(50), &[set("/frames", json!(5))], &mut ok(), 3).is_err());
        let none = h.submit_ops(a, None, &[set("/frames", json!(5))], &mut ok(), 4).unwrap();
        assert!(none.envelope.is_none(), "no-op edit");
        assert_eq!(h.index(), 1);
    }

    #[test]
    fn per_user_undo_through_the_hub() {
        let mut h = Hub::new(&base());
        let a = h.join("ana", Mode::Thin, 0).unwrap().replica;
        let b = h.join("bo", Mode::Thin, 0).unwrap().replica;
        h.submit_ops(a, None, &[set("/layers/0/visible", json!(false))], &mut ok(), 1).unwrap();
        h.submit_ops(b, None, &[set("/layers/1/visible", json!(false))], &mut ok(), 2).unwrap();
        assert!(h.can_undo(a) && h.can_undo(b) && !h.can_redo(a));
        h.undo_redo(a, true, &mut ok(), 3).unwrap();
        let j = h.doc_json();
        assert_eq!(j["layers"][0]["visible"], json!(true));
        assert_eq!(j["layers"][1]["visible"], json!(false), "bo's edit untouched");
        assert!(h.can_redo(a) && !h.can_undo(a));
        h.undo_redo(a, false, &mut ok(), 4).unwrap();
        assert_eq!(h.doc_json()["layers"][0]["visible"], json!(false));
        let nothing = h.undo_redo(a, false, &mut ok(), 5).unwrap();
        assert!(nothing.envelope.is_none());
    }

    #[test]
    fn whole_document_edits_from_the_host_editor_are_diffed() {
        let mut h = Hub::new(&base());
        let mut d = base();
        d["layers"][2]["visible"] = json!(false);
        d["frames"] = json!(30);
        let s = h.submit_doc(LOCAL_ID, &d, &mut ok(), 1).unwrap();
        assert_eq!(s.envelope.unwrap().ops.len(), 2);
        assert_eq!(h.doc_json(), d);
        assert!(h.submit_doc(LOCAL_ID, &d, &mut ok(), 2).unwrap().envelope.is_none());
    }

    #[test]
    fn replica_participants_send_envelopes_and_events_describe_everything() {
        let mut h = Hub::new(&base());
        let info = h.join("rep", Mode::Replica, 0).unwrap();
        let mut me = Replica::from_snapshot(info.replica, h.snapshot()).unwrap();
        let e1 = me.local_op(&set("/frames", json!(40))).unwrap().unwrap();
        let e2 = me.local_op(&set("/meta/k", json!(2))).unwrap().unwrap();
        // out-of-order delivery to the hub is refused with a hint, in order is fine, resending is harmless
        let err = h.submit_envelopes(info.replica, vec![e2.clone()], &mut ok(), 1).unwrap_err();
        assert!(err.errors.last().unwrap().contains("resend from 1"));
        h.submit_envelopes(info.replica, vec![e1.clone(), e2.clone()], &mut ok(), 2).unwrap();
        h.submit_envelopes(info.replica, vec![e1], &mut ok(), 3).unwrap();
        assert_eq!(h.index(), 2);
        assert_eq!(h.doc_json(), me.json());
        assert!(h.submit_envelopes(info.replica, vec![Envelope { replica: 9, ..e2.clone() }], &mut ok(), 3).is_err());
        // a thin participant meanwhile; the replica participant pulls what it lacks and applies it
        let t = h.join("thin", Mode::Thin, 4).unwrap().replica;
        h.submit_ops(t, None, &[set("/layers/0/name", json!("zz"))], &mut ok(), 5).unwrap();
        for (_, e) in h.log_since(2).unwrap() {
            assert!(matches!(me.receive(e).unwrap(), Delivery::Applied(_)));
        }
        assert_eq!(h.doc_json(), me.json());
        assert_eq!(h.snapshot().doc, *me.doc());
        assert!(h.submit_envelopes(t, vec![], &mut ok(), 6).is_err(), "thin participants do not send envelopes");
        assert!(h.submit_ops(info.replica, None, &[set("/frames", json!(1))], &mut ok(), 6).is_err());
        let ev = h.take_events();
        assert!(ev.iter().filter(|e| matches!(e, Event::Ops { .. })).count() == 3);
        let sse = ev.iter().find(|e| matches!(e, Event::Ops { .. })).unwrap().to_sse();
        assert!(sse.starts_with("id: 1\nevent: ops\ndata: {"));
        assert!(h.take_events().is_empty());
    }

    #[test]
    fn presence_users_and_expiry_flow_through_events() {
        let mut h = Hub::new(&base());
        let a = h.join("ana", Mode::Thin, 1000).unwrap();
        assert_eq!(h.users_view().len(), 1);
        let mut p = Presence::new(0, "ignored");
        p.n = 1;
        p.frame = Some(7.0);
        p.selection = vec!["a".into()];
        h.set_presence(a.replica, p, 1500).unwrap();
        let ev = h.take_events();
        assert!(ev.iter().any(|e| matches!(e, Event::Presence(q) if q.replica == a.replica && q.name == "ana" && q.frame == Some(7.0))));
        assert!(h.set_presence(77, Presence::new(77, "x"), 1500).is_err());
        let c = h.catch_up(0).unwrap();
        assert!(c.iter().any(|e| matches!(e, Event::Users(u) if u.len() == 1)) && c.iter().any(|e| matches!(e, Event::Presence(_))));
        assert!(h.expire(2000).is_empty());
        let gone = h.expire(1500 + 15_001);
        assert_eq!(gone, vec![a.replica], "presence expired");
        assert!(h.has_user(a.replica), "the user itself stays until idle");
        h.idle_ms = 100;
        h.expire(100_000);
        assert!(!h.has_user(a.replica));
        assert!(!h.leave(LOCAL_ID));
        let b = h.join("bo", Mode::Thin, 0).unwrap().replica;
        assert!(h.leave(b) && !h.leave(b));
    }

    #[test]
    fn compaction_is_safe_with_lagging_clients() {
        let mut h = Hub::new(&base());
        let a = h.join("ana", Mode::Thin, 0).unwrap().replica;
        let b = h.join("bo", Mode::Thin, 0).unwrap().replica;
        h.submit_ops(a, None, &[Op::Remove { path: "/layers/0".into() }], &mut ok(), 1).unwrap();
        h.submit_ops(b, None, &[set("/frames", json!(3))], &mut ok(), 2).unwrap();
        let before = h.doc_json();
        let st = h.compact();
        assert_eq!(st["purged"], json!(true));
        assert_eq!(st["tombstones_purged"], json!(1));
        assert_eq!(h.doc_json(), before);
        assert!(h.log_since(0).is_err() || h.log_base() == 0);
        // everything still works after the purge, including undo of the removal (re-inserted as a copy)
        h.undo_redo(a, true, &mut ok(), 3).unwrap();
        assert_eq!(h.doc_json()["layers"].as_array().unwrap().len(), 3);
        assert_eq!(h.doc_json()["layers"][0]["name"], "a");
        h.submit_ops(b, Some(h.index()), &[set("/layers/1/visible", json!(false))], &mut ok(), 4).unwrap();
        // a replica-mode participant blocks the purge
        h.join("rep", Mode::Replica, 5).unwrap();
        let st = h.compact();
        assert_eq!(st["purged"], json!(false));
        // a stale base before the compacted log is reported as resync
        let e = h.submit_ops(a, Some(0), &[set("/frames", json!(9))], &mut ok(), 6);
        if h.log_base() > 0 {
            assert!(e.unwrap_err().resync);
        }
    }

    #[test]
    fn many_thin_clients_editing_stale_views_converge_with_a_plain_replica() {
        // Random pointer edits from stale views are merged by the hub; an independent replica fed the log must agree.
        let mut h = Hub::new(&base());
        let ids: Vec<ReplicaId> = (0..4).map(|i| h.join(&format!("u{i}"), Mode::Thin, 0).unwrap().replica).collect();
        let mut rng = crate::collab::SimRng(11);
        let mut views: BTreeMap<ReplicaId, (usize, Value)> = ids.iter().map(|i| (*i, (0, base()))).collect();
        let mut applied = 0;
        for _ in 0..300 {
            let id = ids[rng.below(ids.len())];
            if rng.chance(40) {
                views.insert(id, (h.index(), h.doc_json())); // the user refreshes
            }
            let (b, v) = views[&id].clone();
            if let Some(op) = crate::collab::random_op(&v, &mut rng) {
                if h.submit_ops(id, Some(b), &[op], &mut ok(), 1).map(|s| s.envelope.is_some()).unwrap_or(false) {
                    applied += 1;
                }
            }
        }
        assert!(applied > 100, "{applied}");
        let mut r = Replica::new(50, &base()).unwrap();
        for (_, e) in h.log_since(0).unwrap() {
            r.receive(e).unwrap();
        }
        assert_eq!(r.json(), h.doc_json());
        assert_eq!(h.index(), applied);
    }
}
