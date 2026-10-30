# OSATURA-Animationsleitfaden

Wie du Dinge in Bewegung bringst, Bewegung wiederverwendest, sie aus Audio, Gesichtern, Mocap und Live-Eingaben ansteuerst, prüfst und
klein auslieferst. Jeder Befehl hier wurde mit dem echten Binary ausgeführt (`osatura 0.1.0`, Debug-Build, Linux);
die Ausgaben stammen aus diesen Läufen. Rigging steht in [RIGGING_GUIDE.md](RIGGING_GUIDE.md); das erste Projekt in
[GETTING_STARTED.md](GETTING_STARTED.md); jedes Flag in [CLI_REFERENCE.md](CLI_REFERENCE.md).

Lizenz dieses Dokuments: CC-BY-4.0.

Die Beispiele verwenden den Biped mit 11 Knochen (`biped.json`, 400x360, 24 fps, 24 Frames) aus dem Rigging-Leitfaden: einzelne
Rechtecke und Kreise, die an Knochen befestigt sind, und eine `mouth`-Switch-Ebene mit fünf Kindern, aufgebaut mit einem einzigen `script`-
Aufruf. Jedes Rig mit ähnlich benannten Knochen funktioniert.

![Sechs Frames (0, 4, 8, 12, 16, 20) des generierten Laufzyklus eines einfachen Bipeds aus Rechtecken: Die Beine schwingen gegenläufig, die Arme schwingen entgegengesetzt, die Hüfte wippt.](../../img/walk-cycle.png)

## 1. Kanäle und Keys

Alles, was sich bewegt, ist ein **Kanal**: ein Ziel (Knochen, Ebene, Kamera, Deformer-Punkt), eine Eigenschaft, ein
Modus und eine Quelle.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Quellen:** `keys` (Keyframes), `expr` (ein Ausdruck), `noise` (fraktales Value-Noise in `[-amp, amp]`, `freq` in
  Zyklen pro Sekunde) und `driven` (eine Kurve, die den Wert einer anderen Eigenschaft auf diese abbildet). Befehle, die backen
  (Physik, Walk, Clips, Lip-Sync...), schreiben gewöhnliche `keys`. `osatura timeline` zeigt die Quelle jedes Tracks:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)` oder die Liste der Key-Frames.
* **Modi:** `replace`, `add` (auf das, was darunter liegt) und `multiply`. Kanäle werden in Listenreihenfolge kombiniert.
* **Extrapolate** (was ein Track außerhalb seines ersten und letzten Keys tut): `hold` (Standard), `linear` (die Steigung am Ende
  fortsetzen), `loop`, `ping_pong` und `loop_offset` (wiederholen und dabei in jedem Zyklus die Änderung von erstem zu letztem Key addieren, was zu einem
  laufenden x-Offset passt). Setze es mit `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; ein falscher Name listet die gültigen auf.
* Knocheneigenschaften sind **Deltas zur Ruhepose**: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. Zu den Ebeneneigenschaften gehören `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` und `layer_switch`. Deformer verwenden `cage_dx`... wie im Rigging-Leitfaden.

