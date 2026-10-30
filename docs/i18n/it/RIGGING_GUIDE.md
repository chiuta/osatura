# Guida al rigging di OSATURA

Questa guida tratta tutto ciò che fa muovere un disegno: ossa, vincoli, cinematica inversa,
skinning, deformatori, fisica, auto-rigging, mirroring, view swapping, rig del volto e lavoro con la camera.
Ogni esempio è stato eseguito con il binario reale (`osatura 0.1.0`, build di debug, Linux) e i numeri
riportati sono quelli che ha stampato. Se non hai ancora costruito un primo progetto, parti da
[GETTING_STARTED.md](GETTING_STARTED.md). I flag di ogni comando si trovano in
[CLI_REFERENCE.md](CLI_REFERENCE.md); il formato dei file è descritto in [PROJECT_FORMAT.md](PROJECT_FORMAT.md).

Licenza di questo documento: CC-BY-4.0.

![Otto piccoli render: una striscia con skinning piegata da un osso, tre varianti di IK (bersaglio fuori portata, stretch 2, soft 0.5) e quattro deformatori vettoriali (cage, lattice, spine, pin), ciascuno spostato al fotogramma 12.](../../img/rigging-sheet.png)

*Foglio di contatto degli esempi che seguono: pesi di skin con FK, FABRIK con un bersaglio irraggiungibile,
`stretch = 2`, `soft = 0.5`, deformatori cage, lattice, spine e pin.*

## 1. Due modi di modificare un rig

* **Comandi e `script`.** La maggior parte delle modifiche al rig ha un comando (`autorig`, `cage`, `physics`, `skin`...) oppure
  una funzione Rhai in `osatura script` (`add_bone`, `add_ik`, `add_aim`, `add_copy_rotation`, `add_pin`,
  `set_layer`, `set_bone`, `set_parent`, `delete_bone`...). Uno script è tutto o niente: se anche una sola richiesta
  viene rifiutata, non viene scritto nulla. Uno script può contenere al massimo 5000 richieste.
* **JSON.** Alcune cose non hanno né un comando né un setter negli script: `soft` e `stretch` dell'IK, le molle, i deformatori
  vettoriali (lattice, spine, pin) e i pesi di skin per singola ancora. Modifica il JSON del progetto (con un qualsiasi editor, uno
  script Python o il pannello JSON dello studio) ed esegui poi `osatura check`.

`check`, `lint`, `explain` e `render --bones` sono i quattro comandi che userai di continuo mentre fai il rigging.
`explain` è il più prezioso: elenca, in ordine, ogni valore di riposo, canale, vincolo e contributo IK che ha prodotto
la trasformazione finale di un osso, così non devi mai tirare a indovinare perché qualcosa si trovi dov'è.

```
$ osatura explain con1.json b3 0
- bone 'b3' (length 80), parent 'b2'
- ancestry: b3 ← b2 ← b1
- rest pose: offset (80.00, 0.00), angle 0.00°
- no channels animate this bone directly
- constraint 'b3_aims' (Aim { target: "target", offset: 0.0 }) at weight 1.00
- result at frame 0: head (203.6, 101.4), tip (173.5, 27.3), world angle -112.1°
```

## 2. Ossa e gerarchie

Un osso ha un `name`, un `parent` (nessuno = radice), una `length`, un `rest_offset` (dove si trova la sua testa, nel
sistema di riferimento **del genitore**; per un osso radice è il canvas) e un `rest_angle` in gradi relativo al
genitore. Convenzioni:

* y punta verso il basso; l'angolo 0 punta lungo +x; gli angoli positivi ruotano in senso orario sullo schermo; **90 è giù, -90 è su**.
* `rest_offset` viene ruotato dell'angolo mondo del genitore. Un figlio di un osso che punta verso l'alto e ha
  `rest_offset` (70, 0) si trova 70 unità *lungo il genitore, verso l'alto*, non 70 unità a destra rispetto al canvas.
* Le ossa devono essere elencate partendo dai genitori. Qualsiasi altro ordine viene rifiutato:
  `bone 'a' has parent 'ghost' which is missing or listed after it`.

I canali di animazione su un osso sono **delta rispetto alla posa di riposo**: `bone_angle`, `bone_offset_x`,
`bone_offset_y`, `bone_scale`. Un valore di chiave pari a 0 significa «riposo».

