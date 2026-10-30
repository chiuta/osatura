# Ghid de rigging OSATURA

Acest ghid acoperă tot ce pune un desen în mișcare: oase, constrângeri, cinematică inversă,
skinning, deformatoare, fizică, auto-rigging, oglindire, schimbarea vederilor, rig-uri de față și lucrul cu camera.
Fiecare exemplu a fost rulat cu binarul real (`osatura 0.1.0`, build de debug, Linux), iar cifrele
citate sunt cele afișate de acesta. Dacă nu ai construit încă un prim proiect, parcurge mai întâi
[GETTING_STARTED.md](GETTING_STARTED.md). Opțiunile fiecărei comenzi se găsesc în
[CLI_REFERENCE.md](CLI_REFERENCE.md); formatul de fișier este descris în [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Licența acestui document: CC-BY-4.0.

![Opt randări mici: o fâșie cu skin îndoită de un os, trei variante de IK (țintă în afara razei de acțiune, stretch 2, soft 0.5) și patru deformatoare vectoriale (cage, lattice, spine, pin), fiecare mutat la cadrul 12.](../../img/rigging-sheet.png)

*Planșă de contact a exemplelor de mai jos: greutăți de skin cu FK, FABRIK cu o țintă inaccesibilă,
`stretch = 2`, `soft = 0.5`, deformatoare cage, lattice, spine și pin.*

## 1. Două moduri de a edita un rig

* **Comenzi și `script`.** Majoritatea editărilor de rig au o comandă (`autorig`, `cage`, `physics`, `skin`...) sau
  o funcție Rhai în `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Un script este de tip „totul sau nimic": dacă o
  cerere este respinsă, nu se scrie nimic. Un script poate conține cel mult 5000 de cereri.
* **JSON.** Câteva lucruri nu au comandă sau setter în script: `soft` și `stretch` la IK, arcurile, deformatoarele
  vectoriale (lattice, spine, pin) și greutățile de skin per ancoră. Editează JSON-ul proiectului (cu orice editor, cu un
  script Python sau cu panoul JSON al studioului) și rulează apoi `osatura check`.

`check`, `lint`, `explain` și `render --bones` sunt cele patru comenzi pe care le vei folosi constant la rigging.
`explain` este cea mai valoroasă: listează, în ordine, fiecare valoare de repaus, canal, constrângere și contribuție
IK care a produs transformarea finală a unui os, așa că nu mai trebuie să ghicești de ce ceva se află unde se află.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Oase și ierarhii

Un os are un `name`, un `parent` (niciunul = rădăcină), o `length`, un `rest_offset` (locul capului său, în
sistemul de referință al **părintelui**; pentru un os rădăcină este canvasul) și un `rest_angle` în grade relativ la
părinte. Convenții:

* y indică în jos; unghiul 0 indică de-a lungul lui +x; unghiurile pozitive rotesc în sens orar pe ecran; **90 este în jos, -90 este în sus**.
* `rest_offset` este rotit cu unghiul global al părintelui. Un copil al unui os orientat în sus, cu
  `rest_offset` (70, 0), se află la 70 de unități *în sus pe părinte*, nu la 70 de unități în dreapta pe canvas.
* Oasele trebuie listate cu părinții primii. Orice altceva este respins:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Canalele de animație ale unui os sunt **delte față de poziția de repaus**: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. O valoare de cheie 0 înseamnă „repaus".

Un lanț minimal de 3 oase, folosit în majoritatea exemplelor de mai jos (`lab.json`; este generat, vezi secțiunea 4):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Comenzi utile: `reparent <in> <out> <bone> <new_parent|none>` păstrează poziția de repaus (am verificat: `b3`
rămâne cu capul în (220,150) când este reasociat la none), `set_parent` într-un script este forma de nivel mai jos, iar
`bone-vis <in> <out> <bone> <from> <to>` afișează un os doar în suprapuneri (overlay) pentru un interval de cadre.

### Atașarea rigidă: cel mai simplu rig

Un strat poate însoți rigid un os cu `set_layer("layer", "bone", "bone_name")`. Așa sunt realizate
personajul din tutorial și biped-ul de mai jos: piese separate, fiecare atașată unui os. Nu are nevoie de
greutăți și supraviețuiește oricărui export. Alege-o atunci când desenul tău este deja tăiat în piese.

**Capcană.** Atașează straturile cât timp oasele sunt în poziția de repaus și desenează fiecare piesă astfel încât articulația ei să stea
pe capul osului. O piesă desenată cu câteva unități alături se va balansa în jurul unui punct greșit.

**Capcană.** Nu seta unghiul de repaus al oaselor brațelor la ±100 „ca să obții o poză în T" cu un offset de repaus 0: cele
două brațe pornesc dintr-un singur punct. Dă fiecărui umăr un `rest_offset` lateral și un unghi de 180 sau
0, ca la biped (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Constrângeri

Constrângerile sunt listate în array-ul `constraints` al proiectului și rulează **în ordinea listei**, după
canale. Creează-le cu `add_aim`, `add_copy_rotation` și `add_pin` (script) sau în JSON.

| Constrângere | Script | Efect |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | rotește `bone` astfel încât să indice spre capul osului `target` |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` preia (o parte ponderată din) rotația globală a lui `source` |
| Pin | `add_pin(name, bone, target, weight)` | mută capul lui `bone` peste capul lui `target` |

Verificat cu lanțul de 3 oase, `target` fiind un os în (250,40):

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

Ponderea amestecă liniar între unghiul animat și unghiul constrâns, deci 0.5 înjumătățește rotația.

Limitele de unghi sunt proprietăți ale osului (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Ele limitează rotația **animată**: dacă setezi o cheie pe `b1` la 60 de grade, aceasta este ținută la 20:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Capcană: ordinea contează.** Un copy-rotation plasat pe *părintele* unui os cu aim interacționează cu aim-ul.
Dacă un os ajunge să indice într-o direcție ciudată, rulează `explain` pe el și pe părintele lui, citește lista de contribuții
de sus în jos, apoi reordonează array-ul `constraints`.

## 4. Cinematică inversă

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Două oase: un solver analitic. `bend` (+1 sau -1) alege în ce parte se pliază articulația.
* Trei sau mai multe oase: FABRIK. `iterations` este un prag minim; solverul se oprește devreme când vârful ajunge la
  mai puțin de 0.01 px, altfel rulează până la o limită maximă fixă.
* `mix` este amestecul FK/IK (0 = poziția FK cu chei, 1 = IK complet). Poate fi animat cu proprietatea
  `ik_mix`, așa comută un braț între FK și IK. `autorig` adaugă lanțuri cu `mix 0`, astfel încât animația FK existentă
  să continue să funcționeze.
* Ținta este **capul** unui os; nu-i da desen și dă-i o lungime mică (`goal`, `hand_ctrl`).

### Ținte inaccesibile: drept, soft, stretch

Lanțul `b1..b3` are o rază de acțiune de 240 de la rădăcina sa (60,150). Mutăm goal-ul în (330,60), la 285 distanță:

| Setare | Rezultat (poziția vârfului) |
|---|---|
| implicit | complet extins spre goal, vârf (287.7, 74.1), exact la 240 de rădăcină |
| `"stretch": 2` | lanțul se alungește, vârful ajunge în (330.0, 60.0) |
| `"soft": 0.5` (3 oase) | lanțul rămâne îndoit, vârf (258.8, 83.7); nu se îndreaptă |

`soft` (0 până la 0.9) și `stretch` (1 până la 4) sunt câmpuri ale lanțului IK din JSON-ul proiectului; nu există
comandă sau setter în script:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

Pe brațul lui Buddy (rază de acțiune 80 de la umărul din (160,140)) cu goal-ul în (250,140), la 90 distanță:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

IK soft pe un membru cu două oase se apropie asimptotic de extensia completă: pe măsură ce goal-ul se îndepărtează la 100, 120,
135 și 150, abscisa x a vârfului se stabilizează la 231.1, 234.6, 236.3 și 237.4 și nu ajunge niciodată la 240. Acesta este
comportamentul dorit, „fără salt la genunchi". Când `stretch` este de asemenea peste 1, zona soft este ultima fracțiune `soft` din
raza întinsă (`reach x stretch`): lanțul se alungește continuu și intră lin în lungimea sa maximă (înainte de wave 7,
`soft` era ignorat dincolo de raza completă, iar lanțul sărea brusc în poziție dreaptă în acel punct).

**Capcană.** `lint` raportează țintele inaccesibile ca informații, nu ca erori:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Citește aceste linii; un picior care „plutește" este aproape întotdeauna unul dintre aceste cazuri.

**Capcană.** `fuzz` (vezi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) eșantionează poziții aleatorii, inclusiv poziții ale controlerelor
departe în afara razei de acțiune, așa că un rig cu IK arată un număr mare `ik_unreachable`. Aceasta este o constatare
despre intervalul eșantionat, nu o eroare.

## 5. Skinning

OSATURA are patru mecanisme de skinning; alege în funcție de tipul de desen pe care îl ai.

### 5.1 Ancore vectoriale cu greutăți (JSON)

Ancorele unei căi vectoriale pot purta `weights`, o listă de `{bone, weight}`. Ancora
urmează atunci un amestec liniar al transformărilor oaselor. Aceasta este o fâșie cu 12 ancore, cu skin pe cele
trei oase; `weights` trec în rampă de la `b1` la `b2` la `b3`:

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

La cadrul 12, unghiul global al lui `b3` este 50.0 de grade, iar capul lui se află în (191.4, 211.3) (din `explain`). Fâșia se îndoaie
lin între articulații, deoarece greutățile se suprapun acolo.

**Capcană.** Nu există nicio comandă care să calculeze greutățile vectoriale pentru tine: API-ul proiectului are un
skinner vectorial automat (`Project::auto_skin`), dar este o funcție de bibliotecă, folosită de demo, și nu este expus ca
 comandă CLI. Pentru desen vectorial folosește atașarea rigidă (secțiunea 2) sau scrie greutățile ca mai sus.

**Capcană.** Puține ancore, puține locuri de îndoit. Un dreptunghi simplu are patru ancore, deci se poate îndoi doar
în colțuri. Subdivizează forma (adaugă ancore de-a lungul laturilor) înainte de skinning sau de a folosi un deformator.

### 5.2 Mesh-uri de imagine: `mesh-trace` și `skin-diffuse`

Pentru desen raster, urmărește silueta alfa într-un mesh triunghiular, apoi calculează greutățile prin difuzie de căldură:

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

`skin-diffuse <project> <layer> <out> [--influences N]` atribuie fiecărui vertex al mesh-ului până la N greutăți de os. Are
nevoie de un mesh: pe un strat de imagine fără mesh se oprește cu `layer 'flag' has no mesh`. Cu un singur os,
fiecare vertex primește greutatea 1.0 pe el (steagul are un singur os, `mast`), ceea ce este corect și nu prea
interesant; folosește cel puțin două oase ca să vezi un amestec. `mesh-trace` are și `hotspot=x,y,r` pentru a face mesh-ul mai
dens acolo unde te aștepți la îndoire, precum și opțiuni pentru a face skin în timpul urmăririi (`bones=R`, `skin=1`; vezi
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Seturi de skin (ținute, expresii, stări de deteriorare)

Comanda `skin` este o idee diferită: un **set de skin** comută laolaltă grupuri întregi de straturi (și, opțional,
oase, constrângeri și lanțuri IK).

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

`apply` afișează un skin și le ascunde pe celelalte din același grup. `schedule` animează alegerea cu **chei de tip step
pe opacitatea stratului** (proiectul nu are un flag „visible" animabil), așa că folosește-l pe straturi a căror opacitate nu o animezi altfel.
Oasele per-skin inactive nu pot fi șterse din rig fără a strica indicii, așa că sunt **colapsate**:
un canal `multiply` pe `bone_scale` cu chei step 0 și 1 face scara osului 0, iar tot ce se sprijină
pe el sau este ponderat pe el se reduce la un punct:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Reguli impuse de validare: un os aparține cel mult unui skin; fiecare descendent al unui os de skin aparține aceluiași skin; oasele de skin
nu trebuie să aibă `scale_limits`. Folosește `skin rig <group> <name> <constraint,...|-> <ik,...|->` pentru constrângeri și lanțuri IK care
trebuie să fie active doar cu un anumit skin.

### 5.4 Piese rigide plus un strat switch

Pentru lucruri care își schimbă forma discret (guri, mâini, ochi), pune alternativele într-un strat `switch`
și pune chei pe `layer_switch`. Vezi secțiunea 11.

## 6. Deformatoare

Deformatoarele mută punctele unui strat. Ele sunt listate în `deformers` al stratului, sunt animate prin
canale cu o țintă în trei părți `"layer::deformer::point"` și acționează asupra **ancorelor** unei căi sau asupra vertecșilor unui
mesh de imagine. **Dacă o formă are prea puține ancore, se va deforma abia perceptibil: subdivizeaz-o mai întâi.**

### Cage (coordonate cu valoare medie)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` învelește un strat într-un poligon (`auto` = limite cu margine, `hull` = anvelopă convexă, `verts=N` = segmente pe
latură); conținutul dinăuntru urmează vârfurile poligonului, conținutul din afară rămâne neatins. `cage-pose` pune chei pe offseturile
vârfurilor (canale `cage_dx`, `cage_dy` cu ținta `strip::cage::1`). `timeline` le listează sub
„Deformers and paths". Un canal blocat este refuzat.

### Lattice, spine, pin (JSON)

Aceste trei nu au comandă; adaugă-le în `deformers` al unui strat și animează-le cu canale:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: o grilă liberă (free-form). Id-ul punctului este `col,row`. Util pentru squash și stretch.
* **Spine**: o polilinie cu o `radius` de influență; mutarea unui punct al spine-ului îndoaie desenul din apropiere (cozi, tentacule).
* **Pin**: deformare de tip puppet-pin; fiecare pin trage punctele aflate în `radius`.

Fiecare dintre acestea s-a randat corect în planșa de contact de mai sus, când a fost aplicat unei fâșii fără greutăți de os. O cale
care este *și* cu skin pe oase va fi deformată de ambele; elimină greutățile din fâșia de test dacă vrei să vezi un
deformator singur.

### Deformarea unei selecții: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` rotește, scalează sau mută o selecție de puncte de control (cage, lattice, cloth, pin, spine sau un
mesh de imagine de tip grilă) în jurul unui pivot (`x,y`, `bone:NAME`, `centroid`, `center`) și pune chei pe rezultat. `radius` adaugă o
atenuare de selecție soft (`linear`, `smooth`, `gauss`, `sharp`, `constant`); exemplul de mai sus a selectat vertecșii 1-2, iar
atenuarea i-a adăugat pe 7 și 8, deci au primit chei 8 canale. `soft-scale` face redimensionare proporțională (`aspect=free` permite ca
cele două axe să difere). `layer-pivot` mută pivotul unui strat fără a muta desenul acestuia.

### Blocări de mesh și glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: vertecșii follower-ului
  urmează suprafața host-ului prin deplasare baricentrică. Folosește-l pentru a ține un tatuaj, un petic sau un nasture pe un
  cloth care se deformează. Ciclurile sunt refuzate. `mesh-lock list` a afișat `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: perechi de vertecși
  ale două straturi sunt sudate astfel încât `b` să urmeze `a`. `glue list` a afișat
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Cloth

Cloth are nevoie de un mesh de tip **grilă**. Un mesh liber urmărit (traced) este refuzat:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Cere o grilă cu `cols=` și `rows=`:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Bake-ul a durat 0.43 s în build-ul de debug și a produs 180 de canale cu chei (6480 de chei), astfel că rezultatul este complet
editabil, se exportă ca orice altă animație și este determinist. `pin=` acceptă `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` sau `...@bone` pentru a prinde de un os. `cols`/`rows` stabilesc grila de simulare; imaginea randată este deformată de aceasta
și este posibil să vezi în randările PNG, la zoom mare, mici cusături între triunghiuri.

## 7. Fizică și arcuri

### Fizică precalculată (baked): `physics`

`physics` simulează un lanț de oase (gravitație, vânt, coliziuni) și coace (bake) rezultatul în chei obișnuite de unghi de os.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` este unul dintre `hair`, `tail`, `rope`, `antenna`, `cape`.
* `circle=x,y,r[@bone];...` și `capsule=x1,y1,x2,y2,r[@bone];...` sunt colizori; `@bone` face ca un colizor să urmeze un os.
* `drive=bone:gain` face ca mișcarea altui os să împingă lanțul; `vortex=bone|x,y:swirl[:radial[:radius]]`
  adaugă un câmp de forțe turbionar; `follow=bone` și `lateral=` modelează modul în care lanțul urmează un părinte.
* `merge=keep` (implicit) **nu suprascrie** un os care are deja animație:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Spune `merge=replace` când chiar asta vrei.
* Lanțurile în stilul `b2,b3/b1` și un os necunoscut (`no bone 'b9'`) sunt validate.

### Arcuri live: `springs`

Un `Spring` adaugă unui os o continuare amortizată a mișcării (follow-through) în momentul evaluării, fără nimic precalculat. Se definește doar în JSON:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` este în Hz (mai mare înseamnă mai rigid), `damping` este un raport (sub 1 oscilează, 1 este amortizat critic), iar `warmup`
(activ implicit) rulează în prealabil o trecere, astfel încât buclele să pornească în regim staționar. `explain` spune:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Pentru a potrivi o senzație pe care ai desenat-o, fă un fit:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Capcană.** Arcurile nu fac parte din Lottie. Coace-le (`physics`) dacă exporți într-un format care nu are
echivalent; `export --report` listează ce a fost aproximat.

## 8. Auto-rig, etichetare și `coach`

Denumește-ți oasele și straturile cu cuvinte standard (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`) și OSATURA poate
recunoaște rolurile lor:

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

`autorig <in> <out> [seed]` adaugă apoi automatizare nedistructivă pornind de la aceste roluri:

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

Proiectul a trecut de la 11 oase și 0 lanțuri IK la 15 oase și 4 lanțuri IK, cu 3 canale noi. Tot ce
s-a adăugat sunt date obișnuite: șterge ce nu vrei. Părțile lipsă sunt omise cu un motiv
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), așa că citește raportul.

