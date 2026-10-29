<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA-Dokumentationsindex

*Beginne hier, um die richtige Seite zu finden. Alle Dokumente sind englisches Markdown unter CC-BY-4.0, mit Ausnahme der generierten Dateien zu Drittanbietern (siehe deren Header).*

OSATURA ist eine 2D-Vektor-Rigging- und Animations-Engine in Rust. Es ist ein Alpha-Projekt (die ersten Builds sind als Entwurfs-Vorabversion `v0.1.0-alpha.3` vorbereitet; die Crates stehen weiterhin auf 0.1.0); lies [LIMITATIONS.md](LIMITATIONS.md), bevor du dich auf eine Funktion verlässt.

Inhalt: [Alle Dokumente](#alle-dokumente) - [Lesepfade nach Zielgruppe](#lesepfade-nach-zielgruppe) - [Dokumente im Stammverzeichnis des Repositorys](#dokumente-im-stammverzeichnis-des-repositorys) - [Wo jede Frage beantwortet wird](#wo-jede-frage-beantwortet-wird) - [Konventionen](#in-den-dokumenten-verwendete-konventionen)

## Alle Dokumente

| Dokument | Was es ist | Zielgruppe |
|---|---|---|
| [GETTING_STARTED.md](GETTING_STARTED.md) | Installation, erstes Projekt, erstes Rendern und Exportieren, erste Studio-Sitzung | alle, als Erstes zu lesen |
| [CLI_REFERENCE.md](CLI_REFERENCE.md) | Jeder `osatura`-Befehl, jede Option, jeder Exit-Code und jedes Beispiel | Anwender, Skript-Autoren, CI-Autoren |
| [STUDIO.md](STUDIO.md) | Der Browser-Editor: Panels, Tastenkürzel, Zusammenarbeit, Scrub-Vorschauen | Animatoren, Rigger |
| [RIGGING_GUIDE.md](RIGGING_GUIDE.md) | Ein Rig bauen: Knochen, IK, Skinning, Deformer, Driver, Zustandsautomaten, Layout | Rigger, Technical Artists |
| [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) | Keys, Easing, Clips und Takes, Physik, Lip-Sync, Audio, Wahl des Exports | Animatoren |
| [TUTORIAL_SVG.md](../../TUTORIAL_SVG.md) | Von einer Vektorzeichnung zu einem laufenden GIF: Import, Rig per Skript, Export (Befehle und Ausgaben sind echt) | Künstler |
| [COMPARISON.md](../../COMPARISON.md) | Ein ehrlicher, datierter Vergleich mit anderen 2D-Animationswerkzeugen | Evaluierende |
| [VERIFY_DOWNLOADS.md](../../VERIFY_DOWNLOADS.md) | Release-Dateien prüfen: Sigstore-Signaturen, Build-Provenienz, SHA-256-Summen | alle, die installieren |
| [UNINSTALL.md](../../UNINSTALL.md) | OSATURA und alle Dateien entfernen, die es anlegen kann | alle, die installieren |
| [../CODE_SIGNING_POLICY.md](../../../CODE_SIGNING_POLICY.md) · [../PRIVACY.md](../../../PRIVACY.md) | Code-Signing-Richtlinie (SignPath Foundation), Rollen im Team, Datenschutz | alle, die installieren, Maintainer |
| [GOOD_FIRST_TASKS.md](../../GOOD_FIRST_TASKS.md) | Kleine Aufgaben für einen ersten Beitrag | neue Mitwirkende |
| [PROJECT_FORMAT.md](PROJECT_FORMAT.md) | Das JSON-Projektformat Feld für Feld | Werkzeugentwickler, Rigger, die JSON bearbeiten |
| [EXPORT_FORMATS.md](EXPORT_FORMATS.md) | Was jedes Export- und Importformat enthält und weglässt | Pipeline-Entwickler, Integratoren |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Crates, Module, Datenfluss, Entwurf von Auswertung und Rendering | Mitwirkende, Reviewer |
| [GLOSSARY.md](GLOSSARY.md) | Begriffe aus Engine, Dokumentation und CLI | alle |
| [FAQ.md](FAQ.md) | Kurze Antworten auf häufige Fragen | alle |
| [API.md](API.md) | Die stabile Rust-Fassade (Version 1.4.0): Aufrufe, Fehler, Grenzen, Beispiele | Rust-Integratoren |
| [C_ABI.md](C_ABI.md) | Die C-Schnittstelle (`osatura-capi`): Funktionen, Speicherregeln, C- und Python-Beispiele | Integratoren für C, C++, Python, Go, C# |
| [WASM.md](WASM.md) | WebAssembly-Modul und JavaScript-Wrapper. **Status: in CI gebaut und unter Node getestet; in Headless-Chromium ausgeführt, noch nicht in Firefox oder Safari**; enthält die Checkliste für den ersten Lauf im Browser | Webentwickler |
| [CI.md](CI.md) | Animations-CI (`osatura ci`): Manifest, Schwellwerte, Goldens, Berichte, Pipeline-Beispiele | Technical Artists, Build-Ingenieure |
| [DEVELOPMENT.md](DEVELOPMENT.md) | Bauen, Tests, Konventionen, Checkliste für neue Module, Release-Checkliste, Debugging | Mitwirkende, Maintainer |
| [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md) | Schritte auf Eigentümerseite vor und nach der Veröffentlichung der Entwurfsversion | Eigentümer |
| [SECURITY.md](SECURITY.md) | Bedrohungsmodell, eingebaute Schutzmaßnahmen, bekannte Lücken, Meldung, Härtungs-Checkliste | Sicherheits-Reviewer, Betreiber |
| [LIMITATIONS.md](LIMITATIONS.md) | Nummerierte Liste bekannter Einschränkungen mit Schweregrad, Umgehung und Roadmap-ID | alle, die OSATURA bewerten |
| [PATENT_NOTES.md](PATENT_NOTES.md) | Hinweis zum Patentstatus (keine Rechtsberatung) | Lizenznehmer, Juristen |
| [THIRD_PARTY.md](THIRD_PARTY.md) | Generiertes Verzeichnis der Crates und Lizenzen von Drittanbietern | Lizenznehmer, Paketierer |
| [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) | Generierte Lizenztexte der Abhängigkeiten, die mit Binärdateien auszuliefern sind | Paketierer |

## Lesepfade nach Zielgruppe

Jeder Pfad ist eine Reihenfolge, keine Vorschrift. Hör auf, wenn du hast, was du brauchst.

**Neue Anwender (wollen es funktionieren sehen).** [../README.md](../../../README.de.md) (Schnellstart) -> [GETTING_STARTED.md](GETTING_STARTED.md) -> [STUDIO.md](STUDIO.md) -> [FAQ.md](FAQ.md) -> [LIMITATIONS.md](LIMITATIONS.md).

**Rigger.** GETTING_STARTED -> [RIGGING_GUIDE.md](RIGGING_GUIDE.md) -> [PROJECT_FORMAT.md](PROJECT_FORMAT.md) (für die Felder, die du von Hand bearbeitest) -> [GLOSSARY.md](GLOSSARY.md) -> [CLI_REFERENCE.md](CLI_REFERENCE.md) (die Befehle `lint`, `check`, `fuzz`, `autorig`, `machine`) -> [CI.md](CI.md), um das Rig abzusichern.

**Animator.** GETTING_STARTED -> STUDIO -> [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md) -> [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (das Ausgabeformat wählen) -> CLI_REFERENCE (`export`, `export-fit`).

**Pipeline- oder Werkzeugentwickler (Skripte, andere Formate).** CLI_REFERENCE -> PROJECT_FORMAT -> EXPORT_FORMATS -> [CI.md](CI.md) -> [SECURITY.md](SECURITY.md) Abschnitte 4 bis 7 (Regeln für Assets und Dateien).

**Integrator (bettet die Engine ein).** Überblick in [ARCHITECTURE.md](ARCHITECTURE.md) -> die Schicht wählen: Rust ([API.md](API.md)), C und andere Sprachen ([C_ABI.md](C_ABI.md)), Browser ([WASM.md](WASM.md), zuerst den Statuskasten lesen) -> [LIMITATIONS.md](LIMITATIONS.md) Abschnitte 1 und 8 -> [SECURITY.md](SECURITY.md) Abschnitt 8.

**Mitwirkende.** [../CONTRIBUTING.md](CONTRIBUTING.md) (DCO) -> [DEVELOPMENT.md](DEVELOPMENT.md) -> ARCHITECTURE -> [../CLEANROOM.md](CLEANROOM.md) -> [../ROADMAP.md](ROADMAP.md) Abschnitt 9 (was es gibt und was nicht) -> LIMITATIONS (etwas zum Beheben aussuchen).

**Sicherheits-Reviewer oder Betreiber.** [SECURITY.md](SECURITY.md) -> [DEVELOPMENT.md](DEVELOPMENT.md) Abschnitt 3 (was getestet wird) -> LIMITATIONS Abschnitt 2 und 11 -> `osatura-cli/src/http.rs`, `studio.rs`, `collab_srv.rs`, `osatura-core/src/assets.rs`, `fsutil.rs` -> [CI.md](CI.md) Einschränkungen.

**Lizenznehmer, Paketierer oder Jurist.** [../LICENSING.md](LICENSING.md) -> [../COPYRIGHT](../../../../../../COPYRIGHT) und [../LICENSES/](../../../LICENSES/) -> [THIRD_PARTY.md](THIRD_PARTY.md) -> [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md) -> [PATENT_NOTES.md](PATENT_NOTES.md) -> [../TRADEMARK.md](TRADEMARK.md) -> [../CLEANROOM.md](CLEANROOM.md) -> Release-Checkliste in [DEVELOPMENT.md](DEVELOPMENT.md) Abschnitt 7.

**Maintainer, der ein Release erstellt.** DEVELOPMENT Abschnitt 7 -> [../CHANGELOG.md](CHANGELOG.md) -> LIMITATIONS (aktualisieren) -> Statuskasten der README.

## Dokumente im Stammverzeichnis des Repositorys

| Datei | Zweck |
|---|---|
| [../README.md](../../../README.de.md), [../README.ro.md](../../../README.de.md) | Überblick, Status, Schnellstart, Lizenzübersicht (Englisch und Rumänisch) |
| [../CHANGELOG.md](CHANGELOG.md) | Rekonstruierte Historie; Format nach Keep a Changelog |
| [../ROADMAP.md](ROADMAP.md) | Geordneter Plan; **Abschnitt 9 ist das datierte Log dessen, was jeder Punkt leistet und was nicht**, und maßgeblich, wenn Dokumente sich widersprechen |
| [../FEATURES.md](FEATURES.md) | Funktionsmatrix im Vergleich mit anderen Werkzeugen (Teile sind veraltet, siehe LIMITATIONS) |
| [../CLEANROOM.md](CLEANROOM.md) | Herkunftslog und Regeln für unabhängige Implementierung |
| [../CONTRIBUTING.md](CONTRIBUTING.md) | Beitragsregeln, Inbound = Outbound-Lizenz, DCO 1.1 |
| [../LICENSING.md](LICENSING.md), [../COPYRIGHT](../../../../../../COPYRIGHT), [../LICENSES/](../../../LICENSES/) | Lizenzentscheidung (Variante A), Texte |
| [../TRADEMARK.md](TRADEMARK.md) | Richtlinie zu Name und Logo |
| `../REUSE.toml`, `../deny.toml` | Lizenz-Metadaten für Nicht-Quelldateien; Dependency-Richtlinie |

## Wo jede Frage beantwortet wird

| Frage | Seite |
|---|---|
| Wie installiere ich es und mache meine erste Animation? | GETTING_STARTED |
| Was macht Befehl X? | CLI_REFERENCE |
| Was bedeutet dieses Feld im JSON? | PROJECT_FORMAT, GLOSSARY |
| Welches Exportformat behält Masken / Verläufe / Audio? | EXPORT_FORMATS |
| Kann ich es aus Python / C / dem Browser verwenden? | C_ABI / WASM (Status!) |
| Ist es sicher, das Studio in einem Netzwerk freizugeben? | SECURITY Abschnitt 11 |
| Warum funktioniert X nicht? Ist das bekannt? | LIMITATIONS, FAQ |
| Wie wird es gebaut und getestet? | DEVELOPMENT |
| Kann ich es kommerziell nutzen? Welche Lizenzen gelten? | README (Lizenz), LICENSING, THIRD_PARTY |
| Was hat sich geändert? | CHANGELOG, ROADMAP Abschnitt 9 |

## In den Dokumenten verwendete Konventionen

* **„Verified“ (verifiziert)** bedeutet, dass ein Befehl oder Programm beim Schreiben der Dokumente ausgeführt wurde, in einer Umgebung mit Linux x86-64, ohne GPU, ohne Webcam und nur mit Headless-Chromium. **„Not tested“ (nicht getestet)** bedeutet genau das. Aussagen zu WebAssembly in einer echten Engine sind stets unverifiziert.
* Codebeispiele in API.md und C_ABI.md sind die Programme, die kompiliert und ausgeführt wurden; Beispiele in WASM.md liefen nur gegen den JavaScript-Mock.
* Abschnitt 9 der ROADMAP ist maßgeblich dafür, was eine Funktion leistet und was nicht. Wenn ein Dokument ihm oder dem Code widerspricht, gilt der Code; bitte melde den Unterschied.
* Pfade sind relativ zum Stammverzeichnis des Repositorys, sofern sie nicht mit `docs/` beginnen.
* Zahlen (Testanzahlen, Grenzen) stammen aus dem Quellcode oder aus einem Lauf am 22. Okt. 2026 und können nach der nächsten Änderung veraltet sein; maßgeblich ist der Quellcode.
