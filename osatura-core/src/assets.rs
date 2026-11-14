// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! One gate for every file a project refers to (image `source.file`, normal maps, sound tracks, ...).
//!
//! A project is data from somebody else: it must not be able to make the engine read `/etc/passwd`, a named
//! pipe, `/dev/zero`, or a URL that ffmpeg would fetch. [`resolve`] therefore accepts only relative paths
//! without `..` that, after symbolic links are followed, stay inside the project directory. The process-wide
//! switch [`set_allow_external`] (CLI `--allow-external-assets`) lifts the path restriction, and nothing else:
//! the file still has to be a regular file under the size caps ([`MAX_IMAGE_BYTES`], [`MAX_AUDIO_BYTES`]).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// Largest image / normal-map file read from disk.
pub const MAX_IMAGE_BYTES: u64 = 64 << 20;
/// Largest audio file read or handed to ffmpeg.
pub const MAX_AUDIO_BYTES: u64 = 256 << 20;
/// Largest "other" referenced file (golden images, project JSON in a manifest, ...).
pub const MAX_DATA_BYTES: u64 = 64 << 20;

static ALLOW_EXTERNAL: AtomicBool = AtomicBool::new(false);

/// Allow project files to name absolute paths and paths outside the project directory (default: off).
pub fn set_allow_external(on: bool) {
    ALLOW_EXTERNAL.store(on, Ordering::SeqCst);
}

pub fn allow_external() -> bool {
    ALLOW_EXTERNAL.load(Ordering::SeqCst)
}

fn looks_absolute(rel: &str) -> bool {
    let b = rel.as_bytes();
    Path::new(rel).is_absolute() || rel.starts_with('/') || rel.starts_with('\\') || (b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic())
}

/// The directory relative paths are taken from: an empty path (a project given as `p.json`) means `.`.
pub fn base_or_dot(base: &Path) -> PathBuf {
    if base.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        base.to_path_buf()
    }
}

/// Lexical rules only (no file system access): not empty, no NUL, no absolute path, no `..`.
pub fn check_relative(rel: &str) -> Result<(), String> {
    if rel.is_empty() {
        return Err("asset path is empty".into());
    }
    if rel.contains('\0') {
        return Err(format!("asset path {rel:?} contains a NUL byte"));
    }
    if looks_absolute(rel) {
        return Err(format!("asset '{rel}': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override)"));
    }
    if rel.split(['/', '\\']).any(|c| c == "..") {
        return Err(format!("asset '{rel}': '..' is not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override)"));
    }
    Ok(())
}

/// Same as [`resolve`] with the switch given explicitly (tests use this to stay independent of the global).
pub fn resolve_with(base_dir: &Path, rel: &str, allow_external: bool) -> Result<PathBuf, String> {
    let base = base_or_dot(base_dir);
    if !allow_external {
        check_relative(rel)?;
    } else if rel.is_empty() || rel.contains('\0') {
        return Err("asset path is empty or contains a NUL byte".into());
    }
    let joined = if allow_external && looks_absolute(rel) { PathBuf::from(rel) } else { base.join(rel) };
    match std::fs::canonicalize(&joined) {
        Ok(real) => {
            if !allow_external {
                let root = std::fs::canonicalize(&base).map_err(|e| format!("asset '{rel}': cannot access the project directory {}: {e}", base.display()))?;
                if !real.starts_with(&root) {
                    return Err(format!("asset '{rel}': resolves to {} which is outside the project directory (symbolic link?); use --allow-external-assets to override", real.display()));
                }
            }
            Ok(real)
        }
        // a file that does not exist is not read; callers report their own "cannot read" error for it
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(joined),
        Err(e) => Err(format!("asset '{rel}': cannot access {}: {e}", joined.display())),
    }
}

/// Resolve `rel` against `base_dir` under the project-directory rule (see the module comment). The result is a
/// canonical absolute path for existing files (so ffmpeg can never read it as a URL or option), otherwise the
/// plain joined path.
pub fn resolve(base_dir: &Path, rel: &str) -> Result<PathBuf, String> {
    resolve_with(base_dir, rel, allow_external())
}

