<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->
# OSATURA

**Eine 2D-Vektor-Rigging- und Animations-Engine in Rust: Knochen, IK, Skinning, Deformer, eine Zustandsautomaten-Laufzeitumgebung, eine Kommandozeilen-Toolbox, ein Browser-Studio und einbettbare Bibliotheken, alles in einem Workspace.** Rumänische Version: [README.ro.md](README.ro.md).

![Ein Charakter in Seitenansicht (Kopf, Ohr, Pferdeschwanz, Rumpf, Arme mit Fingern, Beine, Füße) läuft auf der Stelle, der Boden zieht vorbei; echte Ausgabe von osatura export für das eingebaute Demoprojekt](docs/img/walk.gif)

*Das Ergebnis: die eingebaute laufende Person, exportiert mit `osatura demo-person` und `osatura export` (das ganze Projekt ist eine JSON-Datei). In zwei Minuten ausprobieren: [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md).*

## Zusammenfassung und Philosophie

Mit OSATURA baust du einen Charakter oder eine Szene aus Vektorformen und Bildern auf, riggst sie mit Knochen und Constraints, animierst sie mit Keys, Clips und Physik und gibst sie als GIF, Video, Lottie, glTF, als eigenständigen HTML-Player oder als interaktive Laufzeitumgebung aus. OSATURA besteht aus kleinen, deterministischen Bausteinen: Dasselbe Projekt und derselbe Seed ergeben dieselben Pixel, jedes Werkzeug liest und schreibt ein dokumentiertes JSON-Projekt, und jeder Befehl läuft im Terminal. So lässt sich Animation wie Quellcode skripten, reviewen, per Diff vergleichen und testen.

Prinzipien, die das Projekt prägen:

* **Unabhängige Implementierung.** Geschrieben nach öffentlichen Spezifikationen, Fachartikeln und Lehrbuchmathematik. Die Funktionslisten von Mitbewerbern wurden für den *Umfang* herangezogen; es wurde kein fremder Quellcode kopiert, und das Projekt liest und schreibt kein proprietäres Format. Es ist kein strenger Clean-Room-Prozess (siehe [CLEANROOM.md](docs/i18n/de/CLEANROOM.md), einschließlich der Offenlegung der Recherche), und es ist weder mit Moho oder einem anderen kommerziellen Produkt verbunden noch davon abgeleitet oder damit kompatibel. Der Code wurde größtenteils mit einem KI-Modell geschrieben.
* **Ehrlicher Status.** Jede Funktion wird zusammen mit dem beschrieben, was sie *nicht* leistet ([docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md), [ROADMAP.md](docs/i18n/de/ROADMAP.md) Abschnitt 9).
* **Standardmäßig sicher.** Keine Panics bei Eingaben, Prüfung auf endliche Werte, Ressourcenobergrenzen, atomare Schreibvorgänge, Asset-Pfade auf den Projektordner beschränkt ([docs/SECURITY.md](docs/i18n/de/SECURITY.md)).
* **Offene Formate zuerst.** OpenRaster, OCA, SVG, Lottie, glTF, APNG, WebM, EXR; bewusst keine proprietären Dateiformate.
* **Kleiner Dependency-Fußabdruck** mit einer Lizenzrichtlinie, die durch Tests und `cargo deny` geprüft wird.

## Status

*Name: Das Projekt hieß bis zum 24. Okt. 2026 ARMATURA und wurde nach einer Markenrecherche in OSATURA umbenannt ([docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md) L-78). Auch die Repository-Adresse und die Site-Domain wurden umbenannt (alte Repository-Links werden weitergeleitet).*