Una catena minima di 3 ossa usata dalla maggior parte degli esempi seguenti (`lab.json`; viene generata, vedi la sezione 4):

```
b1  root, length 80, head at (60,150), angle 0
b2  child of b1, rest_offset (80,0)
b3  child of b2, rest_offset (80,0)
```

Comandi utili: `reparent <in> <out> <bone> <new_parent|none>` mantiene la posa di riposo (verificato: `b3`
resta con la testa in (220,150) quando viene riassegnato a none), `set_parent` in uno script è la forma di livello più basso, e
`bone-vis <in> <out> <bone> <from> <to>` mostra un osso solo negli overlay per un intervallo di fotogrammi.

### Attacco rigido: il rig più semplice

Un livello può viaggiare in modo rigido su un osso con `set_layer("layer", "bone", "bone_name")`. Così sono fatti il
personaggio del tutorial e il bipede qui sotto: pezzi separati, ciascuno attaccato a un osso. Non servono
pesi e funziona con ogni esportazione. Sceglilo quando la tua grafica è già tagliata in pezzi.

**Insidia.** Attacca i livelli mentre le ossa sono nella posa di riposo e disegna ogni pezzo in modo che la sua articolazione
cada sulla testa del suo osso. Un pezzo disegnato con qualche unità di scarto oscillerà attorno al punto sbagliato.

**Insidia.** Non impostare l'angolo di riposo delle ossa del braccio a ±100 «per ottenere una T-pose» con un offset di riposo pari a 0: le
due braccia partono da un unico punto. Dai a ciascuna spalla un `rest_offset` laterale e un angolo di 180 oppure
0, come nel bipede (`add_bone("arm_upper_l", "torso", 50, 62, 22, 180)`).

## 3. Vincoli

I vincoli sono elencati nell'array `constraints` del progetto e vengono eseguiti **nell'ordine della lista** dopo i
canali. Creali con `add_aim`, `add_copy_rotation` e `add_pin` (script) oppure nel JSON.

| Vincolo | Script | Effetto |
|---|---|---|
| Aim | `add_aim(name, bone, target[, offset, weight])` | ruota `bone` in modo che punti alla testa dell'osso `target` |
| Copy rotation | `add_copy_rotation(name, bone, source, weight)` | `bone` assume (una quota pesata della) la rotazione mondo di `source` |
| Pin | `add_pin(name, bone, target, weight)` | sposta la testa di `bone` sulla testa di `target` |

Verificato con la catena di 3 ossa, con `target` un osso in (250,40):

```
add_aim("b3_aims", "b3", "target", 0, 1);        // weight 1   -> world angle -74.7 deg
add_aim("b3_aims", "b3", "target", 0, 0.5);      // weight 0.5 -> world angle -37.4 deg
```

Il peso fa una miscela lineare tra l'angolo animato e l'angolo vincolato, quindi 0.5 dimezza la rotazione.

I limiti angolari sono proprietà dell'osso (`set_bone(name, "angle_min", -20); set_bone(name, "angle_max", 20)`).
Limitano la rotazione **animata**: se porti `b1` a 60 gradi con una chiave, viene mantenuto a 20:

```
$ osatura explain l2.json b1 12
- channel #0 BoneAngle: keys → 60.000 (2 key(s), Hold outside range); mode replace, weight 1.00
- angle limits: animated rotation is clamped to [-20, 20]°
- result at frame 12: head (60.0, 150.0), tip (135.2, 177.4), world angle 20.0°
```

**Insidia: l'ordine conta.** Un copy-rotation posto sul *genitore* di un osso con aim interagisce con l'aim.
Se un osso finisce per puntare in una direzione strana, esegui `explain` su di esso e sul suo genitore e leggi l'elenco dei contributi
dall'alto verso il basso, poi riordina l'array `constraints`.

## 4. Cinematica inversa

```
add_bone("goal", "", 10, 200, 60, 0);
add_ik("chain", ["b1","b2","b3"], "goal", 1);   // name, bones root->tip, target bone, bend
```

* Due ossa: un solver analitico. `bend` (+1 o -1) sceglie in quale direzione si piega l'articolazione.
* Tre o più ossa: FABRIK. `iterations` è un minimo; il solver si ferma in anticipo quando la punta è entro
  0.01 px e altrimenti procede fino a un tetto massimo fisso.