/// `path` must be a regular file of at most `max` bytes. Returns its size. Missing files are an error here.
pub fn check_regular(path: &Path, max: u64, what: &str) -> Result<u64, String> {
    let md = std::fs::metadata(path).map_err(|e| format!("cannot read {what} {}: {e}", path.display()))?;
    if !md.is_file() {
        return Err(format!("{what} {} is not a regular file (devices, pipes and directories are refused)", path.display()));
    }
    if md.len() > max {
        return Err(format!("{what} {} is too large ({} bytes, limit {max})", path.display(), md.len()));
    }
    Ok(md.len())
}

/// Resolve and read a referenced file under the size cap `max`.
pub fn read(base_dir: &Path, rel: &str, max: u64, what: &str) -> Result<Vec<u8>, String> {
    let full = resolve(base_dir, rel)?;
    read_path(&full, max, what)
}

/// Read an already resolved path: regular file, size cap (also enforced while reading).
pub fn read_path(full: &Path, max: u64, what: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    check_regular(full, max, what)?;
    let f = std::fs::File::open(full).map_err(|e| format!("cannot read {what} {}: {e}", full.display()))?;
    let mut out = Vec::new();
    f.take(max + 1).read_to_end(&mut out).map_err(|e| format!("cannot read {what} {}: {e}", full.display()))?;
    if out.len() as u64 > max {
        return Err(format!("{what} {} is too large (limit {max} bytes)", full.display()));
    }
    Ok(out)
}

// ---- the directory relative assets are taken from ---------------------------------------------

static DEFAULT_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Process-wide directory that [`crate::engine::Engine::new`] resolves relative assets against (default `.`).
/// The command line sets it to the directory of the project it loaded, so a command that validates a derived
/// project (`write_json`, `lod`, `dof`, `size-report`, ...) looks for `img.png` next to the INPUT project and
/// not in whatever the current directory is. Libraries never set it.
pub fn set_default_dir(dir: Option<PathBuf>) {
    if let Ok(mut g) = DEFAULT_DIR.lock() {
        *g = dir;
    }
}

/// The directory set by [`set_default_dir`], or `.`.
pub fn default_dir() -> PathBuf {
    DEFAULT_DIR.lock().ok().and_then(|g| g.clone()).unwrap_or_else(|| PathBuf::from("."))
}

/// `target` expressed relative to `base` without any `..` (target must lie below base), else `None`.
fn relative_below(base: &Path, target: &Path) -> Option<String> {
    let rel = target.strip_prefix(base).ok()?;
    let parts: Vec<String> = rel.components().map(|c| c.as_os_str().to_string_lossy().into_owned()).collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}

fn rewrite_one(rel: &mut String, from: &Path, to: &Path, from_c: &Path, to_c: &Path, notes: &mut Vec<String>) {
    if looks_absolute(rel) || rel.is_empty() {
        return; // absolute paths stay valid wherever the project goes
    }
    let src = from.join(&*rel);
    let Ok(real) = std::fs::canonicalize(&src) else {
        return; // does not exist at the input either: nothing to keep valid (check reports it)
    };
    if let Some(new_rel) = relative_below(to_c, &real) {
        *rel = new_rel;
        return;
    }
    let _ = from_c;
    if allow_external() {
        *rel = real.to_string_lossy().into_owned();
        return;
    }
    notes.push(format!("asset '{rel}' ({}) is not inside the output directory {}: the project will not find it there; move the output next to the asset, copy the asset, or use --allow-external-assets", real.display(), to.display()));
}