> **Alpha. Erste Builds als Entwurf einer Vorabversion vorbereitet: `v0.1.0-alpha.3`.** Die Programme für Windows x86-64, macOS arm64 und Linux x86-64 sowie das WebAssembly-Modul werden von GitHub Actions gebaut und an eine *Entwurfs*-Vorabversion auf <https://github.com/chiuta/osatura/releases> angehängt (ein Entwurf ist nur für den Eigentümer sichtbar, bis er von Hand veröffentlicht wird). Alle Crates tragen weiterhin die Version `0.1.0`; „v0.5 / v0.6 / v0.7“ sind *Funktionsstufen* in der Historie, keine Release-Nummern ([CHANGELOG.md](docs/i18n/de/CHANGELOG.md)).
>
> | Verifiziert | Nicht verifiziert |
> |---|---|
> | **Lokal (Linux x86-64, 24. Okt. 2026):** 1338 Tests von `cargo test --workspace --exclude osatura-gpu --exclude osatura-gui` plus 19 Doc-Tests bestehen (das GPU-Crate braucht einen Adapter und das Fenster-Crate einen Desktop; davor lag die Zahl einschließlich des GPU-Crates bei 1368). **Auf GitHub Actions (24. Okt. 2026, Toolchain 1.97.0):** Die Testsuite und `clippy -D warnings` bestehen unter Linux, macOS (arm64) und Windows; `cargo deny` und `reuse lint` bestehen; das WebAssembly-Modul lässt sich für `wasm32-unknown-unknown` bauen, und der Test des Wrappers besteht gegen das echte Modul unter Node 20; die Release-Binärdateien bestehen einen Smoke-Test (`--version`, `demo`, `check`, `render`), und `osatura-studio --selftest` läuft unter Windows und macOS. Vier Headless-Chromium-Smoke-Tests des Studios bestehen; die CLI, die Rust-Fassade und die C-ABI (auch unter ASan/UBSan) wurden mit den Beispielen aus der Dokumentation ausgeführt | **Keine Person hat die Programme auf einem echten Windows- oder macOS-Desktop benutzt**: Der Windows-Doppelklick-Starter und die Fenster-App `osatura-studio` (WebView2 / WKWebView) wurden in CI nur kompiliert und selbstgetestet; die Binärdateien sind **nicht codesigniert** (SmartScreen „Unbekannter Herausgeber“, macOS Gatekeeper); die Linux-Binärdatei wurde nur auf einem Ubuntu-Runner gebaut; **das WebAssembly-Modul wurde nie in einem Browser ausgeführt**, und seine Geschwindigkeit und sein Speicherbedarf sind nicht gemessen; das Studio wurde nie von einer Person produktiv genutzt, auch nicht in einem anderen Browser als Chromium; es wurden nie eine echte GPU (nur ein Software-Adapter), eine Webcam, ein Foto oder eine Motion-Capture-Datei verwendet; kein Security-Audit, keine anwaltliche Prüfung und keine Patentrecherche |
>
> Lies [docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md) (78 nummerierte Einschränkungen mit Schweregrad und Umgehung), bevor du dich auf irgendetwas verlässt.

## Download

