# Guide du rigging d'OSATURA

Ce guide couvre tout ce qui fait bouger un dessin : os, contraintes, cinématique inverse,
skinning, déformateurs, physique, auto-rigging, symétrie, changement de vue, rigs de visage et travail de caméra.
Chaque exemple a été exécuté avec le vrai binaire (`osatura 0.1.0`, build debug, Linux) et les chiffres
cités sont ceux qu'il a affichés. Si vous n'avez pas encore construit de premier projet, commencez par
[GETTING_STARTED.md](GETTING_STARTED.md). Les options de chaque commande figurent dans
[CLI_REFERENCE.md](CLI_REFERENCE.md) ; le format de fichier est décrit dans [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Licence de ce document : CC-BY-4.0.

![Huit petits rendus : une bande skinnée pliée par un os, trois variantes d'IK (cible hors de portée, stretch 2, soft 0.5) et quatre déformateurs vectoriels (cage, lattice, spine, pin) chacun déplacé à l'image 12.](../../img/rigging-sheet.png)

*Planche-contact des exemples ci-dessous : poids de skin avec FK, FABRIK avec une cible hors de portée,
`stretch = 2`, `soft = 0.5`, déformateurs cage, lattice, spine et pin.*

## 1. Deux façons d'éditer un rig

* **Commandes et `script`.** La plupart des modifications de rig ont une commande (`autorig`, `cage`, `physics`, `skin`...) ou
  une fonction Rhai dans `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Un script est du type tout ou rien : si une requête
  est rejetée, rien n'est écrit. Un script peut contenir au plus 5000 requêtes.
* **JSON.** Quelques éléments n'ont ni commande ni setter de script : `soft` et `stretch` de l'IK, les ressorts, les déformateurs
  vectoriels (lattice, spine, pin) et les poids de skin par ancre. Modifiez le JSON du projet (n'importe quel éditeur, un
  script Python ou le panneau JSON du studio), puis lancez `osatura check`.

`check`, `lint`, `explain` et `render --bones` sont les quatre commandes que vous utiliserez en permanence pendant le rigging.
`explain` est la plus précieuse : elle liste, dans l'ordre, chaque valeur de repos, canal, contrainte et contribution d'IK
qui a produit la transformation finale d'un os, de sorte que vous n'avez jamais à deviner pourquoi quelque chose se trouve là où il est.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Os et hiérarchies

Un os possède un `name`, un `parent` (aucun = racine), une `length`, un `rest_offset` (la position de sa tête, dans le repère
**du parent** ; pour un os racine, il s'agit du canevas) et un `rest_angle` en degrés relatif au
parent. Conventions :

* y pointe vers le bas ; l'angle 0 pointe selon +x ; les angles positifs tournent dans le sens horaire à l'écran ; **90 est vers le bas, -90 vers le haut**.
* `rest_offset` est tourné par l'angle monde du parent. L'enfant d'un os qui pointe vers le haut et dont le
  `rest_offset` vaut (70, 0) se situe 70 unités *le long du parent vers le haut*, et non 70 unités à droite du canevas.
* Les os doivent être listés en commençant par les parents. Tout autre ordre est rejeté :
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

Les canaux d'animation d'un os sont des **deltas par rapport à la pose de repos** : `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. Une valeur de clé de 0 signifie « repos ».

Une chaîne minimale de 3 os utilisée par la plupart des exemples ci-dessous (`lab.json` ; elle est générée, voir la section 4) :

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Commandes utiles : `reparent <in> <out> <bone> <new_parent|none>` conserve la pose de repos (vérifié : `b3`
reste avec la tête en (220,150) lorsqu'il est rattaché à none), `set_parent` dans un script en est la forme de plus bas niveau, et
`bone-vis <in> <out> <bone> <from> <to>` n'affiche un os dans les surcouches que sur une plage d'images.

### Rattachement rigide : le rig le plus simple

Un calque peut suivre un os de façon rigide avec `set_layer("layer", "bone", "bone_name")`. C'est ainsi que sont réalisés le
personnage du tutoriel et le biped ci-dessous : des pièces séparées, chacune rattachée à un os. Cela ne demande aucun
poids et résiste à tous les exports. Choisissez cette méthode lorsque votre dessin est déjà découpé en pièces.

**Piège.** Rattachez les calques lorsque les os sont dans leur pose de repos et dessinez chaque pièce de sorte que son articulation soit
sur la tête de son os. Une pièce dessinée avec quelques unités de décalage pivotera autour du mauvais point.

**Piège.** Ne réglez pas l'angle de repos des os des bras à ±100 « pour obtenir une T-pose » avec un décalage de repos de 0 : les
deux bras partent d'un même point. Donnez à chaque épaule un `rest_offset` latéral et un angle de 180 ou
0, comme dans le biped (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Contraintes

Les contraintes sont listées dans le tableau `constraints` du projet et s'exécutent **dans l'ordre de la liste** après les
canaux. Créez-les avec `add_aim`, `add_copy_rotation` et `add_pin` (script) ou en JSON.

| Contrainte | Script | Effet |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | fait pivoter `bone` pour qu'il pointe vers la tête de l'os `target` |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` prend (une part pondérée de) la rotation monde de `source` |
| Pin | `add_pin(name, bone, target, weight)` | déplace la tête de `bone` sur la tête de `target` |

Vérifié avec la chaîne de 3 os, `target` étant un os en (250,40) :

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

Le poids mélange linéairement l'angle animé et l'angle contraint, si bien que 0.5 réduit la rotation de moitié.

Les limites d'angle sont des propriétés de l'os (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Elles bornent la rotation **animée** : si vous animez `b1` à 60 degrés, il est maintenu à 20 :

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Piège : l'ordre compte.** Un copy-rotation placé sur le *parent* d'un os soumis à un aim interagit avec l'aim.
Si un os finit par pointer dans une direction étrange, lancez `explain` sur lui et sur son parent, lisez la liste des contributions
de haut en bas, puis réordonnez le tableau `constraints`.

## 4. Cinématique inverse

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Deux os : un solveur analytique. `bend` (+1 ou -1) choisit dans quel sens l'articulation se plie.
* Trois os ou plus : FABRIK. `iterations` est un plancher ; le solveur s'arrête plus tôt dès que l'extrémité est à moins de
  0.01 px, et sinon s'exécute jusqu'à un plafond strict.
* `mix` est le mélange FK/IK (0 = la pose FK animée par clés, 1 = IK complète). Il peut être animé avec la propriété
  `ik_mix`, c'est ainsi qu'un bras passe de FK à IK. `autorig` ajoute des chaînes avec `mix 0` afin que l'animation
  FK existante continue de fonctionner.
* La cible est la **tête** d'un os ; ne lui donnez aucun dessin et une petite longueur (`goal`, `hand_ctrl`).

### Cibles hors de portée : tendue, soft, stretch

La chaîne `b1..b3` a une portée de 240 depuis sa racine (60,150). Si l'on déplace le goal en (330,60), à 285 de distance :

| Réglage | Résultat (position de l'extrémité) |
|---|---|
| par défaut | complètement étendue vers le goal, extrémité (287.7, 74.1), exactement à 240 de la racine |
| `"stretch": 2` | la chaîne s'allonge, l'extrémité atteint (330.0, 60.0) |
| `"soft": 0.5` (3 os) | la chaîne reste pliée, extrémité (258.8, 83.7) ; elle ne se tend pas |

`soft` (0 à 0.9) et `stretch` (1 à 4) sont des champs de la chaîne d'IK dans le JSON du projet ; il n'existe
ni commande ni setter de script :

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

Sur le bras de Buddy (portée de 80 depuis l'épaule en (160,140)) avec le goal en (250,140), à 90 de distance :

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

Le soft IK sur un membre à deux os approche l'extension complète de façon asymptotique : lorsque le goal s'éloigne à 100, 120,
135 et 150, le x de l'extrémité se stabilise à 231.1, 234.6, 236.3 et 237.4 sans jamais atteindre 240. C'est le comportement
voulu, « sans à-coup du genou ». Avec en plus un `stretch` supérieur à 1, la zone soft est la dernière fraction `soft` de la portée
étirée (`reach x stretch`) : la chaîne s'allonge continûment et s'amortit vers sa longueur maximale (avant la vague 7,
`soft` était ignoré au-delà de la portée complète et la chaîne se tendait brusquement à ce moment-là).

**Piège.** `lint` signale les cibles hors de portée comme des informations, et non comme des erreurs :
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Lisez ces lignes ; une jambe qui « flotte » est presque toujours l'une d'elles.

**Piège.** `fuzz` (voir [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) échantillonne des poses aléatoires, y compris des positions de
contrôleurs très éloignées de la portée, si bien qu'un rig avec IK affiche un grand nombre de `ik_unreachable`. C'est un constat
sur la plage échantillonnée, pas un bug.

## 5. Skinning

OSATURA dispose de quatre mécanismes de skinning ; choisissez selon le type de dessin dont vous disposez.

### 5.1 Ancres vectorielles avec poids (JSON)

Les ancres d'un tracé vectoriel peuvent porter des `weights`, une liste de `{bone, weight}`. L'ancre
suit alors un mélange linéaire des transformations des os. Voici une bande de 12 ancres skinnée sur les
trois os ; les `weights` passent progressivement de `b1` à `b2` puis à `b3` :

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

À l'image 12, l'angle monde de `b3` est de 50.0 degrés et sa tête se trouve en (191.4, 211.3) (d'après `explain`). La bande se plie
en douceur entre les articulations parce que les poids s'y chevauchent.

**Piège.** Aucune commande ne calcule les poids vectoriels à votre place : l'API du projet propose un skinner
vectoriel automatique (`Project::auto_skin`), mais c'est une fonction de bibliothèque, utilisée par la démo, et non exposée comme
commande CLI. Pour un dessin vectoriel, utilisez le rattachement rigide (section 2) ou écrivez les poids comme ci-dessus.

**Piège.** Peu d'ancres, peu d'endroits où plier. Un simple rectangle a quatre ancres, il ne peut donc se plier qu'à
ses coins. Subdivisez la forme (ajoutez des ancres le long des bords) avant de la skinner ou d'utiliser un déformateur.

### 5.2 Maillages d'images : `mesh-trace` et `skin-diffuse`

Pour un dessin matriciel, tracez la silhouette alpha en un maillage de triangles, puis calculez les poids par diffusion de chaleur :

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

`skin-diffuse <project> <layer> <out> [--influences N]` attribue à chaque sommet du maillage jusqu'à N poids d'os. Elle
requiert un maillage : sur un calque d'image qui n'en a pas, elle s'arrête avec `layer 'flag' has no mesh`. Avec un seul os,
chaque sommet reçoit le poids 1.0 sur celui-ci (le drapeau n'a qu'un os, `mast`), ce qui est correct mais peu
intéressant ; utilisez au moins deux os pour voir un mélange. `mesh-trace` dispose aussi de `hotspot=x,y,r` pour rendre le maillage
plus dense là où vous attendez une flexion, et d'options pour skinner pendant le tracé (`bones=R`, `skin=1` ; voir
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Skin sets (tenues, expressions, états d'endommagement)

La commande `skin` repose sur une autre idée : un **skin set** échange ensemble des groupes entiers de calques (et, en option,
des os, des contraintes et des chaînes d'IK).

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

`apply` affiche un skin et masque les autres du même groupe. `schedule` anime ce choix avec des **clés en escalier
sur l'opacité du calque** (le projet n'a pas de drapeau « visible » animable) ; utilisez-le donc sur des calques dont vous n'animez pas l'opacité par ailleurs.
Les os propres à un skin inactif ne peuvent pas être supprimés du rig sans casser les indices ; ils sont donc **réduits** :
un canal `multiply` sur `bone_scale` avec des clés en escalier 0 et 1 ramène l'échelle de l'os à 0, et tout ce qui le suit
ou lui est pondéré se réduit à un point :

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Règles imposées par la validation : un os appartient à un seul skin au plus ; tout descendant d'un os de skin appartient au même skin ; les os de skin
ne doivent pas avoir de `scale_limits`. Utilisez `skin rig <group> <name> <constraint,...|-> <ik,...|->` pour les contraintes et chaînes d'IK qui
ne doivent être actives qu'avec un skin donné.

### 5.4 Pièces rigides plus un calque switch

Pour ce qui change de forme de manière discrète (bouches, mains, yeux), placez les alternatives dans un calque `switch`
et animez `layer_switch` par clés. Voir la section 11.

## 6. Déformateurs

Les déformateurs déplacent les points d'un calque. Ils sont listés dans les `deformers` du calque, s'animent par
des canaux avec une cible en trois parties `"layer::deformer::point"`, et agissent sur les **ancres** d'un tracé ou sur les sommets d'un
maillage d'image. **Si une forme a trop peu d'ancres, elle se déformera à peine : subdivisez-la d'abord.**

### Cage (coordonnées de valeur moyenne)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` enveloppe un calque dans un polygone (`auto` = limites avec marge, `hull` = enveloppe convexe, `verts=N` = segments par
côté) ; le contenu à l'intérieur suit les sommets du polygone, le contenu à l'extérieur n'est pas touché. `cage-pose` anime par clés les
décalages de sommets (canaux `cage_dx`, `cage_dy` avec la cible `strip::cage::1`). `timeline` les liste sous
« Deformers and paths ». Un canal verrouillé est refusé.

### Lattice, spine, pin (JSON)

Ces trois-là n'ont pas de commande ; ajoutez-les aux `deformers` d'un calque et animez-les avec des canaux :

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice** : une grille libre (free-form). L'identifiant de point est `col,row`. Idéal pour le squash and stretch.
* **Spine** : une polyligne dotée d'un `radius` d'influence ; déplacer un point de la spine plie le dessin voisin (queues, tentacules).
* **Pin** : déformation de type puppet-pin ; chaque pin tire les points situés dans son `radius`.

Chacun d'eux s'est rendu correctement dans la planche-contact ci-dessus, appliqué à une bande sans poids d'os. Un tracé
qui est *aussi* skinné sur des os sera déformé par les deux ; retirez les poids de la bande de test si vous voulez voir un
déformateur seul.

### Déformer une sélection : `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` fait pivoter, met à l'échelle ou déplace une sélection de points de contrôle (cage, lattice, cloth, pin, spine, ou un
maillage d'image en grille) autour d'un pivot (`x,y`, `bone:NAME`, `centroid`, `center`) et anime le résultat par clés. `radius` ajoute une
décroissance de sélection douce (`linear`, `smooth`, `gauss`, `sharp`, `constant`) ; l'exemple ci-dessus sélectionnait les sommets 1-2 et
la décroissance a ajouté 7 et 8, de sorte que 8 canaux ont été animés. `soft-scale` effectue un redimensionnement proportionnel (`aspect=free` laisse les
deux axes différer). `layer-pivot` déplace le pivot d'un calque sans déplacer son dessin.

### Verrous de maillage et glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]` : les sommets du follower
  suivent la surface de l'hôte par déplacement barycentrique. Utilisez-le pour garder un tatouage, une pièce ou un bouton sur un tissu
  qui se déforme. Les cycles sont refusés. `mesh-lock list` a affiché `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]` : des paires de sommets
  de deux calques sont soudées de sorte que `b` suive `a`. `glue list` a affiché
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Tissu

Le tissu (cloth) exige un maillage en **grille**. Un maillage libre tracé est refusé :
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Demandez une grille avec `cols=` et `rows=` :

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Ce bake a pris 0.43 s en build debug et produit 180 canaux animés (6480 clés), de sorte que le résultat est entièrement
modifiable, s'exporte comme n'importe quelle autre animation et est déterministe. `pin=` accepte `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` ou `...@bone` pour épingler à un os. `cols`/`rows` définissent la grille de simulation ; l'image rendue est déformée par celle-ci
et vous pouvez voir de faibles raccords de triangles dans un rendu PNG à fort zoom.

## 7. Physique et ressorts

### Physique bakée : `physics`

`physics` simule une chaîne d'os (gravité, vent, collisions) et bake le résultat en clés d'angle d'os ordinaires.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` vaut `hair`, `tail`, `rope`, `antenna` ou `cape`.
* `circle=x,y,r[@bone];...` et `capsule=x1,y1,x2,y2,r[@bone];...` sont des colliders ; `@bone` fait suivre un os à un collider.
* `drive=bone:gain` fait pousser la chaîne par le mouvement d'un autre os ; `vortex=bone|x,y:swirl[:radial[:radius]]`
  ajoute un champ de force tourbillonnant ; `follow=bone` et `lateral=` modèlent la façon dont la chaîne suit un parent.
* `merge=keep` (par défaut) **n'écrase pas** un os qui a déjà une animation :
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Indiquez `merge=replace` lorsque c'est bien ce que vous voulez.
* Les chaînes de style `b2,b3/b1` et un os inconnu (`no bone 'b9'`) sont validés.

### Ressorts vivants : `springs`

Un `Spring` ajoute à un os un suivi amorti à l'évaluation, sans rien baker. Il se définit uniquement en JSON :

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` est en Hz (plus élevée = plus rigide), `damping` est un rapport (en dessous de 1 le ressort oscille, 1 correspond à l'amortissement critique), et `warmup`
(activé par défaut) précalcule une passe afin que les boucles démarrent en régime établi. `explain` indique :
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Pour retrouver un ressenti que vous avez dessiné, ajustez-le :

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Piège.** Les ressorts ne font pas partie de Lottie. Bakez-les (`physics`) si vous exportez vers un format qui n'a pas
d'équivalent ; `export --report` liste ce qui a été approximé.

## 8. Auto-rig, étiquetage et `coach`

Nommez vos os et vos calques avec des mots standard (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`) et OSATURA peut
reconnaître leurs rôles :

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

`autorig <in> <out> [seed]` ajoute ensuite une automatisation non destructive à partir de ces rôles :

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

Le projet est passé de 11 os et 0 chaîne d'IK à 15 os et 4 chaînes d'IK avec 3 nouveaux canaux. Tout ce qui a été
ajouté est de la donnée ordinaire : supprimez ce dont vous ne voulez pas. Les parties manquantes sont ignorées avec une raison
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`) ; lisez donc le rapport.

**Piège.** `autorig` est une commande de projet à projet et la sortie peut être l'entrée. Sur un fichier auquel vous tenez,
écrivez vers un nouveau nom : `osatura autorig buddy.json buddy.json` remplace `buddy.json` (l'écriture est atomique, mais
il n'y a pas d'annulation).

`coach <project>` lit l'animation et affiche des suggestions sur le timing et les principes d'animation ; `lint` vérifie le rig
statiquement. Les deux sont indicatifs.

## 9. Symétrie et miroir

`mirror <in> <out>` reflète tous les canaux animés par rapport à l'axe vertical : les cibles échangent leur côté (préfixes `L_`/`R_`,
suffixes `_L`/`_R` et `_l`/`_r`, mots left/right), et les rotations et translations horizontales
changent de signe. Les canaux reflétés **remplacent** les canaux du projet. Sur le biped en marche, les clés de la cuisse gauche
`0.0, 10.7, 19.8, 25.9` sont devenues celles de la cuisse droite `-0.0, -10.7, -19.8, -25.9` et inversement, et `hips` (sans côté)
a conservé ses valeurs. `diff` liste les six canaux modifiés :

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Piège.** Sur un projet sans noms gauche/droite, `mirror` écrit silencieusement une copie inchangée et affiche
`wrote`. Faites toujours un `diff` du résultat. Nommez vos côtés de façon cohérente (`_l`/`_r`) avant d'animer.

## 10. Changement de vue et changement de parent

### Turnarounds : `view-swap`

Un turnaround (face, trois-quarts, profil) est un calque switch avec un enfant par vue, piloté par la propriété d'un os contrôleur :

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Canal produit : un canal `driven` `layer_switch` ← `view_ctrl.bone_offset_x` avec une courbe en escalier (enfant 0 en dessous de 0,
enfant 2 à partir de 0, enfant 4 à partir de 40). Animez le `bone_offset_x` de `view_ctrl` pour changer de vue. Les index d'enfants doivent exister et le calque
doit être un switch (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` crée un **pose slider**, un os contrôleur `ctrl_<name>` dont
l'`offset_x` mélange des poses mémorisées :

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Changer de parent au fil du temps : `parent-switch`

Pour passer un objet d'une main à l'autre (ou pour ramasser un accessoire) sans à-coup :

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Elle bake des canaux de décalage et d'angle d'os (ici 3 canaux, une clé à chaque image) afin que l'os conserve sa position
monde lors du changement. Elle ne fonctionne pas pour les os avec IK, contraintes ou ressorts.

## 11. Rigs de visage

### Formes de bouche

Dessinez les formes de bouche comme enfants d'un **calque switch** dans cet ordre, que suppose toute commande de lip-sync :
**rest, small, open, round, wide**. La bouche du biped est `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Si votre switch a moins d'enfants, `--map 0,1,2,3,4` (cinq nombres, un par forme) choisit l'enfant utilisé par chaque forme ; la valeur par défaut
suit le nombre d'enfants (1 enfant : tout ; 2 : fermée+ouverte ; 3 : fermée+ouverte+ronde/large ; 4 : fermée+petite+ouverte+ronde/large ; 5+ : dans cet ordre).

Rattachez le switch à l'os de la tête (`set_layer("mouth", "bone", "head")`) pour qu'il tourne avec la tête. Son animation fait
l'objet de la [section lip-sync du guide d'animation](ANIMATION_GUIDE.md#8-lip-sync-et-parole).

### Yeux et masquage

`eye-clip <in> <out>` masque les calques d'iris et de pupille par la forme de l'œil du même côté, de sorte qu'une pupille ne puisse pas sortir de
l'œil. `autorig` l'exécute. Sur un rig sans calques d'iris/pupille, elle écrit le fichier inchangé. Les clignements sont un
`layer_scale_y` animé par clés ou procédural sur le calque de l'œil :

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` génèrent des canaux procéduraux à graine (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`) : la même graine donne les mêmes instants de clignement sur toutes les machines.

### Piloter un visage à partir d'une performance

`face-synth`, `face-track`, `face-bake` et le panneau Face du studio transforment un flux de repères en canaux. Ils sont
présentés, avec des chiffres de précision honnêtes, dans la [section sur le suivi du visage du guide d'animation](ANIMATION_GUIDE.md#10-suivi-du-visage-et-ses-limites).

## 12. Caméra, plans et parallaxe

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` enregistre un cadrage de caméra nommé (centre de la caméra et zoom) ;
  `camera-seq <in> <out> <frame:shot[:cut],...>` anime la caméra par clés à travers des plans enregistrés ; `timeline` affiche les
  canaux de caméra obtenus.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` répartit un facteur `parallax` sur les
  calques de premier niveau (les plus lointains bougent moins lorsque la caméra fait un panoramique).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` floute chaque calque de premier niveau selon
  `|parallax - focus|` ; `dof-export` rend une plage vers un GIF, un APNG ou un dossier. La mise au point et l'intensité suivent
  les canaux `camera_focus` / `camera_blur` sauf si vous les fixez.

Vérifié sur le biped en marche :

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Les effets de calque (flou, ombre portée, teinte, remplissage, contour, posterize...) peuvent s'appliquer par calque ou en chaîne : voir le
[guide d'animation](ANIMATION_GUIDE.md#15-effets-de-calque).

## 13. Une checklist de rigging

1. `check` après chaque modification structurelle ; `lint` avant d'animer.
2. Gardez la pose de repos sobre et symétrique ; animez des deltas par rapport à elle.
3. Placez les contrôleurs (goals d'IK, `view_ctrl`, `root_ctrl`) dans le rig sous forme d'os sans dessin.
4. Préférez les pièces rigides pour un dessin cartoon, ne skinnez que ce qui doit se plier, et subdivisez avant de déformer.
5. Utilisez `explain` quand quelque chose ne va pas ; c'est plus rapide que de deviner.
6. Bakez la physique et le tissu si l'animation doit survivre dans un format dépourvu de ces fonctionnalités, et lisez `export --report`.
7. Écrivez vers de nouveaux noms de fichier tant que vous ne faites pas confiance à une commande ; les commandes de projet à projet peuvent écraser leur entrée.

## 14. Limites à connaître

* Le skinning du dessin vectoriel est un **linear blend** (pas de dual quaternion), avec des poids que vous fournissez ; il n'existe pas de skinner
  vectoriel automatique en CLI.
* Lattice, spine, pin, `soft`/`stretch` de l'IK et ressorts ne se définissent qu'en JSON.
* La physique et le tissu sont des bakes : modifiez le rig et bakez à nouveau. `physics` n'écrase pas l'animation existante sauf demande explicite.
* Le tissu exige un maillage en grille ; les maillages libres tracés servent au skinning et aux verrous de maillage.
* Lottie et GIF ne peuvent pas tout représenter (traits à largeur variable, dégradés cisaillés, ressorts...). Des avertissements sont
  affichés à l'export ; voir [EXPORT_FORMATS.md](EXPORT_FORMATS.md) et [LIMITATIONS.md](LIMITATIONS.md).
* Le détecteur de visage n'est entraîné que sur des visages synthétiques (section 11 et guide d'animation).
