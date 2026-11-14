// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Semantic diff and three-way merge of project documents (I-01, the engine behind `vcs/`).
//!
//! Pure functions on `serde_json::Value`; no files, no clock. Own design from textbook ideas:
//! * three-way merge of sequences: S. Khanna, K. Kunal, B. C. Pierce, "A formal investigation of
//!   diff3", FSTTCS 2007 (change chunks between synchronisation points that both sides keep);
//! * longest common subsequence by dynamic programming with common prefix/suffix trimming (the
//!   Hunt-McIlroy 1977 / Myers 1986 problem; plain O(n*m) table here, capped);
//! * tree diff by stable identities instead of positions (the same notion of identity as
//!   `collab/`: an object is the same object when `id`, `name` or `target`+`prop` match).
//!
//! # Model
//! A project is a JSON tree. Arrays whose elements all carry an identity are *keyed arrays*: an
//! element is identified by `id`, else `name`, else `target|prop|source` (channels), else `frame`
//! (keyframes). Duplicated identities get a `#2`, `#3`... suffix by occurrence. Keyed arrays are
//! merged per element (so two animators editing different layers, bones, channels or keys never
//! conflict) and their order is merged with diff3 on the identity lists. Arrays without identities
//! (anchor lists, `["thigh","shin"]`) are merged as sequences with diff3 on whole elements.
//! Objects are merged per key. Everything is canonicalised first (sorted keys, integral floats are
//! integers) so equal documents are equal values.
//!
//! # Conflicts (never silent)
//! `BothChanged` (same scalar changed differently), `DeleteEdit` (one side removed what the other
//! edited), `KeyRetimed` (same key moved to different frames), `OrderConflict` (both reordered the
//! same region), `ArrayConflict` (overlapping edits of an anonymous list). The merged value carries
//! the *ours* side for every conflict; a [`Conflict`] records base/ours/theirs.
//!
//! # Limits
//! A rename is a remove plus an add (the JSON file has no persistent ids); a retimed key edited
//! at the same time on the other side is a `DeleteEdit`; a value that two sides changed in
//! ways that are each valid but jointly meaningless (two different parents, say) merges cleanly;
//! validity of the merged project is the caller's business.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

// ---------------------------------------------------------------------------------------------
// Canonical form
// ---------------------------------------------------------------------------------------------

/// Canonical copy of `v`: integral floats become integers, `-0.0` becomes `0`. Object order is
/// handled at serialisation time by [`canon_string`].
pub fn canonicalize(v: &Value) -> Value {
    match v {
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                return v.clone();
            }
            match n.as_f64() {
                Some(f) if f.is_finite() && f.fract() == 0.0 && f.abs() < 9.0e15 => Value::from(f as i64),
                Some(0.0) => Value::from(0i64),
                _ => v.clone(),
            }
        }
        Value::Array(a) => Value::Array(a.iter().map(canonicalize).collect()),
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, x) in m {
                out.insert(k.clone(), canonicalize(x));
            }
            Value::Object(out)
        }
        _ => v.clone(),
    }
}

/// Deterministic JSON text: keys sorted bytewise, no insignificant whitespace (`pretty` = false) or
/// two-space indentation (`pretty` = true). Numbers use the shortest round-trip form.
pub fn canon_string(v: &Value, pretty: bool) -> String {
    let mut s = String::new();
    write_canon(v, pretty, 0, &mut s);
    if pretty {
        s.push('\n');
    }
    s
}

fn write_canon(v: &Value, pretty: bool, depth: usize, out: &mut String) {
    let nl = |out: &mut String, d: usize| {
        if pretty {
            out.push('\n');
            for _ in 0..d {
                out.push_str("  ");
            }
        }
    };
    match v {
        Value::Array(a) => {
            if a.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                nl(out, depth + 1);
                write_canon(x, pretty, depth + 1, out);
            }
            nl(out, depth);
            out.push(']');
        }
        Value::Object(m) => {
            if m.is_empty() {
                out.push_str("{}");
                return;
            }
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                nl(out, depth + 1);
                out.push_str(&serde_json::to_string(k).unwrap_or_default());
                out.push(':');
                if pretty {
                    out.push(' ');
                }
                write_canon(&m[*k], pretty, depth + 1, out);
            }
            nl(out, depth);
            out.push('}');
        }
        other => out.push_str(&serde_json::to_string(other).unwrap_or_default()),
    }
}

// ---------------------------------------------------------------------------------------------
// Identities
// ---------------------------------------------------------------------------------------------

fn scalar_text(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(_) => Some(canon_string(&canonicalize(v), false)),
        _ => None,
    }
}

/// Identity of one array element, if it has one.
pub fn elem_identity(v: &Value) -> Option<String> {
    let m = v.as_object()?;
    if let Some(i) = m.get("id").and_then(scalar_text) {
        return Some(i);
    }
    if let Some(Value::String(n)) = m.get("name") {
        return Some(n.clone());
    }
    if let (Some(Value::String(t)), Some(Value::String(p))) = (m.get("target"), m.get("prop")) {
        let src = m.get("source").and_then(|s| s.as_str()).unwrap_or("");
        return Some(format!("{t}|{p}|{src}"));
    }
    if let Some(f @ Value::Number(_)) = m.get("frame") {
        return scalar_text(f);
    }
    None
}

/// Identities of all elements (with `#n` for repeats), or `None` if any element has none.
pub fn elem_ids(arr: &[Value]) -> Option<Vec<String>> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut out = Vec::with_capacity(arr.len());
    for e in arr {
        let id = elem_identity(e)?;
        let n = seen.entry(id.clone()).or_insert(0);
        *n += 1;
        out.push(if *n == 1 { id } else { format!("{id}#{n}") });
    }
    Some(out)
}

