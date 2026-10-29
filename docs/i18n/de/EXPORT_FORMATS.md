<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# Export- und Importformate von OSATURA

Was jedes Dateiformat, das OSATURA schreibt oder liest, erhält, annähert und weglässt, welcher Befehl es erzeugt und wie du
das Ergebnis prüfst. Dieser Text steht unter der Lizenz CC-BY-4.0. Dokumentierte Version: **osatura 0.1.0**.

Jede unten gezeigte Befehlszeile wurde gegen das eingebaute Demoprojekt (`osatura demo demo.json`) oder eine 8-Frame-Kopie davon
(`small.json`, dasselbe Projekt mit `"frames": 8`) ausgeführt. Die genannten Größen stammen von diesem 8-Frame-Projekt mit 640x480 und dienen nur
zur Orientierung: Sie skalieren mit Frames, Ankern und Ebenen, nicht mit etwas Festem. Details zu den Befehlen (jede Option,
jeder Fehler) stehen in [CLI_REFERENCE.md](CLI_REFERENCE.md); das Dateiformat des Projekts selbst steht in
[PROJECT_FORMAT.md](PROJECT_FORMAT.md); der Codec- und Patentstatus steht in [PATENT_NOTES.md](PATENT_NOTES.md) (keine Rechtsberatung).

## Inhalt

