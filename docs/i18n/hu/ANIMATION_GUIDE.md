# OSATURA animációs útmutató

Hogyan mozgass dolgokat, hogyan használd újra a mozgást, hogyan vezéreld hanggal, arccal, mocappel és élő bemenettel, hogyan ellenőrizd,
és hogyan szállítsd kis méretben. Minden itt szereplő parancsot a valódi binárissal futtattam le (`osatura 0.1.0`, debug build, Linux);
a kimenetek ezekből a futtatásokból vannak másolva. A rigging a [RIGGING_GUIDE.md](RIGGING_GUIDE.md) fájlban van; az első projekt a
[GETTING_STARTED.md](GETTING_STARTED.md) fájlban; minden kapcsoló a [CLI_REFERENCE.md](CLI_REFERENCE.md) fájlban.

A dokumentum licence: CC-BY-4.0.

A példák a rigging útmutató 11 csontos bipedjét használják (`biped.json`, 400x360, 24 fps, 24 képkocka): különálló
téglalapok és körök csontokhoz rögzítve, egy öt gyermekes `mouth` switch réteg, egyetlen `script`
hívással felépítve. Bármely hasonlóan elnevezett csontokkal rendelkező rig megfelel.

![Hat képkocka (0, 4, 8, 12, 16, 20) egy téglalapokból álló egyszerű biped generált járásciklusából: a lábak ellentétesen lengenek, a karok ellenlengenek, a csípő bólogat.](../../img/walk-cycle.png)

## 1. Csatornák és kulcsok

Minden, ami mozog, **csatorna**: egy cél (csont, réteg, kamera, deformer-pont), egy tulajdonság, egy
mód és egy forrás.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Források:** `keys` (kulcsképkockák), `expr` (kifejezés), `noise` (fraktál értékzaj a `[-amp, amp]` tartományban, a `freq`
  ciklus/másodpercben) és `driven` (egy görbe, amely egy másik tulajdonság értékét erre képezi le). A bake-elő parancsok
  (physics, walk, clipek, lip-sync...) hagyományos `keys` kulcsokat írnak. Az `osatura timeline` megmutatja minden sáv forrását:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, vagy a kulcsképkockák listáját.
* **Módok:** `replace`, `add` (az alatta lévők tetején) és `multiply`. A csatornák a lista sorrendjében kombinálódnak.
* **Extrapolate** (mit csinál egy sáv az első és az utolsó kulcsán kívül): `hold` (alapértelmezett), `linear` (a végi
  meredekség folytatása), `loop`, `ping_pong` és `loop_offset` (ismétlés, minden ciklusban hozzáadva az első-utolsó változást, ami egy
  járás x-eltolásához illik). Állítsd be így: `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; téves név esetén kilistázza az érvényeseket.
* A csont-tulajdonságok **a nyugalmi pózhoz képesti deltákat** adnak: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. A réteg-tulajdonságok közé tartozik a `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` és `layer_switch`. A deformerek a `cage_dx`... tulajdonságokat használják, mint a rigging útmutatóban.

Kulcsokat létrehozhatsz szkriptből (`set_key(target, prop, frame, value[, ease])`), CLI-ből a `keyedit add` paranccsal, a
stúdióból vagy JSON-ban.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

Az `osatura timeline <project> [--json]` minden sávot csont/réteg szerint listáz a kulcsképkockáival; az `osatura explain <project> <bone|layer> <frame>`
megmutatja, hogyan számolódott ki egy képkocka.

## 2. Interpoláció és easing

Minden kulcs annak a szakasznak az easingjét tárolja, amely **ennél a kulcsnál indul**. A paraméter nélküli easing-nevek:
`linear`, `step` (az érték megtartása a következő kulcsig), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (egy Catmull-Rom
spline a szomszédos kulcsokon át: sima, túllövés-szabályozás nélkül) és `bounce`. JSON-ban az ease egy objektum:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

A `keyedit ... ease --sel SEL --ease E` elfogad egyszerű nevet, `bezier:x1,y1,x2,y2` alakot (`cubic_bezier`-ként tárolva) vagy JSON-objektumot a
`spring` és `back` esetén; mind a négy alakot alkalmaztuk a járásciklus 17 csípőkulcsára hiba nélkül. A dope-sheet CLI-ben a kijelölési
szintaxis, a `SEL`: `target:prop@first..last`, `target:prop` vagy `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Add meg kimenetként a `-` jelet, hogy bármely `keyedit` **próbafuttatás** legyen, amely JSON-ként kiírja a tervet, és semmit sem ír ki fájlba:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Buktató.** A `keyedit` elutasítja azokat a szerkesztéseket, amelyek kivinnék a kulcsokat a projekt képkocka-tartományából, hacsak nem adod meg a `--free` kapcsolót:
`scale: frame 27 is outside the project range 0..=24` (ugyanez a parancs `--free` kapcsolóval ezt írta: `scale: 17 keys changed`, a
többletkulcsok pedig az utolsó képkockán túl ülnek, ahol a `lint` `L007`-et jelent). A zárolt csatornákat is elutasítja:
`channel hand_ctrl.BoneOffsetY is locked` (lásd `chan-meta ... lock=on`).

