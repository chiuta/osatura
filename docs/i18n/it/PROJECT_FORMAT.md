<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<!-- Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio) -->

# Formato di progetto di OSATURA

Questo è il riferimento per il file di progetto di OSATURA: un unico documento JSON che
contiene la scena, il rig, la grafica e l'animazione. È scritto a partire dalle
struct serde in `osatura-core/src/project.rs` e dai moduli a cui rimandano,
e ogni esempio delle sezioni da 13 a 15 è stato verificato con il binario `osatura`
(`osatura check`, `osatura render`, `osatura explain`).

Indice

1. [Convenzioni e tipi primitivi](#1-convenzioni-e-tipi-primitivi)
2. [Caricamento, versioni e campi sconosciuti](#2-caricamento-versioni-e-campi-sconosciuti)
3. [Campi di primo livello](#3-campi-di-primo-livello)
4. [Rig: bones, ik, constraints, springs](#4-rig-bones-ik-constraints-springs)
5. [Livelli](#5-livelli)
6. [Tracciati, ancore, paint, stroke](#6-tracciati-ancore-paint-stroke)
7. [Deformatori, effetti, normal map](#7-deformatori-effetti-normal-map)
8. [Canali, tracce, easing](#8-canali-tracce-easing)
9. [Espressioni](#9-espressioni)
10. [Camera, inquadrature, audio, metadati](#10-camera-shot-audio-metadati)
11. [Clip, skin, link, interazione, lod](#11-clip-skin-collegamenti-interazione-lod)
12. [Regole di validazione, limiti e messaggi di errore](#12-regole-di-validazione-limiti-e-messaggi-di-errore)
13. [Esempio 1: palla che rimbalza](#13-esempio-1-palla-che-rimbalza)
14. [Esempio 2: braccio con rig e IK](#14-esempio-2-braccio-con-rig-e-ik)
15. [Esempio 3: bocca con livello switch](#15-esempio-3-bocca-con-livello-switch)
16. [Ordine di valutazione](#16-ordine-di-valutazione)
17. [Ordine di rendering](#17-ordine-di-rendering)
18. [Asset e percorsi](#18-asset-e-percorsi)
19. [Politica di versionamento](#19-politica-di-versionamento)
20. [Divergenze note tra codice e commenti](#20-lacune-note-tra-codice-e-commenti)

## 1. Convenzioni e tipi primitivi

| Tipo | Forma JSON | Note |
| --- | --- | --- |
| number | numero JSON | Memorizzato come `f32`, salvo che il campo indichi un intero. Ogni valore in virgola mobile deve soddisfare `abs(x) <= 1e7` (sezione 12). |
| `Vec2` | `[x, y]` | Due numeri. Le unità della scena sono pixel, +x verso destra, +y verso il basso. |
| `Rgba` | stringa | `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, oppure uno tra `black`, `white`, `red`, `green` (0,128,0), `blue`, `yellow`, `transparent`. L'output è sempre in esadecimale minuscolo. |
| `Paint` | stringa od oggetto | Una stringa di colore è un paint a tinta unita. `{"from": [x,y], "to": [x,y], "stops": [...]}` è un gradiente lineare. `{"center": [x,y], "radius": r, "stops": [...]}` è un gradiente radiale. Il tipo è un enum senza tag. |
| `Stop` | `{"offset": 0..1, "color": Rgba}` | Gli stop vengono ordinati (in modo stabile) al momento del disegno e gli offset sono limitati a 0..1 nel rasterizzatore. Un gradiente degenere dipinge l'ultimo stop. |
| angle | numero | Gradi ovunque nel file (angoli delle ossa, rotazione dei livelli, skew, rotazione della camera, `angle_limits`). |
| time | numero | Fotogrammi nelle chiavi e nei marker, secondi nei suoni, nelle macchine e negli emettitori. Ogni campo specifica quale. |
| names | stringa | Nomi di ossa, livelli, tracciati, deformatori, ik, vincoli. Le sezioni 5 e 12 elencano le restrizioni sui caratteri. |

Un colore errato in un contesto paint produce il messaggio serde
`data did not match any variant of untagged enum Paint`, perché `Paint` è privo di tag e serde
non può dire quale ramo si intendesse.

Le tabelle dei campi qui sotto usano questa notazione: **Obbl.** significa che il campo non ha un valore predefinito e deve essere presente;
altrimenti viene indicato il valore predefinito. «Intervallo» è ciò che il loader o il motore impone. Un intervallo indicato come
«limitato» non è un errore: il motore limita silenziosamente il valore.

## 2. Caricamento, versioni e campi sconosciuti

`Project::from_json(text)` è l'unico punto di ingresso usato dalla CLI, dalla API C, dal modulo WASM
e dallo Studio. Esegue, in ordine:

1. Analizza il JSON con serde. Serde ignora i campi che non conosce.
2. Esegue il controllo delle grandezze: qualsiasi letterale in virgola mobile il cui valore assoluto sia superiore a `1e7` fa fallire il caricamento
   con `invalid JSON in <file>: value ... is outside the supported range of +-1e7 ...`. I letterali interi vengono intercettati più tardi, dal
   nuovo controllo in `Engine::new` (vedi sezione 12).
3. Applica il controllo del formato: un `format` maggiore del `FORMAT_VERSION` della build (attualmente `1`) viene
   rifiutato con `project format 2 is newer than this build supports (1); update OSATURA`.

`Engine::new` (o `Engine::with_asset_dir`) convalida poi l'intero progetto e restituisce tutti i
problemi che trova sotto forma di elenco. La CLI li stampa così:

```text
<file> is not a valid project:
  - first problem
  - second problem
```

I testi degli errori sono pensati per le persone. Non costituiscono un contratto stabile; confrontali con il testo solo nei test.

### Campi sconosciuti

Serde scarta le chiavi sconosciute al caricamento, quindi un refuso come `fil` al posto di `fill` andrebbe perso in silenzio. Il
modulo `unknownfields` confronta il JSON di input con il progetto riserializzato e segnala le chiavi che
non sono sopravvissute:

```text
warning: unknown field layers[0].paths[0].fil (ignored)
```

`osatura check` stampa questi avvisi. I valori uguali a quelli predefiniti non vengono segnalati (il writer
omette i valori predefiniti, quindi un `"visible": true` esplicito va bene). Alcune strutture sono invece rigorose anziché
tolleranti e rifiutano le chiavi sconosciute con un errore bloccante: `Transition` e `Condition` delle macchine, la configurazione CI, la configurazione
dell'input dal vivo, gli overlay e la configurazione dei parametri del volto; i file di configurazione vengono letti con `parse_strict`.

### Round trip dei metadati dell'editor

Il progetto viene serializzato con `skip_serializing_if` per le liste facoltative. Un file scritto a mano può
essere più corto di quello che `osatura` riscrive; i due sono equivalenti.

## 3. Campi di primo livello

| Nome JSON | Tipo | Predefinito | Intervallo / regola | Significato |
| --- | --- | --- | --- | --- |
| `format` | intero | `FORMAT_VERSION` (1) | deve essere `<=` alla versione della build | Versione del formato, sezione 19. |
| `name` | stringa | **Obbl.** | qualsiasi | Nome visualizzato. Stampato da `check`. |
| `width` | numero | **Obbl.** | `> 0`, `<= 32768` | Larghezza della scena in pixel. |
| `height` | numero | **Obbl.** | `> 0`, `<= 32768` | Altezza della scena in pixel. |
| `fps` | numero | **Obbl.** | `(0, 1000]` | Fotogrammi al secondo. |
| `frames` | intero (`u32`) | **Obbl.** | `1..=200000` | Lunghezza della timeline. Gli indici dei fotogrammi sono `0..frames-1`. Un letterale in virgola mobile come `48.0` viene rifiutato: `invalid type: floating point `48.0`, expected u32`. |
| `background` | `Rgba` o null | null (trasparente) | | Colore della scena dietro a tutto. |
| `color_space` | stringa | `"srgb"` (non scritto) | `srgb`, `linear` | Spazio in cui i compositor raster e float fanno la miscelazione. `linear`: i colori vengono convertiti in luce lineare, composti in f32 e riconvertiti una sola volta (un nero al 50 % su bianco dà 188/255, non 128/255). Si imposta con `osatura color-space`. SVG, Lottie, il renderer GPU, `dof` e `fx` lo ignorano. |
| `camera` | oggetto | valori predefiniti | sezione 10 | Camera globale. |
| `bones` | array | `[]` | al massimo 4096 | Scheletro, sezione 4. |
| `ik` | array | `[]` | | Catene IK. |
| `constraints` | array | `[]` | | Aim, copy-rotation, pin. |
| `springs` | array | `[]` | | Movimento secondario, calcolato con bake al caricamento. |
| `layers` | array | `[]` | | Livelli radice, disegnati nell'ordine della lista. |
| `channels` | array | `[]` | | Canali di animazione, valutati nell'ordine della lista. |
| `markers`, `comps`, `events`, `bone_vis`, `channel_meta`, `shots` | array | omessi se vuoti | non convalidati | Metadati di timeline e di editor, sezione 10. |
| `sounds` | array | omesso se vuoto | convalidato | Tracce audio, sezione 10. |
| `skins`, `clips`, `clip_baked`, `lods` | array | omessi se vuoti | non convalidati al caricamento | Sezione 11. |
| `mesh_locks`, `glues` | array | omessi se vuoti | convalidati | Collegamenti di mesh tra livelli, sezione 11. |
| `machines`, `bindings`, `containers`, `scrolls` | array | omessi se vuoti | convalidati | Interazione e layout, sezione 11. |

Progetto valido minimo:

```json
{"name": "empty", "width": 100, "height": 100, "fps": 24, "frames": 1}
```

Grandezze: ogni valore in virgola mobile nel file deve soddisfare `abs(x) <= 1e7`. Vale per coordinate,
scale, offset e valori delle chiavi allo stesso modo. Larghezza e altezza sono inoltre limitate a 32768, e l'output
raster a 16384 per lato e 100 megapixel (sezione 18).

## 4. Rig: bones, ik, constraints, springs

### 4.1 `bones[]`

Le ossa devono essere elencate partendo dai genitori. Il genitore di un osso deve comparire prima nella lista.

| Nome JSON | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `name` | stringa | **Obbl.** | univoco | Referenziato da pesi, ik, canali, `Layer.bone`. |
| `parent` | stringa o null | null | un osso precedente | Osso genitore. Null indica una radice. |
| `length` | numero | **Obbl.** | finito, `>= 0` | Lunghezza dell'osso in pixel, lungo l'asse +x dell'osso. |
| `rest_offset` | `Vec2` | `[0,0]` | | Posizione della testa nello spazio del genitore (spazio della scena per le radici). |
| `rest_angle` | numero | 0 | | Rotazione di riposo in gradi relativa al genitore. |
| `angle_limits` | `[min, max]` o null | null | `min <= max` | Limite sul delta dell'angolo animato, in gradi. |
| `scale_limits` | `[min, max]` o null | null | `0 < min <= max` | Limite sulla scala animata. |

Interazioni: i canali `bone_angle` si sommano a `rest_angle`; `bone_offset_x/y` si sommano a `rest_offset`;
`bone_scale` moltiplica. `clamp_limits` viene eseguito dopo tutti i canali e prima di FK, e di nuovo dopo ogni
catena IK.

```json
{"name": "upper", "parent": null, "length": 90, "rest_offset": [60, 120],
 "rest_angle": 0, "angle_limits": [-120, 120]}
```

### 4.2 `ik[]`

| Nome JSON | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `name` | stringa | **Obbl.** | univoco | Target dei canali `ik_mix`. |
| `bones` | array di stringhe | **Obbl.** | almeno 2; ciascuno figlio diretto del precedente | Catena dalla radice alla punta. |
| `target` | stringa | **Obbl.** | un osso, non nella catena | Osso la cui testa la punta cerca di raggiungere. |
| `bend` | numero | 1 | +1 o -1 | In che direzione si piega una catena di due ossa. |
| `mix` | numero | 1 | limitato a 0..1 | Miscela iniziale da FK a IK; si anima con `ik_mix`. `<= 0` salta la catena. |
| `iterations` | intero | 16 | | Passate di FABRIK per catene più lunghe di due ossa. È un minimo: il solver esce in anticipo quando la punta è entro 0.01 px, altrimenti prosegue fino a 256 passate. |
| `soft` | numero | 0 | `0..0.9` | Frazione della portata su cui l'estensione si attenua. Con `stretch > 1` la zona è l'ultima frazione della portata allungata (`reach x stretch`). |
| `stretch` | numero | 1 | `1..4` | Fattore massimo di allungamento; sopra 1 la scala dell'osso radice cresce per raggiungere il bersaglio. |

Una catena di due ossa usa il solver analitico basato sul teorema del coseno; le catene più lunghe usano FABRIK. Dopo la risoluzione,
gli angoli vengono miscelati nella posa come `old + wrap_pi(new - old) * mix`.

```json
{"name": "reach", "bones": ["upper", "lower"], "target": "target", "bend": 1, "mix": 1}
```

### 4.3 `constraints[]`

I tipi di vincolo usano un tag interno `"type"`, appiattito accanto a `name`, `bone` e `weight`.

| Nome JSON | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `name` | stringa | **Obbl.** | Target dei canali `constraint_weight`. |
| `bone` | stringa | **Obbl.** | L'osso vincolato. |
| `type` | `aim`, `copy_rotation`, `pin` | **Obbl.** | Tipo. |
| `target` | stringa | **Obbl.** per `aim`, `pin` | Osso verso cui mirare o su cui scivolare. |
| `offset` | numero | 0 | Solo `aim`: gradi aggiuntivi. |
| `source` | stringa | **Obbl.** per `copy_rotation` | Osso la cui rotazione mondo viene copiata. |
| `weight` | numero | 1 | Influenza, limitata a 0..1. `<= 0` salta il vincolo. |

Un vincolo non può avere se stesso come target. I vincoli vengono eseguiti nell'ordine della lista dopo tutte le catene IK.

```json
{"name": "aim", "bone": "root", "type": "aim", "target": "look", "offset": 0, "weight": 0.5}
```

### 4.4 `springs[]`

Una molla smorzata che fa ritardare la rotazione mondo di un osso rispetto alla sua rotazione animata. Viene
simulata una volta al caricamento e calcolata con bake, quindi qualsiasi fotogramma può essere valutato in qualsiasi ordine.

| Nome JSON | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `bone` | stringa | **Obbl.** | una molla per osso | Osso da pilotare. |
| `frequency` | numero | 3 | `(0, 30]` Hz | Rigidità. |
| `damping` | numero | 0.35 | `>= 0` | Rapporto di smorzamento; sotto 1 oscilla, 1 è critico. |
| `warmup` | bool | true | | Pre-calcola un passaggio in modo che i loop partano già a regime. |

Gli angoli calcolati con bake sostituiscono l'angolo dell'osso dopo i vincoli (passo 8 della valutazione).

## 5. Livelli

Un livello è `{"name": ..., "type": ..., <campi del tipo>, <campi comuni>}`. `type` è il tag di
`LayerKind`; i suoi campi sono appiattiti nell'oggetto del livello.

### 5.1 Campi comuni

| Nome JSON | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `name` | stringa | **Obbl.** | non vuoto, senza `/`, senza `::`; il percorso completo deve essere univoco | Nome del livello. I figli sono indirizzati come `parent/child`. |
| `type` | stringa | **Obbl.** | `vector`, `group`, `switch`, `liquid`, `boolean`, `emitter`, `image` | Tipo. |
| `transform` | oggetto | identità | | `position` `[0,0]`, `rotation` 0, `scale` `[1,1]`, `skew` 0, `anchor` `[0,0]`. |
| `opacity` | numero | 1 | limitato a 0..1 | Si moltiplica nella composizione del livello. |
| `visible` | bool | true | | I livelli nascosti vengono saltati durante il disegno ma possono ancora essere sorgenti di maschera. |
| `blend` | stringa | `normal` | vedi sotto | Modalità di fusione. |
| `mask` | `{layer, invert}` | null | nessuna auto-maschera, nessun ciclo | Usa l'alpha di un altro livello come maschera. |
| `bone` | stringa | null | un osso noto | Attacca a un osso: la matrice di skin dell'osso viene applicata tra la matrice di scena del genitore e la trasformazione locale. |
| `parallax` | numero | 1 | | Fattore di profondità della camera. Conta solo il valore del livello di primo livello. |
| `deformers` | array | omesso se vuoto | solo livelli vector e image | Sezione 7. |
| `wobble` | oggetto | null | | `amp`, `scale` (0.03), `rate` (8), `seed` (0): spostamento deterministico con noise. Anima `amp` con `layer_wobble`. |
| `reference` | bool | false | | Livello di riferimento: nascosto a meno che l'editor non mostri i riferimenti. |
| `locked` | bool | false | | Blocco dell'editor, vedi sotto. |
| `normal_map` | oggetto | null | | Sezione 7. |
| `effects` | array | omesso se vuoto | al massimo 16 | Sezione 7. |

Matrice locale: `T(position) * R(rotation) * Shear(skew) * S(scale) * T(-anchor)`.

Modalità di fusione: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color_dodge`,
`color_burn`, `hard_light`, `soft_light`, `difference`, `exclusion`, `add` (plus-lighter di CSS).

`locked` blocca solo le operazioni dell'editor: selezione sul canvas, gizmo, spostamenti minimi, eliminazioni, modifiche alla timeline e
ai canali (tramite `assert_unlocked`), per il livello e i suoi discendenti. Il rendering e
la valutazione lo ignorano, e le semplici `Op` (sezione «Editor ops» in ARCHITECTURE.md) si applicano comunque.

I campi comuni vengono scritti dopo i campi del tipo nell'output, ma l'ordine è irrilevante in input.

### 5.2 `vector`

| Campo | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `paths` | array di tracciati | `[]` | Sezione 6. |

### 5.3 `group`

| Campo | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `layers` | array di livelli | `[]` | I figli ereditano la trasformazione del gruppo. |

### 5.4 `switch`

| Campo | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `layers` | array di livelli | `[]` | Disegni candidati. |
| `selected` | intero | 0 | Indice iniziale del figlio. Si anima con `layer_switch` (chiavi a gradino). |

Viene mostrato esattamente un figlio. Il valore viene arrotondato; un valore negativo diventa 0; i valori pari o superiori al numero
di figli diventano l'ultimo figlio. La validazione non rifiuta le chiavi fuori intervallo. Vedi la sezione 20 per una
discrepanza di formulazione del lint a questo proposito.

### 5.5 `liquid`

Liquido a metaball. Il campo è la somma di `r^2 / d^2` sulle blob; il contorno è l'isocurva,
estratta con marching squares su una griglia limitata a 600 per 600 celle e levigata con Catmull-Rom.

| Campo | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `blobs` | array di `{pos, radius, bone?}` | **Obbl.** | almeno 1; `radius > 0` | Una blob. `bone` la attacca a un osso. |
| `threshold` | numero | 1 | `> 0` | Livello dell'isocurva; con 1 una blob isolata diventa un cerchio del suo raggio. |
| `cell` | numero | 3 | `>= 0.25` | Cella della griglia in unità del livello. |
| `fill` | `Paint` | null | | |
| `stroke` | oggetto stroke | null | | |

### 5.6 `boolean`

| Campo | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `op` | `union`, `intersect`, `difference`, `xor` | **Obbl.** | Operazione. |
| `paths` | array di tracciati | **Obbl.**, almeno 2 | `paths[0]` è il soggetto; il ritaglio è l'unione degli altri. |

Il risultato prende il fill e lo stroke del soggetto; la tolleranza di appiattimento è 0.25. I tracciati risultanti sono
chiamati `<first path name><index>`.

### 5.7 `emitter`

`{"type": "emitter", "spec": {...}}`. Le particelle sono una funzione in forma chiusa del tempo, quindi ogni fotogramma
è valutabile. I tempi sono in secondi.

| Campo di `spec` | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- |
| `pos` | `[0,0]` | | Posizione dell'emettitore (spazio del livello). |
| `bone` | null | | Attacca a un osso. |
| `rate` | 20 | `(0, 2000]` | Particelle al secondo. |
| `lifetime` | 1 | `> 0` | Secondi. |
| `lifetime_jitter` | 0 | 0..1 | |
| `speed` | 60 | | Pixel al secondo. |
| `speed_jitter` | 0 | 0..1 | |
| `direction` | -90 | | Gradi; -90 è verso l'alto. |
| `spread` | 30 | | Gradi di ampiezza del cono. |
| `gravity` | `[0,0]` | | Accelerazione, pixel al secondo quadrato. |
| `size_start`, `size_end` | 3, 3 | | Raggio durante la vita. |
| `color_start`, `color_end` | `#fff`, `#fff0` | | Colore durante la vita. |
| `seed` | 0 | | Seed deterministico. |
| `start` | 0 | | Tempo di inizio in secondi. |
| `duration` | null | | L'emissione si ferma dopo questo numero di secondi; null significa per sempre. |
| `max_particles` | 2000 | | Tetto massimo di particelle vive. |

### 5.8 `image`

| Campo | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `source` | `{file?, png_base64?}` | **Obbl.** | uno dei due | `file` è un percorso relativo alla cartella del progetto (sezione 18); `png_base64` è autonomo. Nessuno dei due: `image has neither 'png_base64' nor 'file'`. |
| `origin` | `Vec2` | `[0,0]` | | Angolo in alto a sinistra nello spazio del livello. |
| `size` | `Vec2` o null | null (dimensione in pixel) | | Dimensione visualizzata. |
| `mesh` | oggetto o null | null | | Mesh di deformazione. |

`mesh`: `cols`, `rows` (mesh a griglia, ciascuno almeno 1), `weights` (pesi di ossa per vertice; vuoto
significa statica) e, facoltativamente, `free` = `{vertices, triangles, uvs}`. Una mesh libera richiede almeno 3
vertici, almeno 1 triangolo, al massimo 200000 vertici e 400000 triangoli, coordinate finite,
indici dei triangoli validi, e `uvs` vuoto oppure della stessa lunghezza di `vertices`. Con deformatori ma
senza mesh, viene usata una griglia 16 per 16.

```json
{"name": "img2", "type": "image", "source": {"png_base64": "iVBOR..."},
 "origin": [100, 0], "size": [80, 80],
 "mesh": {"free": {"vertices": [[100,0],[180,0],[180,80],[100,80]],
                   "triangles": [[0,1,2],[0,2,3]],
                   "uvs": [[0,0],[8,0],[8,8],[0,8]]}, "weights": []}}
```

## 6. Tracciati, ancore, paint, stroke

### 6.1 Tracciato (`paths[]`)

| Nome JSON | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `name` | stringa | `""` | Un nome vuoto viene sostituito da `path{index}` per l'indirizzamento. Non deve contenere `::`. Univoco all'interno del livello. |
| `closed` | bool | false | |
| `anchors` | array | **Obbl.** | Ancore di Bézier. |
| `fill` | `Paint` | null | |
| `stroke` | stroke | null | |
| `morphs` | array di `{name, anchors}` | omesso se vuoto | Blend shape additive. Il numero di ancore deve essere uguale a quello del tracciato di base. Il peso si imposta con `path_morph`. |

### 6.2 Ancora

| Nome JSON | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `pos` | `Vec2` | **Obbl.** | Posizione. |
| `handle_in`, `handle_out` | `Vec2` | null | Punti di controllo nello stesso spazio di `pos` (assoluti, non relativi). Una maniglia mancante equivale a `pos` (un angolo). |
| `weights` | array di `{bone, weight}` | omesso se vuoto | Skinning. Ogni osso deve esistere e ogni peso è finito e `>= 0`. I pesi vengono normalizzati durante la valutazione; vuoto significa statico. Anche le maniglie hanno lo skinning. |

### 6.3 Stroke

| Nome JSON | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `paint` | `Paint` | **Obbl.** | |
| `width` | numero | 1 | |
| `cap` | `butt`, `round`, `square` | `round` | |
| `join` | `miter`, `round`, `bevel` | `round` | Un valore sconosciuto dà ``unknown variant `foo`, expected one of `miter`, `round`, `bevel` ``. |
| `dash` | array | omesso | Usato solo se la sua lunghezza è pari, almeno 2, e ogni voce è `> 0`. |
| `dash_offset` | numero | 0 | |
| `width_profile` | array di `[t, factor]` | omesso | Moltiplicatore di larghezza lineare a tratti lungo la lunghezza (`t` in 0..1), costante oltre le estremità, limite di mitra 2. Uno stroke a larghezza variabile ignora `dash` e `join`. |

Esempio di fill con gradiente radiale e stroke (dall'esempio della palla):

```json
"fill": {"center": [-8, -10], "radius": 44,
         "stops": [{"offset": 0, "color": "#ffd9a0"}, {"offset": 1, "color": "#d6336c"}]},
"stroke": {"paint": "#3d2c2e", "width": 2}
```

Il linter (`osatura lint`) segnala problemi sui gradienti (L011), sulle chiavi degli switch (L012) e sui dati dell'editor (L013); vedi
`osatura-core/src/lint.rs`.

## 7. Deformatori, effetti, normal map

### 7.1 `deformers[]`

Consentiti solo sui livelli `vector` e `image`. Ognuno ha un tag `"type"` e un `name` (univoco all'interno
del livello, senza `::`). Vengono eseguiti nell'ordine della lista sulla geometria nello spazio di riposo, prima dello skinning e prima di
`wobble`. Gli offset sono statici; animali con i canali dei sotto-target.

| `type` | Campi | Vincoli | Si anima con (target) |
| --- | --- | --- | --- |
| `lattice` | `cols`, `rows`, `rect` `[x,y,w,h]`, `offsets` | `cols, rows >= 1`; `w, h > 0`; `offsets` vuoto oppure `(cols+1)*(rows+1)` per righe | `lattice_dx/dy` (`layer::name::c,r`) |
| `spine` | `rest` (punti della curva), `offsets`, `radius` (bordo morbido facoltativo) | almeno 2 punti | `spine_dx/dy` (`layer::name::i`) |
| `pin` | `pins`, `offsets`, `radius` | da 1 a 64 pin, `radius > 0`, pin non coincidenti | `pin_dx/dy` (`layer::name::i`) |
| `cage` | `rest` (poligono), `offsets` | da 3 a 256 vertici, poligono semplice, area non nulla | `cage_dx/dy` (`layer::name::i`) |
| `cloth` | come `lattice` | come `lattice` | `lattice_dx/dy` (`layer::name::c,r`) |

Il lattice miscela gli offset con pesi di Bézier-Bernstein (morbido); il cloth sposta ogni vertice della griglia
in modo esatto (bilineare). Il pin usa una funzione di base radiale a supporto compatto di Wendland. Il cage usa le
coordinate a valor medio di Floater: il contenuto all'interno del poligono lo segue, quello all'esterno resta
intatto.

```json
"deformers": [
  {"type": "lattice", "name": "lat", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []},
  {"type": "spine", "name": "sp", "rest": [[0,0],[50,0],[100,0]], "offsets": [], "radius": 30},
  {"type": "pin", "name": "pn", "pins": [[0,0],[100,100]], "offsets": [], "radius": 80},
  {"type": "cage", "name": "cg", "rest": [[0,0],[100,0],[100,100],[0,100]], "offsets": []},
  {"type": "cloth", "name": "cl", "cols": 2, "rows": 2, "rect": [0, 0, 100, 100], "offsets": []}
]
```

### 7.2 `effects[]`

Al massimo 16 per livello. Ogni effetto ha `"type"`, `enabled` (predefinito true) e i campi seguenti. Il
target di canale `fxN` indicizza l'array includendo gli effetti disattivati.

| `type` | Campi | Intervallo |
| --- | --- | --- |
| `blur` | `radius` | 0..64 |
| `drop_shadow` | `dx`, `dy`, `blur`, `color`, `opacity` (1) | offset al massimo 1024; blur 0..64; opacity 0..1 |
| `tint` | `black`, `white`, `amount` (1) | amount 0..1 |
| `fill` | `color`, `opacity` (1) | opacity 0..1 |

Si anima con `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` sul target `layer::fxN`.

### 7.3 `normal_map`

`{"data": base64-PNG | "path": file, "fit": "stretch" | "tile" | "exact" (predefinito stretch),
"strength": 0..8 (predefinito 1), "green_down": bool (predefinito false)}`. Richiede `data` oppure `path`. Alimenta
il post-effetto `relight`; i render normali non lo usano. Come `Layer.normal_map` viene adattata al riquadro di delimitazione in pixel di quel livello. `fit` `exact` richiede una mappa esattamente della dimensione del fotogramma e in caso contrario è un errore.

## 8. Canali, tracce, easing

### 8.1 Canale

```json
{"target": "ball", "prop": "layer_y", "source": "keys",
 "track": {"keys": [{"frame": 0, "value": 40}, {"frame": 24, "value": 170}]},
 "mode": "replace", "weight": 1}
```

| Nome JSON | Tipo | Predefinito | Significato |
| --- | --- | --- | --- |
| `target` | stringa | `""` | Nome, oppure forma `layer::sub`. Ignorato per le proprietà della camera. |
| `prop` | stringa | **Obbl.** | Una delle proprietà in snake_case qui sotto. |
| `source` | `keys`, `expr`, `noise`, `driven` | **Obbl.** | Tag appiattito, con i campi di quella sorgente accanto. |
| `mode` | `replace`, `add`, `multiply` | `replace` | Come il valore si combina con quello corrente. |
| `weight` | numero | 1 | Influenza, deve essere finito; miscela verso il risultato. |

Combinazione: replace è `cur + (v - cur) * w`; add è `cur + v * w`; multiply è
`cur * (1 + (v - 1) * w)`. I valori non finiti vengono saltati. I canali vengono eseguiti nell'ordine della lista, quindi un canale
successivo vede i risultati dei precedenti (tramite `get(...)` e `driven`).

Sorgenti:

| `source` | Campi |
| --- | --- |
| `keys` | `track`: `{keys: [{frame, value, ease?}], extrapolate?}` |
| `expr` | `expr`: stringa (sezione 9) |
| `noise` | `amp`, `freq` (cicli al secondo), `seed` (0), `octaves` (1). Output in `[-amp, amp]`; basato su hash, deterministico. |
| `driven` | `input` (target), `input_prop`, `curve`: una traccia i cui «fotogrammi» sono valori di input (per esempio dall'angolo del gomito alla quantità di rigonfiamento). |

### 8.2 Proprietà

Ci sono 39 proprietà. È la proprietà a decidere cosa significhi `target`.

| Gruppo | Proprietà | `target` |
| --- | --- | --- |
| Osso | `bone_angle`, `bone_scale`, `bone_offset_x`, `bone_offset_y` | nome dell'osso |
| Livello | `layer_opacity`, `layer_x`, `layer_y`, `layer_rotation`, `layer_scale_x`, `layer_scale_y`, `layer_skew`, `layer_switch`, `layer_wobble` | percorso del livello (`parent/child`) |
| Camera | `camera_x`, `camera_y`, `camera_zoom`, `camera_rotation`, `camera_focus`, `camera_blur` | ignorato |
| Rig | `ik_mix`, `constraint_weight` | nome dell'ik o del vincolo |
| Tracciato | `path_morph`, `point_dx`, `point_dy`, `grad_dx`, `grad_dy` | `layer::path::morphName`, `layer::path::anchorIndex`, `layer::path::fill.from` / `fill.to` / `stroke.from` / `stroke.to` |
| Deformatore | `lattice_dx/dy`, `spine_dx/dy`, `pin_dx/dy`, `cage_dx/dy` | `layer::deformer::c,r` (lattice, cloth) oppure `layer::deformer::i` |
| Effetto | `fx_blur`, `fx_dx`, `fx_dy`, `fx_soft`, `fx_amount` | `layer::fxN` |

Semantica delle maniglie dei gradienti: per i gradienti lineari `.from` e `.to` spostano il punto iniziale e quello finale di
`(dx, dy)`. Per i gradienti radiali `.from` sposta il centro e `.to` cambia il raggio di `dx`.
Questi target esistono solo per i tracciati il cui fill o stroke è un gradiente. `layer_switch` richiede un livello
`switch`. Una proprietà sconosciuta elenca nell'errore tutte le varianti valide.

Limiti in scrittura: opacity 0..1, zoom della camera almeno 0.01, blur della camera almeno 0, `ik_mix` e
`constraint_weight` 0..1.

### 8.3 Traccia e chiave

`Track` = `{"keys": [...], "extrapolate": "hold"}`. Una chiave è `{"frame": number, "value": number,
"ease": {...}}`. Le chiavi devono essere finite e strettamente crescenti nel fotogramma:
`channel N ('target'): keys must be finite and strictly increasing in frame`. L'ease di una chiave
modella il segmento che inizia in quella chiave.

Extrapolate (fuori dalla prima e dall'ultima chiave): `hold` (predefinito), `linear`, `loop`, `ping_pong`,
`loop_offset`.

### 8.4 Easing

`ease` è un oggetto con un tag `type`. Il valore predefinito è `{"type": "linear"}`.

| `type` | Parametri | Curva |
| --- | --- | --- |
| `linear` | | `u` |
| `step` | | Mantiene il valore della chiave fino alla chiave successiva. |
| `ease_in` | | `u^2` |
| `ease_out` | | `1 - (1-u)^2` |
| `ease_in_out` | | Smoothstep. |
| `cubic_bezier` | `x1, y1, x2, y2` | Curva in stile CSS attraverso (0,0) e (1,1). |
| `smooth` | | Hermite di Catmull-Rom non uniforme attraverso le chiavi vicine. |
| `spring` | `bounces`, `decay` | `1 - e^(-decay*u) * cos(2*pi*bounces*u) * window`, window = `1 - smooth01((u-0.85)/0.15)`; atterra esattamente sul bersaglio. |
| `back` | `overshoot` | Cubica con pendenza iniziale `3 + overshoot` (limitata a 0 o più). Overshoot 1 ha un picco di 1.037 (u = 2/3); overshoot 2 ha un picco di 1.1317 (u = 0.5556). |
| `bounce` | | Restituzione 0.6, tre rimbalzi, durata totale 3.352 unità di tempo. Primo impatto a u = 0.2983; i picchi dei rimbalzi sono 0.64, 0.8704, 0.953344; termina a 1. |

`back` e `bounce` sono derivazioni originali (vedi `CLEANROOM.md`).

## 9. Espressioni

Un canale con `"source": "expr"` valuta una formula a ogni fotogramma. Il sorgente è limitato a 4096 byte
e a una profondità di annidamento di 64; gli errori vengono riportati durante la validazione con una posizione, per esempio
`channel 2 ('upper'): expression error: unexpected end of expression`.

Variabili: `frame`, `time` (= frame / fps), `fps`, `frames`, `pi`, `t` (= frame / frames).

Operatori (dalla precedenza più bassa alla più alta): `?:`, `||`, `&&`, confronti, `+ -`, `* / %`, unari `- !`,
`^` (associativo a destra). La divisione per zero dà 0; `%` è euclideo (`rem_euclid`).

Funzioni: `sin cos tan asin acos atan atan2 abs sqrt pow min max clamp lerp smoothstep floor ceil
round fract sign mod exp ln noise` (1 o 2 argomenti) `fbm` (2 o 3) `wiggle` (2 o 3) `saw tri
square deg rad`, e `get("target", "prop")`. Il risultato non finito di una funzione diventa 0.

`get("target", "prop")` legge il valore corrente di qualsiasi proprietà, compresi i risultati dei canali
precedenti. Entrambe le stringhe devono risolversi durante la validazione (`channel 2: get("q", "bone_angle"): unknown
target`). Un canale che legge un valore deve venire dopo il canale che lo scrive.

Esempio dal progetto della palla:

```json
{"target": "shadow", "prop": "layer_opacity", "source": "expr",
 "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"}
```

Le espressioni dei binding (sezione 11) usano una forma ristretta: la variabile è `value`, e le variabili di tempo
e `get()` sono vietate.

## 10. Camera, shot, audio, metadati

### 10.1 `camera`

| Nome JSON | Tipo | Predefinito | Intervallo | Significato |
| --- | --- | --- | --- | --- |
| `position` | `Vec2` o null | null (centro della scena) | | Punto del mondo al centro dello schermo. |
| `zoom` | numero | 1 | limitato ad almeno 0.01 | |
| `rotation` | numero | 0 | gradi | |
| `focus` | numero | 1 | | La profondità `parallax` del livello che è a fuoco (usata da `dof` e `dof-export`). |
| `blur` | numero | 12 | finito, `>= 0` | Raggio di sfocatura in pixel per unità di distanza di parallasse. |

Parallasse: un livello con `parallax = p` vede `zoom = 1 + (zoom - 1) * p`,
`pos = centre + (pos - centre) * p`, `rotation = -rotation * p`. Un `focus` o un `blur` non finito
produce `camera focus must be finite and camera blur a finite number >= 0`.

### 10.2 Array di metadati (non validati al caricamento)

Contengono dati dell'editor e degli strumenti. Il motore li accetta come semplici dati; le operazioni
che li usano li controllano quando vengono eseguite.

| Array | Campi dell'elemento |
| --- | --- |
| `markers` | `name`, `frame`, `end` (facoltativo; una regione se impostato) |
| `comps` | `name`, `layers`: mappa da percorso del livello a `{visible, opacity, selected?}` |
| `events` | `name`, `frame`, `data` (qualsiasi JSON) |
| `bone_vis` | `bone`, `from`, `to` (intervallo di fotogrammi in cui l'osso è mostrato) |
| `channel_meta` | `target`, `prop`, `color`, `shy`, `solo`, `lock` (flag dell'editor; `lock` blocca set-key) |
| `shots` | `name`, `x`, `y`, `zoom`, `rotation` (inquadrature della camera salvate) |

```json
"markers": [{"name": "intro", "frame": 0, "end": 12}, {"name": "hit", "frame": 20}],
"events": [{"name": "step", "frame": 12, "data": {"sound": "step.wav"}}],
"shots": [{"name": "wide", "x": 200, "y": 150, "zoom": 1, "rotation": 0}]
```

### 10.3 `sounds[]`

| Nome JSON | Tipo | Intervallo | Significato |
| --- | --- | --- | --- |
| `name` | stringa | univoco | |
| `file` | stringa | un percorso di asset (sezione 18) | WAV o altro formato audio supportato. |
| `start` | numero | secondi | Posizionamento sulla timeline. |
| `gain` | numero | 0..4 | |
| `trim_in`, `trim_out` | numero | `trim <= 86400` | Secondi tagliati dall'inizio e dalla fine del file. |
| `fade_in`, `fade_out` | numero | rientra nella clip ritagliata | Secondi. |

`check` (non `Engine::new`) segnala un file mancante: `sound 's': cannot read sound file ./nofile.wav: ...`.

## 11. Clip, skin, collegamenti, interazione, lod

### 11.1 `clips[]` e `clip_baked`

Una clip colloca sulla timeline un `take` riutilizzabile. Il baking delle clip aggiunge canali
ordinari (dopo quelli creati a mano) e salva un'istantanea in `clip_baked`, così un nuovo baking è
idempotente e il lavoro manuale non viene mai cancellato.

| Campo della clip | Significato |
| --- | --- |
| `name`, `start`, `speed`, `loops` | Posizionamento. |
| `take` | `{name, length, tracks: [{target, prop, keys}]}` |
| `mode` | `replace`, `additive`, `multiply` |
| `weight`, `weight_keys` | Influenza; traccia animata facoltativa. |
| `include`, `exclude` | Elenchi di pattern che selezionano le tracce. |
| `track_weights` | `[{pattern, prop, weight}]` |
| `fade_in`, `fade_out` | Fotogrammi. |
| `rename` | `[[from, to]]` ridenominazioni dei target. |

### 11.2 `skins[]`

`{name, group, layers, constraints, iks, bones}`. Le skin dello stesso `group` sono varianti
esclusive. La scala degli ossi per skin viene ridotta a un canale a gradini `bone_scale` di tipo
multiply al momento del baking. Le regole delle skin vengono controllate solo quando si eseguono le
operazioni sulle skin.

### 11.3 `mesh_locks[]` e `glues[]`

Una coppia di glue usa `VertexRef` = `{layer, path (default 0), index}`; un mesh lock indica interi percorsi con `{layer, path}` e un elenco facoltativo di indici dei vertici follower (vuoto significa tutti i vertici).

```json
"mesh_locks": [{"name": "lock", "host": {"layer": "hostm", "path": 0},
                "follower": {"layer": "follm", "path": 0}, "vertices": [], "strength": 1}],
"glues": [{"name": "g", "mode": "rigid", "strength": 1,
           "pairs": [{"a": {"layer": "hostm", "path": 0, "index": 0},
                      "b": {"layer": "s1", "path": 0, "index": 0}}]}]
```

Il `mode` di un glue è `snap`, `rigid` o `offset`. Entrambi sono validati da `links::resolve` e
applicati nello spazio della scena in ordine topologico (passo 12 della valutazione).

### 11.4 `machines[]` e `bindings[]` (solo runtime)

Le macchine a stati e i binding dei dati sono guidati dal Player durante l'esecuzione. Per
l'esportazione e il baking diventano canali `replace` costanti aggiunti al progetto. Ogni macchina
richiede almeno un livello.

```json
"machines": [{"name": "m",
  "inputs": [{"name": "go", "kind": "bool", "default": 0},
             {"name": "fire", "kind": "trigger", "hold": false}],
  "layers": [{"name": "main", "entry": "idle", "mask": [],
    "states": [
      {"name": "idle", "source": {"kind": "region", "marker": "intro"},
       "mode": "loop", "speed": 1, "blend_in": 0.1},
      {"name": "wave", "source": {"kind": "clip", "clip": "wave"},
       "mode": "once", "speed": 1, "blend_in": 0.1}],
    "transitions": [
      {"from": "idle", "to": "wave", "when": [{"input": "go", "op": "eq", "value": 1}],
       "duration": 0.2, "ease": {"type": "ease_in_out"}},
      {"from": "wave", "to": "idle", "exit_time": 1}]}]}]
```

Il `kind` di un input è `bool`, `number` o `trigger`. Il `source.kind` di uno stato è `clip`, `segment`, `region`
o `blend`; `mode` è `loop`, `once` o `ping_pong`. `Transition` e `Condition` rifiutano le chiavi
sconosciute.

```json
"bindings": [
  {"name": "hp", "path": "player.health", "expr": "value/100",
   "remap": {"in_min": 0, "in_max": 1, "out_min": 0, "out_max": 1, "clamp": true,
             "curve": {"type": "linear"}},
   "smooth": 0.1,
   "target": {"kind": "prop", "target": "sw", "prop": "layer_opacity"},
   "mode": "replace", "weight": 1},
  {"name": "go", "path": "input.go",
   "target": {"kind": "input", "machine": "m", "input": "go"}}]
```

Il `weight` di un binding deve essere compreso tra 0..1 (`binding 'b': weight must be within 0..1`). I target
delle macchine e dei binding vengono controllati con una compilazione di prova.

### 11.5 `containers[]` e `scrolls[]`

Layout monodimensionale in stile flexbox. Campi del container: `id`, `members`, `axis` (`row`, `column`), `gap`,
`cross_gap`, `padding` (4 numeri), `justify`, `align`, `wrap`, `rect`. Un `rect` ha un `kind`:
`fixed` (`x,y,w,h`), `pin`, `aspect` o `hug`. Un membro ha esattamente uno tra `layer`, `container` o
`spacer`, più `grow`, `shrink`, `width`, `height`, `align`, `resize`. Limiti: 100000 membri,
profondità di annidamento 64. Uno scroll ha `id`, `viewport`, `content`, `axis`, `offset`, `elastic`, `snap_x`,
`snap_y`, `fling`, `clip_layer`, `input`.

```json
"containers": [{"id": "row", "axis": "row", "gap": 4, "padding": [2,2,2,2],
  "justify": "start", "align": "start", "wrap": "none",
  "rect": {"kind": "fixed", "x": 0, "y": 0, "w": 200, "h": 60},
  "members": [{"layer": "c1"}, {"spacer": true, "grow": 1},
              {"layer": "c2", "grow": 0, "shrink": 1, "width": 30, "height": 30,
               "align": "center", "resize": true}]}],
"scrolls": [{"id": "sc", "viewport": {"kind": "fixed", "x": 0, "y": 100, "w": 100, "h": 80},
  "content": "list", "axis": "y", "offset": [0, 10],
  "elastic": {"limit": 40, "stiffness": 180, "damping_ratio": 0.9},
  "snap_x": [], "snap_y": [0, 40], "fling": 0.15, "clip_layer": "mask_rect", "input": "scroll.y"}]
```

Gli offset dei container vengono sommati alla posizione del livello e moltiplicati nella scala (passo 7 della valutazione).
`apply_clip_masks` aggiunge riferimenti di maschera al progetto che riceve, quindi modifica la copia del motore.

### 11.6 `lods[]`

Un registro di un passaggio di level-of-detail scritto da `osatura lod`: `level`, `tolerance_px`, `scale`,
`declared_px`, `bound_stage`, `path_px`, `mesh_px`, `keys_px`, `cull_px`, `stats`
(`path_anchors`, `mesh_vertices`, `keys`, `layers`, `bytes`) e `notes`. È la documentazione di ciò che ha
fatto un passaggio; il renderer non lo legge mai.


## 12. Regole di validazione, limiti e messaggi di errore

`Engine::new` valida in un solo passaggio e restituisce tutti i problemi. La tabella elenca ciò che viene imposto;
i messaggi tra virgolette sono stati prodotti da `osatura check` su file deliberatamente danneggiati.

### 12.1 Limiti globali

| Regola | Limite | Messaggio |
| --- | --- | --- |
| fps | `(0, 1000]` | `fps must be within (0, 1000]` |
| frames | `1..=200000` | `frames must be within 1..=200000` |
| width, height | `> 0` | `width and height must be > 0` |
| width, height | `<= 32768` | `width and height must be at most 32768` |
| qualsiasi float | `abs <= 1e7` | `value 100000000.0 at /bones/0/length is outside the supported range of +-1e7 (coordinates, scales, offsets and key values must stay below it)` |
| bones | al massimo 4096 | `at most 4096 bones` |
| format | `<= 1` | `project format 2 is newer than this build supports (1); update OSATURA` |
| dimensione JSON | CLI `--max-input-bytes`, predefinito 256 MiB; API `MAX_JSON_BYTES` 256 MiB | |

Il limite di 1e7 viene applicato due volte. Al caricamento, un letterale float oltre il limite fa fallire il parsing e il messaggio
viene incapsulato come `invalid JSON in <file>: value ...`. Un letterale senza punto decimale (un intero)
supera il parsing e viene intercettato dal nuovo controllo in `Engine::new`, dove compare come una normale
voce di elenco. Il limite esiste perché una moltiplicazione successiva non possa far andare in overflow un `f32` fino all'infinito.

### 12.2 Rig

| Regola | Messaggio |
| --- | --- |
| il genitore di un osso deve esistere prima | `bone 'lower' has parent 'nope' which is missing or listed after it` |
| lunghezza dell'osso finita, `>= 0` | `bone 'lower' has invalid length` |
| `angle_limits` `min <= max`; `scale_limits` `0 < min <= max` | |
| l'ik richiede 2 ossi | `ik 'reach': needs at least two bones` |
| catena ik collegata dal genitore | `ik 'reach': each bone must be a direct child of the previous one` |
| il target dell'ik non è nella catena | |
| soft, stretch | `ik 'reach': soft must be within 0..0.9 and stretch within 1..4` |
| un vincolo non può avere se stesso come target | |
| spring | `spring on 'upper': frequency must be in (0, 30] Hz and damping >= 0`; una per osso |

### 12.3 Livelli, percorsi, geometria

| Regola | Messaggio |
| --- | --- |
| nome del livello non vuoto, senza `/`, senza `::` | `layer name 'a/b' must be non-empty and contain neither '/' nor '::'` |
| percorso duplicato | `duplicate layer path 'arm'` (e righe per i target animabili duplicati) |
| i nomi dei percorsi e dei deformatori non contengono `::` e sono univoci all'interno del livello | |
| il numero di ancore del morph è uguale a quello del percorso base | |
| i pesi di skin indicano ossi noti; peso finito, `>= 0` | |
| mesh libera | almeno 3 vertici, almeno 1 triangolo, al massimo 200000 / 400000, finiti, indici validi, `uvs` vuoto o dello stesso numero |
| mesh a griglia | `cols`, `rows >= 1` |
| maschera | nessuna maschera su se stesso; `mask cycle involving layer 'arm'` |
| sorgente immagine | `layer 'im': image has neither 'png_base64' nor 'file'` |
| liquid | almeno 1 blob; `threshold > 0`; `cell >= 0.25`; blob `radius > 0` |
| boolean | almeno 2 percorsi |
| emitter | `rate` in `(0, 2000]`; `lifetime > 0`; jitter 0..1 |
| deformatori | solo su livelli vettoriali e di immagine; per ogni tipo vedi la sezione 7.1 |
| effetti | al massimo 16; blur e softness 0..64; offset al massimo 1024; quantità 0..1 |
| normal map | `strength` 0..8; richiede `data` o `path` |
| camera | `camera focus must be finite and camera blur a finite number >= 0` |

### 12.4 Canali

| Regola | Messaggio |
| --- | --- |
| il target viene risolto | `channel 0: unknown target 'zzz' for BoneOffsetX` |
| `layer_switch` su un livello switch | `channel 2: layer_switch needs a switch layer, 'arm' is not one` |
| peso finito | |
| chiavi finite e strettamente crescenti | `channel 0 ('target'): keys must be finite and strictly increasing in frame` |
| l'espressione viene analizzata | `channel 2 ('upper'): expression error: unexpected end of expression` |
| i target e le proprietà di `get()` vengono risolti | `channel 2: get("q", "bone_angle"): unknown target` |
| l'input pilotato viene risolto | |

I nomi sconosciuti di `prop` o `ease` producono errori serde che elencano tutte le varianti valide, per esempio
``unknown variant `foo`, expected one of ...``. I tipi interi errati producono
``invalid type: floating point `48.0`, expected u32``. Un campo obbligatorio mancante produce
``missing field `frames` ``.

### 12.5 Altre sezioni validate

- Sounds: `gain` 0..4, `trim` al massimo 86400 s, le dissolvenze rientrano nella clip ritagliata, nomi univoci.
  L'esistenza del file è verificata da `check`, non da `Engine::new`.
- Machines: almeno un livello (`machine 'm': needs at least one layer`); le transizioni e
  le condizioni rifiutano le chiavi sconosciute. Bindings: `binding 'b': weight must be within 0..1`. I loro target
  vengono controllati compilandoli una volta.
- Containers e scrolls: membri univoci, al massimo 100000, profondità di annidamento 64, regole dei rect.
- Mesh lock e glue: riferimenti risolti da `links::resolve`.
- File di immagini e audio: sezione 18.

### 12.6 Dati dell'editor: avvisi, ed errori solo per i numeri non finiti

`Engine::new` e `check` esaminano `skins`, `comps`, `markers`, `events`, `shots`, `channel_meta`, `bone_vis`, `clips` e
`lods` (modulo `editordata`). Per far sì che ogni progetto salvato in precedenza continui a caricarsi, **sono errori solo i valori che non possono esistere in un file
salvato**: NaN o infinito in markers, events, shots, `bone_vis`, opacità delle comp, lods e opacità dei livelli. Tutto il resto è
un **avviso** stampato da `check` (e segnalato da `lint` come L013): zoom della camera `<= 0` (il motore lo limita a 0.01), `opacity`
del livello fuori dall'intervallo 0..1 (limitata), `ik_mix` e `constraint_weight` fuori da 0..1 (limitati), valori di `layer_opacity` e
`camera_zoom` con chiavi fuori dal loro intervallo, riferimenti sconosciuti a livelli/ossi/canali, nomi duplicati, tempi fuori
dall'animazione, skin non valide (`skins::validate`), clip (`clipstack::validate`, e target delle tracce dei take che non vengono risolti,
tenendo conto di `rename`), `channel_meta` e `lods`. Le fixture in `out/` si caricano senza avvisi.

`osatura check` stampa gli avvisi di lint di questo tipo; il linter completo (`osatura lint`) ha più regole (da L001 in poi).

## 13. Esempio 1: palla che rimbalza

Verificato con `osatura check` (`ok: 'bouncing ball' 320×240 @ 24 fps, 48 frames, 0 bones, 0 ik,
3 layers, 5 channels`) e `osatura render ball.json 7 out.png`. Cosa mostra:

- `layer_y` con chiavi e easing `bounce` sul primo segmento: la palla cade da y=40 al
  pavimento a y=170 e rimbalza tre volte. Il primo impatto avviene intorno al fotogramma 7.2 (24 / 3.352).
  Il segmento da 24 a 47 risale con `ease_out`.
- Un'ombra che segue la palla: `layer_x` e `layer_opacity` sono espressioni che leggono
  la palla con `get()`. Sono elencate dopo i canali della palla, perché i canali vengono eseguiti in ordine.
- Schiacciamento al contatto: un'espressione restituisce 0.8 per `layer_scale_y` quando la palla è sotto y=160.
- Cerchio da quattro ancore Bezier con maniglie lunghe 16.57 (0.5523 volte il raggio 30).

```json
{
  "format": 1,
  "name": "bouncing ball",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "floor",
      "type": "vector",
      "paths": [
        {
          "name": "line",
          "closed": false,
          "anchors": [{"pos": [0, 200]}, {"pos": [320, 200]}],
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "shadow",
      "type": "vector",
      "paths": [
        {
          "name": "blob",
          "closed": true,
          "anchors": [
            {"pos": [-26, 0], "handle_out": [-26, -5], "handle_in": [-26, 5]},
            {"pos": [0, -6], "handle_in": [-14, -6], "handle_out": [14, -6]},
            {"pos": [26, 0], "handle_in": [26, -5], "handle_out": [26, 5]},
            {"pos": [0, 6], "handle_in": [14, 6], "handle_out": [-14, 6]}
          ],
          "fill": "#00000055"
        }
      ],
      "transform": {"position": [160, 200]}
    },
    {
      "name": "ball",
      "type": "vector",
      "paths": [
        {
          "name": "disc",
          "closed": true,
          "anchors": [
            {"pos": [30, 0], "handle_in": [30, -16.57], "handle_out": [30, 16.57]},
            {"pos": [0, 30], "handle_in": [16.57, 30], "handle_out": [-16.57, 30]},
            {
              "pos": [-30, 0],
              "handle_in": [-30, 16.57],
              "handle_out": [-30, -16.57]
            },
            {
              "pos": [0, -30],
              "handle_in": [-16.57, -30],
              "handle_out": [16.57, -30]
            }
          ],
          "fill": {
            "center": [-8, -10],
            "radius": 44,
            "stops": [
              {"offset": 0, "color": "#ffd9a0"},
              {"offset": 1, "color": "#d6336c"}
            ]
          },
          "stroke": {"paint": "#3d2c2e", "width": 2}
        }
      ],
      "transform": {"position": [160, 40]}
    }
  ],
  "channels": [
    {
      "target": "ball",
      "prop": "layer_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 40, "ease": {"type": "bounce"}},
          {"frame": 24, "value": 170, "ease": {"type": "ease_out"}},
          {"frame": 47, "value": 40}
        ]
      }
    },
    {
      "target": "ball",
      "prop": "layer_x",
      "source": "keys",
      "track": {"keys": [{"frame": 0, "value": 60}, {"frame": 47, "value": 260}]}
    },
    {
      "target": "shadow",
      "prop": "layer_x",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_x\")"
    },
    {
      "target": "shadow",
      "prop": "layer_opacity",
      "source": "expr",
      "expr": "clamp((get(\"ball\", \"layer_y\") - 40) / 130, 0.15, 1)"
    },
    {
      "target": "ball",
      "prop": "layer_scale_y",
      "source": "expr",
      "expr": "get(\"ball\", \"layer_y\") > 160 ? 0.8 : 1",
      "mode": "replace"
    }
  ]
}
```

## 14. Esempio 2: braccio con rig e IK

Verificato con `osatura check` (3 bones, 1 ik, 3 layers, 2 channels). `osatura explain arm.json
24` riporta la punta in (170, 190), uguale all'osso `target`, e `osatura render arm.json 24
out.png --bones` disegna lo scheletro. Cosa mostra:

- `upper` è una radice con un limite d'angolo; `lower` è il suo figlio (`rest_offset` `[90, 0]` è uguale alla
  lunghezza del genitore); `target` è un osso radice libero che la catena IK cerca di raggiungere.
- `reach` è una catena IK a due ossi, quindi viene usato il solver analitico.
- `sleeve` ha la skin: ogni ancora elenca i pesi degli ossi. `hand` e `goal` sono collegati con
  `bone`.
- Spostare l'osso `target` con i canali `bone_offset_x` e `bone_offset_y` fa allungare il braccio.

```json
{
  "format": 1,
  "name": "rigged arm",
  "width": 320,
  "height": 240,
  "fps": 24,
  "frames": 48,
  "background": "#f4efe6",
  "bones": [
    {
      "name": "upper",
      "parent": null,
      "length": 90,
      "rest_offset": [60, 120],
      "rest_angle": 0,
      "angle_limits": [-120, 120]
    },
    {
      "name": "lower",
      "parent": "upper",
      "length": 80,
      "rest_offset": [90, 0],
      "rest_angle": 0
    },
    {
      "name": "target",
      "parent": null,
      "length": 10,
      "rest_offset": [230, 120],
      "rest_angle": 0
    }
  ],
  "ik": [
    {
      "name": "reach",
      "bones": ["upper", "lower"],
      "target": "target",
      "bend": 1,
      "mix": 1
    }
  ],
  "layers": [
    {
      "name": "arm",
      "type": "vector",
      "paths": [
        {
          "name": "sleeve",
          "closed": true,
          "anchors": [
            {"pos": [60, 106], "weights": [{"bone": "upper", "weight": 1}]},
            {
              "pos": [150, 104],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [228, 110], "weights": [{"bone": "lower", "weight": 1}]},
            {"pos": [228, 130], "weights": [{"bone": "lower", "weight": 1}]},
            {
              "pos": [150, 136],
              "weights": [
                {"bone": "upper", "weight": 0.5},
                {"bone": "lower", "weight": 0.5}
              ]
            },
            {"pos": [60, 134], "weights": [{"bone": "upper", "weight": 1}]}
          ],
          "fill": "#ffb347",
          "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
        }
      ]
    },
    {
      "name": "hand",
      "type": "vector",
      "bone": "lower",
      "paths": [
        {
          "name": "palm",
          "closed": true,
          "anchors": [
            {"pos": [246, 120], "handle_in": [246, 111], "handle_out": [246, 129]},
            {"pos": [232, 134], "handle_in": [241, 134], "handle_out": [223, 134]},
            {"pos": [218, 120], "handle_in": [218, 129], "handle_out": [218, 111]},
            {"pos": [232, 106], "handle_in": [223, 106], "handle_out": [241, 106]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "goal",
      "type": "vector",
      "bone": "target",
      "paths": [
        {
          "name": "ring",
          "closed": false,
          "anchors": [{"pos": [222, 120]}, {"pos": [238, 120]}],
          "stroke": {"paint": "#d6336c", "width": 3, "dash": [4, 4]}
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "target",
      "prop": "bone_offset_x",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": -60, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    },
    {
      "target": "target",
      "prop": "bone_offset_y",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "ease_in_out"}},
          {"frame": 24, "value": 70, "ease": {"type": "ease_in_out"}},
          {"frame": 47, "value": 0}
        ]
      }
    }
  ]
}
```

## 15. Esempio 3: bocca con livello switch

Verificato con `osatura check` e renderizzando i fotogrammi 0, 5 e 9, che mostrano tre bocche
diverse. Le chiavi fuori intervallo vengono limitate: un valore di chiave 7 mostra il figlio 2 e -3 mostra il figlio 0.

```json
{
  "format": 1,
  "name": "switch mouth",
  "width": 200,
  "height": 160,
  "fps": 12,
  "frames": 25,
  "background": "#f4efe6",
  "layers": [
    {
      "name": "face",
      "type": "vector",
      "paths": [
        {
          "name": "head",
          "closed": true,
          "anchors": [
            {"pos": [180, 80], "handle_in": [180, 35], "handle_out": [180, 125]},
            {"pos": [100, 150], "handle_in": [145, 150], "handle_out": [55, 150]},
            {"pos": [20, 80], "handle_in": [20, 125], "handle_out": [20, 35]},
            {"pos": [100, 10], "handle_in": [55, 10], "handle_out": [145, 10]}
          ],
          "fill": "#ffd9a0",
          "stroke": {"paint": "#3d2c2e", "width": 3}
        }
      ]
    },
    {
      "name": "mouth",
      "type": "switch",
      "selected": 0,
      "layers": [
        {
          "name": "closed",
          "type": "vector",
          "paths": [
            {
              "name": "line",
              "closed": false,
              "anchors": [{"pos": [75, 110]}, {"pos": [125, 110]}],
              "stroke": {"paint": "#3d2c2e", "width": 4, "cap": "round"}
            }
          ]
        },
        {
          "name": "open",
          "type": "vector",
          "paths": [
            {
              "name": "oval",
              "closed": true,
              "anchors": [
                {"pos": [75, 106]},
                {"pos": [125, 106]},
                {
                  "pos": [112, 132],
                  "handle_in": [122, 132],
                  "handle_out": [102, 134]
                },
                {"pos": [88, 132], "handle_in": [98, 134], "handle_out": [78, 132]}
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3, "join": "round"}
            }
          ]
        },
        {
          "name": "round",
          "type": "vector",
          "paths": [
            {
              "name": "o",
              "closed": true,
              "anchors": [
                {
                  "pos": [112, 118],
                  "handle_in": [112, 110],
                  "handle_out": [112, 126]
                },
                {
                  "pos": [100, 130],
                  "handle_in": [106, 130],
                  "handle_out": [94, 130]
                },
                {"pos": [88, 118], "handle_in": [88, 126], "handle_out": [88, 110]},
                {
                  "pos": [100, 106],
                  "handle_in": [94, 106],
                  "handle_out": [106, 106]
                }
              ],
              "fill": "#7a1f3d",
              "stroke": {"paint": "#3d2c2e", "width": 3}
            }
          ]
        }
      ]
    }
  ],
  "channels": [
    {
      "target": "mouth",
      "prop": "layer_switch",
      "source": "keys",
      "track": {
        "keys": [
          {"frame": 0, "value": 0, "ease": {"type": "step"}},
          {"frame": 4, "value": 1, "ease": {"type": "step"}},
          {"frame": 8, "value": 2, "ease": {"type": "step"}},
          {"frame": 12, "value": 1, "ease": {"type": "step"}},
          {"frame": 16, "value": 0, "ease": {"type": "step"}}
        ]
      }
    }
  ]
}
```

## 16. Ordine di valutazione

`Engine::evaluate(frame)` è una funzione pura del numero di fotogramma e del progetto. Le molle (spring) sono
simulate una sola volta quando il motore viene costruito; nient'altro mantiene uno stato tra un fotogramma e l'altro. I passi:

1. **Stato iniziale** (`initial_state`): trasformazioni statiche dei livelli, opacità limitata a 0..1, `selected`
   degli switch, ampiezza del wobble, camera, mix degli `ik`, pesi dei vincoli e offset iniziali degli
   slot dei parametri (offset degli ossi, offset dei deformatori, pesi dei morph e così via).
2. **Canali** (`apply_channels`) nell'ordine dell'elenco, ciascuno combinato con il valore corrente tramite il proprio
   `mode` e `weight`. I risultati non finiti vengono saltati. Le scritture vengono limitate come elencato nella sezione 8.2.
3. **Limitazione ai limiti** (`clamp_limits`): `angle_limits` e `scale_limits` per osso.
4. **Cinematica diretta (FK)**: matrici mondo di tutti gli ossi, prima i genitori.
5. **Catene IK** nell'ordine dell'elenco, saltando le catene con `mix <= 0`. Dopo ogni catena la posa viene limitata
   e la FK ricalcolata, così la catena successiva vede il risultato.
6. **Vincoli** nell'ordine dell'elenco, saltando `weight <= 0`: aim, copy_rotation, pin.
7. **Container di layout**: gli offset vengono sommati a x e y del livello e moltiplicati nella scala.
8. **Molle baked**: gli angoli baked sostituiscono l'angolo dell'osso, poi la FK viene ricalcolata.
9. **Matrici di skin**: `skin_mat = world * inverse(rest world)` per osso.
10. **Camera**: una matrice di camera per ogni livello radice, usando il `parallax` del livello radice (sezione 10.1).
11. **Nodi** in pre-ordine. La matrice di scena di un nodo è `parentStage * boneAttach(skin_mat) * local`.
    La geometria vettoriale viene costruita in questo ordine:
    1. morph (blend shape additive);
    2. offset `point_dx/dy` per ancora;
    3. spostamenti delle maniglie dei gradienti (`grad_dx/dy`);
    4. deformatori nell'ordine dell'elenco, poi `wobble`;
    5. skinning con pesi normalizzati (pesi vuoti significano statico; anche le maniglie dei gradienti
       hanno la skin).
    La geometria con skin vive nelle coordinate di riposo della scena, e la trasformazione del livello viene applicata dopo.
    Uno switch mostra un figlio (arrotondato, limitato). La geometria di liquid, boolean ed emitter viene
    sintetizzata per ogni fotogramma. Un'immagine riceve una mesh a griglia o libera; con deformatori ma senza mesh riceve una
    griglia 16 per 16.
12. **Mesh lock e glue** nello spazio della scena, in ordine topologico.
13. **Riparazione dei valori non finiti** (`repair_nonfinite`): un nodo la cui geometria contiene NaN o infinito
    viene nascosto e `Frame.nonfinite` viene impostato. `evaluate_checked` e `check_all_frames_finite` lo trasformano
    in un errore.

Conseguenze su cui puoi fare affidamento:

- Un canale vede solo i canali che lo precedono nell'elenco (tramite `get` e `driven`).
- L'IK sovrascrive la FK per gli ossi della catena, in proporzione a `mix` (la miscela è
  `old + wrap_pi(new - old) * mix`). Con `mix` 0 la catena è pura FK.
- I vincoli vengono eseguiti dopo l'IK, quindi un vincolo aim su un osso IK prevale.
- Le molle agiscono per ultime sull'angolo dell'osso, dopo l'IK e i vincoli.

## 17. Ordine di rendering

- I livelli radice vengono disegnati nell'ordine dell'elenco, i figli nell'ordine dell'elenco all'interno del loro genitore.
- Un nodo non `visible` viene saltato (può comunque essere una sorgente di maschera).
- Un nodo viene renderizzato su un livello offscreen quando ha `opacity < 1`, effetti, una modalità di fusione diversa
  da `normal`, o una maschera. Passi: contenuto; effetti del livello nell'ordine dell'elenco (su un canvas con padding); maschera
  (il livello maschera viene disegnato con la propria trasformazione; il suo alpha, inclusa la sua opacità, moltiplica;
  `invert` lo inverte); composizione con opacità e modalità di fusione.
- La profondità di campo è applicata solo da `dof` e `dof-export`: raggio = `abs(parallax - focus) *
  blur`, al massimo 24 pixel per impostazione predefinita.
- La sovrapposizione degli ossi (`--bones`) viene disegnata per ultima.

## 18. Asset e percorsi

I progetti possono fare riferimento a file: `image.source.file`, `normal_map.path`, `sounds[].file`.

- I percorsi devono essere relativi, non devono contenere `..` e, dopo aver seguito i symlink, devono restare all'interno
  della directory del progetto: `layer 'im': asset '/etc/passwd': absolute paths are not allowed (a project
  may only reference files inside its own directory; use --allow-external-assets to override)`, e
  `layer 'im': asset '../x.png': '..' is not allowed (...)`.
- `--allow-external-assets` toglie questa restrizione per un singolo comando. Vengono aperti solo file regolari.
- Limiti di dimensione: immagini 64 MB su disco, audio 256 MB. Immagini decodificate: ogni lato al massimo 16384 e al
  massimo 64 megapixel.
- Limiti di output: raster al massimo 16384 per lato e 100 megapixel; sequenze al massimo 2 GiB; budget
  di esportazione fotogrammi per ancore al massimo 40 milioni.
- Quando la CLI scrive un progetto, lo rivalida tramite `Engine::new` e riscrive i percorsi degli asset
  in modo relativo alla posizione di output.

Preferisci `png_base64` per un file autonomo.

## 19. Politica di versionamento

- `format` è un intero, attualmente `1` (`FORMAT_VERSION`).
- Un file con un `format` maggiore di quello supportato dalla build viene rifiutato con il messaggio della sezione 12.1;
  il motore non tira a indovinare.
- Aggiungere un campo facoltativo con un valore predefinito non è un cambio di versione. Le build precedenti lo ignorano (e
  `check` lo segnala con un avviso come campo sconosciuto).
- Rimuovere o reinterpretare un campo richiede un incremento di versione, e l'incremento richiede un passo di migrazione in
  `Project::from_json` che aggiorna i documenti più vecchi prima della validazione.
- L'API della libreria Rust ha una propria versione, `API_VERSION` 1.4.0, protetta dal test di snapshot
  `api_surface.txt`; l'ABI C ha `ABI_VERSION` 1. Vedi `docs/API.md`.

## 20. Lacune note tra codice e commenti

Sono state trovate durante la stesura di questo riferimento.

1. **Corretto.** La regola delle chiavi di switch (`lint::switch_key_warnings`, L012) ora dice ciò che fa il motore: una chiave inferiore a 0 mostra il figlio 0,
   una chiave pari o superiore al numero dei figli mostra l'ultimo figlio (uno switch vuoto non mostra nulla). Test: unit test di `lint`, verificati
   rispetto al motore.
2. Il commento in `project.rs` dice che i file senza `format` sono di versione 1, ma il default di serde è
   `FORMAT_VERSION`. Oggi concordano solo perché la costante vale 1. Quando la costante verrà aumentata, un
   file senza `format` verrebbe letto come la versione più recente.
3. **Corretto.** `unknownfields` non segnala più una struttura vuota scritta esplicitamente (`clips[].weight_keys: {"keys": []}`);
   `default_like` è ricorsivo, e un test fissa l'insieme delle strutture facoltative che il writer omette quando sono vuote.
4. **Corretto (come avvisi).** I campi dell'editor sono validati; vedi la sezione 12.6.
5. **Corretto nella wave 7 (L-71).** La geometria di un livello con skin è nello spazio di riposo della scena e la trasformazione del genitore viene applicata dopo
   di essa. `Layer.bone` ora si collega con `world = bone_skin * parent_stage * local`: la matrice di skin (mondo dell'osso x inverso del riposo) è
   un movimento nello spazio della scena, quindi viene applicata dopo l'intera catena dei genitori. A riposo la matrice di skin è l'identità, quindi nulla
   si muove; quando l'osso viene **posato**, un gruppo genitore con scala 2 non raddoppia più lo spostamento di un livello collegato. Un livello
   senza un osso proprio segue l'osso dell'antenato collegato a un osso più vicino; l'osso proprio di un livello sostituisce quello dell'antenato
   (i due movimenti non si sommano). Test: `bone_attach_under_a_transformed_group_composes_bone_after_the_group`,
   `bone_attach_is_inherited_and_a_child_bone_replaces_the_parents`. I progetti che si basavano sul vecchio spostamento raddoppiato (un osso
   posato sotto un gruppo scalato o ruotato) vengono renderizzati in modo diverso; i progetti senza questa combinazione restano invariati.
