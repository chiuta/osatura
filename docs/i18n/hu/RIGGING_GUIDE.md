# OSATURA rigging útmutató

Ez az útmutató mindent bemutat, amitől egy rajz mozogni kezd: csontok, kényszerek, inverz kinematika,
skinning, deformerek, fizika, auto-rigging, tükrözés, nézetváltás, arcrigek és kamera. Minden példát a valódi
binárissal futtattam le (`osatura 0.1.0`, debug build, Linux), az idézett számok pedig pontosan azok, amelyeket
a program kiírt. Ha még nem készítettél első projektet, kezdd a
[GETTING_STARTED.md](GETTING_STARTED.md) fájllal. Minden parancs kapcsolói a
[CLI_REFERENCE.md](CLI_REFERENCE.md) fájlban találhatók; a fájlformátum leírása a [PROJECT_FORMAT.md](PROJECT_FORMAT.md) fájlban van.

A dokumentum licence: CC-BY-4.0.

![Nyolc kis render: egy csont által meghajlított skinelt csík, három IK-változat (elérhetetlen cél, stretch 2, soft 0.5) és négy vektoros deformer (cage, lattice, spine, pin), mindegyik a 12. képkockán elmozdítva.](../../img/rigging-sheet.png)

*Az alábbi példák kontaktlapja: skin-súlyok FK-val, FABRIK elérhetetlen céllal,
`stretch = 2`, `soft = 0.5`, cage, lattice, spine és pin deformerek.*

## 1. A rig szerkesztésének két módja

* **Parancsok és `script`.** A legtöbb rig-szerkesztéshez van parancs (`autorig`, `cage`, `physics`, `skin`...) vagy
  Rhai-függvény az `osatura script` parancsban (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). A szkript vagy mindent, vagy semmit ír: ha bármelyik
  kérést elutasítják, semmi sem íródik ki. Egy szkript legfeljebb 5000 kérést tartalmazhat.
* **JSON.** Néhány dolognak nincs parancsa vagy szkriptes beállítója: az IK `soft` és `stretch` mezői, a rugók,
  a vektoros deformerek (lattice, spine, pin) és a horgonypontonkénti skin-súlyok. Szerkeszd a projekt JSON-ját (bármilyen szerkesztővel, egy
  Python-szkripttel vagy a stúdió JSON-paneljén), utána pedig futtasd az `osatura check` parancsot.

A `check`, a `lint`, az `explain` és a `render --bones` az a négy parancs, amelyet riggelés közben folyamatosan használni fogsz.
Az `explain` a leghasznosabb: sorrendben felsorol minden alapértéket, csatornát, kényszert és IK-hozzájárulást, amely
egy csont végső transzformációját létrehozta, így sosem kell találgatnod, miért van valami ott, ahol.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Csontok és hierarchiák

Egy csontnak van `name` neve, `parent` szülője (nincs = gyökér), `length` hossza, `rest_offset` eltolása (a feje hol van, a
**szülő** koordináta-rendszerében; gyökércsont esetén ez a vászon) és a szülőhöz képest fokban megadott `rest_angle` szöge. Konvenciók:

* az y lefelé mutat; a 0 fokos szög a +x irányba néz; a pozitív szögek a képernyőn az óramutató járásával megegyező irányba fordítanak; **a 90 lefelé, a -90 felfelé mutat**.
* a `rest_offset` értékét a szülő világszöge forgatja el. Egy felfelé mutató csont gyermeke, amelynek
  `rest_offset` értéke (70, 0), 70 egységgel *a szülő mentén felfelé* ül, nem pedig 70 egységgel a vászon jobb oldalán.
* A csontokat szülők-előbb sorrendben kell felsorolni. Minden más elutasításra kerül:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Egy csont animációs csatornái a **nyugalmi pózhoz képesti deltákat** adnak meg: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. A 0 kulcsérték azt jelenti: „nyugalmi helyzet”.