A stúdió görbeszerkesztője ugyanezeket a görbéket rajzolja, húzható érintőkkel (`handle`, `smooth`; lásd
[STUDIO.md](STUDIO.md)). A kulcsoknak opcionális bejövő/kimenő érintőfogantyúi vannak (`free`, `aligned` vagy `auto`).

## 3. A dope sheet parancssorból: `keyedit`

Mindaz, amit a stúdió idővonala tud, létezik `keyedit` alparancsként, egyetlen visszavonás-biztos kötegként alkalmazva:

| Alparancs | Hatás |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | kulcsok eltolása időben és/vagy értékben |
| `scale --sel SEL --pivot F --factor X` | átidőzítés egy képkocka körül |
| `dup --sel SEL --offset N` | másolás |
| `delete --sel SEL` | törlés |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | ease, érték beállítása vagy automatikus simítás |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | vágólap projektek között |
| `add --target T --prop P --at F [--value V] [--ease E]` | kulcs beszúrása |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | idő beszúrása vagy eltávolítása |
| `offset --channels 0,1 --dt N` | egész csatornák eltolása |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | ismétlés az utolsó kulcs után |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | görbeszerkesztő érintőfogantyúja |
| `lock --target T [--prop P] [--value on\|off]` | csatorna zárolása |

## 4. Hurkok

Egy N képkockás projekt a 0-tól az N-1-ig terjedő képkockákat játssza újra és újra, ezért az **N. képkockának ugyanannak a pózának kell lennie, mint a 0.-nak, és sosem jelenik meg
kétszer**. Három parancs segít:

* a `keyedit ... loop --channels 0,1` `extrapolate: loop` értéket állít be a csatornákon (a kulcsokhoz nem nyúl):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* a `loop-solve <in> <out> [--window 24] [--inclusive]` a kulcsolt csatornákat varratmentesen ismétlődővé teszi, és jelenti a varratot:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* a `make-loop <in> <out> [targets|all] [props] [--inclusive]` **eltávolítja az elmozdulást**, hogy a mozgás a helyén működjön:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Az a labda 23 képkocka alatt 280 egységet haladt jobbra. A `make-loop` tökéletes, helyben ismétlődő hurkot csinált belőle... egy olyan labdából, amely többé nem
mozog (a lint ezután ezt jelzi: `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
Ezt jelenti az „eltávolítja az elmozdulást”: helyben mozgó járásciklusokhoz használd, a teljes karaktert pedig egy
`follow` útvonallal vagy kamerával told el (9. szakasz). A `loop-solve` nem javítja az elmozdulást: ugyanezen a labdán ezt jelenti: `seam 280.000 -> 280.000`.
Ha azt akarod, hogy az elmozdulás a ciklusokon át **folytatódjon**, add a csatornának a `loop_offset` extrapolációt (1. szakasz): az `explain` ekkor a labdára a 10. képkockán ezt jelenti:
`keys → 121.739 (2 key(s), LoopOffset outside range)`.

Az `--inclusive` a régebbi konvenciót választja, amelyben az utolsó megjelenített képkocka megegyezik az elsővel.

**Buktató: a zaj nem ismétlődik.** A `noise` csatorna nem periodikus. Ismétlődő hullámzáshoz használj `t`-ből képzett kifejezést
(0..1 a klip alatt), például `8 * sin(t * 2 * pi)`-t, vagy tudatosan szakítsd meg a hurkot.

## 5. Kifejezések, zaj és driverek

Egy **kifejezés**-csatorna minden képkockán kiértékel egy kis képletet. Változók: `frame`, `time` (másodperc), `fps`, `frames`,
`pi` és **`t` = 0..1 a klip alatt**. Operátorok: `+ - * / % ^`, összehasonlítások, `&& || !`, `?:`. A függvények közé tartozik a
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (zaj; két vagy három argumentummal, például `wiggle(3, 5, 7)`)
és `get("target","prop")`, amely egy másik csatorna aktuális értékét olvassa (a csatornák a lista sorrendjében futnak, a későbbiek látják a korábbiak eredményét).

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

Az `explain ex1.json head <frame>` eredménye: a fej világszöge -90.0 a 0., 12. és 24. képkockán, **-82.0 a 6. képkockán**, és
-98.0 a 18. képkockán (egy teljes szinusz a klip alatt; a fej nyugalmi szöge -90 világfok). A csípőn lévő zaj
a `hips` csontot egy egység törtrészével mozgatja: a feje a 0. képkockán y 169.7-en, a 24.-en 170.5-en ül (zaj nélkül 170.0), így nem ismétlődik (lásd fent).

A hibákat még az írás előtt elkapja:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Buktató.** Egy kifejezésben a `t` normalizált (0-tól 1-ig), nem másodperc és nem képkocka. A `sin(t * 6.2832 / 24)` jónak tűnik,
mégis rossz: alig mozog. Használd a `time` (másodperc) vagy a `frame` változót.

### Driverek, csúszkák és adatkötések

* A `slider` és a `view-swap` (rigging útmutató) vezérlőcsontokat és `driven` csatornákat hoz létre.
* A `bind` egy külső értékútvonalat (játékból, felhasználói felületről vagy tesztből) egy tulajdonsághoz vagy állapotgép-bemenethez köt, átskálázással,
  simítással és opcionális kifejezéssel:

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

A 0..1 tartományon a `0.75` a -20..20 tartományra leképezve 10, ahogy látható. A `bind-eval --data out.json` ugyanezeket a számokat JSON-ként írja ki.

## 6. Járásciklusok és a mozgáskönyvtár

A `walk` procedurális járást generál: a lábak és (opcionálisan) a karok és a csípő csontneveit kéri.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

A `--period` a ciklus hossza képkockában, a `--stride` a comb lengése fokban, a `--knee` a térdhajlítás, a `--bob`
a csípő bólogatása pixelben, a `--swing` pedig a kar lengése. A hét csatornának 1,5 képkockánként van kulcsa, hurkolódnak
(`extrapolate: loop`), és 0 észrevétellel mennek át a `lint`-en. Mivel a ciklus helyben generálódik, nincs elmozdulás,
amit el kellene távolítani.

### Take-ek és klipek

A **take** egy rögzített animációdarab, amelyet egy projektek között újrahasználható **könyvtár**-fájlban tárolsz:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

A `take-record` utolsó argumentumai a felvételbe kerülő célnevek előtagjai. A `take-apply` egy képkockától kezdve írja be a take-et,
opcionális `from=to` átnevezésekkel, egy `speed` értékkel és a `--blend FRAMES` kapcsolóval a meglévő animációval való átúsztatáshoz.

### A klipverem: rétegezés és súlyok

A **klip** egy módot, súlyt és maszkokat kapott take az idővonalon. A klipek sorrendben rétegződnek; a későbbiek a korábbiak fölé kerülnek.
Példa: egy járás, plusz egy integető jobb kar, amely be- és kiúszik.

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

A verem sosem nyúl a kézzel készített vagy a zárolt csatornákhoz. Az `--include`/`--exclude` névmintákat fogad,
a `--track-weight arm_*=1,leg_*=0.2` a sávokat egyenként súlyozza, a `--mode additive|multiply` pedig felváltás helyett rétegez.

**Buktató.** A `clip bake` írja ki a csatornákat: a klipek módosítása után futtasd újra (a parancs is ezt mondja).

## 7. Állapotgépek és kötések

Az **állapotgép** futásidőben, bemenetekből (logikai értékek, számok, triggerek) választ klipek között. JSON-ban írják le, és
hozzáadáskor ellenőrzik:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Az állapotok egy klipet, egy képkocka-`segment`et, egy jelölő-`region`t vagy több klip 1D `blend` keverékét játsszák; az átmenetek akkor sülnek el, ha az összes
`when` feltétel (`input op value`, ÉS) teljesül, opcionálisan csak az `exit_time` után (az állapot hosszának 0-tól 1-ig terjedő hányada); a sorrendben
első illeszkedő átmenet nyer; a `"from":"*"` bármely állapotot jelenti.

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

A `machine-sim` időzített bemeneti eseményeket játszik le, és állapotnaplót (`sim.json`) meg SVG-képkockákat ír, így az állapotgépek runtime nélkül is tesztelhetők
és diffelhetők. Az állapotgép a bemeneti naplójának tiszta függvénye: ugyanaz a szkript mindig ugyanazt az eredményt adja.

## 8. Lip-sync és beszéd

Minden hangbemenetnek **WAV**-nak kell lennie; más formátumokat előbb alakíts át az ffmpeg-gel. A projektnél hosszabb hangot **levágja** a
projekt hosszára; előbb hosszabbítsd meg a projektet (a fájlban a `frames` mezőt), különben kevesebb képkockát kapsz, mint amennyi a hang.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

A projekt egy 72 képkockás (3 s) másolatán:

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* A `lipsync` a szájformákat kizárólag a hang **hangerejéből** választja.
* A `lipsync-text` az átiratodat (`Hello there, nice to meet you today`) a hang energiájához igazítja, és a szöveget használja a
  formákhoz. **Nem fonémafelismerés**; az eszköz ezt minden alkalommal kiírja. A `--words-out` a szavak és szótagok időzítését írja ki
  JSON-ként vagy SRT-ként: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Nyelvek: `--lang ro|en`.
* A `speech-motion` a szavakra időzített fejbólintásokat, esést és billentést ad hozzá (additív).
* A formák sorrendje: rest, small, open, round, wide (a `--map` a te switchedre képezi át).

### Kézzel készített jelzések: `mouth`

Ha egy másik eszközből vannak jelzéseid, a `mouth <project> <cues.txt> <switch_layer> <out>` **képkocka-számozott** sorokat
(Moho switch stílus: `frame PHONEME`) vagy **másodperceket** (Rhubarb TSV) fogad: a **tizedesponttal írt idő
másodperc, a csupasz egész szám képkocka**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Az alakbetűk az öt szájosztályra képeződnek: `X A MBP rest` a restre; `B G H L FV etc` (és az ismeretlen betűk) a smallra; `D AI` az openre;
`F O U WQ` a roundra; `C E` a wide-ra. A `--rhubarb` egyetlen dolgot változtat: az `E` betűt ekkor a Rhubarb kerek formájaként (round) kezeli, nem
a Papagayo széles formájaként. Itt az `X C B D A` ezt adta: rest, wide, small, open, rest.

**Buktató.** A `0.00 X` a 0. másodperc; a `0 X` a 0. képkocka lenne. Egy fájlban keverni őket szabályos, de zavaró.

### Hangra reagálás és ütemek

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

A `sound mix` is a projekt hosszára vág (egy 3 s-os hang egy 1 s-os projektben 1.00 s lett). Az `export` `.html`-be mp3-ként ágyazza be a kevert
hangot (ennek a projektnek az oldala 50 KB-ról 131 KB-ra nőtt), amihez `ffmpeg` kell; nélküle az oldal hang nélkül íródik ki. A `.mp4`,
`.webm` és az mp3/ogg keverés szintén `ffmpeg`-et igényel.

## 9. Mozgatás útvonalak mentén, utánmozgás, talajra rögzítés

* **Útvonalkövetés:** rajzolj egy vonalréteget, majd a `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` kulcsolja a pozíciót
  és (hacsak nincs `no-orient`) a forgatást az útvonal mentén:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(Az `add_line` három vagy több pontot igényel: `points need an even number of coordinates: 3..1000 points`.)

* **Lánc utánmozgás:** a `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` egy driver mozgását egy
  követőláncra másolja, mindegyik későbbi és gyengébb (átfedő mozgás, farkak, csápok):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  A követők kulcsai a késleltetéssel későbbre tolódnak, így néhány az utolsó képkockán túlra kerül (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`), és a másolt ismétlődésre támaszkodnak.
