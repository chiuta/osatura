// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Durable log of a collaboration session (R-111 persistence): the text format and the loader. The file
//! handling (append, fsync, atomic rewrite) is in the CLI's `collab_srv`; everything here is pure so it can
//! be tested without a disk.
//!
//! Format (version 2), one record per line: `<64 hex digits> <json>`. The hex number is a chained SHA-256,
//! `h(n) = sha256(h(n-1) as 64 hex digits ++ json(n))` with `h(0) = sha256("osatura-collab-log")`, so a flipped
//! byte, a deleted, duplicated or reordered line is detected at the first line it touches. It is a checksum
//! against damage, not a defence against somebody who edits the file on purpose (they can recompute it; the
//! file itself is created readable by its owner only). Version 1 logs used a 16-digit FNV-1a chain; they are
//! refused with a message saying so (they cannot be checked with the stronger hash).
//!
//! * line 1, the checkpoint: `{"osatura_collab_log":1,"snapshot":{doc,clock,lamport},"log_base":N,"next_id":K,"origin":"<digest>"}`;
//! * every further line: `{"i":<absolute log position>,"by":<replica>,"env":<Envelope>}`, positions consecutive from `log_base`.
//!
//! A *torn tail* (the last line cut short by a crash while it was being appended, so it has no newline or
//! fails its checksum and nothing follows) is not an error: the entry was never acknowledged to anybody, the
//! loader drops it and reports `torn_tail`. Damage anywhere else is an error naming the line.

use crate::collab::{Envelope, ReplicaId, Snapshot};
use crate::collabhub::Hub;
use serde_json::{json, Value};

pub const MAGIC: &str = "osatura-collab-log";
pub const VERSION: u64 = 2;

/// The hash a log starts its chain from.
pub fn chain_start() -> String {
    crate::vcs::sha256_hex(MAGIC.as_bytes())
}

/// Digest of a project document (used to tie a log to the project it belongs to).
pub fn digest(doc: &Value) -> String {
    crate::vcs::sha256_hex(doc.to_string().as_bytes())
}

fn link(prev: &str, json: &str) -> String {
    crate::vcs::sha256_hex(format!("{prev}{json}").as_bytes())
}

/// One finished line (with `\n`) and its hash, which the next line chains from.
pub fn encode_line(prev: &str, body: &Value) -> (String, String) {
    let json = body.to_string();
    let h = link(prev, &json);
    (format!("{h} {json}\n"), h)
}

/// The checkpoint line for a hub: its snapshot, how many entries preceded it, the next participant id.
///
/// `origin` is the digest of the document the session first started from (`digest(&hub.doc_json())` for a new
/// session; carried over unchanged when a compaction rewrites the checkpoint), so the original project file
/// still matches the log after compaction.
pub fn encode_header(hub: &Hub, origin: &str) -> (String, String) {
    let body = json!({"osatura_collab_log": VERSION, "snapshot": hub.snapshot(), "log_base": hub.index(),
        "next_id": hub.next_id(), "origin": origin, "doc_digest": digest(&hub.doc_json())});
    encode_line(&chain_start(), &body)
}

pub fn encode_entry(prev: &str, index: usize, by: ReplicaId, env: &Envelope) -> (String, String) {
    encode_line(prev, &json!({"i": index, "by": by, "env": env}))
}

pub struct Loaded {
    pub hub: Hub,
    /// Digest of the document the session started from (the first checkpoint's document).
    pub origin: String,
    /// Digest of the document after replay.
    pub current: String,
    pub entries: usize,
    /// Byte length of the valid prefix of the file and the chain hash after it (to continue appending).
    pub valid_len: usize,
    pub last_hash: String,
    pub torn_tail: bool,
}

