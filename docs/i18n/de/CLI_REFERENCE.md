<!--
SPDX-License-Identifier: CC-BY-4.0
Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
-->

# OSATURA-Kommandozeilenreferenz

Referenz für das Programm `osatura` (Crate `osatura-cli`, AGPL-3.0-or-later; es bindet die MPL-2.0-Engine
`osatura-core` ein). Dokumentierte Version: **osatura 0.1.0**. Dieser Text steht unter der Lizenz CC-BY-4.0.

Jede Synopsis unten ist aus dem Hilfetext des Programms kopiert (`osatura` ohne Argumente oder
`osatura <command> --help`). Jedes Beispiel wurde am eingebauten Demo-Projekt ausgeführt, und die gezeigte Ausgabe
ist aus der echten Ausgabe gekürzt. Wo Hilfetext und tatsächliches Verhalten voneinander abweichen, beschreibt dieses Dokument das
tatsächliche Verhalten und führt die Abweichung am Ende unter [Bekannte Abweichungen zwischen Dokumentation und Verhalten](#bekannte-abweichungen-zwischen-dokumentation-und-verhalten) auf.

Verwandte Dokumente: [EXPORT_FORMATS.md](EXPORT_FORMATS.md) (was jedes Export-/Importformat behält und verwirft),
[CI.md](CI.md) (Animation-CI-Manifeste), [API.md](API.md) (Bibliothek/C-ABI), [PATENT_NOTES.md](PATENT_NOTES.md)
(Codec- und Patentstatus, keine Rechtsberatung).

## Inhalt

1. [Konventionen für alle Befehle](#1-konventionen-für-alle-befehle)
2. [Erste Schritte: das Demo-Projekt](#2-erste-schritte-das-demo-projekt)
3. [Projekte erstellen und untersuchen](#3-projekte-erstellen-und-untersuchen)
4. [Rendern und Exportieren](#4-rendern-und-exportieren)
5. [Rigging, Skins, IK und Layout-Helfer](#5-rigging-skins-ik-und-layout-helfer)
6. [Animation, Keys, Timeline und Generatoren](#6-animation-keys-timeline-und-generatoren)
7. [Physik, Cloth und Deformer](#7-physik-cloth-und-deformer)
8. [Audio, Lippensynchronisation und Sounds](#8-audio-lippensynchronisation-und-sounds)
9. [Import (SVG, OpenRaster, Lottie, BVH, GIF/APNG, PNG-Sequenzen, Bitmaps)](#9-import)
10. [Live-Eingabe, MIDI, Gesichtstracking und Motion Capture](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture)
11. [Zustandsautomaten, Datenbindungen, Container und Scroll-Bereiche](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche)
12. [Effekte, Tiefenunschärfe, Onion Skin, Trails und Kamera](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera)
13. [Level of Detail, Größenberichte und export-fit](#13-level-of-detail-größenberichte-und-export-fit)
14. [Qualitätssicherung: check, lint, ci, golden, fuzz, diff](#14-qualitätssicherung)
15. [Versionsverwaltung (`vcs`)](#15-versionsverwaltung-vcs)
16. [Studio, Zusammenarbeit, MCP-Agent-Server und UI-Registry](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry)
17. [Pose-Datenbank, Vorschläge und weitere Helfer](#17-pose-datenbank-vorschläge-und-weitere-helfer)
18. [Bekannte Abweichungen zwischen Dokumentation und Verhalten](#bekannte-abweichungen-zwischen-dokumentation-und-verhalten)
19. [Alphabetischer Befehlsindex](#alphabetischer-befehlsindex)

---

## 1. Konventionen für alle Befehle

### 1.1 Form der Kommandozeile

```
osatura <command> [sub-command] <positional arguments...> [options]
```

* Optionen dürfen **an beliebiger Stelle** der Zeile stehen, auch nach dem Unterbefehl und nach Positionsargumenten
  (`osatura render demo.json 0 o.png --scale 2` und `osatura --scale 2 render ...` sind dasselbe).
* **Wertoptionen** nehmen das nächste Wort (`--scale 2`) oder die Form `--key=value` (`--scale=2`). Die `=`-Form funktioniert bei
  jeder Wertoption. Ein Wert, der selbst `=` enthält, ist bei der Form mit Leerzeichen unproblematisch (`--remap a=b`).
* **Flags** (boolesche Optionen wie `--bones`) nehmen keinen Wert. `--bones=1` wird abgelehnt:
  `--bones is a flag and takes no value: write --bones (without =), got '--bones=1'`.
* **Strikte Optionen.** Eine Option (Flag oder Wertoption), die der Befehl nicht dokumentiert, ist ein Fehler mit einem
  „meintest du“-Hinweis (Exit 1), statt ignoriert zu werden:
  ```
  $ osatura render demo.json 0 o.svg --bone
  unknown option --bone for 'render' (did you mean --bones?); see: osatura render --help
  ```
  Die Prüfung gilt für jeden Befehl mit einer Usage-Zeile, auch für `check`, `stats`, `lint`, `coach`, `contracts` und
  `machine list` (ältere Versionen ignorierten dort zusätzliche Flags stillschweigend). `--json`, `--quiet` und `--allow-empty` sind **nicht**
  global: Jeder Befehl, dessen Usage-Zeile sie nicht aufführt, lehnt sie ab.
  **Die Prüfung gilt pro Unterbefehl (pro Synopsis-Form), nicht pro Befehl.** Eine Option, die für eine Unterform aufgeführt ist, wird bei
  ihren Geschwistern abgelehnt, und die Meldung nennt die Unterform, die sie akzeptiert:
  ```
  $ osatura ci diff a.png b.png --quiet
  unknown option --quiet for 'ci diff' (--quiet applies to: ci run); see: osatura ci --help
  ```
  `--quiet` gilt also nur für `ci run`, `--json` nur für `vcs log|status|diff|show`, `ui commands|shortcuts` und die
  Form `trail ... --target`, `--tol` nur für `golden check` (und `ci diff`), `--scale` nur für `golden write`, und
  jede `keyedit`-Bearbeitung akzeptiert nur die Optionen, die in der Synopsis hinter ihrem Namen stehen. (`export` fasst seine beiden
  Synopsis-Zeilen weiterhin zusammen, weil das Format an der Dateiendung erkannt wird; `--codec`, `--columns` und `--scale`, die für
  die gewählte Endung nicht gelten, meldet `export` selbst.)
* Einige ältere Befehle nehmen **Wörter** der Form `key=value` statt `--options` (`cloth`, `physics`, `cage`, `warp-pivot`,
  `soft-scale`, `mesh-lock`, `glue`, `audio-react`, `beats`, `import-seq`, `import-anim`, `layer-pivot`, `chan-meta`,
  `waveform`). Diese sind positional und werden genau so geschrieben, wie die Synopsis sie zeigt (`gravity=900`, nicht `--gravity 900`).
* Ein **Frame** ist eine Zahl (Bruchteile erlaubt, wo es sinnvoll ist). Ein Projekt mit `frames = N` hat die Frames `0..N-1`.
  Ebenen werden über den **Pfad** (`group/child`) angesprochen, Knochen über den **Namen**; die Kamera hat das leere Target `""`.
* **Eigenschaftsnamen** (verwendet von `--prop`, `slider`, `drive`, Mapping-Dateien, ...) sind die Namen in snake_case:
  `bone_angle bone_scale bone_offset_x bone_offset_y layer_opacity layer_x layer_y layer_rotation layer_scale_x
  layer_scale_y layer_skew layer_switch layer_wobble camera_x camera_y camera_zoom camera_rotation camera_focus
  camera_blur ik_mix constraint_weight path_morph point_dx point_dy lattice_dx lattice_dy spine_dx spine_dy pin_dx
  pin_dy cage_dx cage_dy fx_blur fx_dx fx_dy fx_soft fx_amount grad_dx grad_dy`.  Einige Befehle geben dieselben Eigenschaften in ihren Meldungen in CamelCase aus (`BoneAngle`); die Optionswerte sind immer snake_case.

### 1.2 Globale Optionen

| Option | Bedeutung |
|---|---|
| `--allow-external-assets` | Erlaubt einem Projekt, Dateien per absolutem Pfad oder außerhalb des eigenen Verzeichnisses zu benennen (Sounds, Bilder, Normal Maps ...). Standard: abgelehnt. |
| `--follow-symlinks` | Schreibt in das **Ziel** eines symbolischen Links, der als Ausgabedatei angegeben wurde. Standard: abgelehnt. |
| `--max-input-bytes N` | Größte Projektdatei, die gelesen wird. Standard `268435456` (256 MiB). Muss eine positive ganze Zahl sein. |
| `<command> --help` | Gibt die Usage-Zeilen eines Befehls auf stdout aus (Exit 0). |
| `<command> --version`, `--version` | `osatura 0.1.0`; ohne Befehl zusätzlich die Lizenzzeile und das AGPL-Quellcode-Angebot. |
| `--json`, `--quiet`, `--allow-empty` | Optionen einzelner Befehle, die von Befehlen, deren Synopsis sie nicht aufführt, **abgelehnt** werden (`unknown option`) (`--json`: `pick`, `select-box`, `timeline`, `trail`, `size-report`, `collab-sim`, `vcs`, `ui`, `face-detect`, `pose-suggest`; `--allow-empty`: `physics`, `live-map`, `face-bake`; `--quiet`: `ci run`). Skripte, die sie einheitlich an jeden Befehl übergeben haben, müssen das lassen. |

Beispiele (verifiziert):

```
$ osatura --max-input-bytes 100 check demo.json
cannot read demo.json: 51450 bytes exceed the input limit of 100 bytes (raise it with --max-input-bytes N)
[exit 1]

$ osatura check /dev/zero
cannot read /dev/zero: not a regular file (devices, pipes and directories are refused)
[exit 1]

$ ln -s real_target.svg link.svg                       # dangling link
$ osatura render demo.json 0 link.svg
cannot write link.svg: link.svg: refusing to write through a symbolic link; pass --follow-symlinks to write to the link's target, or name the target file yourself
[exit 1]
```

### 1.3 Eingabedateien lesen

* Eingabedateien müssen **reguläre Dateien** sein, die nicht größer als `--max-input-bytes` sind. Geräte, Pipes, Verzeichnisse und
  Prozesssubstitutionen (`<(...)`) werden abgelehnt: Schreibe die Daten zuerst in eine Datei.
* Projektdateien sind JSON. Ein Projekt mit unbekannten Schlüsseln wird **akzeptiert**, und die unbekannten Schlüssel werden von `check` (und `lint`, `studio`) als
  `warning: unknown field bogus_key (ignored)` gemeldet. Nebendateien, die du für einen Befehl selbst schreibst
  (Contracts, Mappings, Machines, Container, Scroll-Bereiche, CI-Manifeste, Drive-Maps ...), werden **strikt** geparst: Ein unbekannter
  Schlüssel ist ein Fehler, und nichts wird angewendet (`unknown field(s): elastic.damping (typo? nothing was applied)`).
* Ein Projekt wird immer validiert, wenn es zum Rendern oder Exportieren geladen wird (`<file> is not a valid project:` gefolgt von
  einem Problem pro Zeile). Bearbeitende Befehle kompilieren das Projekt, das sie schreiben wollen, außerdem neu und weigern sich, eines zu schreiben, das nicht mehr validiert.
* **Asset-Pfade** innerhalb eines Projekts (Sounddateien, Bilddateien, Normal Maps) werden relativ zum **Verzeichnis der Projektdatei** aufgelöst.
  Absolute Pfade und jeder Pfad, der `..` enthält, werden abgelehnt, sofern nicht `--allow-external-assets` angegeben ist:
  ```
  $ osatura sound add s.json o.json x /etc/passwd
  sound add: asset '/etc/passwd': absolute paths are not allowed (a project may only reference files inside its own directory; use --allow-external-assets to override) (nothing was added)
  ```
  Das Flag muss bei jedem späteren Befehl, der dieses Projekt lädt, erneut angegeben werden (`check ext.json` schlägt ohne es fehl und gelingt mit ihm).

### 1.4 Ausgabedateien schreiben

* **Atomares Schreiben.** Die meisten Befehle schreiben über eine temporäre Datei im **selben Verzeichnis** (`.<name>.tmp-<pid>-<random>`), führen `fsync` darauf aus,
  kopieren die Berechtigungen der alten Datei und benennen sie per `rename` über das Ziel, sodass ein Absturz oder ein volles Laufwerk nie eine halb geschriebene Datei hinterlässt.
  Kann keine temporäre Datei angelegt werden (schreibgeschütztes Verzeichnis, aber beschreibbare Datei), wird die Datei direkt überschrieben, was **nicht** atomar ist.
  FIFOs und Spezialdateien wie `/dev/stdout` werden direkt beschrieben.
* **Symbolische Links** als Ausgabe werden abgelehnt, sofern nicht `--follow-symlinks` angegeben ist (dann wird das Ziel des Links geschrieben; ein hängender Link ist weiterhin ein Fehler).
  Ein Verzeichnis als Ausgabedatei ist ein Fehler (`is a directory`).
* **Nie über die Eingabe.** `render`, `render-hdr`, `dof`, `dof-export`, `export` (auch die `.json`-Sprite-Sheet-Begleitdatei einer `.png`
  und `--report`), `layout-preview`, `render-gpu`, `export-fit`, `fuzz-grid`, `onion`, `trail` und `golden` lehnen eine Ausgabe ab, die dieselbe Datei
  wie das Eingabeprojekt ist: `refusing to overwrite the input project demo.json with output of the same name`.
* **Bearbeitende Befehle** (solche mit einem Paar `<project.json> <out.json>`: `reparent`, `keys`, `physics`, ...) dürfen **an Ort und Stelle** schreiben
  (`osatura reparent p.json p.json hand none` funktioniert und ersetzt `p.json`). `import-lottie` lehnt eine Ausgabe (oder `fx.json`) ab, die dieselbe Datei
  wie seine Eingabe ist; die anderen Importbefehle lesen eine andere Art von Datei, als sie schreiben.
* **Vorhandene Ausgaben werden von jedem Befehl stillschweigend überschrieben.** Es gibt kein `--force`.
* **Jede vom Benutzer adressierte Ausgabe** (einschließlich `ci run`-Berichten und Goldens, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, der `face-*`-Dateien,
  Pose-Dateien, Snapshots, `vcs` und des Collaboration-Logs des Studios) läuft über denselben atomaren Writer, der Symlinks ablehnt. Die
  von ffmpeg geschriebenen `.mp4`-/`.webm`-Ausgaben werden auf einen Symlink geprüft, bevor ffmpeg startet. Übergeordnete Ordner: `lod`, `export-fit`, `ci` und `face-*`
  legen fehlende Ordner an; `fuzz`, `fuzz-grid` und `export` tun das nicht (`No such file or directory`).

### 1.5 stdout, stderr und Exit-Codes

* Ergebnisse („wrote out.json“, Tabellen, JSON) gehen an **stdout**. Fehler und die meisten Warnungen gehen an **stderr**. `import-svg` gibt seine
  Warnungen auf stdout aus; `export` gibt Lottie-/GIF-/GLB-Warnungen auf stderr aus.
* Exit-Code **0** bedeutet Erfolg. Exit-Code **1** bedeutet jeden Fehler *oder* eine fehlgeschlagene Prüfung: eine nicht lesbare/ungültige Datei, falsche Argumente,
  `lint` mit Fehlern, `diff` mit Unterschieden, einen verletzten Contract, ein abweichendes Golden, eine Fuzz-Fehlerquote über dem Limit,
  ein `export-fit`-Budget, das nicht eingehalten werden konnte, `lod --verify` über der Schranke, eine `collab-sim`-Divergenz, eine fehlschlagende `ci`-Prüfung.
  Exit-Code **2** wird nur von `ci` verwendet (nichts konnte ausgewertet werden: fehlendes Projekt/Manifest, manipuliertes Golden).
* Keine Argumente, ein unbekannter Befehl, die falsche Anzahl von Positionsargumenten oder `osatura --help` geben die vollständige Usage auf
  **stderr** aus und beenden mit **1**. `osatura <command> --help` gibt die Usage-Zeilen dieses Befehls auf stdout aus und beendet mit 0.
  **Ausnahme (Windows, seit 0.1.0-alpha.2):** Wird `osatura.exe` per Doppelklick im Explorer gestartet (das Konsolenfenster wurde dafür angelegt, und kein anderer Prozess teilt es) und hat keine Argumente, gibt es die Usage nicht aus: Es legt `Documents\OSATURA\first-rig.json` (aus `demo-person`) an, falls die Datei fehlt, führt `studio` mit dieser Datei auf `127.0.0.1:8080` aus, öffnet den Browser und hält die Konsole offen, bis du sie schließt oder nach einem Fehler die Eingabetaste drückst. Aus einem Terminal gestartet, verhält es sich wie oben beschrieben. Der Rest des Programms läuft auf einem Thread mit 256 MiB Stack (der Hauptthread unter Windows hat nur 1 MiB).

### 1.6 Externe Programme

`ffmpeg` (auf dem `PATH` gefunden) wird benötigt für: `.mp4`-/`.webm`-Export und `dof-export` in diese Formate, `sound mix`,
Audio in `.html` (mp3) und `face-track` auf einer Videodatei. Benötigte Encoder: `libx264` (mp4), `libvpx-vp9` / `libaom-av1` (webm),
`aac` (mp4-Ton), `libopus` (webm-Ton), `libvorbis`, `libmp3lame` (sound mix). Es ist kein ffmpeg-Code und keine ffmpeg-Binärdatei gebündelt; siehe
[PATENT_NOTES.md](PATENT_NOTES.md). Sonst wird nichts ausgeführt.

### 1.7 Hinweis zur Performance

Alle Zeitangaben in diesem Dokument stammen aus einem **Debug**-Build und sind 10-30-mal langsamer als bei einem Release-Build. Frame-intensive Befehle
(`safety`, `golden`, `ci`, `export-fit`, `proxy-bench`) wurden an einer verkleinerten Kopie der Demo mit 8 Frames (`small.json`, erzeugt durch Setzen von
`"frames": 8`) ausgeführt.

---

## 2. Erste Schritte: das Demo-Projekt

```
$ osatura demo demo.json
wrote demo.json
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura export demo.json demo.gif
wrote demo.gif
```

Die Demo hat die Knochen `arm_up arm_low hand thigh shin foot_ctrl`, eine IK-Kette `leg` und die Ebenen der obersten Ebene
`sky sun hills ground shadow arm leg target blush mouth(switch: closed/open/smile) ink`. Alle Beispiele in dieser Referenz verwenden sie,
sofern nichts anderes gesagt wird.

---

## 3. Projekte erstellen und untersuchen

### `demo`

**Synopsis:** `osatura demo <out.json>`

Schreibt die eingebaute Rig-Demo (640x480, 24 fps, 48 Frames, 6 Knochen, IK-Bein, 14 Ebenen) als Projektdatei.
**Ausgabe:** `wrote <out.json>`. **Fehler:** fehlendes Argument (Usage, Exit 1); nicht beschreibbarer Pfad. **Verwandt:** `demo-fx`, `check`.
**Grenzen:** Die Demo ist ein festes Asset; sie ist die Grundlage jedes Beispiels in diesem Dokument.

```
$ osatura demo demo.json
wrote demo.json
```

### `demo-person`

**Synopsis:** `osatura demo-person <out.json>`

Schreibt die Demo „laufende Person“ (640x480, 24 fps, 32 Frames, 42 Knochen, 45 Ebenen) als Projektdatei: eine Person in Seitenansicht mit Kopf, Ohr und Pferdeschwanz, Hals, Rumpf, zwei Armen mit Händen und je fünf Fingern sowie zwei Beinen mit Füßen und Zehen. Der Laufzyklus umfasst zwei Schritte und läuft ohne Naht in einer Schleife: Das Becken wippt und kippt, der Rumpf neigt sich und dreht gegenläufig, der Kopf bleibt waagerecht mit einem kleinen nachlaufenden Nicken, die Arme schwingen gegen die Beine, wobei sich die Ellbogen beim Vorschwung stärker beugen, und Hände, Finger, Ohr und Pferdeschwanz folgen etwas später. Jeder Fuß setzt mit der Ferse auf, rollt flach ab und löst sich über den Fußballen; Boden, Zaun und Schatten laufen genau mit der Geschwindigkeit des aufgesetzten Fußes, sodass die Füße nicht rutschen. Dies ist das Projekt, das die Windows-Starter (`osatura.exe`, `osatura-studio.exe`) beim ersten Start als `first-rig.json` anlegen; `osatura demo` bleibt die kleine Grundlage der Beispiele in diesem Dokument.
**Ausgabe:** `wrote <out.json>`. **Fehler:** fehlendes Argument (Usage, Exit 1); nicht beschreibbarer Pfad. **Verwandt:** `demo`, `studio`.
**Grenzen:** Die Keys werden alle zwei Frames abgetastet, mit einem zusätzlichen Key vor Frame 0 und einem nach dem letzten Frame, damit die Schleife weich ist; `lint` meldet diese beiden Keys als außerhalb der Zeitleiste (Information, kein Fehler). Die Figur ist eine Seitenansicht, daher gibt es weder Drehung noch Tiefe.

```
$ osatura demo-person person.json && osatura check person.json
wrote person.json
ok: 'OSATURA walking person' 640×480 @ 24 fps, 32 frames, 42 bones, 0 ik, 45 layers, 48 channels
```

### `demo-fx`

**Synopsis:** `osatura demo-fx <out.json>`

Schreibt die Demo „liquid / boolean / particles“ (640x480, 72 Frames, 3 Knochen, 3 Ebenen), die prozedurale Ebenen ausreizt.

```
$ osatura demo-fx fx.json && osatura check fx.json
wrote fx.json
ok: 'OSATURA fx demo' 640×480 @ 24 fps, 72 frames, 3 bones, 0 ik, 3 layers, 3 channels
```

### `check`

**Synopsis:** `osatura check <project.json>`

Lädt und validiert ein Projekt. Gibt `warning:`-Zeilen aus (unbekannte Felder, Gradient-Probleme, Switch-Key-Probleme und die Editor-Daten-Warnungen des Lint-Codes L013: Kamera-Zoom <= 0, Ebenen-Deckkraft außerhalb von 0..1, ungültige Referenzen oder Zeiten in Skins, Comps, Markern, Events, Shots, channel_meta, bone_vis, Clips und lods) und eine `ok:`-Zusammenfassungszeile. Nur Werte, die in einer gespeicherten Datei nicht vorkommen können (NaN oder Unendlich in diesen Strukturen), sind Fehler, damit ältere Projekte weiterhin laden.
Auch Sound-Tracks werden geöffnet: Eine nicht lesbare/unbrauchbare Audiodatei ist ein **Fehler**. **Exit:** 0 gültig; 1 ungültig.

```
$ osatura check demo.json
ok: 'OSATURA demo' 640×480 @ 24 fps, 48 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check unk.json                      # a project with two unknown keys
warning: unknown field bogus_key (ignored)
warning: unknown field bones[0].lenght (ignored)
ok: 'small' 640×480 @ 24 fps, 8 frames, 6 bones, 1 ik, 14 layers, 14 channels

$ osatura check bad3.json                     # IK chain whose target bone does not exist
bad3.json is not a valid project:
  - ik 'leg': unknown target bone 'nope'
[exit 1]

$ osatura check bad.json                      # {"format":1,"name":"x"}
invalid JSON in bad.json: missing field `width`
[exit 1]
```

**Verwandt:** `lint` (Rig-/Animationsqualität), `ci` (Regressions-Gate). **Fallstrick:** `check` rendert nichts; ein Projekt kann `check` bestehen und später bei einem Frame
trotzdem fehlschlagen (`NaN`-Posen werden von `render`/`export` abgefangen, die zuerst jeden Frame auswerten).

### `stats`

**Synopsis:** `osatura stats <project.json>`

Gibt Zählwerte als JSON aus: `name, size, fps, frames, seconds, bones, ik_chains, constraints, springs, layers, channels, keyed_channels, keys, markers`.
Die Ausgabe ist immer JSON; es gibt keine Option `--json` (sie wird wie jede unbekannte Option abgelehnt).

```
$ osatura stats demo.json
{ "name": "OSATURA demo", "size": [640.0, 480.0], "fps": 24.0, "frames": 48, "seconds": 2.0, "bones": 6,
  "ik_chains": 1, "constraints": 0, "springs": 0, "layers": 14, "channels": 14, "keyed_channels": 8, "keys": 37, "markers": 0 }
```
(Die echte Ausgabe ist mehrzeilig formatiert.) Beachte, dass `layers` jeden Knoten zählt, einschließlich der Kinder von Gruppen/Switches, während `check` dieselbe Zahl ausgibt.

### `diff`

**Synopsis:** `osatura diff <a.json> <b.json>`

Semantischer Unterschied zwischen zwei Projekten: `+` hinzugefügt, `-` entfernt, `~` geändert. Reihenfolgeunabhängig bei Listen, die über den Namen adressiert werden.
**Exit:** 0 identisch (`no differences`); **1**, wenn es Unterschiede gibt (die Anzahl `N difference(s)` geht an stderr), sodass es ein Skript steuern kann.

```
$ osatura diff demo.json demo.json
no differences
$ osatura diff demo.json d2.json              # fps 24->30, arm_up longer, layer 'sun' deleted
~ fps: 24 → 30
~ bone 'arm_up' changed (length)
- layer 'sun' removed
3 difference(s)            (stderr)
[exit 1]
```
**Verwandt:** `vcs diff` (Verlauf), `compare` (Pixelunterschied), `ci diff` (Bildunterschied).

### `tag`

**Synopsis:** `osatura tag <project.json>`

Schlägt anhand ihrer Namen Standard-Rollennamen (`arm.upper.l`, `hand.l`, `face.mouth` ...) für Knochen und Ebenen mit einer Konfidenz vor und listet die Standardrollen auf, für die kein Knochen gefunden wurde. Nur lesend; verwende die Rollen als Orientierung für `mocap-match` oder `autorig`.

```
$ osatura tag demo.json
bone  arm_up                   -> arm.upper.l    (45%)
bone  hand                     -> hand.l         (45%)
layer mouth                    -> face.mouth     (90%)
no bone found for: head, neck, torso, hips, face.eye.l, ...
```

### `explain`

**Synopsis:** `osatura explain <project.json> <bone-or-layer> <frame>`

„Warum ist das hier?“ - listet die geordneten Beiträge (Ruhepose, jeder Kanal mit Modus und Gewicht, Constraints, IK) auf, aus denen die Transformation eines Knochens oder einer Ebene in einem Frame entsteht, sowie den resultierenden Kopf-/Spitzen-/Weltwinkel.

```
$ osatura explain demo.json hand 12
- bone 'hand' (length 45), parent 'arm_low'
- ancestry: hand ← arm_low ← arm_up
- rest pose: offset (100.00, 0.00), angle 0.00°
- channel #2 BoneAngle: keys → 20.000 (5 key(s), Hold outside range); mode replace, weight 1.00
- channel #3 BoneAngle: expression `6 * sin(time * 2 * pi * 2)`; mode add, weight 1.00
- result at frame 12: head (174.6, 126.4), tip (203.5, 92.0), world angle -50.0°
```
**Fehler:** unbekannter Knochen-/Ebenenname; Frame außerhalb der Animation.

### `pick`

**Synopsis:** `osatura pick <project.json> --frame N --x X --y Y [--tol PX] [--root] [--bones] [--points a,b] [--locked a,b] [--json]`

Was unter einem Canvas-Punkt liegt, das Oberste zuerst (Hit-Testing wie im Studio).

| Option | Bedeutung |
|---|---|
| `--frame N`, `--x X`, `--y Y` | Erforderlich: Frame und Canvas-Koordinaten. |
| `--tol PX` | Pick-Toleranz in Canvas-Pixeln (Standard 3). |
| `--root` | Meldet den Vorfahren der obersten Ebene (`mouth`) statt der innersten Ebene (`mouth/closed`). |
| `--bones` | Berücksichtigt auch Knochen. |
| `--points a,b` | Ebenen, deren Kontrollpunkte (Anker/Handles/Mesh-Vertices) pickbar sind. |
| `--locked a,b` | Zu überspringende Ebenen (werden als gesperrt behandelt). |
| `--json` | `{"hit":{...},"hits":[...]}`. |

```
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4
hit: layer ink (distance 0.00)
  under: layer mouth/closed (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 209 --y 104 --tol 4 --root --locked mouth
hit: layer ink (distance 0.00)
  under: layer arm (distance 0.00)
  under: layer sky (distance 0.00)
$ osatura pick demo.json --frame 0 --x 300 --y 240 --json
{"hit":{"distance":0.0,"target":{"kind":"layer","path":"sky"}},"hits":[{"distance":0.0,"target":{"kind":"layer","path":"sky"}}]}
```
**Fallstrick:** Die Hintergrundebene (`sky`) ist ein normales Trefferziel, daher gibt es fast immer mindestens einen Treffer.

### `select-box`

**Synopsis:** `osatura select-box <project.json> --frame N (--rect x0,y0,x1,y1 | --lasso x,y;x,y;x,y) [--mode bbox|contain|intersect] [--root] [--bones] [--json]`

Marquee-/Lasso-Auswahl. `--mode` legt fest, wann eine Ebene als innerhalb gilt: `bbox` (ihre Bounding Box berührt den Bereich), `contain` (vollständig innerhalb), `intersect` (Standard: ihre gezeichnete Geometrie schneidet den Bereich). `--root` meldet Vorfahren der obersten Ebene, `--bones` bezieht Knochen ein.
Setze einen `--lasso`-Wert in der Shell in Anführungszeichen (das `;` trennt Punkte).

```
$ osatura select-box demo.json --frame 0 --rect 200,95,220,115 --root
layer sky
layer arm
layer mouth
layer ink
$ osatura select-box demo.json --frame 0 --rect 0,0,640,480 --mode contain
layer sun
layer shadow
...
$ osatura select-box demo.json --frame 0 --lasso "100,100;300,100;300,300" --bones --json
{"targets":[{"kind":"bone","name":"arm_low"},{"kind":"bone","name":"hand"},{"kind":"layer","path":"sky"}, ...]}
```

### `timeline`

**Synopsis:** `osatura timeline <project.json> [--json]`

Die Dope-Sheet-Ansicht als Text: Tracks pro Kanal, nach Knochen/Ebene gruppiert, mit Key-Frames (`keys @ ...`), `expr`-/`driven`-Kanälen, Sperren und Farben. Die Nummern `#N` sind die Kanalindizes, die `keyedit --channels` und die Auswahlen `#N@F0..F1` verwenden.

```
$ osatura timeline demo.json
timeline: 48 frames at 24 fps, 14 channels
Bones
  arm_up
    #0 bone_angle (keys @ 0 12 24 36 48)
    arm_low
      #1 bone_angle (keys @ 0 12 24 36 48)
      hand
        #2 bone_angle (keys @ 0 12 24 36 48)
        #3 bone_angle (expr)
  foot_ctrl
    #4 bone_offset_x (keys @ 0 12 24 36 48)
...
```
`--json` liefert Gruppen mit IDs, Eltern, Kindern, Tracks, `frames`, `locked`.

### `timecode`

**Synopsis:** `osatura timecode <frame|HH:MM:SS;FF> [--fps 24] [--drop] [--nodrop]`

SMPTE-Timecode-Umrechnung. Eine bloße Zahl ist ein Frame und gibt einen Timecode aus; ein Argument mit `:` oder `;` wird geparst und gibt die Frame-Nummer aus.
29,97, 59,94 und 119,88 fps verwenden standardmäßig Drop-Frame-Timecode (`;` vor den Frames); `--nodrop` zählt stattdessen 30 (bzw. 60/120); `--drop` erzwingt Drop-Frame (Fehler bei anderen Raten).
`--drop` und `--nodrop` schließen sich aus.

```
$ osatura timecode 100
00:00:04:04
$ osatura timecode 100 --fps 29.97
00:00:03;10
$ osatura timecode 1800 --fps 29.97 --drop
00:01:00;02
$ osatura timecode 1800 --fps 29.97 --nodrop
00:01:00:00
$ osatura timecode 00:00:04:04
100
$ osatura timecode 00:00:04;04 --fps 24
drop-frame timecode is only defined for 29.97, 59.94 and 119.88 fps         [exit 1]
```
Liest kein Projekt; `--fps` hat den Standardwert 24 (nicht die fps eines Projekts).

### `snapshot`

**Synopsis:**
```
osatura snapshot save <project.json> <dir> [--keep 20]
osatura snapshot list <project.json> <dir>
osatura snapshot restore <project.json> <dir> <out.json> [--number N]
osatura snapshot recover <project.json> <dir> <out.json>
```
`--keep` gehört nur zu `save` und `--number` nur zu `restore`; jede andere Kombination wird abgelehnt (`unknown option --keep for 'snapshot list' (applies to: snapshot save)`), ebenso ein fehlendes oder überzähliges Dateiargument.

Ein nummerierter Verlauf eines Projekts in einem Verzeichnis (unabhängig von `vcs`). `save` speichert das aktuelle Projekt als `rev-NNNNNN-<hash>.json` und behält nur die neuesten `--keep` Revisionen (Standard 20, ältere Dateien werden gelöscht); das Speichern eines Projekts, das mit der letzten Revision identisch ist, gibt `unchanged since the last revision` aus und speichert nichts. `list` gibt Revisionen aus. `restore` schreibt die Revision `--number N` nach `<out.json>`; `recover` schreibt die neueste Revision.

```
$ osatura snapshot save s.json snaps --keep 3
saved revision 1 (4922f48a86f24554)
$ osatura snapshot list s.json snaps
     1  4922f48a86f24554  snaps/rev-000001-4922f48a86f24554.json
$ osatura snapshot restore s.json snaps restored.json --number 1
wrote restored.json
$ osatura snapshot recover s.json snaps recovered.json
wrote recovered.json
```
`list` gibt bei einem fehlenden Verzeichnis nichts aus und beendet mit 0. **Verwandt:** `vcs` für Branches und Merges.

---

## 4. Rendern und Exportieren

### `render`

**Synopsis:** `osatura render <project.json> <frame> <out.svg|out.png> [--bones] [--scale N] [--clamp] [--proxy N]`

`--proxy N` rendert einen Scrub-Proxy (N = 1..4): Eine `.png` ist eine Bitmap mit 1/2^N der Auflösung, eine `.svg` besteht aus vereinfachten Vektoren; beide lassen feine Konturen, Verläufe und Überblendungen weg.

Rendert einen Frame. Ein `<frame>` an oder hinter der Projektlänge ist ein Fehler (`frame 99 is outside the animation ... valid frames are 0..N-1 ... pass --clamp`); `--clamp` rendert stattdessen den letzten Frame und gibt eine Warnung auf stderr aus. (Nur `render` hat diese Prüfung; `dof`, `render-hdr` und `fx` nicht.) Die Dateiendung wählt das Format (`.svg` Vektor, `.png` 8-Bit-RGBA-Raster); jede andere Endung wird abgelehnt, **bevor** etwas geschrieben wird.

| Option | Standard | Bedeutung |
|---|---|---|
| `--bones` | aus | Zeichnet das Skelett-Overlay (Köpfe als Punkte, Knochen als Linien). |
| `--scale N` | 1 | Ausgabeskalierung, eine Zahl in (0, 16]. Bei `.png` multipliziert sie die Pixelgröße; bei `.svg` behält das Dokument seine viewBox. |
| `--proxy N` | aus | Scrub-Proxy, N = 1..4: `.png` = Bitmap mit 1/2^N der Auflösung, `.svg` = vereinfachte Vektoren; beide lassen feine Konturen, Verläufe und Blend-Modi weg. Gedacht für interaktives Scrubbing, nicht für Lieferergebnisse. |

```
$ osatura render demo.json 12 f12.png --bones --scale 2
wrote f12.png
$ osatura render demo.json 0 /tmp/x.xyz
render writes .png or .svg; '/tmp/x.xyz' has another extension (nothing was written)       [exit 1]
$ osatura render demo.json 0 o.svg --scale 0
--scale needs a number in (0, 16]                                                         [exit 1]
$ osatura render demo.json abc o.svg
bad frame number 'abc'                                                                    [exit 1]
$ osatura render demo.json 2 px.png --proxy 2        # 640x480 project -> 160x120 PNG
wrote px.png
```
**Fehler:** fehlendes/ungültiges Projekt, nicht endliche Pose bei diesem Frame, Ausgabe entspricht dem Eingabeprojekt. **Verwandt:** `frames`, `render-hdr`, `onion`, `compare`, `fx`.
**Grenzen:** Der PNG-Rasterisierer ist der eigene der Engine (8 Bit, sRGB; das Blending ist nicht farbverwaltet, es sei denn, das Projekt aktiviert es mit `color-space ... linear`). Ebeneneffekte, Masken und Blend-Modi werden von beiden gezeichnet.

### `frames`

**Synopsis:** `osatura frames <project.json> <out_dir> [--png] [--bones] [--scale N]`

Rendert **jeden** Frame in ein Verzeichnis (wird angelegt, falls es fehlt) als `frame_0000.svg` ... (oder `.png` mit `--png`). Zuerst werden alle Frames auf endliche Posen geprüft.

```
$ osatura frames demo.json fr --png
wrote 48 frames to fr
$ ls fr | head -2
frame_0000.png
frame_0001.png
```
**Fallstrick:** Vorhandene Dateien mit denselben Namen werden überschrieben; fremde Dateien im Verzeichnis bleiben unberührt. Verwende dies als Eingabe für `ffmpeg` oder `import-seq`.

### `export`

**Synopsis:**
```
osatura export <project.json> <out.gif> [--scale N] [--report r.json]
osatura export <project.json> <out.html|out.htm>
osatura export <project.json> <out.json|out.lottie> [--report r.json]
osatura export <project.json> <out.mp4> [--scale N]
osatura export <project.json> <out.apng> [--scale N]
osatura export <project.json> <out.webm> [--scale N] [--codec vp9|av1]
osatura export <project.json> <out.png> [--scale N] [--columns N]
osatura export <project.json> <out.glb> [--report r.json]
```
(eine Form pro Format; die Endung der Ausgabedatei, ohne Beachtung der Groß-/Kleinschreibung, wählt die Form. Eine Option, die die Form nicht aufführt, wird abgelehnt: `unknown option --scale for 'export .json' (applies to: export .gif, export .mp4, ...)`. Bei einer unbekannten oder fehlenden Endung wird stattdessen der Formatfehler gemeldet.)

Die Endung der Ausgabedatei wählt das Format:

| Endung | Ergebnis | Braucht ffmpeg | Geltende Optionen |
|---|---|---|---|
| `.gif` | Endlos-GIF mit der Frame-Rate des Projekts (siehe [EXPORT_FORMATS.md](EXPORT_FORMATS.md)) | nein | `--scale`, `--report` |
| `.html`, `.htm` | ein eigenständiger HTML-Player (Inline-SVG-Frames, Play/Pause, Scrubber, optional eingebettetes Audio) | nur für Audio | - |
| `.json` | Lottie (Bodymovin) | nein | `--report` |
| `.lottie` | dotLottie-Container (Zip: `manifest.json` + `a/animation.json`) | nein | `--report` |
| `.mp4` | H.264 yuv420p (+ AAC, wenn das Projekt Sounds hat) | ja (`libx264`, `aac`) | `--scale` |
| `.apng` | animiertes PNG | nein | `--scale` |
| `.webm` | VP9 (Standard) oder AV1 mit Alpha (+ Opus-Ton) | ja (`libvpx-vp9` / `libaom-av1`, `libopus`) | `--scale`, `--codec vp9\|av1` |
| `.png` | Sprite Sheet **plus** `<out>.atlas.json`-Metadaten (nie `<out>.json`) | nein | `--scale`, `--columns N` |
| `.glb` | glTF 2.0 binär (Knochen, Animation, geskinnte Füllungen/Konturen, Verläufe, Masken, Bilder, Morph Targets) | nein | `--report` |

Optionen:

| Option | Bedeutung |
|---|---|
| `--scale N` | Raster-Skalierung (0, 16] für `.gif .mp4 .apng .webm .png`. Bei anderen Formaten wird sie abgelehnt (`unknown option --scale for 'export .json'`); vor Wave 7 war es eine Warnung. |
| `--report FILE` | Schreibt die Liste der Dinge, die das Format nicht tragen konnte, als JSON (`.gif .json .lottie .glb`): `{"format","count","warnings":[{"code","layer","message"}]}`. Dieselben Warnungen werden immer auf stderr ausgegeben. Die Datei darf nicht das Eingabeprojekt sein. |
| `--codec vp9\|av1` | Nur `.webm` (Standard `vp9`); jeder andere Wert: `unknown --codec 'foo' (vp9 or av1)`. |
| `--columns N` | Nur `.png`-Sprite-Sheet: Kacheln pro Zeile (Standard: ein annähernd quadratisches Raster, ceil(sqrt(frames)): 3 Spalten bei 8 Frames). |

Beispiele (alle an einer 8-Frame-Kopie der Demo verifiziert, `--scale 0.5`):

```
$ osatura export small.json out.gif --scale 0.5
wrote out.gif
$ osatura export small.json out.json --report rep.json
wrote out.json
warning: [variable-width-stroke] ink: variable-width stroke profiles are not supported by Lottie; the constant base width is used
warning: [gradient-anisotropic] shadow: a gradient on a layer with non-uniform scale or skew is approximated (Lottie gradients are not sheared)
$ cat rep.json
{ "count": 2, "format": "lottie", "warnings": [ {"code": "variable-width-stroke", "layer": "ink", ...}, {"code": "gradient-anisotropic", ...} ] }
$ osatura export small.json out3.png --columns 4 --scale 0.25
wrote out3.json
wrote out3.png                          # 4 columns x 2 rows of 160x120 tiles; out3.json lists {index,x,y,w,h} per frame
$ osatura export small.json out.glb
wrote out.glb
warning: [glb-limit] blend mode Screen of 'sun' is not exported (glTF materials only alpha-blend); it is drawn as normal
warning: [glb-limit] animated opacity of 'target' is not exported (frozen at frame 0)
$ osatura export small.json out.mp4 --scale 0.5       # 320x240 h264 yuv420p, 8 frames
wrote out.mp4
$ osatura export small.json out.webm --codec av1      # needs a libaom-av1 build of ffmpeg
```

Fehler und Ablehnungen:

```
$ osatura export small.json z.xyz
unknown export format '.xyz' (use .gif .html .htm .json .lottie .mp4 .apng .webm .png .glb)
$ osatura export small.json small.json
refusing to overwrite the input project small.json with output of the same name
$ osatura export ora.json ora.glb                       # a project of image layers and no bones
the project has no bones to export
$ PATH=/nonexistent osatura export small.json y.mp4
cannot run ffmpeg (...); install it or export a GIF/HTML instead                          [exit 1, nothing written]
```
Weitere Ablehnungen: `ffmpeg failed` (fehlender Encoder), `export would write about N million values ...` (die Größenschranke für Lottie/HTML/GLB: Das Limit liegt bei 40 Millionen Werten = Frames x (Anker + Knochen + Ebenen); kürze die Animation oder vereinfache die Pfade), nicht endliche Posen bei irgendeinem Frame und `image too large for GIF`.

**Ausgaben / Nebeneffekte:** `.png` schreibt außerdem die Sprite-Sheet-Metadaten nach `<out-name>.atlas.json` (z. B. `out.png` -> `out.atlas.json`) und kann daher keine Lottie-Datei `out.json` mehr ersetzen; die Begleitdatei unterliegt denselben Eingabe-/Symlink-Schutzmaßnahmen. `.mp4`/`.webm` rendern PNG-Frames in ein privates temporäres Verzeichnis, das danach entfernt wird.
**Ton:** `.mp4` und `.webm` mischen die Sound-Tracks des Projekts (Start, Gain, Trim, Fades; an der Projektlänge abgeschnitten); `.html` bettet ein gemischtes mp3 ein (Warnung ab 3 MiB, mit Warnung verworfen ab 16 MiB); `.gif` und `.apng` geben `note: GIF cannot carry audio; ...` aus und bleiben stumm; Lottie warnt mit `[audio]`.
**Transparenz:** Ein Projekt mit `"background": null` rendert überall dort mit transparenten Pixeln, wo nichts gezeichnet wird. `.apng`, das `.png`-Sprite-Sheet und `.webm` (`yuva420p`, Container-Tag `alpha_mode=1`) behalten den Alphakanal; `.gif` behält 1-Bit-Transparenz (verifiziert: Das Flag für die transparente Farbe ist gesetzt); `.mp4` hat kein Alpha. Ein Projekt mit Hintergrundfarbe (die Demo hat `#f6f1e7`) ist deckend.
**Verwandt:** `export-fit` (ein Byte-Budget einhalten), `size-report`, `lod`, `dof-export`, `export-oca`, `render-hdr`. Details zu allen Formaten: [EXPORT_FORMATS.md](EXPORT_FORMATS.md).

### `export-oca`

**Synopsis:** `osatura export-oca <project.json> <out_dir> [--layers]`

Schreibt einen **Open-Cel-Animation**-Ordner: `<out_dir>/<project name>.oca` (JSON, `ocaVersion` 1.1) plus PNG-Cels `layerN/fNNNN.png`. Ohne `--layers` gibt es eine Ebene `render` mit den zusammengesetzten Frames; mit `--layers` wird jede Ebene der **obersten Ebene** allein auf Transparenz in ihre eigene OCA-Ebene gerendert (11 Ebenen bei der Demo). `--scale N` multipliziert die Cel-Größe.

```
$ osatura export-oca small.json oca2 --layers
wrote oca2/small.oca
$ ls oca2
layer0 layer1 ... layer10 small.oca
```
**Grenzen:** Cels sind Raster (keine Vektoren, Knochen oder Kanäle bleiben erhalten); Ebenen-Deckkraft/-Blend werden nicht geschrieben (jede Ebene ist `paintlayer`, Deckkraft 1); Sounds werden nicht exportiert; der Ordner wird bei Bedarf angelegt, und Dateien darin werden überschrieben.

### `render-hdr`

**Synopsis:** `osatura render-hdr <project.json> <frame> <out.exr|out.png> [linear] [--precision half|float] [--float32] [--8bit] [--scale N] [--clamp]`

Float-Render (32 Bit) (ein `<frame>` außerhalb des Projekts ist ein Fehler, `--clamp` nimmt wie bei `render` den nächsten gültigen Frame): `.exr` schreibt ein einteiliges, zeilenweises, unkomprimiertes **OpenEXR** (standardmäßig HALF-Kanäle, FLOAT mit `--precision float` oder `--float32`), linear-light und premultipliziert; `.png` schreibt ein **16-Bit-RGBA-PNG**, sRGB-kodiert, sofern nicht das Wort `linear` als fünftes Argument angegeben wird. Der Projekthintergrund wird **nicht** gezeichnet (das Bild ist dort transparent, wo nichts gezeichnet wird). `--8bit` verwendet den alten 8-Bit-Rasterisierer als Quelle (nicht mit Float32-Ausgabe kombinierbar). `--scale N` wird angewendet.

```
$ osatura render-hdr small.json 3 f.exr
wrote f.exr (640x480)                       # 465,611 bytes: HALF
$ osatura render-hdr small.json 3 f2.exr linear --precision float
wrote f2.exr (640x480)                      # 923,211 bytes: FLOAT
$ osatura render-hdr small.json 3 f4.png
wrote f4.png (640x480)                      # PNG 16-bit/colour RGBA
$ osatura render-hdr small.json 3 f6.exr --precision bogus
unknown precision 'bogus' (use half or float)
$ osatura render-hdr small.json 3 f7.jpg
output must end in .exr or .png
```
`ffprobe` liest die EXR als `exr 640x480 gbrapf32le`. Bei `.exr` hat das Wort `linear` keine Wirkung (EXR ist immer linear). **Grenzen:** keine Kompression, ein Frame pro Datei, keine Ebenen/AOVs; die Quellgrafik hat gewöhnliche sRGB-Farben, die Datei enthält also Werte im Anzeigebereich mit höherer Genauigkeit, kein szenenbezogenes HDR-Licht. Über die Umrechnung sRGB<->linear hinaus nicht farbverwaltet.

### `render-gpu`

**Synopsis:** `osatura render-gpu <project.json> <out.png> [--frame N] [--adapter software|auto] [--scale S] [--bench N]`

GPU-Render (wgpu) eines Frames. Nur in Builds verfügbar, die mit `--features gpu` erstellt wurden; der Standard-Build antwortet:

```
$ osatura render-gpu small.json g.png
render-gpu needs a build with the gpu feature: cargo build -p osatura-cli --features gpu       [exit 1]
```
In einem GPU-Build werden nicht unterstützte Szenen (Bilder, Knochen) als Fehler gemeldet und nie falsch gezeichnet. `--bench N` ist eine Benchmark-Option. Dieser Befehl konnte im Test-Build (ohne GPU-Feature) nicht ausgeführt werden, daher stammen seine Flags nur aus dem Hilfetext.

### `compare`

**Synopsis:** `osatura compare <a.json> <b.json> <frame> <out.png> [--gain 4 --scale N]`

Schreibt ein PNG: **Vorher | Nachher | Differenz** nebeneinander (das dritte Feld ist eine Heatmap, Schwarz = identisch, Rot -> Gelb = größere Änderung, multipliziert mit `--gain`, Standard 4). Beide Projekte müssen in derselben Größe rendern (`frame sizes differ (...); compare projects of the same size`).

```
$ osatura compare small.json d2.json 3 cmp.png --gain 4 --scale 0.25
wrote cmp.png
```
**Verwandt:** `diff` (Struktur), `ci diff` (zwei PNGs, perzeptuelle Metriken), `golden`.

### `proxy-bench`

**Synopsis:** `osatura proxy-bench <project.json> [--frames N]`

Millisekunden pro Frame, ein Thread, in voller Qualität und bei jeder Scrub-Proxy-Stufe (siehe `render --proxy`).

```
$ osatura proxy-bench small.json --frames 2
2 frames of small (640x480), ms per frame, single thread:
  evaluate only           0.08
  svg (no raster)         0.48   vector proxy: level 0 0.32, level 2 0.24, level 4 0.24
  raster level 0       1360.87   (full quality)
  raster level 1        319.14   (4.3x faster than level 0)
  ...
  raster level 4          6.84   (199.0x faster than level 0)
```
(Zahlen aus dem Debug-Build). 

---

## 5. Rigging, Skins, IK und Layout-Helfer

Alle Befehle in diesem Abschnitt folgen dem Muster `osatura <cmd> <project.json> <out.json> ...`: Sie lesen ein Projekt, ändern es, **validieren es erneut**
und schreiben `<out.json>` (das der Eingabe entsprechen darf). Bei einem Fehler wird nichts geschrieben.

### `autorig`

**Synopsis:** `osatura autorig <project.json> <out.json> [seed]`

Fügt Standard-Automatisierung hinzu, die aus den **Namen** von Knochen/Ebenen erraten wird: IK-Ketten (Mix 0, damit FK weiter funktioniert), eine Atem-Schleife (braucht einen Torso-Knochen) und Blinzeln (braucht Augen-Ebenen). Gibt pro Entscheidung eine Zeile aus: `+` hinzugefügt, `-` übersprungen mit Begründung. Führt auch `eye-clip` aus.

```
$ osatura autorig small.json auto.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
- arm_r: upper/lower bone not found
- leg_l: already has an IK chain
- breathing: no torso bone
- blink: no eye layers
wrote auto.json
```
`seed` (optionales Positionsargument) initialisiert das zufällige Blinzel-Timing. **Grenzen:** rein namensbasiert (`tag` zeigt, wie Namen gelesen werden); legt nie Knochen an.

### `slider`

**Synopsis:** `osatura slider <project.json> <out.json> <name> <poses>`, wobei `poses` die Form `at:target.prop=val,target.prop=val;at:...` hat

Erzeugt einen **Controller-Knochen** `ctrl_<name>`, dessen `offset_x` mehrere Eigenschaften zwischen gespeicherten Posen steuert (ein Pose-Slider). Animiere das `bone_offset_x` des ausgegebenen Controllers.

```
$ osatura slider small.json sl.json pose "0:arm_up.bone_angle=0;24:arm_up.bone_angle=30"
controller bone: ctrl_pose (animate its offset_x)
wrote sl.json
```
`target.prop` verwendet die Eigenschaftsnamen in snake_case (`arm_up.angle` scheitert mit `unknown property 'angle'`). Der letzte Punkt trennt Target von Eigenschaft, daher funktionieren Ebenenpfade wie `grp/child.layer_opacity`.

### `view-swap`

**Synopsis:** `osatura view-swap <project.json> <out.json> <switch_layer> <bone.prop> <from:child,from:child,...>`

Steuert eine Switch-Ebene über eine Knocheneigenschaft mit einer Stufenfunktion: Die Paare `from:child` werden nach `from` sortiert, und der Switch zeigt das Kind des letzten Paares, dessen `from` nicht über dem Treiberwert liegt (`-40:0,0:1,40:2` = Kind 0 unter 0, Kind 1 ab 0, Kind 2 ab 40). Verwende es für Drehansichten (vorn/seitlich/hinten), die von einem Yaw-Knochen gesteuert werden. Die Kind-Indizes müssen existieren (`child index 5 is out of range (the layer has 3)`); die Ebene muss ein Switch sein.

```
$ osatura view-swap small.json vs.json mouth foot_ctrl.bone_offset_x "-40:0,0:1,40:2"
wrote vs.json
```

### `align` und `distribute`

**Synopsis:**
`osatura align <project.json> <out.json> <mode> <layer,layer,...|layer> [stage]`  (mode: `left right top bottom center-x center-y`)
`osatura distribute <project.json> <out.json> <x|y> <gaps|centers> <layer,layer,...>`

`align` verschiebt Ebenen so, dass die Bounding Boxes ihrer abgeflachten Kurven ausgerichtet sind (aneinander oder, bei einer einzelnen Ebene gefolgt vom Wort `stage`, an der Canvas). `distribute` verteilt drei oder mehr Ebenen gleichmäßig entlang einer Achse, wobei entweder die **Abstände** (gaps) zwischen den Boxen oder die **Mittelpunkte** (centers) angeglichen werden.

```
$ osatura align small.json al.json center-x sun,hills stage
wrote al.json
$ osatura align small.json al2.json top ground
select at least two layers (or one with --stage)                 [exit 1]
$ osatura distribute small.json di.json x gaps sun,ground,hills
wrote di.json
```
Diese bearbeiten Ebenenpositionen (statische Transformation), sodass gekeyte `layer_x`-/`layer_y`-Kanäle sie weiterhin überschreiben. (Die Fehlermeldung erwähnt `--stage`; das tatsächliche Argument ist das bloße Wort `stage`.)

### `trim`

**Synopsis:** `osatura trim <project.json> <out.json> <layer> <path_index> <start> <end> [offset]`

Backt einen *Trim Path*: ersetzt den Pfad Nummer `path_index` einer Vektorebene durch nur den Teil zwischen `start` und `end` (Bruchteile 0..1 seiner Länge, beide um `offset` verschoben; geschlossene Pfade umlaufen). Das Ergebnis ist eine **offene Polylinie mit derselben Kontur und ohne Füllung** - destruktiv, bewahre also das Original auf. Für einen Draw-on-Effekt backe pro Frame eine getrimmte Kopie.

```
$ osatura trim small.json tr.json ink 0 0 0.5 0.25
wrote tr.json
```

### `reparent`

**Synopsis:** `osatura reparent <project.json> <out.json> <bone> <new_parent|none>`

Ändert das Elternteil eines Knochens, **ohne ihn zu bewegen**: Der Ruhe-Offset und -Winkel werden so neu berechnet, dass die Ruhepose unverändert bleibt (verifiziert: Danach unterscheiden sich nur `parent`, `rest_angle`, `rest_offset`). `none` macht ihn zur Wurzel. Ein Elternteil, das einen Zyklus erzeugen würde, wird abgelehnt.

```
$ osatura reparent small.json rp.json hand none
wrote rp.json
$ osatura diff small.json rp.json
~ bone 'hand' changed (parent, rest_angle, rest_offset)
```

### `parent-switch`

**Synopsis:** `osatura parent-switch <project.json> <out.json> <bone> <frame:parent|none,...>`

Backt animiertes Umhängen von Eltern (eine Hand, die ein Requisit greift): Bei jedem Wechsel-Frame wird die Weltpose des Knochens relativ zum neuen Elternteil erfasst und beibehalten (starres Folgen) oder bei `none` im Weltraum eingefroren. Der Knochen wird zur Wurzel, und seine Weltpose wird für jeden Frame als einfache Offset-/Winkel-Keys geschrieben; seine alten Kanäle bleiben in der Datei, werden aber von den gebackenen überschrieben. **Grenzen:** abgelehnt für Knochen, die an IK, Constraints oder Spring-Knochen beteiligt sind; die Skalierung wird nicht übernommen.

```
$ osatura parent-switch demo.json ps.json hand 0:none,24:arm_up
wrote ps.json
```

### `skin`

**Synopsis (Unterbefehle):**
```
osatura skin <project.json> <out.json> add <group> <name> <layer,layer,...>
osatura skin <project.json> <out.json> rig <group> <name> <constraint,...|-> <ik,...|->
osatura skin <project.json> <out.json> bones <group> <name> <bone,...|->
osatura skin <project.json> <out.json> apply <group> <name>
osatura skin <project.json> <out.json> schedule <group> <frame:name,frame:name,...>
```

Ein *Skin-Set* wird durch ein `<group>`-Label identifiziert (ein „Outfit“- oder „Ausdrucks“-Set; im Beispiel entspricht es zufällig dem Namen einer Ebenengruppe). Jeder benannte Skin zeigt eine Menge von Ebenen, angegeben über den **Ebenenpfad** (z. B. `skins/box`); das Anwenden eines Skins zeigt seine Ebenen und blendet die Ebenen jedes anderen Skins derselben Gruppe aus.
`add` definiert einen Skin; `rig` listet Constraints und IK-Ketten auf, die nur aktiv sind, solange dieser Skin es ist; `bones` listet Knochen auf, die nur existieren, solange er aktiv ist (sonst auf Skalierung 0 zusammengeklappt); `apply` macht einen Skin statisch aktiv; `schedule` keyt den aktiven Skin über die Zeit als Step-Keys auf der **Ebenen-Deckkraft** (animiere die Deckkraft dieser Ebenen daher nicht anderweitig). `-` bedeutet „keine“.

```
$ osatura skin sc.json sk1.json add skins red skins/box
wrote sk1.json
$ osatura skin sk1.json sk2.json add skins green skins/box2
$ osatura skin sk2.json sk3.json apply skins green
$ osatura skin sk3.json sk4.json schedule skins 0:red,4:green
$ osatura skin sk4.json sk5.json bones skins red -
$ osatura skin sk4.json sk5.json rig skins red - leg
$ osatura skin sc.json x.json add skins red box
no layer 'box'                                                  [exit 1]     (use the full path skins/box)
```
**Grenzen (ROADMAP §9):** Ein Knochen, der nur in einem Skin existiert, wird nie aus dem ausgewerteten Rig entfernt (die Indizes bleiben; er wird auf 0 skaliert); eine Ebene, die auf einen nur in einem Skin vorhandenen Knochen gewichtet ist, aber nicht in diesem Skin liegt, verschwindet mit ihm, und das wird nicht geprüft.

### `bone-vis`

**Synopsis:** `osatura bone-vis <project.json> <out.json> <bone> <from> <to>|clear`

Zeigt den Knochen nur für einen Frame-Bereich in den **Overlays** des Studios (Kontrolle über Unübersichtlichkeit). Ändert weder Rendering noch Export. `clear` entfernt den Bereich (`range removed`).

### `chan-meta`

**Synopsis:** `osatura chan-meta <project.json> <out.json> <target> <prop|*> [color=#rrggbb] [shy=on|off] [solo=on|off] [lock=on|off]`

Dope-Sheet-Metadaten eines Kanals: Anzeigefarbe, *Shy* (in Listen ausgeblendet), *Solo* (nur diese Kanäle im Studio in der Vorschau) und **Lock** (lehnt Bearbeitungen ab: gesperrte Kanäle werden von `keys` übersprungen, von `keyedit`, `audio-react` und `live-map` abgelehnt und von `clip bake` nie angefasst). Auswertung und Export ignorieren alle vier. `*` wählt jede Eigenschaft des Targets.

```
$ osatura chan-meta small.json cm.json arm_up bone_angle 'color=#ff0000' shy=on lock=on
wrote cm.json
```

### `reference`

**Synopsis:** `osatura reference <project.json> <out.json> <layer> <on|off>`

Markiert eine Ebene als **Pausreferenz** (Tracing-Referenz): im Studio sichtbar, nie exportiert.

### `parallax`

**Synopsis:** `osatura parallax <project.json> <out.json> <far_layer,...,near_layer> <far_value> <near_value>`

Weist den aufgeführten Ebenen der obersten Ebene in der angegebenen Reihenfolge (erste = am weitesten entfernt) eine Multiplane-Tiefe zu (`parallax`-Werte, verteilt zwischen `far_value` und `near_value`). Bei Kamerabewegung verschieben sich die Ebenen um unterschiedliche Beträge; `dof` unscharft nach `|parallax - focus|`.

```
$ osatura parallax small.json pa.json sky,hills,ground 0.2 1.0
wrote pa.json
```

### `eye-clip`

**Synopsis:** `osatura eye-clip <project.json> <out.json>`

Maskiert jede Iris-/Pupillen-Ebene mit der Augenform-Ebene auf derselben Seite (über den Namen gefunden, siehe `tag`) mithilfe einer gewöhnlichen Ebenenmaske, sodass Pupillen das Auge nie verlassen, während sie sich bewegen. Wird auch von `autorig` ausgeführt. Werden keine passenden Namen gefunden, wird die Datei trotzdem unverändert geschrieben.

### `comp`

**Synopsis:**
```
osatura comp capture|apply <project.json> <out.json> <name>
osatura comp list <project.json>
```

Benannte Ebenenzustände („Comps“: Sichtbarkeit, Deckkraft und das gewählte Kind von Switch-Ebenen, pro Ebenenpfad; **keine** Transformationen). `capture` speichert den aktuellen Zustand als `name`, `apply` stellt ihn wieder her, `list` gibt die Namen aus (`list` nimmt keine Ausgabedatei und schreibt nichts; ein altes viertes Wort, `comp list p.json p.json`, wird weiterhin akzeptiert und ignoriert).

```
$ osatura comp capture small.json c1.json pose1
wrote c1.json
$ osatura comp list c1.json c1.json
pose1 (14 layers)
$ osatura comp apply c1.json c2.json pose1
wrote c2.json
```

### `retarget`

**Synopsis:** `osatura retarget <source.json> <dest.json> <out.json> --pairs srcChain=dstChain[,..]`

Retargeting nach *Absicht*: Die Animation des IK-**Effektors** (Target-Knochen) jeder Quellkette wird mit dem Verhältnis der Reichweiten beider Gliedmaßen (Summe der Knochenlängen) skaliert und auf den Target-Knochen der Zielkette umgelenkt. Nicht gekeyte Effektor-Kanäle lassen sich nicht skalieren; sie werden umbenannt durchgereicht und als `note:`-Zeilen gemeldet. `--pairs srcChain=dstChain[,...]` ist erforderlich. Im Beispiel sind Quelle und Ziel dasselbe Projekt, was nur als Smoke-Test nützlich ist.

```
$ osatura retarget demo.json demo.json rt.json --pairs leg=leg
wrote rt.json (+14 channels)
```
**Verwandt:** `mocap-retarget` für BVH-Daten; `walk`, `mirror`.

---

## 6. Animation, Keys, Timeline und Generatoren

### `keys`

**Synopsis:**
```
osatura keys shift <project.json> <out.json> --by N [--from F --to F --target NAME --prop PROP]
osatura keys scale <project.json> <out.json> --factor X [--pivot F --from F --to F --target NAME --prop PROP]
osatura keys ease <project.json> <out.json> --ease linear|step|ease_in|ease_out|ease_in_out|smooth [--from F --to F --target NAME --prop PROP]
osatura keys reverse <project.json> <out.json> [--from F --to F --target NAME --prop PROP]
```
Jede Operation akzeptiert nur ihre eigenen Optionen (`keys shift --factor 2` wird abgelehnt); `--by`, `--factor` und `--ease` sind für ihre jeweilige Operation erforderlich (ein ausgelassenes `--by` oder `--factor` war früher ein stilles No-op).

Massenoperationen auf Keys über **alle gekeyten Kanäle** (gefiltert durch `--target` / `--prop`; ein Bereich `--from/--to` begrenzt, welche Keys angefasst werden). Gesperrte Kanäle werden mit `skipped locked channel ...` übersprungen.

| Operation | Benötigt | Wirkung |
|---|---|---|
| `shift` | `--by N` | Verschiebt Keys um N Frames. |
| `scale` | `--pivot F --factor X` | Skaliert die Key-Zeiten um den Pivot-Frame. |
| `ease` | `--ease E` | Setzt das Easing der Keys (benannte Easings; für `bezier:` verwende `keyedit ease`). |
| `reverse` | - | Kehrt die Key-Reihenfolge innerhalb des Bereichs um. |

```
$ osatura keys shift small.json k1.json --target arm_up --prop bone_angle --by 2
wrote k1.json (5 key(s) changed)
$ osatura keys scale small.json k2.json --pivot 0 --factor 0.5 --from 0 --to 48
wrote k2.json (37 key(s) changed)
$ osatura keys reverse small.json k4.json
wrote k4.json (37 key(s) changed)
```
Keys, die außerhalb von `0..frames-1` verschoben werden, bleiben in der Datei (lint meldet `L007`); dieser Befehl verlängert die Projektlänge nicht. **Verwandt:** `keyedit` (präzise, auswahlbasiert), `loop-solve`.

### `keyedit`

**Synopsis:** `osatura keyedit <project.json> <out.json|-> <sub-command> [options]`

Dope-Sheet-Bearbeitungen als **ein Undo-sicherer Batch**. Mit `-` als Ausgabe wird nichts geschrieben, und die geplante Operation wird als JSON ausgegeben (Dry Run). Unterbefehle und Optionen:

```
osatura keyedit <project.json> <out.json|-> move --sel SEL [--dt N] [--dv N] [--collision replace|merge|refuse] [--no-snap] [--free]
osatura keyedit <project.json> <out.json|-> scale --sel SEL --pivot F --factor X | dup --sel SEL --offset N | delete --sel SEL | ease --sel SEL --ease E | value --sel SEL --value V | smooth --sel SEL [--kind monotone|catmull_rom]
osatura keyedit <project.json> <out.json|-> copy --sel SEL --clip clip.json | paste --clip clip.json --at F [--remap from_target:prop=to_target:prop]
osatura keyedit <project.json> <out.json|-> add --target T --prop P --at F [--value V] [--ease E] | ripple --at F --by N --mode insert|delete [--scope 0,1] [--adjust-frames]
osatura keyedit <project.json> <out.json|-> offset --channels 0,1 --dt N | extrapolate --channels 0,1 --extrapolate loop | loop --channels 0,1 [--end F] | handle --sel SEL --side in|out --t F --v V [--mode free|aligned|auto] | lock --target T [--prop P] [--value on|off]
```

* **SEL** sind Elemente der Form `TARGET:PROP[@F0..F1]`, durch `;` verbunden, oder `#CHANNEL[@F0..F1]` (Kanalindex aus `timeline`), oder `all`. Setze es in Anführungszeichen (`;`, `@`, `#`).
* **E** (Easing) ist `linear`, `step`, `ease_in`, `ease_out`, `ease_in_out`, `smooth`, `bounce`, `bezier:x1,y1,x2,y2` oder ein JSON-Objekt.
* `--collision` legt fest, was passiert, wenn ein verschobener Key auf einem vorhandenen Key landet (`replace` Standard, `merge`, `refuse`); `--no-snap` behält gebrochene Frames bei; `--free` hebt die Beschränkung auf ganze Frames auf.
* Ausgabezeile: `<op>: N keys changed, wrote <out>`. `copy` mit echter Ausgabe gibt `copy: 0 keys changed, project unchanged, clip saved` aus.
* Operationen, die einen Key außerhalb von `0..=frames` platzieren würden, werden mit einer präzisen Meldung abgelehnt (`duplicate: frame 52 is outside the project range 0..=48`), sofern nicht `ripple --adjust-frames` das Projekt verlängert.

```
$ osatura keyedit demo.json - move --sel "arm_up:bone_angle@12..36" --dt 2        # dry run: prints the batch JSON
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", ... } ] } }
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 2 --collision merge
move: 3 keys changed, wrote k1.json
$ osatura keyedit demo.json k5.json ease --sel "arm_up:bone_angle" --ease bezier:0.4,0,0.2,1
ease: 5 keys changed, wrote k5.json
$ osatura keyedit demo.json k7.json smooth --sel "arm_up:bone_angle" --kind monotone
smooth: 4 keys changed, wrote k7.json
$ osatura keyedit demo.json k8.json copy --sel "arm_up:bone_angle@0..12" --clip c2.json
copy: 0 keys changed, project unchanged, clip saved
$ osatura keyedit demo.json k8.json paste --clip c2.json --at 30 --remap arm_up:bone_angle=arm_low:bone_angle
paste: 2 keys changed, wrote k8.json
$ osatura keyedit demo.json k9.json add --target arm_up --prop bone_angle --at 6 --value 5 --ease linear
add: 1 keys changed, wrote k9.json
$ osatura keyedit demo.json k10.json ripple --at 12 --by 6 --mode insert --adjust-frames
ripple: 28 keys changed, wrote k10.json            # project now 54 frames
$ osatura keyedit demo.json k1.json move --sel "arm_up:bone_angle@12..36" --dt 12 --collision refuse
a key already exists at frame 48                    [exit 1]
$ osatura keyedit demo.json k1.json move --sel "zzz:bone_angle" --dt 1
nothing matches 'zzz:bone_angle'                    [exit 1]
$ osatura keyedit demo.json k15.json lock --target arm_up --prop bone_angle --value on
lock: 1 keys changed, wrote k15.json
```
**Fallstricke:** `delete` mit `#0@12..12` entfernt genau einen Key; `ripple --mode delete --scope 0,1` beschränkt die Verschiebung auf die Kanäle 0 und 1; `copy` und `paste` tauschen eine Clip-Datei aus, nicht die System-Zwischenablage. Gesperrte Kanäle lehnen `move/scale/...` ab (verwende zuerst `lock --value off`).

### `marker`

**Synopsis:** `osatura marker <project.json> <out.json> <name> <frame> [--end F]`

Fügt einen Timeline-Marker hinzu (einen benannten Punkt oder, mit `--end`, einen Bereich). Ein Marker mit demselben Namen wird ersetzt. Marker werden nach Frame sortiert.

```
$ osatura marker small.json m1.json hit 4 --end 6
wrote m1.json (marker 'hit' at 4 = 00:00:00:04)
```

### `event`

**Synopsis:** `osatura event <project.json> <out.json> <name> <frame> [--data JSON]`

Fügt ein benanntes Event (einen Runtime-Trigger für Player) bei einem Frame hinzu, mit optionaler JSON-Nutzlast. Ungültiges JSON: `--data is not valid JSON: ...`.

```
$ osatura event small.json ev1.json boom 3 --data '{"power":2}'
wrote ev1.json (1 events)
```

### `idle`

**Synopsis:**
```
osatura idle blink <project.json> <out.json> --target T --prop P [--interval S --jitter X --closed F --seed N]
osatura idle breath <project.json> <out.json> --target T --prop P [--amp A --period F]
osatura idle sway <project.json> <out.json> --target T --prop P [--amp A --freq HZ --seed N]
```
Jeder Generator akzeptiert nur seine eigenen Optionen (`idle breath --seed 2` wird abgelehnt).

Erzeugt einen prozeduralen Idle-Kanal und **hängt** ihn **an** (vorhandene Kanäle bleiben).

| Generator | Optionen (Standardwerte) | Typisches Target |
|---|---|---|
| `blink` | `--interval` mittlere Sekunden zwischen Blinzeln, `--jitter`, `--closed` Frames geschlossen, `--seed` | ein `layer_switch` (Auge offen/geschlossen) oder `layer_scale_y` |
| `breath` | `--amp` (0.03), `--period` Frames (3 s bei den fps des Projekts) | `bone_scale` eines Torsos |
| `sway` | `--amp` (2), `--freq` Hz (0.3), `--seed` (1) | `bone_angle` |

```
$ osatura idle blink small.json i1.json --target mouth --prop layer_switch --interval 1 --seed 3
wrote i1.json
$ osatura idle sway small.json i3.json --target arm_up --prop bone_angle --amp 3 --freq 0.5
wrote i3.json
```
`--target` und `--prop` sind erforderlich (`--target is required`).

### `follow` (zwei Formen)

**Follow-through (Optionen):** `osatura follow <project.json> <out.json> --target SRC --prop P --chain a,b,c [--delay F --decay X]`
Fügt den Knochen der Kette verzögerte, abklingende Kopien eines vorhandenen Kanals hinzu (`--target`/`--prop` benennen die Quelle): Overlapping Action / Follow-through (`--delay` Frames pro Glied, Standard 3; `--decay` pro Glied, Standard 0.7). Eine verzögerte Kopie wird beim letzten Frame des Projekts abgeschnitten (nach dem Frame `frames` wird kein Key geschrieben, sodass `lint` L007 nicht auslöst); die Frames zwischen dem letzten behaltenen Key und dem Schnitt werden abgetastet, sodass die verzögerte Kurve bei jedem ganzen Frame exakt ist.

```
$ osatura follow small.json f1.json --target arm_up --prop bone_angle --chain arm_low,hand --delay 2 --decay 0.6
wrote f1.json (+2 channels)
```

**Pfadfolge (positional):** `osatura follow <project.json> <out.json> <layer|bone:NAME> <layer_with_path> <start_frame> <end_frame> [no-orient]`
Backt eine Ebene (oder den Wurzelknochen `bone:NAME`), die sich zwischen zwei Frames entlang des ersten Pfads einer anderen Ebene bewegt und sich zur Pfadtangente dreht, sofern nicht `no-orient` angegeben ist.

```
$ osatura follow small.json fp.json target ink 0 7
wrote fp.json
$ osatura follow small.json fp2.json bone:hand ink 0 7 no-orient
path following needs a root bone (no parent)                      [exit 1]
```

### `take-record`, `take-apply`, `take-list`

**Synopsis:**
```
osatura take-record <project.json> <library.json> <name> <from> <to> [target_prefix,...]
osatura take-apply  <project.json> <out.json> <library.json> <name> <at> [from=to,...] [speed] [--blend FRAMES]
osatura take-list   <library.json>
```
Ein **Take** ist ein wiederverwendbarer Ausschnitt einer Animation, der in einer Bibliotheksdatei gespeichert wird. `take-record` erfasst die Frames `from..to` der Kanäle, deren Target mit einem der Präfixe beginnt (alle Kanäle, wenn ausgelassen); die Bibliothek wird angelegt, falls sie fehlt, und ein Take mit demselben Namen wird ersetzt. `take-apply` fügt den Take bei Frame `at` ein, optional mit **Retargeting** (`from=to`-Paare benennen Targets um) und geänderter `speed`; die Argumente werden strikt geprüft: Ein Eintrag ohne `=`, ein Quellname, für den der Take keinen Track hat, oder eine Geschwindigkeit, die keine positive Zahl ist, ist ein Fehler, und eine nicht lesbare Bibliothek nennt die Datei (`cannot read the take library ...`); `--blend N` blendet an beiden Enden über N Frames mit der vorhandenen Animation über.

```
$ osatura take-record demo.json lib.json wave 0 24 arm_
recorded 2 tracks into lib.json
$ osatura take-list lib.json
wave (24 frames, 2 tracks)
$ osatura take-apply demo.json ta.json lib.json wave 10 arm_up=arm_low 1.5 --blend 4
applied 2 tracks; wrote ta.json
```

### `clip`

**Synopsis:**
```
osatura clip add <project.json> <out.json> <library.json> <take> <clip> [--at F --speed S --loops N --mode replace|additive|multiply --weight W
                   --include a_*,b --exclude c --track-weight arm_*=1,leg_*=0.2 --rename a=b --fade-in F --fade-out F]
osatura clip list <project.json>
osatura clip remove <project.json> <out.json> <clip>
osatura clip set-weight <project.json> <out.json> <clip> [W] [--keys frame:w,frame:w] [--track PATTERN[:prop]]
osatura clip bake <project.json> <out.json>
```
Ein geschichteter **Clip-Stack** (nichtlineare Animation): Clips verweisen auf Takes einer Bibliothek, spielen bei `--at` mit `--speed`/`--loops` und blenden `replace`, `additive` oder `multiply` mit einem Gewicht (konstant, über die Zeit gekeyt oder pro Track-Muster). `clip add/set-weight/remove` bearbeiten nur den Stack; **`clip bake` erzeugt die dem Stack gehörenden Kanäle neu** (von Hand erstellte und gesperrte Kanäle werden nie angefasst).

```
$ osatura clip add demo.json cl1.json lib.json wave w1 --at 6 --speed 1 --loops 2 --mode additive --weight 0.8 --fade-in 2 --include 'arm_*' --rename arm_up=hand
added clip 'w1' (1 clips); run clip bake to generate channels
$ osatura clip list cl1.json
1. w1 take 'wave' frames 6..30 x1 speed 1 Replace weight 1
0 baked channels
$ osatura clip set-weight cl1.json cl2.json w1 0.5
updated clip 'w1'; run clip bake to update channels
$ osatura clip set-weight cl1.json cl3.json w1 --keys 0:0,12:1
$ osatura clip bake cl2.json cl5.json
baked 1 clips into 4 channels (0 replaced); wrote cl5.json
$ osatura clip remove cl2.json cl6.json w1
removed clip 'w1'; run clip bake to update channels
```
(Das Beispiel `clip list` zeigt das zweite `clip add` mit demselben Namen, das das erste ersetzt hat.) **Grenzen (ROADMAP §9):** Gebackene Kanäle werden pro ganzem Frame abgetastet; es gibt kein `clip refresh`, um einen bearbeiteten Bibliotheks-Take neu zu laden.

### `make-loop`

**Synopsis:** `osatura make-loop <project.json> <out.json> [targets|all] [props] [--inclusive]`

Lässt von Hand gekeyte `replace`-Kanäle dort enden, wo sie begonnen haben: Jede Kurve wird pro Frame abgetastet, die Gerade von ihrem ersten zu ihrem letzten Wert wird abgezogen (Drift-Entfernung) und der erste Wert wieder addiert. Auf den X-Offset eines wandernden Knochens angewendet, wird aus einem Walk ein Walk auf der Stelle. Die Schleifenperiode ist N = die `frames` des Projekts, sodass wiederholtes Abspielen von `0..N-1` keinen doppelten Nahtframe zeigt. `--inclusive` ist die ältere Konvention, bei der der letzte Frame dem ersten entspricht. Ohne Argumente wird jeder gekeyte Kanal verarbeitet. Gibt pro geändertem Kanal eine Zeile `removed drift` aus.

```
$ osatura make-loop demo.json ml2.json foot_ctrl bone_offset_x
foot_ctrl BoneOffsetX: removed drift 0.79
wrote ml2.json
$ osatura make-loop demo.json ml.json all bone_angle --inclusive
```
**Grenzen:** klassische lineare Trendbereinigung, keine Neu-Zeitgebung der Bewegung; ausdrucksgesteuerte Kanäle werden nicht angefasst. **Die Fortbewegung wird absichtlich entfernt** (ein Projekt, das `0..N-1` wiederholt abspielt, kann nicht ewig wandern, daher ist die Schleife auf der Stelle): Ein Kanal, dessen gesamte Änderung Fortbewegung war, wird konstant, und der Befehl gibt dafür eine Zeile `note: ... is now constant` aus; `lint` meldet diesen Kanal dann als L005 (Information), was zu erwarten ist. Benenne Targets und Props, um die Fortbewegung anderer Kanäle in Ruhe zu lassen, oder verwende `loop-solve` für Bewegung, die bereits zyklisch ist.

### `loop-solve`

**Synopsis:** `osatura loop-solve <project.json> <out.json> [--window 24] [--inclusive]`

Löst die **Naht**: Blendet das Ende gekeyter Kanäle über ein Fenster in ihren Anfang über, sodass die Animation ohne Sprung loopt. Gibt den Nahtfehler vorher und nachher aus.

```
$ osatura loop-solve demo.json ls.json --window 24
wrote ls.json: 8 channel(s) adjusted, seam 0.000 -> 0.000
```
`--window` ist in Frames angegeben (Standard 24). Vergleiche den `loop`-Contract von `lint` (`contracts`).

### `walk`

**Synopsis:** `osatura walk <project.json> <out.json> --thigh-l B --thigh-r B [--shin-l B --shin-r B --arm-l B --arm-r B --hips B --period N --stride DEG --knee DEG --bob PX --swing DEG]`

Erzeugt einen Loop-Walk-Zyklus als gekeyte Kanäle auf den Knochen, die du benennst (mindestens eine Knochenoption ist erforderlich: `name at least one bone (--thigh-l, --thigh-r, ...)`). Standardwerte: `--period 24` Frames, `--stride 28`, `--knee 40`, `--bob 4`, `--swing 22`. Die Kanäle verwenden Loop-Extrapolation und werden **angehängt**.

```
$ osatura walk small.json wk.json --thigh-l arm_up --thigh-r arm_low --period 8 --stride 20
wrote wk.json (+2 channels)
```

### `mirror`

**Synopsis:** `osatura mirror <project.json> <out.json>`

Spiegelt alle gekeyten Kanäle an der vertikalen Achse: Targets tauschen die Seiten (Präfixe `L_`/`R_`, Suffixe `_L`/`_R` und `_l`/`_r`, die Wörter `left`/`right`), und Rotationen und horizontale Translationen wechseln das Vorzeichen. Die gespiegelten Kanäle **ersetzen** die Kanäle des Projekts (sie werden nicht neben den Originalen hinzugefügt). Die Ergebniszeile lautet `wrote X (N of M channel target(s) swapped sides)`; hat kein Target einen Links/Rechts-Namen oder das Projekt keine Kanäle, weist eine Warnung auf stderr darauf hin (es werden nur Vorzeichen umgekehrt).

### `foot-lock`, `stretch`, `magnet`

```
osatura foot-lock <project.json> <out.json> <ctrl_bone> <ground_y> [tolerance] [min_frames]
osatura stretch   <project.json> <out.json> <follower_bone> <leader_bone> [min=1] [max=3]
osatura magnet    <project.json> <out.json> <ctrl_bone> <bone:NAME|x,y> <radius> <strength 0..1>
```

* **`foot-lock`** tastet die Weltposition des Controllers in jedem Frame ab; Abschnitte von mindestens `min_frames` Frames, in denen er am Boden ist (die Stage-y-Achse zeigt nach unten: Der Fuß ist unten, wenn `y >= ground_y - tolerance`), sind *Kontakte*, und ein **additiver** Korrekturkanal hält den Controller dort fest, wo jeder Kontakt begann. Die ursprüngliche Animation bleibt unberührt; lösche die hinzugefügten Kanäle, um es rückgängig zu machen.
  ```
  $ osatura foot-lock demo.json fl.json foot_ctrl 385 2 2
  contact frames 23..47 pinned at (479.2, 384.1)
  wrote fl.json
  ```
* **`stretch`** lässt `follower_bone` auf den Kopf von `leader_bone` zielen und skaliert (gleichmäßig, zwischen `min` und `max`, Standardwerte 1 und 3), sodass seine Spitze ihn erreicht. Gebacken als additiver Winkelkanal plus multiplizierender Skalierungskanal; die Kinder des Followers werden mitskaliert. Der Leader darf nicht vom Follower abhängen (`the leader depends on the follower (it is one of its descendants)`).
  ```
  $ osatura stretch p.json st.json flw lead 1 2
  corrected 8 frames
  wrote st.json
  ```
* **`magnet`** backt einen Näherungszug eines Controllers zu einem Knochenkopf (pro Frame verfolgt) oder einem festen Punkt: weicher Abfall, volle `strength` (0..1) am Ziel und null bei `radius`. Gebacken als additive Offset-Kanäle.
  ```
  $ osatura magnet p.json mg.json lead 200,100 200 0.8
  pulled on 8 frames
  wrote mg.json
  ```
**Grenzen (ROADMAP §9):** Alle drei sind gebackene Ergebnisse aus Abtastung pro Frame: Führe sie nach dem Bearbeiten der Quellanimation erneut aus. `stretch` verwendet eine gleichmäßige Skalierung. `foot-lock` und `magnet` erwarten einen Wurzel-Controller-Knochen.

### `shot`, `camera-seq`

```
osatura shot       <project.json> <out.json> <name> <x> <y> <zoom> [rotation]
osatura camera-seq <project.json> <out.json> <frame:shot[:cut],...>
```
`shot` speichert einen benannten **Kameraausschnitt** (Mittelpunkt x, y, Zoom, optional Rotation in Grad). `camera-seq` keyt die Kamera durch gespeicherte Shots: Bei jedem `frame` bewegt sich die Kamera (mit Easing) zum Shot oder springt mit dem Suffix `:cut` zu ihm. Schreibt die Kanäle `camera_x/y/zoom/rotation`.

```
$ osatura shot small.json sh.json wide 320 240 1 15
wrote sh.json (1 shots)
$ osatura shot sh.json sh2.json close 400 200 2
wrote sh2.json (2 shots)
$ osatura camera-seq sh2.json cs.json 0:wide,4:close:cut
wrote cs.json
```

### `spring-fit`

**Synopsis:** `osatura spring-fit --start V --samples a,b,c,... [--fps 24]`

Passt **Frequenz (Hz) und Dämpfungsgrad** einer Feder an eine gezeichnete Antwortkurve an (Samples pro Frame eines Sprungs von `--start` zum letzten Wert). Gibt das angepasste Paar und das JSON zum Einfügen in die `springs` eines Projekts aus. Liest kein Projekt.

```
$ osatura spring-fit --start 0 --samples 0.5,0.9,1.1,1.0,0.98,1.0 --fps 24
frequency 4.68 Hz, damping 0.61 (rms error 0.028 of the step size)
add to the project: {"bone": "<bone>", "frequency": 4.68, "damping": 0.61}
```

### `script`

**Synopsis:** `osatura script <project.json> <out.json> <script.rhai>`

Führt ein **Rhai-Skript in einer Sandbox** aus, das das Projekt über validierte Anfragen bearbeitet (eine abgelehnte Anfrage lässt alles unverändert:
`request 5 (box.LayerOpacity at 0) was rejected, nothing changed: ...`). Eingebaute Funktionen: `fps() frames() bones() layers() set_key(target, prop, frame, value[, ease]) add_bone(name, parent (empty = root), length, x, y, angle) add_rect add_circle add_polygon|add_blob add_line add_gradient_rect add_ik add_aim add_copy_rotation add_pin add_group(name[, [layers]]) add_switch move_layer set_parent delete_layer delete_bone set_layer set_bone add_channel_expr add_noise add_marker add_event bone_info layer_info world print`.
Ein Skript, das keine Anfrage anwendet, gibt `warning: script: 0 requests were applied` aus. Builds ohne das Feature `scripting` antworten mit `this build has no scripting support`.

```
$ cat t.rhai
add_rect("box", 10.0, 10.0, 50.0, 50.0, "#ff0000");
add_rect("box2", 80.0, 10.0, 50.0, 50.0, "#00ff00");
add_group("skins", ["box", "box2"]);
set_key("skins/box", "layer_opacity", 0, 1.0);
print(fps());
$ osatura script small.json sc.json t.rhai
24.0
4 request(s) applied; wrote sc.json
```
Ebenen innerhalb einer Gruppe werden über den Pfad angesprochen (`skins/box`). `print` geht an stdout.

---

## 7. Physik, Cloth und Deformer

Alle diese Befehle **backen** eine Simulation oder ein geometrisches Ergebnis in gewöhnliche gekeyte Kanäle: Das Projekt bleibt deterministisch, und das Ergebnis lässt sich überall abspielen (auch in Lottie/GLB, soweit zutreffend). Führe den Befehl erneut aus, nachdem du die Quellanimation geändert hast.
Sie nehmen `<project.json> <out.json>` und Wörter der Form `key=value` (ohne `--`).

### `physics`

**Synopsis:**
```
osatura physics <project.json> <out.json> <bone,bone,...> [gravity=900] [wind=x,y] [gust=0.5] [damping=0.985]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [follow=bone] [--allow-empty]
    [drive=bone:gain;...] [vortex=bone|x,y:swirl[:radial[:radius]];...] [preset=hair|tail|rope|antenna|cape]
    [merge=keep|replace] [lateral=0..1]
```
Backt eine Verlet-**Kettensimulation** (Haare, Schwanz, Seil, Antenne, Umhang) auf die aufgeführten Knochen (Wurzel der Kette zuerst) als `bone_angle`-Keys, einen pro Frame.

| Wort | Bedeutung |
|---|---|
| `gravity=` | Beschleunigung nach unten (Standard 900 Stage-Einheiten/s^2). |
| `wind=x,y`, `gust=`, | Konstanter Windvektor; Böenamplitude eines rauschmodulierten Winds. |
| `damping=` | Geschwindigkeitsdämpfung pro Schritt (Standard 0.985). |
| `circle=x,y,r[@bone];...`, `capsule=x1,y1,x2,y2,r[@bone];...` | Collider, optional an einen Knochen gebunden (sie folgen ihm). Kapseln werden akzeptiert (verifiziert mit `capsule=100,300,300,300,20`; ein älterer Hinweis „capsule colliders NOT done“ in ROADMAP §9 ist veraltet). |
| `follow=bone` | Die Wurzel der Kette wird mit der Bewegung dieses Knochens mitgeführt (Trägheit). |
| `drive=bone:gain;...` | Körper, die die Kette anschieben (ihre Geschwindigkeit mal Gain). |
| `vortex=bone\|x,y:swirl[:radial[:radius]];...` | Wirbelfelder um einen Knochen oder Punkt. |
| `preset=` | Startwerte für `hair tail rope antenna cape`; Wörter, die **nach** `preset=` stehen, überschreiben es. |
| `merge=keep\|replace` | `keep` (Standard) lässt Knochen, die bereits Animation haben, unberührt; `replace` überschreibt sie. |
| `lateral=0..1` | Bei mehreren Ketten (`bone,bone/bone,bone`) bindet es benachbarte Stränge aneinander. |
| `--allow-empty` | Akzeptiert einen Lauf, der nichts gebacken hat. |

```
$ osatura physics p.json ph.json tail1,tail2,tail3 gravity=500 preset=tail
baked 3 bones; wrote ph.json
$ osatura physics p.json ph4.json tail1,tail2/tail3 lateral=0.5            # two strands
baked 3 bones; wrote ph4.json
$ osatura physics p.json ph3.json tail1,tail2,tail3 follow=arm_up circle=300,300,40 wind=10,0 gust=0.5 lateral=0.3 drive=lead:0.5
baked 3 bones; wrote ph3.json
$ osatura physics demo.json ph.json arm_low,hand gravity=500 preset=tail   # these bones are keyed already
kept existing animation on 'arm_low' (use merge=replace to overwrite)
kept existing animation on 'hand' (use merge=replace to overwrite)
physics: nothing baked - 2 bone(s) already have animation and merge=keep (the default) leaves it alone; nothing was written. Use merge=replace to overwrite the existing animation, or --allow-empty to accept an unchanged file     [exit 1]
$ osatura physics demo.json ph.json hand preset=bogus
unknown preset 'bogus' (try hair, tail, rope, antenna, cape)                [exit 1]
```
(`p.json` ist die Demo plus drei ungekeyte Knochen `tail1..tail3`, die mit einem Rhai-Skript erzeugt wurden - siehe `script`.) **Fallstrick:** Ein Backvorgang, der nichts ändert, ist ein Fehler, sofern nicht `--allow-empty` angegeben ist.
**Grenzen (ROADMAP §9):** Collider ignorieren die Knochenskalierung (nur starre Transformation); statische Kreise kollidieren nur an Punkten (angehängte Kreise auch entlang der Verbindungen); keine Kapsel-Kapsel- und keine Selbstkollision.

### `cloth`

**Synopsis:**
```
osatura cloth <project.json> <out.json> <layer> [pin=top,left@bone,row:N,col:N,v:N,corners,none] [gravity=900]
    [wind=x,y] [gust=0.5] [gust-hz=0.5] [gust-len=px] [stiffness=1] [shear=0.05] [bend=0.2] [max-stretch=1.1]
    [damping=0.99] [cols=N rows=N] [rect=x,y,w,h] [name=cloth] [substeps=4] [iterations=10] [preroll=frames]
    [circle=x,y,r[@bone];...] [capsule=x1,y1,x2,y2,r[@bone];...] [from=F] [to=F]
```
Backt eine deterministische **Stoffoberfläche** auf das Mesh einer *Bild*-Ebene oder, wenn `rect=x,y,w,h` angegeben ist, auf eine *Vektor*-Ebene (die Vektorebene erhält über diesem Rechteck ein Gitter-Mesh) als Deformationskanäle pro Vertex. `pin=` wählt die fixierten Vertices (Kanten `top`, `left`..., `corners`, `row:N`, `col:N`, `v:N`, `@bone` bindet die Pins an einen Knochen, `none`). `from`/`to` begrenzen den gebackenen Frame-Bereich, `preroll` lässt die Simulation vor dem Frame `from` laufen, sodass der Stoff in Ruhe startet.

```
$ osatura cloth small.json cloth.json sun rect=100,60,120,120 cols=6 rows=6 pin=top gravity=900 wind=40,0
baked cloth on 'sun' (84 channels); wrote cloth.json
$ osatura cloth small.json cloth.json sun
a vector layer needs rect=x,y,w,h                    [exit 1]
$ osatura cloth small.json cloth.json nosuch
no layer at 'nosuch'                                 [exit 1]
```
Die `84 channels` sind `lattice_dx`-/`lattice_dy`-Keys (Target `sun::cloth::<col>,<row>`) der 42 nicht fixierten Punkte des 7x7-Gitters (die obere Reihe ist fixiert). Es sind echte Kanäle in der Datei, daher macht ein Cloth-Bake das Projekt größer (hier 98 statt 14 Kanäle) - verwende `size-report`, um die Kosten zu sehen. **Grenzen (ROADMAP §9):** Der Stoff ist planar (keine Falten aus der Ebene heraus, daher ist die Scherung standardmäßig nachgiebig); er lehnt freie Meshes ab (nur Gitter-Meshes); keine Selbstkollision; pro ganzem Frame gebacken.

### `cage`

**Synopsis:** `osatura cage <project.json> <out.json> <layer> [rect=x,y,w,h|auto] [hull] [verts=N] [pad=P] [name=cage]`

Fügt einen **Mean-Value-Coordinates-Cage-Deformer** um eine Ebene hinzu: Inhalt innerhalb des Cage folgt den Cage-Vertices, Inhalt außerhalb bleibt unverändert. `auto` (Standard, wenn kein `rect` angegeben ist) verwendet die aufgefüllten Grenzen, `verts=N` ist die Anzahl der Vertices pro Seite, `hull` verwendet die konvexe Hülle, `pad=` füllt die Box auf, `name=` benennt den Deformer (Standard `cage`).

```
$ osatura cage small.json cg.json sun auto verts=3
added cage 'cage' (12 vertices) to 'sun'; wrote cg.json
```
**Grenzen (ROADMAP §9):** keine automatische Cage-Anpassung über eine aufgefüllte Box oder konvexe Hülle hinaus (keine konkave Nachzeichnung); außerhalb des Cage gibt es keinen weichen Abfall (wo die Grafik den Cage-Rand kreuzt, erscheint eine sichtbare Stufe); freie Meshes haben keine gekeyten Offsets pro Vertex (verwende einen Cage).

### `cage-pose`

**Synopsis:** `osatura cage-pose <project.json> <out.json> <layer> <frame> [name=cage] [move=i:dx,dy;...] [all=dx,dy] [ease=linear]`

Keyt Cage-Vertex-Offsets bei einem Frame (Kanäle `cage_dx` / `cage_dy`). `move=` verschiebt einzelne Vertices, `all=` jeden Vertex. Gesperrte Kanäle werden abgelehnt.

```
$ osatura cage-pose cg.json cp.json sun 4 all=5,0
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
$ osatura cage-pose cg.json cp.json sun 4 move=0:5,5
keyed cage 'cage' on 'sun' at frame 4; wrote cp.json
```

### `warp-pivot` und `soft-scale`

**Synopsis:**
```
osatura warp-pivot <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=1,4-7] [region=x,y,w,h]
    [pivot=x,y|bone:NAME|centroid|center] [rotate=DEG] [scale=S|sx,sy] [move=dx,dy] [radius=R] [falloff=smooth] [ease=linear]
osatura soft-scale <project.json> <out.json> <layer> <frame> [deformer=NAME] [verts=..|region=..] scale=S|sx,sy
    [radius=R] [falloff=smooth] [pivot=..]
```
`warp-pivot` rotiert/skaliert/verschiebt eine **Auswahl** von Kontrollpunkten (Cage-, Lattice-, Cloth-, Pin-, Spine-Deformer; ein Gitter-Bild-Mesh erhält ein Warp-Gitter pro Vertex) um einen Pivot und keyt das Ergebnis beim Frame. Die Auswahl besteht aus `verts=` (Indizes und Bereiche) und/oder `region=`; eine leere Auswahl ist ein Fehler (`the selection is empty (give verts= and/or region=)`). `radius=` fügt einen **weichen Abfall** hinzu (`linear smooth gauss sharp constant`). `soft-scale` ist proportionale Skalierung (Soft Selection); ohne `verts`/`region` wird das gesamte Mesh um den Mittelpunkt seiner Bounding Box mit `scale=F`, `width=W`, `height=H` oder `size=WxH` und `aspect=keep|free` in der Größe geändert.

```
$ osatura warp-pivot cg.json wp.json sun 4 verts=0-3 rotate=15 pivot=centroid radius=30 falloff=gauss
warp-pivot on 'sun' keyed at frame 4; wrote wp.json
$ osatura soft-scale cg.json ss.json sun 4 scale=1.2 pivot=center
soft-scale on 'sun' keyed at frame 4; wrote ss.json
```

### `layer-pivot`

**Synopsis:** `osatura layer-pivot <project.json> <out.json> <layer> pivot=x,y|bone:NAME`

Verschiebt den Pivot (Anker) einer Ebene, **ohne das Bild zu verändern**, sodass Rotations- und Skalierungskanäle um den neuen Punkt drehen.

```
$ osatura layer-pivot small.json lp.json sun pivot=100,100
moved the pivot of 'sun'; wrote lp.json
```

### `mesh-lock`

**Synopsis:** `osatura mesh-lock add <project.json> <out.json> <name> host=LAYER[:path] follower=LAYER[:path] [verts=1,4-7] [strength=1]`
und `osatura mesh-lock list|remove <project.json> [<out.json> <name>]`

Lässt die Vertices des Followers in jedem Frame der **Oberfläche des Host-Meshes** folgen (baryzentrische Verschiebungsübertragung), z. B. ein Hemd, das einem Körper folgt. Zyklen werden abgelehnt; Host und Follower müssen dasselbe `parallax` haben (`mesh lock 'lock1': host and follower must have the same parallax`).

```
$ osatura mesh-lock add two.json ml1.json lock1 host=ra follower=rb verts=0-1 strength=0.5
added mesh lock 'lock1'; wrote ml1.json
$ osatura mesh-lock list ml1.json
lock1: rb:0 follows ra:0 vertices [0, 1] strength 0.5
1 mesh lock(s)
$ osatura mesh-lock remove ml1.json ml2.json lock1
removed mesh lock 'lock1'; wrote ml2.json
```

### `glue`

**Synopsis:** `osatura glue add <project.json> <out.json> <name> a=LAYER[:path] b=LAYER[:path] pairs=i:j;i:j|auto=RADIUS [mode=rigid|snap|offset] [strength=1]`
und `osatura glue list|remove <project.json> [<out.json> <name>]`

Klebt Vertices der Ebene `b` an Vertices der Ebene `a` (b folgt a). `pairs=i:j;...` listet Vertex-Paare explizit auf, `auto=RADIUS` paart jeden `b`-Vertex, der innerhalb des Radius eines `a`-Vertex liegt. Modi: `rigid` behält den Ruhe-Offset im Bezugssystem von `a`, `snap` lässt beide zusammenfallen, `offset` behält einen konstanten Offset.

```
$ osatura glue add two.json gl1.json g1 a=ra b=rb auto=5 mode=snap
added glue 'g1' (4 pair(s)); wrote gl1.json
$ osatura glue list gl1.json
g1: Snap strength 1 ra:0#0->rb:0#0 ra:0#1->rb:0#1 ra:0#2->rb:0#2 ra:0#3->rb:0#3
1 glue(s)
$ osatura glue add small.json x.json g1 a=sun b=hills auto=20 mode=snap
no vertex of 'hills' lies within 20 of a vertex of 'sun'          [exit 1]
```

### `mesh-trace`

**Synopsis:** `osatura mesh-trace <project.json> <out.json> <layer> [density=24] [tolerance=1.5] [smooth=N] [hotspot=x,y,r;...] [edge=D/4] [grading=0.5] [lambda=0.5] [taubin=1] [smooth-boundary=0] [threshold=128] [bones=R] [hot-size=E] [skin=1]`

Zeichnet die Alpha-Silhouette einer **Bild-Ebene** in ein freies Dreiecks-Mesh mit variabler Dichte nach (dichter entlang der Kontur, um Hotspots und um Knochen), glättet es und skinnt es an die Knochen. Gibt Vertex-/Dreieckszahlen und die Qualität aus.

```
$ osatura mesh-trace o.json mt.json top density=12 tolerance=2 bones=0
wrote mt.json: 9589 vertices (1494 on the outline), 17682 triangles, area 1227677 px2
  quality min 0.484 mean 0.890, min angle 18.3 deg, mean edge 3.0 px at the outline / 13.7 px inside
```
(`o.json` ist ein OpenRaster-Import mit 1280x960; eine niedrigere `density` ergibt weniger, größere Dreiecke. Der Lauf dauerte in einem Debug-Build 5 s.) **Grenzen:** braucht eine Bild-Ebene (eine Alpha-Silhouette); ein so dichtes Mesh macht die Projektdatei groß - prüfe `size-report`. Freie Meshes lassen sich nicht als Cloth simulieren.

### `skin-diffuse`

**Synopsis:** `osatura skin-diffuse <project.json> <layer> <out.json> [--influences 4]`

Berechnet **Heat-Diffusion-Skin-Gewichte** für eine Mesh-Ebene aus den Knochen des Projekts und behält höchstens `--influences` (Standard 4) Knochen pro Vertex. Beachte die Argumentreihenfolge: **Ebene vor Ausgabe**. Die Ebene muss eine Bild-Ebene mit einem Mesh sein.

```
$ osatura skin-diffuse demo.json arm sd.json
layer 'arm' is not an image layer                          [exit 1]
$ osatura skin-diffuse o.json top sd.json
layer 'top' has no mesh                                    [exit 1]
$ osatura skin-diffuse mt.json top sd.json               # mt.json from mesh-trace
wrote sd.json
```

---

## 8. Audio, Lippensynchronisation und Sounds

Audio-Eingaben für die Analysebefehle (`lipsync`, `lipsync-text`, `speech-motion`, `audio-react`, `beats`, `waveform`) müssen **WAV** sein (8/16/24/32-Bit-Integer- oder 32-Bit-Float-PCM, beliebige Kanalzahl, auf Mono heruntergemischt, bis zu 256 MiB). MP3/OGG müssen zuerst konvertiert werden (`ffmpeg -i in.mp3 in.wav`). In einem Projekt gespeicherte Sound-**Tracks** (`sound`) dürfen jedes Format haben, das ffmpeg lesen kann.

### `lipsync`

**Synopsis:** `osatura lipsync <project.json> <audio.wav> <switch_layer> <out.json> [--map 0,1,2,3,4]`

Steuert eine **Switch-Ebene** (Mundformen) aus Lautstärke und spektraler Helligkeit: eine Heuristik aus Amplitude / Nulldurchgängen, die pro Frame eine von fünf Formen wählt (`rest, small, open, round, wide`) und Flackern entfernt. **Es ist keine Phonemerkennung.** `--map` gibt den Kind-Index des Switches für jede der fünf Formen an; der Standard richtet sich nach der Kinderzahl des Switches: 1 Kind -> alle; 2 -> closed + open; 3 -> closed + open + round/wide; 4 -> closed + small + open + round/wide; 5+ -> in der obigen Reihenfolge. Ersetzt einen vorhandenen gekeyten `layer_switch`-Kanal dieser Ebene. Die Projektlänge wird **nicht** geändert (eine 2-s-WAV in einem Projekt mit 8 Frames keyt 8 Frames).

```
$ osatura lipsync small.json speech.wav mouth lip.json
wrote lip.json (2.00s of audio → 8 frames)
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1,2,1,0
$ osatura lipsync small.json speech.wav mouth lip.json --map 0,1
--map needs exactly 5 numbers (rest,small,open,round,wide)          [exit 1]
$ osatura lipsync small.json speech.wav ink lip.json
layer 'ink' is not a switch layer                                   [exit 1]
$ osatura lipsync small.json notwav.mp3 mouth x.json
invalid WAV: Failed to read enough bytes.                           [exit 1]
```

### `lipsync-text`

**Synopsis:** `osatura lipsync-text <project.json> <audio.wav> <transcript.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--lang ro|en] [--words-out timing.json|timing.srt]`
Gleicht ein **Transkript** mit dem Audio ab (Energie + Textabgleich: Wörter und Silben werden über die im Audio gefundenen Sprachbereiche gelegt) und leitet aus den Silben Mundformen ab. `--lang` (Standard `ro`) wählt die Vokal-/Silbenregeln; es gibt nur `ro` und `en`. `--words-out` schreibt die Wortzeiten als JSON (Wörter mit Silben) oder, bei einem Namen auf `.srt`, als Untertitel.

```
$ osatura lipsync-text small.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (3 words, 2 exposure changes; energy + text alignment, not phoneme recognition)
$ head -c 140 words.json
{ "words": [ { "text": "hello", "start": 0.0, "end": 0.22, "syllables": [ {"text":"hel","start":0.0,"end":0.16}, {"text":"lo",...
$ osatura lipsync-text ... --words-out w.srt ; head -4 w.srt
1
00:00:00,000 --> 00:00:01,220
hello there world
```
**Grenzen (ROADMAP §9):** keine Zeitgebung pro Buchstabe (Phonem-Ebene), keine sprecherspezifischen Ratenmodelle, keine Sprache außer ro/en. Ein synthetischer Ton wird als Sprache abgeglichen - verwende echte Aufnahmen.

### `speech-motion`

**Synopsis:** `osatura speech-motion <project.json> <audio.wav> <transcript.txt> <out.json> [--head bone] [--brow bone_or_layer] [--nod deg] [--fall deg] [--tilt deg] [--brow-px px] [--seed n] [--lang ro|en] [--words-out timing.json|timing.srt]`

Fügt **additive** Kopfnicken, Kopffallen, Kopfneigen und Augenbrauenheben hinzu, zeitlich an den abgeglichenen Wörtern ausgerichtet. `--head` benennt den Kopfknochen, `--brow` einen Knochen oder eine Ebene für die Augenbrauen; die Winkeloptionen sind in Grad, `--brow-px` in Pixeln, `--seed` legt die Variation fest.

```
$ osatura speech-motion small.json speech.wav transcript.txt sm.json --head arm_up --lang en
wrote sm.json (1 additive channel(s) from 3 words)
```

### `mouth`

**Synopsis:** `osatura mouth <project.json> <cues.txt> <switch_layer> <out.json> [--map 0,1,2,3,4] [--rhubarb]`

Importiert fertige Mund-**Cues**, statt Audio zu analysieren: Papagayo-/Moho-Switch-Dateien (`frame PHONEME` pro Zeile, optional erste Zeile `MohoSwitch1`) oder Rhubarb-TSV (`seconds<TAB>shape`, am Dezimalpunkt erkannt; Sekunden werden mit den fps des Projekts umgerechnet). Leerzeilen und `#`-Kommentare werden ignoriert. Phoneme werden auf die fünf Formklassen abgebildet (`rest/MBP/X/A` rest; `etc/FV/L/B/G/H` small; `AI/D` open; `O/U/WQ/F` round; `E/C` wide); unbekannte Formen zählen als `small`. `--rhubarb` behandelt das `E` von Rhubarb als *round* (das `E` von Papagayo ist wide). Der Standard für `--map` ist `0,1,2,3,4`.

```
$ printf '0 rest\n2 open\n4 round\n6 wide\n' > cues.txt
$ osatura mouth small.json cues.txt mouth mo.json --rhubarb
wrote mo.json (4 cues)
```

### `audio-react`

**Synopsis:** `osatura audio-react <project.json> <out.json> <audio.wav> <target> <prop> [gain=1] [offset=0] [floor=0.05] [attack=1] [release=4] [add=on]`

Die Lautstärke steuert eine Eigenschaft: ein Key pro Frame, der dem geglätteten Pegel folgt (RMS relativ zum lautesten Frame). Ausgabe = (durch `floor` gegateter Pegel) x `gain` + `offset`; `attack`/`release` sind die Anstiegs-/Abfallzeiten in Frames; `add=on` macht den Kanal additiv, statt zu ersetzen. Ein gesperrter Kanal wird abgelehnt. Optionen sind `key=value`-Wörter; eine unbekannte ergibt `unknown option 'foo'`.

```
$ osatura audio-react small.json ar.json speech.wav ink layer_rotation gain=10
wrote ar.json
$ osatura audio-react small.json ar.json speech.wav arm_up bone_angle foo=1
unknown option 'foo'                                                [exit 1]
```

### `beats`

**Synopsis:** `osatura beats <project.json> <out.json> <audio.wav> [sensitivity=1.5] [gap=4] [as=markers|events] [name=beat]`

Erkennt Lautstärkesprünge über dem jüngsten Durchschnitt (`sensitivity` 1.2-3, höher = weniger Beats) mit mindestens `gap` Frames Abstand und zeichnet sie als Timeline-**Marker** (Standard) oder als **Events** mit dem Namen `name` auf. Gibt die gefundenen Frames aus.

```
$ osatura beats small.json bt.json speech.wav as=events name=hit sensitivity=1.2
1 beats at frames [1]; wrote bt.json
$ osatura beats small.json bt.json speech.wav
0 beats at frames []; wrote bt.json
```

### `waveform`

**Synopsis:** `osatura waveform <audio.wav> <out.json> [buckets=800]`

Min-/Max-Peaks zum Zeichnen einer Wellenform: `{"sample_rate","duration","peaks":[[min,max],...]}`. Liest kein Projekt.

```
$ osatura waveform speech.wav wf.json buckets=50
wrote wf.json (50 buckets)
```

### `sound`

**Synopsis:**
```
osatura sound list   <project.json>
osatura sound add    <project.json> <out.json> <name> <file.wav|ogg|mp3|...> [start_frame] [gain] [trim/fade options]
osatura sound edit   <project.json> <out.json> <name> [--start F] [--gain G] [trim/fade options]
osatura sound remove <project.json> <out.json> <name>
osatura sound mix    <project.json> <out.wav|out.ogg|out.mp3>
    trim/fade options (seconds, or frames with an f suffix such as 36f): --trim-in T --trim-out T --fade-in T --fade-out T
```
Sound-Tracks sind **Referenzen** (Dateipfad relativ zum Projekt, Start-Frame, Gain 0..4, Trim-Fenster innerhalb der Datei, Fades). Nichts wird im Projekt eingebettet; die Audiodatei muss mit ihm mitwandern und im Projektverzeichnis liegen, sofern nicht `--allow-external-assets` angegeben ist. `edit` ändert nur die angegebenen Felder (`0` löscht einen Trim). `--trim-out 0` bedeutet „bis zum Ende“. `sound mix` rendert nur das gemischte Audio für die **Projektlänge** (über ffmpeg: wav = 44,1 kHz PCM 16 Bit, ogg = Vorbis q4, mp3 = 96 kb/s).
Gemischtes Audio wird auch in `.mp4` (AAC) und `.webm` (Opus) gemuxt und in `.html` eingebettet; `.gif`/`.apng` können es nicht tragen.

```
$ osatura sound add small.json s1.json music speech.wav 2 0.8 --fade-in 0.5 --trim-out 36f
wrote s1.json
$ osatura sound list s1.json
music: speech.wav at frame 2 gain 0.8 trim 0s..1.5s fade-in 0.5s
$ osatura sound edit s1.json s2.json music --gain 0.5 --trim-in 0.1 --fade-out 4f
$ osatura sound list s2.json
music: speech.wav at frame 2 gain 0.5 trim 0.1s..1.5s fade-in 0.5s fade-out 0.16666667s
$ osatura sound mix s2.json mix.mp3
wrote mix.mp3 (0.33 s)
$ osatura sound mix s2.json mix.flac
unknown audio format '.flac' (use .wav, .ogg or .mp3)            [exit 1]
$ osatura sound add small.json x.json x nosuch.wav              # a missing file is refused, nothing written
```
`check` schlägt fehl, wenn eine referenzierte Sounddatei fehlt oder nicht lesbar ist. Ohne ffprobe wird bei Nicht-WAV-Dateien nur die Existenz geprüft (nicht, ob die Datei Audio ist). **Grenzen (ROADMAP §9):** kein Pan/EQ/Ducking pro Track, nur die lineare Fade-Kurve, kein Loopen eines Clips, keine Neu-Zeitgebung.

---

## 9. Import

### `import-svg`

**Synopsis:** `osatura import-svg <file.svg> <out.json> [--split]`

Usage line: `osatura import-svg <file.svg> <out.json> [--split]  SVG subset -> one vector layer, or one layer per shape named by its id (warnings for the rest)`

Erzeugt ein neues Projekt (24 fps, 24 Frames; Größe aus der `viewBox` oder aus `width`/`height`, sonst 512x512) mit **einer Vektor-Ebene `svg`**, die die Formen einer SVG-Teilmenge enthält: `path` (alle Befehle), `rect` (abgerundete Ecken werden ignoriert), `circle`, `ellipse`, `line`, `polyline`, `polygon`, `g`/`a` mit `transform` sowie einfarbige `fill`/`stroke`/`stroke-width`. Alles andere wird mit einer `warning:`-Zeile **auf stdout** übersprungen (wenn das Element außerhalb von `<defs>` steht; ein `<filter>`, `<clipPath>`, `<mask>` oder `<pattern>` innerhalb von `<defs>` sowie die Attribute `filter`, `clip-path` und `mask` werden ohne Warnung ignoriert): `<text>`, `<filter>`, `<clipPath>`, `<mask>`, `<pattern>`, `<linearGradient>`/`<radialGradient>`, `<image>`, `<use>`, `<foreignObject>`. Eine Verlaufsfüllung (`fill="url(#g)"`) fällt auf Grau zurück. Grenzen: 8 MB SVG, 200.000 Anker, eine Obergrenze für Elemente.

```
$ osatura import-svg s.svg svg.json
warning: unsupported paint 'url(#g)'; using grey
warning: <text> is not supported and was skipped
warning: <filter> is not supported and was skipped
warning: <image> is not supported and was skipped
wrote svg.json (3 paths)
$ osatura check svg.json
ok: 's' 200×100 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 0 channels
```
Mit `--split` wird jede Form zu **einer eigenen Ebene**, benannt nach der `id` der Form in Dateireihenfolge. Eine Form ohne id heißt nach ihrem Tag und ihrer Position (`rect1`, `circle6`), eine wiederholte id erhält `_1`, `_2`..., und Zeichen, die ein Ebenenname nicht enthalten darf (`/`, `::`, Steuerzeichen), werden zu `_`; Namen werden auf 100 Zeichen gekürzt. Das ist die Form der Wahl, wenn die Zeichnung Teil für Teil gerigged werden soll; siehe [TUTORIAL_SVG.md](TUTORIAL_SVG.md).

```
$ osatura import-svg pip.svg pip.json --split
wrote pip.json (16 paths, 16 layers)
```
Import und anschließendes Zusammenführen: Es gibt keinen Befehl, der die Ebene in ein bestehendes Projekt einfügt; verwende `script` oder bearbeite das JSON.

### `import-ora`

**Synopsis:** `osatura import-ora <file.ora> <out.json> [--fps N] [--frames N]`

OpenRaster -> Projekt: eine **Bild-Ebene pro Raster-Ebene** (eingebettetes PNG), die Reihenfolge von unten nach oben bleibt erhalten, versteckte Ebenen bleiben erhalten, sind aber unsichtbar, Gruppen werden abgeflacht, dazu pro Ebene `opacity`, `visibility`, Position (`x`,`y`) und die gängigen `svg:`-Kompositionsoperationen (multiply, screen, overlay, darken, lighten, color-dodge/burn, hard/soft-light, difference, exclusion, plus -> add). Das Projekt hat `--frames` Frames (Standard 1) bei `--fps` (Standard 24). Die Deckkraft eines verschachtelten Stapels wird mit der seiner Ebenen multipliziert und sein Mischmodus wird vererbt, aber sich überlagernde Ebenen innerhalb eines durchscheinenden Stapels werden **nicht** vorher abgeflacht (sie scheinen durcheinander hindurch). Doppelte Ebenennamen erhalten die Suffixe `_1`, `_2`.

```
$ osatura import-ora t.ora ora.json
wrote ora.json (2 layers)
$ osatura import-ora t.ora ora2.json --fps 12 --frames 24
$ osatura import-ora layer.png bad.json
not a zip/ORA file: invalid Zip archive: Could not find EOCD       [exit 1]
```
Grenzen: 2000 Archiveinträge, 128 MiB pro Eintrag, Ebenen bis 64 Megapixel.

### `import-lottie`

**Synopsis:** `osatura import-lottie <in.json> <out.json> [fx.json]`

Importiert eine Lottie-Datei (Bodymovin) - eine bewusste **Teilmenge** - als Projekt. Gibt `imported N layers; wrote <out>` und pro übersprungenem Feature eine `warning:`-Zeile aus.

* **Importiert:** Shape-Ebenen mit `sh`-Pfaden, `rc`, `el`, einfarbige Füllungen und Verlaufsfüllungen (animierte Endpunkte/Mittelpunkte/Radien von Verläufen werden zu `grad_dx/grad_dy`-Kanälen), Konturen, Gruppen, Ebenentransformationen (statische oder per Key animierte Position/Skalierung/Rotation/Deckkraft, gekrümmte Positionspfade pro Frame eingebacken), Mischmodi, eine einfarbige Hintergrund-Ebene, Bild-Ebenen mit eingebetteten PNG/JPEG-Data-URIs, Null-Ebenen als Eltern (Transformationen der Eltern werden in die Kinder eingebacken), Ebenenmasken (add/subtract/intersect mit Deckkraft, Invertierung, Key-animierte Formen), Luma-Mattes (näherungsweise), Precompositions, Ein-/Aus-Bereiche von Ebenen (als Deckkraft-Stufenspur), Key-Easing (hold -> step; Bézier-Griffe -> cubic-bezier; keine Griffe -> linear).
* **Animierte Pfade:** Ein Pfad, dessen Form-Keys dieselbe Topologie haben (gleiche Ankeranzahl, offen/geschlossen), wird zu einem Basispfad plus einem **Morph-Target** pro weiterem unterschiedlichem Key (`k1`, `k2`, ...), mit `path_morph`-Gewichtskanälen, die das Easing tragen. Bei abweichender Topologie wird die erste Form verwendet und eine Warnung ausgegeben.
* **Effekte (`ef`):** Gaußscher Weichzeichner, Schlagschatten, Tönung und Füllung werden zu **Effekten pro Ebene** (`effects` an der Ebene, jede Ebenenart) mit animierten Zahlen als `fx_*`-Kanäle; statische Tönung/Füllung/Tritone/Levels auf Shape-Ebenen werden in Farben eingebacken. Effektfarben können nicht animiert werden (der erste Wert wird verwendet); andere Effekte sowie Farbeffekte nach einem Weichzeichner/Schatten auf Nicht-Shape-Ebenen führen zu Warnungen. Pro Ebene werden höchstens die ersten paar Effekte behalten.
* **Mit Warnung übersprungen:** Text und andere Ebenentypen, Audio-Assets, animierte Ankerpunkte (erster Wert), gekrümmte Bewegungspfade bei anderen Eigenschaften als der Position (gerade Segmente), Masken an einer Track-Matte-Quelle, zu lange Eltern-Ketten zum Einbacken, mehr als 2000 Ebenen, Animationen länger als 20.000 Frames.
* `fx.json` wird aus Kompatibilitätsgründen beibehalten: Es wird ein leeres Array geschrieben (Weichzeichner und Schlagschatten sind jetzt Effekte pro Ebene innerhalb des Projekts).

```
$ osatura import-lottie out2.json li.json                   # a Lottie file made by `osatura export`
imported 11 layers; wrote li.json
$ osatura import-lottie in.json out.json                    # hand-made file: animated triangle with animated blur, plus a null and a text layer
warning: layer 'text' of type 5 skipped
imported 1 layers; wrote out.json
$ osatura check out.json
ok: 't' 200×200 @ 24 fps, 24 frames, 0 bones, 0 ik, 1 layers, 2 channels       # channels: blob::path::k1 path_morph, blob::fx0 fx_blur
$ osatura import-lottie out2.json li2.json fx.json
wrote 0 post effect(s) to fx.json
imported 11 layers; wrote li2.json
$ echo '{}' > bad.json; osatura import-lottie bad.json o.json
missing 'w'                                                        [exit 1]
```
**Stolperfalle:** `import-lottie in.json in.json` wird abgelehnt (`refusing to overwrite the input ...`), ebenso für `fx.json`. **Grenzen:** Eine `.lottie`-Datei (zip) als Eingabe wird nicht gelesen (`stream did not contain valid UTF-8`) - entpacke zuerst `a/animation.json`; der quadratische Aufwand bei Form-Morphs ist nur durch die Größenobergrenzen beschränkt.

### `import-bvh`

**Synopsis:** `osatura import-bvh <project.json> <file.bvh> <out.json> --joint-map Joint=bone,.. [--axis Zrotation --tol 0.5]`

Bildet ausgewählte BVH-Gelenk-Rotationskanäle auf Rig-Knochen ab, als Key-animierte `bone_angle`-Kanäle, abgetastet mit der Projekt-fps und der Projekt-Framezahl. `--joint-map` ist erforderlich; `--axis` (Standard `Zrotation`) wählt, welcher BVH-Rotationskanal (`Zrotation`, `Xrotation`, `Yrotation`) gelesen wird; `--tol` (Standard 0,5 Grad) vereinfacht Keys. Für automatisches Zuordnen, Root-Bewegung, Ansichten und Skalierung verwende `mocap-retarget` (Abschnitt 10).

```
$ osatura import-bvh demo.json m.bvh ib.json --joint-map LeftUpLeg=thigh,LeftLeg=shin --axis Zrotation --tol 0.5
wrote ib.json (+2 channels from 4 BVH frames)
```

### `import-seq`

**Synopsis:** `osatura import-seq <project.json> <out.json> <layer_name> <folder> [hold=1] [loop=on|off] [start=0] [x=0] [y=0]`

Importiert eine **PNG-Sequenz** (alle PNGs des Ordners in der Reihenfolge der Dateinamen) als Frame-für-Frame-Ebene: eine Switch-Ebene namens `<layer_name>` mit einer Bild-Ebene pro Frame und einem Step-Kanal, der jede davon ab Frame `start` für `hold` Frames einblendet, an der Position `(x, y)`; `loop=on` wiederholt sie. Alle Bilder müssen dieselbe Größe haben (`img.png: size 640x480 differs from the first image (160x120)`).

```
$ osatura import-seq demo.json is.json frames seq hold=2 loop=on start=4 x=10 y=20
imported 3 images; wrote is.json
```

### `import-anim`

**Synopsis:** `osatura import-anim <project.json> <out.json> <layer_name> <file.gif|file.apng|file.png> [loop=on|off] [start=0] [x=0] [y=0]`

Importiert ein **animiertes GIF oder APNG** als Frame-für-Frame-Ebene mit den eigenen Frame-Verzögerungen der Datei (in Projekt-Frames umgerechnet). Ein einfaches PNG ergibt einen Frame.

```
$ osatura import-anim demo.json ia.json gifl dof.gif loop=off start=0
imported 4 frames as layer 'gifl'; wrote ia.json
$ osatura import-anim demo.json ia2.json apngl dof.apng
imported 3 frames as layer 'apngl'; wrote ia2.json
```
Die Frame-Bilder werden zu eingebetteten PNGs: Ein langes GIF macht das Projekt groß.

### `trace`

**Synopsis:** `osatura trace <image.png> <out.json> [--threshold 128 --tolerance 1.0 --min-area 12 --fill #000000]`

Einfache Bitmap -> Vektor-Umwandlung für Bilder mit **Transparenz**: Der Befehl vektorisiert den **Alphakanal** des PNG (Marching Squares bei `--threshold`, 0..255, mit linearer Kanteninterpolation), vereinfacht die Konturen (Ramer-Douglas-Peucker) auf `--tolerance` Pixel, verwirft Bereiche unter `--min-area`, erkennt Löcher durch Enthaltensein und schreibt ein neues Projekt (Name `traced`, Canvas = Bildgröße) mit den Konturen als gefüllten Pfaden in der Farbe `--fill`. Ein vollständig deckendes Bild ergibt daher eine Kontur um das gesamte Bild (wie im Beispiel). Gibt `wrote <out> (N contours)` aus. Verwende `autotrace` für Farbbilder.

```
$ osatura trace img.png tr.json --threshold 128 --tolerance 1.0 --min-area 12 --fill '#112233'
wrote tr.json (1 contours)
```

### `autotrace`

**Synopsis:** `osatura autotrace <image.png|jpg> <out.json|out.svg> [--mode fill|line] [--colors N=8] [--min-area A=8] [--tol T=1] [--smooth N=3] [--corner DEG=55] [--denoise N=0] [--quant kmeans|mediancut] [--seed S] [--fit bezier|polygon|vw] [--zorder area|luminance] [--threshold T] [--invert] [--prune N=4] [--into project.json] [--name L] [--fit-stage] [--png preview.png]`

Lokale Bitmap-Vektorisierung - **kein Modell, kein Netzwerk**: Farbquantisierung (`--colors`, k-means oder Median Cut), Konturextraktion, kubische Bézier-Anpassung nach Schneider (`--fit`; `polygon` und Visvalingam-Whyatt `vw` sind Alternativen), Löcher nach der Even-Odd-Regel; `--mode line` dünnt Striche zu Mittellinien aus (Zhang-Suen, `--prune` entfernt kurze Ausläufer). Die Ausgabe ist ein neues Projekt, eine neue `.svg` oder - mit `--into project.json` - eine Kopie eines bestehenden Projekts mit einer neuen Ebenengruppe `--name` (Standard `trace`); `--fit-stage` (mit `--into`) skaliert die neue Gruppe gleichmäßig, damit sie auf die Bühne des Projekts passt, und zentriert sie dort, `--png` schreibt ein Vorschau-Rendering. Der Befehl gibt Zählwerte und den **gemessenen Fehler gegenüber der Quelle** aus (mittlerer absoluter Fehler, RMSE, PSNR, SSIM).

```
$ osatura autotrace img.png at.json --colors 4 --png prev.png
layer group 'trace' added
wrote at.json: 44 regions, 0 strokes, 44 paths, 324 anchors
fidelity vs source: MAE 13.38/255, RMSE 17.59, PSNR 23.2 dB, SSIM 0.960
$ osatura autotrace img.png at.svg --mode line --threshold 100
wrote at.svg: 0 regions, 11 strokes, 11 paths, 47 anchors
fidelity vs source: MAE 52.47/255, RMSE 58.73, PSNR 12.8 dB, SSIM 0.903
$ osatura autotrace img.png at2.json --into d.json --name traced
layer group 'traced' added
wrote at2.json: 51 regions, 0 strokes, 51 paths, 347 anchors
```
**Grenzen (ROADMAP §9):** ein klassischer Vektorisierer - Fotos werden zu Farbbändern; keine gelernte oder semantische Vektorisierung. `--png` ist hier eine Option mit Wert (Dateiname der Vorschau), anders als bei anderen `--png`-Flags.

### `autotrace-bench`

**Synopsis:** `osatura autotrace-bench`

Vektorisiert eingebaute synthetische Formen, Verläufe und Striche und gibt eine Genauigkeitstabelle aus (MAE, RMSE, PSNR, SSIM, Anzahl der Bereiche/Anker, Millisekunden). Nimmt keine Argumente entgegen; nur lesend.

```
$ osatura autotrace-bench
image          mode  colors  tol     MAE    RMSE   PSNR   SSIM regions strokes  anchors      ms
shapes         fill       6    1    2.19   12.10   26.5  0.975      11       0       33    13.6
...
```

---

## 10. Live-Eingabe, MIDI, Gesichts-Tracking und Motion Capture

### `live-map`

**Synopsis:** `osatura live-map <project.json> <events.json|song.mid> <mapping.json> <out.json> [--takes lib.json] [--tol 0.01] [--allow-empty]`

Wandelt eine **aufgezeichnete Live-Performance** (MIDI, Maus, Gamepad, Tasten, Gesichtssignale) in gewöhnliche Key-Kanäle, Events und Takes um (nicht destruktiv, ganz oder gar nicht, Sperren werden respektiert). Die Eingabe ist entweder eine Standard-MIDI-Datei `.mid` oder ein Events-JSON `{"events":[{"source","id","value","time"}]}` (`value` normalisiert auf 0..1, `time` in Sekunden; `source` = `midi_cc midi_note midi_pitch_bend mouse_x mouse_y mouse_button gamepad_axis gamepad_button key face`; `id` = bei MIDI `"<channel 1-16>:<number>"`, bei der Maustaste `"0"`, beim Gamepad `"<pad>:<index>"`, bei Tasten ein DOM-Code wie `KeyA`).

Die Mapping-Datei hat `params` (source/id -> `target`+`prop`, mit `in_min in_max out_min out_max deadzone invert curve smooth additive`) und `triggers` (Überschreiten eines Schwellenwerts -> Aktion `jump_to_frame`, `play_take`, `set_key`, ...); unbekannte Schlüssel sind Fehler. `--takes` ist eine Take-Bibliothek für `play_take`-Aktionen (muss existieren), `--tol` vereinfacht Keys.

```
$ cat map.json
{"params":[{"source":"mouse_x","id":"0","target":"arm_up","prop":"bone_angle","out_min":-30,"out_max":30}],
 "triggers":[{"source":"key","id":"KeyA","action":{"type":"jump_to_frame","frame":3}}]}
$ osatura live-map small.json ev.json map.json out.json
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote out.json
$ osatura live-map small.json song.mid mapm.json outm.json              # MIDI CC 1 on channel 1 -> arm_up
3 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 0 events (0 after the last frame skipped); wrote outm.json
$ osatura live-map small.json ev.json map.json out.json --takes lib.json    # lib.json missing
cannot read lib.json: No such file or directory (os error 2)             [exit 1]
$ osatura live-map small.json empty.json map.json out.json
note: map 0 (arm_up.BoneAngle): no matching events, nothing keyed
0 events -> +0 channels, ...; wrote out.json
```
**Stolperfalle:** Wenn Events gelesen wurden, aber **nichts erzeugt wurde** (Tippfehler in einer id, falsche source), schlägt der Befehl fehl und schreibt nichts, es sei denn, `--allow-empty` ist gesetzt; bei null Eingabe-Events schreibt er einfach das unveränderte Projekt. Sprung-Trigger werden als Event namens `jump` mit `data.to` aufgezeichnet (die Zeitachse wird nicht umgeschrieben).
**Grenzen (ROADMAP §9):** keine Webcam-/Audiopegel-Live-Eingabe im Studio über diesen Befehl (`audio-react` arbeitet offline); die interaktive Seite stellt das Studio bereit.

### `drive`

**Synopsis:** `osatura drive <project.json> <signals.csv> <maps.json> <out.json> [--tol 0.5]`

Performance -> Keys aus einer **CSV**: Die Kopfzeile benennt die Signale, jede weitere Zeile ist ein Frame (Zeilenindex = Frame). `maps.json` ist ein Array aus `{input, target, prop, in_min, in_max, out_min, out_max, deadzone, invert, curve, smooth}`; `--tol` (Standard 0,5) vereinfacht die Keys.

```
$ printf 'jaw,speed\n0,0\n0.5,0.2\n1,0.4\n0.5,0.6\n' > sig.csv
$ echo '[{"input":"jaw","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":40,"smooth":0.2}]' > maps.json
$ osatura drive demo.json sig.csv maps.json dr.json --tol 0.5
wrote dr.json (+1 channels)
```

### `face-synth`, `face-train`, `face-eval`, `face-detect`, `face-track`, `face-calibrate`, `face-bake`

Eine gesichtsgesteuerte Pipeline, die **vollständig lokal läuft und ausschließlich auf synthetischen Gesichtern trainiert wurde** (stilisierte/Cartoon-Köpfe, 23 Landmarken, Layout `osatura23`). Ein kleines, synthetisch trainiertes Modell ist in die Binärdatei eingebaut; `--model` überschreibt es. **Fotografien, Webcams und echtes Video sind nicht validiert** (ROADMAP §9; siehe auch [PATENT_NOTES.md](PATENT_NOTES.md): Gesichtsgesteuertes Puppenspiel wurde nicht auf Patente geprüft).

| Befehl | Synopsis | Was er tut |
|---|---|---|
| `face-synth` | `osatura face-synth <out-dir> [--frames 90] [--size 160] [--seed 7] [--fps 30]` | Synthetische Kopfanimation: PNG-Frames `000000.png...`, `truth.ndjson` (wahre Landmarken) und `params.json` (wahre Signale); `--fps` ist die Frame-Rate, mit der die Signale getaktet werden (im NDJSON-Header vermerkt). |
| `face-train` | `osatura face-train --samples N --seed S --out model.bin [--mine-rounds 1] [--scene N] [--reg-win N] [--reg-cell N] [--det-win N] [--det-cell N] [--ridge-lambda X] [--stages N] [--refine-win N] [--stage-patch N] [--stage-cell N] [--stage-lambda X]` | Trainiert den eigenen Detektor ausschließlich auf synthetischen Gesichtern (keine Downloads). `--scene` ist die Größe der Trainingsbilder (64..512); `--reg-win`, `--reg-cell`, `--det-win`, `--det-cell`, `--ridge-lambda`, `--stages`, `--refine-win`, `--stage-patch`, `--stage-cell` und `--stage-lambda` sind die Stellschrauben für Modellgröße und Regression (die Standardwerte sind die, mit denen das eingebaute Modell trainiert wurde). |
| `face-eval` | `osatura face-eval [--model m.bin] [--eval-n 200] [--seed 9] [--scene 128] [--report r.json] [--json]` | Genauigkeit auf zurückgehaltenen synthetischen Daten, False Positives und Robustheits-Sweeps (JSON; `--report` schreibt es, `--json` gibt es auch zusammen mit `--report` aus; `--scene` ist die Größe der Testbilder). |
| `face-detect` | `osatura face-detect <image> [--model m.bin] [--json] [--overlay o.png] [--threshold T] [--max-faces N] [--min-side N] [--max-side N] [--max-dim N]` | Gesichtsrahmen und 23 Landmarken; `--overlay` zeichnet sie auf ein PNG; `--min-side` / `--max-side` begrenzen die Gesichtsgröße in Pixeln, und `--max-dim` skaliert ein großes Bild vorher herunter. |
| `face-track` | `osatura face-track <video-or-frame-dir> <out.ndjson> [--model m.bin] [--fps 30] [--max-dim 256]` | Landmarken-NDJSON für `face-bake`. Ein Verzeichnis mit Bildern wird direkt gelesen; ein Video läuft über ffmpeg. |
| `face-calibrate` | `osatura face-calibrate <landmarks.ndjson> <out.json> [--calibrate-frames N] [--map mapfile.json] [--fps F] [--layout L]` | Berechnet die Kalibrierung des neutralen Gesichts aus den ersten N Frames. |
| `face-bake` | `osatura face-bake <project.json> <landmarks.ndjson> <mapfile.json> <out.json> [--calibrate-frames N\|0] [--calib calib.json] [--fps F] [--layout L] [--tol T] [--allow-empty]` | Backt Gesichtssignale (`mouth_open`, `eye_l_open`, `eye_r_open`, `brow_raise`, `yaw`, `pitch`, `roll`, ...) in Kanäle, mit einer Mapping-Datei im Stil von `live-map`, deren `source` `face` ist. |

```
$ osatura face-synth syn --frames 6 --size 96 --seed 3
wrote 6 frames (96x96), truth.ndjson and params.json to syn
$ osatura face-train --samples 80 --seed 1 --out m.bin --scene 64        # 47 s in a debug build
wrote m.bin (649900 bytes) in 47.0s: 80 samples, seed 1, mined negatives 9
trained on synthetic faces only: expect stylised/cartoon faces to work, photographs and webcams are NOT validated
$ osatura face-train --samples 60 --seed 1 --out m.bin --scene 2
scene size must be 64..512                                                [exit 1]
$ osatura face-detect syn/000000.png --model m.bin
syn/000000.png: face 0: box centre (48.1, 47.4) side 69.0 score 4.49
$ osatura face-track syn trk.ndjson --model m.bin --fps 15 --max-dim 96
wrote trk.ndjson: 6 frames, face found in 6
$ osatura face-calibrate syn/truth.ndjson calib.json --calibrate-frames 3 --fps 15
6 frames, neutral from the first 3; wrote calib.json
$ echo '{"params":[{"source":"face","id":"mouth_open","target":"arm_low","prop":"bone_angle","out_min":0,"out_max":60}],"triggers":[]}' > map.json
$ osatura face-bake small.json trk.ndjson map.json baked.json --calibrate-frames 3 --fps 15 --tol 0.5
6 landmark frames -> +1 channels (K keys in them), 0 trigger keys set, 0 events; wrote baked.json
$ osatura face-eval --eval-n 6 --model m.bin --report ev.json            # writes the JSON report
```
Diese Befehle schreiben ihre Modell-/Berichtsdateien mit dem atomaren Writer, der Symlinks ablehnt (übergeordnete Ordner werden angelegt). `face-bake` und `live-map` melden `+N channels (K keys in them), M trigger keys set` und warnen auf stderr, wenn die Aufnahme länger als das Projekt ist (die überzähligen Frames werden nicht eingebacken; die Projektlänge bleibt unverändert).

### `mocap-info`, `mocap-match`, `mocap-retarget`

```
osatura mocap-info     <file.bvh>
osatura mocap-match    <project.json> <file.bvh> [--map map.json]
osatura mocap-retarget <project.json> <file.bvh> <out.json> [--view front|side|top|yaw=DEG] [--map map.json] [--scale auto|N] [--fps F]
    [--smooth N] [--start F] [--end F] [--rest-frame F] [--min-conf C] [--weight W] [--mirror] [--no-root] [--in-place] [--keep-length] [--footlock BONE --ground Y]
```
* **`mocap-info`** gibt Hierarchie, Kanäle, Bildrate und Ausdehnung einer BVH-Datei aus.
  ```
  $ osatura mocap-info m.bvh
  4 joints (2 end sites), 15 channels per frame
  4 frames at 0.0417 s (24.000 fps), duration 0.125 s
  Hips  offset (0.000, 0.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
    LeftUpLeg  offset (5.000, 0.000, 0.000)  [Zrot Xrot Yrot]
  ...
  ```
* **`mocap-match`** gibt die **automatische Zuordnungstabelle Gelenk -> Knochen** aus (Slot, BVH-Gelenk, Rig-Knochen, Konfidenz, Evidenz `via`, Längenverhältnis, Hinweis), danach die nicht zugeordneten Gelenke/Knochen und Warnungen. Dabei werden sowohl Namen als auch Topologie genutzt. `--map` ist ein JSON-Objekt `{"bvh joint": "rig bone" | null}` (`null` oder `""` schließt ein Gelenk aus; die Umhüllung `{"map": {...}}` wird akzeptiert). Der Exit-Code ist immer 0 - wenn nichts zugeordnet wurde, warnt er nur mit `no bones matched: supply a --map file`.
  ```
  $ osatura mocap-match demo.json m.bvh --map map2.json         # {"Hips":null,"LeftUpLeg":"thigh","LeftLeg":"shin"}
  slot         bvh joint                  rig bone                conf  via         len x  note
  shin.l       LeftLeg                    shin                    1.00  manual      12.00  manual
  thigh.l      LeftUpLeg                  thigh                   1.00  manual      12.00  manual
  unmatched BVH joints: Hips, RightUpLeg
  ...
  ```
* **`mocap-retarget`** backt die Bewegung als **angehängte** Kanäle: Es projiziert die 3D-Rotationen auf die 2D-Ansicht `--view` (`front` Standard, `side`, `top` oder `yaw=DEG`), skaliert die Root-Bewegung (`--scale auto` oder eine Zahl), tastet auf `--fps` neu ab (Standard: die des Projekts), glättet (`--smooth` Frames), nimmt die BVH-Frames `--start..--end`, verwendet `--rest-frame` als Ruhepose, verwirft Zuordnungen unter `--min-conf`, überblendet mit `--weight`, spiegelt mit `--mirror`, verwirft die Root-Bewegung mit `--no-root`, wandelt einen Clip mit Fortbewegung mit `--in-place` in einen am Ort verbleibenden um und fixiert mit `--footlock BONE --ground Y` einen Fuß. Gibt die Tabelle, Hinweise und `N bones, F frames at R fps, root scale S, +C channels; wrote <out>` aus. Ein Clip, der länger als das Projekt ist, **verlängert das Projekt**; ein Hinweis nach der Zusammenfassung nennt, von welcher auf welche Länge. `--keep-length` behält stattdessen die Projektlänge bei und kappt die Keys am letzten Frame (dort wird ein interpolierter Key gesetzt; ein Hinweis zählt die nicht eingebackenen Keys). Liegt nichts über der Konfidenzschwelle: `no bone matches to retarget (all below the confidence threshold?)` (Exit 1). Hinweis: Bei diesem Befehl ist `--scale` `auto|N`, nicht die übliche Render-Skalierung.
  ```
  $ osatura mocap-retarget demo.json m.bvh mr.json --map map2.json --view side --smooth 1 --in-place --no-root
  ... (table)
  2 bones, 4 frames at 24 fps, root scale 12.0000, +2 channels; wrote mr.json
  ```
**Grenzen (ROADMAP §9):** Es wurde keine echte Motion-Capture-Datei getestet (keine ist mitgeliefert: Die Beispiele verwenden ein handgeschriebenes BVH mit 4 Frames); die 2D-Projektion verwirft die Tiefe (ein Knochen, der auf die Kamera zeigt, behält seinen letzten Winkel).

---

## 11. Zustandsautomaten, Datenbindungen, Container und Scroll-Bereiche

Diese Befehle verwalten die Teile eines Projekts, die die **Laufzeit-Interaktion** betreffen: Zustandsautomaten (`machines`), Datenbindungen
(`bindings`), Layout-Container (`containers`) und Scroll-Bereiche (`scrolls`). Alle speichern ihre Daten in der
Projektdatei; die `add`-Formen lesen eine kleine JSON-Zusatzdatei, validieren sie und schreiben ein neues Projekt.
Das Laufzeitmodell selbst ist in [PROJECT_FORMAT.md](PROJECT_FORMAT.md) dokumentiert.

### `machine` (add, list, remove, validate)

**Synopsis:**
```
osatura machine add      <project.json> <out.json> <machine.json>
osatura machine list     <project.json>
osatura machine remove   <project.json> <out.json> <name>
osatura machine validate <project.json>
```

Ein **Automat** hat benannte `inputs` (`bool`, `number` oder `trigger`) und eine oder mehrere `layers`; jede Ebene hat einen `entry`-Zustand,
`states` und `transitions`. Ein Zustand spielt eine **Quelle** ab: `{"kind":"segment","start":F,"end":F}` (Frames der
Key-Kanäle des Projekts), `{"kind":"clip","clip":NAME}`, `{"kind":"region","marker":NAME}` oder einen 1D-Blend-Tree
`{"kind":"blend","input":NAME,"children":[{"threshold":T,"source":{...}}]}`. Zustandsfelder: `name`, `source`,
`mode` (`loop` Standard, `once`, `ping_pong`), `speed` (1), `blend_in` (Sekunden). Übergangsfelder: `from` (ein Zustand oder
`"*"`), `to`, `when` (eine Liste aus `{"input":NAME,"op":eq|ne|lt|le|gt|ge,"value":V}`, alle müssen zutreffen; `{"input":"go"}`
allein bedeutet „gleich 1“), `exit_time` (0..1), `duration` (Überblendung in Sekunden) und `ease`. Unbekannte Schlüssel in einem Übergang
sind ein **Fehler** (ein Tippfehler wie `conditions` würde sonst einen bedingten Übergang in einen unbedingten verwandeln).

```
$ cat mach.json
{"name":"m","inputs":[{"name":"go","kind":"bool"}],"layers":[{"name":"base","entry":"idle","states":[
 {"name":"idle","source":{"kind":"segment","start":0,"end":12}},
 {"name":"act","source":{"kind":"segment","start":12,"end":36}}],
 "transitions":[{"from":"idle","to":"act","when":[{"input":"go"}],"duration":0.1}]}]}
$ osatura machine add demo.json d1.json mach.json
added machine 'm' (1 machines); wrote d1.json
$ osatura machine list d1.json
m  inputs [go:bool]
  layer base (entry idle, mask []): 2 states, 1 transitions
$ osatura machine validate d1.json
1 machines, 0 bindings: ok
```

`machine remove` weigert sich, einen Automaten zu entfernen, den noch eine Bindung steuert
(`binding 'goBind' still drives machine 'm'`, Exit 1): Entferne zuerst die Bindung.
**Fehler:** Ein ungültiger Automat (unbekannter Zustand, doppelte Namen, fehlerhafter Input-Verweis) wird mit einer Meldung abgelehnt, und
es wird nichts geschrieben. **Siehe auch:** `machine-sim`, `bind`, `state-bake` (das ältere Frame-basierte Automatenformat).
**Grenzen:** Automaten werden von der Laufzeitumgebung und von `machine-sim` ausgewertet; die Exporter backen nichts davon ein (Lottie, GIF
und die anderen Dateien enthalten nur Frames, siehe [EXPORT_FORMATS.md](EXPORT_FORMATS.md)).

### `machine-sim`

**Synopsis:** `osatura machine-sim <project.json> <script.json> <out_dir|out.json> [--fps N --svg-frames N]`

Spielt zeitlich festgelegte Eingabe-Events gegen einen Automaten ab und protokolliert den Zustand bei jedem Tick. Das Skript ist
`{"machine": NAME?, "duration": SECONDS?, "events": [{"t": SECONDS, "set": {"input": value}, "trigger": ["name"],
"value": {"external.path": number}}]}`. `machine` ist standardmäßig der einzige Automat des Projekts, `duration` das letzte
Event plus eine Sekunde; `value` speist Datenbindungen (siehe `bind`). Die Standard-`--fps` ist die fps des Projekts.
`--svg-frames N` rendert zusätzlich N SVG-Frames, gleichmäßig über den Lauf verteilt.

```
$ cat sc.json
{"duration":1.5,"events":[{"t":0.5,"value":{"hud.go":1}}]}
$ osatura machine-sim d3.json sc.json sim.json --fps 24 --svg-frames 0
   0.000s  base     Entered idle
   0.500s  base     Exited idle
   0.500s  base     Entered act
37 ticks at 24 fps, 3 events, 0 svg frames; wrote sim.json
$ osatura machine-sim d3.json sc.json simdir --svg-frames 2     # a name that does not end in .json is a folder
...
37 ticks at 24 fps, 3 events, 2 svg frames; wrote simdir/sim.json
$ ls simdir
frame_00000.svg  frame_00036.svg  sim.json
```
`sim.json` enthält `machine`, `fps`, `ticks[]` (`tick`, `time`, `states` als `[layer,state]`-Paare, `fading`) und das
Event-Protokoll. **Siehe auch:** `bind`, `machine`.

### `state-bake`

**Synopsis:** `osatura state-bake <project.json> <machine.json> <events.json> <out.json> [--tol 0.01]`

Backt einen **Frame-basierten** Zustandsautomaten (ein anderes, älteres Format als `machine`) in gewöhnliche Key-Kanäle.
`machine.json` ist `{"initial":NAME,"states":[{"name","length","looped","channels":[...]}],"transitions":[{"from":NAME
or "*","to","when":COND,"fade":FRAMES,"ease"}]}`, wobei `COND` eines von `{"type":"event","name"}`,
`{"type":"above"|"below"|"equals","input","value"}`, `{"type":"finished"}`, `{"type":"after","frames"}` ist.
`events.json` ist eine Liste aus `{"frame":N,"name":NAME,"value":V}`. Das Ergebnis wird an die Kanäle des Projekts angehängt
(`--tol` ist die Toleranz für die Key-Reduktion), das Projekt wird kompiliert und geschrieben.

```
$ echo '[{"frame":10,"name":"go"}]' > ev.json
$ osatura state-bake demo.json sm.json ev.json baked.json --tol 0.01
wrote baked.json (+1 channels)
```
**Stolperfalle:** Der eingebackene Kanal wird hinter den vorhandenen angehängt; Kanäle laufen in Listenreihenfolge, entferne oder stumme also den
ursprünglichen Kanal für dasselbe Ziel, wenn das Backen gewinnen soll. Eine Automatendatei im `machine`-Format
(mit `layers`) wird abgelehnt: `invalid JSON in mach.json: missing field 'initial'`.

### `bind` (add, list, remove) und `bind-eval`

**Synopsis:**
```
osatura bind add    <project.json> <out.json> <name> <value.path> <target:prop|@machine:input>
                     [--expr E --remap a,b,c,d --no-clamp --smooth S --mode replace|add|multiply --weight W]
osatura bind list   <project.json>
osatura bind remove <project.json> <out.json> <name>
osatura bind-eval   <project.json> <values.json> [--data out.json]
```

Eine **Datenbindung** bildet einen externen Wertepfad (zum Beispiel `hud.size`) auf eine Projekteigenschaft (`sun:layer_opacity`,
die Eigenschaftsnamen sind die in snake_case aus Abschnitt 1) oder auf einen Automaten-Input (`@m:go`) ab. `--remap a,b,c,d` bildet den
Eingabebereich `a..b` auf `c..d` ab (begrenzt, außer mit `--no-clamp`); `--expr` ist ein Ausdruck über den Wert;
`--smooth S` glättet per Tiefpass über S Sekunden; `--mode` und `--weight` steuern, wie sich das Ergebnis mit den Kanälen verbindet.

```
$ osatura bind add d1.json d2.json goBind hud.go @m:go
added binding 'goBind' (1 bindings); wrote d2.json
$ osatura bind add d2.json d3.json sz hud.size sun:layer_opacity --remap 0,100,0.2,1 --smooth 0.1
added binding 'sz' (2 bindings); wrote d3.json
$ osatura bind list d3.json
goBind  hud.go -> @m:go
sz  hud.size -> sun:LayerOpacity remap 0..100 -> 0.2..1 clamp smooth 0.1s
$ echo '{"hud.go":1,"hud.size":50}' > vals.json
$ osatura bind-eval d3.json vals.json --data bv.json
goBind = 1
sz = 0.6000000238418579
$ osatura bind add d2.json x.json sz hud.size sun:scale
unknown property 'scale'                                                  [exit 1]
$ osatura bind remove d3.json d7.json sz
removed binding 'sz'
```
`bind-eval` gibt die Ausgabe jeder Bindung für die übergebenen Werte aus (`--data` schreibt sie zusätzlich als JSON). `list` gibt die Eigenschafts-
namen in CamelCase aus; die Kommandozeile verlangt immer snake_case. **Siehe auch:** `machine`, `machine-sim` (dessen `value`-Map).

### `container` (add, list, remove, solve)

**Synopsis:**
```
osatura container add    <project.json> <out.json> <container.json>
osatura container list   <project.json>
osatura container remove <project.json> <out.json> <id>
osatura container solve  <project.json> [--size WxH --data out.json]
```

Ein **Container** ist eine Flexbox-ähnliche Layout-Box: `id`, `members`, `axis` (`row`|`column`), `gap`, `cross_gap`, `padding`
(`[left,top,right,bottom]`), `justify` (`start|center|end|space_between|space_around`), `align`
(`start|center|end|stretch`), `wrap` (`none|wrap`) und `rect`, eine Regel für sein eigenes Rechteck auf dem Canvas:
`{"kind":"fixed",...}`, `{"kind":"pin","left","right","top","bottom","width","height"}` (Abstände von den Canvas-Rändern; je Achse beliebige zwei
davon), `{"kind":"aspect",...}` oder `{"kind":"hug"}`. Ein **Mitglied** ist genau eines von `{"layer":PATH}`,
`{"container":ID}` oder `{"spacer":true}`, mit `grow`, `shrink`, `width`, `height`, `min_w/max_w/min_h/max_h` und
einer `align`-Überschreibung; `"resize":true` erlaubt dem Solver, die Ebene auf ihren Platz zu skalieren.

```
$ cat cont.json
{"id":"row1","axis":"row","gap":10,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"sun"},{"layer":"blush","grow":1,"resize":true}]}
$ osatura container add d3.json d4.json cont.json
added container 'row1' (1 containers); wrote d4.json
$ osatura container list d4.json
row1: Row, 2 members, gap 10, justify Start, align Start, wrap None, rect {"kind":"pin","left":10.0,...}
$ osatura container solve d4.json --size 800x480 --data solved.json
canvas 800x480
container row1: x 10.00 y 10.00 w 780.00 h 100.00
layer sun: dx -430.000 dy 0.000 scale 1.0000 x 1.0000
layer blush: dx 280.432 dy -158.242 scale 33.8889 x 1.0000
```
Ohne `--size` wird die eigene Größe des Projekts verwendet. `solve` gibt das Rechteck jedes Containers und pro Ebene den Versatz und die
Skalierung aus, die das Layout zusätzlich zur Animation anwendet. **Fehler:** Eine unbekannte Ebene, ein fehlender verschachtelter Container oder ein
Zyklus wird von `add` abgelehnt (es wird nichts geschrieben). **Siehe auch:** `scroll`, `layout-preview`.
**Grenzen:** Das Layout wird von der Laufzeitumgebung bzw. der Vorschau angewendet; exportierte Dateien enthalten das für die eigene Projektgröße berechnete Layout.

### `scroll` (add, list, remove, eval)

**Synopsis:**
```
osatura scroll add    <project.json> <out.json> <scroll.json>
osatura scroll list   <project.json>
osatura scroll remove <project.json> <out.json> <id>
osatura scroll eval   <project.json> <id> [--offset X,Y --size WxH --velocity VX,VY --time S]
```

Ein **Scroll-Bereich** hat `id`, `content` (die id eines Containers, der kein Mitglied eines anderen Containers ist), `viewport`
(eine Rechteckregel wie oben), `axis` (`x`, `y` oder beide), `offset`, `elastic` (`{"limit","stiffness","damping_ratio"}`; Gummiband
statt harter Begrenzung), `snap_x`/`snap_y` (Ruhe-Offsets), `fling` (Sekunden der Loslass-Geschwindigkeit, die projiziert werden, um
den Landepunkt zu finden, Standard 0,15), `clip_layer` (eine Rechteck-Ebene, die als Maske für den Inhalt dient) und `input` (ein externer Wertepfad, der
den Offset steuert).

```
$ cat scroll.json
{"id":"sc1","content":"row1","axis":"x","viewport":{"kind":"pin","left":0,"right":0,"top":0,"height":120},
 "elastic":{"limit":60,"stiffness":200,"damping_ratio":0.8},"snap_x":[0,100,200]}
$ osatura scroll add d4.json d5.json scroll.json
added scroll 'sc1' (1 scrolls); wrote d5.json
$ osatura scroll list d5.json
sc1: content row1, axis X, offset [0.0, 0.0], elastic true, snap x [0.0, 100.0, 200.0] y [], clip -
$ osatura scroll eval d5.json sc1 --offset 400,0
viewport [0.0, 0.0, 640.0, 120.0] content [-59.92364, 0.0, 640.0, 120.0] max [0.0, 0.0]
requested offset [400.0, 0.0] -> applied [59.92364, 0.0]
$ osatura scroll eval d5.json sc1 --offset 80,0 --velocity -500,0 --time 0.5
...
requested offset [80.0, 0.0] -> applied [44.184174, 0.0]
t 0.100s offset [22.794, 0.000]
t 0.300s offset [0.051, 0.000]
t 0.500s offset [-0.197, 0.000]
```
Ohne `--velocity` zeigt `eval` den begrenzten (oder per Gummiband gedehnten) Offset zum angeforderten. Mit `--velocity` simuliert es
das Loslassen (Schwung, Snap, Feder) für `--time` Sekunden und gibt alle 0,1 s eine Zeile aus. `--size WxH` wertet für eine andere Canvas-Größe aus.
Im Beispiel ist der Inhalt so breit wie der Viewport, daher ist alles jenseits von 0 Überscrollen und zeigt das Gummiband.

### `layout-preview`

**Synopsis:** `osatura layout-preview <project.json> <out.svg> --size WxH [--frame N --scroll id=X,Y]`

Rendert einen Frame mit dem **für eine andere Canvas-Größe neu berechneten** Layout und eingezeichneten Container-Umrissen, damit du
prüfen kannst, wie ein Design auf einem Smartphone oder einem breiten Bildschirm umbricht. `--scroll id=X,Y` setzt einen Scroll-Offset (für mehrere Bereiche wiederholen).

```
$ osatura layout-preview d5.json lp.svg --size 800x480 --scroll sc1=50,0
wrote lp.svg (800x480, 1 containers, 2 layers placed)
```

## 12. Effekte, Tiefenunschärfe, Onion Skin, Trails und Kamera

### `fx`

**Synopsis:** `osatura fx <project.json> <frame> <chain.json> <out.png> [--scale N] [--layer-normals] [--clamp]`

Ein `<frame>` außerhalb des Projekts ist ein Fehler; `--clamp` rendert den nächstgelegenen gültigen Frame.

Rendert einen Frame und lässt eine **Post-Effekt-Kette** darüber laufen. `chain.json` ist eine Liste von Effekten, jeweils mit einem `type`:
`blur {radius}`, `drop_shadow {dx,dy,blur,color,opacity}`, `glow {radius,strength,color?}`, `outline {width,color}`,
`posterize {levels}`, `pixelate {size}`, `levels {in_black,in_white,gamma,out_black?,out_white?}`, `fade {amount}` und
`relight {ambient,bevel,height,lights:[{x,y,z,color?,intensity?,radius?,cast_shadow?,spot?,falloff?}],ground_shadow?,
normal_map?,ambient_color?}`. Ohne `normal_map` schattiert das Relight eine Abschrägung (Bevel), die aus dem weichgezeichneten Alphakanal erzeugt wird.

```
$ cat chain.json
[{"type":"drop_shadow","dx":6,"dy":6,"blur":3,"color":"#000000","opacity":0.5},{"type":"glow","radius":6,"strength":0.5}]
$ osatura fx demo.json 12 chain.json fx.png --scale 0.5
wrote fx.png (2 effect(s))
$ cat relight.json
[{"type":"relight","ambient":0.4,"bevel":6,"height":8,"lights":[{"x":300,"y":100,"z":120,"intensity":1.0}],"ground_shadow":0.3}]
$ osatura fx demo.json 12 relight.json lit.png --scale 0.5
wrote lit.png (1 effect(s))
```
`--layer-normals` schattiert jede Ebene mit ihrer eigenen Normal-Map (siehe `layer-normal`) statt mit einer Map für das ganze Bild.
Ein Licht ohne `z` wird abgelehnt (`missing field 'z'`). **Grenzen:** Die Kette ist eine reine PNG-Standbild-Pipeline; keiner
dieser Effekte gelangt in Lottie, GIF oder den HTML-Player (der Lottie-Exporter hat eine eigene Effekt-Teilmenge, siehe EXPORT_FORMATS.md).

### `color-space`

**Synopsis:** `osatura color-space <project.json> <out.json> srgb|linear`

Setzt den `color_space` des Projekts, also den Raum, in dem der Raster-Compositor (PNG, GIF, APNG, Video, Sprite-Sheet, `compare`, `ci`) und der Float-Compositor (`render-hdr`) mischen. `srgb` (der Standard, und der Raum, mit dem jedes Referenzbild aufgenommen wurde) mischt die gespeicherten sRGB-kodierten Werte; `linear` wandelt jede Farbe (Füllungen, Konturen, Verlaufsstopps, Hintergrund, Bildpixel) in linares Licht um, komponiert dort in f32 und wandelt am Ende einmal nach sRGB um. Eine zu 50 % schwarze Ebene über Weiß ergibt dann 188/255 statt 128/255, und Verläufe werden im linearen Licht interpoliert. Bei `srgb` wird der Schlüssel nicht geschrieben, sodass sich bestehende Dateien und Referenzbilder nicht ändern.

```
$ osatura color-space demo.json lin.json linear
wrote lin.json (color_space: linear)
$ osatura render lin.json 0 lin.png
wrote lin.png
$ osatura color-space lin.json x.json p3
unknown colour space 'p3' (srgb or linear)
```
**Grenzen:** Der 8-Bit-Renderer wechselt für lineare Projekte auf den Float-Compositor (eigener Coverage-Rasterizer, daher weichen geglättete Kanten leicht von der sRGB-Ausgabe ab); Farben von Ebeneneffekten (Schatten, Glow, Tönung) werden wie angegeben verwendet und nicht umgerechnet; `dof` und die Post-Effekt-Kette (`fx`) bleiben sRGB; SVG, Lottie und der GPU-Renderer ignorieren die Einstellung; die Bild-Neuabtastung geschieht vor der Umrechnung in sRGB. Mischmodi werden auf die linearen Werte angewendet.

### `layer-normal`

**Synopsis:** `osatura layer-normal <project.json> <out.json> <layer> <normal.png|none> [--strength S] [--fit stretch|tile|exact] [--green-down]`

Hängt eine Tangent-Space-Normal-Map an eine Bild-Ebene an (oder entfernt sie mit `none`), für `fx ... --layer-normals`.
`--fit` legt fest, wie eine Map anderer Größe angepasst wird (`stretch` Standard, `tile`, `exact`); `--green-down` kehrt den Grünkanal um
(Maps im DirectX-Stil); `--strength` skaliert die Neigung (1 = wie gespeichert, 0 = flach).

```
$ osatura layer-normal demo.json dn.json sun n.png --strength 1 --fit stretch
wrote dn.json
$ osatura fx dn.json 12 chain.json fxn.png --scale 0.5 --layer-normals
wrote fxn.png (2 effect(s))
$ osatura layer-normal dn.json dn2.json sun none
wrote dn2.json
```

### `onion` (zwei Formen)

**Synopsis:**
```
osatura onion <project.json> <frame> <out.png> [--before 2 --after 1 --step 1 --scale N]
osatura onion <project.json> <out.svg> --frame N [--before B --after A --step S --keyed --outline --all-parts
               --layers a,b --tint-before C --tint-after C --opacity X --falloff X --ghosts-only]
```
Die erste Form schreibt ein PNG mit verblassten Geistern der Frames davor und danach. Die zweite schreibt ein SVG-Standbild: den
Frame mit eingefärbten Onion-Skin-Geistern darüber. `--keyed` verwendet die nächstgelegenen **Key-Frames** statt fester Schritte;
`--outline` zeichnet Geister als Umrisse; `--all-parts` zeigt alle Teile als Geister (Standard: bewegte Teile), `--layers` beschränkt sie,
`--ghosts-only` lässt den Frame selbst weg.

```
$ osatura onion demo.json 12 on.png --before 2 --after 1 --step 2 --scale 0.5
wrote on.png
$ osatura onion demo.json on.svg --frame 12 --before 2 --after 2 --step 3 --outline --opacity 0.4
wrote on.svg (4 ghost(s) around frame 12)
$ osatura onion demo.json on2.svg --frame 12 --keyed --ghosts-only
wrote on2.svg (3 ghost(s) around frame 12)
```
Die Form wird durch das zweite Argument bestimmt: Eine Zahl bedeutet die PNG-Form, ein Name mit der Endung `.svg` die SVG-Form.

### `trail` (zwei Formen)

**Synopsis:**
```
osatura trail <project.json> <bone> <out.svg> [--from F --to F]
osatura trail <project.json> [out.svg] --target layer:PATH|bone:NAME[:head|tip]|point:LAYER:KIND:A[:B] [--from F --to F --smooth N --json]
```
Zeichnet den **Bewegungspfad** einer Knochenspitze (erste Form) oder eines beliebigen Ziels (zweite Form: eine Ebene, ein Knochenanfang/eine Knochenspitze oder ein
Deformer-Punkt), mit einer Markierung pro Frame und Markern an Keys. `--smooth N` glättet die Kurve; `--json` gibt die abgetastete
Kurve als JSON aus (dann ist `out.svg` optional).

```
$ osatura trail demo.json hand tr.svg --from 0 --to 24
wrote tr.svg (25 points)
$ osatura trail demo.json tr3.svg --target layer:sun --from 0 --to 24
wrote tr3.svg (25 ticks, 0 key marker(s))
$ osatura trail demo.json tr2.svg --target bone:hand:tip --from 0 --to 24 --smooth 2 --json
{ "curve": [ [212.8, 83.84], [213.77, 83.99], ...
```

### `dof` und `dof-export`

**Synopsis:**
```
osatura dof        <project.json> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24 --scale N --clamp]
osatura dof-export <project.json> <out.gif|out.apng|out.mp4|out.webm|out-folder> [--from F --to F --focus N --strength N --max-radius N --scale N --codec vp9|av1]
```
Ein `<frame>` außerhalb des Projekts ist bei `dof` (und bei `render-hdr` und `fx`) ein Fehler; `--clamp` rendert stattdessen den nächstgelegenen gültigen Frame, wie bei `render`.

Tiefenunschärfe: Jede **Ebene der obersten Ebene** wird um `|parallax - focus|` mal `strength` weichgezeichnet (Pixel pro Einheit
Parallaxen-Abstand, begrenzt auf `--max-radius`). Ohne `--focus` / `--strength` folgen die Werte den Key-animierten
Kanälen `camera_focus` / `camera_blur` (Standard-Fokus 1.0). Die Ebenen beziehen ihre Tiefe aus `parallax` (siehe Abschnitt 5).
`dof-export` rendert den Bereich `--from`..`--to` (ganze Frame-Nummern; Standard: von 0 bis zum Ende) in ein animiertes GIF, ein
animiertes PNG, ein `.mp4`/`.webm`-Video (benötigt `ffmpeg`; `--codec` ist standardmäßig `vp9`; es wird kein Audio eingebunden) oder, bei einem Namen ohne
Endung, in einen Ordner mit nummerierten PNGs (`f0000.png`...). Jede andere Endung wird abgelehnt (`dof-export writes .gif, .apng,
.mp4, .webm or a folder (no extension)`, es wird nichts geschrieben).

```
$ osatura dof demo.json 12 dof.png --focus 1.0 --strength 12 --max-radius 24 --scale 0.5
wrote dof.png
$ osatura dof-export demo.json dof.gif --from 0 --to 6 --scale 0.25
wrote dof.gif
$ osatura dof-export demo.json dofdir --from 0 --to 2 --scale 0.25      # dofdir/f0000.png ... f0002.png
wrote dofdir
```
**Siehe auch:** `parallax`, `render --proxy`, `shot`/`camera-seq` für Kamerabewegungen. **Grenzen:** Die Unschärfe ist eine
Näherung pro Ebene, kein Linsenmodell (keine Bokeh-Formen, keine Verdeckungsbehandlung); sie wird nicht nach Lottie exportiert.

## 13. Level of Detail, Größenberichte und export-fit

### `size-report`

**Synopsis:** `osatura size-report <project.json> [--tol T] [--json]`

Zeigt, **wo die Bytes sind**: in der Projektdatei, im eingebackenen Lottie und auf der HTML-Seite, aufgeschlüsselt nach Ebene, Bild und Kanal
(Keys, und wie viele bei der Toleranz `--tol` entfernbar sind, Standard 0,05).

```
$ osatura size-report small.json
project file: 17959 bytes (3511 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2983  arm
         2053  leg
  ...
  by channel (keys, removable at tol 0.05):
          420  foot_ctrl.BoneOffsetX  keys 5 removable 1
  ...
baked Lottie: 46843 bytes (11180 gzip), by layer:
         9181  arm
```
`--json` gibt dieselben Daten als JSON aus (`html.defs_bytes_total`, `frame_bytes_max`, ...). Verwende es vor `export-fit` oder `lod`,
um zu sehen, welche Ebene du vereinfachen solltest.

### `lod`

**Synopsis:** `osatura lod <project.json> --tolerance-px 0.5 [--scale S | --size WxH] [--levels 4] [--ratio 2] [--out-dir D] [--report r.json] [--verify] [--frames N]`

Erzeugt **Detailstufen** (Levels of Detail) L1..Ln, jeweils mit einer *deklarierten Fehlerschranke in Ausgabepixeln*: Pfadvereinfachung, Key-
Reduktion, Mesh-Reduktion und Culling. Stufe k hat die Toleranz `--tolerance-px x ratio^(k-1)`. `--scale` oder `--size` gibt an,
wie groß die Ausgabe dargestellt wird (Standard 1 px pro Einheit). `--out-dir` (Standard `.`) nimmt `<name>_L1.json` ... auf und
`--report` einen JSON-Bericht. `--verify` rendert `--frames N` Frames und **misst** die tatsächliche Abweichung (Exit 1, wenn sie über
der Schranke liegt).

```
$ osatura lod small.json --tolerance-px 1 --levels 3 --out-dir lv --report rep.json --verify --frames 4
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  path  mesh  keys  cull  measured_px  ratio
L0      0.000       74       0    37      14    17959       0.0000 ...                              -      -
L1      1.000       70       0    37      14    17651       0.3861 0.386 0.000 0.000 0.000       0.3342   1.2x
L2      2.000       69       0    37      14    17573       0.9277 ...                           0.8185   1.1x
L3      4.000       65       0    37      14    17251       1.5372 ...                           1.3367   1.1x
scale 1 px per unit; level k tolerance = 1 px x 2^(k-1); files lv/s_L1..L3.json
  L1: keys: 3 of 8 keyed channels have a documented lever, 0 reduced
  L1: removed 4 path anchor(s)
$ ls lv
s_L1.json  s_L2.json  s_L3.json
```
**Stolperfalle:** Die Einsparungen bei einer kleinen, von Hand gebauten Demo sind winzig (etwa 4 % bei 4 px); LODs lohnen sich bei vektorisierter oder Mesh-lastiger Grafik.
Der Bericht und die Stufendateien werden mit einem einfachen (nicht atomaren) Schreibvorgang geschrieben.

### `lod-pick`

**Synopsis:** `osatura lod-pick <project.json> --screen-px WxH [--max-px 0.5] [--tolerance-px 0.25] [--levels 4] [--out chosen.json]`

Wählt die **gröbste Stufe**, deren deklarierte Schranke innerhalb von `--max-px` Pixeln bleibt, wenn die Grafik auf einem
Bildschirm von `--screen-px` dargestellt wird (der Maßstab ist Bildschirmgröße geteilt durch Projektgröße). `--out` schreibt diese Stufe.

```
$ osatura lod-pick small.json --screen-px 320x240 --out chosen.json
level  tol_px  anchors ...
screen 320x240 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.4638 px, 69 anchors, 0 mesh vertices, 37 keys)
wrote chosen.json
$ osatura lod-pick small.json --screen-px 64x48 --max-px 2 --out chosen2.json
screen 64x48 -> scale 0.1000 px/unit; budget 2 px -> level L4 (declared 0.5310 px, 62 anchors, 0 mesh vertices, 37 keys)
```

### `export-fit`

**Synopsis:** `osatura export-fit <project.json> <out> --max-size 500KB [--format lottie|html|gif|apng|webm|project] [--min-ssim X] [--max-seconds 60] [--measure raw|gzip] [--frames N] [--search ladder|descent|grid] [--report out.json] [--pareto out.csv] [--audit] [--no-baseline]`

Findet den **Export mit der besten Qualität, der in ein Byte-Budget passt**: Der Befehl durchsucht die Qualitätsregler des Exporters (Lottie: `decimals`,
`path_eps`, `key_tol`, `frame_step`; andere Formate haben eigene) und bewertet jeden Kandidaten per SSIM gegen das
ursprüngliche Rendering. `--max-size` akzeptiert `500KB` (1000 Bytes), `1.5MB`, `64000`, `200KiB` (1024 Bytes). Das Format ergibt sich aus
`--format` oder der Endung der Ausgabedatei (`.json` ist Lottie, `.html`, `.gif`, `.apng`, `.webm`); `project` schreibt ein verschlanktes
Projekt-JSON. `--measure gzip` budgetiert die gzip-Größe (für Server, die komprimieren). `--min-ssim` ist eine Qualitätsuntergrenze.
`--max-seconds` ist ein **echtes Wanduhr-Budget** (Standard 60; `0` = unbegrenzt): Nach Ablauf wird das bisher beste Ergebnis behalten.
`--frames N` gibt an, wie viele Frames für den SSIM abgetastet werden (Standard 8). `--search` wählt die Strategie (`descent` ist der Standard).
`--report` / `--pareto` schreiben das Suchprotokoll als JSON / CSV; `--audit` gibt jeden Kandidaten aus; `--no-baseline` überspringt
den Vergleich mit den Standardeinstellungen.

```
$ osatura export-fit small.json fit.json --max-size 30KB --format lottie --report fr.json
lottie -> fit.json: 25756 bytes raw, 5736 gzip (budget 30000 raw); SSIM mean 0.9699, worst 0.9308 at frame 5, max CIEDE2000 68.67; 5 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (...)
$ osatura export-fit small.json fit.html --max-size 60KB --format html --max-seconds 20 --frames 4
html -> fit.html: 59166 bytes raw, 8550 gzip (budget 60000 raw); SSIM mean 1.0000, worst 1.0000 at frame 5, max CIEDE2000 8.54; 1 candidates built
existing exporter at defaults: 66391 bytes raw, 8933 gzip
$ osatura export-fit small.json fit2.json --max-size 1KB --format lottie --max-seconds 10 --frames 3
lottie -> fit2.json: 14751 bytes raw, 2418 gzip (budget 1000 raw); ...
knobs at their floor: decimals, path_eps, key_tol, frame_step
budget not met: the smallest export found is 14751 bytes (raw), above the budget of 1000 bytes: ...          [exit 1]
$ osatura export-fit small.json fit.xyz --max-size 1KB
cannot tell the format from the output name; pass --format lottie|html|gif|apng|webm|project   [exit 1]
```
**Exit-Codes:** 0, wenn das Ergebnis ins Budget passt (auch wenn das Zeitbudget die Suche vorzeitig beendet hat); 1, wenn das Budget
nicht eingehalten werden kann (der kleinste Export wird trotzdem geschrieben) oder bei fehlerhaften Argumenten. **Stolperfallen:** Der Befehl arbeitet auf der eingebackenen Ausgabe,
kann also nicht mehr entfernen, als die Regler zulassen (ein Lottie von 1 KB ist für ein Rig mit 14 Ebenen unmöglich); jeder Kandidat ist ein vollständiger
Export, große Projekte brauchen daher ein großzügiges `--max-seconds` (ein Debug-Build ist viel langsamer als ein Release-Build). Die Berichts-
dateien werden mit einem einfachen Schreibvorgang geschrieben. **Siehe auch:** `size-report`, `lod`, `export` (Abschnitt 4), EXPORT_FORMATS.md (Tipps zur Größe).

## 14. Qualitätssicherung

Verwandt, weiter oben dokumentiert: `check` (Validierung, Abschnitt 3), `diff` (semantischer Diff, Abschnitt 3), `compare` (Bild-Diff zweier
Renderings, Abschnitt 4).

### `lint`

**Synopsis:** `osatura lint <project.json>`

Statische Prüfungen von Rig und Animation. Gibt pro Befund eine Zeile aus: `Severity CODE [target:property] message`. Exit 1, wenn ein
Befund den Schweregrad Error oder höher hat; Info und Warning enden mit Exit 0.
```
$ osatura lint small.json
Info L007 [:CameraX] key at frame 24 lies outside the 0..8 timeline
Info L007 [arm_low:BoneAngle] key at frame 12 lies outside the 0..8 timeline
```
`L007` ist ein Key außerhalb der Timeline (hier, weil `small.json` die auf 8 Frames gekürzte Demo ist).

### `contracts`

**Synopsis:** `osatura contracts <project.json> <contracts.json>`

**Bewegungsverträge** (Motion Contracts): Unit-Tests für Animation. `contracts.json` ist eine Liste von Verträgen mit einem `type`:
`loop_seamless {tol}`, `tip_range {bone,axis,min,max}`, `max_speed {bone,px_per_frame}` (weitere Typen gibt es im
`Contract`-Enum der Engine). Verstöße werden auf stdout ausgegeben, die Anzahl auf stderr, und der Exit-Code ist 1.
```
$ cat contracts.json
[{"type":"loop_seamless","tol":0.5},{"type":"tip_range","bone":"hand","axis":"y","min":0,"max":400},{"type":"max_speed","bone":"hand","px_per_frame":3}]
$ osatura contracts small.json contracts.json
contract 0 violated at frame 8: bone 'arm_up' ends 35.4 px away from where it started (tolerance 0.5)
...
contract 2 violated at frame 6: tip of 'hand' moves 3.3 px/frame (limit 3)
7 violation(s)                                                            [stderr, exit 1]
```

### `coach`

**Synopsis:** `osatura coach <project.json>`

Vorschläge zu Timing und Animationsprinzipien (Overlap, Anticipation, Easing). Nur Ratschläge, Exit immer 0.
```
$ osatura coach small.json
C003 [, arm_low, arm_up, foot_ctrl, hand, mouth, target] frame 0: 7 parts start moving on frame 0; offset them by a few frames (overlap) for a more natural feel
```

### `safety`

**Synopsis:** `osatura safety <project.json> [--scale N]`

Bericht zu Blitzen, roten Blitzen und Kamerabewegung (eine Annäherung an WCAG 2.3.1; ein Hilfsmittel für die Entwicklung, keine
Sendezertifizierung). Rendert jeden Frame mit `--scale` (verwende bei langen Projekten einen kleinen Wert).
```
$ osatura safety small.json --scale 0.25
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.06 widths/s, rotation 0°/s, zoom ×1.00/s
```

### `reduced-motion`

**Synopsis:** `osatura reduced-motion <project.json> <out.json> [--factor 0.25]`

Schreibt eine ruhigere Kopie: Kamerabewegung und Wobble werden mit `--factor` skaliert (Standard 0,25).
```
$ osatura reduced-motion small.json calm.json --factor 0.25
wrote calm.json
```

### `golden`

**Synopsis:**
```
osatura golden write <project.json> <golden.json> [--scale N]
osatura golden check <project.json> <golden.json> [--tol T]
```
Bei `write --scale N` liegt N in (0, 1]: Ein größerer Wert wird **abgelehnt** (`golden write: --scale 2 is above 1 ...`, es wird nichts geschrieben; früher wurde er stillschweigend auf 1 begrenzt). `check` rendert neu mit der in der Golden-Datei gespeicherten Skalierung und hat keine Option `--scale`: `golden check ... --scale N` wird als unbekannte Option abgelehnt.


Ein leichtgewichtiger Regressionstest: `write` zeichnet einen Hash und ein 16x16-Thumbnail jedes Frames in `golden.json` auf; `check`
rendert neu und vergleicht. **Standardmäßig exakt** (`--tol 0`): Jeder Unterschied im Pixel-Hash schlägt fehl. `--tol T` (T > 0) lockert
die Prüfung: Ein geänderter Hash besteht trotzdem, wenn der Thumbnail-Abstand höchstens T beträgt. Ein negatives oder nicht endliches `--tol` ist ein Fehler.
Für bildbasierte Referenzen mit einer Wahrnehmungsmetrik verwende `ci`.
```
$ osatura golden write small.json g.json --scale 0.25
wrote g.json (8 frames)
$ osatura golden check small.json g.json
all 8 frames match exactly
$ osatura golden check small.json g.json --tol 0.01
all 8 frames match (within thumbnail tolerance 0.01)
```
Abweichende Frames werden aufgelistet (Hash-Änderung, Thumbnail-Abstand, Anzahl der abweichenden der 256 Thumbnail-Zellen, größter Kanal-
unterschied von 255), und der Exit-Code ist 1:
```
$ osatura golden check changed.json g.json
frame 1 differs: pixel hash changed, thumbnail distance 0.003, 12 of 256 thumbnail cells differ, max channel difference 31/255
2 frame(s) differ (exact comparison; --tol T accepts small differences)       [stderr, exit 1]
```
Ein Frame, der nur auf einer Seite vorhanden ist, wird als `one side only` gemeldet. **Grenzen:** Die Golden-Datei behält nur den Hash und das
Thumbnail, die gemeldeten Zahlen gelten also auf Thumbnail-Ebene, nicht auf Pixel-Ebene. Eine Änderung, die im Rendering unsichtbar ist (zum Beispiel
eine Hintergrundfarbe, die von einer deckenden Ebene vollständig verdeckt wird, wie der Himmel der Demo), lässt den Hash berechtigterweise unverändert.

### `ci` (init, run, bless, diff)

**Synopsis:**
```
osatura ci init <project.json> [--out osatura-ci.json] [--no-goldens]
osatura ci run  [manifest] [--update] [--bless] [--prune] [--only a,b] [--junit f.xml] [--html f.html] [--json f.json] [--artifacts dir] [--quiet]
osatura ci bless [manifest]
osatura ci diff a.png b.png [--metric ssim|de2000|de76|mse] [--tol T] [--ignore x,y,w,h;...] [--aa N] [--out heat.png] [--side s.png] [--overlay o.png] [--flicker f.gif]
```
**Animation-CI**: Referenzbilder, verglichen mit einer Wahrnehmungsmetrik (Luma-SSIM, CIEDE2000), dazu strukturelle Prüfungen und
Budgets. Das Manifestformat und der Workflow sind in [CI.md](CI.md) beschrieben. `exit 0` = bestanden, `1` = eine Prüfung ist fehlgeschlagen,
`2` = Fehler (Manifest nicht lesbar, ...).
```
$ osatura ci init small.json --out ci.json
wrote ci.json with 8 checks:
  validate     validate
  render       frames
  render       frames-float
  determinism  determinism
  fuzz         pose-fuzz
  limits       limits
  export-size  gif-size
$ osatura ci run ci.json --junit j.xml --html r.html --json ci_r.json
[PASS   ] validate     validate           project compiles; 8 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[INFO   ] fuzz         pose-fuzz          98 of 100 pose samples fail (98.00%); not a gate: non-gating (gate = false)
[PASS   ] export-size  gif-size           gif export is 82881 bytes (budget 124322)
[INFO   ] perf         render-speed       median 429.67 ms/frame, budget 1633 ms (+50% tolerance = 2449.50 ms)
8 checks: 6 passed, 0 failed, 0 errors, 2 info, 0 updated, 0 blessed (14.7 s)
$ osatura ci diff a.png b.png --metric ssim --out heat.png
size        480x360
SSIM        0.90536   (luma)   channels min 0.90300   (R, G, B)
de2000      max 68.674   mean 1.9434
pixels      11124 differ (6.4375%), 31 forgiven as anti-aliasing, 0 ignored
regions     118
images differ: luma SSIM 0.90536 < 0.99; ...                                [stderr, exit 1]
$ osatura ci run nonexist.json
cannot read manifest nonexist.json: No such file or directory               [exit 2]
```
`init` schreibt das Manifest und, außer mit `--no-goldens`, die Referenz-PNGs in den Ordner `goldens/` daneben. `run
--update` schreibt fehlende oder geänderte Referenzbilder neu, `--bless` übernimmt die neuen Renderings als Referenzbilder, `--prune` entfernt
Referenzdateien, die keine Prüfung mehr referenziert, `--only a,b` führt die genannten Prüfungen aus. `bless` ist `run --bless`. `diff` vergleicht zwei PNGs: `--tol` ist der Schwellenwert
der Metrik, `--ignore` maskiert Rechtecke (`x,y,w,h;...`), `--aa N` verzeiht Anti-Aliasing-Unterschiede, und `--out`,
`--side`, `--overlay`, `--flicker` schreiben eine Heatmap, ein Nebeneinander-Bild, ein Overlay und ein Flicker-GIF.
**Stolperfalle:** Die Berichte (`--junit`, `--html`, `--json`) und Referenzdateien werden mit dem atomaren, Symlinks ablehnenden Writer geschrieben (übergeordnete
Ordner werden angelegt); ein Berichtspfad, der ein Symlink ist, wird abgelehnt, nachdem die Prüfungen gelaufen sind (die Prüfungen sind langsam, übergib also einen echten Pfad). Ein Debug-Build ist langsam: Die 8 Prüfungen oben brauchten bei einem Projekt mit 8 Frames 15 s.

### `fuzz`

**Synopsis:** `osatura fuzz <project.json> [--samples N] [--seed S] [--strategy random|lhs|halton|sweep|corners|pca] [--frame F] [--report out.json] [--heatmap out.svg] [--minimize] [--gate] [--max-fail-rate R] [--props angle,scale,offx,offy] [--only BONES] [--margin DEG] [--db posedb.json] [--ignore KINDS] [--no-deformers]`

**Pose-Fuzzing**: tastet Posen innerhalb der Knochengrenzen ab und meldet, wo das Rig bricht (Falten, Dehnung, Spitzen,
Selbstdurchdringung, IK nicht erreichbar, Verletzungen von Limits, NaN, außerhalb des Canvas). `--strategy` wählt den Sampler (`pca` tastet entlang der PCA-Achsen der eigenen Key-Posen des Projekts oder der mit `--db` angegebenen
Pose-Datenbank ab), `--minimize` schrumpft eine fehlschlagende Pose auf einen minimalen Reproduzenten, `--heatmap` schreibt eine Belastungskarte,
`--ignore` überspringt Fehlerarten, `--only` beschränkt auf Knochen. Ohne Gate ist `fuzz` ein **Bericht**: Er gibt die Zusammenfassung aus und endet mit Exit 0 (`(report only: exit 0; add --gate, ...)`). `--gate` schlägt fehl (Exit 1), sobald ein Sample fehlschlägt; `--max-fail-rate R` (ein **Anteil** 0..1) impliziert ein Gate bei dieser Rate. (Die eigene `fuzz`-Prüfung von `ci` hat eine eigene `gate`-Einstellung und bleibt unberührt.)
```
$ osatura fuzz small.json --samples 30 --seed 1 --strategy lhs --report fz.json --heatmap hm.svg --max-fail-rate 1.0
wrote fz.json
wrote hm.svg
fuzz 'small': 30 samples (lhs, seed 1, frame 0), 25 parameters, 30 failures (100.0%), max severity 54.62, baseline 0.00, 0.40 ms/sample
  ik_unreachable: 27
  off_canvas: 1
  stretch: 2
$ osatura fuzz small.json --samples 10 --max-fail-rate 0.5
fuzz: failure rate 100.00% exceeds --max-fail-rate 50.00%                   [stderr, exit 1]
```
Das IK-Ziel der Demo ist absichtlich nur in einem kleinen Bereich erreichbar, daher ist fast jede zufällige Pose „nicht erreichbar“:
Betrachte die Rate als Regressionszahl, nicht als Bestanden/Durchgefallen für dieses Rig. Der Bericht und die Heatmap werden atomar geschrieben, und ein Pfad, der ein Symlink ist, wird abgelehnt (außer mit `--follow-symlinks`); der übergeordnete Ordner muss existieren.

### `fuzz-grid`

**Synopsis:** `osatura fuzz-grid <project.json> --x PARAM --y PARAM [--res N] out.svg|out.png|out.json`

Eine Schweregrad-Karte zweier Parameter (`bone:NAME:PROP`, zum Beispiel `bone:arm_up:angle`) auf einem `N x N`-Raster (Standard 32).
Der Ausgabetyp richtet sich nach der Dateiendung.
```
$ osatura fuzz-grid small.json --x bone:arm_up:angle --y bone:arm_low:angle --res 8 grid.svg
wrote grid.svg (8x8 cells, 59.4% failing; x = bone:arm_up:angle, y = bone:arm_low:angle)
```
Derselbe Vorbehalt zum einfachen Schreibvorgang wie bei `fuzz`.

## 15. Versionsverwaltung (`vcs`)

**Synopsis:**
```
osatura vcs init <project.json> [--author A]
osatura vcs commit [-m MSG | --message MSG] [--author A] [--repo DIR]
osatura vcs log [REV] [--limit N] [--json] [--repo DIR]
osatura vcs status [--json] [--repo DIR]
osatura vcs diff [A [B]] [--json] [--repo DIR]
osatura vcs show [REV] [--json] [--repo DIR]
osatura vcs branch [NAME [AT] | -d NAME] [--repo DIR]
osatura vcs tag [NAME [AT]] [--repo DIR]
osatura vcs checkout REV [--force] [--repo DIR]
osatura vcs merge REV [--no-ff] [-m MSG | --message MSG] [--author A] [--abort] [--repo DIR]
osatura vcs cherry-pick|revert REV [--author A] [--repo DIR]
osatura vcs resolve PATH [--ours] [--theirs] [--value V] [--repo DIR]
osatura vcs fsck|gc [--repo DIR]
```
Die Optionen gelten pro Unterbefehl: `--json` nur bei `log`, `status`, `diff` und `show`; `--force` nur bei `checkout`; `--ours`, `--theirs`, `--value` (genau eine) nur bei `resolve`.

Eine kleine inhaltsadressierte Historie für **eine Projektdatei** mit einem **semantischen Merge**: Änderungen werden als
Dokumentpfade (JSON-Pointer) verglichen, nicht als Textzeilen, sodass zwei Personen, die verschiedene Keys bearbeiten, nie in Konflikt geraten. `init` legt
einen Ordner `.osatura-vcs/` neben dem Projekt an; der Standard-Branch ist `main`. Revisionen (`REV`) sind Branch-Namen, Tag-
Namen oder Präfixe von Commit-IDs.

```
$ osatura vcs init p.json
initialised ./.osatura-vcs for p.json (no commits yet; run: osatura vcs commit -m "...")
$ osatura vcs commit -m first --author alexio
[5f80a63179] first
$ (edit p.json)
$ osatura vcs status
on branch main
HEAD 5f80a63179
uncommitted changes:
  project: background changed "#f6f1e7" -> "#ffffff"
$ osatura vcs commit -m "white bg"
[29c09d9368] white bg
$ osatura vcs branch alt          # branch alt at 29c09d9368
$ osatura vcs checkout alt        # checked out alt (29c09d9368)
$ osatura vcs log --limit 5
4e21f15940  unknown  30 fps
29c09d9368  unknown  white bg
5f80a63179  alexio  first
$ osatura vcs merge alt           # fast-forward or a merge commit; --no-ff forces a merge commit
merge committed as f08db3f77f
$ osatura vcs tag v1 ; osatura vcs tag
tag v1 at 4e21f15940
v1  4e21f15940
$ osatura vcs fsck
ok: 50 objects, history and refs are consistent
$ osatura vcs gc
gc: kept 50 objects, removed 0 unreachable
```

**Konflikte.** Haben beide Seiten denselben Wert geändert, hält `merge` an, behält *unsere* Seite in der Projektdatei, listet die
Konflikte auf und endet mit Exit 1:
```
$ osatura vcs merge main
merge stopped with 1 conflict(s); the project file holds our side for each:
  CONFLICT /background
      project: background: ours "#222222" vs theirs "#333333" (base "#000000")
resolve each with: osatura vcs resolve <path> --ours|--theirs|--value <json>, then: osatura vcs commit   (or: osatura vcs merge --abort)
$ osatura vcs resolve background --theirs
no conflict at 'background' (see 'vcs status')                              [exit 1]   (the path needs its leading slash)
$ osatura vcs resolve /background --theirs
resolved /background; 0 conflict(s) left - run: osatura vcs commit
$ osatura vcs commit -m merged
[bd405f446a] merged
$ osatura vcs merge --abort       # alternatively: merge aborted; project restored to HEAD
```
Solange ein Merge, Cherry-Pick oder Revert ungelöst ist, werden `checkout`, `revert` und `commit` abgelehnt mit
`a merge / cherry-pick / revert is in progress: resolve the conflicts and commit, or run 'vcs merge --abort'`.
Weitere Fakten: `diff` ohne Argument vergleicht die Arbeitsdatei mit HEAD, `diff A B` zwei Revisionen (`--json` gibt eine Liste
aus `{kind,path,description}` aus); `show` gibt einen Commit und seine Änderungsliste aus; `branch` ohne Argument listet die Branches auf (`*` markiert
den aktuellen); `branch -d NAME` weigert sich, den ausgecheckten Branch zu löschen; `checkout --force` verwirft nicht committete
Änderungen; `revert REV` und `cherry-pick REV` erzeugen einen neuen Commit (oder halten bei Konflikten an wie `merge`); `--repo DIR`
wählt einen anderen Repository-Ordner als den aktuellen; `--author` ist standardmäßig `unknown`; `log --json` gibt die Commit-
Objekte aus. **Grenzen:** eine Datei pro Repository, nur lokal (keine Remotes, kein Push/Pull), keine Umbenennungsverfolgung über JSON-Pfade hinaus;
vom Projekt referenzierte binäre Assets werden nicht versioniert.

## 16. Studio, Zusammenarbeit, MCP-Agent-Server und UI-Registry

### `studio`

**Synopsis:** `osatura studio <project.json> [--port N] [--ui-config F] [--bind ADDR] [--allow-host NAMES] [--public-origin URL] [--source-url URL] [--collab-token T (16+ characters)] [--collab-secret-file F [--collab-revoke NAMES]] [--collab-rate N] [--collab-log LOG [--collab-nosync]]`

Startet den **Browser-Editor** (Rückgängig/Wiederholen, Timeline, Live- und Gesichts-Panels, VCS und agentenartige Operationen) als lokalen Webserver
und blockiert bis Strg-C. Standardwerte: `--port 8080`, `--bind 127.0.0.1`. Das Projekt wird beim Start validiert; gespeichert wird
in dieselbe Datei (`POST /api/save`).
```
$ osatura studio demo.json --port 18931 --ui-config ui.json
AGPL-3.0 source offer: http://127.0.0.1:18931/source
OSATURA studio on http://127.0.0.1:18931  (Ctrl-C to stop; saves go to demo.json)
UI settings (layout, shortcuts) in ui.json
```
Optionen: `--ui-config F` hält Layout und Tastenkürzel (Standard `~/.config/osatura/ui.json`, unter Windows `%APPDATA%\OSATURA\ui.json`); `--source-url URL` setzt den
Link zum veröffentlichten Quellcode, der unter `/source` und von `--version` angezeigt wird (AGPL-3.0 Abschnitt 13; die offiziellen Release-Binärdateien verwenden standardmäßig den Commit, aus dem sie gebaut wurden); `--allow-host NAMES` und `--public-origin URL` akzeptieren
Anfragen unter einem öffentlichen Namen hinter einem TLS-Reverse-Proxy; `--collab-token T` (mindestens 16 Zeichen, zum Beispiel
`openssl rand -hex 16`) aktiviert die Mehrbenutzer-Sitzung und ist **für jedes `--bind` außerhalb von Loopback erforderlich**; `--collab-nosync` ist nur zusammen mit `--collab-log` gültig (sonst ein Fehler); `--collab-log LOG`
speichert die gemeinsame Sitzung dauerhaft und nimmt sie nach einem Neustart wieder auf (`--collab-nosync` überspringt die Synchronisierung beim Start).

**Härtungsoptionen** (Welle 7; Details und das Bedrohungsmodell in [SECURITY.md](SECURITY.md)): `--collab-secret-file F` ersetzt
das eine gemeinsame Bearer-Token durch **Tokens pro Teilnehmer**: F enthält ein zufälliges Geheimnis (erste Zeile, mindestens 16 Zeichen, Modus 600 unter
Unix, sonst wird es abgelehnt), und jede Person erhält ein Token von `osatura collab-token NAME`; ein Token ist an einen Namen gebunden
(beim Beitreten wird dieser Name verwendet, niemand kann als jemand anderes beitreten), läuft ab und wird durch Neuberechnung seines HMAC-SHA-256 geprüft, sodass der
Server keine Tabelle führt. `--collab-revoke NAMES` (kommagetrennt, erfordert `--collab-secret-file`) lehnt die Tokens dieser
Namen ab, bevor sie ablaufen. `--collab-token` und `--collab-secret-file` lassen sich kombinieren (das gemeinsame Token wirkt dann als
Admin-Schlüssel); jedes von beiden erfüllt die Regel, dass eine Bind-Adresse außerhalb von Loopback eine Zugangsberechtigung braucht. `--collab-rate N` begrenzt jede Client-
Adresse auf N Anfragen pro Sekunde (Bursts von 2N; Antwort `429` mit `Retry-After`); der Standard ist 100, wenn eine Zugangsberechtigung
verlangt wird, und sonst kein Limit; 0 schaltet es ab. Eine Bind-Adresse außerhalb von Loopback gibt Warnungen aus (unverschlüsseltes HTTP; alle Schnittstellen bei
`0.0.0.0` / `::`; ein gemeinsames Token hat keine Namen und kein Ablaufdatum).

**Sicherheitsregeln** (verifiziert): Der Server beantwortet nur Anfragen, deren `Host` einer seiner eigenen Namen ist
(`curl -H 'Host: evil.example' .../api/state` ergibt 421); `Origin` und `Sec-Fetch-Site` von einer anderen Seite ergeben 403; ein POST-Body
muss `application/json` sein (`curl -X POST -d '{}'` ergibt 415). Ein kurzes Token wird abgelehnt
(`--collab-token must be at least 16 characters`), und `--bind 0.0.0.0` ohne Token wird abgelehnt
(`refusing to listen on 0.0.0.0:PORT: the studio has no accounts and no TLS, ...`). Der Server hat keine Konten und kein TLS: Setze
einen TLS-Proxy davor, wenn es um mehr als deinen eigenen Rechner geht; ist `--public-origin` eine `https://`-URL, erhält das Login-Cookie das Attribut `Secure` für Anfragen, die unter diesem Namen eintreffen (oder mit `Origin` https / `X-Forwarded-Proto: https`). Die Seite liefert unter `/favicon.ico` ein kleines SVG-Favicon aus. Die HTTP-API (zum Beispiel `GET /api/state`, `GET /api/project`,
`POST /api/op`, `POST /api/save`, `GET /frame/N.svg`) existiert für die Seite selbst und ist keine stabile öffentliche API.

### `collab-token`

**Synopsis:** `osatura collab-token <name> [--secret-file F] [--ttl 28800]`

Gibt ein Zusammenarbeits-Token pro Teilnehmer für ein Studio aus, das mit `--collab-secret-file F` gestartet wurde: `ap1.<name>.<expires>.<mac>`,
wobei `mac` der hexadezimale HMAC-SHA-256 aus dem Namen und dem Ablaufzeitpunkt unter dem Geheimnis ist. `<name>` besteht aus 1 bis 32 Zeichen: Buchstaben,
Ziffern, `_` und `-`. `--ttl` ist die Lebensdauer in Sekunden (Standard 28800 = 8 Stunden, höchstens 2592000 = 30 Tage). Das Geheimnis wird aus
der Datei `F` (erste Zeile) oder, ohne `--secret-file`, aus der Umgebungsvariable `OSATURA_COLLAB_SECRET` gelesen; es wird nie
als Argument akzeptiert, wo andere Benutzer es sehen können. Das Token geht nach stdout, der Ablaufzeitpunkt nach stderr.

```
$ openssl rand -hex 32 > collab.secret && chmod 600 collab.secret
$ osatura studio demo.json --collab-secret-file collab.secret
$ osatura collab-token ana --secret-file collab.secret --ttl 3600
ap1.ana.1790000000.3f0c...
token for 'ana', valid until Unix time 1790000000 (in 3600 s); give it to that person only; ...
$ osatura collab-token ana --secret-file collab.secret --ttl 0
--ttl needs a whole number of seconds (1 to 2592000)
```
Die Person fügt das Token auf der Login-Seite des Studios ein (es wird dann in einem HttpOnly-Cookie gehalten, das mit dem Token abläuft) oder sendet es als
`X-Collab-Token`. Ein abgelaufenes Token wird mit `401` beantwortet, mit dem Ablaufzeitpunkt in der Meldung; der Name des Teilnehmers wird dem
Token entnommen. **Siehe auch:** `studio`.

### `collab-sim`

**Synopsis:** `osatura collab-sim <project.json> [--clients N] [--ops M] [--seed S] [--undo-pct P] [--replays K] [--require-valid] [--raw] [--json]`

Ein Test-Harness für die Zusammenarbeits-Engine: N Replikate bearbeiten das Projekt gleichzeitig bei zufälliger Zustellreihenfolge und
Duplikaten, danach wird geprüft, ob alle dasselbe Dokument halten (Exit 1, wenn nicht). Die Standard-Bearbeitungen sind gültige Projekt-
Bearbeitungen (Zahlen, Key-Werte, Sichtbarkeit, eigene Ebenen); `--raw` verwendet beliebige JSON-Bearbeitungen (meist kein gültiges Projekt);
`--require-valid` lässt auch ein nicht kompilierbares Ergebnis mit Exit 1 enden.
```
$ osatura collab-sim small.json --clients 3 --ops 20 --seed 2 --json
{"buffered":12,"clients":3,"clocks_identical":true,"converged":true,"converged_ok":true,"deliveries":34,"duplicates_dropped":0,"edits":20,"envelopes":17,...}
```
`converged_ok` ist die Konvergenz allein; `ok` verlangt zusätzlich, dass das zusammengeführte Dokument kompiliert.

### `mcp`

**Synopsis:** `osatura mcp <project.json>`

Ein **Agent-Server**: JSON-RPC 2.0 (das Model Context Protocol) über stdin/stdout, eine JSON-Nachricht pro Zeile. Er lädt das
Projekt und stellt die Tools `get_project`, `stats`, `lint`, `coach`, `safety`, `apply_ops`, `set_key`, `undo`, `redo`,
`render_frame` und `save` bereit. Es wird nichts geschrieben, bis `save` aufgerufen wird. Stdout führt nur Protokollnachrichten.
```
$ printf '%s\n' \
 '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","version":"0"}}}' \
 '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
 '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
 '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"lint","arguments":{}}}' | osatura mcp demo.json
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"protocolVersion":"2024-11-05","serverInfo":{"name":"osatura","version":"0.1.0"}}}
{"id":2,"jsonrpc":"2.0","result":{"tools":[{"description":"Return the project JSON, or the value at a JSON pointer ...","name":"get_project",...}]}}
{"id":3,"jsonrpc":"2.0","result":{"content":[{"text":"[]","type":"text"}],"isError":false}}
```
Der Server beendet sich (Code 0), wenn stdin geschlossen wird.

### `ui`

**Synopsis:**
```
osatura ui commands|shortcuts [--ui-config F] [--json]
osatura ui layout presets|show <name>|validate <file.json>
osatura ui search <query...> [--ui-config F]
```
Die **Befehls-Registry** des Studios: `commands` listet jeden Befehl mit Kategorie, Titel und Tastenbelegung auf;
`shortcuts` listet die wirksamen Tastenkürzel (Standardwerte plus die aus `--ui-config`) und etwaige Konflikte auf; `layout presets` listet die
Dock-Layout-Presets auf, `layout show NAME` gibt eines als JSON aus und `layout validate FILE` prüft eine Layout-Datei;
`search` ordnet Befehle so, wie es die Befehlspalette tut.
```
$ osatura ui commands
Edit
  edit.undo                    Undo                               global:Ctrl+Z
  edit.redo                    Redo                               global:Ctrl+Shift+Z  global:Ctrl+Y
$ osatura ui layout presets
animate  V0.62(H0.74([*stage] | [*layers selection bones live face json]) | [*timeline])  [valid]
rig      H0.66(V0.84([*stage] | [*timeline (folded)]) | V0.50([*layers bones] | [*selection json]))  hidden: live,face  [valid]
compact  V0.62([*stage] | [*timeline layers selection bones live face json])  [valid]
live     H0.58(V0.70([*stage] | [*timeline]) | [*live face bones selection])  hidden: layers,json  [valid]
$ osatura ui layout validate demo.json
demo.json: unknown field `format`, expected one of `version`, `root`, `floating`, `hidden` at line 2 column 10   [exit 1]
```

## 17. Pose-Datenbank, Vorschläge und weitere Helfer

Diese Befehle bauen eine **Pose-Datenbank** auf (Knochenrotationen an den Key-Frames eines Projekts und jedes Takes einer Bibliothek)
und nutzen deren gelernte Struktur (PCA-Achsen, optional ein kleines Netz), um Posen vorzuschlagen. Es ist ein statistisches Hilfsmittel, trainiert auf
*deinen* Posen, kein allgemeines Modell. Ihre Modell- und Berichtsdateien werden mit einem einfachen Schreibvorgang geschrieben.

### `pose-db` (build, info, train-mlp)

**Synopsis:**
```
osatura pose-db build <out.json> [--project P.json] [--library takes.json] [--step N]
osatura pose-db info  <db.json>
osatura pose-db train-mlp <db.json> <out.json|out.bin> [--hidden 64,64] [--epochs N] [--lr R] [--seed S]
```
```
$ osatura pose-db build db.json --project demo.json
wrote db.json: 5 poses of 3 bones in 1 sequence(s)
$ osatura pose-db info db.json
5 poses, 3 bones: arm_up, arm_low, hand
  sequence 0: project (project, 5 poses)
PCA: 2 axes keep 99.2% of the variance (total 950.0 deg^2)
  axis 1: std 27.34 deg, cumulative 78.7%
  axis 2: std 13.95 deg, cumulative 99.2%
typical nearest-pose spacing 14.43 deg, typical step between consecutive poses 35.82 deg
$ osatura pose-db train-mlp db.json mlp.json --hidden 8 --epochs 20 --seed 1
wrote mlp.json: 6 -> [8] -> 3 (epochs 20, loss 1.0189 -> 0.4921)
trained on the library itself (no held-out data here); run pose-bench for held-out accuracy
```
`--project` fügt die Posen an den Key-Frames des Projekts hinzu, `--library` die Takes einer Take-Bibliothek (`take-record`, Abschnitt 6),
`--step N` tastet zusätzlich jeden N-ten Frame ab. Das kleine Netz (einfaches SGD) ist optional; verwende es mit `--model` weiter unten.

### `pose-suggest` (between, complete, next)

**Synopsis:**
```
osatura pose-suggest between  <project.json> <db.json|-> --a F1 --b F2 [--t 0.5] [--snap 0.5] [--top 5] [--apply RANK --out out.json] [--json]
osatura pose-suggest complete <project.json> <db.json|-> --frame F [--fix bone=angle,...] [--reach bone=x,y,...] [--free b1,b2] [--prior 1] [--model m.json] [--apply RANK --out out.json]
osatura pose-suggest next     <project.json> <db.json|-> --frame F [--k 8]
```
`-` statt einer Datenbank lernt aus dem Projekt selbst. `between` schlägt eine Zwischenpose zweier Frames vor, die auf der
gelernten Mannigfaltigkeit bleibt (Kandidaten `linear`, `pca`, `manifold`); `complete` findet die plausibelste vollständige Pose bei vorgegebenen festen
Winkeln (`--fix`), Spitzenzielen (`--reach`) und freien Knochen; `next` zeigt, was die Bibliothek nach Posen wie der am
Frame getan hat. `--apply RANK` (mit `--out`) schreibt den gewählten Kandidaten als Keys.
```
$ osatura pose-suggest between demo.json db.json --a 0 --b 24 --t 0.5 --top 3
between: 5 library poses, 2 PCA axes (99% of the variance)
 1. linear      score 1.00  plausibility 1.00  key at frame 12.0
      linear blend of the angles
      arm_up=0.0 arm_low=-7.5 hand=-10.0
 3. manifold    score 1.00  plausibility 1.00  key at frame 12.0
      PCA blend pulled toward the library
$ osatura pose-suggest complete demo.json db.json --frame 12 --fix arm_up=-40
 1. pca-ls      score 0.96  plausibility 1.00  constraint error 0.18  key at frame 12.0
      regularised least squares in PCA space (start 1)
      arm_up=-39.8 arm_low=24.9 hand=11.2
 2. library-nn  score 0.25  plausibility 1.00  constraint error 15.00  key at frame 12.0
(scores rank candidates against this library; they are heuristics, not probabilities)
$ osatura pose-suggest next demo.json db.json --frame 12 --k 3
 1. successor   score 0.60  plausibility 1.00  key at frame 24.0
      what followed pose 1 of 'project'
```

### `pose-bench`

**Synopsis:** `osatura pose-bench`

Gibt die Genauigkeit der Vorschlagsmethoden auf zurückgehaltenen Daten einer **synthetischen gangartigen Bibliothek** aus (24 Trainingszyklen, 6
zurückgehaltene; die Vervollständigung sieht 3 von 10 Knochen; außerdem wird das kleine Netz trainiert). Niedriger ist besser. Es liest kein Projekt. Es
ist **in einem Debug-Build langsam** (Minuten); Zahlen aus einem Release-Build:
```
completion  mean pose                                   20.96 deg RMS
completion  nearest library pose                         1.77 deg RMS
completion  PCA least squares                            0.51 deg RMS
completion  MLP                                          1.64 deg RMS
between     linear blend                                 1.83 deg RMS
next        Markov mean                                  0.72 deg RMS
```

### `pose-match`

**Synopsis:** `osatura pose-match <project.json> <params.json> <goals.json> <out.json> [--generations N --seed N]`

Sucht (ein evolutionärer Optimierer, Standard 80 Generationen, Seed 1, gültiger Bereich 1..100000) nach Werten für eine Menge von
Parametern, die Knochenspitzen an Zielpositionen im Bühnenraum bringen. `params.json` ist eine Liste von
`{"target","prop","min","max","start"}`; `goals.json` ist eine Liste von `{"bone","at":[x,y],"weight"}`. Das Ergebnis wird als
Kanäle angehängt; der Verlust ist die gewichtete Summe der quadrierten Spitzenfehler in px².
```
$ echo '[{"target":"arm_up","prop":"bone_angle","min":-60,"max":40,"start":0},{"target":"arm_low","prop":"bone_angle","min":-60,"max":60,"start":0}]' > params.json
$ echo '[{"bone":"hand","at":[260,200],"weight":1.0}]' > goals.json
$ osatura pose-match demo.json params.json goals.json out.json --generations 30 --seed 3
wrote out.json (loss 3756.156 px²)
```
**Stolperfalle:** Der Verlust im Beispiel ist groß, weil die Bereiche das Ziel nicht erreichen können: Prüfe die Zahl, nicht nur den
Exit-Code (ein Ergebnis wird auch bei schlechter Übereinstimmung geschrieben).
## Bekannte Abweichungen zwischen Dokumentation und Verhalten

Wo der Hilfetext des Programms selbst (den `osatura` ohne Argumente ausgibt) oder ein benachbartes Dokument das eine sagt
und das Programm etwas anderes tut, beschreiben die Abschnitte oben, was das Programm tatsächlich tut. Die Abweichungen, die beim Überprüfen
jedes Beispiels gefunden wurden, in der Reihenfolge, wie wahrscheinlich sie dich treffen:

1. **Behoben - unbekannte Flags.** Jeder Befehl mit einer Usage-Zeile lehnt jetzt undokumentierte `--words` ab (mit einem „meintest du“-Hinweis),
   darunter `check`, `stats`, `lint`, `coach`, `contracts` und `machine list`. `--json`, `--quiet` und `--allow-empty` gelten
   pro Befehl und werden anderswo abgelehnt (eine inkompatible Änderung für Skripte, die sie überall übergeben haben). Seit Welle 5 gilt die Prüfung **pro Unterbefehl** (`ci diff ... --quiet`, `trail <bone> ... --json`, `ui layout presets --json`, `golden write --tol` werden abgelehnt) und umfasst auch Optionen mit Wert. Tests: `osatura-cli/tests/doc_mismatch.rs` (Akzeptanzmatrix) und `options_are_validated_per_sub_command` in `main.rs`.
2. **Behoben - Ausgaben mit einfachem `fs::write`.** `ci run`, `export-fit`, `lod`, `fuzz`, `fuzz-grid`, `face-*`, Pose-Dateien, Snapshots,
   `vcs` und das Zusammenarbeits-Log verwenden den atomaren, Symlinks ablehnenden Writer; ffmpeg-Ausgaben werden vorab geprüft. Ein tabellengesteuerter Test
   deckt etwa 24 Befehle ab. Nicht durchgängig abgedeckt: `face-train`/`face-eval` (Unit-Test nur für ihren Writer).
3. **Behoben - `golden check`** ist standardmäßig exakt; `--tol` lockert die Prüfung; es gibt die Anzahl der Abweichungen und den größten Unterschied aus.
   Die Behauptung „Änderung der Hintergrundfarbe besteht“ war ein Demo-Artefakt (die Himmel-Ebene deckt den Hintergrund ab), kein Fehler.
4. **Behoben - Exit-Code von `fuzz`.** Standard ist nur Bericht (Exit 0); `--gate` oder `--max-fail-rate` schaltet ein Gate. Siehe `fuzz`.
5. **Behoben - `dof-export`** schreibt `.mp4` und `.webm` (über ffmpeg, mit `--codec`, ohne Audio); der Hilfetext listet sie jetzt auf. Ein Name
   mit einer anderen Endung wird abgelehnt, und es wird nichts geschrieben.
6. **Behoben - `export` `.png`** schreibt `<name>.atlas.json`, nie `<name>.json`; die Fehlermeldung für unbekannte Formate listet jede Endung auf.
7. **Standardwerte, die der Hilfetext nicht nennt:** `studio` lauscht auf `127.0.0.1:8080`; `fuzz-grid --res` ist 32; `export-fit`
   tastet 8 Frames ab, sucht mit `descent` und bricht nach 60 s ab (`--max-seconds 0` = unbegrenzt); `reduced-motion --factor`
   ist 0,25; `size-report --tol` ist 0,05; `pose-match` läuft mit 80 Generationen und Seed 1; `lod --out-dir` ist der aktuelle
   Ordner.
8. **`machine remove` kann ablehnen** (`binding 'goBind' still drives machine 'm'`); der Hilfetext erwähnt die
   Abhängigkeitsprüfung nicht. `machine` und `state-bake` lesen zwei verschiedene Automatenformate (`layers`/`inputs` gegenüber
   `initial`/`states`); die Usage-Zeile von `state-bake` hat keine Beschreibung und nennt kein Format.
9. **Behoben - `stats`** akzeptiert `--json` nicht mehr (die Ausgabe ist immer JSON; die Usage-Zeile sagt das).
10. **`vcs`:** Der Standard-Branch ist `main` (`checkout master` schlägt mit `unknown revision 'master'` fehl); `vcs resolve`
    nimmt den Konfliktpfad *mit* führendem Schrägstrich, genau wie `merge` ihn ausgibt (`/background`), `background` allein ergibt
    `no conflict at 'background'`; der Hilfetext schreibt „see: osatura vcs“, aber `osatura vcs` ohne Argumente gibt eine
    einzeilige Usage aus und endet mit Exit 1.
11. **Usage-Zeilen ohne Beschreibung.** `state-bake`, `pose-match`, `pose-bench`, `follow`, `keys`, `event`, `marker`,
    `idle`, `mouth`, `beats`, `export`, `frames`, `onion` und weitere haben eine Synopsis, aber keine Erklärung, oder eine Beschreibung, die
    in einer Folgezeile steht; ihr Verhalten in diesem Dokument stammt aus dem Ausführen.
12. **Benennung von Eigenschaften.** Meldungen und `list`-Ausgaben geben Eigenschaften in CamelCase aus (`LayerOpacity`, `BoneAngle`),
    aber jede Option, Mapping-Datei und Zusatzdatei erwartet die snake_case-Namen (`layer_opacity`, `bone_angle`); die CamelCase-Form
    wird abgelehnt (`bind add ... sun:LayerOpacity` schlägt mit `unknown property 'LayerOpacity'` fehl).
13. **`cloth`:** Der Hilfetext spricht von „per-vertex deformation“; das gebackene Ergebnis wird als `lattice_dx` / `lattice_dy`-Kanäle
    an den Gitterpunkten des Tuchs gespeichert (beim Standardraster 7x7 sind das die 42 nicht angehefteten Punkte), siehe Abschnitt 7.
14. **`render-gpu`** existiert nur in Builds, die mit `--features gpu` erstellt wurden; im Test-Build ließ es sich nicht ausführen, daher
    stammen seine Optionen aus dem Hilfetext und sind hier nicht verifiziert.
15. **Langsam in Debug-Builds.** `pose-bench`, `face-train`, `ci run` und `export-fit` brauchen in einem nicht optimierten Build Minuten.
    Die Zeitangaben in diesem Dokument stammen aus der Debug-Binärdatei und sind in einem Release-Build deutlich kürzer.
16. **Behoben (Welle 5) - `golden write --scale`** über 1 wurde früher stillschweigend auf 1 begrenzt; jetzt wird es abgelehnt (die Skalierung liegt in (0, 1]).
    `golden check` ignorierte ein `--scale` und verwendete das gespeicherte; jetzt lehnt es die Option ab.
17. **Behoben (Welle 5) - Synopsen.** Jede Option aus dem Hilfetext eines Befehls steht hier in seiner Synopsis und umgekehrt (Test
    `cli_reference_synopses_match_usage`), und jede Usage-Zeile ohne Alternativen wird hier wortwörtlich wiederholt (Test
    `cli_reference_repeats_usage_lines_verbatim`). `render-gpu --features gpu` und `layer-normal ... relight --layer-normals`
    in den Beschreibungen des Hilfetexts nennen ein Build-Feature und die Option eines anderen Befehls, keine Optionen dieser Befehle.
18. **Andere Dokumente sind an zwei Stellen älter als das Programm** (siehe [EXPORT_FORMATS.md](EXPORT_FORMATS.md) Abschnitt 17):
    `docs/LIMITATIONS.md` L-29 sagt, der Lottie-Import nehme keine Bilder, Effekte, Precompositions oder Mattes entgegen (der Importer verarbeitet
    sie jetzt, mit den in Abschnitt 9 aufgeführten Warnungen), und L-32 führt OCA unter den Importformaten auf, während die CLI nur
    `export-oca` hat.
## Alphabetischer Befehlsindex

Jeder Befehl mit einer einzeiligen Zusammenfassung und dem Abschnitt, der ihn dokumentiert. Unterbefehle (`machine add`, `vcs commit`, ...) stehen im Abschnitt ihres übergeordneten Befehls.

| Befehl | Zusammenfassung | Abschnitt |
|---|---|---|
| `--version` | gibt Version, Lizenz, Quellcode-Angebot und Support-Links aus | [1](#1-konventionen-für-alle-befehle) |
| `align` | richtet Ebenen aneinander oder an der Bühne aus | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `audio-react` | Lautstärke steuert eine Eigenschaft | [8](#8-audio-lippensynchronisation-und-sounds) |
| `autorig` | IK, Atmen und Blinzeln, eingerichtet anhand von Knochen- und Ebenennamen | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `autotrace` | lokale Bitmap-Vektorisierung (Farbbänder, Bézier-Anpassung) | [9](#9-import) |
| `autotrace-bench` | Genauigkeitstabelle des Vektorisierers auf synthetischen Formen | [9](#9-import) |
| `beats` | erkennt Lautstärkesprünge und zeichnet Marker oder Events auf | [8](#8-audio-lippensynchronisation-und-sounds) |
| `bind` | Datenbindungen: add, list, remove | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `bind-eval` | wertet Bindungen für gegebene Werte aus | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `bone-vis` | zeigt einen Knochen nur für einen Frame-Bereich in Overlays | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `cage` | fügt einen Cage-Deformer mit Mean-Value-Koordinaten hinzu | [7](#7-physik-cloth-und-deformer) |
| `cage-pose` | setzt Keys für Cage-Vertex-Versätze an einem Frame | [7](#7-physik-cloth-und-deformer) |
| `camera-seq` | animiert die Kamera per Keys durch gespeicherte Shots | [6](#6-animation-keys-timeline-und-generatoren) |
| `chan-meta` | Kanalfarbe, Shy-, Solo- und Sperr-Flags | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `check` | validiert ein Projekt (Exit 1, wenn ungültig) | [3](#3-projekte-erstellen-und-untersuchen) |
| `ci` | Animation-CI: Referenzbilder, Wahrnehmungs-Diff, strukturelle Prüfungen | [14](#14-qualitätssicherung) |
| `clip` | legt einen Take auf den Clip-Stapel | [6](#6-animation-keys-timeline-und-generatoren) |
| `cloth` | backt eine deterministische Tuchsimulation | [7](#7-physik-cloth-und-deformer) |
| `coach` | Vorschläge zu Timing und Prinzipien | [14](#14-qualitätssicherung) |
| `collab-sim` | Konvergenztest der Zusammenarbeit | [16](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry) |
| `collab-token` | erzeugt ein Zusammenarbeits-Token pro Teilnehmer (HMAC-SHA-256, läuft ab) | [16](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry) |
| `color-space` | wählt sRGB- oder linear-light-Komposition für das Projekt | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `comp` | benannte Ebenenzustände (Sichtbarkeit, Deckkraft, Switch-Auswahl) | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `compare` | Vorher-/Nachher-/Differenzbild zweier Projekte | [4](#4-rendern-und-exportieren) |
| `container` | Layout-Container: add, list, remove, solve | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `contracts` | Bewegungsverträge (Unit-Tests für Animation) | [14](#14-qualitätssicherung) |
| `demo` | schreibt das eingebaute Rig-Demoprojekt | [3](#3-projekte-erstellen-und-untersuchen) |
| `demo-fx` | schreibt die Demo mit Flüssigkeit / Boolean / Partikeln | [3](#3-projekte-erstellen-und-untersuchen) |
| `diff` | semantische Unterschiede zwischen zwei Projekten | [3](#3-projekte-erstellen-und-untersuchen) |
| `distribute` | verteilt drei oder mehr Ebenen gleichmäßig | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `dof` | Tiefenunschärfe-Rendering eines Frames | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `dof-export` | animierte Tiefenunschärfe (GIF/APNG/Video/Ordner) | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `drive` | CSV-Signale -> Keys über eine Mapping-Datei | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `event` | fügt ein benanntes Laufzeit-Event hinzu (mit optionalem JSON-Payload) | [6](#6-animation-keys-timeline-und-generatoren) |
| `explain` | warum eine Eigenschaft ihren Wert hat: geordnete Beiträge | [3](#3-projekte-erstellen-und-untersuchen) |
| `export` | exportiert GIF, APNG, WebM/MP4, PNG-Sheet, HTML, Lottie, .lottie, glTF/GLB, ... | [4](#4-rendern-und-exportieren) |
| `export-fit` | Export bester Qualität, der in ein Byte-Budget passt | [13](#13-level-of-detail-größenberichte-und-export-fit) |
| `export-oca` | schreibt einen Open-Cel-Animation-Ordner | [4](#4-rendern-und-exportieren) |
| `eye-clip` | maskiert Iris-/Pupillen-Ebenen mit der Augenform | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `face-bake` | Landmarken -> Keys | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-calibrate` | Kalibrierung des neutralen Gesichts | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-detect` | Gesichtsrahmen und 23 Landmarken in einem Bild | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-eval` | Genauigkeitsbericht des Detektors auf zurückgehaltenen Daten | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-synth` | synthetische Kopfanimation | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-track` | Landmarken-NDJSON aus Video oder Frames | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `face-train` | trainiert den Gesichtsdetektor auf synthetischen Gesichtern | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `follow` | Generatoren für Overlapping Action / Follow-Through | [6](#6-animation-keys-timeline-und-generatoren) |
| `foot-lock` | fixiert einen Controller während des Bodenkontakts | [6](#6-animation-keys-timeline-und-generatoren) |
| `frames` | rendert jeden Frame in einen Ordner (SVG, oder PNG mit --png) | [4](#4-rendern-und-exportieren) |
| `fuzz` | findet Posen, bei denen das Rig bricht | [14](#14-qualitätssicherung) |
| `fuzz-grid` | Schweregrad-Karte zweier Parameter | [14](#14-qualitätssicherung) |
| `fx` | rendert einen Frame mit einer Post-Effekt-Kette | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `glue` | verklebt Mesh-Vertices miteinander | [7](#7-physik-cloth-und-deformer) |
| `golden` | schreibt/prüft Regressions-Frames | [14](#14-qualitätssicherung) |
| `idle` | erzeugt einen prozeduralen Idle-Kanal | [6](#6-animation-keys-timeline-und-generatoren) |
| `import-anim` | animiertes GIF/APNG -> Frame-für-Frame-Ebene | [9](#9-import) |
| `import-bvh` | BVH-Motion-Capture -> Kanäle | [9](#9-import) |
| `import-lottie` | Lottie-Teilmenge -> Projekt | [9](#9-import) |
| `import-ora` | OpenRaster -> Projekt | [9](#9-import) |
| `import-seq` | PNG-Sequenz -> Frame-für-Frame-Ebene | [9](#9-import) |
| `import-svg` | SVG-Teilmenge -> Vektor-Ebene | [9](#9-import) |
| `keyedit` | Dope-Sheet-Bearbeitungen als ein Undo-sicherer Stapel | [6](#6-animation-keys-timeline-und-generatoren) |
| `keys` | Massenoperationen auf Keys über alle Key-Kanäle | [6](#6-animation-keys-timeline-und-generatoren) |
| `layer-normal` | hängt eine Normal-Map an eine Ebene an | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `layer-pivot` | verschiebt den Pivot einer Ebene, ohne das Bild zu ändern | [7](#7-physik-cloth-und-deformer) |
| `layout-preview` | rendert mit neu berechnetem Layout für eine andere Canvas-Größe | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `lint` | statische Rig-/Animationsprüfungen (Exit 1 bei Fehlern) | [14](#14-qualitätssicherung) |
| `lipsync` | steuert eine Mund-Switch-Ebene aus der Audio-Lautstärke (nicht phonembasiert) | [8](#8-audio-lippensynchronisation-und-sounds) |
| `lipsync-text` | gleicht ein Transkript mit dem Audio ab und leitet Mundformen ab | [8](#8-audio-lippensynchronisation-und-sounds) |
| `live-map` | aufgezeichnete Live-Eingabe (MIDI, Maus, Gamepad, Tasten) -> Keys, Takes, Events | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `lod` | Detailstufen mit deklarierten Pixel-Fehlerschranken | [13](#13-level-of-detail-größenberichte-und-export-fit) |
| `lod-pick` | gröbste Stufe, die auf einem Bildschirm in ein Pixelbudget passt | [13](#13-level-of-detail-größenberichte-und-export-fit) |
| `loop-solve` | lässt Key-Kanäle nahtlos loopen | [6](#6-animation-keys-timeline-und-generatoren) |
| `machine` | Laufzeit-Zustandsautomaten: add, list, remove, validate | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `machine-sim` | spielt zeitlich festgelegte Eingaben ab: Zustandsprotokoll plus SVG-Frames | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `magnet` | backt einen Annäherungszug | [6](#6-animation-keys-timeline-und-generatoren) |
| `make-loop` | lässt einen Zyklus nahtlos loopen (optional an Ort und Stelle) | [6](#6-animation-keys-timeline-und-generatoren) |
| `marker` | fügt einen Timeline-Marker oder -Bereich hinzu | [6](#6-animation-keys-timeline-und-generatoren) |
| `mcp` | Agent-Server (JSON-RPC über stdio) | [16](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry) |
| `mesh-lock` | Follower folgen der Oberfläche eines Host-Meshes | [7](#7-physik-cloth-und-deformer) |
| `mesh-trace` | vektorisiert die Alpha-Silhouette eines Bildes in ein geskinntes Dreiecks-Mesh | [7](#7-physik-cloth-und-deformer) |
| `mirror` | spiegelt alle Key-Kanäle links<->rechts | [6](#6-animation-keys-timeline-und-generatoren) |
| `mocap-info` | Hierarchie und Ausdehnung einer BVH-Datei | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `mocap-match` | Zuordnung BVH-Gelenk zu Rig-Knochen | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `mocap-retarget` | backt Mocap als angehängte Kanäle | [10](#10-live-eingabe-midi-gesichts-tracking-und-motion-capture) |
| `mouth` | importiert Mund-Cues (Papagayo/Rhubarb) | [8](#8-audio-lippensynchronisation-und-sounds) |
| `onion` | Onion-Skin-PNG oder -SVG | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `parallax` | Multiplane-Tiefe für Ebenen der obersten Ebene | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `parent-switch` | backt animiertes Umhängen von Eltern ohne Sprung | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `physics` | backt eine Kettensimulation | [7](#7-physik-cloth-und-deformer) |
| `pick` | was unter einem Canvas-Punkt liegt (oberstes zuerst) | [3](#3-projekte-erstellen-und-untersuchen) |
| `pose-bench` | Genauigkeit der Pose-Methoden auf zurückgehaltenen Daten | [17](#17-pose-datenbank-vorschläge-und-weitere-helfer) |
| `pose-db` | Pose-Datenbank: build, info, train-mlp | [17](#17-pose-datenbank-vorschläge-und-weitere-helfer) |
| `pose-match` | passt Parameter an, damit Knochenspitzen Ziele erreichen | [17](#17-pose-datenbank-vorschläge-und-weitere-helfer) |
| `pose-suggest` | Vorschläge für Zwischenposen, Vervollständigung und nächste Pose | [17](#17-pose-datenbank-vorschläge-und-weitere-helfer) |
| `proxy-bench` | Millisekunden pro Frame bei jeder Proxy-Stufe | [4](#4-rendern-und-exportieren) |
| `reduced-motion` | schreibt eine ruhigere Kopie eines Projekts | [14](#14-qualitätssicherung) |
| `reference` | markiert eine Ebene als Durchzeichen-Referenz (wird nie exportiert) | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `render` | rendert einen Frame als PNG oder SVG (Scrub-Proxys, Knochen, Skalierung) | [4](#4-rendern-und-exportieren) |
| `render-gpu` | GPU-Rendering (nur wgpu-Builds) | [4](#4-rendern-und-exportieren) |
| `render-hdr` | Float-Rendering nach OpenEXR oder 16-Bit-PNG | [4](#4-rendern-und-exportieren) |
| `reparent` | ändert den Eltern-Knochen eines Knochens und behält die Ruhepose | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `retarget` | kopiert Animation zwischen Rigs über Kettenpaare | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `safety` | Bericht zu Blitzen / roten Blitzen / Kamerabewegung | [14](#14-qualitätssicherung) |
| `script` | führt ein Rhai-Skript in der Sandbox aus, das das Projekt bearbeitet | [6](#6-animation-keys-timeline-und-generatoren) |
| `scroll` | Scroll-Bereiche: add, list, remove, eval | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `select-box` | Auswahl per Aufziehrahmen | [3](#3-projekte-erstellen-und-untersuchen) |
| `shot` | speichert einen Kamera-Shot | [6](#6-animation-keys-timeline-und-generatoren) |
| `size-report` | wo die Bytes eines Projekts/Exports liegen | [13](#13-level-of-detail-größenberichte-und-export-fit) |
| `skin` | benannte Skins: Ebenensätze, Skin-eigene Knochen, Constraints, IK-Ketten | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `skin-diffuse` | Skin-Gewichte per Wärmediffusion | [7](#7-physik-cloth-und-deformer) |
| `slider` | erzeugt einen Controller-Knochen, der gespeicherte Posen überblendet | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `snapshot` | nummerierte Revisionshistorie eines Projekts in einem Ordner (save, list, restore) | [3](#3-projekte-erstellen-und-untersuchen) |
| `soft-scale` | proportionales Skalieren einer Auswahl | [7](#7-physik-cloth-und-deformer) |
| `sound` | Sound-Spuren des Projekts: list, add, edit, remove, mix | [8](#8-audio-lippensynchronisation-und-sounds) |
| `speech-motion` | additive Nicker und Augenbrauen-Heber, getaktet auf ausgerichtete Wörter | [8](#8-audio-lippensynchronisation-und-sounds) |
| `spring-fit` | passt Federfrequenz/-dämpfung an eine Kurve an | [6](#6-animation-keys-timeline-und-generatoren) |
| `state-bake` | backt einen Frame-basierten Zustandsautomaten in Keys | [11](#11-zustandsautomaten-datenbindungen-container-und-scroll-bereiche) |
| `stats` | Anzahl von Knochen, Ebenen, Kanälen, Keys (JSON) | [3](#3-projekte-erstellen-und-untersuchen) |
| `stretch` | backt einen Follower, der sich zum Ziel streckt | [6](#6-animation-keys-timeline-und-generatoren) |
| `studio` | Browser-Editor (lokaler Webserver) | [16](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry) |
| `tag` | schlägt Standard-Rollennamen für Knochen und Ebenen vor | [3](#3-projekte-erstellen-und-untersuchen) |
| `take-apply` | wendet einen Take aus der Bibliothek mit Überblendungen an | [6](#6-animation-keys-timeline-und-generatoren) |
| `take-list` | listet die Takes einer Bibliothek auf | [6](#6-animation-keys-timeline-und-generatoren) |
| `take-record` | zeichnet einen Take in eine Bibliothek auf | [6](#6-animation-keys-timeline-und-generatoren) |
| `timecode` | Frame <-> SMPTE-Timecode (Drop-Frame-fähig) | [3](#3-projekte-erstellen-und-untersuchen) |
| `timeline` | Spuren pro Kanal mit Keys, Sperren und Farben | [3](#3-projekte-erstellen-und-untersuchen) |
| `trace` | vektorisiert den Alphakanal eines PNG zu einem Vektorpfad | [9](#9-import) |
| `trail` | Bewegungspfad eines Knochens, einer Ebene oder eines Punkts | [12](#12-effekte-tiefenunschärfe-onion-skin-trails-und-kamera) |
| `trim` | backt einen Trim-Pfad in eine Vektor-Ebene | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `ui` | Befehls-Registry, Tastenkürzel, Layouts und Suche des Studios | [16](#16-studio-zusammenarbeit-mcp-agent-server-und-ui-registry) |
| `vcs` | Versionsverwaltung mit semantischem Merge für eine Projektdatei | [15](#15-versionsverwaltung-vcs) |
| `view-swap` | steuert eine Switch-Ebene aus einer Knocheneigenschaft (Stufenfunktion) | [5](#5-rigging-skins-ik-und-layout-helfer) |
| `walk` | erzeugt einen Laufzyklus | [6](#6-animation-keys-timeline-und-generatoren) |
| `warp-pivot` | dreht/skaliert/verschiebt eine Auswahl von Kontrollpunkten | [7](#7-physik-cloth-und-deformer) |
| `waveform` | Min-/Max-Spitzen zum Zeichnen | [8](#8-audio-lippensynchronisation-und-sounds) |
