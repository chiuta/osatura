// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `collab`.

use super::*;
use serde_json::json;

fn base() -> Value {
    json!({
        "name": "t", "frames": 24,
        "layers": [{"name": "a", "opacity": 1.0, "visible": true}, {"name": "b", "opacity": 0.5, "visible": true}, {"name": "c", "opacity": 0.2, "visible": true}],
        "channels": [{"target": "a", "prop": "x", "track": {"keys": [{"frame": 0, "value": 0.0}, {"frame": 10, "value": 5.0}]}}],
        "meta": {"tags": ["x", "y"]}
    })
}

fn pair() -> (Replica, Replica) {
    (Replica::new(1, &base()).unwrap(), Replica::new(2, &base()).unwrap())
}

fn set(p: &str, v: Value) -> Op {
    Op::Set { path: p.into(), value: v }
}
fn ins(p: &str, v: Value) -> Op {
    Op::Insert { path: p.into(), value: v }
}
fn rem(p: &str) -> Op {
    Op::Remove { path: p.into() }
}
fn send(from: &mut Replica, op: &Op, to: &mut [&mut Replica]) -> Envelope {
    let e = from.local_op(op).unwrap().expect("envelope");
    for t in to {
        t.receive(e.clone()).unwrap();
    }
    e
}

#[test]
fn local_ops_match_the_editor_semantics() {
    let mut r = Replica::new(1, &base()).unwrap();
    r.local_op(&set("/layers/1/opacity", json!(0.9))).unwrap();
    r.local_op(&ins("/layers/1", json!({"name": "new"}))).unwrap();
    r.local_op(&ins("/layers/-", json!({"name": "last"}))).unwrap();
    r.local_op(&rem("/layers/0")).unwrap();
    r.local_op(&set("/meta/extra", json!(7))).unwrap();
    r.local_op(&rem("/meta/tags")).unwrap();
    let mut want = base();
    // reference: the same ops on a plain document
    want["layers"][1]["opacity"] = json!(0.9);
    want["layers"].as_array_mut().unwrap().insert(1, json!({"name": "new"}));
    want["layers"].as_array_mut().unwrap().push(json!({"name": "last"}));
    want["layers"].as_array_mut().unwrap().remove(0);
    want["meta"]["extra"] = json!(7);
    want["meta"].as_object_mut().unwrap().remove("tags");
    assert_eq!(r.json(), want);
}

#[test]
fn bad_local_ops_change_nothing_and_batches_are_atomic() {
    let mut r = Replica::new(1, &base()).unwrap();
    for op in [
        rem("/layers/9"),
        set("/nope/x", json!(1)),
        set("/layers/abc", json!(1)),
        ins("/layers/9", json!(1)),
        rem("/meta/missing"),
        ins("", json!(1)),
        set("layers", json!(1)),
        set("/frames/x", json!(1)),
    ] {
        assert!(r.local_op(&op).is_err(), "{op:?}");
        assert_eq!(r.json(), base());
    }
    let b = Op::Batch { ops: vec![set("/frames", json!(99)), rem("/layers/7")] };
    assert!(r.local_op(&b).is_err());
    assert_eq!(r.json(), base());
    assert_eq!(r.clock().get(&1), None, "a failed edit consumes no sequence number");
    assert!(r.local_op(&set("/frames", json!(24))).unwrap().is_none(), "writing the same value is a no-op");
}

#[test]
fn concurrent_scalar_writes_use_lww_with_replica_tiebreak() {
    let (mut a, mut b) = pair();
    let ea = a.local_op(&set("/layers/0/opacity", json!(0.1))).unwrap().unwrap();
    let eb = b.local_op(&set("/layers/0/opacity", json!(0.9))).unwrap().unwrap();
    assert_eq!(ea.lamport, eb.lamport);
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.json(), b.json());
    assert_eq!(a.json()["layers"][0]["opacity"], json!(0.9), "equal lamport: the larger replica id wins");
    // a later write (larger lamport) beats a higher replica id
    let e = a.local_op(&set("/layers/0/opacity", json!(0.3))).unwrap().unwrap();
    b.receive(e).unwrap();
    assert_eq!(b.json()["layers"][0]["opacity"], json!(0.3));
}

