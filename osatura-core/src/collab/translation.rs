// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `collab` (see the module docs in `mod.rs`).
use super::*;

// ---------------------------------------------------------------------------------------------
// local translation: pointer ops / whole-document edits -> primitives

pub(super) struct Tx {
    pub(super) meta: Meta,
    pub(super) idx: u32,
    pub(super) gen: IdGen,
    pub(super) prims: Vec<Prim>,
    pub(super) undos: Vec<Undo>,
}

pub(super) fn emit(doc: &mut Doc, tx: &mut Tx, prim: Prim) -> Result<(), String> {
    if tx.prims.len() >= MAX_PRIMS {
        return Err("edit is too large".into());
    }
    let u = doc.apply_prim(&tx.meta, tx.idx, &mut tx.gen, &prim, true)?;
    tx.idx += 1;
    tx.undos.extend(u);
    tx.prims.push(prim);
    Ok(())
}

pub(super) fn unescape(tok: &str) -> String {
    tok.replace("~1", "/").replace("~0", "~")
}

pub(super) fn tokens(path: &str) -> Result<Vec<String>, String> {
    if path.is_empty() {
        return Ok(vec![]);
    }
    if !path.starts_with('/') {
        return Err(format!("bad JSON pointer '{path}'"));
    }
    Ok(path[1..].split('/').map(unescape).collect())
}

pub(super) fn ptr_of(toks: &[String]) -> String {
    toks.iter().map(|t| format!("/{}", t.replace('~', "~0").replace('/', "~1"))).collect()
}

pub(super) fn child_of(doc: &Doc, cur: &Val, tok: &str) -> Option<Val> {
    let Val::Ref(id) = cur else { return None };
    match doc.nodes.get(id)? {
        Cont::Obj(m) => m.get(tok).filter(|r| r.val != Val::Absent).map(|r| r.val.clone()),
        Cont::Arr(_) => {
            let i: usize = tok.parse().ok()?;
            doc.visible(*id).get(i).map(|e| e.reg.val.clone())
        }
    }
}

pub(super) fn walk(doc: &Doc, toks: &[String]) -> Result<Val, String> {
    let mut cur = doc.root.val.clone();
    for (n, t) in toks.iter().enumerate() {
        cur = child_of(doc, &cur, t).ok_or_else(|| format!("no such path '{}'", ptr_of(&toks[..=n])))?;
    }
    Ok(cur)
}

pub(super) fn slot_prim(slot: &Slot, value: &Value) -> Prim {
    match slot {
        Slot::Root => Prim::SetRoot { value: value.clone() },
        Slot::Key(o, k) => Prim::SetKey { obj: *o, key: k.clone(), value: value.clone() },
        Slot::Elem(a, e) => Prim::SetElem { arr: *a, elem: *e, value: value.clone() },
    }
}

/// Same kind of thing? Objects that carry a different `id` / `name` / `target` / `prop` are different
/// entities: replacing one by the other is a remove plus an insert, not a field-by-field rewrite.
pub(super) fn same_identity(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => ["id", "name", "target", "prop"].iter().all(|k| match (x.get(*k), y.get(*k)) {
            (Some(p), Some(q)) => p == q,
            _ => true,
        }),
        (Value::Array(_), Value::Array(_)) => true,
        (Value::Object(_), _) | (_, Value::Object(_)) | (Value::Array(_), _) | (_, Value::Array(_)) => false,
        _ => true,
    }
}

