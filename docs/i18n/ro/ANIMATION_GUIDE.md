# Ghid de animație OSATURA

Cum să pui lucrurile în mișcare, să reutilizezi mișcarea, să o comanzi din audio, fețe, mocap și intrare live, să o verifici și
să o livrezi la dimensiuni mici. Fiecare comandă de aici a fost rulată cu binarul real (`osatura 0.1.0`, build de debug, Linux);
ieșirile sunt copiate din aceste rulări. Rigging-ul se găsește în [RIGGING_GUIDE.md](RIGGING_GUIDE.md); primul proiect în
[GETTING_STARTED.md](GETTING_STARTED.md); fiecare opțiune în [CLI_REFERENCE.md](CLI_REFERENCE.md).

Licența acestui document: CC-BY-4.0.

Exemplele folosesc biped-ul cu 11 oase (`biped.json`, 400x360, 24 fps, 24 de cadre) din ghidul de rigging: dreptunghiuri și cercuri
separate, atașate oaselor, un strat switch `mouth` cu cinci copii, construit printr-un singur apel `script`.
Funcționează orice rig cu oase denumite asemănător.

![Șase cadre (0, 4, 8, 12, 16, 20) din ciclul de mers generat al unui biped simplu făcut din dreptunghiuri: picioarele se balansează în opoziție, brațele se contra-balansează, șoldurile se mișcă sus-jos.](../../img/walk-cycle.png)

## 1. Canale și chei

Tot ce se mișcă este un **canal**: o țintă (os, strat, cameră, punct de deformator), o proprietate, un
mod și o sursă.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Surse:** `keys` (chei de animație), `expr` (o expresie), `noise` (zgomot fractal de valoare în `[-amp, amp]`, `freq` în
  cicluri pe secundă) și `driven` (o curbă care transformă valoarea altei proprietăți în aceasta). Comenzile care coc (bake)
  (physics, walk, clips, lip-sync...) scriu `keys` obișnuite. `osatura timeline` arată sursa fiecărei piste:
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, sau lista cadrelor cu chei.
* **Moduri:** `replace`, `add` (peste ce se află dedesubt) și `multiply`. Canalele se combină în ordinea listei.
* **Extrapolate** (ce face o pistă în afara primei și ultimei chei): `hold` (implicit), `linear` (continuă panta de la
  capăt), `loop`, `ping_pong` și `loop_offset` (repetă, adăugând la fiecare ciclu variația dintre prima și ultima cheie, ceea ce se potrivește unui
  offset x de mers). Setează-l cu `keyedit ... extrapolate --channels 0 --extrapolate loop_offset`; un nume greșit listează numele valide.
* Proprietățile oaselor sunt **delte față de poziția de repaus**: `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. Proprietățile stratului includ `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` și `layer_switch`. Deformatoarele folosesc `cage_dx`... ca în ghidul de rigging.

Creează chei dintr-un script (`set_key(target, prop, frame, value[, ease])`), din CLI cu `keyedit add`, din
studio sau în JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` listează fiecare pistă pe os/strat, cu cadrele ei cu chei; `osatura explain <project> <bone|layer> <frame>`
arată cum a fost calculat un cadru.

## 2. Interpolare și easing

Fiecare cheie stochează easing-ul segmentului care **începe** la acea cheie. Numele de easing care nu au nevoie de parametri sunt
`linear`, `step` (ține valoarea până la următoarea cheie), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (un spline Catmull-Rom
prin cheile învecinate: neted, fără control al depășirii) și `bounce`. În JSON un easing este un obiect:

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` acceptă un nume simplu, `bezier:x1,y1,x2,y2` (stocat ca `cubic_bezier`) sau un obiect JSON pentru
`spring` și `back`; toate cele patru forme au fost aplicate celor 17 chei ale șoldului din ciclul de mers fără erori. În CLI-ul dope sheet, sintaxa
de selecție `SEL` este `target:prop@first..last`, `target:prop` sau `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Adaugă `-` ca ieșire pentru a face din orice `keyedit` o **rulare de probă** (dry run) care afișează planul ca JSON fără să scrie nimic:

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Capcană.** `keyedit` refuză editările care ar scoate cheile din intervalul de cadre al proiectului, dacă nu transmiți `--free`:
`scale: frame 27 is outside the project range 0..=24` (aceeași comandă cu `--free` a afișat `scale: 17 keys changed`, iar
cheile suplimentare se află apoi dincolo de ultimul cadru, unde `lint` raportează `L007`). Canalele blocate sunt refuzate și ele:
`channel hand_ctrl.BoneOffsetY is locked` (vezi `chan-meta ... lock=on`).

