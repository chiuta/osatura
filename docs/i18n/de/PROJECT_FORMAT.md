<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# OSATURA-Projektformat

Dies ist die Referenz für die OSATURA-Projektdatei: ein einzelnes JSON-Dokument, das
die Bühne, das Rig, die Grafik und die Animation enthält. Sie ist anhand der serde-Structs in
`osatura-core/src/project.rs` und der von ihnen referenzierten Module geschrieben,
und jedes Beispiel in den Abschnitten 13 bis 15 wurde mit dem `osatura`-Binary geprüft
(`osatura check`, `osatura render`, `osatura explain`).

Inhalt

1. [Konventionen und primitive Typen](#1-konventionen-und-primitive-typen)
2. [Laden, Versionierung und unbekannte Felder](#2-laden-versionierung-und-unbekannte-felder)
3. [Felder der obersten Ebene](#3-felder-der-obersten-ebene)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Ebenen](#5-ebenen)
6. [Pfade, Anker, Paint, Stroke](#6-pfade-anker-paint-stroke)
7. [Deformer, Effekte, Normal Map](#7-deformer-effekte-normal-map)
8. [Kanäle, Tracks, Easing](#8-kanäle-tracks-easing)
9. [Ausdrücke](#9-ausdrücke)
10. [Kamera, Shots, Audio, Metadaten](#10-kamera-shots-audio-metadaten)
11. [Clips, Skins, Links, Interaktion, LODs](#11-clips-skins-links-interaktion-lods)
12. [Validierungsregeln, Grenzen und Fehlermeldungen](#12-validierungsregeln-grenzen-und-fehlermeldungen)
13. [Beispiel 1: springender Ball](#13-beispiel-1-springender-ball)
14. [Beispiel 2: gerigter Arm mit IK](#14-beispiel-2-gerigter-arm-mit-ik)
15. [Beispiel 3: Mund als Switch-Ebene](#15-beispiel-3-mund-mit-switch-ebene)
16. [Auswertungsreihenfolge](#16-auswertungsreihenfolge)
17. [Rendering-Reihenfolge](#17-rendering-reihenfolge)
18. [Assets und Pfade](#18-assets-und-pfade)
19. [Versionierungsrichtlinie](#19-versionierungsrichtlinie)
20. [Bekannte Abweichungen zwischen Code und Kommentaren](#20-bekannte-abweichungen-zwischen-code-und-kommentaren)

## 1. Konventionen und primitive Typen

| Typ | JSON-Form | Hinweise |
| --- | --- | --- |
| number | JSON-Zahl | Als `f32` gespeichert, sofern das Feld nicht Ganzzahl sagt. Jeder Float muss `abs(x) <= 1e7` erfüllen (Abschnitt 12). |
| `Vec2` | `[x, y]` | Zwei Zahlen. Bühneneinheiten sind Pixel, +x rechts, +y unten. |
| `Rgba` | string | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa` oder eines von `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. Die Ausgabe ist immer Hex in Kleinbuchstaben. |
| `Paint` | string oder object | Ein Farb-String ist eine einfarbige Füllung. `{"from": [x,y], "to": [x,y], "stops": [...]}` ist ein linearer Verlauf. `{"center": [x,y], "radius": r, "stops": [...]}` ist ein radialer Verlauf. Der Typ ist ein untagged Enum. |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Stops werden beim Zeichnen (stabil) sortiert, und Offsets werden im Rasterizer auf 0..1 begrenzt. Ein degenerierter Verlauf malt den letzten Stop. |
| angle | number | In der Datei überall Grad (Knochenwinkel, Ebenenrotation, Skew, Kamerarotation, `angle_limits`). |
| time | number | Frames in Keys und Markern, Sekunden in Sounds, Automaten und Emittern. Jedes Feld sagt, welche. |
| names | string | Namen von Knochen, Ebenen, Pfaden, Deformern, IK, Constraints. Die Abschnitte 5 und 12 nennen die Zeichenbeschränkungen. |

Eine ungültige Farbe in einem Paint-Kontext erzeugt die serde-Meldung
`data did not match any variant of untagged enum Paint`, weil `Paint` untagged ist und serde
nicht sagen kann, welcher Zweig gemeint war.

Die Feldtabellen unten verwenden diese Notation: **Req** bedeutet, dass das Feld keinen Standardwert hat und vorhanden sein muss;
andernfalls wird der Standardwert angegeben. „Range“ ist das, was der Loader oder die Engine erzwingt. Ein als
„clamped“ angegebener Bereich ist kein Fehler: Die Engine begrenzt den Wert stillschweigend.

## 2. Laden, Versionierung und unbekannte Felder

`Project::from_json(text)` ist der einzige Einstiegspunkt, den die CLI, die C-API, das WASM-Modul
und das Studio verwenden. Er tut der Reihe nach Folgendes:

1. Das JSON mit serde parsen. Serde ignoriert Felder, die es nicht kennt.
2. Die Größenprüfung ausführen: Jedes Float-Literal, dessen Betrag über `1e7` liegt, lässt das Laden
   mit `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...` scheitern. Ganzzahl-
   Literale werden später durch die erneute Prüfung in `Engine::new` abgefangen (siehe Abschnitt 12).
3. Das Format-Gate anwenden: Ein `format` größer als die `FORMAT_VERSION` des Builds (derzeit `1`) wird
   mit `project format 2 is newer than this build supports (1); update OSATURA` abgelehnt.

`Engine::new` (oder `Engine::with_asset_dir`) validiert dann das gesamte Projekt und gibt jedes
gefundene Problem als Liste zurück. Die CLI gibt sie so aus:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

Fehlertexte sind für Menschen. Sie sind kein stabiler Vertrag; gleiche den Text nur in Tests ab.

### Unbekannte Felder

Serde verwirft unbekannte Schlüssel beim Laden, ein Tippfehler wie `fil` statt `fill` ginge also stillschweigend verloren. Das
Modul `unknownfields` vergleicht das Eingabe-JSON mit dem erneut serialisierten Projekt und meldet Schlüssel, die
nicht überlebt haben:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` gibt diese Warnungen aus. Werte, die dem Standard entsprechen, werden nicht gemeldet (der Writer
lässt Standardwerte weg, ein explizites `"visible": true` ist also in Ordnung). Einige Strukturen sind stattdessen strikt statt
nachsichtig und lehnen unbekannte Schlüssel mit einem harten Fehler ab: Automaten-`Transition` und -`Condition`, CI-Konfiguration,
Live-Input-Konfiguration, Overlays und die Konfiguration der Gesichtsparameter; Konfigurationsdateien werden mit `parse_strict` gelesen.

### Round Trip der Editor-Metadaten

Das Projekt serialisiert optionale Listen mit `skip_serializing_if`. Eine von Hand geschriebene Datei kann
kürzer sein als das, was `osatura` zurückschreibt; beide sind gleichwertig.

## 3. Felder der obersten Ebene

| JSON-Name | Typ | Standard | Bereich / Regel | Bedeutung |
| --- | --- | --- | --- | --- |
| `format` | integer | `FORMAT_VERSION` (1) | muss `<=` der Version des Builds sein | Formatversion, Abschnitt 19. |
| `name` | string | **Req** | beliebig | Anzeigename. Wird von `check` ausgegeben. |
| `width` | number | **Req** | `> 0`, `<= 32768` | Bühnenbreite in Pixeln. |
| `height` | number | **Req** | `> 0`, `<= 32768` | Bühnenhöhe in Pixeln. |
| `fps` | number | **Req** | `(0, 1000]` | Frames pro Sekunde. |
| `frames` | integer (`u32`) | **Req** | `1..=200000` | Länge der Timeline. Frame-Indizes sind `0..frames-1`. Ein Float-Literal wie `48.0` wird abgelehnt: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` oder null | null (transparent) | | Bühnenfarbe hinter allem. |
| `color_space` | string | `"srgb"` (nicht geschrieben) | `srgb`, `linear` | Raum, in dem der Raster- und der Float-Compositor mischen. `linear`: Farben werden in lineares Licht umgerechnet, in f32 zusammengesetzt und einmal zurückgerechnet (ein 50-%-Schwarz über Weiß ergibt 188/255, nicht 128/255). Setzbar mit `osatura color-space`. SVG, Lottie, der GPU-Renderer, `dof` und `fx` ignorieren es. |
| `camera` | object | Standardwerte | Abschnitt 10 | Globale Kamera. |
| `bones` | array | `[]` | höchstens 4096 | Skelett, Abschnitt 4. |
| `ik` | array | `[]` | | IK-Ketten. |
| `constraints` | array | `[]` | | Aim, Copy-Rotation, Pin. |
| `springs` | array | `[]` | | Sekundärbewegung, beim Laden gebacken. |
| `layers` | array | `[]` | | Wurzelebenen, in Listenreihenfolge gezeichnet. |
| `channels` | array | `[]` | | Animationskanäle, in Listenreihenfolge ausgewertet. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | arrays | weggelassen, wenn leer | nicht validiert | Timeline- und Editor-Metadaten, Abschnitt 10. |
| `sounds` | array | weggelassen, wenn leer | validiert | Audiospuren, Abschnitt 10. |
| `skins`, `clips`, `clip_baked`, `lods` | arrays | weggelassen, wenn leer | beim Laden nicht validiert | Abschnitt 11. |
| `mesh_locks`, `glues` | arrays | weggelassen, wenn leer | validiert | Mesh-Verknüpfungen über Ebenen hinweg, Abschnitt 11. |
| `machines`, `bindings`, `containers`, `scrolls` | arrays | weggelassen, wenn leer | validiert | Interaktion und Layout, Abschnitt 11. |

Minimal gültiges Projekt:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Größenordnungen: Jeder Float in der Datei muss `abs(x) <= 1e7` erfüllen. Das gilt gleichermaßen für Koordinaten,
Skalierungen, Offsets und Key-Werte. Breite und Höhe sind zusätzlich auf 32768 begrenzt, und die Raster-
Ausgabe auf 16384 pro Seite und 100 Megapixel (Abschnitt 18).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

Knochen müssen mit den Eltern zuerst aufgelistet sein. Der Elternknochen eines Knochens muss weiter vorn in der Liste stehen.

| JSON-Name | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | eindeutig | Wird von Gewichten, ik, Kanälen und `Layer.bone` referenziert. |
| `parent` | string oder null | null | ein früherer Knochen | Elternknochen. Null bedeutet eine Wurzel. |
| `length` | number | **Req** | endlich, `>= 0` | Knochenlänge in Pixeln, entlang der +x-Achse des Knochens. |
| `rest_offset` | `Vec2` | `[0,0]` | | Kopfposition im Raum des Elternknochens (Bühnenraum bei Wurzeln). |
| `rest_angle` | number | 0 | | Ruherotation in Grad relativ zum Elternknochen. |
| `angle_limits` | `[min, max]` oder null | null | `min <= max` | Begrenzung des animierten Winkel-Deltas, in Grad. |
| `scale_limits` | `[min, max]` oder null | null | `0 < min <= max` | Begrenzung der animierten Skalierung. |

Wechselwirkungen: `bone_angle`-Kanäle werden zu `rest_angle` addiert; `bone_offset_x/y` werden zu `rest_offset` addiert;
`bone_scale` multipliziert. `clamp_limits` läuft nach allen Kanälen und vor FK, und erneut nach jeder
IK-Kette.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| JSON-Name | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | eindeutig | Ziel von `ik_mix`-Kanälen. |
| `bones` | array of strings | **Req** | mindestens 2; jeder ein direktes Kind des vorherigen | Kette von der Wurzel zur Spitze. |
| `target` | string | **Req** | ein Knochen, nicht in der Kette | Knochen, nach dessen Kopf sich die Spitze streckt. |
| `bend` | number | 1 | +1 oder -1 | In welche Richtung eine Kette aus zwei Knochen knickt. |
| `mix` | number | 1 | clamped 0..1 | Anfängliche FK-zu-IK-Mischung; animierbar mit `ik_mix`. `<= 0` überspringt die Kette. |
| `iterations` | integer | 16 | | FABRIK-Durchläufe für Ketten mit mehr als zwei Knochen. Das ist eine Untergrenze: Der Solver beendet früh, wenn die Spitze innerhalb von 0.01 px liegt, und läuft sonst bis zu 256 Durchläufe weiter. |
| `soft` | number | 0 | `0..0.9` | Anteil der Reichweite, über den die Streckung abklingt. Bei `stretch > 1` ist die Zone der letzte Anteil der gestreckten Reichweite (`reach x stretch`). |
| `stretch` | number | 1 | `1..4` | Maximaler Streckfaktor; über 1 wächst die Skalierung des Wurzelknochens, um zu reichen. |

Eine Kette aus zwei Knochen verwendet den analytischen Kosinussatz-Solver; längere Ketten verwenden FABRIK. Nach dem Lösen
werden die Winkel als `old + wrap_pi(new - old) * mix` in die Pose gemischt.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

Constraint-Typen verwenden ein internes `"type"`-Tag, das neben `name`, `bone` und `weight` abgeflacht ist.

| JSON-Name | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `name` | string | **Req** | Ziel von `constraint_weight`-Kanälen. |
| `bone` | string | **Req** | Der eingeschränkte Knochen. |
| `type` | `aim`, `copy_rotation`, `pin` | **Req** | Art. |
| `target` | string | **Req** für `aim`, `pin` | Knochen, auf den gezielt wird oder auf den gerutscht wird. |
| `offset` | number | 0 | Nur `aim`: zusätzliche Grad. |
| `source` | string | **Req** für `copy_rotation` | Knochen, dessen Weltrotation kopiert wird. |
| `weight` | number | 1 | Einfluss, auf 0..1 begrenzt. `<= 0` überspringt die Constraint. |

Eine Constraint darf nicht auf sich selbst zielen. Constraints laufen in Listenreihenfolge nach allen IK-Ketten.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Eine gedämpfte Feder, die die Weltrotation eines Knochens hinter seiner animierten Rotation zurückbleiben lässt. Sie wird
einmal beim Laden simuliert und gebacken, sodass jeder Frame in beliebiger Reihenfolge ausgewertet werden kann.

| JSON-Name | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `bone` | string | **Req** | eine Feder pro Knochen | Anzusteuernder Knochen. |
| `frequency` | number | 3 | `(0, 30]` Hz | Steifigkeit. |
| `damping` | number | 0.35 | `>= 0` | Dämpfungsverhältnis; unter 1 schwingt es nach, 1 ist kritisch. |
| `warmup` | bool | true | | Einen Durchlauf vorrollen, damit Loops im eingeschwungenen Zustand starten. |

Gebackene Winkel ersetzen den Knochenwinkel nach den Constraints (Auswertungsschritt 8).

## 5. Ebenen

Eine Ebene ist `{"name": ..., "type": ..., <kind fields>, <common fields>}`. `type` ist das Tag von
`LayerKind`; seine Felder sind in das Ebenenobjekt abgeflacht.

### 5.1 Gemeinsame Felder

| JSON-Name | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `name` | string | **Req** | nicht leer, kein `/`, kein `::`; der vollständige Pfad muss eindeutig sein | Ebenenname. Kinder werden als `parent/child` adressiert. |
| `type` | string | **Req** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Art. |
| `transform` | object | Identität | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | number | 1 | clamped 0..1 | Multipliziert sich in den Ebenen-Composite. |
| `visible` | bool | true | | Versteckte Ebenen werden beim Zeichnen übersprungen, können aber weiterhin Maskenquellen sein. |
| `blend` | string | `normal` | siehe unten | Blend-Modus. |
| `mask` | `{layer, invert}` | null | keine Selbstmaske, keine Zyklen | Das Alpha einer anderen Ebene als Maske verwenden. |
| `bone` | string | null | ein bekannter Knochen | An einen Knochen binden: Die Skin-Matrix des Knochens wird zwischen der Bühnenmatrix des Elternelements und der lokalen Transformation angewendet. |
| `parallax` | number | 1 | | Kamera-Tiefenfaktor. Nur der Wert der obersten Ebene zählt. |
| `deformers` | array | weggelassen, wenn leer | nur Vektor- und Bildebenen | Abschnitt 7. |
| `wobble` | object | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): deterministische Noise-Verschiebung. `amp` mit `layer_wobble` animieren. |
| `reference` | bool | false | | Referenzebene: versteckt, sofern der Editor keine Referenzen anzeigt. |
| `locked` | bool | false | | Editor-Sperre, siehe unten. |
| `normal_map` | object | null | | Abschnitt 7. |
| `effects` | array | weggelassen, wenn leer | höchstens 16 | Abschnitt 7. |

Lokale Matrix: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Blend-Modi: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (CSS plus-lighter).

`locked` blockiert nur Editor-Operationen: Canvas-Picking, Gizmos, Nudges, Löschen, Timeline-Änderungen und
Kanaländerungen (über `assert_unlocked`), für die Ebene und ihre Nachfahren. Rendering und
Auswertung ignorieren es, und einfache `Op`s (Abschnitt „Editor ops“ in ARCHITECTURE.md) werden weiterhin angewendet.

Die gemeinsamen Felder werden in der Ausgabe nach den Art-Feldern geschrieben, aber die Reihenfolge ist bei der Eingabe unerheblich.

### 5.2 `vector`

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `paths` | array of path | `[]` | Abschnitt 6. |

### 5.3 `group`

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Kinder erben die Transformation der Gruppe. |

### 5.4 `switch`

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `layers` | array of layers | `[]` | Kandidatenzeichnungen. |
| `selected` | integer | 0 | Anfänglicher Kind-Index. Animierbar mit `layer_switch` (Step-Keys). |

Genau ein Kind wird angezeigt. Der Wert wird gerundet; negative Werte werden zu 0; Werte ab der Kinderzahl
werden zum letzten Kind. Die Validierung lehnt Keys außerhalb des Bereichs nicht ab. Siehe Abschnitt 20 zu einer
abweichenden Lint-Formulierung dazu.

### 5.5 `liquid`

Metaball-Flüssigkeit. Das Feld ist die Summe von `r^2 / d^2` über die Blobs; die Kontur ist die Iso-Kontur,
per Marching Squares auf einem Gitter von höchstens 600 mal 600 Zellen gewonnen und mit Catmull-Rom geglättet.

| Feld | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `blobs` | array of `{pos, radius, bone?}` | **Req** | mindestens 1; `radius > 0` | Ein Blob. `bone` bindet ihn an einen Knochen. |
| `threshold` | number | 1 | `> 0` | Iso-Level; 1 macht einen isolierten Blob zu einem Kreis seines Radius. |
| `cell` | number | 3 | `>= 0.25` | Gitterzelle in Ebeneneinheiten. |
| `fill` | `Paint` | null | | |
| `stroke` | stroke object | null | | |

### 5.6 `boolean`

| Feld | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Req** | Operation. |
| `paths` | array of path | **Req**, mindestens 2 | `paths[0]` ist das Subjekt; der Clip ist die Vereinigung der übrigen. |

Das Ergebnis übernimmt Füllung und Strich des Subjekts; die Flattening-Toleranz ist 0.25. Ergebnispfade werden
`<first path name><index>` genannt.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Partikel sind eine geschlossene Funktion der Zeit, sodass jeder Frame
auswertbar ist. Zeiten sind in Sekunden.

| `spec`-Feld | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Emitterposition (Ebenenraum). |
| `bone` | null | | An einen Knochen binden. |
| `rate` | 20 | `(0, 2000]` | Partikel pro Sekunde. |
| `lifetime` | 1 | `> 0` | Sekunden. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixel pro Sekunde. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Grad; -90 ist oben. |
| `spread` | 30 | | Grad der Kegelbreite. |
| `gravity` | `[0,0]` | | Beschleunigung, Pixel pro Sekunde zum Quadrat. |
| `size_start`, `size_end` | 3, 3 | | Radius über die Lebensdauer. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Farbe über die Lebensdauer. |
| `seed` | 0 | | Deterministischer Seed. |
| `start` | 0 | | Startzeit in Sekunden. |
| `duration` | null | | Die Emission endet nach so vielen Sekunden; null ist für immer. |
| `max_particles` | 2000 | | Harte Obergrenze für lebende Partikel. |

### 5.8 `image`

| Feld | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Req** | eines der beiden | `file` ist ein Pfad relativ zum Projektverzeichnis (Abschnitt 18); `png_base64` ist in sich geschlossen. Keines von beiden: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Obere linke Ecke im Ebenenraum. |
| `size` | `Vec2` oder null | null (Pixelgröße) | | Angezeigte Größe. |
| `mesh` | object oder null | null | | Deformations-Mesh. |

`mesh`: `cols`, `rows` (Grid-Mesh, jeweils mindestens 1), `weights` (Knochengewichte pro Vertex; leer
bedeutet statisch) und optional `free` = `{vertices, triangles, uvs}`. Ein freies Mesh braucht mindestens 3
Vertices, mindestens 1 Dreieck, höchstens 200000 Vertices und 400000 Dreiecke, endliche Koordinaten,
gültige Dreiecksindizes, und `uvs` entweder leer oder von gleicher Länge wie `vertices`. Mit Deformern, aber
ohne Mesh wird ein Gitter von 16 mal 16 verwendet.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Pfade, Anker, Paint, Stroke

### 6.1 Pfad (`paths[]`)

| JSON-Name | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `name` | string | `""` | Ein leerer Name wird zur Adressierung durch `path{index}` ersetzt. Darf kein `::` enthalten. Innerhalb der Ebene eindeutig. |
| `closed` | bool | false | |
| `anchors` | array | **Req** | Bézier-Anker. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array of `{name, anchors}` | weggelassen, wenn leer | Additive Blend Shapes. Die Ankerzahl muss der des Basispfads entsprechen. Gewicht über `path_morph`. |

### 6.2 Anker

| JSON-Name | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Req** | Position. |
| `handle_in`, `handle_out` | `Vec2` | null | Kontrollpunkte im selben Raum wie `pos` (absolut, nicht relativ). Ein fehlendes Handle entspricht `pos` (eine Ecke). |
| `weights` | array of `{bone, weight}` | weggelassen, wenn leer | Skinning. Jeder Knochen muss existieren, und jedes Gewicht ist endlich und `>= 0`. Gewichte werden bei der Auswertung normalisiert; leer bedeutet statisch. Handles werden ebenfalls geskinnt. |

### 6.3 Stroke

| JSON-Name | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Req** | |
| `width` | number | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Ein unbekannter Wert ergibt ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | weggelassen | Wird nur verwendet, wenn seine Länge gerade und mindestens 2 ist und jeder Eintrag `> 0`. |
| `dash_offset` | number | 0 | |
| `width_profile` | array of `[t, factor]` | weggelassen | Stückweise linearer Breitenmultiplikator entlang der Länge (`t` in 0..1), jenseits der Enden konstant, Mitre-Limit 2. Ein Strich variabler Breite ignoriert `dash` und `join`. |

Beispiel einer radialen Verlaufsfüllung mit einem Strich (aus dem Ball-Beispiel):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

Der Linter (`osatura lint`) warnt bei Verlaufsproblemen (L011), Switch-Keys (L012) und Problemen mit Editor-Daten (L013); siehe
`osatura-core/src/lint.rs`.

## 7. Deformer, Effekte, Normal Map

### 7.1 `deformers[]`

Nur bei `vector`- und `image`-Ebenen erlaubt. Jeder hat ein `"type"`-Tag und einen `name` (eindeutig innerhalb
der Ebene, kein `::`). Sie laufen in Listenreihenfolge auf der Geometrie im Ruheraum, vor dem Skinning und vor
`wobble`. Offsets sind statisch; animiere sie mit den Kanälen für Unterziele.

| `type` | Felder | Einschränkungen | Animieren mit (Ziel) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` leer oder `(cols+1)*(rows+1)` zeilenweise | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (Kurvenpunkte), `offsets`, `radius` (optionale weiche Kante) | mindestens 2 Punkte | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | 1 bis 64 Pins, `radius > 0`, Pins nicht deckungsgleich | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (Polygon), `offsets` | 3 bis 256 Vertices, einfaches Polygon, Fläche ungleich null | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | wie `lattice` | wie `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Lattice mischt Offsets mit Bézier-Bernstein-Gewichten (glatt); Cloth verschiebt jeden Gitter-Vertex
exakt (bilinear). Pin verwendet eine Wendland-Radialbasisfunktion mit kompaktem Träger. Cage verwendet
Floater-Mean-Value-Koordinaten: Inhalt innerhalb des Polygons folgt ihm, Inhalt außerhalb bleibt
unberührt.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

Höchstens 16 pro Ebene. Jeder Effekt hat `"type"`, `enabled` (Standard true) und die unten genannten Felder. Das
`fxN`-Kanalziel indiziert das Array einschließlich deaktivierter Effekte.

| `type` | Felder | Bereich |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | Offsets höchstens 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Animiere mit `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` auf dem Ziel `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (default stretch),
"strength": 0..8 (default 1), "green_down": bool (default false)}`. Braucht `data` oder `path`. Es speist
den Post-Effekt `relight`; einfache Renderings verwenden es nicht. Als `Layer.normal_map` wird es an die Pixel-Bounding-Box dieser Ebene angepasst. `fit` `exact` verlangt eine Map genau in Frame-Größe und ist andernfalls ein Fehler.

## 8. Kanäle, Tracks, Easing

### 8.1 Kanal

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| JSON-Name | Typ | Standard | Bedeutung |
| --- | --- | --- | --- |
| `target` | string | `""` | Name oder Form `layer::sub`. Bei Kamera-Props ignoriert. |
| `prop` | string | **Req** | Eine der snake_case-Props unten. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Req** | Abgeflachtes Tag, mit den Feldern dieser Quelle daneben. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Wie sich der Wert mit dem aktuellen verbindet. |
| `weight` | number | 1 | Einfluss, muss endlich sein; Überblendung zum Ergebnis hin. |

Kombination: replace ist `cur + (v - cur) * w`; add ist `cur + v * w`; multiply ist
`cur * (1 + (v - 1) * w)`. Nicht endliche Werte werden übersprungen. Kanäle laufen in Listenreihenfolge, ein späterer
Kanal sieht also die Ergebnisse früherer (über `get(...)` und `driven`).

Quellen:

| `source` | Felder |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: string (Abschnitt 9) |
| `noise` | `amp`, `freq` (Zyklen pro Sekunde), `seed` (0), `octaves` (1). Ausgabe in `[-amp, amp]`; hash-basiert, deterministisch. |
| `driven` | `input` (Ziel), `input_prop`, `curve`: ein Track, dessen „Frames“ Eingabewerte sind (zum Beispiel Ellbogenwinkel auf Wölbungsstärke). |

### 8.2 Props

Es gibt 39 Props. Die Prop bestimmt, was `target` bedeutet.

| Gruppe | Props | `target` |
| --- | --- | --- |
| Bone | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | Knochenname |
| Layer | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | Ebenenpfad (`parent/child`) |
| Camera | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignoriert |
| Rig | `ik_mix`, `constraint_weight` | Name von ik oder Constraint |
| Path | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformer | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (Lattice, Cloth) oder `layer::deformer::i` |
| Effect | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Semantik der Verlaufs-Handles: Bei linearen Verläufen verschieben `.from` und `.to` den Start- und Endpunkt um
`(dx, dy)`. Bei radialen Verläufen verschiebt `.from` den Mittelpunkt, und `.to` ändert den Radius um `dx`.
Nur Pfade, deren Füllung oder Strich ein Verlauf ist, haben diese Ziele. `layer_switch` setzt eine
`switch`-Ebene voraus. Eine unbekannte Prop listet im Fehler alle gültigen Varianten auf.

Begrenzungen beim Schreiben: opacity 0..1, Kamera-Zoom mindestens 0.01, Kamera-Blur mindestens 0, `ik_mix` und
`constraint_weight` 0..1.

### 8.3 Track und Key

`Track` = `{"keys": [...], "extrapolate": "hold"}`. Ein Key ist `{"frame": number, "value": number,
"ease": {...}}`. Keys müssen endlich und im Frame streng aufsteigend sein:
`channel N ('target'): keys must be finite and strictly increasing in frame`. Das Easing eines Keys
formt das Segment, das an diesem Key beginnt.

Extrapolation (außerhalb des ersten und letzten Keys): `hold` (Standard), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` ist ein Objekt mit einem `type`-Tag. Der Standard ist `{"type": "linear"}`.

| `type` | Parameter | Kurve |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Den Key-Wert bis zum nächsten Key halten. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Kurve im CSS-Stil durch (0,0) und (1,1). |
| `smooth` | | Nichtuniformer Catmull-Rom-Hermite durch benachbarte Keys. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; landet exakt auf dem Ziel. |
| `back` | `overshoot` | Kubisch mit Startsteigung `3 + overshoot` (auf mindestens 0 begrenzt). Overshoot 1 erreicht den Gipfel bei 1.037 (u = 2/3); Overshoot 2 bei 1.1317 (u = 0.5556). |
| `bounce` | | Restitution 0.6, drei Rebounds, Gesamtdauer 3.352 Zeiteinheiten. Erster Aufprall bei u = 0.2983; Rebound-Gipfel 0.64, 0.8704, 0.953344; endet bei 1. |

`back` und `bounce` sind eigene Herleitungen (siehe `CLEANROOM.md`).

## 9. Ausdrücke

Ein Kanal mit `"source": "expr"` wertet pro Frame eine Formel aus. Der Quelltext ist auf 4096 Bytes
und eine Verschachtelungstiefe von 64 begrenzt; Fehler werden bei der Validierung mit einer Position gemeldet, zum Beispiel
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variablen: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operatoren (von niedriger zu hoher Präzedenz): `?:`, `||`, `&&`, Vergleiche, `+ -`, `* / %`, unäres `- !`,
`^` (rechtsassoziativ). Division durch null ergibt 0; `%` ist euklidisch (`rem_euclid`).

Funktionen: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 oder 2 Argumente) `fbm` (2 oder 3) `wiggle` (2 oder 3) `saw tri
square deg rad` und `get("target", "prop")`. Ein nicht endliches Funktionsergebnis wird zu 0.

`get("target", "prop")` liest den aktuellen Wert einer beliebigen Eigenschaft, einschließlich der Ergebnisse früherer
Kanäle. Beide Strings müssen bei der Validierung auflösbar sein (`channel 2: get("q", "bone_angle"): unknown
target`). Ein Kanal, der einen Wert liest, muss nach dem Kanal stehen, der ihn schreibt.

Beispiel aus dem Ball-Projekt:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Binding-Ausdrücke (Abschnitt 11) verwenden eine eingeschränkte Form: Die Variable ist `value`, und die Zeitvariablen
sowie `get()` sind verboten.

## 10. Kamera, Shots, Audio, Metadaten

### 10.1 `camera`

| JSON-Name | Typ | Standard | Bereich | Bedeutung |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` oder null | null (Bühnenmitte) | | Weltpunkt in der Bildschirmmitte. |
| `zoom` | Zahl | 1 | wird auf mindestens 0.01 begrenzt | |
| `rotation` | Zahl | 0 | Grad | |
| `focus` | Zahl | 1 | | Die `parallax`-Tiefe der Ebene, die scharf ist (verwendet von `dof` und `dof-export`). |
| `blur` | Zahl | 12 | endlich, `>= 0` | Unschärferadius in Pixeln pro Einheit Parallaxenabstand. |

Parallaxe: Eine Ebene mit `parallax = p` sieht `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Ein nicht endliches `focus` oder `blur`
ergibt `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Metadaten-Arrays (beim Laden nicht validiert)

Diese enthalten Daten für Editor und Werkzeuge. Die Engine akzeptiert sie als Daten; Operationen, die sie verwenden,
prüfen sie, wenn sie laufen.

| Array | Item-Felder |
| --- | --- |
| `markers` | `name`, `frame`, `end` (optional; eine Region, wenn gesetzt) |
| `comps` | `name`, `layers`: Map von Ebenenpfad auf `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (beliebiges JSON) |
| `bone_vis` | `bone`, `from`, `to` (Frame-Bereich, in dem der Knochen angezeigt wird) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (Editor-Flags; `lock` blockiert set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (gespeicherte Kameraansichten) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| JSON-Name | Typ | Bereich | Bedeutung |
| --- | --- | --- | --- |
| `name` | String | eindeutig | |
| `file` | String | ein Asset-Pfad (Abschnitt 18) | WAV oder anderes unterstütztes Audioformat. |
| `start` | Zahl | Sekunden | Platzierung auf der Timeline. |
| `gain` | Zahl | 0..4 | |
| `trim_in`, `trim_out` | Zahl | `trim <= 86400` | Sekunden, die am Anfang und am Ende der Datei abgeschnitten werden. |
| `fade_in`, `fade_out` | Zahl | passt in den getrimmten Clip | Sekunden. |

`check` (nicht `Engine::new`) meldet eine fehlende Datei: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clips, Skins, Links, Interaktion, LODs

### 11.1 `clips[]` und `clip_baked`

Ein Clip platziert einen wiederverwendbaren `take` auf der Timeline. Das Baken von Clips hängt gewöhnliche Kanäle an (nach den
von Hand erstellten) und speichert einen Snapshot in `clip_baked`, sodass erneutes Baken idempotent ist und Handarbeit
nie gelöscht wird.

| Clip-Feld | Bedeutung |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Platzierung. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Einfluss; optionale animierte Spur. |
| `include`, `exclude` | Musterlisten, die Spuren auswählen. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Frames. |
| `rename` | `[[from, to]]` Umbenennungen von Zielen. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Skins in derselben `group` sind sich gegenseitig ausschließende Varianten.
Die Knochenskalierung pro Skin wird beim Baken zu einem multiplikativen `bone_scale`-Step-Kanal zusammengefasst. Skin-Regeln
werden nur geprüft, wenn Skin-Operationen laufen.

### 11.3 `mesh_locks[]` und `glues[]`

Ein Glue-Paar verwendet `VertexRef` = `{layer, path (default 0), index}`; ein Mesh-Lock benennt ganze Pfade mit `{layer, path}` und eine optionale Liste von Follower-Vertex-Indizes (leer bedeutet jeden Vertex).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

Der Glue-`mode` ist `snap`, `rigid` oder `offset`. Beide werden von `links::resolve` validiert und im
Bühnenraum in topologischer Reihenfolge angewendet (Auswertungsschritt 12).

### 11.4 `machines[]` und `bindings[]` (nur zur Laufzeit)

Zustandsautomaten und Datenbindungen werden zur Laufzeit vom Player gesteuert. Für Export und Baken werden sie zu konstanten
`replace`-Kanälen, die an das Projekt angehängt werden. Jeder Automat braucht mindestens eine Ebene.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

Der Input-`kind` ist `bool`, `number` oder `trigger`. Ein State-`source.kind` ist `clip`, `segment`, `region`
oder `blend`; `mode` ist `loop`, `once` oder `ping_pong`. `Transition` und `Condition` lehnen unbekannte
Schlüssel ab.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

Das Binding-`weight` muss innerhalb von 0..1 liegen (`binding 'b': weight must be within 0..1`). Die Ziele von Automaten und Bindings
werden durch eine Probekompilierung geprüft.

### 11.5 `containers[]` und `scrolls[]`

Eindimensionales Layout im Flexbox-Stil. Container-Felder: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 Zahlen), `justify`, `align`, `wrap`, `rect`. Ein `rect` hat einen `kind`:
`fixed` (`x,y,w,h`), `pin`, `aspect` oder `hug`. Ein Member hat genau eines von `layer`, `container` oder
`spacer`, dazu `grow`, `shrink`, `width`, `height`, `align`, `resize`. Grenzen: 100000 Member,
Verschachtelungstiefe 64. Ein Scroll hat `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
`snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Container-Offsets werden zur Ebenenposition addiert und in die Skalierung multipliziert (Auswertungsschritt 7).
`apply_clip_masks` fügt dem übergebenen Projekt Maskenreferenzen hinzu und verändert daher die Kopie der Engine.

### 11.6 `lods[]`

Ein Eintrag über einen Level-of-Detail-Durchlauf, geschrieben von `osatura lod`: `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) und `notes`. Er dokumentiert, was ein
Durchlauf getan hat; der Renderer liest ihn nie.


## 12. Validierungsregeln, Grenzen und Fehlermeldungen

`Engine::new` validiert in einem Durchgang und gibt alle Probleme zurück. Die Tabelle listet auf, was erzwungen wird;
Meldungen in Anführungszeichen wurden von `osatura check` bei absichtlich fehlerhaften Dateien erzeugt.

### 12.1 Globale Grenzen

| Regel | Grenze | Meldung |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| beliebiger Float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | höchstens 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| JSON-Größe | CLI `--max-input-bytes`, Standard 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

Die 1e7-Grenze wird zweimal angewendet. Beim Laden lässt ein Float-Literal darüber das Parsen scheitern, und die Meldung
wird als `invalid JSON in <file>: value ...` verpackt. Ein Literal ohne Dezimalpunkt (eine Ganzzahl)
passiert das Parsen und wird von der erneuten Prüfung in `Engine::new` abgefangen, wo es als gewöhnlicher
Listeneintrag erscheint. Die Grenze existiert, damit eine spätere Multiplikation `f32` nicht ins Unendliche überlaufen lassen kann.

### 12.2 Rig

| Regel | Meldung |
| --- | --- |
| Eltern-Bone muss früher existieren | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| Bone-Länge endlich, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| ik braucht 2 Bones | `ik 'reach': needs at least two bones` |
| ik-Kette über Eltern verknüpft | `ik 'reach': each bone must be a direct child of the previous one` |
| ik-Ziel nicht in der Kette | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| Constraint kann nicht sich selbst als Ziel haben | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; eine pro Bone |

### 12.3 Ebenen, Pfade, Geometrie

| Regel | Meldung |
| --- | --- |
| Ebenenname nicht leer, kein `/`, kein `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| doppelter Pfad | `duplicate layer path 'arm'` (und Zeilen für die doppelten animierbaren Ziele) |
| Pfad- und Deformer-Namen enthalten kein `::` und sind innerhalb der Ebene eindeutig | |
| Anzahl der Morph-Anker entspricht dem Basispfad | |
| Skin-Gewichte nennen bekannte Bones; Gewicht endlich, `>= 0` | |
| freies Mesh | mindestens 3 Vertices, mindestens 1 Dreieck, höchstens 200000 / 400000, endlich, gültige Indizes, `uvs` leer oder gleiche Anzahl |
| Gitter-Mesh | `cols`, `rows >= 1` |
| Maske | keine Selbstmaskierung; `mask cycle involving layer 'arm'` |
| Bildquelle | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | mindestens 1 Blob; `threshold > 0`; `cell >= 0.25`; Blob-`radius > 0` |
| boolean | mindestens 2 Pfade |
| emitter | `rate` in `(0, 2000]`; `lifetime > 0`; Jitter 0..1 |
| deformers | nur auf Vektor- und Bildebenen; siehe Abschnitt 7.1 für jeden Typ |
| effects | höchstens 16; Blur und Softness 0..64; Offsets höchstens 1024; Beträge 0..1 |
| normal map | `strength` 0..8; braucht `data` oder `path` |
| camera | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Kanäle

| Regel | Meldung |
| --- | --- |
| Ziel wird aufgelöst | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` auf einer Switch-Ebene | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| weight endlich | |
| Keys endlich und streng aufsteigend | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| Ausdruck lässt sich parsen | `channel 2 ('upper'): expression error: unexpected end of expression` |
| Ziele und Props von `get()` werden aufgelöst | `channel 2: get("q", "bone_angle"): unknown target` |
| angesteuerter Input wird aufgelöst | |

Unbekannte `prop`- oder `ease`-Namen erzeugen serde-Fehler, die jede gültige Variante auflisten, zum Beispiel
``unknown variant `foo`, expected one of ...``. Falsche Ganzzahltypen erzeugen
``invalid type: floating point `48.0`, expected u32``. Ein fehlendes Pflichtfeld erzeugt
``missing field `frames` ``.

### 12.5 Weitere validierte Abschnitte

- Sounds: `gain` 0..4, `trim` höchstens 86400 s, Fades passen in den getrimmten Clip, Namen eindeutig.
  Die Existenz der Datei prüft `check`, nicht `Engine::new`.
- Machines: mindestens eine Ebene (`machine 'm': needs at least one layer`); Transitions und
  Conditions lehnen unbekannte Schlüssel ab. Bindings: `binding 'b': weight must be within 0..1`. Ihre Ziele
  werden durch einmaliges Kompilieren geprüft.
- Container und Scrolls: Member eindeutig, höchstens 100000, Verschachtelungstiefe 64, Rect-Regeln.
- Mesh-Locks und Glues: Referenzen werden von `links::resolve` aufgelöst.
- Bild- und Audiodateien: Abschnitt 18.

### 12.6 Editor-Daten: Warnungen, und Fehler nur für nicht endliche Zahlen

`Engine::new` und `check` betrachten `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` und
`lods` (Modul `editordata`). Damit jedes früher gespeicherte Projekt weiter geladen werden kann, sind **nur Werte, die in einer gespeicherten Datei
nicht vorkommen können, Fehler**: NaN oder Unendlich in Markern, Events, Shots, `bone_vis`, Comp-Opacity, LODs und Ebenen-Opacity. Alles andere ist
eine **Warnung**, die `check` ausgibt (und die `lint` als L013 meldet): Kamera-Zoom `<= 0` (die Engine begrenzt ihn auf 0.01), Ebenen-
`opacity` außerhalb von 0..1 (wird begrenzt), `ik_mix` und `constraint_weight` außerhalb von 0..1 (werden begrenzt), animierte `layer_opacity`- und
`camera_zoom`-Werte außerhalb ihres Bereichs, unbekannte Referenzen auf Ebenen/Bones/Kanäle, doppelte Namen, Zeiten außerhalb der
Animation, ungültige Skins (`skins::validate`), Clips (`clipstack::validate`, sowie Take-Track-Ziele, die sich nicht auflösen lassen,
unter Berücksichtigung von `rename`), `channel_meta` und `lods`. Die `out/`-Fixtures laden ohne Warnungen.

`osatura check` gibt die Lint-Warnungen dieser Art aus; der vollständige Linter (`osatura lint`) hat weitere Regeln (ab L001).

## 13. Beispiel 1: Springender Ball

Verifiziert mit `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) und `osatura render ball.json 7 out.png`. Was es zeigt:

- Animiertes `layer_y` mit `bounce`-Easing auf dem ersten Segment: Der Ball fällt von y=40 auf den
  Boden bei y=170 und springt dreimal. Der erste Aufprall liegt bei etwa Frame 7.2 (24 / 3.352).
  Das Segment von 24 bis 47 steigt mit `ease_out` wieder an.
- Ein Schatten, der dem Ball folgt: `layer_x` und `layer_opacity` sind Ausdrücke, die den
  Ball mit `get()` auslesen. Sie stehen nach den eigenen Kanälen des Balls, weil Kanäle der Reihe nach laufen.
- Squash beim Kontakt: Ein Ausdruck liefert 0.8 für `layer_scale_y`, wenn der Ball unterhalb von y=160 liegt.
- Kreis aus vier Bézier-Ankern mit Handle-Länge 16.57 (0.5523 mal der Radius 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Beispiel 2: Gerigter Arm mit IK

Verifiziert mit `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` meldet die Spitze bei (170, 190), gleich dem `target`-Bone, und `osatura render arm.json 24
out.png --bones` zeichnet das Skelett. Was es zeigt:

- `upper` ist ein Root mit einem Winkellimit; `lower` ist sein Kind (`rest_offset` `[90, 0]` entspricht der
  Länge des Elternteils); `target` ist ein freier Root-Bone, nach dem die IK-Kette greift.
- `reach` ist eine IK-Kette aus zwei Bones, daher wird der analytische Solver verwendet.
- `sleeve` ist geskinnt: Jeder Anker listet Bone-Gewichte auf. `hand` und `goal` sind mit
  `bone` angehängt.
- Das Bewegen des `target`-Bones mit den Kanälen `bone_offset_x` und `bone_offset_y` lässt den Arm danach greifen.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Beispiel 3: Mund mit Switch-Ebene

Verifiziert mit `osatura check` und durch das Rendern der Frames 0, 5 und 9, die drei verschiedene
Münder zeigen. Key-Werte außerhalb des Bereichs werden begrenzt: Ein Key-Wert von 7 zeigt Child 2 und -3 zeigt Child 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "switch",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Auswertungsreihenfolge

`Engine::evaluate(frame)` ist eine reine Funktion der Frame-Nummer und des Projekts. Springs werden
einmal simuliert, wenn die Engine aufgebaut wird; sonst trägt nichts Zustand von Frame zu Frame. Die Schritte:

1. **Anfangszustand** (`initial_state`): statische Ebenentransformationen, auf 0..1 begrenzte Opacity, Switch-
   `selected`, Wobble-Amplitude, Kamera, `ik`-Mix, Constraint-Gewichte und Anfangs-Offsets der
   Parameter-Slots (Bone-Offsets, Deformer-Offsets, Morph-Gewichte und so weiter).
2. **Kanäle** (`apply_channels`) in Listenreihenfolge, jeder kombiniert sich mit dem aktuellen Wert gemäß seinem
   `mode` und `weight`. Nicht endliche Ergebnisse werden übersprungen. Schreibzugriffe werden begrenzt wie in Abschnitt 8.2 aufgeführt.
3. **Limit-Begrenzung** (`clamp_limits`): `angle_limits` und `scale_limits` pro Bone.
4. **Forward Kinematics**: Weltmatrizen aller Bones, Eltern zuerst.
5. **IK-Ketten** in Listenreihenfolge, Ketten mit `mix <= 0` werden übersprungen. Nach jeder Kette wird die Pose begrenzt
   und FK neu berechnet, sodass die nächste Kette das Ergebnis sieht.
6. **Constraints** in Listenreihenfolge, `weight <= 0` wird übersprungen: aim, copy_rotation, pin.
7. **Layout-Container**: Offsets werden zu Ebenen-x und -y addiert und in die Skalierung multipliziert.
8. **Gebakene Springs**: Die gebakenen Winkel ersetzen den Bone-Winkel, danach wird FK neu berechnet.
9. **Skin-Matrizen**: `skin_mat = world * inverse(rest world)` pro Bone.
10. **Kamera**: eine Kameramatrix pro Root-Ebene, mit dem `parallax` der Root-Ebene (Abschnitt 10.1).
11. **Nodes** in Pre-Order. Die Bühnenmatrix eines Nodes ist `parentStage * boneAttach(skin_mat) * local`.
    Vektorgeometrie wird in dieser Reihenfolge aufgebaut:
    1. Morphs (additive Blend Shapes);
    2. `point_dx/dy`-Offsets pro Anker;
    3. Verschiebungen der Verlaufs-Handles (`grad_dx/dy`);
    4. Deformer in Listenreihenfolge, danach `wobble`;
    5. Skinning mit normalisierten Gewichten (leere Gewichte bedeuten statisch; Verlaufs-Handles werden
       ebenfalls geskinnt).
    Geskinnte Geometrie liegt in Bühnen-Ruhekoordinaten, und die Ebenentransformation wird danach angewendet.
    Ein Switch zeigt ein Child (gerundet, begrenzt). Liquid-, Boolean- und Emitter-Geometrie wird pro Frame
    synthetisiert. Ein Bild erhält ein Gitter- oder freies Mesh; mit Deformern, aber ohne Mesh erhält es ein
    16-mal-16-Gitter.
12. **Mesh-Locks und Glues** im Bühnenraum, in topologischer Reihenfolge.
13. **Reparatur nicht endlicher Werte** (`repair_nonfinite`): Ein Node, dessen Geometrie NaN oder Unendlich enthält, wird
    ausgeblendet, und `Frame.nonfinite` wird gesetzt. `evaluate_checked` und `check_all_frames_finite` machen daraus
    einen Fehler.

Folgen, auf die du dich verlassen kannst:

- Ein Kanal sieht nur Kanäle, die in der Liste vor ihm stehen (über `get` und `driven`).
- IK überschreibt FK für die Bones in der Kette, im Verhältnis zu `mix` (die Überblendung ist
  `old + wrap_pi(new - old) * mix`). Bei `mix` 0 ist die Kette reines FK.
- Constraints laufen nach IK, daher setzt sich ein Aim-Constraint auf einem IK-Bone durch.
- Springs wirken zuletzt auf den Bone-Winkel, nach IK und Constraints.

## 17. Rendering-Reihenfolge

- Root-Ebenen werden in Listenreihenfolge gezeichnet, Children in Listenreihenfolge innerhalb ihres Parents.
- Ein Node, der nicht `visible` ist, wird übersprungen (er kann trotzdem Maskenquelle sein).
- Ein Node wird auf eine Offscreen-Ebene gerendert, wenn er `opacity < 1`, Effekte, einen anderen Blend-Modus
  als `normal` oder eine Maske hat. Schritte: Inhalt; Ebeneneffekte in Listenreihenfolge (auf einer vergrößerten Zeichenfläche); Maske
  (die Maskenebene wird mit ihrer eigenen Transformation gezeichnet; ihr Alpha, einschließlich ihrer Opacity, multipliziert sich;
  `invert` kehrt es um); Komposition mit Opacity und Blend-Modus.
- Schärfentiefe wird nur von `dof` und `dof-export` angewendet: Radius = `abs(parallax - focus) *
  blur`, standardmäßig höchstens 24 Pixel.
- Das Bone-Overlay (`--bones`) wird zuletzt gezeichnet.

## 18. Assets und Pfade

Projekte können auf Dateien verweisen: `image.source.file`, `normal_map.path`, `sounds[].file`.

- Pfade müssen relativ sein, dürfen kein `..` enthalten und müssen nach dem Auflösen von Symlinks innerhalb des
  Projektverzeichnisses bleiben: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)` und
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` hebt diese Einschränkung für einen einzelnen Befehl auf. Es werden nur reguläre Dateien
  geöffnet.
- Größenobergrenzen: Bilder 64 MB auf der Festplatte, Audio 256 MB. Dekodierte Bilder: jede Seite höchstens 16384 und
  höchstens 64 Megapixel.
- Ausgabeobergrenzen: Raster höchstens 16384 pro Seite und 100 Megapixel; Sequenzen höchstens 2 GiB; Export-
  Budget Frames mal Anker höchstens 40 Millionen.
- Wenn die CLI ein Projekt schreibt, validiert sie es erneut über `Engine::new` und schreibt Asset-Pfade
  relativ zum Ausgabeort um.

Bevorzuge `png_base64` für eine in sich geschlossene Datei.

## 19. Versionierungsrichtlinie

- `format` ist eine Ganzzahl, derzeit `1` (`FORMAT_VERSION`).
- Eine Datei mit einem größeren `format`, als der Build unterstützt, wird mit der Meldung aus Abschnitt 12.1 abgelehnt;
  die Engine rät nicht.
- Das Hinzufügen eines optionalen Feldes mit Standardwert ist keine Versionsänderung. Ältere Builds ignorieren es (und
  `check` warnt davor als unbekanntes Feld).
- Das Entfernen oder Umdeuten eines Feldes erfordert einen Versionssprung, und der Sprung erfordert einen Migrationsschritt in
  `Project::from_json`, der ältere Dokumente vor der Validierung aktualisiert.
- Die Rust-Bibliotheks-API hat eine eigene Version, `API_VERSION` 1.4.0, die durch den Snapshot-Test
  `api_surface.txt` abgesichert ist; die C-ABI hat `ABI_VERSION` 1. Siehe `docs/API.md`.

## 20. Bekannte Abweichungen zwischen Code und Kommentaren

Diese wurden beim Schreiben dieser Referenz gefunden.

1. **Behoben.** Die Switch-Key-Regel (`lint::switch_key_warnings`, L012) sagt jetzt, was die Engine tut: Ein Key unter 0 zeigt Child 0,
   ein Key ab der Anzahl der Children zeigt das letzte Child (ein leerer Switch zeigt nichts). Test: `lint`-Unit-Tests, gegen
   die Engine verifiziert.
2. Der Kommentar in `project.rs` sagt, Dateien ohne `format` seien Version 1, aber der serde-Standardwert ist
   `FORMAT_VERSION`. Sie stimmen heute nur überein, weil die Konstante 1 ist. Wenn die Konstante erhöht wird, würde eine
   Datei ohne `format` als die neueste Version gelesen.
3. **Behoben.** `unknownfields` meldet eine ausdrücklich geschriebene leere Struktur (`clips[].weight_keys: {"keys": []}`) nicht mehr;
   `default_like` ist rekursiv, und ein Test legt die Menge der optionalen Strukturen fest, die der Writer weglässt, wenn sie leer sind.
4. **Behoben (als Warnungen).** Die Editor-Felder werden validiert; siehe Abschnitt 12.6.
5. **Behoben in Welle 7 (L-71).** Die Geometrie einer geskinnten Ebene liegt im Bühnen-Ruheraum, und die Transformation des Parents wird danach
   angewendet. `Layer.bone` hängt jetzt mit `world = bone_skin * parent_stage * local` an: Die Skin-Matrix (Bone-Welt mal inverse Ruhelage) ist
   eine Bewegung im Bühnenraum und wird daher nach der gesamten Parent-Kette angewendet. In der Ruhelage ist die Skin-Matrix die Identität, also bewegt sich
   nichts; wenn der Bone **gepost** ist, verdoppelt eine Parent-Gruppe mit Skalierung 2 den Weg einer angehängten Ebene nicht mehr. Eine Ebene
   ohne eigenen Bone folgt dem Bone ihres nächsten Vorfahren mit Bone-Anbindung; der eigene Bone einer Ebene ersetzt den des Vorfahren
   (die beiden Bewegungen werden nicht gestapelt). Tests: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. Projekte, die sich auf den alten verdoppelten Weg stützten (ein gepostetter
   Bone unter einer skalierten oder rotierten Gruppe), werden anders gerendert; Projekte ohne diese Kombination bleiben unverändert.
