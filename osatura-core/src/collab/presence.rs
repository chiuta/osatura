// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `collab` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// presence (cursor, selection, name, colour) with expiry

pub(super) const PALETTE: [&str; 8] = ["#e6194b", "#3cb44b", "#4363d8", "#f58231", "#911eb4", "#008080", "#9a6324", "#d2691e"];

pub fn colour_for(replica: ReplicaId) -> String {
    PALETTE[(replica as usize) % PALETTE.len()].to_string()
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Presence {
    pub replica: ReplicaId,
    pub name: String,
    pub colour: String,
    /// Playhead of that user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<f32>,
    /// Pointer position in canvas units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<[f32; 2]>,
    /// Selected layer paths.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selection: Vec<String>,
    /// Canvas rectangle `[x0, y0, x1, y1]` around the selected layers at the user's playhead; set by the
    /// server from the selection (never taken from the client), so the others can draw the selection box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f32; 4]>,
    /// Sender's counter: an older update never overwrites a newer one.
    #[serde(default)]
    pub n: u64,
    /// Time of the last update in the caller's clock (milliseconds).
    #[serde(default)]
    pub seen_ms: u64,
}

pub(super) fn valid_colour(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|x| x.is_ascii_hexdigit())
}

impl Presence {
    pub fn new(replica: ReplicaId, name: &str) -> Presence {
        Presence { replica, name: clean_name(name, replica), colour: colour_for(replica), frame: None, cursor: None, selection: vec![], bounds: None, n: 0, seen_ms: 0 }
    }
    /// Make an untrusted update safe: bounded sizes, finite numbers, a valid `#rrggbb` colour.
    pub fn sanitized(mut self) -> Presence {
        self.name = clean_name(&self.name, self.replica);
        if !valid_colour(&self.colour) {
            self.colour = colour_for(self.replica);
        }
        if let Some(f) = self.frame {
            if !f.is_finite() {
                self.frame = None;
            }
        }
        if let Some(c) = self.cursor {
            if !(c[0].is_finite() && c[1].is_finite()) {
                self.cursor = None;
            }
        }
        if let Some(b) = self.bounds {
            if !b.iter().all(|v| v.is_finite()) {
                self.bounds = None;
            }
        }
        self.selection.truncate(64);
        for s in self.selection.iter_mut() {
            *s = s.chars().filter(|c| !c.is_control()).take(200).collect();
        }
        self
    }
}

pub(super) fn clean_name(n: &str, replica: ReplicaId) -> String {
    let s: String = n.chars().filter(|c| !c.is_control()).take(40).collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        format!("user{replica}")
    } else {
        s
    }
}

/// Who is here. The caller supplies the time (milliseconds, any monotone clock), so this is deterministic.
#[derive(Clone, Debug)]
pub struct PresenceBoard {
    pub(super) entries: BTreeMap<ReplicaId, Presence>,
    pub ttl_ms: u64,
}

impl PresenceBoard {
    pub fn new(ttl_ms: u64) -> PresenceBoard {
        PresenceBoard { entries: BTreeMap::new(), ttl_ms }
    }
    /// Returns true when the update was accepted (not older than what is stored).
    pub fn update(&mut self, p: Presence, now_ms: u64) -> bool {
        let mut p = p.sanitized();
        if let Some(old) = self.entries.get(&p.replica) {
            if p.n < old.n {
                return false;
            }
        }
        p.seen_ms = now_ms;
        self.entries.insert(p.replica, p);
        true
    }
    pub fn touch(&mut self, replica: ReplicaId, now_ms: u64) {
        if let Some(p) = self.entries.get_mut(&replica) {
            p.seen_ms = now_ms;
        }
    }
    pub fn remove(&mut self, replica: ReplicaId) -> bool {
        self.entries.remove(&replica).is_some()
    }
    /// Drop entries not refreshed within the TTL; returns who expired.
    pub fn expire(&mut self, now_ms: u64) -> Vec<ReplicaId> {
        let ttl = self.ttl_ms;
        let gone: Vec<ReplicaId> = self.entries.iter().filter(|(_, p)| now_ms.saturating_sub(p.seen_ms) > ttl).map(|(r, _)| *r).collect();
        for r in &gone {
            self.entries.remove(r);
        }
        gone
    }
    pub fn get(&self, r: ReplicaId) -> Option<&Presence> {
        self.entries.get(&r)
    }
    pub fn all(&self) -> Vec<Presence> {
        self.entries.values().cloned().collect()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------------------------
// seeded simulation (used by the tests and by `osatura collab-sim`)

/// splitmix64: tiny deterministic generator, no dependency.
pub struct SimRng(pub u64);

impl SimRng {
    // Not `Iterator::next`: an endless generator returning `u64`, and the name is part of the stable API.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
    pub fn chance(&mut self, pct: usize) -> bool {
        self.below(100) < pct
    }
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i + 1);
            v.swap(i, j);
        }
    }
}

