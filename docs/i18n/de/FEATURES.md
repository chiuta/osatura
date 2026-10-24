# OSATURA — Feature-Matrix + Checklisten zu Referenz- und Konkurrenzprodukten

> Hinweis zu Versionsangaben: Alle Crates haben die Version `0.1.0` (die ersten Builds sind die Entwurfs-Vorabversion `v0.1.0-alpha.3`); „v0.5 / v0.6 / v0.7" bezeichnen in diesem Repository *Funktionsstufen* (siehe [CHANGELOG.md](CHANGELOG.md)), und `ROADMAP.md` Abschnitt 9 ist maßgeblich dafür, was gebaut ist.

Ehrlicher Stand dessen, was existiert. **✅ fertig & getestet · 🟡 teilweise · ❌ nicht implementiert.**

> Hinweis zum Umfang: OSATURA ist eine unabhängig implementierte Engine (aus öffentlichen Spezifikationen und Fachartikeln; Feature-Listen der Konkurrenz wurden zur Abgrenzung des Umfangs herangezogen; es wurde kein fremder Quellcode kopiert; zur Offenlegung der Recherche siehe CLEANROOM.md), keine Neuimplementierung eines
> kommerziellen Produkts, und sie erreicht **keine** Parität mit einer 25 Jahre alten kommerziellen Suite
> (ausgereifte GUI, Zeichenwerkzeuge, Live-Eingabe, 3D, Ökosystem). Die ersten Tabellen sind nach
> Fähigkeiten gegliedert; die **zweite Hälfte dieser Datei** ist eine Checkliste der öffentlich dokumentierten Funktionen
> des Referenzprodukts, Version für Version (Quellen am Ende), mit
> dem jeweiligen Status in OSATURA. „Innovationen" werden nur relativ zur allgemeinen Technik behauptet,
> nicht als nachweislich in keinem anderen Werkzeug vorhanden.

---

# ★ BUILD LADDER — von einfach zu komplex, mit Urheberrechts-/IP-Status (Funktionsstufe v0.7, 20.–22. Okt. 2026)

Dieser Abschnitt ist die **einzige Stelle, an der man sehen kann, was sich bauen lässt, in welcher Reihenfolge und ob es unbedenklich ist**.
Er ersetzt die verstreuten „Kandidat #"-Listen weiter unten; Details und IDs stehen in `ROADMAP.md`
(R-xxx / I-xx), die Clean-Room-Methode in `CLEANROOM.md`, die Lizenzen in `LICENSING.md`.

**Status:** ✅ gebaut & getestet · 🟡 teilweise gebaut · ⬜ geplant (nicht begonnen)
**IP-Status (nach den Alternativen aus ROADMAP §8):**
🟢 unbedenklich — offener Standard, Lehrbuchmathematik oder eigenes Design ·
🟠 unbedenklich, *wenn* eine Bedingung eingehalten wird (in der letzten Spalte genannt) ·
🔴 nicht bauen.
*Technische Einschätzung, keine Rechtsberatung; ein Anwalt sollte jedes 🟠 vor einer kommerziellen Veröffentlichung bestätigen.*

## Stufe 1 — Grundlagen (je Stunden bis Tage; keinerlei IP-Risiko)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| Projekt-JSON, Validierung, reines `evaluate(frame)` | ✅ | 🟢 | eigenes Schema |
| Bézier-Pfade, Füllungen, Konturen, Verläufe, Strichelungen, Konturen mit variabler Breite | ✅ | 🟢 | Lehrbuchgeometrie |
| FK-Knochen, Limits, 2-Knochen-/FABRIK-IK, Constraints | ✅ | 🟢 | Lehrbuchkinematik |
| Keyframes, Easing, Ausdrücke, Rauschen | ✅ | 🟢 | eigenes Design |
| Patch-Op-Editor mit exaktem Rückgängig/Wiederholen | ✅ | 🟢 | JSON-Pointer im Stil von RFC 6902 |
| SVG-, PNG-, GIF-, HTML-Player-, Lottie-Export | ✅ | 🟢 | öffentliche Spezifikationen |
| Provenienzprotokoll, `stats` / semantisches `diff`, Golden-Frame-Regression (exakter Hash + Toleranz), Ressourcenobergrenzen, Mutations-Fuzz-Test, Versionssperre für das Dateiformat (R-001…R-005) | ✅ | 🟢 | eigenes Design |
| Eigenes Standard-Rollenvokabular + namensbasierter Tagger (`tag`) (R-006) | ✅ | 🟢 | eigene Namen, einfache Anatomiebegriffe |
| JSON-Schema-Export (R-003 schema) | ⬜ | 🟢 | eigenes Schema |

## Stufe 2 — Grundlagen des Authorings und offene Ausgaben (Tage)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| APNG, Sprite-Sheet PNG+JSON | ✅ | 🟢 | PNG-/APNG-Spezifikationen |
| WebM VP9 / AV1 mit Alpha (über dein ffmpeg) | ✅ | 🟢 | offen spezifizierte Codecs; Patentlage nicht geprüft |
| OCA-Export (Open Cel Animation) | ✅ | 🟢 | öffentliche Spezifikation; Import ⬜ |
| OpenRaster-Import (.ora) | ✅ | 🟢 | offene Spezifikation, Zip-Bomb-Limits |
| MP4 / H.264 | ✅ | 🟠 | nur über das *eigene* ffmpeg des Nutzers; niemals einen Encoder mitliefern |
| Key verschieben/skalieren/umkehren/kopieren/einfügen, Batch-Easing, Zeitversatz pro Kanal, Marker & Regionen, SMPTE-Timecode (R-010, R-011, R-044) | ✅ | 🟢 | allgemeines Editieren |
| Revisions-Snapshots mit Bereinigung und Absturzwiederherstellung (R-014); Layer-Comps (R-015); Timeline-Events mit Payloads (R-045); Umhängen (Reparent) unter Beibehaltung der Ruhepose (Teil von R-013) | ✅ | 🟢 | allgemeines Editieren |
| Ausrichten/Verteilen, Referenz-Ebene, Shy-/Solo-/Lock-Tags (R-012, R-013, R-016) | ✅ | 🟢 | allgemeines Editieren |
| Onion Skin, Bewegungsspuren (Overlays werden nie exportiert) (R-020) | ✅ | 🟢 | eigenes Design |
| Nachbearbeitungskette: Weichzeichner, Schlagschatten, Glühen, Umriss, Posterisierung, Verpixelung, Levels/Gamma, Ausblenden (Teil von R-022) | ✅ | 🟢 | Lehrbuch-Bildverarbeitung |
| Trim Paths, Filter pro Ebene, Masken, Tiefenunschärfe (R-021, R-024, R-025, R-034) | ✅ (eingebackene Trim Paths; Grenzen der Ebeneneffekte: siehe ROADMAP §9) | 🟢 | übliche Bildmathematik |
| SVG-Import (Pfade, Formen, Gruppen, Transformationen, einfarbige Füllungen; Verläufe/Text/Clip werden als Warnungen gemeldet) (R-032) | ✅ | 🟢 | W3C-SVG-1.1-Grammatik |
| BVH-Import (eine Rotationsachse → Knochenwinkel, begrenzte Key-Reduktion) (R-036) | ✅ | 🟢 | offenes Textformat |
| dotLottie-Container-Export (R-03A) | ✅ | 🟢 | öffentliche Container-Spezifikation (Zip + Manifest) |
| Animierter SVG-Export, Bildsequenz-Import, OpenEXR-/16-Bit-Ausgabe, glTF-Export (R-032, R-037…R-039) | 🟡 (glTF: Formen von Frame 0 plus Knochen und Morph-Gewichte, siehe ROADMAP R-039) | 🟢 | öffentliche Formate |
| Import von Papagayo- (Moho-Switch) und Rhubarb-Mundsignalen (R-03B) | ✅ | 🟢 | nur deren Klartextausgabe lesen |

## Stufe 3 — Rig- und Animationstiefe (Wochen)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| Federn (eingebacken), Lattices (FFD), Spine-Deformer, Switch-Ebenen, Partikel, Flüssigkeit, Booleans | ✅ | 🟢 | Lehrbuch |
| Audio → Mundform-Lippensynchronisation, Walk-Cycle-Generator, Spiegeln, Retime, Clip, Crossfade | ✅ | 🟢 | eigenes Design |
| **Pin Warp** (ersetzt Cage-/ARAP-artiges Warping) | ✅ | 🟢 | Wendland 1995 RBF |
| **Auto-Skin-Gewichte per Wärmediffusion** | ✅ | 🟢 | Laplace-Gleichung |
| **Performance-Aufzeichnung** mit begrenzter Key-Reduktion | ✅ | 🟢 | Douglas–Peucker |
| **Input Mapping** (CSV-/JSON-Signale → Parameter) | ✅ | 🟢 | kein Tracker mitgeliefert |
| Soft-/Stretch-IK (R-051), Pose-Driver: Slider + 2D-Joystick (R-050), winkelgesteuerter Ansichtswechsel (R-056), Tag-Auto-Rig (R-058) | ✅ | 🟢 | allgemeines Rigging; Lehrbuchmathematik, eigenes Design |
| Pfad-Constraints, Foot-Lock-IK, Parent-Switch-Keys, Skins, Magneten (R-040…R-049, R-052…R-055, R-057) | 🟡 (gebaut: R-040…R-043, R-044, R-047, R-048, R-052…R-055, R-057; die als NOT done markierten Teile siehe ROADMAP Abschnitt 9) | 🟢 | allgemeines Rigging |
| Idle-Generatoren (Blinzeln, Atmen, Schwanken) und Follow-through-Ketten (R-047, R-048) | ✅ | 🟢 | mit Seed, deterministisch |
| Wind, Schwerkraft, Kollisionen, Haar-/Stoffstreifen (R-070…R-072) | 🟡 (R-070, R-071 fertig; R-072-Streifen bestehen nur aus Knochen, der Stoff ist eine ebene Fläche) | 🟢 | Masse-Feder-/PBD-Lehrbücher |

## Stufe 4 — Smarte Werkzeuge (Wochen bis Monate)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| **Bild → Vektor-Trace** (ersetzt ML-Auto-Trace) | ✅ | 🟢 | Marching Squares + RDP |
| **Zustandsautomat auf Basis eines Event-Logs** + Bake in Kanäle (ersetzt Zustandsautomaten-Klone auf Runtime-Ebene) | ✅ | 🟢 | eigene Semantik — unsere Namen beibehalten |
| **Retargeting per Effector-Reichweite** | ✅ | 🟢 | eigene Methode |
| **Pose-Match-Solver** (Evolutionsstrategie, berücksichtigt Gelenklimits) | ✅ | 🟢 | Beyer & Schwefel |
| **WCAG-2.3.1-Prüfung auf Flackern / Rotflackern + Kamerabewegungs-Bericht + Reduced-Motion-Zwilling** | ✅ | 🟠 | nur öffentliche Definitionen; „technisches Hilfsmittel" sagen, nie „zertifiziert" |
| **Rig-Linter**, **Motion Contracts**, **Seamless-Loop-Solver**, **Principles Coach** (I-02, I-04, I-05, I-14) | ✅ | 🟢 | eigene Ideen |
| **Draw-the-overshoot-Federanpassung** (I-08) — bestimmt Frequenz/Dämpfung mit dem Integrator der Engine selbst | ✅ | 🟢 | eigene Idee |
| **Explain-this-pixel** für Knochen und Ebenen (I-07) | ✅ | 🟢 | eigene Idee |
| Before | after | Differenz-Review-Bild (erster Schritt von I-06; die Wahrnehmungsmetrik ist `ci diff`, SSIM und CIEDE2000) | ✅ | 🟢 | eigene Idee |
| Normal-Map-Beleuchtung, Parallaxe, Tiefenunschärfe (R-064, R-065, R-024) | ✅ | 🟢 | Lehrbuch |
| Transkript-ausgerichtete Lippensynchronisation (R-081) | ✅ (eigener Aligner, `speech.rs`) | 🟠 | einen permissiv lizenzierten Aligner verwenden; dessen Modelllizenz prüfen |
| Webcam-Gesichts-/Körper-Tracking als Eingabe für den Input Mapper (R-082) | 🟡 (eigener Detektor, nur auf synthetischen Gesichtern trainiert; echte Webcams und Fotografien ungetestet) | 🟠 | der Nutzer bringt einen permissiv lizenzierten Tracker mit; keine Modellgewichte mitliefern, sofern die Lizenz es nicht erlaubt |
| Vollständiges BVH-Mocap-Retargeting mit Auto-Zuordnung über die Hierarchie (R-083) | ✅ | 🟢 | baut auf dem BVH-Import auf |
| PSD-Import (R-035) | ⬜ | 🟠 | nur lesend, veröffentlichte Spezifikation, keine Adobe-Marken; oder einfach ORA verwenden |

## Stufe 5 — Plattform (Monate)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| Browser-Studio mit Rückgängig/Wiederholen | 🟡 | 🟢 | eigene UI; keine Kopie von Layouts oder Namen anderer Hersteller |
| WASM-Laufzeitumgebung, stabile Rust-API + C-ABI, Scripting in der Sandbox (R-090…R-093) | 🟡 (R-091, R-092, R-093 fertig; R-090: das WASM-Modul wird in CI gebaut und besteht den Test des Wrappers unter Node, und wurde nur in Headless-Chromium ausgeführt) | 🟢 | eigene API |
| **Agent-/MCP-Server** über Patch-Ops: inspizieren, bearbeiten (atomar, rückgängig machbar), linten, Sicherheit, rendern, nur in die eigene Datei speichern (R-094) | ✅ | 🟢 | offenes JSON-RPC-Protokoll |
| Vollständige GUI: Gizmos, Timeline, Kurveneditor (R-100…R-103) | 🟡 (R-101, R-102 fertig; R-100 und R-103: die Seiten existieren, wurden vom Autor aber nie in einem echten Browser ausgeführt) | 🟢 | allgemeine Widgets; eigene Optik |
| Versionskontrolle für Animationen mit semantischem Merge, perzeptuellem CI-Diff, Explain-this-pixel (I-01, I-06, I-07) | ✅ (I-01 `osatura vcs`, I-06 `osatura ci`, I-07 `explain`) | 🟢 | eigene Ideen |
| Pose-Fuzzing mit Stress-Heatmaps, größenzielgerichteter Export, deterministischer Detailgrad (Level of Detail) mit deklarierter Fehlerschranke (I-03, I-10, I-11) | 🟡 (alle drei gebaut; die Schranke von I-11 deckt keine Effekte, Skinned Paths, IK-gesteuerten Werte oder Zustandsautomaten-Posen ab) | 🟢 | eigene Ideen, Lehrbuchmathematik |

## Stufe 6 — Frontier (Forschungsniveau)
| Feature | Status | IP | Unbedenkliche Grundlage / Bedingung |
|---|---|---|---|
| GPU-Renderer mit CPU-Paritätstests (R-110) | 🟡 (Crate `osatura-gpu` gebaut und getestet; Hardware-Läufe begrenzt, siehe ROADMAP) | 🟢 | wgpu (permissiv) |
| Echtzeit-Mehrbenutzerbearbeitung über das Op-Log (R-111) | 🟡 (fertig mit optionalem persistiertem Sitzungslog `--collab-log`; Teilnehmer und Undo-Verlauf werden nicht persistiert; einfaches HTTP, gemeinsames Token) | 🟢 | CRDT-Literatur |
| Data-Binding-/Layout-Runtime (R-091, R-095) | ✅ (eigenes Design; Patentprüfung nicht erfolgt, docs/PATENT_NOTES.md) | 🟠 | vor der Auslieferung auf Patente prüfen |
| ML-gestützte Werkzeuge (R-112) | 🟡 | 🟠 | nur klassische Methoden, keine Modellgewichte: `autotrace` (k-Means, Schneider-Bézier-Fit, Zhang-Suen-Linienmodus) und aus den eigenen Posen gelernte Vorschläge (PCA, Kernel, optional ein kleines MLP in einem eigenen Gewichtsformat); kein Deep Learning |