* **Foot-lock:** a láb a talajon marad, miközben a test mozog. **Gyökérvezérlőt** (szülő nélküli csontot) és
  IK-lábcélt igényel:

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

  A `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` megkeresi azokat a képkockákat, ahol a vezérlő érinti a talajt,
  és kompenzáló csatornákat ad hozzá (`add` módban), hogy ne csússzon. Szülő nélküli vezérlő nélkül leáll:
  `foot lock needs a root controller bone (no parent)`.
* **Kapcsolódó bake-ek:** `stretch <in> <out> <follower> <leader>` (a követő a vezetőre céloz és megnyúlik felé), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (közelségi vonzás), `bone-vis` (átfedések láthatósága).

## 10. Arckövetés és korlátai

**Légy őszinte azzal kapcsolatban, mi ez.** Az OSATURA saját kis arcdetektort és landmark-modellt tartalmaz (23 landmark, az elrendezés neve
`osatura23`). A binárisba van építve, helyben fut, nem igényel letöltést, és **kizárólag olyan arcokon tanították, amelyeket
a projekt saját szintetikus generátora állított elő**: stilizált és rajzfilmes fejeken. **Fényképeken, webkamerán vagy valódi
videón nem validálták.** A mért számok (`osatura face-eval --eval-n 50 --seed 9`):