1. [Welches Format für welche Aufgabe](#1-welches-format-für-welche-aufgabe)
2. [Ausgabeformate im Überblick](#2-ausgabeformate-im-überblick)
3. [Einzelbilder: SVG, PNG, 16-Bit-PNG, OpenEXR](#3-einzelbilder)
4. [Raster-Animation: PNG-Sprite-Sheet, GIF, APNG, WebM, MP4](#4-raster-animation)
5. [HTML-Player](#5-html-player)
6. [Lottie (.json) und dotLottie (.lottie)](#6-lottie-json-und-dotlottie-lottie)
7. [glTF binär (.glb)](#7-gltf-binär-glb)
8. [Ordner im Open-Cel-Animation-Format (OCA)](#8-ordner-im-open-cel-animation-format-oca)
9. [Import: SVG, OpenRaster, Lottie, Bilder, GIF/APNG](#9-import-svg-openraster-lottie-bilder-gifapng)
10. [Import: BVH-Motion-Capture, MIDI und andere Live-Eingaben](#10-import-bvh-motion-capture-midi-und-andere-live-eingaben)
11. [Audioformate](#11-audioformate)
12. [Matrix der Funktionsunterstützung](#12-matrix-der-funktionsunterstützung)
13. [Dateien verkleinern: export-fit, lod, size-report](#13-dateien-verkleinern)
14. [ffmpeg-Anforderungen](#14-ffmpeg-anforderungen)
15. [Hinweise zu Patenten und Lizenzen](#15-hinweise-zu-patenten-und-lizenzen)
16. [Das Geschriebene validieren](#16-das-geschriebene-validieren)
17. [Unterschiede zu anderen Dokumenten](#17-unterschiede-zu-anderen-dokumenten)

---

## 1. Welches Format für welche Aufgabe

| Du willst... | Verwende | Warum |
|---|---|---|
| Das exakte Aussehen, als eine Datei geteilt, läuft in jedem Browser, scrubbar | `.html` | Inline-SVG-Frames, gezeichnet vom eigenen Renderer von OSATURA; nichts wird angenähert. |
| Eine kleine Vektoranimation für eine Webseite oder App mit einem Lottie-Player | `.json` oder `.lottie` | Vektor, auflösungsunabhängig, klein; aber siehe die Liste der Auslassungen (Bilder, Striche mit variabler Breite, Audio...). |
| Eine Endlosschleifen-Vorschau für Chat oder README | `.gif` | Funktioniert überall; 256 Farben pro Frame, 1-Bit-Transparenz, kein Ton. |
| Ein GIF-Ersatz mit voller Farbe und Alpha | `.apng` | Verlustfreies 8-Bit-RGBA; größer als GIF. |
| Ein Video mit Ton | `.mp4` (H.264 + AAC) oder `.webm` (VP9/AV1 + Opus) | Benötigt ein von dir installiertes `ffmpeg`. `.webm` behält die Transparenz. |
| Sprite-Sheet für eine Game-Engine | `.png` | Ein PNG plus ein JSON-Atlas mit den Kachelrechtecken. |
| Ein Skelett und Bewegung für ein 3D-Werkzeug | `.glb` | Bones, Animation, flache Formen als Dreiecksnetze. |
| Frames für Compositing-/Malwerkzeuge | `export-oca` (Open Cel Animation) oder `frames --png` | Cels als PNG, ein Ordner. |
| Linear-Light- oder hochbittiefe Einzelbilder | `render-hdr` (`.exr` oder 16-Bit-`.png`) | Float-Renderer, 16-Bit- oder Half-/Float-OpenEXR. |
| Ein einzelnes Standbild | `render` (`.svg` oder `.png`) | Vektor oder 8-Bit-Raster. |

## 2. Ausgabeformate im Überblick

| Format | Befehl | Endung | Braucht ffmpeg | Alpha | Audio | Größe für die 8-Frame-Demo* |
|---|---|---|---|---|---|---|
| SVG (ein Frame) | `render` | `.svg` | nein | ja | nein | einige KB |
| SVG-/PNG-Sequenz | `frames` | Ordner | nein | ja | nein | eine Datei pro Frame |
| PNG (ein Frame) | `render` | `.png` | nein | ja (8-Bit) | nein | |
| 16-Bit-PNG | `render-hdr` | `.png` | nein | ja | nein | |
| OpenEXR (half/float) | `render-hdr` | `.exr` | nein | ja | nein | 466 KB half, 923 KB float (640x480) |
| PNG-Sprite-Sheet + JSON | `export` | `.png` (+ `.atlas.json`) | nein | ja | nein | |
| GIF | `export` | `.gif` | nein | 1-Bit | nein | 83 KB |
| APNG | `export` | `.apng` | nein | ja | nein | 131 KB |
| WebM | `export` | `.webm` | ja | ja | ja (Opus) | 10.5 KB |
| MP4 | `export` | `.mp4` | ja | nein | ja (AAC) | 8.9 KB |
| HTML-Player | `export` | `.html`, `.htm` | nur für Audio | ja | eingebettetes mp3 | 66 KB |
| Lottie | `export` | `.json` | nein | ja | nein | 47 KB |
| dotLottie | `export` | `.lottie` | nein | ja | nein | 11.5 KB |
| glTF binär | `export` | `.glb` | nein | ja | nein | 115 KB |
| Open Cel Animation | `export-oca` | Ordner | nein | ja | nein | ein PNG pro Cel |

\* GIF, APNG, WebM und MP4 mit `--scale 0.5` (320x240); Vektorformate in voller Größe. „Braucht ffmpeg“ bedeutet ein funktionierendes
`ffmpeg` im `PATH` (Abschnitt 14). Ausgabenamen werden anhand der Endung gewählt, ohne Beachtung der Groß-/Kleinschreibung; eine unbekannte Endung wird abgelehnt und
nichts wird geschrieben (`unknown export format '.xyz' (use .gif .html .json .mp4 .apng .webm .png .glb)`, und `.lottie` wird ebenfalls
akzeptiert).

Alle Writer weigern sich, das Eingabeprojekt mit der Ausgabe zu überschreiben, schreiben über eine temporäre Datei und benennen sie um und lehnen
symbolische Links ab, sofern nicht `--follow-symlinks` angegeben ist (CLI_REFERENCE Abschnitt 1.4). Der Größenschutz für Vektor-Exporter
(Lottie, HTML, GLB) lehnt einen Export von mehr als etwa 40 Millionen Werten ab (Frames x Anker, plus Bones und Ebenen pro Frame)
mit `export would write about N million values ...`.

## 3. Einzelbilder

### SVG (`render ... out.svg`, `frames`)

```
$ osatura render demo.json 12 f12.svg --bones
wrote f12.svg
$ osatura frames demo.json fr            # fr/frame_0000.svg ... frame_0047.svg
```
Die Vektorausgabe des eigenen Renderers von OSATURA: Pfade, Verläufe, Striche mit Breitenprofilen, Strichelungen, Blend-Modi, Masken und Ebenen-
Effekte werden als SVG geschrieben. `--bones` zeichnet das Skelett-Overlay. Bei `.svg` lässt `--scale` die viewBox des Dokuments unverändert. **Bekannte Auslassungen/Grenzen:** Der Renderer meldet keine; wie ein
bestimmter SVG-Viewer Blend-Modi und Filter behandelt, liegt bei diesem Viewer. `--proxy N` schreibt einen vereinfachten Scrub-Proxy, der feine Striche, Verläufe und Blend-Modi weglässt (nicht für
Auslieferungen). **Validierung:** im Browser öffnen oder `python3 -c "import xml.dom.minidom,sys; xml.dom.minidom.parse(sys.argv[1])" f12.svg`.

### PNG (`render ... out.png`, `frames --png`)

8-Bit-RGBA, sRGB, gerendert vom eigenen Rasterisierer der Engine. `--scale N` in (0, 16] multipliziert die Pixelgröße. Ein Projekt mit
`"background": null` rendert transparent, wo nichts gezeichnet wird. Standardmäßig nicht farbverwaltet: Werte werden im sRGB-kodierten Raum gemischt.
Seit Welle 7 kann ein Projekt sich für Linear-Light-Compositing entscheiden (`osatura color-space p.json out.json linear`, Feld `color_space`):
Die Ausgaben Raster, GIF, APNG, Video und `render-hdr` werden dann in f32-Linear-Light zusammengesetzt und einmal nach sRGB konvertiert; SVG und
Lottie ändern sich nicht (LIMITATIONS L-22).

### 16-Bit-PNG und OpenEXR (`render-hdr`)

```
$ osatura render-hdr small.json 3 f.exr                           # HALF, 640x480, 465,611 bytes
$ osatura render-hdr small.json 3 f2.exr linear --precision float # FLOAT, 923,211 bytes
$ osatura render-hdr small.json 3 f4.png                          # 16-bit/colour RGBA, sRGB-encoded
```
* **OpenEXR:** einteilig, Scanline, **unkomprimiert**, Kanäle `A B G R`, Linear Light, vormultipliziertes Alpha. Standardmäßig `HALF`,
  `FLOAT` mit `--precision float` (oder `--float32`). Das Wort `linear` hat bei EXR keine Wirkung (immer linear).
* **16-Bit-PNG:** sRGB-kodiert, sofern nicht das Wort `linear` angegeben ist.
* **Treue:** Der Float-Renderer trägt mehr Präzision als der 8-Bit-Renderer, aber das Artwork hat gewöhnliche sRGB-Farben, daher
  enthält die Datei Werte im Anzeigebereich in höherer Präzision, kein szenenbezogenes HDR-Licht. Der Projekthintergrund wird nicht gezeichnet.
* **Bekannte Auslassungen/Grenzen:** ein Frame pro Datei; keine Kompression, Ebenen oder AOVs; kein ICC-Profil; `--8bit` fällt auf den 8-Bit-
  Rasterisierer als Quelle zurück.
* **Validierung:** `ffprobe f.exr` meldet `exr 640x480 gbrapf32le`; jeder EXR-Viewer öffnet die Datei.

## 4. Raster-Animation

Alle diese Formate rendern zuerst jeden Frame über die Raster-Engine. Ein Projekt mit `"background": null` rendert mit transparenten
Pixeln überall dort, wo nichts gezeichnet wird; die Demo hat eine Hintergrundfarbe (`#f6f1e7`) und ist daher deckend. `--scale N` in (0, 16]
gilt für `.gif .apng .webm .mp4 .png`; bei anderen Formaten wird es abgelehnt (`unknown option --scale for 'export .json'`).

### PNG-Sprite-Sheet (`export ... out.png`)

```
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.atlas.json
wrote out3.png                    # 4 columns x 2 rows of 160x120 tiles
```
Ein PNG mit allen Frames als Kacheln (`--columns N`, standardmäßig ein nahezu quadratisches Raster: ceil(sqrt(frames)) Spalten) und ein Atlas
`out3.atlas.json` (der Ausgabename mit durch `.atlas.json` ersetzter Endung), der pro Frame `{index, x, y, w, h}` auflistet. Alpha bleibt erhalten.
Der Atlas hieß früher `out3.json` und konnte eine Lottie-Datei mit demselben Stamm stillschweigend überschreiben; der neue Name vermeidet das, und der Atlas-Pfad durchläuft dieselben Eingabe-/Symlink-Schutzmechanismen.

### GIF (`export ... out.gif`)

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif                      # 8 frames, 83 KB
```
* Ein Bild pro Quell-Frame, **Endlosschleife**, eine lokale Palette mit bis zu 256 Farben pro Frame (quantisiert, ohne Dithering).
* **Timing:** GIF-Verzögerungen sind ganze Hundertstelsekunden, und Viewer behandeln alles unter 2 cs als 10 cs, daher sind 50 fps das tatsächliche Maximum.
  Bis 50 fps werden die Verzögerungen per Fehlerdiffusion verteilt (29.97 fps wechselt zwischen 3 und 4 cs), sodass die Gesamtdauer innerhalb von 1 cs von
  `frames / fps` bleibt. Oberhalb von 50 fps wird die Sequenz **auf 50 fps dezimiert**, die Dauer bleibt erhalten und eine Warnung wird ausgegeben
  (`GIF cannot hold 60 fps ... use APNG, WebM or MP4 for the full rate`).
* **Transparenz:** 1-Bit (das Flag für die transparente Farbe wird gesetzt); halbtransparente Pixel bleiben nicht erhalten.
* **Bekannte Auslassungen:** kein Audio (`note: GIF cannot carry audio; ...`, wenn das Projekt Sounds hat); kein partielles Alpha; Bilder, die breiter
  oder höher als 65535 Pixel sind, werden abgelehnt (`image too large for GIF`).
* **`--report r.json`:** schreibt die Liste dessen, was das Format nicht tragen konnte (hier: die Dezimierung der Framerate, falls vorhanden).
* **Tipps zur Größe:** ein kleineres `--scale`, weniger Frames oder `export-fit --format gif` (Abschnitt 13). GIF ist unter den Rasterformaten das am wenigsten effiziente
  für fotografische oder verlaufsreiche Grafik.
* **Validierung:** `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` gibt die
  Anzahl der Frames aus (8).

### APNG (`export ... out.apng`)

Animiertes PNG, 8-Bit-RGBA, verlustfrei, ein Frame pro Projekt-Frame mit der Projekt-fps (0.0417 s pro Frame bei 24 fps). Volles Alpha bleibt erhalten. Kein Audio. Größer
als GIF bei flacher Grafik (131 KB gegenüber 83 KB im Beispiel), weil es keine Palette gibt. Die meisten Browser spielen APNG ab; einige Werkzeuge nicht,
dann nimm WebM oder GIF. **Validierung:** `ffprobe` mit `-count_frames` (8 Frames beim Beispiel).

### WebM (`export ... out.webm`) und MP4 (`export ... out.mp4`)

```
$ osatura export small.json out.mp4  --scale 0.5     # h264, yuv420p, 320x240, 8 frames
$ osatura export small.json out.webm --scale 0.5     # vp9, yuv420p + alpha_mode=1
$ osatura export small.json out.webm --codec av1     # needs an ffmpeg with libaom-av1
```
* **MP4:** H.264 (`libx264`), `yuv420p`, **kein Alpha**. AAC-Audio, wenn das Projekt Tonspuren hat.
* **WebM:** VP9 (`libvpx-vp9`, Standard) oder AV1 (`libaom-av1`, `--codec av1`); der Alphakanal bleibt erhalten (`yuva420p`, Container-Tag
  `alpha_mode=1`); Opus-Audio (`libopus`), wenn das Projekt Sounds hat. Jeder andere `--codec`-Wert: `unknown --codec 'foo' (vp9 or av1)`.
* **Ton:** Die Tonspuren des Projekts werden gemischt (Start, Gain, Trim, Fades; an der Projektlänge abgeschnitten) und gemuxt.
* **Braucht ffmpeg:** Frames werden in einem privaten temporären Verzeichnis als PNG gerendert, an `ffmpeg` übergeben und danach entfernt. Wenn
  `ffmpeg` fehlt: `cannot run ffmpeg (...); install it or export a GIF/HTML instead` (Exit 1, nichts geschrieben). Fehlt
  der Encoder: `ffmpeg failed`.
* **Bekannte Grenzen:** nur 8 Bit; die Videogröße folgt der Projektgröße und `--scale` (prüfe das Ergebnis mit `ffprobe`).
  `dof-export` schreibt ebenfalls `.mp4`/`.webm`, ohne Audio.
* **Validierung:** `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4`
  gab `h264,320,240,yuv420p,24/1,8` aus.
* **Patente:** siehe Abschnitt 15; H.264 und AAC fallen unter Lizenzpools, VP9/AV1 wurden nicht bewertet.

## 5. HTML-Player

```
$ osatura export small.json out.html
wrote out.html                     # 66 KB for 8 frames
```
Eine eigenständige HTML-Datei: **jeder Frame als Inline-SVG**, eine Play/Pause-Schaltfläche, ein Scrubber (Range-Input) und eine Frame-Anzeige,
gesteuert von einem Inline-`<script>`. Sie stellt **keine Netzwerkanfrage**: Die einzige URL in der Datei ist der SVG-Namespace. Zum Öffnen braucht es
nichts als einen Browser. Das Bild ist genau das, was der SVG-Renderer erzeugt, daher ist es das originalgetreueste der „abspielbaren“
Formate (Masken, Blend-Modi, Effekte, Bones, IK, Deformer, Partikel sind alle in die Frames gebacken).

* **Audio:** Wenn das Projekt Sounds hat, wird das gemischte Audio als mp3-Data-URI eingebettet (braucht `ffmpeg`; eine Warnung ab 3 MiB, ab 16 MiB wird das
  Audio mit einer Warnung weggelassen).
* **Größe:** wächst linear mit den Frames (hier etwa 7.7 KB pro Frame) und mit dem Vektordetail; die Datei ist nicht komprimiert. Typische
  Tipps: gzip-komprimiert ausliefern (das 66-KB-Beispiel hat gzip-komprimiert etwa 9 KB), die Frame-Anzahl oder das Pfaddetail mit `lod` senken und
  `export-fit --format html` verwenden.
* **Grenzen:** `--report` wird abgelehnt (`unknown option --report for 'export .html'`);
  der Größenschutz für Vektor-Exporte gilt; der Player bietet nur Play/Pause und einen Scrubber.
* **Validierung:** öffnen; `grep -c '<svg' out.html` zählt die Frames; `grep -o 'http[^"<]*' out.html` sollte nur den SVG-Namespace auflisten.

## 6. Lottie (.json) und dotLottie (.lottie)

```
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ osatura export small.json out.lottie       # a zip: manifest.json + a/animation.json
```

### Wie die Datei aufgebaut ist

Die Datei ist Bodymovin (`"v": "5.7.0"`) mit `fr` = Projekt-fps, `ip` = 0, `op` = Frame-Anzahl und `w`/`h` = Bühnengröße. **Jede
Vektorebene wird zu einer Shape-Ebene, deren Pfadgeometrie pro Frame in den Weltraum gebacken wird (Hold-Keyframes)**, sodass Bones, IK,
Skinning, Deformer, Springs, Partikel, die Kamera (einschließlich Parallaxe) und Parent-Transformationen alle *in* der Geometrie stecken. Das Ergebnis
spielt wie das Original, ist aber kein editierbares Rig: Es gibt keine Bones, keine Kanäle, keine Ausdrücke. Eine einfarbige Hintergrundebene wird
geschrieben, wenn das Projekt eine Hintergrundfarbe hat. Folglich wächst die Datei mit `frames x anchors`; sie ist viel größer als ein
von Hand gebautes Lottie mit Transformationsanimation (47 KB für die 8-Frame-Demo, 11.5 KB innerhalb einer `.lottie`).

### Exportiert

Einfarbige Füllungen und Verlaufsfüllungen sowie Striche (Verlaufsstopps mit Alpha; die Endpunkte der Verläufe folgen der animierten Ebenentransformation), Strichelungen,
Caps, Joins und Miter-Limit, mit der Ebenenskalierung skalierte Strichbreiten, Blend-Modi, Ebenen-Opacity, Alpha- und invertierte-Alpha-
Track-Mattes für Ebenenmasken (eine Kopie der Maskenquelle dient als Matte), Ebeneneffekte (Blur, Schlagschatten, Tint, Fill) auf
Ebenen mit einem einzelnen Blatt, und **Gruppen-Opacity als Pre-Composition**, damit sich überlappende Children nicht gegenseitig durchscheinen. Zahlen werden
in ihrer kürzesten Dezimaldarstellung geschrieben.

### Mit Warnung (nie stillschweigend weggelassen)

Jeder der folgenden Punkte wird auf stderr als `warning: [code] layer: message` ausgegeben und nach `--report FILE` geschrieben als
`{"format":"lottie","count":N,"warnings":[{"code","layer","message"}]}`:

| Code | Bedeutung |
|---|---|
| `image-layer` | Bildebenen werden nicht exportiert (die Lottie-Ausgabe enthält nur Vektorebenen). |
| `variable-width-stroke` | Breitenprofile werden nicht unterstützt; die konstante Basisbreite wird verwendet. |
| `stroke-anisotropic` | Ungleichmäßige Skalierung oder Scherung auf der Ebene; Lottie-Striche sind einheitlich, daher wird die Breite mit dem geometrischen Mittel skaliert. |
| `gradient-anisotropic` | Ein Verlauf auf einer solchen Ebene wird angenähert (Lottie-Verläufe werden nicht geschert). |
| `blend-add` | Der Blend-Modus `add` hat kein Lottie-Äquivalent; stattdessen wird `screen` verwendet. |
| `group-blend` | Ein Blend-Modus auf einer Gruppe wird nicht exportiert. |
| `group-mask` | Eine Maske auf einer Gruppe wird nicht exportiert (Masken auf einzelnen Ebenen schon). |
| `group-effect` | Ebeneneffekte auf einer Gruppe (aus mehreren Ebenen) werden nicht exportiert. |
| `mask-source-not-vector` | Die Maskenquelle ist keine Vektorebene, daher wird die Maske nicht exportiert. |
| `depth-of-field` | Die Kanäle `camera_focus`/`camera_blur` werden nicht angewendet; verwende `dof-export`. |
| `audio` | Lottie kann kein Audio tragen; N Tonspur(en) sind nicht enthalten. |

### Ohne Lottie-Äquivalent weggelassen

Bones, IK, Kanäle, Zustandsautomaten, Datenbindungen, Container, Scroll-Bereiche, Marker/Events, Sounds, Post-Effekt-Ketten (`fx`),
Normal Maps und die editierbare Struktur des Rigs. Effektfarben pro Ebene können nicht animiert werden (der erste Wert wird verwendet), und
Größen sind in Bühneneinheiten (LIMITATIONS L-23).

### dotLottie (`.lottie`)

Ein [dotLottie-2.0](https://dotlottie.io/spec/2.0/)-Zip mit `manifest.json` (`{"animations":[{"id":"animation"}],"generator":"OSATURA","version":"2"}`) und
`a/animation.json` (dasselbe Lottie wie oben, mit Deflate komprimiert: aus 46,843 Bytes wurde eine Datei mit 11,515 Bytes). `--report` funktioniert genauso.
Das **Importieren** einer `.lottie` wird nicht unterstützt (`import-lottie` braucht das JSON; entpacke zuerst `a/animation.json`). Bis v0.1.0-alpha.4 war der Container dotLottie 1.0 (`animations/animation.json`, `autoplay`/`loop` im Manifest); das 2.0-Layout läuft in den aktuellen dotLottie-Playern (geprüft mit @lottiefiles/dotlottie-web).

### Validierung und Vorbehalte

* `python3 -m json.tool out.json > /dev/null` für die Wohlgeformtheit; `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],len(d['layers']))"`.
* `unzip -l out.lottie` muss genau die zwei Einträge zeigen.
* Round Trip: `osatura import-lottie out.json back.json` gefolgt von `osatura check back.json` verifiziert, dass OSATURA die eigene
  Ausgabe lesen kann (der Round Trip ist verlustbehaftet: Das Rig ist weg).
* **Es stand kein unabhängiger Lottie-Player zur Verfügung**, als dies geschrieben wurde (LIMITATIONS L-28): Die Ausgabe wurde gegen das Lottie-Feldlayout
  und gegen den eigenen Importer von OSATURA geprüft. Teste in deinem Ziel-Player (lottie-web, dotLottie-Player, Skottie, ...) und lies die
  Warnungen; wenn es auf Exaktheit ankommt, verwende `.html` oder Video.
* **Tipps zur Größe:** `export-fit --format lottie` durchsucht vier Stellschrauben (`decimals`, `path_eps`, `key_tol`, `frame_step`) gegen ein Byte-Budget;
  `lod` reduziert Anker vor dem Export; `size-report` zeigt die gebackene Lottie-Größe pro Ebene (Abschnitt 13).
* **Patente:** Lottie und dotLottie sind offene Formate; in PATENT_NOTES.md betrifft sie nichts.

## 7. glTF binär (.glb)

```
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
```
glTF 2.0 binär (Magic `glTF`, Version 2, `"generator": "osatura"`), nur Kernfunktionen. Das Projekt muss Bones haben
(`the project has no bones to export` sonst).

* **Exakt:** die Bone-Hierarchie und ihre Animation (bei jedem Frame abgetastet, lineare Interpolation, Shear weggelassen).
* **Füllungen** werden zu flachen Dreiecksnetzen (Ear Clipping). Formen sind starr (eine Ebene mit einem Bone folgt diesem Bone-Node) oder **geskinnt**
  (JOINTS_0/WEIGHTS_0, 4 Einflüsse pro Vertex), wenn jeder Anker Bone-Gewichte trägt.
* **Striche** werden in eigene Netze trianguliert (Breite, Breitenprofil, Butt-/Round-/Square-Caps, Miter-/Round-/Bevel-Joins, Strichelungen);
  durchscheinende Striche können dunkler aussehen, wo sich Dreiecke überlappen.
* **Verläufe** (linear und radial) werden zu einem kleinen generierten PNG, das als Basisfarben-Textur verwendet wird; Pad-Ausbreitung (Clamp).
* **Masken** werden durch Beschneiden von Dreiecken gegen die Füllung der Maskenebene gebacken, nur für eine Vektor-Maskenquelle mit deckenden Füllungen und ohne
  Striche, und nur für die Maskenform von Frame 0. **Bildebenen** werden zu texturierten Quads oder Deformationsnetzen mit eingebettetem PNG.
* **Pfad-Morphs** werden zu Morph-Targets mit animierten Gewichten, wenn die Prüfung pro Frame die Geometrie der Engine reproduziert; andernfalls
  bleibt der Pfad eine statische Form von Frame 0, und der Grund wird gemeldet.
* **Nicht exportiert (als `[glb-limit]`-Warnungen gemeldet):** andere Blend-Modi als normal (als normal gezeichnet), animierte Ebenen-Opacity (bei
  Frame 0 eingefroren), Transformationsanimation von Ebenen ohne Bone, Ebeneneffekte, Deformer über ihre Form von Frame 0 hinaus, animierte Pfade ohne gültigen
  Morph, animierte Maskenformen, Sound.
* **Koordinaten:** y wird negiert (glTF ist y-up); Formen werden mit einem kleinen z-Versatz (0.01 pro Form) für die Zeichenreihenfolge geschichtet.
* **Validierung:** Der Header ist `glTF`, Version 2, und das Längenfeld entspricht der Dateigröße (`python3`-Einzeiler in Abschnitt 16). Verwende einen glTF-
  Validator oder Viewer deiner Wahl für die vollständige Prüfung; keiner ist mitgeliefert, und für dieses Dokument wurde keiner ausgeführt. Das obige Beispiel hat 22 Nodes,
  15 Meshes, 1 Skin, 1 Animation und 7 eingebettete Bilder.
* **Tipps zur Größe:** weniger Vektoranker (`lod`), keine Verläufe (jeder ist eine Textur), weniger Frames (die Animation wird pro Frame abgetastet).
* **Patente:** offenes Format; kein Hinweis in PATENT_NOTES.md.

## 8. Ordner im Open-Cel-Animation-Format (OCA)

```
$ osatura export-oca small.json oca1                 # one layer "render" with all composited frames
$ osatura export-oca small.json oca2 --layers        # one OCA layer per top-level layer
wrote oca2/small.oca
```
Schreibt `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus PNG-Cels `layerN/fNNNN.png`. Ohne `--layers` gibt es eine Ebene
`render` mit den zusammengesetzten Frames; mit `--layers` wird jede Ebene der obersten Ebene allein auf Transparenz gerendert (11 Ebenen bei der Demo).
`--scale N` multipliziert die Cel-Größe. **Bekannte Auslassungen:** Cels sind Raster (keine Vektoren, Bones oder Kanäle bleiben erhalten); Ebenen-Opacity und Blend
werden nicht geschrieben (jede Ebene ist ein `paintlayer`, Opacity 1); Sounds werden nicht exportiert; Dateien in einem bestehenden Ordner werden überschrieben.
**Import:** Es gibt in der CLI keinen Befehl `import-oca`. **Validierung:** Die `.oca`-Datei ist JSON (`python3 -m json.tool`); jedes aufgeführte Cel
muss als PNG existieren.

## 9. Import: SVG, OpenRaster, Lottie, Bilder, GIF/APNG

Importe erzeugen ein **neues** Projekt (sie werden nicht in ein bestehendes zusammengeführt), sofern nicht anders angegeben; nutze `script` oder bearbeite das JSON, um sie zu kombinieren.

### SVG (`import-svg`)

```
$ osatura import-svg art.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
wrote svg.json (3 paths)
```
Teilmenge: `path` (alle Befehle), `rect` (abgerundete Ecken werden ignoriert), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` mit `transform`,
einfarbiges `fill`, `stroke`, `stroke-width`. Ergebnis: 24 fps, 24 Frames, Größe aus `viewBox` oder `width`/`height` (sonst 512x512), **eine Vektor-
Ebene `svg`**. Mit einer `warning:`-Zeile (auf **stdout** ausgegeben) übersprungen: `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`,
`<linearGradient>`/`<radialGradient>` (eine Verlaufsfarbe fällt auf Grau zurück), `<image>`, `<use>`, `<foreignObject>`. Grenzen: 8 MB SVG,
200,000 Anker. Für Fotos und Raster verwende `trace` / `autotrace` (CLI_REFERENCE Abschnitt 9).

### OpenRaster (`import-ora`)

```
$ osatura import-ora art.ora ora.json --fps 12 --frames 24
```
Eine **Bildebene pro Rasterebene** (eingebettetes PNG), Reihenfolge von unten nach oben, versteckte Ebenen bleiben erhalten, sind aber unsichtbar, Gruppen werden verflacht, Opacity,
Sichtbarkeit und Position pro Ebene sowie die gängigen `svg:`-Kompositionsoperationen (multiply, screen, overlay, darken, lighten, color-dodge,
color-burn, hard-light, soft-light, difference, exclusion, plus wird auf add abgebildet). Die Opacity eines verschachtelten Stapels multipliziert sich in seine Ebenen, und sein
Blend-Modus wird vererbt, aber überlappende Ebenen innerhalb eines durchscheinenden Stapels werden **nicht** zuvor verflacht. Doppelte Namen erhalten `_1`, `_2`.
Grenzen: 2000 Archiveinträge, 128 MiB pro Eintrag, Ebenen mit 64 Mpixel. Eine Datei, die kein Zip ist: `not a zip/ORA file: ...` (Exit 1). Es gibt keinen ORA-
Export. Der PSD-Import ist nicht implementiert (LIMITATIONS L-32).

### Lottie (`import-lottie`)

```
$ osatura import-lottie in.json out.json [fx.json]
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
```
Eine bewusste **Teilmenge**; jede übersprungene Funktion ist eine `warning:`-Zeile.
* **Importiert:** Shape-Ebenen mit `sh`-Pfaden, `rc`, `el`; einfarbige Füllungen und Verlaufsfüllungen (animierte Verlaufs-Endpunkte/Mittelpunkt/Radius werden zu
  `grad_dx`/`grad_dy`-Kanälen); Striche; Gruppen; Ebenentransformationen (statische oder animierte Position, Skalierung, Rotation, Opacity; gekrümmte Positions-
  pfade werden pro Frame gebacken); Blend-Modi; eine einfarbige Hintergrundebene; Bildebenen mit eingebetteten PNG-/JPEG-Data-URIs; Null-Ebenen als Parents
  (ihre Transformationen werden in die Children gebacken); Ebenenmasken (add/subtract/intersect mit Opacity, Invertierung, animierte Formen); Luma-Mattes
  (angenähert); Pre-Compositions; In-/Out-Bereiche von Ebenen (als Opacity-Step-Spur); Keyframe-Easing (Hold wird zu step, Bézier-Handles werden zu
  cubic-bezier, keines wird zu linear).
* **Animierte Pfade:** Shape-Keys mit gleicher Topologie (gleiche Ankeranzahl und offen/geschlossen) werden zu einem Basispfad plus einem **Morph-Target**
  pro weiterem unterschiedlichen Key (`k1`, `k2`, ...) mit `path_morph`-Gewichtskanälen, die das Easing tragen. Eine andere Topologie verwendet die erste Form
  und gibt eine Warnung aus.
* **Effekte (`ef`):** Gaußscher Blur, Schlagschatten, Tint und Fill werden zu **Effekten pro Ebene**, wobei animierte Zahlen zu `fx_*`-Kanälen werden;
  statische Tint-/Fill-/Tritone-/Levels-Effekte auf Shape-Ebenen werden in Farben gebacken. Effektfarben können nicht animiert werden (der erste Wert wird verwendet); andere
  Effekte sind Warnungen.
* **Mit Warnung übersprungen:** Text und andere Ebenentypen, Audio-Assets, animierte Ankerpunkte (erster Wert wird verwendet), gekrümmte Bewegungspfade auf anderen
  Eigenschaften als der Position, Masken auf einer Track-Matte-Quelle, zu lange Parent-Ketten zum Backen, mehr als 2000 Ebenen, Animationen länger
  als 20,000 Frames.
* `fx.json` bleibt aus Kompatibilitätsgründen erhalten: Es wird ein leeres Array geschrieben.
* **Nicht unterstützt:** `.lottie`-Zip-Eingabe. **Stolperfalle:** Es gibt keinen Überschreibschutz gegenüber der Eingabe: `import-lottie in.json in.json` ersetzt das
  Lottie durch das Projekt.

### Bilder, GIF und APNG als Frame-für-Frame-Ebenen

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20      # folder of PNGs
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0                    # animated GIF/APNG
```
`import-seq` nimmt alle PNGs eines Ordners in der Reihenfolge der Dateinamen (alle gleich groß); `import-anim` liest ein animiertes GIF oder APNG mit den
eigenen Verzögerungen der Datei (ein einfaches PNG ergibt einen Frame). Beide fügen eine Switch-Ebene mit einem eingebetteten Bild pro Frame plus einem Step-Kanal hinzu; sie arbeiten an einem
**bestehenden** Projekt. Ein langes GIF macht das Projekt groß (jeder Frame ist ein eingebettetes PNG). `trace` und `autotrace` wandeln Bitmaps in Vektor-
pfade um (`trace` liest den Alphakanal; `autotrace` quantisiert Farben und passt Bézier-Kurven an).

## 10. Import: BVH-Motion-Capture, MIDI und andere Live-Eingaben

### BVH (`import-bvh`, `mocap-info`, `mocap-match`, `mocap-retarget`)

```
$ osatura mocap-info walk.bvh
$ osatura import-bvh demo.json walk.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```
`import-bvh` bildet ausgewählte Gelenk-Rotationskanäle auf Bones ab, als animierte `bone_angle`-Kanäle, abgetastet mit der Projekt-fps und Frame-Anzahl
(`--axis` wählt `Zrotation`, `Xrotation` oder `Yrotation`; `--tol` vereinfacht Keys). `mocap-match` schlägt eine Zuordnung von Gelenken zu Bones mit einer
Konfidenztabelle vor, und `mocap-retarget` backt angehängte Kanäle mit Root-Motion, einer Ansicht (`front|side|top|yaw=DEG`) und Skalierung. Das BVH
ist nur als Textteilmenge unterstützt: keine binären Mocap-Formate, kein FBX (gewollt). **Grenzen (LIMITATIONS L-45):** Es wurde keine echte Motion-Capture-Datei getestet; die 2D-
Projektion verwirft die Tiefe; Verdrehung um eine Bone-Achse, Finger und Gesicht werden ignoriert; das Rig behält seine Bone-Längen, daher wird das Fußgleiten nur
verringert.

### MIDI und andere Live-Eingaben (`live-map`)

```
$ osatura live-map small.json song.mid mapping.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote out.json
```
Eine **Standard MIDI File** (`.mid`; MIDI-1.0-Nachrichten, Running Status, Tempo-Meta-Events, Ticks oder SMPTE-Division) oder ein Events-JSON
(`{"events":[{"source","id","value","time"}]}`, Wert normalisiert 0..1, Zeit in Sekunden) wird über eine Mapping-Datei in animierte Kanäle, Events und Takes umgewandelt.
Quellen: `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`. Höchstens
3,000,000 Events; Zeiten innerhalb von 7 Tagen. Nichts spricht von der CLI aus mit einem MIDI-Gerät; das Studio nutzt das Web MIDI des Browsers. MIDI ist **nur
Import** (es gibt keinen MIDI-Export). Fehlerhafte Dateien liefern einen Fehler, nie einen Absturz.

## 11. Audioformate

| Verwendung | Formate | Hinweise |
|---|---|---|
| Analyse (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) | nur **WAV** | 8/16/24/32-Bit-Integer- oder 32-Bit-Float-PCM, beliebige Kanalanzahl auf Mono gemischt, bis zu 256 MiB. Konvertiere MP3/OGG zuerst (`ffmpeg -i in.mp3 out.wav`). |
| Tonspuren des Projekts (`sound add`) | WAV, OGG, MP3, alles, was `ffprobe` akzeptiert | Wird als **Referenz** gespeichert (relativer Pfad, Startframe, Gain 0..4, Trim-Fenster, Fades); nichts wird eingebettet; die Datei muss im Projektordner bleiben, sofern nicht `--allow-external-assets` angegeben ist. |
| Gemischte Ausgabe (`sound mix`) | `.wav` (44.1 kHz PCM 16-Bit), `.ogg` (Vorbis q4), `.mp3` (96 kb/s) | Über ffmpeg für die **Projektlänge** gemischt; `.flac` und andere: `unknown audio format '.flac' (use .wav, .ogg or .mp3)`. |
| In Exporten | `.mp4` AAC, `.webm` Opus, `.html` eingebettetes mp3 | `.gif`/`.apng` bleiben mit einem Hinweis stumm; Lottie warnt mit `[audio]`; GLB und OCA lassen Sound weg. |
| Mundsignale (`mouth`) | Papagayo-/Moho-Switch-Text, Rhubarb-TSV | Kein Audio: Cue-Dateien. |

Verwendete Encoder: `libmp3lame` (mp3), `libvorbis` (ogg), `aac`, `libopus`. Kein Pan/EQ/Ducking oder Looping pro Spur (ROADMAP Abschnitt 9).

## 12. Matrix der Funktionsunterstützung

Zeilen sind Dinge, die ein Projekt enthalten kann, Spalten die Ausgabeformate. **yes** = wie im Projekt übernommen, **approx** = mit sichtbarem
Unterschied übernommen, **baked** = nur als Ergebnisse pro Frame vorhanden, **no** = weggelassen. Zellen für Formate, die ich nicht in einem unabhängigen Viewer prüfen konnte,
beschreiben, was der Exporter schreibt.

| Funktion | SVG / PNG | Sprite-Sheet / APNG | GIF | WebM / MP4 | HTML | Lottie / .lottie | GLB | OCA |
|---|---|---|---|---|---|---|---|---|
| Vektorformen, Füllungen | yes | baked (raster) | baked (raster) | baked (raster) | yes (SVG) | yes | approx (triangles) | baked (raster) |
| Verläufe | yes | baked | baked | baked | yes | yes (approx if layer skewed) | yes (texture) | baked |
| Striche mit Breitenprofil | yes | baked | baked | baked | yes | approx (constant width) | yes | baked |
| Strichelungen, Caps, Joins | yes | baked | baked | baked | yes | yes | yes (triangulated) | baked |
| Bild-(Bitmap-)Ebenen | yes | baked | baked | baked | yes | no (`image-layer`) | yes | baked |
| Bones und Animation | baked | baked | baked | baked | baked | baked | **yes (exact)** | baked |
| Skinning / Deformer | baked | baked | baked | baked | baked | baked | skinned; deformers at frame 0 | baked |
| Kamera und Parallaxe | baked | baked | baked | baked | baked | baked (camera moves are in the geometry) | no (shapes come from frame 0) | baked |
| Blend-Modi | yes | baked | baked | baked | yes | approx (`add` becomes `screen`; none on groups) | no (normal) | baked |
| Ebenenmasken | yes | baked | baked | baked | yes | single layers only | frame-0 shape, restricted | baked |
| Ebeneneffekte (Blur, Schatten...) | yes | baked | baked | baked | yes | single-leaf layers | no | baked |
| Animierte Opacity | baked | baked | baked | baked | baked | yes | no (frozen at frame 0) | baked |
| Pfad-Morphs | baked | baked | baked | baked | baked | baked | morph targets (when valid) | baked |
| Schärfentiefe | via `dof-export` | via `dof-export` | via `dof-export` | via `dof-export` | no | no (warned) | no | no |
| Post-Effekt-Kette (`fx`) | PNG only | no | no | no | no | no | no | no |
| Transparenz | yes | yes (alpha) | 1-bit | WebM yes, MP4 no | yes | yes | yes | yes |
| Audio | no | no | no | **yes** | yes (mp3) | no (warned) | no | no |
| Looping | n/a | sheet: n/a, APNG: yes | yes | player-dependent | player | Player (dotLottie 2.0 überlässt die Schleife dem Player oder einer Zustandsmaschine) | n/a | n/a |
| Rig/Kanäle danach editierbar | no | no | no | no | no | no | bones only | no |
| Zustandsautomaten, Bindings, Container, Scroll | no | no | no | no | no | no | no | no |
| Auflösungsunabhängig | yes (SVG) | no | no | no | yes | yes | yes | no |
| Braucht ffmpeg | no | no | no | **yes** | audio only | no | no | no |

„Baked“ bedeutet, dass die Wirkung der Funktion in den Pixeln oder der Geometrie pro Frame steckt, die Funktion selbst aber danach nicht mehr bearbeitet werden kann.

## 13. Dateien verkleinern

1. **Sieh nach, wo die Bytes stecken:** `osatura size-report project.json` listet die Projektdatei, das gebackene Lottie und die HTML-Seite nach Ebene,
   Bild und Kanal auf (`--json` für Skripte).
2. **Ein Budget automatisch einhalten:** `osatura export-fit project.json out.json --max-size 50KB --format lottie` durchsucht die Stellschrauben des Exporters und
   behält den besten SSIM, der passt. Formate: `lottie`, `html`, `gif`, `apng`, `webm`, `project` (ein abgespecktes Projekt-JSON). `--measure gzip` budgetiert die
   gzip-Größe, `--min-ssim X` setzt eine Qualitätsuntergrenze, `--max-seconds N` ist ein echtes Wanduhr-Limit (das bisher beste Ergebnis bleibt erhalten). Exit 1 mit dem
   kleinsten Ergebnis, wenn das Budget nicht eingehalten werden kann.
3. **Zuerst das Artwork reduzieren:** `osatura lod project.json --tolerance-px 1 --levels 3 --out-dir lv --verify` schreibt Level-of-Detail-Projekte mit einem
   deklarierten (und mit `--verify` gemessenen) Fehler in Ausgabepixeln; `lod-pick --screen-px WxH` wählt die gröbste Stufe für einen gegebenen Bildschirm.
4. **Pro Format:**
   * Lottie/HTML/GLB wachsen mit `frames x anchors`: weniger Frames, weniger Anker, eine niedrigere fps, gekürzte Key-Bereiche (`keys`-Werkzeuge).
   * GIF/APNG wachsen mit Pixel x Frames: `--scale 0.5` ist eine 4-fache Verkleinerung; GIF eignet sich für flache Grafik, APNG für Alpha, WebM/MP4 für alles mit Bewegung.
   * `.lottie` ist dasselbe Lottie mit Deflate (im Beispiel etwa 4-mal kleiner); liefere `.html`/`.json` gzip-komprimiert aus.
   * Bilder in einem Projekt werden als Base64-PNG eingebettet: verkleinere sie vor dem Import.
5. Grenzen: Ein Lottie eines Rigs mit 14 Ebenen lässt sich nicht beliebig klein machen (ein 1-KB-Budget beim Beispiel meldet „every knob is at its floor“); die
   Suche braucht bei großen Projekten Zeit.

## 14. ffmpeg-Anforderungen

OSATURA bündelt oder linkt ffmpeg nie; es führt das im `PATH` gefundene `ffmpeg` (und `ffprobe`) als **externen Subprozess** deiner eigenen Installation aus.

| Benötigt für | Encoder/Decoder | Geprüft mit |
|---|---|---|
| `export .mp4` | `libx264` (`yuv420p`), `aac` | ffmpeg 6.1.1 |
| `export .webm` (Standard) | `libvpx-vp9`, `libopus` | ffmpeg 6.1.1 |
| `export .webm --codec av1` | `libaom-av1`, `libopus` | ffmpeg 6.1.1 |
| `dof-export .mp4/.webm` | wie oben, `--codec` | ffmpeg 6.1.1 |
| `sound mix .ogg` | `libvorbis` | ffmpeg 6.1.1 |
| `sound mix .mp3`, `.html`-Audio | `libmp3lame` | ffmpeg 6.1.1 |
| `face-track` aus einem Video | nur Decoder | |
| Inhaltsprüfung bei `sound add` | `ffprobe` (ohne es wird bei Nicht-WAV-Dateien nur die Existenz geprüft) | |

Prüfe deines mit `ffmpeg -hide_banner -encoders | grep -E 'libx264|aac|libvpx-vp9|libaom-av1|libopus|libvorbis|libmp3lame'`. Alles andere
(GIF, APNG, HTML ohne Audio, Lottie, dotLottie, GLB, OCA, PNG, SVG, EXR) funktioniert ohne ffmpeg.
Wenn ffmpeg fehlt, scheitert der Befehl mit `cannot run ffmpeg (...)` und schreibt nichts; ein fehlender Encoder ergibt `ffmpeg failed`.

## 15. Hinweise zu Patenten und Lizenzen

* [PATENT_NOTES.md](PATENT_NOTES.md) hält fest, was **nicht** geprüft wurde. Für den Export ist die relevante Zeile *Video- und Audiocodecs über ffmpeg
  (`export` .mp4 / .webm, `soundtrack`)*: H.264 und AAC fallen unter Lizenzpools; VP9 und AV1 werden als lizenzgebührenfrei vermarktet, aber für beide wurden Pool- und Einzel-
  ansprüche angekündigt; **Status: nicht bewertet, VP9/AV1 nicht als freigegeben behandeln.** Weil ffmpeg die eigene Installation des Nutzers ist, liegt jede
  Codec-Wahl in der Verantwortung des Nutzers, und OSATURA darf ffmpeg nicht bündeln.
* Die deterministischen Writer für Raster, GIF, APNG, PNG, SVG, OpenEXR, OCA, ORA, glTF und Lottie sind eigene Implementierungen von OSATURA nach öffentlichen
  Formatbeschreibungen (Clean-Room, siehe CLEANROOM.md). Ihr Patentstatus ist darüber hinaus nicht bewertet: PATENT_NOTES.md listet nur die oben genannten Bereiche
  (gesichtsgesteuertes Puppeteering, Zustandsautomaten und Datenbindung, FABRIK/MVC/Gewichte, Stencil-then-Cover-Füllung, Codecs).
* GIF: Die LZW-Patente sind in allen Rechtsordnungen längst abgelaufen; das ist Allgemeinwissen, keine im Repository festgehaltene Freigabe.
* Lizenzen der erzeugten Dateien: Dateien, die du exportierst, gehören dir; die Engine steht unter MPL-2.0 und die CLI unter AGPL-3.0-or-later, was für die Programme gilt, nicht für die
  Dateien, die sie schreiben. Das Studio bietet seinen zugehörigen Quellcode unter `/source` an (AGPL Abschnitt 13).
* Dies ist keine Rechtsberatung.

## 16. Das Geschriebene validieren

| Format | Prüfung | Erwartung |
|---|---|---|
| jedes Projekt | `osatura check out.json` | `ok: ...` (auch nach `import-*`) |
| GIF | `ffprobe -v error -count_frames -select_streams v -show_entries stream=nb_read_frames -of csv=p=0 out.gif` | Frame-Anzahl (8) |
| APNG | derselbe Befehl auf `out.apng` | Frame-Anzahl (8) |
| MP4 | `ffprobe -v error -show_entries stream=codec_name,width,height,pix_fmt,nb_frames,r_frame_rate -of csv=p=0 out.mp4` | `h264,320,240,yuv420p,24/1,8` |
| WebM | dasselbe auf `out.webm` mit `:stream_tags=alpha_mode` | `vp9,...,yuv420p,...,1` (alpha_mode 1) |
| EXR | `ffprobe f.exr` | `exr 640x480 gbrapf32le` |
| Lottie | `python3 -c "import json;d=json.load(open('out.json'));print(d['v'],d['fr'],d['op'],d['w'],d['h'],len(d['layers']))"` | `5.7.0 24.0 8 640.0 480.0 12` |
| Lottie-Warnungen | `osatura export p.json out.json --report rep.json` | `rep.json` listet jede Annäherung auf |
| dotLottie | `unzip -l out.lottie` | nur `manifest.json` und `a/animation.json` |
| Lottie-Round-Trip | `osatura import-lottie out.json back.json && osatura check back.json` | `ok:` |
| GLB | `python3 -c "import struct;b=open('out.glb','rb').read();print(b[:4],struct.unpack('<II',b[4:12]),len(b))"` | `b'glTF' (2, N)` mit N gleich der Länge |
| HTML | `grep -c '<svg' out.html`; `grep -o 'http[^"<]*' out.html` | ein `<svg` pro Frame; nur die URL des SVG-Namespace |
| SVG | im Browser öffnen oder als XML parsen | wohlgeformt |
| Sprite-Sheet | `python3 -m json.tool out.atlas.json` | `{index,x,y,w,h}` pro Frame |
| OCA | `python3 -m json.tool <name>.oca` | jedes referenzierte Cel existiert |
| Visuelle Regression | `osatura ci diff a.png b.png --metric ssim` | Exit 0, wenn gleich; siehe [CI.md](CI.md) |

Für visuelle Prüfungen eines verlustbehafteten Exports gegen das Original meldet `osatura export-fit` bereits SSIM und CIEDE2000 gegenüber dem ursprünglichen Render, und
`osatura compare` schreibt ein Vorher-/Nachher-/Differenzbild zweier Projekte.

## 17. Unterschiede zu anderen Dokumenten

* `docs/LIMITATIONS.md` L-29 sagt, der Lottie-Import nehme keine Bild-Assets, Effekte, Precomps oder Mattes auf; der aktuelle Importer importiert Bilder
  mit eingebetteten Data-URIs, Pre-Compositions, Effekte pro Ebene (Blur, Schlagschatten, Tint, Fill) und nähert Luma-Mattes an (Abschnitt 9). Die Liste der
  übersprungenen Funktionen in Abschnitt 9 ist das, wovor der Befehl tatsächlich warnt.
* `docs/LIMITATIONS.md` L-32 führt OCA unter den Importformaten auf; die CLI hat einen OCA-**Export** (`export-oca`) und keinen Befehl `import-oca`.
* Der Usage-Text von `export` nennt nur `.gif .html .json .mp4` plus „die offenen Formate“; die vollständige Liste der Endungen steht in Abschnitt 2.
* Weitere Abweichungen zwischen Usage-Text und Verhalten sind am Ende von [CLI_REFERENCE.md](CLI_REFERENCE.md) gesammelt.
