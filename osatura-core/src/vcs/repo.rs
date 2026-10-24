// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `vcs` (see the module docs in `mod.rs`).
use super::*;

pub const VCS_DIR: &str = ".osatura-vcs";

// ---------------------------------------------------------------------------------------------
// SHA-256 (FIPS 180-4)
// ---------------------------------------------------------------------------------------------

pub(super) const K256: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 digest of `data`.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64).wrapping_mul(8)).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K256[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v[7] = v[6];
            v[6] = v[5];
            v[5] = v[4];
            v[4] = v[3].wrapping_add(t1);
            v[3] = v[2];
            v[2] = v[1];
            v[1] = v[0];
            v[0] = t1.wrapping_add(t2);
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    let mut out = [0u8; 32];
    for i in 0..8 {
        out[4 * i..4 * i + 4].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

pub fn sha256_hex(data: &[u8]) -> String {
    sha256(data).iter().map(|b| format!("{b:02x}")).collect()
}

pub(super) fn is_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// ---------------------------------------------------------------------------------------------
// Object encoding, trees
// ---------------------------------------------------------------------------------------------

pub(super) fn encode_object(kind: &str, payload: &[u8]) -> Vec<u8> {
    let mut out = format!("ARMOBJ1 {kind} {}\n", payload.len()).into_bytes();
    out.extend_from_slice(payload);
    out
}

pub(super) fn decode_object(bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
    let nl = bytes.iter().position(|b| *b == b'\n').ok_or("object has no header")?;
    let head = std::str::from_utf8(&bytes[..nl]).map_err(|_| "object header is not text")?;
    let mut it = head.split(' ');
    if it.next() != Some("ARMOBJ1") {
        return Err("object has a wrong magic".into());
    }
    let kind = it.next().ok_or("object header lacks a kind")?;
    let len: usize = it.next().and_then(|l| l.parse().ok()).ok_or("object header lacks a length")?;
    if it.next().is_some() || !matches!(kind, "blob" | "tree" | "commit") {
        return Err(format!("object header is malformed ({head:?})"));
    }
    let payload = &bytes[nl + 1..];
    if payload.len() != len {
        return Err(format!("object length {} differs from header {len}", payload.len()));
    }
    Ok((kind.to_string(), payload.to_vec()))
}

/// Split a canonical project document into named parts (see the module doc).
pub fn split_parts(doc: &Value) -> Result<BTreeMap<String, Value>, String> {
    let m = doc.as_object().ok_or("a project must be a JSON object")?;
    let mut parts = BTreeMap::new();
    let mut meta = serde_json::Map::new();
    for (k, v) in m {
        match v {
            Value::Array(a) => match semdiff::elem_ids(a) {
                Some(ids) => {
                    parts.insert(format!("order/{k}"), Value::Array(ids.iter().map(|i| Value::String(i.clone())).collect()));
                    for (id, e) in ids.iter().zip(a) {
                        parts.insert(format!("arr/{k}/{id}"), e.clone());
                    }
                }
                None => {
                    parts.insert(format!("val/{k}"), v.clone());
                }
            },
            Value::Object(_) => {
                parts.insert(format!("obj/{k}"), v.clone());
            }
            _ => {
                meta.insert(k.clone(), v.clone());
            }
        }
    }
    parts.insert("meta".into(), Value::Object(meta));
    Ok(parts)
}

/// Inverse of [`split_parts`]; errors when a part is missing or malformed.
pub fn join_parts(parts: &BTreeMap<String, Value>) -> Result<Value, String> {
    let mut out = serde_json::Map::new();
    match parts.get("meta") {
        Some(Value::Object(m)) => out.extend(m.clone()),
        _ => return Err("tree lacks the meta part".into()),
    }
    for (name, v) in parts {
        if let Some(k) = name.strip_prefix("obj/").or_else(|| name.strip_prefix("val/")) {
            out.insert(k.to_string(), v.clone());
        } else if let Some(k) = name.strip_prefix("order/") {
            let ids = v.as_array().ok_or_else(|| format!("part {name} is not a list"))?;
            let mut arr = Vec::new();
            for id in ids {
                let id = id.as_str().ok_or_else(|| format!("part {name} holds a non-text id"))?;
                arr.push(parts.get(&format!("arr/{k}/{id}")).ok_or_else(|| format!("tree lacks part arr/{k}/{id}"))?.clone());
            }
            out.insert(k.to_string(), Value::Array(arr));
        }
    }
    Ok(Value::Object(out))
}

// ---------------------------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub tree: String,
    pub parents: Vec<String>,
    pub author: String,
    pub time: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeState {
    /// "merge", "cherry-pick" or "revert".
    pub kind: String,
    /// Second parent of the resulting commit (merge only).
    pub theirs: Option<String>,
    pub message: String,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Status {
    pub branch: Option<String>,
    pub head: Option<String>,
    pub changes: Vec<Change>,
    pub merge: Option<String>,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergeOutcome {
    UpToDate,
    FastForward(String),
    Merged(String),
    Conflicted(Vec<Conflict>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct GcReport {
    pub kept: usize,
    pub removed: usize,
}

#[derive(Debug)]
pub struct Repo {
    /// Directory holding the project file and `.osatura-vcs/`.
    pub root: PathBuf,
    pub(super) dir: PathBuf,
    /// Timestamp override (seconds since the epoch) for reproducible tests and demos.
    pub fixed_time: Option<u64>,
}

pub(super) fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    crate::fsutil::write_atomic(path, bytes).map_err(|e| format!("cannot write {e}"))
}

pub(super) fn valid_ref_name(n: &str) -> Result<(), String> {
    let ok = !n.is_empty()
        && n.len() <= 100
        && !n.starts_with(['.', '-', '/'])
        && !n.ends_with(['/', '.'])
        && !n.contains("..")
        && !n.contains("//")
        && !n.ends_with(".lock")
        && n.chars().all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c));
    if ok {
        Ok(())
    } else {
        Err(format!("invalid name '{n}' (use letters, digits and ._-/ ; no leading '.', '-' or '/', no '..')"))
    }
}

impl Repo {
    // ----- setup -----

    /// Create a repository next to `project_file` (a path to an existing project JSON).
    pub fn init(project_file: &Path, author: Option<&str>) -> Result<Repo, String> {
        let root = project_file.parent().map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p }).unwrap_or(Path::new(".")).to_path_buf();
        let name = project_file.file_name().and_then(|n| n.to_str()).ok_or("bad project file name")?.to_string();
        let text = fs::read_to_string(project_file).map_err(|e| format!("cannot read {}: {e}", project_file.display()))?;
        let v: Value = serde_json::from_str(&text).map_err(|e| format!("{} is not JSON: {e}", project_file.display()))?;
        if !v.is_object() {
            return Err(format!("{} is not a project (a JSON object is expected)", project_file.display()));
        }
        let dir = root.join(VCS_DIR);
        if dir.exists() {
            return Err(format!("{} already exists", dir.display()));
        }
        for sub in ["objects", "refs/heads", "refs/tags"] {
            fs::create_dir_all(dir.join(sub)).map_err(|e| format!("cannot create {}: {e}", dir.join(sub).display()))?;
        }
        let r = Repo { root, dir, fixed_time: None };
        write_atomic(&r.dir.join("HEAD"), b"ref: refs/heads/main\n")?;
        let cfg = json!({"project": name, "author": author.unwrap_or("")});
        write_atomic(&r.dir.join("config"), semdiff::canon_string(&cfg, true).as_bytes())?;
        Ok(r)
    }

    /// Open the repository containing `start` (walks up through the parent directories).
    pub fn open(start: &Path) -> Result<Repo, String> {
        let abs = fs::canonicalize(start).map_err(|e| format!("cannot open {}: {e}", start.display()))?;
        for d in abs.ancestors() {
            if d.join(VCS_DIR).is_dir() {
                return Ok(Repo { root: d.to_path_buf(), dir: d.join(VCS_DIR), fixed_time: None });
            }
        }
        Err(format!("no {VCS_DIR} repository found from {} upwards (run: osatura vcs init <project.json>)", abs.display()))
    }

    pub(super) fn config(&self) -> Result<Value, String> {
        let t = fs::read_to_string(self.dir.join("config")).map_err(|e| format!("cannot read config: {e}"))?;
        serde_json::from_str(&t).map_err(|e| format!("config is damaged: {e}"))
    }

    /// Absolute path of the tracked project file.
    pub fn project_path(&self) -> Result<PathBuf, String> {
        let c = self.config()?;
        let name = c.get("project").and_then(|p| p.as_str()).ok_or("config lacks the project file name")?;
        let p = Path::new(name);
        if p.is_absolute() || p.components().any(|c| matches!(c, std::path::Component::ParentDir)) || name.is_empty() {
            return Err(format!("config names an unsafe project path '{name}'"));
        }
        Ok(self.root.join(p))
    }

    pub fn default_author(&self) -> String {
        if let Ok(c) = self.config() {
            if let Some(a) = c.get("author").and_then(|a| a.as_str()).filter(|a| !a.is_empty()) {
                return a.to_string();
            }
        }
        std::env::var("OSATURA_AUTHOR").ok().filter(|a| !a.is_empty()).or_else(|| std::env::var("USER").ok().filter(|a| !a.is_empty())).unwrap_or_else(|| "unknown".into())
    }

    pub(super) fn now(&self) -> u64 {
        self.fixed_time.unwrap_or_else(crate::clock::unix_seconds)
    }

    // ----- object store -----

    pub(super) fn obj_path(&self, h: &str) -> Result<PathBuf, String> {
        if !is_hash(h) {
            return Err(format!("'{h}' is not an object hash"));
        }
        Ok(self.dir.join("objects").join(&h[..2]).join(&h[2..]))
    }

    pub fn put_object(&self, kind: &str, payload: &[u8]) -> Result<String, String> {
        let bytes = encode_object(kind, payload);
        let h = sha256_hex(&bytes);
        let p = self.obj_path(&h)?;
        if p.exists() {
            return Ok(h);
        }
        if let Some(d) = p.parent() {
            fs::create_dir_all(d).map_err(|e| format!("cannot create {}: {e}", d.display()))?;
        }
        write_atomic(&p, &bytes)?;
        Ok(h)
    }

    pub fn get_object(&self, h: &str) -> Result<(String, Vec<u8>), String> {
        let bytes = fs::read(self.obj_path(h)?).map_err(|_| format!("object {} is missing", &h[..h.len().min(12)]))?;
        if sha256_hex(&bytes) != h {
            return Err(format!("object {} is corrupt (hash mismatch)", &h[..12]));
        }
        decode_object(&bytes)
    }

    pub(super) fn get_json(&self, h: &str, kind: &str) -> Result<Value, String> {
        let (k, p) = self.get_object(h)?;
        if k != kind {
            return Err(format!("object {} is a {k}, expected a {kind}", &h[..12]));
        }
        serde_json::from_slice(&p).map_err(|e| format!("object {} is not valid JSON: {e}", &h[..12]))
    }

    /// Store a canonical project document as a tree; returns the tree hash.
    pub fn write_tree(&self, doc: &Value) -> Result<String, String> {
        let doc = semdiff::canonicalize(doc);
        let mut names = BTreeMap::new();
        for (name, part) in split_parts(&doc)? {
            names.insert(name, Value::String(self.put_object("blob", semdiff::canon_string(&part, false).as_bytes())?));
        }
        let tree = json!({"parts": Value::Object(names.into_iter().collect())});
        self.put_object("tree", semdiff::canon_string(&tree, false).as_bytes())
    }

    pub fn read_tree(&self, h: &str) -> Result<Value, String> {
        let t = self.get_json(h, "tree")?;
        let names = t.get("parts").and_then(|p| p.as_object()).ok_or("tree has no parts")?;
        let mut parts = BTreeMap::new();
        for (n, ph) in names {
            parts.insert(n.clone(), self.get_json(ph.as_str().ok_or("tree part hash is not text")?, "blob")?);
        }
        join_parts(&parts)
    }

    pub fn read_commit(&self, h: &str) -> Result<Commit, String> {
        let v = self.get_json(h, "commit")?;
        let c: Commit = serde_json::from_value(v).map_err(|e| format!("commit {} is malformed: {e}", &h[..12]))?;
        if !is_hash(&c.tree) || c.parents.iter().any(|p| !is_hash(p)) {
            return Err(format!("commit {} holds a malformed hash", &h[..12]));
        }
        Ok(c)
    }

    pub(super) fn doc_of(&self, commit: &str) -> Result<Value, String> {
        self.read_tree(&self.read_commit(commit)?.tree)
    }

    // ----- refs -----

    pub(super) fn read_ref_file(&self, rel: &str) -> Result<Option<String>, String> {
        let p = self.dir.join(rel);
        match fs::read_to_string(&p) {
            Ok(t) => {
                let h = t.trim().to_string();
                if is_hash(&h) {
                    Ok(Some(h))
                } else {
                    Err(format!("ref {rel} is damaged"))
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("cannot read ref {rel}: {e}")),
        }
    }

    pub(super) fn write_ref(&self, rel: &str, h: &str) -> Result<(), String> {
        let p = self.dir.join(rel);
        if let Some(d) = p.parent() {
            fs::create_dir_all(d).map_err(|e| format!("cannot create {}: {e}", d.display()))?;
        }
        write_atomic(&p, format!("{h}\n").as_bytes())
    }

    /// Current branch name, or `None` for a detached HEAD.
    pub fn current_branch(&self) -> Result<Option<String>, String> {
        let t = fs::read_to_string(self.dir.join("HEAD")).map_err(|e| format!("cannot read HEAD: {e}"))?;
        let t = t.trim();
        if let Some(r) = t.strip_prefix("ref: refs/heads/") {
            valid_ref_name(r)?;
            Ok(Some(r.to_string()))
        } else if is_hash(t) {
            Ok(None)
        } else {
            Err("HEAD is damaged".into())
        }
    }

    pub fn head_commit(&self) -> Result<Option<String>, String> {
        match self.current_branch()? {
            Some(b) => self.read_ref_file(&format!("refs/heads/{b}")),
            None => {
                let t = fs::read_to_string(self.dir.join("HEAD")).map_err(|e| format!("cannot read HEAD: {e}"))?;
                Ok(Some(t.trim().to_string()))
            }
        }
    }

    pub(super) fn set_head_commit(&self, h: &str) -> Result<(), String> {
        match self.current_branch()? {
            Some(b) => self.write_ref(&format!("refs/heads/{b}"), h),
            None => write_atomic(&self.dir.join("HEAD"), format!("{h}\n").as_bytes()),
        }
    }

    pub(super) fn list_refs(&self, sub: &str) -> Result<Vec<(String, String)>, String> {
        let base = self.dir.join(sub);
        let mut out = Vec::new();
        fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) -> Result<(), String> {
            let rd = match fs::read_dir(dir) {
                Ok(r) => r,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(e) => return Err(format!("cannot list {}: {e}", dir.display())),
            };
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                let full = if prefix.is_empty() { name.clone() } else { format!("{prefix}/{name}") };
                if e.path().is_dir() {
                    walk(&e.path(), &full, out)?;
                } else if !name.starts_with('.') {
                    out.push(full);
                }
            }
            Ok(())
        }
        let mut names = Vec::new();
        walk(&base, "", &mut names)?;
        names.sort();
        for n in names {
            if let Some(h) = self.read_ref_file(&format!("{sub}/{n}"))? {
                out.push((n, h));
            }
        }
        Ok(out)
    }

    pub fn branches(&self) -> Result<Vec<(String, String, bool)>, String> {
        let cur = self.current_branch()?;
        Ok(self.list_refs("refs/heads")?.into_iter().map(|(n, h)| {
            let c = cur.as_deref() == Some(n.as_str());
            (n, h, c)
        }).collect())
    }

    pub fn tags(&self) -> Result<Vec<(String, String)>, String> {
        self.list_refs("refs/tags")
    }

    /// Resolve a branch, tag, `HEAD`, a hash prefix (at least 4 hex digits), each optionally followed
    /// by `~N` (N-th first-parent ancestor) to a commit hash.
    pub fn resolve_rev(&self, rev: &str) -> Result<String, String> {
        let (base, back) = match rev.split_once('~') {
            Some((b, n)) => (b, if n.is_empty() { 1 } else { n.parse::<usize>().map_err(|_| format!("bad revision '{rev}'"))? }),
            None => (rev, 0),
        };
        let mut h = self.resolve_base(base)?;
        for _ in 0..back {
            h = self.read_commit(&h)?.parents.first().cloned().ok_or_else(|| format!("'{rev}' goes past the first commit"))?;
        }
        Ok(h)
    }

    pub(super) fn resolve_base(&self, s: &str) -> Result<String, String> {
        if s == "HEAD" {
            return self.head_commit()?.ok_or_else(|| "there are no commits yet".to_string());
        }
        if valid_ref_name(s).is_ok() {
            if let Some(h) = self.read_ref_file(&format!("refs/heads/{s}"))? {
                return Ok(h);
            }
            if let Some(h) = self.read_ref_file(&format!("refs/tags/{s}"))? {
                return Ok(h);
            }
        }
        if s.len() >= 4 && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            let mut hits = Vec::new();
            for h in self.all_object_names()? {
                if h.starts_with(s) && self.get_object(&h).map(|o| o.0 == "commit").unwrap_or(false) {
                    hits.push(h);
                }
            }
            return match hits.len() {
                1 => Ok(hits.remove(0)),
                0 => Err(format!("unknown revision '{s}'")),
                _ => Err(format!("revision prefix '{s}' is ambiguous")),
            };
        }
        Err(format!("unknown revision '{s}'"))
    }

    pub(super) fn all_object_names(&self) -> Result<Vec<String>, String> {
        let mut out = Vec::new();
        let rd = fs::read_dir(self.dir.join("objects")).map_err(|e| format!("cannot list objects: {e}"))?;
        for d in rd.flatten() {
            let dn = d.file_name().to_string_lossy().to_string();
            if dn.len() != 2 || !d.path().is_dir() {
                continue;
            }
            if let Ok(rd2) = fs::read_dir(d.path()) {
                for f in rd2.flatten() {
                    let n = format!("{dn}{}", f.file_name().to_string_lossy());
                    if is_hash(&n) {
                        out.push(n);
                    }
                }
            }
        }
        out.sort();
        Ok(out)
    }

    // ----- working file -----

    pub fn read_working(&self) -> Result<Value, String> {
        let p = self.project_path()?;
        let t = fs::read_to_string(&p).map_err(|e| format!("cannot read {}: {e}", p.display()))?;
        let v: Value = serde_json::from_str(&t).map_err(|e| format!("{} is not valid JSON: {e}", p.display()))?;
        if !v.is_object() {
            return Err(format!("{} is not a project (a JSON object is expected)", p.display()));
        }
        Ok(semdiff::canonicalize(&v))
    }

    /// Replace the project file atomically (temporary file in the same directory, then rename).
    pub fn write_working(&self, doc: &Value) -> Result<(), String> {
        write_atomic(&self.project_path()?, semdiff::canon_string(&semdiff::canonicalize(doc), true).as_bytes())
    }

    /// Whether the working file differs from the commit at HEAD (an unborn branch counts as clean
    /// only if there is no working file to lose; use `status` for details).
    pub fn is_dirty(&self) -> Result<bool, String> {
        let w = self.read_working()?;
        match self.head_commit()? {
            Some(h) => Ok(w != self.doc_of(&h)?),
            None => Ok(true),
        }
    }

    // ----- merge state -----

    pub(super) fn load_state(&self) -> Result<Option<MergeState>, String> {
        match fs::read_to_string(self.dir.join("MERGE_STATE")) {
            Ok(t) => Ok(Some(serde_json::from_str(&t).map_err(|e| format!("MERGE_STATE is damaged: {e}"))?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("cannot read MERGE_STATE: {e}")),
        }
    }

    pub(super) fn save_state(&self, s: &MergeState) -> Result<(), String> {
        let v = serde_json::to_value(s).map_err(|e| e.to_string())?;
        write_atomic(&self.dir.join("MERGE_STATE"), semdiff::canon_string(&v, true).as_bytes())
    }

    pub(super) fn clear_state(&self) -> Result<(), String> {
        match fs::remove_file(self.dir.join("MERGE_STATE")) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("cannot remove MERGE_STATE: {e}")),
        }
    }

    pub fn merge_state(&self) -> Result<Option<MergeState>, String> {
        self.load_state()
    }

    pub(super) fn require_no_merge(&self) -> Result<(), String> {
        if self.load_state()?.is_some() {
            return Err("a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'".into());
        }
        Ok(())
    }

    // ----- commits -----

    pub(super) fn make_commit(&self, doc: &Value, parents: Vec<String>, message: &str, author: Option<&str>) -> Result<String, String> {
        let tree = self.write_tree(doc)?;
        let c = Commit { tree, parents, author: author.map(String::from).unwrap_or_else(|| self.default_author()), time: self.now(), message: message.to_string() };
        let v = serde_json::to_value(&c).map_err(|e| e.to_string())?;
        self.put_object("commit", semdiff::canon_string(&v, false).as_bytes())
    }

    /// Commit the working file on the current branch (or HEAD). With a merge in progress this
    /// creates the merge commit once all conflicts are resolved.
    pub fn commit(&self, message: Option<&str>, author: Option<&str>) -> Result<String, String> {
        let doc = self.read_working()?;
        let head = self.head_commit()?;
        let mut parents: Vec<String> = head.iter().cloned().collect();
        let mut msg = message.map(String::from);
        if let Some(st) = self.load_state()? {
            if !st.conflicts.is_empty() {
                return Err(format!("{} unresolved conflict(s): resolve them with 'vcs resolve <path> --ours|--theirs|--value <json>'", st.conflicts.len()));
            }
            if let Some(t) = &st.theirs {
                parents.push(t.clone());
            }
            msg.get_or_insert(st.message.clone());
        } else if let Some(h) = &head {
            if doc == self.doc_of(h)? {
                return Err("nothing to commit (the project is unchanged)".into());
            }
        }
        let msg = msg.filter(|m| !m.trim().is_empty()).ok_or("a commit message is required (-m)")?;
        let h = self.make_commit(&doc, parents, &msg, author)?;
        self.set_head_commit(&h)?;
        self.clear_state()?;
        Ok(h)
    }

    /// Commits reachable from `rev`, newest first (a commit never precedes one of its children).
    pub fn log(&self, rev: &str, limit: usize) -> Result<Vec<(String, Commit)>, String> {
        let start = self.resolve_rev(rev)?;
        let mut all: HashMap<String, Commit> = HashMap::new();
        let mut stack = vec![start.clone()];
        while let Some(h) = stack.pop() {
            if all.contains_key(&h) {
                continue;
            }
            let c = self.read_commit(&h)?;
            stack.extend(c.parents.iter().cloned());
            all.insert(h, c);
        }
        let mut children: HashMap<&str, usize> = HashMap::new();
        for c in all.values() {
            for p in &c.parents {
                *children.entry(p.as_str()).or_insert(0) += 1;
            }
        }
        let mut ready: Vec<&String> = all.keys().filter(|h| !children.contains_key(h.as_str())).collect();
        let mut out = Vec::new();
        while !ready.is_empty() && out.len() < limit {
            ready.sort_by(|a, b| (all[*a].time, a.as_str()).cmp(&(all[*b].time, b.as_str())));
            let h = ready.pop().unwrap();
            let c = &all[h];
            out.push((h.clone(), c.clone()));
            for p in &c.parents {
                if let Some(n) = children.get_mut(p.as_str()) {
                    *n -= 1;
                    if *n == 0 {
                        ready.push(all.get_key_value(p).map(|x| x.0).unwrap());
                    }
                }
            }
        }
        Ok(out)
    }

    pub(super) fn ancestors(&self, h: &str) -> Result<HashSet<String>, String> {
        let mut seen = HashSet::new();
        let mut stack = vec![h.to_string()];
        while let Some(x) = stack.pop() {
            if seen.insert(x.clone()) {
                stack.extend(self.read_commit(&x)?.parents);
            }
        }
        Ok(seen)
    }

    /// Best common ancestor of two commits (lowest in the DAG; the newest if there are several).
    pub fn merge_base(&self, a: &str, b: &str) -> Result<Option<String>, String> {
        let (aa, ab) = (self.ancestors(a)?, self.ancestors(b)?);
        let common: Vec<&String> = aa.intersection(&ab).collect();
        let mut redundant: HashSet<String> = HashSet::new();
        for c in &common {
            let mut anc = self.ancestors(c)?;
            anc.remove(*c);
            redundant.extend(anc);
        }
        let mut best: Vec<&&String> = common.iter().filter(|c| !redundant.contains(**c)).collect();
        let mut keyed = Vec::new();
        for c in best.drain(..) {
            keyed.push((self.read_commit(c)?.time, (**c).clone()));
        }
        keyed.sort();
        Ok(keyed.pop().map(|x| x.1))
    }

}
