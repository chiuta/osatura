// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `collab` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// replica

#[derive(Clone, Debug, PartialEq)]
pub enum Delivery {
    /// Newly applied envelopes, in application order (the one received plus any it released).
    Applied(Vec<Envelope>),
    Duplicate,
    /// Held back: a predecessor or a dependency is missing.
    Buffered,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompactStats {
    pub log_dropped: usize,
    pub tombstones_purged: usize,
    pub nodes_collected: usize,
    /// True when tombstones could be purged (every peer had exactly this replica's clock).
    pub purged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub doc: Doc,
    pub clock: VClock,
    pub lamport: u64,
}

#[derive(Clone)]
pub struct Replica {
    pub(super) id: ReplicaId,
    pub(super) lamport: u64,
    pub(super) applied: VClock,
    pub(super) doc: Doc,
    pub(super) pending: Vec<Envelope>,
    pub(super) log: Vec<Envelope>,
    pub(super) undo: Vec<Vec<Undo>>,
    pub(super) redo: Vec<Vec<Undo>>,
    /// Primitives dropped because they named something unknown (should stay 0 with honest peers).
    pub dropped: u64,
}

impl Replica {
    /// A replica holding the shared base document. All replicas must be created from the same base.
    pub fn new(id: ReplicaId, base: &Value) -> Result<Replica, String> {
        if id == 0 {
            return Err("replica id 0 is reserved for the base document".into());
        }
        Ok(Replica { id, lamport: 0, applied: VClock::new(), doc: Doc::from_json(base), pending: vec![], log: vec![], undo: vec![], redo: vec![], dropped: 0 })
    }

    /// Join an existing session from a snapshot (a fresh replica id, or a known one to continue it).
    pub fn from_snapshot(id: ReplicaId, snap: Snapshot) -> Result<Replica, String> {
        if id == 0 {
            return Err("replica id 0 is reserved for the base document".into());
        }
        Ok(Replica { id, lamport: snap.lamport, applied: snap.clock, doc: snap.doc, pending: vec![], log: vec![], undo: vec![], redo: vec![], dropped: 0 })
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot { doc: self.doc.clone(), clock: self.applied.clone(), lamport: self.lamport }
    }

    /// Same replica state seen from another identity (a server-side proxy for one user).
    pub fn fork_as(&self, id: ReplicaId) -> Result<Replica, String> {
        Replica::from_snapshot(id, self.snapshot())
    }