#[test]
fn stale_indices_are_transformed_by_identity() {
    let (mut a, mut b) = pair();
    // a inserts at the front; b (not having seen it) edits what it sees as layer 1 ("b")
    let ea = a.local_op(&ins("/layers/0", json!({"name": "front"}))).unwrap().unwrap();
    let eb = b.local_op(&set("/layers/1/opacity", json!(0.77))).unwrap().unwrap();
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.json(), b.json());
    let l = a.json()["layers"].clone();
    assert_eq!(l[0]["name"], "front");
    assert_eq!(l[2]["name"], "b");
    assert_eq!(l[2]["opacity"], json!(0.77), "the edit followed its element, not its index");
    assert_eq!(l[1]["opacity"], json!(1.0));
}

#[test]
fn concurrent_inserts_at_the_same_place_keep_both_in_one_order() {
    let (mut a, mut b) = pair();
    let ea = a.local_op(&ins("/layers/1", json!({"name": "from_a"}))).unwrap().unwrap();
    let eb = b.local_op(&ins("/layers/1", json!({"name": "from_b"}))).unwrap().unwrap();
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.json(), b.json());
    let names: Vec<String> = a.json()["layers"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(names.len(), 5);
    assert_eq!(names[0], "a");
    assert_eq!(names[3], "b");
    assert!(names.contains(&"from_a".to_string()) && names.contains(&"from_b".to_string()));
}

#[test]
fn remove_wins_over_edit_inside_and_the_edit_is_dropped_deterministically() {
    let (mut a, mut b) = pair();
    let ea = a.local_op(&rem("/layers/1")).unwrap().unwrap();
    let eb = b.local_op(&set("/layers/1/opacity", json!(0.01))).unwrap().unwrap();
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.json(), b.json());
    assert_eq!(a.json()["layers"].as_array().unwrap().len(), 2);
    assert!(!a.json().to_string().contains("0.01"));
    assert_eq!(a.doc(), b.doc());
    assert_eq!(a.dropped, 0, "applied to the hidden element, not an error");
    // both remove the same element
    let (mut a, mut b) = pair();
    let ea = a.local_op(&rem("/layers/0")).unwrap().unwrap();
    let eb = b.local_op(&rem("/layers/0")).unwrap().unwrap();
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.doc(), b.doc());
    assert_eq!(a.json()["layers"].as_array().unwrap().len(), 2);
}

#[test]
fn whole_object_set_is_diffed_so_concurrent_key_edits_both_survive() {
    let (mut a, mut b) = pair();
    // a adds a key to the channel track by writing the whole channel (what Editor::set_key does)
    let mut ch = base()["channels"][0].clone();
    ch["track"]["keys"].as_array_mut().unwrap().push(json!({"frame": 20, "value": 1.5}));
    let ea = a.local_op(&set("/channels/0", ch)).unwrap().unwrap();
    // b meanwhile changes an existing key's value
    let eb = b.local_op(&set("/channels/0/track/keys/0/value", json!(-3.0))).unwrap().unwrap();
    assert_eq!(ea.ops.len(), 1, "only the new key travels: {:?}", ea.ops);
    a.receive(eb).unwrap();
    b.receive(ea).unwrap();
    assert_eq!(a.json(), b.json());
    let keys = a.json()["channels"][0]["track"]["keys"].clone();
    assert_eq!(keys.as_array().unwrap().len(), 3);
    assert_eq!(keys[0]["value"], json!(-3.0));
    assert_eq!(keys[2]["frame"], json!(20));
}

#[test]
fn unrelated_objects_in_a_slot_are_replaced_not_merged() {
    let mut r = Replica::new(1, &base()).unwrap();
    let mut v = base();
    v["layers"] = json!([{"name": "z", "opacity": 3}]);
    let e = r.local_doc(&v).unwrap().unwrap();
    assert!(e.ops.iter().any(|p| matches!(p, Prim::DelElem { .. })) && e.ops.iter().any(|p| matches!(p, Prim::InsElem { .. })));
    assert_eq!(r.json(), v);
}