## Niemals bauen (🔴)
- Native Projektdateien von **Live2D, Spine, Moho, Harmony, Rive, Cartoon Animator** lesen oder schreiben (proprietär; Risiko durch EULA/Reverse Engineering) — stattdessen Lottie, OCA, ORA, APNG, SVG verwenden.
- **ARAP-artige schnelle Solver zur Verformung von Artwork**, wie in Adobes Patenten beschrieben — Pin Warp, FFD und Skinning decken den Anwendungsfall ab.
- Das **UI-Layout, die Icons, Namen, Beispielinhalte oder Magic Numbers/Terminologie von Dateiformaten** eines Herstellers kopieren.
- Das **Harding**-Werkzeug für Photosensitivität nachahmen oder Rundfunk-Konformität behaupten.
- **H.264-/H.265-/AAC-Encoder** mitliefern.

## Bedingungen, die für jede 🟢-Zeile gelten
Provenienzprotokoll wird geführt (`CLEANROOM.md`; unabhängige Implementierung, kein strenger Clean Room); nur permissive Abhängigkeiten (neu erzeugen mit `scripts/licence-audit.sh`; derzeit 363 Fremd-Crates über alle Plattformen, alle der Klasse MIT / Apache-2.0 / BSD / Zlib / Unlicense bis auf sechs MPL-2.0-Crates (smartstring über das Feature `scripting`, das standardmäßig aktiv ist, und fünf über die Fenster-App `osatura-gui`); siehe docs/THIRD_PARTY.md); keine fremden Grafiken oder Audiodateien ohne eigene Lizenzdatei ausgeliefert; der Name **OSATURA** vor dem Launch in einer Markenrecherche freigegeben.


# Detaillierte Fähigkeitstabellen
## Zeichnen & Formen
| Feature | Status | Hinweise |
|---|---|---|
| Kubische Bézier-Pfade, offen/geschlossen, Handles | ✅ | |
| Füllung: einfarbig, linearer & radialer Verlauf | ✅ | Verläufe folgen der Ebenentransformation |
| Kontur: Breite, Caps, Joins, Strichelungen | ✅ | |
| Kontur mit variabler Breite (Breitenprofil) | ✅ | Polygonumriss; geschlossene Pfade → zwei Ringe |
| Catmull-Rom-Glättung, Hilfsfunktionen für Rechteck/Kreis/Ellipse/Polygon | ✅ | |
| Polygon-Booleans (Vereinigung/Schnitt/Differenz/XOR) | ✅ | `boolean`-Ebene; Löcher werden zu einem Polygon verbunden; das Ergebnis ist polygonal (abgeflacht) |
| Metaball-/Liquid-Ebenen | ✅ | Marching Squares, Knochen können Blobs steuern |
| Bildebenen (PNG), eingebettet oder als Datei | ✅ | |
| Bitmap-/Vektorpinsel, Texturfüllungen, Text/Schriften | ❌ | |
| Freihandzeichnen, Punktbearbeitungswerkzeuge, Bild-Trace | ❌ | keine Zeichen-GUI |

