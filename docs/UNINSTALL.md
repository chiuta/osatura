<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Uninstalling OSATURA

OSATURA has no installer. It does not write to the Windows registry, does not change `PATH`, and installs no service, driver, scheduled task or startup entry. To uninstall it, delete the files listed below.

## 1. The program

Delete the folder you unpacked from the release archive (for example `osatura-windows-x86_64`, `osatura-macos-arm64` or `osatura-linux-x86_64`). If you built from source, delete the `target` folder of your checkout, or remove a `cargo install` copy with `cargo uninstall osatura-cli`.

## 2. Files OSATURA may have created

| What | Windows | macOS | Linux | Created when |
|---|---|---|---|---|
| First project | `%USERPROFILE%\Documents\OSATURA\first-rig.json` | `~/Documents/OSATURA/first-rig.json` | (not created) | you double-click `osatura.exe`, or start `osatura-studio` without a project; an existing file is never overwritten |
| Studio settings (shortcuts, panel layout) | `%APPDATA%\OSATURA\ui.json` | `~/.config/osatura/ui.json` | `$XDG_CONFIG_HOME/osatura/ui.json` or `~/.config/osatura/ui.json` | you change a setting in the studio (`$OSATURA_UI_CONFIG` overrides the path) |
| Web view cache of the window app | `osatura-studio.exe.WebView2` next to `osatura-studio.exe` (WebView2's default location) | WebKit data under `~/Library/WebKit/` and `~/Library/Caches/` named after `osatura-studio` | (no window app) | you open `osatura-studio` |

The `Documents\OSATURA` folder may also contain your own projects; look before deleting it.

Everything else (renders, exports, projects) is written only to the paths you give the commands.

## 3. Optional

* `ffmpeg`, if you installed it for video export, is a separate program; uninstall it the way you installed it.
* Nothing else is left behind.
