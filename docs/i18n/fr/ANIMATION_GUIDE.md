# Guide de l'animation d'OSATURA

Comment faire bouger des éléments, réutiliser le mouvement, le piloter à partir de l'audio, de visages, de mocap et d'entrées en direct, le vérifier, et
le livrer sous une forme légère. Chaque commande présentée ici a été exécutée avec le vrai binaire (`osatura 0.1.0`, build debug, Linux) ;
les sorties sont copiées de ces exécutions. Le rigging est traité dans [RIGGING_GUIDE.md](RIGGING_GUIDE.md) ; le premier projet dans
[GETTING_STARTED.md](GETTING_STARTED.md) ; toutes les options dans [CLI_REFERENCE.md](CLI_REFERENCE.md).

Licence de ce document : CC-BY-4.0.

Les exemples utilisent le biped à 11 os (`biped.json`, 400x360, 24 fps, 24 images) du guide du rigging : des
rectangles et cercles séparés rattachés à des os, un calque switch `mouth` à cinq enfants, construit par un seul appel à `script`.
Tout rig dont les os portent des noms similaires convient.

![Six images (0, 4, 8, 12, 16, 20) du cycle de marche généré d'un biped simple fait de rectangles : les jambes oscillent en opposition, les bras balancent en sens inverse, les hanches rebondissent.](../../img/walk-cycle.png)

## 1. Canaux et clés

Tout ce qui bouge est un **canal** : une cible (os, calque, caméra, point de déformateur), une propriété, un
mode et une source.

```
{"target":"hand_ctrl","prop":"bone_offset_y","source":"keys",
 "track":{"keys":[{"frame":0,"value":0,"ease":{"type":"ease_in_out"}}, ...],"extrapolate":"hold"},
 "mode":"replace","weight":1.0}
```

* **Sources :** `keys` (images clés), `expr` (une expression), `noise` (bruit de valeur fractal dans `[-amp, amp]`, `freq` en
  cycles par seconde) et `driven` (une courbe qui associe la valeur d'une autre propriété à celle-ci). Les commandes qui bakent
  (physics, walk, clips, lip-sync...) écrivent des `keys` ordinaires. `osatura timeline` affiche la source de chaque piste :
  `#1 bone_offset_y (noise)`, `#0 bone_angle (expr)`, `#0 layer_switch (driven)`, ou la liste des images clés.
* **Modes :** `replace`, `add` (par-dessus tout ce qui se trouve en dessous) et `multiply`. Les canaux se combinent dans l'ordre de la liste.
* **Extrapolate** (ce que fait une piste en dehors de sa première et de sa dernière clé) : `hold` (par défaut), `linear` (prolonge la
  pente de fin), `loop`, `ping_pong` et `loop_offset` (répète en ajoutant à chaque cycle la variation entre la première et la dernière clé, ce qui convient à un
  décalage en x de marche). Réglez-le avec `keyedit ... extrapolate --channels 0 --extrapolate loop_offset` ; un nom erroné liste les noms valides.
* Les propriétés d'os sont des **deltas par rapport à la pose de repos** : `bone_angle`, `bone_offset_x`, `bone_offset_y`,
  `bone_scale`. Les propriétés de calque comprennent `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`,
  `layer_opacity` et `layer_switch`. Les déformateurs utilisent `cage_dx`... comme dans le guide du rigging.

Créez des clés depuis un script (`set_key(target, prop, frame, value[, ease])`), depuis la CLI avec `keyedit add`, depuis le
studio, ou en JSON.

```
$ osatura keyedit walk.json k3.json add --target torso --prop bone_angle --at 12 --value 6 --ease ease_in_out
add: 1 keys changed, wrote k3.json
```

`osatura timeline <project> [--json]` liste chaque piste par os/calque avec ses images clés ; `osatura explain <project> <bone|layer> <frame>`
montre comment une image a été calculée.

## 2. Interpolation et easing

Chaque clé stocke l'ease du segment qui **commence** à cette clé. Les noms d'easing qui ne demandent aucun paramètre sont
`linear`, `step` (maintient la valeur jusqu'à la clé suivante), `ease_in`, `ease_out`, `ease_in_out`, `smooth` (une spline
de Catmull-Rom passant par les clés voisines : lisse, sans contrôle du dépassement) et `bounce`. En JSON, un ease est un objet :

```
{"type":"ease_in_out"}
{"type":"cubic_bezier","x1":0.25,"y1":0.1,"x2":0.25,"y2":1}    // CSS-style curve through (0,0) and (1,1)
{"type":"spring","bounces":3,"decay":4}                          // damped-spring settle that lands exactly on the target
{"type":"back","overshoot":0.6}                                  // ease out with overshoot
```

`keyedit ... ease --sel SEL --ease E` accepte un nom simple, `bezier:x1,y1,x2,y2` (stocké en `cubic_bezier`) ou un objet JSON pour
`spring` et `back` ; les quatre formes ont été appliquées aux 17 clés de hanche du cycle de marche sans erreur. Dans la CLI du dope sheet, la syntaxe de
sélection `SEL` est `target:prop@first..last`, `target:prop` ou `all`.

```
$ osatura keyedit walk.json k2.json ease --sel all --ease smooth
ease: 119 keys changed, wrote k2.json
```

Ajoutez `-` comme sortie pour faire de n'importe quel `keyedit` un **essai à blanc** qui affiche le plan en JSON sans rien écrire :

```
$ osatura keyedit walk.json - move --sel 'thigh_l:bone_angle@0..6' --dt 2
{ "op": { "op": "batch", "ops": [ { "op": "set", "path": "/channels/0/track", "value": { "extrapolate": "loop", "keys": [ ... ] } } ] }, ... }
```

**Piège.** `keyedit` refuse les modifications qui feraient sortir des clés de la plage d'images du projet, sauf si vous passez `--free` :
`scale: frame 27 is outside the project range 0..=24` (la même commande avec `--free` a affiché `scale: 17 keys changed`, et les
clés supplémentaires se trouvent alors au-delà de la dernière image, où `lint` signale `L007`). Les canaux verrouillés sont refusés eux aussi :
`channel hand_ctrl.BoneOffsetY is locked` (voir `chan-meta ... lock=on`).

L'éditeur de courbes du studio dessine les mêmes courbes, avec des tangentes déplaçables (`handle`, `smooth` ; voir
[STUDIO.md](STUDIO.md)). Les clés ont des poignées de tangente d'entrée/sortie facultatives (`free`, `aligned` ou `auto`).

## 3. Le dope sheet en ligne de commande : `keyedit`

Tout ce que la timeline du studio sait faire existe sous forme de sous-commande de `keyedit`, appliquée comme un seul lot sûr vis-à-vis de l'annulation :

| Sous-commande | Effet |
|---|---|
| `move --sel SEL [--dt N] [--dv N] [--collision replace\|merge\|refuse] [--no-snap] [--free]` | décale des clés dans le temps et/ou en valeur |
| `scale --sel SEL --pivot F --factor X` | modifie le timing autour d'une image |
| `dup --sel SEL --offset N` | duplique |
| `delete --sel SEL` | supprime |
| `ease --sel SEL --ease E` / `value --sel SEL --value V` / `smooth --sel SEL [--kind monotone\|catmull_rom]` | définit l'ease, la valeur, ou un lissage automatique |
| `copy --sel SEL --clip clip.json` / `paste --clip clip.json --at F [--remap a:p=b:q]` | presse-papiers entre projets |
| `add --target T --prop P --at F [--value V] [--ease E]` | insère une clé |
| `ripple --at F --by N --mode insert\|delete [--scope 0,1] [--adjust-frames]` | insère ou retire du temps |
| `offset --channels 0,1 --dt N` | fait glisser des canaux entiers |
| `extrapolate --channels 0,1 --extrapolate loop` / `loop --channels 0,1 [--end F]` | boucle après la dernière clé |
| `handle --sel SEL --side in\|out --t F --v V [--mode free\|aligned\|auto]` | poignée de tangente de la courbe |
| `lock --target T [--prop P] [--value on\|off]` | verrouille un canal |

## 4. Boucles

Un projet de N images joue les images 0 à N-1 en boucle ; **l'image N doit donc être la même pose que l'image 0 et n'est jamais affichée
deux fois**. Trois commandes aident :

* `keyedit ... loop --channels 0,1` règle `extrapolate: loop` sur des canaux (les clés ne sont pas modifiées) :

```
$ osatura keyedit s4.json s6.json loop --channels 0,1
loop: 2 keys changed, wrote s6.json
```

* `loop-solve <in> <out> [--window 24] [--inclusive]` fait boucler sans raccord des canaux animés et indique le raccord :

```
$ osatura loop-solve 04-keys.json buddy-final.json
wrote buddy-final.json: 2 channel(s) adjusted, seam 0.000 -> 0.000
```

* `make-loop <in> <out> [targets|all] [props] [--inclusive]` **supprime le déplacement** pour que le mouvement s'effectue sur place :

```
$ osatura make-loop t1.json t2.json
ball LayerX: removed drift 280.00
wrote t2.json
```

Cette balle se déplaçait de 280 unités vers la droite en 23 images. `make-loop` en a fait une parfaite boucle sur place... d'une balle qui ne
bouge plus (lint signale alors `Info L005 [ball:LayerX] all keys have the same value; the channel never changes anything`).
C'est ce que signifie « supprime le déplacement » : utilisez-le pour des cycles de marche sur place, puis déplacez tout le personnage avec un
tracé `follow` ou une caméra (section 9). `loop-solve` ne corrige pas le déplacement : sur la même balle, il indique `seam 280.000 -> 280.000`.
Si vous voulez au contraire que le déplacement **se poursuive** d'un cycle à l'autre, donnez au canal l'extrapolation `loop_offset` (section 1) : `explain` indique alors
`keys → 121.739 (2 key(s), LoopOffset outside range)` pour la balle à l'image 10.

`--inclusive` sélectionne l'ancienne convention selon laquelle la dernière image affichée est égale à la première.

**Piège : le bruit ne boucle pas.** Un canal `noise` n'est pas périodique. Pour un frémissement en boucle, utilisez une expression de `t`
(0..1 sur le clip) telle que `8 * sin(t * 2 * pi)`, ou rompez la boucle délibérément.

## 5. Expressions, bruit et drivers

Un canal **expression** évalue une petite formule à chaque image. Variables : `frame`, `time` (secondes), `fps`, `frames`,
`pi`, et **`t` = 0..1 sur le clip**. Opérateurs `+ - * / % ^`, comparaisons, `&& || !`, `?:`. Les fonctions comprennent
`sin`, `abs`, `min`, `clamp(x, lo, hi)`, `lerp(a, b, t)`, `wiggle(...)` (bruit ; deux ou trois arguments, par exemple `wiggle(3, 5, 7)`)
et `get("target","prop")`, qui lit la valeur courante d'un autre canal (les canaux s'exécutent dans l'ordre de la liste et les suivants voient les résultats des précédents).

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

Résultat de `explain ex1.json head <frame>` : l'angle monde de la tête est de -90.0 aux images 0, 12 et 24, **-82.0 à l'image 6**, et
-98.0 à l'image 18 (une sinusoïde complète sur le clip ; l'angle de repos de la tête est de -90 degrés monde). Le bruit sur les hanches
déplace `hips` d'une fraction d'unité : sa tête se trouve à y 169.7 à l'image 0 et 170.5 à l'image 24 (170.0 sans le bruit), il ne boucle donc pas (voir ci-dessus).