pub(super) fn diff_slot(doc: &mut Doc, tx: &mut Tx, slot: Slot, new: &Value) -> Result<(), String> {
    let cur = doc.slot_val(&slot)?;
    match (&cur, new) {
        (Val::Ref(id), Value::Object(nm)) if doc.obj(*id).is_some() => {
            let id = *id;
            let existing: BTreeSet<String> = doc.obj(id).map(|m| m.iter().filter(|(_, r)| r.val != Val::Absent).map(|(k, _)| k.clone()).collect()).unwrap_or_default();
            for k in &existing {
                if !nm.contains_key(k) {
                    emit(doc, tx, Prim::DelKey { obj: id, key: k.clone() })?;
                }
            }
            let mut keys: Vec<&String> = nm.keys().collect();
            keys.sort();
            for k in keys {
                if existing.contains(k) {
                    diff_slot(doc, tx, Slot::Key(id, k.clone()), &nm[k])?;
                } else {
                    emit(doc, tx, Prim::SetKey { obj: id, key: k.clone(), value: nm[k].clone() })?;
                }
            }
            Ok(())
        }
        (Val::Ref(id), Value::Array(na)) if doc.arr(*id).is_some() => diff_array(doc, tx, *id, na),
        (Val::Scalar(s), v) if s == v => Ok(()),
        _ => emit(doc, tx, slot_prim(&slot, new)),
    }
}

