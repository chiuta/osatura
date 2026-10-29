<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Verifying a download

*The release files are signed by the project's CI with **Sigstore** (keyless) and carry a **GitHub build-provenance attestation**. Both prove that a file was built by `.github/workflows/release.yml` in `chiuta/osatura` from a specific commit. They are **not** operating-system code signatures (Authenticode, Apple Developer ID): Windows SmartScreen and macOS Gatekeeper still warn (see [LIMITATIONS.md](LIMITATIONS.md), L-74).*

**A valid signature or attestation is not the same as "this is a release".** It says where and how a file was built. The release workflow only runs for a version tag (`vMAJOR.MINOR.PATCH[-pre]`) whose commit is on `main`, or by hand from `main`; but files built by earlier versions of the workflow, or by a manual run, carry an equally valid signature. The commands below therefore pin **the workflow** and **the tag** you expect; use them as written and do not drop the last option.

Set the Git reference the release was built from, once, for the commands below (Linux/macOS shell; in PowerShell write `$REF = '...'`):

```
REF=refs/tags/v0.1.0-alpha.5      # refs/tags/<the tag of the release you downloaded>
```

**Exception, v0.1.0-alpha.4 only:** that release was built by the workflow from a commit on `main` (the tag was created when the release was published), so its signatures and attestations name `refs/heads/main`: use `REF=refs/heads/main` for it. From the next release on the workflow runs on the tag itself and `REF` is always `refs/tags/<tag>`.

## Strongest check: who built the file, from which tag

**With the GitHub CLI** (`gh`, logged in to any GitHub account):

```
gh attestation verify osatura-windows-x86_64.zip -R chiuta/osatura \
  --signer-workflow chiuta/osatura/.github/workflows/release.yml \
  --source-ref $REF
```

It prints the repository, the workflow, the reference and the commit the file was built from. Any change to the file makes it fail. Without `--signer-workflow` and `--source-ref` the same command also accepts a file built from any branch of the repository by anyone who can push to it; that is why they are part of the command. Check that `REF` is the tag of the release you are looking at, not a branch (except for alpha.4, above).

**With cosign** (no GitHub account needed; download the file and its `.sigstore.json` from the same release):

```
cosign verify-blob osatura-windows-x86_64.zip \
  --bundle osatura-windows-x86_64.zip.sigstore.json \
  --certificate-identity "https://github.com/chiuta/osatura/.github/workflows/release.yml@$REF" \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com
```

`Verified OK` means the signature was made by that workflow, in that repository, for that reference, and is recorded in the public Rekor log. The identity is the exact reference, not a pattern: a signature from a branch (a manual run from `main`) does not verify against a tag, on purpose. Signing `SHA256SUMS.txt` the same way and then checking the sums below covers every file at once. A build from a fork is signed with the fork's own identity and fails this check, as intended.

## Quick check: checksums

Every release carries `SHA256SUMS.txt` next to the archives (`osatura-linux-x86_64.tar.gz`, `osatura-linux-x86_64-musl.tar.gz`, `osatura-macos-arm64.tar.gz`, `osatura-windows-x86_64.zip`, `osatura-web.tar.gz`) and `osatura.spdx.json`, a software bill of materials (SPDX 2.3 JSON, every crate in `Cargo.lock` with its licence), signed and attested like the archives. **v0.1.0-alpha.4 has neither the musl archive nor the SBOM**; they start with the next release. Download the archive and the sums file into the same folder. Each command below ends with a non-zero exit status (or an error) when the file does not match.

**Linux:**

```
sha256sum -c --ignore-missing SHA256SUMS.txt
```

**macOS** (the file name is the one you downloaded; the status of the command is the status of the check):

```
grep -F ' osatura-macos-arm64.tar.gz' SHA256SUMS.txt | shasum -a 256 -c -
```

**Windows (PowerShell):**

```
$f = 'osatura-windows-x86_64.zip'
$want = ((Select-String -Path .\SHA256SUMS.txt -SimpleMatch " $f").Line -split '\s+')[0]
$have = (Get-FileHash ".\$f" -Algorithm SHA256).Hash
if ($want -and ($have -ieq $want)) { "OK: $f" } else { Write-Error "MISMATCH or file not listed: do not run $f" }
```

A different value means a damaged or replaced file: do not run it.

A sum proves the file matches what was published with it; it does not prove who published it, so check the signature above too, and get both files from the project's release page.

## Building it yourself

Every archive contains `SOURCE.txt` with the commit it was built from, and the studio's `/source` page points to the same commit (AGPL section 13). The Rust version is fixed in `rust-toolchain.toml`; the steps are in [DEVELOPMENT.md](DEVELOPMENT.md). **The build is not bit-for-bit reproducible yet:** you get a working binary from the same source, but its SHA-256 will differ from the published file (one known cause is `const-random`, which the scripting engine's hasher pulls in and which draws random constants at compile time; there may be others). Do not use a hash comparison with your own build as a check until this page says the build is reproducible.