Les erreurs sont détectées avant que quoi que ce soit ne soit écrit :

```
$ osatura script biped.json ex2.json ex2.rhai       # "8 * sin(tt * 2 * pi)"
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unknown variable 'tt'
$ osatura script biped.json ex3.json ex3.rhai       # "8 * sin("
request 1 (channel head.BoneAngle) was rejected, nothing changed: channel 0 ('head'): expression error: unexpected end of expression
```

**Piège.** Dans une expression, `t` est normalisé (0 à 1), ce ne sont ni des secondes ni des images. `sin(t * 6.2832 / 24)` a l'air juste
et est faux : il bouge à peine. Utilisez `time` pour les secondes ou `frame`.

### Drivers, sliders et liaisons de données

* `slider` et `view-swap` (guide du rigging) créent des os contrôleurs ainsi que des canaux `driven`.
* `bind` relie un chemin de valeur externe (issu d'un jeu, d'une interface ou d'un test) à une propriété ou à une entrée de machine à états, avec remappage,
  lissage et une expression facultative :

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

`0.75` sur la plage 0..1 remappée vers -20..20 donne 10, comme indiqué. `bind-eval --data out.json` écrit les mêmes nombres en JSON.

## 6. Cycles de marche et bibliothèque de mouvements

`walk` génère une marche procédurale : elle requiert les noms d'os des jambes et (facultativement) des bras et des hanches.

```
$ osatura walk biped.json walk.json --thigh-l thigh_l --thigh-r thigh_r --shin-l shin_l --shin-r shin_r \
    --arm-l arm_upper_l --arm-r arm_upper_r --hips hips --period 24 --stride 28 --knee 40 --bob 4 --swing 25
wrote walk.json (+7 channels)
```

`--period` est la longueur du cycle en images, `--stride` l'oscillation de la cuisse en degrés, `--knee` la flexion du genou, `--bob`
le rebond des hanches en pixels et `--swing` le balancement des bras. Les sept canaux ont une clé toutes les 1.5 images, bouclent
(`extrapolate: loop`) et passent `lint` avec 0 constat. Comme le cycle est généré sur place, il n'y a aucun déplacement
à supprimer.

### Takes et clips

Un **take** est un morceau d'animation enregistré, stocké dans un fichier de **bibliothèque** que vous pouvez réutiliser d'un projet à l'autre :

```
$ osatura take-record walk.json lib.json walk 0 24 thigh,shin,arm,hips
recorded 7 tracks into lib.json
$ osatura take-list lib.json
walk (24 frames, 7 tracks)
$ osatura take-apply biped.json ta.json lib.json walk 0
applied 7 tracks; wrote ta.json
```

Les derniers arguments de `take-record` sont des préfixes de noms de cibles à inclure. `take-apply` écrit le take à partir d'une image,
avec des renommages `from=to` facultatifs, une `speed` et `--blend FRAMES` pour fondre en enchaîné avec l'animation existante.

### La pile de clips : superposition et poids

Un **clip** est un take placé sur la timeline avec un mode, un poids et des masques. Les clips se superposent dans l'ordre ; les clips suivants passent par-dessus
les précédents. Exemple : une marche, plus un bras droit qui salue et dont l'effet apparaît puis disparaît progressivement.

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

La pile ne touche jamais aux canaux faits à la main ni aux canaux verrouillés. `--include`/`--exclude` acceptent des motifs de noms,
`--track-weight arm_*=1,leg_*=0.2` pondère les pistes individuellement, `--mode additive|multiply` superpose au lieu de remplacer.

**Piège.** C'est `clip bake` qui écrit les canaux : après avoir modifié des clips, relancez-le (la commande le rappelle).

## 7. Machines à états et liaisons

Une **machine à états** choisit entre des clips à l'exécution à partir d'entrées (booléens, nombres, triggers). Elle se décrit en JSON et
est vérifiée à l'ajout :

```
{"name":"hero","inputs":[{"name":"greet","kind":"trigger"},{"name":"walking","kind":"bool","default":1}],
 "layers":[{"name":"base","entry":"walk",
  "states":[{"name":"walk","source":{"kind":"clip","clip":"walkclip"},"mode":"loop"},
            {"name":"wave","source":{"kind":"clip","clip":"waveclip"},"mode":"once","blend_in":0.15}],
  "transitions":[{"from":"walk","to":"wave","when":[{"input":"greet"}],"duration":0.15},
                 {"from":"wave","to":"walk","exit_time":1.0,"duration":0.2}]}]}
```

Les états jouent un clip, un `segment` d'images, une `region` de marqueurs, ou un `blend` 1D de plusieurs ; les transitions se déclenchent lorsque toutes les
conditions `when` (`input op value`, ET) sont réunies, éventuellement seulement après `exit_time` (0 à 1 de la longueur de l'état) ; la première
transition correspondante dans l'ordre l'emporte ; `"from":"*"` désigne n'importe quel état.

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

`machine-sim` rejoue des événements d'entrée horodatés et écrit un journal d'états (`sim.json`) ainsi que des images SVG, de sorte que les machines à états sont testables
et comparables avec diff sans runtime. Une machine à états est une fonction pure de son journal d'entrées : le même script donne toujours le même
résultat.

## 8. Lip-sync et parole

Toutes les entrées audio doivent être au format **WAV** ; convertissez d'abord les autres formats avec ffmpeg. Un audio plus long que le projet est **tronqué** à la
longueur du projet ; allongez d'abord le projet (`frames` dans le fichier), sinon vous obtenez moins d'images que le son n'en contient.

```
$ osatura lipsync biped.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 24 frames)       # the project has 24 frames = 1 s: the last 2 s are cut
```

Sur une copie du projet de 72 images (3 s) :

```
$ osatura lipsync talk0.json speech.wav mouth ls.json
wrote ls.json (3.00s of audio → 72 frames)
$ osatura lipsync-text talk0.json speech.wav transcript.txt mouth lt.json --lang en --words-out words.json
wrote lt.json (7 words, 18 exposure changes; energy + text alignment, not phoneme recognition)
$ osatura speech-motion lt.json speech.wav transcript.txt sm.json --head head --nod 4 --fall 3 --tilt 2 --seed 5 --lang en
wrote sm.json (1 additive channel(s) from 7 words)
```

* `lipsync` choisit les formes de bouche d'après le seul **volume sonore** de l'audio.
* `lipsync-text` aligne votre transcription (`Hello there, nice to meet you today`) sur l'énergie de l'audio et utilise le texte pour
  les formes. Ce n'est **pas de la reconnaissance de phonèmes** ; l'outil le rappelle à chaque fois. `--words-out` écrit le timing des mots et des syllabes
  en JSON ou en SRT : `00:00:00,220 --> 00:00:02,690  Hello there, nice to meet you today`. Langues : `--lang ro|en`.
* `speech-motion` ajoute des hochements, des chutes et des inclinaisons de la tête cadencés sur les mots (additif).
* L'ordre des formes est rest, small, open, round, wide (`--map` les remappe vers votre switch).

### Signaux faits main : `mouth`

Si vous avez des repères issus d'un autre outil, `mouth <project> <cues.txt> <switch_layer> <out>` accepte soit des lignes
**numérotées par image** (style switch de Moho : `frame PHONEME`), soit des **secondes** (TSV de Rhubarb) : un temps **écrit avec un point décimal
est en secondes, un entier seul est une image**.

```
printf '0.00 X\n0.20 C\n0.35 B\n0.60 D\n1.00 A\n' > cues.txt
osatura mouth talk0.json cues.txt mouth mc.json                 # wrote mc.json (5 cues)
osatura mouth talk0.json cues.txt mouth mc.json --rhubarb       # also wrote mc.json (5 cues)
```

Les lettres de forme correspondent aux cinq classes de bouche : `X A MBP rest` vers rest ; `B G H L FV etc` (et les lettres inconnues) vers small ; `D AI` vers open ;
`F O U WQ` vers round ; `C E` vers wide. `--rhubarb` ne change qu'une chose : la lettre `E` est alors traitée comme la forme arrondie de Rhubarb (round) plutôt que comme la forme large de
Papagayo. Ici, `X C B D A` a donné rest, wide, small, open, rest.

**Piège.** `0.00 X` désigne la seconde 0 ; `0 X` désignerait l'image 0. Les mélanger dans un même fichier est permis et source de confusion.

### Réactivité à l'audio et battements

```
osatura audio-react biped.json ar.json speech.wav head bone_angle gain=10 add=on   # loudness drives a property
osatura beats biped.json bt.json speech.wav as=markers name=syl                    # 2 beats at frames [5, 14]
osatura sound add biped.json s1.json voice speech.wav 0 1.0
osatura sound mix s1.json mix.wav                                                  # wrote mix.wav (1.00 s)
```

`sound mix` tronque lui aussi à la longueur du projet (un son de 3 s est devenu 1.00 s pour un projet de 1 s). `export` vers `.html` intègre le son mixé
en mp3 (la page de ce projet est passée de 50 Ko à 131 Ko), ce qui requiert `ffmpeg` ; sans lui, la page est écrite sans son. `.mp4`,
`.webm` et le mixage mp3/ogg requièrent aussi `ffmpeg`.

## 9. Déplacer des éléments le long de tracés, follow-through, foot-lock

* **Suivi de tracé :** dessinez un calque de ligne, puis `follow <in> <out> <layer|bone:NAME> <path_layer> <start> <end> [no-orient]` anime la position
  et (sauf `no-orient`) la rotation le long de celui-ci :

```
cat > path.rhai <<'EOF'
add_line("route", [40, 300, 120, 260, 200, 290, 300, 240, 360, 280], "#bbbbbb", 2, false);
add_circle("ball", 0, 0, 10, "#c1121f");
EOF
osatura script biped.json p0.json path.rhai
osatura follow p0.json p1.json ball route 0 23         # 3 channels: layer_x, layer_y, layer_rotation
```

(`add_line` demande trois points ou plus : `points need an even number of coordinates: 3..1000 points`.)

* **Follow-through de chaîne :** `follow <in> <out> --target SRC --prop P --chain a,b,c [--delay F --decay X]` copie le mouvement d'un driver vers une
  chaîne de suiveurs, chacun plus tardif et plus faible (action qui se chevauche, queues, antennes) :

```
$ osatura follow walk.json fo.json --target hips --prop bone_offset_y --chain torso,head --delay 2 --decay 0.6
wrote fo.json (+2 channels)
```

  Les clés des suiveurs sont décalées plus tard du délai, certaines tombent donc au-delà de la dernière image (`lint` : `Info L007 [head:BoneOffsetY] key at
  frame 25 lies outside the 0..24 timeline`) et s'appuient sur la boucle copiée.
* **Foot-lock :** garde un pied planté pendant que le corps se déplace. Il requiert un **contrôleur racine** (un os sans parent) et
  un goal de pied en IK :

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

  `foot-lock <in> <out> <ctrl_bone> <ground_y> [tolerance] [min_frames]` trouve les images où le contrôleur touche le sol
  et ajoute des canaux compensateurs (mode `add`) pour qu'il ne glisse pas. Sans contrôleur sans parent, elle s'arrête :
  `foot lock needs a root controller bone (no parent)`.
* **Bakes associés :** `stretch <in> <out> <follower> <leader>` (le suiveur vise le meneur et s'étire vers lui), `magnet <in> <out> <ctrl>
  <bone:NAME|x,y> <radius> <strength>` (attraction de proximité), `bone-vis` (visibilité en surcouche).

## 10. Suivi du visage et ses limites

**Soyez honnête sur ce que c'est.** OSATURA contient son propre petit détecteur de visage et son modèle de repères (23 repères, disposition
`osatura23`). Il est intégré au binaire, s'exécute en local, ne nécessite aucun téléchargement, et a été **entraîné uniquement sur des visages générés
par le générateur synthétique de ce projet** : des têtes stylisées et de dessin animé. Il n'est **pas validé sur des photographies, des webcams ni de la vraie
vidéo.** Les chiffres mesurés (`osatura face-eval --eval-n 50 --seed 9`) sont les suivants :

| Mesure | Résultat |
|---|---|
| Visages synthétiques mis de côté, taux de détection | 1.0 (50 sur 50) |
| Erreur de repère (NME) moyenne / médiane / 90e centile | 0.028 / 0.026 / 0.046 |
| Faux positifs sur 50 scènes sans visage | 2 |
| Designs de dessin animé, NME là où détecté | round-emoji 0.164, manga-oval 0.166, line-drawing 0.105 |
| Designs de dessin animé, détectés | round-emoji 6/6, manga-oval 6/6, line-drawing 6/6, **boxy-robot 0/6** |

Ces chiffres valent pour *son propre style synthétique*. Ne les lisez pas comme la précision sur une personne devant une caméra. Si vous
avez besoin d'un suivi de visages réels, produisez les repères avec un outil de confiance et fournissez-les en NDJSON (le format d'entrée de `face-bake`
est ouvert) ; le panneau Face du studio attend que vous fournissiez une fonction `window.osaturaLandmarkProvider` pour l'entrée caméra.

Le pipeline :

```
osatura face-synth face --frames 60 --size 160 --seed 7        # synthetic head animation: PNG frames, truth.ndjson, params.json
osatura face-track face face/track.ndjson --fps 24             # wrote face/track.ndjson: 60 frames, face found in 60
osatura face-bake biped.json face/track.ndjson facemap.json fb.json --fps 24
```
```
60 landmark frames -> +2 channels (K keys in them), 0 trigger keys set, 2 events; wrote fb.json   # K = number of keys in the 2 new channels
```

avec un fichier de mapping (ici `facemap.json`) :

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

Identifiants de signaux : `head_yaw`, `head_pitch`, `head_roll`, `eye_l_open`, `eye_r_open`, `mouth_open`, `mouth_wide`, `mouth_round`,
`brow_raise`, `brow_l_raise`, `brow_r_raise`. Un identifiant erroné liste les valides : `map 0: unknown face signal 'yaw' (known: head_yaw, head_pitch, ...)`.
`face-bake` calibre le visage neutre à partir des premières images (`--calibrate-frames`, `--calib`).

**À savoir.** Le bake est **tronqué à la longueur du projet** : 60 images de repères sur un projet de 24 images
n'ont produit que les clés 0 à 23, et un avertissement sur stderr indique combien d'images de l'enregistrement n'ont pas été bakées (la longueur du projet reste inchangée). La ligne de résumé affiche `(K keys in them)` pour les nouveaux canaux ; `N trigger keys set` compte les clés écrites dans des canaux préexistants. `face-detect <image> --overlay o.png` montre ce que le détecteur a trouvé sur une image fixe.

## 11. Mocap et BVH

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

* Un rig 2D ne peut utiliser qu'une projection du mouvement 3D : `--view front|side|top|yaw=DEG` la choisit.
* `mocap-match` affiche un tableau de confiance (les lignes `shin` ci-dessus portent une note parce que la longueur de leur membre s'écarte de 2.1 fois de la médiane).
  `--map map.json` remplace la déduction automatique et `--min-conf C` fixe la confiance en dessous de laquelle une articulation n'est pas utilisée.
* **`mocap-retarget` allonge le projet** pour contenir le clip (24 est devenu 48 images ; une note le signale, et `--keep-length` tronque le clip à la place). `--in-place` supprime le déplacement de la racine,
  `--footlock BONE --ground Y` épingle un pied (il requiert le contrôleur racine sans parent, comme `foot-lock` : `foot lock needs a root controller bone (no parent)`),
  et `--mirror` reflète le mouvement.
* `import-bvh` est la forme plus ancienne et explicite, avec `--joint-map Joint=bone,...`.

## 12. Entrées en direct (MIDI, souris, manette, touches)

Enregistrez une performance dans le panneau Live du studio, ou rejouez hors ligne un enregistrement ou un fichier MIDI :

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

La molette de modulation (CC 1 sur le canal 1) a fait balayer au bras de 0 à 160 degrés au travers de 24 clés bakées, et la note on à 0.5 s est devenue l'événement
de projet `cymbal` à l'image 12. Les sources sont `midi_cc`, `midi_note`, `midi_pitch_bend`, `mouse_x`, `mouse_y`, `mouse_button`,
`gamepad_axis`, `gamepad_button`, `key` et `face`. Les identifiants MIDI sont `"<channel>:<number>"`, les canaux commençant à 1. Le Web MIDI dans le
studio requiert un navigateur qui le prend en charge (voir [STUDIO.md](STUDIO.md)) ; la commande hors ligne n'a besoin que du fichier.

## 13. Conteneurs de mise en page et défilement

Les conteneurs de mise en page (layout) recomposent les calques pour une autre taille de canevas (animation d'interface adaptative) ; les régions de défilement ajoutent un défilement
borné ou élastique. Les conteneurs sont en JSON et le solveur est déterministe :

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

`dx`/`dy` sont des décalages appliqués par-dessus l'animation. Les détails des conteneurs et de `scroll add|list|remove|eval` (limites élastiques, points d'accrochage, simulation
de fling) figurent dans [CLI_REFERENCE.md](CLI_REFERENCE.md#scroll-add-list-remove-eval). La mise en page est appliquée par le runtime et les aperçus ;
**les fichiers exportés contiennent la mise en page résolue pour la taille propre du projet.**

## 14. Caméra, plans, profondeur de champ

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom [rotation]
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5
```

`timeline sh3.json` affiche 11 canaux : les 7 de la marche plus ceux de la caméra. `dof` floute chaque calque de premier niveau selon `|parallax - focus|`
(l'intensité et le rayon sont plafonnés par `--max-radius`, 24 par défaut). La profondeur de champ est un rendu de post-traitement ; `dof-export` l'écrit dans un
GIF, un APNG ou un dossier.

## 15. Effets de calque

Les effets forment une liste ordonnée sur un calque (flou, ombre portée, teinte, remplissage), rendue dans une passe isolée avant le masque, l'opacité et le mode de fusion du calque,
jusqu'à 16 par calque. Leurs valeurs numériques sont animables avec des canaux ciblés `"<layer>::fx<index>"` et l'une des propriétés `fx_blur`, `fx_dx`,
`fx_dy`, `fx_soft`, `fx_amount` (les couleurs ne sont pas animables) :

```
"effects":[{"type":"drop_shadow","dx":4,"dy":6,"blur":8,"color":"#00000099","opacity":0.8}]      // on layer "head"
{"target":"head::fx0","prop":"fx_dx","source":"keys","track":{"keys":[{"frame":0,"value":-6},{"frame":24,"value":6}]}}
```

`check` passe et l'ombre glisse de -6 à +6 pendant la boucle. Une **chaîne de post-effets** (contour, posterize, relight avec cartes de normales...) rend une
image à partir d'une liste JSON à la place :

```
$ echo '[{"type":"outline","width":3,"color":"#222222"},{"type":"posterize","levels":6}]' > chain.json
$ osatura fx biped.json 0 chain.json fxc.png
wrote fxc.png (2 effect(s))
```

Contrôles de sécurité pour les flashs et les mouvements de caméra (une aide d'ingénierie fondée sur la définition publique WCAG 2.3.1, **et non une certification de diffusion**) :

```
$ osatura safety walk.json
flashes      ok    worst 0 transitions/s (frame 0)
red flashes  ok    worst 0 transitions/s (frame 0)
camera       pan 0.00 widths/s, rotation 0°/s, zoom ×1.00/s
$ osatura reduced-motion walk.json rm.json --factor 0.25
wrote rm.json
```

## 16. Onion skin et trajectoires (surcouches)

```
osatura onion p1.json 12 onion.png --before 3 --after 2                  # ghosts of neighbouring frames, raster
osatura onion p1.json onion.svg --frame 12 --before 3 --after 2 --keyed --outline   # SVG; --keyed ghosts key frames
osatura trail p1.json trail.svg --target layer:ball --from 0 --to 23     # wrote trail.svg (24 ticks, 24 key marker(s))
```

`--keyed` dessine les fantômes aux images *clés* voisines plutôt qu'à pas fixe. Les surcouches ne modifient jamais le projet et ne font pas partie d'un export.
Le studio les affiche en direct (`Alt+N` onion, `Alt+M` trajectoires, `Alt+7` panneau).

## 17. Idles procéduraux

```
osatura idle blink  p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
osatura idle breath p1.json i2.json --target torso  --prop bone_scale   --amp 0.03 --period 48
osatura idle sway   p1.json i3.json --target head   --prop bone_angle   --amp 3 --freq 0.5 --seed 2
```

Avec graine : la même commande produit donc la même animation partout. `autorig` ajoute pour vous la respiration et les clignements lorsque les noms correspondent.

## 18. Assurance qualité : check, lint, fuzz, CI

* `check` compile le projet exactement comme le fera le moteur de rendu. Code de sortie 1 en cas d'erreur.
* `lint` trouve les problèmes statiques (code de sortie 1 en cas d'erreurs ; les informations et avertissements ne font pas échouer) : cibles d'IK hors de portée (`L003`), canaux qui
  ne changent jamais (`L005`), clés hors de la timeline (`L007`)...
* `explain`, `coach`, `diff a.json b.json` (`+` ajouté, `-` supprimé, `~` modifié) et `vcs` (gestion de versions avec merge sémantique pour les fichiers de projet).
* **Fuzz** pose le rig au hasard et signale où il casse :

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

  **Un `fuzz` simple est un rapport et se termine avec le code 0 ; ajoutez `--gate` (toute défaillance donne le code 1) ou `--max-fail-rate R` (une fraction) pour en faire un verrou.** La plage d'échantillonnage est la plage complète de chaque paramètre, si bien qu'un rig avec IK
  échoue dans presque tous les échantillons (`98.0%`, 48 `ik_unreachable` sur 50, sur le biped avec foot-lock). Restreignez avec `--only`, `--props`,
  `--margin` et `--ignore KINDS`, choisissez la plage que vous jugez légale et fixez le seuil que vous acceptez.
  `--report out.json`, `--heatmap out.svg` et `--minimize` (plus petite pose reproduisant l'échec) aident. Stratégies : `random`, `lhs`, `halton`, `sweep`, `corners`, `pca`.
* **Goldens et CI.** `golden write walk.json gold.json` enregistre des signatures d'images ; `golden check` compare exactement par défaut (code de sortie 1 à la
  moindre modification ; `--tol T` assouplit la comparaison). Vérifier une animation *différente* contre elles liste les images qui diffèrent : `frame 4 differs: pixel hash changed, thumbnail distance 0.013, ...`, ...
  `ci` englobe cela et davantage dans un manifeste :

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

  `ci run` se termine avec le code 0 quand tout passe, 1 quand un contrôle échoue et 2 en cas d'erreur. `ci bless` accepte les rendus actuels comme nouvelles images de référence,
  `ci diff a.png b.png --metric ssim|de2000|de76|mse` explique un changement avec des cartes de chaleur. Le seuil de 46 % du pose-fuzz dans le manifeste a été calibré sur le
  projet « tel qu'il est maintenant » (l'outil affiche cette remarque) ; resserrez-le délibérément. Les détails figurent dans [CI.md](CI.md).

## 19. Taille et niveau de détail

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

`size-report` montre quels calques, images et canaux coûtent des octets. Quand une plateforme impose un budget, `export-fit` trouve l'export de meilleure qualité qui y tient :

```
$ osatura export-fit walk.json fit.gif --max-size 60KB --format gif
gif -> fit.gif: 44540 bytes raw, 43822 gzip (budget 60000 raw); SSIM mean 0.9271, worst 0.9057 at frame 3, max CIEDE2000 74.98; 8 candidates built
time budget exhausted (60 s, checked inside every candidate): the search stopped early and the best result so far is kept (exit code 0 if it fits the size budget, 1 if not; ...)
$ osatura export-fit walk.json fit.json --max-size 8KB --format lottie
lottie -> fit.json: 14952 bytes raw, 1429 gzip (budget 8000 raw); SSIM mean 0.9350, worst 0.9060 at frame 10, max CIEDE2000 74.98; 5 candidates built
existing exporter at defaults: 76430 bytes raw, 6654 gzip
budget not met: the smallest export found is 14952 bytes (raw), above the budget of 8000 bytes: every knob is at its floor ...
```

La deuxième commande s'est terminée avec le code **1** et a quand même écrit le plus petit fichier trouvé (14952 octets). Les budgets sont vérifiés par défaut sur la taille brute ;
`--measure gzip` vérifie la taille compressée (le Lottie ci-dessus fait 1429 octets en gzip). La recherche est bornée par `--max-seconds` (60 s de temps réel par défaut) ;
dans le build debug, la recherche GIF a été arrêtée par cette limite et a conservé son meilleur résultat.

Pour les dessins comportant de nombreuses courbes et de nombreux maillages, le **niveau de détail** produit des copies simplifiées avec une borne d'erreur *déclarée* en pixels de sortie :

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

`--verify` mesure l'écart réel (code de sortie 1 s'il dépasse la borne) ; dans l'exécution ci-dessus, chaque erreur mesurée était inférieure à sa borne déclarée. Notez le
peu qu'économise un minuscule projet à base de rectangles (de 17507 à 16080 octets) : le LOD est rentable sur un dessin détaillé, pas sur celui-ci.

## 20. Limites assumées

* Le lip-sync repose sur l'énergie et l'alignement du texte, pas sur la reconnaissance de phonèmes.
* Le détecteur de visage est uniquement synthétique (section 10) : il ne rend pas utilisable à lui seul une performance filmée par webcam.
* `make-loop` supprime le déplacement ; `loop-solve` ne le corrige pas ; le bruit ne boucle pas.
* L'audio est tronqué à la longueur du projet ; `mocap-retarget` allonge le projet.
* Lottie, GIF et HTML ne peuvent pas porter toutes les fonctionnalités ; les avertissements d'export et `--report` listent ce qui a été approximé ([EXPORT_FORMATS.md](EXPORT_FORMATS.md)).
* Seul Chromium a été testé pour le studio ([STUDIO.md](STUDIO.md)).
