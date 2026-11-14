// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Revision snapshots and crash recovery: numbered copies of a project in a folder, with
//! duplicate suppression and pruning. No clock needed for ordering (a counter is used), so it
//! behaves identically everywhere.

use crate::inspect::fnv1a;
use crate::project::Project;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Snapshots {
    dir: PathBuf,
    keep: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Revision {
    pub number: u64,
    pub hash: String,
    pub path: PathBuf,
}

impl Snapshots {
    pub fn new(dir: impl AsRef<Path>, keep: usize) -> Snapshots {
        Snapshots { dir: dir.as_ref().to_path_buf(), keep: keep.max(1) }
    }

    pub fn list(&self) -> Vec<Revision> {
        let mut out = Vec::new();
        if let Ok(rd) = fs::read_dir(&self.dir) {
            for e in rd.flatten() {
                let name = e.file_name().to_string_lossy().to_string();
                // rev-000012-<hash>.json
                if let Some(rest) = name.strip_prefix("rev-").and_then(|r| r.strip_suffix(".json")) {
                    if let Some((n, h)) = rest.split_once('-') {
                        if let Ok(number) = n.parse::<u64>() {
                            out.push(Revision { number, hash: h.to_string(), path: e.path() });
                        }
                    }
                }
            }
        }
        out.sort_by_key(|r| r.number);
        out
    }

    /// Save a snapshot unless it is identical to the newest one. Returns the revision if written.
    pub fn save(&self, p: &Project) -> Result<Option<Revision>, String> {
        fs::create_dir_all(&self.dir).map_err(|e| format!("cannot create {}: {e}", self.dir.display()))?;
        let json = p.to_json_pretty();
        let hash = format!("{:016x}", fnv1a(json.as_bytes()));
        let existing = self.list();
        if existing.last().is_some_and(|r| r.hash == hash) {
            return Ok(None);
        }
        let number = existing.last().map_or(1, |r| r.number + 1);
        let path = self.dir.join(format!("rev-{number:06}-{hash}.json"));
        // atomic (temporary file, fsync, rename) and symlink-refusing, so a crash never leaves a half-written revision
        crate::fsutil::write_atomic(&path, json.as_bytes())?;
        let all = self.list();
        if all.len() > self.keep {
            for old in &all[..all.len() - self.keep] {
                let _ = fs::remove_file(&old.path);
            }
        }
        Ok(Some(Revision { number, hash, path }))
    }

    /// Load revision `number` (or the newest when `None`). Fails if the file no longer parses.
    pub fn load(&self, number: Option<u64>) -> Result<Project, String> {
        let all = self.list();
        let rev = match number {
            Some(n) => all.iter().find(|r| r.number == n),
            None => all.last(),
        }
        .ok_or("no such revision")?;
        let text = fs::read_to_string(&rev.path).map_err(|e| e.to_string())?;
        Project::from_json(&text).map_err(|e| format!("revision {} is damaged: {e}", rev.number))
    }

    /// Newest revision that still parses — used after a crash.
    pub fn recover(&self) -> Option<Project> {
        self.list().iter().rev().find_map(|r| fs::read_to_string(&r.path).ok().and_then(|t| Project::from_json(&t).ok()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo;

    fn tmp(tag: &str) -> crate::fsutil::ScratchDir {
        crate::fsutil::ScratchDir::new(&format!("osatura-snap-{tag}"))
    }

    #[test]
    fn saves_dedupes_prunes_and_loads() {
        let d = tmp("a");
        let s = Snapshots::new(&d, 3);
        let mut p = demo::project();
        assert!(s.save(&p).unwrap().is_some());
        assert!(s.save(&p).unwrap().is_none(), "identical project is not saved twice");
        for i in 0..5 {
            p.fps = 25.0 + i as f32;
            s.save(&p).unwrap();
        }
        let l = s.list();
        assert_eq!(l.len(), 3, "pruned to the newest three");
        assert_eq!(l.last().unwrap().number, 6);
        assert_eq!(s.load(None).unwrap().fps, 29.0);
        assert_eq!(s.load(Some(5)).unwrap().fps, 28.0);
        assert!(s.load(Some(1)).is_err(), "pruned revision is gone");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn recovery_skips_damaged_revisions() {
        let d = tmp("b");
        let s = Snapshots::new(&d, 10);
        let mut p = demo::project();
        s.save(&p).unwrap();
        p.fps = 30.0;
        let r = s.save(&p).unwrap().unwrap();
        fs::write(&r.path, "{ truncated").unwrap();
        assert!(s.load(None).is_err());
        assert_eq!(s.recover().unwrap().fps, 24.0, "falls back to the last good revision");
        let _ = fs::remove_dir_all(&d);
        assert!(Snapshots::new(tmp("c"), 2).recover().is_none());
    }
}