Fertige Archive findest du, sobald der Eigentümer die Entwurfsversion veröffentlicht hat, auf der [Releases-Seite](https://github.com/chiuta/osatura/releases) des Projekts:

| Archiv | Inhalt |
|---|---|
| `osatura-windows-x86_64.zip` | `osatura.exe` (Kommandozeile + Studio-Server), `osatura-studio.exe` (Fenster-App), C-Bibliothek und Header, Lizenzen, Hinweise zu Drittanbietern |
| `osatura-macos-arm64.tar.gz` | `osatura`, `osatura-studio` (nur Apple Silicon), C-Bibliothek und Header, Lizenzen, Hinweise |
| `osatura-linux-x86_64.tar.gz` | `osatura`, C-Bibliothek und Header, Lizenzen, Hinweise (keine Fenster-App: nutze `osatura studio` und einen Browser) |
| `osatura-web.tar.gz` | `osatura_wasm.wasm`, `osatura.js`, Demo `index.html` (in Browsern nicht verifiziert, siehe [docs/WASM.md](docs/i18n/de/WASM.md)) |
| `SHA256SUMS.txt` | prüfe jeden Download dagegen |

* **Windows:** Doppelklicke auf `osatura.exe`, um `Documents\OSATURA\first-rig.json` anzulegen und das Studio in deinem Browser zu öffnen, oder doppelklicke auf `osatura-studio.exe` für ein eigenes Fenster. Beide Programme sind unsigniert: Wähle in SmartScreen *Weitere Informationen* und dann *Trotzdem ausführen*. Siehe [docs/GETTING_STARTED.md](docs/i18n/de/GETTING_STARTED.md).
* **macOS:** Bestätige das Programm beim ersten Mal unter *Systemeinstellungen > Datenschutz & Sicherheit*.
* **Quellcode:** Jedes Archiv nennt den Commit, aus dem es gebaut wurde; die AGPL-Teile werden mit ihrem Quellcode-Angebot ausgeliefert (`osatura --version`, `/source`).

## Funktionen nach Bereichen

Details und genaue Optionen: [docs/CLI_REFERENCE.md](docs/i18n/de/CLI_REFERENCE.md); was nur teilweise oder gar nicht vorhanden ist: [docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md).

* **Zeichenmodell.** Vektorpfade, Füllungen, Konturen (Strichelungen, variable Breite), lineare und radiale Verläufe, Masken, Mischmodi, Effekte pro Ebene (Weichzeichnung, Schatten, Tönung, Füllung), Bildebenen mit Mesh-Warp, flüssige Metaballs, Polygon-Booleans, deterministische Partikel, Multiplane-Kamera mit gespeicherten Einstellungen, Tiefenschärfe.
* **Rigging.** Knochen mit Vorwärtskinematik; Zwei-Knochen-, Soft-, Stretch- und FABRIK-IK; Limits, Constraints, Federn; Skinning mit Werkzeugen für Skin-Gewichte; Lattice-, Spine-, Morph-, Wobble- und Cage-Deformer; Mesh-Tracing aus Alpha; Warp-Pivots und Glue; Driver (Slider, Joysticks, Ansichtswechsel); Skin-Sets; Auto-Rig aus Ebenennamen; Augen-Clipping; Layout-Container und Scrollbereiche.
* **Animation.** Keys mit Bézier-, Step-, Spring- und Bounce-Easing, Ausdrücke und gesteuerte Kanäle, Switch-Ebenen, Takes und ein geschichteter Clip-Stapel, Loop- und In-Place-Konvertierung, Foot Lock, Magnete, Pfadfolgen, Idle- und Overlap-Generatoren, Ketten-/Haar-/Stoffphysik, Lippensynchronisation aus Audio oder einem Transkript, sprachbewusste Kopfbewegung, BVH-Retargeting, Live-Eingabe (MIDI, Maus, Gamepad, Webcam-Landmarks, die du lieferst), Audiospuren mit Trimmen, Ausblenden und Mischen.
* **Laufzeitumgebung.** Zustandsautomaten mit eindimensionalen Blend-Trees, Datenbindung, Scroll-Physik, Detailstufen mit Fehlergrenze, eine Rhai-Scripting-Sandbox, ein Agent-Server (MCP) über Patch-Operationen.
* **Ausgabe.** SVG, PNG, PNG-Sequenzen und Sprite-Sheets, GIF, APNG, MP4/WebM (über ffmpeg), eigenständiger HTML-Player, Lottie und dotLottie, glTF-2.0-Binärformat, OpenEXR/16-Bit-PNG, OCA; größenorientierter Export (`export-fit`). Import: SVG, OpenRaster, Lottie, BVH, PNG-/GIF-/APNG-Sequenzen.
* **Qualitätswerkzeuge.** `check`, `lint`, semantischer `diff`, Pose-Fuzzing mit Heatmaps, Animations-CI mit Wahrnehmungs-Diff und Referenzbildern ([docs/CI.md](docs/i18n/de/CI.md)), Versionskontrolle mit semantischem Merge (`vcs`), Sicherheitsprüfungen für Flackern und Bewegung.
* **Studio.** Vom CLI bereitgestellter Browser-Editor: Gizmos auf der Zeichenfläche, Timeline, Dope Sheet und Kurveneditor, Overlays, Befehlspalette und Tastenkürzel, Rückgängig/Wiederholen, Mehrbenutzerbearbeitung in Echtzeit mit optionalem Token und persistentem Log ([docs/STUDIO.md](docs/i18n/de/STUDIO.md)).
* **Bibliotheken.** Stabile Rust-Fassade ([docs/API.md](docs/i18n/de/API.md)), C-ABI ([docs/C_ABI.md](docs/i18n/de/C_ABI.md)), WebAssembly-Modul und JavaScript-Wrapper (in CI gebaut und unter Node getestet, noch nicht in einem Browser ausgeführt; [docs/WASM.md](docs/i18n/de/WASM.md)), optionaler wgpu-Renderer.

## Schnellstart

Voraussetzungen: eine aktuelle stabile Rust-Toolchain (oder ein fertiges Archiv aus dem Abschnitt [Download](#download): überspringe die Zeile mit `cargo build` und nutze das entpackte `osatura`). `ffmpeg` wird nur für den MP4/WebM-Export und für mp3/ogg-Audio benötigt; es wird nie mitgeliefert. Jeder der folgenden Befehle wurde für diese README ausgeführt (Debug-Build, Linux). Release-Builds (`cargo build --release`) sind beim Rendern deutlich schneller und werden genauso mit `target/release` verwendet.

```sh
cargo build -p osatura-cli                       # builds target/debug/osatura
export PATH="$PWD/target/debug:$PATH"             # so that `osatura` is found

osatura --version                                # version, licence, source offer, support links
osatura demo demo.json                           # write the built-in rig demo (640x480, 48 frames)
osatura check demo.json                          # validate: ok: 'OSATURA demo' 640x480 @ 24 fps, 48 frames, 6 bones...
osatura lint demo.json                           # static rig/animation checks: 0 finding(s), 0 error(s)
osatura render demo.json 12 frame12.png --bones  # one frame, with the skeleton overlay
osatura render demo.json 12 frame12.svg          # the same frame as vector output
osatura frames demo.json frames_out --png        # every frame as a PNG
osatura export demo.json demo.gif                # also .html (player), .json (Lottie), .lottie, .mp4 (ffmpeg)
osatura demo-fx fx.json                          # second demo: liquid, booleans, particles
osatura lipsync demo.json voice.wav mouth out.json   # mouth shapes from a WAV file (switch layer "mouth")
osatura studio demo.json --port 8080             # browser editor at http://127.0.0.1:8080
osatura ci init demo.json && osatura ci run     # animation CI: goldens + checks (about 40 s in a debug build)
```

`voice.wav` ist eine beliebige Mono- oder Stereo-WAV-Datei, die du bereitstellst. Das Studio lauscht nur auf `127.0.0.1` und weist Anfragen für andere Hostnamen ab; wenn du es freigeben willst, lies zuerst [docs/SECURITY.md](docs/i18n/de/SECURITY.md).

Die Bibliotheken verwenden:

```sh
cargo build -p osatura-capi                      # target/debug/libosatura_capi.so and .a
gcc -std=c99 -o smoke osatura-capi/tests/smoke.c -Ltarget/debug -losatura_capi
LD_LIBRARY_PATH=target/debug ./smoke demo.json    # abi 1, 48 frames, 320x240, rc=0, checksum=...
```

Die Rust-Fassade ist `osatura_core::prelude` ([docs/API.md](docs/i18n/de/API.md)); weitere C- und Python-Beispiele findest du in [docs/C_ABI.md](docs/i18n/de/C_ABI.md).

Tests (etwa 14 Minuten in einem Debug-Build auf einem geteilten Rechner; siehe [docs/DEVELOPMENT.md](docs/i18n/de/DEVELOPMENT.md)):

```sh
cargo test --workspace
cargo test --doc -p osatura-core
cargo test -p osatura-cli --test spdx_audit
```

## Repository-Aufbau

| Pfad | Inhalt | Lizenz |
|---|---|---|
| `osatura-core/` | Engine, Formate, Werkzeuge, stabile Fassade `api` | MPL-2.0 |
| `osatura-cli/` | die `osatura`-Binärdatei, Studio und Kollaborationsserver, End-to-End- und Smoke-Tests | AGPL-3.0-or-later |
| `osatura-gui/` | `osatura-studio`, das Studio in einem nativen Fenster (Windows und macOS; unter Linux ein Platzhalter) | AGPL-3.0-or-later |
| `osatura-capi/` | C-ABI (`osatura.h`, dynamische und statische Bibliothek) | MPL-2.0 |
| `osatura-wasm/` | WebAssembly-ABI, JavaScript-Wrapper, Demoseite (in CI gebaut) | MPL-2.0 |
| `osatura-gpu/` | optionaler wgpu-Renderer (kein Standardmitglied) | MPL-2.0 |
| `docs/` | Dokumentation (CC-BY-4.0) und generierte Dateien zu Drittanbietern | CC-BY-4.0 |
| `.github/workflows/` | CI (`ci.yml`) und Release-Builds (`release.yml`) | MPL-2.0 |
| `scripts/` | `e2e.sh`, `licence-audit.sh` | MPL-2.0 |
| `LICENSES/`, `REUSE.toml`, `deny.toml` | Lizenztexte, Lizenz-Metadaten, Dependency-Richtlinie | - |
| `ROADMAP.md`, `FEATURES.md`, `CLEANROOM.md`, `LICENSING.md`, `TRADEMARK.md`, `CONTRIBUTING.md`, `CHANGELOG.md` | Projektdokumente | CC-BY-4.0 |

## Dokumentationsübersicht

Beginne bei [docs/INDEX.md](docs/i18n/de/INDEX.md) (alle Dokumente, Zielgruppen, Lesepfade).

| Seite | Für |
|---|---|
| [docs/GETTING_STARTED.md](docs/i18n/de/GETTING_STARTED.md) | erstes Projekt, Rendern, Export |
| [docs/CLI_REFERENCE.md](docs/i18n/de/CLI_REFERENCE.md) | jeder Befehl und jede Option |
| [docs/STUDIO.md](docs/i18n/de/STUDIO.md) | der Browser-Editor |
| [docs/RIGGING_GUIDE.md](docs/i18n/de/RIGGING_GUIDE.md) | Rigs bauen |
| [docs/ANIMATION_GUIDE.md](docs/i18n/de/ANIMATION_GUIDE.md) | Animieren, Clips, Physik, Audio |
| [docs/PROJECT_FORMAT.md](docs/i18n/de/PROJECT_FORMAT.md) | das JSON-Projektformat |
| [docs/EXPORT_FORMATS.md](docs/i18n/de/EXPORT_FORMATS.md) | was jedes Format enthält und was es weglässt |
| [docs/ARCHITECTURE.md](docs/i18n/de/ARCHITECTURE.md) | Entwurf und Module |
| [docs/GLOSSARY.md](docs/i18n/de/GLOSSARY.md) | Begriffe |
| [docs/FAQ.md](docs/i18n/de/FAQ.md) | häufige Fragen |
| [docs/API.md](docs/i18n/de/API.md) | Rust-Fassade 1.4 |
| [docs/C_ABI.md](docs/i18n/de/C_ABI.md) | C-Schnittstelle |
| [docs/WASM.md](docs/i18n/de/WASM.md) | WebAssembly (in CI gebaut, nicht in einem Browser ausgeführt) |
| [docs/CI.md](docs/i18n/de/CI.md) | Animations-CI |
| [docs/DEVELOPMENT.md](docs/i18n/de/DEVELOPMENT.md) | Build, Test, Konventionen, Release |
| [docs/SECURITY.md](docs/i18n/de/SECURITY.md) | Bedrohungsmodell und Härtung |
| [docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md) | bekannte Einschränkungen |
| [docs/PATENT_NOTES.md](docs/i18n/de/PATENT_NOTES.md), [docs/THIRD_PARTY.md](docs/i18n/de/THIRD_PARTY.md), [docs/THIRD_PARTY_NOTICES.md](docs/THIRD_PARTY_NOTICES.md) | Patente, Lizenzen der Abhängigkeiten |
| [CHANGELOG.md](docs/i18n/de/CHANGELOG.md), [ROADMAP.md](docs/i18n/de/ROADMAP.md), [FEATURES.md](docs/i18n/de/FEATURES.md) | Historie, Plan und detaillierter Status, Funktionsmatrix |
| [CLEANROOM.md](docs/i18n/de/CLEANROOM.md), [LICENSING.md](docs/i18n/de/LICENSING.md), [TRADEMARK.md](docs/i18n/de/TRADEMARK.md), [CONTRIBUTING.md](docs/i18n/de/CONTRIBUTING.md) | Herkunft, Lizenzen, Namensrichtlinie, Beitragsregeln |

## Lizenzübersicht

* **MPL-2.0** für die Bibliotheken `osatura-core`, `osatura-capi`, `osatura-wasm`, `osatura-gpu` und für `scripts/`: Du darfst sie in dein eigenes Programm einbinden, ob offen oder geschlossen; Änderungen an diesen Dateien müssen unter der MPL veröffentlicht werden.
* **AGPL-3.0-or-later** für `osatura-cli` (die Kommandozeile und das Studio) und `osatura-gui` (das Fenster darum herum). Wenn du es änderst und anderen die Interaktion damit über ein Netzwerk ermöglichst, verpflichtet dich Abschnitt 13 dazu, ihnen deinen Quellcode anzubieten: `osatura studio --source-url URL` veröffentlicht, wo er zu finden ist; `/source` und `osatura --version` enthalten das Angebot.
* **CC-BY-4.0** für die Dokumentation und synthetische Assets (einschließlich des synthetischen Gesichtserkennungsmodells).
* **Deine Projekte und Renderings bleiben deine.**
* Name und Logo fallen nicht unter diese Lizenzen ([TRADEMARK.md](docs/i18n/de/TRADEMARK.md)). Der Patentstatus wurde nicht geprüft ([docs/PATENT_NOTES.md](docs/i18n/de/PATENT_NOTES.md)). Eine kommerzielle oder Dual-Lizenz für beigesteuerten Code ist ohne Contributor-Lizenzvereinbarung nicht möglich, und es gibt keine.
* Crates von Drittanbietern: 363 im Inventar vom 24. Okt. 2026 (alle Plattformen und Targets zusammen, einschließlich der nur unter Windows / macOS / Linux genutzten Crates der Fenster-App; eine einzelne Binärdatei enthält eine Teilmenge), alle permissiv bis auf sechs MPL-2.0-Crates mit schwachem Copyleft: `smartstring` hinter dem optionalen Feature `scripting` und fünf (`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`), die nur die Fenster-App `osatura-gui` einbinden kann ([docs/THIRD_PARTY.md](docs/i18n/de/THIRD_PARTY.md)). ffmpeg wird nie mitgeliefert.

Siehe [LICENSE](LICENSE), [LICENSES/](LICENSES/) und [LICENSING.md](docs/i18n/de/LICENSING.md). Nichts davon ist Rechtsberatung; eine anwaltliche Prüfung steht noch aus.

## Mitwirken

Beiträge sind willkommen, und zwar unter der Lizenz des Teils, den sie ändern. Lies [CONTRIBUTING.md](docs/i18n/de/CONTRIBUTING.md) (jeder Commit braucht eine `Signed-off-by`-Zeile, DCO 1.1), [CLEANROOM.md](docs/i18n/de/CLEANROOM.md) (nur aus öffentlichen Quellen implementieren; nichts aus proprietärem oder inkompatibel lizenziertem Code kopieren) und [docs/DEVELOPMENT.md](docs/i18n/de/DEVELOPMENT.md) (Konventionen, Checkliste für ein neues Modul, Tests). Gute erste Beiträge: eine Korrektur zu einem Punkt aus [docs/LIMITATIONS.md](docs/i18n/de/LIMITATIONS.md), ein echter Durchlauf der WebAssembly-Checkliste, ein Test der Windows- / macOS-Programme auf einem echten Desktop, ein Benchmark auf einer echten GPU.

Sicherheitsprobleme: Eröffne kein öffentliches Issue; folge [docs/SECURITY.md](docs/i18n/de/SECURITY.md) Abschnitt 10.

## Unterstützung

OSATURA wird von Alexandru-Ionuț Chiuță (Alexio) entwickelt. Wenn es dir nützt, kannst du die Arbeit unterstützen:

* Buy Me a Coffee: <https://buymeacoffee.com/echo.of.the.strings>
* Patreon: <https://patreon.com/c/alexio_tf>

Kontakt zum Maintainer (wird auch von `osatura --version` ausgegeben): alexio@trom.tf.

## Danksagungen

* Die Autoren der öffentlichen Fachartikel und Lehrbücher, denen die Algorithmen folgen, pro Modul benannt in [CLEANROOM.md](docs/i18n/de/CLEANROOM.md) (Bézier- und Spline-Mathematik, inverse Kinematik, Linear-Blend-Skinning, Mean-Value-Koordinaten, Kurvenanpassung und weitere).
* Die offenen Spezifikationen, die OSATURA liest und schreibt: SVG, OpenRaster, OCA, Lottie/dotLottie, glTF, APNG, WebM, OpenEXR, BVH, PNG, GIF.
* Die Rust-Crates, auf denen es aufbaut, mit ihren Lizenzen aufgeführt in [docs/THIRD_PARTY.md](docs/i18n/de/THIRD_PARTY.md), insbesondere `tiny-skia`, `serde`, `png`, `zip`, `rhai` und `wgpu`.
* Alle, die das Projekt über die obigen Links unterstützen.
