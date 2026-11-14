// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `vcs` (see the module docs in `mod.rs`).
use super::*;

impl Repo {
    pub fn is_ancestor(&self, anc: &str, of: &str) -> Result<bool, String> {
        Ok(self.ancestors(of)?.contains(anc))
    }

    // ----- status, diff, show -----

    pub fn status(&self) -> Result<Status, String> {
        let working = self.read_working()?;
        let head = self.head_commit()?;
        let base = match &head {
            Some(h) => self.doc_of(h)?,
            None => Value::Object(Default::default()),
        };
        let st = self.load_state()?;
        Ok(Status {
            branch: self.current_branch()?,
            head,
            changes: semdiff::diff_docs(&base, &working),
            merge: st.as_ref().map(|s| s.kind.clone()),
            conflicts: st.map(|s| s.conflicts).unwrap_or_default(),
        })
    }

    /// Document at a revision, or the working file for `None`.
    pub(super) fn doc_at(&self, rev: Option<&str>) -> Result<Value, String> {
        match rev {
            Some(r) => self.doc_of(&self.resolve_rev(r)?),
            None => self.read_working(),
        }
    }

    /// Semantic differences between two revisions; `None` means the working file.
    pub fn diff(&self, a: Option<&str>, b: Option<&str>) -> Result<Vec<Change>, String> {
        Ok(semdiff::diff_docs(&self.doc_at(a)?, &self.doc_at(b)?))
    }

    /// Changes a commit made relative to its first parent (everything for a root commit).
    pub fn show(&self, rev: &str) -> Result<(String, Commit, Vec<Change>), String> {
        let h = self.resolve_rev(rev)?;
        let c = self.read_commit(&h)?;
        let new = self.read_tree(&c.tree)?;
        let old = match c.parents.first() {
            Some(p) => self.doc_of(p)?,
            None => Value::Object(Default::default()),
        };
        Ok((h, c, semdiff::diff_docs(&old, &new)))
    }

    // ----- branches, tags, checkout -----

    pub fn branch_create(&self, name: &str, at: Option<&str>) -> Result<String, String> {
        valid_ref_name(name)?;
        if self.read_ref_file(&format!("refs/heads/{name}"))?.is_some() {
            return Err(format!("branch '{name}' already exists"));
        }
        let h = self.resolve_rev(at.unwrap_or("HEAD"))?;
        self.write_ref(&format!("refs/heads/{name}"), &h)?;
        Ok(h)
    }

    pub fn branch_delete(&self, name: &str) -> Result<(), String> {
        valid_ref_name(name)?;
        if self.current_branch()?.as_deref() == Some(name) {
            return Err("cannot delete the checked-out branch".into());
        }
        if self.read_ref_file(&format!("refs/heads/{name}"))?.is_none() {
            return Err(format!("no branch '{name}'"));
        }
        fs::remove_file(self.dir.join("refs/heads").join(name)).map_err(|e| format!("cannot delete branch: {e}"))
    }

    pub fn tag_create(&self, name: &str, at: Option<&str>) -> Result<String, String> {
        valid_ref_name(name)?;
        if self.read_ref_file(&format!("refs/tags/{name}"))?.is_some() {
            return Err(format!("tag '{name}' already exists"));
        }
        let h = self.resolve_rev(at.unwrap_or("HEAD"))?;
        self.write_ref(&format!("refs/tags/{name}"), &h)?;
        Ok(h)
    }

    /// Switch the working file to a branch (stays on it), or to any other revision (detached).
    /// Refuses when the working file has uncommitted changes unless `force`.
    pub fn checkout(&self, target: &str, force: bool) -> Result<String, String> {
        self.require_no_merge()?;
        let h = self.resolve_rev(target)?;
        if !force && self.head_commit()?.is_some() && self.is_dirty()? {
            return Err("the project has uncommitted changes (commit them or use --force)".into());
        }
        let doc = self.doc_of(&h)?;
        self.write_working(&doc)?;
        if valid_ref_name(target).is_ok() && self.read_ref_file(&format!("refs/heads/{target}"))?.is_some() {
            write_atomic(&self.dir.join("HEAD"), format!("ref: refs/heads/{target}\n").as_bytes())?;
        } else {
            write_atomic(&self.dir.join("HEAD"), format!("{h}\n").as_bytes())?;
        }
        Ok(h)
    }