pub(super) fn diff_array(doc: &mut Doc, tx: &mut Tx, arr: Id, new: &[Value]) -> Result<(), String> {
    let old: Vec<(Id, Value)> = doc.visible(arr).iter().map(|e| (e.id, doc.val_json(&e.reg.val, 0).unwrap_or(Value::Null))).collect();
    let (n, m) = (old.len(), new.len());
    let mut pre = 0;
    while pre < n && pre < m && old[pre].1 == new[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < n - pre && suf < m - pre && old[n - 1 - suf].1 == new[m - 1 - suf] {
        suf += 1;
    }
    let mut matches: Vec<(usize, usize)> = (0..pre).map(|i| (i, i)).collect();
    let (on, nn) = (n - pre - suf, m - pre - suf);
    if on > 0 && nn > 0 && on.saturating_mul(nn) <= 4_000_000 {
        // longest common subsequence of the middle parts
        let w = nn + 1;
        let mut t = vec![0u32; (on + 1) * w];
        for i in (0..on).rev() {
            for j in (0..nn).rev() {
                t[i * w + j] = if old[pre + i].1 == new[pre + j] { t[(i + 1) * w + j + 1] + 1 } else { t[(i + 1) * w + j].max(t[i * w + j + 1]) };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < on && j < nn {
            if old[pre + i].1 == new[pre + j] {
                matches.push((pre + i, pre + j));
                i += 1;
                j += 1;
            } else if t[(i + 1) * w + j] >= t[i * w + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    for k in 0..suf {
        matches.push((n - suf + k, m - suf + k));
    }
    matches.push((n, m)); // sentinel
    let (mut oi, mut nj) = (0usize, 0usize);
    let mut anchor: Option<Id> = None;
    for (mi, mj) in matches {
        let (lo, ln) = (mi - oi, mj - nj);
        let k = lo.min(ln);
        for t in 0..k {
            let (oid, ov) = (&old[oi + t].0, &old[oi + t].1);
            let nv = &new[nj + t];
            if same_identity(ov, nv) {
                diff_slot(doc, tx, Slot::Elem(arr, *oid), nv)?;
                anchor = Some(*oid);
            } else {
                emit(doc, tx, Prim::DelElem { arr, elem: *oid })?;
                let nid = tx.gen.peek();
                emit(doc, tx, Prim::InsElem { arr, after: anchor, value: nv.clone() })?;
                anchor = Some(nid);
            }
        }
        for t in k..lo {
            emit(doc, tx, Prim::DelElem { arr, elem: old[oi + t].0 })?;
        }
        for t in k..ln {
            let nid = tx.gen.peek();
            emit(doc, tx, Prim::InsElem { arr, after: anchor, value: new[nj + t].clone() })?;
            anchor = Some(nid);
        }
        if mi < n {
            anchor = Some(old[mi].0);
        }
        oi = mi + 1;
        nj = mj + 1;
    }
    Ok(())
}

pub(super) fn translate(doc: &mut Doc, tx: &mut Tx, op: &Op) -> Result<(), String> {
    match op {
        Op::Batch { ops } => {
            for o in ops {
                translate(doc, tx, o)?;
            }
            Ok(())
        }
        Op::Set { path, value } => {
            let toks = tokens(path)?;
            if toks.is_empty() {
                return diff_slot(doc, tx, Slot::Root, value);
            }
            let (pt, last) = toks.split_at(toks.len() - 1);
            let tok = &last[0];
            let parent = walk(doc, pt).map_err(|_| format!("no such path '{}'", ptr_of(pt)))?;
            let Val::Ref(pid) = parent else { return Err(format!("'{}' is not a container", ptr_of(pt))) };
            match doc.nodes.get(&pid) {
                Some(Cont::Obj(m)) => {
                    if m.get(tok).map(|r| r.val != Val::Absent).unwrap_or(false) {
                        diff_slot(doc, tx, Slot::Key(pid, tok.clone()), value)
                    } else {
                        emit(doc, tx, Prim::SetKey { obj: pid, key: tok.clone(), value: value.clone() })
                    }
                }
                Some(Cont::Arr(_)) => {
                    let i: usize = tok.parse().map_err(|_| format!("bad array index '{tok}'"))?;
                    let id = doc.visible(pid).get(i).map(|e| e.id).ok_or_else(|| format!("index {i} out of range at '{}'", ptr_of(pt)))?;
                    diff_slot(doc, tx, Slot::Elem(pid, id), value)
                }
                None => Err(format!("'{}' is not a container", ptr_of(pt))),
            }
        }
        Op::Insert { path, value } => {
            let toks = tokens(path)?;
            if toks.is_empty() {
                return Err("path must not be empty here".into());
            }
            let (pt, last) = toks.split_at(toks.len() - 1);
            let tok = &last[0];
            let parent = walk(doc, pt).map_err(|_| format!("no such path '{}'", ptr_of(pt)))?;
            let Val::Ref(pid) = parent else { return Err(format!("'{}' is not a container", ptr_of(pt))) };
            match doc.nodes.get(&pid) {
                Some(Cont::Obj(m)) => {
                    if m.get(tok).map(|r| r.val != Val::Absent).unwrap_or(false) {
                        diff_slot(doc, tx, Slot::Key(pid, tok.clone()), value)
                    } else {
                        emit(doc, tx, Prim::SetKey { obj: pid, key: tok.clone(), value: value.clone() })
                    }
                }
                Some(Cont::Arr(_)) => {
                    let vis = doc.visible(pid);
                    let i = if tok == "-" { vis.len() } else { tok.parse().map_err(|_| format!("bad array index '{tok}'"))? };
                    if i > vis.len() {
                        return Err(format!("insert index {i} out of range at '{}'", ptr_of(pt)));
                    }
                    let after = if i == 0 { None } else { Some(vis[i - 1].id) };
                    emit(doc, tx, Prim::InsElem { arr: pid, after, value: value.clone() })
                }
                None => Err(format!("'{}' is not a container", ptr_of(pt))),
            }
        }
        Op::Remove { path } => {
            let toks = tokens(path)?;
            if toks.is_empty() {
                return Err("path must not be empty here".into());
            }
            let (pt, last) = toks.split_at(toks.len() - 1);
            let tok = &last[0];
            let parent = walk(doc, pt).map_err(|_| format!("no such path '{}'", ptr_of(pt)))?;
            let Val::Ref(pid) = parent else { return Err(format!("'{}' is not a container", ptr_of(pt))) };
            match doc.nodes.get(&pid) {
                Some(Cont::Obj(m)) => {
                    if m.get(tok).map(|r| r.val != Val::Absent).unwrap_or(false) {
                        emit(doc, tx, Prim::DelKey { obj: pid, key: tok.clone() })
                    } else {
                        Err(format!("no key '{tok}' at '{}'", ptr_of(pt)))
                    }
                }
                Some(Cont::Arr(_)) => {
                    let i: usize = tok.parse().map_err(|_| format!("bad array index '{tok}'"))?;
                    let id = doc.visible(pid).get(i).map(|e| e.id).ok_or_else(|| format!("index {i} out of range at '{}'", ptr_of(pt)))?;
                    emit(doc, tx, Prim::DelElem { arr: pid, elem: id })
                }
                None => Err(format!("'{}' is not a container", ptr_of(pt))),
            }
        }
    }
}

/// Merge records so that undoing a multi-step edit restores the original, not an intermediate state.
pub(super) fn coalesce(undos: Vec<Undo>) -> Vec<Undo> {
    let mut out: Vec<Undo> = Vec::new();
    for u in undos {
        match u {
            Undo::Restore { slot, stamp, prev } => {
                let mut merged = false;
                for x in out.iter_mut() {
                    if let Undo::Restore { slot: s2, stamp: st2, .. } = x {
                        if *s2 == slot {
                            *st2 = stamp;
                            merged = true;
                            break;
                        }
                    }
                }
                if !merged {
                    out.push(Undo::Restore { slot, stamp, prev });
                }
            }
            Undo::Reinsert { arr, elem, left, value } => {
                if let Some(p) = out.iter().position(|x| matches!(x, Undo::Remove { elem: e2, .. } if *e2 == elem)) {
                    out.remove(p);
                } else {
                    out.push(Undo::Reinsert { arr, elem, left, value });
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Express the inverse of one record against the CURRENT tree (this is the "transform against later ops").
pub(super) fn plan_inverse(doc: &mut Doc, tx: &mut Tx, u: &Undo) -> Result<(), String> {
    match u {
        Undo::Remove { arr, elem } => {
            let live = doc.arr(*arr).and_then(|v| v.iter().find(|e| e.id == *elem)).map(|e| e.tomb.is_none()).unwrap_or(false);
            if live {
                emit(doc, tx, Prim::DelElem { arr: *arr, elem: *elem })?;
            }
            Ok(())
        }
        Undo::Reinsert { arr, elem, left, value } => {
            let v = doc.arr(*arr).ok_or("array is gone")?;
            let nearest_before = |p: usize| v[..=p].iter().rev().find(|e| e.tomb.is_none()).map(|e| e.id);
            let after = match v.iter().position(|e| e.id == *elem) {
                Some(p) => {
                    if v[p].tomb.is_none() {
                        return Ok(());
                    }
                    nearest_before(p)
                }
                // the tombstone was compacted away: use the old left neighbour if it is still there
                None => match left {
                    None => None,
                    Some(l) => match v.iter().position(|e| e.id == *l) {
                        Some(p) => nearest_before(p),
                        None => v.iter().rev().find(|e| e.tomb.is_none()).map(|e| e.id),
                    },
                },
            };
            emit(doc, tx, Prim::InsElem { arr: *arr, after, value: value.clone() })
        }
        Undo::Restore { slot, stamp, prev } => {
            if doc.reg_stamp(slot) != Some(*stamp) {
                return Ok(()); // somebody wrote after us: their write stays
            }
            if let Slot::Elem(a, e) = slot {
                if doc.arr(*a).and_then(|v| v.iter().find(|x| x.id == *e)).map(|x| x.tomb.is_some()).unwrap_or(true) {
                    return Ok(());
                }
            }
            match (slot, prev) {
                (Slot::Root, Some(v)) => diff_slot(doc, tx, Slot::Root, v),
                (Slot::Root, None) => Ok(()),
                (Slot::Key(o, k), None) => emit(doc, tx, Prim::DelKey { obj: *o, key: k.clone() }),
                (s, Some(v)) => diff_slot(doc, tx, s.clone(), v),
                (Slot::Elem(..), None) => Ok(()),
            }
        }
    }
}
