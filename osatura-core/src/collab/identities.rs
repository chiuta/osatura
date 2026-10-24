// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `collab` (see the module docs in `mod.rs`).
use super::*;

pub type ReplicaId = u32;

/// Vector clock: highest envelope `seq` applied per replica (missing = 0).
pub type VClock = BTreeMap<ReplicaId, u64>;

pub(super) const MAX_DEPTH: usize = 400;

pub(super) const MAX_PENDING: usize = 20_000;

pub(super) const MAX_PRIMS: usize = 200_000;

pub(super) const MAX_UNDO: usize = 2_000;

// ---------------------------------------------------------------------------------------------
// identities and stamps

/// Identity of a node or array element: created by envelope `(r, s)` as its `i`-th allocation.
/// `r == 0` is the shared base document (every replica builds it identically).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Id {
    pub r: u32,
    pub s: u64,
    pub i: u32,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.r, self.s, self.i)
    }
}

impl Id {
    pub fn parse(s: &str) -> Option<Id> {
        let mut it = s.split('.');
        let r = it.next()?.parse().ok()?;
        let sq = it.next()?.parse().ok()?;
        let i = it.next()?.parse().ok()?;
        if it.next().is_some() {
            return None;
        }
        Some(Id { r, s: sq, i })
    }
}

impl Serialize for Id {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Id, D::Error> {
        let s = String::deserialize(d)?;
        Id::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("bad id '{s}'")))
    }
}

/// Total order of writes to a register: Lamport time, then replica, then position inside the envelope.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Stamp {
    pub lamport: u64,
    pub replica: u32,
    pub idx: u32,
}

impl Stamp {
    pub(super) const ZERO: Stamp = Stamp { lamport: 0, replica: 0, idx: 0 };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum Val {
    Absent,
    Scalar(Value),
    Ref(Id),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Reg {
    pub(super) stamp: Stamp,
    pub(super) val: Val,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Elem {
    pub(super) id: Id,
    /// Lamport time of the insert (RGA ordering key together with `id`).
    pub(super) ins: u64,
    pub(super) reg: Reg,
    /// Envelope `(replica, seq)` of the (smallest) remove, if removed.
    pub(super) tomb: Option<(u32, u64)>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) enum Cont {
    Obj(BTreeMap<String, Reg>),
    Arr(Vec<Elem>),
}

/// Where a value lives.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Slot {
    Root,
    Key(Id, String),
    Elem(Id, Id),
}

// ---------------------------------------------------------------------------------------------
// primitive operations (what travels in an envelope)

/// An identity-addressed edit. `value` payloads are plain JSON; the containers inside get fresh
/// identities derived from the envelope, identically on every replica.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Prim {
    SetRoot { value: Value },
    SetKey { obj: Id, key: String, value: Value },
    DelKey { obj: Id, key: String },
    SetElem { arr: Id, elem: Id, value: Value },
    InsElem { arr: Id, after: Option<Id>, value: Value },
    DelElem { arr: Id, elem: Id },
}

/// The unit that travels between replicas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    pub replica: ReplicaId,
    pub seq: u64,
    pub lamport: u64,
    pub deps: VClock,
    pub ops: Vec<Prim>,
}

#[derive(Clone, Copy)]
pub(super) struct Meta {
    pub(super) replica: u32,
    pub(super) seq: u64,
    pub(super) lamport: u64,
}

pub(super) struct IdGen {
    pub(super) r: u32,
    pub(super) s: u64,
    pub(super) next: u32,
}

impl IdGen {
    pub(super) fn peek(&self) -> Id {
        Id { r: self.r, s: self.s, i: self.next }
    }
    pub(super) fn take(&mut self) -> Id {
        let id = self.peek();
        self.next += 1;
        id
    }
}

/// How to reverse one primitive (kept locally, never sent).
#[derive(Clone, Debug)]
pub(super) enum Undo {
    Restore { slot: Slot, stamp: Stamp, prev: Option<Value> },
    Remove { arr: Id, elem: Id },
    Reinsert { arr: Id, elem: Id, left: Option<Id>, value: Value },
}

// ---------------------------------------------------------------------------------------------
// the replicated tree

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc {
    pub(super) nodes: BTreeMap<Id, Cont>,
    pub(super) root: Reg,
}

impl Doc {
    pub fn from_json(v: &Value) -> Doc {
        let mut d = Doc { nodes: BTreeMap::new(), root: Reg { stamp: Stamp::ZERO, val: Val::Absent } };
        let mut gen = IdGen { r: 0, s: 0, next: 0 };
        let val = d.build(v, &mut gen, Stamp::ZERO, 0, 0);
        d.root.val = val;
        d
    }