pub(super) fn collect_paths(v: &Value, path: &mut String, out: &mut Vec<String>, depth: usize) {
    out.push(path.clone());
    if depth > 12 || out.len() > 4000 {
        return;
    }
    match v {
        Value::Object(m) => {
            for (k, x) in m {
                let n = path.len();
                path.push('/');
                path.push_str(&k.replace('~', "~0").replace('/', "~1"));
                collect_paths(x, path, out, depth + 1);
                path.truncate(n);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                let n = path.len();
                path.push_str(&format!("/{i}"));
                collect_paths(x, path, out, depth + 1);
                path.truncate(n);
            }
        }
        _ => {}
    }
}

pub(super) fn bump_leaf(v: &mut Value, rng: &mut SimRng) -> bool {
    match v {
        Value::Number(n) => {
            let x = n.as_f64().unwrap_or(0.0);
            let d = (rng.below(21) as f64 - 10.0) / 2.0;
            *v = serde_json::Number::from_f64(x + d).map(Value::Number).unwrap_or(Value::Null);
            true
        }
        Value::String(s) => {
            *s = format!("s{}", rng.below(1000));
            true
        }
        Value::Bool(b) => {
            *b = !*b;
            true
        }
        Value::Object(m) => {
            let keys: Vec<String> = m.keys().cloned().collect();
            if keys.is_empty() {
                return false;
            }
            let k = &keys[rng.below(keys.len())];
            bump_leaf(m.get_mut(k).unwrap(), rng)
        }
        Value::Array(a) => {
            if a.is_empty() {
                return false;
            }
            let i = rng.below(a.len());
            bump_leaf(&mut a[i], rng)
        }
        Value::Null => false,
    }
}

/// A random edit of `doc` (any JSON), or `None` when the dice picked something impossible.
pub fn random_op(doc: &Value, rng: &mut SimRng) -> Option<Op> {
    let mut paths = Vec::new();
    collect_paths(doc, &mut String::new(), &mut paths, 0);
    let path = paths[rng.below(paths.len())].clone();
    let at = if path.is_empty() { doc } else { doc.pointer(&path)? };
    let parent_ptr = path.rfind('/').map(|i| path[..i].to_string());
    let parent = parent_ptr.as_ref().and_then(|p| if p.is_empty() { Some(doc) } else { doc.pointer(p) });
    let in_array = parent.map(|p| p.is_array()).unwrap_or(false);
    let fresh = |rng: &mut SimRng| serde_json::json!({"name": format!("n{}", rng.below(50)), "v": rng.below(100), "tags": [rng.below(5), rng.below(5)]});
    match at {
        Value::Number(_) | Value::String(_) | Value::Bool(_) => {
            let mut nv = at.clone();
            bump_leaf(&mut nv, rng);
            if in_array && rng.chance(25) {
                return Some(Op::Remove { path });
            }
            if in_array && rng.chance(20) {
                let i = path.rsplit('/').next()?.to_string();
                let p = parent_ptr?;
                return Some(Op::Insert { path: format!("{p}/{i}"), value: nv });
            }
            Some(Op::Set { path, value: nv })
        }
        Value::Null => Some(Op::Set { path, value: Value::from(rng.below(10) as u64) }),
        Value::Array(a) => {
            let r = rng.below(10);
            if r < 6 || a.is_empty() {
                let i = rng.below(a.len() + 2);
                let ptr = if i > a.len() { format!("{path}/-") } else { format!("{path}/{i}") };
                let v = if !a.is_empty() && rng.chance(40) { a[rng.below(a.len())].clone() } else { fresh(rng) };
                Some(Op::Insert { path: ptr, value: v })
            } else if r < 8 {
                let i = rng.below(a.len());
                Some(Op::Remove { path: format!("{path}/{i}") })
            } else {
                let mut nv = at.clone();
                bump_leaf(&mut nv, rng);
                Some(Op::Set { path, value: nv })
            }
        }
        Value::Object(m) => {
            let r = rng.below(10);
            if r < 4 {
                let k = format!("k{}", rng.below(4));
                let v = if rng.chance(50) { Value::from(rng.below(100) as u64) } else { fresh(rng) };
                Some(Op::Set { path: format!("{path}/{k}"), value: v })
            } else if r < 6 && !m.is_empty() && !path.is_empty() {
                let keys: Vec<&String> = m.keys().collect();
                let k = keys[rng.below(keys.len())].replace('~', "~0").replace('/', "~1");
                Some(Op::Remove { path: format!("{path}/{k}") })
            } else if r < 9 {
                // whole-object replacement with one leaf changed: exercises the structural diff
                let mut nv = at.clone();
                if !bump_leaf(&mut nv, rng) {
                    return None;
                }
                Some(Op::Set { path, value: nv })
            } else if !path.is_empty() && in_array {
                Some(Op::Remove { path })
            } else {
                None
            }
        }
    }
}

