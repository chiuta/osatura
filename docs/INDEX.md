<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA documentation index

*Start here to find the right page. All documents are English Markdown under CC-BY-4.0, except the generated third-party files (see their headers). The Romanian README is [../README.ro.md](../README.ro.md).*

OSATURA is a 2D vector rigging and animation engine in Rust. It is an alpha project (the first builds are prepared as the draft pre-release `v0.1.0-alpha.3`; crates are still 0.1.0); read [LIMITATIONS.md](LIMITATIONS.md) before depending on a feature.

Contents: [All documents](#all-documents) - [Reading paths by audience](#reading-paths-by-audience) - [Documents at the repository root](#documents-at-the-repository-root) - [Where each question is answered](#where-each-question-is-answered) - [Conventions](#conventions-used-in-the-documents)

## All documents

| Document | What it is | Audience |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Install, first project, first render and export, first studio session | everyone, first read |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Every `osatura` command, option, exit code and example | users, scripters, CI authors |
| [STUDIO.md](STUDIO.md) | The browser editor: panels, shortcuts, collaboration, scrub previews | animators, riggers |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Building a rig: bones, IK, skinning, deformers, drivers, state machines, layout | riggers, technical artists |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Keys, easing, clips and takes, physics, lip-sync, audio, export choices | animators |
| [TUTORIAL_SVG.md](TUTORIAL_SVG.md) | From a vector drawing to a walking GIF: import, rig by script, export (commands and outputs are real) | artists |
| [COMPARISON.md](COMPARISON.md) | An honest, dated comparison with other 2D animation tools | evaluators |
| [VERIFY_DOWNLOADS.md](VERIFY_DOWNLOADS.md) | Checking release archives against the SHA-256 sums | everyone installing |
| [GOOD_FIRST_TASKS.md](GOOD_FIRST_TASKS.md) | Small jobs for a first contribution | new contributors |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | The JSON project format field by field | tool builders, riggers editing JSON |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | What each export and import format carries and drops | pipeline builders, integrators |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, modules, data flow, evaluation and rendering design | contributors, reviewers |
| [GLOSSARY.md](GLOSSARY.md) | Terms used in the engine, the docs and the CLI | everyone |
| [FAQ.md](FAQ.md) | Short answers to common questions | everyone |
| [API.md](API.md) | The stable Rust facade (version 1.4.0): calls, errors, limits, examples | Rust integrators |
| [C_ABI.md](C_ABI.md) | The C interface (`osatura-capi`): functions, memory rules, C and Python examples | C, C++, Python, Go, C# integrators |
| [WASM.md](WASM.md) | WebAssembly module and JavaScript wrapper. **Status: built and tested under Node in CI; run in headless Chromium, not yet in Firefox or Safari**; includes the checklist for the first browser run | web developers |
| [CI.md](CI.md) | Animation CI (`osatura ci`): manifest, thresholds, goldens, reports, pipeline examples | technical artists, build engineers |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Building, tests, conventions, new-module checklist, release checklist, debugging | contributors, maintainers |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Owner-side steps before and after publishing the draft release | owner |
| [SECURITY.md](SECURITY.md) | Threat model, built-in protections, known gaps, reporting, hardening checklist | security reviewers, deployers |
| [LIMITATIONS.md](LIMITATIONS.md) | Numbered list of known limits with severity, workaround and roadmap id | everyone evaluating OSATURA |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Patent status note (not legal advice) | licensees, lawyers |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Generated inventory of third-party crates and licences | licensees, packagers |
| [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) | Generated licence texts of dependencies, to ship with binaries | packagers |

## Reading paths by audience

Each path is an order, not a requirement. Stop when you have what you need.

**New user (wants to see it work).** [../README.md](../README.md) (quick start) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (for the fields you edit by hand) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (the `lint`, `check`, `fuzz`, `autorig`, `machine` commands) -> [CI.md](CI.md) to guard the rig.

**Animator.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (choose the output format) -> CLI_REFERENCE (`export`, `export-fit`).

**Pipeline or tools developer (scripts, other formats).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) sections 4 to 7 (asset and file rules).