Egy minimális, 3 csontos lánc, amelyet a lenti példák többsége használ (`lab.json`; generált, lásd a 4. szakaszt):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Hasznos parancsok: a `reparent <in> <out> <bone> <new_parent|none>` megtartja a nyugalmi pózt (ellenőriztem: a `b3`
feje (220,150)-en marad, ha none szülőhöz kerül), a szkriptbeli `set_parent` ennek alacsonyabb szintű változata, a
`bone-vis <in> <out> <bone> <from> <to>` pedig csak egy képkockatartományra mutat meg egy csontot az átfedésekben.

### Merev rögzítés: a legegyszerűbb rig

Egy réteg merevan ülhet egy csonton a `set_layer("layer", "bone", "bone_name")` hívással. Így készült a
bemutató karakter és az alábbi biped: különálló darabok, mindegyik egy csonthoz rögzítve. Nincs szüksége
súlyokra, és minden exportot átvészel. Akkor válaszd, ha a rajzod már eleve darabokra van vágva.

**Buktató.** A rétegeket akkor rögzítsd, amikor a csontok nyugalmi pózban vannak, és úgy rajzold meg az egyes darabokat, hogy az ízületük
a csontjuk fejére essen. Egy néhány egységgel arrébb rajzolt darab rossz pont körül fog lengeni.

