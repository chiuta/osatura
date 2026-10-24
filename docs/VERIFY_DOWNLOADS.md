<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Verifying a download

*The release archives are **not code-signed**: Windows SmartScreen and macOS Gatekeeper will warn about them (see [LIMITATIONS.md](LIMITATIONS.md)). Until signing exists, the SHA-256 sums are the way to check that a file is the one CI built.*

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