**Capcană.** `autorig` este o comandă de la proiect la proiect, iar ieșirea poate fi chiar intrarea. Pe un fișier la care ții,
scrie într-un nume nou: `osatura autorig buddy.json buddy.json` înlocuiește `buddy.json` (scrierea este atomică, dar
nu există undo).

`coach <project>` citește animația și afișează sugestii despre timing și principii; `lint` verifică rig-ul
static. Ambele sunt consultative.

## 9. Oglindire și simetrie

`mirror <in> <out>` oglindește toate canalele cu chei față de axa verticală: țintele își schimbă partea (prefixele `L_`/`R_`,
sufixele `_L`/`_R` și `_l`/`_r`, cuvintele left/right), iar rotațiile și translațiile orizontale
își schimbă semnul. Canalele oglindite **înlocuiesc** canalele proiectului. Pe biped-ul care merge, cheile coapsei stângi
`0.0, 10.7, 19.8, 25.9` au devenit cele ale coapsei drepte `-0.0, -10.7, -19.8, -25.9` și invers, iar `hips` (fără parte)
și-a păstrat valorile. `diff` listează cele șase canale modificate:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Capcană.** Pe un proiect fără nume de stânga/dreapta, `mirror` scrie în tăcere o copie nemodificată și spune
`wrote`. Fă întotdeauna `diff` pe rezultat. Denumește-ți părțile consecvent (`_l`/`_r`) înainte de a anima.