Editorul de grafice al studioului desenează aceleași curbe, cu tangente care pot fi trase (`handle`, `smooth`; vezi
[STUDIO.md](STUDIO.md)). Cheile au opțional mânere de tangentă de intrare/ieșire (`free`, `aligned` sau `auto`).

## 3. Dope sheet din linia de comandă: `keyedit`

Tot ce poate face timeline-ul studioului există ca sub-comandă `keyedit`, aplicată ca un singur lot (batch) sigur la undo:

| Sub-comandă | Efect |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | deplasează cheile în timp și/sau valoare |
| `scale --sel SEL --pivot F --factor X` | schimbă timingul în jurul unui cadru |
| `dup --sel SEL --offset N` | duplică |
| `delete --sel SEL` | șterge |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | setează easing-ul, valoarea sau netezirea automată |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | clipboard între proiecte |
| `add --target T --prop P --at F [--value V] [--ease E]` | inserează o cheie |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | inserează sau elimină timp |
| `offset --channels 0,1 --dt N` | glisează canale întregi |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | buclă după ultima cheie |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | mâner de tangentă în grafic |
| `lock --target T [--prop P] [--value on\|off]` | blochează un canal |

## 4. Bucle

Un proiect de N cadre redă cadrele de la 0 la N-1 în mod repetat, deci **cadrul N trebuie să fie aceeași poziție ca cadrul 0 și nu este afișat
niciodată de două ori**. Trei comenzi ajută:

* `keyedit ... loop --channels 0,1` setează `extrapolate: loop` pe canale (cheile rămân neatinse):

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` face canalele cu chei să facă buclă fără cusătură și raportează cusătura:

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **elimină deplasarea** astfel încât mișcarea să funcționeze pe loc:

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Mingea aceea s-a deplasat 280 de unități spre dreapta în 23 de cadre. `make-loop` a transformat-o într-o buclă perfectă pe loc... a unei mingi care nu se mai
mișcă (lint raportează atunci `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
Asta înseamnă „elimină deplasarea": folosește-l pentru cicluri de mers care se desfășoară pe loc, apoi mută întregul personaj cu o
cale `follow` sau cu o cameră (secțiunea 9). `loop-solve` nu rezolvă deplasarea: pe aceeași minge raportează `seam 280.000 -> 280.000`.
Dacă vrei ca deplasarea să **continue** de la un ciclu la altul, dă canalului extrapolarea `loop_offset` (secțiunea 1): `explain` raportează atunci
`keys → 121.739 (2 key(s), LoopOffset outside range)` pentru minge la cadrul 10.

`--inclusive` selectează convenția mai veche, în care ultimul cadru afișat este egal cu primul.

**Capcană: zgomotul nu face buclă.** Un canal `noise` nu este periodic. Pentru o oscilație în buclă folosește o expresie în `t`
(0..1 pe durata clipului), de exemplu `8 * sin(t * 2 * pi)`, sau rupe bucla în mod deliberat.

## 5. Expresii, zgomot și drivere

Un canal de tip **expresie** evaluează o formulă mică la fiecare cadru. Variabile: `frame`, `time` (secunde), `fps`, `frames`,
`pi` și **`t` = 0..1 pe durata clipului**. Operatori `+ - * / % ^`, comparații, `&& || !`, `?:`. Funcțiile includ
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (zgomot; două sau trei argumente, de exemplu `wiggle(3, 5, 7)`)
și `get("target","prop")`, care citește valoarea curentă a altui canal (canalele rulează în ordinea listei, iar cele de după văd rezultatele celor dinainte).

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

Rezultatul `explain ex1.json head <frame>`: unghiul global al capului este -90.0 la cadrele 0, 12 și 24, **-82.0 la cadrul 6** și
-98.0 la cadrul 18 (o sinusoidă completă pe durata clipului; unghiul de repaus al capului este -90 de grade globale). Zgomotul de pe șolduri
mișcă `hips` cu o fracțiune de unitate: capul lui se află la y 169.7 la cadrul 0 și 170.5 la cadrul 24 (170.0 fără zgomot), deci nu face buclă (vezi mai sus).

Greșelile sunt prinse înainte de a se scrie ceva:

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Capcană.** Într-o expresie, `t` este normalizat (0 până la 1), nu secunde și nu cadre. `sin(t * 6.2832 / 24)` pare corect
și este greșit: abia se mișcă. Folosește `time` pentru secunde sau `frame`.

