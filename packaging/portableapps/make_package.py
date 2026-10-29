#!/usr/bin/env python3 -I
# SPDX-License-Identifier: MPL-2.0
# Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
"""Assembles the PortableApps.com folder layout of OSATURA Portable from the release archive for Windows.

    python3 -I make_package.py osatura-windows-x86_64.zip 0.1.0-alpha.5 [output-folder]

Writes <output>/OSATURAPortable/ (and OSATURAPortable_<version>_layout.zip next to it). The launcher program
(OSATURAPortable.exe) and the installer are built afterwards on Windows with the PortableApps.com Launcher Generator and
Installer (see README.md): they cannot be produced here. Standard library only."""
import re
import shutil
import sys
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent


def package_version(version: str):
    """`0.1.0-alpha.5` -> ('0.1.0.5', '0.1.0 Alpha 5'). The 4th number keeps releases ordered: alpha.N = N,
    beta.N = 50+N, rc.N = 80+N, the final release = 100 (a package version is four dot-separated numbers)."""
    m = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(?:-(alpha|beta|rc)\.(\d+))?", version)
    if not m:
        sys.exit(f"version '{version}' is not MAJOR.MINOR.PATCH or MAJOR.MINOR.PATCH-(alpha|beta|rc).N")
    a, b, c, kind, n = m.groups()
    base = {"alpha": 0, "beta": 50, "rc": 80}.get(kind, 100)
    last = base + int(n or 0)
    if kind and int(n) > 49 or last > 255:
        sys.exit("pre-release number too large for the package version scheme")
    shown = f"{a}.{b}.{c}" + (f" {("RC" if kind == "rc" else kind.capitalize())} {n}" if kind else "")
    return f"{a}.{b}.{c}.{last}", shown


def main(argv):
    if len(argv) not in (3, 4):
        sys.exit(__doc__)
    archive, version = Path(argv[1]), argv[2]
    out = Path(argv[3]) if len(argv) == 4 else Path.cwd()
    pkg_ver, shown = package_version(version)
    root = out / "OSATURAPortable"
    if root.exists():
        shutil.rmtree(root)
    # the application: everything of the release archive, under App\OSATURA
    app = root / "App" / "OSATURA"
    with zipfile.ZipFile(archive) as z:
        names = [n for n in z.namelist() if not n.endswith("/")]
        top = {n.split("/", 1)[0] for n in names}
        if len(top) != 1:
            sys.exit(f"{archive}: expected one top-level folder, found {sorted(top)}")
        for n in names:
            rel = Path(n.split("/", 1)[1])
            if rel.is_absolute() or ".." in rel.parts:
                sys.exit(f"{archive}: unsafe path {n}")
            dst = app / rel
            dst.parent.mkdir(parents=True, exist_ok=True)
            with z.open(n) as s, open(dst, "wb") as d:
                shutil.copyfileobj(s, d)
    for need in ("osatura.exe", "osatura-studio.exe"):
        if not (app / need).is_file():
            sys.exit(f"{archive}: {need} is missing; this is not the Windows archive")
    # package metadata, launcher configuration, icons, default data
    info = root / "App" / "AppInfo"
    shutil.copytree(HERE / "App" / "AppInfo", info)
    tpl = (info / "appinfo.ini.in").read_text(encoding="utf-8")
    (info / "appinfo.ini.in").unlink()
    (info / "appinfo.ini").write_text(tpl.replace("@PACKAGE_VERSION@", pkg_ver).replace("@DISPLAY_VERSION@", shown), encoding="utf-8", newline="\r\n")
    for f in (info / "Launcher").glob("*.ini"):
        f.write_text(f.read_text(encoding="utf-8"), encoding="utf-8", newline="\r\n")
    shutil.copytree(HERE / "App" / "DefaultData", root / "App" / "DefaultData")
    (root / "Other" / "Source").mkdir(parents=True)
    shutil.copy(HERE / "Other" / "Source" / "README.txt", root / "Other" / "Source" / "README.txt")
    shutil.copy(info / "Launcher" / "OSATURAPortable.ini", root / "Other" / "Source" / "OSATURAPortable.ini")
    help_html = (HERE / "help.html").read_text(encoding="utf-8").replace("@DISPLAY_VERSION@", shown)
    (root / "help.html").write_text(help_html, encoding="utf-8")
    # a layout archive, for the person who builds the launcher and the installer
    zpath = out / f"OSATURAPortable_{version}_layout.zip"
    with zipfile.ZipFile(zpath, "w", zipfile.ZIP_DEFLATED) as z:
        for f in sorted(root.rglob("*")):
            if f.is_file():
                z.write(f, f.relative_to(out).as_posix())
    print(f"{root}\n{zpath}\npackage version {pkg_ver} ({shown})")


if __name__ == "__main__":
    main(sys.argv)