/// Parse and replay a log. Errors are complete sentences saying what is wrong and on which line.
pub fn load(text: &str) -> Result<Loaded, String> {
    if text.trim().is_empty() {
        return Err("the collaboration log is empty".into());
    }
    // version 1 logs start with a 16-digit checksum
    if let Some((h, _)) = text.lines().next().and_then(|l| l.split_once(' ')) {
        if h.len() == 16 && h.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("this collaboration log is in the old version 1 format (16-digit FNV-1a checksums); this program writes version 2 (SHA-256) and cannot check or resume it. Move it away to start a new log".into());
        }
    }
    let mut prev = chain_start();
    let mut hub: Option<Hub> = None;
    let mut origin = String::new();
    let mut entries = 0usize;
    let mut valid_len = 0usize;
    let mut pos = 0usize;
    let mut torn = false;
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    for (n, raw) in lines.iter().enumerate() {
        let lineno = n + 1;
        let last = n + 1 == lines.len();
        let complete = raw.ends_with('\n');
        let line = raw.trim_end_matches('\n');
        let bad = |why: String| -> Result<Loaded, String> {
            Err(format!(
                "collaboration log is damaged at line {lineno}: {why}"
            ))
        };
        let parsed = (|| -> Result<(String, Value), String> {
            let (h, json) = line.split_once(' ').ok_or("no checksum")?;
            if h.len() != 64 || link(&prev, json) != h {
                return Err(
                    "checksum does not match (the file was changed, truncated or damaged)".into(),
                );
            }
            let v: Value = serde_json::from_str(json).map_err(|e| format!("not JSON: {e}"))?;
            Ok((h.to_string(), v))
        })();
        let (h, v) = match parsed {
            Ok(x) if complete => x,
            Ok(_) | Err(_) if last && hub.is_some() => {
                torn = true; // a cut-off final append
                break;
            }
            Err(e) => return bad(e),
            Ok(_) => return bad("incomplete line".into()),
        };
        if let Some(live) = hub.as_mut() {
            let i = v["i"].as_u64().ok_or("entry lacks 'i'");
            let env: Result<Envelope, _> = serde_json::from_value(v["env"].clone());
            let (Ok(i), Ok(env)) = (i, env) else {
                return bad("entry is not {i, by, env}".into());
            };
            if v["by"].as_u64() != Some(env.replica as u64) {
                return bad("entry 'by' differs from the envelope's replica".into());
            }
            if let Err(e) = live.replay(i as usize, env) {
                return bad(e);
            }
            entries += 1;
        } else {
            if v["osatura_collab_log"].as_u64() != Some(VERSION) {
                return bad(format!(
                    "not a version {VERSION} collaboration log (found {})",
                    v["osatura_collab_log"]
                ));
            }
            let snap: Snapshot = serde_json::from_value(v["snapshot"].clone())
                .map_err(|e| format!("collaboration log checkpoint has a bad snapshot: {e}"))?;
            let base = v["log_base"]
                .as_u64()
                .ok_or("collaboration log checkpoint lacks log_base")?
                as usize;
            let next = v["next_id"]
                .as_u64()
                .filter(|x| *x < u32::MAX as u64)
                .ok_or("collaboration log checkpoint lacks a valid next_id")?
                as ReplicaId;
            let hb = Hub::restore(snap, base, next)
                .map_err(|e| format!("collaboration log checkpoint: {e}"))?;
            if v["doc_digest"].as_str() != Some(digest(&hb.doc_json()).as_str()) {
                return bad("the checkpoint document does not match its recorded digest".into());
            }
            origin = v["origin"]
                .as_str()
                .filter(|o| o.len() == 64)
                .ok_or("collaboration log checkpoint lacks an origin digest")?
                .to_string();
            hub = Some(hb);
        }
        prev = h;
        pos += raw.len();
        valid_len = pos;
    }
    let mut hub = hub.ok_or("the collaboration log has no checkpoint line")?;
    hub.finish_restore();
    let current = digest(&hub.doc_json());
    Ok(Loaded {
        hub,
        origin,
        current,
        entries,
        valid_len,
        last_hash: prev,
        torn_tail: torn,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collab::Presence;
    use crate::collabhub::Mode;
    use crate::edit::Op;

    fn base() -> Value {
        json!({"frames": 24, "layers": [{"name": "a", "visible": true}, {"name": "b", "visible": true}], "meta": {"k": 1}})
    }
    fn base_hub_doc() -> Value {
        Hub::new(&base()).doc_json()
    }
    fn ok() -> impl FnMut(&Value) -> Result<(), Vec<String>> {
        |_| Ok(())
    }
    fn set(p: &str, v: Value) -> Op {
        Op::Set {
            path: p.into(),
            value: v,
        }
    }

    /// Run a session while writing the log the way the server does; return the hub and the file text.
    fn session() -> (Hub, String) {
        let mut h = Hub::new(&base());
        let (mut text, mut prev) = encode_header(&h, &digest(&base_hub_doc()));
        let a = h.join("ana", Mode::Thin, 0).unwrap().replica;
        let b = h.join("bo", Mode::Thin, 0).unwrap().replica;
        let mut ops = vec![
            (a, set("/frames", json!(30))),
            (
                b,
                Op::Insert {
                    path: "/layers/0".into(),
                    value: json!({"name": "front"}),
                },
            ),
            (a, set("/layers/1/visible", json!(false))),
            (
                b,
                Op::Remove {
                    path: "/layers/2".into(),
                },
            ),
        ];
        ops.push((a, set("/meta/k", json!(2))));
        for (now, (who, op)) in (1..).zip(ops) {
            h.submit_ops(who, None, &[op], &mut ok(), now).unwrap();
        }
        h.undo_redo(a, true, &mut ok(), 9).unwrap();
        for e in h.take_events() {
            if let crate::collabhub::Event::Ops {
                index,
                by,
                envelope,
            } = e
            {
                let (l, nh) = encode_entry(&prev, index, by, &envelope);
                text.push_str(&l);
                prev = nh;
            }
        }
        (h, text)
    }

    #[test]
    fn a_resumed_hub_has_the_same_state_clock_and_index_and_keeps_working() {
        let (h, text) = session();
        let l = load(&text).unwrap();
        assert_eq!(
            (l.entries, l.torn_tail, l.valid_len),
            (6, false, text.len())
        );
        assert_eq!(l.hub.doc_json(), h.doc_json());
        assert_eq!(
            (l.hub.clock().clone(), l.hub.lamport(), l.hub.index()),
            (h.clock().clone(), h.lamport(), h.index())
        );
        assert_eq!(l.origin, digest(&Hub::new(&base()).doc_json()));
        assert_eq!(l.current, digest(&h.doc_json()));
        // new participants never reuse an id whose envelopes are in the log, and edits go on from the same clock
        let mut r = l.hub;
        let c = r.join("cy", Mode::Thin, 0).unwrap();
        assert!(c.replica >= h.next_id() && c.index == h.index());
        let s = r
            .submit_ops(c.replica, None, &[set("/frames", json!(99))], &mut ok(), 1)
            .unwrap();
        assert_eq!(
            (s.index, r.doc_json()["frames"].clone()),
            (h.index() + 1, json!(99))
        );
        // the host's own editing path is intact too
        let mut d = r.doc_json();
        d["meta"]["k"] = json!(7);
        assert!(r
            .submit_doc(crate::collabhub::LOCAL_ID, &d, &mut ok(), 2)
            .unwrap()
            .envelope
            .is_some());
        let _ = Presence::new(1, "x");
    }

    #[test]
    fn checksums_are_sha256_and_old_fnv_logs_get_a_clear_message() {
        let (_, text) = session();
        let first = text.lines().next().unwrap();
        let h = first.split_once(' ').unwrap().0;
        assert_eq!(h.len(), 64);
        assert_eq!(chain_start(), crate::vcs::sha256_hex(MAGIC.as_bytes()));
        // a version 1 line: 16 hex digits, then JSON
        let old = "cbf29ce484222325 {\"osatura_collab_log\":1}\n";
        let e = load(old).err().unwrap();
        assert!(e.contains("old version 1") && e.contains("SHA-256"), "{e}");
    }

    #[test]
    fn damage_in_the_middle_is_an_error_naming_the_line() {
        let (_, text) = session();
        let mut lines: Vec<String> = text.lines().map(String::from).collect();
        // flip one digit inside the third line's JSON
        let pos = lines[2].find("\"i\":").unwrap() + 4;
        let c = lines[2].as_bytes()[pos];
        lines[2].replace_range(pos..pos + 1, if c == b'1' { "2" } else { "1" });
        let e = load(&(lines.join("\n") + "\n")).err().unwrap();
        assert!(e.contains("line 3") && e.contains("checksum"), "{e}");
        // a deleted line breaks the chain at the next one
        let mut l2: Vec<&str> = text.lines().collect();
        l2.remove(2);
        let e = load(&(l2.join("\n") + "\n")).err().unwrap();
        assert!(e.contains("line 3"), "{e}");
        // swapped lines
        let mut l3: Vec<&str> = text.lines().collect();
        l3.swap(2, 3);
        assert!(load(&(l3.join("\n") + "\n")).is_err());
    }

    #[test]
    fn a_torn_final_append_is_dropped_but_a_torn_checkpoint_is_not_survivable() {
        let (h, text) = session();
        let cut = &text[..text.len() - 20];
        let l = load(cut).unwrap();
        assert!(l.torn_tail && l.entries == 5 && l.valid_len < cut.len());
        assert!(l.hub.index() + 1 == h.index());
        // an intact last line that only lacks its newline is a torn tail as well (we cannot know it was complete)
        let l = load(text.trim_end_matches('\n')).unwrap();
        assert!(l.torn_tail && l.entries == 5);
        // appending after the valid prefix continues the chain
        let mut fixed = text[..l.valid_len].to_string();
        let (line, _) = encode_line(
            &l.last_hash,
            &json!({"i": l.hub.index(), "by": 9, "env": null}),
        );
        fixed.push_str(&line);
        assert!(load(&fixed).err().unwrap().contains("not {i, by, env}"));
        assert!(load(&text[..40]).is_err());
        assert!(load("").is_err());
        assert!(load("\n").is_err());
    }

    #[test]
    fn foreign_or_inconsistent_files_are_refused_with_a_reason() {
        let (_, text) = session();
        assert!(load("hello\n").err().unwrap().contains("line 1"));
        let (l0, h0) = encode_line(&chain_start(), &json!({"osatura_collab_log": 3}));
        assert!(load(&l0).err().unwrap().contains("version 2"));
        let _ = h0;
        // an entry whose position is not the next one
        let mut ls: Vec<String> = text.lines().map(String::from).collect();
        let l = load(&text).unwrap();
        let ent: Value = serde_json::from_str(ls[3].split_once(' ').unwrap().1).unwrap();
        let prev = ls[1].split_once(' ').unwrap().0.to_string();
        let (bad, _) = encode_line(&prev, &json!({"i": 7, "by": ent["by"], "env": ent["env"]}));
        ls[2] = bad.trim_end().to_string();
        ls.truncate(3);
        let e = load(&(ls.join("\n") + "\n")).err().unwrap();
        assert!(e.contains("line 3") && e.contains("expected"), "{e}");
        // an envelope whose predecessor is missing (not causally deliverable)
        let mut ls2: Vec<String> = text.lines().map(String::from).collect();
        ls2.remove(2);
        // re-chain so only the causal check can complain
        let mut prev = chain_start();
        let mut out = String::new();
        for (n, line) in ls2.iter().enumerate() {
            let body: Value = serde_json::from_str(line.split_once(' ').unwrap().1).unwrap();
            let body = if n >= 2 {
                json!({"i": n - 1, "by": body["by"], "env": body["env"]})
            } else {
                body
            };
            let (t, h) = encode_line(&prev, &body);
            out.push_str(&t);
            prev = h;
        }
        let e = load(&out).err().unwrap();
        assert!(e.contains("does not fit"), "{e}");
        let _ = l;
    }

    #[test]
    fn a_compacted_hub_is_checkpointed_and_resumes_at_its_index() {
        let (mut h, _) = session();
        let st = h.compact();
        assert_eq!(st["purged"], json!(true));
        let (mut text, mut prev) = encode_header(&h, &digest(&base_hub_doc()));
        let a = h.join("dee", Mode::Thin, 0).unwrap().replica;
        h.submit_ops(a, None, &[set("/frames", json!(48))], &mut ok(), 1)
            .unwrap();
        for e in h.take_events() {
            if let crate::collabhub::Event::Ops {
                index,
                by,
                envelope,
            } = e
            {
                let (l, nh) = encode_entry(&prev, index, by, &envelope);
                text.push_str(&l);
                prev = nh;
            }
        }
        let r = load(&text).unwrap();
        assert_eq!(r.hub.doc_json(), h.doc_json());
        assert_eq!(
            (r.hub.index(), r.hub.log_base(), r.entries),
            (h.index(), h.index() - 1, 1)
        );
        assert_eq!(r.hub.clock(), h.clock());
        assert_eq!(r.hub.lamport(), h.lamport());
        assert_eq!(
            r.origin,
            digest(&base_hub_doc()),
            "compaction keeps the original origin"
        );
        assert_ne!(r.current, r.origin);
    }
}
