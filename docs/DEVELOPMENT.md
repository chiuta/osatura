<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Development guide

*For people who build, change, test and release OSATURA. Documentation CC-BY-4.0. Everything here is taken from the source tree, `ROADMAP.md` section 9, [CI.md](CI.md), [CLEANROOM.md](../CLEANROOM.md), [CONTRIBUTING.md](../CONTRIBUTING.md) and [LICENSING.md](../LICENSING.md); commands marked "run" were run while writing this page (Linux x86-64, rustc 1.97.0, offline registry).*

Contents: [Environment](#1-environment) - [Build and test commands](#2-build-and-test-commands) - [The test suites](#3-the-test-suites) - [Coding conventions](#4-coding-conventions) - [Checklist for a new module or command](#5-checklist-for-a-new-module-or-command) - [Clean-room rules and DCO](#6-clean-room-rules-and-dco) - [Release checklist](#7-release-checklist) - [Continuous integration](#8-continuous-integration) - [Debugging](#9-debugging) - [Profiling](#10-profiling)

## 1. Environment

| Need | Why | Notes |
|---|---|---|
| Rust (stable, edition 2021 workspace) | everything | developed with rustc 1.97.0; `Cargo.lock` is committed, builds are `--locked`-friendly |
| A C compiler (`cc`) | the C ABI example/test (optional) | see [C_ABI.md](C_ABI.md) |
| `ffmpeg` / `ffprobe` on `PATH` | video export, audio decoding of mp3/ogg, `dof-export` to .mp4/.webm, `soundtrack` | **never bundled or required for building**; commands that need it say so and fail without a success message |
| Node.js >= 18 and `playwright-core` + Chromium | the five studio smoke tests; the WASM wrapper test | defaults: `/opt/npm-tools/node_modules/playwright-core`, `/opt/pw-browsers/chromium-1194/chrome-linux/chrome`; override with `PLAYWRIGHT_CORE`, `CHROME` |
| `cargo-deny` | licence/ban/source policy | `scripts/licence-audit.sh` runs it when installed (`--no-deny` skips) |
| `python3` | `scripts/licence-audit.sh` generates docs with an inline script | runs with `python3 -I` |
| A Vulkan or GL adapter | only for real-GPU numbers from `osatura-gpu` | tests fall back to a software adapter; see section 2 |
| `rustup target wasm32-unknown-unknown`, `wasm-opt` | only for [WASM.md](WASM.md) | never installed in the development environment; CI installs it |

Workspace layout (see [ARCHITECTURE.md](ARCHITECTURE.md) for the design):

| Crate | Licence | Role |
|---|---|---|
| `osatura-core` | MPL-2.0 | engine, formats, tools, the stable `api` facade ([API.md](API.md)); features: `scripting` (default, Rhai) |
| `osatura-cli` | AGPL-3.0-or-later | the `osatura` binary, studio server, collab server; features: `scripting` (default), `gpu` |
| `osatura-capi` | MPL-2.0 | C ABI ([C_ABI.md](C_ABI.md)) |
| `osatura-wasm` | MPL-2.0 | WebAssembly ABI + JS wrapper ([WASM.md](WASM.md)) |
| `osatura-gpu` | MPL-2.0 | optional wgpu renderer; **not** a default member |
| `osatura-gui` | AGPL-3.0-or-later | `osatura-studio`, the studio in a native window (`tao` + `wry`, Windows and macOS only; a stub elsewhere); **not** a default member, `publish = false` |

Repository-level files: `ROADMAP.md` (section 9 is the dated feature and review log), `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `REUSE.toml`, `deny.toml`, `LICENSES/`, `scripts/`.

## 2. Build and test commands

```sh
cargo build                                  # default members: core, cli, capi, wasm (gpu is opt-in)
cargo test --workspace                       # run: 1343 tests + 19 doc-tests without gpu and gui (section 3)
cargo test -p osatura-core                  # one crate (the engine and the facade tests)
cargo test -p osatura-core api::            # a module or a name filter
cargo test -p osatura-cli --test e2e        # one integration test file
cargo test --doc -p osatura-core            # doc-tests only (the facade examples)
cargo test -p osatura-cli --test spdx_audit # SPDX header audit (0.01 s)
cargo clippy --workspace --all-targets -- -D warnings   # zero warnings (gated in CI since wave 8)
scripts/licence-audit.sh                     # regenerate THIRD_PARTY*, SPDX check, cargo deny
scripts/e2e.sh                               # the end-to-end pipeline in release mode (--debug for debug)
```

Features and opt-in parts:

```sh
cargo build -p osatura-cli --no-default-features         # no Rhai (no `script` command, no smartstring)
cargo build -p osatura-core --no-default-features        # what capi, wasm and gpu use
cargo build -p osatura-cli --features gpu                # adds `render-gpu` (wgpu)
cargo test  -p osatura-gpu                               # GPU tests (31: 18 unit + 13 parity); NOT in the workspace default
OSATURA_GPU_BACKEND=vulkan osatura render-gpu ...     # restrict the wgpu backend: vulkan or gl
```

Rules of thumb: use debug builds for the test suites (the repository was developed with disk space in mind: do not copy `target/`); use `--release` only for timing questions. The facade snapshot is blessed with `OSATURA_BLESS=1 cargo test -p osatura-core api::` (section 5). `OSATURA_UI_CONFIG` selects the studio's UI settings file in tests, `OSATURA_AUTHOR` the author name in version-control commits, `OSATURA_E2E_TIMINGS=<file>` makes the e2e test write its per-stage timings.

## 3. The test suites

Counts from `ROADMAP.md` section 9 and from a full `cargo test --workspace --offline` run on 22 Oct 2026 for this page (debug profile, a machine shared with other jobs, so the timings are indicative and **will be slower than on an idle machine**):

| Suite | Tests | Time (this run) | What it covers |
|---|---:|---:|---|
| `osatura-core` unit (`src/lib.rs`) | 1167 | 349 s | every module; includes `api::tests::api_surface_snapshot` |
| `osatura-core/tests/prelude_surface.rs` | 6 | 0.7 s | the facade from outside the crate (`#[non_exhaustive]`, error mapping, a representative subset of calls) |
| `osatura-cli` unit (`src/usage_tests.rs`, `src/args.rs`, studio, collab, http) | 80 | 42 s | option parsing, USAGE vs parser, HTTP server, policy, collab log |
| `tests/e2e.rs` | 2 | 214 s | the whole pipeline through the real binary (about 19 s in an idle debug build and 40 s in release according to the ROADMAP; here the machine was loaded) |
| `tests/fixes.rs`, `tests/wave4.rs`, `tests/robust.rs` | 7, 15, 10 | 19 s, 1.2 s, 1.2 s | regressions of the review waves and hostile-input tests |
| `tests/doc_mismatch.rs` | 20 | 1.8 s | doc/behaviour regressions: atomic outputs, strict options, the `--json`/`--quiet`/`--allow-empty` acceptance matrix, `golden` scale rules |
| `tests/source_offer.rs` | 3 | 0.2 s | AGPL section 13 `/source` offer |
| `tests/face_cli.rs` | 1 | 0.8 s | face tracking CLI |
| `osatura-core/tests/wasm_safety.rs` | 2 | 0.1 s | no direct `Instant` / `SystemTime` / `process::id` / `thread::spawn` outside `clock.rs` (source scan) |
| `osatura-cli/tests/wave7.rs` | 7 | 7 s | wave 7: per-sub-command options, `follow` / `make-loop`, `color-space`, `collab-token`, studio secret file, frame range of `dof` / `render-hdr` / `fx` |
| `tests/spdx_audit.rs` | 2 | 0.01 s | every `.rs/.js/.mjs/.sh` has an SPDX line equal to its crate licence; REUSE.toml covers the rest |
| `osatura-capi/tests/abi.rs` | 7 | 1.1 s | the C ABI through the Rust side (C smoke test: `osatura-capi/tests/smoke.c`, compiled by hand, see [C_ABI.md](C_ABI.md)) |
| `osatura-wasm` unit | 8 | 1.6 s | the ABI functions on the host |
| `osatura-gpu` unit + `tests/parity.rs` | 18 + 13 | 0.2 s + 6.5 s | GPU vs CPU parity (software adapter when no GPU) |
| Doc-tests (`osatura-core`) | 19 | 171 s (each compiles a program) | the facade examples (`api/`, `lib.rs`); `osatura-capi`, `osatura-wasm` have none |

Total: 1343 tests in the unit and integration binaries and 19 doc-tests, counted on 24 Oct 2026 with `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` (the 1368 of the next sentence also included the GPU crate's tests; recounted from a full `cargo test --workspace` run on 22 Oct 2026 after wave 7, the per-suite timings above are from an earlier run), plus 19 doc-tests; the whole `cargo test --workspace --offline` took 836 s on the shared machine.

Suites outside `cargo test`:

* **Studio smoke tests (Chromium).** Five Node scripts drive the real studio in headless Chromium with `playwright-core`: `studio_ui_smoke.js` (128 checks), `studio_collab_smoke.js` (60, two pages on one machine), `studio_overlays_smoke.js` (89), `studio_suggest_smoke.js` (35) and `studio_demo_smoke.js` (25: the guided demo loads all 13 steps, never writes, restores the project byte for byte and recovers a closed tab); all five passed on 24 Oct 2026 (counts as printed by the scripts). They are **not** run by `cargo test` and not part of CI by default. Run:

  ```sh
  cargo build -p osatura-cli
  node osatura-cli/tests/studio_ui_smoke.js        # same for collab / overlays / suggest / demo
  ```

  Environment: `OSATURA_BIN` (default `target/debug/osatura`), `PLAYWRIGHT_CORE`, `CHROME`, and for the overlay script `SHOTS` (screenshot directory). The scripts clean up their temporary files.
* **WASM wrapper test.** `node osatura-wasm/js/test.mjs` (mock only unless you pass a project path and have built the module; see [WASM.md](WASM.md)).
* **Licence audit.** `scripts/licence-audit.sh`: rebuilds `docs/THIRD_PARTY.md` and `docs/THIRD_PARTY_NOTICES.md` from `cargo metadata --locked --all-features` (normal and build dependencies, dev dependencies excluded), checks every source file's SPDX line against its crate's `license`, then runs `cargo deny check licenses bans sources` unless `--no-deny`. `deny.toml` allows permissive licences and MPL-2.0 only for named crates.
* **clippy.** `cargo clippy --workspace --all-targets -- -D warnings` exits 0 with no warnings (wave 8) and is gated in CI. Where a lint is wrong for the case, the code carries a narrow `#[allow(clippy::...)]` with a comment (index loops over parallel arrays, `too_many_arguments` on stable signatures, `manual_clamp` where NaN handling differs, inherent `next` / `mul` names that are public API); do not add crate-level allows.
* **CI wiring.** The repository ships `osatura ci` for *animation* checks ([CI.md](CI.md)); it is a different thing from the CI of the code. The project has no code CI configuration in the tree; section 8 gives a minimal recipe.

Adding tests: put engine tests next to the code (`#[cfg(test)]`), CLI behaviour tests in `osatura-cli/tests/`, and every fixed bug gets a regression test (the review waves in [CHANGELOG.md](../CHANGELOG.md) list what was fixed and where). Tests must not leak temporary files (use drop guards), and must not depend on the clock or the network. Time budgets are tested with a deterministic test clock.

## 4. Coding conventions

* **No panics on input.** Anything that reads a project, a file, an argument or a network byte returns an error. Indexing, slicing, `unwrap` and arithmetic overflow on untrusted values are bugs. The facade (`api/`) additionally catches panics and returns `ApiError::Internal` (only where the target unwinds), but that is the last line of defence, not a design.
* **Finite-value policy.** Numbers from files and arguments are checked for NaN and infinity, and for sane magnitudes (the facade's ranges are in [API.md](API.md): magnitudes up to 1e7, fps in (0, 1000], frames 1..=200000, stage up to 32768, 4096 bones, render side up to 16384 px). Reject, do not clamp, unless the function is documented to clamp (frames are clamped to the timeline).
* **Check sizes before allocating.** Pixel buffers, mesh sizes and sample counts are computed with checked arithmetic and refused before `vec![..]`.
* **Atomic writes.** Write results with `fsutil::write_atomic` (temporary file in the same directory, `fsync`, rename; refuses symbolic links unless `--follow-symlinks`; falls back to an in-place `fsync`ed overwrite, documented as not atomic, when the directory is not writable). Never `std::fs::write` a user's file directly. Temporary directories come from `fsutil` (mode 0700, files 0600). Random bytes come from `fsutil::random_bytes` (`/dev/urandom`).
* **Assets go through `assets::resolve`.** Relative image, normal-map and sound paths resolve against the process-wide asset directory (`assets::default_dir`, set from the input project's folder); absolute paths and `..` are refused unless `--allow-external-assets`; image files are capped at 64 MB and audio at 256 MB. Commands that write a derived project validate against the *input* project's directory and rewrite or warn about relative asset paths (`write_json`).
* **Deterministic randomness.** Any random behaviour (fuzzing, noise, jitter, simulation) uses the engine's own seeded generator with a `--seed` option and is bit-reproducible. No `rand` crate, no system time as a seed.
* **Println and closed pipes.** The CLI shadows `println!`/`print!` with macros that exit quietly on a closed pipe (`osatura ... | head` must not panic). Keep using them in modules declared after the macros in `main.rs`.
* **Errors tell what to do.** Messages say what was wrong, which value, and the allowed range or the fix (the facade tests assert on this style).
<!-- REUSE-IgnoreStart -->
* **SPDX headers.** Every `.rs`, `.js`, `.mjs` and `.sh` file starts with `SPDX-License-Identifier:` equal to its crate's licence (MPL-2.0 for core/capi/wasm/gpu/scripts, AGPL-3.0-or-later for the CLI) and a copyright line; Markdown docs carry the CC-BY-4.0 header comment, as this file does. Other files (JSON, images, the license texts) are covered by `REUSE.toml`; when you add a new path pattern, add it there. `cargo test -p osatura-cli --test spdx_audit` fails otherwise.
<!-- REUSE-IgnoreEnd -->
* **Dependencies.** Few, pure Rust, permissive licence. A new crate needs a `deny.toml` check (`scripts/licence-audit.sh`), a line in `docs/THIRD_PARTY.md` (regenerated) and a justification in the ROADMAP entry. `osatura-core` is built without default features by capi, wasm, gpu and the CLI, so anything heavy goes behind a feature.
* **Facade first.** New user-visible engine capability that other languages should reach goes into `osatura-core/src/api/` with a snapshot line, a doc-test and a unit test; the C and wasm layers wrap the facade only.

## 5. Checklist for a new module or command

1. **Module** in `osatura-core/src/<name>.rs`, registered in `lib.rs`; SPDX header; unit tests (valid, boundary, hostile inputs, determinism); no panics.
2. **CLI command**: a `USAGE` entry (exact spelling of every option) in `osatura-cli/src/usage.rs`, the dispatch arm in one of `osatura-cli/src/cmds_1.rs` to `cmds_6.rs` (or in `main.rs` for `ui`), and the option tables in `osatura-cli/src/args.rs`: every value-taking option must be listed in `VALUE_OPTS`; flags that take no value in a command's `CMD_OPTS` entry where the global table would be wrong (for example `ci --prune` is a flag, `export-oca --layers` and `autotrace --png` take values, `keyedit --free` is a flag). The USAGE-versus-parser unit tests fail if the text and the parser disagree. Support the global flags (`--json`, `--quiet`, `--allow-empty`, `--allow-external-assets`, `--follow-symlinks`, `--max-input-bytes`), `<command> --help` and `--key=value`.
3. **Writing output** only with `fsutil::write_atomic` or `write_json`; commands that produce nothing exit non-zero unless `--allow-empty`.
4. **Documentation**: the command in [CLI_REFERENCE.md](CLI_REFERENCE.md) if you add one, a guide mention where relevant, and the entry in `ROADMAP.md` section 9 (what was added, what is *not* done: that list feeds [LIMITATIONS.md](LIMITATIONS.md)).
5. **CLEANROOM.md row**: for every new algorithm record the public source it was implemented from (textbook, standard, paper) and the independent derivation; no row, no merge.
6. **Facade surface re-bless** if `api/` changed: run `OSATURA_BLESS=1 cargo test -p osatura-core api::`, review the diff of `osatura-core/src/api_surface.txt` (a removed or changed line is a breaking change and needs a major version of the facade; an added line is a minor), then run the tests again without the variable. Update [API.md](API.md), and the C/WASM wrappers and their headers if they should expose it.
7. **End-to-end**: add the command to the e2e pipeline if it belongs to the production path; add a smoke check to the studio scripts if it has UI.
8. **Run**: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `scripts/licence-audit.sh` (when dependencies or headers changed).

## 6. Clean-room rules and DCO

The project is an independent implementation. In short (the binding text is [CLEANROOM.md](../CLEANROOM.md) and [CONTRIBUTING.md](../CONTRIBUTING.md)):

* Implement from public standards, textbooks and papers, and cite them. Do not read, decompile, copy or reverse-engineer code, files or interfaces of proprietary animation tools, and do not read or paste source code of competing tools while writing a feature. Do not paste code from projects with an incompatible licence.
* Record each new method in the `CLEANROOM.md` table with its public source.
* Patent notes live in [PATENT_NOTES.md](PATENT_NOTES.md); a feature that touches a patent-sensitive area is discussed there first. This is not legal advice.
* Contributions are accepted under the licence of the part they change (inbound equals outbound): MPL-2.0 for the libraries and scripts, AGPL-3.0-or-later for `osatura-cli`, CC-BY-4.0 for documentation and synthetic assets.
* **Sign-off.** Every commit needs `Signed-off-by: Name <email>` (`git commit -s`), certifying the Developer Certificate of Origin 1.1 (text in CONTRIBUTING.md). The sign-off certifies origin only: it grants **no** relicensing right, and no commercial or dual licence for contributed code is possible while no CLA exists.
* The name and logo are covered by [TRADEMARK.md](../TRADEMARK.md).

## 7. Release checklist

The first builds are prepared as the draft pre-release `v0.1.0-alpha.3` (all crates are still 0.1.0; the version labels "v0.5/v0.6/v0.7" in README, FEATURES and ROADMAP are feature levels, not crate versions; see [CHANGELOG.md](../CHANGELOG.md)). To cut one:

1. **Version.** Decide the version of each crate. The facade has its own version (`API_VERSION`, 1.4.0 today: major on removal or change of a surface line, minor on addition) and `OSATURA_ABI_VERSION` / `arm_abi_version()` (both 1) change only on a breaking change of the C or wasm ABI. Align `Cargo.toml` versions, `osatura --version` output, README status box and CHANGELOG.
2. **Green gates.** `cargo test --workspace`, `cargo test --doc -p osatura-core`, `cargo test -p osatura-cli --test spdx_audit`, `cargo clippy --workspace --all-targets -- -D warnings`, the five Chromium smoke tests, `scripts/e2e.sh` (release), `cargo test -p osatura-gpu` if the GPU code changed.
3. **Third-party notices.** Run `scripts/licence-audit.sh`: it regenerates `docs/THIRD_PARTY.md` (inventory) and `docs/THIRD_PARTY_NOTICES.md` (licence texts of everything shipped), checks SPDX lines and runs `cargo deny`. Commit the regenerated files. A binary distribution must ship the notices bundle next to the binary.
4. **Licence files.** `COPYRIGHT`, `LICENSES/` (texts of every licence used; the root `LICENSE-*` files are verbatim copies of them) and `REUSE.toml` are in sync with [LICENSING.md](../LICENSING.md); the CC-BY-4.0 attribution for documentation is intact.
5. **No ffmpeg.** Do not bundle, link or download ffmpeg in release artifacts. It is detected on `PATH` at run time and the documentation says the user installs it.
6. **AGPL source offer.** The binary of `osatura-cli` serves its own source offer at `/source` (AGPL section 13) and prints a notice with `--version`; `release.yml` sets `OSATURA_SOURCE_URL` at compile time to `<server>/<repository>/tree/<commit>` so that release binaries name the exact source they were built from (the workflow's smoke test checks `--version` for it; a plain `cargo build` has no built-in address); operators of modified studio servers set `--source-url` to where their modified source is. Tests: `tests/source_offer.rs`. For a binary release, provide the corresponding source (tag, tarball) at the URL users can reach.
7. **Docs.** Update the status box in README, [LIMITATIONS.md](LIMITATIONS.md) and the CHANGELOG (move *Unreleased* under the new version with a date).
8. **Build the draft on GitHub.** `.github/workflows/release.yml` builds Windows, macOS and Linux archives and the web module on every push to `main`; a commit on `main` whose message contains `[release vX.Y.Z-label]` (or a pushed tag `v*`) also creates a *draft pre-release* with the archives and `SHA256SUMS.txt` (an older draft with the same tag is replaced). Review the draft, then publish it by hand.
9. **Tag and publish** with a signed tag; attach checksums. Do not publish crates under names that are not yours; the TRADEMARK.md rules apply to forks.

## 8. Continuous integration

Two different things share the word (a third workflow, `.github/workflows/release.yml`, builds the release archives and the draft release, see section 7):

* **Animation CI** (`osatura ci`): checks of *projects* (lint, render goldens, performance, determinism). Fully described in [CI.md](CI.md): manifest `osatura-ci.json`, thresholds, goldens (`--update` versus `bless`), reports, GitHub Actions and plain-shell examples (marked "example, not run" there).
* **Code CI** for this repository: `.github/workflows/ci.yml` has four kinds of job: `test` (matrix ubuntu / windows / macos: `cargo test --workspace --exclude osatura-gpu --locked` and clippy with `-D warnings`), `wasm32` (builds `osatura-wasm` for `wasm32-unknown-unknown`, copies the module next to `osatura.js` and runs `node osatura-wasm/js/test.mjs` with a demo project, including its real-module part), `gpu parity` (informational; skips when no adapter exists, runs on the Windows software DX12 adapter) and `licences` (`cargo-deny check` and `reuse lint`). The toolchain is pinned (1.97.0) so that new clippy lints do not break the build. The `test` job also builds `osatura-gui` and runs `osatura-studio --selftest` (a real self-test on Windows and macOS). **First fully green run: 2026-10-24** on GitHub-hosted runners. On failure the steps print the failing tests as workflow annotations (readable without log access). Windows skips the POSIX-only tests (symlinks, permissions, `/dev/zero`, POSIX path separators).

  ```sh
  cargo test --workspace --locked
  cargo test --doc -p osatura-core --locked
  cargo clippy --workspace --all-targets --locked -- -D warnings
  scripts/licence-audit.sh            # needs python3 and (optional) cargo-deny
  ```

  Cache `~/.cargo/registry` and `target/` between runs, give the e2e test several minutes in debug builds, and run the Chromium smoke tests only on runners that have Chromium and `playwright-core`.

## 9. Debugging

* **Backtraces.** `RUST_BACKTRACE=1 osatura ...` for any panic (a panic on input is a bug; please report it with the project that triggers it).
* **Machine-readable output.** `--json` on commands that support it; `--quiet` for scripts. Exit codes: 0 on success, 1 on failure (failed check, unmet budget, bad usage; run: an unknown command and a missing argument both exit 1); `osatura ci` separates 1 (a check failed) from 2 (error).
* **Inspect a project.** `osatura check`, `osatura lint` (exit 1 on errors) and `osatura diff a.json b.json` give structure and problems without rendering; `osatura render project.json N out.png` renders one frame, and `out.svg` gives vector output you can open in a browser (`--bones` shows the skeleton).
* **Studio.** Run `osatura studio project.json --port 0` (prints the real port). Browser developer tools show the JSON API: requests are plain `POST /api/...` with `Content-Type: application/json`. A 421 means a Host header not in `--allow-host`; a 403 means an Origin or `Sec-Fetch-Site` check failed (use `--public-origin` behind a proxy); 415 means the content type; 429 is the token back-off.
* **Demo stages.** The Studio's guided demo (and the step-by-step demo on the project site) are generated from `osatura demo-person` by `scripts/build-studio-tour.mjs` and `scripts/lib/person-stages.mjs`; the build fails if the finished demo no longer matches the figures quoted in the texts (`EXPECT` in the library). After changing `osatura-core/src/demo.rs`, rerun the script and rebuild the CLI.
* **Collaboration log.** The log of a shared session (`--collab-log`) is text, one `<hash> <json>` line per operation; a broken chain is reported on load.
* **A test fails only sometimes.** Look for the clock, temp-directory names and iteration order of `HashMap`; engine code is deterministic by construction, so intermittent failures are almost always test hygiene.
* **Facade snapshot fails.** You changed a public signature: see section 5, item 6.
* **GPU.** `osatura render-gpu <project> <out.png> --adapter software|auto --bench N` (only in builds with `--features gpu`) compares with the CPU renderer (a software adapter prints that its timings mean nothing); `OSATURA_GPU_BACKEND` selects the backend.
* **Sanitizers for the C ABI.** Compile `smoke.c` or your own caller with `-fsanitize=address,undefined` against the static library (done during the documentation work, clean; see [C_ABI.md](C_ABI.md)).

## 10. Profiling

* **Per-stage timings of the production pipeline:** `scripts/e2e.sh` prints a table (`OSATURA_E2E_TIMINGS` names the file); release numbers in `ROADMAP.md` section 9 come from it.
* **GPU against CPU:** `osatura render-gpu <project> <out.png> --bench N`.
* **Size and speed budget:** `osatura size-report`, `osatura export-fit --max-seconds` (the time budget is checked inside the candidate loops; one ffmpeg or quantiser call is not interruptible).
* **Render cost of a project:** `osatura ci` with a performance threshold ([CI.md](CI.md)) records per-frame time on your machine; timings are machine dependent, so compare on the same machine.
* **General tools:** build with `--release` and use `perf record`/`perf report` or `cargo flamegraph` (not part of the project; install separately). Debug builds are about an order of magnitude slower and say nothing about real speed.
