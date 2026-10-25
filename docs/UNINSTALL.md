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

| Backups of the settings | `ui.json.bak`, `ui.json.bak.1`, ... next to `ui.json` (up to 1000) | same, next to `ui.json` | same, next to `ui.json` | the settings file could not be read and was replaced by defaults, or the studio updated it from an older format |
| Temporary files | the system temp folder (`%TEMP%`), folders named `osatura-*` | `$TMPDIR`, folders named `osatura-*` | `/tmp`, folders named `osatura-*` | audio mixing, video export and a few tools; normally removed when the command ends, but a crash can leave them |
| Collaboration log | wherever you pointed `--collab-log` | same | same | you ran `osatura studio --collab-log LOG`; it holds the whole edit history of the session (see [PRIVACY.md](PRIVACY.md)) |

Files in a synced folder (OneDrive, iCloud, Dropbox) are also in that service's copy and its recycle bin: delete them there too if you want them gone.

The `Documents\OSATURA` folder may also contain your own projects; look before deleting it.

Everything else (renders, exports, projects) is written only to the paths you give the commands.

## 3. Optional

* `ffmpeg`, if you installed it for video export, is a separate program; uninstall it the way you installed it.
* Nothing else is left behind.
