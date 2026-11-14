// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Windows only: embeds the file metadata (product name, version, copyright) into `osatura-studio.exe`. Code signing
//! (SignPath Foundation, CODE_SIGNING_POLICY.md) requires every signed file to carry the project's product name and
//! the release version. Other platforms: nothing happens.
//!
//! `OSATURA_VERSION` (set by the release workflow, e.g. `0.1.0-alpha.4`) overrides the crate version. A missing
//! resource compiler is only a warning, so a local Windows build without the Windows SDK still works; the release
//! workflow sets `OSATURA_REQUIRE_WIN_METADATA=1` to make it an error.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=OSATURA_VERSION");
    println!("cargo:rerun-if-env-changed=OSATURA_REQUIRE_WIN_METADATA");
    #[cfg(windows)]
    embed();
}

#[cfg(windows)]
fn embed() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let pkg = std::env::var("CARGO_PKG_VERSION").unwrap_or_default();
    let version = std::env::var("OSATURA_VERSION").ok().filter(|v| !v.trim().is_empty()).unwrap_or(pkg);
    let mut r = winresource::WindowsResource::new();
    r.set("ProductName", "OSATURA")
        .set("FileDescription", "OSATURA Studio")
        .set("CompanyName", "Alexandru-Ionuț Chiuță (Alexio)")
        .set("LegalCopyright", "Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio). AGPL-3.0-or-later")
        .set("OriginalFilename", "osatura-studio.exe")
        .set("InternalName", "osatura-studio")
        .set("ProductVersion", &version)
        .set("FileVersion", &version)
        .set("Comments", "https://github.com/chiuta/osatura");

    if let Err(e) = r.compile() {
        if std::env::var("OSATURA_REQUIRE_WIN_METADATA").as_deref() == Ok("1") {
            panic!("cannot embed the Windows file metadata: {e}");
        }
        println!("cargo:warning=Windows file metadata not embedded ({e}); install the Windows SDK to get it");
    }
}