**Buktató.** Ne állítsd a kar csontjainak nyugalmi szögét ±100-ra „T-póz” elérése végett 0 rest offsettel: a
két kar egy pontból indul. Adj minden vállnak oldalirányú `rest_offset` értéket és 180 vagy
0 fokos szöget, ahogyan a bipedben (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Kényszerek

A kényszerek a projekt `constraints` tömbjében szerepelnek, és a csatornák után **a lista sorrendjében** futnak. Hozd létre őket
az `add_aim`, `add_copy_rotation` és `add_pin` hívásokkal (szkript) vagy JSON-ban.

| Kényszer | Szkript | Hatás |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | elforgatja a `bone` csontot, hogy a `target` csont fejére mutasson |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | a `bone` átveszi a `source` világforgatásának (súlyozott részét) |
| Pin | `add_pin(name, bone, target, weight)` | a `bone` fejét a `target` fejére mozgatja |

A 3 csontos lánccal ellenőrizve, a `target` egy (250,40)-nél lévő csont:

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

A súly lineárisan keveri az animált és a kényszerített szöget, így a 0.5 megfelezi a fordulatot.

A szöghatárok a csont tulajdonságai (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Az **animált** forgatást korlátozzák: kulcsold a `b1` csontot 60 fokra, és 20-on megáll:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Buktató: a sorrend számít.** Egy célzott csont *szülőjére* tett copy-rotation kölcsönhatásba lép az aimmel.
Ha egy csont furcsa irányba mutat, futtasd rá és a szülőjére az `explain` parancsot, olvasd végig a hozzájárulások
listáját fentről lefelé, majd rendezd át a `constraints` tömböt.

## 4. Inverz kinematika

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Két csont: analitikus megoldó. A `bend` (+1 vagy -1) választja meg, merre hajlik az ízület.
* Három vagy több csont: FABRIK. Az `iterations` alsó korlát; a megoldó korán leáll, amint a csúcs 0.01 px-en
  belülre kerül, egyébként egy szigorú felső korlátig fut.
* A `mix` az FK/IK keverés (0 = a kulcsolt FK-póz, 1 = teljes IK). Animálható az `ik_mix`
  tulajdonsággal, így vált egy kar FK és IK között. Az `autorig` `mix 0` értékkel adja hozzá a láncokat, hogy a meglévő
  FK-animáció tovább működjön.
* A cél egy csont **feje**; ne adj neki rajzot, és legyen kicsi a hossza (`goal`, `hand_ctrl`).

### Elérhetetlen célok: egyenes, soft, stretch

A `b1..b3` lánc elérése a gyökerétől (60,150) számítva 240. A célt a (330,60) pontra mozgatva, 285 távolságra:

| Beállítás | Eredmény (a csúcs helye) |
|---|---|
| alapértelmezett | teljesen kinyúlik a cél felé, csúcs (287.7, 74.1), pontosan 240-re a gyökértől |
| `"stretch": 2` | a lánc megnyúlik, a csúcs eléri a (330.0, 60.0) pontot |
| `"soft": 0.5` (3 csont) | a lánc meghajlott marad, csúcs (258.8, 83.7); nem egyenesedik ki |

A `soft` (0-tól 0.9-ig) és a `stretch` (1-től 4-ig) az IK-lánc mezői a projekt JSON-jában; nincs hozzájuk
parancs vagy szkriptes beállító:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

A Buddy karján (elérés 80 a (160,140)-en lévő vállból) a célt (250,140)-re téve, 90 távolságra:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

A soft IK egy kétcsontos végtagon aszimptotikusan közelíti a teljes kinyúlást: ahogy a cél 100, 120,
135 és 150 távolságra kerül, a csúcs x-koordinátája 231.1, 234.6, 236.3 és 237.4 értéken áll meg, és sosem éri el a 240-et. Ez a
szándékos „nincs térdugrás” viselkedés. Ha a `stretch` is 1 fölött van, a soft zóna a megnyújtott elérés
(`reach x stretch`) utolsó `soft` hányada: a lánc folyamatosan megnyúlik, és puhán éri el a maximális hosszát (a 7. hullám előtt
a `soft` a teljes elérésen túl figyelmen kívül maradt, és a lánc ott egyenesre ugrott).

**Buktató.** A `lint` az elérhetetlen célokat információként jelzi, nem hibaként:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Olvasd el ezeket a sorokat; a „lebegő” láb szinte mindig ezek egyike.

**Buktató.** A `fuzz` (lásd [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) véletlen pózokat mintavételez, köztük elérhetetlenül
távoli vezérlőpozíciókat is, ezért egy IK-s rigen nagy `ik_unreachable` szám jelenik meg. Ez a mintavételezett tartományról szól,
nem hiba.

## 5. Skinning

Az OSATURA négy skinning-mechanizmust ismer; válassz aszerint, milyen rajzod van.

### 5.1 Vektoros horgonypontok súlyokkal (JSON)

Egy vektoros útvonal horgonypontjai hordozhatnak `weights` listát, amely `{bone, weight}` párokból áll. A
horgonypont ekkor a csontok transzformációinak lineáris keverékét követi. Ez egy 12 horgonypontos csík, amely a három
csonton át van skinelve; a `weights` a `b1`-től a `b2`-n át a `b3`-ig lépcsőzik:

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

A 12. képkockán a `b3` világszöge 50.0 fok, feje pedig a (191.4, 211.3) pontban van (az `explain` szerint). A csík
simán hajlik az ízületek között, mert ott átfednek a súlyok.

**Buktató.** Nincs parancs, amely kiszámolná a vektoros súlyokat: a projekt API-jában van automatikus
vektoros skinner (`Project::auto_skin`), de ez könyvtári függvény, a demó használja, és nincs kivezetve CLI-parancsként.
Vektoros rajzhoz használj merev rögzítést (2. szakasz), vagy írd meg a súlyokat a fentiek szerint.

**Buktató.** Kevés horgonypont, kevés hajlási hely. Egy egyszerű téglalapnak négy horgonypontja van, ezért csak a
sarkainál tud hajlani. Skinelés vagy deformer használata előtt oszd fel a formát (adj horgonypontokat az élek mentén).

### 5.2 Képhálók: `mesh-trace` és `skin-diffuse`

Raszteres rajzhoz kövesd le az alfa-sziluettet háromszöghálóba, majd számítsd ki a súlyokat hődiffúzióval:

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

A `skin-diffuse <project> <layer> <out> [--influences N]` minden hálócsúcshoz legfeljebb N csontsúlyt rendel. Háló
kell hozzá: háló nélküli képrétegen `layer 'flag' has no mesh` üzenettel leáll. Egyetlen csont esetén
minden csúcs 1.0 súlyt kap rajta (a zászlónak egy csontja van, a `mast`), ami helyes, de nem túl
érdekes; legalább két csontot használj, hogy lásd a keveredést. A `mesh-trace` tudja a `hotspot=x,y,r` beállítást is, amellyel
sűrűbbé teheted a hálót ott, ahol hajlást vársz, valamint a követés közbeni skinelés kapcsolóit (`bones=R`, `skin=1`; lásd
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Skin-készletek (öltözékek, kifejezések, sérülési állapotok)

A `skin` parancs más elgondolás: egy **skin-készlet** rétegek egész csoportjait (és opcionálisan
csontokat, kényszereket és IK-láncokat) cseréli együtt.

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

Az `apply` megmutat egy skint, és elrejti az ugyanabba a csoportba tartozó többit. A `schedule` a választást **lépcsős kulcsokkal
a réteg átlátszatlanságán** animálja (a projektben nincs animálható „látható” jelző), ezért olyan rétegeken használd, amelyek átlátszatlanságát egyébként nem animálod.
A nem aktív, skinenként tartozó csontokat nem lehet törölni a rigből az indexek sérülése nélkül, ezért azokat **összecsukjuk**:
egy `bone_scale`-en lévő `multiply` csatorna 0 és 1 lépcsős kulcsokkal nullára állítja a csont méretét, és minden, ami rajta ül
vagy hozzá van súlyozva, egyetlen pontra zsugorodik:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

A validáció által kikényszerített szabályok: egy csont legfeljebb egy skinhez tartozhat; egy skin-csont minden leszármazottja ugyanahhoz a skinhez tartozik; a skin-csontoknak
nem lehet `scale_limits` értékük. A `skin rig <group> <name> <constraint,...|-> <ik,...|->` parancsot olyan kényszerekhez és IK-láncokhoz használd, amelyeknek
csak egy skinnel együtt kell aktívnak lenniük.

### 5.4 Merev darabok plusz egy switch réteg

Diszkréten változó formájú dolgokhoz (szájak, kezek, szemek) tedd az alternatívákat egy `switch` rétegbe,
és kulcsold a `layer_switch` tulajdonságot. Lásd a 11. szakaszt.

## 6. Deformerek

A deformerek egy réteg pontjait mozgatják. A réteg `deformers` mezőjében szerepelnek, csatornákon át animálhatók `"layer::deformer::point"`
alakú, háromrészes céllal, és egy útvonal **horgonypontjaira** vagy egy képháló csúcsaira hatnak. **Ha egy alakzatnak túl kevés horgonypontja van, alig fog deformálódni: előbb oszd fel.**

### Cage (mean-value koordináták)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

A `cage` egy sokszöggel veszi körbe a réteget (`auto` = kitöltött határoló doboz, `hull` = konvex burok, `verts=N` = szakaszok száma
oldalanként); a belül lévő tartalom a sokszög csúcsait követi, a kívül lévő érintetlen marad. A `cage-pose` csúcseltolásokat
kulcsol (`cage_dx`, `cage_dy` csatornák, a cél `strip::cage::1`). A `timeline` felsorolja őket a
„Deformers and paths” alatt. A zárolt csatornát elutasítja.

### Lattice, spine, pin (JSON)

Ennek a háromnak nincs parancsa; add hozzá őket a réteg `deformers` mezőjéhez, és animáld csatornákkal:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: szabad formájú rács. A pont azonosítója `col,row`. Jó squash és stretch effektekhez.
* **Spine**: törött vonal `radius` hatássugárral; egy spine-pont mozgatása meghajlítja a közeli rajzot (farok, csáp).
* **Pin**: bábfigura-szerű pin-warp; minden pin a `radius` sugarán belüli pontokat húzza magával.

Ezek mindegyike helyesen renderelődött a fenti kontaktlapon, csontsúlyok nélküli csíkra alkalmazva. Az az útvonal,
amely *egyben* csontokkal is skinelt, mindkettővel deformálódik; távolítsd el a súlyokat a próbacsíkról, ha egy
deformert önmagában akarsz látni.

### Kijelölés warpolása: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

A `warp-pivot` vezérlőpontok egy kijelölését (cage, lattice, cloth, pin, spine vagy rácsháló) forgatja, méretezi vagy mozgatja egy forgáspont körül
(`x,y`, `bone:NAME`, `centroid`, `center`), és az eredményt kulcsolja. A `radius` lágy kijelölési lecsengést ad hozzá (`linear`, `smooth`, `gauss`, `sharp`, `constant`); a fenti példában az 1-2. csúcsot jelöltük ki, és a
lecsengés hozzáadta a 7. és 8. csúcsot, így 8 csatorna kapott kulcsot. A `soft-scale` arányos átméretezést végez (az `aspect=free` megengedi, hogy a
két tengely különbözzön). A `layer-pivot` a réteg forgáspontját mozgatja a rajz elmozdítása nélkül.

### Háló-zárak és glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: a követő csúcsai
  barycentrikus elmozdulással követik a gazda felületét. Használd tetoválás, folt vagy gomb deformálódó
  ruhán tartására. A ciklusokat elutasítja. A `mesh-lock list` kiírta: `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: két réteg csúcspárjai
  össze vannak hegesztve, így a `b` követi az `a`-t. A `glue list` kiírta:
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Cloth

A cloth **rács** hálót igényel. A követéssel készült szabad formájú hálót elutasítja:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Rácsot a `cols=` és `rows=` megadásával kérhetsz:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Ez a bake 0.43 s-ig tartott debug buildben, és 180 kulcsolt csatornát (6480 kulcsot) eredményezett, így az eredmény teljesen
szerkeszthető, mint bármely más animáció exportálható, és determinisztikus. A `pin=` elfogadja a `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` értékeket, vagy a `...@bone` alakot csonthoz rögzítéshez. A `cols`/`rows` a szimulációs rácsot adja meg; a renderelt kép ezzel van warpolva,
és nagy nagyításnál a PNG-renderben halvány háromszög-varratok látszhatnak.

## 7. Fizika és rugók

### Beledolgozott fizika: `physics`

A `physics` egy csontláncot szimulál (gravitáció, szél, ütközések), és az eredményt hagyományos csontszög-kulcsokba süti.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* A `preset=` értéke `hair`, `tail`, `rope`, `antenna` vagy `cape` lehet.
* A `circle=x,y,r[@bone];...` és a `capsule=x1,y1,x2,y2,r[@bone];...` ütköztetők; a `@bone` hatására az ütköztető követ egy csontot.
* A `drive=bone:gain` hatására egy másik csont mozgása meglöki a láncot; a `vortex=bone|x,y:swirl[:radial[:radius]]`
  örvénylő erőteret ad hozzá; a `follow=bone` és a `lateral=` azt alakítja, hogyan követi a lánc a szülőt.
* A `merge=keep` (alapértelmezett) **nem ír felül** olyan csontot, amelynek már van animációja:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Ha felülírást akarsz, mondd ki: `merge=replace`.
* A `b2,b3/b1` alakú láncokat és az ismeretlen csontot (`no bone 'b9'`) ellenőrzi.

### Élő rugók: `springs`

A `Spring` csillapított utánmozgást ad egy csonthoz kiértékeléskor, bármiféle bake nélkül. Csak JSON-ban érhető el:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

A `frequency` Hz-ben van (magasabb = merevebb), a `damping` arány (1 alatt rezeg, 1 a kritikusan csillapított), a `warmup`
(alapból be van kapcsolva) pedig előre lefuttat egy menetet, hogy a hurkok egyensúlyi állapotban induljanak. Az `explain` ezt mondja:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Ha egy megrajzolt érzethez akarsz igazodni, illeszd rá:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Buktató.** A rugók nem részei a Lottie-nak. Süsd be őket (`physics`), ha olyan formátumba exportálsz, amelyben nincs
megfelelőjük; az `export --report` felsorolja, mi lett közelítve.

## 8. Auto-rig, címkézés és `coach`

Nevezd el a csontjaidat és rétegeidet szabványos szavakkal (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`), és az OSATURA fel tudja
ismerni a szerepüket:

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

Az `autorig <in> <out> [seed]` ezután ezekből a szerepekből nem romboló automatizálást ad hozzá:

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

A projekt 11 csontról és 0 IK-láncról 15 csontra és 4 IK-láncra bővült 3 új csatornával. Minden, ami
hozzáadódott, hagyományos adat: töröld, amit nem szeretnél. A hiányzó részeket indoklással kihagyja
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), ezért olvasd el a jelentést.

**Buktató.** Az `autorig` projektből projektbe dolgozó parancs, és a kimenet lehet a bemenet. Egy fontos fájlnál
írj új névre: az `osatura autorig buddy.json buddy.json` felülírja a `buddy.json` fájlt (az írás atomi, de
nincs visszavonás).

A `coach <project>` elolvassa az animációt, és időzítési és alapelvi javaslatokat ír ki; a `lint` statikusan ellenőrzi a rigt.
Mindkettő tanácsadó jellegű.

## 9. Tükrözés és szimmetria

A `mirror <in> <out>` az összes kulcsolt csatornát tükrözi a függőleges tengelyen: a célok oldalt cserélnek (`L_`/`R_`
előtagok, `_L`/`_R` és `_l`/`_r` utótagok, a left/right szavak), a forgatások és a vízszintes eltolások
előjelet váltanak. A tükrözött csatornák **lecserélik** a projekt csatornáit. A sétáló bipeden a bal comb
`0.0, 10.7, 19.8, 25.9` kulcsai a jobb comb `-0.0, -10.7, -19.8, -25.9` kulcsaivá lettek, és fordítva, a `hips` (oldal nélkül)
pedig megtartotta az értékeit. A `diff` a hat megváltozott csatornát listázza:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Buktató.** Bal/jobb elnevezés nélküli projekten a `mirror` csendben egy változatlan másolatot ír, és azt mondja:
`wrote`. Mindig `diff`-eld az eredményt. Animálás előtt nevezd el következetesen az oldalakat (`_l`/`_r`).

## 10. Nézetváltás és szülőváltás

### Turnaroundok: `view-swap`

A turnaround (elölnézet, háromnegyed, oldalnézet) egy switch réteg nézetenként egy gyermekkel, amelyet egy vezérlőcsont
tulajdonsága hajt:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Kimeneti csatorna: egy `driven` csatorna, `layer_switch` ← `view_ctrl.bone_offset_x`, lépcsős görbével (0 alatt a 0. gyermek,
0-tól a 2. gyermek, 40-től a 4. gyermek). Nézetváltáshoz animáld a `view_ctrl` `bone_offset_x` értékét. A gyermekindexeknek léteznie kell, a rétegnek
pedig switchnek kell lennie (`child index 5 is out of range (the layer has 3)`).

A `slider <in> <out> <name> "at:target.prop=val,...;at:..."` **póz-csúszkát** hoz létre, egy `ctrl_<name>` vezérlőcsontot, amelynek
`offset_x` értéke eltárolt pózok között keveredik:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Szülőváltás az idő során: `parent-switch`

Ha egy tárgyat egyik kézből a másikba kell átadni (vagy egy kelléket felvenni) ugrás nélkül:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Csont-eltolás és szög csatornákat süt be (itt 3 csatornát, minden képkockán egy kulccsal), hogy a csont megtartsa a világbeli
helyzetét a váltáskor. IK-val, kényszerekkel vagy rugókkal rendelkező csontokra nem működik.

## 11. Arcrigek

### Szájformák

A szájformákat egy **switch réteg** gyermekeiként rajzold meg ebben a sorrendben, amelyet minden lip-sync parancs feltételez:
**rest, small, open, round, wide**. A biped szája `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Ha a switchednek kevesebb gyermeke van, a `--map 0,1,2,3,4` (öt szám, formánként egy) választja ki, melyik gyermeket használja az egyes formák; az alapértelmezés
a gyermekek számát követi (1 gyermek: mind; 2: zárt+nyitott; 3: zárt+nyitott+kerek/széles; 4: zárt+kicsi+nyitott+kerek/széles; 5+: ebben a sorrendben).

Rögzítsd a switchet a fej csontjához (`set_layer("mouth", "bone", "head")`), hogy együtt forduljon a fejjel. Az animálása
az [animációs útmutató lip-sync szakaszának](ANIMATION_GUIDE.md#8-lip-sync-és-beszéd) témája.

### Szemek és vágás

Az `eye-clip <in> <out>` az írisz- és pupillarétegeket az azonos oldali szemforma szerint maszkolja, így a pupilla nem hagyhatja el a
szemet. Az `autorig` lefuttatja. Írisz-/pupillarétegek nélküli rigen változatlanul írja ki a fájlt. A pislogás a szemrétegen kulcsolt vagy procedurális
`layer_scale_y`:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

Az `idle blink|breath|sway` seedelt procedurális csatornákat generál (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): ugyanaz a seed minden gépen ugyanazokat a pislogási időpontokat adja.

### Arc vezérlése előadásból

A `face-synth`, a `face-track`, a `face-bake` és a stúdió Face panelje egy landmark-folyamot alakít csatornákká. Ezeket
őszinte pontossági adatokkal együtt az [animációs útmutató arckövetési szakasza](ANIMATION_GUIDE.md#10-arckövetés-és-korlátai) tárgyalja.

## 12. Kamera, snittek és parallaxis

* A `shot <in> <out> <name> <x> <y> <zoom> [rotation]` elment egy elnevezett kamerakeretezést (kameraközéppont és zoom);
  a `camera-seq <in> <out> <frame:shot[:cut],...>` a kamerát az elmentett snitteken át kulcsolja; a `timeline` megmutatja a
  keletkező kamera-csatornákat.
* A `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` egy `parallax` tényezőt oszt el a
  legfelső szintű rétegeken (a távoliak kevésbé mozdulnak, amikor a kamera pásztáz).
* A `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` minden legfelső szintű réteget
  `|parallax - focus|` mértékben elmos; a `dof-export` egy tartományt renderel GIF-be, APNG-be vagy mappába. A fókusz és az erősség a
  `camera_focus` / `camera_blur` csatornákat követi, hacsak nem rögzíted őket.

A sétáló bipeden ellenőrizve:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

A rétegeffektek (blur, drop shadow, tint, fill, outline, posterize...) lehetnek rétegenkéntiek vagy láncként alkalmazhatók: lásd az
[animációs útmutatót](ANIMATION_GUIDE.md#15-rétegeffektek).

## 13. Riggelési ellenőrzőlista

1. Minden szerkezeti változtatás után `check`; animálás előtt `lint`.
2. A nyugalmi póz legyen unalmas és szimmetrikus; a deltákat animáld ehhez képest.
3. A vezérlőket (IK-célok, `view_ctrl`, `root_ctrl`) rajz nélküli csontokként told be a rigbe.
4. Rajzfilmes rajzhoz részesítsd előnyben a merev darabokat, csak azt skineld, aminek hajolnia kell, és deformálás előtt oszd fel a formákat.
5. Ha valami nem stimmel, használd az `explain` parancsot; gyorsabb, mint a találgatás.
6. Süsd be a fizikát és a clothot, ha az animációnak olyan formátumban is túl kell élnie, amelyben ezek a funkciók nincsenek meg, és olvasd el az `export --report` kimenetét.
7. Írj új fájlnevekre, amíg meg nem bízol egy parancsban; a projektből projektbe dolgozó parancsok felülírhatják a bemenetüket.

## 14. Korlátok, amelyeket érdemes ismerned

* A vektoros rajz skinningje **lineáris keverés** (nincs dual quaternion), a súlyokat te adod meg; nincs CLI-s automatikus
  vektoros skinner.
* A lattice, a spine, a pin, az IK `soft`/`stretch` és a rugók csak JSON-ban érhetők el.
* A fizika és a cloth bake-ek: ha módosítod a rigt, süsd be újra. A `physics` nem írja felül a meglévő animációt, hacsak nem kéred.
* A cloth rácshálót igényel; a követéssel készült szabad formájú hálók skinninghez és háló-zárakhoz valók.
* A Lottie és a GIF nem tud mindent ábrázolni (változó szélességű vonalak, nyírt színátmenetek, rugók...). Az
  exportáláskor figyelmeztetéseket ír ki; lásd [EXPORT_FORMATS.md](EXPORT_FORMATS.md) és [LIMITATIONS.md](LIMITATIONS.md).
* Az arcdetektor kizárólag szintetikus arcokon van betanítva (11. szakasz és az animációs útmutató).