    pub(super) fn build(&mut self, v: &Value, gen: &mut IdGen, stamp: Stamp, ins: u64, depth: usize) -> Val {
        if depth > MAX_DEPTH {
            return Val::Scalar(Value::Null);
        }
        match v {
            Value::Object(m) => {
                let id = gen.take();
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                let mut ent = BTreeMap::new();
                for k in keys {
                    let val = self.build(&m[k], gen, stamp, ins, depth + 1);
                    ent.insert(k.clone(), Reg { stamp, val });
                }
                self.nodes.insert(id, Cont::Obj(ent));
                Val::Ref(id)
            }
            Value::Array(a) => {
                let id = gen.take();
                let mut elems = Vec::with_capacity(a.len());
                for x in a {
                    let eid = gen.take();
                    let val = self.build(x, gen, stamp, ins, depth + 1);
                    elems.push(Elem { id: eid, ins, reg: Reg { stamp, val }, tomb: None });
                }
                self.nodes.insert(id, Cont::Arr(elems));
                Val::Ref(id)
            }
            other => Val::Scalar(other.clone()),
        }
    }

    pub(super) fn val_json(&self, v: &Val, depth: usize) -> Option<Value> {
        if depth > MAX_DEPTH {
            return Some(Value::Null);
        }
        match v {
            Val::Absent => None,
            Val::Scalar(s) => Some(s.clone()),
            Val::Ref(id) => match self.nodes.get(id) {
                Some(Cont::Obj(m)) => {
                    let mut out = serde_json::Map::new();
                    for (k, r) in m {
                        if let Some(x) = self.val_json(&r.val, depth + 1) {
                            out.insert(k.clone(), x);
                        }
                    }
                    Some(Value::Object(out))
                }
                Some(Cont::Arr(es)) => Some(Value::Array(es.iter().filter(|e| e.tomb.is_none()).filter_map(|e| self.val_json(&e.reg.val, depth + 1)).collect())),
                None => Some(Value::Null),
            },
        }
    }

    /// The visible JSON document.
    pub fn to_json(&self) -> Value {
        self.val_json(&self.root.val, 0).unwrap_or(Value::Null)
    }

    pub(super) fn arr(&self, id: Id) -> Option<&Vec<Elem>> {
        match self.nodes.get(&id) {
            Some(Cont::Arr(v)) => Some(v),
            _ => None,
        }
    }
    pub(super) fn obj(&self, id: Id) -> Option<&BTreeMap<String, Reg>> {
        match self.nodes.get(&id) {
            Some(Cont::Obj(m)) => Some(m),
            _ => None,
        }
    }

    pub(super) fn visible(&self, arr: Id) -> Vec<&Elem> {
        self.arr(arr).map(|v| v.iter().filter(|e| e.tomb.is_none()).collect()).unwrap_or_default()
    }

    pub(super) fn slot_val(&self, slot: &Slot) -> Result<Val, String> {
        match slot {
            Slot::Root => Ok(self.root.val.clone()),
            Slot::Key(o, k) => Ok(self.obj(*o).ok_or("unknown object")?.get(k).map(|r| r.val.clone()).unwrap_or(Val::Absent)),
            Slot::Elem(a, e) => Ok(self.arr(*a).ok_or("unknown array")?.iter().find(|x| x.id == *e).ok_or("unknown element")?.reg.val.clone()),
        }
    }

    pub(super) fn slot_json(&self, slot: &Slot) -> Option<Value> {
        self.slot_val(slot).ok().and_then(|v| self.val_json(&v, 0))
    }

    pub(super) fn reg_stamp(&self, slot: &Slot) -> Option<Stamp> {
        match slot {
            Slot::Root => Some(self.root.stamp),
            Slot::Key(o, k) => self.obj(*o).and_then(|m| m.get(k)).map(|r| r.stamp),
            Slot::Elem(a, e) => self.arr(*a).and_then(|v| v.iter().find(|x| x.id == *e)).map(|x| x.reg.stamp),
        }
    }

    pub(super) fn reg_mut(&mut self, slot: &Slot) -> Result<&mut Reg, String> {
        match slot {
            Slot::Root => Ok(&mut self.root),
            Slot::Key(o, k) => match self.nodes.get_mut(o) {
                Some(Cont::Obj(m)) => Ok(m.entry(k.clone()).or_insert(Reg { stamp: Stamp::ZERO, val: Val::Absent })),
                _ => Err("unknown object".into()),
            },
            Slot::Elem(a, e) => match self.nodes.get_mut(a) {
                Some(Cont::Arr(v)) => v.iter_mut().find(|x| x.id == *e).map(|x| &mut x.reg).ok_or_else(|| "unknown element".to_string()),
                _ => Err("unknown array".into()),
            },
        }
    }

