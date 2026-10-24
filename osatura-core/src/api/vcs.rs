// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Part of `api` (see the module docs in `mod.rs`).
#![allow(unused_imports)]
use super::*;

// ---------------------------------------------------------------------------------------------
// Version control (write operations), 1.3
// ---------------------------------------------------------------------------------------------

/// One unresolved conflict of a merge, cherry-pick or revert (see [`VcsMergeOutcome`]).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct VcsConflict {
    /// Stable identity pointer of the conflicting item; pass it to [`vcs_resolve`].
    pub path: String,
    /// `"BothChanged"`, `"DeleteEdit"`, `"KeyRetimed"`, `"OrderConflict"` or `"ArrayConflict"`
    /// (more may follow in later minor versions).
    pub kind: String,
    /// Human-readable one-line description.
    pub description: String,
    /// The value as JSON text in the common ancestor, in the current branch and in the other one;
    /// `None` when the item does not exist there.
    pub base: Option<String>,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

/// Result of [`vcs_merge`], [`vcs_cherry_pick`] and [`vcs_revert`]. Conflicts are a normal result,
/// not an error: the project file then holds the merged state with the conflicting items at their
/// current-branch value.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum VcsMergeOutcome {
    /// The other revision is already part of the current history; nothing changed.
    UpToDate,
    /// The branch simply moved forward to this commit.
    FastForward(String),
    /// A new commit with this id was created.
    Merged(String),
    /// Resolve every conflict with [`vcs_resolve`], then call [`vcs_commit`] (or [`vcs_abort`]).
    Conflicted(Vec<VcsConflict>),
}

/// How [`vcs_resolve`] settles one conflict.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum VcsChoice {
    /// Keep the current branch's version.
    Ours,
    /// Take the other revision's version.
    Theirs,
    /// Use this JSON text as the value.
    Value(String),
}

pub(super) fn vcs_open(path: &str) -> Result<crate::vcs::Repo, ApiError> {
    crate::vcs::Repo::open(std::path::Path::new(path)).map_err(ApiError::InvalidArgument)
}

pub(super) fn vcs_str<T>(r: Result<T, String>) -> Result<T, ApiError> {
    r.map_err(ApiError::InvalidArgument)
}

pub(super) fn vcs_no_merge(repo: &crate::vcs::Repo) -> Result<(), ApiError> {
    match vcs_str(repo.merge_state())? {
        Some(st) => Err(ApiError::MergeInProgress(format!("a {} is unfinished ({} conflict(s) left); resolve and commit, or call vcs_abort", st.kind, st.conflicts.len()))),
        None => Ok(()),
    }
}

pub(super) fn vcs_clean(repo: &crate::vcs::Repo) -> Result<(), ApiError> {
    if vcs_str(repo.head_commit())?.is_some() && vcs_str(repo.is_dirty())? {
        return Err(ApiError::DirtyTree("the project file differs from the last commit: commit it first".into()));
    }
    Ok(())
}

pub(super) fn vcs_conflict(c: &crate::semdiff::Conflict) -> VcsConflict {
    let slot = |s: &crate::semdiff::Slot| match s {
        crate::semdiff::Slot::Value(v) => Some(v.to_string()),
        _ => None,
    };
    let kind = serde_json::to_value(c.kind).ok().and_then(|v| v.as_str().map(String::from)).unwrap_or_else(|| format!("{:?}", c.kind));
    VcsConflict { path: c.path.clone(), kind, description: c.description.clone(), base: slot(&c.base), ours: slot(&c.ours), theirs: slot(&c.theirs) }
}

pub(super) fn vcs_outcome(o: crate::vcs::MergeOutcome) -> VcsMergeOutcome {
    match o {
        crate::vcs::MergeOutcome::UpToDate => VcsMergeOutcome::UpToDate,
        crate::vcs::MergeOutcome::FastForward(h) => VcsMergeOutcome::FastForward(h),
        crate::vcs::MergeOutcome::Merged(h) => VcsMergeOutcome::Merged(h),
        crate::vcs::MergeOutcome::Conflicted(cs) => VcsMergeOutcome::Conflicted(cs.iter().map(vcs_conflict).collect()),
    }
}

