# OSATURA-Rigging-Leitfaden

Dieser Leitfaden behandelt alles, was eine Zeichnung in Bewegung versetzt: Knochen, Constraints, inverse Kinematik,
Skinning, Deformer, Physik, Auto-Rigging, Spiegelung, Ansichtswechsel, Gesichts-Rigs und Kameraarbeit.
Jedes Beispiel wurde mit dem echten Binary ausgeführt (`osatura 0.1.0`, Debug-Build, Linux), und die
genannten Zahlen sind die Zahlen, die es ausgegeben hat. Wenn du noch kein erstes Projekt gebaut hast, arbeite zuerst
[GETTING_STARTED.md](GETTING_STARTED.md) durch. Die Flags aller Befehle stehen in
[CLI_REFERENCE.md](CLI_REFERENCE.md), das Dateiformat in [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Lizenz dieses Dokuments: CC-BY-4.0.

![Acht kleine Renderings: ein von einem Knochen gebogener Skin-Streifen, drei IK-Varianten (Ziel außer Reichweite, Stretch 2, Soft 0.5) und vier Vektor-Deformer (Cage, Lattice, Spine, Pin), jeweils bei Frame 12 bewegt.](../../img/rigging-sheet.png)

*Kontaktbogen der folgenden Beispiele: Skin-Gewichte mit FK, FABRIK mit unerreichbarem Ziel,
`stretch = 2`, `soft = 0.5`, Cage-, Lattice-, Spine- und Pin-Deformer.*

## 1. Zwei Wege, ein Rig zu bearbeiten

* **Befehle und `script`.** Die meisten Rig-Änderungen haben einen Befehl (`autorig`, `cage`, `physics`, `skin`...) oder
  eine Rhai-Funktion in `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Ein Skript gilt ganz oder gar nicht: Wird auch nur eine Anfrage
  abgelehnt, wird nichts geschrieben. Ein Skript darf höchstens 5000 Anfragen enthalten.
* **JSON.** Einiges hat weder einen Befehl noch einen Skript-Setter: IK-`soft` und -`stretch`, Federn, Vektor-Deformer
  (Lattice, Spine, Pin) und Skin-Gewichte pro Anker. Bearbeite das Projekt-JSON (mit einem beliebigen Editor, einem
  Python-Skript oder dem JSON-Panel des Studios) und führe danach `osatura check` aus.

`check`, `lint`, `explain` und `render --bones` sind die vier Befehle, die du beim Rigging ständig brauchst.
`explain` ist der wertvollste: Er listet der Reihe nach jeden Ruhewert, jeden Kanal, jede Constraint und jeden
IK-Beitrag auf, die zur finalen Transformation eines Knochens geführt haben, sodass du nie raten musst, warum etwas dort ist, wo es ist.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Knochen und Hierarchien

Ein Knochen hat einen `name`, einen `parent` (keiner = Wurzel), eine `length`, einen `rest_offset` (wo sein Kopf sitzt, im
Koordinatensystem des **Elternknochens**; bei einem Wurzelknochen ist das die Canvas) und einen `rest_angle` in Grad relativ zum
Elternknochen. Konventionen:

* y zeigt nach unten; der Winkel 0 zeigt entlang +x; positive Winkel drehen auf dem Bildschirm im Uhrzeigersinn; **90 ist unten, -90 ist oben**.
* `rest_offset` wird um den Weltwinkel des Elternknochens gedreht. Ein Kind eines Knochens, der nach oben zeigt und
  den `rest_offset` (70, 0) hat, sitzt 70 Einheiten *am Elternknochen entlang nach oben*, nicht 70 Einheiten rechts von der Canvas.
* Knochen müssen mit den Eltern zuerst aufgelistet sein. Alles andere wird abgelehnt:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Animationskanäle an einem Knochen sind **Deltas zur Ruhepose**: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. Ein Key-Wert von 0 bedeutet „Ruhe“.

Eine minimale Kette aus 3 Knochen, die die meisten der folgenden Beispiele verwenden (`lab.json`; sie wird generiert, siehe Abschnitt 4):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Nützliche Befehle: `reparent <in> <out> <bone> <new_parent|none>` behält die Ruhepose bei (ich habe geprüft: `b3`
bleibt bei Kopf (220,150), wenn er auf none umgehängt wird), `set_parent` in einem Skript ist die niedrigere Form, und
`bone-vis <in> <out> <bone> <from> <to>` zeigt einen Knochen nur für einen Frame-Bereich in den Overlays.

### Starre Befestigung: das einfachste Rig

Eine Ebene kann mit `set_layer("layer", "bone", "bone_name")` starr an einem Knochen mitfahren. So sind der
Tutorial-Charakter und der unten verwendete Biped aufgebaut: einzelne Teile, jeweils an einen Knochen gebunden. Das braucht keine
Gewichte und übersteht jeden Export. Wähle es, wenn deine Grafik bereits in Teile zerlegt ist.

**Stolperfalle.** Befestige Ebenen, solange die Knochen in der Ruhepose sind, und zeichne jedes Teil so, dass sein Gelenk
auf dem Kopf seines Knochens liegt. Ein Teil, das ein paar Einheiten daneben gezeichnet ist, schwingt um den falschen Punkt.

**Stolperfalle.** Setze den Ruhewinkel der Armknochen nicht auf ±100, „um eine T-Pose zu bekommen“, mit einem Rest-Offset von 0: Die
beiden Arme beginnen dann im selben Punkt. Gib jeder Schulter einen seitlichen `rest_offset` und einen Winkel von 180 oder
0, wie beim Biped (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Constraints

Constraints stehen im Array `constraints` des Projekts und werden nach den Kanälen **in Listenreihenfolge** ausgeführt.
Erzeuge sie mit `add_aim`, `add_copy_rotation` und `add_pin` (Skript) oder in JSON.

| Constraint | Skript | Wirkung |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | dreht `bone` so, dass er auf den Kopf des Knochens `target` zeigt |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` übernimmt (einen gewichteten Anteil der) Weltrotation von `source` |
| Pin | `add_pin(name, bone, target, weight)` | bewegt den Kopf von `bone` auf den Kopf von `target` |

Geprüft mit der 3-Knochen-Kette, `target` ein Knochen bei (250,40):

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

Das Gewicht blendet linear zwischen dem animierten Winkel und dem Constraint-Winkel, 0.5 halbiert also die Drehung.

Winkelgrenzen sind Knocheneigenschaften (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Sie begrenzen die **animierte** Rotation: Keyst du `b1` auf 60 Grad, wird er bei 20 gehalten:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Stolperfalle: Die Reihenfolge zählt.** Eine Copy-Rotation auf dem *Elternknochen* eines Aim-Knochens beeinflusst das Aim.
Zeigt ein Knochen in eine seltsame Richtung, führe `explain` für ihn und seinen Elternknochen aus, lies die Beitragsliste
von oben nach unten und ordne dann das Array `constraints` um.

## 4. Inverse Kinematik

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Zwei Knochen: ein analytischer Solver. `bend` (+1 oder -1) bestimmt, in welche Richtung das Gelenk knickt.
* Drei oder mehr Knochen: FABRIK. `iterations` ist eine Untergrenze; der Solver stoppt früh, sobald die Spitze innerhalb von
  0.01 px liegt, und läuft sonst bis zu einer festen Obergrenze.
* `mix` ist die FK/IK-Mischung (0 = die gekeyte FK-Pose, 1 = volle IK). Sie lässt sich mit der Eigenschaft `ik_mix`
  animieren, womit ein Arm zwischen FK und IK wechselt. `autorig` fügt Ketten mit `mix 0` hinzu, damit bestehende
  FK-Animation weiter funktioniert.
* Das Ziel ist der **Kopf** eines Knochens; gib ihm keine Grafik und eine kleine Länge (`goal`, `hand_ctrl`).

### Unerreichbare Ziele: gestreckt, Soft, Stretch

Die Kette `b1..b3` hat von ihrer Wurzel (60,150) aus eine Reichweite von 240. Das Ziel wird nach (330,60) verschoben, 285 entfernt:

| Einstellung | Ergebnis (Position der Spitze) |
|---|---|
| Standard | voll zum Ziel hin gestreckt, Spitze (287.7, 74.1), genau 240 von der Wurzel entfernt |
| `"stretch": 2` | die Kette verlängert sich, die Spitze erreicht (330.0, 60.0) |
| `"soft": 0.5` (3 Knochen) | die Kette bleibt geknickt, Spitze (258.8, 83.7); sie streckt sich nicht durch |

`soft` (0 bis 0.9) und `stretch` (1 bis 4) sind Felder der IK-Kette im Projekt-JSON; es gibt dafür
weder einen Befehl noch einen Skript-Setter:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

Am Buddy-Arm (Reichweite 80 von der Schulter bei (160,140)) mit dem Ziel bei (250,140), 90 entfernt:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

Soft-IK nähert sich bei einer zweiknochigen Gliedmaße der vollen Streckung asymptotisch: Wenn das Ziel 100, 120,
135 und 150 entfernt liegt, pendelt sich das x der Spitze bei 231.1, 234.6, 236.3 und 237.4 ein und erreicht 240 nie. Das ist das
gewollte „kein Knie-Pop“-Verhalten. Ist zusätzlich `stretch` größer als 1, ist die Soft-Zone der letzte `soft`-Anteil der
gestreckten Reichweite (`reach x stretch`): Die Kette verlängert sich kontinuierlich und läuft sanft in ihre maximale Länge aus (vor Wave 7
wurde `soft` jenseits der vollen Reichweite ignoriert, und die Kette sprang an dieser Stelle gerade).

**Stolperfalle.** `lint` meldet unerreichbare Ziele als Information, nicht als Fehler:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Lies diese Zeilen; ein Bein, das „schwebt“, ist fast immer eine davon.

**Stolperfalle.** `fuzz` (siehe [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) tastet zufällige Posen ab, darunter auch Controller-
Positionen weit außer Reichweite, daher zeigt ein Rig mit IK eine große Zahl bei `ik_unreachable`. Das ist eine Aussage
über den abgetasteten Bereich, kein Fehler.

## 5. Skinning

OSATURA hat vier Skinning-Mechanismen; wähle nach der Art deiner Grafik.

### 5.1 Vektor-Anker mit Gewichten (JSON)

Die Anker eines Vektorpfads können `weights` tragen, eine Liste von `{bone, weight}`. Der
Anker folgt dann einer linearen Mischung der Transformationen der Knochen. Dies ist ein Streifen mit 12 Ankern, der über die
drei Knochen geskinnt ist; die `weights` laufen von `b1` über `b2` zu `b3`:

```
python3 - <<'EOF'
import json
d = {"format":1,"name":"Lab","width":400,"height":300,"fps":24,"frames":24,"background":"#f6f1e7"}
d["bones"] = [{"name":"b1","parent":None,"length":80,"rest_offset":[60,150],"rest_angle":0},
              {"name":"b2","parent":"b1","length":80,"rest_offset":[80,0],"rest_angle":0},
              {"name":"b3","parent":"b2","length":80,"rest_offset":[80,0],"rest_angle":0}]
def w(x):
    t = (x - 60) / 80.0
    if t <= 0.5: return [{"bone":"b1","weight":1.0}]
    if t < 1:    a = (t-0.5)/0.5; return [{"bone":"b1","weight":1-a},{"bone":"b2","weight":a}]
    if t < 1.5:  return [{"bone":"b2","weight":1.0}]
    if t < 2:    a = (t-1.5)/0.5; return [{"bone":"b2","weight":1-a},{"bone":"b3","weight":a}]
    return [{"bone":"b3","weight":1.0}]
xs  = list(range(60, 301, 20))
top = [{"pos":[x,138],"weights":w(x)} for x in xs]
bot = [{"pos":[x,162],"weights":w(x)} for x in reversed(xs)]
d["layers"] = [{"name":"strip","type":"vector","paths":[{"name":"strip","closed":True,"anchors":top+bot,"fill":"#e07a5f"}]}]
ease = {"type":"ease_in_out"}
d["channels"] = [{"target":"b2","prop":"bone_angle","source":"keys","track":{"keys":[
    {"frame":0,"value":0,"ease":ease},{"frame":12,"value":50,"ease":ease},{"frame":24,"value":0,"ease":ease}]}}]
json.dump(d, open('lab.json','w'), indent=1)
EOF
osatura check lab.json && osatura render lab.json 12 lab12.png --bones
```
```
ok: 'Lab' 400×300 @ 24 fps, 24 frames, 3 bones, 0 ik, 1 layers, 1 channels
wrote lab12.png
```

Bei Frame 12 beträgt der Weltwinkel von `b3` 50.0 Grad und sein Kopf liegt bei (191.4, 211.3) (laut `explain`). Der Streifen biegt sich
weich zwischen den Gelenken, weil sich die Gewichte dort überlappen.

**Stolperfalle.** Es gibt keinen Befehl, der Vektorgewichte für dich berechnet: Die Projekt-API hat zwar einen automatischen
Vektor-Skinner (`Project::auto_skin`), aber er ist eine Bibliotheksfunktion, wird vom Demo verwendet und ist nicht als
CLI-Befehl verfügbar. Verwende für Vektorgrafik starre Befestigung (Abschnitt 2) oder schreibe die Gewichte wie oben.

**Stolperfalle.** Wenige Anker, wenige Stellen zum Biegen. Ein einfaches Rechteck hat vier Anker und kann sich daher nur an
seinen Ecken biegen. Unterteile die Form (füge Anker entlang der Kanten hinzu), bevor du skinnst oder einen Deformer verwendest.

### 5.2 Bild-Meshes: `mesh-trace` und `skin-diffuse`

Bei Rastergrafik zeichnest du die Alpha-Silhouette zu einem Dreiecks-Mesh nach und berechnest die Gewichte dann per Wärmediffusion:

```
osatura mesh-trace flag.json flag_m.json flag density=12
```
```
wrote flag_m.json: 369 vertices (178 on the outline), 558 triangles, area 17237 px2
  quality min 0.480 mean 0.867, min angle 19.5 deg, mean edge 3.0 px at the outline / 10.4 px inside
ok: ...
```
```
osatura skin-diffuse flag_m.json flag flag_s.json --influences 4
```

`skin-diffuse <project> <layer> <out> [--influences N]` weist jedem Mesh-Vertex bis zu N Knochengewichte zu. Es
braucht ein Mesh: Bei einer Bildebene ohne Mesh bricht es mit `layer 'flag' has no mesh` ab. Bei nur einem Knochen
bekommt jeder Vertex das Gewicht 1.0 darauf (die Flagge hat einen Knochen, `mast`), was korrekt und nicht sehr
interessant ist; verwende mindestens zwei Knochen, um eine Mischung zu sehen. `mesh-trace` hat außerdem `hotspot=x,y,r`, um das Mesh
dort dichter zu machen, wo du Biegung erwartest, sowie Optionen, um schon beim Nachzeichnen zu skinnen (`bones=R`, `skin=1`; siehe
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Skin-Sets (Outfits, Ausdrücke, Schadenszustände)

Der Befehl `skin` ist ein anderes Konzept: Ein **Skin-Set** tauscht ganze Gruppen von Ebenen (und optional
Knochen, Constraints und IK-Ketten) gemeinsam aus.

```
cat > hats.rhai <<'EOF'
add_rect("hat_red", 138, 62, 44, 14, "#c1121f");
add_rect("hat_blue", 138, 62, 44, 14, "#1d4e89");
add_bone("tail", "body", 40, 0, 0, 180);
add_rect("tail_shape", 100, 185, 60, 8, "#3d405b");
set_layer("hat_red", "bone", "neck");
set_layer("hat_blue", "bone", "neck");
set_layer("tail_shape", "bone", "tail");
EOF
osatura script blob.json k0.json hats.rhai
osatura skin k0.json k1.json add hat red hat_red
osatura skin k1.json k2.json add hat blue hat_blue
osatura skin k2.json k3.json bones hat red tail        # the tail exists only with the red hat
osatura skin k3.json k4.json apply hat blue            # show blue, hide red
osatura skin k4.json k5.json schedule hat 0:red,12:blue
osatura check k5.json
```

`apply` zeigt einen Skin und blendet die anderen derselben Gruppe aus. `schedule` animiert die Auswahl mit **Step-Keys
auf der Ebenen-Deckkraft** (das Projekt hat kein animierbares „visible“-Flag); verwende es also bei Ebenen, deren Deckkraft du sonst nicht animierst.
Inaktive Knochen einzelner Skins lassen sich nicht aus dem Rig löschen, ohne die Indizes zu zerstören, daher werden sie **eingeklappt**:
Ein `multiply`-Kanal auf `bone_scale` mit Step-Keys 0 und 1 setzt die Skalierung des Knochens auf 0, und alles, was an ihm
befestigt oder auf ihn gewichtet ist, schrumpft zu einem Punkt:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Von der Validierung erzwungene Regeln: Ein Knochen gehört zu höchstens einem Skin; jeder Nachfahre eines Skin-Knochens gehört zum selben Skin; Skin-Knochen
dürfen keine `scale_limits` haben. Verwende `skin rig <group> <name> <constraint,...|-> <ik,...|->` für Constraints und IK-Ketten, die
nur mit einem Skin aktiv sein sollen.

### 5.4 Starre Teile plus eine Switch-Ebene

Für Dinge, die ihre Form diskret ändern (Münder, Hände, Augen), lege die Alternativen in eine `switch`-Ebene
und key `layer_switch`. Siehe Abschnitt 11.

## 6. Deformer

Deformer bewegen die Punkte einer Ebene. Sie stehen in den `deformers` der Ebene, werden über Kanäle mit einem dreiteiligen Ziel
`"layer::deformer::point"` animiert und wirken auf die **Anker** eines Pfads oder die Vertices eines Bild-Meshes. **Hat eine Form zu wenige Anker, verformt sie sich kaum: Unterteile sie zuerst.**

### Cage (Mean-Value-Koordinaten)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` umschließt eine Ebene mit einem Polygon (`auto` = Bounds mit Polsterung, `hull` = konvexe Hülle, `verts=N` = Segmente pro
Seite); Inhalt innerhalb folgt den Eckpunkten des Polygons, Inhalt außerhalb bleibt unberührt. `cage-pose` keyt Vertex-
Offsets (Kanäle `cage_dx`, `cage_dy` mit dem Ziel `strip::cage::1`). `timeline` listet sie unter
„Deformers and paths“ auf. Ein gesperrter Kanal wird abgelehnt.

### Lattice, Spine, Pin (JSON)

Für diese drei gibt es keinen Befehl; füge sie den `deformers` einer Ebene hinzu und animiere sie mit Kanälen:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: ein Free-Form-Gitter. Die Punkt-ID ist `col,row`. Gut für Squash und Stretch.
* **Spine**: eine Polylinie mit einem Einflussradius `radius`; das Bewegen eines Spine-Punkts biegt die Grafik in der Nähe (Schwänze, Tentakel).
* **Pin**: Puppet-Pin-Warp; jeder Pin zieht Punkte innerhalb von `radius`.

Jeder davon wurde im Kontaktbogen oben korrekt gerendert, angewendet auf einen Streifen ohne Knochengewichte. Ein Pfad,
der *zusätzlich* per Knochen geskinnt ist, wird von beiden verformt; entferne die Gewichte vom Teststreifen, wenn du einen
Deformer allein sehen willst.

### Eine Auswahl verformen: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` rotiert, skaliert oder verschiebt eine Auswahl von Kontrollpunkten (Cage, Lattice, Cloth, Pin, Spine oder ein
Grid-Bild-Mesh) um einen Pivot (`x,y`, `bone:NAME`, `centroid`, `center`) und keyt das Ergebnis. `radius` fügt einen
Soft-Selection-Abfall hinzu (`linear`, `smooth`, `gauss`, `sharp`, `constant`); im obigen Beispiel wurden die Vertices 1-2 ausgewählt, und der
Abfall fügte 7 und 8 hinzu, sodass 8 Kanäle gekeyt wurden. `soft-scale` skaliert proportional (`aspect=free` lässt die
beiden Achsen voneinander abweichen). `layer-pivot` verschiebt den Pivot einer Ebene, ohne ihre Grafik zu bewegen.

### Mesh-Locks und Glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: Die Vertices des Followers
  folgen der Oberfläche des Hosts per baryzentrischer Verschiebung. Damit bleiben ein Tattoo, ein Flicken oder ein Knopf auf einem sich verformenden
  Stoff. Zyklen werden abgelehnt. `mesh-lock list` gab `lock: patch:0 follows host:0 vertices all strength 1` aus.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: Vertex-Paare
  zweier Ebenen werden verschweißt, sodass `b` `a` folgt. `glue list` gab
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3` aus.

### Cloth

Cloth braucht ein **Grid**-Mesh. Ein nachgezeichnetes Free-Form-Mesh wird abgelehnt:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Ein Grid fordert man mit `cols=` und `rows=` an:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Dieser Bake dauerte im Debug-Build 0,43 s und erzeugte 180 gekeyte Kanäle (6480 Keys), sodass das Ergebnis vollständig
editierbar ist, wie jede andere Animation exportiert wird und deterministisch ist. `pin=` akzeptiert `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` oder `...@bone`, um an einen Knochen zu pinnen. `cols`/`rows` legen das Simulationsgitter fest; das gerenderte Bild wird davon
verzerrt, und bei starkem Zoom kannst du in einem PNG-Rendering schwache Dreiecksnähte sehen.

## 7. Physik und Federn

### Gebackene Physik: `physics`

`physics` simuliert eine Kette von Knochen (Schwerkraft, Wind, Kollisionen) und backt das Ergebnis in gewöhnliche Knochenwinkel-Keys.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` ist eines von `hair`, `tail`, `rope`, `antenna`, `cape`.
* `circle=x,y,r[@bone];...` und `capsule=x1,y1,x2,y2,r[@bone];...` sind Collider; mit `@bone` folgt ein Collider einem Knochen.
* `drive=bone:gain` lässt die Bewegung eines anderen Knochens die Kette anschieben; `vortex=bone|x,y:swirl[:radial[:radius]]`
  fügt ein wirbelndes Kraftfeld hinzu; `follow=bone` und `lateral=` formen, wie die Kette einem Elternknochen folgt.
* `merge=keep` (Standard) **überschreibt keinen** Knochen, der bereits Animation hat:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Gib `merge=replace` an, wenn du das willst.
* Ketten im Stil `b2,b3/b1` und ein unbekannter Knochen (`no bone 'b9'`) werden validiert.

### Live-Federn: `springs`

Eine `Spring` fügt einem Knochen zur Auswertungszeit ein gedämpftes Nachschwingen hinzu, ohne dass etwas gebacken wird. Sie ist nur per JSON verfügbar:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` ist in Hz (höher ist steifer), `damping` ist ein Verhältnis (unter 1 schwingt es nach, 1 ist kritisch gedämpft), und `warmup`
(standardmäßig an) rollt einen Durchlauf vor, damit Loops im eingeschwungenen Zustand starten. `explain` meldet:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Um ein Gefühl zu treffen, das du gezeichnet hast, fitte es:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Stolperfalle.** Federn sind nicht Teil von Lottie. Backe sie (`physics`), wenn du in ein Format exportierst, das kein
Äquivalent hat; `export --report` listet auf, was angenähert wurde.

## 8. Auto-Rig, Tagging und `coach`

Benenne deine Knochen und Ebenen mit Standardwörtern (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`), dann kann OSATURA ihre Rollen
erkennen:

```
$ osatura tag biped.json | head -8
bone  hips                     -> hips           (90%)
bone  torso                    -> torso          (80%)
bone  head                     -> head           (90%)
bone  arm_upper_l              -> arm.upper.l    (90%)
bone  arm_lower_l              -> arm.lower.l    (90%)
...
no bone found for: neck, face.eye.l, face.eye.r, face.brow.l, ...
```

`autorig <in> <out> [seed]` fügt dann aus diesen Rollen zerstörungsfreie Automatisierung hinzu:

```
$ osatura autorig biped.json ar.json 7
+ IK arm_l -> arm_l_ctrl (mix 0, bend +1)
+ IK arm_r -> arm_r_ctrl (mix 0, bend +1)
+ IK leg_l -> leg_l_ctrl (mix 0, bend -1)
+ IK leg_r -> leg_r_ctrl (mix 0, bend -1)
+ breathing on torso
+ blink on eye_l
+ blink on eye_r
wrote ar.json
```

Das Projekt ging von 11 Knochen und 0 IK-Ketten auf 15 Knochen und 4 IK-Ketten mit 3 neuen Kanälen. Alles
Hinzugefügte sind gewöhnliche Daten: Lösche, was du nicht willst. Fehlende Teile werden mit einer Begründung übersprungen
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), lies also den Bericht.