* `mix` è la miscela FK/IK (0 = la posa FK con le chiavi, 1 = IK completo). Può essere animato con la proprietà
  `ik_mix`, ed è così che un braccio passa da FK a IK. `autorig` aggiunge catene con `mix 0` in modo che l'animazione FK
  esistente continui a funzionare.
* Il bersaglio è la **testa** di un osso; non dargli grafica e dagli una lunghezza piccola (`goal`, `hand_ctrl`).

### Bersagli irraggiungibili: dritto, soft, stretch

La catena `b1..b3` ha una portata di 240 dalla sua radice (60,150). Spostando il goal in (330,60), a 285 di distanza:

| Impostazione | Risultato (posizione della punta) |
|---|---|
| predefinita | completamente estesa verso il goal, punta (287.7, 74.1), esattamente a 240 dalla radice |
| `"stretch": 2` | la catena si allunga, la punta raggiunge (330.0, 60.0) |
| `"soft": 0.5` (3 ossa) | la catena resta piegata, punta (258.8, 83.7); non si raddrizza |

`soft` (da 0 a 0.9) e `stretch` (da 1 a 4) sono campi della catena IK nel JSON del progetto; non esiste
alcun comando né setter negli script:

```
python3 - <<'EOF'
import json
d = json.load(open('03-ik.json'))        # the Buddy project of GETTING_STARTED.md
d['ik'][0]['soft'] = 0.5                 # or: d['ik'][0]['stretch'] = 1.5
json.dump(d, open('ik_soft.json', 'w'))
EOF
osatura check ik_soft.json
```

Sul braccio di Buddy (portata 80 dalla spalla in (160,140)) con il goal in (250,140), a 90 di distanza:

```
soft 0,   stretch 1    tip (240.0, 140.0)    straight, 10 short of the goal
soft 0.5, stretch 1    tip (228.5, 140.0)    held back, elbow still bent
soft 0,   stretch 1.5  tip (250.0, 140.0)    arm lengthened, goal reached
```

Il soft IK su un arto a due ossa si avvicina alla completa estensione in modo asintotico: man mano che il goal si allontana a 100, 120,
135 e 150 di distanza, la x della punta si stabilizza a 231.1, 234.6, 236.3 e 237.4 e non raggiunge mai 240. È il comportamento
voluto, quello del «nessun scatto del ginocchio». Con `stretch` anche superiore a 1, la zona soft è l'ultima frazione `soft` della
portata allungata (`reach x stretch`): la catena si allunga in modo continuo e raggiunge dolcemente la sua lunghezza massima (prima della wave 7
`soft` veniva ignorato oltre la portata piena e in quel punto la catena scattava dritta).

**Insidia.** `lint` segnala i bersagli irraggiungibili come informazioni, non come errori:
`Info L003 [leg_ik_l] target 'foot_goal_l' is out of reach at frame 12 (148 px > 144 px); the chain will be fully stretched`.
Leggi quelle righe: una gamba che «fluttua» è quasi sempre una di esse.

**Insidia.** `fuzz` (vedi [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md)) campiona pose casuali, comprese posizioni dei controller
molto fuori portata, quindi un rig con IK mostra un conteggio elevato di `ik_unreachable`. È un'affermazione
sull'intervallo campionato, non un bug.

## 5. Skinning

OSATURA ha quattro meccanismi di skinning; scegli in base al tipo di grafica che hai.

### 5.1 Ancore vettoriali con pesi (JSON)

Le ancore di un tracciato vettoriale possono portare `weights`, un elenco di `{bone, weight}`. L'ancora
segue allora una miscela lineare delle trasformazioni delle ossa. Questa è una striscia di 12 ancore con skinning sulle
tre ossa; i `weights` sfumano da `b1` a `b2` a `b3`:

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

Al fotogramma 12 l'angolo mondo di `b3` è 50.0 gradi e la sua testa è in (191.4, 211.3) (da `explain`). La striscia si piega
in modo morbido tra le articolazioni perché lì i pesi si sovrappongono.

**Insidia.** Non esiste alcun comando che calcoli per te i pesi vettoriali: l'API del progetto ha uno skinner
vettoriale automatico (`Project::auto_skin`), ma è una funzione di libreria, usata dalla demo, e non è esposta come
comando della CLI. Per la grafica vettoriale usa l'attacco rigido (sezione 2) oppure scrivi i pesi come sopra.