### Drivere, slidere și legături de date

* `slider` și `view-swap` (ghidul de rigging) creează oase controler plus canale `driven`.
* `bind` conectează o cale de valoare externă (dintr-un joc, o interfață sau un test) la o proprietate sau la o intrare a unei mașini de stări, cu remapare,
  netezire și o expresie opțională:

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

`0.75` pe intervalul 0..1 mapat la -20..20 este 10, așa cum se vede. `bind-eval --data out.json` scrie aceleași numere ca JSON.

## 6. Cicluri de mers și biblioteca de mișcări

`walk` generează un mers procedural: are nevoie de numele oaselor pentru picioare și (opțional) pentru brațe și șolduri.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` este lungimea ciclului în cadre, `--stride` balansul coapsei în grade, `--knee` îndoirea genunchiului, `--bob`
mișcarea sus-jos a șoldurilor în pixeli și `--swing` balansul brațelor. Cele șapte canale au o cheie la fiecare 1.5 cadre, fac buclă
(`extrapolate: loop`) și trec `lint` cu 0 constatări. Deoarece ciclul este generat pe loc, nu există deplasare
de eliminat.

### Take-uri și clipuri

Un **take** este o bucată de animație înregistrată, stocată într-un fișier de **bibliotecă** pe care îl poți reutiliza între proiecte:

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Ultimele argumente ale lui `take-record` sunt prefixe de nume de țintă de inclus. `take-apply` scrie take-ul începând de la un cadru,
cu redenumiri opționale `from=to`, un `speed` și `--blend FRAMES` pentru cross-fade cu animația existentă.

### Stiva de clipuri: straturire și greutăți

Un **clip** este un take plasat pe timeline cu un mod, o greutate și măști. Clipurile se așază în straturi în ordine; cele de după le acoperă
pe cele dinainte. Exemplu: un mers, plus un braț drept care salută și care apare și dispare treptat.

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

Stiva nu atinge niciodată canalele făcute manual sau canalele blocate. `--include`/`--exclude` primesc tipare de nume,
`--track-weight arm_*=1,leg_*=0.2` ponderează piste individual, `--mode additive|multiply` așază în straturi în loc să înlocuiască.

**Capcană.** `clip bake` este cel care scrie canalele: după ce schimbi clipurile, rulează-l din nou (comanda o spune).

## 7. Mașini de stări și legături

O **mașină de stări** alege între clipuri în timpul rulării, pe baza unor intrări (booleene, numere, trigger-e). Este descrisă în JSON și
verificată în momentul adăugării:

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Stările redau un clip, un `segment` de cadre, o `region` de markere sau un `blend` 1-D al mai multora; tranzițiile se declanșează când toate
condițiile `when` (`input op value`, AND) sunt îndeplinite, opțional doar după `exit_time` (0 până la 1 din lungimea stării); prima
tranziție care se potrivește, în ordine, câștigă; `"from":"*"` înseamnă orice stare.

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

`machine-sim` reia evenimente de intrare temporizate și scrie un jurnal de stări (`sim.json`) plus cadre SVG, așa că mașinile de stări pot fi testate
și comparate (diff) fără un runtime. O mașină de stări este o funcție pură a jurnalului ei de intrări: același script dă mereu același
rezultat.

## 8. Lip-sync și vorbire

Toate intrările audio trebuie să fie **WAV**; convertește mai întâi alte formate cu ffmpeg. Audio mai lung decât proiectul este **trunchiat** la
lungimea proiectului; lungește mai întâi proiectul (`frames` din fișier) sau vei obține mai puține cadre decât are sunetul.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

Pe o copie de 72 de cadre a proiectului (3 s):

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` alege formele gurii doar după **volumul** audio.
* `lipsync-text` aliniază transcrierea ta (`Hello there, nice to meet you today`) la energia audio și folosește textul pentru
  forme. **Nu este recunoaștere de foneme**; instrumentul o spune de fiecare dată. `--words-out` scrie timingul cuvintelor și al silabelor
  ca JSON sau SRT: `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Limbi: `--lang ro|en`.
* `speech-motion` adaugă înclinări din cap, căderi și înclinări laterale sincronizate cu cuvintele (aditiv).
* Ordinea formelor este rest, small, open, round, wide (`--map` o remapează la switch-ul tău).

### Indicii făcute manual: `mouth`

Dacă ai indicii (cues) din alt instrument, `mouth <project> <cues.txt> <switch_layer> <out>` primește fie linii
**numerotate pe cadre** (stil switch Moho: `frame PHONEME`), fie **secunde** (TSV Rhubarb): un moment **scris cu punct zecimal
este în secunde, un întreg simplu este un cadru**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Literele de formă se mapează pe cele cinci clase de gură: `X A MBP rest` la rest; `B G H L FV etc` (și literele necunoscute) la small; `D AI` la open;
`F O U WQ` la round; `C E` la wide. `--rhubarb` schimbă un singur lucru: litera `E` este atunci tratată ca forma rotunjită Rhubarb (round), nu ca cea largă din
Papagayo (wide). Aici `X C B D A` a dat rest, wide, small, open, rest.

**Capcană.** `0.00 X` înseamnă secunda 0; `0 X` ar fi cadrul 0. Amestecarea lor într-un singur fișier este permisă și derutantă.

### Reacție la audio și beat-uri

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

`sound mix` trunchiază și el la lungimea proiectului (un sunet de 3 s a devenit 1.00 s într-un proiect de 1 s). `export` către `.html` încorporează sunetul
mixat ca mp3 (pagina acestui proiect a crescut de la 50 KB la 131 KB), ceea ce necesită `ffmpeg`; fără el, pagina se scrie fără sunet. `.mp4`,
`.webm` și mixarea mp3/ogg au nevoie și ele de `ffmpeg`.

## 9. Mișcarea de-a lungul căilor, follow-through, foot-lock

* **Path follow:** desenează un strat de linie, apoi `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` pune chei pe poziție
  și (dacă nu e `no-orient`) pe rotație de-a lungul ei:

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` are nevoie de trei sau mai multe puncte: `points need an even number of coordinates: 3..1000 points`.)