/// Make the relative asset paths of `p` (image files, layer normal maps, sound tracks) stay valid when the
/// project is written to directory `to` after being read from directory `from`. A path is rewritten when the
/// file exists and lies below `to`; otherwise (a sibling directory: `..` is not allowed in a project) a note is
/// returned and the path is left alone (with `--allow-external-assets` it becomes absolute). Embedded
/// (base64) images and missing files are untouched. Returns the notes.
pub fn rewrite_relative_assets(p: &mut crate::project::Project, from: &Path, to: &Path) -> Vec<String> {
    let (from, to) = (base_or_dot(from), base_or_dot(to));
    let (Ok(from_c), Ok(to_c)) = (std::fs::canonicalize(&from), std::fs::canonicalize(&to)) else {
        return Vec::new();
    };
    if from_c == to_c {
        return Vec::new();
    }
    let mut notes = Vec::new();
    fn walk(ls: &mut [crate::project::Layer], from: &Path, to: &Path, fc: &Path, tc: &Path, notes: &mut Vec<String>) {
        for l in ls {
            if let Some(nm) = &mut l.normal_map {
                if let Some(pth) = &mut nm.path {
                    rewrite_one(pth, from, to, fc, tc, notes);
                }
            }
            match &mut l.kind {
                crate::project::LayerKind::Image { source, .. } => {
                    if let Some(f) = &mut source.file {
                        rewrite_one(f, from, to, fc, tc, notes);
                    }
                }
                crate::project::LayerKind::Group { layers } | crate::project::LayerKind::Selector { layers, .. } => walk(layers, from, to, fc, tc, notes),
                _ => {}
            }
        }
    }
    walk(&mut p.layers, &from, &to, &from_c, &to_c, &mut notes);
    for s in &mut p.sounds {
        rewrite_one(&mut s.file, &from, &to, &from_c, &to_c, &mut notes);
    }
    notes.sort();
    notes.dedup();
    notes
}

// ---- decompression bombs ---------------------------------------------------------------------

/// Most pixels one decoded image may have (64 Mpixel = 256 MB as RGBA8).
pub const MAX_PIXELS: u64 = 64_000_000;
/// Longest side of a decoded image.
pub const MAX_SIDE: u32 = 16384;

/// Size rule for decoded images: both sides 1..=16384 and at most 64 Mpixel.
pub fn check_dims(w: u32, h: u32) -> Result<(), String> {
    if w == 0 || h == 0 {
        return Err("image is empty (width or height is 0)".into());
    }
    if w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!("image is {w}x{h}: a side may be at most {MAX_SIDE} pixels"));
    }
    if (w as u64) * (h as u64) > MAX_PIXELS {
        return Err(format!("image is {w}x{h} = {} Mpixel, more than the limit of {} Mpixel", (w as u64) * (h as u64) / 1_000_000, MAX_PIXELS / 1_000_000));
    }
    Ok(())
}