**Stolperfalle.** `autorig` ist ein Projekt-zu-Projekt-Befehl, und die Ausgabe darf die Eingabe sein. Schreibe bei einer Datei, die dir wichtig ist,
in einen neuen Namen: `osatura autorig buddy.json buddy.json` ersetzt `buddy.json` (das Schreiben ist atomar, aber
es gibt kein Rückgängig).

`coach <project>` liest die Animation und gibt Vorschläge zu Timing und Prinzipien aus; `lint` prüft das Rig
statisch. Beide sind beratend.

## 9. Spiegelung und Symmetrie

`mirror <in> <out>` spiegelt alle gekeyten Kanäle an der vertikalen Achse: Ziele tauschen die Seiten (Präfixe `L_`/`R_`,
Suffixe `_L`/`_R` und `_l`/`_r`, die Wörter left/right), und Rotationen und horizontale Translationen
wechseln das Vorzeichen. Die gespiegelten Kanäle **ersetzen** die Kanäle des Projekts. Beim laufenden Biped wurden die Keys des linken Oberschenkels
`0.0, 10.7, 19.8, 25.9` zu `-0.0, -10.7, -19.8, -25.9` des rechten Oberschenkels und umgekehrt, und `hips` (ohne Seite)
behielt seine Werte. `diff` listet die sechs geänderten Kanäle auf:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Stolperfalle.** Bei einem Projekt ohne Links/Rechts-Namen schreibt `mirror` stillschweigend eine unveränderte Kopie und meldet
`wrote`. Prüfe das Ergebnis immer mit `diff`. Benenne deine Seiten konsequent (`_l`/`_r`), bevor du animierst.

