// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Small file-system helpers shared by the library and the command line: random bytes, atomic writes
//! (temporary file in the same directory, `fsync`, `rename`), private temporary directories (created
//! exclusively with mode 0700, so a predictable name in a shared `/tmp` cannot be pre-created or hijacked
//! by another user) and files that only the owner may read (mode 0600).

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// `n` unpredictable bytes: the operating system's generator when it can be read, otherwise a SHA-256 of
/// clock, process id, a counter and addresses (weaker, but never a fixed or sequential value).
pub fn random_bytes(n: usize) -> Vec<u8> {
    let mut out = vec![0u8; n];
    #[cfg(unix)]
    {
        use std::io::Read;
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            if f.read_exact(&mut out).is_ok() {
                return out;
            }
        }
    }
    #[cfg(windows)]
    {
        // The system generator (RtlGenRandom, exported by advapi32 as SystemFunction036): no extra crate, no handle.
        #[link(name = "advapi32")]
        extern "system" {
            fn SystemFunction036(buf: *mut u8, len: u32) -> u8;
        }
        let ok = out.chunks_mut(1 << 20).all(|c| {
            // SAFETY: `c` is a valid, writable slice of exactly `c.len()` bytes (at most 1 MiB, so it fits a u32).
            unsafe { SystemFunction036(c.as_mut_ptr(), c.len() as u32) != 0 }
        });
        if ok {
            return out;
        }
    }
    let mut filled = 0;
    while filled < n {
        let t = crate::clock::unix_nanos();
        let c = COUNTER.fetch_add(1, Ordering::SeqCst);
        let local = 0u8;
        let seed = format!("{t}/{}/{c}/{:p}/{:?}", crate::clock::process_id(), &local, std::thread::current().id());
        let h = crate::vcs::sha256_hex(seed.as_bytes());
        for i in (0..h.len()).step_by(2) {
            if filled < n {
                out[filled] = u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
                filled += 1;
            }
        }
    }
    out
}

/// `n` random bytes as lowercase hex (2n characters).
pub fn random_hex(n: usize) -> String {
    random_bytes(n).iter().map(|b| format!("{b:02x}")).collect()
}

fn sibling_tmp(path: &Path) -> PathBuf {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "out".into());
    path.with_file_name(format!(".{name}.tmp-{}-{}", crate::clock::process_id(), random_hex(6)))
}

#[cfg(unix)]
fn open_new(path: &Path, mode: u32) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new().write(true).create_new(true).mode(mode).open(path)
}

#[cfg(not(unix))]
fn open_new(path: &Path, _mode: u32) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new().write(true).create_new(true).open(path)
}

/// Create a new file that only the owner can read or write (mode 0600); fails when it already exists.
pub fn create_private_file(path: &Path) -> std::io::Result<std::fs::File> {
    open_new(path, 0o600)
}