/// A random edit that keeps a *project* document valid: numeric properties (bone rest pose, layer opacity / position,
/// key values of position/rotation channels), layer visibility, and layers the simulation itself created
/// (`sim_*`: inserted, moved by re-creating under a new name, removed). Referenced layers are never removed or
/// renamed, so every merge of such edits is still a project that compiles.
pub fn project_op(doc: &Value, rng: &mut SimRng) -> Option<Op> {
    let arr = |p: &str| doc.pointer(p).and_then(Value::as_array);
    let layers = arr("/layers")?;
    let delta = |rng: &mut SimRng, x: f64, span: usize| x + (rng.below(span * 2 + 1) as f64 - span as f64) / 2.0;
    let num = |v: &Value| v.as_f64();
    for _ in 0..8 {
        match rng.below(9) {
            0 => {
                let bones = arr("/bones")?;
                if bones.is_empty() {
                    continue;
                }
                let i = rng.below(bones.len());
                let x = num(&bones[i]["rest_angle"])?;
                return Some(Op::Set { path: format!("/bones/{i}/rest_angle"), value: Value::from(delta(rng, x, 10)) });
            }
            1 => {
                let bones = arr("/bones")?;
                if bones.is_empty() {
                    continue;
                }
                let (i, k) = (rng.below(bones.len()), rng.below(2));
                let x = num(&bones[i]["rest_offset"][k])?;
                return Some(Op::Set { path: format!("/bones/{i}/rest_offset/{k}"), value: Value::from(delta(rng, x, 6)) });
            }
            2 => {
                if layers.is_empty() {
                    continue;
                }
                let i = rng.below(layers.len());
                let v = (0.2 + rng.below(81) as f64 / 100.0).min(1.0);
                return Some(Op::Set { path: format!("/layers/{i}/opacity"), value: Value::from(v) });
            }
            3 => {
                if layers.is_empty() {
                    continue;
                }
                let (i, k) = (rng.below(layers.len()), rng.below(2));
                let x = num(&layers[i]["transform"]["position"][k]).unwrap_or(0.0);
                return Some(Op::Set { path: format!("/layers/{i}/transform/position/{k}"), value: Value::from(delta(rng, x, 20)) });
            }
            4 => {
                if layers.is_empty() {
                    continue;
                }
                let i = rng.below(layers.len());
                let v = layers[i]["visible"].as_bool().unwrap_or(true);
                return Some(Op::Set { path: format!("/layers/{i}/visible"), value: Value::Bool(!v) });
            }
            5 => {
                // a key value of a position / rotation channel
                let chans = arr("/channels")?;
                let ok = ["bone_angle", "bone_offset_x", "bone_offset_y", "layer_x", "layer_y", "layer_rotation"];
                let keyed: Vec<usize> = (0..chans.len())
                    .filter(|&c| chans[c]["source"] == "keys" && chans[c]["prop"].as_str().is_some_and(|p| ok.contains(&p)) && chans[c]["track"]["keys"].as_array().is_some_and(|k| !k.is_empty()))
                    .collect();
                if keyed.is_empty() {
                    continue;
                }
                let c = keyed[rng.below(keyed.len())];
                let keys = chans[c]["track"]["keys"].as_array()?;
                let k = rng.below(keys.len());
                let x = num(&keys[k]["value"])?;
                return Some(Op::Set { path: format!("/channels/{c}/track/keys/{k}/value"), value: Value::from(delta(rng, x, 10)) });
            }
            6 => {
                // insert a fresh layer of our own
                let name = format!("sim_{}", rng.below(1_000_000_000));
                let (x, y) = (rng.below(300) as f32, rng.below(300) as f32);
                let l = crate::project::Layer::vector(&name, vec![crate::path::VPath::rect("r", x, y, 20.0 + rng.below(60) as f32, 20.0 + rng.below(60) as f32).with_fill("#3366aa")]);
                let at = rng.below(layers.len() + 1);
                return Some(Op::Insert { path: format!("/layers/{at}"), value: serde_json::to_value(&l).ok()? });
            }
            7 => {
                let own: Vec<usize> = (0..layers.len()).filter(|&i| layers[i]["name"].as_str().is_some_and(|n| n.starts_with("sim_"))).collect();
                if own.is_empty() {
                    continue;
                }
                return Some(Op::Remove { path: format!("/layers/{}", own[rng.below(own.len())]) });
            }
            _ => {
                // move one of our layers: re-create it under a new name at another index (no duplicate names even when two
                // replicas move the same layer concurrently)
                let own: Vec<usize> = (0..layers.len()).filter(|&i| layers[i]["name"].as_str().is_some_and(|n| n.starts_with("sim_"))).collect();
                if own.is_empty() {
                    continue;
                }
                let i = own[rng.below(own.len())];
                let mut v = layers[i].clone();
                v["name"] = Value::from(format!("sim_{}", rng.below(1_000_000_000)));
                let at = rng.below(layers.len());
                return Some(Op::Batch { ops: vec![Op::Remove { path: format!("/layers/{i}") }, Op::Insert { path: format!("/layers/{}", at.min(layers.len() - 1)), value: v }] });
            }
        }
    }
    None
}

