<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Verifying a download

*The release files are signed by the project's CI with **Sigstore** (keyless) and carry a **GitHub build-provenance attestation**. Both prove that a file was built by `.github/workflows/release.yml` in `chiuta/osatura` from a specific commit. They are **not** operating-system code signatures (Authenticode, Apple Developer ID): Windows SmartScreen and macOS Gatekeeper still warn (see [LIMITATIONS.md](LIMITATIONS.md), L-74).*

## Strongest check: who built the file

**With the GitHub CLI** (`gh`, logged in to any GitHub account):

```
gh attestation verify osatura-windows-x86_64.zip -R chiuta/osatura
```

It prints the repository, the workflow and the commit the file was built from. Any change to the file makes it fail.

**With cosign** (no GitHub account needed; download the file and its `.sigstore.json` from the same release):

```
cosign verify-blob osatura-windows-x86_64.zip \
  --bundle osatura-windows-x86_64.zip.sigstore.json \
  --certificate-identity-regexp '^https://github\.com/chiuta/osatura/\.github/workflows/release\.yml@refs/(tags/v.+|heads/main)$' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

`Verified OK` means the signature was made by that workflow, in that repository, and is recorded in the public Rekor log. Signing `SHA256SUMS.txt` the same way and then checking the sums below covers every file at once. A build from a fork is signed with the fork's own identity and fails this check, as intended.

## Quick check: checksums

Every release carries `SHA256SUMS.txt` next to the archives (`osatura-linux-x86_64.tar.gz`, `osatura-macos-arm64.tar.gz`, `osatura-windows-x86_64.zip`, `osatura-web.tar.gz`). Download the archive and the sums file into the same folder.

**Linux:**

```
sha256sum -c --ignore-missing SHA256SUMS.txt
```

**macOS:**

```
shasum -a 256 -c SHA256SUMS.txt 2>&1 | grep ': OK$'
```

**Windows (PowerShell):**

```
(Get-FileHash .\osatura-windows-x86_64.zip -Algorithm SHA256).Hash.ToLower()
```

Compare the printed value with the line for that file in `SHA256SUMS.txt`. A different value means a damaged or replaced file: do not run it.

A sum proves the file matches what was published with it; it does not prove who published it, so get both files from the project's release page. Every archive also contains `SOURCE.txt` with the commit it was built from, and the studio's `/source` page points to the same commit (AGPL section 13), so you can rebuild it yourself: [DEVELOPMENT.md](DEVELOPMENT.md).