/// Keyframe-like arrays: every element an object with a numeric `frame` and no other identity.
fn is_frame_keyed(arr: &[Value]) -> bool {
    !arr.is_empty()
        && arr.iter().all(|e| {
            e.as_object().is_some_and(|m| {
                m.get("frame").is_some_and(|f| f.is_number()) && !m.contains_key("id") && !m.contains_key("name") && !(m.contains_key("target") && m.contains_key("prop"))
            })
        })
}

fn frame_of(v: &Value) -> f64 {
    v.get("frame").and_then(|f| f.as_f64()).unwrap_or(0.0)
}

fn keyed(arr: &[Value]) -> Option<Vec<(String, Value)>> {
    let ids = elem_ids(arr)?;
    Some(ids.into_iter().zip(arr.iter().cloned()).collect())
}

// ---------------------------------------------------------------------------------------------
// Pointers into the tree (identity-aware)
// ---------------------------------------------------------------------------------------------

pub fn pointer_of(segs: &[String]) -> String {
    let mut s = String::new();
    for g in segs {
        s.push('/');
        s.push_str(&g.replace('~', "~0").replace('/', "~1"));
    }
    s
}

pub fn segs_of(pointer: &str) -> Result<Vec<String>, String> {
    if pointer.is_empty() {
        return Ok(vec![]);
    }
    if !pointer.starts_with('/') {
        return Err(format!("path '{pointer}' must start with '/'"));
    }
    Ok(pointer[1..].split('/').map(|g| g.replace("~1", "/").replace("~0", "~")).collect())
}

/// Value at an identity pointer (`/layers/sky/opacity`, `/channels/arm_up|bone_angle|keys/track/keys/12/value`).
pub fn get_at<'a>(root: &'a Value, segs: &[String]) -> Option<&'a Value> {
    let mut cur = root;
    for g in segs {
        cur = match cur {
            Value::Object(m) => m.get(g)?,
            Value::Array(a) => match elem_ids(a) {
                Some(ids) => &a[ids.iter().position(|i| i == g)?],
                None => a.get(g.parse::<usize>().ok()?)?,
            },
            _ => return None,
        };
    }
    Some(cur)
}

/// Set (`Some`) or remove (`None`) the value at an identity pointer. A missing keyed element is
/// appended (frame-keyed arrays are re-sorted by frame). Missing parents are an error.
pub fn set_at(root: &mut Value, segs: &[String], new: Option<Value>) -> Result<(), String> {
    let Some((last, parents)) = segs.split_last() else {
        return match new {
            Some(v) => {
                *root = v;
                Ok(())
            }
            None => Err("cannot remove the document root".into()),
        };
    };
    let mut cur = root;
    for g in parents {
        cur = match cur {
            Value::Object(m) => m.get_mut(g).ok_or_else(|| format!("no '{g}' on the path"))?,
            Value::Array(a) => {
                let idx = match elem_ids(a) {
                    Some(ids) => ids.iter().position(|i| i == g),
                    None => g.parse::<usize>().ok().filter(|i| *i < a.len()),
                };
                &mut a[idx.ok_or_else(|| format!("no element '{g}' on the path"))?]
            }
            _ => return Err(format!("'{g}' is inside a scalar")),
        };
    }
    match cur {
        Value::Object(m) => {
            match new {
                Some(v) => {
                    m.insert(last.clone(), v);
                }
                None => {
                    m.remove(last);
                }
            }
            Ok(())
        }
        Value::Array(a) => {
            let ids = elem_ids(a);
            let idx = match &ids {
                Some(ids) => ids.iter().position(|i| i == last),
                None => last.parse::<usize>().ok().filter(|i| *i < a.len()),
            };
            match (idx, new) {
                (Some(i), Some(v)) => a[i] = v,
                (Some(i), None) => {
                    a.remove(i);
                }
                (None, Some(v)) => {
                    if ids.is_none() && !a.is_empty() {
                        return Err(format!("no element '{last}'"));
                    }
                    a.push(v);
                    if is_frame_keyed(a) {
                        a.sort_by(|x, y| frame_of(x).partial_cmp(&frame_of(y)).unwrap_or(std::cmp::Ordering::Equal));
                    }
                }
                (None, None) => {}
            }
            Ok(())
        }
        _ => Err("parent is a scalar".into()),
    }
}

// ---------------------------------------------------------------------------------------------
// diff3 on sequences
// ---------------------------------------------------------------------------------------------

const LCS_CELL_CAP: usize = 16_000_000;