Erzeuge Keys aus einem Skript (`set_key(target, prop, frame, value[, ease])`), über die CLI mit `keyedit add`, im
Studio oder in JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` listet jeden Track nach Knochen/Ebene mit seinen Key-Frames auf; `osatura explain <project> <bone|layer> <frame>`
zeigt, wie ein Frame berechnet wurde.

## 2. Interpolation und Easing

Jeder Key speichert das Easing des Segments, das an diesem Key **beginnt**. Die Easing-Namen, die keine Parameter brauchen, sind
`linear`, `step` (den Wert bis zum nächsten Key halten), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (ein Catmull-Rom-
Spline durch benachbarte Keys: weich, ohne Overshoot-Kontrolle) und `bounce`. In JSON ist ein Easing ein Objekt:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` akzeptiert einen einfachen Namen, `bezier:x1,y1,x2,y2` (gespeichert als `cubic_bezier`) oder ein JSON-Objekt für
`spring` und `back`; alle vier Formen wurden ohne Fehler auf die 17 Hüft-Keys des Laufzyklus angewendet. In der Dope-Sheet-CLI ist die Auswahl-
syntax `SEL` entweder `target:prop@first..last`, `target:prop` oder `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Gib `-` als Ausgabe an, um jedes `keyedit` zu einem **Trockenlauf** zu machen, der den Plan als JSON ausgibt, ohne etwas zu schreiben:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Stolperfalle.** `keyedit` lehnt Änderungen ab, die den Frame-Bereich des Projekts verlassen würden, außer du gibst `--free` an:
`scale: frame 27 is outside the project range 0..=24` (derselbe Befehl mit `--free` gab `scale: 17 keys changed` aus, und die
zusätzlichen Keys liegen dann hinter dem letzten Frame, wo `lint` `L007` meldet). Gesperrte Kanäle werden ebenfalls abgelehnt:
`channel hand_ctrl.BoneOffsetY is locked` (siehe `chan-meta ... lock=on`).

Der Kurveneditor des Studios zeichnet dieselben Kurven, mit ziehbaren Tangenten (`handle`, `smooth`; siehe
[STUDIO.md](STUDIO.md)). Keys haben optionale Ein-/Aus-Tangenten-Handles (`free`, `aligned` oder `auto`).

## 3. Das Dope Sheet auf der Kommandozeile: `keyedit`

Alles, was die Timeline des Studios kann, gibt es als `keyedit`-Unterbefehl, angewendet als ein undo-sicherer Batch:

| Unterbefehl | Wirkung |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | Keys in Zeit und/oder Wert verschieben |
| `scale --sel SEL --pivot F --factor X` | um einen Frame herum neu timen |
| `dup --sel SEL --offset N` | duplizieren |
| `delete --sel SEL` | löschen |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | Easing, Wert oder automatische Glättung setzen |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | Zwischenablage über Projekte hinweg |
| `add --target T --prop P --at F [--value V] [--ease E]` | einen Key einfügen |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | Zeit einfügen oder entfernen |
| `offset --channels 0,1 --dt N` | ganze Kanäle verschieben |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | nach dem letzten Key loopen |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | Tangenten-Handle im Kurveneditor |
| `lock --target T [--prop P] [--value on\|off]` | einen Kanal sperren |

## 4. Loops

Ein Projekt mit N Frames spielt die Frames 0 bis N-1 wiederholt ab, also **muss Frame N dieselbe Pose wie Frame 0 sein und wird nie zweimal
gezeigt**. Drei Befehle helfen:

* `keyedit ... loop --channels 0,1` setzt `extrapolate: loop` auf Kanälen (die Keys bleiben unberührt):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` macht gekeyte Kanäle nahtlos loopfähig und meldet die Naht:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **entfernt die Fortbewegung**, sodass die Bewegung an Ort und Stelle funktioniert:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Dieser Ball bewegte sich in 23 Frames um 280 Einheiten nach rechts. `make-loop` machte daraus einen perfekten Loop an Ort und Stelle ... eines Balls, der sich nicht mehr
bewegt (lint meldet dann `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
Das bedeutet „entfernt die Fortbewegung“: Verwende es für Laufzyklen, die an Ort und Stelle laufen, und bewege dann den ganzen Charakter mit einem
`follow`-Pfad oder einer Kamera (Abschnitt 9). `loop-solve` behebt keine Fortbewegung: Beim selben Ball meldet es `seam 280.000 -> 280.000`.
Soll die Fortbewegung über die Zyklen hinweg **weitergehen**, gib dem Kanal die Extrapolation `loop_offset` (Abschnitt 1): `explain` meldet dann
`keys → 121.739 (2 key(s), LoopOffset outside range)` für den Ball bei Frame 10.

`--inclusive` wählt die ältere Konvention, bei der der letzte gezeigte Frame dem ersten entspricht.

**Stolperfalle: Noise loopt nicht.** Ein `noise`-Kanal ist nicht periodisch. Verwende für ein loopendes Wackeln einen Ausdruck in `t`
(0..1 über den Clip) wie `8 * sin(t * 2 * pi)`, oder brich den Loop bewusst.

## 5. Ausdrücke, Noise und Driver

Ein **Ausdrucks**-Kanal wertet in jedem Frame eine kleine Formel aus. Variablen: `frame`, `time` (Sekunden), `fps`, `frames`,
`pi` und **`t` = 0..1 über den Clip**. Operatoren `+ - * / % ^`, Vergleiche, `&& || !`, `?:`. Zu den Funktionen gehören
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (Noise; zwei oder drei Argumente, zum Beispiel `wiggle(3, 5, 7)`)
und `get("target","prop")`, das den aktuellen Wert eines anderen Kanals liest (Kanäle laufen in Listenreihenfolge, und spätere sehen frühere Ergebnisse).

```
cat > ex.rhai <<'EOF'
add_channel_expr("head", "bone_angle", "8 * sin(t * 2 * pi)", "add");
add_noise("hips", "bone_offset_y", 1.5, 2.0, 7, "add");
EOF
osatura script biped.json ex1.json ex.rhai
```
```
2 request(s) applied; wrote ex1.json
```

Ergebnis von `explain ex1.json head <frame>`: Der Weltwinkel des Kopfes beträgt -90.0 bei den Frames 0, 12 und 24, **-82.0 bei Frame 6** und
-98.0 bei Frame 18 (eine volle Sinuswelle über den Clip; der Ruhewinkel des Kopfes ist -90 Welt-Grad). Das Noise auf der Hüfte
bewegt `hips` um einen Bruchteil einer Einheit: Ihr Kopf sitzt bei Frame 0 auf y 169.7 und bei Frame 24 auf 170.5 (170.0 ohne das Noise), er loopt also nicht (siehe oben).

Fehler werden gefangen, bevor etwas geschrieben wird:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Stolperfalle.** In einem Ausdruck ist `t` normalisiert (0 bis 1), weder Sekunden noch Frames. `sin(t * 6.2832 / 24)` sieht richtig aus
und ist falsch: Es bewegt sich kaum. Verwende `time` für Sekunden oder `frame`.

### Driver, Slider und Datenbindungen

* `slider` und `view-swap` (Rigging-Leitfaden) erzeugen Controller-Knochen plus `driven`-Kanäle.
* `bind` verbindet einen externen Wertpfad (aus einem Spiel, einer UI oder einem Test) mit einer Eigenschaft oder einem Zustandsautomat-Eingang, mit Remap,
  Glättung und einem optionalen Ausdruck:

```
$ osatura bind add m1.json b1.json look player.look head:bone_angle --remap 0,1,-20,20 --smooth 0.1
added binding 'look' (1 bindings); wrote b1.json
$ osatura bind add b1.json b2.json run player.running @hero:walking
$ osatura bind list b2.json
look  player.look -> head:BoneAngle remap 0..1 -> -20..20 clamp smooth 0.1s
run  player.running -> @hero:walking
$ echo '{"player.look":0.75,"player.running":0}' > vals.json
$ osatura bind-eval b2.json vals.json
look = 10
run = 0
```

`0.75` im Bereich 0..1, abgebildet auf -20..20, ergibt 10, wie gezeigt. `bind-eval --data out.json` schreibt dieselben Zahlen als JSON.

## 6. Laufzyklen und die Motion-Bibliothek

`walk` generiert einen prozeduralen Lauf: Er braucht Knochennamen für die Beine und (optional) Arme und Hüfte.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` ist die Zykluslänge in Frames, `--stride` der Oberschenkelschwung in Grad, `--knee` die Kniebeugung, `--bob`
das Hüftwippen in Pixeln und `--swing` der Armschwung. Die sieben Kanäle haben alle 1,5 Frames einen Key, loopen
(`extrapolate: loop`) und bestehen `lint` mit 0 Befunden. Da der Zyklus an Ort und Stelle generiert wird, gibt es keine Fortbewegung,
die entfernt werden müsste.

### Takes und Clips

Ein **Take** ist ein aufgezeichnetes Stück Animation, das in einer **Bibliotheks**-Datei gespeichert ist und sich über Projekte hinweg wiederverwenden lässt:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Die letzten Argumente von `take-record` sind Präfixe von Zielnamen, die einbezogen werden. `take-apply` schreibt den Take ab einem Frame,
mit optionalen Umbenennungen `from=to`, einer `speed` und `--blend FRAMES`, um mit vorhandener Animation überzublenden.

### Der Clip-Stack: Schichten und Gewichte

Ein **Clip** ist ein Take, der mit einem Modus, einem Gewicht und Masken auf der Timeline platziert ist. Clips schichten sich der Reihe nach; spätere Clips liegen über
früheren. Beispiel: ein Lauf plus ein winkender rechter Arm, der ein- und ausgeblendet wird.

```
$ osatura clip add biped.json c1.json lib.json walk walkclip --at 0 --loops 1 --mode replace --weight 1
added clip 'walkclip' (1 clips); run clip bake to generate channels
$ osatura clip add c1.json c2.json lib.json wave waveclip --at 0 --mode replace --weight 1 \
    --include 'arm_*_r' --fade-in 4 --fade-out 4
$ osatura clip list c2.json
1. walkclip take 'walk' frames 0..24 x1 speed 1 Replace weight 1
2. waveclip take 'wave' frames 0..24 x1 speed 1 Replace weight 1 include arm_*_r
$ osatura clip bake c2.json c3.json
baked 2 clips into 18 channels (0 replaced); wrote c3.json
$ osatura clip set-weight c3.json c4.json waveclip --keys 0:0,6:1,18:1,24:0
updated clip 'waveclip'; run clip bake to update channels
```

Der Stack berührt nie von Hand erstellte Kanäle oder gesperrte Kanäle. `--include`/`--exclude` nehmen Namensmuster,
`--track-weight arm_*=1,leg_*=0.2` gewichtet Tracks einzeln, `--mode additive|multiply` schichtet, statt zu ersetzen.

**Stolperfalle.** `clip bake` schreibt die Kanäle: Führe es nach Änderungen an Clips erneut aus (der Befehl weist darauf hin).

## 7. Zustandsautomaten und Bindungen

Ein **Zustandsautomat** wählt zur Laufzeit anhand von Eingängen (Booleans, Zahlen, Trigger) zwischen Clips. Er wird in JSON beschrieben und
beim Hinzufügen geprüft:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Zustände spielen einen Clip, ein Frame-`segment`, eine Marker-`region` oder einen 1-D-`blend` mehrerer; Übergänge feuern, wenn alle
`when`-Bedingungen (`input op value`, UND) erfüllt sind, optional erst nach `exit_time` (0 bis 1 der Länge des Zustands); der erste
passende Übergang in der Reihenfolge gewinnt; `"from":"*"` bedeutet jeden Zustand.

```
$ osatura machine add c2.json m1.json machine.json
added machine 'hero' (1 machines); wrote m1.json
$ osatura machine validate m1.json
1 machines, 0 bindings: ok
$ echo '{"duration":3.0,"events":[{"t":1.0,"trigger":["greet"]}]}' > script.json
$ osatura machine-sim m1.json script.json simout --fps 12
   0.000s  base     Entered walk
   1.000s  base     Exited walk
   1.000s  base     Entered wave
   2.000s  base     Exited wave
   2.000s  base     Entered walk
37 ticks at 12 fps, 5 events, 12 svg frames; wrote simout/sim.json
```

`machine-sim` spielt zeitgesteuerte Eingabeereignisse ab und schreibt ein Zustandsprotokoll (`sim.json`) plus SVG-Frames, sodass Zustandsautomaten ohne Laufzeitumgebung testbar
und diffbar sind. Ein Zustandsautomat ist eine reine Funktion seines Eingabeprotokolls: Dasselbe Skript liefert immer dasselbe
Ergebnis.

## 8. Lip-Sync und Sprache

Alle Audio-Eingaben müssen **WAV** sein; konvertiere andere Formate vorher mit ffmpeg. Audio, das länger als das Projekt ist, wird auf die
Projektlänge **gekürzt**; verlängere zuerst das Projekt (`frames` in der Datei), sonst bekommst du weniger Frames, als der Ton hat.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

Bei einer Kopie des Projekts mit 72 Frames (3 s):

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` wählt Mundformen allein anhand der **Lautstärke** des Audios.
* `lipsync-text` gleicht dein Transkript (`Hello there, nice to meet you today`) mit der Energie des Audios ab und nutzt den Text für
  die Formen. Es ist **keine Phonemerkennung**; das Tool sagt das jedes Mal. `--words-out` schreibt Wort- und Silbentiming
  als JSON oder SRT: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Sprachen: `--lang ro|en`.
* `speech-motion` fügt Kopfnicken, Senken und Neigen hinzu, getimt auf die Wörter (additiv).
* Die Reihenfolge der Formen ist rest, small, open, round, wide (`--map` ordnet sie deinem Switch zu).

### Von Hand erstellte Cues: `mouth`

Hast du Cues aus einem anderen Tool, nimmt `mouth <project> <cues.txt> <switch_layer> <out>` entweder
**Frame-nummerierte** Zeilen (Moho-Switch-Stil: `frame PHONEME`) oder **Sekunden** (Rhubarb-TSV): Eine Zeit, die **mit Dezimalpunkt geschrieben
ist, sind Sekunden, eine nackte Ganzzahl ist ein Frame**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Formbuchstaben werden den fünf Mundklassen zugeordnet: `X A MBP rest` auf rest; `B G H L FV etc` (und unbekannte Buchstaben) auf small; `D AI` auf open;
`F O U WQ` auf round; `C E` auf wide. `--rhubarb` ändert eine Sache: Der Buchstabe `E` wird dann als Rhubarbs gerundete Form (round) behandelt statt als
Papagayos weite. Hier ergaben `X C B D A` rest, wide, small, open, rest.

**Stolperfalle.** `0.00 X` ist Sekunde 0; `0 X` wäre Frame 0. Beides in einer Datei zu mischen ist erlaubt und verwirrend.

### Audioreaktiv und Beats

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

`sound mix` kürzt ebenfalls auf die Projektlänge (ein Ton von 3 s wurde zu 1.00 s eines Projekts von 1 s). `export` nach `.html` bettet den gemischten
Ton als mp3 ein (die Seite dieses Projekts wuchs von 50 KB auf 131 KB), was `ffmpeg` braucht; ohne es wird die Seite ohne Ton geschrieben. `.mp4`,
`.webm` und das Mischen von mp3/ogg brauchen ebenfalls `ffmpeg`.

## 9. Dinge entlang von Pfaden bewegen, Follow-Through, Foot-Lock

* **Path Follow:** Zeichne eine Linienebene, dann keyt `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` Position
  und (außer bei `no-orient`) Rotation entlang davon:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` braucht drei oder mehr Punkte: `points need an even number of coordinates: 3..1000 points`.)