    // ----- merge, cherry-pick, revert -----

    pub(super) fn require_clean(&self) -> Result<(), String> {
        if self.is_dirty()? {
            return Err("the project has uncommitted changes: commit them first".into());
        }
        Ok(())
    }

    // internal merge helper threading one merge context; kept flat to match its callers.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn apply_three_way(&self, kind: &str, base: &Value, ours: &Value, theirs: &Value, second_parent: Option<String>, message: String, author: Option<&str>) -> Result<MergeOutcome, String> {
        let (merged, conflicts) = semdiff::merge_docs(base, ours, theirs);
        self.write_working(&merged)?;
        if conflicts.is_empty() {
            if merged == semdiff::canonicalize(ours) && second_parent.is_none() {
                return Err(format!("nothing to {kind}: the change is already present"));
            }
            let mut parents: Vec<String> = self.head_commit()?.into_iter().collect();
            parents.extend(second_parent);
            let h = self.make_commit(&merged, parents, &message, author)?;
            self.set_head_commit(&h)?;
            return Ok(MergeOutcome::Merged(h));
        }
        self.save_state(&MergeState { kind: kind.to_string(), theirs: second_parent, message, conflicts: conflicts.clone() })?;
        Ok(MergeOutcome::Conflicted(conflicts))
    }

    /// Merge `rev` into the current branch. Fast-forwards when possible (unless `no_ff`).
    pub fn merge(&self, rev: &str, no_ff: bool, message: Option<&str>, author: Option<&str>) -> Result<MergeOutcome, String> {
        self.require_no_merge()?;
        let head = self.head_commit()?.ok_or("nothing to merge into: there are no commits yet")?;
        let theirs = self.resolve_rev(rev)?;
        self.require_clean()?;
        let base = self.merge_base(&head, &theirs)?;
        if base.as_deref() == Some(theirs.as_str()) {
            return Ok(MergeOutcome::UpToDate);
        }
        let theirs_doc = self.doc_of(&theirs)?;
        if base.as_deref() == Some(head.as_str()) && !no_ff {
            self.write_working(&theirs_doc)?;
            self.set_head_commit(&theirs)?;
            return Ok(MergeOutcome::FastForward(theirs));
        }
        let base_doc = match &base {
            Some(b) => self.doc_of(b)?,
            None => Value::Object(Default::default()),
        };
        let msg = message.map(String::from).unwrap_or_else(|| match self.current_branch() {
            Ok(Some(b)) => format!("Merge {rev} into {b}"),
            _ => format!("Merge {rev}"),
        });
        self.apply_three_way("merge", &base_doc, &self.doc_of(&head)?, &theirs_doc, Some(theirs), msg, author)
    }

    /// Replay the change a commit made (relative to its first parent) onto the current state.
    pub fn cherry_pick(&self, rev: &str, author: Option<&str>) -> Result<MergeOutcome, String> {
        self.require_no_merge()?;
        let head = self.head_commit()?.ok_or("there are no commits yet")?;
        self.require_clean()?;
        let c = self.resolve_rev(rev)?;
        let commit = self.read_commit(&c)?;
        let base = match commit.parents.first() {
            Some(p) => self.doc_of(p)?,
            None => Value::Object(Default::default()),
        };
        self.apply_three_way("cherry-pick", &base, &self.doc_of(&head)?, &self.read_tree(&commit.tree)?, None, commit.message.clone(), author)
    }

    /// Undo the change a commit made (relative to its first parent) on the current state.
    pub fn revert(&self, rev: &str, author: Option<&str>) -> Result<MergeOutcome, String> {
        self.require_no_merge()?;
        let head = self.head_commit()?.ok_or("there are no commits yet")?;
        self.require_clean()?;
        let c = self.resolve_rev(rev)?;
        let commit = self.read_commit(&c)?;
        let parent = commit.parents.first().ok_or("cannot revert the root commit")?;
        let msg = format!("Revert \"{}\"", commit.message.lines().next().unwrap_or(""));
        self.apply_three_way("revert", &self.read_tree(&commit.tree)?, &self.doc_of(&head)?, &self.doc_of(parent)?, None, msg, author)
    }

    /// Drop an in-progress merge / cherry-pick / revert and restore the committed project.
    pub fn abort(&self) -> Result<(), String> {
        if self.load_state()?.is_none() {
            return Err("no merge in progress".into());
        }
        let head = self.head_commit()?.ok_or("there are no commits yet")?;
        self.write_working(&self.doc_of(&head)?)?;
        self.clear_state()
    }

    /// Resolve one recorded conflict by its path.
    pub fn resolve(&self, path: &str, choice: &Choice) -> Result<usize, String> {
        let mut st = self.load_state()?.ok_or("no merge in progress")?;
        let idx = st.conflicts.iter().position(|c| c.path == path).ok_or_else(|| format!("no conflict at '{path}' (see 'vcs status')"))?;
        let mut doc = self.read_working()?;
        semdiff::resolve_in(&mut doc, &st.conflicts[idx], choice)?;
        self.write_working(&doc)?;
        st.conflicts.remove(idx);
        // a conflict list may hold the same path twice (order conflicts); drop the same-path duplicates too
        st.conflicts.retain(|c| c.path != path);
        let left = st.conflicts.len();
        self.save_state(&st)?;
        Ok(left)
    }

    // ----- gc, fsck -----

    pub(super) fn roots(&self) -> Result<Vec<String>, String> {
        let mut r: Vec<String> = Vec::new();
        r.extend(self.list_refs("refs/heads")?.into_iter().map(|x| x.1));
        r.extend(self.list_refs("refs/tags")?.into_iter().map(|x| x.1));
        if let Some(h) = self.head_commit()? {
            r.push(h);
        }
        if let Some(st) = self.load_state()? {
            r.extend(st.theirs);
        }
        Ok(r)
    }

    pub(super) fn reachable(&self) -> Result<HashSet<String>, String> {
        let mut seen = HashSet::new();
        let mut stack = self.roots()?;
        while let Some(h) = stack.pop() {
            if !seen.insert(h.clone()) {
                continue;
            }
            let c = self.read_commit(&h)?;
            stack.extend(c.parents.iter().cloned());
            let t = self.get_json(&c.tree, "tree")?;
            seen.insert(c.tree.clone());
            for (_, ph) in t.get("parts").and_then(|p| p.as_object()).ok_or("tree has no parts")? {
                let ph = ph.as_str().ok_or("tree part hash is not text")?;
                if !is_hash(ph) {
                    return Err("tree holds a malformed hash".into());
                }
                seen.insert(ph.to_string());
            }
        }
        Ok(seen)
    }

    /// Delete objects no ref (branch, tag, HEAD, pending merge) reaches. Aborts without deleting
    /// anything if the reachable history cannot be read.
    pub fn gc(&self) -> Result<GcReport, String> {
        let keep = self.reachable().map_err(|e| format!("gc aborted, nothing deleted: {e}"))?;
        let mut rep = GcReport::default();
        for h in self.all_object_names()? {
            if keep.contains(&h) {
                rep.kept += 1;
            } else {
                fs::remove_file(self.obj_path(&h)?).map_err(|e| format!("cannot delete object: {e}"))?;
                rep.removed += 1;
            }
        }
        Ok(rep)
    }

    /// Integrity check: every object rehashes to its name and parses, every reference resolves,
    /// every ref points to a commit. Returns the list of problems (empty = healthy).
    pub fn fsck(&self) -> Vec<String> {
        let mut bad = Vec::new();
        let rd = match fs::read_dir(self.dir.join("objects")) {
            Ok(r) => r,
            Err(e) => return vec![format!("cannot list objects: {e}")],
        };
        let mut count = 0;
        for d in rd.flatten() {
            let dn = d.file_name().to_string_lossy().to_string();
            let Ok(rd2) = fs::read_dir(d.path()) else {
                bad.push(format!("objects/{dn} is not a directory"));
                continue;
            };
            for f in rd2.flatten() {
                let fname = f.file_name().to_string_lossy().to_string();
                let name = format!("{dn}{fname}");
                if fname.starts_with('.') {
                    bad.push(format!("stray temporary file objects/{dn}/{fname}"));
                    continue;
                }
                count += 1;
                if !is_hash(&name) {
                    bad.push(format!("objects/{dn}/{fname}: not a valid object name"));
                    continue;
                }
                let bytes = match fs::read(f.path()) {
                    Ok(b) => b,
                    Err(e) => {
                        bad.push(format!("object {}: unreadable ({e})", &name[..12]));
                        continue;
                    }
                };
                if sha256_hex(&bytes) != name {
                    bad.push(format!("object {}: content does not match its hash (corrupt)", &name[..12]));
                    continue;
                }
                match decode_object(&bytes) {
                    Err(e) => bad.push(format!("object {}: {e}", &name[..12])),
                    Ok((kind, payload)) => match (kind.as_str(), serde_json::from_slice::<Value>(&payload)) {
                        (_, Err(e)) => bad.push(format!("object {}: payload is not JSON ({e})", &name[..12])),
                        ("commit", Ok(v)) => match serde_json::from_value::<Commit>(v) {
                            Err(e) => bad.push(format!("commit {}: malformed ({e})", &name[..12])),
                            Ok(c) => {
                                let mut refs = vec![(c.tree.clone(), "tree")];
                                refs.extend(c.parents.iter().map(|p| (p.clone(), "commit")));
                                for (r, k) in refs {
                                    match self.get_object(&r) {
                                        Ok((rk, _)) if rk == k => {}
                                        Ok((rk, _)) => bad.push(format!("commit {}: {k} {} is a {rk}", &name[..12], &r[..r.len().min(12)])),
                                        Err(e) => bad.push(format!("commit {}: {e}", &name[..12])),
                                    }
                                }
                            }
                        },
                        ("tree", Ok(v)) => match v.get("parts").and_then(|p| p.as_object()) {
                            None => bad.push(format!("tree {}: no parts", &name[..12])),
                            Some(p) => {
                                for (pn, ph) in p {
                                    match ph.as_str().map(|h| self.get_object(h)) {
                                        Some(Ok((k, _))) if k == "blob" => {}
                                        Some(Ok((k, _))) => bad.push(format!("tree {}: part {pn} is a {k}", &name[..12])),
                                        Some(Err(e)) => bad.push(format!("tree {}: part {pn}: {e}", &name[..12])),
                                        None => bad.push(format!("tree {}: part {pn} has no hash", &name[..12])),
                                    }
                                }
                            }
                        },
                        _ => {}
                    },
                }
            }
        }
        let _ = count;
        for sub in ["refs/heads", "refs/tags"] {
            match self.list_refs(sub) {
                Err(e) => bad.push(e),
                Ok(rs) => {
                    for (n, h) in rs {
                        match self.get_object(&h) {
                            Ok((k, _)) if k == "commit" => {}
                            Ok((k, _)) => bad.push(format!("{sub}/{n} points to a {k}")),
                            Err(e) => bad.push(format!("{sub}/{n}: {e}")),
                        }
                    }
                }
            }
        }
        match self.current_branch() {
            Err(e) => bad.push(e),
            Ok(None) => match self.head_commit() {
                Ok(Some(h)) if self.read_commit(&h).is_ok() => {}
                _ => bad.push("detached HEAD does not point to a commit".into()),
            },
            Ok(Some(_)) => {
                if let Err(e) = self.head_commit() {
                    bad.push(e);
                }
            }
        }
        if let Err(e) = self.load_state() {
            bad.push(e);
        }
        if let Err(e) = self.config() {
            bad.push(e);
        }
        bad
    }

    /// Counts for the report of `gc --dry-run`-like inspection.
    pub fn object_count(&self) -> Result<usize, String> {
        Ok(self.all_object_names()?.len())
    }

    /// Hash set of every ref target, for tests and tools.
    pub fn ref_targets(&self) -> Result<BTreeSet<String>, String> {
        Ok(self.roots()?.into_iter().collect())
    }
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------