* **Follow-through pe lanț:** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` copiază mișcarea unui driver pe un
  lanț de urmăritori, fiecare mai târziu și mai slab (acțiune suprapusă, cozi, antene):

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  Cheile urmăritorilor sunt deplasate mai târziu cu întârzierea (delay), așa că unele ajung dincolo de ultimul cadru (`lint`: `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) și se bazează pe bucla copiată.
* **Foot-lock:** ține un picior înfipt în sol în timp ce corpul se mișcă. Are nevoie de un **controler rădăcină** (un os fără părinte) și de
  un goal IK pentru picior:

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

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` găsește cadrele în care controlerul atinge solul
  și adaugă canale compensatorii (mod `add`) astfel încât să nu alunece. Fără un controler fără părinte, se oprește:
  `foot lock needs a root controller bone (no parent)`.
* **Bake-uri înrudite:** `stretch <in> <out> <follower> <leader>` (urmăritorul țintește și se întinde spre lider), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (atracție de proximitate), `bone-vis` (vizibilitate în overlay).

## 10. Face tracking și limitele lui

**Fii sincer cu ce este acesta.** OSATURA conține propriul detector de fețe și model de repere (23 de repere, schemă
`osatura23`). Este încorporat în binar, rulează local, nu necesită descărcări și a fost **antrenat doar pe fețe generate
de generatorul sintetic al acestui proiect**: capete stilizate și de desen animat. **Nu este validat pe fotografii, webcam-uri sau
video real.** Cifrele măsurate (`osatura face-eval --eval-n 50 --seed 9`) sunt:

| Măsură | Rezultat |
|---|---|
| Fețe sintetice excluse din antrenare, rata de detecție | 1.0 (50 din 50) |
| Eroarea reperelor (NME) medie / mediană / percentila 90 | 0.028 / 0.026 / 0.046 |
| Fals pozitive pe 50 de scene fără față | 2 |
| Designuri cartoon, NME unde s-a detectat | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Designuri cartoon, detectate | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Aceste cifre sunt pentru *propriul stil sintetic*. Nu le citi ca acuratețe pe o persoană aflată în fața unei camere. Dacă
ai nevoie de urmărire de fețe reale, produ repere cu un instrument în care ai încredere și furnizează-le ca NDJSON (formatul de intrare al lui `face-bake`
este deschis); panoul Face al studioului se așteaptă să furnizezi o funcție `window.osaturaLandmarkProvider` pentru intrarea de la cameră.

Fluxul:

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

cu un fișier de mapare (aici `facemap.json`):

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

Id-uri de semnal: `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Un id greșit listează id-urile valide: `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
`face-bake` calibrează fața neutră din primele cadre (`--calibrate-frames`, `--calib`).

**Lucruri de știut.** Bake-ul este **trunchiat la lungimea proiectului**: 60 de cadre de repere aplicate pe un proiect de 24 de cadre au
produs chei doar de la 0 la 23, iar un avertisment pe stderr spune câte cadre din înregistrare nu au fost coapte (lungimea proiectului nu se schimbă). Linia de rezumat arată `(K keys in them)` pentru canalele noi; `N trigger keys set` numără cheile scrise în canale preexistente. `face-detect <image> --overlay o.png` arată ce a găsit detectorul pe o imagine statică.

## 11. Mocap și BVH

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

* Un rig 2D poate folosi doar o proiecție a mișcării 3D: `--view front|side|top|yaw=DEG` o alege.
* `mocap-match` afișează un tabel de încredere (rândurile `shin` de mai sus au o notă deoarece lungimea membrului lor este de 2.1 ori față de mediană).
  `--map map.json` suprascrie presupunerea, iar `--min-conf C` stabilește încrederea sub care o articulație nu este folosită.
* **`mocap-retarget` lungește proiectul** pentru a încăpea clipul (24 au devenit 48 de cadre; o notă spune asta, iar `--keep-length` trunchiază în schimb clipul). `--in-place` elimină deplasarea rădăcinii,
  `--footlock BONE --ground Y` fixează un picior (are nevoie de controlerul rădăcină fără părinte, ca `foot-lock`: `foot lock needs a root controller bone (no parent)`),
  iar `--mirror` oglindește mișcarea.
* `import-bvh` este forma mai veche, explicită, cu `--joint-map Joint=bone,...`.

## 12. Intrare live (MIDI, mouse, gamepad, taste)

Înregistrează o performanță în panoul Live al studioului sau redă offline o înregistrare ori un fișier MIDI:

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

Mod-wheel (CC 1 pe canalul 1) a mișcat brațul de la 0 la 160 de grade prin 24 de chei coapte, iar nota pornită la 0.5 s a devenit evenimentul de proiect
`cymbal` la cadrul 12. Sursele sunt `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` și `face`. Id-urile MIDI sunt `"<channel>:<number>"`, cu canale începând de la 1. Web MIDI în
studio are nevoie de un browser care îl suportă (vezi [STUDIO.md](STUDIO.md)); comanda offline are nevoie doar de fișier.

## 13. Containere de layout și scroll

Containerele de layout reașază straturile pe o altă dimensiune de canvas (animație de interfață responsivă); regiunile de scroll adaugă
scroll limitat (clamped) sau elastic. Containerele sunt în JSON, iar solverul este determinist:

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

`dx`/`dy` sunt offseturi aplicate peste animație. Detalii despre containere, `scroll add|list|remove|eval` (limite elastice, puncte de snap, simulare
de fling) se găsesc în [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). Layout-ul este aplicat de runtime și de previzualizări;
**fișierele exportate conțin layout-ul rezolvat pentru dimensiunea proprie a proiectului.**

## 14. Cameră, shot-uri, adâncime de câmp

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` arată 11 canale: cele 7 ale mersului plus cele ale camerei. `dof` estompează fiecare strat de nivel superior cu `|parallax - focus|`
(intensitatea și raza sunt limitate de `--max-radius`, implicit 24). Adâncimea de câmp este o randare de post-procesare; `dof-export` o scrie într-un
GIF, APNG sau folder.