**Insidia.** Poche ancore, pochi punti in cui piegarsi. Un semplice rettangolo ha quattro ancore, quindi può piegarsi solo ai
suoi angoli. Suddividi la forma (aggiungi ancore lungo i bordi) prima di applicare lo skinning o un deformatore.

### 5.2 Mesh di immagine: `mesh-trace` e `skin-diffuse`

Per la grafica raster, traccia la silhouette dell'alpha in una mesh di triangoli, poi calcola i pesi per diffusione del calore:

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

`skin-diffuse <project> <layer> <out> [--influences N]` assegna a ogni vertice della mesh fino a N pesi di ossa. Richiede
una mesh: su un livello immagine che non ne ha una si ferma con `layer 'flag' has no mesh`. Con un solo osso
ogni vertice riceve peso 1.0 su di esso (la bandiera ha un solo osso, `mast`), il che è corretto ma poco
interessante; usa almeno due ossa per vedere una miscela. `mesh-trace` ha anche `hotspot=x,y,r` per rendere la mesh
più fitta dove ti aspetti una piegatura, e opzioni per fare lo skinning durante il tracciamento (`bones=R`, `skin=1`; vedi
[CLI_REFERENCE.md](CLI_REFERENCE.md)).

### 5.3 Skin set (outfit, espressioni, stati di danno)

Il comando `skin` è un'idea diversa: uno **skin set** scambia insieme interi gruppi di livelli (e, facoltativamente,
ossa, vincoli e catene IK).

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

`apply` mostra una skin e nasconde le altre dello stesso gruppo. `schedule` anima la scelta con **chiavi a gradino
sull'opacità del livello** (il progetto non ha un flag «visible» animabile), quindi usalo su livelli la cui opacità non animi già per altri scopi.
Le ossa per-skin inattive non possono essere eliminate dal rig senza rompere gli indici, quindi vengono **collassate**:
un canale `multiply` su `bone_scale` con chiavi a gradino 0 e 1 porta a 0 la scala dell'osso, e tutto ciò che viaggia
su di esso o è pesato su di esso si riduce a un punto:

```
{"target":"tail","prop":"bone_scale", ... "keys":[{"frame":0,"value":1,"ease":{"type":"step"}},
                                                  {"frame":12,"value":0,"ease":{"type":"step"}}], "mode":"multiply"}
```

Regole imposte dalla validazione: un osso appartiene al massimo a una skin; ogni discendente di un osso di skin appartiene alla stessa skin; le ossa di skin
non devono avere `scale_limits`. Usa `skin rig <group> <name> <constraint,...|-> <ik,...|->` per vincoli e catene IK che
devono essere attivi solo con una skin.

### 5.4 Pezzi rigidi più un livello switch

Per le cose che cambiano forma in modo discreto (bocche, mani, occhi), metti le alternative in un livello `switch`
e inserisci chiavi su `layer_switch`. Vedi la sezione 11.

## 6. Deformatori

I deformatori spostano i punti di un livello. Sono elencati nei `deformers` del livello, vengono animati tramite
canali con un target in tre parti `"layer::deformer::point"` e agiscono sulle **ancore** di un tracciato o sui vertici di una mesh
di immagine. **Se una forma ha troppo poche ancore si deformerà appena: suddividila prima.**

### Cage (coordinate a valor medio)

```
$ osatura cage lab0.json c1.json strip auto verts=3 pad=10
added cage 'cage' (12 vertices) to 'strip'; wrote c1.json
$ osatura cage-pose c1.json c2.json strip 12 move=1:0,-40 ease=ease_in_out
keyed cage 'cage' on 'strip' at frame 12; wrote c2.json
```

`cage` avvolge un livello in un poligono (`auto` = limiti con margine, `hull` = involucro convesso, `verts=N` = segmenti per
lato); il contenuto all'interno segue i vertici del poligono, quello all'esterno resta intatto. `cage-pose` inserisce chiavi sugli scostamenti
dei vertici (canali `cage_dx`, `cage_dy` con target `strip::cage::1`). `timeline` li elenca sotto
«Deformers and paths». Un canale bloccato viene rifiutato.

### Lattice, spine, pin (JSON)

Questi tre non hanno comandi; aggiungili ai `deformers` di un livello e animali con i canali:

```
"deformers": [{"type":"lattice","name":"ffd","cols":4,"rows":1,"rect":[60,138,240,24]}]
"channels":  [{"target":"strip::ffd::2,0","prop":"lattice_dy","source":"keys",
               "track":{"keys":[{"frame":0,"value":0},{"frame":12,"value":-40}]}}]

"deformers": [{"type":"spine","name":"curve","rest":[[60,150],[180,150],[300,150]],"radius":40}]
"channels":  [{"target":"strip::curve::1","prop":"spine_dy", ...}]

"deformers": [{"type":"pin","name":"pins","pins":[[60,150],[180,150],[300,150]],"radius":90}]
"channels":  [{"target":"strip::pins::2","prop":"pin_dy", ...}]
```

* **Lattice**: una griglia a forma libera. L'id del punto è `col,row`. Ottimo per squash e stretch.
* **Spine**: una polilinea con un `radius` di influenza; spostando un punto della spine si piega la grafica vicina (code, tentacoli).
* **Pin**: warp con puppet-pin; ogni pin attira i punti entro `radius`.

Ognuno di questi è stato reso correttamente nel foglio di contatto qui sopra quando applicato a una striscia senza pesi di osso. Un tracciato
che ha *anche* lo skinning a ossa sarà deformato da entrambi; rimuovi i pesi dalla striscia di prova se vuoi vedere un
deformatore da solo.

### Deformare una selezione: `warp-pivot`, `soft-scale`, `layer-pivot`

```
osatura warp-pivot c1.json w1.json strip 12 deformer=cage verts=1-2 rotate=25 pivot=centroid radius=60 falloff=smooth ease=ease_in_out
osatura soft-scale c1.json w2.json strip 12 deformer=cage scale=1.5
osatura soft-scale c1.json w3.json strip 12 height=60 aspect=free
osatura layer-pivot lab0.json w4.json strip pivot=60,150
```

`warp-pivot` ruota, scala o sposta una selezione di punti di controllo (cage, lattice, cloth, pin, spine o una
mesh di immagine a griglia) attorno a un pivot (`x,y`, `bone:NAME`, `centroid`, `center`) e inserisce chiavi sul risultato. `radius` aggiunge un
decadimento di soft-selection (`linear`, `smooth`, `gauss`, `sharp`, `constant`); l'esempio qui sopra ha selezionato i vertici 1-2 e
il decadimento ha aggiunto il 7 e l'8, quindi sono stati creati 8 canali con chiavi. `soft-scale` esegue un ridimensionamento proporzionale (`aspect=free` lascia che i
due assi differiscano). `layer-pivot` sposta il pivot di un livello senza spostarne la grafica.

### Mesh lock e glue

* `mesh-lock add <in> <out> <name> host=LAYER follower=LAYER [verts=...] [strength=1]`: i vertici del follower
  seguono la superficie dell'host per spostamento baricentrico. Usalo per tenere un tatuaggio, una toppa o un bottone su un
  cloth che si deforma. I cicli vengono rifiutati. `mesh-lock list` ha stampato `lock: patch:0 follows host:0 vertices all strength 1`.
* `glue add <in> <out> <name> a=LAYER b=LAYER auto=RADIUS|pairs=i:j;i:j [mode=rigid|snap|offset]`: coppie di vertici
  di due livelli vengono saldate in modo che `b` segua `a`. `glue list` ha stampato
  `weld: Rigid strength 1 host:0#1->edge:0#0 host:0#2->edge:0#3`.

### Cloth

Il cloth richiede una mesh **a griglia**. Una mesh libera tracciata viene rifiutata:
`cloth needs a grid mesh; the layer has a free-form (traced) mesh`. Chiedi una griglia con `cols=` e `rows=`:

```
$ osatura cloth flag.json flag_c.json flag pin=top gravity=900 wind=140,0 gust=0.5 cols=8 rows=10 from=0 to=35
baked cloth on 'flag' (180 channels); wrote flag_c.json
```

Quel bake ha richiesto 0,43 s nella build di debug e ha prodotto 180 canali con chiavi (6480 chiavi), quindi il risultato è completamente
modificabile, si esporta come qualsiasi altra animazione ed è deterministico. `pin=` accetta `top`, `left`, `corners`, `none`,
`row:N`, `col:N`, `v:N` oppure `...@bone` per fissare a un osso. `cols`/`rows` impostano la griglia di simulazione; l'immagine renderizzata viene deformata da essa
e in un render PNG a zoom elevato potresti notare lievi giunzioni tra i triangoli.