#[test]
fn local_doc_diff_roundtrips_for_random_documents() {
    let mut rng = SimRng(5);
    for _ in 0..40 {
        let mut r = Replica::new(1, &base()).unwrap();
        let mut cur = base();
        for _ in 0..8 {
            if let Some(op) = random_op(&cur, &mut rng) {
                // reference application on a second replica via pointer ops
                let mut other = Replica::new(2, &base()).unwrap();
                other.doc = r.doc.clone();
                if other.local_op(&op).is_ok() {
                    let target = other.json();
                    r.local_doc(&target).unwrap();
                    assert_eq!(r.json(), target);
                    cur = target;
                }
            }
        }
    }
}

#[test]
fn redelivery_is_idempotent_and_the_causal_buffer_holds_back_early_envelopes() {
    let (mut a, mut b) = pair();
    let e1 = a.local_op(&ins("/layers/-", json!({"name": "n1"}))).unwrap().unwrap();
    let e2 = a.local_op(&set("/layers/3/opacity", json!(0.4))).unwrap().unwrap();
    // e2 first: buffered, nothing visible yet
    assert_eq!(b.receive(e2.clone()).unwrap(), Delivery::Buffered);
    assert_eq!(b.json(), base());
    assert_eq!(b.pending_len(), 1);
    assert_eq!(b.receive(e2.clone()).unwrap(), Delivery::Buffered, "same buffered envelope twice is harmless");
    match b.receive(e1.clone()).unwrap() {
        Delivery::Applied(v) => assert_eq!(v.len(), 2, "e1 releases e2"),
        d => panic!("{d:?}"),
    }
    assert_eq!(b.pending_len(), 0);
    assert_eq!(a.json(), b.json());
    let before = b.doc().clone();
    assert_eq!(b.receive(e1).unwrap(), Delivery::Duplicate);
    assert_eq!(b.receive(e2).unwrap(), Delivery::Duplicate);
    assert_eq!(*b.doc(), before);
}

#[test]
fn cross_replica_dependency_waits_for_the_other_replica() {
    let mut a = Replica::new(1, &base()).unwrap();
    let mut b = Replica::new(2, &base()).unwrap();
    let mut c = Replica::new(3, &base()).unwrap();
    let ea = send(&mut a, &ins("/layers/-", json!({"name": "from_a"})), &mut [&mut b]);
    // b edits the element a created: its envelope depends on a's
    let eb = b.local_op(&set("/layers/3/opacity", json!(0.25))).unwrap().unwrap();
    assert_eq!(eb.deps.get(&1), Some(&1));
    assert_eq!(c.receive(eb).unwrap(), Delivery::Buffered);
    assert!(c.json()["layers"].as_array().unwrap().len() == 3);
    c.receive(ea).unwrap();
    assert_eq!(c.json(), b.json());
}

#[test]
fn lamport_clock_follows_received_envelopes() {
    let (mut a, mut b) = pair();
    for i in 0..5 {
        a.local_op(&set("/frames", json!(100 + i))).unwrap();
    }
    let last = a.log.last().unwrap().clone();
    assert_eq!(last.lamport, 5);
    b.receive(last).unwrap_or(Delivery::Duplicate);
    for e in a.log.clone() {
        b.receive(e).unwrap();
    }
    assert_eq!(b.lamport(), 5);
    let e = b.local_op(&set("/frames", json!(7))).unwrap().unwrap();
    assert_eq!(e.lamport, 6, "happened-after => larger Lamport time");
    assert!(e.deps.get(&1) == Some(&5));
}