* **Chain Follow-Through:** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` kopiert die Bewegung eines Drivers auf eine
  Kette von Followern, jeder später und schwächer (Overlapping Action, Schwänze, Antennen):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  Die Keys der Follower werden um die Verzögerung nach hinten verschoben, sodass einige hinter dem letzten Frame landen (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) und sich auf das kopierte Looping verlassen.
* **Foot-Lock:** Einen Fuß am Boden halten, während sich der Körper bewegt. Er braucht einen **Root-Controller** (einen Knochen ohne Elternknochen) und
  ein IK-Fußziel:

```
cat > ctrl.rhai <<'EOF'
add_bone("root_ctrl", "", 10, 200, 312, 0);
add_bone("foot_goal_l", "", 10, 212, 312, 0);
add_ik("leg_ik_l", ["thigh_l", "shin_l"], "foot_goal_l", -1);
EOF
osatura script biped.json fl0.json ctrl.rhai
cat > fl1.rhai <<'EOF'
set_key("foot_goal_l", "bone_offset_x", 0, 0);
set_key("foot_goal_l", "bone_offset_x", 12, -40);
set_key("foot_goal_l", "bone_offset_x", 24, 0);
set_key("foot_goal_l", "bone_offset_y", 0, 0);
set_key("foot_goal_l", "bone_offset_y", 6, -15);
set_key("foot_goal_l", "bone_offset_y", 12, 0);
set_key("foot_goal_l", "bone_offset_y", 24, 0);
EOF
osatura script fl0.json fl2.json fl1.rhai
$ osatura foot-lock fl2.json fl3.json foot_goal_l 312 1 2
contact frames 12..23 pinned at (172.0, 312.0)
wrote fl3.json
```

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` findet Frames, in denen der Controller den Boden berührt,
  und fügt ausgleichende Kanäle hinzu (Modus `add`), damit er nicht rutscht. Ohne einen Controller ohne Elternknochen bricht er ab:
  `foot lock needs a root controller bone (no parent)`.