/// Start version control for the project file `project_file` (it must exist and be JSON): creates
/// the history next to it, on branch `main`, with no commit yet. `author` is the default author
/// of later commits. Fails if a history already exists there.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-init-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// assert!(vcs_log(path, 5)?.is_empty());           // no commit yet
/// assert!(vcs_init(path, None).is_err());          // already initialised
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_init(project_file: &str, author: Option<&str>) -> Result<(), ApiError> {
    guard(|| vcs_str(crate::vcs::Repo::init(std::path::Path::new(project_file), author)).map(|_| ()))
}

/// Commit the project file on the current branch and return the new commit id. With a finished
/// merge in progress this creates the merge commit (the message then defaults to the merge's).
/// Errors: [`ApiError::Conflicts`] while conflicts are unresolved, [`ApiError::InvalidArgument`]
/// when there is nothing to commit or no message was given.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-commit-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// let first = vcs_commit(path, Some("first"), None)?;
/// assert_eq!(vcs_log(path, 5)?[0].id, first);
/// assert!(vcs_commit(path, Some("again"), None).is_err()); // unchanged
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_commit(path: &str, message: Option<&str>, author: Option<&str>) -> Result<String, ApiError> {
    guard(|| {
        let repo = vcs_open(path)?;
        if let Some(st) = vcs_str(repo.merge_state())? {
            if !st.conflicts.is_empty() {
                return Err(ApiError::Conflicts(st.conflicts.iter().map(|c| c.path.clone()).collect()));
            }
        }
        vcs_str(repo.commit(message, author))
    })
}

/// Create branch `name` at revision `at` (a branch, tag or commit id; default: the current
/// commit) without switching to it. Returns the commit it points at.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-branch-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// let c = vcs_commit(path, Some("first"), None)?;
/// assert_eq!(vcs_branch(path, "feature", None)?, c);
/// assert!(vcs_branch(path, "feature", None).is_err());   // exists
/// assert!(vcs_branch(path, "bad name", None).is_err());  // invalid name
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_branch(path: &str, name: &str, at: Option<&str>) -> Result<String, ApiError> {
    guard(|| vcs_str(vcs_open(path)?.branch_create(name, at)))
}

/// Tag a revision (default: the current commit) with `name`. Returns the commit id.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-tag-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// let c = vcs_commit(path, Some("first"), None)?;
/// assert_eq!(vcs_tag(path, "v1", None)?, c);
/// assert!(vcs_tag(path, "v1", None).is_err());           // tags are not moved
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_tag(path: &str, name: &str, at: Option<&str>) -> Result<String, ApiError> {
    guard(|| vcs_str(vcs_open(path)?.tag_create(name, at)))
}

/// Switch the project file to a branch (stays on it) or to any other revision (detached). Returns
/// the commit now checked out. Errors: [`ApiError::DirtyTree`] if the file has uncommitted changes
/// and `force` is false, [`ApiError::MergeInProgress`] during an unfinished merge.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-checkout-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// vcs_commit(path, Some("first"), None)?;
/// vcs_branch(path, "feature", None)?;
/// // edit the file without committing
/// let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
/// let bone = a.bone_names()[0].clone();
/// a.set_key(&bone, "bone_angle", 7.0, 12.0, Interpolation::Linear)?;
/// std::fs::write(path, a.to_json()).unwrap();
/// assert!(matches!(vcs_checkout(path, "feature", false), Err(ApiError::DirtyTree(_))));
/// vcs_checkout(path, "feature", true)?;            // force discards the edit
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_checkout(path: &str, target: &str, force: bool) -> Result<String, ApiError> {
    guard(|| {
        let repo = vcs_open(path)?;
        vcs_no_merge(&repo)?;
        if !force {
            vcs_clean(&repo)?;
        }
        vcs_str(repo.checkout(target, force))
    })
}