**Integrator (embedding the engine).** [ARCHITECTURE.md](ARCHITECTURE.md) overview -> choose the layer: Rust ([API.md](API.md)), C and other languages ([C_ABI.md](C_ABI.md)), browser ([WASM.md](WASM.md), read the status box first) -> [LIMITATIONS.md](LIMITATIONS.md) sections 1 and 8 -> [SECURITY.md](SECURITY.md) section 8.

**Contributor.** [../CONTRIBUTING.md](../CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](../CLEANROOM.md) -> [../ROADMAP.md](../ROADMAP.md) section 9 (what exists and does not) -> LIMITATIONS (pick something to fix).

**Security reviewer or deployer.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) section 3 (what is tested) -> LIMITATIONS section 2 and 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> [CI.md](CI.md) limits.

**Licensee, packager or lawyer.** [../LICENSING.md](../LICENSING.md) -> [../COPYRIGHT](../COPYRIGHT) and [../LICENSES/](../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](../TRADEMARK.md) -> [../CLEANROOM.md](../CLEANROOM.md) -> release checklist in [DEVELOPMENT.md](DEVELOPMENT.md) section 7.

**Maintainer making a release.** DEVELOPMENT section 7 -> [../CHANGELOG.md](../CHANGELOG.md) -> LIMITATIONS (update) -> README status box.

## Documents at the repository root

| File | Purpose |
|---|---|
| [../README.md](../README.md), [../README.ro.md](../README.ro.md) | Overview, status, quick start, licence summary (English and Romanian) |
| [../CHANGELOG.md](../CHANGELOG.md) | Reconstructed history; Keep-a-Changelog format |
| [../ROADMAP.md](../ROADMAP.md) | Ordered plan; **section 9 is the dated log of what each item does and does not do** and the authority when documents disagree |
| [../FEATURES.md](../FEATURES.md) | Feature matrix against other tools (parts are stale, see LIMITATIONS) |
| [../CLEANROOM.md](../CLEANROOM.md) | Provenance log and rules for independent implementation |
| [../CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution rules, inbound = outbound licence, DCO 1.1 |
| [../LICENSING.md](../LICENSING.md), [../COPYRIGHT](../COPYRIGHT), [../LICENSES/](../LICENSES/) | Licence decision (Variant A), texts |
| [../TRADEMARK.md](../TRADEMARK.md) | Name and logo policy |
| `../REUSE.toml`, `../deny.toml` | Licence metadata for non-source files; dependency policy |

## Where each question is answered

| Question | Page |
|---|---|
| How do I install and make my first animation? | GETTING_STARTED |
| What does command X do? | CLI_REFERENCE |
| What does this field in the JSON mean? | PROJECT_FORMAT, GLOSSARY |
| Which export format keeps masks / gradients / audio? | EXPORT_FORMATS |
| Can I use it from Python / C / the browser? | C_ABI / WASM (status!) |
| Is it safe to expose the studio to a network? | SECURITY section 11 |
| Why does X not work? Is it known? | LIMITATIONS, FAQ |
| How is it built and tested? | DEVELOPMENT |
| Can I use it commercially? What are the licences? | README (licence), LICENSING, THIRD_PARTY |
| What changed? | CHANGELOG, ROADMAP section 9 |

## Conventions used in the documents

* **"Verified"** means a command or program was run while the documents were written, in an environment with Linux x86-64, no GPU, no webcam and only headless Chromium. **"Not tested"** means exactly that. Statements about WebAssembly in a real engine are always unverified.
* Code samples in API.md and C_ABI.md are the programs that were compiled and run; samples in WASM.md ran only against the JavaScript mock.
* Section 9 of the ROADMAP is the authority for what a feature does and does not do. If a document disagrees with it or with the code, the code wins; please report the difference.
* Paths are relative to the repository root unless they start with `docs/`.
* Numbers (test counts, limits) come from the source or from a run on 22 Oct 2026 and may be out of date after the next change; the source is authoritative.