    pub(super) fn check_slot(&self, slot: &Slot) -> Result<(), String> {
        match slot {
            Slot::Root => Ok(()),
            Slot::Key(o, _) => self.obj(*o).map(|_| ()).ok_or_else(|| "unknown object".into()),
            Slot::Elem(a, e) => {
                let v = self.arr(*a).ok_or("unknown array")?;
                if v.iter().any(|x| x.id == *e) {
                    Ok(())
                } else {
                    Err("unknown element".into())
                }
            }
        }
    }

    /// LWW write (`value == None` deletes). Identities inside the value are always allocated, even
    /// when the write loses, so that every replica allocates identically.
    pub(super) fn write(&mut self, slot: &Slot, stamp: Stamp, value: Option<&Value>, gen: &mut IdGen, want_undo: bool) -> Result<Vec<Undo>, String> {
        self.check_slot(slot)?;
        let prev = if want_undo { Some(self.slot_json(slot)) } else { None };
        let new = match value {
            Some(v) => self.build(v, gen, stamp, stamp.lamport, 0),
            None => Val::Absent,
        };
        let reg = self.reg_mut(slot)?;
        if stamp > reg.stamp {
            reg.stamp = stamp;
            reg.val = new;
            if let Some(p) = prev {
                return Ok(vec![Undo::Restore { slot: slot.clone(), stamp, prev: p }]);
            }
        }
        Ok(vec![])
    }

    pub(super) fn apply_prim(&mut self, m: &Meta, idx: u32, gen: &mut IdGen, prim: &Prim, want_undo: bool) -> Result<Vec<Undo>, String> {
        let stamp = Stamp { lamport: m.lamport, replica: m.replica, idx };
        match prim {
            Prim::SetRoot { value } => self.write(&Slot::Root, stamp, Some(value), gen, want_undo),
            Prim::SetKey { obj, key, value } => self.write(&Slot::Key(*obj, key.clone()), stamp, Some(value), gen, want_undo),
            Prim::DelKey { obj, key } => self.write(&Slot::Key(*obj, key.clone()), stamp, None, gen, want_undo),
            Prim::SetElem { arr, elem, value } => self.write(&Slot::Elem(*arr, *elem), stamp, Some(value), gen, want_undo),
            Prim::InsElem { arr, after, value } => {
                let v = self.arr(*arr).ok_or("unknown array")?;
                let mut pos = match after {
                    None => 0,
                    Some(a) => v.iter().position(|e| e.id == *a).ok_or("unknown anchor element")? + 1,
                };
                let id = gen.take();
                let val = self.build(value, gen, stamp, stamp.lamport, 0);
                let v = match self.nodes.get_mut(arr) {
                    Some(Cont::Arr(v)) => v,
                    _ => return Err("unknown array".into()),
                };
                // RGA: pass over elements inserted concurrently with a larger (lamport, id)
                while pos < v.len() && (v[pos].ins, v[pos].id) > (stamp.lamport, id) {
                    pos += 1;
                }
                v.insert(pos, Elem { id, ins: stamp.lamport, reg: Reg { stamp, val }, tomb: None });
                Ok(if want_undo { vec![Undo::Remove { arr: *arr, elem: id }] } else { vec![] })
            }
            Prim::DelElem { arr, elem } => {
                let (value, was_tomb, left) = {
                    let v = self.arr(*arr).ok_or("unknown array")?;
                    let p = v.iter().position(|e| e.id == *elem).ok_or("unknown element")?;
                    let e = &v[p];
                    let left = v[..p].iter().rev().find(|x| x.tomb.is_none()).map(|x| x.id);
                    (if want_undo && e.tomb.is_none() { self.val_json(&e.reg.val, 0) } else { None }, e.tomb.is_some(), left)
                };
                if let Some(Cont::Arr(v)) = self.nodes.get_mut(arr) {
                    if let Some(e) = v.iter_mut().find(|e| e.id == *elem) {
                        let t = (m.replica, m.seq);
                        e.tomb = Some(match e.tomb {
                            Some(old) if old <= t => old,
                            _ => t,
                        });
                    }
                }
                Ok(match (want_undo && !was_tomb, value) {
                    (true, Some(value)) => vec![Undo::Reinsert { arr: *arr, elem: *elem, left, value }],
                    _ => vec![],
                })
            }
        }
    }

    pub(super) fn reachable(&self) -> BTreeSet<Id> {
        let mut seen = BTreeSet::new();
        let mut stack = Vec::new();
        if let Val::Ref(id) = self.root.val {
            stack.push(id);
        }
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            match self.nodes.get(&id) {
                Some(Cont::Obj(m)) => {
                    for r in m.values() {
                        if let Val::Ref(c) = r.val {
                            stack.push(c);
                        }
                    }
                }
                Some(Cont::Arr(v)) => {
                    for e in v {
                        if let Val::Ref(c) = e.reg.val {
                            stack.push(c);
                        }
                    }
                }
                None => {}
            }
        }
        seen
    }
}