* **Verwandte Bakes:** `stretch <in> <out> <follower> <leader>` (der Follower zielt auf den Leader und streckt sich zu ihm), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (Annäherungszug), `bone-vis` (Overlay-Sichtbarkeit).

## 10. Face-Tracking und seine Grenzen

**Sei ehrlich darüber, was das ist.** OSATURA enthält einen eigenen kleinen Gesichtsdetektor und ein Landmark-Modell (23 Landmarks, Layout
`osatura23`). Es ist im Binary eingebaut, läuft lokal, braucht keine Downloads und wurde **nur mit Gesichtern trainiert, die vom
eigenen synthetischen Generator dieses Projekts erzeugt wurden**: stilisierte und Cartoon-Köpfe. Es ist **nicht an Fotos, Webcams oder echtem
Video validiert.** Die gemessenen Werte (`osatura face-eval --eval-n 50 --seed 9`) sind:

| Messgröße | Ergebnis |
|---|---|
| Zurückgehaltene synthetische Gesichter, Erkennungsrate | 1.0 (50 von 50) |
| Landmark-Fehler (NME) Mittelwert / Median / 90. Perzentil | 0.028 / 0.026 / 0.046 |
| Falsch-Positive bei 50 Szenen ohne Gesicht | 2 |
| Cartoon-Designs, NME wo erkannt | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Cartoon-Designs, erkannt | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Diese Werte gelten für den *eigenen synthetischen Stil*. Lies sie nicht als Genauigkeit bei einer Person vor einer Kamera. Brauchst du
Tracking echter Gesichter, erzeuge Landmarks mit einem Tool, dem du vertraust, und gib sie als NDJSON ein (das Eingabeformat von `face-bake`
ist offen); das Face-Panel des Studios erwartet, dass du für Kamera-Eingaben eine Funktion `window.osaturaLandmarkProvider` bereitstellst.