    pub fn id(&self) -> ReplicaId {
        self.id
    }
    pub fn lamport(&self) -> u64 {
        self.lamport
    }
    pub fn clock(&self) -> &VClock {
        &self.applied
    }
    pub fn doc(&self) -> &Doc {
        &self.doc
    }
    pub fn json(&self) -> Value {
        self.doc.to_json()
    }
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }
    pub fn log_len(&self) -> usize {
        self.log.len()
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn tombstones(&self) -> usize {
        self.doc.nodes.values().map(|c| if let Cont::Arr(v) = c { v.iter().filter(|e| e.tomb.is_some()).count() } else { 0 }).sum()
    }
    pub fn node_count(&self) -> usize {
        self.doc.nodes.len()
    }

    pub(super) fn local_tx<F>(&mut self, f: F) -> Result<Option<(Envelope, Vec<Undo>)>, String>
    where
        F: FnOnce(&mut Doc, &mut Tx) -> Result<(), String>,
    {
        let backup = self.doc.clone();
        let seq = self.applied.get(&self.id).copied().unwrap_or(0) + 1;
        let lamport = self.lamport + 1;
        let mut tx = Tx { meta: Meta { replica: self.id, seq, lamport }, idx: 0, gen: IdGen { r: self.id, s: seq, next: 0 }, prims: vec![], undos: vec![] };
        if let Err(e) = f(&mut self.doc, &mut tx) {
            self.doc = backup;
            return Err(e);
        }
        if tx.prims.is_empty() {
            return Ok(None);
        }
        let deps = self.applied.clone();
        self.lamport = lamport;
        self.applied.insert(self.id, seq);
        let env = Envelope { replica: self.id, seq, lamport, deps, ops: tx.prims };
        self.log.push(env.clone());
        Ok(Some((env, coalesce(tx.undos))))
    }

    /// Apply a local edit given as the editor's pointer ops. Returns the envelope to broadcast, or `None`
    /// when the edit changed nothing. On error nothing changed (a batch is atomic).
    pub fn local_op(&mut self, op: &Op) -> Result<Option<Envelope>, String> {
        let r = self.local_tx(|doc, tx| translate(doc, tx, op))?;
        Ok(r.map(|(env, undos)| {
            self.push_undo(undos, true);
            env
        }))
    }

    /// Apply a local edit given as the complete new document (diffed against the current one).
    pub fn local_doc(&mut self, new_doc: &Value) -> Result<Option<Envelope>, String> {
        let r = self.local_tx(|doc, tx| diff_slot(doc, tx, Slot::Root, new_doc))?;
        Ok(r.map(|(env, undos)| {
            self.push_undo(undos, true);
            env
        }))
    }

    pub(super) fn push_undo(&mut self, undos: Vec<Undo>, clear_redo: bool) {
        if !undos.is_empty() {
            self.undo.push(undos);
            if self.undo.len() > MAX_UNDO {
                self.undo.remove(0);
            }
        }
        if clear_redo {
            self.redo.clear();
        }
    }

    /// Undo this replica's own most recent edit (skipping steps that no longer have any effect).
    pub fn undo(&mut self) -> Result<Option<Envelope>, String> {
        self.step_back(true)
    }
    pub fn redo(&mut self) -> Result<Option<Envelope>, String> {
        self.step_back(false)
    }

    pub(super) fn step_back(&mut self, undoing: bool) -> Result<Option<Envelope>, String> {
        loop {
            let recs = if undoing { self.undo.pop() } else { self.redo.pop() };
            let Some(recs) = recs else { return Ok(None) };
            let r = self.local_tx(|doc, tx| {
                for u in recs.iter().rev() {
                    // a record whose target is gone is skipped; whatever was emitted before stays consistent
                    let _ = plan_inverse(doc, tx, u);
                }
                Ok(())
            })?;
            if let Some((env, inv)) = r {
                let inv = coalesce(inv);
                if !inv.is_empty() {
                    let stack = if undoing { &mut self.redo } else { &mut self.undo };
                    stack.push(inv);
                }
                return Ok(Some(env));
            }
        }
    }

    pub(super) fn deliverable(&self, e: &Envelope) -> bool {
        let have = self.applied.get(&e.replica).copied().unwrap_or(0);
        e.seq == have + 1 && e.deps.iter().all(|(r, s)| self.applied.get(r).copied().unwrap_or(0) >= *s || (*r == e.replica && *s < e.seq && *s <= have))
    }

    /// Applied envelopes still in the log, oldest first (after compaction a suffix of the full history).
    pub fn log_entries(&self) -> &[Envelope] {
        &self.log
    }

    /// Apply one deliverable envelope only if `validate` accepts the merged document; otherwise leave the
    /// replica exactly as it was. Used by a server that must never hold an invalid project.
    pub fn receive_checked(&mut self, e: Envelope, validate: &mut dyn FnMut(&Value) -> Result<(), Vec<String>>) -> Result<(), Vec<String>> {
        self.validate(&e).map_err(|x| vec![x])?;
        if !self.deliverable(&e) {
            return Err(vec![format!("envelope {}:{} is not deliverable now (duplicate, or a predecessor is missing)", e.replica, e.seq)]);
        }
        let backup = (self.doc.clone(), self.applied.clone(), self.lamport, self.dropped);
        let log_len = self.log.len();
        self.apply_env(e);
        if let Err(errs) = validate(&self.doc.to_json()) {
            self.doc = backup.0;
            self.applied = backup.1;
            self.lamport = backup.2;
            self.dropped = backup.3;
            self.log.truncate(log_len);
            return Err(errs);
        }
        Ok(())
    }

    /// Would `receive` apply this envelope right now (not a duplicate, all predecessors present)?
    pub fn is_deliverable(&self, e: &Envelope) -> bool {
        self.deliverable(e)
    }

    pub(super) fn validate(&self, e: &Envelope) -> Result<(), String> {
        if e.replica == 0 || e.seq == 0 {
            return Err("envelope needs replica >= 1 and seq >= 1".into());
        }
        if e.lamport == 0 || e.lamport > (1u64 << 60) {
            return Err("envelope lamport time out of range".into());
        }
        if e.ops.len() > MAX_PRIMS || e.deps.len() > 4096 {
            return Err("envelope is too large".into());
        }
        if e.replica == self.id && e.seq > self.applied.get(&self.id).copied().unwrap_or(0) {
            return Err("an envelope carries this replica's id but was not made by it".into());
        }
        Ok(())
    }

    pub(super) fn apply_env(&mut self, e: Envelope) {
        let meta = Meta { replica: e.replica, seq: e.seq, lamport: e.lamport };
        let mut gen = IdGen { r: e.replica, s: e.seq, next: 0 };
        for (i, p) in e.ops.iter().enumerate() {
            if self.doc.apply_prim(&meta, i as u32, &mut gen, p, false).is_err() {
                self.dropped += 1;
            }
        }
        self.applied.insert(e.replica, e.seq);
        self.lamport = self.lamport.max(e.lamport);
        self.log.push(e);
    }

    /// Receive an envelope from another replica (or a re-delivery of any envelope).
    pub fn receive(&mut self, e: Envelope) -> Result<Delivery, String> {
        self.validate(&e)?;
        if e.seq <= self.applied.get(&e.replica).copied().unwrap_or(0) {
            return Ok(Delivery::Duplicate);
        }
        if self.pending.iter().any(|p| p.replica == e.replica && p.seq == e.seq) {
            return Ok(Delivery::Buffered);
        }
        if self.pending.len() >= MAX_PENDING {
            return Err("too many envelopes waiting for their predecessors".into());
        }
        self.pending.push(e);
        let mut applied = Vec::new();
        loop {
            let next = self.pending.iter().enumerate().filter(|(_, p)| self.deliverable(p)).min_by_key(|(_, p)| (p.replica, p.seq)).map(|(i, _)| i);
            let Some(i) = next else { break };
            let env = self.pending.remove(i);
            applied.push(env.clone());
            self.apply_env(env);
        }
        Ok(if applied.is_empty() { Delivery::Buffered } else { Delivery::Applied(applied) })
    }

    /// Envelopes a peer with clock `clock` still lacks, in a causal order. `Err` when the log was
    /// compacted past what the peer needs (it must fetch a snapshot instead).
    pub fn missing_for(&self, clock: &VClock) -> Result<Vec<Envelope>, String> {
        let mut need = 0usize;
        for (r, s) in &self.applied {
            need += (*s - clock.get(r).copied().unwrap_or(0).min(*s)) as usize;
        }
        let out: Vec<Envelope> = self.log.iter().filter(|e| e.seq > clock.get(&e.replica).copied().unwrap_or(0)).cloned().collect();
        if out.len() < need {
            return Err("log was compacted past the peer's clock: fetch a snapshot".into());
        }
        Ok(out)
    }

    /// Snapshot + log compaction. `peers` are the clocks of every other replica that takes part.
    pub fn compact(&mut self, peers: &[VClock]) -> CompactStats {
        let mut st = CompactStats::default();
        let mut min = self.applied.clone();
        for p in peers {
            for (r, s) in min.iter_mut() {
                *s = (*s).min(p.get(r).copied().unwrap_or(0));
            }
        }
        // only a prefix of the log is dropped, so positions stay meaningful for a relay that numbers them
        let k = self.log.iter().take_while(|e| e.seq <= min.get(&e.replica).copied().unwrap_or(0)).count();
        self.log.drain(..k);
        st.log_dropped = k;
        let nz = |c: &VClock| -> VClock { c.iter().filter(|(_, v)| **v > 0).map(|(k, v)| (*k, *v)).collect() };
        let mine = nz(&self.applied);
        if self.pending.is_empty() && peers.iter().all(|p| nz(p) == mine) {
            st.purged = true;
            for c in self.doc.nodes.values_mut() {
                match c {
                    Cont::Arr(v) => {
                        let n = v.len();
                        v.retain(|e| e.tomb.is_none());
                        st.tombstones_purged += n - v.len();
                    }
                    Cont::Obj(m) => {
                        m.retain(|_, r| r.val != Val::Absent);
                    }
                }
            }
            let keep = self.doc.reachable();
            let n = self.doc.nodes.len();
            self.doc.nodes.retain(|id, _| keep.contains(id));
            st.nodes_collected = n - self.doc.nodes.len();
        }
        st
    }
}