#[derive(Clone, Debug)]
pub struct SimConfig {
    pub clients: usize,
    pub ops: usize,
    pub seed: u64,
    /// Percent of steps that are an undo / redo of the acting client.
    pub undo_pct: usize,
    /// Percent of sent envelopes that are delivered twice.
    pub dup_pct: usize,
    /// Edit with [`project_op`] (valid project edits) instead of arbitrary JSON edits ([`random_op`]).
    pub project_aware: bool,
}

impl SimConfig {
    pub fn new(clients: usize, ops: usize, seed: u64) -> SimConfig {
        SimConfig { clients, ops, seed, undo_pct: 12, dup_pct: 8, project_aware: false }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimReport {
    pub clients: usize,
    pub edits: usize,
    pub rejected_local: usize,
    pub noops: usize,
    pub undos: usize,
    pub envelopes: usize,
    pub deliveries: usize,
    pub duplicates: usize,
    pub buffered: usize,
    pub dropped_prims: u64,
    pub converged: bool,
    pub state_identical: bool,
    pub clocks_identical: bool,
    pub pending_left: usize,
    pub tombstones: usize,
    pub doc_bytes: usize,
    pub divergence: Option<String>,
    /// Every envelope produced, in creation order (for replaying in other orders).
    pub trace: Vec<Envelope>,
    pub final_doc: Value,
}

/// Run `cfg.clients` replicas that edit concurrently and exchange envelopes in random order, with
/// duplicates and reordering, then deliver everything and compare all replicas.
pub fn simulate(base: &Value, cfg: &SimConfig) -> Result<SimReport, String> {
    if cfg.clients == 0 || cfg.clients > 64 {
        return Err("clients must be 1..=64".into());
    }
    let mut rng = SimRng(cfg.seed);
    let mut reps: Vec<Replica> = (0..cfg.clients).map(|i| Replica::new(i as u32 + 1, base)).collect::<Result<_, _>>()?;
    let mut inbox: Vec<Vec<Envelope>> = vec![vec![]; cfg.clients];
    let mut rep = SimReport { clients: cfg.clients, ..Default::default() };
    let deliver_one = |reps: &mut Vec<Replica>, inbox: &mut Vec<Vec<Envelope>>, who: usize, rng: &mut SimRng, rep: &mut SimReport| {
        if inbox[who].is_empty() {
            return;
        }
        let k = rng.below(inbox[who].len());
        let e = inbox[who].swap_remove(k);
        rep.deliveries += 1;
        match reps[who].receive(e) {
            Ok(Delivery::Duplicate) => rep.duplicates += 1,
            Ok(Delivery::Buffered) => rep.buffered += 1,
            _ => {}
        }
    };
    let mut done = 0;
    let mut guard = 0;
    while done < cfg.ops && guard < cfg.ops * 20 {
        guard += 1;
        // random background delivery
        for _ in 0..rng.below(3) {
            let who = rng.below(cfg.clients);
            deliver_one(&mut reps, &mut inbox, who, &mut rng, &mut rep);
        }
        let c = rng.below(cfg.clients);
        let env = if rng.chance(cfg.undo_pct) {
            rep.undos += 1;
            if rng.chance(70) {
                reps[c].undo()?
            } else {
                reps[c].redo()?
            }
        } else {
            let cur = reps[c].json();
            let gen = |cur: &Value, rng: &mut SimRng| if cfg.project_aware { project_op(cur, rng) } else { random_op(cur, rng) };
            let Some(op) = gen(&cur, &mut rng) else { continue };
            let op = if rng.chance(15) {
                match gen(&cur, &mut rng) {
                    // indices of the second op were chosen against the same document: after a structural first op (insert /
                    // remove / move) they would hit other elements, so project mode batches plain value sets only
                    Some(o2) if !cfg.project_aware || (matches!(op, Op::Set { .. }) && matches!(o2, Op::Set { .. })) => Op::Batch { ops: vec![op, o2] },
                    _ => op,
                }
            } else {
                op
            };
            match reps[c].local_op(&op) {
                Ok(e) => e,
                Err(_) => {
                    rep.rejected_local += 1;
                    continue;
                }
            }
        };
        done += 1;
        rep.edits += 1;
        match env {
            None => rep.noops += 1,
            Some(e) => {
                rep.envelopes += 1;
                rep.trace.push(e.clone());
                for (j, q) in inbox.iter_mut().enumerate() {
                    if j != c {
                        q.push(e.clone());
                        if rng.chance(cfg.dup_pct) {
                            q.push(e.clone());
                        }
                    }
                }
            }
        }
    }
    for who in 0..cfg.clients {
        while !inbox[who].is_empty() {
            deliver_one(&mut reps, &mut inbox, who, &mut rng, &mut rep);
        }
    }
    let j0 = reps[0].json();
    rep.converged = reps.iter().all(|r| r.json() == j0);
    rep.state_identical = reps.iter().all(|r| r.doc == reps[0].doc);
    rep.clocks_identical = reps.iter().all(|r| r.applied == reps[0].applied);
    rep.pending_left = reps.iter().map(|r| r.pending.len()).sum();
    rep.dropped_prims = reps.iter().map(|r| r.dropped).sum();
    rep.tombstones = reps[0].tombstones();
    rep.doc_bytes = j0.to_string().len();
    if !rep.converged {
        let i = reps.iter().position(|r| r.json() != j0).unwrap_or(0);
        rep.divergence = Some(format!("replica {} differs from replica 1", i + 1));
    } else if !rep.state_identical {
        rep.divergence = Some("visible documents equal but internal state differs".into());
    }
    rep.final_doc = j0;
    Ok(rep)
}

// ---------------------------------------------------------------------------------------------