## Rigging
| Feature | Status | Hinweise |
|---|---|---|
| Knochenhierarchie, FK, Ruhepose | ✅ | |
| Winkel-/Skalierungslimits | ✅ | |
| IK: analytisch mit 2 Knochen (Beugung ±) | ✅ | |
| IK: N-Knochen-Ketten (FABRIK) | ✅ | |
| FK↔IK-Mix (animierbar) | ✅ | |
| Constraints: Aim, Copy-Rotation, Pin (Gewicht animierbar) | ✅ | |
| Linear-Blend-Skinning von Ankern + Handles, Auto-Skin | ✅ | |
| An Knochen gebundene Ebenen, Ebenen-Parenting / Gruppen | ✅ | |
| Sekundärbewegung: eingebackene gedämpfte Federn | ✅ | deterministisch, wahlfreier Zugriff (beim Laden eingebacken) |
| Posengesteuerte Kanäle („Driven Keys") | 🟡 | eine Kurve bildet jede Eigenschaft auf eine andere ab; keine dedizierte Smart-Bone-UI/-Actions |
| Starrkörperphysik, Stoff, Weichkörper | ❌ | |

## Deformation
| Feature | Status | Hinweise |
|---|---|---|
| Offsets pro Punkt (animierbar) | ✅ | adressiert als `layer::path::index` |
| Morph Targets / Blend Shapes (additive Gewichte) | ✅ | |
| Freiform-Lattice (Bernstein-FFD) | ✅ | animierbare Kontrollpunkte |
| Spine-/Kurven-Biege-Deformer | ✅ | optionaler weicher Radius |
| „Boil"-Wackeln (handgezeichnetes Zittern) | ✅ | kohärentes Rauschen, deterministisch |
| Bild-Mesh-Warp mit Knochengewichten | ✅ | Gitter-Mesh; kein Mesh-Editor |

## Animation
| Feature | Status | Hinweise |
|---|---|---|
| Keyframes + Easing (linear/step/ease/cubic-bezier) | ✅ | |
| Extrapolation: hold/linear/loop/ping-pong/loop-offset | ✅ | |
| Ausdruckskanäle | ✅ | kleine, sichere Ausdruckssprache; kann andere Kanäle lesen |
| Rauschkanäle, Blend-Modi add/multiply/replace, Gewichte | ✅ | |
| Switch-Ebenen + Exposure (Einzelbildanimation, Mundwechsel) | ✅ | |
| Clip-Retime/-Offset, Retarget, Spiegeln L↔R, Crossfade-Bake | ✅ | Funktionen + CLI `mirror`; keine NLA-UI |
| Prozeduraler Walk Cycle | ✅ | einfacher sinusförmiger Generator, kein professionelles Walk-Werkzeug |
| Audio-Lippensynchronisation aus WAV | 🟡 | Heuristik aus Lautstärke/Helligkeit → 5 Mundformen, **keine** Phonemerkennung |
| Deterministischer Partikelemitter | ✅ | geschlossene Form: jeder Frame wird direkt berechnet |
| Onion Skin, Bewegungsspuren | ✅ | CLI `onion` und `trail`, Studio-Overlays; nie exportiert |
| Audiowiedergabe / Wellenform-Scrubbing | 🟡 | Befehl `waveform` und Audiowiedergabe im Studio; keine Aussage zu einer Wellenform-Scrubbing-UI |

## Kamera & Compositing
| Feature | Status | Hinweise |
|---|---|---|
| 2D-Kamera Schwenken/Zoomen/Drehen (animierbar) | ✅ | |
| Multiplane-Parallaxe pro Ebene der obersten Ebene | ✅ | |
| Deckkraft, Blend-Modi, Alpha-Masken (+Invertieren) | ✅ | |
| Echte 3D-Ebenen / -Kamera, Stereo | ❌ | |

## Ausgabe & Austausch
| Feature | Status | Hinweise |
|---|---|---|
| SVG-Frames | ✅ | |
| PNG-Frames (tiny-skia-Raster) | ✅ | |
| Animiertes GIF | ✅ | |
| MP4 | ✅ | erfordert `ffmpeg` im PATH |
| Eigenständiger HTML-Player | ✅ | alle Frames inline, Scrubber |
| Lottie-JSON | 🟡 | eingebackene Geometrie pro Frame; Verläufe → erste Farbe; keine Masken/Blend-Modi/Bilder; **nicht in einem echten Lottie-Player validiert** |
| Import von Dateien anderer Werkzeuge, PSD, FBX usw. | ❌ | (bewusst: keine proprietären Formate) |
| Scripting-/Plugin-API | 🟡 | `script` mit Rhai in der Sandbox über validierte Anfragen, plus Ausdrücke; keine Plugin-API |

## Bearbeitung
| Feature | Status | Hinweise |
|---|---|---|
| JSON-Patch-Editiermodell mit Rückgängig/Wiederholen | ✅ | jede Bearbeitung wird validiert; ungültige Bearbeitungen werden zurückgerollt |
| Hilfsfunktionen zum Setzen/Löschen von Keyframes | ✅ | |
| Web-Studio (Browser) | 🟡 | Vorschau, Scrubber, Keying des Winkels pro Knochen, JSON-Editor, Rückgängig/Wiederholen, Speichern, Exporte. **Kein Zeichnen, keine Punkt-/Knochenbearbeitung auf der Canvas, kein Kurveneditor.** |

## Bescheidene „Verbesserungen" gegenüber der Lehrbuch-Basis
(es wird nicht behauptet, dass sie anderswo fehlen)
- Jede Projektbearbeitung ist ein validierter, umkehrbarer JSON-Patch → Rückgängig kann ein Projekt nie beschädigen.
- Alle Simulationen (Federn, Partikel) sind deterministisch mit wahlfreiem Zugriff: Beliebige Frames lassen sich in beliebiger Reihenfolge scrubben oder rendern und liefern identische Pixel.
- Jede Eigenschaft — auch Kontrollpunkte von Deformern und Morph-Gewichte — ist über einen einzigen Pfad `layer::sub::id` adressierbar, sodass Ausdrücke und gesteuerte Kurven alles mit allem verbinden können.
- Die Validierung meldet *alle* Probleme in einem Durchlauf mit den betroffenen Namen.
- Eine Engine → SVG, Raster, GIF, MP4, HTML, Lottie; dasselbe `evaluate(frame)` speist alle.

## Tests
`cargo test` — 142 Unit-/Integrationstests (Mathematik, Tracks, Ausdrücke, Rig-Solver, Deformer,
Render-Pixel, Exporter, Editor-Rückgängig/Wiederholen, Audio, Partikel, Booleans, Metaballs).

---

# Checkliste gegen das Referenzprodukt, Version für Version

Referenz: Moho (früher Anime Studio), 1999 → 14.5 „SHOWTIME" (12. Okt. 2026). Die Feature-Listen stammen
von Release-Seiten des Herstellers und aus der Fachpresse (siehe Quellen). Lücken in der Dokumentation: Zu Anime Studio 6–8,
12.5 und 14.1 konnte ich keine detaillierte öffentliche Liste finden, sie sind daher nur über die
allgemeine Historie abgedeckt. Der Status bezieht sich auf OSATURA: ✅ Äquivalent vorhanden · 🟡 teilweise / anderer Ansatz · ❌ fehlt.

## Frühe Versionen (1999 – Anime Studio 8, nur allgemeine Historie)
| Feature | OSATURA |
|---|---|
| Partikel | ✅ deterministische Emitter-Ebene |
| Ebenenmasken | ✅ Alpha-Masken + Invertieren |
| Onion Skins | ✅ (CLI `onion`, Studio-Overlay) |
| Knochen-Constraints | ✅ Aim / Copy-Rotation / Pin, Winkel- & Skalierungslimits |
| Lua-Scripting | 🟡 (Rhai, nicht Lua; zusätzlich Ausdruckskanäle) |
| Lip-Sync | 🟡 Amplitudenheuristik aus WAV |
| Physik | 🟡 Federn + Partikelschwerkraft; keine Starrkörper |
| 3D-Erstellung / -Import | ❌ |
| Photoshop-Import | ❌ |
| Wacom-Unterstützung | ❌ (keine Zeichen-GUI) |
| Smart Bones, IK | 🟡 gesteuerte Kanäle / ✅ 2-Knochen + FABRIK |

## Anime Studio 9 (2013)
| Feature | OSATURA |
|---|---|
| 64 Bit, GPU-Beschleunigung | 🟡 natives 64-Bit-Rust; nur CPU-Rasterizer (keine GPU) |
| Smart Bones (Drehregler zur Gelenkbiegung) | 🟡 gesteuerte Kanäle bilden jeden Wert auf jeden anderen ab; keine Regler-UI/-Actions |
| Editierbare Bewegungsgraphen | ❌ (Kurven sind Daten mit Cubic-Bézier-Easing; kein Kurveneditor) |
| Überarbeitete Timeline | ❌ (das Studio hat nur einen Scrubber + Key-Markierungen) |
| Wacom-Multitouch | ❌ |

## Anime Studio 10 (2014)
| Feature | OSATURA |
|---|---|
| Unabhängige (globale) Winkel-Constraints | 🟡 Copy-Rotation auf einen Steuerknochen nähert es an |
| Squash & Stretch | 🟡 die Knochenskalierung ist animierbar; kein automatisches Squash/Stretch |
| Smart-Bone-Regler / -Einrichtung | 🟡 siehe oben |
| Bounce-/Elastic-/Stagger-Interpolation | 🟡 Bounce, Spring (elastikähnlich), Back, Cubic-Bézier ✅; Stagger ❌ |
| Farbeimer, Blob-Pinsel, Radierer, Punkte reduzieren | ❌ |
| Vorinstallierte Charaktere / Bibliotheksinhalte | ❌ |
| 3D-Objekt-Import | ❌ |

## Anime Studio 11 (2015)
| Feature | OSATURA |
|---|---|
| Einzelbildanimation | ✅ Switch-Ebenen + Exposure-Kanäle |
| Ebenenreferenzierung (verknüpfte Duplikate) | ❌ |
| Animierte Formreihenfolge | ❌ (die Zeichenreihenfolge ist pro Ebene fest; Switch-Ebenen können tauschen) |
| Editierbare Bewegungskurven | 🟡 Kurvendaten ja, keine Kurven-UI |
| PSD-Import (inkl. Live-Aktualisierung) | ❌ |
| FBX-/Unity-Export | ❌ |
| Audio importieren/aufnehmen + automatische Lippensynchronisation | 🟡 WAV importieren + heuristische Lippensynchronisation; keine Aufnahme |
| Konturen zusammenführen, Color-Points-Werkzeug | ❌ |

## Moho 12 (2016) und 12.x
| Feature | OSATURA |
|---|---|
| Bézier-Handles | ✅ |
| Verbesserte Freihandwerkzeuge | ❌ |
| Smart Warp (eigene Meshes) | 🟡 Bild-Gitter-Mesh mit Knochengewichten; Lattice-/Spine-Deformer für Vektoren; kein Mesh-Editor |
| Bewegungsunschärfe | ❌ |
| Pin-Knochen | 🟡 Pin-Constraint, keine Ein-Punkt-Pin-Knochen |
| Import/Export von Actions zwischen ähnlichen Rigs | 🟡 Funktionen `clip`, `retarget`, `mirror` auf Kanalmengen; keine Action-Bibliotheks-UI |
| Mehrere Ebenen gleichzeitig animieren | ✅ beliebig viele Kanäle pro Timeline |
| Neu gestaltete UI, Knochenverankerung, automatisches Einfrieren der Position | ❌ |

## Moho 13 (2019)
| Feature | OSATURA |
|---|---|
| Bitmap-Einzelbildanimation + Onion Skin | 🟡 PNG-Bildebenen innerhalb von Switch-Ebenen; kein Onion Skin, kein Bitmap-Zeichnen |
| Bitmap-Zeichenwerkzeuge / eigene Pinsel | ❌ |
| Storyboard- & Animatic-Werkzeuge | ❌ |
| Actions-Fenster | ❌ (nur Clips auf Datenebene) |
| 3D-Lichter, Materialien, Toon-Shading | ❌ |
| Mehr Ausgabe-Presets | 🟡 CLI-Exportformate, kein Preset-Manager |
| Mitgelieferte Charaktere | ❌ |

## Moho 13.5 (2021) und 13.5.x
| Feature | OSATURA |
|---|---|
| Vitruvian Bones (Zeichnungen *und* Knochensets austauschen) | 🟡 Switch-Ebenen tauschen Zeichnungen; kein Austausch von Knochensets |
| Quad-Meshes (Perspektive) | 🟡 Gitter-Mesh + Lattice ergeben Freiform-Warps; kein projektives Quad |
| Winddynamik (Stärke, Richtung, Turbulenz) | ❌ (Federn haben keinen Wind; Rauschkanäle können ihn vortäuschen) |
| Relatives Keyframing | ❌ |
| Zufällig gewählte Viseme-Varianten bei der Lippensynchronisation | ❌ (das Mapping kann Formen zusammenführen, keine zufälligen Varianten) |
| Audio als WAV exportieren | ❌ |
| Deep-Keyframe-Kopieren/-Einfügen, Pivot-Anpassung, Timeline-Marker | ❌ (keine GUI) |
| Floating-Lizenzen | n/a |

## Moho 14 (2023)
| Feature | OSATURA |
|---|---|
| Neue Grafik-Engine, Echtzeit-Effekte (Weichzeichner, Schatten, Rauschen) | ❌ keine Filter für Weichzeichner/Schatten/Rauschen; Blend-Modi ✅ |
| Liquid Shapes (zusammenführen/subtrahieren/schneiden/überblenden) | 🟡 Metaball-Liquid ✅ + Polygon-Booleans ✅; nicht dasselbe Formmodell |
| Curvers / Compressible Curvers | 🟡 der Spine-Biege-Deformer ist das nächste Analogon |
| Verbesserter SVG-Import | ❌ (nur SVG-**Export**) |
| Freihand-Verbesserungen, neue Stile, Formen zusammenführen | ❌ / ❌ / 🟡 (Booleans) |
| Multi-Stroke Fill, Connect & Create | ❌ |
| Smart Line Boil, Pinseloptionen | 🟡 „Boil"-Wackeln mit einstellbarer Rate ✅; Pinseloptionen ❌ |
| Punktfarben, verbessertes Add Point | ❌ |
| Dynamic Bones (+ Schwerkraft) | 🟡 Knochen mit gedämpfter Feder ✅; kein Schwerkraftterm |
| Verzögerte Constraints | 🟡 Federn erzeugen ein Nachlaufen hinter einem Ziel; kein expliziter Delay-Constraint |
| Neue Timeline-Optionen | ❌ |
| Verbesserungen bei der Einzelbildanimation | 🟡 |
| Update der Scripting-Schnittstelle (Lua 5.4) | ❌ |

## Moho 14.1 – 14.2 (2023-11, 2024-06)
| Feature | OSATURA |
|---|---|
| (14.1: keine öffentlichen Details gefunden) | — |
| Animierter Offset des Kurvenprofils | ❌ (Breitenprofile sind statisch) |
| Smart Boil für Vektorrauschen | 🟡 das Wackeln taktet mit einer festen Rate; kein Modus „nur bei Bewegung" |
| Überarbeitetes Transform-Bone-Werkzeug | ❌ (keine Canvas-Werkzeuge) |
| Pixelierungsanzeige | ❌ |
| Blend-Modus Colour Burn | ✅ |
| Geringerer minimaler Pinselabstand, PSD-Clipped-Layer | ❌ |
| Timeline-Nummerierung auf Basis der Framerate | ❌ |
| Anzeigeoption „Maskierte Ebenen ausschalten" | ❌ |
| Bessere Lippensynchronisation bei niedrigen Framerates | 🟡 |
| Mehrsprachige Namen für Knochen/Ebenen/Marker | ✅ (UTF-8-Namen) |
| Antialiasing der Exclude-Strokes-Maske, farbige Onion Skins auf Bildern | ❌ |
| Smart Bones, die die Knochendynamik steuern | ❌ (Feder-Einstellungen sind nicht animierbar) |
| Lua-Skripte beim Start; Windows-Konsolen-Flag | ❌ / n/a |
| ProRes-Alpha-Export | ❌ (nur GIF/PNG/MP4) |

## Moho 14.3 (2024-12)
| Feature | OSATURA |
|---|---|
| Pinseldeckkraft pro Punkt (druckgesteuert) | ❌ |
| Farbdrift durch Stiftdruck | ❌ |
| Steuerung von Freihand-/Linienbreiten-Werkzeugen | ❌ |

## Moho 14.4 „SIDE QUEST" (2025-11)
| Feature | OSATURA |
|---|---|
| Export nach Unreal / Unity / Godot | ❌ |
| glTF-Export nach Blender / in 3D-Werkzeuge | 🟡 (`.glb`: Formen von Frame 0, Knochen, Morph-Gewichte) |
| 360°-Smart-Bones | ❌ |
| Stil aus Form erstellen | ❌ |
| Partikeloptionen (Replizieren entlang einer Kurve, Verzögerung, geordnete Emission, Verteilung) | 🟡 Startverzögerung ✅, deterministische geordnete Emission ✅; Replizieren entlang einer Kurve ❌ |
| Layer Selector, Magnet-Falloff, PSD-Import, Freihand-Glättung, Lua-Ergänzungen | ❌ |
| Einfachere Zyklenerstellung | ✅ Loop-Extrapolation + Walk-Cycle-Generator |

## Moho 14.5 „SHOWTIME" (2026-10-12)
| Feature | OSATURA |
|---|---|
| Live Playback (Webcam / Gamepad / Mikrofon, virtuelle Webcam, OBS) | ❌ |
| Gesichts-Tracking → Knochen / Smart Bones / Mesh | 🟡 (synthetisch trainierter Detektor, `face-bake`; nicht an echten Gesichtern validiert) |
| Live-Aufnahme in die Timeline, Steuerung mehrerer Charaktere | ❌ |
| Video-Tracking (Bildsequenzen) | ❌ |
| Freihand-Algorithmen, gerade Linien, Enden kürzen | ❌ |
| Neu gebauter Farbeimer (Vector / Trace / Recolor) | ❌ |
| Magnet mit umgekehrter Bewegung, Select Points per Formauswahl | ❌ |
| Eigene Pinselkategorien | ❌ |
| Farbe/Deckkraft der Onion Skins, Shift & Trace, magnetische Onion Skins | ❌ |
| Filter / Mehrfachauswahl im Ebenenfenster | ❌ |
| Performance: schnelleres Rückgängig/Wiederholen, schlankere Dateien | 🟡 Rückgängig ist eine günstige JSON-Patch-Inverse; Dateien sind schlichtes JSON |
| MP4-/H.264-Export | ✅ über ffmpeg |
| PNG-Farbprofile, EXR-Import/-Export | ❌ |
| Batch-Exporter, CLI-Flags `-exr` / `-pngprofile` | 🟡 CLI `frames` / `export`, keine Batch-Warteschlange |
| Framerate mit 3 Nachkommastellen (z. B. 23.976) | ✅ |
| SVG-Export behandelt Konturen mit variabler Breite & Liquid Shapes | ✅ (Konturen werden zu Umrissen, Liquids zu Pfaden) |
| Liquid Hide-Edge, reine Kontur-Liquid-Shapes, Stroke Exposure | ❌ |
| Linux über Wine/Proton | ✅ natives Rust (Linux/macOS/Windows); fertige Archive für Windows x86-64, macOS arm64 und Linux x86-64 in der Entwurfs-Vorabversion; von CI auf allen dreien getestet, von einer Person nur unter Linux |

## Zusammenfassung (nur Referenzprodukt; Konkurrenten folgen unten)
Von den dokumentierten Funktionen des Referenzprodukts deckt OSATURA den Kern aus **Rigging, Deformation,
prozeduraler Animation, Compositing-Mathematik und Export** ab (grob: Skelett/IK/Constraints,
Mesh- & Lattice-Deformation, Keyframes & Ausdrücke, Masken/Blends/Verläufe, Partikel,
Metaballs/Booleans, GIF/MP4/HTML/Lottie), hat aber **keine Zeichenwerkzeuge, keine Live-Eingabe, kein 3D,
keine PSD-/FBX-I/O** (zum Zeitpunkt der ersten Erhebung; glTF-Export, EXR-Ausgabe, Rhai-Scripting, ein Kurven-/Timeline-Studio, Onion Skin und Weichzeichner-Effekte kamen seither hinzu, siehe `ROADMAP.md` Abschnitt 9).
Das sind die großen, überwiegend GUI-gebundenen Blöcke des Produkts und die ehrliche Lücke.

## Quellen
- Moho-14.5-SHOWTIME-Ankündigung, moho.lostmarble.com (Blog, 12. Okt. 2026)
- Moho-Pro-Funktionsseite, mohoanimation.com/pages/features
- Moho-14.4-SIDE-QUEST-Appcast, lostmarble.com/appcasts/Moho14.4Update.html
- Moho 14.3: Digital Production, 27. Dez. 2024
- Moho-14.2-Release-Post, moho.lostmarble.com; Tutorial-Liste „Moho 14: What is new?"
- Neue Funktionen in Moho Pro 13.5, moho.lostmarble.com; Lester Banks (Apr. 2021); Notizen zu 13.5.3
- Moho 13: Lester Banks (Jun. 2019)
- Moho Pro 12: 80.lv; Digital Production (Nov. 2016)
- Anime Studio 11: Architosh (Mai 2015); 10: Architosh (März 2014); 9: Renderosity
- Versionsdaten: mohoscripting.com/moho_versions; allgemeine Historie: Wikipedia „Moho (software)"

---

# Nachtrag: Geschichte von Moho / Anime Studio, Versionen 1 – 11 (ausgefüllt)

Quelle: das vom Hersteller gehostete PDF mit der Versionshistorie (lostmarble.com, „Anime Studio history", 2015) sowie die
Pressseiten zu Anime Studio 7 und 8. Es ersetzt die Zeilen „nur allgemeine Historie" weiter oben in der
Referenz-Checkliste. Auch hier nicht gefunden: Details zu 8.x, 10.1.x, 11.1, 12.5, 14.1.

| Version | Dokumentierte Funktionen | OSATURA |
|---|---|---|
| Moho 1 (1999) | BeOS, Windows-Portierung, Flash-Ausgabe | ❌ Flash/SWF (veraltet) |
| 2.x | QuickTime-I/O, Rendering bis 4096², Ebenen-Shading, Switch-Ebenen, Farbfelder in Stilen, flexible Knochenbindung, verbesserter Illustrator-Import | ✅ Switch-Ebenen, 🟡 Gewichtsbindung, ❌ Illustrator-Import, ✅ große Raster-Skalierungen (≤16×) |
| 3.x | Partikel, Bild-Wrapping, Ebenenmasken, Onion Skins, virtuelle Kamera mit Tiefe, Bild-Warping, erweiterte Switch-Ebenen, Tablet-Unterstützung, SMPTE-Timecode, Artwork außerhalb des Bildes ausblenden, umkehrbare Masken, Zufallsgenerator für Linienstärke, Safe Zones, Ebenen-Shear | ✅ Partikel, Masken (+Invertieren), Kamera, Bild-Mesh-Warp, Ebenen-Skew; ❌ Onion Skin, Tablet, Timecode, Safe Zones, Stärken-Zufallsgenerator |
| 4.x | Grafische Animationskurven, Knochen-Constraints, Batch-Rendering, Audio, animierbare Parameter, Animationszyklen, Keyframe-Skalierung, größenveränderliche Projekte, Partikeltiefe, zufällig erzeugte Menschenmengen, automatische flexible Bindung, Linux-Release | ✅ Constraints, Zyklen (Loop-Extrapolation), animierbare Parameter, größenveränderliche Projekte, `clip`-Retime ≈ Keyframe-Skalierung, Linux; 🟡 Menschenmengen (Partikel), 🟡 Audio (nur Lippensynchronisation); ❌ Kurveneditor-UI |
| 5.x | Neue UI, SWF-Export, 3D- und Notizebenen, Actions, Stile, Lua-Scripting | ❌ 3D- / Notizebenen / Lua; 🟡 Actions (Clip-Funktionen); ✅ Stile sind Farbauftrag pro Pfad |
| Anime Studio 6 | Motion Tracking, integrierte Lippensynchronisation, Scatter Brush, Lua 5.1, Auto-Shading, **Morph Targets**, Sequencer, HD-Video | ✅ Morph Targets; 🟡 Lippensynchronisation; ❌ Motion Tracking, Scatter Brush, Auto-Shading, Sequencer |
| 6.1 / 6.2 | Facebook-Upload, Import transparenter GIFs | ❌ / 🟡 GIF-Export ✅, Import ❌ |
| Anime Studio 7 | Simulierte Physik (Schwerkraft, Dichte, Reibung, Elastizität), 3D-Formdesign, Audioaufnahme, Inhaltsbibliothek, Poser-Szenen-Import, **Follow-Path-Werkzeug**, Gather Media | 🟡 Federn/Partikelschwerkraft; ❌ Starrkörperphysik, 3D, Aufnahme, Bibliothek, Poser, Follow-Path |
| Anime Studio 8 | Automatisches Bild-Tracing, Character Wizard, Beginner-Modus, Echtzeit-Medienverbindung, globale Rendering-Stile, fortgeschrittenes Knochen-Rigging | ❌ alles (kein Tracing, kein Wizard, keine Live-Verbindung, keine globalen Stile) |
| Anime Studio 9 / 9.5 | Editierbare Bewegungsgraphen, Smart Bones, **Layer Comps**, verbesserte Onion Skins, 64 Bit + GPU, erweitertes Keyframing, farbige Timeline; 9.5: Smart-Bone-Erweiterungen, Depth Shifting, verschachtelte Ebenensteuerung, Flexi-Binding, Verbesserungen bei Switch-Ebenen | siehe Tabelle AS 9; ❌ Layer Comps, Depth Shifting |
| Anime Studio 10 / 10.1 | Neue Constraints, Smart-Bone-Einrichtung, Bounce/Elastic/Stagger, separater Render-Prozess, kombinierte Knochen- und Punktwerkzeuge, Animationsvorschau, Multi-Dokument, Shortcut-Editor, Mehrfachformauswahl, Frame Zero, Punkte-/Knochen-Ausblenden, Mehrebenenbearbeitung, **Kurven mit variabler Breite**, Texturtransparenz, einstellbare Partikelquelle, Tiefenunschärfe, gebrochene Unschärfe; 10.1: automatisches Bild-Tracing, globale Stile, Patch-Ebenen, PSD-Import, Echtzeit-Medien, Character Wizard | ✅ Konturen mit variabler Breite, ✅ Mehrebenen-Animation; 🟡 Tiefenunschärfe ❌ Weichzeichner ❌; ❌ Patch-Ebenen, übrige wie oben |
| Anime Studio 11 | Einzelbildanimation, Ebenenreferenzierung, animierte Formreihenfolge, **animierte Knochenziele**, **animiertes Knochen-Parenting**, Knochen-Flipping, Gruppierung von Auswahlebenen, normalisierte Ebenenskalierung, Batch-Export, neues Dateiformat | ✅ FbF, ✅ animiertes IK-Ziel (das Ziel ist einfach ein Knochen), ❌ animiertes *Umhängen*, 🟡 Knochen-Flipping (`mirror`), ✅ Batch über CLI, ✅ Dateiformat aus schlichtem JSON |

**Hier aufgedeckte neue Lücken** (in die Roadmap aufnehmen): Layer Comps (gespeicherte Ebenen-Sichtbarkeitszustände),
animiertes Umhängen von Knochen, Patch-Ebenen, Depth Shifting, Follow-Path-Ebenen, Safe-Zone-Overlays,
SMPTE-Timecode, Auto-Shading, Scatter Brush, Bild-Auto-Trace.

---

# Funktionen der Konkurrenz, die die obige Checkliste nicht abdeckt

Untersuchte Produkte: Toon Boom Harmony 24, Spine 4.2, Live2D Cubism 5.x, Cartoon Animator 5.3, Rive,
Synfig Studio, OpenToonz, TVPaint Animation 12, Adobe Animate, Cavalry, Procreate Dreams, Duik (After
Effects), Blender Grease Pencil. Die folgenden Punkte sind jene, die in der Referenz-Checkliste **noch nicht
aufgeführt** sind. Die Qualität der Quellen schwankt: Die meisten Zeilen stammen von Feature-/Release-Seiten der Hersteller, die ich abgerufen habe;
mit † markierte Zeilen stammen aus allgemeinem Wissen oder nur aus einem Titel/Inhaltsverzeichnis und wurden nicht auf einer Seite bestätigt.
Der Status bezieht sich auf OSATURA: ✅ · 🟡 · ❌.

## Rigging, Constraints, Deformation
| Feature | Gesehen in | OSATURA |
|---|---|---|
| Gewichtswerkzeuge: Gewichte über Meshes hinweg verschweißen, Gewichte zwischen Vertices kopieren/einfügen | Spine 4.2 | ❌ (nur Auto-Skin, kein Malen) |
| Knochen-Physik-Constraints (Trägheit, Wind, Schwerkraft, Mix) | Spine 4.2 | 🟡 gedämpfte Federn (Frequenz/Dämpfung); kein Wind/keine Schwerkraft/Trägheit |
| Pfad-Constraint: Knochen folgen einem zusammengesetzten Bézier-Spline | Spine, Rive (Follow Path) | ❌ |
| Constraint-Mixes, Duplizieren mit Knochen, Multi-Edit, Ordner | Spine 4.2 | 🟡 IK-Mix & Constraint-Gewicht animierbar; keine Ordner/kein Multi-Edit |
| Skins (Attachment-Sets an einem Skelett austauschen) | Spine, Rive Solos | 🟡 Switch-Ebenen |
| Multi-Knochen-IK-„Pose-Werkzeug" (Ende einer Gliedmaße ziehen) | Spine | ❌ (benötigt Canvas-GUI) |
| Bounding-Box-/Hit-Test-Attachments | Spine | ❌ (für Offline-Ausgabe nicht nötig) |
| Free-Form Deformation, Mesh-Gewichte | Spine, Harmony | ✅ Lattice + gewichtetes Bild-Mesh |
| Deformer-Kette: Bone, Game Bone, Curve, Envelope, Free-Form, Mesh Warp | Harmony | 🟡 Bone ✅ Curve (Spine) ✅ Free-Form ✅ Mesh Warp ✅; Envelope/Cage ❌; Game Bone ❌ |
| Shape-Aware Deformation (Punkte, Knochen, Cages) | Harmony 24 | ❌ |
| Baking aller Deformationsarten | Harmony 24 | 🟡 Federn eingebacken; Geometrie pro Frame im Lottie-Export eingebacken |
| Master Controller / Transformationsauswahl für Deform-Gruppen | Harmony 24 | ❌ |
| Pegs und Ebenenhierarchie | Harmony, Animate | ✅ Gruppen, an Knochen gebundene Ebenen, Parenting |
| Auto-Muskel, automatisches Foot-Roll, kinematische Ausgabe † | Harmony | ❌ |
| Rotations-/Warp-Deformer + Keyform-Parameter, Glue | Live2D | 🟡 Parameter ≈ gesteuerte Kanäle, Warp ≈ Lattice; Glue ❌ |
| Blend Shapes, die Rotation, Deckkraft, Zeichenreihenfolge, Multiply-/Screen-Farbe steuern | Live2D 5.0 | 🟡 Morph-Gewichte steuern nur Geometrie |
| Auto-Mesh mit gleichmäßiger Dichtesteuerung für dünne Formen | Live2D 5.0 | ❌ (nur regelmäßiges Gitter) |
| Warp-Deformer passt seine Größe automatisch an, um untergeordnete Keyforms zu enthalten | Live2D 5.3 | ❌ |
| Deform-Pinsel (Quadrat-/Linien-/Erweiterungsformen) | Live2D 5.3 | ❌ |
| KI-gestützter Gesichts-Deformer / Generierung von Gesichtsbewegung | Live2D 5.0 | ❌ |
| Joysticks (2D-Steuerung, auf Animationen abgebildet) | Rive | 🟡 ein gesteuerter Kanal bildet eine Eingabe ab; kein Joystick mit 2 Eingaben |
| Constraints zwischen beliebigen Objekten (Verschieben/Drehen/Skalieren/Abstand) | Rive | 🟡 nur Aim, Copy-Rotation, Pin |
| Bitmap-Rigging mit automatischen Zwischenbildern | TVPaint 12 | 🟡 Bild-Mesh ✅; automatische Zwischenbilder ❌ |
| Plastic-Werkzeug (Mesh-basiertes Rigging) | OpenToonz | 🟡 Bild-Mesh + Gewichte |
| Auto-Rigging, Controller, Federdynamik | Duik † | 🟡 Federn ✅; Auto-Rig ❌ |

## Animation, Timeline, Interaktivität
| Feature | Gesehen in | OSATURA |
|---|---|---|
| Animation Mixing: gleichzeitige Animationen zur Laufzeit überblenden/crossfaden | Spine | 🟡 Add-/Multiply-Modi + Gewichte; eingebackenes `crossfade`; keine Übergänge zur Laufzeit |
| Dopesheet + Kurveneditor + Laufzeitvorschau | Spine, Rive, OpenToonz | ❌ (die Daten ermöglichen es; keine UI) |
| Zustandsautomat: Inputs, Listener, Übergänge mit Bedingungen, Blend States, Ebenen | Rive | ❌ |
| Events, Audio-Trigger, dynamische Assets | Rive | ❌ |
| Data Binding (Designelemente ↔ Live-Daten) | Rive | ❌ (Ausdrücke können Kanäle lesen, keine externen Daten) |
| Eingebettetes Scripting (Luau, JavaScript, ToonzScript) | Rive, Cavalry, OpenToonz | ❌ |
| Responsive Layouts / Komponenten / Bibliotheken | Rive | ❌ |
| Trim Paths, Vektor-Feathering, N-Slicing | Rive | ❌ |
| Auflösungsunabhängiges Timing, Vektor-Tweening | Synfig | ✅ |
| Parameterverknüpfung über Ebenen hinweg per Ausdruck | Synfig | ✅ |
| Prozeduraler Node-Graph (Generatoren, Transformer, Effektoren), Duplikatoren | Cavalry | ❌ (kein Node-Graph; kein Ebenen-Instancing) |
| Datengetriebene Animation aus JSON/CSV | Cavalry | ❌ |
| Performing: Live-Gesten als Keyframes aufzeichnen | Procreate Dreams | ❌ |
| Trigger-Hotkeys für die Puppet-Bühne, Motion Pilot (cursorgesteuerte Gehbewegungen) | Cartoon Animator 5.3 | ❌ |
| Camera Switcher: gespeicherte Kameras, Hotkey-Schnitte, Kamerawechsel-Spur | Cartoon Animator 5.3 | 🟡 eine Kamera; Schnitte lassen sich mit Step-Keys vortäuschen |
| Filmische Bewegungen (Dolly Zoom, Verfolgen eines Ziels) | Cartoon Animator 5.3 | 🟡 animierbare Kamera; kein Ziel-Tracking |
| Gesichts-Mocap (Webcam / iPhone), kopfgesteuerter Körper, Mikrofonlautstärke | Cartoon Animator, Live2D | ❌ (Mikrofonlautstärke nur über Offline-WAV) |
| Livestream-Modus (OBS/Twitch, Stream-Deck-Zuordnung) | Cartoon Animator 5.3 | ❌ |
| Motion-Sync-Lippensynchronisation, die Timeline-Keys erzeugt | Live2D 5.0 | 🟡 heuristisch WAV → Exposure-Keys |
| Rhubarb-Lippensynchronisations-Integration | OpenToonz | ❌ (nur Heuristik) |
| Root-Motion-Extraktion (Delta-Kompensation) | Spine 4.2 | 🟡 Loop-Offset-Extrapolation |
| Handle-Synchronisation bei Loop-Kurven | Spine 4.2 | ✅ Loop-Extrapolation (keine Handle-UI) |
| Morphing mit Morph-Hinweisen | Harmony | 🟡 Morph Targets + Pfad-Lerp; keine Hinweise |
| Key-basierte 2D-Kamera mit Timeline-Timing | TVPaint 12 | ✅ |

## Zeichnen, Malen, Bitmap-Werkzeuge
| Feature | Gesehen in | OSATURA |
|---|---|---|
| Pinsel: Randomisierung (Stempelanzahl, Versatz), Dual Papers, Mehrfachtextur-Pinsel | Harmony 24, TVPaint 12 | ❌ |
| Bleistift-Deckkraft nach Druck/Neigung/Geschwindigkeit/Taper/Fade | Harmony 24 | ❌ |
| Symmetrie-Hilfslinien (vertikal, horizontal, doppelt) | Harmony 24 | ❌ |
| Cutter-Werkzeug (Lasso, Trim), Konturen-Editor, Lücken schließen | Harmony | ❌ |
| Zeichenhilfen, Perspektiv-/Lineal-Hilfslinien † | Harmony | ❌ |
| Papierlose Animation: Onion Skin, Flipping, Leuchttisch | Harmony, Procreate Dreams, TVPaint | 🟡 (Onion Skin ja; keine Zeichenwerkzeuge oder Leuchttisch) |
| Bereinigung gescannter Zeichnungen, Scannen, Kameraaufnahme, Stop-Motion | OpenToonz | ❌ |
| Auswahlpinsel, Bildbibliothek | TVPaint 12 | ❌ |
| Schärfen bei Transformation | TVPaint 12 | n/a |
| Stiftdruckempfindlichkeit | Synfig | ❌ |
| Pfadbasierte Verlaufsschattierung | Synfig | ❌ |
| Text mit variablem/dynamischem Laufzeitinhalt, in jeder Sprache | Rive, Procreate Dreams | ❌ |
| Eigene Schriften, Titel, Abspann | Procreate Dreams | ❌ |
| Verwaltung von Paletten und Stilen | OpenToonz | ❌ |
| >50 Ebenentypen (Fraktale, Filter, Verzerrungen) | Synfig | ❌ (≈7 Ebenenarten) |

## Effekte und Compositing
| Feature | Gesehen in | OSATURA |
|---|---|---|
| Compositing-Netzwerk in der Node-Ansicht mit benannten Ports | Harmony 24 | ❌ (Ebenenbaum + Masken/Blends) |
| Mattes, Gruppen unter Composite | Harmony | ✅ Masken, isolierte Gruppen |
| CryptoMatte lesen/schreiben | Harmony 24 | ❌ |
| Toon-Shader (Umriss, Posterisierung) | Harmony 24 | ❌ |
| Unscharf maskieren, Kanaltausch | Harmony 24 | ❌ |
| Weichzeichner-Familie (radial, Spin, direktional), Bokeh, fraktales Rauschen, Kacheln | OpenToonz | ❌ |
| Partikel-FX | OpenToonz | ✅ Emitter-Ebene |
| 15 RGB- + 5 Alpha-Blend-Modi, Offscreen-Zeichnen pro Teil | Live2D 5.3 | 🟡 13 Blend-Modi; Rendering isolierter Gruppen ✅ |
| Clipping (Clip-IDs pro Teil) | Live2D, Rive | ✅ Alpha-Masken |
| HDR-/HDRI-Bildverarbeitung | Synfig | ❌ |
| Video-Import, ProRes, Rotoskopie über Footage | Procreate Dreams | ❌ |
| Audio-Engine (Voice-over, Musik, SFX) | Procreate Dreams | ❌ |

## 3D, Pipeline, Austausch, Runtimes
| Feature | Gesehen in | OSATURA |
|---|---|---|
| 3D-Szene, FBX-Knochenposing, Vulkan-Szenengraph, Zeichnen auf Perspektivebenen | Harmony 24 | ❌ |
| Externes Rendering über Maya / Blender | Harmony 24 | ❌ |
| Vektor-Import: PDF / AI / SVG; mehrlagiges EXR; DPX-Alpha; OpenColorIO (ACES) | Harmony 24 | ❌ |
| PSD-Import ohne Photoshop | Spine 4.2, Rive | ❌ |
| Lottie- / Figma-Paste-Import | Rive | ❌ (nur Lottie-**Export**) |
| Texture Packer / Atlanten | Spine | ❌ |
| JSON-/Binär-Skelett-Export + 19 offizielle / 40+ Community-Runtimes | Spine | 🟡 eigenes JSON; keine Runtimes |
| Open-Source-Runtimes (Web, Apple, Android, React, Flutter, C++, Unity, Unreal) | Rive | ❌ |
| Pixels-Modus (Sprites an Bildschirmpixeln einrasten) | Spine 4.2 | ❌ |
| Export: WebM mit Transparenz | Live2D 5.3 | ❌ |
| Export: SVG/PDF, PNG-/EXR-Sequenzen, 16-/32-Bit-Tiefe, Batch, ProRes | Cavalry | 🟡 SVG, PNG ✅; PDF/EXR/ProRes/hohe Bittiefe ❌ |
| Cloud-Rendering, Share-Links, Embeds | Rive | ❌ |
| Verteiltes Rendering (Toonz Farm), Versionskontrolle | OpenToonz | ❌ |
| PDF- / TVPaint-JSON-Export | OpenToonz | ❌ |
| Echtzeit-Teamzusammenarbeit, Community-Remixing | Rive | ❌ |
| HTML5-Canvas- / interaktiver Web-Export | Animate † | 🟡 HTML-Player (vorgerendert, nicht interaktiv) |
| Natives Linux/macOS/Windows | Synfig, OpenToonz | ✅ |
| Open-Source-Lizenz | Synfig, OpenToonz | ✅ MIT/Apache-2.0 |
| Ansicht für Ebenen-Parenting, Symbole/Bibliotheksinstanzen † | Animate | 🟡 Parenting ✅; keine Instanzbibliothek |
| Modulare, abdockbare UI, eigene Arbeitsbereiche, dunkles/helles Design | TVPaint 12, Live2D 5.0 | ❌ |

## Roadmap-Kandidaten (geordnet nach Nutzen ÷ Aufwand, zunächst ohne GUI)
1. Trim Paths + Pfadfolgen (Knochen/Ebenen entlang einer Bézier-Kurve) — Rive, Spine
2. Onion Skin + Bewegungsspur-Overlays — nahezu jedes 2D-Werkzeug
3. SVG-Import (Pfade, Verläufe) — Moho 14, Harmony, Synfig
4. Wind-/Schwerkraftterme für Federn, Knochendynamik, gesteuert durch Eingaben im Smart-Bone-Stil
5. Weichzeichner, Schlagschatten, Glühen, Umriss, Posterisierung (Ebenenfilter)
6. Ebenen-Instancing / Duplikatoren (N Klone entlang eines Pfades, mit Versatz pro Kopie)
7. Interaktiver Zustandsautomat in der HTML-/JSON-Runtime (Inputs, Übergänge, Blend)
8. Envelope-(Cage-)Deformer, Werkzeuge zum Kopieren/Verschweißen/Glätten von Gewichten, Auto-Mesh mit gleichmäßiger Dichte
9. Kameraschnitte / gespeicherte Kameras, Kamera mit Zielverfolgung
10. glTF-Export des Rigs; EXR-/PDF-/WebM-Alpha-Export
11. Textebenen (erfordert das Laden von Schriften) mit Pfadfolgen
12. Studio-GUI: Punkt-/Knochenbearbeitung auf der Canvas, Kurveneditor, Onion Skin

## Zusätzliche Quellen (Konkurrenz)
- Toon Boom Harmony 24 Release Notes und Premium Getting Started Guide (docs.toonboom.com)
- Spine 4.2 „The physics revolution" und „What is Spine?" (esotericsoftware.com)
- Live2D Cubism 5.0 und 5.3 „New Features" (docs.live2d.com)
- Cartoon Animator 5.3 Feature Highlights (discussions.reallusion.com)
- Rive-Funktionsseite (rive.app/features)
- Synfig Studio Features (zoftwarehub.com); OpenToonz-Dokumentation „What's New"
- TVPaint Animation 12 (tvpaint.com); Cavalry-Review (superrendersfarm.com)
- Adobe Animate „Layer parenting" (adobe.com/learn); Procreate Dreams (Cartoon Brew)
- Duik: Tag-Seite von Lester Banks (nur Titel †); Blender Grease Pencil: Handbuchseite nicht abrufbar (nicht abgedeckt)

---

# Nachtrag — Versionshistorien der Konkurrenz (Runde 2, 19.10.2026)

Legende wie oben: ✅ fertig, 🟡 teilweise, ❌ noch nicht. Aufgeführt sind nur Funktionen, die auf Seiten der Hersteller/Projekte oder in der Presse bestätigt sind. Piratenseiten wurden nicht verwendet; web.archive.org ist von dieser Umgebung aus nicht erreichbar, daher bleiben einige ältere Releases undokumentiert (siehe „Bekannte Lücken").

## Toon Boom Harmony 20 (Release Notes des Herstellers)
| Feature | OSATURA |
|---|---|
| Centerline-Editor: Punkte löschen, Glättungsoption | ❌ |
| Texturierte Bleistiftlinien → texturierte Pinselstriche; Pipette für Bleistifttexturen | ❌ |
| Gekrümmtes / S-förmiges Linienwerkzeug; Auto-Fill-Pinsel; Repaint-Pinsel | ❌ |
| Ausrichtungshilfen, Einrasten beschränkt auf ausgewählte Ebenen | ❌ |
| Quick Ease / Ease In / Ease Out auf ausgewählten Keyframes | 🟡 Easing pro Key ✅; Batch-Ease-Befehle ❌ |
| Szenenmarker (ohne Dauer, Ripple-Edit) | ❌ |
| Ansicht für Zeichnungsersetzung (Umbenennen/Filtern) | 🟡 Switch-Ebenen ✅ |
| Surface-Shading-Nodes: Ambient Occlusion, Cast Shadow, Surface Map/Normal | ❌ |
| **Weighted-Deform-Node** | 🟡 gewichtetes Bild-Mesh ✅ |
| Point-Kinematic-Output-Node (1-/2-/3-Punkt-Sampling) | ❌ |
| Pre-Render-Cache-Node | 🟡 Cache für eingebackene Federn ✅ |
| Nodes für Colour Fade / Curves / Levels / Gamma | ❌ |
| Planare Partikelform anhand einer Vektorzeichnung | 🟡 Emitter ✅; eigene Form ❌ |
| Master Controller: Deformer auf Deformer, Function Wizard, Slider Wizard | 🟡 Kanäle ✅; Wizard-UIs ❌ |
| Python-Scripting (PythonManager) | 🟡 JSON-Patch-Ops + CLI; kein eingebettetes Scripting |
| Farbraumverwaltung, 16-Bit-Rendering, per Skript gesteuerter Film-Nachbearbeitungsschritt | ❌ (ffmpeg-Nachbearbeitung von Hand) |
| Control Center: Batch-Kopieren/-Importieren von Szenen/-Ausführen von Skripten, Render-Warteschlange | ❌ |

## OpenToonz 1.7
| Feature | OSATURA |
|---|---|
| Geradlinien-Modus für Pinsel | ❌ |
| Unterstützung für GIMP- + MyPaint-Pinsel | ❌ |
| Füllmodus Auswahl+Freihand | ❌ |
| Bild aus der Zwischenablage einfügen | ❌ |
| Pre-Production-Dashboard / Storyboard | ❌ |
| Eigene Panels, transluzentes Hauptfenster, Viewer drehen/spiegeln | ❌ |
| Xsheet-Skalierung + Tag-Navigation | 🟡 Timeline-Daten ✅; keine GUI |
| Export mit Kamerabewegungsinformationen | 🟡 Kamera im Frame ✅ |
| **OCA-Export (Open Cel Animation)** | ❌ |
| MOV- und APNG-Export | 🟡 MP4/GIF ✅; APNG/MOV ❌ |
| Batch-Konvertierung TZP→TLV | ❌ |
| Floor Bump Iwa, Flow Fx; Float- + Linearfarb-Rendering | ❌ |
| Haiku-OS-Unterstützung | n/a |

## Synfig 1.4.2 (und 1.4.0 aus Runde 1)
| Feature | OSATURA |
|---|---|
| Lottie-Export inkl. erweiterter Outline- und Blur-Ebenen, Gamma | 🟡 Lottie-Export ✅ (Formen/Transformation); Blur ❌ |
| Editor für Tastenkürzel, Ein-Tasten-Werkzeugkürzel | ❌ |
| Editierbarer TimeLoop im Arbeitsbereich | 🟡 Loop-Clips ✅ |
| Duplizieren von Skeleton-/Skeleton-Deformation-Ebenen mit Knochenverknüpfungen | ✅ (Knochen sind Projektdaten) |
| SVG-Importer, der Style-Attribute verarbeitet | ❌ (kein SVG-Import) |

## Punkte aus Runde 1 (Herstellerseiten, gekürzt)
- **Spine 4.2/4.3**: Physik-Constraints, Slider, Constraint-Mixing, Pixel-Snapping — Federn ✅ (eingebacken), Slider/Mix 🟡 (IK-`mix` ✅), Pixels-Modus ❌.
- **Live2D Cubism 4.0–5.3**: ArtPath, Onion Skin, WebM-Alpha-Export — ArtPath ❌; Onion Skin und WebM-Alpha ✅.
- **Harmony 21/22**: gewichtete Deformer, Line-Boil-Texturen — 🟡 / ✅ (Wackeln).
- **Cartoon Animator 5** — siehe die Konkurrenztabellen oben.

## Neue Roadmap-Kandidaten aus dieser Runde
13. OCA-Export (Open Cel Animation) und APNG-Export — OpenToonz 1.7
14. Batch-Ease- / „Quick-Ease"-Befehle im Editor — Harmony 20
15. Farbkorrekturfilter (Fade, Levels, Curves, Gamma) — Harmony 20
16. Surface Shading (Cast Shadow / AO) für Vektorebenen — Harmony 20
17. Eingebettetes Scripting (Python/Lua) über die Patch-Op-API — Harmony 20
18. Pre-Render-Cache-Node / zwischengespeicherte Auswertung — Harmony 20
19. Szenenmarker + Ripple-Editing in der Timeline — Harmony 20
20. Layer Comps, animiertes Umhängen von Knochen, Patch-Ebenen, Follow-Path, SMPTE-Timecode, Safe Zones, Auto-Trace, Scatter Brush (bei der Moho-Recherche gefundene Lücken)

## Bekannte Lücken (aus zugänglichen Quellen nicht abrufbar)
Anime Studio 8.x, 10.1.x, 11.1; Details zu Moho 12.5 und 14.1; Spine 3.0–4.1 (der Blog zu 3.7 ist nur eine Videoseite); Harmony 17, 23, 25–27 (Dokumentationsseiten lieferten Fehler); Synfig 1.4.1; TVPaint 11; Cartoon Animator 4/5.1/5.2; Rive-Changelog.

## Quellen (Runde 2)
- docs.toonboom.com — Release Notes zu Harmony 20
- en.ubunlog.com — Artikel zu OpenToonz 1.7
- synfig.readthedocs.io — Release Notes zu Synfig 1.4.2
- esotericsoftware.com — Features-Rundown zu Spine 3.7 (Videoseite, keine Liste)

---

# Nachtrag — YouTube-Quellen (Runde 3, 19.10.2026)

Eine auf youtube.com beschränkte WebSearch liefert Videotitel; WebFetch liefert die **Beschreibung/Kapitelliste** jedes Videos (keine Transkripte). Aufgeführt sind nur dort genannte Funktionen; „neu in Version X" entspricht der Angabe des Uploaders (häufig der Hersteller).

## Moho 14.1 (Videobeschreibung)
| Feature | OSATURA |
|---|---|
| Handles im Curvature-Werkzeug ein-/ausblenden | ❌ (keine GUI-Canvas) |
| Shaded- und Halo-Formeffekte im Bearbeitungsmodus sichtbar | 🟡 Verläufe ✅; Halo-/Shade-Effekte ❌ |
| Pixelierungseffekt in den Live-Projekteinstellungen | ❌ |
| Preview Animation: höhere Auflösung, ausgeblendeter Ausgaberahmen | 🟡 Raster-Vorschau ✅ |
| Modeless-Dialoge für Lua-Skripte | ❌ (kein Scripting) |
| PSD-Blend-Modus „Linear Dodge (Add)"; aktualisierte Blend-Modi | 🟡 Blend-Liste prüfen; Add/Dodge ❌ |
| Anzeige ungespeicherter Dokumente in der Titelleiste | ✅ das Studio verfolgt den Undo-Zustand |
| Fixes: Smart-Bone-Auswertung, Subframe-Bewegungsunschärfe + Knochendynamik, Bézier-Handles der Länge null | n/a |

## Moho 12.5
Die Titel-/Beschreibungsseiten lieferten keine Funktionsliste (Video veraltet; kostenloser Download von 12.5 zurückgezogen). Verwandte Titel erwähnen Smart-Bones-Tutorials, Smart-Warp-2.5D-Rigs und die Installation von „Mynd Tools" — Smart Bones 🟡 (Kanäle/Morphs), Smart Warp 🟡 (FFD-Lattice ✅).

## Spine
| Version | Feature | OSATURA |
|---|---|---|
| 3.7 | Audio-Unterstützung | ❌ (WAV wird nur für die Lippensynchronisation analysiert) |
| 3.7 | Überarbeitete Exporte | 🟡 |
| 3.7 | Type-to-Search | ❌ |
| 3.7 | Stretchy / Compressed / Uniform IK | ❌ (IK erhält die Länge) |
| 3.7 | Skins kombinieren, Skins duplizieren, Platzhalter | 🟡 Switch-Ebenen |
| 3.7 | Vertex kopieren/einfügen | ❌ |
| 3.7 | Rendering mit Pixelraster | ❌ |
| 3.7 | Additives Animations-Blending | 🟡 nur additive Morphs |
| 3.7 | spine-cpp-Runtime; Web-Player | ❌ / 🟡 HTML-Player |
| 3.8 | Mesh Tracing | 🟡 automatisches Gitter-Mesh; kein Bild-Tracing |
| 3.8 | Polygonales Texture Packing | ❌ |
| 3.8 | **Soft IK** | ❌ |
| 3.8 | Skin-Knochen und -Constraints | ❌ |
| 3.8 | Mix-and-Match-Projekt | ❌ |
| 3.8 | Ordner, Lineale, erweiterter PSD-Export, Willkommensbildschirm | ❌ |
| 3.8 | Kleinere .json/.skel; Laden von Binärdaten in spine-ts | ❌ (nur JSON) |
| 4.0 beta | **Graph-/Kurveneditor** | 🟡 Easing pro Key ✅; kein Editor |
| 4.1 beta | Sequence-Funktion (Bildsequenz-Attachment) | ❌ |

## Toon Boom Harmony 25 / 25.2
| Feature | OSATURA |
|---|---|
| Pencil-Retouch-Werkzeug | ❌ |
| Photoshop-Pinsel importieren | ❌ |
| **Breakdown Assistant für Cut-out-Animation** | ❌ (der Walk-Cycle-Generator ist verwandt) |
| Timeline-Lineal sperren; Einstellungen für Action-Vorlagen | ❌ |
| Bessere Unterstützung für Bild-, 3D- und USDZ-Dateien | ❌ |
| Art Layer Manager (Art-Ebenen tauschen, leeren, zusammenführen, duplizieren; 25.2+) | ❌ (Ebene duplizieren über Patch-Ops ✅) |
| Master Controller Grid Wizard | ❌ |

## Cartoon Animator 5 (Serie)
| Feature | OSATURA |
|---|---|
| 5.1 PSD- und Vektor-Roundtrip-Bearbeitung | ❌ |
| 5.1 Import von Bildsequenzen und GIF | ❌ |
| 5.2 **Motion Pilot** (Bewegung per Ziehen puppenspielerisch steuern) | ❌ |
| 5.2 Motion-Path-Animation / Pfadbearbeitung | ❌ (Roadmap #1) |
| Federdynamik, FFD, Vektoranimation (bereits vorhanden) | ✅ / ✅ / ✅ |

## Neue Roadmap-Kandidaten aus dieser Runde
21. Soft IK + Stretchy-IK-Modi — Spine 3.8/3.7
22. Bildsequenz-Ebene (Frame-Listen-Attachment) und GIF-/APNG-Import — Spine 4.1, CA 5.1
23. Pixelierungs- / Pixel-Snap-Ausgabefilter — Moho 14.1, Spine 4.2
24. Aufzeichnung im Motion-Pilot-Stil: Live-Ziehen eines Knochens/Handles in Keys erfassen — CA 5.2
25. Breakdown-Assistent (automatische Zwischenbilder mit Bögen für Cut-out-Rigs) — Harmony 25
26. Mesh Tracing aus dem Alpha-Umriss — Spine 3.8
27. Skin-Sets (Gruppen von Attachments gemeinsam tauschen) — Spine 3.7/3.8
28. Audiospur im Projekt + Wiedergabesynchronisation (nicht nur Analyse) — Spine 3.7
29. Lua-/Python-Scripting-Hooks — Moho, Harmony 20

## Quellen (Runde 3, YouTube)
- Moho 14.1: A Complete Breakdown — youtube.com/watch?v=QT4AiV1wO1k; Moho 14.1 Update — watch?v=FCiBKBjDaNg
- Spine 3.7 features rundown — watch?v=Fi0ks2LeYSM; Spine 3.8 features rundown — watch?v=kU26B5465SQ; Spine v4-beta curve editor — watch?v=37CeFRzrjqc
- Harmony 25 changes — watch?v=kJ58C2DNIk8; Art Layer Manager 25.2+ — watch?v=AZc-8zHucOo
- Cartoon Animator 5.2 Motion Pilot — watch?v=Eu7vO2uGGN4; 5.1 sequence/GIF import — watch?v=O8EGwVwqgfs

---

# Nachtrag — Erschöpfende Lückensuche (Runde 4, 19.10.2026)

Quellen: offizielle Release Notes von Synfig/Rive, YouTube-Titel/-Beschreibungen. Aufgeführt sind nur benannte Funktionen.

## Anime Studio 8 (YouTube-Serie „Pro Feature", Titel + Beschreibungen)
| Feature | OSATURA |
|---|---|
| Automatisches Bild-Tracing (Bitmap → editierbarer Vektor) | ❌ (Roadmap #20) |
| Character Wizard (geführte Charaktererstellung) | 🟡 Demo-Rigs/Walk-Generator; kein Wizard |
| Photoshop-Dateien (PSD) importieren | ❌ |
| Real-Time Media Connection (Live-Verbindung zu externen Medien) | ❌ |
| Scale-Points-Werkzeug | 🟡 Patch-Ops auf Punkten |
| 1080p-HD-Projektvorgaben | ✅ beliebige Größe |
Anime Studio 10.1 / 11.1: keine zugängliche Quelle gefunden.

## Synfig 1.4.1 – 1.4.4
| Version | Feature | OSATURA |
|---|---|---|
| 1.4.1 | Sound-Ebenen in das exportierte Video gemischt | ❌ (keine Audiospur) |
| 1.4.1 | Ebene duplizieren kopiert alle ausgewählten Ebenen | ✅ (Batch-Ops) |
| 1.4.3 | Bone-Link-Abfrage: Transformation statt Ursprung verknüpfen | ✅ |
| 1.4.3 | Papagayo-Import (.pgo); SVG-Import-Fixes; Framerate bei Bildsequenzen | ❌ / ❌ / ❌ |
| 1.4.3 | GIF-Render-Fixes, FFmpeg-3.x-Kompatibilität | ✅ GIF/MP4 |
| 1.4.3 | Autosave (bis hinunter zu 15 s) | ❌ (explizites Speichern) |
| 1.4.4 | Fix beim Export des letzten Frames | ✅ durch Tests abgedeckt |

## Rive (offizielle Release Notes, Sept.–Okt. 2026) — der aktivste Konkurrent
| Feature | OSATURA |
|---|---|
| String-Template-Converter (Text aus View-Model-Werten, Zahlenformatierung) | ❌ |
| Gestrichelte Konturen (Dash Effect), aus SVG importiert | ❌ |
| Q/W/E gedrückt halten + ziehen zum Verschieben/Drehen/Skalieren | ❌ (keine GUI) |
| Pinselformen im Animate-Modus keybar | ❌ |
| Kopieren/Einfügen von Constraint-Keyframes; Converter behalten Formeln | 🟡 Constraints ✅, Kopieren ❌ |
| Relatives Data Binding; View Models; Blob-Eigenschaften | ❌ |
| Luau-Scripting (Fetch, Textsatz, GPU-Canvas, Shader/WGSL, instanziierte Bild-Meshes) | ❌ |
| Blend States / Mix-by-View-Model-Timeline | 🟡 Crossfade-Bake |
| Tastatur-, Fokus-, Gamepad-Listener; Texteingabe; Barrierefreiheits-Semantik | ❌ |
| Grid- und Stack-Layouts, virtualisierte Listen, Scroll-Constraints & Scroll-Physik | ❌ |
| Set Parent (Alt+P) unter Beibehaltung von Position und Zeichenreihenfolge | 🟡 Umhängen über Patch-Ops; Beibehalten der Weltposition ❌ |
| Align to Key Object; Root-Bone-Offset; Scroll to selection | ❌ |
| Additive Stärke für Füllungen/Konturen; getrennte RGBA-Kanäle keybar/bindbar | 🟡 additive Morphs; RGBA keybar ✅ |
| Innere/äußere Konturen (Early Access) | ❌ |
| Auto Trace (besser für Pixel Art) | ❌ |
| Konvertierung von Flash `.fla`/XFL → Rive | ❌ |
| Render-Presets: Zustandsautomat → Video | 🟡 Clip → MP4/GIF |
| `.riv`-Export im Free-Plan; CLI (`rive push/publish/ls/docs`), VS-Code-Erweiterung, headless Eingabesimulation | 🟡 CLI ✅ (eigene Befehle) |
| KI-Agent im Editor | ❌ |
| Android-Vorschau; Runtimes für React Native / Apple / Android / Flutter | ❌ |
| Revisionsverlauf & Wiederherstellung, Offline-Sync, Rückgängig, das den Zoom beibehält | 🟡 Rückgängig/Wiederholen ✅ |

## TVPaint Animation 11 (YouTube-Beschreibungen)
Erstellen von Tastenkürzeln, entfernbare Unterzeichnungen, Linienglättung, Zeichenaufzeichnung, KeyFramer-Pivotpunkt per Klicken und Ziehen (11.7.1), Sprite Generator, Inlay Texture FX, Hilfslinien, Video „end of update releases" — alles ❌ (Raster-Malwerkzeuge; OSATURA ist Vektor/Rig).

## Toon Boom Harmony 17 (YouTube)
Free Form Deformer 🟡 (FFD-Lattice ✅), aufgebohrte Master Controller 🟡, RGB Difference Keyer ❌, Motion Blur ❌ (Roadmap #5).

## Cartoon Animator 4 / Spine 3.6 / Harmony 22–23
Gefundene Videos (CA 4: Rokoko-Mocap-Plugin, Update 3D-Bewegung-zu-2D, Normal Maps & Beleuchtung in 4.5; Spine 3.4 Pfade; Feature-Video zu Spine 3.6; Zeichenwerkzeuge in Harmony 22), aber die Beschreibungen nennen keine Funktionen → nicht beansprucht. Nur dem Titel nach bemerkenswert: **Mocap-Retargeting** (CA 4) ❌, **Normal Maps/Beleuchtung** (CA 4.5) ❌, **Pfad-Constraints** (Spine 3.4) ❌.

## Neue Roadmap-Kandidaten aus dieser Runde
30. Dash-Effekt + innere/äußere Konturen
31. Eingebettete Scripting-Runtime (Lua/Luau) mit GPU-freier API
32. Data Binding / View Models + Zustandsautomaten-Inputs (Tastatur/Zeiger/Fokus) für den HTML-Player
33. Scroll-Constraints / Layout-Container (Grid, Stack) für Animation im UI-Stil
34. Papagayo-Import (.pgo) für Lippensynchronisation; Import im Rhubarb-Stil
35. Set Parent unter Beibehaltung der Welttransformation
36. Autosave + Revisionsverlauf im Studio
37. Motion-Blur- und RGB-Keyer-Filter
38. Pfad-Constraints (Knochen entlang einer Bézier-Kurve) — Spine 3.4, geht in Roadmap #1 auf
39. Motion-Capture-Retargeting auf ein 2D-Rig (BVH/JSON) — CA 4
40. Normal-Map-Beleuchtung für Bildebenen — CA 4.5
41. In den MP4-Export gemischte Tonspur — Synfig 1.4.1
42. .fla- / XFL- und PSD-Import

## Weiterhin nicht abrufbar
Anime Studio 10.1 / 11.1, ausführliche Notizen zu Moho 12.5, Detailseiten zu Spine 3.0–3.6, Harmony 23, ausführliche Notizen zu Cartoon Animator 4.x, Changelog-Text von TVPaint 11.

## Quellen (Runde 4)
- synfig.readthedocs.io — Release Notes 1.4.1, 1.4.3, 1.4.4
- releasebot.io/updates/rive — Rive Editor 0.9–0.9.272, CLI 1.2–1.6
- YouTube: Anime Studio 8 Pro Feature series (q_HH2zOE-Yk, CUemDIlPH6Y, 408CLy652mU, l9eXLUO9Dus); TVPaint 11 demo (MoqNrCAUy5k); Harmony 17 (1CU_3RJBcl4); Cartoon Animator 4 playlist

---

# Nachtrag — Soziale Netzwerke und Community-Kanäle (Runde 5, 19.10.2026)

**Zugangslage:** X/Twitter-Seiten sind für den Abruf per robots.txt gesperrt (nur Snippets/Titel aus Suchergebnissen sind sichtbar); Facebook ebenso; Reddit-Suchen mit Domainfilter wurden vom Such-Proxy abgelehnt, und ungefilterte Suchen lieferten keine Reddit-Threads; das Lost-Marble-Forum liefert eine JavaScript-Weiterleitungshülle ohne lesbaren Inhalt. Nichts wurde umgangen. Lesbar war Folgendes:

## Offizielle Moho-Kanäle (moho.lostmarble.com News, X-Snippets)
| Eintrag | Hinweise | OSATURA |
|---|---|---|
| Moho 13.5.3 (Apr. 2022) | verbesserte automatische Lippensynchronisation, bessere Oberfläche, weniger aufdringliche Knochen | ✅ Lippensynchronisation (WAV-Heuristik) |
| Moho 13.5.5 (X, Mai 2022) | **Relative Pfade** für verknüpfte Assets | ✅ das Asset-Verzeichnis ist relativ |
| Moho 14.3 „Mantis Shrimp" (Dez. 2024) | pinselfokussiertes Update | ❌ |
| Moho 14.4 „Side Quest" (Nov. 2025) | Export in Game-Engines und 3D-Software | 🟡 Lottie/eigenes JSON; glTF/FBX ❌ |
| Moho 14.5 „Showtime" (Okt. 2026) | magnetische Onion Skins, Shift & Trace, Optionen für Liquid Shapes, EXR/PNG mit Farbprofilen, Live-Playback-Modus zum Streamen/Steuern von Charakteren in Echtzeit | siehe Hauptmatrix; Live-Modus ❌ |
| Ambassador-Programm (Jun. 2026); offizielle Haltung: keine KI-Werkzeuge geplant (Mär. 2025) | Richtlinie | n/a — OSATURA hat ebenfalls keine KI-Funktionen |
| Webinare (Víctor Paredes usw.) | Tutorials | n/a |

## Community-Vergleiche (Spine-Forum, „Spine vs Moho 12")
- Nutzer loben an Moho: integrierte Physik, Ebenenmaskierung, Smart Bones, Vektorbearbeitung.
- Nutzer kritisieren: FFD gilt als schwach; **keine echte Runtime**, Game-Export auf FBX beschränkt.
- Creature wird für integrierte Motion Capture gelobt (FFD kritisiert).
- Die Physik-Constraints von Spine 4.2 stoßen auf X (Nutzerbeiträge) auf Begeisterung.
OSATURA: Physik ✅ (eingebackene Federn), Maskierung 🟡 (Clip über Booleans), Smart Bones 🟡, Vektorbearbeitung ✅, Runtime ❌, Mocap ❌.

## Gesehene Titel von Forum-Wunschlisten (Inhalt nicht lesbar)
„Concentrate on Timeline/Workflow improvements" (2024), „Road map for Moho", „Help Shape the Future of Anime Studio", „13.5.2 performance issue" — deutet darauf hin, dass Timeline/Workflow und Performance die am häufigsten gewünschten Bereiche sind; keine verifizierte Funktionsliste.

## Ergänzungen zur Roadmap
43. Echtzeit-Live-Playback- / Streaming-Modus (WebSocket-Poseneingabe → gerenderte Frames) — Moho 14.5
44. Runtime-Bibliothek in einer kleinen portablen Sprache (WASM-Build des Evaluators) — die am häufigsten genannte Schwäche von Moho
45. Ebenenmaskierung als vollwertiger Ebenentyp (nicht nur Boolesche Operationen)
46. Ergonomie von Timeline/Workflow im Studio (Kurveneditor, Mehrfachauswahl von Keys) — Community-Wunschlisten

## Quellen (Runde 5)
- moho.lostmarble.com/blogs/news (17 Beiträge, Dez. 2021 – Okt. 2026)
- X-Suchsnippets: Beiträge von @MohoAnimation (13.5.5 Relative Paths; Preise von Moho 14), Nutzerbeiträge zur Physik in Spine 4.2
- Spine-Forum-Thread „8033 spine vs moho 12" (de.esotericsoftware.com)
- Thread-Titel des Lost-Marble-Forums (Inhalt nicht lesbar)

---

# Nachtrag — Wunschlisten und Feature-Request-Boards (Runde 6, 19.10.2026)

Aufgeführt sind nur tatsächlich lesbare Wünsche. Die Spalte „OSATURA" gibt an, ob die Engine den Wunsch bereits abdeckt.

## Moho (Lost-Marble-Forum)
Lesbar: nur der Forenindex. Es gibt die Bereiche „Feature Request Discussions" und „Unofficial Bug Reports and Feature Requests"; der neueste sichtbare Wunschtitel: **„Dock Windows – Custom Interface"** (18. Okt. 2026). Der seit Langem laufende Thread „Help Shape the Future of Anime Studio" (828 Antworten, 2011–2021) und weitere Wunschlisten-Threads („Road map for Moho", „My wish for Moho in 2024 – Concentrate on Timeline/Workflow improvements", „Request for top 10 Anime Studio Pro 6 features") sind JavaScript-Weiterleitungshüllen → Inhalt nicht lesbar. Konkrete Forenvergleiche (Spine-Forum): Physik, Maskierung, Smart Bones werden in Spine gewünscht; Moho wird für schwaches FFD und fehlende Runtime kritisiert.
| Wunsch | OSATURA |
|---|---|
| Andockbare Fenster / eigene Oberfläche | ❌ (Web-Studio mit festem Layout) |
| Verbesserungen bei Timeline / Workflow | 🟡 |
| Echte Runtime für Spiele | ❌ |
| Stärkeres FFD | ✅ Bernstein-Lattice + Mesh |

## Cartoon Animator (Reallusion Feedback Tracker + Wunschlisten-Thread)
| Wunsch (Stimmen, wo angezeigt) | OSATURA |
|---|---|
| Standard-Keyframe-Kurve in den Einstellungen | ✅ Easing pro Key |
| Keyframe-Bearbeitung über mehrere Kanäle | 🟡 Patch-Batch-Ops |
| Persistente Panel-Einstellungen | ❌ |
| Farbcodierte Marker und Kanäle (links rot / rechts grün) | ❌ (Spiegeln L↔R ✅) |
| „Uber Key", um alle animierten Kanäle anzuzeigen | ❌ |
| SVG-Unterstützung (Affinity, Inkscape); Vektorpfade einfügen | ❌ |
| Ausrichten-Werkzeug/-Panel | ❌ |
| Tiefenunschärfe / Weichzeichner-Filter | ❌ |
| GIF-/APNG-/Video-Import per Drag & Drop, auf der Timeline platziert | ❌ |
| Spot-/Umgebungslichter mit Schatten | ❌ |
| Automatisches Scrollen der Timeline | n/a |
| Besserer Text: animierbares Kerning/Zeilenabstand | ❌ |
| Animierbare SVG-Farbe | ✅ |
| Globales Ausblenden / „Shy"-Kanäle | ❌ |
| Angedockte Panels | ❌ |
| Mehrere Szenen pro Projekt | ❌ |
| **Auto-Animate-Panel (prozedurale Idle-Bewegung an Knochen)** | 🟡 Wackeln mit kohärentem Rauschen ✅ |
| Live-Zeichenebene / dynamische editierbare Props (18) | ❌ |
| Hitbox und Physik (14) | 🟡 Federn |
| Import von transparentem MOV (14) | ❌ |
| 360°-Panorama (5); Ideen zur KI-Integration (6) | ❌ |

## Feature-Request-Board des Live2D Cubism Editors (30 Threads, 2023–2026)
| Wunsch | OSATURA |
|---|---|
| Linux-Unterstützung | ✅ |
| Tastenkürzel in der manuellen Mesh-Bearbeitung | n/a |
| Blendshape-Limit bis −100 % | ✅ Morphs unbegrenzt |
| Proportionale Größenänderung; Gravity-Werkzeug; Mesh-zu-Mesh-Sperre | ❌ / ❌ / ❌ |
| Glue für Deformer + präzise Glue-Gewichte | 🟡 Gewichte ✅ |
| Physik importieren, ohne bestehende Gruppen zu überschreiben | ❌ |
| Parameter im Animator sperren; Loop-Edit vs. Wiederholung | ❌ |
| Optionen „Expand Warp based on Child"; Rotationszentrum des Warps ändern | ❌ |
| Werkzeug im Stil von „Set Vertex Flow", Auswahl von Loop-Kanten, Mesh-Mapping von Konturen mit variabler Dichte | ❌ (Auto-Mesh gleichmäßig) |
| Pixel-Art-Unterstützung | ❌ |
| Vertex-Deckkraft; Teile filtern und übersichtlichere Anzeige | ❌ |
| Keyframes einfach verschieben | ✅ Patch-Ops |
| Zwischengespeicherten Atlas aus dem Speicher entfernen | n/a |
| Physik-Fenster: maximale Ausgabe in %, Bereiche von Pendeln gesammelt bearbeiten | 🟡 |
| RISC-V / eingebetteter Cubism Core | ❌ |
| Optionale Drawables in der CDI3-Spezifikation | n/a |

## Toon Boom Harmony
Das Nutzerforum ist geschlossen und leitet auf Discord um (nicht durchsuchbar); die alten Threads des „Feature Requests"-Boards („One can only dream: A Feature List", „Software Upgrade Suggestions Part 2", „Bitmap Layer Overhaul LONG OVERDUE", „Various Quality of Life suggestions") zeigen nur den Hinweis auf die Schließung. Keine Wünsche wiederherstellbar.

## Synfig (GSoC-Ideenliste 2019–2022, verlinkte GitHub-Issue-Nummern)
| Wunsch | OSATURA |
|---|---|
| Waypoints kopieren/einfügen (#2139) | ✅ Keys über Ops |
| Tab-Wechsel mit Strg+Tab, Fensterkürzel, Auswahl des Wiedergabebereichs, Suche in den Eigenschaften, Lineale ein-/ausblenden, mausgesteuerte Parameteränderung | ❌ (GUI) |
| Performance: schnelleres Rendern/Exportieren; Ebenenparameter cachen; inkrementeller Render-Task-Baum | ✅ reine Funktion evaluate; kein Cache ❌ |
| Portierung auf GTK 4, Builds mit Conan/vcpkg | n/a |

## Spine
Wünsche gehen ins Forum und werden auf ein öffentliches Trello-Board verschoben (nicht abrufbar: robots.txt). Aus dem Forum: Die historisch höchste Priorität sind Stabilität des Editors und Runtime-Updates. Der Plan aus der Kickstarter-Zeit (Meshes → FFD → Skinning → IK-Pinning) ist inzwischen vollständig umgesetzt, daher stehen Mesh/FFD/Skinning in OSATURA auf Augenhöhe mit dieser alten Liste; IK-Pinning ✅ (Pin-Constraint). Neuere Titel von Forumswünschen: Unterstützung für das neueste three.js / React Three Fiber; Unterstützung von Spine 4.3 in Defold.

## Rive
Die öffentliche Roadmap = „Feature Requests"-Kanal mit Statuslabels (in Arbeit / geplant / veröffentlicht); zum Lesen ist ein Login nötig. Über die Release Notes (Runde 4) hinaus ist nichts wiederherstellbar.

## OpenToonz, TVPaint, Pencil2D
GitHub-Seiten (OpenToonz/Synfig) sind per robots.txt gesperrt; kein lesbares TVPaint-Board gefunden; ein Pencil2D-Thread „Feature requests to add" existiert, wurde aber nicht geöffnet (außerhalb des Umfangs).

## Roadmap-Ergänzungen aus den Wunschlisten
47. Auto-Animate / prozedurale Idle-Ebene (Atmen, Blinzeln, Kopfschwanken mit Seltenheit & Intensität) — CA-Wunsch #20; baut auf dem vorhandenen Wackeln auf
48. Kamera-Tiefenunschärfe und Weichzeichner-Filter — CA
49. Lichter (Spot/Umgebung) mit Schattenwurf für Bildebenen — CA
50. Operationen zum Ausrichten / Verteilen (Editor-Op) — CA
51. Farbcodierung von Kanälen und „Shy"-Sichtbarkeit von Kanälen im Studio — CA
52. Mehrere Szenen pro Projekt + Szenenschnitte — CA
53. Auto-Mesh mit variabler Dichte; Glättung des Vertex-Flows — Live2D
54. Parameter für das Rotationszentrum des Warp-Deformers; proportionale Größenänderung — Live2D
55. Physik-Import mit Zusammenführen, ohne Gruppen zu überschreiben — Live2D
56. Parametersperre pro Track während der Loop-Bearbeitung — Live2D
57. Andockbares/anpassbares Studio-Layout — Moho, CA
58. Parameter-Cache über Frames hinweg (inkrementelle Auswertung) — Synfig

## Quellen (Runde 6)
- Reallusion Feedback Tracker: „Wish list for new features for future updates"; Cartoon-Animator-5-Board (reallusion.com/FeedBackTracker)
- community.live2d.com — Feature-Requests zum Cubism Editor (Seite 1 von 7)
- Synfig GSoC 2022 ideas list (synfig-docs-dev.readthedocs.io)
- Lost-Marble-Forenindex (lostmarble.net/forum); Spine-Forum-Threads 2250, 8033; Kickstarter-Thread 1501
- Rive-FAQ: „Does Rive have a public roadmap?"

---

# Nachtrag — Open-Source-Projekte auf Git-Plattformen (Runde 7, 19.10.2026)

**Zugang:** Die Such-API von GitHub ist in dieser Umgebung nicht verfügbar („sessions are bound to their configured repositories"), daher wurden Projekte per Websuche gefunden und aus ihren öffentlichen READMEs gelesen. Die APIs von GitLab und Codeberg sind durch den Proxy gesperrt; Seiten von Codeberg/GitLab tauchten nur in Suchergebnissen auf (Friction war lesbar). Sternezahlen konnten nicht abgerufen werden. Mehrere Projekte sind sehr neu/früh — die aufgeführten Funktionen entsprechen den Angaben in den READMEs und wurden nicht durch Ausführen verifiziert.

## Ähnliche Open-Source-Werkzeuge für Rigging / Animation
| Projekt | Plattform | Bemerkenswerte Funktionen | Vergleich mit OSATURA |
|---|---|---|---|
| **Rigby2D** (jaisurya-dev-art/rigby2d) | GitHub, MIT/Apache | Editor im Spine-Stil (PSD-Import, Mesh + Weight Painting, verknüpfte Meshes, IK-/Transform-/Pfad-/Physik-/**Slider**-Constraints, Skins mit skinspezifischen Knochen, Events, Bounding-Box-/Clipping-/Punkt-/Pfad-Attachments, Dopesheet + Kurveneditor), Export als Spine-JSON 3.8/4.2/4.3 + .skel, Atlas, JS-/Rust-/Godot-Runtimes, **integrierter MCP-Server** für KI-Assistenten, Tauri-Desktop | OSATURA fehlen Editor-GUI, Atlas, Runtimes, Skins, Events, Slider-/Pfad-Constraints; es hat einen Evaluator als reine Funktion, Liquid-/Boolean-/Emitter-FX, Lippensynchronisation, Walk-Generator |
| **SkelForm** (Retropaint/SkelForm, Rust) | GitHub | FABRIK-IK, nativ + Web + mobiler Wrapper | ähnlicher Rig-Kern ✅; keine GUI ❌ |
| **Ankhimate editor** | GitHub, Pre-Alpha | Modi Setup/Animate, Slots + Skins, FABRIK beliebiger Länge, Pfad-/Physik-Constraints, Mesh-Silhouetten-Tracing aus Alpha, Weight Brush mit Blend-Modi + Knochensperre, **FFD-Timelines**, Events mit Audio, Marker, Track-Offsets pro Knochen, **vorlagenbasierter Editor für Exportformate**, Rust-Referenz-Runtime | ❌ überwiegend GUI; Ideen: Export-Vorlagen, Track-Offsets pro Knochen |
| **rig2d-rs** (tanhieuno41) | GitHub, Rust | Portierung der Spine-4.2-Physik, IK mit **Mix/Softness/Bend/Compress/Stretch/Uniform**, automatisches Binden der Gewichte per Distanz-Falloff, Gewichte glätten/normalisieren/auf 4 begrenzen, Shelf-Atlas-Packer, Onion-Skin-Ghosting, Spine-JSON-Import/-Export | ❌ Soft-/Stretch-IK, Gewichtswerkzeuge, Atlas, Spine-Austausch |
| **Boner2D**, **Skelly**, **omarvision 2d-bones-IK** | GitHub | Skelett-Werkzeuge für Unity / Game Designer | einfach |
| **Skeletal-Animator** (ostad-ai) | GitHub, Python | Knochen + FABRIK, Rechteck-/Kapsel-/Kreisformen, 4 Easings, Onion Skin, Kamera-Clip-Fenster mit Keyframes, **Sichtbarkeitsbereiche pro Knochen**, Spritesheet PNG+JSON, **transparentes MP4/WebM/MOV/GIF** | ❌ Sichtbarkeitsbereiche, transparenter Video-/Spritesheet-Export, Onion Skin |
| **Stretchy Studio** (MangoLion) | GitHub, MIT | PSD-/PNG-Import, **KI-Auto-Rigging** (DWPose-Posenerkennung), Setup-Wizard, Vertex-Animation pro Teil per Mesh, Vertex-Skinning für Gliedmaßen, **automatisches Augen-Clipping** (Iris innerhalb des Auges), **Shape Keys im Blender-Stil** mit Influence-Slidern, Audiospuren in der Timeline, **Export als Spine-4.0-JSON** | ✅ Morphs ≈ Shape Keys; ❌ Auto-Rig, Augen-Clipping, Audio |
| **Iki** (zeikar/iki) | GitHub, MIT | Web-Engine im Live2D-Stil: parametergesteuerte Quads, Warp-Mesh-/Grid-Deformer, Stencil-Masken, Feder-/Ketten-Physik, PSD-Import, **Auto-Rigger aus nach Rollen benannten Ebenen** (Blinzeln, Blick, Lippensynchronisation, Kopfdrehung, Nicken, Neigen, Brauen, Atmen, Haarphysik), offenes JSON-Schema mit **StandardParameter-IDs**, MCP-Server + Claude-Code-Plugin | ✅ JSON-Schema, Lippensynchronisation; ❌ Standard-Parameter-IDs, Auto-Rigger, Masken |
| **Umamo** (umamoorg/umamo) | GitHub, Java | Live2D-kompatibler Editor: CMO3/MOC3 lesen/schreiben (bis Cubism 5.4), PSD-/KRA-/CLIP-Import, Befehlspalette, Presets für Tastenbelegungen, **Diagnose-CLI** (Modelle dumpen/extrahieren/konvertieren/diffen, Qualitäts-Presets für Auto-Mesh); Animation noch nicht unterstützt | ❌ Modell-Diff-CLI, Befehlspalette |
| **Ayatsuri2D** | GitHub | `.aom`-MessagePack-Modell (Obermenge von moc3), moc3-v3–6-Export, C-ABI, Fuzzing in der CI, Runtime ohne Panics, p99.9-Latenz-Benchmarks (früh, „not usable yet") | ❌ Binärformat, C-ABI, Fuzzing |
| **Ayagami** (AyagamiDev) | GitHub, Rust/wgpu | MOC3 bis SDK 5.0 inkl. Glue- und Blendshape-Parametern, teilweise Neuberechnung, lineares/sRGB-Blending, erweitertes Blending/Offscreen aus SDK 5.3, Godot-Komponente | ❌ GPU-Renderer, MOC3 |
| **kasane-2d**, **img2live**, **claude-live-2d**, **minivt** | GitHub | Live2D-ähnliche Modellierung / Forschung von Einzelbild zu Puppet | n/a |
| **Glaxnimate** (KDE) | GitHub-Mirror / KDE | Vektoranimation; die Build-Abhängigkeiten zeigen Potrace-Bitmap-Tracing, libav, Python-Scripting, libarchive; Fokus auf Lottie | ❌ Tracing, Scripting |
| **Friction** (friction/friction, Codeberg) | Codeberg, GPL-3 | Vektor- + Raster-Animation für Web/Video, Motion Graphics, SVG, Skia + FFmpeg + GLSL (basiert auf enve) | ❌ SVG-Animation, Shader |
| **Tahoma2D** | GitHub | 2D + Stop-Motion, basiert auf der Toonz-Studio-Ghibli-Version | ❌ |
| **Pencil2D** | GitHub, GPL-2 | handgezeichnet in Bitmap + Vektor, Onion Skin | ❌ |
| **q0idmation** (q0s0id) | GitHub | Editor + Player + **Skriptsprache** | ❌ Scripting |
| **svgMotion** (michaelsboost) | GitHub | Vektoranimation im Browser | ❌ |
| **kinetic** | GitHub | SVG-Keyframe-Timeline → CSS/GSAP-Export | ❌ CSS/GSAP-Export |
| **dotlottie-rs** (LottieFiles) | GitHub, Rust | **Zustandsautomaten** (Guards/Übergänge/Actions), Theming/**Slots** (Überschreibungen von Farbe/Skalar/Text/Vektor zur Laufzeit), `.lottie`-Zip-Container mit mehreren Animationen, C-API + WASM, ThorVG-Renderer | 🟡 Lottie-Export ✅; ❌ Slots/Zustandsautomaten/dotLottie-Container |
| **rive-runtime** | GitHub | C++-Runtime: GPU-Vektor-Renderer (Metal, Vulkan, D3D11/12, GL/WebGL), eingebunden in Apple/Android/Flutter/Unity/Unreal/Web | ❌ |
| **spine-runtimes** | GitHub | Runtimes für Unity, Godot, Unreal, libGDX, MonoGame, XNA, Flutter, Haxe, C/C++/C#/TS, Android, iOS, GLFW, SDL, SFML; Richtlinie „ein Branch pro Editor-Version" | ❌ |
| **Moho-Tools-Repo** (wanderjimenezRD/Moho-Pro-14.4-2D-Animation-Tools) | GitHub | nur Auflistung; die Beschreibung nennt keine Funktionen — nicht bewertet | — |

Aufgetauchte Repos mit Raubkopien (z. B. „activate-free" für Adobe Animate): nicht geöffnet oder verwendet.

## Übernehmenswerte Ideen (neue Roadmap-Kandidaten)
59. **MCP-Server / Agent-API** über den Patch-Op-Editor, damit KI-Assistenten riggen und animieren können (Rigby2D, Iki) — passt zur JSON-Patch-Architektur von OSATURA
60. **Standard-Parameter-IDs** (ParamAngleX, EyeOpen, MouthOpen…), damit jeder Host ein Modell steuern kann (Iki)
61. **Auto-Rigger aus nach Rollen benannten Ebenen**: Blinzeln, Blick, Kopfdrehung/Nicken/Neigen, Brauen, Atmen, Haarphysik (Iki)
62. **Automatisches Augen-Clipping** (Iris durch die Augenform maskiert) (Stretchy Studio) — über eine Boolean-Ebene
63. **Slider-Constraint** und **Pfad-Constraint** (Rigby2D, Spine)
64. Modi **Soft / Stretchy / Compress / Uniform IK** (rig2d)
65. **Gewichtswerkzeuge**: glätten, normalisieren, auf N begrenzen, kopieren, Knochensperre, Blend-Modi des Pinsels (rig2d, Ankhimate)
66. **Mesh-Silhouetten-Tracing aus Alpha** (Ankhimate)
67. **Sichtbarkeitsbereiche pro Knochen** (Skeletal-Animator)
68. **Video-Export mit transparentem Hintergrund (WebM/MOV mit Alpha) und Spritesheet PNG+JSON** (Skeletal-Animator)
69. **Export-Template-Engine** (benutzerdefinierte Ausgabeformate) (Ankhimate)
70. **dotLottie-`.lottie`-Container + Runtime-Slots/Theming + Zustandsautomaten** (dotlottie-rs)
71. **Modell-Diff-/Dump-/Convert-CLI** (`osatura diff a.json b.json`) (Umamo)
72. **Befehlspalette + umbelegbare Tastenkürzel** im Studio (Umamo)
73. **Gefuzzter Loader, Runtime-Pfad ohne Panics, Latenz-Benchmarks** (Ayatsuri2D) — Härtung von `Engine::new`
74. **Spine-JSON-Import/-Export** für den Austausch (Rigby2D, rig2d, Stretchy Studio)
75. **Shelf-Atlas-Packer** für Bildebenen (rig2d, Rigby2D)
76. **Events mit Key-Werten und Audio-Cues** auf der Timeline (Rigby2D, Ankhimate)
77. **Track-Offsets / Zeitverschiebung pro Knochen** (Ankhimate)

## Quellen (Runde 7)
- github.com/jaisurya-dev-art/rigby2d; Retropaint/SkelForm; Ankhimate/editor; tanhieuno41/rig2d-rs (Release v0.1.0)
- github.com/zeikar/iki; MangoLion/stretchystudio; umamoorg/umamo; Yoru-KenomoVT/ayatsuri_2d; AyagamiDev/ayagami
- github.com/ostad-ai/Skeletal-Animator; KDE/glaxnimate; tahoma2d/tahoma2d; pencil2d/pencil
- github.com/rive-app/rive-runtime; EsotericSoftware/spine-runtimes; LottieFiles/dotlottie-rs
- github.com/topics/animation-editor, /topics/vector-animation
- codeberg.org/friction/friction

---

# Nachtrag — Weitere öffentliche Quellen (Runde 8, 19.10.2026)

Quellen: itch.io, Steam, AlternativeTo, Wikipedia, Ergebnisse von Akademik-/Patentsuchen, Testberichte zu After-Effects-Werkzeugen. Die Angaben stammen von Shop-/Projektseiten, nicht aus praktischen Tests.

## Indie-/Nischenwerkzeuge für Rigging (itch.io, Steam)
| Werkzeug | Bemerkenswerte Funktionen | OSATURA |
|---|---|---|
| **Inochi Creator / Inochi2D** (Kitsunebi Games, Open Source, Beta) | offenes, auf VTuber ausgerichtetes 2D-Puppet-Rigging + Echtzeit-Runtime; eigenes Format (nicht kompatibel mit Live2D/Spine); GIF-/Bildsequenz-Export; mehrsprachige UI | 🟡 gleiche Nische (offenes Rig-Format) |
| **Keyframe.it** (Browser, kostenlos) | Knochen + IK + gewichtete Meshes + Federbewegung; **Game-Symbol-Clips** (Idle, Land, Anticipation, Win); integrierter Bildeditor + PSD-Import; Export als GIF, Sprite Sheet, Video, eigenständiger HTML-Player, **Spine-3.8/4.2-Bundles**; Autosave im Browser; **keyframe-mcp**-Konnektor für KI-Agenten; optionaler integrierter KI-Assistent | ✅ HTML-Player/GIF; ❌ Sprite Sheet, Spine-Bundle, MCP, Editor im Browser |
| **RigBone 2D** (Godot) | Knochenposing, Pose-als-Frame, Onion Skin, Zuschneiden + transparenter Hintergrund, PNG-Frame-Sequenz (keine Runtime nötig) | ❌ Onion Skin, PNG-Sequenz mit Alpha-Zuschnitt |
| **SpineBones** | Slots + Bild-Attachments, Gruppen, IK, **Audio-Vorschau auf der Timeline**, Hintergrund-Referenzbild, Export als JSON / Sprite Sheet / WebM | ❌ |
| **PySpine, Tangent (sprite animator)** | auf itch.io gelistet; kein Featuretext gelesen | — |
| **Mocap2D / FBX2SPINE** (Steam) | FBX-/BVH-Mocap-Import; **Auto Match** des Skeletts nach Namen + Hierarchie; **Smart Retargeting** (Projektion, gut für Seitenansicht); **Direct Retargeting** pro Achse; **automatische In-Place-Konvertierung** für Fortbewegungsschleifen; **IK mit Bodenkontakt** (Foot Lock); Live-Vorschau; Import/Export von FBX, BVH, Spine-Skeletten, Unity 2D Animation | 🟡 Retarget nach Namen ✅; ❌ Mocap-Import, Foot-Lock-IK, In-Place-Konvertierung |
| **VTube Studio** (Steam) | Webcam-Tracking (OpenSeeFace), **sprachgesteuerter Mund**, Maus/Stift als Tracking-Eingabe, Zuordnung beliebiger Eingaben auf beliebige Modellparameter, Hotkeys für Ausdrücke/Animationen, angeheftete PNG-„Items", die dem Modell folgen; nur Live2D-Modelle | 🟡 Parameterkanäle ✅; ❌ Live-Input-Mapping, Hotkeys, angeheftete Items |
| VBridger, Vitamins | Steam-Werkzeuge rund um VTubing; nicht bewertet | — |

## After-Effects-Rigging-Ökosystem (Review von School of Motion)
| Feature | Quelle | OSATURA |
|---|---|---|
| **Joystick-Controller** (5 Schlüsselposen: Ursprung/rechts/links/oben/unten), in 2D interpoliert | Joysticks 'n Sliders | ❌ — starker Kandidat |
| **Slider**, die beliebig viele Posen mischen | JnS, Duik | 🟡 Morph-Kanäle |
| Strukturen (Knochen unabhängig vom Artwork, wiederverwendbar), Autorig | Duik Bassel | ✅ |
| Controller: Slider, 2D-Slider, Winkel | Duik | ❌ |
| **Connector** (Slider/Joystick/Rotation, die eine beliebige Eigenschaft mit einer anderen verknüpfen), Master-Property-Control | Duik | 🟡 Ausdrücke? (Ausdrucks-Engine ✅ in Kanälen) |
| Zwischenposen-Keys zwischen Posenzuständen | Duik | ✅ |
| Maskenpfade animieren | JnS | ✅ Pfad-Morphs |

## Konkurrierende / alternative Apps (AlternativeTo, Wikipedia-Liste Stand 2026)
Tahoma2D (OpenToonz-Fork, Stop-Motion), TupiTube, enve (eingestellt; wurde zu Friction), Expressive Animator (interaktives SVG), Cavalry, Jitter (Web-Motion-Design), Blazy (Figma → Animation, CSS-/React-Export), Animation Desk, Pivot Animator, FlipaClip, Krita-Animation, Blender (Mär. 2026), Clip Studio Paint (Animation), Adobe Character Animator (Webcam-Puppenspiel; die „What's new"-Seite ist per robots.txt gesperrt), Adobe Animate (letzter gelisteter Release Okt. 2023), CrazyTalk, DigiCel FlipBook, Apple Motion, RETAS. Keine davon bringt Rigging-Funktionen, die über das hinausgehen, was frühere Runden abdecken; neue Ideen: **Figma-/Design-Werkzeug → Animations-Presets mit CSS-/React-Export** (Blazy) und **Webcam-Puppenspiel** (Character Animator, VTube Studio).

## Akademische Literatur und Patente (nur Titel/Abstracts)
- Jacobson et al. 2011, **Bounded Biharmonic Weights** für Echtzeit-Deformation — automatische glatte Skinning-Gewichte aus einem Cage/Skelett ✅ Idee: `auto_skin` verbessern (derzeit distanzbasiert).
- Adobe-Patente US9865073 / US10078910, **„Computer animation of artwork using fast 2-dimensional deformation solving"** (Mesh-Warp von Character Animator). *Clean-Room-Hinweis:* OSATURA verwendet Bernstein-FFD und gewichtetes Skinning, nicht diesen patentierten Solver; ein künftiger Solver im ARAP-Stil sollte mit einem Anwalt geprüft werden. Dies ist keine Rechtsberatung.
- arXiv 2025–26, Auto-Rigging: **ASMR** (3D-Mesh-Skelett + Skin über generative 2D-Priors), UniRig, „How to Train Your Dragon" (Diffusions-Rigging), HumanRig, SPRig, UniMate — alles 3D/ML; nur als künftige Funktion „Auto-Rig-Vorschlag" relevant.
- Stony Brook TVC 2017 **Retargeting automatischer Skinning-Gewichte** — Gewichte zwischen Meshes kopieren (✅ Roadmap: Gewichte kopieren).

## Neue Roadmap-Kandidaten aus dieser Runde
78. **Joystick-Controller**: 2D-Kontrollpunkt, der 5 Schlüsselposen (und N-Posen-Varianten) mischt → steuert beliebige Kanäle — JnS / Duik / Live2D-Parameter
79. Mocap-Import (BVH) + Auto-Match des Skeletts + In-Place-Loop-Konvertierung + Foot-Lock-IK — Mocap2D
80. Live-Input-Mapping: beliebige Eingabe (Maus, MIDI, Audiopegel, Webcam-Landmarken) → Parameter, plus Hotkeys für Ausdrücke — VTube Studio
81. Angeheftete Items/Props, die einem Knochen/Anker mit eigenem Offset folgen — VTube Studio
82. Presets für Game-Symbol-Clips (Idle / Land / Anticipation / Win) — Keyframe.it
83. Export als Spritesheet (PNG + JSON) und transparente PNG-Sequenz mit Zuschnitt — Keyframe.it, RigBone
84. Audio-Vorschau auf der Timeline mit Wellenform — SpineBones
85. Referenzbild-Ebene (nicht gerenderter Hintergrund) — SpineBones
86. Bounded-Biharmonic- oder Heat-Diffusion-Auto-Skin-Gewichte (ersetzen den Distanz-Falloff) — BBW-Paper
87. Design-Werkzeug-Presets → CSS-/React-Export — Blazy
88. Webcam-Gesichts-Tracking-Puppenspiel über das Studio (WebRTC + Landmarken-Modell) — Character Animator, VTube Studio

## Quellen (Runde 8)
- itch.io: kitsunebi-games/inochi-creator, keyframeit.itch.io, itch.io/blog/1586091 (RigBone 2.0), itch.io/t/6177139 (SpineBones)
- Steam: FBX2SPINE / Mocap2D (app 2467960), VTube Studio (app 1325860)
- schoolofmotion.com — Joysticks 'n Sliders vs Duik Bassel
- alternativeto.net (Alternativen zu Moho, Expressive Animator); en.wikipedia.org/wiki/List_of_2D_animation_software
- Jacobson 2011 BBW (homes.cs.washington.edu/~jovan/papers/jacobson-2011-bbw.pdf); Patente US9865073B2 / US10078910; arXiv 2503.13579, 2503.15586, 2504.12451; Stony Brook TVC 2017 weight retargeting

---

# Nachtrag — Adobe Character Animator, vollständiger Funktionsumfang (Runde 9, 19.10.2026)

Quellen: Versionstabelle von Wikipedia (inkl. 26.0 vom Jan. 2026), Adobe-Produktseite, Adobe-Help-Seite „Behaviors" und Release Notes (Okt. 2018, Okt. 2020/Feb. 2021, Okt./Dez. 2021, 2023). Adobes eigener „What's new"-Index liefert nur 2023 und 2026; 24.x und 25.x werden von Wikipedia als reine Stabilitäts-Releases geführt.

## Konzept
Ebenenbasiertes Photoshop-/Illustrator-Artwork (seit Okt. 2018 auch EPS/SVG) wird zu einem **Puppet**, das automatisch anhand von **Ebenennamen/Tags** geriggt wird; die Performance wird live (Webcam-Gesicht/-Körper, Mikrofon, Tastatur, Maus, MIDI) als **Takes** auf einer Timeline aufgezeichnet; die Ausgabe geht an PNG-Sequenzen, WAV, Media Encoder (inkl. **Video mit Alpha**), Dynamic Link zu After Effects / Premiere sowie live per Syphon / Mercury Transmit. Der Starter-Modus ist kostenlos; Pro erfordert Creative Cloud.

## Behaviors (vollständige Liste aus Adobe Help)
| Behavior | Funktion | OSATURA |
|---|---|---|
| **Face** | Kopf, Augen, Brauen, Nase, Mund über Webcam; Parallaxe und Stärke der Kopfposition | ❌ (kein Live-Tracking) — Roadmap #88 |
| **Body** (Body Tracker, Sensei AI, 22.0) | Arme, Oberkörper, Beine über Webcam | ❌ |
| **Lip Sync** | Mund-Viseme aus der Stimme; transkriptgestützte Genauigkeit (22.1); Engine 3.4 besser bei F, W-Oo, weniger L/M | 🟡 WAV → 5 Viseme ✅; Transkript-Alignment ❌ |
| **Nutcracker Jaw** | Unterkiefer aus Gesicht/Stimme | ❌ |
| **Eye Gaze** | Pupillen bewegen sich unabhängig | ✅ Aim-Constraints/Kanäle |
| **Head & Body Turner** | tauscht Ebenengruppen (frontal, Dreiviertel, seitlich), während sich Kopf/Körper drehen | 🟡 Switch-Ebenen ✅; kein winkelgesteuerter Auto-Wechsel |
| **Walk** | Walk Cycle, Laufen, Posenbetonung, Schulter-/Hüftschwung, **Pin Feet When Standing** (3.4/3.5) | ✅ Walk-Generator; ❌ Pin Feet |
| **Limb IK** (früher Arm IK) | Beugerichtung + Dehnung für Arme und Beine | 🟡 2-Knochen-IK ✅; Dehnung ❌ |
| **Leader/Follower** (22.5) | formt, wie sich Gliedmaßen beugen/dehnen | ❌ |
| **Dragger** | Maus/Touch bewegt eine Region | ❌ (das Studio ist auf der Canvas nicht interaktiv) |
| **Physics / Dangle** | Pendeln, Kollision, Schwerkraft, Wind; Squash & Stretch, Dämpfung | 🟡 Federn ✅; Kollision/Wind ❌ |
| **Magnets** | Handles rasten nach Nähe ineinander ein | ❌ |
| **Particles** | emittiert Physikobjekte (Schnee, Kanonenfeuer) | ✅ Emitter |
| **Motion Lines** | Nachziehlinien hinter schnellen Objekten | ❌ |
| **Wiggler** | Wackeln bei Vektor-Artwork | ✅ Wackeln/Boil |
| **Breath** | Atembewegung | 🟡 manueller Kanal; Roadmap #61 |
| **Auto Blink** | automatisches Blinzeln / Ebenenflackern | ❌ (Roadmap #61) |
| **Motion Library** (23.0, 350+ Bewegungen, Stärkeparameter 23.1) | Ganzkörperanimationen anwenden | 🟡 nur Walk; ❌ Bibliothek |
| **Cycle Layers** | Ebenen einer Gruppe nacheinander durchlaufen | 🟡 Switch-Ebenen ✅; Frame-Zyklen ❌ |
| **Layer Picker / Motion Trigger / Fader** | eine Ebene zeigen; nach Bewegungsrichtung umschalten; bei Trigger ausblenden | 🟡 |
| **Transform** | Verschieben/Skalieren/Drehen/Deckkraft von Handles | ✅ |
| **Handle Fixer** | Teile eines Meshes sperren | ✅ Pins |
| **Triggers** | Tastendruck zeigt Ebenen/Gesten; **Controls-Panel** mit Slidern, Winkelreglern, Buttons, MIDI-Zuordnung | ❌ — Roadmap #80 |
| **Collide, Dynamic, Cycle Pause, Auto-swap (22.5)** | Tags/Behaviors für Physik und Frame-ähnliche Wechsel | ❌ |

## Funktionen für Timeline, Aufnahme und Workflow
| Feature | Version | OSATURA |
|---|---|---|
| Takes, Replays (wiederverwendbare, auslösbare Aufnahmen), Smart Replays, editierbare Replays | 2.0 / 22.0 / 23.6 | 🟡 Clips ✅; Replays ❌ |
| Keyframes, Zeitdehnung von Keyframes, Szenenkameras, auslösbares Audio | 3.0 / 3.2 | ✅ Keys/Kamera; ❌ auslösbares Audio |
| Speech-Aware Animation (Kopf-/Brauenbewegung aus der Stimme, Sensei) | 3.4 | ❌ — Roadmap #61 |
| Takes zusammenführen/aneinanderreihen; farbige Take-Zeilen; Suchfilter (`kind:` `shared:` `armed:`); Shy-Zeilen; Takes einklappen | 3.4 | ❌ |
| An Frames / Objekten einrasten; Timeline-Snapping | 3.5 | ❌ |
| Arbeitsbereich-Operationen: Ripple Delete, Zeit einfügen, Loop, Trim, Export | 2.0 | 🟡 Clip-Ops ✅ |
| Rigging-Issues-Bereich; Suche nach Tags & Triggern | 3.0 | ✅ die Validierung sammelt alle Fehler |
| Verlaufs-Lesezeichen; automatische Wiederherstellung beschädigter Projekte (4.2); neues kompaktes, korruptionsresistentes Dateiformat (4.0) | | 🟡 Rückgängig ✅; Wiederherstellung ❌ |
| Puppet Maker (Vorlagen: Haare, Haut, Accessoires) | 22.1.1 | ❌ |
| Characterizer (ein Foto/Gemälde in ein Puppet verwandeln) | 2.0 | ❌ |
| Quick Export fürs Teilen in sozialen Netzwerken | 22.5 | 🟡 GIF/MP4 ✅ |
| Szenen-Snapshots, Aufnahme-Countdown, eigene Grafiken für Steuerungsbuttons | 1.5 | ❌ |
| Kameraauswahl, Multi-Monitor, HiDPI | 2.0 | n/a |
| Live-Ausgabe (Syphon, Mercury Transmit), Twitch-Character-Trigger-Erweiterung | 1.1 / 2.1 | ❌ — Roadmap #43 |
| MIDI, Skripte über V8 (JavaScript) | 1.1 / 2.0 | ❌ — Roadmap #31 |
| Puppets teilen/veröffentlichen, teilbare Puppet-Dateien | Beta 5 | 🟡 JSON portabel |
| Dynamic Link zu AE / Premiere | Beta 5 | ❌ |
| Gemeldete Schwächen (Review): steile Lernkurve, Abhängigkeit von Hardware (Webcam/Mikrofon), lange Vorbereitungszeit der Charaktere, kostenpflichtiges Pro, aufwendige Trigger-Einrichtung | | Chance: deterministische Offline-Pipeline von OSATURA, CLI/JSON |

## Was für OSATURA hervorsticht (neue Roadmap-Kandidaten)
89. **Tag-basiertes Auto-Rig aus Ebenennamen** (Kopf, Augen, Brauen, Mund-Viseme, Arm-/Bein-Tags) — Kernidee von Character Animator; erweitert Roadmap #61
90. **Takes & Replays**: Parameter-Performance als wiederverwendbare, auslösbare Clips mit Blend-Easing aufzeichnen
91. **Pin Feet When Standing** und Bodenkontaktsperre für den Walk-Generator
92. Limb-Stretch-Behaviors **Leader/Follower**; **Magnets**; **Motion Lines**
93. Format einer **Motion Library** (benannte Bewegungs-Clips mit Stärke 0–100 %)
94. **Head & Body Turner**: Auto-Wechsel von Ansichtsgruppen aus einem Yaw-Winkel-Kanal
95. **Transkriptgestützte Lippensynchronisation** (Forced Alignment von Text auf WAV-Phoneme)
96. **Sprachbewusste Kopf-/Brauenbewegung** aus Audioenergie und Pausen
97. **Video-Export mit Alpha** (ProRes 4444 / WebM VP9 Alpha) — auch in Skeletal-Animator (Roadmap #68)
98. **Auslösbares Audio** und audioreaktive Parameter

## Quellen (Runde 9)
- en.wikipedia.org/wiki/Adobe_Character_Animator (Funktionsliste und Versionstabelle bis 26.0, Jan. 2026)
- adobe.com/products/character-animator.html
- helpx.adobe.com — Behaviors in Character Animator; Feature-Zusammenfassungen Okt./Dez. 2021, Okt. 2020/Feb. 2021; What's new (2023–2026)
- community.adobe.com — „Character Animator October 2018 detailed changes"
- manytools.com/review/adobe-character-animator (Stärken/Schwächen)