## 7. Fisica e molle

### Fisica con bake: `physics`

`physics` simula una catena di ossa (gravità, vento, collisioni) e converte il risultato in normali chiavi di angolo dell'osso.

```
osatura physics lab0.json ph1.json b2,b3 preset=tail gravity=600 --allow-empty
osatura physics lab0.json ph2.json b2,b3 preset=hair circle=200,230,30
osatura physics lab0.json ph3.json b2,b3 gravity=600 capsule=150,200,250,220,15 wind=40,0 gust=0.5
osatura physics lab.json  ph4.json b2,b3 gravity=300 drive=b1:0.5 vortex=b1:0.8:0.2:120
```

* `preset=` è uno tra `hair`, `tail`, `rope`, `antenna`, `cape`.
* `circle=x,y,r[@bone];...` e `capsule=x1,y1,x2,y2,r[@bone];...` sono collider; `@bone` fa seguire a un collider un osso.
* `drive=bone:gain` fa sì che il movimento di un altro osso spinga la catena; `vortex=bone|x,y:swirl[:radial[:radius]]`
  aggiunge un campo di forze a vortice; `follow=bone` e `lateral=` determinano come la catena segue un genitore.
* `merge=keep` (predefinito) **non sovrascrive** un osso che ha già un'animazione:
  `kept existing animation on 'b2' (use merge=replace to overwrite)`. Indica `merge=replace` quando è davvero ciò che vuoi.
* Le catene in stile `b2,b3/b1` e un osso sconosciuto (`no bone 'b9'`) vengono validati.

### Molle in tempo reale: `springs`

Una `Spring` aggiunge a un osso un follow-through smorzato al momento della valutazione, senza alcun bake. È solo JSON:

```
"springs": [{"bone":"b3","frequency":4.0,"damping":0.3}]
```

`frequency` è in Hz (più alta è più rigida), `damping` è un rapporto (sotto 1 oscilla, 1 è smorzamento critico) e `warmup`
(attivo per impostazione predefinita) esegue un passaggio di pre-calcolo così i loop partono già a regime. `explain` dice:
`spring: world rotation lags behind via a 4.0 Hz, damping 0.30 oscillator (pre-rolled for loops)`.

Per ottenere la sensazione che hai disegnato, esegui il fit:

```
$ osatura spring-fit --start 0 --samples 0.3,0.7,1.1,1.2,1.05,0.95,1.0,1.02 --fps 24
frequency 3.36 Hz, damping 0.52 (rms error 0.055 of the step size)
add to the project: {"bone": "<bone>", "frequency": 3.36, "damping": 0.52}
```

**Insidia.** Le molle non fanno parte di Lottie. Esegui il bake (`physics`) se esporti verso un formato che non ha
un equivalente; `export --report` elenca ciò che è stato approssimato.

## 8. Auto-rig, tagging e `coach`

Dai alle tue ossa e ai tuoi livelli nomi con parole standard (`arm_upper_l`, `thigh_r`, `eye_l`, `mouth`) e OSATURA può
riconoscerne i ruoli:

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

`autorig <in> <out> [seed]` aggiunge poi un'automazione non distruttiva a partire da quei ruoli:

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

Il progetto è passato da 11 ossa e 0 catene IK a 15 ossa e 4 catene IK con 3 nuovi canali. Tutto ciò che
viene aggiunto è dato ordinario: elimina ciò che non vuoi. Le parti mancanti vengono saltate con un motivo
(`- arm_l: upper/lower bone not found`, `- breathing: no torso bone`, `- blink: no eye layers`), quindi leggi il rapporto.

**Insidia.** `autorig` è un comando da progetto a progetto e l'output può coincidere con l'input. Su un file a cui tieni,
scrivi su un nuovo nome: `osatura autorig buddy.json buddy.json` sostituisce `buddy.json` (la scrittura è atomica, ma
non c'è annulla).

`coach <project>` legge l'animazione e stampa suggerimenti su timing e principi; `lint` controlla il rig
in modo statico. Entrambi sono consultivi.

## 9. Mirroring e simmetria

`mirror <in> <out>` specchia tutti i canali con chiavi rispetto all'asse verticale: i target scambiano lato (prefissi `L_`/`R_`,
suffissi `_L`/`_R` e `_l`/`_r`, le parole left/right) e le rotazioni e le traslazioni orizzontali
cambiano segno. I canali specchiati **sostituiscono** i canali del progetto. Sul bipede in camminata le chiavi della coscia sinistra
`0.0, 10.7, 19.8, 25.9` sono diventate quelle della coscia destra `-0.0, -10.7, -19.8, -25.9` e viceversa, mentre `hips` (senza lato)
ha mantenuto i suoi valori. `diff` elenca i sei canali modificati:

```
$ osatura mirror walk.json mir.json
wrote mir.json
$ osatura diff walk.json mir.json
~ channel 'arm_upper_l:BoneAngle' changed (track)
~ channel 'arm_upper_r:BoneAngle' changed (track)
...
6 difference(s)
```

**Insidia.** Su un progetto senza nomi sinistra/destra `mirror` scrive in silenzio una copia invariata e dice
`wrote`. Esegui sempre `diff` sul risultato. Dai ai lati nomi coerenti (`_l`/`_r`) prima di animare.

## 10. View swap e cambio di genitore

### Turnaround: `view-swap`

Un turnaround (frontale, tre quarti, laterale) è un livello switch con un figlio per vista, pilotato dalla proprietà di un osso
controller:

```
cat > ctl.rhai <<'EOF'
add_bone("view_ctrl", "", 10, 20, 20, 0);
EOF
osatura script biped.json v0.json ctl.rhai
osatura view-swap v0.json v1.json mouth view_ctrl.bone_offset_x "-40:0,0:2,40:4"
```

Canale prodotto: un canale `driven` `layer_switch` ← `view_ctrl.bone_offset_x` con una curva a gradino (figlio 0 sotto 0,
figlio 2 da 0, figlio 4 da 40). Anima il `bone_offset_x` di `view_ctrl` per cambiare vista. Gli indici dei figli devono esistere e il livello
deve essere uno switch (`child index 5 is out of range (the layer has 3)`).

`slider <in> <out> <name> "at:target.prop=val,...;at:..."` crea un **pose slider**, un osso controller `ctrl_<name>` il cui
`offset_x` fa una miscela tra pose memorizzate:

```
$ osatura slider biped.json sl.json look "0:head.bone_angle=0;24:head.bone_angle=15"
controller bone: ctrl_look (animate its offset_x)
```

### Cambio di genitore nel tempo: `parent-switch`

Per passare un oggetto da una mano all'altra (o per raccogliere un oggetto di scena) senza scatti:

```
$ osatura parent-switch biped.json ps.json arm_upper_r 0:torso,12:head
wrote ps.json
```

Esegue il bake dei canali di offset e angolo dell'osso (qui 3 canali, con una chiave su ogni fotogramma) in modo che l'osso mantenga la sua posizione
mondo al momento del cambio. Non funziona per ossa con IK, vincoli o molle.

## 11. Rig del volto

### Forme della bocca