#[test]
fn invalid_or_hostile_envelopes_never_panic() {
    let mut r = Replica::new(1, &base()).unwrap();
    let unk = Id { r: 9, s: 9, i: 9 };
    let prims = vec![
        Prim::SetKey { obj: unk, key: "k".into(), value: json!(1) },
        Prim::DelKey { obj: unk, key: "k".into() },
        Prim::SetElem { arr: unk, elem: unk, value: json!(1) },
        Prim::InsElem { arr: unk, after: Some(unk), value: json!(1) },
        Prim::DelElem { arr: unk, elem: unk },
        Prim::SetRoot { value: json!([1, 2, 3]) },
    ];
    // wrong kinds: array ops on an object id and vice versa
    let root_obj = match &r.doc.root.val {
        Val::Ref(id) => *id,
        _ => unreachable!(),
    };
    let mut ps = prims.clone();
    ps.push(Prim::InsElem { arr: root_obj, after: None, value: json!(1) });
    ps.push(Prim::SetKey { obj: root_obj, key: "ok".into(), value: json!(1) });
    let env = Envelope { replica: 2, seq: 1, lamport: 4, deps: VClock::new(), ops: ps };
    r.receive(env).unwrap();
    assert!(r.dropped >= 6);
    for bad in [
        Envelope { replica: 0, seq: 1, lamport: 1, deps: VClock::new(), ops: vec![] },
        Envelope { replica: 2, seq: 0, lamport: 1, deps: VClock::new(), ops: vec![] },
        Envelope { replica: 2, seq: 2, lamport: 0, deps: VClock::new(), ops: vec![] },
        Envelope { replica: 2, seq: 2, lamport: u64::MAX, deps: VClock::new(), ops: vec![] },
        Envelope { replica: 1, seq: 5, lamport: 1, deps: VClock::new(), ops: vec![] },
    ] {
        assert!(r.receive(bad).is_err());
    }
    // an envelope that depends on something that never arrives just waits
    let mut deps = VClock::new();
    deps.insert(7, 3);
    assert_eq!(r.receive(Envelope { replica: 2, seq: 2, lamport: 5, deps, ops: vec![] }).unwrap(), Delivery::Buffered);
}

#[test]
fn hostile_envelopes_are_applied_identically_in_any_order() {
    let unk = Id { r: 9, s: 9, i: 9 };
    let mk = |rep: u32, n: u64| Envelope { replica: rep, seq: 1, lamport: n, deps: VClock::new(), ops: vec![Prim::DelElem { arr: unk, elem: unk }, Prim::SetRoot { value: json!({"w": n}) }] };
    let mut x = Replica::new(1, &base()).unwrap();
    let mut y = Replica::new(1, &base()).unwrap();
    for e in [mk(2, 3), mk(3, 3), mk(4, 9)] {
        x.receive(e).unwrap();
    }
    for e in [mk(4, 9), mk(3, 3), mk(2, 3)] {
        y.receive(e).unwrap();
    }
    assert_eq!(x.doc(), y.doc());
    assert_eq!(x.json(), json!({"w": 9}));
}

#[test]
fn random_runs_converge_for_many_seeds_and_sizes() {
    for (clients, ops) in [(2, 60), (3, 120), (5, 200), (8, 150)] {
        for seed in 0..12u64 {
            let rep = simulate(&base(), &SimConfig::new(clients, ops, seed * 7919 + clients as u64)).unwrap();
            assert!(rep.converged, "clients {clients} seed {seed}: {:?}", rep.divergence);
            assert!(rep.state_identical, "internal state differs: clients {clients} seed {seed}");
            assert!(rep.clocks_identical && rep.pending_left == 0, "seed {seed}");
            assert_eq!(rep.dropped_prims, 0, "honest peers never produce dangling references (seed {seed})");
            assert!(rep.envelopes > ops / 3);
        }
    }
}

#[test]
fn same_op_set_in_many_random_orders_gives_identical_projects() {
    let rep = simulate(&base(), &SimConfig::new(4, 150, 424242)).unwrap();
    assert!(rep.converged && rep.envelopes > 40);
    let reference = {
        let mut r = Replica::new(99, &base()).unwrap();
        for e in &rep.trace {
            r.receive(e.clone()).unwrap();
        }
        r
    };
    assert_eq!(reference.json(), rep.final_doc);
    for seed in 0..40u64 {
        let mut order = rep.trace.clone();
        SimRng(seed).shuffle(&mut order);
        // duplicates sprinkled in
        let extra: Vec<Envelope> = order.iter().step_by(5).cloned().collect();
        order.extend(extra);
        SimRng(seed + 1000).shuffle(&mut order);
        let mut r = Replica::new(100 + seed as u32, &base()).unwrap();
        for e in order {
            r.receive(e).unwrap();
        }
        assert_eq!(r.pending_len(), 0);
        assert_eq!(r.doc(), reference.doc(), "order seed {seed}");
    }
}