## 15. Efecte de strat

Efectele sunt o listă ordonată pe un strat (blur, drop shadow, tint, fill), randate într-o trecere izolată înaintea măștii, opacității și modului de amestecare
ale stratului, cel mult 16 pe strat. Valorile lor numerice pot fi animate cu canale cu ținta `"<layer>::fx<index>"` și una dintre proprietățile `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (culorile nu pot fi animate):

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

`check` trece, iar umbra alunecă de la -6 la +6 pe durata buclei. Un **lanț de post-efecte** (outline, posterize, relight cu normal maps...) randează un singur
cadru pornind de la o listă JSON:

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Verificări de siguranță pentru sclipiri și mișcarea camerei (un ajutor de inginerie bazat pe definiția publică WCAG 2.3.1, **nu o certificare de difuzare**):

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin și trasee (overlay-uri)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` desenează fantomele la cadrele *cu chei* învecinate, în loc de un pas fix. Overlay-urile nu modifică niciodată proiectul și nu fac parte dintr-un export.
Studioul le arată live (`Alt+N` onion, `Alt+M` trasee, `Alt+7` panou).

## 17. Idle-uri procedurale

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Cu seed, deci aceeași comandă produce aceeași animație oriunde. `autorig` adaugă respirația și clipitul pentru tine când numele se potrivesc.

