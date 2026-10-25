<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Privacy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

In detail, for `osatura`, `osatura-studio`, the C library and the WebAssembly module:

* **No telemetry, analytics, crash reporting, advertising or update checks.** The programs contain no code that contacts a server on their own.
* **The studio is a local server.** `osatura studio` and `osatura-studio` listen on `127.0.0.1` (this computer only) by default. They are reachable from other machines only if you start them with `--bind` on another address yourself ([docs/SECURITY.md](docs/SECURITY.md)). The page they serve loads nothing from the internet (its Content-Security-Policy allows only the local server).
* **Links you click.** The studio and the documentation contain links (for example to GitHub or a donation page). Following one opens that site in your browser, under that site's privacy policy.
* **External programs.** `ffmpeg` is started only for video and some audio formats, only when you ask for such an export, and works on local files.
* **Your files stay yours.** Projects, renders and settings are written only where you tell the program to write them, plus the small settings file listed in [docs/UNINSTALL.md](docs/UNINSTALL.md).

The project's website (<https://osatura.chiuta.ro>) is a static page hosted by GitHub Pages; GitHub's own privacy statement applies to it.

Questions: <alexio@trom.tf>.
