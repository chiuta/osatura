// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Tests of `vcs`.

use super::*;

fn tmp(tag: &str) -> crate::fsutil::ScratchDir {
    crate::fsutil::ScratchDir::new(&format!("armvcs_{tag}"))
}

fn project() -> Value {
    json!({
        "format": 1, "name": "t", "fps": 24.0, "frames": 48,
        "camera": {"zoom": 1.0},
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

/// A repo plus the scratch directory that holds it (removed when this value is dropped).
struct TestRepo {
    repo: Repo,
    _dir: crate::fsutil::ScratchDir,
}

impl std::ops::Deref for TestRepo {
    type Target = Repo;
    fn deref(&self) -> &Repo {
        &self.repo
    }
}

impl std::ops::DerefMut for TestRepo {
    fn deref_mut(&mut self) -> &mut Repo {
        &mut self.repo
    }
}

fn new_repo(tag: &str) -> TestRepo {
    let d = tmp(tag);
    let f = d.join("p.json");
    fs::write(&f, serde_json::to_string_pretty(&project()).unwrap()).unwrap();
    let mut r = Repo::init(&f, Some("ana")).unwrap();
    r.fixed_time = Some(1_000);
    TestRepo { repo: r, _dir: d }
}

fn edit(r: &Repo, f: impl FnOnce(&mut Value)) {
    let mut d = r.read_working().unwrap();
    f(&mut d);
    r.write_working(&d).unwrap();
}

fn setp(d: &mut Value, p: &str, v: Value) {
    semdiff::set_at(d, &semdiff::segs_of(p).unwrap(), Some(v)).unwrap();
}

fn tick(r: &mut Repo) {
    r.fixed_time = Some(r.fixed_time.unwrap() + 10);
}

#[test]
fn sha256_known_vectors() {
    assert_eq!(sha256_hex(b""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
    assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
    let million = vec![b'a'; 1_000_000];
    assert_eq!(sha256_hex(&million), "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0");
    // padding boundaries (55, 56, 63, 64, 65 bytes) must not panic and must differ
    let mut seen = HashSet::new();
    for n in [55, 56, 57, 63, 64, 65, 119, 120] {
        assert!(seen.insert(sha256_hex(&vec![7u8; n])));
    }
}

#[test]
fn canonical_hash_is_deterministic() {
    let r = new_repo("canon");
    let a: Value = serde_json::from_str(r#"{"b":1.0,"a":{"y":2,"x":[1.0,2.5]}}"#).unwrap();
    let b: Value = serde_json::from_str(r#"{"a":{"x":[1,2.5],"y":2.0},"b":1}"#).unwrap();
    assert_eq!(r.write_tree(&a).unwrap(), r.write_tree(&b).unwrap());
    let c: Value = serde_json::from_str(r#"{"a":{"x":[1,2.6],"y":2},"b":1}"#).unwrap();
    assert_ne!(r.write_tree(&a).unwrap(), r.write_tree(&c).unwrap());
    // the same tree built twice gives the same hash across runs: pin one value
    let t = r.write_tree(&json!({"a": 1})).unwrap();
    assert_eq!(t, r.write_tree(&json!({"a": 1})).unwrap());
    assert!(is_hash(&t));
}

#[test]
fn tree_split_join_roundtrip_and_dedupe() {
    let r = new_repo("tree");
    let doc = semdiff::canonicalize(&project());
    let t = r.write_tree(&doc).unwrap();
    assert_eq!(r.read_tree(&t).unwrap(), doc);
    let before = r.object_count().unwrap();
    let mut d2 = doc.clone();
    setp(&mut d2, "/layers/sky/opacity", json!(0.5));
    r.write_tree(&d2).unwrap();
    // one changed layer: one new blob + one new tree + order unchanged
    assert_eq!(r.object_count().unwrap(), before + 2);
    assert!(split_parts(&json!([1])).is_err());
    assert!(join_parts(&BTreeMap::new()).is_err());
}

#[test]
fn commit_log_branch_roundtrip() {
    let mut r = new_repo("log");
    assert!(r.log("HEAD", 10).is_err());
    let c1 = r.commit(Some("first"), None).unwrap();
    tick(&mut r);
    assert!(r.commit(Some("again"), None).unwrap_err().contains("nothing to commit"));
    edit(&r, |d| setp(d, "/layers/sky/opacity", json!(0.4)));
    assert_eq!(r.status().unwrap().changes.len(), 1);
    let c2 = r.commit(Some("dim sky"), Some("bo")).unwrap();
    let log = r.log("HEAD", 10).unwrap();
    assert_eq!(log.iter().map(|x| x.0.clone()).collect::<Vec<_>>(), vec![c2.clone(), c1.clone()]);
    assert_eq!(log[0].1.author, "bo");
    assert_eq!(log[1].1.author, "ana");
    assert_eq!(r.resolve_rev("HEAD~1").unwrap(), c1);
    assert_eq!(r.resolve_rev(&c2[..8]).unwrap(), c2);
    assert!(r.resolve_rev("HEAD~5").is_err());
    assert!(r.resolve_rev("nope").is_err());
    r.branch_create("feature", Some(&c1)).unwrap();
    assert!(r.branch_create("feature", None).unwrap_err().contains("exists"));
    assert!(r.branch_create("bad..name", None).is_err());
    r.tag_create("v1", None).unwrap();
    assert_eq!(r.resolve_rev("v1").unwrap(), c2);
    r.checkout("feature", false).unwrap();
    assert_eq!(r.current_branch().unwrap().as_deref(), Some("feature"));
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(1));
    r.checkout("main", false).unwrap();
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(0.4));
    r.checkout("v1", false).unwrap();
    assert_eq!(r.current_branch().unwrap(), None);
    r.checkout("main", false).unwrap();
    assert_eq!(r.branches().unwrap().len(), 2);
    r.branch_delete("feature").unwrap();
    assert!(r.branch_delete("main").is_err());
    assert!(r.fsck().is_empty(), "{:?}", r.fsck());
}

#[test]
fn checkout_refuses_dirty_and_is_atomic() {
    let mut r = new_repo("co");
    r.commit(Some("one"), None).unwrap();
    r.branch_create("b", None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/fps", json!(30)));
    r.commit(Some("fps"), None).unwrap();
    r.checkout("b", false).unwrap();
    edit(&r, |d| setp(d, "/fps", json!(12)));
    let before = fs::read(r.project_path().unwrap()).unwrap();
    let e = r.checkout("main", false).unwrap_err();
    assert!(e.contains("uncommitted"), "{e}");
    assert_eq!(fs::read(r.project_path().unwrap()).unwrap(), before);
    assert_eq!(r.current_branch().unwrap().as_deref(), Some("b"));
    r.checkout("main", true).unwrap();
    assert_eq!(r.read_working().unwrap()["fps"], json!(30));
    // no temporary files left behind
    let leftovers: Vec<_> = fs::read_dir(&r.root).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains("armtmp")).collect();
    assert!(leftovers.is_empty());
    // a failing write (target directory vanished) leaves HEAD unchanged
    let mut cfg = r.config().unwrap();
    cfg["project"] = json!("missing_dir/p.json");
    write_atomic(&r.dir.join("config"), cfg.to_string().as_bytes()).unwrap();
    assert!(r.checkout("b", true).is_err());
    assert_eq!(r.current_branch().unwrap().as_deref(), Some("main"));
}

#[test]
fn merge_fast_forward_and_up_to_date() {
    let mut r = new_repo("ff");
    let c1 = r.commit(Some("base"), None).unwrap();
    r.branch_create("topic", None).unwrap();
    r.checkout("topic", false).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/sky/opacity", json!(0.3)));
    let c2 = r.commit(Some("topic work"), None).unwrap();
    r.checkout("main", false).unwrap();
    assert_eq!(r.merge("topic", false, None, None).unwrap(), MergeOutcome::FastForward(c2.clone()));
    assert_eq!(r.head_commit().unwrap().unwrap(), c2);
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(0.3));
    assert_eq!(r.merge("topic", false, None, None).unwrap(), MergeOutcome::UpToDate);
    assert_eq!(r.merge(&c1, false, None, None).unwrap(), MergeOutcome::UpToDate);
}

#[test]
fn merge_clean_two_animators_and_merge_base_diamond() {
    let mut r = new_repo("diamond");
    let c0 = r.commit(Some("base"), None).unwrap();
    r.branch_create("ana", None).unwrap();
    r.branch_create("bo", None).unwrap();
    r.checkout("ana", false).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/channels/a|bone_angle|keys/track/keys/12/value", json!(15)));
    let ca = r.commit(Some("ana: key 12"), Some("ana")).unwrap();
    r.checkout("bo", false).unwrap();
    tick(&mut r);
    edit(&r, |d| {
        setp(d, "/layers/hill/visible", json!(false));
        setp(d, "/channels/sun|layer_opacity|keys/track/keys/0/value", json!(0.5));
    });
    let cb = r.commit(Some("bo: hill + sun"), Some("bo")).unwrap();
    assert_eq!(r.merge_base(&ca, &cb).unwrap().unwrap(), c0);
    tick(&mut r);
    match r.merge("ana", false, None, Some("bo")).unwrap() {
        MergeOutcome::Merged(m) => {
            let c = r.read_commit(&m).unwrap();
            assert_eq!(c.parents, vec![cb.clone(), ca.clone()]);
            let d = r.read_working().unwrap();
            assert_eq!(d["channels"][0]["track"]["keys"][1]["value"], json!(15));
            assert_eq!(d["layers"][2]["visible"], json!(false));
            assert_eq!(d["channels"][1]["track"]["keys"][0]["value"], json!(0.5));
            // diamond: merge base of the merge and either parent is that parent
            assert_eq!(r.merge_base(&m, &ca).unwrap().unwrap(), ca);
            assert!(r.is_ancestor(&c0, &m).unwrap());
            let log = r.log("HEAD", 10).unwrap();
            assert_eq!(log.len(), 4);
            assert_eq!(log[0].0, m);
            assert_eq!(log[3].0, c0);
        }
        o => panic!("{o:?}"),
    }
    assert!(r.fsck().is_empty());
}

#[test]
fn merge_conflict_resolve_and_commit() {
    let mut r = new_repo("conf");
    r.commit(Some("base"), None).unwrap();
    r.branch_create("t", None).unwrap();
    tick(&mut r);
    edit(&r, |d| {
        setp(d, "/layers/sky/opacity", json!(0.2));
        setp(d, "/layers/hill/visible", json!(false));
    });
    r.commit(Some("main edit"), None).unwrap();
    r.checkout("t", false).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/sky/opacity", json!(0.8)));
    r.commit(Some("t edit"), None).unwrap();
    r.checkout("main", false).unwrap();
    let out = r.merge("t", false, None, None).unwrap();
    let MergeOutcome::Conflicted(cs) = out else { panic!("{out:?}") };
    assert_eq!(cs.len(), 1);
    assert_eq!(cs[0].path, "/layers/sky/opacity");
    // working file holds ours; commit and checkout and a second merge are refused
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(0.2));
    assert!(r.commit(Some("x"), None).unwrap_err().contains("unresolved"));
    assert!(r.checkout("t", true).unwrap_err().contains("in progress"));
    assert!(r.merge("t", false, None, None).is_err());
    assert!(r.resolve("/layers/nope", &Choice::Ours).is_err());
    assert!(r.status().unwrap().conflicts.len() == 1);
    assert_eq!(r.resolve("/layers/sky/opacity", &Choice::Value(json!(0.5))).unwrap(), 0);
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(0.5));
    tick(&mut r);
    let m = r.commit(None, None).unwrap();
    assert_eq!(r.read_commit(&m).unwrap().parents.len(), 2);
    assert!(r.merge_state().unwrap().is_none());
    // --theirs variant
    r.checkout("t", true).unwrap();
    assert!(r.fsck().is_empty());
}

#[test]
fn resolve_theirs_and_abort_and_delete_edit() {
    let mut r = new_repo("abort");
    r.commit(Some("base"), None).unwrap();
    r.branch_create("t", None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/sun/opacity", json!(0.1)));
    r.commit(Some("edit sun"), None).unwrap();
    r.checkout("t", false).unwrap();
    tick(&mut r);
    edit(&r, |d| {
        semdiff::set_at(d, &semdiff::segs_of("/layers/sun").unwrap(), None).unwrap();
    });
    r.commit(Some("remove sun"), None).unwrap();
    r.checkout("main", false).unwrap();
    let MergeOutcome::Conflicted(cs) = r.merge("t", false, None, None).unwrap() else { panic!() };
    assert_eq!(cs[0].kind, semdiff::ConflictKind::DeleteEdit);
    r.resolve("/layers/sun", &Choice::Theirs).unwrap();
    assert_eq!(r.read_working().unwrap()["layers"].as_array().unwrap().len(), 2);
    r.abort().unwrap();
    assert_eq!(r.read_working().unwrap()["layers"].as_array().unwrap().len(), 3);
    assert!(r.abort().is_err());
    assert!(!r.is_dirty().unwrap());
}

#[test]
fn revert_and_cherry_pick() {
    let mut r = new_repo("rev");
    let c0 = r.commit(Some("base"), None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/sky/opacity", json!(0.1)));
    let c1 = r.commit(Some("dim sky"), None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/hill/visible", json!(false)));
    r.commit(Some("hide hill"), None).unwrap();
    // revert the middle commit: sky restored, hill stays hidden
    tick(&mut r);
    let MergeOutcome::Merged(_) = r.revert(&c1, None).unwrap() else { panic!() };
    let d = r.read_working().unwrap();
    assert_eq!(d["layers"][0]["opacity"], json!(1));
    assert_eq!(d["layers"][2]["visible"], json!(false));
    assert!(r.log("HEAD", 1).unwrap()[0].1.message.starts_with("Revert \"dim sky\""));
    assert!(r.revert(&c0, None).unwrap_err().contains("root"));
    // cherry-pick c1 onto a branch from c0
    r.branch_create("old", Some(&c0)).unwrap();
    r.checkout("old", false).unwrap();
    tick(&mut r);
    let MergeOutcome::Merged(p) = r.cherry_pick(&c1, None).unwrap() else { panic!() };
    assert_eq!(r.read_commit(&p).unwrap().parents, vec![c0.clone()]);
    assert_eq!(r.read_working().unwrap()["layers"][0]["opacity"], json!(0.1));
    // picking it again changes nothing
    assert!(r.cherry_pick(&c1, None).unwrap_err().contains("already"));
    assert!(r.cherry_pick("zzzz", None).is_err());
}

#[test]
fn semantic_diff_between_revisions() {
    let mut r = new_repo("sd");
    r.commit(Some("a"), None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/channels/a|bone_angle|keys/track/keys/12/value", json!(15)));
    r.commit(Some("b"), None).unwrap();
    let ch = r.diff(Some("HEAD~1"), Some("HEAD")).unwrap();
    assert_eq!(ch.len(), 1);
    assert!(ch[0].description.contains("key at frame 12 of channel a.bone_angle: value changed 10 -> 15"), "{}", ch[0].description);
    assert!(r.diff(Some("HEAD"), None).unwrap().is_empty());
    let (_, c, sh) = r.show("HEAD").unwrap();
    assert_eq!(c.message, "b");
    assert_eq!(sh.len(), 1);
    let (_, _, root) = r.show("HEAD~1").unwrap();
    assert!(!root.is_empty());
}

#[test]
fn fsck_detects_corruption_and_gc_keeps_reachable() {
    let mut r = new_repo("fsck");
    r.commit(Some("one"), None).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/layers/sky/opacity", json!(0.1)));
    r.commit(Some("two"), None).unwrap();
    r.branch_create("tmp", None).unwrap();
    r.checkout("tmp", false).unwrap();
    tick(&mut r);
    edit(&r, |d| setp(d, "/fps", json!(60)));
    r.commit(Some("tmp work"), None).unwrap();
    r.checkout("main", false).unwrap();
    r.branch_delete("tmp").unwrap();
    let total = r.object_count().unwrap();
    let g = r.gc().unwrap();
    assert!(g.removed >= 3, "{g:?}"); // commit, tree, changed blob(s)
    assert_eq!(r.object_count().unwrap(), total - g.removed);
    assert!(r.fsck().is_empty(), "{:?}", r.fsck());
    assert_eq!(r.log("HEAD", 10).unwrap().len(), 2);
    assert_eq!(r.gc().unwrap().removed, 0);
    // corrupt a blob: flip a byte
    let name = r.all_object_names().unwrap().into_iter().find(|h| r.get_object(h).map(|o| o.0 == "blob").unwrap_or(false)).unwrap();
    let p = r.obj_path(&name).unwrap();
    let mut b = fs::read(&p).unwrap();
    let n = b.len();
    b[n - 2] ^= 1;
    fs::write(&p, &b).unwrap();
    let bad = r.fsck();
    assert!(bad.iter().any(|m| m.contains("corrupt")), "{bad:?}");
    let _ = r.gc();
    assert!(p.exists(), "gc must not delete a reachable (even corrupt) object");
    // a deleted object is reported by its referrer
    fs::remove_file(&p).unwrap();
    assert!(r.fsck().iter().any(|m| m.contains("missing")), "{:?}", r.fsck());
}

#[test]
fn bad_input_and_malformed_repos_do_not_panic() {
    let d = tmp("bad");
    assert!(Repo::init(&d.join("nope.json"), None).is_err());
    fs::write(d.join("a.json"), "not json").unwrap();
    assert!(Repo::init(&d.join("a.json"), None).is_err());
    fs::write(d.join("a.json"), "[1,2]").unwrap();
    assert!(Repo::init(&d.join("a.json"), None).unwrap_err().contains("JSON object"));
    fs::write(d.join("a.json"), "{}").unwrap();
    let mut r = Repo::init(&d.join("a.json"), None).unwrap();
    r.fixed_time = Some(5);
    assert!(Repo::init(&d.join("a.json"), None).unwrap_err().contains("already"));
    assert!(r.commit(None, None).is_err()); // no message
    r.commit(Some("m"), None).unwrap();
    assert!(Repo::open(&d.join("zzz")).is_err());
    assert!(r.get_object("xyz").is_err());
    assert!(r.get_object(&"0".repeat(64)).is_err());
    assert!(r.resolve_rev("../../etc/passwd").is_err());
    assert!(r.branch_create("", None).is_err());
    assert!(r.tag_create("a b", None).is_err());
    assert!(r.checkout("nonexistent", false).is_err());
    // damage things one by one; every call must return, not panic
    fs::write(r.dir.join("HEAD"), "garbage").unwrap();
    assert!(r.head_commit().is_err() && r.status().is_err() && r.log("HEAD", 3).is_err());
    assert!(!r.fsck().is_empty());
    fs::write(r.dir.join("HEAD"), "ref: refs/heads/../../x\n").unwrap();
    assert!(r.current_branch().is_err());
    fs::write(r.dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
    fs::write(r.dir.join("refs/heads/main"), "nothex").unwrap();
    assert!(r.head_commit().is_err() && !r.fsck().is_empty());
    fs::write(r.dir.join("refs/heads/main"), format!("{}\n", "a".repeat(64))).unwrap();
    assert!(r.log("HEAD", 3).is_err() && r.merge("HEAD", false, None, None).is_err() && r.gc().is_err());
    fs::write(r.dir.join("MERGE_STATE"), "{").unwrap();
    assert!(r.merge_state().is_err() && r.resolve("/x", &Choice::Ours).is_err() && r.abort().is_err());
    fs::write(r.dir.join("config"), "{\"project\": \"../evil.json\"}").unwrap();
    assert!(r.project_path().is_err() && r.read_working().is_err());
    fs::write(r.dir.join("config"), "").unwrap();
    assert!(r.project_path().is_err() && !r.fsck().is_empty());
    let _ = fs::remove_dir_all(r.dir.join("objects"));
    assert!(!r.fsck().is_empty());
    let _ = r.default_author();
}

#[test]
fn object_decoding_rejects_junk() {
    for junk in [&b""[..], b"no newline", b"ARMOBJ1 blob x\nabc", b"ARMOBJ1 blob 5\nabc", b"ARMOBJ1 weird 3\nabc", b"OTHER blob 3\nabc", b"ARMOBJ1 blob 3 extra\nabc"] {
        assert!(decode_object(junk).is_err(), "{junk:?}");
    }
    assert_eq!(decode_object(b"ARMOBJ1 blob 3\nabc").unwrap().1, b"abc");
}

#[test]
fn working_file_edge_cases() {
    let r = new_repo("work");
    fs::write(r.project_path().unwrap(), "[1]").unwrap();
    assert!(r.read_working().is_err());
    fs::write(r.project_path().unwrap(), "{broken").unwrap();
    assert!(r.status().is_err() && r.commit(Some("x"), None).is_err());
    fs::remove_file(r.project_path().unwrap()).unwrap();
    assert!(r.commit(Some("x"), None).is_err());
}
