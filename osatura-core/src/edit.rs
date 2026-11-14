// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Editing session with unlimited undo / redo.
//!
//! The project is edited as a JSON document through a tiny patch language
//! (`set` / `insert` / `remove` / `batch` on RFC 6901 JSON pointers). Every applied op
//! returns its exact inverse, which is what undo stores. After each edit the whole
//! project is re-validated by compiling an [`Engine`]; an edit that would make the
//! project invalid is rolled back and its errors are returned, so the session can never
//! hold a broken project.

use crate::anim::Ease;
use crate::engine::Engine;
use crate::project::{Channel, Project, Prop, Source};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Replace the value at `path` (or create an object key). `path` "" replaces everything.
    Set { path: String, value: Value },
    /// Insert into an array at `path` (`…/-` appends) or create an object key.
    Insert { path: String, value: Value },
    /// Remove an array element or object key.
    Remove { path: String },
    /// Several ops as one undo step.
    Batch { ops: Vec<Op> },
}

fn unescape(tok: &str) -> String {
    tok.replace("~1", "/").replace("~0", "~")
}

fn split(path: &str) -> Result<(&str, String), String> {
    if path.is_empty() {
        return Err("path must not be empty here".into());
    }
    let i = path
        .rfind('/')
        .ok_or_else(|| format!("bad JSON pointer '{path}'"))?;
    Ok((&path[..i], unescape(&path[i + 1..])))
}

fn raw_apply(doc: &mut Value, op: &Op) -> Result<Op, String> {
    match op {
        Op::Set { path, value } => {
            if path.is_empty() {
                let old = std::mem::replace(doc, value.clone());
                return Ok(Op::Set {
                    path: String::new(),
                    value: old,
                });
            }
            let (pp, tok) = split(path)?;
            let parent = doc
                .pointer_mut(pp)
                .ok_or_else(|| format!("no such path '{pp}'"))?;
            match parent {
                Value::Object(m) => Ok(match m.insert(tok, value.clone()) {
                    Some(old) => Op::Set {
                        path: path.clone(),
                        value: old,
                    },
                    None => Op::Remove { path: path.clone() },
                }),
                Value::Array(a) => {
                    let i: usize = tok
                        .parse()
                        .map_err(|_| format!("bad array index '{tok}'"))?;
                    let slot = a
                        .get_mut(i)
                        .ok_or_else(|| format!("index {i} out of range at '{pp}'"))?;
                    let old = std::mem::replace(slot, value.clone());
                    Ok(Op::Set {
                        path: path.clone(),
                        value: old,
                    })
                }
                _ => Err(format!("'{pp}' is not a container")),
            }
        }
        Op::Insert { path, value } => {
            let (pp, tok) = split(path)?;
            let parent = doc
                .pointer_mut(pp)
                .ok_or_else(|| format!("no such path '{pp}'"))?;
            match parent {
                Value::Array(a) => {
                    let i = if tok == "-" {
                        a.len()
                    } else {
                        tok.parse()
                            .map_err(|_| format!("bad array index '{tok}'"))?
                    };
                    if i > a.len() {
                        return Err(format!("insert index {i} out of range at '{pp}'"));
                    }
                    a.insert(i, value.clone());
                    Ok(Op::Remove {
                        path: format!("{pp}/{i}"),
                    })
                }
                Value::Object(m) => Ok(match m.insert(tok, value.clone()) {
                    Some(old) => Op::Set {
                        path: path.clone(),
                        value: old,
                    },
                    None => Op::Remove { path: path.clone() },
                }),
                _ => Err(format!("'{pp}' is not a container")),
            }
        }
        Op::Remove { path } => {
            let (pp, tok) = split(path)?;
            let parent = doc
                .pointer_mut(pp)
                .ok_or_else(|| format!("no such path '{pp}'"))?;
            match parent {
                Value::Array(a) => {
                    let i: usize = tok
                        .parse()
                        .map_err(|_| format!("bad array index '{tok}'"))?;
                    if i >= a.len() {
                        return Err(format!("index {i} out of range at '{pp}'"));
                    }
                    Ok(Op::Insert {
                        path: path.clone(),
                        value: a.remove(i),
                    })
                }
                Value::Object(m) => match m.remove(&tok) {
                    Some(old) => Ok(Op::Set {
                        path: path.clone(),
                        value: old,
                    }),
                    None => Err(format!("no key '{tok}' at '{pp}'")),
                },
                _ => Err(format!("'{pp}' is not a container")),
            }
        }
        Op::Batch { ops } => {
            let mut inv: Vec<Op> = Vec::with_capacity(ops.len());
            for o in ops {
                match raw_apply(doc, o) {
                    Ok(i) => inv.push(i),
                    Err(e) => {
                        for i in inv.iter().rev() {
                            let _ = raw_apply(doc, i);
                        }
                        return Err(e);
                    }
                }
            }
            inv.reverse();
            Ok(Op::Batch { ops: inv })
        }
    }
}