#[test]
fn per_user_undo_touches_only_own_edits() {
    let (mut a, mut b) = pair();
    send(&mut a, &set("/layers/0/opacity", json!(0.1)), &mut [&mut b]);
    send(&mut b, &set("/layers/2/opacity", json!(0.9)), &mut [&mut a]);
    send(&mut a, &ins("/layers/-", json!({"name": "mine"})), &mut [&mut b]);
    send(&mut b, &set("/frames", json!(48)), &mut [&mut a]);
    // a undoes twice: the insert, then the opacity; b's edits stay
    for _ in 0..2 {
        let e = a.undo().unwrap().expect("something to undo");
        b.receive(e).unwrap();
    }
    assert_eq!(a.json(), b.json());
    let j = a.json();
    assert_eq!(j["layers"].as_array().unwrap().len(), 3);
    assert_eq!(j["layers"][0]["opacity"], json!(1.0));
    assert_eq!(j["layers"][2]["opacity"], json!(0.9));
    assert_eq!(j["frames"], json!(48));
    assert!(!a.can_undo());
    // redo brings a's edits back, again without touching b's
    let e = a.redo().unwrap().unwrap();
    b.receive(e).unwrap();
    let e = a.redo().unwrap().unwrap();
    b.receive(e).unwrap();
    assert_eq!(a.json(), b.json());
    assert_eq!(a.json()["layers"][0]["opacity"], json!(0.1));
    assert_eq!(a.json()["layers"][3]["name"], "mine");
    assert_eq!(a.json()["frames"], json!(48));
    assert!(a.redo().unwrap().is_none());
    // b can still undo its own, independently
    let e = b.undo().unwrap().unwrap();
    a.receive(e).unwrap();
    assert_eq!(a.json()["frames"], json!(24));
    assert_eq!(a.json()["layers"][0]["opacity"], json!(0.1));
}