Die Pipeline:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

mit einer Mapping-Datei (hier `facemap.json`):

```
{"face": {"mincutoff": 3.0},
 "params": [
  {"source":"face","id":"head_yaw","target":"head","prop":"bone_angle","out_min":-20,"out_max":20},
  {"source":"face","id":"mouth_open","target":"mouth","prop":"layer_scale_y","out_min":0.6,"out_max":1.8}
 ],
 "triggers": [
  {"source":"face","id":"eye_r_open","edge":"release","threshold":0.5,"action":{"type":"fire_event","name":"blink"}}
 ]}
```

Signal-IDs: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Eine falsche ID listet die gültigen auf: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
`face-bake` kalibriert das neutrale Gesicht aus den ersten Frames (`--calibrate-frames`, `--calib`).

**Gut zu wissen.** Der Bake wird **auf die Projektlänge gekürzt**: 60 Landmark-Frames auf einem Projekt mit 24 Frames
ergaben nur die Keys 0 bis 23, und eine Warnung auf stderr sagt, wie viele Aufnahme-Frames nicht gebacken wurden (die Projektlänge bleibt unverändert). Die Zusammenfassungszeile zeigt `(K keys in them)` für die neuen Kanäle; `N trigger keys set` zählt Keys, die in bereits vorhandene Kanäle geschrieben wurden. `face-detect <image> --overlay o.png` zeigt, was der Detektor auf einem Standbild gefunden hat.