/// For each index of `a`, the matching index of `b` in one longest common subsequence.
fn lcs_match<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Option<usize>> {
    let mut res = vec![None; a.len()];
    let mut pre = 0;
    while pre < a.len() && pre < b.len() && a[pre] == b[pre] {
        res[pre] = Some(pre);
        pre += 1;
    }
    let mut suf = 0;
    while suf < a.len() - pre && suf < b.len() - pre && a[a.len() - 1 - suf] == b[b.len() - 1 - suf] {
        res[a.len() - 1 - suf] = Some(b.len() - 1 - suf);
        suf += 1;
    }
    let (am, bm) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    let (n, m) = (am.len(), bm.len());
    if n == 0 || m == 0 || (n + 1).saturating_mul(m + 1) > LCS_CELL_CAP {
        return res; // too large: only the trimmed ends match (coarser, still correct)
    }
    let w = m + 1;
    let mut t = vec![0u32; (n + 1) * w];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            t[i * w + j] = if am[i] == bm[j] { t[(i + 1) * w + j + 1] + 1 } else { t[(i + 1) * w + j].max(t[i * w + j + 1]) };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if am[i] == bm[j] {
            res[pre + i] = Some(pre + j);
            i += 1;
            j += 1;
        } else if t[(i + 1) * w + j] >= t[i * w + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    res
}

/// Three-way merge of sequences. Returns the merged sequence and the number of conflicting
/// chunks (each conflicting chunk contributes the `ours` items, then `theirs` items not already in it).
pub fn diff3<T: PartialEq + Clone>(base: &[T], ours: &[T], theirs: &[T]) -> (Vec<T>, usize) {
    let mo = lcs_match(base, ours);
    let mt = lcs_match(base, theirs);
    let (mut out, mut conflicts) = (Vec::new(), 0);
    let (mut ib, mut io, mut it) = (0usize, 0usize, 0usize);
    loop {
        let mut k = ib;
        let (mut po, mut pt) = (ours.len(), theirs.len());
        let mut at_end = true;
        while k < base.len() {
            if let (Some(a), Some(b)) = (mo[k], mt[k]) {
                po = a;
                pt = b;
                at_end = false;
                break;
            }
            k += 1;
        }
        if at_end {
            k = base.len();
        }
        let (bc, oc, tc) = (&base[ib..k], &ours[io..po.max(io)], &theirs[it..pt.max(it)]);
        if oc == tc || tc == bc {
            out.extend_from_slice(oc);
        } else if oc == bc {
            out.extend_from_slice(tc);
        } else {
            conflicts += 1;
            out.extend_from_slice(oc);
            for x in tc {
                if !oc.contains(x) {
                    out.push(x.clone());
                }
            }
        }
        if at_end {
            break;
        }
        out.push(base[k].clone());
        ib = k + 1;
        io = po + 1;
        it = pt + 1;
    }
    (out, conflicts)
}

// ---------------------------------------------------------------------------------------------
// Three-way merge
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Slot {
    Absent,
    Value(Value),
}

impl Slot {
    pub fn of(v: Option<&Value>) -> Slot {
        v.map_or(Slot::Absent, |x| Slot::Value(x.clone()))
    }
    pub fn into_option(self) -> Option<Value> {
        match self {
            Slot::Absent => None,
            Slot::Value(v) => Some(v),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    BothChanged,
    DeleteEdit,
    KeyRetimed,
    OrderConflict,
    ArrayConflict,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// Identity pointer (see [`pointer_of`]).
    pub path: String,
    pub kind: ConflictKind,
    pub description: String,
    pub base: Slot,
    pub ours: Slot,
    pub theirs: Slot,
}

fn short(v: &Slot) -> String {
    match v {
        Slot::Absent => "(absent)".into(),
        Slot::Value(v) => {
            let s = canon_string(v, false);
            if s.chars().count() > 48 {
                format!("{}...", s.chars().take(45).collect::<String>())
            } else {
                s
            }
        }
    }
}

fn push_conflict(cs: &mut Vec<Conflict>, path: &[String], kind: ConflictKind, b: Option<&Value>, o: Option<&Value>, t: Option<&Value>) {
    let (base, ours, theirs) = (Slot::of(b), Slot::of(o), Slot::of(t));
    let what = describe_path(path);
    let description = match kind {
        ConflictKind::BothChanged => format!("{what}: ours {} vs theirs {} (base {})", short(&ours), short(&theirs), short(&base)),
        ConflictKind::DeleteEdit => {
            if ours == Slot::Absent {
                format!("{what}: removed on our side, edited on theirs")
            } else {
                format!("{what}: edited on our side, removed on theirs")
            }
        }
        ConflictKind::KeyRetimed => format!("{what}: moved to different frames on the two sides"),
        ConflictKind::OrderConflict => format!("{what}: both sides reordered the same region"),
        ConflictKind::ArrayConflict => format!("{what}: overlapping edits of the same list"),
    };
    cs.push(Conflict { path: pointer_of(path), kind, description, base, ours, theirs });
}

/// Three-way merge of whole documents. The result carries `ours` at every conflict.
pub fn merge_docs(base: &Value, ours: &Value, theirs: &Value) -> (Value, Vec<Conflict>) {
    let (b, o, t) = (canonicalize(base), canonicalize(ours), canonicalize(theirs));
    let mut cs = Vec::new();
    let mut path = Vec::new();
    let v = merge3(&mut path, Some(&b), Some(&o), Some(&t), &mut cs).unwrap_or(Value::Null);
    (v, cs)
}

fn merge3(path: &mut Vec<String>, base: Option<&Value>, ours: Option<&Value>, theirs: Option<&Value>, cs: &mut Vec<Conflict>) -> Option<Value> {
    if ours == theirs || base == theirs {
        return ours.cloned();
    }
    if base == ours {
        return theirs.cloned();
    }
    match (ours, theirs) {
        (Some(Value::Object(o)), Some(Value::Object(t))) => {
            let empty = Map::new();
            let b = match base {
                Some(Value::Object(b)) => b,
                _ => &empty,
            };
            let keys: BTreeSet<&String> = o.keys().chain(t.keys()).chain(b.keys()).collect();
            let mut out = Map::new();
            for k in keys {
                path.push(k.clone());
                let r = merge3(path, b.get(k), o.get(k), t.get(k), cs);
                path.pop();
                if let Some(v) = r {
                    out.insert(k.clone(), v);
                }
            }
            Some(Value::Object(out))
        }
        (Some(Value::Array(o)), Some(Value::Array(t))) => {
            let empty = Vec::new();
            let b = match base {
                Some(Value::Array(b)) => b,
                _ => &empty,
            };
            Some(merge_arrays(path, b, o, t, cs))
        }
        _ => {
            let kind = if ours.is_none() || theirs.is_none() { ConflictKind::DeleteEdit } else { ConflictKind::BothChanged };
            push_conflict(cs, path, kind, base, ours, theirs);
            ours.cloned()
        }
    }
}

fn merge_arrays(path: &mut Vec<String>, base: &[Value], ours: &[Value], theirs: &[Value], cs: &mut Vec<Conflict>) -> Value {
    let (Some(kb), Some(ko), Some(kt)) = (keyed(base), keyed(ours), keyed(theirs)) else {
        let (m, n) = diff3(base, ours, theirs);
        if n == 0 {
            return Value::Array(m);
        }
        push_conflict(cs, path, ConflictKind::ArrayConflict, Some(&Value::Array(base.to_vec())), Some(&Value::Array(ours.to_vec())), Some(&Value::Array(theirs.to_vec())));
        return Value::Array(ours.to_vec());
    };
    let frame_keyed = is_frame_keyed(ours) || is_frame_keyed(theirs) || is_frame_keyed(base);
    let mut kt = kt;
    if frame_keyed {
        // the same base key retimed to two different frames: conflict; undo theirs' retime so the rest merges
        let (mo, mt) = (retimes(&kb, &ko), retimes(&kb, &kt));
        for (f, go) in &mo {
            if let Some(gt) = mt.get(f) {
                if gt != go {
                    let oel = ko.iter().find(|(i, _)| i == go).map(|x| &x.1);
                    let tel = kt.iter().find(|(i, _)| i == gt).map(|x| &x.1);
                    let bel = kb.iter().find(|(i, _)| i == f).map(|x| &x.1);
                    path.push(f.clone());
                    push_conflict(cs, path, ConflictKind::KeyRetimed, bel, oel, tel);
                    path.pop();
                    if let (Some(p), Some(b)) = (kt.iter().position(|(i, _)| i == gt), bel) {
                        kt[p] = (f.clone(), b.clone());
                    }
                }
            }
        }
    }
    let ids = |k: &[(String, Value)]| k.iter().map(|x| x.0.clone()).collect::<Vec<_>>();
    let (ib, io, it) = (ids(&kb), ids(&ko), ids(&kt));
    let (order, nconf) = diff3(&ib, &io, &it);
    if nconf > 0 && !frame_keyed {
        push_conflict(cs, path, ConflictKind::OrderConflict, None, None, None);
    }
    let find = |k: &[(String, Value)], id: &str| k.iter().find(|x| x.0 == id).map(|x| x.1.clone());
    let mut out: Vec<(String, Value)> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for id in &order {
        if !seen.insert(id.clone()) {
            continue;
        }
        let (b, o, t) = (find(&kb, id), find(&ko, id), find(&kt, id));
        path.push(id.clone());
        let r = merge3(path, b.as_ref(), o.as_ref(), t.as_ref(), cs);
        path.pop();
        if let Some(v) = r {
            out.push((id.clone(), v));
        }
    }
    // elements the order merge dropped although one side still has them edited: delete-vs-edit
    for (id, bv) in &kb {
        if seen.contains(id) {
            continue;
        }
        let (o, t) = (find(&ko, id), find(&kt, id));
        let edited = |x: &Option<Value>| x.as_ref().is_some_and(|v| v != bv);
        if edited(&o) || edited(&t) {
            path.push(id.clone());
            push_conflict(cs, path, ConflictKind::DeleteEdit, Some(bv), o.as_ref(), t.as_ref());
            path.pop();
            if let Some(v) = o {
                let pos = io.iter().position(|i| i == id).unwrap_or(out.len()).min(out.len());
                out.insert(pos, (id.clone(), v));
            }
        }
    }
    let mut vals: Vec<Value> = out.into_iter().map(|x| x.1).collect();
    if frame_keyed {
        vals.sort_by(|a, b| frame_of(a).partial_cmp(&frame_of(b)).unwrap_or(std::cmp::Ordering::Equal));
    }
    Value::Array(vals)
}

fn strip_frame(v: &Value) -> Value {
    let mut c = v.clone();
    if let Some(m) = c.as_object_mut() {
        m.remove("frame");
    }
    c
}

/// Keys removed from `base` and re-added at another frame with identical content: base id -> new id.
fn retimes(base: &[(String, Value)], side: &[(String, Value)]) -> BTreeMap<String, String> {
    let mut res = BTreeMap::new();
    let mut used: BTreeSet<&str> = BTreeSet::new();
    for (f, bv) in base.iter().filter(|(i, _)| !side.iter().any(|s| &s.0 == i)) {
        let want = strip_frame(bv);
        if let Some((g, _)) = side.iter().find(|(i, v)| !base.iter().any(|b| &b.0 == i) && !used.contains(i.as_str()) && strip_frame(v) == want) {
            used.insert(g);
            res.insert(f.clone(), g.clone());
        }
    }
    res
}

// ---------------------------------------------------------------------------------------------
// Applying a conflict resolution
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Choice {
    Ours,
    Theirs,
    Value(Value),
}

/// Apply the resolution of `c` to the merged document `doc`.
pub fn resolve_in(doc: &mut Value, c: &Conflict, choice: &Choice) -> Result<(), String> {
    let segs = segs_of(&c.path)?;
    let pick: Option<Value> = match choice {
        Choice::Ours => c.ours.clone().into_option(),
        Choice::Theirs => c.theirs.clone().into_option(),
        Choice::Value(v) => Some(canonicalize(v)),
    };
    match c.kind {
        ConflictKind::OrderConflict => match choice {
            Choice::Value(_) => Err("an order conflict is resolved with --ours or --theirs only after reordering by hand; use a value-less choice".into()),
            _ => Ok(()), // the merged order already holds ours first; nothing to rewrite
        },
        ConflictKind::KeyRetimed => {
            let (parent, f) = segs.split_at(segs.len().saturating_sub(1));
            let f = f.first().ok_or("bad path")?.clone();
            let mut drop_ids = vec![f];
            for s in [&c.ours, &c.theirs] {
                if let Slot::Value(v) = s {
                    if let Some(i) = elem_identity(v) {
                        drop_ids.push(i);
                    }
                }
            }
            for id in drop_ids {
                let mut p = parent.to_vec();
                p.push(id);
                set_at(doc, &p, None)?;
            }
            if let Some(v) = pick {
                let mut p = parent.to_vec();
                p.push(elem_identity(&v).ok_or("chosen key has no frame")?);
                set_at(doc, &p, Some(v))?;
            }
            Ok(())
        }
        _ => set_at(doc, &segs, pick),
    }
}

// ---------------------------------------------------------------------------------------------
// Semantic diff
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
    Moved,
    Retimed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub path: String,
    pub description: String,
}

fn singular(top: &str) -> &str {
    match top {
        "layers" => "layer",
        "bones" => "bone",
        "channels" => "channel",
        "ik" => "IK chain",
        "constraints" => "constraint",
        "springs" => "spring",
        "markers" => "marker",
        "comps" => "comp",
        "events" => "event",
        "sounds" => "sound",
        "skins" => "skin",
        "shots" => "camera shot",
        "clips" => "clip",
        "machines" => "state machine",
        "bindings" => "binding",
        "containers" => "container",
        "paths" => "path",
        "anchors" => "anchor",
        other => other,
    }
}

fn nice_id(top: &str, id: &str) -> String {
    if top == "channels" {
        let p: Vec<&str> = id.split('|').collect();
        if p.len() == 3 {
            let t = if p[0].is_empty() { "camera" } else { p[0] };
            return if p[2] == "keys" || p[2].is_empty() { format!("channel {t}.{}", p[1]) } else { format!("channel {t}.{} ({})", p[1], p[2]) };
        }
    }
    format!("{} '{}'", singular(top), id)
}

/// (subject, attribute path) in words for an identity pointer.
fn subject(segs: &[String]) -> (String, Option<String>) {
    if segs.is_empty() {
        return ("project".into(), None);
    }
    let top = &segs[0];
    if segs.len() == 1 {
        return ("project".into(), Some(top.clone()));
    }
    // top-level non-array objects (camera, ...) read as attributes of the project
    let is_el = matches!(top.as_str(), "layers" | "bones" | "channels" | "ik" | "constraints" | "springs" | "markers" | "comps" | "events" | "sounds" | "skins" | "shots" | "clips" | "machines" | "bindings" | "containers" | "bone_vis" | "channel_meta" | "mesh_locks" | "glues");
    if !is_el {
        return ("project".into(), Some(segs.join(".")));
    }
    let mut subj = nice_id(top, &segs[1]);
    let mut rest: &[String] = &segs[2..];
    if rest.len() >= 3 && rest[0] == "track" && rest[1] == "keys" {
        subj = format!("key at frame {} of {}", rest[2], subj);
        rest = &rest[3..];
    }
    (subj, if rest.is_empty() { None } else { Some(rest.join(".")) })
}

fn describe_path(segs: &[String]) -> String {
    match subject(segs) {
        (s, None) => s,
        (s, Some(a)) => format!("{s}: {a}"),
    }
}

fn val_text(v: &Value) -> String {
    let s = canon_string(v, false);
    if s.chars().count() > 60 {
        format!("{}...", s.chars().take(57).collect::<String>())
    } else {
        s
    }
}

fn change(out: &mut Vec<Change>, kind: ChangeKind, path: &[String], text: impl FnOnce(&str, Option<&str>) -> String) {
    let (s, a) = subject(path);
    out.push(Change { kind, path: pointer_of(path), description: text(&s, a.as_deref()) });
}

/// Semantic differences from `a` to `b`, in a stable order (document order of `b`/`a`, keys sorted).
pub fn diff_docs(a: &Value, b: &Value) -> Vec<Change> {
    let (a, b) = (canonicalize(a), canonicalize(b));
    let mut out = Vec::new();
    diff_value(&mut Vec::new(), Some(&a), Some(&b), &mut out);
    out
}

fn diff_value(path: &mut Vec<String>, a: Option<&Value>, b: Option<&Value>, out: &mut Vec<Change>) {
    if a == b {
        return;
    }
    match (a, b) {
        (None, Some(v)) => change(out, ChangeKind::Added, path, |s, at| match at {
            Some(at) => format!("{s}: {at} added ({})", val_text(v)),
            None => format!("{s} added"),
        }),
        (Some(_), None) => change(out, ChangeKind::Removed, path, |s, at| match at {
            Some(at) => format!("{s}: {at} removed"),
            None => format!("{s} removed"),
        }),
        (Some(Value::Object(x)), Some(Value::Object(y))) => {
            let keys: BTreeSet<&String> = x.keys().chain(y.keys()).collect();
            for k in keys {
                path.push(k.clone());
                diff_value(path, x.get(k), y.get(k), out);
                path.pop();
            }
        }
        (Some(Value::Array(x)), Some(Value::Array(y))) => {
            if let (Some(kx), Some(ky)) = (keyed(x), keyed(y)) {
                diff_keyed(path, &kx, &ky, is_frame_keyed(x) || is_frame_keyed(y), out);
            } else if x.len() == y.len() {
                for i in 0..x.len() {
                    path.push(i.to_string());
                    diff_value(path, Some(&x[i]), Some(&y[i]), out);
                    path.pop();
                }
            } else {
                let (n, m) = (x.len(), y.len());
                change(out, ChangeKind::Changed, path, |s, at| format!("{}list changed ({n} -> {m} items)", at.map_or(format!("{s}: "), |a| format!("{s}: {a} "))));
            }
        }
        (Some(x), Some(y)) => change(out, ChangeKind::Changed, path, |s, at| match at {
            Some(at) => format!("{s}: {at} changed {} -> {}", val_text(x), val_text(y)),
            None => format!("{s} changed {} -> {}", val_text(x), val_text(y)),
        }),
        (None, None) => {}
    }
}

fn diff_keyed(path: &mut Vec<String>, a: &[(String, Value)], b: &[(String, Value)], frame_keyed: bool, out: &mut Vec<Change>) {
    let in_a = |id: &str| a.iter().any(|x| x.0 == id);
    let in_b = |id: &str| b.iter().any(|x| x.0 == id);
    let mut retimed: BTreeMap<String, String> = BTreeMap::new();
    if frame_keyed {
        retimed = retimes(a, b);
    }
    let moved_to: BTreeSet<&String> = retimed.values().collect();
    for (id, _) in a.iter().filter(|(i, _)| !in_b(i)) {
        path.push(id.clone());
        if let Some(g) = retimed.get(id) {
            let g = g.clone();
            change(out, ChangeKind::Retimed, path, |s, _| format!("{s} retimed to frame {g}"));
        } else {
            change(out, ChangeKind::Removed, path, |s, at| match at {
                Some(at) => format!("{s}: {at} removed"),
                None => format!("{s} removed"),
            });
        }
        path.pop();
    }
    for (id, v) in b.iter().filter(|(i, _)| !in_a(i)) {
        if moved_to.contains(id) {
            continue;
        }
        path.push(id.clone());
        change(out, ChangeKind::Added, path, |s, at| match at {
            Some(at) => format!("{s}: {at} added"),
            None => {
                if frame_keyed {
                    let val = v.get("value").map(val_text).unwrap_or_default();
                    format!("{s} added{}", if val.is_empty() { String::new() } else { format!(" (value {val})") })
                } else {
                    format!("{s} added")
                }
            }
        });
        path.pop();
    }
    for (id, av) in a {
        if let Some((_, bv)) = b.iter().find(|x| &x.0 == id) {
            path.push(id.clone());
            diff_value(path, Some(av), Some(bv), out);
            path.pop();
        }
    }
    if !frame_keyed {
        let ca: Vec<&String> = a.iter().map(|x| &x.0).filter(|i| in_b(i)).collect();
        let cb: Vec<&String> = b.iter().map(|x| &x.0).filter(|i| in_a(i)).collect();
        let m = lcs_match(&ca, &cb);
        let kept: BTreeSet<usize> = m.iter().flatten().copied().collect();
        for (j, id) in cb.iter().enumerate() {
            if !kept.contains(&j) {
                let from = a.iter().position(|x| &&x.0 == id).unwrap_or(0);
                let to = b.iter().position(|x| &&x.0 == id).unwrap_or(0);
                path.push((*id).clone());
                change(out, ChangeKind::Moved, path, |s, _| format!("{s} moved from position {from} to {to}"));
                path.pop();
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn doc() -> Value {
        json!({
            "name": "t", "fps": 24.0, "frames": 48,
            "layers": [
                {"name": "sky", "opacity": 1.0, "visible": true},
                {"name": "sun", "opacity": 1.0, "visible": true},
                {"name": "hill", "opacity": 0.5, "visible": true}
            ],
            "bones": [{"name": "a", "length": 10.0}],
            "channels": [
                {"target": "a", "prop": "bone_angle", "source": "keys", "track": {"keys": [
                    {"frame": 0, "value": 0.0}, {"frame": 12, "value": 10.0}, {"frame": 24, "value": 0.0}]}},
                {"target": "sun", "prop": "layer_opacity", "source": "keys", "track": {"keys": [{"frame": 0, "value": 1.0}]}}
            ]
        })
    }

    fn set(d: &mut Value, p: &str, v: Value) {
        set_at(d, &segs_of(p).unwrap(), Some(v)).unwrap();
    }

    #[test]
    fn canonical_is_order_and_float_independent() {
        let a: Value = serde_json::from_str(r#"{"b":1.0,"a":[2.0,{"y":0.5,"x":-0.0}]}"#).unwrap();
        let b: Value = serde_json::from_str(r#"{"a":[2,{"x":0,"y":0.5}],"b":1}"#).unwrap();
        assert_eq!(canon_string(&canonicalize(&a), false), canon_string(&canonicalize(&b), false));
        assert_eq!(canon_string(&canonicalize(&a), false), r#"{"a":[2,{"x":0,"y":0.5}],"b":1}"#);
        assert!(canon_string(&canonicalize(&a), true).contains("\n  \"a\": ["));
    }

    #[test]
    fn identities_and_duplicates() {
        let arr = vec![json!({"name": "x"}), json!({"name": "x"}), json!({"target": "t", "prop": "p"})];
        assert_eq!(elem_ids(&arr).unwrap(), vec!["x", "x#2", "t|p|"]);
        assert!(elem_ids(&[json!(1)]).is_none());
        assert!(elem_ids(&[json!({"pos": [1, 2]})]).is_none());
    }

    #[test]
    fn pointer_roundtrip_and_access() {
        let d = canonicalize(&doc());
        let segs = segs_of("/channels/a|bone_angle|keys/track/keys/12/value").unwrap();
        assert_eq!(get_at(&d, &segs), Some(&json!(10)));
        assert_eq!(pointer_of(&segs_of("/a~1b/c~0").unwrap()), "/a~1b/c~0");
        assert!(segs_of("nope").is_err());
        let mut e = d.clone();
        assert!(set_at(&mut e, &segs_of("/layers/nope/x").unwrap(), Some(json!(1))).is_err());
        set_at(&mut e, &segs_of("/layers/sun").unwrap(), None).unwrap();
        assert_eq!(e["layers"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn diff3_cases() {
        let b = vec![1, 2, 3, 4, 5];
        assert_eq!(diff3(&b, &[1, 2, 3, 4, 5, 6], &[0, 1, 2, 3, 4, 5]), (vec![0, 1, 2, 3, 4, 5, 6], 0));
        assert_eq!(diff3(&b, &[1, 3, 4, 5], &[1, 2, 3, 4]).0, vec![1, 3, 4]);
        let (m, n) = diff3(&b, &[1, 9, 3, 4, 5], &[1, 8, 3, 4, 5]);
        assert_eq!(n, 1);
        assert_eq!(m, vec![1, 9, 8, 3, 4, 5]);
        assert_eq!(diff3::<i32>(&[], &[], &[]), (vec![], 0));
        assert_eq!(diff3(&[], &[1], &[1]), (vec![1], 0));
    }

    #[test]
    fn merge_disjoint_layers_and_same_key_different_channels() {
        let base = doc();
        let mut ours = base.clone();
        set(&mut ours, "/layers/sky/opacity", json!(0.5));
        let mut theirs = base.clone();
        set(&mut theirs, "/layers/hill/visible", json!(false));
        set(&mut theirs, "/channels/sun|layer_opacity|keys/track/keys/0/value", json!(0.25));
        set(&mut ours, "/channels/a|bone_angle|keys/track/keys/12/value", json!(15));
        let (m, cs) = merge_docs(&base, &ours, &theirs);
        assert!(cs.is_empty(), "{cs:?}");
        assert_eq!(m["layers"][0]["opacity"], json!(0.5));
        assert_eq!(m["layers"][2]["visible"], json!(false));
        assert_eq!(m["channels"][0]["track"]["keys"][1]["value"], json!(15));
        assert_eq!(m["channels"][1]["track"]["keys"][0]["value"], json!(0.25));
    }

    #[test]
    fn merge_adds_removes_and_key_insert() {
        let base = canonicalize(&doc());
        let mut ours = base.clone();
        set(&mut ours, "/channels/a|bone_angle|keys/track/keys/18", json!({"frame": 18, "value": 5}));
        let mut theirs = base.clone();
        set(&mut theirs, "/layers/moon", json!({"name": "moon", "opacity": 1}));
        set_at(&mut theirs, &segs_of("/layers/sun").unwrap(), None).unwrap();
        let (m, cs) = merge_docs(&base, &ours, &theirs);
        assert!(cs.is_empty(), "{cs:?}");
        let names: Vec<_> = m["layers"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap()).collect();
        assert_eq!(names, vec!["sky", "hill", "moon"]);
        let frames: Vec<_> = m["channels"][0]["track"]["keys"].as_array().unwrap().iter().map(|k| k["frame"].as_i64().unwrap()).collect();
        assert_eq!(frames, vec![0, 12, 18, 24]);
    }

    #[test]
    fn conflict_both_changed_scalar() {
        let base = doc();
        let (mut o, mut t) = (base.clone(), base.clone());
        set(&mut o, "/layers/sky/opacity", json!(0.2));
        set(&mut t, "/layers/sky/opacity", json!(0.8));
        let (m, cs) = merge_docs(&base, &o, &t);
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].kind, ConflictKind::BothChanged);
        assert_eq!(cs[0].path, "/layers/sky/opacity");
        assert_eq!(m["layers"][0]["opacity"], json!(0.2));
        // same change on both sides is not a conflict
        set(&mut t, "/layers/sky/opacity", json!(0.2));
        assert!(merge_docs(&base, &o, &t).1.is_empty());
    }

    #[test]
    fn conflict_delete_vs_edit_both_directions() {
        let base = doc();
        let (mut o, mut t) = (base.clone(), base.clone());
        set_at(&mut o, &segs_of("/layers/sun").unwrap(), None).unwrap();
        set(&mut t, "/layers/sun/opacity", json!(0.3));
        let (m, cs) = merge_docs(&base, &o, &t);
        assert_eq!(cs.len(), 1, "{cs:?}");
        assert_eq!(cs[0].kind, ConflictKind::DeleteEdit);
        assert_eq!(cs[0].ours, Slot::Absent);
        assert_eq!(m["layers"].as_array().unwrap().len(), 2);
        // reverse: ours edits, theirs deletes -> ours (the edited element) is kept in the result
        let (m2, cs2) = merge_docs(&base, &t, &o);
        assert_eq!(cs2.len(), 1);
        assert_eq!(cs2[0].kind, ConflictKind::DeleteEdit);
        assert_eq!(m2["layers"].as_array().unwrap().len(), 3);
        assert_eq!(m2["layers"][1]["opacity"], json!(0.3));
        // delete vs untouched is clean
        let (_, cs3) = merge_docs(&base, &o, &base);
        assert!(cs3.is_empty());
    }

    #[test]
    fn conflict_same_key_retimed_and_clean_single_retime() {
        let base = canonicalize(&doc());
        let key = "/channels/a|bone_angle|keys/track/keys";
        let retime = |frame: i64| {
            let mut d = base.clone();
            set_at(&mut d, &segs_of(&format!("{key}/12")).unwrap(), None).unwrap();
            set(&mut d, &format!("{key}/{frame}"), json!({"frame": frame, "value": 10}));
            d
        };
        let (o, t) = (retime(14), retime(15));
        let (m, cs) = merge_docs(&base, &o, &t);
        assert_eq!(cs.len(), 1, "{cs:?}");
        assert_eq!(cs[0].kind, ConflictKind::KeyRetimed);
        let frames: Vec<_> = m["channels"][0]["track"]["keys"].as_array().unwrap().iter().map(|k| k["frame"].as_i64().unwrap()).collect();
        assert_eq!(frames, vec![0, 14, 24]);
        // one side retimes, the other untouched: clean
        let (m2, cs2) = merge_docs(&base, &o, &base);
        assert!(cs2.is_empty());
        assert_eq!(m2, o);
        // resolve --theirs
        let mut d = m;
        resolve_in(&mut d, &cs[0], &Choice::Theirs).unwrap();
        let frames: Vec<_> = d["channels"][0]["track"]["keys"].as_array().unwrap().iter().map(|k| k["frame"].as_i64().unwrap()).collect();
        assert_eq!(frames, vec![0, 15, 24]);
    }

    #[test]
    fn order_merge_and_conflict() {
        let base = doc();
        let reorder = |names: [&str; 3]| {
            let mut d = base.clone();
            let ls: Vec<Value> = names.iter().map(|n| base["layers"].as_array().unwrap().iter().find(|l| l["name"] == *n).unwrap().clone()).collect();
            d["layers"] = Value::Array(ls);
            d
        };
        // ours moves hill to front, theirs edits sun: clean
        let o = reorder(["hill", "sky", "sun"]);
        let mut t = base.clone();
        set(&mut t, "/layers/sun/opacity", json!(0.1));
        let (m, cs) = merge_docs(&base, &o, &t);
        assert!(cs.is_empty(), "{cs:?}");
        assert_eq!(m["layers"][0]["name"], json!("hill"));
        assert_eq!(m["layers"][2]["opacity"], json!(0.1));
        // both move the same region differently
        let t2 = reorder(["sun", "hill", "sky"]);
        let (_, cs) = merge_docs(&base, &o, &t2);
        assert!(cs.iter().any(|c| c.kind == ConflictKind::OrderConflict), "{cs:?}");
    }

    #[test]
    fn anonymous_list_overlap_conflicts_but_disjoint_merges() {
        let base = json!({"p": {"anchors": [[0, 0], [1, 1], [2, 2], [3, 3]]}});
        let (mut o, mut t) = (base.clone(), base.clone());
        o["p"]["anchors"][0] = json!([9, 9]);
        t["p"]["anchors"][3] = json!([8, 8]);
        let (m, cs) = merge_docs(&base, &o, &t);
        assert!(cs.is_empty());
        assert_eq!(m["p"]["anchors"], json!([[9, 9], [1, 1], [2, 2], [8, 8]]));
        t["p"]["anchors"][0] = json!([7, 7]);
        let (_, cs) = merge_docs(&base, &o, &t);
        assert_eq!(cs[0].kind, ConflictKind::ArrayConflict);
    }

    #[test]
    fn diff_descriptions() {
        let a = canonicalize(&doc());
        let mut b = a.clone();
        set(&mut b, "/channels/a|bone_angle|keys/track/keys/12/value", json!(15));
        set(&mut b, "/layers/sky/opacity", json!(0.5));
        set(&mut b, "/fps", json!(30));
        set(&mut b, "/channels/new|layer_x|keys", json!({"target": "new", "prop": "layer_x", "source": "keys"}));
        set_at(&mut b, &segs_of("/bones/a").unwrap(), None).unwrap();
        let text: Vec<String> = diff_docs(&a, &b).into_iter().map(|c| c.description).collect();
        let all = text.join("\n");
        assert!(all.contains("key at frame 12 of channel a.bone_angle: value changed 10 -> 15"), "{all}");
        assert!(all.contains("layer 'sky': opacity changed 1 -> 0.5"), "{all}");
        assert!(all.contains("project: fps changed 24 -> 30"), "{all}");
        assert!(all.contains("channel new.layer_x added"), "{all}");
        assert!(all.contains("bone 'a' removed"), "{all}");
        assert!(diff_docs(&a, &a).is_empty());
    }

    #[test]
    fn diff_move_and_retime() {
        let a = canonicalize(&doc());
        let mut b = a.clone();
        let l = b["layers"].as_array().unwrap().clone();
        b["layers"] = json!([l[2], l[0], l[1]]);
        let d = diff_docs(&a, &b);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].kind, ChangeKind::Moved);
        assert!(d[0].description.contains("layer 'hill' moved from position 2 to 0"), "{}", d[0].description);
        let key = "/channels/a|bone_angle|keys/track/keys";
        let mut c = a.clone();
        set_at(&mut c, &segs_of(&format!("{key}/12")).unwrap(), None).unwrap();
        set(&mut c, &format!("{key}/13"), json!({"frame": 13, "value": 10}));
        let d = diff_docs(&a, &c);
        assert_eq!(d.len(), 1, "{d:?}");
        assert_eq!(d[0].kind, ChangeKind::Retimed);
        assert!(d[0].description.contains("key at frame 12 of channel a.bone_angle retimed to frame 13"));
    }

    #[test]
    fn merge_handles_odd_shapes_without_panic() {
        let cases = [json!(null), json!(3), json!("s"), json!([1, [2]]), json!({"a": [null, {"name": 1}]}), json!({"layers": "x"})];
        for a in &cases {
            for b in &cases {
                for c in &cases {
                    let _ = merge_docs(a, b, c);
                    let _ = diff_docs(a, b);
                }
            }
        }
    }
}