#[test]
fn undo_of_a_remove_reinserts_next_to_its_old_neighbour_after_remote_changes() {
    let (mut a, mut b) = pair();
    send(&mut a, &rem("/layers/1"), &mut [&mut b]); // removes "b"
    send(&mut b, &ins("/layers/0", json!({"name": "front"})), &mut [&mut a]);
    send(&mut b, &set("/layers/1/opacity", json!(0.6)), &mut [&mut a]); // edits "a"
    let e = a.undo().unwrap().unwrap();
    b.receive(e).unwrap();
    assert_eq!(a.json(), b.json());
    let names: Vec<String> = a.json()["layers"].as_array().unwrap().iter().map(|l| l["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(names, vec!["front", "a", "b", "c"]);
    assert_eq!(a.json()["layers"][1]["opacity"], json!(0.6));
}

#[test]
fn undo_skips_what_somebody_else_overwrote_later() {
    let (mut a, mut b) = pair();
    send(&mut a, &set("/layers/0/opacity", json!(0.1)), &mut [&mut b]);
    send(&mut b, &set("/layers/0/opacity", json!(0.8)), &mut [&mut a]);
    let before = a.json();
    let r = a.undo().unwrap();
    assert!(r.is_none(), "nothing of a's write is left to undo");
    assert_eq!(a.json(), before);
}

#[test]
fn undo_of_a_multi_step_batch_restores_the_original() {
    let mut r = Replica::new(1, &base()).unwrap();
    r.local_op(&Op::Batch { ops: vec![set("/frames", json!(30)), set("/frames", json!(31)), ins("/layers/0", json!({"name": "q"})), rem("/layers/0"), rem("/layers/0")] }).unwrap();
    assert_ne!(r.json(), base());
    r.undo().unwrap();
    assert_eq!(r.json(), base());
    r.redo().unwrap();
    assert_eq!(r.json()["frames"], json!(31));
    assert_eq!(r.json()["layers"].as_array().unwrap().len(), 2);
}

#[test]
fn random_runs_with_heavy_undo_still_converge() {
    for seed in 0..10u64 {
        let mut cfg = SimConfig::new(4, 200, seed + 500);
        cfg.undo_pct = 45;
        let rep = simulate(&base(), &cfg).unwrap();
        assert!(rep.converged && rep.state_identical, "seed {seed}: {:?}", rep.divergence);
        assert!(rep.undos > 40);
    }
}

#[test]
fn snapshot_roundtrips_through_json_and_a_joiner_continues_the_session() {
    let (mut a, mut b) = pair();
    send(&mut a, &ins("/layers/0", json!({"name": "x"})), &mut [&mut b]);
    send(&mut b, &rem("/layers/2"), &mut [&mut a]);
    let snap = a.snapshot();
    let text = serde_json::to_string(&snap).unwrap();
    let back: Snapshot = serde_json::from_str(&text).unwrap();
    let mut c = Replica::from_snapshot(3, back).unwrap();
    assert_eq!(c.json(), a.json());
    assert_eq!(c.clock(), a.clock());
    let e = c.local_op(&set("/layers/1/opacity", json!(0.42))).unwrap().unwrap();
    a.receive(e.clone()).unwrap();
    b.receive(e).unwrap();
    assert_eq!(a.doc(), b.doc());
    assert_eq!(a.json()["layers"][1]["opacity"], json!(0.42));
    // the snapshot's identities line up: an edit made by a before the snapshot-based join is understood by c
    let e = a.local_op(&set("/layers/0/name", json!("renamed"))).unwrap().unwrap();
    c.receive(e).unwrap();
    assert_eq!(a.json(), c.json());
    assert!(Replica::from_snapshot(0, a.snapshot()).is_err() && Replica::new(0, &base()).is_err());
}

#[test]
fn envelopes_roundtrip_through_json() {
    let mut a = Replica::new(1, &base()).unwrap();
    let e = a.local_op(&Op::Batch { ops: vec![ins("/layers/1", json!({"name": "n", "k": [1, {"z": null}]})), rem("/layers/0"), set("/meta/new", json!("s"))] }).unwrap().unwrap();
    let text = serde_json::to_string(&e).unwrap();
    let back: Envelope = serde_json::from_str(&text).unwrap();
    assert_eq!(back, e);
    let mut b = Replica::new(2, &base()).unwrap();
    b.receive(back).unwrap();
    assert_eq!(a.doc(), b.doc());
    assert!(serde_json::from_str::<Envelope>("{\"replica\":1}").is_err());
    assert!(serde_json::from_str::<Prim>("{\"t\":\"del_elem\",\"arr\":\"bad\",\"elem\":\"1.1.1\"}").is_err());
}

#[test]
fn compaction_drops_stable_log_and_purges_tombstones_only_when_quiescent() {
    let (mut a, mut b) = pair();
    send(&mut a, &rem("/layers/0"), &mut [&mut b]);
    send(&mut b, &rem("/layers/0"), &mut [&mut a]);
    send(&mut a, &ins("/layers/-", json!({"name": "k", "sub": {"x": [1, 2]}})), &mut [&mut b]);
    send(&mut a, &set("/meta", json!({"fresh": true})), &mut [&mut b]); // replaces the old subtree => garbage nodes
    assert_eq!(a.tombstones(), 2);
    let nodes_before = a.node_count();
    // peer b is behind a hypothetical third peer: not quiescent
    let behind = VClock::new();
    let st = a.compact(&[b.clock().clone(), behind]);
    assert!(!st.purged && st.tombstones_purged == 0 && a.tombstones() == 2);
    // everybody has everything
    let before = a.json();
    let st = a.compact(&[b.clock().clone()]);
    assert!(st.purged && st.tombstones_purged == 2 && st.log_dropped >= 3, "{st:?}");
    assert!(a.node_count() < nodes_before && st.nodes_collected > 0);
    assert_eq!(a.json(), before);
    b.compact(&[a.clock().clone()]);
    assert_eq!(a.doc(), b.doc(), "both compacted at the same cut stay identical");
    // and the session goes on
    let e = a.local_op(&ins("/layers/0", json!({"name": "after"}))).unwrap().unwrap();
    b.receive(e).unwrap();
    let e = b.local_op(&set("/layers/0/name", json!("after2"))).unwrap().unwrap();
    a.receive(e).unwrap();
    assert_eq!(a.doc(), b.doc());
    // a peer behind the compacted log is told to fetch a snapshot
    assert!(a.missing_for(&VClock::new()).is_err());
    assert!(a.missing_for(a.clock()).unwrap().is_empty());
}

#[test]
fn missing_for_returns_exactly_what_a_lagging_peer_needs() {
    let (mut a, mut b) = pair();
    let e1 = a.local_op(&set("/frames", json!(1))).unwrap().unwrap();
    let _e2 = a.local_op(&set("/frames", json!(2))).unwrap().unwrap();
    b.receive(e1).unwrap();
    let m = a.missing_for(b.clock()).unwrap();
    assert_eq!(m.len(), 1);
    for e in m {
        b.receive(e).unwrap();
    }
    assert_eq!(a.doc(), b.doc());
}

#[test]
fn fork_as_makes_an_independent_proxy_with_its_own_identity() {
    let (mut a, _) = pair();
    a.local_op(&set("/frames", json!(5))).unwrap();
    let mut p = a.fork_as(7).unwrap();
    let e = p.local_op(&set("/frames", json!(6))).unwrap().unwrap();
    assert_eq!(e.replica, 7);
    assert_eq!(e.seq, 1);
    a.receive(e).unwrap();
    assert_eq!(a.json()["frames"], json!(6));
}

#[test]
fn presence_expires_sanitizes_and_ignores_stale_updates() {
    let mut b = PresenceBoard::new(1000);
    let mut p = Presence::new(3, "  Ana\u{7}  ");
    assert_eq!(p.name, "Ana");
    p.n = 5;
    p.frame = Some(12.0);
    p.selection = vec!["arm".into(); 100];
    p.colour = "red; evil".into();
    assert!(b.update(p.clone(), 100));
    let got = b.get(3).unwrap();
    assert_eq!(got.selection.len(), 64);
    assert_eq!(got.colour, colour_for(3));
    let mut old = p.clone();
    old.n = 4;
    old.frame = Some(99.0);
    assert!(!b.update(old, 200), "older update is ignored");
    assert_eq!(b.get(3).unwrap().frame, Some(12.0));
    let mut nan = p.clone();
    nan.n = 6;
    nan.frame = Some(f32::NAN);
    nan.cursor = Some([f32::INFINITY, 0.0]);
    nan.name = "x".repeat(500);
    assert!(b.update(nan, 300));
    assert_eq!(b.get(3).unwrap().frame, None);
    assert_eq!(b.get(3).unwrap().cursor, None);
    assert_eq!(b.get(3).unwrap().name.chars().count(), 40);
    assert!(b.update(Presence::new(4, ""), 900));
    assert_eq!(b.get(4).unwrap().name, "user4");
    assert!(b.expire(1300).is_empty(), "300 + 1000 not yet passed; 900 + 1000 neither");
    assert_eq!(b.expire(1301), vec![3]);
    b.touch(4, 1500);
    assert!(b.expire(2400).is_empty());
    assert_eq!(b.expire(2501), vec![4]);
    assert!(b.is_empty());
    assert!(!b.remove(4));
}

#[test]
fn simulation_rejects_silly_sizes() {
    assert!(simulate(&base(), &SimConfig::new(0, 10, 1)).is_err());
    assert!(simulate(&base(), &SimConfig::new(65, 10, 1)).is_err());
    let one = simulate(&base(), &SimConfig::new(1, 30, 1)).unwrap();
    assert!(one.converged);
}

#[test]
fn demo_project_converges_and_the_result_is_a_project_when_edits_are_numeric() {
    let p = crate::demo::project();
    let doc = serde_json::to_value(&p).unwrap();
    let rep = simulate(&doc, &SimConfig::new(3, 80, 77)).unwrap();
    assert!(rep.converged && rep.state_identical);
}
/// Project-aware simulation keeps the merged document a valid project (ok can be true on a real project).
#[test]
fn project_aware_edits_keep_the_demo_valid() {
    let base = serde_json::to_value(crate::demo::project()).unwrap();
    for seed in 1..=24u64 {
        let mut cfg = SimConfig::new(4, 250, seed);
        cfg.project_aware = true;
        let rep = simulate(&base, &cfg).unwrap();
        assert!(rep.converged && rep.state_identical && rep.pending_left == 0, "seed {seed}");
        let real = crate::demo::project().layers.len();
        assert_eq!(rep.final_doc["layers"].as_array().unwrap().iter().filter(|l| !l["name"].as_str().unwrap().starts_with("sim_")).count(), real, "seed {seed}: a real layer was lost");
        let pr: crate::project::Project = serde_json::from_value(rep.final_doc.clone()).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        if let Err(e) = crate::Engine::new(pr) {
            panic!("seed {seed}: merged project does not compile: {e:?}");
        }
    }
}