## 18. Asigurarea calității: check, lint, fuzz, CI

* `check` compilează proiectul exact așa cum o va face randorul. Cod de ieșire 1 la eroare.
* `lint` găsește probleme statice (cod de ieșire 1 la erori; informațiile și avertismentele nu produc eșec): ținte IK inaccesibile (`L003`), canale care
  nu se schimbă niciodată (`L005`), chei în afara timeline-ului (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` adăugat, `-` eliminat, `~` modificat) și `vcs` (control al versiunilor cu merge semantic pentru fișierele de proiect).
* **Fuzz** poziționează rig-ul aleatoriu și raportează unde cedează:

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

  **`fuzz` simplu este un raport și iese cu 0; adaugă `--gate` (orice eșec iese cu 1) sau `--max-fail-rate R` (o fracție) pentru a-l transforma în poartă de control.** Intervalul de eșantionare este intervalul complet al fiecărui parametru, deci un rig cu IK
  eșuează în aproape toate eșantioanele (`98.0%`, 48 din 50 `ik_unreachable`, pe biped-ul cu foot-lock). Restrânge cu `--only`, `--props`,
  `--margin` și `--ignore KINDS`, alege intervalul pe care îl consideri legal și stabilește pragul pe care îl accepți.
  `--report out.json`, `--heatmap out.svg` și `--minimize` (cea mai mică poziție care reproduce eroarea) ajută. Strategii: `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Golden-uri și CI.** `golden write walk.json gold.json` stochează semnături de cadre; `golden check` compară exact în mod implicit (cod de ieșire 1 la orice
  schimbare; `--tol T` îl relaxează). Verificarea unei animații *diferite* față de ele listează cadrele care diferă: `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` înglobează aceasta și mai mult într-un manifest:

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

  `ci run` iese cu 0 când totul trece, cu 1 când o verificare eșuează și cu 2 la o eroare. `ci bless` acceptă randările curente ca noi cadre golden,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` explică o schimbare cu hărți termice. Pragul pose-fuzz de 46 % din manifest a fost calibrat pe
  proiectul „așa cum este acum" (instrumentul afișează această notă); strânge-l în mod deliberat. Detaliile se găsesc în [CI.md](CI.md).

## 19. Dimensiune și nivel de detaliu

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

`size-report` arată ce straturi, imagini și canale costă octeți. Când o platformă are un buget, `export-fit` găsește exportul de cea mai bună calitate care se încadrează:

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

A doua comandă a ieșit cu **1** și tot a scris cel mai mic fișier găsit (14952 de octeți). Bugetele se verifică implicit pe dimensiunea brută;
`--measure gzip` verifică dimensiunea comprimată (Lottie-ul de mai sus are 1429 de octeți comprimat gzip). Căutarea este limitată de `--max-seconds` (implicit 60 s de timp real);
în build-ul de debug, căutarea GIF a fost oprită de această limită și a păstrat cel mai bun rezultat al său.

Pentru desene cu multe curbe și mesh-uri, **nivelul de detaliu** (level of detail) produce copii simplificate, cu o margine de eroare *declarată* în pixeli de ieșire:

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

`--verify` măsoară abaterea reală (cod de ieșire 1 dacă depășește limita); în rularea de mai sus, fiecare eroare măsurată a fost sub limita ei declarată. Observă cât de
puțin economisește un proiect mic bazat pe dreptunghiuri (de la 17507 la 16080 de octeți): LOD se justifică la desenele detaliate, nu la acesta.

## 20. Limite oneste

* Lip-sync-ul este energie plus aliniere la text, nu recunoaștere de foneme.
* Detectorul de fețe este doar sintetic (secțiunea 10): nu face utilizabilă de unul singur o performanță la webcam.
* `make-loop` elimină deplasarea; `loop-solve` nu o rezolvă; zgomotul nu face buclă.
* Audio este trunchiat la lungimea proiectului; `mocap-retarget` lungește proiectul.
* Lottie, GIF și HTML nu pot transporta toate funcțiile; avertismentele de export și `--report` listează ce a fost aproximat ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Doar Chromium a fost testat pentru studio ([STUDIO.md](STUDIO.md)).