/// Merge revision `rev` into the current branch. Fast-forwards when possible (unless `no_ff`).
/// Conflicts are returned as [`VcsMergeOutcome::Conflicted`], with the file left in the merged
/// state. Errors: [`ApiError::DirtyTree`], [`ApiError::MergeInProgress`].
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-merge-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// let edit = |angle: f32| -> Result<(), ApiError> {
///     let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
///     let bone = a.bone_names()[0].clone();
///     a.set_key(&bone, "bone_angle", 5.0, angle, Interpolation::Linear)?;
///     std::fs::write(path, a.to_json()).unwrap();
///     Ok(())
/// };
/// vcs_init(path, Some("ana"))?;
/// vcs_commit(path, Some("base"), None)?;
/// vcs_branch(path, "other", None)?;
/// edit(20.0)?;
/// vcs_commit(path, Some("main edit"), None)?;
/// vcs_checkout(path, "other", false)?;
/// edit(40.0)?;                                      // the same key, another value
/// vcs_commit(path, Some("other edit"), None)?;
/// vcs_checkout(path, "main", false)?;
/// match vcs_merge(path, "other", false, None, None)? {
///     VcsMergeOutcome::Conflicted(conflicts) => {
///         assert!(!conflicts.is_empty());
///         // while unresolved, committing is refused with a typed error
///         assert!(matches!(vcs_commit(path, None, None), Err(ApiError::Conflicts(_))));
///         for c in &conflicts {
///             vcs_resolve(path, &c.path, &VcsChoice::Theirs)?;
///         }
///         vcs_commit(path, None, None)?;            // the merge commit
///     }
///     other => panic!("expected a conflict, got {other:?}"),
/// }
/// assert_eq!(vcs_log(path, 10)?.len(), 4);
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_merge(path: &str, rev: &str, no_ff: bool, message: Option<&str>, author: Option<&str>) -> Result<VcsMergeOutcome, ApiError> {
    guard(|| {
        let repo = vcs_open(path)?;
        vcs_no_merge(&repo)?;
        vcs_clean(&repo)?;
        vcs_str(repo.merge(rev, no_ff, message, author)).map(vcs_outcome)
    })
}

/// The conflicts still unresolved in the project's pending merge, cherry-pick or revert (empty
/// when none is in progress).
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-conflicts-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, None)?;
/// assert!(vcs_conflicts(path)?.is_empty());
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_conflicts(path: &str) -> Result<Vec<VcsConflict>, ApiError> {
    guard(|| Ok(vcs_str(vcs_open(path)?.merge_state())?.map(|s| s.conflicts.iter().map(vcs_conflict).collect()).unwrap_or_default()))
}

/// Settle one conflict (by [`VcsConflict::path`]). Returns how many conflicts are left; when it
/// reaches 0 call [`vcs_commit`]. A [`VcsChoice::Value`] that is not valid JSON is rejected.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-resolve-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, None)?;
/// // nothing to resolve outside a merge
/// assert!(vcs_resolve(path, "/x", &VcsChoice::Ours).is_err());
/// assert!(vcs_resolve(path, "/x", &VcsChoice::Value("{ nope".into())).is_err());
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_resolve(path: &str, conflict_path: &str, choice: &VcsChoice) -> Result<usize, ApiError> {
    guard(|| {
        let c = match choice {
            VcsChoice::Ours => crate::semdiff::Choice::Ours,
            VcsChoice::Theirs => crate::semdiff::Choice::Theirs,
            VcsChoice::Value(t) => crate::semdiff::Choice::Value(serde_json::from_str(t).map_err(|e| ApiError::InvalidJson(e.to_string()))?),
        };
        vcs_str(vcs_open(path)?.resolve(conflict_path, &c))
    })
}

/// Drop an unfinished merge, cherry-pick or revert and restore the last commit.
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-abort-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, None)?;
/// assert!(vcs_abort(path).is_err());               // no merge in progress
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_abort(path: &str) -> Result<(), ApiError> {
    guard(|| vcs_str(vcs_open(path)?.abort()))
}

/// Replay the change commit `rev` made (relative to its first parent) onto the current branch.
/// Same result and errors as [`vcs_merge`].
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-pick-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// vcs_commit(path, Some("base"), None)?;
/// vcs_branch(path, "side", None)?;
/// vcs_checkout(path, "side", false)?;
/// let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
/// let bone = a.bone_names()[0].clone();
/// a.set_key(&bone, "bone_angle", 5.0, 33.0, Interpolation::Linear)?;
/// std::fs::write(path, a.to_json()).unwrap();
/// let picked = vcs_commit(path, Some("tweak"), None)?;
/// vcs_checkout(path, "main", false)?;
/// assert!(matches!(vcs_cherry_pick(path, &picked, None)?, VcsMergeOutcome::Merged(_)));
/// assert_eq!(vcs_log(path, 5)?[0].message, "tweak");
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_cherry_pick(path: &str, rev: &str, author: Option<&str>) -> Result<VcsMergeOutcome, ApiError> {
    guard(|| {
        let repo = vcs_open(path)?;
        vcs_no_merge(&repo)?;
        vcs_clean(&repo)?;
        vcs_str(repo.cherry_pick(rev, author)).map(vcs_outcome)
    })
}