pub struct Editor {
    doc: Value,
    engine: Engine,
    asset_dir: PathBuf,
    undo: Vec<Op>,
    redo: Vec<Op>,
    /// Bumped whenever the compiled project changes (edit, undo, redo); caches key on it.
    revision: u64,
}

impl Editor {
    pub fn new(project: Project) -> Result<Editor, Vec<String>> {
        Editor::with_asset_dir(project, PathBuf::from("."))
    }

    pub fn with_asset_dir(project: Project, asset_dir: PathBuf) -> Result<Editor, Vec<String>> {
        let doc = serde_json::to_value(&project).map_err(|e| vec![e.to_string()])?;
        let mut engine = Engine::with_asset_dir(project, &asset_dir)?;
        engine.show_reference = true;
        Ok(Editor {
            doc,
            engine,
            asset_dir,
            undo: Vec::new(),
            redo: Vec::new(),
            revision: 0,
        })
    }

    /// Directory image paths of the project are resolved against.
    pub fn asset_dir(&self) -> &std::path::Path {
        &self.asset_dir
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }
    pub fn project(&self) -> &Project {
        &self.engine.project
    }
    pub fn doc(&self) -> &Value {
        &self.doc
    }
    /// Counter that changes every time the project changes (edit, undo, redo); used to invalidate frame caches.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn rebuild(&mut self) -> Result<(), Vec<String>> {
        let project: Project = serde_json::from_value(self.doc.clone())
            .map_err(|e| vec![format!("invalid project: {e}")])?;
        self.engine = Engine::with_asset_dir(project, &self.asset_dir)?;
        self.engine.show_reference = true;
        self.revision += 1;
        Ok(())
    }

    /// Apply an edit. On failure nothing changes and the reasons are returned.
    pub fn apply(&mut self, op: Op) -> Result<(), Vec<String>> {
        let inverse = raw_apply(&mut self.doc, &op).map_err(|e| vec![e])?;
        if let Err(errs) = self.rebuild() {
            let _ = raw_apply(&mut self.doc, &inverse);
            return Err(errs);
        }
        self.undo.push(inverse);
        self.redo.clear();
        Ok(())
    }

    /// Replace the whole document with an externally merged one (collaboration). The result must compile;
    /// on failure nothing changes. The local undo history is cleared because its inverse ops no longer
    /// describe this document (per-user undo lives in `collab`).
    pub fn replace_doc(&mut self, doc: Value) -> Result<(), Vec<String>> {
        let project: Project = serde_json::from_value(doc.clone())
            .map_err(|e| vec![format!("invalid project: {e}")])?;
        let mut engine = Engine::with_asset_dir(project, &self.asset_dir)?;
        engine.show_reference = true;
        self.doc = doc;
        self.engine = engine;
        self.revision += 1;
        self.undo.clear();
        self.redo.clear();
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        let Some(inv) = self.undo.pop() else {
            return false;
        };
        match raw_apply(&mut self.doc, &inv) {
            Ok(redo_op) => {
                // The document was valid before the edit, so this always compiles.
                let _ = self.rebuild();
                self.redo.push(redo_op);
                true
            }
            Err(_) => false,
        }
    }

    pub fn redo(&mut self) -> bool {
        let Some(op) = self.redo.pop() else {
            return false;
        };
        match raw_apply(&mut self.doc, &op) {
            Ok(inv) => {
                let _ = self.rebuild();
                self.undo.push(inv);
                true
            }
            Err(_) => false,
        }
    }

    fn keyed_channel(&self, target: &str, prop: Prop) -> Option<usize> {
        self.project().channels.iter().position(|c| {
            c.target == target
                && c.prop == prop
                && matches!(c.source, Source::Keys { .. })
                && !crate::skins::is_collapse_channel(c)
        })
    }

    /// Set (or add) a keyframe on `target.prop`, creating the channel if needed.
    pub fn set_key(
        &mut self,
        target: &str,
        prop: Prop,
        frame: f32,
        value: f32,
        ease: Ease,
    ) -> Result<(), Vec<String>> {
        crate::chanmeta::assert_unlocked(self.project(), target, prop).map_err(|e| vec![e])?;
        match self.keyed_channel(target, prop) {
            Some(i) => {
                let mut ch = self.project().channels[i].clone();
                if let Source::Keys { track } = &mut ch.source {
                    track.set_key(frame, value, ease);
                }
                let v = serde_json::to_value(&ch).map_err(|e| vec![e.to_string()])?;
                self.apply(Op::Set {
                    path: format!("/channels/{i}"),
                    value: v,
                })
            }
            None => {
                let ch = Channel::keys(
                    target,
                    prop,
                    crate::anim::Track::with_keys(&[(frame, value, ease)]),
                );
                let v = serde_json::to_value(&ch).map_err(|e| vec![e.to_string()])?;
                self.apply(Op::Insert {
                    path: "/channels/-".into(),
                    value: v,
                })
            }
        }
    }

    /// Delete the keyframe at `frame` (removing the channel when it was the last key).
    pub fn delete_key(
        &mut self,
        target: &str,
        prop: Prop,
        frame: f32,
    ) -> Result<bool, Vec<String>> {
        let Some(i) = self.keyed_channel(target, prop) else {
            return Ok(false);
        };
        let mut ch = self.project().channels[i].clone();
        let removed = match &mut ch.source {
            Source::Keys { track } => track.remove_key(frame),
            _ => false,
        };
        if !removed {
            return Ok(false);
        }
        let empty = matches!(&ch.source, Source::Keys { track } if track.keys.is_empty());
        if empty {
            self.apply(Op::Remove {
                path: format!("/channels/{i}"),
            })?;
        } else {
            let v = serde_json::to_value(&ch).map_err(|e| vec![e.to_string()])?;
            self.apply(Op::Set {
                path: format!("/channels/{i}"),
                value: v,
            })?;
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn editor() -> Editor {
        Editor::new(crate::demo::project()).unwrap()
    }

    #[test]
    fn set_undo_redo_round_trip() {
        let mut e = editor();
        let before = e.doc().clone();
        e.apply(Op::Set {
            path: "/fps".into(),
            value: json!(30.0),
        })
        .unwrap();
        assert_eq!(e.project().fps, 30.0);
        assert!(e.undo());
        assert_eq!(e.doc(), &before);
        assert_eq!(e.project().fps, 24.0);
        assert!(e.redo());
        assert_eq!(e.project().fps, 30.0);
        assert!(!e.redo() && e.can_undo());
    }

    #[test]
    fn revision_changes_on_every_edit_undo_redo_but_not_on_failure() {
        let mut e = editor();
        let r0 = e.revision();
        assert!(e
            .apply(Op::Set {
                path: "/fps".into(),
                value: json!(0.0)
            })
            .is_err());
        assert_eq!(e.revision(), r0, "a rejected edit changes nothing");
        e.apply(Op::Set {
            path: "/fps".into(),
            value: json!(30.0),
        })
        .unwrap();
        let r1 = e.revision();
        assert_ne!(r1, r0);
        assert!(e.undo());
        let r2 = e.revision();
        assert!(r2 != r1 && r2 != r0);
        assert!(e.redo());
        assert!(e.revision() != r2);
        assert!(!e.undo() || e.revision() > r2);
    }

    #[test]
    fn invalid_edit_is_rejected_and_rolled_back() {
        let mut e = editor();
        let before = e.doc().clone();
        let errs = e
            .apply(Op::Set {
                path: "/fps".into(),
                value: json!(0.0),
            })
            .unwrap_err();
        assert!(errs.join(" ").contains("fps"), "{errs:?}");
        assert_eq!(e.doc(), &before);
        assert!(!e.can_undo(), "failed edits leave no history");
        assert!(e
            .apply(Op::Remove {
                path: "/nope/1".into()
            })
            .is_err());
        assert!(e
            .apply(Op::Set {
                path: "/bones/999/length".into(),
                value: json!(1)
            })
            .is_err());
    }

    #[test]
    fn insert_and_remove_arrays_invert_exactly() {
        let mut e = editor();
        let n = e.project().channels.len();
        let before = e.doc().clone();
        let ch = serde_json::to_value(Channel::keys(
            "hip",
            Prop::BoneAngle,
            crate::anim::Track::with_keys(&[(0.0, 5.0, Ease::Linear)]),
        ))
        .unwrap();
        // pick an existing bone name from the demo rig
        let bone = e.project().bones[0].name.clone();
        let mut ch = ch;
        ch["target"] = json!(bone);
        e.apply(Op::Insert {
            path: "/channels/0".into(),
            value: ch,
        })
        .unwrap();
        assert_eq!(e.project().channels.len(), n + 1);
        e.apply(Op::Remove {
            path: "/channels/1".into(),
        })
        .unwrap();
        assert_eq!(e.project().channels.len(), n);
        assert!(e.undo() && e.undo());
        assert_eq!(e.doc(), &before);
    }

    #[test]
    fn batch_is_one_undo_step_and_atomic() {
        let mut e = editor();
        let before = e.doc().clone();
        e.apply(Op::Batch {
            ops: vec![
                Op::Set {
                    path: "/fps".into(),
                    value: json!(12.0),
                },
                Op::Set {
                    path: "/frames".into(),
                    value: json!(24),
                },
            ],
        })
        .unwrap();
        assert_eq!((e.project().fps, e.project().frames), (12.0, 24));
        assert!(e.undo());
        assert_eq!(e.doc(), &before);
        // a failing second op must not leave the first applied
        let r = e.apply(Op::Batch {
            ops: vec![
                Op::Set {
                    path: "/fps".into(),
                    value: json!(12.0),
                },
                Op::Remove {
                    path: "/does/not/exist".into(),
                },
            ],
        });
        assert!(r.is_err());
        assert_eq!(e.doc(), &before);
    }

    #[test]
    fn new_edit_clears_redo() {
        let mut e = editor();
        e.apply(Op::Set {
            path: "/fps".into(),
            value: json!(30.0),
        })
        .unwrap();
        e.undo();
        assert!(e.can_redo());
        e.apply(Op::Set {
            path: "/fps".into(),
            value: json!(25.0),
        })
        .unwrap();
        assert!(!e.can_redo());
    }

    #[test]
    fn keyframe_helpers_create_update_and_delete() {
        let mut e = editor();
        let bone = e.project().bones[0].name.clone();
        let count = |e: &Editor| {
            e.project()
                .channels
                .iter()
                .filter(|c| c.target == bone && c.prop == Prop::BoneScale)
                .count()
        };
        assert_eq!(count(&e), 0);
        e.set_key(&bone, Prop::BoneScale, 0.0, 1.0, Ease::Linear)
            .unwrap();
        e.set_key(&bone, Prop::BoneScale, 10.0, 2.0, Ease::Linear)
            .unwrap();
        assert_eq!(count(&e), 1, "second key joins the same channel");
        let f = e.engine().evaluate(5.0);
        let _ = f;
        assert!(e.delete_key(&bone, Prop::BoneScale, 10.0).unwrap());
        assert!(e.delete_key(&bone, Prop::BoneScale, 0.0).unwrap());
        assert_eq!(count(&e), 0, "channel disappears with its last key");
        assert!(!e.delete_key(&bone, Prop::BoneScale, 0.0).unwrap());
        for _ in 0..4 {
            assert!(e.undo());
        }
        assert_eq!(count(&e), 0);
        assert!(!e.undo());
    }

    #[test]
    fn ops_parse_from_json() {
        let op: Op = serde_json::from_str(r#"{"op":"set","path":"/fps","value":30}"#).unwrap();
        assert_eq!(
            op,
            Op::Set {
                path: "/fps".into(),
                value: json!(30)
            }
        );
        let b: Op =
            serde_json::from_str(r#"{"op":"batch","ops":[{"op":"remove","path":"/x"}]}"#).unwrap();
        assert!(matches!(b, Op::Batch { .. }));
    }
}
