<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/).

*Status (25 Oct 2026): the project has applied for, or is about to apply for, a free SignPath Foundation certificate. Until a release says otherwise, the Windows files are **not** Authenticode-signed; every release file already carries a Sigstore signature and a GitHub build-provenance attestation (see [Verification](#verification)).*

## What is signed

Only files built from this repository ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) by its release workflow ([`.github/workflows/release.yml`](.github/workflows/release.yml)) on GitHub-hosted runners are signed. Nothing built on a personal computer is ever signed, and no third-party binary is signed.

| File (in `osatura-windows-x86_64.zip`) | What it is | Licence |
|---|---|---|
| `osatura.exe` | command-line tool and browser studio | AGPL-3.0-or-later |
| `osatura-studio.exe` | the studio in a native window (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | C library | MPL-2.0 |

Every signed file has the product name `OSATURA` and the release version in its file properties (embedded by `build.rs` in each crate and checked by the workflow before anything is signed). The SignPath artifact configuration is kept in [`.signpath/artifact-configuration.xml`](.signpath/artifact-configuration.xml).

## Team roles

The project currently has a single maintainer, who holds all three roles:

| Role | Who |
|---|---|
| Committers and reviewers | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Approvers | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Changes from anyone without write access arrive as pull requests and are reviewed by a committer before they are merged ([CONTRIBUTING.md](CONTRIBUTING.md)).
* A signing request is created only by the release workflow for a version tag (`v*`) or a release commit on `main`, and each request is approved by hand in SignPath by an approver.
* Everyone with one of these roles uses multi-factor authentication for GitHub and SignPath.
* Any change to these roles is recorded in this file.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it. Details: [PRIVACY.md](PRIVACY.md).

## Changes to your system and uninstalling

OSATURA has no installer, writes nothing to the registry and installs no service, driver or startup entry. The few files it may create, and how to remove everything, are listed in [docs/UNINSTALL.md](docs/UNINSTALL.md).

## Licences

All code is open source under OSI-approved licences: MPL-2.0 (libraries) and AGPL-3.0-or-later (command-line tool, studio and window app); the documentation is CC-BY-4.0 ([LICENSE](LICENSE), [LICENSING.md](LICENSING.md)). The signed files contain no proprietary code, and OSATURA is not offered under any commercial or proprietary licence (no dual licensing).

## Verification

* **Authenticode** (once available): right-click a file in Windows Explorer > Properties > Digital Signatures; the signer is SignPath Foundation.
* **Origin of every release file**: `gh attestation verify <file> -R chiuta/osatura`, or `cosign verify-blob` with the `.sigstore.json` bundle ([docs/VERIFY_DOWNLOADS.md](docs/VERIFY_DOWNLOADS.md)).

## Reporting a problem

If you think a file signed for OSATURA is malicious, was not built from this repository, or that this policy was broken, write to <alexio@trom.tf> or open an issue; security reports follow [docs/SECURITY.md](docs/SECURITY.md). Problems with the certificate itself can also be reported to [SignPath Foundation](https://signpath.org/).