## 11. Mocap und BVH

```
$ osatura mocap-info walk.bvh
11 joints (5 end sites), 36 channels per frame
48 frames at 0.0417 s (24.000 fps), duration 1.958 s
Hips  offset (0.000, 95.000, 0.000)  [Xpos Ypos Zpos Zrot Xrot Yrot]
...
$ osatura mocap-match biped.json walk.bvh
slot         bvh joint                  rig bone                conf  via         len x  note
hips         Hips                       hips                    1.00  name         2.00
head         Head                       head                    1.00  name         4.00
shin.l       LeftLeg                    shin_l                  0.63  name         7.20  limb length ratio is 2.1x off the median
...
$ osatura mocap-retarget biped.json walk.bvh mo.json --view side --smooth 2 --weight 1
note: project length extended from 24 to 48 frames
11 bones, 48 frames at 24 fps, root scale 2.7692, +13 channels; wrote mo.json
```

* Ein 2D-Rig kann nur eine Projektion der 3D-Bewegung verwenden: `--view front|side|top|yaw=DEG` wählt sie.
* `mocap-match` gibt eine Konfidenztabelle aus (die `shin`-Zeilen oben tragen eine Notiz, weil ihre Gliedmaßenlänge um das 2,1-Fache vom Median abweicht).
  `--map map.json` überschreibt die Vermutung, und `--min-conf C` legt die Konfidenz fest, unter der ein Gelenk nicht verwendet wird.
