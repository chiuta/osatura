<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Privacy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

In detail, for `osatura`, `osatura-studio`, the C library and the WebAssembly module:

* **No telemetry, analytics, crash reporting, advertising or automatic update checks.** The programs contain no code that contacts a server on their own.
* **One command that does contact a server, only when you type it.** `osatura update-check` makes a single anonymous HTTPS request to GitHub (`api.github.com`) for the public list of releases, using the `curl` program of your system; GitHub sees your IP address and that request, under [GitHub's privacy statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). Nothing is downloaded or installed and nothing is sent about you or your projects. To keep your address from GitHub, add `--proxy-preset tor` (or `tor-browser`, `warp`) or `--proxy-url` with your own proxy; see [docs/CLI_REFERENCE.md](docs/CLI_REFERENCE.md).
* **The studio is a local server.** `osatura studio` and `osatura-studio` listen on `127.0.0.1` (this computer only) by default. They are reachable from other machines only if you start them with `--bind` on another address yourself ([docs/SECURITY.md](docs/SECURITY.md)). The page they serve loads nothing from the internet (its Content-Security-Policy allows only the local server).
* **Links you click.** The studio and the documentation contain links (for example to GitHub or a donation page). Following one opens that site in your browser, under that site's privacy policy.
* **External programs.** `ffmpeg` is started only for video and some audio formats, only when you ask for such an export, and works on local files.
* **Shared sessions keep a history.** With `--collab-log` the studio writes every edit of the session, with the participant names, to a file you choose; with per-participant tokens the names are the ones you minted. Treat that file as personal data of the participants and delete it when the session is over ([docs/SECURITY.md](docs/SECURITY.md), [docs/UNINSTALL.md](docs/UNINSTALL.md)). Face-landmark tools write their results only to the files you name.
* **Your files stay yours.** Projects, renders and settings are written only where you tell the program to write them, plus the small settings file listed in [docs/UNINSTALL.md](docs/UNINSTALL.md).

The project's website (<https://osatura.chiuta.ro>) is a static page hosted by GitHub Pages; GitHub's own privacy statement applies to it.

Questions: <alexio@trom.tf>.