static FOLLOW_SYMLINKS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Let [`write_atomic`] write through a symbolic link (to the link's target) instead of refusing it. Off by default:
/// a link in a shared directory could redirect a write to a file the user never named. The command line sets it
/// from `--follow-symlinks`.
pub fn set_follow_symlinks(on: bool) {
    FOLLOW_SYMLINKS.store(on, Ordering::SeqCst);
}

/// Overwrite an existing regular file in place (truncate, write, `fsync`). Not atomic: a crash or a full disk in the
/// middle leaves a short file. Used only when no temporary file can be created next to the target.
fn write_in_place(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new().write(true).truncate(true).open(path)?;
    f.write_all(data)?;
    f.sync_all()
}

fn write_atomic_mode(path: &Path, data: &[u8], mode: u32) -> Result<(), String> {
    let mut real = path.to_path_buf();
    if let Ok(md) = std::fs::symlink_metadata(path) {
        if md.file_type().is_symlink() {
            let target = std::fs::canonicalize(path).ok();
            match (FOLLOW_SYMLINKS.load(Ordering::SeqCst), target) {
                (true, Some(t)) => real = t,
                (true, None) => return Err(format!("{}: is a symbolic link whose target does not exist", path.display())),
                (false, t) => {
                    return Err(format!(
                        "{}: refusing to write through a symbolic link{}; pass --follow-symlinks to write to the link's target, or name the target file yourself",
                        path.display(),
                        t.map(|t| format!(" (it points to {})", t.display())).unwrap_or_default()
                    ))
                }
            }
        }
    }
    let path = real.as_path();
    if let Ok(md) = std::fs::symlink_metadata(path) {
        if md.is_dir() {
            return Err(format!("{}: is a directory", path.display()));
        }
    }
    let tmp = sibling_tmp(path);
    let io = |what: &str, e: std::io::Error| format!("{}: {what}: {e}", path.display());
    let created = open_new(&tmp, mode);
    let mut f = match created {
        Ok(f) => f,
        Err(e) => {
            // No temporary file can be made next to the target (a read-only or unwritable directory) but the file
            // itself may still be writable: overwrite it in place. The write is then not atomic (a crash mid-write
            // leaves a truncated file), which is the price of an output the directory would otherwise forbid.
            let is_file = std::fs::metadata(path).map(|m| m.is_file()).unwrap_or(false);
            if is_file && matches!(e.kind(), std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem) {
                return write_in_place(path, data).map_err(|e2| io("cannot create a temporary file in this directory and the file itself is not writable either", e2));
            }
            return Err(io("cannot create temporary file", e));
        }
    };
    let res = (|| {
        f.write_all(data).map_err(|e| io("write failed", e))?;
        f.sync_all().map_err(|e| io("fsync failed", e))?;
        drop(f);
        if let Ok(md) = std::fs::metadata(path) {
            let _ = std::fs::set_permissions(&tmp, md.permissions());
        }
        std::fs::rename(&tmp, path).map_err(|e| io("rename failed", e))
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

/// Fail when `path` is a symbolic link (unless [`set_follow_symlinks`] is on). For outputs that another program
/// (ffmpeg) or an append-only log opens itself, so [`write_atomic`] cannot do the check.
pub fn refuse_symlink(path: &Path) -> Result<(), String> {
    if FOLLOW_SYMLINKS.load(Ordering::SeqCst) {
        return Ok(());
    }
    match std::fs::symlink_metadata(path) {
        Ok(md) if md.file_type().is_symlink() => Err(format!(
            "{}: refusing to write through a symbolic link{}; pass --follow-symlinks to write to the link's target, or name the target file yourself",
            path.display(),
            std::fs::canonicalize(path).ok().map(|t| format!(" (it points to {})", t.display())).unwrap_or_default()
        )),
        _ => Ok(()),
    }
}

/// [`write_atomic`] after creating the missing parent directories of `path`.
pub fn write_atomic_mkdir(path: &Path, data: &[u8]) -> Result<(), String> {
    if let Some(d) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(d).map_err(|e| format!("cannot create {}: {e}", d.display()))?;
    }
    write_atomic(path, data)
}

/// Write `data` to `path` atomically: either the old file or the complete new one is ever visible. A symbolic
/// link as `path` is refused unless [`set_follow_symlinks`] is on (then the link's target is written); an existing
/// file keeps its permissions. If the directory does not allow a temporary file but the target file is writable,
/// the file is overwritten in place (not atomic; see `write_in_place`).
pub fn write_atomic(path: &Path, data: &[u8]) -> Result<(), String> {
    write_atomic_mode(path, data, 0o666)
}

/// Like [`write_atomic`] for files with secrets: the new file is created with mode 0600.
pub fn write_atomic_private(path: &Path, data: &[u8]) -> Result<(), String> {
    write_atomic_mode(path, data, 0o600)
}

/// A file name as an ffmpeg/ffprobe argument: never read as an option (`-x`) or as a URL (`http://...`,
/// `concat:...`): such names get a `./` in front.
pub fn file_arg(p: &str) -> String {
    let first = p.split('/').next().unwrap_or("");
    if p.starts_with('/') || p.starts_with("./") || p.starts_with("../") || (!p.starts_with('-') && !first.contains(':') && !first.contains('|')) {
        p.to_string()
    } else {
        format!("./{p}")
    }
}

/// A directory made for this process only; removed with everything in it when dropped.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn join(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

/// A fresh empty scratch directory `<tmp>/<tag>-<random>` for tests; it and its contents are removed when the value is
/// dropped (also when an assertion unwinds). Derefs to the path, so `d.join(..)` and `&d` work as with a `PathBuf`.
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    pub fn new(tag: &str) -> ScratchDir {
        let t = private_temp_dir(tag).expect("scratch dir");
        let path = t.path.clone();
        std::mem::forget(t); // ownership moves to the ScratchDir (same cleanup on drop)
        ScratchDir { path }
    }
}

impl std::ops::Deref for ScratchDir {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.path
    }
}

impl AsRef<Path> for ScratchDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Create `<tmp>/<prefix>-<random>` with `mkdir` (never reusing an existing name) and mode 0700.
pub fn private_temp_dir(prefix: &str) -> Result<TempDir, String> {
    let base = std::env::temp_dir();
    for _ in 0..16 {
        let p = base.join(format!("{prefix}-{}", random_hex(8)));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut b = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            b.mode(0o700);
        }
        match b.create(&p) {
            Ok(()) => return Ok(TempDir { path: p }),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot create a private temporary directory in {}: {e}", base.display())),
        }
    }
    Err("cannot find a free temporary directory name".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_values_differ_and_have_the_right_length() {
        let (a, b) = (random_hex(16), random_hex(16));
        assert_eq!(a.len(), 32);
        assert_ne!(a, b);
        assert!(a.bytes().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn atomic_write_replaces_whole_and_leaves_no_temp_file() {
        let d = private_temp_dir("osatura-fsutil-test").unwrap();
        let p = d.join("a.json");
        write_atomic(&p, b"one").unwrap();
        write_atomic(&p, b"two").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"two");
        let names: Vec<_> = std::fs::read_dir(d.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names.len(), 1, "{names:?}");
        assert!(write_atomic(d.path(), b"x").is_err(), "a directory is not a file");
    }

    #[test]
    fn ffmpeg_file_names_never_look_like_options_or_urls() {
        assert_eq!(file_arg("out.mp4"), "out.mp4");
        assert_eq!(file_arg("/tmp/a b.mp4"), "/tmp/a b.mp4");
        assert_eq!(file_arg("-y"), "./-y");
        assert_eq!(file_arg("http://x/y.mp4"), "http://x/y.mp4".replacen("http:", "./http:", 1));
        assert_eq!(file_arg("concat:a|b"), "./concat:a|b");
        assert_eq!(file_arg("dir/with:colon"), "dir/with:colon");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_targets_are_refused_and_private_dirs_are_0700() {
        use std::os::unix::fs::PermissionsExt;
        let d = private_temp_dir("osatura-fsutil-test").unwrap();
        assert_eq!(std::fs::metadata(d.path()).unwrap().permissions().mode() & 0o777, 0o700);
        let victim = d.join("victim.txt");
        std::fs::write(&victim, b"precious").unwrap();
        let link = d.join("link.json");
        std::os::unix::fs::symlink(&victim, &link).unwrap();
        let e = write_atomic(&link, b"overwritten").unwrap_err();
        assert!(e.contains("symbolic link"), "{e}");
        assert_eq!(std::fs::read(&victim).unwrap(), b"precious");
        let q = d.join("secret.log");
        write_atomic_private(&q, b"s").unwrap();
        assert_eq!(std::fs::metadata(&q).unwrap().permissions().mode() & 0o777, 0o600);
        assert!(create_private_file(&q).is_err(), "exclusive creation");
        let dir_path = d.path().to_path_buf();
        drop(d);
        assert!(!dir_path.exists(), "removed on drop");
    }
}