* **`mocap-retarget` verlängert das Projekt**, damit der Clip passt (aus 24 wurden 48 Frames; eine Notiz weist darauf hin, und `--keep-length` kürzt stattdessen den Clip). `--in-place` entfernt die Fortbewegung der Wurzel,
  `--footlock BONE --ground Y` pinnt einen Fuß (er braucht den Root-Controller ohne Elternknochen, wie `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  und `--mirror` spiegelt die Bewegung.
* `import-bvh` ist die ältere, explizite Form mit `--joint-map Joint=bone,...`.

## 12. Live-Eingabe (MIDI, Maus, Gamepad, Tasten)

Nimm eine Performance im Live-Panel des Studios auf, oder spiele eine Aufnahme oder eine MIDI-Datei offline ab:

```
cat > lmap.json <<'EOF'
{"params":[{"source":"midi_cc","id":"1:1","target":"arm_upper_r","prop":"bone_angle","in_min":0,"in_max":1,"out_min":0,"out_max":160,"smooth":0.2}],
 "triggers":[{"source":"midi_note","id":"1:60","edge":"press","threshold":0.5,"action":{"type":"fire_event","name":"cymbal"}}]}
EOF
osatura live-map biped.json wheel.mid lmap.json lm.json
```
```
27 events -> +1 channels (K keys in them), 0 trigger keys set, 0 takes, 1 events (0 after the last frame skipped); wrote lm.json
```

Das Modulationsrad (CC 1 auf Kanal 1) führte den Arm über 24 gebackene Keys von 0 auf 160 Grad, und das Note-on bei 0.5 s wurde zum Projekt-
Ereignis `cymbal` bei Frame 12. Quellen sind `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` und `face`. MIDI-IDs haben die Form `"<channel>:<number>"`, wobei die Kanäle bei 1 beginnen. Web MIDI im
Studio braucht einen Browser, der es unterstützt (siehe [STUDIO.md](STUDIO.md)); der Offline-Befehl braucht nur die Datei.

## 13. Layout-Container und Scrollen

Layout-Container ordnen Ebenen für eine andere Canvas-Größe neu an (responsive UI-Animation); Scroll-Bereiche fügen begrenztes oder elastisches
Scrollen hinzu. Container sind JSON, und der Solver ist deterministisch:

```
$ cat cont.json
{"id":"faces","axis":"row","gap":12,"padding":[10,10,10,10],
 "rect":{"kind":"pin","left":10,"right":10,"top":10,"height":100},
 "members":[{"layer":"eye_l"},{"layer":"eye_r","grow":1}]}
$ osatura container add biped.json c1.json cont.json
added container 'faces' (1 containers); wrote c1.json
$ osatura container solve c1.json --size 800x360
canvas 800x360
container faces: x 10.00 y 10.00 w 780.00 h 100.00
layer eye_l: dx -186.000 dy -44.000 scale 1.0000 x 1.0000
layer eye_r: dx -146.000 dy -44.000 scale 1.0000 x 1.0000
$ osatura layout-preview c1.json lp.svg --size 800x360
wrote lp.svg (800x360, 1 containers, 2 layers placed)
```

`dx`/`dy` sind Offsets, die zusätzlich zur Animation angewendet werden. Details zu Containern und zu `scroll add|list|remove|eval` (elastische Grenzen, Snap-Punkte, Fling-
Simulation) stehen in [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). Das Layout wird von der Laufzeitumgebung und von Vorschauen angewendet;
**exportierte Dateien enthalten das für die eigene Größe des Projekts gelöste Layout.**

## 14. Kamera, Shots, Tiefenunschärfe

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` zeigt 11 Kanäle: die 7 des Laufs plus die der Kamera. `dof` weichzeichnet jede Ebene der obersten Ebene um `|parallax - focus|`
(Stärke und Radius werden durch `--max-radius` begrenzt, Standard 24). Tiefenunschärfe ist ein Post-Process-Rendering; `dof-export` schreibt sie in ein
GIF, APNG oder einen Ordner.

## 15. Ebeneneffekte

Effekte sind eine geordnete Liste an einer Ebene (Blur, Schlagschatten, Tint, Fill), die in einem isolierten Durchgang vor Maske, Deckkraft und Blend-
Modus der Ebene gerendert wird, bis zu 16 pro Ebene. Ihre Zahlen lassen sich mit Kanälen animieren, die auf `"<layer>::fx<index>"` zielen, mit einer der Eigenschaften `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (Farben sind nicht animierbar):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

`check` besteht, und der Schatten gleitet über den Loop von -6 nach +6. Eine **Post-Effekt-Kette** (Outline, Posterize, Relight mit Normal Maps...) rendert einen
Frame stattdessen aus einer JSON-Liste:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Sicherheitsprüfungen für Blitze und Kamerabewegung (eine technische Hilfe auf Grundlage der öffentlichen Definition WCAG 2.3.1, **keine Broadcast-Zertifizierung**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion Skin und Trails (Overlays)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` zeichnet Ghosts an den benachbarten *Key*-Frames statt in einem festen Abstand. Overlays verändern das Projekt nie und sind nicht Teil eines Exports.
Das Studio zeigt sie live (`Alt+N` Onion, `Alt+M` Trails, `Alt+7` Panel).

## 17. Prozedurale Idles

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Geseedet, sodass derselbe Befehl überall dieselbe Animation liefert. `autorig` fügt Atmen und Blinzeln für dich hinzu, wenn die Namen passen.

## 18. Qualitätssicherung: check, lint, fuzz, CI

* `check` kompiliert das Projekt genau so, wie es der Renderer tun wird. Exit 1 bei einem Fehler.
* `lint` findet statische Probleme (Exit 1 bei Fehlern; Informationen und Warnungen lassen es nicht scheitern): unerreichbare IK-Ziele (`L003`), Kanäle, die
  sich nie ändern (`L005`), Keys außerhalb der Timeline (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` hinzugefügt, `-` entfernt, `~` geändert) und `vcs` (Versionskontrolle mit semantischem Merge für Projektdateien).
* **Fuzz** versetzt das Rig zufällig in Posen und meldet, wo es bricht:

```
$ osatura fuzz walk.json --samples 200 --seed 3 --strategy lhs --only thigh_l,shin_l,arm_upper_l
fuzz 'Biped': 200 samples (lhs, seed 3, frame 0), 12 parameters, 22 failures (11.0%), max severity 1.00, baseline 0.00, 0.78 ms/sample
  off_canvas: 22
  extreme-range failure excess  arm_upper_l              28%
  extreme-range failure excess  thigh_l                  9%
  extreme-range failure excess  shin_l                   2%
  strongest interaction: bone:arm_upper_l:scale + bone:arm_upper_l:offx (cell failure rate 53% over 17 samples)
fuzz: failure rate 11.00% exceeds --max-fail-rate 5.00%
```

  **Ein einfaches `fuzz` ist ein Bericht und endet mit Exit 0; füge `--gate` (jeder Fehler ergibt Exit 1) oder `--max-fail-rate R` (ein Bruchteil) hinzu, um zu gaten.** Der Abtastbereich ist der volle Bereich jedes Parameters, daher scheitert ein Rig mit IK
  in fast allen Samples (`98.0%`, 48 von 50 `ik_unreachable`, beim Biped mit Foot-Lock). Schränke mit `--only`, `--props`,
  `--margin` und `--ignore KINDS` ein, wähle den Bereich, den du für zulässig hältst, und lege die Schwelle fest, die du akzeptierst.
  `--report out.json`, `--heatmap out.svg` und `--minimize` (kleinste reproduzierende Pose) helfen. Strategien: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Goldens und CI.** `golden write walk.json gold.json` speichert Frame-Signaturen; `golden check` vergleicht standardmäßig exakt (Exit 1 bei jeder
  Änderung; `--tol T` lockert das). Wird eine *andere* Animation dagegen geprüft, listet es die abweichenden Frames auf: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` fasst dies und mehr in einem Manifest zusammen:

```
$ osatura ci init walk.json --out osatura-ci.json
wrote osatura-ci.json with 9 checks:
  validate / render frames / render frames-float / determinism / pose-fuzz / limits / gif-size / loop-seam / render-speed
$ osatura ci run osatura-ci.json
[PASS   ] validate     validate           project compiles; 0 lint finding(s), none at 'error' or above
[PASS   ] render       frames             6 frame(s) match
[PASS   ] render       frames-float       2 frame(s) match
[PASS   ] determinism  determinism        6 render(s) repeated byte-identically
[PASS   ] fuzz         pose-fuzz          44 of 100 pose samples fail (44.00% <= 46.00%)
[PASS   ] limits       limits             within all limits
[PASS   ] export-size  gif-size           gif export is 106945 bytes (budget 160418)
[PASS   ] loop         loop-seam          seam 1.728% vs mean step 1.178%
[INFO   ] perf         render-speed       median 64.52 ms/frame, budget 534 ms (+50% tolerance = 801.00 ms)
9 checks: 8 passed, 0 failed, 0 errors, 1 info, 0 updated, 0 blessed (4.9 s)
```

  `ci run` endet mit Exit 0, wenn alles besteht, mit 1, wenn eine Prüfung fehlschlägt, und mit 2 bei einem Fehler. `ci bless` übernimmt die aktuellen Renderings als neue Golden Frames,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` erklärt eine Änderung mit Heatmaps. Die Pose-Fuzz-Schwelle von 46 % im Manifest wurde auf das
  Projekt „wie es jetzt ist“ kalibriert (das Tool gibt diesen Hinweis aus); ziehe sie bewusst an. Details stehen in [CI.md](CI.md).

## 19. Größe und Detailgrad

```
$ osatura size-report walk.json | head -9
project file: 17507 bytes (2185 gzip)
  by layer (JSON bytes; embedded image base64 in brackets):
         2658  mouth
          643  mouth/m_small
           ...
  by channel (keys, removable at tol 0.05000000074505806):
         1116  thigh_r.BoneAngle  keys 17 removable 1
```

`size-report` zeigt, welche Ebenen, Bilder und Kanäle Bytes kosten. Hat eine Plattform ein Budget, findet `export-fit` den Export bester Qualität, der hineinpasst:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

Der zweite Befehl endete mit **1** und schrieb trotzdem die kleinste Datei, die er gefunden hatte (14952 Bytes). Budgets werden standardmäßig an der Rohgröße geprüft;
`--measure gzip` prüft die komprimierte Größe (das Lottie oben hat gezippt 1429 Bytes). Die Suche ist durch `--max-seconds` begrenzt (Standard 60 s Echtzeit);
im Debug-Build wurde die GIF-Suche durch dieses Limit gestoppt und behielt ihr bestes Ergebnis.

Für Zeichnungen mit vielen Kurven und Meshes erzeugt der **Detailgrad** (Level of Detail) vereinfachte Kopien mit einer *deklarierten* Fehlergrenze in Ausgabepixeln:

```
$ osatura lod walk.json --tolerance-px 0.5 --levels 3 --out-dir lods --verify
level  tol_px  anchors  mesh_v  keys  layers    bytes  declared_px  ...  measured_px  ratio
L0      0.000       75       0   119      20    17507       0.0000  ...            -      -
L1      0.500       72       0   100      20    16193       0.1153  ...       0.0895   1.3x
L2      1.000       70       0   100      20    16040       0.3969  ...       0.3597   1.1x
L3      2.000       69       0   100      20    16080       0.5570  ...       0.4997   1.1x
$ osatura lod-pick walk.json --screen-px 200x180 --max-px 0.5
screen 200x180 -> scale 0.5000 px/unit; budget 0.5 px -> level L3 (declared 0.2785 px, 69 anchors, 0 mesh vertices, 100 keys)
```

`--verify` misst die tatsächliche Abweichung (Exit 1, wenn sie die Grenze überschreitet); im obigen Lauf lag jeder gemessene Fehler unter seiner deklarierten Grenze. Beachte, wie
wenig ein winziges, auf Rechtecken basierendes Projekt einspart (17507 auf 16080 Bytes): LOD lohnt sich bei detaillierter Grafik, nicht bei dieser.

## 20. Ehrliche Grenzen

* Lip-Sync ist Energie plus Textabgleich, keine Phonemerkennung.
* Der Gesichtsdetektor ist rein synthetisch (Abschnitt 10): Er macht eine Webcam-Performance nicht von allein nutzbar.
* `make-loop` entfernt die Fortbewegung; `loop-solve` behebt sie nicht; Noise loopt nicht.
* Audio wird auf die Projektlänge gekürzt; `mocap-retarget` verlängert das Projekt.
* Lottie, GIF und HTML können nicht jede Funktion abbilden; Export-Warnungen und `--report` listen auf, was angenähert wurde ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Nur Chromium wurde für das Studio getestet ([STUDIO.md](STUDIO.md)).
