<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA Portable (PortableApps.com format)

Sources of the portable package of the Windows studio, following the
[PortableApps.com Format](https://portableapps.com/development/portableapps.com_format) and the
[PortableApps.com Launcher](https://portableapps.com/apps/development/portableapps.com_launcher).

| File | Role |
|---|---|
| `App/AppInfo/appinfo.ini.in` | package metadata (name, licence flags, version filled in by the script) |
| `App/AppInfo/Launcher/OSATURAPortable.ini` | launcher configuration: starts `osatura-studio.exe`, keeps settings, projects and the WebView2 cache in `Data\` through three environment variables, no registry |
| `App/AppInfo/appicon*.{ico,png}` | package icon (original design, drawn by `tools/make-icons.py`) |
| `App/DefaultData/` | copied to `Data\` on first start |
| `help.html`, `Other/Source/README.txt` | help page and build notes |
| `make_package.py` | assembles the layout from the release archive `osatura-windows-x86_64.zip` |

```
python3 -I packaging/portableapps/make_package.py osatura-windows-x86_64.zip 0.1.0-alpha.5 out
```

**What this does not do.** The launcher program `OSATURAPortable.exe` and the `.paf.exe` installer are produced on Windows
by the PortableApps.com Launcher Generator and Installer (see `Other/Source/README.txt`); that has to be done, and the
result tested on a clean Windows account, before the package is offered. The `appinfo.ini` and launcher files were written
from the published format documentation and have **not** yet been checked by those tools.

The program support is in the application itself: `OSATURA_UI_CONFIG` (settings file, already supported),
`OSATURA_DOCUMENTS` (folder of the first project, new) and the WebView2 variable `WEBVIEW2_USER_DATA_FOLDER`.