| Mérték | Eredmény |
|---|---|
| Félretett szintetikus arcok, észlelési arány | 1.0 (50-ből 50) |
| Landmark-hiba (NME) átlag / medián / 90. percentilis | 0.028 / 0.026 / 0.046 |
| Hamis pozitívok 50, arc nélküli jeleneten | 2 |
| Rajzfilmes dizájnok, NME ahol észlelve | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Rajzfilmes dizájnok, észlelve | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Ezek a számok a *saját szintetikus stílusára* vonatkoznak. Ne olvasd őket pontosságnak egy kamera előtt ülő emberen. Ha
valódi arckövetés kell, állíts elő landmarkokat egy általad megbízhatónak tartott eszközzel, és add át őket NDJSON-ként (a `face-bake` bemeneti formátuma
nyílt); a stúdió Face panelje azt várja, hogy kameraforráshoz te adj meg egy `window.osaturaLandmarkProvider` függvényt.

A folyamat:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

egy leképezőfájllal (itt `facemap.json`):

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

Jelazonosítók: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Téves azonosító esetén kilistázza az érvényeseket: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
A `face-bake` az első képkockákból kalibrálja a semleges arcot (`--calibrate-frames`, `--calib`).

**Tudnivalók.** A bake **a projekt hosszára van vágva**: 60 landmark-képkocka egy 24 képkockás projektre
csak a 0-tól 23-ig terjedő kulcsokat eredményezte, és egy stderr-figyelmeztetés megmondja, hány felvételi képkocka nem lett beszúrva (a projekt hossza nem változik). Az összegző sor az új csatornákhoz a `(K keys in them)` részt mutatja; az `N trigger keys set` a már meglévő csatornákba írt kulcsokat számolja. A `face-detect <image> --overlay o.png` megmutatja, mit talált a detektor egy állóképen.

## 11. Mocap és BVH

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

* Egy 2D rig a 3D mozgásnak csak egy vetületét használhatja: a `--view front|side|top|yaw=DEG` választja ki.
* A `mocap-match` bizonyossági táblázatot ír ki (a fenti `shin` sorok megjegyzést kaptak, mert a végtaghosszuk 2,1-szer tér el a mediántól).
  A `--map map.json` felülbírálja a tippet, a `--min-conf C` pedig megadja azt a bizonyossági szintet, amely alatt egy ízületet nem használ.
