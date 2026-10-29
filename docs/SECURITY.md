<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Security

**Languages:** [English](https://github.com/chiuta/osatura/blob/main/docs/SECURITY.md) · [Română](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/ro/SECURITY.md) · [Français](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/fr/SECURITY.md) · [Italiano](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/it/SECURITY.md) · [Español](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/es/SECURITY.md) · [Português](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/pt/SECURITY.md) · [Deutsch](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/de/SECURITY.md) · [Magyar](https://github.com/chiuta/osatura/blob/landing-page/docs/i18n/hu/SECURITY.md)

*Threat model, built-in protections, known gaps, how to report a problem and how to deploy the studio safely. Documentation CC-BY-4.0. Derived from the source (`osatura-core/src/{api,assets,fsutil,expr,script}.rs`, `osatura-cli/src/{http,studio,collab_srv}.rs`), from `ROADMAP.md` section 9 (review waves 1 to 4) and from the tests named below. **OSATURA has had no independent security audit.** The review waves were done by the project itself; treat this page as a description of intent and of what the tests cover, not as a guarantee.*

Contents: [1 Threat model](#1-threat-model) - [2 Studio HTTP server](#2-studio-http-server) - [3 Authentication and collaboration](#3-authentication-and-collaboration) - [4 Asset resolution and input limits](#4-asset-resolution-and-input-limits) - [5 Expression and script sandbox](#5-expression-and-script-sandbox) - [6 Files, symlinks and temporary data](#6-files-symlinks-and-temporary-data) - [7 External programs (ffmpeg)](#7-external-programs-ffmpeg) - [8 Libraries (C ABI, WASM, facade)](#8-libraries-c-abi-wasm-facade) - [9 Known gaps](#9-known-gaps) - [10 Reporting a vulnerability](#10-reporting-a-vulnerability) - [11 Deployer hardening checklist](#11-deployer-hardening-checklist) - [12 Reverse proxy with TLS: nginx and Caddy](#12-reverse-proxy-with-tls-nginx-and-caddy)

## 1. Threat model

What OSATURA processes: **project files** (JSON), images, audio, Lottie / OCA / glTF-style import files, mapping files, and, in the studio, HTTP requests from a browser. Project files are *untrusted data* in the model below: a project received from someone else may be hostile, and opening it must not run code, read files outside its folder or exhaust the machine.

| Actor | Can | Goal assumed | Main defences |
|---|---|---|---|
| Author of a hostile project / import file | give you a file | read your files, crash or hang the tool, run code, fill the disk | strict parsing, size and depth limits, asset-path refusal, no panics on input, expression and script sandbox, atomic output |
| Web page you visit while the studio runs on localhost | make your browser send requests to `127.0.0.1` (CSRF, DNS rebinding) | edit your project, read it, write files | Host allow-list, Origin / `Sec-Fetch-Site` checks, JSON content type, token with cookie `SameSite=Strict`, CSP |
| Other user on the same machine | connect to the port, read `/tmp` | read or hijack a session or temp files | binds to `127.0.0.1` by default, private temp dirs (0700), 0600 log files, token required to bind elsewhere |
| Someone on your network, when the studio is bound beyond localhost | reach the port | use or guess the token, exhaust connections | mandatory token (16+ characters), constant-time comparison, back-off, connection and size caps; **no TLS** |
| A collaborator with the token | edit the shared session | damage the project | undo history, the hash-chained log; collaborators are trusted with the project, there are no accounts or roles |
| Hostile caller of the libraries | pass bad arguments to the facade, C or WASM ABI | memory corruption, panics across the boundary | argument validation, size checks before allocation, panics caught (where the target unwinds) |

The only outbound request OSATURA can make is `osatura update-check`, and only when you run it (one anonymous HTTPS GET of the public release list through the system `curl`: HTTPS only, TLS 1.2+, 30 s, 2 MB, the printed link is checked to be on `github.com/chiuta/osatura`). It can go through Tor, Tor Browser, Cloudflare WARP or your own proxy; the presets are local `socks5h://` endpoints so DNS is not resolved outside the proxy. The tests are in `update_cmd.rs`.

Out of scope: protecting against a malicious local administrator, a compromised build toolchain or dependency (see [THIRD_PARTY.md](THIRD_PARTY.md) for what is shipped; `cargo deny` polices licences, bans and sources, not vulnerabilities), side channels, and the security of ffmpeg itself.

## 2. Studio HTTP server

`osatura studio` serves the editor from its own small HTTP/1.1 implementation (`osatura-cli/src/http.rs`, standard library only, no third-party web framework). Behaviour, with the tests that pin it (`osatura-cli` unit tests, `tests/robust.rs`, `tests/wave4.rs`, `tests/source_offer.rs`, and the Chromium smoke tests):

**Limits**

| Limit | Value | Result when exceeded |
|---|---|---|
| Time to receive the request head | 10 s | 408 |
| Time to receive the body | 20 s | 408 |
| Time to write a response | 10 s | connection dropped |
| Request head size | 16 KB | 431 |
| Number of headers | 100 | 431 |
| Body size | 32 MB | 413 |
| Concurrent connections | 128 | 503 |
| Chunked request bodies | not supported | 411 |
| Duplicate `Host` header | refused | 400 |
| Control character (CR, NUL, DEL…) in a header value | refused (a tab is allowed) | 400 |

Every response carries `Connection: close` (no keep-alive state to exhaust), `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer` and a `Content-Security-Policy`.

**Request policy** (`Policy::check`, applied to every request):

1. **Host allow-list.** The `Host` header must be the bound address (`127.0.0.1`, `localhost`, the real port) or a name given with `--allow-host`. Otherwise 421 Misdirected Request. This is what stops DNS rebinding: a page on `evil.example` that resolves to `127.0.0.1` sends `Host: evil.example`.
2. **Origin and `Sec-Fetch-Site`.** Requests carrying an `Origin` from another site, or `Sec-Fetch-Site: cross-site` / `same-site`, are refused with 403. `Origin: https://<allowed host>` is accepted for hosts in `--allow-host`; behind a reverse proxy that serves the page at a public URL, pass `--public-origin URL` so that the proxy's origin is accepted. Cross-origin stays refused.
3. **Content type.** `POST` requires `Content-Type: application/json` (415 otherwise), so a plain HTML form cannot post to the API.
4. **Token** (see section 3), when configured; mandatory when `--bind` is not a loopback address.

**Source offer.** The studio serves `/source` (also when a token is required) and `--source-url` sets the address (the official release binaries default to the commit they were built from), as required by AGPL-3.0 section 13 (see [LICENSING.md](../LICENSING.md)).

**Unknown or malformed paths and methods** get a plain error response; the editor page is compiled into the binary (`studio.html`) and is not read from the file system, so there is no path traversal surface for static content.

## 3. Authentication and collaboration

Option `--collab-token T` (or the token required by a non-local `--bind`) enables access control; `--collab-log LOG` persists a shared session.

* **Token format.** At least 16 characters, from `A-Za-z0-9._~+/=-`. Anything with CR, LF, `;` or other characters is refused at startup (header and cookie injection).
* **Presentation.** Header `X-Collab-Token: T`, or the cookie `osatura_token` (`HttpOnly`, `SameSite=Strict`, plus `Secure` when the request came in under an `https://` `--public-origin` name, an https `Origin`, or `X-Forwarded-Proto: https`) set by `POST /api/login`. A wrong cookie counts as a failed attempt like a wrong header.
* **Comparison.** Constant-time (both sides are hashed with SHA-256 and the digests compared), so response time does not reveal prefix matches.
* **Back-off.** Per client IP address, from the **5th** consecutive failure: 2, 4, 8, 16, then 30 seconds of 429 responses; up to 512 client addresses are tracked. The right token is always accepted, even during a back-off (so an attacker cannot lock the owner out). The back-off slows guessing; it does not stop a distributed attack. **Behind a reverse proxy every client appears with the proxy's address**, so they share one back-off counter: a flood of bad requests from one visitor delays the failure counter for all, but the correct token still works.
* **Participant secrets.** Each collaborator gets a random 128-bit secret (`X-Collab-Secret`) identifying their stream; at most 32 participants and 64 event streams.
* **Log integrity.** The session log is a text file, one line per operation: `<hash> <json>`, chained with SHA-256 (format version 2). Editing, deleting or reordering a line breaks the chain, which is reported when the log is loaded. Each request is `fdatasync`ed before the response, the file is created with mode 0600 and rewritten atomically. The chain detects accidental damage and casual tampering; it is **not signed**, so someone who can write the file can rebuild a valid chain.
* **Per-participant tokens (wave 7, optional).** `--collab-secret-file F` replaces the one shared bearer by tokens that are derived from a secret: `ap1.<name>.<expires>.<mac>` with `mac` = HMAC-SHA-256 (RFC 2104, own SHA-256, RFC 4231 test vectors in the tests) over the name and the deadline. The host mints them with `osatura collab-token NAME --secret-file F --ttl SECONDS` (default 8 hours, at most 30 days) and gives each person their own. A token is bound to one name (a joining participant gets that name whatever the page sends, so nobody can join as somebody else), expires on its own, is verified by recomputation (no table on the server, nothing to steal from it but the secret itself), and its cookie gets a `Max-Age` equal to the remaining life. `--collab-revoke NAMES` refuses the tokens of those names before they expire; changing the secret revokes everything. The secret file must be private (mode 600 on Unix, otherwise the studio refuses to start), at least 16 characters, and is never accepted as a command line argument. `--collab-token` and `--collab-secret-file` can be combined; either satisfies the rule that a non-loopback bind needs a credential. Expiry is checked **per request**: an event stream that is already open stays open until it ends, and the next connection is refused. An expired or revoked token is answered `401` with the reason and does not feed the guessing back-off.
* **Rate limit (wave 7).** `--collab-rate N` allows N requests per second per client address, with bursts of 2N (token bucket); more is answered `429` with `Retry-After`. The default is 100 per second when a credential is required and no limit otherwise; 0 switches it off. It is applied after authentication and per **address** (a connection's source), not per TCP connection, because a per-connection limit is circumvented by opening more connections; behind a reverse proxy all clients share the proxy's address, so also limit at the proxy (section 12).
* **Bind-address warnings (wave 7).** A non-loopback `--bind` prints a warning at start (plain HTTP; for `0.0.0.0` / `::` that the studio listens on every interface; for a shared token that it has no names and never expires).
* **Token spelling.** The deadline in a per-participant token has exactly one written form (decimal digits, no sign, no leading zeros, no `+`): any other spelling of the same number is refused as malformed, and the signature is judged before the deadline, so a forged token learns nothing from the answer.
* **Hosts (opt-in).** `--collab-host NAMES` (needs `--collab-secret-file`) names the participants who may **save** (`/api/save`), **compact** the session log (`/api/collab/compact`) and **revoke** (`/api/collab/revoke`, a `POST` that adds names to the revoked list, at most 1000). Everyone else with a valid token can still edit and watch, and is answered `403` on those three. Without `--collab-host` nothing changes: every holder of a valid credential may do all of it, as before. The shared `--collab-token` is not tied to a name and counts as a host.
* **Event stream.** The event stream needs a credential (a token) but not a participant secret; the secret identifies a participant's own stream, it is not a second password.
* **Connections per address.** One client address may hold at most 16 connections at once (in addition to the global cap), so one machine cannot use up all the slots. Loopback addresses are not counted by default, because a reverse proxy on the same host reaches the studio from `127.0.0.1` for every visitor; limit per visitor at the proxy (section 12).
* **No TLS, no accounts.** The server speaks plain HTTP. A token travelling over a network is readable by anyone on the path. Put the studio behind a TLS-terminating reverse proxy or an SSH / VPN tunnel (section 11). There are no user accounts or per-project permissions; the only role is the optional host list above, and any other valid token grants edit access.

## 4. Asset resolution and input limits

* **Assets** (image files, normal maps, sounds named by a project) resolve through `assets::resolve`: relative to the process-wide asset directory (`assets::default_dir`, normally the folder of the input project). **Absolute paths and any `..` component are refused** unless the global flag `--allow-external-assets` is given. Image files are limited to 64 MB, audio files to 256 MB.
* **Project files.** `load_project` reads regular files only (no devices, no pipes), up to 256 MB (`--max-input-bytes N` to change). The facade's `MAX_JSON_BYTES` is 256 MiB.
* **Derived projects.** Commands that write a derived project validate against the *input* project's directory; when the output is elsewhere, relative asset paths of existing files are rewritten or a warning says the output will not find them (`--allow-external-assets` makes them absolute).
* **Rendering.** The facade refuses pictures larger than 16384 px a side (`MAX_RENDER_SIDE`) and scales outside 0.05 to 4; the C and wasm layers refuse output above 256 MiB before allocating. Other facade limits: fps in (0, 1000], frames 1 to 200000, stage up to 32768, 4096 bones, magnitudes within 1e7, `lod_levels` 1 to 12, `fuzz` samples up to 100000, `trace` colors 2 to 64 ([API.md](API.md)).
* **No panics on input.** Hostile-input tests live in `osatura-cli/tests/robust.rs` and the wave tests; fuzz-style tests cover the parsers. A panic reachable from input is treated as a vulnerability.
* **Studio bodies** are limited to 32 MB; edits go through the same validation as files.

## 5. Expression and script sandbox

* **Expressions** (value bindings, constraints, drivers) are parsed by an in-house parser: **source up to 4096 bytes, nesting depth 64**, arithmetic, comparison, the fixed variables `frame`, `time`, `fps`, `frames`, `pi`, `t`, `get("layer","property")` for drivers, and a fixed table of built-in functions (`sin`, `abs`, `min`, ...); no loops, no I/O and no user-defined functions, so evaluation always terminates. Non-finite results are rejected by the finite-value policy.
* **Scripts** (command `script`, feature `scripting`, on by default) run in Rhai with these limits: **500,000 operations, 32 call levels, expression depth (64 top level, 32 in functions), strings 10,000 characters, arrays 10,000 items, maps 1,000 entries, 5,000 API requests, script source 64 KB**, and `no_module` (no `import`, so no file access through modules). The host API exposed to scripts is a fixed set of project-editing functions. Build with `--no-default-features` to remove the scripting engine entirely (also removes the `smartstring` dependency).
* A script is still code you chose to run on a project: it cannot read or write files itself, but it can make the project it edits very large within the limits above, and an infinite loop ends at the operation limit.

## 6. Files, symlinks and temporary data

* **Atomic writes.** Outputs are written with `fsutil::write_atomic`: a temporary file in the same directory (name `.<name>.tmp-<pid>-<random>`), `fsync`, `rename`. A crash leaves the old file or the new one. **Symbolic links given as output are refused**, naming `--follow-symlinks` (this now covers every user-addressed output: `ci run` reports and goldens, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, pose files, snapshots, `vcs`, the collaboration log, and the ffmpeg-written video files, which are pre-checked), so a planted link cannot redirect a write. If the directory is not writable but the file is, the fallback is an in-place overwrite with `fsync` which is **not atomic** (documented).
* **Private data.** Collab logs and other files that only the owner should read are created with mode 0600 (`write_atomic_private`). Temporary directories are created exclusively with mode 0700 (`private_temp_dir`), so a predictable name in a shared `/tmp` cannot be pre-created by another user. Random values (tokens for new participants, temp names) come from `/dev/urandom` (`fsutil::random_bytes`); the engine's seeded deterministic generator is never used for secrets.
* **UI settings.** The studio's `ui.json` ignores unknown keys with a warning and is copied to `ui.json.bak` (then `.bak.1`, ...) before the first save.
* **Windows / non-Unix.** The permission modes above apply on Unix. On Windows and macOS the test suite passes in CI (POSIX-only checks are skipped on Windows), but the symlink, permission and `/dev/urandom` checks run only on Linux and macOS and nobody has reviewed the Windows behaviour by hand.
* **Unsigned binaries and the window app.** The release executables are not code-signed; verify downloads against `SHA256SUMS.txt`. `osatura-studio` starts the same local server (`127.0.0.1` only, random free port) and shows it in the system web view (WebView2 / WKWebView); its web view is as exposed as a browser tab on that address and has had no separate security review.

## 7. External programs (ffmpeg)

ffmpeg and ffprobe are run as separate processes **only** for video export, mp3 / ogg decoding, soundtrack mixing and a few tools; they are never bundled ([LICENSING.md](../LICENSING.md)). Every invocation:

* passes arguments as an array (no shell, no string concatenation into a command line);
* uses `-protocol_whitelist file` (one internal pipe use adds `pipe,fd`), so a media file cannot make ffmpeg open `http://`, `concat:` or other network / special protocols;
* passes file names through `fsutil::file_arg`, which prefixes `./` to names that begin with `-` or contain `:` or `|` in the first path component, so a file name cannot be read as an option or URL;
* for `.wav`, `.mp3` and `.ogg` sound files forces the demuxer with `-f` (`soundtrack::forced_demuxer`), so a file that is not what its name says is refused rather than opened as some other container;
* uses `-loglevel error`.

ffmpeg itself parses complex media formats and has had vulnerabilities of its own: keep it up to date and do not feed it media from sources you do not trust on a machine that holds secrets. Time-budgeted commands (`export-fit --max-seconds`) cannot interrupt a single running ffmpeg call.

## 8. Libraries (C ABI, WASM, facade)

* The **facade** validates every argument, returns `ApiError`, and converts panics to `ApiError::Internal` where the target unwinds. `Animation` and `Player` are `Send + Sync` (checked at compile time).
* The **C ABI** ([C_ABI.md](C_ABI.md)) checks null pointers, lengths and UTF-8 and returns codes; the caller must still pass valid pointers and honour the buffer rules. The last-error string is per thread. The ABI was exercised under AddressSanitizer and UndefinedBehaviorSanitizer with the documented examples (clean). It is not hardened against a caller that violates the documented contracts (double close, use after close, wrong lengths).
* The **WASM module** ([WASM.md](WASM.md)) builds in CI and passes the wrapper's test under Node, and was run once in headless Chromium (bad input gives errors, no leak over 60 open/close cycles, oversize stages refused), but **has not been reviewed as a browser component and was not run in Firefox or Safari**, so its security properties are known only from reading the code and that test. It has no file system or network access by construction; a panic there traps the instance.
* Neither layer sandboxes the *project*: rendering a hostile project is bounded by the limits above, but a project can still ask for a lot of work within them (many layers, large pictures, long timelines).

## 9. Known gaps

Honest list; see [LIMITATIONS.md](LIMITATIONS.md) for the wider list of functional limits.

1. **No TLS and no accounts** in the studio. Since wave 7 the token can be per participant and expiring (section 3), which is still not an account system: there are no roles, no password reset, and no TLS of its own. No native TLS was added because no licence-compatible TLS crate is in `Cargo.lock` (adding one would put a large new dependency, and cryptographic code, into an AGPL/MPL project without an audit); section 12 shows the supported way, a TLS-terminating reverse proxy.
2. **Back-off is per address** and shared behind a proxy; it slows but does not prevent online guessing. Use a long random token (the minimum of 16 characters is a floor, not a recommendation).
3. **The collaboration log is chained, not signed.**
4. **Not an audited codebase.** No third-party penetration test or formal fuzzing campaign; the checks are the project's own tests and review waves.
5. **Dependencies.** CI runs `cargo deny` for advisories (vulnerabilities, yanked crates, unmaintained direct dependencies), licences, banned crates (no TLS or HTTP client stack) and sources. Transitive maintenance notices are reviewed by hand (see `deny.toml`). The check only knows advisories published so far; run it again before you deploy.
6. **Asset directory is process-wide.** `merge` over several input directories uses the last directory for assets; only `write_json` rewrites asset paths (other writers do not).
7. **Non-atomic fallback** when the output directory is not writable.
8. **Path checks use canonical paths.** `assets::resolve` canonicalises existing files and refuses a file whose real location is outside the project directory, so a symbolic link inside a project folder that points outside it is refused (tested in `assets.rs`); links that stay inside the folder work. A race between the check and the later read (time-of-check to time-of-use) by someone who can modify the project folder concurrently is not excluded.
9. **ffmpeg** is outside our control (section 7).
10. **WASM** is untested in a browser engine (it runs only under Node in CI); the browser threat model of a page embedding it (CSP, memory limits) has not been examined.
11. **Denial of service by size.** Limits bound single requests and files, not total work: a project that is valid but very heavy can occupy a CPU for a long time (the connection cap above limits concurrency, not the cost of one request).

## 10. Reporting a vulnerability

* **Do not open a public issue** for a vulnerability. Send a private report to the maintainer: **`alexio@trom.tf`** (the address in the program's `--version` notice and the licence documents), or use GitHub's private vulnerability reporting on the repository's *Security* tab when it is enabled.
* Include: version (`osatura --version`) and platform, the project or request that triggers the problem (a minimal file is best), what you expected and what happened, and whether you want to be credited.
* What to expect *(intent, not a contract: this is a one-person project)*: an acknowledgement within a reasonable time, a fix developed in private, a regression test added to the suite, a note in [CHANGELOG.md](../CHANGELOG.md), and credit if you wish. Please allow time for a fix before publishing details.
* Supported versions: only the newest pre-release (currently `v0.1.0-alpha.4`) and the current tree on `main` are supported.
* Reports about the studio's network exposure when it is deliberately bound to a public address without a reverse proxy are documentation issues first: see section 11.

## 11. Deployer hardening checklist

For anyone running `osatura studio` for more than one person or off localhost.

1. [ ] Prefer **not** exposing it. Single user: leave the default bind (`127.0.0.1`) and use an SSH tunnel (`ssh -L 8080:127.0.0.1:8080 host`).
2. [ ] If exposed, **terminate TLS** in a reverse proxy (nginx, Caddy, ...) in front of it; never expose the plain HTTP port to the Internet.
3. [ ] Use a **long random token** (for example 32+ characters from `openssl rand -base64 24`), pass it through a protected file or environment of the service rather than a shell history, and rotate it when people leave. Remember it is full edit access.
4. [ ] Start with `--bind 127.0.0.1` (or the private interface) plus the proxy; set `--allow-host your.domain` and `--public-origin https://your.domain` so Host and Origin checks pass for the proxy and for nothing else.
5. [ ] Prefer **per-participant tokens** (`--collab-secret-file`, section 3) with a short `--ttl` over one shared token; keep the secret file at mode 600 outside the served folder, and name the people who may save, compact and revoke with `--collab-host`.
6. [ ] Have the proxy **rate-limit** and, if possible, restrict by IP or add a second authentication layer (HTTP basic over TLS, VPN, SSO proxy), because the built-in back-off sees only the proxy's address.
7. [ ] Run as an **unprivileged user** in a dedicated directory containing only the projects to be served; no secrets, no sensitive symlinks. Use a read-only file system elsewhere, a container or systemd sandboxing (`ProtectSystem`, `PrivateTmp`, `NoNewPrivileges`).
8. [ ] Keep `--allow-external-assets` and `--follow-symlinks` **off**.
9. [ ] Set `--source-url` to where the (possibly modified) source can be fetched (AGPL section 13).
10. [ ] Persist with `--collab-log` on a private directory, back it up, and keep the 0600 mode. Treat the log as sensitive: it contains the whole edit history.
11. [ ] Bound resources at the OS level (memory, CPU time, open files, disk quota); the built-in caps limit single requests, not total load.
12. [ ] Keep ffmpeg and the Rust toolchain used for the build current; run `cargo deny check` (or `cargo audit`) on `Cargo.lock` for your release.
13. [ ] Do not feed the server projects from untrusted sources unless it runs in the sandbox above. For scripting-free deployments build with `--no-default-features`.
14. [ ] Watch the logs for 421, 403, 429 and 503 responses; spikes mean probing or misconfiguration.

## 12. Reverse proxy with TLS: nginx and Caddy

The studio speaks plain HTTP. For anything beyond your own machine run it on loopback and let a proxy terminate TLS. These recipes were written from the proxies' public documentation and the studio's behaviour described above; **they were not run in the authoring environment** (no nginx or Caddy, no certificates, no network): test them before relying on them.

Start the studio on loopback and name its public host, so Host and Origin checks accept the proxy and nothing else:

```
osatura studio demo.json --port 8080 --bind 127.0.0.1 \
  --allow-host studio.example.org --public-origin https://studio.example.org \
  --collab-secret-file /etc/osatura/collab.secret --collab-log /var/lib/osatura/session.log
```

With an `https://` `--public-origin` the login cookie gets the `Secure` attribute. The event stream (`/api/collab/events`) is Server-Sent Events: the proxy must not buffer it and must allow long reads.

**nginx** (certificates from your CA or `certbot`):

```
limit_req_zone $binary_remote_addr zone=osatura:10m rate=20r/s;

server {
    listen 80;
    server_name studio.example.org;
    return 301 https://$host$request_uri;
}

server {
    listen 443 ssl;
    server_name studio.example.org;
    ssl_certificate     /etc/letsencrypt/live/studio.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/studio.example.org/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    add_header Strict-Transport-Security "max-age=31536000" always;
    client_max_body_size 64m;

    location / {
        limit_req zone=osatura burst=60 nodelay;
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
    }
    location /api/collab/events {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_cache off;
        proxy_read_timeout 1h;
    }
}
```

**Caddy** (obtains and renews the certificate by itself; `Caddyfile`):

```
studio.example.org {
    encode zstd gzip
    header Strict-Transport-Security "max-age=31536000"
    request_body {
        max_size 64MB
    }
    reverse_proxy 127.0.0.1:8080 {
        flush_interval -1
        transport http {
            read_timeout 1h
        }
    }
}
```

`flush_interval -1` sends the event stream through unbuffered. Caddy sets `X-Forwarded-Proto` and passes the original `Host`. For a request rate limit use the `caddy-ratelimit` plugin or put the nginx `limit_req` in front.

Checks after starting: `curl -I https://studio.example.org/` answers `401` (login page) and never serves the studio without a token; `curl -I http://127.0.0.1:8080/ -H 'Host: evil.example'` answers `421`; the cookie set by `/api/login` carries `Secure; HttpOnly; SameSite=Strict`; the studio's own log of the session stays private (section 6). If the proxy runs on another machine, bind the studio to the private interface that the proxy can reach, firewall the port, and remember the studio itself still speaks plain HTTP on that link.