## 10. Schimbarea vederii și comutarea părintelui

### Turnaround-uri: `view-swap`

Un turnaround (față, trei sferturi, profil) este un strat switch cu câte un copil pentru fiecare vedere, comandat de proprietatea unui
os controler:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Canalul rezultat: un canal `driven` `layer_switch` ← `view_ctrl.bone_offset_x` cu o curbă step (copilul 0 sub 0,
copilul 2 de la 0, copilul 4 de la 40). Animează `bone_offset_x` al lui `view_ctrl` pentru a schimba vederile. Indicii copiilor trebuie să existe, iar stratul
trebuie să fie un switch (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` creează un **slider de poziții**, un os controler `ctrl_<name>` al cărui
`offset_x` amestecă poziții memorate:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Reasocierea părintelui în timp: `parent-switch`

Pentru a trece un obiect de la o mână la alta (sau pentru a ridica un recuzită) fără salt:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Coace canale de offset și unghi de os (aici 3 canale, cu o cheie pe fiecare cadru), astfel încât osul să își păstreze poziția
globală la comutare. Nu funcționează pentru oase cu IK, constrângeri sau arcuri.

## 11. Rig-uri de față

### Forme ale gurii

Desenează formele gurii ca copii ai unui **strat switch** în această ordine, pe care o presupune fiecare comandă de lip-sync:
**rest, small, open, round, wide**. Gura biped-ului este `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Dacă switch-ul tău are mai puțini copii, `--map 0,1,2,3,4` (cinci numere, câte unul pentru fiecare formă) alege ce copil folosește fiecare formă; implicit
se ține cont de numărul de copii (1 copil: toate; 2: închis+deschis; 3: închis+deschis+rotund/larg; 4: închis+mic+deschis+rotund/larg; 5+: în ordinea aceasta).

Atașează switch-ul la osul capului (`set_layer("mouth", "bone", "head")`) pentru ca el să se rotească odată cu capul. Animarea lui este
subiectul [secțiunii despre lip-sync din ghidul de animație](ANIMATION_GUIDE.md#8-lip-sync-și-vorbire).

### Ochii și decuparea (clipping)

`eye-clip <in> <out>` maschează straturile de iris și pupilă cu forma ochiului de pe aceeași parte, astfel încât o pupilă să nu poată ieși din
ochi. `autorig` îl rulează. Pe un rig fără straturi de iris/pupilă, scrie fișierul nemodificat. Clipitul este un
`layer_scale_y` cu chei sau procedural pe stratul ochiului:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` generează canale procedurale cu seed (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): același seed dă aceleași momente de clipit pe orice mașină.

### Comandarea unei fețe dintr-o performanță

`face-synth`, `face-track`, `face-bake` și panoul Face al studioului transformă un flux de repere (landmark) în canale. Sunt
descrise, cu cifre de acuratețe oneste, în [secțiunea despre face tracking din ghidul de animație](ANIMATION_GUIDE.md#10-face-tracking-și-limitele-lui).

## 12. Cameră, shot-uri și parallax

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` salvează un cadraj de cameră cu nume (centrul camerei și zoom);
  `camera-seq <in> <out> <frame:shot[:cut],...>` pune chei pe cameră prin shot-urile salvate; `timeline` arată
  canalele de cameră rezultate.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` distribuie un factor `parallax` pe
  straturile de nivel superior (cele îndepărtate se mișcă mai puțin când camera face pan).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` estompează fiecare strat de nivel superior cu
  `|parallax - focus|`; `dof-export` randează un interval într-un GIF, APNG sau într-un folder. Focusul și intensitatea urmează
  canalele `camera_focus` / `camera_blur`, dacă nu le fixezi.

Verificat pe biped-ul care merge:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Efectele de strat (blur, drop shadow, tint, fill, outline, posterize...) pot fi per strat sau aplicate ca lanț: vezi
[ghidul de animație](ANIMATION_GUIDE.md#15-efecte-de-strat).

## 13. O listă de verificare pentru rigging

1. `check` după fiecare modificare structurală; `lint` înainte de a anima.
2. Păstrează poziția de repaus plictisitoare și simetrică; animează delte față de ea.
3. Pune controlerele (goal-uri IK, `view_ctrl`, `root_ctrl`) în rig ca oase fără desen.
4. Preferă piesele rigide pentru desen de tip cartoon, fă skin doar la ce trebuie să se îndoaie și subdivizează înainte de a deforma.
5. Folosește `explain` când ceva nu merge; este mai rapid decât să ghicești.
6. Coace fizica și cloth-ul dacă animația trebuie să supraviețuiască într-un format fără aceste funcții și citește `export --report`.
7. Scrie în nume de fișier noi până când ai încredere într-o comandă; comenzile de la proiect la proiect pot suprascrie intrarea.

## 14. Limite pe care ar trebui să le cunoști

* Skinning-ul desenului vectorial este de tip **linear blend** (fără dual quaternion), cu greutăți pe care le furnizezi tu; nu există
  un skinner vectorial automat în CLI.
* Lattice, spine, pin, `soft`/`stretch` la IK și arcurile se definesc doar în JSON.
* Fizica și cloth-ul sunt bake-uri: dacă schimbi rig-ul, coace din nou. `physics` nu suprascrie animația existentă decât dacă i se cere.
* Cloth are nevoie de un mesh de tip grilă; mesh-urile libere urmărite (traced) sunt pentru skinning și blocări de mesh.
* Lottie și GIF nu pot reprezenta orice (contururi cu lățime variabilă, gradienți deformați, arcuri...). Avertismentele se
  afișează la export; vezi [EXPORT_FORMATS.md](EXPORT_FORMATS.md) și [LIMITATIONS.md](LIMITATIONS.md).
* Detectorul de fețe este antrenat doar pe fețe sintetice (secțiunea 11 și ghidul de animație).