* **A `mocap-retarget` meghosszabbítja a projektet**, hogy elférjen a klip (24-ből 48 képkocka lett; egy megjegyzés közli, a `--keep-length` pedig inkább a klipet vágja le). Az `--in-place` eltávolítja a gyökér elmozdulását,
  a `--footlock BONE --ground Y` lábat rögzít (a szülő nélküli gyökérvezérlőt igényli, mint a `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  a `--mirror` pedig tükrözi a mozgást.
* Az `import-bvh` a régebbi, explicit forma `--joint-map Joint=bone,...` kapcsolóval.

## 12. Élő bemenet (MIDI, egér, gamepad, billentyűk)

Rögzíts egy előadást a stúdió Live paneljén, vagy játssz le egy felvételt vagy MIDI-fájlt offline:

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

A módosítókerék (CC 1 az 1. csatornán) 24 beszúrt kulcson át söpörte a kart 0-tól 160 fokig, a 0,5 s-nál lévő note on pedig a `cymbal`
projekt-eseménnyé lett a 12. képkockán. A források: `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` és `face`. A MIDI azonosítók alakja `"<channel>:<number>"`, a csatornák 1-től számozódnak. A Web MIDI a
stúdióban olyan böngészőt igényel, amely támogatja (lásd [STUDIO.md](STUDIO.md)); az offline parancsnak csak a fájl kell.

## 13. Elrendezési konténerek és görgetés

Az elrendezési konténerek a rétegeket más vászonméretre rendezik újra (reszponzív UI-animáció); a görgetési területek korlátozott vagy rugalmas
görgetést adnak. A konténerek JSON-ban vannak, a megoldó pedig determinisztikus:

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

A `dx`/`dy` az animáció tetejére alkalmazott eltolás. A konténerek, valamint a `scroll add|list|remove|eval` (rugalmas határok, rögzítési pontok, fling-
szimuláció) részletei a [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval) fájlban vannak. Az elrendezést a runtime és az előnézetek alkalmazzák;
**az exportált fájlok a projekt saját méretére megoldott elrendezést tartalmazzák.**

## 14. Kamera, snittek, mélységélesség

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

A `timeline sh3.json` 11 csatornát mutat: a járás 7-ét plusz a kamerájét. A `dof` minden legfelső szintű réteget `|parallax - focus|` mértékben
elmos (az erősséget és a sugarat a `--max-radius` korlátozza, alapértelmezés 24). A mélységélesség utófeldolgozású render; a `dof-export` GIF-be,
APNG-be vagy mappába írja.

## 15. Rétegeffektek

Az effektek egy réteg rendezett listáját alkotják (blur, drop shadow, tint, fill), egy elszigetelt menetben renderelődnek a réteg maszkja, átlátszatlansága és keverési
módja előtt, rétegenként legfeljebb 16. A számaik csatornákkal animálhatók, amelyek célja `"<layer>::fx<index>"`, tulajdonsága pedig az `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` egyike (a színek nem animálhatók):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

A `check` átmegy, és az árnyék a hurok alatt -6-ról +6-ra csúszik. Az **utóeffekt-lánc** (outline, posterize, relight normáltérképekkel...) egy
képkockát renderel egy JSON-listából:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Biztonsági ellenőrzések villogásra és kameramozgásra (mérnöki segédeszköz a nyilvános WCAG 2.3.1 definíción alapulva, **nem műsorszórási tanúsítvány**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin és nyomvonalak (átfedések)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

A `--keyed` a szellemeket a szomszédos *kulcs*képkockáknál rajzolja, nem fix lépésközzel. Az átfedések sosem módosítják a projektet, és nem részei az exportnak.
A stúdió élőben mutatja őket (`Alt+N` onion, `Alt+M` nyomvonalak, `Alt+7` panel).

## 17. Procedurális idle-ök

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Seedelt, ezért ugyanaz a parancs bárhol ugyanazt az animációt adja. Az `autorig` magától hozzáadja a légzést és a pislogást, ha a nevek egyeznek.

## 18. Minőségbiztosítás: check, lint, fuzz, CI

* A `check` pontosan úgy fordítja le a projektet, ahogyan a renderelő fogja. Hiba esetén kilépési kód: 1.
* A `lint` statikus problémákat talál (hibák esetén kilépési kód 1; az információk és figyelmeztetések nem buknak el): elérhetetlen IK-célok (`L003`), olyan csatornák, amelyek
  sosem változnak (`L005`), az idővonalon kívüli kulcsok (`L007`)...
* Az `explain`, a `coach`, a `diff a.json b.json` (`+` hozzáadva, `-` eltávolítva, `~` módosítva) és a `vcs` (verziókezelés szemantikus merge-dzsel projektfájlokhoz).
* A **fuzz** véletlenszerűen pózolja a rigt, és jelenti, hol romlik el:

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

  **A sima `fuzz` jelentés, és 0-val lép ki; add meg a `--gate` kapcsolót (bármely hiba esetén 1-gyel lép ki) vagy a `--max-fail-rate R` kapcsolót (egy tört), hogy kapuként működjön.** A mintavételezési tartomány minden paraméter teljes tartománya, ezért egy IK-s rig
  szinte az összes mintán elbukik (`98.0%`, 50-ből 48 `ik_unreachable`, a talajra rögzített bipeden). Szűkítsd az `--only`, `--props`,
  `--margin` és `--ignore KINDS` kapcsolókkal, válaszd meg a megengedettnek tartott tartományt, és állítsd be az elfogadott küszöböt.
  Segít a `--report out.json`, a `--heatmap out.svg` és a `--minimize` (a legkisebb reprodukáló póz). Stratégiák: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Golden-ök és CI.** A `golden write walk.json gold.json` képkocka-aláírásokat tárol; a `golden check` alapból pontosan hasonlít össze (bármely
  változásnál 1-gyel lép ki; a `--tol T` lazítja). Egy *másik* animáció ellenőrzése ellenük kilistázza az eltérő képkockákat: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  A `ci` ezt és még többet egy manifestbe foglal:

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

  A `ci run` 0-val lép ki, ha minden átmegy, 1-gyel, ha egy ellenőrzés elbukik, és 2-vel hiba esetén. A `ci bless` az aktuális rendereket fogadja el új golden képkockáknak,
  a `ci diff a.png b.png --metric ssim|de2000|de76|mse` hőtérképekkel magyaráz meg egy változást. A manifestben szereplő 46 %-os pose-fuzz küszöböt a
  projekt „jelenlegi állapotára” kalibrálták (az eszköz kiírja ezt a megjegyzést); szigorítsd tudatosan. A részletek a [CI.md](CI.md) fájlban vannak.

## 19. Méret és részletességi szint

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

A `size-report` megmutatja, mely rétegek, képek és csatornák költenek bájtokat. Ha egy platformnak keretösszege van, az `export-fit` megtalálja a legjobb minőségű, még belefér exportot:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

A második parancs **1**-gyel lépett ki, és mégis kiírta a talált legkisebb fájlt (14952 bájt). A keretösszeget alapból a nyers méreten ellenőrzi;
a `--measure gzip` a tömörített méretet ellenőrzi (a fenti Lottie gzip-pel 1429 bájt). A keresést a `--max-seconds` korlátozza (alapértelmezés 60 s valós idő);
a debug buildben a GIF-keresést ez a korlát állította meg, és a legjobb eredményét megtartotta.

Sok görbét és hálót tartalmazó rajzokhoz a **részletességi szint (level of detail)** egyszerűsített másolatokat készít *deklarált* hibahatárral, kimeneti pixelben:

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

A `--verify` a valódi eltérést méri (1-gyel lép ki, ha meghaladja a határt); a fenti futtatásban minden mért hiba a deklarált határ alatt maradt. Figyeld meg, milyen
keveset spórol egy apró, téglalapokra épülő projekt (17507-ről 16080 bájtra): a LOD részletgazdag rajzokon térül meg, ezen nem.

## 20. Őszinte korlátok

* A lip-sync energia plusz szövegigazítás, nem fonémafelismerés.
* Az arcdetektor kizárólag szintetikus (10. szakasz): egymagában nem teszi használhatóvá a webkamerás előadást.
* A `make-loop` eltávolítja az elmozdulást; a `loop-solve` nem javítja; a zaj nem ismétlődik.
* A hang a projekt hosszára van vágva; a `mocap-retarget` meghosszabbítja a projektet.
* A Lottie, a GIF és a HTML nem tud minden funkciót hordozni; az exportálási figyelmeztetések és a `--report` felsorolják, mi lett közelítve ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* A stúdiót kizárólag Chromiumban tesztelték ([STUDIO.md](STUDIO.md)).
