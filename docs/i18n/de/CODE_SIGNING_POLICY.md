<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# Richtlinie zur Code-Signierung

Kostenlose Code-Signierung durch [SignPath.io](https://about.signpath.io/), Zertifikat der [SignPath Foundation](https://signpath.org/).

*Stand (25. Okt. 2026): Das Projekt hat ein kostenloses Zertifikat der SignPath Foundation beantragt oder steht kurz davor. Solange ein Release nichts anderes sagt, sind die Windows-Dateien **nicht** mit Authenticode signiert; jede Release-Datei trägt bereits eine Sigstore-Signatur und eine GitHub-Attestierung zur Build-Provenienz (siehe [Prüfung](#prüfung)).*

## Was signiert wird

Signiert werden nur Dateien, die aus diesem Repository ([github.com/chiuta/osatura](https://github.com/chiuta/osatura)) von dessen Release-Workflow ([`.github/workflows/release.yml`](../../../.github/workflows/release.yml)) auf von GitHub gehosteten Runnern gebaut wurden. Nichts, was auf einem privaten Computer gebaut wurde, wird jemals signiert, und keine Binärdatei von Dritten wird signiert.

| Datei (in `osatura-windows-x86_64.zip`) | Was es ist | Lizenz |
|---|---|---|
| `osatura.exe` | Kommandozeilenwerkzeug und Browser-Studio | AGPL-3.0-or-later |
| `osatura-studio.exe` | das Studio in einem nativen Fenster (WebView2) | AGPL-3.0-or-later |
| `lib/osatura_capi.dll` | C-Bibliothek | MPL-2.0 |

Jede signierte Datei trägt den Produktnamen `OSATURA` und die Release-Version in ihren Dateieigenschaften (von `build.rs` in jedem Crate eingebettet und vom Workflow geprüft, bevor irgendetwas signiert wird). Die SignPath-Artefaktkonfiguration liegt in [`.signpath/artifact-configuration.xml`](../../../.signpath/artifact-configuration.xml).

## Rollen im Team

Das Projekt hat derzeit einen einzigen Maintainer, der alle drei Rollen innehat:

| Rolle | Wer |
|---|---|
| Committer und Reviewer | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |
| Freigeber | [Alexandru-Ionuț Chiuță (Alexio), @chiuta](https://github.com/chiuta) |

* Änderungen von Personen ohne Schreibrecht kommen als Pull Requests und werden von einem Committer geprüft, bevor sie zusammengeführt werden ([CONTRIBUTING.md](CONTRIBUTING.md)).
* Eine Signieranfrage wird nur vom Release-Workflow für ein Versions-Tag (`vMAJOR.MINOR.PATCH[-pre]`) erstellt, dessen Commit auf `main` liegt, und jede Anfrage wird in SignPath von Hand durch einen Freigeber genehmigt.
* Alle mit einer dieser Rollen müssen für GitHub und SignPath eine Multi-Faktor-Authentifizierung verwenden (eine Bedingung der SignPath Foundation; der Maintainer bestätigt dies für jedes Konto, wenn das Zertifikat erteilt wird).
* Jede Änderung dieser Rollen wird in dieser Datei festgehalten.

## Datenschutzrichtlinie

Dieses Programm überträgt keine Informationen an andere vernetzte Systeme, es sei denn, der Nutzer oder die Person, die es installiert oder betreibt, verlangt dies ausdrücklich. Einzelheiten: [PRIVACY.md](PRIVACY.md).

## Änderungen an deinem System und Deinstallation

OSATURA hat kein Installationsprogramm, schreibt nichts in die Registry und installiert keinen Dienst, keinen Treiber und keinen Autostart-Eintrag. Die wenigen Dateien, die es anlegen kann, und wie du alles entfernst, sind in [UNINSTALL.md](../../UNINSTALL.md) aufgeführt.

## Lizenzen

Der gesamte Code ist Open Source unter von der OSI anerkannten Lizenzen: MPL-2.0 (Bibliotheken) und AGPL-3.0-or-later (Kommandozeilenwerkzeug, Studio und Fenster-App); die Dokumentation steht unter CC-BY-4.0 ([COPYRIGHT](../../../COPYRIGHT), [LICENSING.md](LICENSING.md)). Die signierten Dateien enthalten keinen proprietären Code, und OSATURA wird unter keiner kommerziellen oder proprietären Lizenz angeboten (kein Dual Licensing).

## Prüfung

* **Authenticode** (sobald verfügbar): Rechtsklick auf eine Datei im Windows-Explorer > Eigenschaften > Digitale Signaturen; der Unterzeichner ist die SignPath Foundation.
* **Herkunft jeder Release-Datei**: `gh attestation verify <file> -R chiuta/osatura` oder `cosign verify-blob` mit dem `.sigstore.json`-Bundle ([VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md)).

## Ein Problem melden

Wenn du glaubst, dass eine für OSATURA signierte Datei bösartig ist, nicht aus diesem Repository gebaut wurde oder dass diese Richtlinie verletzt wurde, schreibe an <alexio@trom.tf> oder eröffne ein Issue; Sicherheitsmeldungen folgen [SECURITY.md](SECURITY.md). Probleme mit dem Zertifikat selbst können auch der [SignPath Foundation](https://signpath.org/) gemeldet werden.