Disegna le forme della bocca come figli di un **livello switch** in questo ordine, che tutti i comandi di lip-sync danno per scontato:
**rest, small, open, round, wide**. La bocca del bipede è `add_switch("mouth", ["m_rest","m_small","m_open","m_round","m_wide"])`.
Se il tuo switch ha meno figli, `--map 0,1,2,3,4` (cinque numeri, uno per forma) sceglie quale figlio usa ciascuna forma; il comportamento predefinito
segue il numero di figli (1 figlio: tutte; 2: chiusa+aperta; 3: chiusa+aperta+tonda/larga; 4: chiusa+piccola+aperta+tonda/larga; 5+: in quell'ordine).

Attacca lo switch all'osso della testa (`set_layer("mouth", "bone", "head")`) in modo che ruoti con la testa. Animarlo è
l'argomento della [sezione sul lip-sync della guida all'animazione](ANIMATION_GUIDE.md#8-lip-sync-e-parlato).

### Occhi e clipping

`eye-clip <in> <out>` maschera i livelli di iride e pupilla con la forma dell'occhio dello stesso lato, così una pupilla non può uscire
dall'occhio. `autorig` lo esegue. Su un rig senza livelli di iride/pupilla scrive il file invariato. Gli ammiccamenti sono un
`layer_scale_y` con chiavi o procedurale sul livello dell'occhio:

```
$ osatura idle blink p1.json i1.json --target eye_l --prop layer_scale_y --interval 2 --seed 4 --closed 0.1
wrote i1.json
```

`idle blink|breath|sway` generano canali procedurali con seed (`--interval`, `--amp`, `--period`, `--freq`,
`--seed`, `--closed`): lo stesso seed dà gli stessi istanti di ammiccamento su ogni macchina.

### Pilotare un volto da una performance

`face-synth`, `face-track`, `face-bake` e il pannello Face dello studio trasformano un flusso di landmark in canali. Sono
trattati, con cifre di accuratezza oneste, nella [sezione sul face tracking della guida all'animazione](ANIMATION_GUIDE.md#10-face-tracking-e-suoi-limiti).

## 12. Camera, inquadrature e parallasse

* `shot <in> <out> <name> <x> <y> <zoom> [rotation]` salva un'inquadratura di camera con nome (centro della camera e zoom);
  `camera-seq <in> <out> <frame:shot[:cut],...>` inserisce chiavi sulla camera attraverso le inquadrature salvate; `timeline` mostra i
  canali di camera risultanti.
* `parallax <in> <out> <far_layer,...,near_layer> <far_value> <near_value>` distribuisce un fattore `parallax` sui
  livelli di primo livello (quelli lontani si muovono meno quando la camera fa una panoramica).
* `dof <in> <frame> <out.png> [--focus 1.0 --strength 12 --max-radius 24]` sfoca ogni livello di primo livello di
  `|parallax - focus|`; `dof-export` renderizza un intervallo in una GIF, una APNG o una cartella. Fuoco e intensità seguono
  i canali `camera_focus` / `camera_blur` a meno che tu non li fissi.

Verificato sul bipede in camminata:

```
osatura shot walk.json sh1.json wide 200 180 1          # name x y zoom
osatura shot sh1.json sh2.json close 200 100 2.2
osatura camera-seq sh2.json sh3.json 0:wide,12:close,23:wide
osatura parallax sh3.json px.json sky,ground 0.2 1.0
osatura dof px.json 12 dof.png --focus 1.0 --strength 10
osatura dof-export px.json dof.gif --from 0 --to 11 --scale 0.5     # 80 KB
```

Gli effetti di livello (blur, drop shadow, tint, fill, outline, posterize...) possono essere per singolo livello o applicati come catena: vedi la
[guida all'animazione](ANIMATION_GUIDE.md#15-effetti-di-livello).

## 13. Una checklist per il rigging

1. Esegui `check` dopo ogni modifica strutturale; esegui `lint` prima di animare.
2. Mantieni la posa di riposo semplice e simmetrica; anima i delta rispetto ad essa.
3. Inserisci nel rig i controller (goal IK, `view_ctrl`, `root_ctrl`) come ossa senza grafica.
4. Preferisci pezzi rigidi per la grafica da cartone animato, applica lo skinning solo a ciò che deve piegarsi e suddividi prima di deformare.
5. Usa `explain` quando qualcosa non va; è più rapido che tirare a indovinare.
6. Esegui il bake di fisica e cloth se l'animazione deve sopravvivere in un formato privo di quelle funzioni, e leggi `export --report`.
7. Scrivi su nuovi nomi di file finché non ti fidi di un comando; i comandi da progetto a progetto possono sovrascrivere il proprio input.

## 14. Limiti da conoscere

* Lo skinning della grafica vettoriale è **linear blend** (non dual quaternion), con pesi che fornisci tu; non esiste alcuno skinner
  vettoriale automatico nella CLI.
* Lattice, spine, pin, `soft`/`stretch` dell'IK e molle sono solo JSON.
* Fisica e cloth sono bake: se cambi il rig, rifai il bake. `physics` non sovrascrive l'animazione esistente se non glielo dici.
* Il cloth richiede una mesh a griglia; le mesh libere tracciate servono per lo skinning e i mesh lock.
* Lottie e GIF non possono rappresentare tutto (tratti a larghezza variabile, gradienti con shear, molle...). Gli avvisi vengono
  stampati durante l'esportazione; vedi [EXPORT_FORMATS.md](EXPORT_FORMATS.md) e [LIMITATIONS.md](LIMITATIONS.md).
* Il rilevatore di volti è addestrato soltanto su volti sintetici (sezione 11 e guida all'animazione).