## 10. Ansichtswechsel und Parent-Wechsel

### Turnarounds: `view-swap`

Ein Turnaround (Front, Dreiviertel, Seite) ist eine Switch-Ebene mit einem Kind pro Ansicht, gesteuert von der Eigenschaft eines
Controller-Knochens:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Ausgabekanal: ein `driven`-Kanal `layer_switch` ← `view_ctrl.bone_offset_x` mit einer Step-Kurve (Kind 0 unter 0,
Kind 2 ab 0, Kind 4 ab 40). Animiere `bone_offset_x` von `view_ctrl`, um die Ansichten zu wechseln. Die Kind-Indizes müssen existieren, und die Ebene
muss ein Switch sein (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` erzeugt einen **Pose-Slider**, einen Controller-Knochen `ctrl_<name>`, dessen
`offset_x` zwischen gespeicherten Posen blendet:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Umhängen über die Zeit: `parent-switch`

Um ein Objekt von einer Hand in die andere weiterzugeben (oder ein Requisit aufzunehmen), ohne dass es springt:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Er backt Kanäle für Knochen-Offset und -Winkel (hier 3 Kanäle, ein Key pro Frame), sodass der Knochen beim Wechsel seine Weltposition
behält. Er funktioniert nicht bei Knochen mit IK, Constraints oder Federn.

## 11. Gesichts-Rigs

### Mundformen

Zeichne die Mundformen als Kinder einer **Switch-Ebene** in dieser Reihenfolge, die jeder Lip-Sync-Befehl voraussetzt:
**rest, small, open, round, wide**. Der Mund des Bipeds ist `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Hat dein Switch weniger Kinder, wählt `--map 0,1,2,3,4` (fünf Zahlen, eine pro Form), welches Kind jede Form verwendet; der Standard
richtet sich nach der Kinderzahl (1 Kind: alle; 2: geschlossen+offen; 3: geschlossen+offen+rund/breit; 4: geschlossen+klein+offen+rund/breit; 5+: in dieser Reihenfolge).

Befestige den Switch am Kopfknochen (`set_layer("mouth", "bone", "head")`), damit er sich mit dem Kopf dreht. Ihn zu animieren ist
das Thema des [Lip-Sync-Abschnitts des Animationsleitfadens](ANIMATION_GUIDE.md#8-lip-sync-und-sprache).

### Augen und Clipping

`eye-clip <in> <out>` maskiert Iris- und Pupillen-Ebenen mit der Augenform derselben Seite, sodass eine Pupille das
Auge nicht verlassen kann. `autorig` führt ihn aus. Bei einem Rig ohne Iris-/Pupillen-Ebenen schreibt er die Datei unverändert. Lidschläge sind ein gekeytes oder prozedurales
`layer_scale_y` auf der Augenebene:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` erzeugen geseedete prozedurale Kanäle (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): Derselbe Seed liefert auf jeder Maschine dieselben Lidschlagzeiten.

### Ein Gesicht aus einer Performance ansteuern

`face-synth`, `face-track`, `face-bake` und das Face-Panel des Studios machen aus einem Landmark-Stream Kanäle. Sie werden
mit ehrlichen Genauigkeitswerten im [Abschnitt zum Face-Tracking im Animationsleitfaden](ANIMATION_GUIDE.md#10-face-tracking-und-seine-grenzen) behandelt.

## 12. Kamera, Shots und Parallaxe

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` speichert einen benannten Kameraausschnitt (Kameramitte und Zoom);
  `camera-seq <in> <out> <frame:shot[:cut],...>` keyt die Kamera durch gespeicherte Shots; `timeline` zeigt die
  resultierenden Kamerakanäle.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` verteilt einen `parallax`-Faktor auf
  Ebenen der obersten Ebene (die fernen bewegen sich weniger, wenn die Kamera schwenkt).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` weichzeichnet jede Ebene der obersten Ebene um
  `|parallax - focus|`; `dof-export` rendert einen Bereich in ein GIF, APNG oder einen Ordner. Fokus und Stärke folgen den Kanälen
  `camera_focus` / `camera_blur`, sofern du sie nicht festlegst.

Geprüft am laufenden Biped:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Ebeneneffekte (Blur, Schlagschatten, Tint, Fill, Outline, Posterize...) können pro Ebene oder als Kette angewendet werden: siehe den
[Animationsleitfaden](ANIMATION_GUIDE.md#15-ebeneneffekte).

## 13. Eine Rigging-Checkliste

1. `check` nach jeder strukturellen Änderung; `lint`, bevor du animierst.
2. Halte die Ruhepose langweilig und symmetrisch; animiere Deltas dazu.
3. Lege Controller (IK-Ziele, `view_ctrl`, `root_ctrl`) als Knochen ohne Grafik im Rig an.
4. Bevorzuge für Cartoon-Grafik starre Teile, skinne nur, was sich biegen muss, und unterteile, bevor du verformst.
5. Verwende `explain`, wenn etwas nicht stimmt; das geht schneller als Raten.
6. Backe Physik und Cloth, wenn die Animation in einem Format ohne diese Funktionen überleben muss, und lies `export --report`.
7. Schreibe in neue Dateinamen, bis du einem Befehl vertraust; Projekt-zu-Projekt-Befehle können ihre Eingabe überschreiben.

## 14. Grenzen, die du kennen solltest

* Das Skinning von Vektorgrafik ist **Linear Blend** (kein Dual Quaternion), mit Gewichten, die du selbst lieferst; es gibt keinen automatischen
  Vektor-Skinner in der CLI.
* Lattice, Spine, Pin, IK-`soft`/-`stretch` und Federn gibt es nur per JSON.
* Physik und Cloth sind Bakes: Ändere das Rig und backe erneut. `physics` überschreibt vorhandene Animation nur auf ausdrückliche Anweisung.
* Cloth braucht ein Grid-Mesh; nachgezeichnete Free-Form-Meshes sind für Skinning und Mesh-Locks gedacht.
* Lottie und GIF können nicht alles darstellen (Striche mit variabler Breite, gescherte Verläufe, Federn...). Warnungen werden
  beim Export ausgegeben; siehe [EXPORT_FORMATS.md](EXPORT_FORMATS.md) und [LIMITATIONS.md](LIMITATIONS.md).
* Der Gesichtsdetektor ist nur mit synthetischen Gesichtern trainiert (Abschnitt 11 und der Animationsleitfaden).