/// Read the size from the PNG header (IHDR) alone and apply [`check_dims`]; nothing is allocated.
pub fn check_png(bytes: &[u8]) -> Result<(u32, u32), String> {
    const SIG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.len() < 24 || bytes[..8] != SIG || &bytes[12..16] != b"IHDR" {
        return Err("not a PNG (bad signature or header)".into());
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    check_dims(w, h)?;
    Ok((w, h))
}

/// Read the size from a JPEG header and apply [`check_dims`] (no pixel data is decoded).
pub fn check_jpeg(bytes: &[u8]) -> Result<(u32, u32), String> {
    let mut d = jpeg_decoder::Decoder::new(bytes);
    d.read_info().map_err(|e| format!("invalid JPEG: {e}"))?;
    let i = d.info().ok_or("invalid JPEG")?;
    check_dims(i.width as u32, i.height as u32)?;
    Ok((i.width as u32, i.height as u32))
}

/// `tiny_skia::Pixmap::decode_png` after the header check: an image that claims a huge size is refused
/// before any pixel buffer is allocated.
pub fn decode_png_checked(bytes: &[u8]) -> Result<tiny_skia::Pixmap, String> {
    check_png(bytes)?;
    tiny_skia::Pixmap::decode_png(bytes).map_err(|e| format!("invalid PNG: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> crate::fsutil::TempDir {
        crate::fsutil::private_temp_dir("osatura-assets-test").unwrap()
    }

    #[test]
    fn relative_paths_inside_the_project_work_absolute_and_dotdot_do_not() {
        let d = dir();
        std::fs::create_dir(d.join("img")).unwrap();
        std::fs::write(d.join("img/a.png"), b"x").unwrap();
        let ok = resolve_with(d.path(), "img/a.png", false).unwrap();
        assert!(ok.ends_with("img/a.png") && ok.is_absolute());
        assert!(resolve_with(d.path(), "./img/../img/a.png", false).is_err(), "'..' is refused even when it would stay inside");
        let abs = d.join("img/a.png");
        let e = resolve_with(d.path(), abs.to_str().unwrap(), false).unwrap_err();
        assert!(e.contains("absolute"), "{e}");
        assert!(resolve_with(d.path(), "../etc/passwd", false).unwrap_err().contains("'..'"));
        assert!(resolve_with(d.path(), "C:\\x", false).is_err() && resolve_with(d.path(), "\\\\srv\\x", false).is_err());
        assert!(resolve_with(d.path(), "", false).is_err() && resolve_with(d.path(), "a\0b", false).is_err());
        // with the switch the same paths pass
        assert!(resolve_with(d.path(), abs.to_str().unwrap(), true).is_ok());
        assert!(resolve_with(&d.join("img"), "../img/a.png", true).is_ok());
    }

    #[cfg(unix)] // asserts POSIX path separators
    #[test]
    fn a_missing_file_resolves_to_the_joined_path_and_an_empty_base_means_dot() {
        let p = resolve_with(Path::new(""), "http://example.invalid/x.wav", false).unwrap();
        assert!(p.starts_with("./"), "never a bare URL-looking path: {}", p.display());
        assert_eq!(base_or_dot(Path::new("")), PathBuf::from("."));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_pointing_out_of_the_project_are_refused() {
        let d = dir();
        let outside = dir();
        std::fs::write(outside.join("secret.txt"), b"s").unwrap();
        std::os::unix::fs::symlink(outside.join("secret.txt"), d.join("link.png")).unwrap();
        std::os::unix::fs::symlink(outside.path(), d.join("dirlink")).unwrap();
        let e = resolve_with(d.path(), "link.png", false).unwrap_err();
        assert!(e.contains("outside the project directory"), "{e}");
        assert!(resolve_with(d.path(), "dirlink/secret.txt", false).is_err());
        assert!(resolve_with(d.path(), "link.png", true).is_ok());
        // a link that stays inside is fine
        std::fs::write(d.join("real.png"), b"x").unwrap();
        std::os::unix::fs::symlink(d.join("real.png"), d.join("inner.png")).unwrap();
        assert!(resolve_with(d.path(), "inner.png", false).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn only_regular_files_under_the_cap_are_read() {
        let d = dir();
        assert!(check_regular(Path::new("/dev/zero"), 1 << 20, "image").unwrap_err().contains("not a regular file"));
        assert!(check_regular(d.path(), 10, "image").is_err(), "a directory");
        let fifo = d.join("pipe.png");
        if std::process::Command::new("mkfifo").arg(&fifo).status().map(|s| s.success()).unwrap_or(false) {
            // must return at once instead of blocking on the pipe
            assert!(read_path(&fifo, 1 << 20, "image").unwrap_err().contains("not a regular file"));
        }
        let big = d.join("big.bin");
        let f = std::fs::File::create(&big).unwrap();
        f.set_len(2000).unwrap();
        assert!(read_path(&big, 1000, "image").unwrap_err().contains("too large"));
        assert_eq!(read_path(&big, 4000, "image").unwrap().len(), 2000);
    }

    fn png_header(w: u32, h: u32) -> Vec<u8> {
        let mut v = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, b'I', b'H', b'D', b'R'];
        v.extend_from_slice(&w.to_be_bytes());
        v.extend_from_slice(&h.to_be_bytes());
        v.extend_from_slice(&[8, 6, 0, 0, 0]);
        v
    }

    #[test]
    fn a_png_header_that_claims_a_huge_size_is_refused_before_allocation() {
        // a 40-byte file declaring 60000 x 60000 (14 GB as RGBA): must fail on the header, instantly
        let e = decode_png_checked(&png_header(60000, 60000)).unwrap_err();
        assert!(e.contains("at most 16384"), "{e}");
        let e = decode_png_checked(&png_header(16384, 16384)).unwrap_err();
        assert!(e.contains("Mpixel"), "{e}");
        assert!(check_png(&png_header(0, 5)).is_err());
        assert!(check_png(b"not a png at all, not a png at all").is_err());
        assert_eq!(check_png(&png_header(8000, 8000)).unwrap(), (8000, 8000));
        let ok = tiny_skia::Pixmap::new(4, 3).unwrap().encode_png().unwrap();
        assert_eq!(decode_png_checked(&ok).unwrap().width(), 4);
    }
}