/// Undo the change commit `rev` made on the current branch, as a new commit. Same result and
/// errors as [`vcs_merge`].
///
/// ```
/// use osatura_core::prelude::*;
/// # fn main() -> Result<(), ApiError> {
/// # let dir = std::env::temp_dir().join(format!("osatura-doc-revert-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// # let file = dir.join("p.json");
/// # std::fs::write(&file, Animation::demo().to_json()).unwrap();
/// let path = file.to_str().unwrap();
/// vcs_init(path, Some("ana"))?;
/// vcs_commit(path, Some("base"), None)?;
/// let mut a = Animation::from_json(&std::fs::read_to_string(path).unwrap())?;
/// let bone = a.bone_names()[0].clone();
/// a.set_key(&bone, "bone_angle", 5.0, 33.0, Interpolation::Linear)?;
/// std::fs::write(path, a.to_json()).unwrap();
/// let bad = vcs_commit(path, Some("oops"), None)?;
/// assert!(matches!(vcs_revert(path, &bad, None)?, VcsMergeOutcome::Merged(_)));
/// assert_eq!(vcs_log(path, 5)?.len(), 3);
/// # std::fs::remove_dir_all(&dir).unwrap();
/// # Ok(()) }
/// ```
pub fn vcs_revert(path: &str, rev: &str, author: Option<&str>) -> Result<VcsMergeOutcome, ApiError> {
    guard(|| {
        let repo = vcs_open(path)?;
        vcs_no_merge(&repo)?;
        vcs_clean(&repo)?;
        vcs_str(repo.revert(rev, author)).map(vcs_outcome)
    })
}

pub(super) fn vcs_changes(v: Vec<crate::semdiff::Change>) -> Vec<VcsChange> {
    v.into_iter().map(|c| VcsChange { kind: format!("{:?}", c.kind), path: c.path, description: c.description }).collect()
}

/// (1.4) List the branches, sorted by name. An empty repository has none.
pub fn vcs_branches(path: &str) -> Result<Vec<VcsBranch>, ApiError> {
    guard(|| Ok(vcs_str(vcs_open(path)?.branches())?.into_iter().map(|(name, commit, current)| VcsBranch { name, commit, current }).collect()))
}

/// (1.4) Delete a branch. The checked-out branch and unknown names are refused. Commits stay in
/// the store until [`vcs_gc`].
pub fn vcs_branch_delete(path: &str, name: &str) -> Result<(), ApiError> {
    guard(|| vcs_str(vcs_open(path)?.branch_delete(name)))
}

/// (1.4) Branch, HEAD, uncommitted semantic changes and any unfinished merge.
pub fn vcs_status(path: &str) -> Result<VcsStatus, ApiError> {
    guard(|| {
        let st = vcs_str(vcs_open(path)?.status())?;
        Ok(VcsStatus { branch: st.branch, head: st.head, changes: vcs_changes(st.changes), merge: st.merge, conflicts: st.conflicts.into_iter().map(|c| c.path).collect() })
    })
}

/// (1.4) Semantic differences between two revisions; `None` stands for the working file, so
/// `vcs_diff(p, Some("HEAD"), None)` is "what did I change since the last commit".
pub fn vcs_diff(path: &str, a: Option<&str>, b: Option<&str>) -> Result<Vec<VcsChange>, ApiError> {
    guard(|| Ok(vcs_changes(vcs_str(vcs_open(path)?.diff(a, b))?)))
}

/// (1.4) One commit and what it changed against its first parent.
pub fn vcs_show(path: &str, rev: &str) -> Result<VcsShow, ApiError> {
    guard(|| {
        let (id, c, ch) = vcs_str(vcs_open(path)?.show(rev))?;
        Ok(VcsShow { commit: VcsCommit { id, author: c.author, time: c.time, message: c.message }, changes: vcs_changes(ch) })
    })
}

/// (1.4) Delete stored objects no branch, tag or HEAD reaches. Aborts without deleting anything
/// if the history cannot be read.
pub fn vcs_gc(path: &str) -> Result<VcsGc, ApiError> {
    guard(|| {
        let r = vcs_str(vcs_open(path)?.gc())?;
        Ok(VcsGc { kept: r.kept, removed: r.removed })
    })
}

/// (1.4) Integrity check of the store: the list of problems, empty when healthy.
pub fn vcs_fsck(path: &str) -> Result<Vec<String>, ApiError> {
    guard(|| Ok(vcs_open(path)?.fsck()))
}
